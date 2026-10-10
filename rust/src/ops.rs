// SPDX-License-Identifier: MIT
//! The object protocol: attributes, operators, comparison, hashing,
//! iteration, items, and str/repr.

use crate::ast::{BinOp, CmpOp, UnOp};
use crate::bigint::BigInt;
use crate::value::*;
use crate::vm::Vm;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::rc::Rc;

fn ps(s: &str) -> Rc<PyStr> {
    Rc::new(PyStr::new(s.to_string()))
}

pub fn op_symbol(op: BinOp) -> &'static str {
    match op {
        BinOp::Add => "+",
        BinOp::Sub => "-",
        BinOp::Mul => "*",
        BinOp::Div => "/",
        BinOp::FloorDiv => "//",
        BinOp::Mod => "%",
        BinOp::Pow => "** or pow()",
        BinOp::LShift => "<<",
        BinOp::RShift => ">>",
        BinOp::BitOr => "|",
        BinOp::BitXor => "^",
        BinOp::BitAnd => "&",
        BinOp::MatMul => "@",
    }
}

fn op_dunder(op: BinOp) -> (&'static str, &'static str, &'static str) {
    match op {
        BinOp::Add => ("__add__", "__radd__", "__iadd__"),
        BinOp::Sub => ("__sub__", "__rsub__", "__isub__"),
        BinOp::Mul => ("__mul__", "__rmul__", "__imul__"),
        BinOp::Div => ("__truediv__", "__rtruediv__", "__itruediv__"),
        BinOp::FloorDiv => ("__floordiv__", "__rfloordiv__", "__ifloordiv__"),
        BinOp::Mod => ("__mod__", "__rmod__", "__imod__"),
        BinOp::Pow => ("__pow__", "__rpow__", "__ipow__"),
        BinOp::LShift => ("__lshift__", "__rlshift__", "__ilshift__"),
        BinOp::RShift => ("__rshift__", "__rrshift__", "__irshift__"),
        BinOp::BitOr => ("__or__", "__ror__", "__ior__"),
        BinOp::BitXor => ("__xor__", "__rxor__", "__ixor__"),
        BinOp::BitAnd => ("__and__", "__rand__", "__iand__"),
        BinOp::MatMul => ("__matmul__", "__rmatmul__", "__imatmul__"),
    }
}

pub fn cmp_symbol(op: CmpOp) -> &'static str {
    match op {
        CmpOp::Eq => "==",
        CmpOp::Ne => "!=",
        CmpOp::Lt => "<",
        CmpOp::Le => "<=",
        CmpOp::Gt => ">",
        CmpOp::Ge => ">=",
        CmpOp::Is => "is",
        CmpOp::IsNot => "is not",
        CmpOp::In => "in",
        CmpOp::NotIn => "not in",
    }
}

fn floor_div_i(a: i64, b: i64) -> i64 {
    let q = a.wrapping_div(b);
    if (a % b != 0) && ((a < 0) != (b < 0)) {
        q - 1
    } else {
        q
    }
}

fn mod_i(a: i64, b: i64) -> i64 {
    let r = a.wrapping_rem(b);
    if r != 0 && ((r < 0) != (b < 0)) {
        r + b
    } else {
        r
    }
}

pub fn fmod_py(a: f64, b: f64) -> f64 {
    let r = a % b;
    if r != 0.0 && ((r < 0.0) != (b < 0.0)) {
        r + b
    } else if r == 0.0 {
        0.0f64.copysign(b)
    } else {
        r
    }
}

pub fn big_of(v: &Value) -> Option<BigInt> {
    match v {
        Value::Int(i) => Some(BigInt::from_i64(*i)),
        Value::Bool(b) => Some(BigInt::from_i64(*b as i64)),
        Value::Big(b) => Some((**b).clone()),
        _ => None,
    }
}

impl Vm {
    // ------------------------------------------------------------------
    // types

    pub fn class_of(&self, v: &Value) -> Rc<Class> {
        let t = &self.t;
        match v {
            Value::Undef | Value::None => t.none.clone(),
            Value::NotImplemented => t.notimpl.clone(),
            Value::Ellipsis => t.ellipsis.clone(),
            Value::Bool(_) => t.bool_.clone(),
            Value::Int(_) | Value::Big(_) => t.int.clone(),
            Value::Float(_) => t.float.clone(),
            Value::Complex(_) => t.complex.clone(),
            Value::Str(_) => t.str_.clone(),
            Value::Bytes(_) => t.bytes.clone(),
            Value::ByteArray(_) => t.bytearray.clone(),
            Value::List(_) => t.list.clone(),
            Value::Tuple(_) => t.tuple.clone(),
            Value::Dict(_) => t.dict.clone(),
            Value::Set(_) => t.set.clone(),
            Value::FrozenSet(_) => t.frozenset.clone(),
            Value::Range(_) | Value::BigRange(_) => t.range.clone(),
            Value::Slice(_) => t.slice.clone(),
            Value::Func(_) => t.function.clone(),
            Value::Native(_) => t.builtin_function.clone(),
            Value::Method(m) => {
                if matches!(m.1, Value::Native(_)) {
                    t.builtin_function.clone()
                } else {
                    t.method.clone()
                }
            }
            Value::Class(c) => c.meta.borrow().clone().unwrap_or_else(|| t.type_.clone()),
            Value::Instance(i) => i.class.borrow().clone(),
            Value::Module(_) => t.module.clone(),
            Value::Iter(it) => match &*it.borrow() {
                Iter::Enumerate(..) => t.enumerate.clone(),
                Iter::Zip(..) => t.zip.clone(),
                Iter::Map(..) => t.map.clone(),
                Iter::Filter(_, _, false) => t.filter.clone(),
                Iter::ListRev(..) | Iter::SeqRev(..) => t.reversed.clone(),
                _ => t.iterator.clone(),
            },
            Value::Gen(g) => {
                if g.borrow().coroutine {
                    t.coroutine.clone()
                } else {
                    t.generator.clone()
                }
            }
            Value::DictView(d) => match d.1 {
                0 => t.dict_keys.clone(),
                1 => t.dict_values.clone(),
                _ => t.dict_items.clone(),
            },
            Value::Property(_) => t.property.clone(),
            Value::StaticMethod(_) => t.staticmethod.clone(),
            Value::ClassMethod(_) => t.classmethod.clone(),
            Value::Super(_) => t.super_.clone(),
            Value::Cell(_) => t.cell.clone(),
            Value::Code(_) => t.code.clone(),
            Value::Obj(o) => o.class.clone(),
        }
    }

    pub fn type_name(&self, v: &Value) -> String {
        self.class_of(v).name()
    }

    /// the built-in value behind an instance of a subclass of a built-in
    #[inline]
    pub fn base_of(&self, v: &Value) -> Value {
        if let Value::Instance(i) = v {
            let b = i.base.borrow();
            if !b.is_undef() {
                return b.clone();
            }
        }
        v.clone()
    }

    pub fn isinstance(&mut self, v: &Value, cls: &Rc<Class>) -> bool {
        let c = self.class_of(v);
        if c.is_subclass(cls) {
            return true;
        }
        // bool is an int
        false
    }

    // ------------------------------------------------------------------
    // attributes

    /// a special method looked up on the type, bound to `obj`
    pub fn lookup_special(&mut self, obj: &Value, name: &Rc<PyStr>) -> Option<Value> {
        let cls = self.class_of(obj);
        let v = cls.lookup(name)?;
        Some(self.bind(v, obj, &cls))
    }

    pub fn bind(&mut self, v: Value, obj: &Value, cls: &Rc<Class>) -> Value {
        match v {
            ref f if binds(f) => Value::Method(Rc::new((obj.clone(), v))),
            Value::ClassMethod(f) => Value::Method(Rc::new((Value::Class(cls.clone()), (*f).clone()))),
            Value::StaticMethod(f) => (*f).clone(),
            other => other,
        }
    }

    pub fn getattr(&mut self, obj: &Value, name: &Rc<PyStr>) -> PyResult<Value> {
        match self.getattr_opt(obj, name)? {
            Some(v) => Ok(v),
            None => Err(self.no_attr(obj, name)),
        }
    }

    pub fn no_attr(&mut self, obj: &Value, name: &PyStr) -> Exc {
        let msg = match obj {
            Value::Module(m) => format!("module '{}' has no attribute '{}'", m.name.s, name.s),
            Value::Class(c) => format!("type object '{}' has no attribute '{}'", c.name(), name.s),
            _ => format!("'{}' object has no attribute '{}'", self.type_name(obj), name.s),
        };
        let e = self.attr_error(msg);
        if let Value::Instance(i) = &e.value {
            let mut d = i.dict.borrow_mut();
            d.set_s("name", Value::str(&name.s));
            d.set_s("obj", obj.clone());
        }
        e
    }

    pub fn getattr_opt(&mut self, obj: &Value, name: &Rc<PyStr>) -> PyResult<Option<Value>> {
        match obj {
            Value::Instance(inst) => self.instance_getattr(obj, inst, name),
            Value::Class(c) => self.class_getattr(c, name),
            Value::Module(m) => {
                if let Some(v) = m.dict.borrow().get_str(name) {
                    return Ok(Some(v.clone()));
                }
                if &*name.s == "__dict__" {
                    return Ok(Some(Value::Dict(m.dict.clone())));
                }
                if &*name.s == "__name__" {
                    return Ok(Some(Value::Str(m.name.clone())));
                }
                let ga = m.dict.borrow().get_s("__getattr__");
                if let Some(ga) = ga {
                    return match self.call_vec(&ga, vec![Value::Str(name.clone())], &[]) {
                        Ok(v) => Ok(Some(v)),
                        Err(e) if self.exc_is(&e, "AttributeError") => Ok(None),
                        Err(e) => Err(e),
                    };
                }
                Ok(None)
            }
            Value::Func(f) => {
                if let Some(d) = &*f.dict.borrow() {
                    if let Some(v) = d.borrow().get_str(name) {
                        return Ok(Some(v.clone()));
                    }
                }
                if let Some(v) = crate::builtins::func_attr(self, f, &name.s) {
                    return Ok(Some(v));
                }
                self.type_attr(obj, name)
            }
            Value::Super(s) => self.super_getattr(&s.0, &s.1, name),
            _ => {
                if let Some(v) = crate::builtins::special_attr(self, obj, &name.s)? {
                    return Ok(Some(v));
                }
                self.type_attr(obj, name)
            }
        }
    }

    /// an attribute found on the type of a built-in value
    pub fn type_attr(&mut self, obj: &Value, name: &Rc<PyStr>) -> PyResult<Option<Value>> {
        let cls = self.class_of(obj);
        match cls.lookup(name) {
            Some(Value::Property(p)) => {
                let g = p.fget.clone();
                Ok(Some(self.call_vec(&g, vec![obj.clone()], &[])?))
            }
            Some(v) => Ok(Some(self.bind(v, obj, &cls))),
            None => {
                if &*name.s == "__class__" {
                    return Ok(Some(Value::Class(cls)));
                }
                if &*name.s == "__doc__" {
                    return Ok(Some(Value::None));
                }
                Ok(None)
            }
        }
    }

    pub fn instance_getattr(&mut self, obj: &Value, inst: &Rc<Instance>, name: &Rc<PyStr>) -> PyResult<Option<Value>> {
        let cls = inst.class.borrow().clone();
        if cls.has_getattribute.get() {
            let ga = cls.lookup(&self.n.getattribute).unwrap();
            let r = self.call_vec(&ga, vec![obj.clone(), Value::Str(name.clone())], &[]);
            return match r {
                Ok(v) => Ok(Some(v)),
                Err(e) if self.exc_is(&e, "AttributeError") && cls.has_getattr.get() => {
                    let g = cls.lookup(&self.n.getattr).unwrap();
                    Ok(Some(self.call_vec(&g, vec![obj.clone(), Value::Str(name.clone())], &[])?))
                }
                Err(e) => Err(e),
            };
        }
        self.generic_getattr(obj, inst, &cls, name)
    }

    pub fn generic_getattr(&mut self, obj: &Value, inst: &Rc<Instance>, cls: &Rc<Class>, name: &Rc<PyStr>) -> PyResult<Option<Value>> {
        let tattr = cls.lookup(name);
        // data descriptors win over the instance dict
        if let Some(t) = &tattr {
            match t {
                Value::Property(p) => {
                    if p.fget.is_none() {
                        return Err(self.attr_error(format!("property '{}' of '{}' object has no getter", name.s, cls.name())));
                    }
                    let g = p.fget.clone();
                    return Ok(Some(self.call_vec(&g, vec![obj.clone()], &[])?));
                }
                Value::Instance(_) => {
                    let dcls = self.class_of(t);
                    if dcls.lookup(&self.n.set).is_some() || dcls.lookup(&self.n.delete).is_some() {
                        if let Some(g) = dcls.lookup(&self.n.get) {
                            return Ok(Some(self.call_vec(&g, vec![t.clone(), obj.clone(), Value::Class(cls.clone())], &[])?));
                        }
                    }
                }
                Value::Obj(_) if crate::tb::is_member(t) => {
                    return match crate::tb::member_read(self, t, obj) {
                        Ok(v) => Ok(Some(v)),
                        Err(e) if cls.has_getattr.get() && self.exc_is(&e, "AttributeError") => {
                            let g = cls.lookup(&self.n.getattr).unwrap();
                            Ok(Some(self.call_vec(&g, vec![obj.clone(), Value::Str(name.clone())], &[])?))
                        }
                        Err(e) => Err(e),
                    };
                }
                _ => {}
            }
        }
        if let Some(v) = inst.dict.borrow().get_str(name) {
            return Ok(Some(v.clone()));
        }
        if let Some(t) = tattr {
            return Ok(Some(match t {
                ref f if binds(f) => Value::Method(Rc::new((obj.clone(), t))),
                Value::ClassMethod(f) => Value::Method(Rc::new((Value::Class(cls.clone()), (*f).clone()))),
                Value::StaticMethod(f) => (*f).clone(),
                Value::Instance(_) | Value::Obj(_) => {
                    let dcls = self.class_of(&t);
                    match dcls.lookup(&self.n.get) {
                        Some(g) => self.call_vec(&g, vec![t.clone(), obj.clone(), Value::Class(cls.clone())], &[])?,
                        None => t,
                    }
                }
                other => other,
            }));
        }
        match &*name.s {
            "__class__" => return Ok(Some(Value::Class(cls.clone()))),
            "__dict__" if has_dict(cls) => return Ok(Some(Value::Dict(inst.dict.clone()))),
            _ => {}
        }
        if let Some(v) = crate::builtins::instance_special(self, obj, inst, &name.s)? {
            return Ok(Some(v));
        }
        if cls.has_getattr.get() {
            let g = cls.lookup(&self.n.getattr).unwrap();
            return Ok(Some(self.call_vec(&g, vec![obj.clone(), Value::Str(name.clone())], &[])?));
        }
        Ok(None)
    }

    pub fn class_getattr(&mut self, c: &Rc<Class>, name: &Rc<PyStr>) -> PyResult<Option<Value>> {
        // the metaclass's data descriptors first
        let meta = c.meta.borrow().clone().unwrap_or_else(|| self.t.type_.clone());
        let mattr = if Rc::ptr_eq(&meta, &self.t.type_) { None } else { meta.lookup(name) };
        if let Some(Value::Property(p)) = &mattr {
            let g = p.fget.clone();
            return Ok(Some(self.call_vec(&g, vec![Value::Class(c.clone())], &[])?));
        }
        // type's own data descriptors
        if matches!(&*name.s, "__name__" | "__qualname__" | "__dict__" | "__mro__" | "__bases__") {
            if let Some(v) = crate::builtins::class_special(self, c, &name.s)? {
                return Ok(Some(v));
            }
        }
        if let Some(v) = c.lookup(name) {
            return Ok(Some(match v {
                Value::ClassMethod(f) => Value::Method(Rc::new((Value::Class(c.clone()), (*f).clone()))),
                Value::StaticMethod(f) => (*f).clone(),
                Value::Instance(_) => {
                    let dcls = self.class_of(&v);
                    match dcls.lookup(&self.n.get) {
                        Some(g) => self.call_vec(&g, vec![v.clone(), Value::None, Value::Class(c.clone())], &[])?,
                        None => v,
                    }
                }
                other => other,
            }));
        }
        if let Some(v) = crate::builtins::class_special(self, c, &name.s)? {
            return Ok(Some(v));
        }
        if let Some(m) = mattr {
            return Ok(Some(self.bind(m, &Value::Class(c.clone()), &meta)));
        }
        // methods of type itself (mro(), __subclasses__...)
        if let Some(v) = self.t.type_.lookup(name) {
            return Ok(Some(self.bind(v, &Value::Class(c.clone()), &meta)));
        }
        if !Rc::ptr_eq(&meta, &self.t.type_) {
            if let Some(g) = meta.lookup(&self.n.getattr) {
                return match self.call_vec(&g, vec![Value::Class(c.clone()), Value::Str(name.clone())], &[]) {
                    Ok(v) => Ok(Some(v)),
                    Err(e) if self.exc_is(&e, "AttributeError") => Ok(None),
                    Err(e) => Err(e),
                };
            }
        }
        Ok(None)
    }

    fn super_getattr(&mut self, start: &Rc<Class>, obj: &Value, name: &Rc<PyStr>) -> PyResult<Option<Value>> {
        // super(C, cls) walks cls's own MRO; super(Meta, cls) inside a
        // metaclass walks type(cls)'s and binds like an instance
        let class_mode = matches!(obj, Value::Class(c) if c.is_subclass(start));
        let ocls = match obj {
            Value::Class(c) if class_mode => c.clone(),
            other => self.class_of(other),
        };
        let mro = ocls.mro.borrow().clone();
        let pos = mro.iter().position(|c| Rc::ptr_eq(c, start)).map(|p| p + 1).unwrap_or(mro.len());
        for c in &mro[pos..] {
            let v = c.dict.borrow().get_str(name).cloned();
            if let Some(v) = v {
                return Ok(Some(match v {
                    ref f if binds(f) => match obj {
                        Value::Class(_) if class_mode && &*name.s != "__new__" => v,
                        _ => {
                            if &*name.s == "__new__" {
                                v
                            } else {
                                Value::Method(Rc::new((obj.clone(), v)))
                            }
                        }
                    },
                    Value::ClassMethod(f) => Value::Method(Rc::new((Value::Class(ocls.clone()), (*f).clone()))),
                    Value::StaticMethod(f) => (*f).clone(),
                    Value::Property(p) => {
                        let g = p.fget.clone();
                        self.call_vec(&g, vec![obj.clone()], &[])?
                    }
                    Value::Instance(_) => {
                        let dcls = self.class_of(&v);
                        match dcls.lookup(&self.n.get) {
                            Some(g) => self.call_vec(&g, vec![v.clone(), obj.clone(), Value::Class(ocls.clone())], &[])?,
                            None => v,
                        }
                    }
                    other => other,
                }));
            }
        }
        if &*name.s == "__class__" {
            return Ok(Some(Value::Class(self.t.super_.clone())));
        }
        Ok(None)
    }

    pub fn load_method(&mut self, obj: Value, name: &Rc<PyStr>) -> PyResult<(Value, Value)> {
        // plain functions on the type are called with obj, without binding
        match &obj {
            Value::Instance(inst) => {
                let cls = inst.class.borrow().clone();
                if !cls.has_getattribute.get() {
                    if let Some(t) = cls.lookup(name) {
                        if binds(&t) && inst.dict.borrow().get_str(name).is_none() {
                            return Ok((t, obj));
                        }
                    }
                }
            }
            Value::Module(_) | Value::Class(_) | Value::Super(_) | Value::Func(_) => {}
            _ => {
                let cls = self.class_of(&obj);
                if let Some(t) = cls.lookup(name) {
                    if binds(&t) {
                        return Ok((t, obj));
                    }
                }
            }
        }
        let v = self.getattr(&obj, name)?;
        Ok((v, Value::Undef))
    }

    pub fn setattr(&mut self, obj: &Value, name: &Rc<PyStr>, v: Value) -> PyResult<()> {
        match obj {
            Value::Instance(inst) => {
                let cls = inst.class.borrow().clone();
                if cls.has_setattr.get() {
                    if let Some(sa @ Value::Func(_)) = cls.lookup(&self.n.setattr) {
                        self.call_vec(&sa, vec![obj.clone(), Value::Str(name.clone()), v], &[])?;
                        return Ok(());
                    }
                }
                self.generic_setattr(obj, inst, &cls, name, v)
            }
            Value::Class(c) => {
                let meta = c.meta.borrow().clone();
                if let Some(m) = meta {
                    if !Rc::ptr_eq(&m, &self.t.type_) {
                        if let Some(sa @ Value::Func(_)) = m.lookup(&self.n.setattr) {
                            self.call_vec(&sa, vec![obj.clone(), Value::Str(name.clone()), v], &[])?;
                            return Ok(());
                        }
                        if let Some(Value::Property(p)) = m.lookup(name) {
                            let s = p.fset.clone();
                            self.call_vec(&s, vec![obj.clone(), v], &[])?;
                            return Ok(());
                        }
                    }
                }
                self.class_setattr(c, name, v)
            }
            Value::Module(m) => {
                m.dict.borrow_mut().set_str(name.clone(), v);
                Ok(())
            }
            Value::Func(f) => {
                match &*name.s {
                    "__name__" => {
                        if let Value::Str(s) = &v {
                            *f.name.borrow_mut() = s.clone();
                        }
                        return Ok(());
                    }
                    "__qualname__" => {
                        if let Value::Str(s) = &v {
                            *f.qualname.borrow_mut() = s.clone();
                        }
                        return Ok(());
                    }
                    "__defaults__" => {
                        *f.defaults.borrow_mut() = match &v {
                            Value::Tuple(t) => t.to_vec(),
                            _ => vec![],
                        };
                        return Ok(());
                    }
                    "__kwdefaults__" => {
                        let mut kd = vec![];
                        if let Value::Dict(d) = &v {
                            for e in d.borrow().iter() {
                                if let Value::Str(s) = &e.key {
                                    kd.push((Rc::from(s.s.as_str()), e.value.clone()));
                                }
                            }
                        }
                        *f.kwdefaults.borrow_mut() = kd;
                        return Ok(());
                    }
                    "__annotations__" => {
                        *f.annotations.borrow_mut() = v;
                        return Ok(());
                    }
                    _ => {}
                }
                let mut d = f.dict.borrow_mut();
                if d.is_none() {
                    *d = Some(Rc::new(RefCell::new(Dict::new())));
                }
                d.as_ref().unwrap().borrow_mut().set_str(name.clone(), v);
                Ok(())
            }
            Value::Method(m) if matches!(m.1, Value::Func(_)) => Err(self.attr_error(format!("'method' object has no attribute '{}'", name.s))),
            _ => {
                if let Value::Obj(o) = obj {
                    if crate::tb::obj_setattr(self, o, &name.s, v.clone())? {
                        return Ok(());
                    }
                }
                let tn = self.type_name(obj);
                let has = self.class_of(obj).lookup(name).is_some();
                if has {
                    Err(self.attr_error(format!("'{}' object attribute '{}' is read-only", tn, name.s)))
                } else {
                    Err(self.attr_error(format!("'{}' object has no attribute '{}' and no __dict__ for setting new attributes", tn, name.s)))
                }
            }
        }
    }

    pub fn generic_setattr(&mut self, obj: &Value, inst: &Rc<Instance>, cls: &Rc<Class>, name: &Rc<PyStr>, v: Value) -> PyResult<()> {
        if let Some(t) = cls.lookup(name) {
            match &t {
                Value::Obj(_) if crate::tb::is_member(&t) => {
                    return crate::tb::member_write(self, &t, obj, Some(v));
                }
                Value::Property(p) => {
                    if p.fset.is_none() {
                        return Err(self.attr_error(format!("property '{}' of '{}' object has no setter", name.s, cls.name())));
                    }
                    let s = p.fset.clone();
                    self.call_vec(&s, vec![obj.clone(), v], &[])?;
                    return Ok(());
                }
                Value::Instance(_) => {
                    let dcls = self.class_of(&t);
                    if let Some(s) = dcls.lookup(&self.n.set) {
                        self.call_vec(&s, vec![t.clone(), obj.clone(), v], &[])?;
                        return Ok(());
                    }
                }
                _ => {}
            }
        }
        if let Some(slots) = &*cls.slots.borrow() {
            let allowed = slots.iter().any(|s| **s == *name.s) || slots.iter().any(|s| &**s == "__dict__");
            let all_slotted = cls.mro.borrow().iter().all(|c| c.slots.borrow().is_some() || c.builtin.get() == Builtin::Object || c.builtin.get() != Builtin::User && !Rc::ptr_eq(c, cls));
            if !allowed && all_slotted {
                let mut any = false;
                for c in cls.mro.borrow().iter() {
                    if let Some(s) = &*c.slots.borrow() {
                        if s.iter().any(|x| **x == *name.s) {
                            any = true;
                        }
                    }
                }
                if !any {
                    return Err(self.attr_error(format!("'{}' object has no attribute '{}' and no __dict__ for setting new attributes", cls.name(), name.s)));
                }
            }
        }
        if &*name.s == "__class__" {
            if let Value::Class(nc) = v {
                *inst.class.borrow_mut() = nc;
                return Ok(());
            }
            return Err(self.type_error("__class__ must be set to a class"));
        }
        if &*name.s == "__dict__" {
            if let Value::Dict(d) = &v {
                let copy = d.borrow().copy();
                *inst.dict.borrow_mut() = copy;
                return Ok(());
            }
        }
        if cls.builtin.get() == Builtin::BaseException {
            match &*name.s {
                "args" => {
                    let t = Value::tuple(self.to_vec(&v)?);
                    *inst.base.borrow_mut() = t;
                    return Ok(());
                }
                "__traceback__" => {
                    let ok = v.is_none() || matches!(&v, Value::Obj(o) if matches!(&*o.kind.borrow(), ObjKind::Traceback { .. }));
                    if !ok {
                        return Err(self.type_error("__traceback__ must be a traceback or None"));
                    }
                    inst.extra.borrow_mut().set_str(name.clone(), v);
                    return Ok(());
                }
                "__cause__" | "__context__" => {
                    let ok = v.is_none() || matches!(&v, Value::Instance(i) if i.class.borrow().is_subclass(&self.t.base_exception));
                    if !ok {
                        let what = if &*name.s == "__cause__" { "cause" } else { "context" };
                        return Err(self.type_error(format!("exception {} must be None or derive from BaseException", what)));
                    }
                    if &*name.s == "__cause__" {
                        inst.extra.borrow_mut().set_str(self.n.suppress.clone(), Value::Bool(true));
                    }
                    inst.extra.borrow_mut().set_str(name.clone(), v);
                    return Ok(());
                }
                "__suppress_context__" => {
                    let b = self.truthy(&v)?;
                    inst.extra.borrow_mut().set_str(name.clone(), Value::Bool(b));
                    return Ok(());
                }
                _ => {}
            }
        }
        inst.dict.borrow_mut().set_str(name.clone(), v);
        Ok(())
    }

    pub fn class_setattr(&mut self, c: &Rc<Class>, name: &Rc<PyStr>, v: Value) -> PyResult<()> {
        if c.builtin.get() != Builtin::User && c.builtin.get() != Builtin::BaseException && c.mro.borrow().len() <= 2 && !c.dict.borrow().get_s("__module__").is_some() {
            return Err(self.type_error(format!("cannot set '{}' attribute of immutable type '{}'", name.s, c.name())));
        }
        match &*name.s {
            "__name__" => {
                if let Value::Str(s) = &v {
                    *c.name.borrow_mut() = s.clone();
                }
                return Ok(());
            }
            "__qualname__" => {
                if let Value::Str(s) = &v {
                    *c.qualname.borrow_mut() = s.clone();
                }
                return Ok(());
            }
            "__bases__" => {
                let bs: Vec<Rc<Class>> = self.to_vec(&v)?.into_iter().filter_map(|b| if let Value::Class(c) = b { Some(c) } else { None }).collect();
                *c.bases.borrow_mut() = bs;
                self.compute_mro(c)?;
                return Ok(());
            }
            _ => {}
        }
        c.dict.borrow_mut().set_str(name.clone(), v);
        self.update_flags(c);
        Ok(())
    }

    pub fn delattr(&mut self, obj: &Value, name: &Rc<PyStr>) -> PyResult<()> {
        match obj {
            Value::Instance(inst) => {
                let cls = inst.class.borrow().clone();
                if cls.has_setattr.get() {
                    if let Some(da @ Value::Func(_)) = cls.lookup(&self.n.delattr) {
                        self.call_vec(&da, vec![obj.clone(), Value::Str(name.clone())], &[])?;
                        return Ok(());
                    }
                }
                if let Some(t) = cls.lookup(name) {
                    match &t {
                        Value::Obj(_) if crate::tb::is_member(&t) => {
                            return crate::tb::member_write(self, &t, obj, None);
                        }
                        Value::Property(p) => {
                            if p.fdel.is_none() {
                                return Err(self.attr_error(format!("property '{}' of '{}' object has no deleter", name.s, cls.name())));
                            }
                            let d = p.fdel.clone();
                            self.call_vec(&d, vec![obj.clone()], &[])?;
                            return Ok(());
                        }
                        Value::Instance(_) => {
                            let dcls = self.class_of(&t);
                            if let Some(d) = dcls.lookup(&self.n.delete) {
                                self.call_vec(&d, vec![t.clone(), obj.clone()], &[])?;
                                return Ok(());
                            }
                        }
                        _ => {}
                    }
                }
                if inst.dict.borrow_mut().remove_str(name).is_none() {
                    return Err(self.no_attr(obj, name));
                }
                Ok(())
            }
            Value::Class(c) => {
                if c.dict.borrow_mut().remove_str(name).is_none() {
                    return Err(self.no_attr(obj, name));
                }
                self.update_flags(c);
                Ok(())
            }
            Value::Module(m) => {
                if m.dict.borrow_mut().remove_str(name).is_none() {
                    return Err(self.no_attr(obj, name));
                }
                Ok(())
            }
            Value::Func(f) => {
                if let Some(d) = &*f.dict.borrow() {
                    if d.borrow_mut().remove_str(name).is_some() {
                        return Ok(());
                    }
                }
                Err(self.no_attr(obj, name))
            }
            _ => Err(self.no_attr(obj, name)),
        }
    }

    // ------------------------------------------------------------------
    // numbers

    pub fn to_f64(&mut self, v: &Value) -> PyResult<f64> {
        match v {
            Value::Float(f) => Ok(*f),
            Value::Int(i) => Ok(*i as f64),
            Value::Bool(b) => Ok(*b as i64 as f64),
            Value::Big(b) => {
                let f = b.to_f64();
                if f.is_infinite() {
                    return Err(self.err("OverflowError", "int too large to convert to float"));
                }
                Ok(f)
            }
            Value::Instance(_) => {
                let b = self.base_of(v);
                if matches!(v, Value::Instance(_)) && !matches!(b, Value::Instance(_)) {
                    return self.to_f64(&b);
                }
                match self.lookup_special(v, &self.n.float_.clone()) {
                    Some(f) => {
                        let r = self.call_vec(&f, vec![], &[])?;
                        self.to_f64(&r)
                    }
                    None => match self.lookup_special(v, &self.n.index.clone()) {
                        Some(f) => {
                            let r = self.call_vec(&f, vec![], &[])?;
                            self.to_f64(&r)
                        }
                        None => {
                            let tn = self.type_name(v);
                            Err(self.type_error(format!("must be real number, not {}", tn)))
                        }
                    },
                }
            }
            _ => {
                let tn = self.type_name(v);
                Err(self.type_error(format!("must be real number, not {}", tn)))
            }
        }
    }

    /// an index (int, bool, or __index__)
    pub fn to_index(&mut self, v: &Value) -> PyResult<i64> {
        match v {
            Value::Int(i) => Ok(*i),
            Value::Bool(b) => Ok(*b as i64),
            Value::Big(_) => Err(self.err("OverflowError", "cannot fit 'int' into an index-sized integer")),
            Value::Instance(_) => {
                let b = self.base_of(v);
                if let Value::Int(i) = b {
                    return Ok(i);
                }
                match self.lookup_special(v, &self.n.index.clone()) {
                    Some(f) => {
                        let r = self.call_vec(&f, vec![], &[])?;
                        self.to_index(&r)
                    }
                    None => {
                        let tn = self.type_name(v);
                        Err(self.type_error(format!("'{}' object cannot be interpreted as an integer", tn)))
                    }
                }
            }
            _ => {
                let tn = self.type_name(v);
                Err(self.type_error(format!("'{}' object cannot be interpreted as an integer", tn)))
            }
        }
    }

    pub fn int_binop(&mut self, op: BinOp, x: i64, y: i64) -> PyResult<Value> {
        Ok(match op {
            BinOp::Add => match x.checked_add(y) {
                Some(v) => Value::Int(v),
                None => Value::from_big(BigInt::from_i64(x).add(&BigInt::from_i64(y))),
            },
            BinOp::Sub => match x.checked_sub(y) {
                Some(v) => Value::Int(v),
                None => Value::from_big(BigInt::from_i64(x).sub(&BigInt::from_i64(y))),
            },
            BinOp::Mul => match x.checked_mul(y) {
                Some(v) => Value::Int(v),
                None => Value::from_big(BigInt::from_i64(x).mul(&BigInt::from_i64(y))),
            },
            BinOp::Div => {
                if y == 0 {
                    return Err(self.err("ZeroDivisionError", "division by zero"));
                }
                Value::Float(x as f64 / y as f64)
            }
            BinOp::FloorDiv => {
                if y == 0 {
                    return Err(self.err("ZeroDivisionError", "integer division or modulo by zero"));
                }
                if x == i64::MIN && y == -1 {
                    return Ok(Value::from_big(BigInt::from_i64(x).neg()));
                }
                Value::Int(floor_div_i(x, y))
            }
            BinOp::Mod => {
                if y == 0 {
                    return Err(self.err("ZeroDivisionError", "integer modulo by zero"));
                }
                if y == -1 {
                    return Ok(Value::Int(0));
                }
                Value::Int(mod_i(x, y))
            }
            BinOp::Pow => {
                if y < 0 {
                    if x == 0 {
                        return Err(self.err("ZeroDivisionError", "zero to a negative power"));
                    }
                    return Ok(Value::Float((x as f64).powf(y as f64)));
                }
                match (y <= u32::MAX as i64).then(|| x.checked_pow(y as u32)).flatten() {
                    Some(v) => Value::Int(v),
                    None => {
                        if y > 100_000_000 {
                            return Err(self.err("OverflowError", "exponent too large"));
                        }
                        Value::from_big(BigInt::from_i64(x).pow(y as u64))
                    }
                }
            }
            BinOp::LShift => {
                if y < 0 {
                    return Err(self.value_error("negative shift count"));
                }
                if y < 63 && (x.unsigned_abs() >> (63 - y)) == 0 {
                    Value::Int(x << y)
                } else {
                    Value::from_big(BigInt::from_i64(x).shl(y as usize))
                }
            }
            BinOp::RShift => {
                if y < 0 {
                    return Err(self.value_error("negative shift count"));
                }
                Value::Int(if y >= 64 { if x < 0 { -1 } else { 0 } } else { x >> y })
            }
            BinOp::BitOr => Value::Int(x | y),
            BinOp::BitXor => Value::Int(x ^ y),
            BinOp::BitAnd => Value::Int(x & y),
            BinOp::MatMul => return Err(self.binop_error(op, &Value::Int(x), &Value::Int(y))),
        })
    }

    fn big_binop(&mut self, op: BinOp, a: &BigInt, b: &BigInt) -> PyResult<Value> {
        Ok(match op {
            BinOp::Add => Value::from_big(a.add(b)),
            BinOp::Sub => Value::from_big(a.sub(b)),
            BinOp::Mul => Value::from_big(a.mul(b)),
            BinOp::Div => {
                if b.is_zero() {
                    return Err(self.err("ZeroDivisionError", "division by zero"));
                }
                // exact enough: divide after scaling
                let (q, r) = a.divmod(b).unwrap();
                let f = q.to_f64() + r.to_f64() / b.to_f64();
                if f.is_infinite() {
                    return Err(self.err("OverflowError", "integer division result too large for a float"));
                }
                Value::Float(f)
            }
            BinOp::FloorDiv => match a.divmod(b) {
                Some((q, _)) => Value::from_big(q),
                None => return Err(self.err("ZeroDivisionError", "integer division or modulo by zero")),
            },
            BinOp::Mod => match a.divmod(b) {
                Some((_, r)) => Value::from_big(r),
                None => return Err(self.err("ZeroDivisionError", "integer modulo by zero")),
            },
            BinOp::Pow => {
                if b.neg {
                    return Ok(Value::Float(a.to_f64().powf(b.to_f64())));
                }
                match b.to_i64() {
                    Some(e) if e <= 100_000_000 => Value::from_big(a.pow(e as u64)),
                    _ => return Err(self.err("OverflowError", "exponent too large")),
                }
            }
            BinOp::LShift => match b.to_i64() {
                Some(n) if n >= 0 => Value::from_big(a.shl(n as usize)),
                _ => return Err(self.value_error("negative shift count")),
            },
            BinOp::RShift => match b.to_i64() {
                Some(n) if n >= 0 => Value::from_big(a.shr(n as usize)),
                Some(_) => return Err(self.value_error("negative shift count")),
                None => Value::Int(if a.neg { -1 } else { 0 }),
            },
            BinOp::BitOr => Value::from_big(a.bitop(b, '|')),
            BinOp::BitXor => Value::from_big(a.bitop(b, '^')),
            BinOp::BitAnd => Value::from_big(a.bitop(b, '&')),
            BinOp::MatMul => return Err(self.type_error("unsupported operand type(s) for @")),
        })
    }

    fn float_binop(&mut self, op: BinOp, x: f64, y: f64) -> PyResult<Value> {
        Ok(Value::Float(match op {
            BinOp::Add => x + y,
            BinOp::Sub => x - y,
            BinOp::Mul => x * y,
            BinOp::Div => {
                if y == 0.0 {
                    return Err(self.err("ZeroDivisionError", "float division by zero"));
                }
                x / y
            }
            BinOp::FloorDiv => {
                if y == 0.0 {
                    return Err(self.err("ZeroDivisionError", "float floor division by zero"));
                }
                (x / y).floor()
            }
            BinOp::Mod => {
                if y == 0.0 {
                    return Err(self.err("ZeroDivisionError", "float modulo by zero"));
                }
                fmod_py(x, y)
            }
            BinOp::Pow => {
                // C99 pow has CPython's special cases for nan and infinite exponents
                if x.is_nan() || !y.is_finite() {
                    return Ok(Value::Float(x.powf(y)));
                }
                if x == 0.0 && y < 0.0 {
                    return Err(self.err("ZeroDivisionError", "zero to a negative power"));
                }
                if x < 0.0 && x.is_finite() && y.is_finite() && y.fract() != 0.0 {
                    // a complex result
                    let r = x.abs().powf(y);
                    let th = std::f64::consts::PI * y;
                    return Ok(Value::Complex(Rc::new((r * th.cos(), r * th.sin()))));
                }
                let r = x.powf(y);
                if r.is_infinite() && x.is_finite() && y.is_finite() {
                    return Err(self.err("OverflowError", "(34, 'Numerical result out of range')"));
                }
                r
            }
            _ => {
                return Err(self.binop_error(op, &Value::Float(x), &Value::Float(y)));
            }
        }))
    }

    pub fn binop_error(&mut self, op: BinOp, a: &Value, b: &Value) -> Exc {
        let (an, bn) = (self.type_name(a), self.type_name(b));
        if op == BinOp::Add && matches!(a, Value::Str(_)) {
            return self.type_error(format!("can only concatenate str (not \"{}\") to str", bn));
        }
        if op == BinOp::Add && matches!(a, Value::List(_)) && !matches!(b, Value::List(_)) {
            return self.type_error(format!("can only concatenate list (not \"{}\") to list", bn));
        }
        if op == BinOp::Add && matches!(a, Value::Tuple(_)) && !matches!(b, Value::Tuple(_)) {
            return self.type_error(format!("can only concatenate tuple (not \"{}\") to tuple", bn));
        }
        if op == BinOp::Mul && (matches!(a, Value::Str(_) | Value::List(_) | Value::Tuple(_)) || matches!(b, Value::Str(_) | Value::List(_) | Value::Tuple(_))) {
            let other = if matches!(a, Value::Str(_) | Value::List(_) | Value::Tuple(_)) { &bn } else { &an };
            return self.type_error(format!("can't multiply sequence by non-int of type '{}'", other));
        }
        self.type_error(format!("unsupported operand type(s) for {}: '{}' and '{}'", op_symbol(op), an, bn))
    }

    /// arithmetic on built-in values; None when the types do not apply
    pub fn builtin_binop(&mut self, op: BinOp, a: &Value, b: &Value) -> PyResult<Option<Value>> {
        use Value::*;
        Ok(Some(match (a, b) {
            (Int(x), Int(y)) => self.int_binop(op, *x, *y)?,
            (Bool(x), Bool(y)) if matches!(op, BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor) => Bool(match op {
                BinOp::BitAnd => *x & *y,
                BinOp::BitOr => *x | *y,
                _ => *x ^ *y,
            }),
            (Bool(x), Int(y)) => self.int_binop(op, *x as i64, *y)?,
            (Int(x), Bool(y)) => self.int_binop(op, *x, *y as i64)?,
            (Bool(x), Bool(y)) => self.int_binop(op, *x as i64, *y as i64)?,
            (Float(x), Float(y)) => self.float_binop(op, *x, *y)?,
            (Int(_) | Bool(_) | Big(_), Float(y)) => {
                if matches!(op, BinOp::LShift | BinOp::RShift | BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor | BinOp::MatMul) {
                    return Ok(Option::None);
                }
                let x = self.to_f64(a)?;
                self.float_binop(op, x, *y)?
            }
            (Float(x), Int(_) | Bool(_) | Big(_)) => {
                if matches!(op, BinOp::LShift | BinOp::RShift | BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor | BinOp::MatMul) {
                    return Ok(Option::None);
                }
                let y = self.to_f64(b)?;
                self.float_binop(op, *x, y)?
            }
            (Big(_), Int(_) | Bool(_) | Big(_)) | (Int(_) | Bool(_), Big(_)) => {
                let (x, y) = (big_of(a).unwrap(), big_of(b).unwrap());
                self.big_binop(op, &x, &y)?
            }
            (Str(_), _) if op == BinOp::Mod => Value::string(crate::format::percent_format(self, a, b)?),
            (Complex(_), _) | (_, Complex(_)) => match crate::builtins::complex_binop(self, op, a, b)? {
                Some(v) => v,
                Option::None => return Ok(Option::None),
            },
            (Str(x), Str(y)) if op == BinOp::Add => {
                let mut s = String::with_capacity(x.s.len() + y.s.len());
                s.push_str(&x.s);
                s.push_str(&y.s);
                Value::string(s)
            }
            (Str(x), Int(n)) | (Int(n), Str(x)) if op == BinOp::Mul => {
                if *n == 1 {
                    return Ok(Some(Str(x.clone())));
                }
                if *n <= 0 {
                    Value::str("")
                } else {
                    let n = self.repeat_count(x.s.len(), *n)?;
                    Value::string(x.s.repeat(n))
                }
            }
            (Str(x), Bool(n)) | (Bool(n), Str(x)) if op == BinOp::Mul => Value::string(if *n { x.s.clone() } else { String::new() }),
            (Bytes(x), Bytes(y)) if op == BinOp::Add => {
                let mut v = (**x).clone();
                v.extend_from_slice(y);
                Value::Bytes(Rc::new(v))
            }
            (Bytes(x), Int(n)) if op == BinOp::Mul => {
                let n = self.repeat_count(x.len(), *n)?;
                Value::Bytes(Rc::new(x.repeat(n)))
            }
            (Bytes(_), _) if op == BinOp::Mod => {
                let s = crate::builtins::bytes_to_latin1(a);
                let r = crate::format::bytes_percent_format(self, &Value::string(s), b)?;
                Value::Bytes(Rc::new(r.chars().map(|c| c as u32 as u8).collect()))
            }
            (ByteArray(x), _) if op == BinOp::Mod => {
                let bs = Value::Bytes(Rc::new(x.borrow().clone()));
                match self.builtin_binop(op, &bs, b)? {
                    Some(Value::Bytes(r)) => Value::ByteArray(Rc::new(RefCell::new((*r).clone()))),
                    _ => return Ok(Option::None),
                }
            }
            (List(x), List(y)) if op == BinOp::Add => {
                let mut v = x.borrow().clone();
                v.extend(y.borrow().iter().cloned());
                Value::list(v)
            }
            (List(x), Int(n)) | (Int(n), List(x)) if op == BinOp::Mul => {
                let len = x.borrow().len();
                let n = self.repeat_count(len, *n)?;
                let src = x.borrow();
                let mut v = Vec::with_capacity(len * n);
                for _ in 0..n {
                    v.extend(src.iter().cloned());
                }
                Value::list(v)
            }
            (Tuple(x), Tuple(y)) if op == BinOp::Add => {
                let mut v = (**x).clone();
                v.extend(y.iter().cloned());
                Value::tuple(v)
            }
            (Tuple(x), Int(n)) | (Int(n), Tuple(x)) if op == BinOp::Mul => {
                let n = self.repeat_count(x.len(), *n)?;
                let mut v = Vec::with_capacity(x.len() * n);
                for _ in 0..n {
                    v.extend(x.iter().cloned());
                }
                Value::tuple(v)
            }
            (Set(_) | FrozenSet(_), Set(_) | FrozenSet(_)) if matches!(op, BinOp::BitOr | BinOp::BitAnd | BinOp::Sub | BinOp::BitXor) => {
                crate::builtins::set_op(self, op, a, b)?
            }
            (DictView(x), _) if x.1 != 1 && matches!(op, BinOp::BitOr | BinOp::BitAnd | BinOp::Sub | BinOp::BitXor) => {
                let sa = crate::builtins::to_set(self, a)?;
                let sb = crate::builtins::to_set(self, b)?;
                crate::builtins::set_op(self, op, &sa, &sb)?
            }
            (Set(_) | FrozenSet(_), DictView(_)) if matches!(op, BinOp::BitOr | BinOp::BitAnd | BinOp::Sub | BinOp::BitXor) => {
                let sb = crate::builtins::to_set(self, b)?;
                crate::builtins::set_op(self, op, a, &sb)?
            }
            (Dict(x), Dict(y)) if op == BinOp::BitOr => {
                let d = Rc::new(RefCell::new(x.borrow().copy()));
                let snapshot = y.borrow().items();
                for (k, v) in snapshot {
                    self.dict_set(&d, k, v)?;
                }
                Value::Dict(d)
            }
            _ => return Ok(Option::None),
        }))
    }

    pub fn binop(&mut self, op: BinOp, a: &Value, b: &Value) -> PyResult<Value> {
        let a_inst = matches!(a, Value::Instance(_) | Value::Class(_));
        let b_inst = matches!(b, Value::Instance(_) | Value::Class(_));
        if !a_inst && !b_inst {
            if let Some(v) = self.builtin_binop(op, a, b)? {
                return Ok(v);
            }
            return Err(self.binop_error(op, a, b));
        }
        let (fwd, rev, _) = op_dunder(op);
        let fwd = ps(fwd);
        let rev = ps(rev);
        let ca = self.class_of(a);
        let cb = self.class_of(b);
        // the right operand's reflected method first when it is a subclass
        let b_first = b_inst && !Rc::ptr_eq(&ca, &cb) && cb.is_subclass(&ca) && cb.lookup(&rev).is_some();
        if b_first {
            if let Some(r) = self.try_dunder(b, &rev, a)? {
                return Ok(r);
            }
        }
        if a_inst {
            if let Some(r) = self.try_dunder(a, &fwd, b)? {
                return Ok(r);
            }
        }
        if b_inst && !b_first {
            if let Some(r) = self.try_dunder(b, &rev, a)? {
                return Ok(r);
            }
        }
        // str % anything formats the operand itself
        if op == BinOp::Mod && !a_inst && matches!(a, Value::Str(_) | Value::Bytes(_) | Value::ByteArray(_)) {
            let bt = self.base_of(b);
            let operand = if matches!(bt, Value::Tuple(_)) { bt } else { b.clone() };
            if let Some(v) = self.builtin_binop(op, a, &operand)? {
                return Ok(v);
            }
        }
        // instances wrapping a built-in value
        let ab = self.base_of(a);
        let bb = self.base_of(b);
        if !matches!(ab, Value::Instance(_)) && !matches!(bb, Value::Instance(_)) && (a_inst || b_inst) && !matches!(ab, Value::Class(_)) && !matches!(bb, Value::Class(_)) {
            if let Some(v) = self.builtin_binop(op, &ab, &bb)? {
                return Ok(v);
            }
        }
        if op == BinOp::BitOr {
            // X | Y on types builds a union
            if matches!(a, Value::Class(_) | Value::None) && matches!(b, Value::Class(_) | Value::None) || matches!(a, Value::Class(_)) || matches!(b, Value::Class(_)) {
                return crate::builtins::make_union(self, a, b);
            }
        }
        Err(self.binop_error(op, a, b))
    }

    /// calls obj.name(other) found on the type; None for NotImplemented
    fn try_dunder(&mut self, obj: &Value, name: &Rc<PyStr>, other: &Value) -> PyResult<Option<Value>> {
        let cls = self.class_of(obj);
        let f = match cls.lookup(name) {
            Some(f) => f,
            None => {
                if let Value::Class(c) = obj {
                    // a metaclass operator, e.g. Flag | Flag
                    let _ = c;
                }
                return Ok(None);
            }
        };
        let f = self.bind(f, obj, &cls);
        let r = self.call_vec(&f, vec![other.clone()], &[])?;
        if matches!(r, Value::NotImplemented) {
            Ok(None)
        } else {
            Ok(Some(r))
        }
    }

    /// n as a repeat count for a sequence of len items; MemoryError when
    /// the result could not be allocated
    pub fn repeat_count(&mut self, len: usize, n: i64) -> PyResult<usize> {
        if n <= 0 || len == 0 {
            return Ok(if n <= 0 { 0 } else { n as usize });
        }
        match len.checked_mul(n as usize) {
            Some(t) if (t as u64) <= if usize::BITS == 32 { 1u64 << 28 } else { 1u64 << 32 } => Ok(n as usize),
            _ => Err(self.err("MemoryError", "")),
        }
    }

    pub fn inplace(&mut self, op: BinOp, a: &Value, b: &Value) -> PyResult<Value> {
        match (a, op) {
            (Value::List(l), BinOp::Add) => {
                let items = self.to_vec(b)?;
                l.borrow_mut().extend(items);
                return Ok(a.clone());
            }
            (Value::List(l), BinOp::Mul) => {
                let n = self.to_index(b)?;
                let src = l.borrow().clone();
                let n = self.repeat_count(src.len(), n)? as i64;
                let mut lb = l.borrow_mut();
                lb.clear();
                for _ in 0..n.max(0) {
                    lb.extend(src.iter().cloned());
                }
                return Ok(a.clone());
            }
            (Value::Set(s), BinOp::BitOr | BinOp::BitAnd | BinOp::Sub | BinOp::BitXor) => {
                if let Value::Set(_) | Value::FrozenSet(_) | Value::DictView(_) = b {
                    let other = crate::builtins::to_set(self, b)?;
                    let r = crate::builtins::set_op(self, op, a, &other)?;
                    if let Value::Set(r) = r {
                        let nd = r.borrow().copy();
                        *s.borrow_mut() = nd;
                    }
                    return Ok(a.clone());
                }
            }
            (Value::Dict(d), BinOp::BitOr) => {
                let pairs = self.mapping_items(b)?;
                for (k, v) in pairs {
                    self.dict_set(d, k, v)?;
                }
                return Ok(a.clone());
            }
            (Value::ByteArray(x), BinOp::Add) => {
                let other = crate::builtins::bytes_of(self, b)?;
                x.borrow_mut().extend(other);
                return Ok(a.clone());
            }
            (Value::Instance(_), _) => {
                let (_, _, iname) = op_dunder(op);
                let iname = ps(iname);
                let cls = self.class_of(a);
                if cls.lookup(&iname).is_some() {
                    if let Some(r) = self.try_dunder(a, &iname, b)? {
                        return Ok(r);
                    }
                }
                let base = self.base_of(a);
                if let (Value::List(_) | Value::Dict(_) | Value::Set(_), true) = (&base, cls.lookup(&ps(op_dunder(op).0)).map(|f| matches!(f, Value::Native(_))).unwrap_or(true)) {
                    if matches!(op, BinOp::Add | BinOp::BitOr) {
                        self.inplace(op, &base, b)?;
                        return Ok(a.clone());
                    }
                }
            }
            _ => {}
        }
        self.binop(op, a, b)
    }

    pub fn unary(&mut self, op: UnOp, a: &Value) -> PyResult<Value> {
        Ok(match (op, a) {
            (UnOp::Not, _) => Value::Bool(!self.truthy(a)?),
            (UnOp::Neg, Value::Int(i)) => match i.checked_neg() {
                Some(v) => Value::Int(v),
                None => Value::from_big(BigInt::from_i64(*i).neg()),
            },
            (UnOp::Neg, Value::Bool(b)) => Value::Int(-(*b as i64)),
            (UnOp::Neg, Value::Float(f)) => Value::Float(-f),
            (UnOp::Neg, Value::Big(b)) => Value::from_big(b.neg()),
            (UnOp::Neg, Value::Complex(c)) => Value::Complex(Rc::new((-c.0, -c.1))),
            (UnOp::Pos, Value::Int(_) | Value::Float(_) | Value::Big(_) | Value::Complex(_)) => a.clone(),
            (UnOp::Pos, Value::Bool(b)) => Value::Int(*b as i64),
            (UnOp::Invert, Value::Int(i)) => Value::Int(!i),
            (UnOp::Invert, Value::Bool(b)) => Value::Int(!(*b as i64)),
            (UnOp::Invert, Value::Big(b)) => Value::from_big(b.neg().sub(&BigInt::from_i64(1))),
            (_, Value::Instance(_)) => {
                let name = match op {
                    UnOp::Neg => self.n.neg.clone(),
                    UnOp::Pos => self.n.pos.clone(),
                    _ => self.n.invert.clone(),
                };
                match self.lookup_special(a, &name) {
                    Some(f) if !matches!(f, Value::Method(ref m) if matches!(m.1, Value::Native(_))) => self.call_vec(&f, vec![], &[])?,
                    _ => {
                        let b = self.base_of(a);
                        if matches!(a, Value::Instance(_)) && !matches!(b, Value::Instance(_)) {
                            return self.unary(op, &b);
                        }
                        let tn = self.type_name(a);
                        let sym = match op {
                            UnOp::Neg => "-",
                            UnOp::Pos => "+",
                            _ => "~",
                        };
                        return Err(self.type_error(format!("bad operand type for unary {}: '{}'", sym, tn)));
                    }
                }
            }
            _ => {
                let tn = self.type_name(a);
                let sym = match op {
                    UnOp::Neg => "-",
                    UnOp::Pos => "+",
                    _ => "~",
                };
                return Err(self.type_error(format!("bad operand type for unary {}: '{}'", sym, tn)));
            }
        })
    }

    // ------------------------------------------------------------------
    // comparison

    pub fn compare(&mut self, op: CmpOp, a: &Value, b: &Value) -> PyResult<Value> {
        Ok(match op {
            CmpOp::Is => Value::Bool(a.is(b)),
            CmpOp::IsNot => Value::Bool(!a.is(b)),
            CmpOp::In => Value::Bool(self.contains(b, a)?),
            CmpOp::NotIn => Value::Bool(!self.contains(b, a)?),
            CmpOp::Eq => {
                if let Some(r) = fast_eq(a, b) {
                    if !matches!(a, Value::Instance(_)) && !matches!(b, Value::Instance(_)) {
                        return Ok(Value::Bool(r));
                    }
                }
                return self.rich_eq(a, b, false);
            }
            CmpOp::Ne => {
                if let Some(r) = fast_eq(a, b) {
                    if !matches!(a, Value::Instance(_)) && !matches!(b, Value::Instance(_)) {
                        return Ok(Value::Bool(!r));
                    }
                }
                return self.rich_eq(a, b, true);
            }
            _ => return self.rich_order(op, a, b),
        })
    }

    pub fn eq(&mut self, a: &Value, b: &Value) -> PyResult<bool> {
        if let Some(r) = fast_eq(a, b) {
            if !matches!(a, Value::Instance(_)) && !matches!(b, Value::Instance(_)) {
                return Ok(r);
            }
            if r {
                return Ok(true);
            }
        }
        let r = self.rich_eq(a, b, false)?;
        self.truthy(&r)
    }

    pub fn rich_eq(&mut self, a: &Value, b: &Value, negate: bool) -> PyResult<Value> {
        let (name, rname) = if negate { (self.n.ne.clone(), self.n.ne.clone()) } else { (self.n.eq.clone(), self.n.eq.clone()) };
        let a_inst = matches!(a, Value::Instance(_));
        let b_inst = matches!(b, Value::Instance(_));
        if a_inst || b_inst {
            let ca = self.class_of(a);
            let cb = self.class_of(b);
            let b_first = b_inst && !Rc::ptr_eq(&ca, &cb) && cb.is_subclass(&ca);
            if b_first {
                if let Some(r) = self.try_user_cmp(b, &rname, a)? {
                    return Ok(r);
                }
            }
            if a_inst {
                if let Some(r) = self.try_user_cmp(a, &name, b)? {
                    return Ok(r);
                }
            }
            if b_inst && !b_first {
                if let Some(r) = self.try_user_cmp(b, &rname, a)? {
                    return Ok(r);
                }
            }
            if negate {
                // __ne__ falls back to the inverse of __eq__
                let eqn = self.n.eq.clone();
                if a_inst {
                    if let Some(r) = self.try_user_cmp(a, &eqn, b)? {
                        let t = self.truthy(&r)?;
                        return Ok(Value::Bool(!t));
                    }
                }
                if b_inst {
                    if let Some(r) = self.try_user_cmp(b, &eqn, a)? {
                        let t = self.truthy(&r)?;
                        return Ok(Value::Bool(!t));
                    }
                }
            }
            let ab = self.base_of(a);
            let bb = self.base_of(b);
            if !matches!(ab, Value::Instance(_)) && !matches!(bb, Value::Instance(_)) {
                let r = self.eq_builtin(&ab, &bb)?;
                return Ok(Value::Bool(r != negate));
            }
            let same = a.is(b);
            return Ok(Value::Bool(same != negate));
        }
        let r = self.eq_builtin(a, b)?;
        Ok(Value::Bool(r != negate))
    }

    fn try_user_cmp(&mut self, obj: &Value, name: &Rc<PyStr>, other: &Value) -> PyResult<Option<Value>> {
        let cls = self.class_of(obj);
        match cls.lookup(name) {
            Some(f @ Value::Func(_)) | Some(f @ Value::Method(_)) | Some(f @ Value::Instance(_)) => {
                let f = self.bind(f, obj, &cls);
                let r = self.call_vec(&f, vec![other.clone()], &[])?;
                if matches!(r, Value::NotImplemented) {
                    Ok(None)
                } else {
                    Ok(Some(r))
                }
            }
            // natives of classes made in Rust (SimpleNamespace)
            Some(f @ Value::Native(_)) if matches!(&f, Value::Native(n) if !n.name.starts_with("object.")) && cls.builtin.get() == Builtin::User => {
                let r = self.call_vec(&f, vec![obj.clone(), other.clone()], &[])?;
                if matches!(r, Value::NotImplemented) {
                    Ok(None)
                } else {
                    Ok(Some(r))
                }
            }
            Some(Value::None) => Ok(None),
            _ => Ok(None),
        }
    }

    /// equality of built-in containers and scalars
    pub fn eq_builtin(&mut self, a: &Value, b: &Value) -> PyResult<bool> {
        if let Some(r) = fast_eq(a, b) {
            return Ok(r);
        }
        match (a, b) {
            (Value::List(x), Value::List(y)) => {
                if Rc::ptr_eq(x, y) {
                    return Ok(true);
                }
                let (xv, yv) = (x.borrow().clone(), y.borrow().clone());
                self.seq_eq(&xv, &yv)
            }
            (Value::Tuple(x), Value::Tuple(y)) => {
                let (xv, yv) = (x.to_vec(), y.to_vec());
                self.seq_eq(&xv, &yv)
            }
            (Value::Dict(x), Value::Dict(y)) => {
                if x.borrow().len() != y.borrow().len() {
                    return Ok(false);
                }
                let items = x.borrow().items();
                for (k, v) in items {
                    match self.dict_get(y, &k)? {
                        Some(v2) => {
                            self.enter(" in comparison")?;
                            let r = self.eq(&v, &v2);
                            self.depth -= 1;
                            if !r? {
                                return Ok(false);
                            }
                        }
                        None => return Ok(false),
                    }
                }
                Ok(true)
            }
            (Value::Set(x) | Value::FrozenSet(x), Value::Set(y) | Value::FrozenSet(y)) => {
                if x.borrow().len() != y.borrow().len() {
                    return Ok(false);
                }
                let keys = x.borrow().keys();
                for k in keys {
                    if self.dict_get(y, &k)?.is_none() {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            (Value::DictView(x), Value::DictView(y)) if x.1 == 0 && y.1 == 0 => {
                let sa = crate::builtins::to_set(self, a)?;
                let sb = crate::builtins::to_set(self, b)?;
                self.eq_builtin(&sa, &sb)
            }
            (Value::DictView(x), Value::Set(_) | Value::FrozenSet(_)) if x.1 != 1 => {
                let sa = crate::builtins::to_set(self, a)?;
                self.eq_builtin(&sa, b)
            }
            (Value::Range(_) | Value::BigRange(_), Value::Range(_) | Value::BigRange(_)) => Ok(crate::range::eq(a, b)),
            (Value::Slice(x), Value::Slice(y)) => Ok(self.eq(&x.0, &y.0)? && self.eq(&x.1, &y.1)? && self.eq(&x.2, &y.2)?),
            (Value::ByteArray(x), Value::ByteArray(y)) => Ok(*x.borrow() == *y.borrow()),
            (Value::ByteArray(x), Value::Bytes(y)) | (Value::Bytes(y), Value::ByteArray(x)) => Ok(*x.borrow() == **y),
            (Value::Method(x), Value::Method(y)) => Ok(x.0.is(&y.0) && x.1.is(&y.1)),
            _ => Ok(a.is(b)),
        }
    }

    fn seq_eq(&mut self, x: &[Value], y: &[Value]) -> PyResult<bool> {
        self.enter(" in comparison")?;
        let r = self.seq_eq_inner(x, y);
        self.depth -= 1;
        r
    }

    /// counts a level of Rust recursion against the Python recursion limit
    pub fn enter(&mut self, what: &str) -> PyResult<()> {
        self.depth += 1;
        if self.depth > self.max_depth + 50 {
            self.depth -= 1;
            return Err(self.err("RecursionError", format!("maximum recursion depth exceeded{}", what)));
        }
        Ok(())
    }

    fn seq_eq_inner(&mut self, x: &[Value], y: &[Value]) -> PyResult<bool> {
        if x.len() != y.len() {
            return Ok(false);
        }
        for (p, q) in x.iter().zip(y.iter()) {
            if p.is(q) {
                continue;
            }
            if !self.eq(p, q)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub fn rich_order(&mut self, op: CmpOp, a: &Value, b: &Value) -> PyResult<Value> {
        let (name, rname) = match op {
            CmpOp::Lt => (self.n.lt.clone(), self.n.gt.clone()),
            CmpOp::Le => (self.n.le.clone(), self.n.ge.clone()),
            CmpOp::Gt => (self.n.gt.clone(), self.n.lt.clone()),
            _ => (self.n.ge.clone(), self.n.le.clone()),
        };
        let a_inst = matches!(a, Value::Instance(_));
        let b_inst = matches!(b, Value::Instance(_));
        if a_inst || b_inst {
            let ca = self.class_of(a);
            let cb = self.class_of(b);
            let b_first = b_inst && !Rc::ptr_eq(&ca, &cb) && cb.is_subclass(&ca);
            if b_first {
                if let Some(r) = self.try_user_cmp(b, &rname, a)? {
                    return Ok(r);
                }
            }
            if a_inst {
                if let Some(r) = self.try_user_cmp(a, &name, b)? {
                    return Ok(r);
                }
            }
            if b_inst && !b_first {
                if let Some(r) = self.try_user_cmp(b, &rname, a)? {
                    return Ok(r);
                }
            }
            let ab = self.base_of(a);
            let bb = self.base_of(b);
            if !matches!(ab, Value::Instance(_)) && !matches!(bb, Value::Instance(_)) {
                return self.rich_order(op, &ab, &bb);
            }
            return Err(self.order_error(op, a, b));
        }
        match self.order_builtin(a, b)? {
            Some(ord) => Ok(Value::Bool(match op {
                CmpOp::Lt => ord == Ordering::Less,
                CmpOp::Le => ord != Ordering::Greater,
                CmpOp::Gt => ord == Ordering::Greater,
                _ => ord != Ordering::Less,
            })),
            None => {
                // NaN and sets give "unordered"
                match (a, b) {
                    (Value::Float(_) | Value::Int(_) | Value::Bool(_) | Value::Big(_), Value::Float(_) | Value::Int(_) | Value::Bool(_) | Value::Big(_)) => Ok(Value::Bool(false)),
                    (Value::Set(_) | Value::FrozenSet(_) | Value::DictView(_), Value::Set(_) | Value::FrozenSet(_) | Value::DictView(_)) => crate::builtins::set_compare(self, op, a, b),
                    _ => Err(self.order_error(op, a, b)),
                }
            }
        }
    }

    pub fn order_error(&mut self, op: CmpOp, a: &Value, b: &Value) -> Exc {
        let (an, bn) = (self.type_name(a), self.type_name(b));
        self.type_error(format!("'{}' not supported between instances of '{}' and '{}'", cmp_symbol(op), an, bn))
    }

    /// ordering of built-in values; Ok(None) when unordered or unsupported
    pub fn order_builtin(&mut self, a: &Value, b: &Value) -> PyResult<Option<Ordering>> {
        use Value::*;
        Ok(match (a, b) {
            (Int(x), Int(y)) => Some(x.cmp(y)),
            (Float(x), Float(y)) => x.partial_cmp(y),
            (Int(x), Float(y)) => {
                if y.is_nan() {
                    Option::None
                } else if (*x as f64) == *y {
                    // exact comparison for large values
                    let yi = *y as i128;
                    Some((*x as i128).cmp(&yi))
                } else {
                    (*x as f64).partial_cmp(y)
                }
            }
            (Float(x), Int(y)) => {
                if x.is_nan() {
                    Option::None
                } else {
                    x.partial_cmp(&(*y as f64))
                }
            }
            (Bool(_), _) | (_, Bool(_)) if matches!(a, Bool(_) | Int(_) | Float(_) | Big(_)) && matches!(b, Bool(_) | Int(_) | Float(_) | Big(_)) => {
                let x = match a {
                    Bool(v) => Int(*v as i64),
                    o => o.clone(),
                };
                let y = match b {
                    Bool(v) => Int(*v as i64),
                    o => o.clone(),
                };
                return self.order_builtin(&x, &y);
            }
            (Big(_), Int(_) | Big(_)) | (Int(_), Big(_)) => Some(big_of(a).unwrap().cmp(&big_of(b).unwrap())),
            (Big(x), Float(y)) => x.to_f64().partial_cmp(y),
            (Float(x), Big(y)) => x.partial_cmp(&y.to_f64()),
            (Str(x), Str(y)) => Some(x.s.cmp(&y.s)),
            (Bytes(x), Bytes(y)) => Some(x.cmp(y)),
            (List(x), List(y)) => {
                let (xv, yv) = (x.borrow().clone(), y.borrow().clone());
                return self.seq_order(&xv, &yv);
            }
            (Tuple(x), Tuple(y)) => {
                let (xv, yv) = (x.to_vec(), y.to_vec());
                return self.seq_order(&xv, &yv);
            }
            _ => Option::None,
        })
    }

    fn seq_order(&mut self, x: &[Value], y: &[Value]) -> PyResult<Option<Ordering>> {
        self.enter(" in comparison")?;
        let r = self.seq_order_inner(x, y);
        self.depth -= 1;
        r
    }

    fn seq_order_inner(&mut self, x: &[Value], y: &[Value]) -> PyResult<Option<Ordering>> {
        for (p, q) in x.iter().zip(y.iter()) {
            if p.is(q) || self.eq(p, q)? {
                continue;
            }
            return match self.order_builtin(p, q)? {
                Some(o) => Ok(Some(o)),
                None => {
                    // user-defined ordering inside a sequence
                    let lt = self.rich_order(CmpOp::Lt, p, q)?;
                    if self.truthy(&lt)? {
                        Ok(Some(Ordering::Less))
                    } else {
                        Ok(Some(Ordering::Greater))
                    }
                }
            };
        }
        Ok(Some(x.len().cmp(&y.len())))
    }

    /// a < b for sorting
    pub fn less(&mut self, a: &Value, b: &Value) -> PyResult<bool> {
        match (a, b) {
            (Value::Int(x), Value::Int(y)) => Ok(x < y),
            (Value::Float(x), Value::Float(y)) => Ok(x < y),
            (Value::Str(x), Value::Str(y)) => Ok(x.s < y.s),
            _ => {
                let r = self.rich_order(CmpOp::Lt, a, b)?;
                self.truthy(&r)
            }
        }
    }

    // ------------------------------------------------------------------
    // truth, hash, len

    pub fn truthy(&mut self, v: &Value) -> PyResult<bool> {
        Ok(match v {
            Value::None | Value::Undef => false,
            Value::Bool(b) => *b,
            Value::Int(i) => *i != 0,
            Value::Big(_) => true,
            Value::Float(f) => *f != 0.0,
            Value::Complex(c) => c.0 != 0.0 || c.1 != 0.0,
            Value::Str(s) => !s.s.is_empty(),
            Value::Bytes(b) => !b.is_empty(),
            Value::ByteArray(b) => !b.borrow().is_empty(),
            Value::List(l) => !l.borrow().is_empty(),
            Value::Tuple(t) => !t.is_empty(),
            Value::Dict(d) | Value::Set(d) | Value::FrozenSet(d) => !d.borrow().is_empty(),
            Value::Range(r) => crate::builtins::range_len(r) > 0,
            Value::BigRange(r) => !r.len.is_zero(),
            Value::DictView(d) => !d.0.borrow().is_empty(),
            Value::Instance(_) => {
                if let Some(f) = self.lookup_special(v, &self.n.bool_.clone()) {
                    let r = self.call_vec(&f, vec![], &[])?;
                    return match r {
                        Value::Bool(b) => Ok(b),
                        _ => {
                            let tn = self.type_name(&r);
                            Err(self.type_error(format!("__bool__ should return bool, returned {}", tn)))
                        }
                    };
                }
                if let Some(f) = self.lookup_special(v, &self.n.len.clone()) {
                    let r = self.call_vec(&f, vec![], &[])?;
                    return Ok(self.to_index(&r)? != 0);
                }
                let b = self.base_of(v);
                if matches!(v, Value::Instance(_)) && !matches!(b, Value::Instance(_)) {
                    return self.truthy(&b);
                }
                true
            }
            _ => true,
        })
    }

    pub fn hash(&mut self, v: &Value) -> PyResult<u64> {
        if let Some(h) = fast_hash(v) {
            return Ok(h);
        }
        match v {
            Value::Instance(_) => {
                let cls = self.class_of(v);
                match cls.lookup(&self.n.hash) {
                    Some(Value::None) => {
                        let tn = cls.name();
                        Err(self.type_error(format!("unhashable type: '{}'", tn)))
                    }
                    Some(f @ Value::Func(_)) => {
                        let f = self.bind(f, v, &cls);
                        let r = self.call_vec(&f, vec![], &[])?;
                        match r {
                            Value::Int(i) => Ok(hash_int(i)),
                            Value::Big(b) => Ok(hash_big(&b)),
                            Value::Bool(b) => Ok(b as u64),
                            Value::Instance(ref i) if matches!(&*i.base.borrow(), Value::Int(_) | Value::Big(_)) => {
                                let b = i.base.borrow().clone();
                                self.hash(&b)
                            }
                            _ => Err(self.type_error("__hash__ method should return an integer")),
                        }
                    }
                    _ => {
                        // a class defining __eq__ without __hash__ is unhashable
                        let user_eq = cls.mro.borrow().iter().take_while(|c| c.builtin.get() == Builtin::User || !c.dict.borrow().is_empty()).any(|c| matches!(c.dict.borrow().get_str(&self.n.eq), Some(Value::Func(_))));
                        let has_hash = cls.mro.borrow().iter().any(|c| matches!(c.dict.borrow().get_str(&self.n.hash), Some(Value::Func(_))));
                        let eq_def = cls.dict.borrow().get_str(&self.n.eq).is_some();
                        if eq_def && !has_hash && user_eq {
                            let tn = cls.name();
                            return Err(self.type_error(format!("unhashable type: '{}'", tn)));
                        }
                        let b = self.base_of(v);
                        if matches!(v, Value::Instance(_)) && !matches!(b, Value::Instance(_)) && !matches!(b, Value::Float(f) if f.is_nan()) {
                            return self.hash(&b);
                        }
                        Ok((ptr_id(v) as u64) >> 4)
                    }
                }
            }
            Value::Tuple(items) => {
                let mut acc: u64 = 0x27D4EB2F165667C5;
                for it in items.iter() {
                    let lane = self.hash(it)?;
                    acc = acc.wrapping_add(lane.wrapping_mul(0xC2B2AE3D27D4EB4F));
                    acc = acc.rotate_left(31);
                    acc = acc.wrapping_mul(0x9E3779B185EBCA87);
                }
                Ok(acc.wrapping_add((items.len() as u64) ^ (0x27D4EB2F165667C5 ^ 3527539)))
            }
            _ => {
                let tn = self.type_name(v);
                Err(self.type_error(format!("unhashable type: '{}'", tn)))
            }
        }
    }

    pub fn len(&mut self, v: &Value) -> PyResult<usize> {
        Ok(match v {
            Value::Str(s) => s.char_len(),
            Value::List(l) => l.borrow().len(),
            Value::Tuple(t) => t.len(),
            Value::Dict(d) | Value::Set(d) | Value::FrozenSet(d) => d.borrow().len(),
            Value::Bytes(b) => b.len(),
            Value::ByteArray(b) => b.borrow().len(),
            Value::Range(r) => crate::builtins::range_len(r),
            Value::BigRange(r) => crate::range::len(self, r)?,
            Value::DictView(d) => d.0.borrow().len(),
            Value::Instance(_) | Value::Obj(_) => {
                if let Some(f) = self.lookup_special(v, &self.n.len.clone()) {
                    let r = self.call_vec(&f, vec![], &[])?;
                    if let Value::Big(b) = &r {
                        if b.neg {
                            return Err(self.value_error("__len__() should return >= 0"));
                        }
                    }
                    let n = self.to_index(&r)?;
                    if n < 0 {
                        return Err(self.value_error("__len__() should return >= 0"));
                    }
                    return Ok(n as usize);
                }
                let b = self.base_of(v);
                if matches!(v, Value::Instance(_)) && !matches!(b, Value::Instance(_)) {
                    return self.len(&b);
                }
                let tn = self.type_name(v);
                return Err(self.type_error(format!("object of type '{}' has no len()", tn)));
            }
            _ => {
                let tn = self.type_name(v);
                return Err(self.type_error(format!("object of type '{}' has no len()", tn)));
            }
        })
    }

    // ------------------------------------------------------------------
    // dict and set storage

    pub fn dict_get(&mut self, d: &Rc<RefCell<Dict>>, k: &Value) -> PyResult<Option<Value>> {
        if let Value::Str(s) = k {
            if let Some(v) = d.borrow().get_str(s) {
                return Ok(Some(v.clone()));
            }
        }
        let h = self.hash(k)?;
        let r = d.borrow().lookup(h, k);
        match r {
            Lookup::Found(i) => Ok(d.borrow().entries[i].as_ref().map(|e| e.value.clone())),
            Lookup::Missing => Ok(None),
            Lookup::Slow(cands) => {
                for i in cands {
                    let key = match d.borrow().entries.get(i) {
                        Some(Some(e)) => e.key.clone(),
                        _ => continue,
                    };
                    if self.eq(&key, k)? {
                        let v = match d.borrow().entries.get(i) {
                            Some(Some(e)) if e.key.is(&key) => Some(Some(e.value.clone())),
                            _ => None,
                        };
                        return match v {
                            Some(v) => Ok(v),
                            None => self.dict_get(d, k),
                        };
                    }
                }
                Ok(None)
            }
        }
    }

    /// the entry index of `k`
    pub fn dict_find(&mut self, d: &Rc<RefCell<Dict>>, k: &Value) -> PyResult<(u64, Option<usize>)> {
        let h = self.hash(k)?;
        let r = d.borrow().lookup(h, k);
        match r {
            Lookup::Found(i) => Ok((h, Some(i))),
            Lookup::Missing => Ok((h, None)),
            Lookup::Slow(cands) => {
                for i in cands {
                    let key = match d.borrow().entries.get(i) {
                        Some(Some(e)) => e.key.clone(),
                        _ => continue,
                    };
                    if self.eq(&key, k)? {
                        // __eq__ may have changed the dict: look again
                        let same = matches!(d.borrow().entries.get(i), Some(Some(e)) if e.key.is(&key));
                        if !same {
                            return self.dict_find(d, k);
                        }
                        return Ok((h, Some(i)));
                    }
                }
                Ok((h, None))
            }
        }
    }

    pub fn dict_set(&mut self, d: &Rc<RefCell<Dict>>, k: Value, v: Value) -> PyResult<()> {
        if let Value::Str(s) = &k {
            let s = s.clone();
            d.borrow_mut().set_str(s, v);
            return Ok(());
        }
        let (h, i) = self.dict_find(d, &k)?;
        match i {
            Some(i) => d.borrow_mut().set_at(i, v),
            None => d.borrow_mut().push_new(h, k, v),
        }
        Ok(())
    }

    pub fn dict_del(&mut self, d: &Rc<RefCell<Dict>>, k: &Value) -> PyResult<Option<Value>> {
        let (_, i) = self.dict_find(d, k)?;
        match i {
            Some(i) => Ok(d.borrow_mut().remove_at(i).map(|e| e.value)),
            None => Ok(None),
        }
    }

    pub fn set_add(&mut self, s: &Rc<RefCell<Dict>>, k: Value) -> PyResult<()> {
        let (h, i) = self.dict_find(s, &k)?;
        if i.is_none() {
            s.borrow_mut().push_new(h, k, Value::None);
        }
        Ok(())
    }

    pub fn set_contains(&mut self, s: &Rc<RefCell<Dict>>, k: &Value) -> PyResult<bool> {
        match k {
            Value::Set(inner) => {
                // a set is looked up as the equal frozenset
                let f = Value::FrozenSet(inner.clone());
                Ok(self.dict_find(s, &f)?.1.is_some())
            }
            _ => Ok(self.dict_find(s, k)?.1.is_some()),
        }
    }

    /// (key, value) pairs of a dict, a mapping with keys(), or pairs
    pub fn mapping_items(&mut self, v: &Value) -> PyResult<Vec<(Value, Value)>> {
        match v {
            Value::Dict(d) => Ok(d.borrow().items()),
            Value::Instance(i) => {
                if let Value::Dict(d) = &*i.base.borrow() {
                    // a dict subclass, unless it overrides keys()
                    let cls = i.class.borrow().clone();
                    if !matches!(cls.lookup(&ps("keys")), Some(Value::Func(_))) {
                        return Ok(d.borrow().items());
                    }
                }
                let keys = self.getattr(v, &ps("keys"))?;
                let ks = self.call_vec(&keys, vec![], &[])?;
                let ks = self.to_vec(&ks)?;
                let mut out = vec![];
                for k in ks {
                    let val = self.getitem(v, &k)?;
                    out.push((k, val));
                }
                Ok(out)
            }
            _ => {
                if let Ok(Some(keys)) = self.getattr_opt(v, &ps("keys")) {
                    let ks = self.call_vec(&keys, vec![], &[])?;
                    let ks = self.to_vec(&ks)?;
                    let mut out = vec![];
                    for k in ks {
                        let val = self.getitem(v, &k)?;
                        out.push((k, val));
                    }
                    return Ok(out);
                }
                let tn = self.type_name(v);
                Err(self.type_error(format!("'{}' object is not a mapping", tn)))
            }
        }
    }

    // ------------------------------------------------------------------
    // iteration

    pub fn iter(&mut self, v: &Value) -> PyResult<Value> {
        let it = match v {
            Value::List(l) => Iter::List(l.clone(), 0),
            Value::Tuple(t) => Iter::Tuple(t.clone(), 0),
            Value::Str(s) => Iter::Str(s.clone(), 0),
            Value::Bytes(b) => Iter::Bytes(b.clone(), 0),
            Value::ByteArray(b) => Iter::Bytes(Rc::new(b.borrow().clone()), 0),
            Value::Range(r) => crate::range::iter_i128(r.0 as i128, r.2 as i128, crate::builtins::range_len(r) as i128),
            Value::BigRange(r) => crate::range::iter(r),
            Value::Dict(d) | Value::Set(d) | Value::FrozenSet(d) => {
                let n = d.borrow().len();
                Iter::Dict(d.clone(), 0, 0, n)
            }
            Value::DictView(dv) => {
                let n = dv.0.borrow().len();
                Iter::Dict(dv.0.clone(), 0, dv.1, n)
            }
            Value::Iter(_) | Value::Gen(_) => return Ok(v.clone()),
            Value::Instance(_) | Value::Obj(_) | Value::Class(_) => {
                if let Value::Class(c) = v {
                    // a class with a metaclass __iter__ (Enum)
                    let meta = c.meta.borrow().clone();
                    if let Some(m) = meta {
                        if let Some(f) = m.lookup(&self.n.iter) {
                            let f = self.bind(f, v, &m);
                            return self.call_vec(&f, vec![], &[]);
                        }
                    }
                    let tn = self.type_name(v);
                    return Err(self.type_error(format!("'{}' object is not iterable", tn)));
                }
                if let Some(f) = self.lookup_special(v, &self.n.iter.clone()) {
                    if let Value::Method(m) = &f {
                        if let Value::Native(n) = &m.1 {
                            if n.name.ends_with(".__iter__") && matches!(v, Value::Instance(_)) {
                                let b = self.base_of(v);
                                if matches!(v, Value::Instance(_)) && !matches!(b, Value::Instance(_)) {
                                    return self.iter(&b);
                                }
                            }
                        }
                    }
                    let r = self.call_vec(&f, vec![], &[])?;
                    if self.lookup_special(&r, &self.n.next.clone()).is_none() && !matches!(r, Value::Iter(_) | Value::Gen(_)) {
                        let tn = self.type_name(&r);
                        return Err(self.type_error(format!("iter() returned non-iterator of type '{}'", tn)));
                    }
                    return Ok(r);
                }
                if matches!(v, Value::Instance(_)) {
                    let b = self.base_of(v);
                    if matches!(v, Value::Instance(_)) && !matches!(b, Value::Instance(_)) {
                        return self.iter(&b);
                    }
                }
                if self.lookup_special(v, &self.n.getitem.clone()).is_some() {
                    Iter::Seq(v.clone(), 0)
                } else {
                    let tn = self.type_name(v);
                    return Err(self.type_error(format!("'{}' object is not iterable", tn)));
                }
            }
            _ => {
                let tn = self.type_name(v);
                return Err(self.type_error(format!("'{}' object is not iterable", tn)));
            }
        };
        Ok(Value::Iter(Rc::new(RefCell::new(it))))
    }

    /// the next item, or None when exhausted
    pub fn next(&mut self, it: &Value) -> PyResult<Option<Value>> {
        match it {
            Value::Iter(r) => self.iter_next(r),
            Value::Gen(g) => self.resume(g, Value::None, None),
            _ => {
                let f = match self.lookup_special(it, &self.n.next.clone()) {
                    Some(f) => f,
                    None => {
                        let tn = self.type_name(it);
                        return Err(self.type_error(format!("'{}' object is not an iterator", tn)));
                    }
                };
                match self.call_vec(&f, vec![], &[]) {
                    Ok(v) => Ok(Some(v)),
                    Err(e) if self.exc_is(&e, "StopIteration") => {
                        let a = self.exc_args(&e.value);
                        self.last_return = Some(a.into_iter().next().unwrap_or(Value::None));
                        Ok(None)
                    }
                    Err(e) => Err(e),
                }
            }
        }
    }

    fn iter_next(&mut self, r: &Rc<RefCell<Iter>>) -> PyResult<Option<Value>> {
        let mut b = r.borrow_mut();
        match &mut *b {
            Iter::List(l, i) => {
                let lb = l.borrow();
                if *i < lb.len() {
                    *i += 1;
                    Ok(Some(lb[*i - 1].clone()))
                } else {
                    *i = usize::MAX / 2;
                    Ok(None)
                }
            }
            Iter::ListRev(l, i) => {
                let lb = l.borrow();
                if *i > 0 && *i <= lb.len() {
                    *i -= 1;
                    Ok(Some(lb[*i].clone()))
                } else {
                    *i = 0;
                    Ok(None)
                }
            }
            Iter::Tuple(t, i) => {
                if *i < t.len() {
                    *i += 1;
                    Ok(Some(t[*i - 1].clone()))
                } else {
                    Ok(None)
                }
            }
            Iter::Str(s, i) => {
                if *i >= s.s.len() {
                    return Ok(None);
                }
                let c = s.s[*i..].chars().next().unwrap();
                *i += c.len_utf8();
                Ok(Some(crate::builtins::char_value(c)))
            }
            Iter::Bytes(bs, i) => {
                if *i < bs.len() {
                    *i += 1;
                    Ok(Some(Value::Int(bs[*i - 1] as i64)))
                } else {
                    Ok(None)
                }
            }
            Iter::Range(cur, left, step) => {
                if *left > 0 {
                    let v = *cur;
                    *left -= 1;
                    *cur = cur.wrapping_add(*step);
                    Ok(Some(Value::Int(v)))
                } else {
                    Ok(None)
                }
            }
            Iter::BigRange(st) => Ok(crate::range::iter_next(st)),
            Iter::Dict(d, pos, kind, size) => {
                let db = d.borrow();
                if db.len() != *size {
                    let changed = db.len();
                    let was = *size;
                    *size = changed;
                    drop(db);
                    drop(b);
                    let what = "dictionary";
                    let _ = was;
                    return Err(self.err("RuntimeError", format!("{} changed size during iteration", what)));
                }
                match db.next_from(*pos) {
                    Some((p, e)) => {
                        *pos = p + 1;
                        Ok(Some(match kind {
                            0 => e.key.clone(),
                            1 => e.value.clone(),
                            _ => Value::tuple(vec![e.key.clone(), e.value.clone()]),
                        }))
                    }
                    None => {
                        *pos = usize::MAX / 2;
                        Ok(None)
                    }
                }
            }
            Iter::Empty => Ok(None),
            Iter::Vec(v) => Ok(v.next()),
            Iter::Count(cur, step) => {
                let c = cur.clone();
                let s = step.clone();
                drop(b);
                let n = self.binop(BinOp::Add, &c, &s)?;
                if let Iter::Count(cur, _) = &mut *r.borrow_mut() {
                    *cur = n;
                }
                Ok(Some(c))
            }
            Iter::Repeat(v, n) => {
                if *n == 0 {
                    return Ok(None);
                }
                if *n > 0 {
                    *n -= 1;
                }
                Ok(Some(v.clone()))
            }
            Iter::Enumerate(inner, count) => {
                let inner = inner.clone();
                let c = count.clone();
                drop(b);
                match self.next(&inner)? {
                    Some(v) => {
                        let nc = match &c {
                            Value::Int(i) => Value::Int(i + 1),
                            other => self.binop(BinOp::Add, other, &Value::Int(1))?,
                        };
                        if let Iter::Enumerate(_, count) = &mut *r.borrow_mut() {
                            *count = nc;
                        }
                        Ok(Some(Value::tuple(vec![c, v])))
                    }
                    None => Ok(None),
                }
            }
            Iter::Zip(its, strict) => {
                let its = its.clone();
                let strict = *strict;
                drop(b);
                let mut out = Vec::with_capacity(its.len());
                for (k, it) in its.iter().enumerate() {
                    match self.next(it)? {
                        Some(v) => out.push(v),
                        None => {
                            if strict {
                                if k > 0 {
                                    return Err(self.value_error(format!("zip() argument {} is shorter than argument{} {}", k + 1, if k > 1 { "s 1-" } else { " " }, k)));
                                }
                                for (j, rest) in its.iter().enumerate().skip(1) {
                                    if self.next(rest)?.is_some() {
                                        return Err(self.value_error(format!("zip() argument {} is longer than argument{} {}", j + 1, if j > 1 { "s 1-" } else { " " }, j)));
                                    }
                                }
                            }
                            return Ok(None);
                        }
                    }
                }
                if its.is_empty() {
                    return Ok(None);
                }
                Ok(Some(Value::tuple(out)))
            }
            Iter::Map(f, its) => {
                let f = f.clone();
                let its = its.clone();
                drop(b);
                let mut args = Vec::with_capacity(its.len());
                for it in &its {
                    match self.next(it)? {
                        Some(v) => args.push(v),
                        None => return Ok(None),
                    }
                }
                Ok(Some(self.call_vec(&f, args, &[])?))
            }
            Iter::Filter(f, it, invert) => {
                let f = f.clone();
                let it = it.clone();
                let invert = *invert;
                drop(b);
                loop {
                    match self.next(&it)? {
                        Some(v) => {
                            let keep = if f.is_none() { self.truthy(&v)? } else {
                                let r = self.call_vec(&f, vec![v.clone()], &[])?;
                                self.truthy(&r)?
                            };
                            if keep != invert {
                                return Ok(Some(v));
                            }
                        }
                        None => return Ok(None),
                    }
                }
            }
            Iter::Seq(obj, i) => {
                let obj = obj.clone();
                let idx = *i;
                *i += 1;
                drop(b);
                match self.getitem(&obj, &Value::Int(idx)) {
                    Ok(v) => Ok(Some(v)),
                    Err(e) if self.exc_is(&e, "IndexError") || self.exc_is(&e, "StopIteration") => {
                        *r.borrow_mut() = Iter::Empty;
                        Ok(None)
                    }
                    Err(e) => Err(e),
                }
            }
            Iter::SeqRev(obj, i) => {
                if *i < 0 {
                    return Ok(None);
                }
                let obj = obj.clone();
                let idx = *i;
                *i -= 1;
                drop(b);
                match self.getitem(&obj, &Value::Int(idx)) {
                    Ok(v) => Ok(Some(v)),
                    Err(e) if self.exc_is(&e, "IndexError") || self.exc_is(&e, "StopIteration") => {
                        *r.borrow_mut() = Iter::Empty;
                        Ok(None)
                    }
                    Err(e) => Err(e),
                }
            }
            Iter::Callable(f, sentinel) => {
                let f = f.clone();
                let s = sentinel.clone();
                drop(b);
                let v = self.call_vec(&f, vec![], &[])?;
                // exhausted by a reentrant call while f ran
                if matches!(&*r.borrow(), Iter::Empty) {
                    return Ok(None);
                }
                if self.eq(&v, &s)? {
                    *r.borrow_mut() = Iter::Empty;
                    Ok(None)
                } else {
                    Ok(Some(v))
                }
            }
            Iter::Chain(its, idx, cur) => {
                loop {
                    let (cur_v, its_v, i) = {
                        let b2 = &mut *r.borrow_mut();
                        match b2 {
                            Iter::Chain(its, idx, cur) => (cur.clone(), its.clone(), *idx),
                            _ => unreachable!(),
                        }
                    };
                    let _ = (&its, &idx, &cur);
                    if !cur_v.is_none() {
                        if let Some(v) = self.next(&cur_v)? {
                            return Ok(Some(v));
                        }
                    }
                    if i >= its_v.len() {
                        return Ok(None);
                    }
                    let nit = self.iter(&its_v[i])?;
                    if let Iter::Chain(_, idx, cur) = &mut *r.borrow_mut() {
                        *idx = i + 1;
                        *cur = nit;
                    }
                }
            }
        }
    }

    /// all items of an iterable
    pub fn to_vec(&mut self, v: &Value) -> PyResult<Vec<Value>> {
        match v {
            Value::List(l) => Ok(l.borrow().clone()),
            Value::Tuple(t) => Ok(t.to_vec()),
            Value::Dict(d) | Value::Set(d) | Value::FrozenSet(d) => Ok(d.borrow().keys()),
            Value::Range(r) => Ok(crate::builtins::range_items(r).into_iter().map(Value::Int).collect()),
            Value::BigRange(r) => {
                crate::range::len(self, r)?;
                Ok(crate::range::items(r))
            }
            Value::Str(s) => Ok(s.s.chars().map(crate::builtins::char_value).collect()),
            _ => {
                let it = self.iter(v)?;
                let mut out = vec![];
                while let Some(x) = self.next(&it)? {
                    out.push(x);
                }
                Ok(out)
            }
        }
    }

    pub fn unpack_n(&mut self, v: &Value, n: usize) -> PyResult<Vec<Value>> {
        let it = match self.iter(v) {
            Ok(it) => it,
            Err(e) if self.exc_is(&e, "TypeError") => {
                let tn = self.type_name(v);
                return Err(self.type_error(format!("cannot unpack non-iterable {} object", tn)));
            }
            Err(e) => return Err(e),
        };
        let mut out = Vec::with_capacity(n);
        while let Some(x) = self.next(&it)? {
            out.push(x);
            if out.len() > n {
                return Err(self.value_error(format!("too many values to unpack (expected {})", n)));
            }
        }
        if out.len() < n {
            return Err(self.value_error(format!("not enough values to unpack (expected {}, got {})", n, out.len())));
        }
        Ok(out)
    }

    // ------------------------------------------------------------------
    // items

    pub fn getitem(&mut self, obj: &Value, idx: &Value) -> PyResult<Value> {
        match obj {
            Value::Dict(d) => match self.dict_get(d, idx)? {
                Some(v) => Ok(v),
                None => Err(self.key_error(idx.clone())),
            },
            Value::List(_) | Value::Tuple(_) | Value::Str(_) | Value::Bytes(_) | Value::ByteArray(_) | Value::Range(_) | Value::BigRange(_) => crate::builtins::seq_getitem(self, obj, idx),
            Value::Instance(inst) => {
                let cls = inst.class.borrow().clone();
                match cls.lookup(&self.n.getitem) {
                    Some(f @ Value::Func(_)) => {
                        let f = self.bind(f, obj, &cls);
                        self.call_vec(&f, vec![idx.clone()], &[])
                    }
                    other => {
                        let b = self.base_of(obj);
                        if let Value::Dict(d) = &b {
                            return match self.dict_get(d, idx)? {
                                Some(v) => Ok(v),
                                None => {
                                    if let Some(m) = cls.lookup(&self.n.missing) {
                                        let m = self.bind(m, obj, &cls);
                                        return self.call_vec(&m, vec![idx.clone()], &[]);
                                    }
                                    Err(self.key_error(idx.clone()))
                                }
                            };
                        }
                        if !matches!(b, Value::Instance(_)) {
                            return self.getitem(&b, idx);
                        }
                        if let Some(f) = other {
                            let f = self.bind(f, obj, &cls);
                            return self.call_vec(&f, vec![idx.clone()], &[]);
                        }
                        let tn = cls.name();
                        Err(self.type_error(format!("'{}' object is not subscriptable", tn)))
                    }
                }
            }
            Value::Class(c) => {
                // generic aliases: list[int], MyClass[T]
                if let Some(f) = c.lookup(&self.n.class_getitem) {
                    let f = self.bind_classmethod(&f, obj);
                    return self.call_vec(&f, vec![idx.clone()], &[]);
                }
                let meta = c.meta.borrow().clone();
                if let Some(m) = meta {
                    if let Some(f) = m.lookup(&self.n.getitem) {
                        let f = self.bind(f, obj, &m);
                        return self.call_vec(&f, vec![idx.clone()], &[]);
                    }
                }
                if c.builtin.get() != Builtin::User {
                    return crate::builtins::generic_alias(self, obj, idx);
                }
                Err(self.type_error(format!("type '{}' is not subscriptable", c.name())))
            }
            Value::DictView(_) => Err(self.type_error(format!("'{}' object is not subscriptable", self.type_name(obj)))),
            Value::Obj(_) => {
                if let Some(f) = self.lookup_special(obj, &self.n.getitem.clone()) {
                    return self.call_vec(&f, vec![idx.clone()], &[]);
                }
                Err(self.type_error(format!("'{}' object is not subscriptable", self.type_name(obj))))
            }
            _ => {
                let tn = self.type_name(obj);
                Err(self.type_error(format!("'{}' object is not subscriptable", tn)))
            }
        }
    }

    pub fn setitem(&mut self, obj: &Value, idx: Value, v: Value) -> PyResult<()> {
        match obj {
            Value::Dict(d) => self.dict_set(d, idx, v),
            Value::List(_) | Value::ByteArray(_) => crate::builtins::seq_setitem(self, obj, &idx, v),
            Value::Instance(inst) => {
                let cls = inst.class.borrow().clone();
                match cls.lookup(&self.n.setitem) {
                    Some(f @ Value::Func(_)) => {
                        let f = self.bind(f, obj, &cls);
                        self.call_vec(&f, vec![idx, v], &[])?;
                        Ok(())
                    }
                    other => {
                        let b = self.base_of(obj);
                        if matches!(obj, Value::Instance(_)) && !matches!(b, Value::Instance(_)) {
                            return self.setitem(&b, idx, v);
                        }
                        if let Some(f) = other {
                            let f = self.bind(f, obj, &cls);
                            self.call_vec(&f, vec![idx, v], &[])?;
                            return Ok(());
                        }
                        let tn = cls.name();
                        Err(self.type_error(format!("'{}' object does not support item assignment", tn)))
                    }
                }
            }
            _ => {
                let tn = self.type_name(obj);
                Err(self.type_error(format!("'{}' object does not support item assignment", tn)))
            }
        }
    }

    pub fn delitem(&mut self, obj: &Value, idx: &Value) -> PyResult<()> {
        match obj {
            Value::Dict(d) => match self.dict_del(d, idx)? {
                Some(_) => Ok(()),
                None => Err(self.key_error(idx.clone())),
            },
            Value::List(_) | Value::ByteArray(_) => crate::builtins::seq_delitem(self, obj, idx),
            Value::Instance(inst) => {
                let cls = inst.class.borrow().clone();
                match cls.lookup(&self.n.delitem) {
                    Some(f @ Value::Func(_)) => {
                        let f = self.bind(f, obj, &cls);
                        self.call_vec(&f, vec![idx.clone()], &[])?;
                        Ok(())
                    }
                    _ => {
                        let b = self.base_of(obj);
                        if matches!(obj, Value::Instance(_)) && !matches!(b, Value::Instance(_)) {
                            return self.delitem(&b, idx);
                        }
                        let tn = cls.name();
                        Err(self.type_error(format!("'{}' object does not support item deletion", tn)))
                    }
                }
            }
            _ => {
                let tn = self.type_name(obj);
                Err(self.type_error(format!("'{}' object does not support item deletion", tn)))
            }
        }
    }

    pub fn contains(&mut self, container: &Value, item: &Value) -> PyResult<bool> {
        match container {
            Value::Str(s) => match item {
                Value::Str(i) => Ok(s.s.contains(&*i.s)),
                _ => {
                    let tn = self.type_name(item);
                    Err(self.type_error(format!("'in <string>' requires string as left operand, not {}", tn)))
                }
            },
            Value::List(l) => {
                let items = l.borrow().clone();
                for x in items.iter() {
                    if x.is(item) || self.eq(x, item)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            Value::Tuple(t) => {
                for x in t.iter() {
                    if x.is(item) || self.eq(x, item)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            Value::Dict(d) => {
                if let Value::List(_) | Value::Dict(_) | Value::Set(_) = item {
                    let tn = self.type_name(item);
                    return Err(self.type_error(format!("unhashable type: '{}'", tn)));
                }
                Ok(self.dict_get(d, item)?.is_some())
            }
            Value::Set(s) | Value::FrozenSet(s) => self.set_contains(s, item),
            Value::DictView(dv) => match dv.1 {
                0 => Ok(self.dict_get(&dv.0, item)?.is_some()),
                1 => {
                    let vals = dv.0.borrow().values();
                    for v in vals {
                        if self.eq(&v, item)? {
                            return Ok(true);
                        }
                    }
                    Ok(false)
                }
                _ => {
                    if let Value::Tuple(t) = item {
                        if t.len() == 2 {
                            if let Some(v) = self.dict_get(&dv.0, &t[0])? {
                                return self.eq(&v, &t[1]);
                            }
                        }
                    }
                    Ok(false)
                }
            },
            Value::Range(r) if matches!(item, Value::Int(_)) => {
                let i = match item {
                    Value::Int(i) => *i as i128,
                    _ => 0,
                };
                let (start, stop, step) = (r.0 as i128, r.1 as i128, r.2 as i128);
                Ok(if step > 0 { i >= start && i < stop && (i - start) % step == 0 } else { i <= start && i > stop && (start - i) % (-step) == 0 })
            }
            Value::Range(_) | Value::BigRange(_) => Ok(crate::range::find(self, container, item, false)?.is_some()),
            Value::Bytes(b) => match item {
                Value::Int(i) => Ok(b.contains(&(*i as u8))),
                Value::Bytes(sub) => Ok(sub.is_empty() || b.windows(sub.len()).any(|w| w == &sub[..])),
                _ => Err(self.type_error("a bytes-like object is required")),
            },
            Value::Instance(_) | Value::Obj(_) | Value::Class(_) => {
                if let Value::Class(c) = container {
                    let meta = c.meta.borrow().clone();
                    if let Some(m) = meta {
                        if let Some(f) = m.lookup(&self.n.contains) {
                            let f = self.bind(f, container, &m);
                            let r = self.call_vec(&f, vec![item.clone()], &[])?;
                            return self.truthy(&r);
                        }
                    }
                }
                if let Some(f) = self.lookup_special(container, &self.n.contains.clone()) {
                    let is_native = matches!(&f, Value::Method(m) if matches!(m.1, Value::Native(_)));
                    if !is_native {
                        let r = self.call_vec(&f, vec![item.clone()], &[])?;
                        return self.truthy(&r);
                    }
                }
                let b = self.base_of(container);
                if !matches!(b, Value::Instance(_) | Value::Obj(_) | Value::Class(_)) {
                    return self.contains(&b, item);
                }
                let it = self.iter(container)?;
                while let Some(x) = self.next(&it)? {
                    if x.is(item) || self.eq(&x, item)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            _ => {
                let it = match self.iter(container) {
                    Ok(it) => it,
                    Err(_) => {
                        let tn = self.type_name(container);
                        return Err(self.type_error(format!("argument of type '{}' is not iterable", tn)));
                    }
                };
                while let Some(x) = self.next(&it)? {
                    if self.eq(&x, item)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
        }
    }

    // ------------------------------------------------------------------
    // str and repr

    pub fn repr(&mut self, v: &Value) -> PyResult<String> {
        crate::format::repr(self, v)
    }

    pub fn str_of(&mut self, v: &Value) -> PyResult<String> {
        match v {
            Value::Str(s) => Ok(s.s.clone()),
            Value::Instance(_) => {
                let cls = self.class_of(v);
                if let Some(f) = cls.lookup(&self.n.str_) {
                    if !matches!(f, Value::Native(_)) {
                        let f = self.bind(f, v, &cls);
                        let r = self.call_vec(&f, vec![], &[])?;
                        return match r {
                            Value::Str(s) => Ok(s.s.clone()),
                            Value::Instance(_) => match self.base_of(&r) {
                                Value::Str(s) => Ok(s.s.clone()),
                                _ => Err(self.type_error("__str__ returned non-string")),
                            },
                            _ => {
                                let tn = self.type_name(&r);
                                Err(self.type_error(format!("__str__ returned non-string (type {})", tn)))
                            }
                        };
                    }
                }
                if cls.builtin.get() == Builtin::BaseException {
                    return crate::builtins::exc_str(self, v);
                }
                let b = self.base_of(v);
                if !matches!(b, Value::Instance(_)) && cls.lookup(&self.n.repr).map(|r| matches!(r, Value::Native(_))).unwrap_or(true) {
                    return self.str_of(&b);
                }
                self.repr(v)
            }
            _ => crate::format::str_builtin(self, v),
        }
    }

    pub fn format_value(&mut self, v: &Value, spec: &str) -> PyResult<String> {
        if let Value::Instance(_) = v {
            let cls = self.class_of(v);
            if let Some(f) = cls.lookup(&self.n.format) {
                if !matches!(f, Value::Native(_)) {
                    let f = self.bind(f, v, &cls);
                    let r = self.call_vec(&f, vec![Value::str(spec)], &[])?;
                    return match r {
                        Value::Str(s) => Ok(s.s.clone()),
                        _ => Err(self.type_error("__format__ must return a str")),
                    };
                }
            }
            if spec.is_empty() {
                return self.str_of(v);
            }
            let b = self.base_of(v);
            if matches!(v, Value::Instance(_)) && !matches!(b, Value::Instance(_)) {
                // a str/int subclass with its own __str__ formats as the base
                return crate::format::format_spec(self, &b, spec);
            }
            let tn = self.type_name(v);
            return Err(self.type_error(format!("unsupported format string passed to {}.__format__", tn)));
        }
        if spec.is_empty() {
            return self.str_of(v);
        }
        crate::format::format_spec(self, v, spec)
    }
}

/// whether instances of `cls` have a __dict__: not when every class in the
/// MRO up to object declares __slots__ without "__dict__"
pub fn has_dict(cls: &Rc<Class>) -> bool {
    for c in cls.mro.borrow().iter() {
        match &*c.slots.borrow() {
            Some(s) => {
                if s.iter().any(|x| &**x == "__dict__") {
                    return true;
                }
            }
            None => {
                if c.builtin.get() == Builtin::BaseException {
                    return true;
                }
                let user = matches!(c.dict.borrow().get_s("__module__"), Some(Value::Str(m)) if &*m.s != "builtins");
                if user {
                    return true;
                }
            }
        }
    }
    false
}
