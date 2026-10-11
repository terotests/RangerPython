// SPDX-License-Identifier: MIT
//! Python values: one enum, with the heap kinds behind Rc.

use crate::bigint::BigInt;
use crate::code::Code;
use std::cell::{Cell, RefCell};
use std::hash::Hasher;
use std::rc::Rc;

pub type Name = Rc<str>;

/// A Python str: the text, whether it is all ASCII (so indexing is by
/// byte), and its hash computed once. A str with lone surrogates also has
/// its exact code points in `wtf`, and U+FFFD for each in `s` (see wtf8.rs).
pub struct PyStr {
    pub s: String,
    pub ascii: bool,
    pub wtf: Option<Box<[u8]>>,
    hash: Cell<u64>,
}

impl PyStr {
    pub fn new(s: String) -> PyStr {
        let ascii = s.is_ascii();
        PyStr { s, ascii, wtf: None, hash: Cell::new(0) }
    }
    #[inline(always)]
    pub fn hash(&self) -> u64 {
        let h = self.hash.get();
        if h != 0 {
            return h;
        }
        self.compute_hash()
    }
    #[cold]
    #[inline(never)]
    fn compute_hash(&self) -> u64 {
        let h = hash_bytes(self.wtf8());
        self.hash.set(h);
        h
    }
    pub fn char_len(&self) -> usize {
        if self.ascii {
            self.s.len()
        } else {
            self.s.chars().count()
        }
    }
}

impl std::fmt::Debug for PyStr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(&crate::format::pystr_repr(self))
    }
}

/// hash of bytes, and of an ASCII str with those bytes (hash('a') == hash(b'a'))
pub fn hash_bytes(b: &[u8]) -> u64 {
    let mut f = FxHasher::default();
    f.write(b);
    // already reduced as hash_int would, so an object whose __hash__
    // returns hash(some_str) collides with that str
    (f.finish() % MODULUS).max(1)
}

#[derive(Default, Clone, Copy)]
pub struct FxHasher {
    hash: u64,
}

impl Hasher for FxHasher {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        for chunk in bytes.chunks(8) {
            let mut v = [0u8; 8];
            v[..chunk.len()].copy_from_slice(chunk);
            self.add(u64::from_le_bytes(v));
        }
    }
    #[inline]
    fn write_u64(&mut self, i: u64) {
        self.add(i);
    }
    #[inline]
    fn finish(&self) -> u64 {
        self.hash
    }
}

impl FxHasher {
    #[inline]
    fn add(&mut self, w: u64) {
        self.hash = (self.hash.rotate_left(5) ^ w).wrapping_mul(0x51_7c_c1_b7_27_22_0a_95);
    }
}

pub type Cellv = Rc<RefCell<Value>>;

#[derive(Clone)]
pub enum Value {
    /// an unbound local; never seen by Python code
    Undef,
    None,
    NotImplemented,
    Ellipsis,
    Bool(bool),
    Int(i64),
    Big(Rc<BigInt>),
    Float(f64),
    Complex(Rc<(f64, f64)>),
    Str(Rc<PyStr>),
    Bytes(Rc<Vec<u8>>),
    ByteArray(Rc<RefCell<Vec<u8>>>),
    List(Rc<RefCell<Vec<Value>>>),
    Tuple(Rc<Vec<Value>>),
    Dict(Rc<RefCell<Dict>>),
    Set(Rc<RefCell<Dict>>),
    FrozenSet(Rc<RefCell<Dict>>),
    Range(Rc<(i64, i64, i64)>),
    /// a range whose bounds or length need big integers
    BigRange(Rc<crate::range::BigRange>),
    Slice(Rc<(Value, Value, Value)>),
    Func(Rc<Function>),
    Native(Rc<NativeFn>),
    /// a function bound to its first argument
    Method(Rc<(Value, Value)>),
    Class(Rc<Class>),
    Instance(Rc<Instance>),
    Module(Rc<Module>),
    Iter(Rc<RefCell<Iter>>),
    Gen(Rc<RefCell<crate::vm::Generator>>),
    DictView(Rc<(Rc<RefCell<Dict>>, u8)>),
    Property(Rc<Property>),
    StaticMethod(Rc<Value>),
    ClassMethod(Rc<Value>),
    Super(Rc<(Rc<Class>, Value)>),
    Cell(Cellv),
    Code(Rc<Code>),
    Obj(Rc<NativeObj>),
}

pub struct Property {
    pub fget: Value,
    pub fset: Value,
    pub fdel: Value,
    pub doc: RefCell<Value>,
    /// set by __set_name__; Undef until then
    pub name: RefCell<Value>,
    /// the doc came from fget
    pub getter_doc: Cell<bool>,
}

pub type NativeFnPtr = fn(&mut crate::vm::Vm, &[Value], &[(Rc<PyStr>, Value)]) -> PyResult<Value>;

pub struct NativeFn {
    pub name: &'static str,
    pub f: NativeFnPtr,
    /// data a native closure carries (functools.partial and the like)
    pub data: Value,
    /// a method of a type binds to its instance; a module function does not
    pub bind: bool,
    /// the positional arguments a method takes besides self, when checked
    pub arity: Option<(u8, u8)>,
}

impl NativeFn {
    pub fn new(name: &'static str, f: NativeFnPtr, data: Value) -> NativeFn {
        NativeFn { name, f, data, bind: true, arity: None }
    }

    /// the name after the type's ("__add__" of "int.__add__")
    pub fn short_name(&self) -> &'static str {
        self.name.rsplit('.').next().unwrap_or(self.name)
    }

    /// a type's method that fills one of CPython's type slots (__add__,
    /// __repr__, ...): a slot wrapper there, a method-wrapper once bound
    pub fn is_slot(&self) -> bool {
        self.bind && SLOT_NAMES.binary_search(&self.short_name()).is_ok()
    }
}

/// the methods CPython's builtin types expose as slot wrappers, sorted
const SLOT_NAMES: &[&str] = &[
    "__abs__", "__add__", "__aiter__", "__and__", "__anext__", "__await__", "__bool__", "__buffer__", "__call__",
    "__contains__", "__del__", "__delattr__", "__delete__", "__delitem__", "__divmod__", "__eq__", "__float__",
    "__floordiv__", "__ge__", "__get__", "__getattribute__", "__getitem__", "__gt__", "__hash__", "__iadd__",
    "__iand__", "__ifloordiv__", "__ilshift__", "__imatmul__", "__imod__", "__imul__", "__index__", "__init__",
    "__int__", "__invert__", "__ior__", "__ipow__", "__irshift__", "__isub__", "__iter__", "__itruediv__",
    "__ixor__", "__le__", "__len__", "__lshift__", "__lt__", "__matmul__", "__mod__", "__mul__", "__ne__",
    "__neg__", "__next__", "__or__", "__pos__", "__pow__", "__radd__", "__rand__", "__rdivmod__",
    "__release_buffer__", "__repr__", "__rfloordiv__", "__rlshift__", "__rmatmul__", "__rmod__", "__rmul__",
    "__ror__", "__rpow__", "__rrshift__", "__rshift__", "__rsub__", "__rtruediv__", "__rxor__", "__set__",
    "__setattr__", "__setitem__", "__str__", "__sub__", "__truediv__", "__xor__",
];

/// a function found on a type binds to the instance it is read from
pub fn binds(v: &Value) -> bool {
    match v {
        Value::Func(_) => true,
        Value::Native(n) => n.bind,
        _ => false,
    }
}

pub struct Function {
    /// replaceable through __code__
    pub code: RefCell<Rc<Code>>,
    pub globals: Rc<RefCell<Dict>>,
    pub defaults: RefCell<Vec<Value>>,
    pub kwdefaults: RefCell<Vec<(Name, Value)>>,
    pub closure: Vec<Cellv>,
    pub name: RefCell<Rc<PyStr>>,
    pub qualname: RefCell<Rc<PyStr>>,
    pub dict: RefCell<Option<Rc<RefCell<Dict>>>>,
    pub annotations: RefCell<Value>,
    pub gc: crate::gc::Slot,
}

pub struct Class {
    pub name: RefCell<Rc<PyStr>>,
    pub qualname: RefCell<Rc<PyStr>>,
    pub bases: RefCell<Vec<Rc<Class>>>,
    pub mro: RefCell<Vec<Rc<Class>>>,
    pub dict: Rc<RefCell<Dict>>,
    /// which built-in type this is or derives from
    pub builtin: Cell<Builtin>,
    pub meta: RefCell<Option<Rc<Class>>>,
    /// instances need the slow path for attribute access
    pub has_getattr: Cell<bool>,
    pub has_getattribute: Cell<bool>,
    pub has_setattr: Cell<bool>,
    pub has_descriptors: Cell<bool>,
    /// instances run __del__ when the last reference goes
    pub has_del: Cell<bool>,
    /// a type defined in Python standing in for a static C type: its
    /// attributes cannot be set (Py_TPFLAGS_IMMUTABLETYPE)
    pub immutable: Cell<bool>,
    pub slots: RefCell<Option<Vec<Name>>>,
    /// the attribute names its instances share, for a class whose instances
    /// keep their attributes inline (CPython's ht_cached_keys)
    pub shared_keys: RefCell<Option<Rc<SharedKeys>>>,
    /// never reused, unlike the address: keys the lookup cache
    id: u64,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Builtin {
    Object,
    Type,
    Int,
    Bool,
    Float,
    Complex,
    Str,
    Bytes,
    ByteArray,
    List,
    Tuple,
    Dict,
    Set,
    FrozenSet,
    Range,
    Slice,
    NoneType,
    Function,
    BuiltinFunction,
    Method,
    Module,
    Iterator,
    Generator,
    DictKeys,
    DictValues,
    DictItems,
    Property,
    StaticMethod,
    ClassMethod,
    Super,
    BaseException,
    NotImplementedType,
    Ellipsis,
    Cell,
    Code,
    Obj,
    /// a user class with no built-in base other than object
    User,
}

pub struct Instance {
    pub class: RefCell<Rc<Class>>,
    pub attrs: RefCell<Attrs>,
    /// the built-in value a subclass of int, str, list, dict... wraps
    pub base: RefCell<Value>,
    /// values kept outside __dict__: __slots__ members, and an exception's
    /// __traceback__, __cause__, __context__ and __suppress_context__
    pub extra: RefCell<Dict>,
    pub gc: crate::gc::Slot,
}

/// Where an instance keeps its attributes. As in CPython 3.13, an instance
/// of an ordinary class starts with them inline, in a table split on its
/// class's shared keys, and only gets a dict object when `__dict__` is asked
/// for. Here that dict takes the table over, and it stays split until it
/// cannot be (see Dict::shared).
pub enum Attrs {
    /// in the instance itself (CPython's inline values); for a class
    /// without shared keys, an empty table until the first is set
    Inline(Dict),
    /// in the dict made of the inline values, when `__dict__` was asked for
    /// or they could take no more keys (or the first was set, without
    /// shared keys)
    Materialized(Rc<RefCell<Dict>>),
    /// in a dict that was never the inline values: the one assigned to
    /// `__dict__`, or the fresh one after `del obj.__dict__` or after a
    /// `__class__` assignment
    Dict(Rc<RefCell<Dict>>),
}

impl Instance {
    /// a new instance, its attributes inline when its class allows
    pub fn new(class: Rc<Class>, base: Value) -> Instance {
        let table = match &*class.shared_keys.borrow() {
            Some(k) => Dict::new_split(k.clone()),
            None => Dict::new(),
        };
        let attrs = Attrs::Inline(table);
        Instance { class: RefCell::new(class), attrs: RefCell::new(attrs), base: RefCell::new(base), extra: RefCell::new(Dict::new()), gc: Default::default() }
    }

    /// a new instance whose attributes are `d`
    pub fn with_dict(class: Rc<Class>, d: Dict, base: Value) -> Instance {
        let attrs = Attrs::Dict(Rc::new(RefCell::new(d)));
        Instance { class: RefCell::new(class), attrs: RefCell::new(attrs), base: RefCell::new(base), extra: RefCell::new(Dict::new()), gc: Default::default() }
    }

    /// `f` of the table that holds the attributes
    #[inline]
    pub fn with_attrs<R>(&self, f: impl FnOnce(&Dict) -> R) -> R {
        match &*self.attrs.borrow() {
            Attrs::Inline(d) => f(d),
            Attrs::Materialized(d) | Attrs::Dict(d) => f(&d.borrow()),
        }
    }

    /// `f` of the table that holds the attributes, to change it; inline
    /// values that `f` made combined (or left so) move to a dict object
    pub fn with_attrs_mut<R>(&self, f: impl FnOnce(&mut Dict) -> R) -> R {
        let mut a = self.attrs.borrow_mut();
        match &mut *a {
            Attrs::Inline(d) => {
                let r = f(d);
                if !d.is_split() {
                    let d = std::mem::take(d);
                    *a = Attrs::Materialized(Rc::new(RefCell::new(d)));
                }
                r
            }
            Attrs::Materialized(d) | Attrs::Dict(d) => f(&mut d.borrow_mut()),
        }
    }

    /// the attribute in slot `slot` of the inline values (or of the split
    /// dict that took them over)
    #[inline]
    pub fn slot_attr(&self, slot: usize) -> Option<Value> {
        match &*self.attrs.borrow() {
            Attrs::Inline(d) => d.slot_value(slot).cloned(),
            Attrs::Materialized(d) => d.borrow().slot_value(slot).cloned(),
            Attrs::Dict(_) => None,
        }
    }

    /// sets attribute `name`, whose slot is `slot`, in the inline values,
    /// giving back the old value; Err when they are not where it goes
    #[inline]
    pub fn set_slot_attr(&self, slot: usize, name: &Rc<PyStr>, v: Value) -> Result<Option<Value>, Value> {
        let mut a = self.attrs.borrow_mut();
        let mut d = match &mut *a {
            Attrs::Inline(d) => return d.set_slot(slot, name, v),
            Attrs::Materialized(d) => d.borrow_mut(),
            Attrs::Dict(_) => return Err(v),
        };
        d.set_slot(slot, name, v)
    }

    /// the slot of attribute `name` while the instance keeps it inline
    pub fn attr_slot(&self, name: &PyStr) -> Option<usize> {
        match &*self.attrs.borrow() {
            Attrs::Inline(d) => d.slot_of(name).filter(|&s| d.slot_value(s).is_some()),
            Attrs::Materialized(d) => {
                let d = d.borrow();
                d.slot_of(name).filter(|&s| d.slot_value(s).is_some())
            }
            Attrs::Dict(_) => None,
        }
    }

    /// the attribute `name` the instance itself holds
    #[inline]
    pub fn attr(&self, name: &PyStr) -> Option<Value> {
        self.with_attrs(|d| d.get_str(name).cloned())
    }

    pub fn has_attr(&self, name: &PyStr) -> bool {
        self.with_attrs(|d| d.get_str(name).is_some())
    }

    pub fn set_attr(&self, name: Rc<PyStr>, v: Value) {
        // the value replaced is dropped after the table is let go
        let _old = self.with_attrs_mut(|d| match d.find_str(&name) {
            Some(i) => d.entries[i].as_mut().map(|e| std::mem::replace(&mut e.value, v)),
            None => {
                d.push_new(name.hash(), Value::Str(name), v);
                None
            }
        });
    }

    pub fn del_attr(&self, name: &PyStr) -> Option<Value> {
        self.with_attrs_mut(|d| d.remove_str(name))
    }

    /// `__dict__`: made of the inline values the first time it is asked for
    pub fn dict(&self) -> Rc<RefCell<Dict>> {
        let mut a = self.attrs.borrow_mut();
        if let Attrs::Inline(d) = &mut *a {
            let d = std::mem::take(d);
            *a = Attrs::Materialized(Rc::new(RefCell::new(d)));
        }
        match &*a {
            Attrs::Materialized(d) | Attrs::Dict(d) => d.clone(),
            Attrs::Inline(_) => unreachable!(),
        }
    }

    /// `obj.__dict__ = d`; the old dict keeps what it holds
    pub fn set_dict(&self, d: Rc<RefCell<Dict>>) {
        let _old = std::mem::replace(&mut *self.attrs.borrow_mut(), Attrs::Dict(d));
    }

    /// the attributes leave the inline layout, as on a `__class__`
    /// assignment: a dict made of them, or the one made already, is the
    /// instance's own from now on
    pub fn detach_attrs(&self) {
        let mut a = self.attrs.borrow_mut();
        let d = match &mut *a {
            Attrs::Inline(d) => Rc::new(RefCell::new(std::mem::take(d))),
            Attrs::Materialized(d) => d.clone(),
            Attrs::Dict(_) => return,
        };
        *a = Attrs::Dict(d);
    }

    /// whether the attributes are still kept in the split table the
    /// instance started with (_testinternalcapi.has_inline_values)
    pub fn has_inline_values(&self) -> bool {
        match &*self.attrs.borrow() {
            Attrs::Inline(d) => d.is_split(),
            Attrs::Materialized(d) => d.borrow().is_split(),
            Attrs::Dict(_) => false,
        }
    }
}

crate::gil_static! {
    /// instances whose __del__ is still to run, kept alive until then
    pub static PENDING_DEL: RefCell<Vec<Value>> = const { RefCell::new(Vec::new()) };
    pub static HAS_PENDING_DEL: Cell<bool> = const { Cell::new(false) };
}

impl Drop for Instance {
    fn drop(&mut self) {
        crate::gc::untrack(&self.gc);
        if !self.class.get_mut().has_del.get() {
            return;
        }
        let extra = self.extra.get_mut();
        if extra.get_s("__finalized__").is_some() {
            return;
        }
        // the object comes back to life once, so that __del__ can see it
        let mut extra = std::mem::replace(extra, Dict::new());
        extra.set_s("__finalized__", Value::Bool(true));
        let inst = Instance {
            class: RefCell::new(self.class.get_mut().clone()),
            attrs: RefCell::new(std::mem::replace(self.attrs.get_mut(), Attrs::Inline(Dict::new()))),
            base: RefCell::new(std::mem::replace(self.base.get_mut(), Value::Undef)),
            extra: RefCell::new(extra),
            gc: Default::default(),
        };
        let inst = Rc::new(inst);
        crate::gc::track_inst(&inst);
        let v = Value::Instance(inst);
        let _ = PENDING_DEL.try_with(|p| {
            if let Ok(mut p) = p.try_borrow_mut() {
                p.push(v);
                let _ = HAS_PENDING_DEL.try_with(|h| h.set(true));
            }
        });
    }
}

pub struct Module {
    pub name: Rc<PyStr>,
    pub dict: Rc<RefCell<Dict>>,
}

/// Objects implemented natively: files, StringIO, regex patterns...
pub struct NativeObj {
    pub class: Rc<Class>,
    pub gc: crate::gc::Slot,
    pub kind: RefCell<ObjKind>,
}

pub enum ObjKind {
    /// an open file: its bytes and the byte position; a text file's
    /// encoding and errors handler turn them into str and back. A file on
    /// a host descriptor (`fd`) has in `buf` what was read and not taken
    /// yet, and `closefd` says whether closing it closes the descriptor.
    File { path: String, mode: String, buf: Vec<u8>, pos: usize, closed: bool, writable: bool, binary: bool, encoding: String, errors: String, fd: Option<i32>, closefd: bool },
    StringIO { buf: Vec<char>, pos: usize, closed: bool },
    Deque(std::collections::VecDeque<Value>, Option<usize>),
    MatchPat(Rc<crate::ast::Pattern>),
    Rng(Box<crate::modules::Random>),
    /// a __slots__ member: the slot's name and the class that declared it
    Member(Rc<PyStr>, Rc<Class>),
    /// an attribute of a built-in type, read through the type's own code:
    /// function.__code__ (getset) or function.__globals__ (member)
    GetSet(Rc<PyStr>, Rc<Class>, bool),
    /// a traceback entry; `next` is the entry of the frame it was raised in
    Traceback { frame: Value, line: u32, next: Value, lasti: i64 },
    /// a frame as tracebacks and sys._getframe see it
    Frame { code: Rc<crate::code::Code>, line: u32, globals: Rc<RefCell<Dict>>, locals: Value, back: Value },
    /// the referent of a weakref.ref
    Weak(WeakTarget),
    /// a namedtuple field: _collections._tuplegetter(index, doc)
    TupleGetter(usize, Value),
    /// an ABC's registry and subclass caches (_abc._abc_data)
    Abc(Box<crate::abcmod::AbcData>),
    /// a lock or thread handle of _thread
    Thread(crate::threadmod::ThreadObj),
    /// a decimal.Decimal
    Decimal(Rc<crate::mpdec::Dec>),
    /// a decimal.Context
    DecContext(Box<crate::cdecimal::DecContext>),
    /// a context's flags or traps (decimal.SignalDictMixin), sharing the
    /// context's bits; None until tied to a context
    DecSignals(Option<Rc<Cell<u32>>>),
    Other(Value),
}

pub enum WeakTarget {
    Inst(std::rc::Weak<Instance>),
    Func(std::rc::Weak<Function>),
    Class(std::rc::Weak<Class>),
    Gen(std::rc::Weak<RefCell<crate::vm::Generator>>),
    /// a set or (when the flag is set) a frozenset
    Set(std::rc::Weak<RefCell<Dict>>, bool),
    Obj(std::rc::Weak<NativeObj>),
}

pub enum Iter {
    List(Rc<RefCell<Vec<Value>>>, usize),
    ListRev(Rc<RefCell<Vec<Value>>>, usize),
    Tuple(Rc<Vec<Value>>, usize),
    Str(Rc<PyStr>, usize),
    Bytes(Rc<Vec<u8>>, usize),
    /// current, remaining count, step
    Range(i64, i64, i64),
    /// current, step, remaining count
    BigRange(Box<(crate::bigint::BigInt, crate::bigint::BigInt, crate::bigint::BigInt)>),
    /// dict, position, kind (keys, values, items, set; +4 reversed), size seen, entries left
    Dict(Rc<RefCell<Dict>>, usize, u8, usize, usize),
    /// inner iterator, count, and the last result tuple, refilled while
    /// nothing else holds it
    Enumerate(Value, Value, Option<Rc<Vec<Value>>>),
    Zip(Vec<Value>, bool, Option<Rc<Vec<Value>>>),
    Map(Value, Vec<Value>),
    Filter(Value, Value, bool),
    /// a sequence driven by __getitem__
    Seq(Value, i64),
    /// reversed() over __len__ and __getitem__: the next index to fetch
    SeqRev(Value, i64),
    Callable(Value, Value),
    Count(Value, Value),
    Repeat(Value, i64),
    Chain(Vec<Value>, usize, Value),
    Empty,
    /// values already computed
    Vec(std::vec::IntoIter<Value>),
}

pub type PyResult<T> = Result<T, Exc>;

/// A raised exception: the exception instance.
pub struct Exc {
    pub value: Value,
}

// -------------------------------------------------------------------------
// dict: insertion ordered, open addressing over an index table

pub struct Entry {
    pub hash: u64,
    pub key: Value,
    pub value: Value,
}

crate::gil_static! {
    /// bumped when a class dict gains or loses keys or an MRO changes;
    /// Class::lookup's cache entries hold while it stays the same
    static TYPE_VERSION: Cell<u64> = const { Cell::new(1) };
    static NEXT_CLASS_ID: Cell<u64> = const { Cell::new(0) };
    static LOOKUP_CACHE: RefCell<Vec<CacheEntry>> = RefCell::new((0..LOOKUP_CACHE_SIZE).map(|_| CacheEntry::default()).collect());
    /// interned strs, one object per distinct text: identifiers the
    /// compiler names and what sys.intern was given
    static INTERNED: RefCell<std::collections::HashMap<Box<str>, Rc<PyStr>, std::hash::BuildHasherDefault<FxHasher>>> = RefCell::new(Default::default());
}

const LOOKUP_CACHE_BITS: u32 = 12;
const LOOKUP_CACHE_SIZE: usize = 1 << LOOKUP_CACHE_BITS;
/// names longer than this are looked up without the cache
const CACHED_NAME: usize = 32;

/// one remembered Class::lookup: where along the MRO the name is, if anywhere
#[derive(Default)]
struct CacheEntry {
    version: u64,
    class: u64,
    hash: u64,
    len: u8,
    name: [u8; CACHED_NAME],
    /// (MRO position, entry index)
    found: Option<(u32, u32)>,
}

/// a.same(b) out of line, for the probes that rarely get that far
#[inline(never)]
fn str_eq_slow(a: &PyStr, b: &PyStr) -> bool {
    a.same(b)
}

/// the interned str with this text
pub fn intern(s: &str) -> Rc<PyStr> {
    INTERNED.with(|t| {
        if let Some(v) = t.borrow().get(s) {
            return v.clone();
        }
        let v = Rc::new(PyStr::new(s.to_string()));
        t.borrow_mut().insert(s.into(), v.clone());
        v
    })
}

/// the interned str equal to `s`, which becomes it when there is none
/// (a str with lone surrogates is its own)
pub fn intern_str(s: &Rc<PyStr>) -> Rc<PyStr> {
    if s.wtf.is_some() {
        return s.clone();
    }
    INTERNED.with(|t| t.borrow_mut().entry(s.s.as_str().into()).or_insert_with(|| s.clone()).clone())
}

/// the version Class::lookup's cache entries are valid for
#[inline]
pub fn type_version() -> u64 {
    TYPE_VERSION.with(Cell::get)
}

pub fn types_changed() {
    TYPE_VERSION.with(|v| v.set(v.get() + 1));
}

pub struct Dict {
    pub entries: Vec<Option<Entry>>,
    /// where each key's entry is: by hash, or by the key's slot among the
    /// shared keys in a split table
    index: Vec<i32>,
    pub len: usize,
    /// a class's dict: adding or removing keys invalidates the lookup cache
    watched: bool,
    /// a split table (CPython's): the dict of an instance, whose keys are
    /// found among its class's shared keys rather than hashed here. It
    /// becomes combined, with an index of its own, for a key that is not an
    /// exact str or that the shared keys have no room for, and on popitem()
    shared: Option<Rc<SharedKeys>>,
    /// a number no other state of this or any dict's keys has had, or 0
    /// (none yet, or a key was added since): a lookup that missed stays
    /// valid while the stamp is the one it saw
    stamp: u64,
}

static NEXT_STAMP: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

impl Dict {
    /// the stamp of the keys as they are now (see `stamp`)
    pub fn stamp(&mut self) -> u64 {
        if self.stamp == 0 {
            self.stamp = NEXT_STAMP.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
        self.stamp
    }
    /// the stamp, 0 when there is none to rely on
    #[inline(always)]
    pub fn current_stamp(&self) -> u64 {
        self.stamp
    }
}

/// The attribute names the instances of a class share (CPython's cached
/// keys): each has a fixed slot, in the order the names were first set on
/// any instance, and is never removed
pub struct SharedKeys {
    names: RefCell<Dict>,
}

/// as CPython's SHARED_KEYS_MAX_SIZE
pub const SHARED_KEYS_MAX: usize = 30;

impl Default for SharedKeys {
    fn default() -> Self {
        SharedKeys { names: RefCell::new(Dict::new()) }
    }
}

impl SharedKeys {
    pub fn len(&self) -> usize {
        self.names.borrow().len
    }

    /// the slot of `key`, which is given one if it is new and there is room
    fn slot_or_add(&self, key: &Rc<PyStr>) -> Option<usize> {
        let mut names = self.names.borrow_mut();
        if let Some(slot) = names.find_str(key) {
            return Some(slot);
        }
        if names.len >= SHARED_KEYS_MAX {
            return None;
        }
        names.push_new(key.hash(), Value::Str(key.clone()), Value::None);
        Some(names.len - 1)
    }
}

pub enum Lookup {
    Found(usize),
    Missing,
    /// candidates with equal hash whose equality needs Python code
    Slow(Vec<usize>),
}

impl Default for Dict {
    fn default() -> Self {
        Dict::new()
    }
}

impl Dict {
    pub fn new() -> Dict {
        Dict { entries: Vec::new(), index: Vec::new(), len: 0, watched: false, shared: None, stamp: 0 }
    }
    pub fn with_capacity(n: usize) -> Dict {
        let mut d = Dict { entries: Vec::with_capacity(n), index: Vec::new(), len: 0, watched: false, shared: None, stamp: 0 };
        if n > 0 {
            let mut size = 8;
            while size * 2 < n * 3 {
                size *= 2;
            }
            d.index = vec![-1; size];
        }
        d
    }
    /// an empty split table on `keys`, with room for as many entries as
    /// they hold, as CPython sizes inline values
    pub fn new_split(keys: Rc<SharedKeys>) -> Dict {
        let n = keys.len();
        Dict { entries: Vec::with_capacity(n), index: vec![-1; n], len: 0, watched: false, shared: Some(keys), stamp: 0 }
    }
    pub fn is_split(&self) -> bool {
        self.shared.is_some()
    }
    /// the value of the key in shared slot `slot`, in a split table
    #[inline]
    pub fn slot_value(&self, slot: usize) -> Option<&Value> {
        self.shared.as_ref()?;
        self.entries[self.slot_entry(slot)?].as_ref().map(|e| &e.value)
    }
    /// sets `key`, the key in shared slot `slot`, in a split table, giving
    /// back the value it replaces; Err for a combined table
    #[inline]
    pub fn set_slot(&mut self, slot: usize, key: &Rc<PyStr>, v: Value) -> Result<Option<Value>, Value> {
        if self.shared.is_none() {
            return Err(v);
        }
        match self.slot_entry(slot) {
            Some(ix) => Ok(self.entries[ix].as_mut().map(|e| std::mem::replace(&mut e.value, v))),
            None => {
                if self.index.len() <= slot {
                    self.index.resize(slot + 1, -1);
                }
                self.index[slot] = self.entries.len() as i32;
                self.entries.push(Some(Entry { hash: key.hash(), key: Value::Str(key.clone()), value: v }));
                self.len += 1;
                self.stamp = 0;
                Ok(None)
            }
        }
    }
    /// the slot among the shared keys of `key`, in a split table
    pub fn slot_of(&self, key: &PyStr) -> Option<usize> {
        self.shared.as_ref()?.names.borrow().find_combined(key)
    }
    /// the bytes this dict has allocated, for sys.getsizeof: itself in its
    /// reference counted box, its entries and its index (a split table's
    /// keys are its class's)
    pub fn alloc_size(&self) -> usize {
        std::mem::size_of::<RefCell<Dict>>() + 2 * std::mem::size_of::<usize>() + self.entries.capacity() * std::mem::size_of::<Option<Entry>>() + self.index.capacity() * std::mem::size_of::<i32>()
    }
    /// the entry of the key in shared slot `slot`, in a split table
    #[inline]
    fn slot_entry(&self, slot: usize) -> Option<usize> {
        match self.index.get(slot) {
            Some(&ix) if ix >= 0 => Some(ix as usize),
            _ => None,
        }
    }
    /// a split table becomes combined, indexing its keys itself
    fn combine(&mut self) {
        if self.shared.take().is_some() {
            let mut size = 8;
            while size * 2 <= (self.len + 1) * 3 {
                size *= 2;
            }
            self.rebuild(size);
        }
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// mark this as a class's dict, whose keys Class::lookup caches
    pub fn watch(&mut self) {
        self.watched = true;
        types_changed();
    }

    /// keys were added, removed or moved
    fn reshaped(&self) {
        if self.watched {
            types_changed();
        }
    }

    fn rebuild(&mut self, size: usize) {
        self.reshaped();
        if self.shared.is_some() {
            // a split table drops its dead entries; the slots stay
            let mut moved = vec![-1; self.entries.len()];
            let mut live = Vec::with_capacity(self.len);
            for (i, e) in self.entries.drain(..).enumerate() {
                if e.is_some() {
                    moved[i] = live.len() as i32;
                    live.push(e);
                }
            }
            self.entries = live;
            for ix in self.index.iter_mut().filter(|ix| **ix >= 0) {
                *ix = moved[*ix as usize];
            }
            return;
        }
        let live: Vec<Option<Entry>> = self.entries.drain(..).filter(|e| e.is_some()).collect();
        self.entries = live;
        self.index = vec![-1; size];
        let mask = size - 1;
        for (i, e) in self.entries.iter().enumerate() {
            let h = e.as_ref().unwrap().hash;
            let mut slot = (h as usize) & mask;
            let mut perturb = h;
            while self.index[slot] >= 0 {
                perturb >>= 5;
                slot = (slot * 5 + 1 + perturb as usize) & mask;
            }
            self.index[slot] = i as i32;
        }
    }

    /// probe for `key`; `eq` answers equality where it can without Python
    #[inline]
    pub fn lookup(&self, hash: u64, key: &Value) -> Lookup {
        if let Some(keys) = &self.shared {
            return match keys.names.borrow().lookup(hash, key) {
                Lookup::Found(slot) => self.slot_entry(slot).map_or(Lookup::Missing, Lookup::Found),
                Lookup::Missing => Lookup::Missing,
                Lookup::Slow(slots) => {
                    let at: Vec<usize> = slots.into_iter().filter_map(|s| self.slot_entry(s)).collect();
                    if at.is_empty() {
                        Lookup::Missing
                    } else {
                        Lookup::Slow(at)
                    }
                }
            };
        }
        if self.index.is_empty() {
            return Lookup::Missing;
        }
        let mask = self.index.len() - 1;
        let mut slot = (hash as usize) & mask;
        let mut perturb = hash;
        let mut slow = Vec::new();
        loop {
            let ix = self.index[slot];
            if ix == -1 {
                break;
            }
            if ix >= 0 {
                if let Some(e) = &self.entries[ix as usize] {
                    if e.hash == hash {
                        if e.key.is(key) {
                            return Lookup::Found(ix as usize);
                        }
                        match fast_eq(&e.key, key) {
                            Some(true) => return Lookup::Found(ix as usize),
                            Some(false) => {}
                            None => slow.push(ix as usize),
                        }
                    }
                }
            }
            perturb >>= 5;
            slot = (slot * 5 + 1 + perturb as usize) & mask;
        }
        if slow.is_empty() {
            Lookup::Missing
        } else {
            Lookup::Slow(slow)
        }
    }

    #[inline]
    pub fn get_str(&self, key: &PyStr) -> Option<&Value> {
        if self.shared.is_some() {
            return self.entries[self.find_str(key)?].as_ref().map(|e| &e.value);
        }
        let ix = self.find_combined(key)?;
        // find_combined gives only live entries
        self.entries[ix].as_ref().map(|e| &e.value)
    }

    /// the value of entry `ix` when its key is `key` itself
    #[inline(always)]
    pub fn entry_is(&self, ix: u32, key: &Rc<PyStr>) -> Option<&Value> {
        match self.entries.get(ix as usize) {
            Some(Some(e)) => match &e.key {
                Value::Str(k) if Rc::ptr_eq(k, key) => Some(&e.value),
                _ => None,
            },
            _ => None,
        }
    }

    /// replaces the value of entry `ix` when its key is `key` itself,
    /// giving back the old value; else gives `v` back
    #[inline(always)]
    pub fn set_entry_if(&mut self, ix: u32, key: &Rc<PyStr>, v: Value) -> Result<Value, Value> {
        match self.entries.get_mut(ix as usize) {
            Some(Some(e)) => match &e.key {
                Value::Str(k) if Rc::ptr_eq(k, key) => Ok(std::mem::replace(&mut e.value, v)),
                _ => Err(v),
            },
            _ => Err(v),
        }
    }

    /// the entry whose key is `key` itself, for entry_is
    pub fn entry_of(&self, key: &Rc<PyStr>) -> Option<u32> {
        let ix = self.find_str(key)?;
        self.entry_is(ix as u32, key).map(|_| ix as u32)
    }

    pub fn get_str_mut(&mut self, key: &PyStr) -> Option<&mut Value> {
        let i = self.find_str(key)?;
        Some(&mut self.entries[i].as_mut().unwrap().value)
    }

    /// whether a key that is not an exact str has the hash of `key`; the
    /// str fast paths miss such keys, which may still compare equal
    pub fn str_clash(&self, key: &PyStr) -> bool {
        // a split table's keys are all exact strs
        if self.index.is_empty() || self.shared.is_some() {
            return false;
        }
        let hash = key.hash();
        let mask = self.index.len() - 1;
        let mut slot = (hash as usize) & mask;
        let mut perturb = hash;
        loop {
            let ix = self.index[slot];
            if ix == -1 {
                return false;
            }
            if ix >= 0 {
                if let Some(Some(e)) = self.entries.get(ix as usize) {
                    if e.hash == hash && !matches!(e.key, Value::Str(_)) {
                        return true;
                    }
                }
            }
            perturb >>= 5;
            slot = (slot * 5 + 1 + perturb as usize) & mask;
        }
    }

    #[inline]
    pub fn find_str(&self, key: &PyStr) -> Option<usize> {
        match &self.shared {
            Some(keys) => self.slot_entry(keys.names.borrow().find_combined(key)?),
            None => self.find_combined(key),
        }
    }

    /// find_str in a combined table
    #[inline]
    fn find_combined(&self, key: &PyStr) -> Option<usize> {
        let index = &self.index[..];
        if index.is_empty() {
            return None;
        }
        let entries = &self.entries[..];
        let hash = key.hash();
        let mask = index.len() - 1;
        let mut slot = (hash as usize) & mask;
        let mut perturb = hash;
        loop {
            // the index's length is the mask's plus one
            let ix = unsafe { *index.get_unchecked(slot) };
            if ix == -1 {
                return None;
            }
            if ix >= 0 {
                if let Some(Some(e)) = entries.get(ix as usize) {
                    if e.hash == hash {
                        if let Value::Str(s) = &e.key {
                            if std::ptr::eq(&**s, key) || str_eq_slow(s, key) {
                                return Some(ix as usize);
                            }
                        }
                    }
                }
            }
            perturb >>= 5;
            slot = (slot * 5 + 1 + perturb as usize) & mask;
        }
    }

    /// get_str for a key given as text, without making a str of it
    pub fn get_s(&self, key: &str) -> Option<Value> {
        if let Some(keys) = &self.shared {
            let slot = keys.names.borrow().find_s(key)?;
            return self.entries[self.slot_entry(slot)?].as_ref().map(|e| e.value.clone());
        }
        self.entries[self.find_s(key)?].as_ref().map(|e| e.value.clone())
    }

    /// find_str for a key given as text
    fn find_s(&self, key: &str) -> Option<usize> {
        if self.index.is_empty() {
            return None;
        }
        let hash = hash_bytes(key.as_bytes());
        let mask = self.index.len() - 1;
        let mut slot = (hash as usize) & mask;
        let mut perturb = hash;
        loop {
            let ix = self.index[slot];
            if ix == -1 {
                return None;
            }
            if ix >= 0 {
                if let Some(e) = &self.entries[ix as usize] {
                    if e.hash == hash {
                        if let Value::Str(s) = &e.key {
                            if s.wtf.is_none() && s.s == key {
                                return Some(ix as usize);
                            }
                        }
                    }
                }
            }
            perturb >>= 5;
            slot = (slot * 5 + 1 + perturb as usize) & mask;
        }
    }

    /// insert a key not yet present
    pub fn push_new(&mut self, hash: u64, key: Value, value: Value) {
        self.stamp = 0;
        if let Some(keys) = &self.shared {
            let slot = match &key {
                Value::Str(s) => keys.slot_or_add(s),
                _ => None,
            };
            let Some(slot) = slot else {
                self.combine();
                return self.push_new(hash, key, value);
            };
            if self.index.len() <= slot {
                self.index.resize(slot + 1, -1);
            }
            self.index[slot] = self.entries.len() as i32;
            self.entries.push(Some(Entry { hash, key, value }));
            self.len += 1;
            return;
        }
        self.reshaped();
        if (self.entries.len() + 1) * 3 >= self.index.len() * 2 {
            let mut size = 8;
            while size * 2 <= (self.len + 1) * 3 {
                size *= 2;
            }
            self.rebuild(size.max(8));
        }
        let mask = self.index.len() - 1;
        let mut slot = (hash as usize) & mask;
        let mut perturb = hash;
        while self.index[slot] >= 0 {
            perturb >>= 5;
            slot = (slot * 5 + 1 + perturb as usize) & mask;
        }
        // reuse a deleted slot marker (-2) or an empty one
        self.index[slot] = self.entries.len() as i32;
        self.entries.push(Some(Entry { hash, key, value }));
        self.len += 1;
    }

    pub fn set_at(&mut self, i: usize, value: Value) {
        if let Some(e) = &mut self.entries[i] {
            e.value = value;
        }
    }

    pub fn remove_at(&mut self, i: usize) -> Option<Entry> {
        let e = self.entries[i].take();
        if e.is_some() {
            self.reshaped();
            self.len -= 1;
            if self.shared.is_some() {
                if let Some(ix) = self.index.iter_mut().find(|ix| **ix == i as i32) {
                    *ix = -1;
                }
            }
            // the index slot keeps pointing at a dead entry, which probes skip
            let dead = self.entries.len() - self.len;
            if dead > 16 && dead > self.len {
                let mut size = 8;
                while size * 2 <= self.len * 3 {
                    size *= 2;
                }
                self.rebuild(size);
            }
        }
        e
    }

    pub fn set_str(&mut self, key: Rc<PyStr>, value: Value) {
        if let Some(i) = self.find_str(&key) {
            self.set_at(i, value);
        } else {
            let h = key.hash();
            self.push_new(h, Value::Str(key), value);
        }
    }

    /// set_s with the key interned, for the names of built-in classes
    /// and modules, which lookups then find by identity
    pub fn set_i(&mut self, key: &str, value: Value) {
        self.set_str(intern(key), value);
    }

    /// makes the str keys the interned strs of their text
    pub fn intern_keys(&mut self) {
        if self.shared.is_some() {
            return;
        }
        for e in self.entries.iter_mut().flatten() {
            if let Value::Str(k) = &e.key {
                if k.wtf.is_none() {
                    let i = intern_str(k);
                    if !Rc::ptr_eq(&i, k) {
                        e.key = Value::Str(i);
                    }
                }
            }
        }
    }

    pub fn set_s(&mut self, key: &str, value: Value) {
        self.set_str(Rc::new(PyStr::new(key.to_string())), value);
    }

    pub fn remove_str(&mut self, key: &PyStr) -> Option<Value> {
        let i = self.find_str(key)?;
        self.remove_at(i).map(|e| e.value)
    }

    /// empties the dict; a split table stays one, as CPython's clears the
    /// inline values in place
    pub fn clear(&mut self) {
        self.reshaped();
        self.entries.clear();
        if self.shared.is_some() {
            self.index.fill(-1);
        } else {
            self.index.clear();
        }
        self.len = 0;
    }

    pub fn iter(&self) -> impl Iterator<Item = &Entry> {
        self.entries.iter().filter_map(|e| e.as_ref())
    }

    pub fn keys(&self) -> Vec<Value> {
        self.iter().map(|e| e.key.clone()).collect()
    }
    pub fn values(&self) -> Vec<Value> {
        self.iter().map(|e| e.value.clone()).collect()
    }
    pub fn items(&self) -> Vec<(Value, Value)> {
        self.iter().map(|e| (e.key.clone(), e.value.clone())).collect()
    }

    /// entry at position `pos` or later that is alive, for iteration
    /// the live entry before `pos`, for reversed iteration
    pub fn prev_from(&self, pos: usize) -> Option<(usize, &Entry)> {
        let mut q = pos.min(self.entries.len());
        while q > 0 {
            q -= 1;
            if let Some(e) = &self.entries[q] {
                return Some((q, e));
            }
        }
        None
    }

    pub fn next_from(&self, mut pos: usize) -> Option<(usize, &Entry)> {
        while pos < self.entries.len() {
            if let Some(e) = &self.entries[pos] {
                return Some((pos, e));
            }
            pos += 1;
        }
        None
    }

    pub fn copy(&self) -> Dict {
        let mut d = Dict::with_capacity(self.len);
        for e in self.iter() {
            d.push_new(e.hash, e.key.clone(), e.value.clone());
        }
        d
    }

    pub fn pop_last(&mut self) -> Option<Entry> {
        self.reshaped();
        self.combine();
        while let Some(e) = self.entries.pop() {
            if let Some(e) = e {
                self.len -= 1;
                let size = self.index.len().max(8);
                self.rebuild(size);
                return Some(e);
            }
        }
        None
    }
}

// -------------------------------------------------------------------------
// hashing and equality that need no Python code

const MODULUS: u64 = (1 << 61) - 1;

pub fn hash_int(v: i64) -> u64 {
    let m = (v.unsigned_abs() % MODULUS) as i64;
    let h = if v < 0 { -m } else { m };
    let h = if h == -1 { -2 } else { h };
    h as u64
}

pub fn hash_float(f: f64) -> u64 {
    // CPython's _Py_HashDouble: the value as a fraction mod 2^61-1, so equal
    // ints, floats and Fractions hash alike
    if f.is_nan() {
        // as object.__hash__: by identity, which for a float is its bits
        return f.to_bits() >> 4;
    }
    if f.is_infinite() {
        return if f > 0.0 { 314159 } else { (-314159i64) as u64 };
    }
    let (mut m, mut e) = frexp(f);
    let mut sign = 1i64;
    if m < 0.0 {
        sign = -1;
        m = -m;
    }
    let mut x: u64 = 0;
    while m != 0.0 {
        x = ((x << 28) & MODULUS) | (x >> (61 - 28));
        m *= 268435456.0;
        e -= 28;
        let y = m as u64;
        m -= y as f64;
        x += y;
        if x >= MODULUS {
            x -= MODULUS;
        }
    }
    let e = if e >= 0 { e % 61 } else { 61 - 1 - ((-1 - e) % 61) };
    let x = ((x << e) & MODULUS) | (x >> (61 - e));
    let h = (x as i64) * sign;
    (if h == -1 { -2 } else { h }) as u64
}

/// mantissa in [0.5, 1) and exponent
pub fn frexp(f: f64) -> (f64, i32) {
    if f == 0.0 || !f.is_finite() {
        return (f, 0);
    }
    let bits = f.to_bits();
    let exp = ((bits >> 52) & 0x7ff) as i32;
    if exp == 0 {
        // subnormal: scale up first
        let (m, e) = frexp(f * 2f64.powi(64));
        return (m, e - 64);
    }
    let m = f64::from_bits((bits & !(0x7ffu64 << 52)) | (1022u64 << 52));
    (m, exp - 1022)
}

pub fn hash_big(b: &BigInt) -> u64 {
    // value mod 2^61-1, sign kept
    let mut acc: u128 = 0;
    for &limb in b.mag.iter().rev() {
        acc = ((acc << 32) | limb as u128) % MODULUS as u128;
    }
    let h = acc as i64;
    let h = if b.neg { -h } else { h };
    (if h == -1 { -2 } else { h }) as u64
}

/// as CPython's frozenset_hash: order independent, dispersed for nesting
pub fn frozenset_hash(d: &Dict) -> u64 {
    let shuffle = |h: u64| ((h ^ 89869747) ^ (h << 16)).wrapping_mul(3644798167);
    let mut h: u64 = 0;
    for e in d.iter() {
        h ^= shuffle(e.hash);
    }
    h ^= (d.len() as u64 + 1).wrapping_mul(1927868237);
    h ^= (h >> 11) ^ (h >> 25);
    h = h.wrapping_mul(69069).wrapping_add(907133923);
    if h == u64::MAX {
        590923713
    } else {
        h
    }
}

/// hash for values whose hash needs no Python code; None otherwise
pub fn fast_hash(v: &Value) -> Option<u64> {
    Some(match v {
        Value::None => 0x5f3759df,
        Value::Bool(b) => *b as u64,
        Value::Int(i) => hash_int(*i),
        Value::Big(b) => hash_big(b),
        Value::Float(f) => hash_float(*f),
        Value::Str(s) => s.hash(),
        Value::Bytes(b) => hash_bytes(b),
        Value::Tuple(items) => {
            // xxHash-style combination as CPython does
            let mut acc: u64 = 0x27D4EB2F165667C5;
            for it in items.iter() {
                let lane = fast_hash(it)?;
                acc = acc.wrapping_add(lane.wrapping_mul(0xC2B2AE3D27D4EB4F));
                acc = acc.rotate_left(31);
                acc = acc.wrapping_mul(0x9E3779B185EBCA87);
            }
            let acc = acc.wrapping_add((items.len() as u64) ^ (0x27D4EB2F165667C5 ^ 3527539));
            if acc == u64::MAX {
                1546275796
            } else {
                acc
            }
        }
        Value::FrozenSet(d) => frozenset_hash(&d.borrow()),
        Value::Complex(c) => hash_float(c.0) ^ hash_float(c.1).wrapping_mul(1000003),
        Value::Range(_) | Value::BigRange(_) => crate::range::hash(v),
        Value::Instance(_) | Value::Obj(_) => return None,
        Value::List(_) | Value::Dict(_) | Value::Set(_) | Value::ByteArray(_) => return None,
        // a bound method hashes by the identity of its self
        Value::Method(m) => ((ptr_id(&m.0) as u64) >> 4) ^ fast_hash(&m.1)?,
        Value::Slice(sl) => {
            let mut acc: u64 = 0x27D4EB2F165667C5;
            for it in [&sl.0, &sl.1, &sl.2] {
                acc = acc.wrapping_add(fast_hash(it)?.wrapping_mul(0xC2B2AE3D27D4EB4F)).rotate_left(31).wrapping_mul(0x9E3779B185EBCA87);
            }
            acc
        }
        other => (ptr_id(other) as u64) >> 4,
    })
}

/// how many references share `v`'s allocation, None for values without one
pub fn strong_count(v: &Value) -> Option<usize> {
    Some(match v {
        Value::Str(r) => Rc::strong_count(r),
        Value::List(r) => Rc::strong_count(r),
        Value::Tuple(r) => Rc::strong_count(r),
        Value::Dict(r) | Value::Set(r) | Value::FrozenSet(r) => Rc::strong_count(r),
        Value::Func(r) => Rc::strong_count(r),
        Value::Native(r) => Rc::strong_count(r),
        Value::Method(r) => Rc::strong_count(r),
        Value::Class(r) => Rc::strong_count(r),
        Value::Instance(r) => Rc::strong_count(r),
        Value::Module(r) => Rc::strong_count(r),
        Value::Iter(r) => Rc::strong_count(r),
        Value::Gen(r) => Rc::strong_count(r),
        Value::Big(r) => Rc::strong_count(r),
        Value::Bytes(r) => Rc::strong_count(r),
        Value::ByteArray(r) => Rc::strong_count(r),
        Value::Range(r) => Rc::strong_count(r),
        Value::BigRange(r) => Rc::strong_count(r),
        Value::Slice(r) => Rc::strong_count(r),
        Value::DictView(r) => Rc::strong_count(r),
        Value::Property(r) => Rc::strong_count(r),
        Value::StaticMethod(r) | Value::ClassMethod(r) => Rc::strong_count(r),
        Value::Super(r) => Rc::strong_count(r),
        Value::Cell(r) => Rc::strong_count(r),
        Value::Code(r) => Rc::strong_count(r),
        Value::Obj(r) => Rc::strong_count(r),
        Value::Complex(r) => Rc::strong_count(r),
        _ => return None,
    })
}

pub fn ptr_id(v: &Value) -> usize {
    match v {
        Value::Str(s) => Rc::as_ptr(s) as *const u8 as usize,
        Value::List(r) => Rc::as_ptr(r) as *const u8 as usize,
        Value::Tuple(r) => Rc::as_ptr(r) as *const u8 as usize,
        Value::Dict(r) | Value::Set(r) | Value::FrozenSet(r) => Rc::as_ptr(r) as *const u8 as usize,
        Value::Func(r) => Rc::as_ptr(r) as *const u8 as usize,
        Value::Native(r) => Rc::as_ptr(r) as *const u8 as usize,
        Value::Method(r) => Rc::as_ptr(r) as *const u8 as usize,
        Value::Class(r) => Rc::as_ptr(r) as *const u8 as usize,
        Value::Instance(r) => Rc::as_ptr(r) as *const u8 as usize,
        Value::Module(r) => Rc::as_ptr(r) as *const u8 as usize,
        Value::Iter(r) => Rc::as_ptr(r) as *const u8 as usize,
        Value::Gen(r) => Rc::as_ptr(r) as *const u8 as usize,
        Value::Big(r) => Rc::as_ptr(r) as *const u8 as usize,
        Value::Bytes(r) => Rc::as_ptr(r) as *const u8 as usize,
        Value::ByteArray(r) => Rc::as_ptr(r) as *const u8 as usize,
        Value::Range(r) => Rc::as_ptr(r) as *const u8 as usize,
        Value::BigRange(r) => Rc::as_ptr(r) as *const u8 as usize,
        Value::Slice(r) => Rc::as_ptr(r) as *const u8 as usize,
        Value::DictView(r) => Rc::as_ptr(r) as *const u8 as usize,
        Value::Property(r) => Rc::as_ptr(r) as *const u8 as usize,
        Value::StaticMethod(r) | Value::ClassMethod(r) => Rc::as_ptr(r) as *const u8 as usize,
        Value::Super(r) => Rc::as_ptr(r) as *const u8 as usize,
        Value::Cell(r) => Rc::as_ptr(r) as *const u8 as usize,
        Value::Code(r) => Rc::as_ptr(r) as *const u8 as usize,
        Value::Obj(r) => Rc::as_ptr(r) as *const u8 as usize,
        Value::Complex(r) => Rc::as_ptr(r) as *const u8 as usize,
        Value::None => 0x1000,
        Value::Bool(b) => 0x2000 + *b as usize * 16,
        Value::NotImplemented => 0x3000,
        Value::Ellipsis => 0x4000,
        Value::Undef => 0x5000,
        Value::Int(i) => 0x10_0000 + (*i as usize).wrapping_mul(32),
        Value::Float(f) => f.to_bits() as usize,
    }
}

/// equality for values where no Python method can be involved
/// the metaclass of c when it is a subclass of type rather than type itself
pub fn user_meta(c: &Class) -> Option<Rc<Class>> {
    c.meta.borrow().as_ref().filter(|m| m.mro.borrow().len() > 2).cloned()
}

pub fn fast_eq(a: &Value, b: &Value) -> Option<bool> {
    use Value::*;
    Some(match (a, b) {
        (Int(x), Int(y)) => x == y,
        (Str(x), Str(y)) => Rc::ptr_eq(x, y) || x.same(y),
        (None, None) => true,
        (Bool(x), Bool(y)) => x == y,
        (Bool(x), Int(y)) | (Int(y), Bool(x)) => (*x as i64) == *y,
        (Float(x), Float(y)) => x == y,
        (Int(x), Float(y)) | (Float(y), Int(x)) => crate::bigint::cmp_i64_f64(*x, *y) == Some(std::cmp::Ordering::Equal),
        (Bool(x), Float(y)) | (Float(y), Bool(x)) => (*x as i64 as f64) == *y,
        (Big(x), Big(y)) => x == y,
        (Big(x), Float(y)) | (Float(y), Big(x)) => x.cmp_f64(*y) == Some(std::cmp::Ordering::Equal),
        (Big(_), Int(_)) | (Int(_), Big(_)) => false,
        (Tuple(x), Tuple(y)) => {
            if Rc::ptr_eq(x, y) {
                return Some(true);
            }
            if x.len() != y.len() {
                return Some(false);
            }
            for (p, q) in x.iter().zip(y.iter()) {
                if !p.is(q) && !fast_eq(p, q)? {
                    return Some(false);
                }
            }
            true
        }
        (Bytes(x), Bytes(y)) => x == y,
        (ByteArray(x), ByteArray(y)) => Rc::ptr_eq(x, y) || *x.borrow() == *y.borrow(),
        (ByteArray(x), Bytes(y)) | (Bytes(y), ByteArray(x)) => **y == *x.borrow(),
        (Instance(x), Instance(y)) if Rc::ptr_eq(x, y) => true,
        // classes whose metaclass may define __eq__
        (Class(x), Class(y)) if !Rc::ptr_eq(x, y) && (user_meta(x).is_some() || user_meta(y).is_some()) => return Option::None,
        (Instance(_), _) | (_, Instance(_)) | (Obj(_), _) | (_, Obj(_)) => return Option::None,
        (List(_), List(_)) | (Slice(_), Slice(_)) | (Method(_), Method(_)) | (Dict(_), Dict(_)) | (Set(_), Set(_)) | (FrozenSet(_), FrozenSet(_)) | (Set(_), FrozenSet(_)) | (FrozenSet(_), Set(_)) => return Option::None,
        (DictView(_), _) | (_, DictView(_)) => return Option::None,
        (Complex(x), Complex(y)) => x.0 == y.0 && x.1 == y.1,
        (Complex(c), o @ (Int(_) | Bool(_) | Float(_) | Big(_))) | (o @ (Int(_) | Bool(_) | Float(_) | Big(_)), Complex(c)) => c.1 == 0.0 && fast_eq(&Float(c.0), o)?,
        (Range(_) | BigRange(_), Range(_) | BigRange(_)) => crate::range::eq(a, b),
        (Str(_), _) | (_, Str(_)) => false,
        (Int(_), _) | (_, Int(_)) | (Float(_), _) | (_, Float(_)) => false,
        _ => ptr_id(a) == ptr_id(b) && std::mem::discriminant(a) == std::mem::discriminant(b),
    })
}

// -------------------------------------------------------------------------
// constructors

impl Value {
    #[inline]
    pub fn str(s: &str) -> Value {
        Value::Str(Rc::new(PyStr::new(s.to_string())))
    }
    #[inline]
    pub fn string(s: String) -> Value {
        Value::Str(Rc::new(PyStr::new(s)))
    }
    pub fn list(v: Vec<Value>) -> Value {
        Value::List(Rc::new(RefCell::new(v)))
    }
    pub fn tuple(v: Vec<Value>) -> Value {
        Value::Tuple(Rc::new(v))
    }
    pub fn dict(d: Dict) -> Value {
        Value::Dict(Rc::new(RefCell::new(d)))
    }
    pub fn from_big(b: BigInt) -> Value {
        match b.to_i64() {
            Some(i) => Value::Int(i),
            None => Value::Big(Rc::new(b)),
        }
    }
    pub fn is_none(&self) -> bool {
        matches!(self, Value::None)
    }
    pub fn is_undef(&self) -> bool {
        matches!(self, Value::Undef)
    }
    /// clone() with the values that own nothing copied inline, without
    /// the call to the derived clone
    #[inline(always)]
    pub fn fast_clone(&self) -> Value {
        match self {
            Value::Int(i) => Value::Int(*i),
            Value::Float(x) => Value::Float(*x),
            Value::Bool(b) => Value::Bool(*b),
            Value::None => Value::None,
            _ => self.clone(),
        }
    }
    /// whether dropping this value does nothing
    #[inline(always)]
    pub fn is_plain(&self) -> bool {
        matches!(self, Value::Undef | Value::None | Value::NotImplemented | Value::Ellipsis | Value::Bool(_) | Value::Int(_) | Value::Float(_))
    }
    /// drop() that skips the call to the drop glue for the values that own
    /// nothing
    #[inline(always)]
    pub fn fast_drop(self) {
        if self.is_plain() {
            std::mem::forget(self);
        } else {
            drop(self);
        }
    }
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Str(s) => Some(&s.s),
            _ => None,
        }
    }
    pub fn is(&self, o: &Value) -> bool {
        match (self, o) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a.to_bits() == b.to_bits(),
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::None, Value::None) => true,
            (Value::NotImplemented, Value::NotImplemented) => true,
            (Value::Ellipsis, Value::Ellipsis) => true,
            (Value::Str(a), Value::Str(b)) => Rc::ptr_eq(a, b) || (a.s.len() <= 1 && a.same(b)),
            (Value::Tuple(a), Value::Tuple(b)) => Rc::ptr_eq(a, b) || (a.is_empty() && b.is_empty()),
            _ => std::mem::discriminant(self) == std::mem::discriminant(o) && ptr_id(self) == ptr_id(o),
        }
    }
}

impl Class {
    pub fn new(name: &str, bases: Vec<Rc<Class>>, builtin: Builtin) -> Rc<Class> {
        let c = Rc::new(Class {
            name: RefCell::new(Rc::new(PyStr::new(name.to_string()))),
            qualname: RefCell::new(Rc::new(PyStr::new(name.to_string()))),
            bases: RefCell::new(bases),
            mro: RefCell::new(vec![]),
            dict: Rc::new(RefCell::new(Dict::new())),
            builtin: Cell::new(builtin),
            meta: RefCell::new(None),
            has_getattr: Cell::new(false),
            has_getattribute: Cell::new(false),
            has_setattr: Cell::new(false),
            has_descriptors: Cell::new(false),
            has_del: Cell::new(false),
            immutable: Cell::new(false),
            slots: RefCell::new(None),
            shared_keys: RefCell::new(None),
            id: NEXT_CLASS_ID.with(|n| {
                n.set(n.get() + 1);
                n.get()
            }),
        });
        c.dict.borrow_mut().watch();
        crate::gc::track(crate::gc::Tracked::Class(Rc::downgrade(&c)));
        c
    }

    /// replace the MRO, which the lookup cache depends on
    pub fn set_mro(&self, mro: Vec<Rc<Class>>) {
        *self.mro.borrow_mut() = mro;
        types_changed();
    }

    /// replace the dict, as class creation does with the namespace
    pub fn set_dict(&self, mut d: Dict) {
        d.watch();
        *self.dict.borrow_mut() = d;
    }
    pub fn name(&self) -> String {
        self.name.borrow().s.clone()
    }
    /// an identity no other class ever has
    pub fn id(&self) -> u64 {
        self.id
    }
    /// look a name up along the MRO. Past the class's own dict, where the
    /// name is (or that it is nowhere) is cached until a class dict gains or
    /// loses keys or an MRO changes
    #[inline]
    pub fn lookup(&self, name: &PyStr) -> Option<Value> {
        if let Some(v) = self.dict.borrow().get_str(name) {
            return Some(v.clone());
        }
        let mro = self.mro.borrow();
        if mro.len() <= 2 {
            return mro.get(1).and_then(|c| c.dict.borrow().get_str(name).cloned());
        }
        let hash = name.hash();
        let key = name.s.as_bytes();
        let cacheable = key.len() <= CACHED_NAME && name.wtf.is_none();
        let slot = ((self.id ^ hash).wrapping_mul(0x9E37_79B9_7F4A_7C15) >> (64 - LOOKUP_CACHE_BITS)) as usize;
        let version = TYPE_VERSION.with(Cell::get);
        if cacheable {
            let hit = LOOKUP_CACHE.with(|c| {
                let c = c.borrow();
                let e = &c[slot];
                (e.version == version && e.class == self.id && e.hash == hash && &e.name[..e.len as usize] == key).then_some(e.found)
            });
            if let Some(found) = hit {
                return found.and_then(|(at, i)| mro.get(at as usize)?.dict.borrow().entries.get(i as usize)?.as_ref().map(|e| e.value.clone()));
            }
        }
        let mut found = None;
        let mut value = None;
        for (at, c) in mro.iter().enumerate().skip(1) {
            let d = c.dict.borrow();
            if let Some(i) = d.find_str(name) {
                found = Some((at as u32, i as u32));
                value = d.entries[i].as_ref().map(|e| e.value.clone());
                break;
            }
        }
        if cacheable {
            LOOKUP_CACHE.with(|c| {
                let mut c = c.borrow_mut();
                let e = &mut c[slot];
                e.version = version;
                e.class = self.id;
                e.hash = hash;
                e.len = key.len() as u8;
                e.name[..key.len()].copy_from_slice(key);
                e.found = found;
            });
        }
        value
    }
    pub fn lookup_s(&self, name: &str) -> Option<Value> {
        self.lookup(&PyStr::new(name.to_string()))
    }
    pub fn is_subclass(self: &Rc<Class>, other: &Rc<Class>) -> bool {
        if Rc::ptr_eq(self, other) {
            return true;
        }
        self.mro.borrow().iter().any(|c| Rc::ptr_eq(c, other))
    }
}
