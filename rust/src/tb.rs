// SPDX-License-Identifier: MIT
//! Tracebacks, frames and __slots__ members: the native objects behind
//! `exc.__traceback__`, `tb.tb_frame` and `C.slot`.
use crate::builtins::{def, ps, Kw};
use crate::value::*;
use crate::vm::{Frame, Vm};
use std::cell::RefCell;
use std::rc::Rc;

impl Vm {
    /// a class for native objects, made on first use
    pub fn native_class(&mut self, name: &'static str) -> Rc<Class> {
        if let Some(c) = self.classes.get(name) {
            return c.clone();
        }
        let c = Class::new(name, vec![self.t.object.clone()], Builtin::Obj);
        *c.mro.borrow_mut() = vec![c.clone(), self.t.object.clone()];
        c.dict.borrow_mut().set_s("__module__", Value::str("builtins"));
        match name {
            "member_descriptor" => {
                def(&c, "member_descriptor.__get__", member_get);
                def(&c, "member_descriptor.__set__", member_set);
                def(&c, "member_descriptor.__delete__", member_delete);
                def(&c, "member_descriptor.__repr__", member_repr);
            }
            "getset_descriptor" | "builtin_member_descriptor" => {
                def(&c, "getset_descriptor.__get__", getset_get);
                def(&c, "getset_descriptor.__set__", getset_set);
                def(&c, "getset_descriptor.__repr__", getset_repr);
                if name == "builtin_member_descriptor" {
                    *c.name.borrow_mut() = ps("member_descriptor");
                    *c.qualname.borrow_mut() = ps("member_descriptor");
                }
            }
            "frame" => {
                def(&c, "frame.clear", frame_clear);
                def(&c, "frame.__repr__", frame_repr);
            }
            "traceback" => {
                crate::builtins::def_static(&c, "traceback.__new__", tb_new);
            }
            _ => {}
        }
        self.classes.insert(name, c.clone());
        c
    }

    /// types.SimpleNamespace
    pub fn namespace_class(&mut self) -> Rc<Class> {
        if let Some(c) = self.classes.get("SimpleNamespace") {
            return c.clone();
        }
        let c = Class::new("SimpleNamespace", vec![self.t.object.clone()], Builtin::User);
        *c.mro.borrow_mut() = vec![c.clone(), self.t.object.clone()];
        c.dict.borrow_mut().set_s("__module__", Value::str("types"));
        def(&c, "SimpleNamespace.__init__", ns_init);
        def(&c, "SimpleNamespace.__repr__", ns_repr);
        def(&c, "SimpleNamespace.__eq__", ns_eq);
        def(&c, "SimpleNamespace.__reduce__", ns_reduce);
        def(&c, "SimpleNamespace.__replace__", ns_replace);
        c.dict.borrow_mut().set_s("__hash__", Value::None);
        self.classes.insert("SimpleNamespace", c.clone());
        c
    }

    pub fn native_obj(&mut self, class: &'static str, kind: ObjKind) -> Value {
        let c = self.native_class(class);
        let o = Rc::new(NativeObj { class: c, gc: Default::default(), kind: RefCell::new(kind) });
        crate::gc::track_obj(&o);
        Value::Obj(o)
    }

    pub fn frame_obj(&mut self, f: &Frame, line: u32) -> Value {
        let locals = match &f.ns {
            Some(ns) => Value::Dict(ns.clone()),
            None => Value::None,
        };
        self.native_obj("frame", ObjKind::Frame { code: f.code.clone(), line, globals: f.globals.clone(), locals, back: Value::None })
    }

    /// the frame object of running frame `idx` (an index into self.frames),
    /// made once and shared by tracebacks and sys._getframe
    pub fn frame_value(&mut self, idx: usize) -> Value {
        if idx >= self.frames.len() || idx >= self.frame_objs.len() {
            return Value::None;
        }
        let mut lo = idx;
        while lo > 0 && self.frame_objs[lo].is_undef() {
            lo -= 1;
        }
        for i in lo..=idx {
            if !self.frame_objs[i].is_undef() {
                continue;
            }
            // SAFETY: see Vm::current_frame
            let f = unsafe { &*self.frames[i] };
            let line = f.code.line_at(f.ip.saturating_sub(1)).unwrap_or(0);
            let back = if i > 0 { self.frame_objs[i - 1].clone() } else { Value::None };
            let back = if back.is_undef() { Value::None } else { back };
            let v = self.frame_obj(f, line);
            if let Value::Obj(o) = &v {
                if let ObjKind::Frame { back: b, .. } = &mut *o.kind.borrow_mut() {
                    *b = back;
                }
            }
            self.frame_objs[i] = v;
        }
        self.frame_objs[idx].clone()
    }

    /// adds the frame `f` at `line` to the front of exc.__traceback__
    pub fn tb_here(&mut self, exc: &Value, f: &Frame, line: u32) {
        if let Value::Instance(i) = exc {
            let next = i.extra.borrow().get_str(&self.n.tb).cloned().unwrap_or(Value::None);
            let top = self.frames.len();
            let frame = if top > 0 && std::ptr::eq(self.frames[top - 1] as *const Frame, f) {
                let v = self.frame_value(top - 1);
                if let Value::Obj(o) = &v {
                    if let ObjKind::Frame { line: l, .. } = &mut *o.kind.borrow_mut() {
                        *l = line;
                    }
                }
                v
            } else {
                self.frame_obj(f, line)
            };
            let lasti = (f.ip.saturating_sub(1) * 2) as i64;
            let tb = self.native_obj("traceback", ObjKind::Traceback { frame, line, next, lasti });
            i.extra.borrow_mut().set_str(self.n.tb.clone(), tb);
        }
    }

    /// (filename, line, function) of each traceback entry, outermost first
    pub fn tb_entries(&self, tb: &Value) -> Vec<(String, i64, String)> {
        let mut out = vec![];
        let mut cur = tb.clone();
        while let Value::Obj(o) = &cur {
            let next = match &*o.kind.borrow() {
                ObjKind::Traceback { frame, line, next, .. } => {
                    if let Value::Obj(fo) = frame {
                        if let ObjKind::Frame { code, .. } = &*fo.kind.borrow() {
                            out.push((code.filename.to_string(), *line as i64, code.name.s.to_string()));
                        }
                    }
                    next.clone()
                }
                _ => Value::None,
            };
            cur = next;
            if out.len() > 100000 {
                break;
            }
        }
        out
    }
}

pub fn exc_extra(vm: &Vm, inst: &Instance, name: &str) -> Value {
    let _ = vm;
    inst.extra.borrow().get_s(name).unwrap_or(Value::None)
}

/// attributes of tracebacks, frames and members
pub fn obj_attr(vm: &mut Vm, o: &NativeObj, name: &str) -> PyResult<Option<Value>> {
    Ok(match &*o.kind.borrow() {
        ObjKind::Traceback { frame, line, next, lasti } => match name {
            "tb_frame" => Some(frame.clone()),
            // 0: an op without a location (co_linetable says so)
            "tb_lineno" => Some(if *line == 0 { Value::None } else { Value::Int(*line as i64) }),
            "tb_next" => Some(next.clone()),
            "tb_lasti" => Some(Value::Int(*lasti)),
            _ => None,
        },
        ObjKind::Frame { code, line, globals, locals, back } => match name {
            "f_code" => Some(Value::Code(code.clone())),
            "f_lineno" => Some(if *line == 0 { Value::None } else { Value::Int(*line as i64) }),
            "f_globals" => Some(Value::Dict(globals.clone())),
            "f_locals" => Some(match locals {
                Value::None => Value::dict(Dict::new()),
                l => l.clone(),
            }),
            "f_builtins" => Some(Value::Dict(vm.builtins.clone())),
            "f_back" => Some(back.clone()),
            "f_lasti" => Some(Value::Int(0)),
            "f_trace" => Some(Value::None),
            _ => None,
        },
        ObjKind::GetSet(n, c, _) => match name {
            "__name__" => Some(Value::Str(n.clone())),
            "__qualname__" => Some(Value::string(format!("{}.{}", c.name(), n.s))),
            "__doc__" => Some(Value::None),
            "__objclass__" => Some(Value::Class(c.clone())),
            _ => None,
        },
        ObjKind::Member(n, c) => match name {
            "__name__" => Some(Value::Str(n.clone())),
            "__qualname__" => Some(Value::string(format!("{}.{}", c.qualname.borrow().s, n.s))),
            "__doc__" => Some(Value::None),
            "__objclass__" => Some(Value::Class(c.clone())),
            _ => None,
        },
        _ => None,
    })
}

/// tb.tb_next = ...
pub fn obj_setattr(vm: &mut Vm, o: &NativeObj, name: &str, v: Value) -> PyResult<bool> {
    let mut k = o.kind.borrow_mut();
    if let ObjKind::TupleGetter(_, doc) = &mut *k {
        if name == "__doc__" {
            *doc = v;
            return Ok(true);
        }
    }
    if let ObjKind::Traceback { next, .. } = &mut *k {
        if name == "tb_next" {
            let ok = match &v {
                Value::None => true,
                Value::Obj(n) => matches!(&*n.kind.try_borrow().map_err(|_| vm.value_error("traceback loop"))?, ObjKind::Traceback { .. }),
                _ => false,
            };
            if !ok {
                drop(k);
                let tn = vm.type_name(&v);
                return Err(vm.type_error(format!("expected traceback object or None, got '{}'", tn)));
            }
            // a loop would make the chain endless
            let mut cur = v.clone();
            while let Value::Obj(c) = &cur {
                if std::ptr::eq(&**c, o) {
                    drop(k);
                    return Err(vm.value_error("traceback loop detected"));
                }
                let n = match c.kind.try_borrow() {
                    Ok(r) => match &*r {
                        ObjKind::Traceback { next, .. } => next.clone(),
                        _ => Value::None,
                    },
                    Err(_) => Value::None,
                };
                cur = n;
            }
            *next = v;
            return Ok(true);
        }
    }
    Ok(false)
}

fn tb_new(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    if a.len() != 5 {
        return Err(vm.type_error(format!("traceback() takes exactly 4 arguments ({} given)", a.len().saturating_sub(1))));
    }
    let next = a[1].clone();
    if !next.is_none() && !matches!(&next, Value::Obj(o) if matches!(&*o.kind.borrow(), ObjKind::Traceback { .. })) {
        let tn = vm.type_name(&next);
        return Err(vm.type_error(format!("expected traceback object or None, got '{}'", tn)));
    }
    if !matches!(&a[2], Value::Obj(o) if matches!(&*o.kind.borrow(), ObjKind::Frame { .. })) {
        let tn = vm.type_name(&a[2]);
        return Err(vm.type_error(format!("TracebackType() argument 'tb_frame' must be frame, not {}", tn)));
    }
    let lasti = vm.to_index(&a[3])?;
    let line = vm.to_index(&a[4])? as u32;
    Ok(vm.native_obj("traceback", ObjKind::Traceback { frame: a[2].clone(), line, next, lasti }))
}

fn frame_clear(_vm: &mut Vm, _a: &[Value], _k: Kw) -> PyResult<Value> {
    Ok(Value::None)
}

fn frame_repr(_vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    if let Value::Obj(o) = &a[0] {
        if let ObjKind::Frame { code, line, .. } = &*o.kind.borrow() {
            return Ok(Value::string(format!("<frame at {:#x}, file '{}', line {}, code {}>", Rc::as_ptr(o) as usize, code.filename, line, code.name.s)));
        }
    }
    Ok(Value::str("<frame>"))
}

fn member_name(a: &Value) -> Rc<PyStr> {
    if let Value::Obj(o) = a {
        if let ObjKind::Member(n, _) = &*o.kind.borrow() {
            return n.clone();
        }
    }
    ps("?")
}

pub fn member_owner(a: &Value) -> Option<Rc<Class>> {
    if let Value::Obj(o) = a {
        if let ObjKind::Member(_, c) = &*o.kind.borrow() {
            return Some(c.clone());
        }
    }
    None
}

/// the member descriptor of slot `name` on `owner`
pub fn new_member(vm: &mut Vm, name: Rc<PyStr>, owner: &Rc<Class>) -> Value {
    vm.native_obj("member_descriptor", ObjKind::Member(name, owner.clone()))
}

fn check_owner(vm: &mut Vm, d: &Value, obj: &Value) -> PyResult<Rc<Instance>> {
    let owner = member_owner(d);
    if let Value::Instance(i) = obj {
        let c = i.class.borrow().clone();
        if owner.as_ref().map_or(true, |o| c.is_subclass(o)) {
            return Ok(i.clone());
        }
    }
    let n = member_name(d);
    let on = owner.map(|o| o.name().to_string()).unwrap_or_default();
    let tn = vm.type_name(obj);
    Err(vm.type_error(format!("descriptor '{}' for '{}' objects doesn't apply to a '{}' object", n.s, on, tn)))
}

pub fn member_read(vm: &mut Vm, d: &Value, obj: &Value) -> PyResult<Value> {
    let i = check_owner(vm, d, obj)?;
    let n = member_name(d);
    let v = i.extra.borrow().get_str(&n).cloned();
    match v {
        Some(v) => Ok(v),
        None => {
            let cn = i.class.borrow().name().to_string();
            Err(vm.attr_error(format!("'{}' object has no attribute '{}'", cn, n.s)))
        }
    }
}

pub fn member_write(vm: &mut Vm, d: &Value, obj: &Value, v: Option<Value>) -> PyResult<()> {
    let i = check_owner(vm, d, obj)?;
    let n = member_name(d);
    match v {
        Some(v) => {
            i.extra.borrow_mut().set_str(n, v);
        }
        None => {
            if i.extra.borrow_mut().remove_str(&n).is_none() {
                return Err(vm.attr_error(n.s.to_string()));
            }
        }
    }
    Ok(())
}

fn member_get(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    if a.len() < 2 || a[1].is_none() {
        return Ok(a[0].clone());
    }
    member_read(vm, &a[0], &a[1])
}

fn member_set(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 3)?;
    member_write(vm, &a[0], &a[1], Some(a[2].clone()))?;
    Ok(Value::None)
}

fn member_delete(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 2)?;
    member_write(vm, &a[0], &a[1], None)?;
    Ok(Value::None)
}

fn member_repr(_vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let n = member_name(&a[0]);
    let on = member_owner(&a[0]).map(|o| o.name().to_string()).unwrap_or_default();
    Ok(Value::string(format!("<member '{}' of '{}' objects>", n.s, on)))
}

pub fn is_member(v: &Value) -> bool {
    matches!(v, Value::Obj(o) if matches!(&*o.kind.borrow(), ObjKind::Member(..)))
}


/// puts descriptors for attributes the type serves itself into its dict
pub fn add_getsets(vm: &mut Vm, c: &Rc<Class>, getsets: &[&str], members: &[&str]) {
    for (names, member) in [(getsets, false), (members, true)] {
        for n in names {
            let cls = if member { "builtin_member_descriptor" } else { "getset_descriptor" };
            let d = vm.native_obj(cls, ObjKind::GetSet(ps(n), c.clone(), member));
            c.dict.borrow_mut().set_s(n, d);
        }
    }
}

fn getset_parts(a: &Value) -> (Rc<PyStr>, Option<Rc<Class>>) {
    if let Value::Obj(o) = a {
        if let ObjKind::GetSet(n, c, _) = &*o.kind.borrow() {
            return (n.clone(), Some(c.clone()));
        }
    }
    (ps("?"), None)
}

fn getset_get(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    if a.len() < 2 || a[1].is_none() {
        return Ok(a[0].clone());
    }
    let (n, _) = getset_parts(&a[0]);
    vm.getattr(&a[1], &n)
}

fn getset_set(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 3)?;
    let (n, _) = getset_parts(&a[0]);
    vm.setattr(&a[1], &n, a[2].clone())?;
    Ok(Value::None)
}

fn getset_repr(_vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let (n, c) = getset_parts(&a[0]);
    let member = matches!(&a[0], Value::Obj(o) if matches!(&*o.kind.borrow(), ObjKind::GetSet(_, _, true)));
    let cn = c.map(|c| c.name().to_string()).unwrap_or_default();
    Ok(Value::string(format!("<{} '{}' of '{}' objects>", if member { "member" } else { "attribute" }, n.s, cn)))
}

fn ns_dict(vm: &mut Vm, v: &Value) -> PyResult<Rc<RefCell<Dict>>> {
    match v {
        Value::Instance(i) => Ok(i.dict.clone()),
        _ => Err(vm.type_error("descriptor requires a 'types.SimpleNamespace' object")),
    }
}

fn ns_init(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    if a.len() > 2 {
        return Err(vm.type_error(format!("SimpleNamespace expected at most 1 positional argument, got {}", a.len() - 1)));
    }
    let d = ns_dict(vm, &a[0])?;
    if let Some(m) = a.get(1) {
        // a mapping or an iterable of pairs, as dict() takes them
        let dict_t = Value::Class(vm.t.dict.clone());
        let src = match m {
            Value::Dict(_) => m.clone(),
            other => vm.call_vec(&dict_t, vec![other.clone()], &[])?,
        };
        let items = match vm.base_of(&src) {
            Value::Dict(src) => src.borrow().iter().map(|e| (e.key.clone(), e.value.clone())).collect::<Vec<_>>(),
            _ => vec![],
        };
        for (key, v) in items {
            match &key {
                Value::Str(s) => d.borrow_mut().set_str(s.clone(), v),
                _ => {
                    let tn = vm.type_name(&key);
                    return Err(vm.type_error(format!("keywords must be strings, not '{}'", tn)));
                }
            }
        }
    }
    for (n, v) in k {
        d.borrow_mut().set_str(n.clone(), v.clone());
    }
    Ok(Value::None)
}

fn ns_repr(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let d = ns_dict(vm, &a[0])?;
    let cls = vm.class_of(&a[0]);
    let name = if cls.name().as_str() == "SimpleNamespace" && matches!(cls.dict.borrow().get_s("__module__"), Some(Value::Str(m)) if &*m.s == "types") { "namespace".to_string() } else { cls.name().to_string() };
    let id = Rc::as_ptr(&d) as usize;
    if vm.repr_stack.contains(&id) {
        return Ok(Value::string(format!("{}(...)", name)));
    }
    vm.repr_stack.push(id);
    let items: Vec<(Value, Value)> = d.borrow().iter().map(|e| (e.key.clone(), e.value.clone())).collect();
    let mut parts = vec![];
    for (key, v) in items {
        let r = vm.repr(&v);
        let r = match r {
            Ok(r) => r,
            Err(e) => {
                vm.repr_stack.pop();
                return Err(e);
            }
        };
        parts.push(format!("{}={}", key.as_str().unwrap_or("?"), r));
    }
    vm.repr_stack.pop();
    Ok(Value::string(format!("{}({})", name, parts.join(", "))))
}

fn ns_eq(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 2)?;
    let nc = vm.namespace_class();
    let both = [&a[0], &a[1]].iter().all(|v| matches!(v, Value::Instance(i) if i.class.borrow().is_subclass(&nc)));
    if !both {
        return Ok(Value::NotImplemented);
    }
    let x = ns_dict(vm, &a[0])?;
    let y = ns_dict(vm, &a[1])?;
    let r = vm.eq(&Value::Dict(x), &Value::Dict(y))?;
    Ok(Value::Bool(r))
}

fn ns_reduce(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let d = ns_dict(vm, &a[0])?;
    let cls = vm.class_of(&a[0]);
    let copy = d.borrow().copy();
    Ok(Value::tuple(vec![Value::Class(cls), Value::tuple(vec![]), Value::dict(copy)]))
}

fn ns_replace(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    if a.len() > 1 {
        return Err(vm.type_error("__replace__() takes no positional arguments"));
    }
    let d = ns_dict(vm, &a[0])?;
    let cls = vm.class_of(&a[0]);
    let mut kw: Vec<(Rc<PyStr>, Value)> = d.borrow().iter().filter_map(|e| e.key.as_str().map(|s| (ps(s), e.value.clone()))).collect();
    for (n, v) in k {
        match kw.iter_mut().find(|(m, _)| m.s == n.s) {
            Some(slot) => slot.1 = v.clone(),
            None => kw.push((n.clone(), v.clone())),
        }
    }
    vm.call_vec(&Value::Class(cls), vec![], &kw)
}

/// warnings.warn(msg, category) from native code
pub fn warn(vm: &mut Vm, category: &str, msg: &str) -> PyResult<()> {
    let m = vm.import_one("warnings")?;
    let f = vm.getattr(&m, &ps("warn"))?;
    let c = Value::Class(vm.exc_class(category));
    vm.call_vec(&f, vec![Value::Str(ps(msg)), c], &[])?;
    Ok(())
}
