// SPDX-License-Identifier: MIT
//! The cycle collector behind gc.collect(). Memory is reference counted, so
//! only objects in reference cycles need it. As in CPython it works by trial
//! deletion: the references the objects hold to one another are subtracted
//! from their reference counts, and what is left without a reference from
//! outside, and not reachable from anything that has one, is garbage. Its
//! finalizers run (PEP 442), then its containers are cleared, which breaks
//! the cycles and lets the reference counts free it.
//!
//! A collection starts from the objects that can be mutated to close a
//! cycle, and from the running frames, and walks everything they reach, so
//! the immutable objects in a cycle (tuples, iterators, views, bound
//! methods...) are found too. Instances, functions and native objects are
//! registered when they are made, in a thread-local list they leave when
//! they are freed (each knows its slot). Classes, generators and cells are
//! registered as weak pointers, in a list compacted as it grows. A list,
//! dict or set is registered the first time Python code stores into it
//! something that can refer back: until then it cannot be part of a cycle
//! of its own, and the many short-lived ones cost nothing.

use crate::builtins::{need, Kw};
use crate::modules::{module, put_fns};
use crate::value::*;
use crate::vm::{Frame, Generator, Vm};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::rc::{Rc, Weak};

/// where an instance, function or native object sits in the registry
pub struct Slot(Cell<u32>);

impl Default for Slot {
    fn default() -> Slot {
        Slot(Cell::new(u32::MAX))
    }
}

/// a registered object that leaves the registry when it is freed
#[derive(Clone, Copy)]
enum Owned {
    Inst(*const Instance),
    Func(*const Function),
    Obj(*const NativeObj),
}

impl Owned {
    /// safety: the object is alive, as it leaves the registry when freed
    fn slot(&self) -> &Slot {
        unsafe {
            match *self {
                Owned::Inst(p) => &(*p).gc,
                Owned::Func(p) => &(*p).gc,
                Owned::Obj(p) => &(*p).gc,
            }
        }
    }
    fn value(&self) -> Value {
        unsafe {
            match *self {
                Owned::Inst(p) => {
                    Rc::increment_strong_count(p);
                    Value::Instance(Rc::from_raw(p))
                }
                Owned::Func(p) => {
                    Rc::increment_strong_count(p);
                    Value::Func(Rc::from_raw(p))
                }
                Owned::Obj(p) => {
                    Rc::increment_strong_count(p);
                    Value::Obj(Rc::from_raw(p))
                }
            }
        }
    }
}

/// a registered object, held weakly
pub enum Tracked {
    List(Weak<RefCell<Vec<Value>>>),
    Dict(Weak<RefCell<Dict>>),
    Set(Weak<RefCell<Dict>>),
    Class(Weak<Class>),
    Cell(Weak<RefCell<Value>>),
    Gen(Weak<RefCell<Generator>>),
}

impl Tracked {
    fn alive(&self) -> bool {
        match self {
            Tracked::List(w) => w.strong_count() > 0,
            Tracked::Dict(w) | Tracked::Set(w) => w.strong_count() > 0,
            Tracked::Class(w) => w.strong_count() > 0,
            Tracked::Cell(w) => w.strong_count() > 0,
            Tracked::Gen(w) => w.strong_count() > 0,
        }
    }
    fn upgrade(&self) -> Option<Value> {
        match self {
            Tracked::List(w) => w.upgrade().map(Value::List),
            Tracked::Dict(w) => w.upgrade().map(Value::Dict),
            Tracked::Set(w) => w.upgrade().map(Value::Set),
            Tracked::Class(w) => w.upgrade().map(Value::Class),
            Tracked::Cell(w) => w.upgrade().map(Value::Cell),
            Tracked::Gen(w) => w.upgrade().map(Value::Gen),
        }
    }
}

/// The weakly held objects. A dead entry keeps its object's memory from
/// being freed, so the newest ones, which mostly die young, are looked at
/// often; those that survive that are looked at again as their number
/// doubles.
struct Registry {
    objs: Vec<Tracked>,
    /// objs[..old] survived a look
    old: usize,
    old_limit: usize,
}

const YOUNG: usize = 4;

impl Registry {
    fn compact_young(&mut self) {
        let mut w = self.old;
        for r in self.old..self.objs.len() {
            if self.objs[r].alive() {
                self.objs.swap(w, r);
                w += 1;
            }
        }
        self.objs.truncate(w);
        self.old = w;
        if self.old >= self.old_limit {
            self.compact();
        }
    }
    fn compact(&mut self) {
        self.objs.retain(Tracked::alive);
        self.old = self.objs.len();
        self.old_limit = (self.old * 2).max(1024);
    }
}

type FxMap<K, V> = HashMap<K, V, BuildHasherDefault<FxHasher>>;

thread_local! {
    static OWNED: RefCell<Vec<Owned>> = const { RefCell::new(Vec::new()) };
    static REGISTRY: RefCell<Registry> = const { RefCell::new(Registry { objs: Vec::new(), old: 0, old_limit: 1024 }) };
    /// the tuples a collection found holding nothing it tracks, which
    /// CPython's collector untracks; keyed by address, the weak pointer
    /// keeping the address from being reused
    static UNTRACKED: RefCell<FxMap<usize, Weak<Vec<Value>>>> = RefCell::new(FxMap::default());
    static COLLECTING: Cell<bool> = const { Cell::new(false) };
    /// automatic collection: gc.isenabled(), threshold0 (0 turns it off),
    /// the registry size that makes one due, and whether one is
    static AUTO: Cell<(bool, usize, usize, bool)> = const { Cell::new((true, 2000, 2000, false)) };
    /// collections run and objects found unreachable, for gc.get_stats()
    static STATS: Cell<(usize, usize)> = const { Cell::new((0, 0)) };
}

fn own(o: Owned) {
    let _ = OWNED.try_with(|r| {
        let mut r = r.borrow_mut();
        o.slot().0.set(r.len() as u32);
        r.push(o);
        if r.len() >= AUTO.with(|a| a.get().2) {
            schedule(r.len());
        }
    });
}

/// a collection runs at the next point where finalizers may: once the
/// registry has grown by threshold0 objects, or by a quarter if more,
/// since the last one, so that its cost stays in proportion
#[cold]
fn schedule(n: usize) {
    AUTO.with(|a| {
        let (on, t0, _, _) = a.get();
        if on && t0 > 0 {
            a.set((on, t0, usize::MAX, true));
            HAS_PENDING_DEL.with(|h| h.set(true));
        } else {
            a.set((on, t0, n + t0.max(1), false));
        }
    });
}

/// whether an automatic collection is due; the vm runs it when it runs
/// pending finalizers
#[inline]
pub fn due() -> bool {
    AUTO.with(|a| a.get().3)
}

fn rearm() {
    let n = OWNED.with(|r| r.borrow().len());
    AUTO.with(|a| {
        let (on, t0, _, _) = a.get();
        a.set((on, t0, n + t0.max(n / 4).max(1), false));
    });
}

#[inline]
pub fn track_inst(i: &Rc<Instance>) {
    own(Owned::Inst(Rc::as_ptr(i)));
}

#[inline]
pub fn track_func(f: &Rc<Function>) {
    own(Owned::Func(Rc::as_ptr(f)));
}

#[inline]
pub fn track_obj(o: &Rc<NativeObj>) {
    own(Owned::Obj(Rc::as_ptr(o)));
}

/// takes an object being freed out of the registry
#[inline]
pub fn untrack(slot: &Slot) {
    let i = slot.0.get();
    if i == u32::MAX {
        return;
    }
    let _ = OWNED.try_with(|r| {
        let mut r = r.borrow_mut();
        let i = i as usize;
        r.swap_remove(i);
        if let Some(moved) = r.get(i) {
            moved.slot().0.set(i as u32);
        }
    });
}

impl Drop for Function {
    fn drop(&mut self) {
        untrack(&self.gc);
    }
}

impl Drop for NativeObj {
    fn drop(&mut self) {
        untrack(&self.gc);
    }
}

pub fn track(t: Tracked) {
    let _ = REGISTRY.try_with(|r| {
        if let Ok(mut r) = r.try_borrow_mut() {
            // the object registered last has often died already: dropping
            // its entry now lets the allocator reuse its memory at once
            while r.objs.len() > r.old && !r.objs[r.objs.len() - 1].alive() {
                r.objs.pop();
            }
            r.objs.push(t);
            if r.objs.len() - r.old >= YOUNG {
                r.compact_young();
            }
        }
    });
}

/// a generator, registered
pub fn new_gen(g: Generator) -> Value {
    let g = Rc::new(RefCell::new(g));
    track(Tracked::Gen(Rc::downgrade(&g)));
    Value::Gen(g)
}

/// `v` is being stored into list `l`: the list is registered once it may
/// be part of a cycle (nothing else holds weak pointers to lists)
#[inline]
pub fn list_store(l: &Rc<RefCell<Vec<Value>>>, v: &Value) {
    if holds_refs(v) && Rc::weak_count(l) == 0 {
        track(Tracked::List(Rc::downgrade(l)));
    }
}

/// the same for a dict being given key `k` and value `v`
#[inline]
pub fn dict_store(d: &Rc<RefCell<Dict>>, k: &Value, v: &Value) {
    if (holds_refs(v) || holds_refs(k)) && Rc::weak_count(d) == 0 {
        track(Tracked::Dict(Rc::downgrade(d)));
    }
}

/// the same for a set given `k`; a set something holds a weakref to is
/// left out (those are rare)
#[inline]
pub fn set_store(s: &Rc<RefCell<Dict>>, k: &Value) {
    if holds_refs(k) && Rc::weak_count(s) == 0 {
        track(Tracked::Set(Rc::downgrade(s)));
    }
}

/// a tuple about to be refilled in place counts as new again: CPython
/// re-tracks the result tuples it reuses
pub fn retrack(t: &Rc<Vec<Value>>) {
    if Rc::weak_count(t) > 0 {
        UNTRACKED.with(|u| u.borrow_mut().remove(&(Rc::as_ptr(t) as *const u8 as usize)));
    }
}

/// whether the collector untracked tuple `t`
fn untracked(t: &Rc<Vec<Value>>) -> bool {
    Rc::weak_count(t) > 0 && UNTRACKED.with(|u| u.borrow().contains_key(&(Rc::as_ptr(t) as *const u8 as usize)))
}

impl Value {
    /// whether this can hold references to other objects
    #[inline]
    pub fn holds_refs(&self) -> bool {
        holds_refs(self)
    }
}

/// whether `v` can hold references to other objects
#[inline]
fn holds_refs(v: &Value) -> bool {
    !matches!(
        v,
        Value::Undef
            | Value::None
            | Value::NotImplemented
            | Value::Ellipsis
            | Value::Bool(_)
            | Value::Int(_)
            | Value::Big(_)
            | Value::Float(_)
            | Value::Complex(_)
            | Value::Str(_)
            | Value::Bytes(_)
            | Value::ByteArray(_)
            | Value::Range(_)
            | Value::BigRange(_)
    )
}

/// what the collector is handed each reference of an object
type Visit<'a> = dyn FnMut(&Value) + 'a;

fn push(out: &mut Visit, v: &Value) {
    if holds_refs(v) {
        out(v);
    }
}

fn push_dict(out: &mut Visit, d: &Dict) {
    for e in d.iter() {
        push(out, &e.key);
        push(out, &e.value);
    }
}

fn frame_referents(f: &Frame, out: &mut Visit) {
    out(&Value::Code(f.code.clone()));
    for v in &f.stack {
        push(out, v);
    }
    f.cells.iter().for_each(|c| out(&Value::Cell(c.clone())));
    out(&Value::Dict(f.globals.clone()));
    if let Some(ns) = &f.ns {
        out(&Value::Dict(ns.clone()));
    }
    // the namespaces in f.ext sit behind an Rc of their own that frames can
    // share, so they are left out: what they hold counts as reachable
}

/// hands `out` what `v` holds strong references to, one value per reference;
/// false when a borrow held elsewhere keeps them from being read
fn referents(v: &Value, out: &mut Visit) -> bool {
    match v {
        Value::List(l) => match l.try_borrow() {
            Ok(l) => l.iter().for_each(|x| push(out, x)),
            Err(_) => return false,
        },
        Value::Tuple(t) => t.iter().for_each(|x| push(out, x)),
        Value::Dict(d) | Value::Set(d) | Value::FrozenSet(d) => match d.try_borrow() {
            Ok(d) => push_dict(out, &d),
            Err(_) => return false,
        },
        Value::Slice(s) => [&s.0, &s.1, &s.2].into_iter().for_each(|x| push(out, x)),
        Value::Func(f) => {
            let (Ok(code), Ok(defaults), Ok(kwdefaults), Ok(dict), Ok(ann)) = (f.code.try_borrow(), f.defaults.try_borrow(), f.kwdefaults.try_borrow(), f.dict.try_borrow(), f.annotations.try_borrow()) else {
                return false;
            };
            out(&Value::Code(code.clone()));
            out(&Value::Dict(f.globals.clone()));
            defaults.iter().for_each(|x| push(out, x));
            kwdefaults.iter().for_each(|(_, x)| push(out, x));
            f.closure.iter().for_each(|c| out(&Value::Cell(c.clone())));
            if let Some(d) = &*dict {
                out(&Value::Dict(d.clone()));
            }
            push(out, &ann);
        }
        Value::Native(n) => push(out, &n.data),
        Value::Method(m) => {
            push(out, &m.0);
            push(out, &m.1);
        }
        Value::Class(c) => {
            let (Ok(bases), Ok(mro), Ok(meta)) = (c.bases.try_borrow(), c.mro.try_borrow(), c.meta.try_borrow()) else {
                return false;
            };
            bases.iter().chain(mro.iter()).for_each(|b| out(&Value::Class(b.clone())));
            out(&Value::Dict(c.dict.clone()));
            if let Some(m) = &*meta {
                out(&Value::Class(m.clone()));
            }
        }
        Value::Instance(i) => {
            let (Ok(class), Ok(base), Ok(extra)) = (i.class.try_borrow(), i.base.try_borrow(), i.extra.try_borrow()) else {
                return false;
            };
            out(&Value::Class(class.clone()));
            out(&Value::Dict(i.dict.clone()));
            push(out, &base);
            push_dict(out, &extra);
        }
        Value::Module(m) => out(&Value::Dict(m.dict.clone())),
        Value::Iter(it) => {
            let Ok(it) = it.try_borrow() else {
                return false;
            };
            match &*it {
                Iter::List(l, _) | Iter::ListRev(l, _) => out(&Value::List(l.clone())),
                Iter::Tuple(t, _) => out(&Value::Tuple(t.clone())),
                Iter::Dict(d, ..) => out(&Value::Dict(d.clone())),
                Iter::Enumerate(a, b, last) => {
                    push(out, a);
                    push(out, b);
                    if let Some(t) = last {
                        out(&Value::Tuple(t.clone()));
                    }
                }
                Iter::Zip(its, _, last) => {
                    its.iter().for_each(|x| push(out, x));
                    if let Some(t) = last {
                        out(&Value::Tuple(t.clone()));
                    }
                }
                Iter::Map(f, its) => {
                    push(out, f);
                    its.iter().for_each(|x| push(out, x));
                }
                Iter::Filter(a, b, _) | Iter::Callable(a, b) | Iter::Count(a, b) => {
                    push(out, a);
                    push(out, b);
                }
                Iter::Seq(a, _) | Iter::SeqRev(a, _) | Iter::Repeat(a, _) => push(out, a),
                Iter::Chain(its, _, cur) => {
                    its.iter().for_each(|x| push(out, x));
                    push(out, cur);
                }
                Iter::Vec(rest) => rest.as_slice().iter().for_each(|x| push(out, x)),
                Iter::Str(..) | Iter::Bytes(..) | Iter::Range(..) | Iter::BigRange(_) | Iter::Empty => {}
            }
        }
        Value::Gen(g) => {
            let Ok(g) = g.try_borrow() else {
                return false;
            };
            if let Some(f) = &g.frame {
                frame_referents(f, out);
            }
            g.handled.iter().for_each(|x| push(out, x));
            out(&Value::Code(g.code.clone()));
            if let Some(f) = &g.frame_obj {
                push(out, f);
            }
        }
        Value::DictView(d) => out(&Value::Dict(d.0.clone())),
        Value::Property(p) => {
            let (Ok(doc), Ok(name)) = (p.doc.try_borrow(), p.name.try_borrow()) else {
                return false;
            };
            [&p.fget, &p.fset, &p.fdel, &*doc, &*name].into_iter().for_each(|x| push(out, x));
        }
        Value::StaticMethod(r) | Value::ClassMethod(r) => push(out, r),
        Value::Super(s) => {
            out(&Value::Class(s.0.clone()));
            push(out, &s.1);
        }
        Value::Cell(c) => match c.try_borrow() {
            Ok(c) => push(out, &c),
            Err(_) => return false,
        },
        Value::Code(c) => {
            c.consts.iter().for_each(|x| push(out, x));
            push(out, &c.doc);
        }
        Value::Obj(o) => {
            let Ok(kind) = o.kind.try_borrow() else {
                return false;
            };
            out(&Value::Class(o.class.clone()));
            match &*kind {
                ObjKind::Deque(d, _) => d.iter().for_each(|x| push(out, x)),
                ObjKind::Member(_, c) | ObjKind::GetSet(_, c, _) => out(&Value::Class(c.clone())),
                ObjKind::Traceback { frame, next, .. } => {
                    push(out, frame);
                    push(out, next);
                }
                ObjKind::Frame { code, globals, locals, back, .. } => {
                    out(&Value::Code(code.clone()));
                    out(&Value::Dict(globals.clone()));
                    push(out, locals);
                    push(out, back);
                }
                ObjKind::Other(x) | ObjKind::TupleGetter(_, x) => push(out, x),
                ObjKind::File { .. } | ObjKind::StringIO { .. } | ObjKind::MatchPat(_) | ObjKind::Rng(_) | ObjKind::Weak(_) | ObjKind::Abc(_) => {}
            }
        }
        _ => {}
    }
    true
}

/// The objects a collection looks at: everything reachable from the
/// registered objects and the running frames, each held once more by
/// `objs`, with the references among them (`edges[start[i]..start[i + 1]]`
/// are those of objs[i]).
struct Graph {
    objs: Vec<Value>,
    index: FxMap<usize, u32>,
    start: Vec<u32>,
    edges: Vec<u32>,
    /// the objects whose references could not be read (a borrow was held)
    opaque: Vec<u32>,
}

impl Graph {
    fn new(n: usize) -> Graph {
        let mut index = FxMap::default();
        index.reserve(n);
        Graph { objs: Vec::with_capacity(n), index, start: Vec::with_capacity(n + 1), edges: Vec::with_capacity(n), opaque: Vec::new() }
    }

    /// the index of `v`, which is added if new
    fn add(&mut self, v: &Value) -> u32 {
        let n = self.objs.len() as u32;
        match self.index.entry(ptr_id(v)) {
            std::collections::hash_map::Entry::Occupied(e) => *e.get(),
            std::collections::hash_map::Entry::Vacant(e) => {
                e.insert(n);
                self.objs.push(v.clone());
                n
            }
        }
    }

    /// adds everything reachable, with the references
    fn close(&mut self) {
        let mut i = 0;
        while i < self.objs.len() {
            let at = self.edges.len();
            self.start.push(at as u32);
            let v = self.objs[i].clone();
            if !referents(&v, &mut |k| {
                let j = self.add(k);
                self.edges.push(j);
            }) {
                self.opaque.push(i as u32);
                self.edges.truncate(at);
            }
            i += 1;
        }
        self.start.push(self.edges.len() as u32);
    }

    fn refs_of(&self, i: usize) -> &[u32] {
        &self.edges[self.start[i] as usize..self.start[i + 1] as usize]
    }

    /// the objects nothing outside the graph refers to, directly or
    /// through others
    fn unreachable(&self) -> Vec<usize> {
        // the references from outside: the count, less the graph's own
        // reference and those from the others
        let mut refs: Vec<isize> = self.objs.iter().map(|v| strong_count(v).unwrap_or(1) as isize - 1).collect();
        for &j in &self.edges {
            refs[j as usize] -= 1;
        }
        // whatever has a reference from outside is reachable, and so is
        // everything it refers to
        let mut reached = vec![false; self.objs.len()];
        let mut work: Vec<usize> = (0..self.objs.len()).filter(|&i| refs[i] > 0).chain(self.opaque.iter().map(|&i| i as usize)).collect();
        for &i in &work {
            reached[i] = true;
        }
        while let Some(i) = work.pop() {
            for &j in self.refs_of(i) {
                if !reached[j as usize] {
                    reached[j as usize] = true;
                    work.push(j as usize);
                }
            }
        }
        (0..self.objs.len()).filter(|&i| !reached[i]).collect()
    }

    /// what is left of `garbage` once its finalizers ran: those they made
    /// reachable again are left out, with all they refer to; the
    /// references are read again, as the finalizers may have changed them
    fn still_unreachable(&self, garbage: &[usize]) -> Vec<usize> {
        let mut pos: FxMap<usize, usize> = FxMap::default();
        for (k, &i) in garbage.iter().enumerate() {
            pos.insert(ptr_id(&self.objs[i]), k);
        }
        let mut refs: Vec<isize> = garbage.iter().map(|&i| strong_count(&self.objs[i]).unwrap_or(1) as isize - 1).collect();
        let mut opaque = vec![false; garbage.len()];
        let mut edges: Vec<Vec<usize>> = Vec::with_capacity(garbage.len());
        for (k, &i) in garbage.iter().enumerate() {
            let mut mine = Vec::new();
            opaque[k] = !referents(&self.objs[i], &mut |c| mine.extend(pos.get(&ptr_id(c)).copied()));
            for &j in &mine {
                refs[j] -= 1;
            }
            edges.push(mine);
        }
        let mut reached = vec![false; garbage.len()];
        let mut work: Vec<usize> = (0..garbage.len()).filter(|&k| refs[k] > 0 || opaque[k]).collect();
        for &k in &work {
            reached[k] = true;
        }
        while let Some(k) = work.pop() {
            for &j in &edges[k] {
                if !reached[j] {
                    reached[j] = true;
                    work.push(j);
                }
            }
        }
        (0..garbage.len()).filter(|&k| !reached[k]).map(|k| garbage[k]).collect()
    }
}

/// whether CPython's collector would track `v`'s type at all: atoms, static
/// types and bare object() instances are not
fn gc_type(vm: &Vm, v: &Value) -> bool {
    match v {
        Value::Undef | Value::None | Value::NotImplemented | Value::Ellipsis | Value::Bool(_) | Value::Int(_) | Value::Big(_) | Value::Float(_) | Value::Complex(_) => false,
        Value::Str(_) | Value::Bytes(_) | Value::ByteArray(_) | Value::Range(_) | Value::BigRange(_) | Value::Code(_) => false,
        Value::Class(c) => c.dict.borrow().get_s("__module__").is_some(),
        Value::Instance(i) => !Rc::ptr_eq(&i.class.borrow(), &vm.t.object),
        _ => true,
    }
}

/// whether CPython would report `v` as tracked: a tuple until a collection
/// found it holding nothing tracked, a dict while it holds something that
/// may be
fn gc_tracked(vm: &Vm, v: &Value) -> bool {
    let may = |x: &Value| match x {
        Value::Tuple(_) => gc_tracked(vm, x),
        _ => gc_type(vm, x),
    };
    match v {
        Value::Tuple(t) => !untracked(t),
        Value::Dict(d) => d.borrow().iter().any(|e| may(&e.key) || may(&e.value)),
        _ => gc_type(vm, v),
    }
}

/// what CPython's collector does to the tuples it sees: those holding only
/// untracked objects are untracked
fn untrack_tuples(vm: &Vm, g: &Graph) {
    UNTRACKED.with(|u| {
        let mut u = u.borrow_mut();
        u.retain(|_, w| w.strong_count() > 0);
        // inner tuples are found after the ones holding them
        for v in g.objs.iter().rev() {
            let Value::Tuple(t) = v else { continue };
            let atomic = t.iter().all(|x| match x {
                Value::Tuple(i) => u.contains_key(&(Rc::as_ptr(i) as *const u8 as usize)),
                _ => !gc_type(vm, x),
            });
            if atomic {
                u.insert(Rc::as_ptr(t) as *const u8 as usize, Rc::downgrade(t));
            }
        }
    });
}

/// a weak handle to garbage reads None from now on
fn clear_weakref(o: &NativeObj, dead: &FxMap<usize, ()>) {
    let Ok(mut kind) = o.kind.try_borrow_mut() else { return };
    let ObjKind::Weak(t) = &mut *kind else { return };
    let addr = match t {
        WeakTarget::Inst(w) => w.as_ptr() as *const u8 as usize,
        WeakTarget::Func(w) => w.as_ptr() as *const u8 as usize,
        WeakTarget::Class(w) => w.as_ptr() as *const u8 as usize,
        WeakTarget::Gen(w) => w.as_ptr() as *const u8 as usize,
        WeakTarget::Set(w, _) => w.as_ptr() as *const u8 as usize,
        WeakTarget::Obj(w) => w.as_ptr() as *const u8 as usize,
    };
    if dead.contains_key(&addr) {
        *t = match t {
            WeakTarget::Inst(_) => WeakTarget::Inst(Weak::new()),
            WeakTarget::Func(_) => WeakTarget::Func(Weak::new()),
            WeakTarget::Class(_) => WeakTarget::Class(Weak::new()),
            WeakTarget::Gen(_) => WeakTarget::Gen(Weak::new()),
            WeakTarget::Set(_, f) => WeakTarget::Set(Weak::new(), *f),
            WeakTarget::Obj(_) => WeakTarget::Obj(Weak::new()),
        };
    }
}

/// whether garbage `v` has a finalizer still to run: __del__ of an
/// instance (once, PEP 442), close() of a suspended generator
fn needs_finalizer(v: &Value) -> bool {
    match v {
        Value::Instance(i) => i.class.borrow().has_del.get() && i.extra.borrow().get_s("__finalized__").is_none(),
        Value::Gen(g) => g.try_borrow().is_ok_and(|g| g.started && !g.running && g.frame.is_some()),
        _ => false,
    }
}

/// tp_clear: drops the references garbage `v` holds, into `trash` so that
/// nothing is freed while a borrow is held
fn clear(v: &Value, trash: &mut Vec<Value>) {
    match v {
        Value::List(l) => {
            if let Ok(mut l) = l.try_borrow_mut() {
                trash.append(&mut l);
            }
        }
        Value::Dict(d) | Value::Set(d) | Value::FrozenSet(d) => {
            if let Ok(mut d) = d.try_borrow_mut() {
                let d = std::mem::take(&mut *d);
                trash.extend(d.entries.into_iter().flatten().flat_map(|e| [e.key, e.value]));
            }
        }
        Value::Instance(i) => {
            if let Ok(mut b) = i.base.try_borrow_mut() {
                trash.push(std::mem::replace(&mut *b, Value::Undef));
            }
            if let Ok(mut x) = i.extra.try_borrow_mut() {
                // the mark that __del__ has run stays
                let fin = x.get_s("__finalized__");
                let old = std::mem::take(&mut *x);
                if let Some(f) = fin {
                    x.set_s("__finalized__", f);
                }
                trash.extend(old.entries.into_iter().flatten().flat_map(|e| [e.key, e.value]));
            }
        }
        Value::Class(c) => {
            if let (Ok(mut b), Ok(mut m), Ok(mut meta)) = (c.bases.try_borrow_mut(), c.mro.try_borrow_mut(), c.meta.try_borrow_mut()) {
                trash.extend(b.drain(..).chain(m.drain(..)).chain(meta.take()).map(Value::Class));
            }
        }
        Value::Func(f) => {
            if let Ok(mut d) = f.defaults.try_borrow_mut() {
                trash.append(&mut d);
            }
            if let Ok(mut d) = f.kwdefaults.try_borrow_mut() {
                trash.extend(d.drain(..).map(|(_, v)| v));
            }
            if let Ok(mut d) = f.dict.try_borrow_mut() {
                trash.extend(d.take().map(Value::Dict));
            }
            if let Ok(mut a) = f.annotations.try_borrow_mut() {
                trash.push(std::mem::replace(&mut *a, Value::None));
            }
        }
        Value::Cell(c) => {
            if let Ok(mut c) = c.try_borrow_mut() {
                trash.push(std::mem::replace(&mut *c, Value::Undef));
            }
        }
        Value::Gen(g) => {
            if let Ok(mut g) = g.try_borrow_mut() {
                if let Some(f) = g.frame.take() {
                    let f = *f;
                    trash.extend(f.stack);
                    trash.extend(f.cells.into_iter().map(Value::Cell));
                }
                trash.append(&mut g.handled);
                trash.extend(g.frame_obj.take());
            }
        }
        Value::Iter(it) => {
            if let Ok(mut it) = it.try_borrow_mut() {
                let old = std::mem::replace(&mut *it, Iter::Empty);
                trash.push(Value::Iter(Rc::new(RefCell::new(old))));
            }
        }
        Value::Property(p) => {
            if let (Ok(mut d), Ok(mut n)) = (p.doc.try_borrow_mut(), p.name.try_borrow_mut()) {
                trash.push(std::mem::replace(&mut *d, Value::None));
                trash.push(std::mem::replace(&mut *n, Value::Undef));
            }
        }
        Value::Obj(o) => {
            if let Ok(mut k) = o.kind.try_borrow_mut() {
                match &mut *k {
                    ObjKind::Deque(d, _) => trash.extend(d.drain(..)),
                    ObjKind::Traceback { frame, next, .. } => {
                        trash.push(std::mem::replace(frame, Value::None));
                        trash.push(std::mem::replace(next, Value::None));
                    }
                    ObjKind::Frame { locals, back, .. } => {
                        trash.push(std::mem::replace(locals, Value::None));
                        trash.push(std::mem::replace(back, Value::None));
                    }
                    ObjKind::Other(x) => trash.push(std::mem::replace(x, Value::None)),
                    _ => {}
                }
            }
        }
        _ => {}
    }
}

/// a full collection; the number of unreachable objects found
pub fn collect(vm: &mut Vm) -> usize {
    if COLLECTING.with(|c| c.replace(true)) {
        return 0;
    }
    let n = collect_cycles(vm);
    COLLECTING.with(|c| c.set(false));
    rearm();
    STATS.with(|s| {
        let (runs, found) = s.get();
        s.set((runs + 1, found + n));
    });
    // what the clearing freed may have queued finalizers of its own
    vm.finalize_pending();
    n
}

fn collect_cycles(vm: &mut Vm) -> usize {
    let mut seeds: Vec<Value> = OWNED.with(|r| r.borrow().iter().map(Owned::value).collect());
    REGISTRY.with(|r| {
        let mut r = r.borrow_mut();
        r.compact();
        seeds.extend(r.objs.iter().filter_map(Tracked::upgrade));
    });
    let mut g = Graph::new(seeds.len() * 2);
    for v in &seeds {
        g.add(v);
    }
    drop(seeds);
    // the running frames refer to objects from outside, but what they
    // reach is looked at too (for the tuples it holds)
    for &f in &vm.frames {
        frame_referents(unsafe { &*f }, &mut |k| {
            g.add(k);
        });
    }
    g.close();
    untrack_tuples(vm, &g);

    let mut garbage = g.unreachable();
    if garbage.is_empty() {
        return 0;
    }
    let dead: FxMap<usize, ()> = garbage.iter().map(|&i| (ptr_id(&g.objs[i]), ())).collect();
    for v in &g.objs {
        if let Value::Obj(o) = v {
            clear_weakref(o, &dead);
        }
    }

    // finalizers first, while the garbage is intact; they may make some of
    // it reachable again, which is then left alone
    let fin: Vec<Value> = garbage.iter().map(|&i| &g.objs[i]).filter(|v| needs_finalizer(v)).cloned().collect();
    if !fin.is_empty() {
        for v in &fin {
            if let Value::Instance(i) = v {
                i.extra.borrow_mut().set_s("__finalized__", Value::Bool(true));
            }
        }
        PENDING_DEL.with(|p| p.borrow_mut().extend(fin));
        vm.run_finalizers();
        garbage = g.still_unreachable(&garbage);
    }

    let mut trash = Vec::new();
    for &i in &garbage {
        clear(&g.objs[i], &mut trash);
    }
    drop(trash);
    garbage.len()
}

// -------------------------------------------------------------------------
// the _rpygc module, under lib/gc.py

pub fn gc_module() -> Value {
    let (d, m) = module("_rpygc");
    put_fns(&d, &[("_rpygc.is_tracked", gc_is_tracked), ("_rpygc.collect", gc_collect), ("_rpygc.collections", gc_collections), ("_rpygc.auto", gc_auto), ("_rpygc.is_finalized", gc_is_finalized)]);
    m
}

fn gc_is_tracked(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    need(vm, a, 1)?;
    Ok(Value::Bool(gc_tracked(vm, &a[0])))
}

fn gc_collect(vm: &mut Vm, _a: &[Value], _k: Kw) -> PyResult<Value> {
    Ok(Value::Int(collect(vm) as i64))
}

/// (collections run, objects found unreachable)
fn gc_collections(_vm: &mut Vm, _a: &[Value], _k: Kw) -> PyResult<Value> {
    let (runs, found) = STATS.with(|s| s.get());
    Ok(Value::tuple(vec![Value::Int(runs as i64), Value::Int(found as i64)]))
}

/// auto(enabled, threshold0): sets how collections run on their own
fn gc_auto(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    need(vm, a, 2)?;
    let on = vm.truthy(&a[0])?;
    let t0 = vm.to_index(&a[1])?.max(0) as usize;
    AUTO.with(|x| x.set((on, t0, 0, false)));
    rearm();
    Ok(Value::None)
}

/// whether the __del__ of `obj` has run (PEP 442)
fn gc_is_finalized(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    need(vm, a, 1)?;
    Ok(Value::Bool(matches!(&a[0], Value::Instance(i) if i.extra.borrow().get_s("__finalized__").is_some())))
}
