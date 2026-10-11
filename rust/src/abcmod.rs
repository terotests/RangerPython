// SPDX-License-Identifier: MIT
//! _abc, following CPython's Modules/_abc.c: the registry and the positive
//! and negative subclass caches of each ABC, held weakly and keyed by class
//! identity, so isinstance against an ABC costs one lookup once cached.

use crate::builtins::*;
use crate::modules::{module, put_fns};
use crate::value::*;
use crate::vm::Vm;
use std::cell::Cell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

thread_local! {
    /// bumped by every register() on any ABC; negative caches older than it are stale
    static INVALIDATION: Cell<u64> = const { Cell::new(0) };
}

fn token() -> u64 {
    INVALIDATION.with(|c| c.get())
}

/// classes held weakly, by identity
#[derive(Default)]
pub struct ClassSet(HashMap<usize, Weak<Class>>);

impl ClassSet {
    fn contains(&self, c: &Rc<Class>) -> bool {
        self.0.get(&(Rc::as_ptr(c) as usize)).and_then(Weak::upgrade).is_some_and(|k| Rc::ptr_eq(&k, c))
    }

    fn add(&mut self, c: &Rc<Class>) {
        self.0.insert(Rc::as_ptr(c) as usize, Rc::downgrade(c));
    }

    fn live(&self) -> Vec<Rc<Class>> {
        self.0.values().filter_map(Weak::upgrade).collect()
    }
}

/// what _abc_init attaches to an ABC as _abc_impl
#[derive(Default)]
pub struct AbcData {
    registry: ClassSet,
    cache: ClassSet,
    negative: ClassSet,
    negative_version: u64,
}

pub fn abc_module() -> Value {
    let (d, m) = module("_abc");
    put_fns(
        &d,
        &[
            ("_abc.get_cache_token", get_cache_token),
            ("_abc._abc_init", abc_init),
            ("_abc._abc_register", abc_register),
            ("_abc._abc_instancecheck", abc_instancecheck),
            ("_abc._abc_subclasscheck", abc_subclasscheck),
            ("_abc._get_dump", get_dump),
            ("_abc._reset_registry", reset_registry),
            ("_abc._reset_caches", reset_caches),
        ],
    );
    m
}

thread_local! {
    static ABC_IMPL: Rc<PyStr> = ps("_abc_impl");
}

/// the _abc_impl of `cls`, checked for its type
fn impl_of(vm: &mut Vm, cls: &Value) -> PyResult<Rc<NativeObj>> {
    let v = vm.getattr(cls, &ABC_IMPL.with(Rc::clone))?;
    match v {
        Value::Obj(o) if matches!(&*o.kind.borrow(), ObjKind::Abc(_)) => Ok(o),
        _ => Err(vm.type_error("_abc_impl is set to a wrong type")),
    }
}

fn with_data<R>(o: &NativeObj, f: impl FnOnce(&mut AbcData) -> R) -> R {
    match &mut *o.kind.borrow_mut() {
        ObjKind::Abc(d) => f(d),
        _ => unreachable!("checked by impl_of"),
    }
}

fn class_arg(v: &Value) -> Option<Rc<Class>> {
    match v {
        Value::Class(c) => Some(c.clone()),
        _ => None,
    }
}

fn is_abstract(vm: &mut Vm, v: &Value) -> PyResult<bool> {
    match vm.getattr_opt(v, &ps("__isabstractmethod__"))? {
        Some(f) => vm.truthy(&f),
        None => Ok(false),
    }
}

fn get_cache_token(_vm: &mut Vm, _a: &[Value], _k: Kw) -> PyResult<Value> {
    Ok(Value::Int(token() as i64))
}

/// _abc_init(cls): __abstractmethods__ from the namespace and the bases'
/// abstract names still unimplemented, and fresh caches
fn abc_init(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    need(vm, a, 1)?;
    let cls = &a[0];
    let Some(c) = class_arg(cls) else { return Err(vm.type_error("_abc_init() argument must be a class")) };
    let names = Rc::new(std::cell::RefCell::new(Dict::new()));
    let items = c.dict.borrow().items();
    for (k, v) in items {
        if is_abstract(vm, &v)? {
            vm.set_add(&names, k)?;
        }
    }
    let bases = c.bases.borrow().clone();
    for base in bases {
        let Some(inherited) = vm.getattr_opt(&Value::Class(base), &ps("__abstractmethods__"))? else { continue };
        let it = vm.iter(&inherited)?;
        while let Some(name) = vm.next(&it)? {
            let Value::Str(n) = &name else { continue };
            if let Some(value) = vm.getattr_opt(cls, n)? {
                if is_abstract(vm, &value)? {
                    vm.set_add(&names, name)?;
                }
            }
        }
    }
    vm.setattr(cls, &ps("__abstractmethods__"), Value::FrozenSet(names))?;
    // the pattern-matching flags an ABC declares exclude each other
    if let Some(Value::Int(f)) = c.dict.borrow().get_s("__abc_tpflags__") {
        if f & (1 << 5 | 1 << 6) == (1 << 5 | 1 << 6) {
            return Err(vm.type_error("__abc_tpflags__ cannot be both Py_TPFLAGS_SEQUENCE and Py_TPFLAGS_MAPPING"));
        }
    }
    let data = AbcData { negative_version: token(), ..AbcData::default() };
    let obj = vm.native_obj("_abc_data", ObjKind::Abc(Box::new(data)));
    vm.setattr(cls, &ps("_abc_impl"), obj)?;
    Ok(Value::None)
}

fn abc_register(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    need(vm, a, 2)?;
    let (cls, sub) = (&a[0], &a[1]);
    let Some(sc) = class_arg(sub) else { return Err(vm.type_error("Can only register classes")) };
    if issubclass_of(vm, sub, cls)? {
        return Ok(sub.clone());
    }
    // a cycle would break the algorithm below; X.register(X) was allowed above
    if issubclass_of(vm, cls, sub)? {
        return Err(vm.err("RuntimeError", "Refusing to create an inheritance cycle"));
    }
    let o = impl_of(vm, cls)?;
    with_data(&o, |d| d.registry.add(&sc));
    INVALIDATION.with(|c| c.set(c.get() + 1));
    Ok(sub.clone())
}

fn call_subclasscheck(vm: &mut Vm, cls: &Value, sub: Value) -> PyResult<Value> {
    let f = vm.getattr(cls, &vm.n.subclasscheck.clone())?;
    vm.call_vec(&f, vec![sub], &[])
}

fn abc_instancecheck(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    need(vm, a, 2)?;
    let (cls, inst) = (&a[0], &a[1]);
    let o = impl_of(vm, cls)?;
    let subclass = vm.getattr(inst, &vm.n.class.clone())?;
    if let Some(sc) = class_arg(&subclass) {
        if with_data(&o, |d| d.cache.contains(&sc)) {
            return Ok(Value::Bool(true));
        }
    }
    let subtype = vm.class_of(inst);
    if matches!(&subclass, Value::Class(sc) if Rc::ptr_eq(sc, &subtype)) {
        let now = token();
        if with_data(&o, |d| d.negative_version == now && d.negative.contains(&subtype)) {
            return Ok(Value::Bool(false));
        }
        return call_subclasscheck(vm, cls, subclass);
    }
    let r = call_subclasscheck(vm, cls, subclass)?;
    if vm.truthy(&r)? {
        return Ok(r);
    }
    call_subclasscheck(vm, cls, Value::Class(subtype))
}

fn abc_subclasscheck(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    need(vm, a, 2)?;
    let (cls, sub) = (&a[0], &a[1]);
    let Some(sc) = class_arg(sub) else { return Err(vm.type_error("issubclass() arg 1 must be a class")) };
    let o = impl_of(vm, cls)?;
    if with_data(&o, |d| d.cache.contains(&sc)) {
        return Ok(Value::Bool(true));
    }
    let now = token();
    let negative = with_data(&o, |d| {
        if d.negative_version < now {
            d.negative = ClassSet::default();
            d.negative_version = now;
            false
        } else {
            d.negative.contains(&sc)
        }
    });
    if negative {
        return Ok(Value::Bool(false));
    }
    let hook = vm.getattr(cls, &ps("__subclasshook__"))?;
    let ok = vm.call_vec(&hook, vec![sub.clone()], &[])?;
    match ok {
        Value::Bool(true) => {
            with_data(&o, |d| d.cache.add(&sc));
            return Ok(ok);
        }
        Value::Bool(false) => {
            with_data(&o, |d| d.negative.add(&sc));
            return Ok(ok);
        }
        Value::NotImplemented => {}
        _ => return Err(vm.err("AssertionError", "__subclasshook__ must return either False, True, or NotImplemented")),
    }
    let found = |vm: &mut Vm| -> PyResult<bool> {
        // a direct subclass
        if let Value::Class(c) = cls {
            if sc.mro.borrow().iter().any(|k| Rc::ptr_eq(k, c)) {
                return Ok(true);
            }
        }
        // a subclass of a registered class (recursive)
        let registry = with_data(&o, |d| d.registry.live());
        for r in registry {
            if issubclass_of(vm, sub, &Value::Class(r))? {
                return Ok(true);
            }
        }
        // a subclass of a subclass (recursive)
        let f = vm.getattr(cls, &ps("__subclasses__"))?;
        let subs = vm.call_vec(&f, vec![], &[])?;
        let it = vm.iter(&subs)?;
        while let Some(s) = vm.next(&it)? {
            if issubclass_of(vm, sub, &s)? {
                return Ok(true);
            }
        }
        Ok(false)
    };
    let yes = found(vm)?;
    with_data(&o, |d| if yes { d.cache.add(&sc) } else { d.negative.add(&sc) });
    Ok(Value::Bool(yes))
}

/// _get_dump(cls): the registry and caches as sets of weak references, and
/// the negative cache's version
fn get_dump(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    need(vm, a, 1)?;
    let o = impl_of(vm, &a[0])?;
    let (sets, version) = with_data(&o, |d| ([d.registry.live(), d.cache.live(), d.negative.live()], d.negative_version));
    let weakref = vm.import_one("weakref")?;
    let mkref = vm.getattr(&weakref, &ps("ref"))?;
    let mut out = vec![];
    for classes in sets {
        let s = Rc::new(std::cell::RefCell::new(Dict::new()));
        for c in classes {
            let r = vm.call_vec(&mkref, vec![Value::Class(c)], &[])?;
            vm.set_add(&s, r)?;
        }
        out.push(Value::Set(s));
    }
    out.push(Value::Int(version as i64));
    Ok(Value::tuple(out))
}

fn reset_registry(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    need(vm, a, 1)?;
    let o = impl_of(vm, &a[0])?;
    with_data(&o, |d| d.registry = ClassSet::default());
    Ok(Value::None)
}

fn reset_caches(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    need(vm, a, 1)?;
    let o = impl_of(vm, &a[0])?;
    with_data(&o, |d| {
        d.cache = ClassSet::default();
        d.negative = ClassSet::default();
    });
    Ok(Value::None)
}
