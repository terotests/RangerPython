// SPDX-License-Identifier: MIT
//! Python values: one enum, with the heap kinds behind Rc.

use crate::bigint::BigInt;
use crate::code::Code;
use std::cell::{Cell, RefCell};
use std::hash::Hasher;
use std::rc::Rc;

pub type Name = Rc<str>;

/// A Python str: the text, whether it is all ASCII (so indexing is by
/// byte), and its hash computed once.
pub struct PyStr {
    pub s: String,
    pub ascii: bool,
    hash: Cell<u64>,
}

impl PyStr {
    pub fn new(s: String) -> PyStr {
        let ascii = s.is_ascii();
        PyStr { s, ascii, hash: Cell::new(0) }
    }
    pub fn hash(&self) -> u64 {
        let h = self.hash.get();
        if h != 0 {
            return h;
        }
        let mut f = FxHasher::default();
        f.write(self.s.as_bytes());
        // already reduced as hash_int would, so an object whose __hash__
        // returns hash(some_str) collides with that str
        let h = (f.finish() % MODULUS).max(1);
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
    pub doc: Value,
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
}

/// a function found on a type binds to the instance it is read from
pub fn binds(v: &Value) -> bool {
    match v {
        Value::Func(_) => true,
        Value::Native(n) => n.bind,
        _ => false,
    }
}

pub struct Function {
    pub code: Rc<Code>,
    pub globals: Rc<RefCell<Dict>>,
    pub defaults: RefCell<Vec<Value>>,
    pub kwdefaults: RefCell<Vec<(Name, Value)>>,
    pub closure: Vec<Cellv>,
    pub name: RefCell<Rc<PyStr>>,
    pub qualname: RefCell<Rc<PyStr>>,
    pub dict: RefCell<Option<Rc<RefCell<Dict>>>>,
    pub annotations: RefCell<Value>,
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
    pub slots: RefCell<Option<Vec<Name>>>,
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
    pub dict: Rc<RefCell<Dict>>,
    /// the built-in value a subclass of int, str, list, dict... wraps
    pub base: RefCell<Value>,
    /// values kept outside __dict__: __slots__ members, and an exception's
    /// __traceback__, __cause__, __context__ and __suppress_context__
    pub extra: RefCell<Dict>,
}

pub struct Module {
    pub name: Rc<PyStr>,
    pub dict: Rc<RefCell<Dict>>,
}

/// Objects implemented natively: files, StringIO, regex patterns...
pub struct NativeObj {
    pub class: Rc<Class>,
    pub kind: RefCell<ObjKind>,
}

pub enum ObjKind {
    File { path: String, mode: String, buf: String, pos: usize, closed: bool, writable: bool, binary: bool },
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
    Other(Value),
}

pub enum WeakTarget {
    Inst(std::rc::Weak<Instance>),
    Func(std::rc::Weak<Function>),
    Class(std::rc::Weak<Class>),
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
    Dict(Rc<RefCell<Dict>>, usize, u8, usize),
    Enumerate(Value, Value),
    Zip(Vec<Value>, bool),
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

pub struct Dict {
    pub entries: Vec<Option<Entry>>,
    index: Vec<i32>,
    pub len: usize,
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
        Dict { entries: Vec::new(), index: Vec::new(), len: 0 }
    }
    pub fn with_capacity(n: usize) -> Dict {
        let mut d = Dict { entries: Vec::with_capacity(n), index: Vec::new(), len: 0 };
        if n > 0 {
            let mut size = 8;
            while size * 2 < n * 3 {
                size *= 2;
            }
            d.index = vec![-1; size];
        }
        d
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn rebuild(&mut self, size: usize) {
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
        if self.index.is_empty() {
            return None;
        }
        let hash = key.hash();
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
                            if std::ptr::eq(&**s, key) || s.s == key.s {
                                return Some(&e.value);
                            }
                        }
                    }
                }
            }
            perturb >>= 5;
            slot = (slot * 5 + 1 + perturb as usize) & mask;
        }
    }

    pub fn get_str_mut(&mut self, key: &PyStr) -> Option<&mut Value> {
        let i = self.find_str(key)?;
        Some(&mut self.entries[i].as_mut().unwrap().value)
    }

    pub fn find_str(&self, key: &PyStr) -> Option<usize> {
        if self.index.is_empty() {
            return None;
        }
        let hash = key.hash();
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
                            if s.s == key.s {
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

    pub fn get_s(&self, key: &str) -> Option<Value> {
        self.get_str(&PyStr::new(key.to_string())).cloned()
    }

    /// insert a key not yet present
    pub fn push_new(&mut self, hash: u64, key: Value, value: Value) {
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
            self.len -= 1;
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

    pub fn set_s(&mut self, key: &str, value: Value) {
        self.set_str(Rc::new(PyStr::new(key.to_string())), value);
    }

    pub fn remove_str(&mut self, key: &PyStr) -> Option<Value> {
        let i = self.find_str(key)?;
        self.remove_at(i).map(|e| e.value)
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.index.clear();
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

/// hash for values whose hash needs no Python code; None otherwise
pub fn fast_hash(v: &Value) -> Option<u64> {
    Some(match v {
        Value::None => 0x5f3759df,
        Value::Bool(b) => *b as u64,
        Value::Int(i) => hash_int(*i),
        Value::Big(b) => hash_big(b),
        Value::Float(f) => hash_float(*f),
        Value::Str(s) => s.hash(),
        Value::Bytes(b) => {
            let mut h = FxHasher::default();
            h.write(b);
            h.finish()
        }
        Value::Tuple(items) => {
            // xxHash-style combination as CPython does
            let mut acc: u64 = 0x27D4EB2F165667C5;
            for it in items.iter() {
                let lane = fast_hash(it)?;
                acc = acc.wrapping_add(lane.wrapping_mul(0xC2B2AE3D27D4EB4F));
                acc = acc.rotate_left(31);
                acc = acc.wrapping_mul(0x9E3779B185EBCA87);
            }
            acc.wrapping_add((items.len() as u64) ^ (0x27D4EB2F165667C5 ^ 3527539))
        }
        Value::FrozenSet(d) => {
            let mut acc: u64 = 0;
            for e in d.borrow().iter() {
                acc ^= e.hash.wrapping_mul(0x9E3779B185EBCA87);
            }
            acc
        }
        Value::Complex(c) => hash_float(c.0) ^ hash_float(c.1).wrapping_mul(1000003),
        Value::Range(_) | Value::BigRange(_) => crate::range::hash(v),
        Value::Instance(_) => return None,
        Value::List(_) | Value::Dict(_) | Value::Set(_) | Value::ByteArray(_) => return None,
        other => (ptr_id(other) as u64) >> 4,
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
pub fn fast_eq(a: &Value, b: &Value) -> Option<bool> {
    use Value::*;
    Some(match (a, b) {
        (Int(x), Int(y)) => x == y,
        (Str(x), Str(y)) => Rc::ptr_eq(x, y) || x.s == y.s,
        (None, None) => true,
        (Bool(x), Bool(y)) => x == y,
        (Bool(x), Int(y)) | (Int(y), Bool(x)) => (*x as i64) == *y,
        (Float(x), Float(y)) => x == y,
        (Int(x), Float(y)) | (Float(y), Int(x)) => (*x as f64) == *y && (*y as i64) == *x,
        (Bool(x), Float(y)) | (Float(y), Bool(x)) => (*x as i64 as f64) == *y,
        (Big(x), Big(y)) => x == y,
        (Big(x), Float(y)) | (Float(y), Big(x)) => x.to_f64() == *y,
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
        (Instance(_), _) | (_, Instance(_)) => return Option::None,
        (List(_), List(_)) | (Dict(_), Dict(_)) | (Set(_), Set(_)) | (FrozenSet(_), FrozenSet(_)) | (Set(_), FrozenSet(_)) | (FrozenSet(_), Set(_)) => return Option::None,
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
            (Value::Str(a), Value::Str(b)) => Rc::ptr_eq(a, b) || (a.s.len() <= 1 && a.s == b.s),
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
            slots: RefCell::new(None),
        });
        c
    }
    pub fn name(&self) -> String {
        self.name.borrow().s.clone()
    }
    /// look a name up along the MRO
    #[inline]
    pub fn lookup(&self, name: &PyStr) -> Option<Value> {
        if let Some(v) = self.dict.borrow().get_str(name) {
            return Some(v.clone());
        }
        let mro = self.mro.borrow();
        for c in mro.iter().skip(1) {
            if let Some(v) = c.dict.borrow().get_str(name) {
                return Some(v.clone());
            }
        }
        None
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
