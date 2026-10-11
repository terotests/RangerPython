// SPDX-License-Identifier: MIT
//! Native modules: sys, math (mathmod.rs), time, os, os.path, json, _rpyrandom (the core of _random), and file
//! objects. The rest of the standard library is Python (lib/, pylib/).

use crate::bigint::BigInt;
use crate::builtins::*;
use crate::value::*;
use crate::vm::Vm;
use std::cell::RefCell;
use std::rc::Rc;

pub use crate::mathmod::{erf, fsum, isqrt_big};

pub(crate) fn module(name: &str) -> (Rc<RefCell<Dict>>, Value) {
    let d = new_dict();
    d.borrow_mut().set_s("__name__", Value::str(name));
    d.borrow_mut().set_s("__doc__", Value::None);
    d.borrow_mut().set_s("__package__", Value::str(""));
    let m = Value::Module(Rc::new(Module { name: ps(name), dict: d.clone() }));
    (d, m)
}

pub(crate) fn put_fns(d: &Rc<RefCell<Dict>>, fns: &[(&'static str, NativeFnPtr)]) {
    let mut d = d.borrow_mut();
    for (n, f) in fns {
        let short = n.rsplit('.').next().unwrap();
        d.set_s(short, native(n, *f));
    }
}

pub fn native_module(vm: &mut Vm, name: &str) -> PyResult<Option<Value>> {
    Ok(Some(match name {
        "sys" => sys_module(vm),
        "math" => crate::mathmod::math_module(),
        "_statistics" => crate::mathmod::statistics_module(),
        "_time" => time_module(),
        "os" => os_module(vm)?,
        "os.path" | "posixpath" => os_path_module(),
        "json" => json_module(vm),
        "_rpyrandom" => random_module(),
        "_abc" => crate::abcmod::abc_module(),
        "_rpydecimal" => crate::decimalmod::decimal_module(),
        "_rpydis" => crate::dismod::dis_module(),
        "_rpyinterp" => crate::interpmod::interp_module(),
        "builtins" => Value::Module(Rc::new(Module { name: ps("builtins"), dict: vm.builtins.clone() })),
        "_hashlib" => hashlib_module(),
        "_struct" => struct_module(),
        "_unicodedata" => unicodedata_module(),
        "_rpyweak" => weak_module(),
        "_rpygc" => crate::gc::gc_module(),
        "_rpyitertools" => itertools_module(),
        "_rpycollections" => collections_module(vm),
        "_rpyinternal" => internal_module(),
        "_rpyprocess" => crate::process::module(),
        "_rpyframe" => crate::tb::frame_module(),
        "_testcapi" => crate::testcapi::testcapi_module(vm)?,
        "_testlimitedcapi" => crate::testcapi::testlimitedcapi_module(),
        _ => return Ok(None),
    }))
}

// -------------------------------------------------------------------------
// weak references: what weakref.ref holds

fn weak_module() -> Value {
    let (d, m) = module("_rpyweak");
    put_fns(&d, &[("_rpyweak.new", weak_new), ("_rpyweak.get", weak_get)]);
    m
}

/// a weak handle to an instance, function, class, generator, set or native
/// object; anything else is kept as is
fn weak_new(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let t = match &a[0] {
        Value::Instance(i) => WeakTarget::Inst(Rc::downgrade(i)),
        Value::Func(f) => WeakTarget::Func(Rc::downgrade(f)),
        Value::Class(c) => WeakTarget::Class(Rc::downgrade(c)),
        Value::Gen(g) => WeakTarget::Gen(Rc::downgrade(g)),
        Value::Set(d) => WeakTarget::Set(Rc::downgrade(d), false),
        Value::FrozenSet(d) => WeakTarget::Set(Rc::downgrade(d), true),
        Value::Obj(o) => WeakTarget::Obj(Rc::downgrade(o)),
        other => return Ok(other.clone()),
    };
    Ok(vm.native_obj("weakhandle", ObjKind::Weak(t)))
}

fn weak_get(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    if let Value::Obj(o) = &a[0] {
        if let ObjKind::Weak(t) = &*o.kind.borrow() {
            return Ok(match t {
                WeakTarget::Inst(w) => w.upgrade().map(Value::Instance),
                WeakTarget::Func(w) => w.upgrade().map(Value::Func),
                WeakTarget::Class(w) => w.upgrade().map(Value::Class),
                WeakTarget::Gen(w) => w.upgrade().map(Value::Gen),
                WeakTarget::Set(w, false) => w.upgrade().map(Value::Set),
                WeakTarget::Set(w, true) => w.upgrade().map(Value::FrozenSet),
                WeakTarget::Obj(w) => w.upgrade().map(Value::Obj),
            }
            .unwrap_or(Value::None));
        }
    }
    Ok(a[0].clone())
}

// -------------------------------------------------------------------------
// support for the _testinternalcapi stand-in of the test harness

fn internal_module() -> Value {
    let (d, m) = module("_rpyinternal");
    put_fns(&d, &[("_rpyinternal.dict_getitem_knownhash", dict_getitem_knownhash), ("_rpyinternal.get_recursion_depth", get_recursion_depth)]);
    m
}

/// get_recursion_depth(): how many Python frames are running, the caller's
/// included
fn get_recursion_depth(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::check_nargs(vm, "get_recursion_depth", a, 0, 0)?;
    Ok(Value::Int(vm.frames.len() as i64))
}

/// dict_getitem_knownhash(d, key, hash): d[key] looked up under `hash`, with
/// neither __getitem__ nor __missing__ of a subclass involved
fn dict_getitem_knownhash(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    if a.len() != 3 {
        return Err(vm.type_error(format!("dict_getitem_knownhash expected 3 arguments, got {}", a.len())));
    }
    let h = vm.to_index(&a[2])?;
    let d = match vm.base_of(&a[0]) {
        Value::Dict(d) => d,
        _ => return Err(vm.err("SystemError", "bad argument to internal function")),
    };
    match vm.dict_get_hashed(&d, h as u64, &a[1])? {
        Some(v) => Ok(v),
        None => Err(vm.key_error(a[1].clone())),
    }
}

// -------------------------------------------------------------------------
// support for _collections (pylib/_collections.py): the namedtuple field type

fn collections_module(vm: &mut Vm) -> Value {
    let (d, m) = module("_rpycollections");
    d.borrow_mut().set_s("_tuplegetter", Value::Class(tuplegetter_class(vm)));
    m
}

pub fn tuplegetter_class(vm: &mut Vm) -> Rc<Class> {
    if let Some(c) = vm.classes.get("_tuplegetter") {
        return c.clone();
    }
    let c = vm.native_class("_tuplegetter");
    c.dict.borrow_mut().set_s("__module__", Value::str("_collections"));
    crate::builtins::def_static(&c, "_tuplegetter.__new__", tg_new);
    for (n, f) in [("_tuplegetter.__get__", tg_get as NativeFnPtr), ("_tuplegetter.__set__", tg_set), ("_tuplegetter.__delete__", tg_delete), ("_tuplegetter.__repr__", tg_repr), ("_tuplegetter.__reduce__", tg_reduce)] {
        crate::builtins::def(&c, n, f);
    }
    c
}

fn tg_parts(v: &Value) -> Option<(usize, Value)> {
    match v {
        Value::Obj(o) => match &*o.kind.borrow() {
            ObjKind::TupleGetter(i, d) => Some((*i, d.clone())),
            _ => None,
        },
        _ => None,
    }
}

pub fn is_tuplegetter(v: &Value) -> bool {
    tg_parts(v).is_some()
}

fn tg_new(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    if a.len() != 3 || !k.is_empty() {
        return Err(vm.type_error(format!("_tuplegetter expected 2 arguments, got {}", a.len().saturating_sub(1) + k.len())));
    }
    let i = vm.to_index(&a[1])?;
    if i < 0 {
        return Err(vm.value_error("index must be non-negative"));
    }
    let class = match &a[0] {
        Value::Class(c) => c.clone(),
        _ => tuplegetter_class(vm),
    };
    Ok(Value::Obj(Rc::new(NativeObj { class, gc: Default::default(), kind: RefCell::new(ObjKind::TupleGetter(i as usize, a[2].clone())) })))
}

/// the field read by a _tuplegetter `d` from tuple `obj`
pub fn tuplegetter_read(vm: &mut Vm, d: &Value, obj: &Value) -> PyResult<Value> {
    let (i, _) = tg_parts(d).ok_or_else(|| vm.type_error("expected a _tuplegetter"))?;
    match vm.base_of(obj) {
        Value::Tuple(t) => match t.get(i) {
            Some(v) => Ok(v.clone()),
            None => Err(vm.index_error("tuple index out of range")),
        },
        _ => {
            let tn = vm.type_name(obj);
            Err(vm.type_error(format!("descriptor for index '{}' for tuple subclasses doesn't apply to '{}' object", i, tn)))
        }
    }
}

fn tg_get(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    if a.len() < 2 || a[1].is_none() {
        return Ok(a[0].clone());
    }
    tuplegetter_read(vm, &a[0], &a[1])
}

fn tg_set(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 3)?;
    Err(vm.attr_error("can't set attribute"))
}

fn tg_delete(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 2)?;
    Err(vm.attr_error("can't delete attribute"))
}

fn tg_repr(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let (i, d) = tg_parts(&a[0]).ok_or_else(|| vm.type_error("expected a _tuplegetter"))?;
    let tn = vm.type_name(&a[0]);
    let dr = vm.repr(&d)?;
    Ok(Value::string(format!("{}({}, {})", tn, i, dr)))
}

fn tg_reduce(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let (i, d) = tg_parts(&a[0]).ok_or_else(|| vm.type_error("expected a _tuplegetter"))?;
    let c = vm.class_of(&a[0]);
    Ok(Value::tuple(vec![Value::Class(c), Value::tuple(vec![Value::Int(i as i64), d])]))
}

// -------------------------------------------------------------------------
// support for itertools (pylib/itertools.py)

fn itertools_module() -> Value {
    let (d, m) = module("_rpyitertools");
    put_fns(&d, &[("_rpyitertools.immutable", it_immutable), ("_rpyitertools.reuse", it_reuse)]);
    // methods, which bind to the instance
    for (name, full, f) in [("chain_next", "chain.__next__", it_chain_next as NativeFnPtr), ("tee_next", "_tee.__next__", it_tee_next), ("repeat_next", "repeat.__next__", it_repeat_next)] {
        d.borrow_mut().set_s(name, Value::Native(Rc::new(NativeFn::new(full, f, Value::None))));
    }
    m
}

/// the attribute names the native itertools methods use, interned once
const IT_FIELDS: [&str; 10] = ["_source", "_active", "_index", "_dataobj", "_values", "_running", "_it", "_jumplink", "_cnt", "_element"];

fn it_name(name: &str) -> Rc<PyStr> {
    thread_local! {
        static NAMES: Vec<Rc<PyStr>> = IT_FIELDS.iter().map(|n| crate::value::intern(n)).collect();
    }
    let i = IT_FIELDS.iter().position(|n| *n == name).expect("an itertools field");
    NAMES.with(|n| n[i].clone())
}

/// an attribute an itertools object keeps in its __dict__, read directly as
/// the C code reads its struct fields; anything else through getattr
fn it_field(vm: &mut Vm, o: &Value, name: &str) -> PyResult<Value> {
    let key = it_name(name);
    if let Value::Instance(i) = o {
        if let Some(v) = i.dict.borrow().get_str(&key) {
            return Ok(v.clone());
        }
    }
    vm.getattr(o, &key)
}

fn it_set(vm: &mut Vm, o: &Value, name: &str, v: Value) -> PyResult<()> {
    let key = it_name(name);
    match o {
        Value::Instance(i) => {
            i.dict.borrow_mut().set_str(key, v);
            Ok(())
        }
        _ => vm.setattr(o, &key, v),
    }
}

/// chain.__next__, as CPython's chain_next: an exhausted iterable costs no
/// Python frame or StopIteration object
fn it_chain_next(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let me = &a[0];
    loop {
        let source = it_field(vm, me, "_source")?;
        if source.is_none() {
            break;
        }
        let mut active = it_field(vm, me, "_active")?;
        if active.is_none() {
            let iterable = match vm.next(&source) {
                Ok(Some(x)) => x,
                Ok(None) => {
                    vm.last_return = None;
                    it_set(vm, me, "_source", Value::None)?;
                    break;
                }
                Err(e) => {
                    it_set(vm, me, "_source", Value::None)?;
                    return Err(e);
                }
            };
            active = match vm.iter(&iterable) {
                Ok(i) => i,
                Err(e) => {
                    it_set(vm, me, "_source", Value::None)?;
                    return Err(e);
                }
            };
            it_set(vm, me, "_active", active.clone())?;
        }
        if let Some(v) = vm.next(&active)? {
            return Ok(v);
        }
        vm.last_return = None;
        it_set(vm, me, "_active", Value::None)?;
    }
    Err(vm.stop_iteration(Value::None))
}

/// repeat.__next__, as CPython's repeat_next: _element while _cnt (-1
/// for no limit) lasts
fn it_repeat_next(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let me = &a[0];
    let cnt = it_field(vm, me, "_cnt")?;
    match cnt {
        Value::Int(0) => return Err(vm.stop_iteration(Value::None)),
        Value::Int(n) if n > 0 => it_set(vm, me, "_cnt", Value::Int(n - 1))?,
        Value::Int(_) => {}
        _ => return Err(vm.type_error("repeat count is not an int")),
    }
    it_field(vm, me, "_element")
}

/// the tee link size, _LINKCELLS in pylib/itertools.py
const TEE_LINKCELLS: i64 = 57;

/// _tee.__next__, as CPython's tee_next: item `_index` of the shared
/// _tee_dataobject, which reads one more from the source when it is new
fn it_tee_next(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let me = &a[0];
    let mut index = match it_field(vm, me, "_index")? {
        Value::Int(i) => i,
        _ => return Err(vm.type_error("tee index is not an int")),
    };
    let mut data = it_field(vm, me, "_dataobj")?;
    if index >= TEE_LINKCELLS {
        let jump = vm.getattr(&data, &it_name("_jumplink"))?;
        data = vm.call_vec(&jump, vec![], &[])?;
        it_set(vm, me, "_dataobj", data.clone())?;
        index = 0;
        it_set(vm, me, "_index", Value::Int(0))?;
    }
    let Value::List(values) = it_field(vm, &data, "_values")? else {
        return Err(vm.type_error("tee data values are not a list"));
    };
    let have = values.borrow().get(index as usize).cloned();
    let value = match have {
        Some(v) => v,
        None => {
            let running = it_field(vm, &data, "_running")?;
            if vm.truthy(&running)? {
                return Err(vm.err("RuntimeError", "cannot re-enter the tee iterator"));
            }
            let source = it_field(vm, &data, "_it")?;
            it_set(vm, &data, "_running", Value::Bool(true))?;
            let r = vm.next(&source);
            it_set(vm, &data, "_running", Value::Bool(false))?;
            let v = match r? {
                Some(v) => v,
                None => {
                    vm.last_return = None;
                    return Err(vm.stop_iteration(Value::None));
                }
            };
            crate::gc::list_store(&values, &v);
            values.borrow_mut().push(v.clone());
            v
        }
    };
    it_set(vm, me, "_index", Value::Int(index + 1))?;
    Ok(value)
}

/// mark a class as standing in for a static C type, whose attributes are fixed
fn it_immutable(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    match &a[0] {
        Value::Class(c) => c.immutable.set(true),
        _ => return Err(vm.type_error("immutable() argument must be a type")),
    }
    Ok(a[0].clone())
}

/// reuse(obj, items): a tuple of `items`, kept as obj._result; the tuple
/// already there is refilled in place while obj holds the only reference to
/// it, as CPython's combinatoric iterators do
fn it_reuse(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 2)?;
    let items = vm.to_vec(&a[1])?;
    let Value::Instance(inst) = &a[0] else {
        return Ok(Value::tuple(items));
    };
    let key = PyStr::new("_result".to_string());
    let mut d = inst.dict.borrow_mut();
    if let Some(Value::Tuple(t)) = d.get_str_mut(&key) {
        crate::gc::retrack(t);
        if let Some(v) = Rc::get_mut(t) {
            if v.len() == items.len() {
                *v = items;
                return Ok(Value::Tuple(t.clone()));
            }
        }
    }
    let t = Value::tuple(items);
    d.set_s("_result", t.clone());
    Ok(t)
}

// -------------------------------------------------------------------------
// standard streams and files

fn stream(vm: &mut Vm, name: &str) -> Value {
    Value::Obj(Rc::new(NativeObj { class: vm.t.file.clone(), gc: Default::default(), kind: RefCell::new(ObjKind::Other(Value::str(name))) }))
}

pub fn is_std_stream(v: &Value) -> bool {
    if let Value::Obj(o) = v {
        if let ObjKind::Other(Value::Str(s)) = &*o.kind.borrow() {
            return &*s.s == "<stdout>" || &*s.s == "<stderr>";
        }
    }
    false
}

pub fn is_stderr(v: &Value) -> bool {
    if let Value::Obj(o) = v {
        if let ObjKind::Other(Value::Str(s)) = &*o.kind.borrow() {
            return &*s.s == "<stderr>";
        }
    }
    false
}

fn install_file_class(vm: &mut Vm) {
    let c = vm.t.file.clone();
    if c.dict.borrow().get_s("read").is_some() {
        return;
    }
    let fs: &[(&'static str, NativeFnPtr)] = &[
        ("TextIOWrapper.read", f_read),
        ("TextIOWrapper.readline", f_readline),
        ("TextIOWrapper.readlines", f_readlines),
        ("TextIOWrapper.write", f_write),
        ("TextIOWrapper.writelines", f_writelines),
        ("TextIOWrapper.close", f_close),
        ("TextIOWrapper.flush", f_flush),
        ("TextIOWrapper.__enter__", f_enter),
        ("TextIOWrapper.__exit__", f_exit),
        ("TextIOWrapper.__iter__", f_enter),
        ("TextIOWrapper.__next__", f_next),
        ("TextIOWrapper.seek", f_seek),
        ("TextIOWrapper.tell", f_tell),
        ("TextIOWrapper.readable", f_readable),
        ("TextIOWrapper.writable", f_writable),
        ("TextIOWrapper.seekable", f_readable),
        ("TextIOWrapper.isatty", f_isatty),
        ("TextIOWrapper.fileno", f_fileno),
        ("TextIOWrapper.truncate", f_truncate),
        ("TextIOWrapper.__repr__", f_repr),
    ];
    for (n, f) in fs {
        def(&c, n, *f);
    }
    c.dict.borrow_mut().set_s("__module__", Value::str("_io"));
}

pub fn open_file(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    install_file_class(vm);
    let file = match arg_or_kw(a, 0, k, "file") {
        Some(Value::Str(s)) if s.s.contains('\0') => return Err(vm.value_error("embedded null character")),
        Some(Value::Str(s)) => s.s.clone(),
        Some(Value::Bytes(b)) if b.contains(&0) => return Err(vm.value_error("embedded null byte")),
        Some(Value::Bytes(b)) => String::from_utf8_lossy(&b).into_owned(),
        Some(v) => {
            if let Some(f) = vm.getattr_opt(&v, &ps("__fspath__"))? {
                let r = vm.call_vec(&f, vec![], &[])?;
                vm.str_of(&r)?
            } else {
                let tn = vm.type_name(&v);
                return Err(vm.type_error(format!("expected str, bytes or os.PathLike object, not {}", tn)));
            }
        }
        None => return Err(vm.type_error("open() missing required argument 'file' (pos 1)")),
    };
    let mode = match arg_or_kw(a, 1, k, "mode") {
        Some(Value::Str(s)) => s.s.clone(),
        _ => "r".into(),
    };
    let binary = mode.contains('b');
    let text_arg = |i: usize, name: &str, default: &str| match arg_or_kw(a, i, k, name) {
        Some(Value::Str(s)) => s.s.clone(),
        _ => default.to_string(),
    };
    let (encoding, errors) = (text_arg(3, "encoding", "UTF-8"), text_arg(4, "errors", "strict"));
    let path = vm.fs.normalize(&file);
    let not_found = |vm: &mut Vm| {
        let cls = vm.exc_class("FileNotFoundError");
        let e = vm.new_exc(&cls, vec![Value::Int(2), Value::str("No such file or directory"), Value::str(&file)]);
        Exc { value: e }
    };
    if vm.fs.is_dir(&path) && !vm.fs.is_file(&path) {
        let cls = vm.exc_class("IsADirectoryError");
        let e = vm.new_exc(&cls, vec![Value::Int(21), Value::str("Is a directory"), Value::str(&file)]);
        return Err(Exc { value: e });
    }
    let (buf, pos, writable) = if mode.contains('r') {
        let t = match vm.fs.read(&path) {
            Some(t) => t,
            None => return Err(not_found(vm)),
        };
        (t, 0, mode.contains('+'))
    } else if mode.contains('w') {
        if !vm.fs.is_dir(&parent(&path)) {
            return Err(not_found(vm));
        }
        vm.fs.write(&path, vec![]);
        (vec![], 0, true)
    } else if mode.contains('a') {
        if !vm.fs.is_dir(&parent(&path)) {
            return Err(not_found(vm));
        }
        let t = vm.fs.read(&path).unwrap_or_default();
        let n = t.len();
        vm.fs.write(&path, t.clone());
        (t, n, true)
    } else if mode.contains('x') {
        if vm.fs.is_file(&path) {
            let cls = vm.exc_class("FileExistsError");
            let e = vm.new_exc(&cls, vec![Value::Int(17), Value::str("File exists"), Value::str(&file)]);
            return Err(Exc { value: e });
        }
        vm.fs.write(&path, vec![]);
        (vec![], 0, true)
    } else {
        return Err(vm.value_error(format!("invalid mode: '{}'", mode)));
    };
    let kind = ObjKind::File { path: file.clone(), mode, buf, pos, closed: false, writable, binary, encoding, errors };
    Ok(Value::Obj(Rc::new(NativeObj { class: vm.t.file.clone(), gc: Default::default(), kind: RefCell::new(kind) })))
}

fn parent(p: &str) -> String {
    match p.rfind('/') {
        Some(0) | None => "/".into(),
        Some(i) => p[..i].to_string(),
    }
}

fn file_of(vm: &mut Vm, v: &Value) -> PyResult<Rc<NativeObj>> {
    match v {
        Value::Obj(o) => Ok(o.clone()),
        _ => Err(vm.type_error("not a file")),
    }
}

fn closed_err(vm: &mut Vm) -> Exc {
    vm.value_error("I/O operation on closed file.")
}

fn is_utf8(encoding: &str) -> bool {
    matches!(encoding.to_ascii_lowercase().replace('_', "-").as_str(), "utf-8" | "utf8" | "u8")
}

/// Takes the next part of a file from `pos` on: everything, at most `max`
/// characters (bytes for a binary file) when `max` >= 0, and only up to the
/// end of the line when `line`. A text file's part is decoded.
fn file_take(vm: &mut Vm, o: &NativeObj, max: i64, line: bool) -> PyResult<Value> {
    let mut kind = o.kind.borrow_mut();
    let ObjKind::File { buf, pos, closed, binary, mode, encoding, errors, .. } = &mut *kind else {
        return Ok(Value::str(""));
    };
    if *closed {
        drop(kind);
        return Err(closed_err(vm));
    }
    if !mode.contains('r') && !mode.contains('+') {
        drop(kind);
        return Err(vm.err("UnsupportedOperation", "not readable"));
    }
    let rest = &buf[(*pos).min(buf.len())..];
    let line_end = |b: &[u8]| if line { b.iter().position(|c| *c == b'\n').map_or(b.len(), |i| i + 1) } else { b.len() };
    if *binary {
        let mut end = line_end(rest);
        if max >= 0 {
            end = end.min(max as usize);
        }
        *pos += end;
        return Ok(Value::Bytes(Rc::new(rest[..end].to_vec())));
    }
    let (text, used) = if is_utf8(encoding) {
        // a newline byte is a newline character; characters start at non-continuation bytes
        let mut end = line_end(rest);
        if max >= 0 {
            if let Some((i, _)) = rest[..end].iter().enumerate().filter(|(_, c)| **c & 0xc0 != 0x80).nth(max as usize) {
                end = i;
            }
        }
        let text = match std::str::from_utf8(&rest[..end]) {
            Ok(s) => s.to_string(),
            Err(_) => crate::strmethods::decode_bytes(vm, &rest[..end], encoding, errors)?.into_string_lossy(),
        };
        (text, end)
    } else {
        let all = crate::strmethods::decode_bytes(vm, rest, encoding, errors)?.into_string_lossy();
        let mut end = if line { all.find('\n').map_or(all.len(), |i| i + 1) } else { all.len() };
        if max >= 0 {
            if let Some((i, _)) = all[..end].char_indices().nth(max as usize) {
                end = i;
            }
        }
        let text = all[..end].to_string();
        let used = if end == all.len() { rest.len() } else { crate::strmethods::encode_str(vm, &crate::builtins::ps(&text), encoding, errors)?.len() };
        (text, used)
    };
    *pos += used;
    Ok(Value::string(text))
}

fn f_read(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let o = file_of(vm, &a[0])?;
    let n = match a.get(1) {
        Some(Value::None) | None => -1,
        Some(v) => vm.to_index(v)?,
    };
    file_take(vm, &o, n, false)
}

/// the next line, or an empty str or bytes at the end
fn read_line(vm: &mut Vm, o: &NativeObj) -> PyResult<Value> {
    file_take(vm, o, -1, true)
}

fn is_empty_read(v: &Value) -> bool {
    match v {
        Value::Str(s) => s.s.is_empty(),
        Value::Bytes(b) => b.is_empty(),
        _ => true,
    }
}

fn f_readline(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let o = file_of(vm, &a[0])?;
    let n = match a.get(1) {
        Some(Value::None) | None => -1,
        Some(v) => vm.to_index(v)?,
    };
    file_take(vm, &o, n, true)
}

fn f_readlines(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let o = file_of(vm, &a[0])?;
    let mut out = vec![];
    loop {
        let l = read_line(vm, &o)?;
        if is_empty_read(&l) {
            break;
        }
        out.push(l);
    }
    Ok(Value::list(out))
}

fn f_next(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let o = file_of(vm, &a[0])?;
    let l = read_line(vm, &o)?;
    if is_empty_read(&l) {
        return Err(vm.stop_iteration(Value::None));
    }
    Ok(l)
}

fn f_write(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 2)?;
    let o = file_of(vm, &a[0])?;
    margs(vm, "write", a, 1, 1)?;
    let text: Option<Rc<PyStr>> = match &a[1] {
        Value::Str(s) => Some(s.clone()),
        Value::Bytes(_) | Value::ByteArray(_) => None,
        v => match vm.base_of(v) {
            Value::Str(s) => Some(s),
            _ => {
                let tn = vm.type_name(v);
                return Err(vm.type_error(format!("write() argument must be str, not {}", tn)));
            }
        },
    };
    // a std stream
    if let ObjKind::Other(Value::Str(s)) = &*o.kind.borrow() {
        if let Some(t) = &text {
            crate::strmethods::check_utf8(vm, t)?;
        }
        let text = text.map(|t| t.s.clone()).unwrap_or_else(|| bytes_to_latin1(&a[1]));
        if &*s.s == "<stderr>" {
            vm.write_err(&text);
        } else {
            vm.write_out(&text);
        }
        return Ok(Value::Int(text.chars().count() as i64));
    }
    let (closed, writable, binary, encoding, errors) = match &*o.kind.borrow() {
        ObjKind::File { closed, writable, binary, encoding, errors, .. } => (*closed, *writable, *binary, encoding.clone(), errors.clone()),
        _ => return Ok(Value::Int(0)),
    };
    if closed {
        return Err(closed_err(vm));
    }
    if !writable {
        return Err(vm.err("UnsupportedOperation", "not writable"));
    }
    // a text file writes its characters encoded; a binary one the bytes
    let (data, n) = match (binary, text) {
        (true, Some(_)) => return Err(vm.type_error("a bytes-like object is required, not 'str'")),
        (false, None) => return Err(vm.type_error("write() argument must be str, not bytes")),
        (false, Some(t)) => {
            let n = t.char_len();
            (crate::strmethods::encode_str(vm, &t, &encoding, &errors)?, n)
        }
        (true, None) => {
            let b = crate::builtins::bytes_of(vm, &a[1])?;
            let n = b.len();
            (b, n)
        }
    };
    let (path, buf) = {
        let mut kind = o.kind.borrow_mut();
        let ObjKind::File { path, buf, pos, mode, .. } = &mut *kind else { unreachable!() };
        if mode.contains('a') {
            *pos = buf.len();
        }
        if *pos > buf.len() {
            buf.resize(*pos, 0);
        }
        let end = (*pos + data.len()).min(buf.len());
        buf.splice(*pos..end, data.iter().copied());
        *pos += data.len();
        (path.clone(), buf.clone())
    };
    vm.fs.write(&path, buf);
    Ok(Value::Int(n as i64))
}

fn f_writelines(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 2)?;
    let items = vm.to_vec(&a[1])?;
    for x in items {
        f_write(vm, &[a[0].clone(), x], &[])?;
    }
    Ok(Value::None)
}

fn f_close(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let o = file_of(vm, &a[0])?;
    if let ObjKind::File { closed, .. } = &mut *o.kind.borrow_mut() {
        *closed = true;
    }
    Ok(Value::None)
}

/// what an open file does when it is lost: a ResourceWarning naming it,
/// then it is closed
pub fn file_finalize(vm: &mut Vm, o: &Rc<NativeObj>) {
    let v = Value::Obj(o.clone());
    let r = f_repr(vm, std::slice::from_ref(&v), &[]).and_then(|r| {
        let msg = format!("unclosed file {}", vm.str_of(&r)?);
        let m = vm.import_one("warnings")?;
        let warn = vm.getattr(&m, &ps("warn"))?;
        let cat = Value::Class(vm.exc_class("ResourceWarning"));
        vm.call_vec(&warn, vec![Value::string(msg), cat], &[(ps("source"), v.clone())])
    });
    if let ObjKind::File { closed, .. } = &mut *o.kind.borrow_mut() {
        *closed = true;
    }
    if let Err(e) = r {
        vm.unraisable(e, &v);
    }
}

fn f_flush(_vm: &mut Vm, _a: &[Value], _k: Kw) -> PyResult<Value> {
    Ok(Value::None)
}

fn f_enter(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let o = file_of(vm, &a[0])?;
    if let ObjKind::File { closed: true, .. } = &*o.kind.borrow() {
        return Err(vm.value_error("I/O operation on closed file."));
    }
    Ok(a[0].clone())
}

fn f_exit(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    f_close(vm, a, k)?;
    Ok(Value::Bool(false))
}

fn f_seek(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 2)?;
    let o = file_of(vm, &a[0])?;
    let off = vm.to_index(&a[1])?;
    let whence = match a.get(2) {
        Some(v) => vm.to_index(v)?,
        None => 0,
    };
    if let ObjKind::File { pos, buf, .. } = &mut *o.kind.borrow_mut() {
        let base = match whence {
            1 => *pos as i64,
            2 => buf.len() as i64,
            _ => 0,
        };
        *pos = (base + off).clamp(0, buf.len() as i64) as usize;
        return Ok(Value::Int(*pos as i64));
    }
    Ok(Value::Int(0))
}

fn f_tell(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let o = file_of(vm, &a[0])?;
    if let ObjKind::File { pos, .. } = &*o.kind.borrow() {
        return Ok(Value::Int(*pos as i64));
    }
    Ok(Value::Int(0))
}

fn f_truncate(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let o = file_of(vm, &a[0])?;
    let size = match a.get(1) {
        Some(Value::None) | None => None,
        Some(v) => Some(vm.to_index(v)? as usize),
    };
    let target = if let ObjKind::File { pos, buf, path, .. } = &mut *o.kind.borrow_mut() {
        let n = size.unwrap_or(*pos).min(buf.len());
        buf.truncate(n);
        Some((path.clone(), buf.clone(), n))
    } else {
        None
    };
    if let Some((p, b, n)) = target {
        vm.fs.write(&p, b);
        return Ok(Value::Int(n as i64));
    }
    Ok(Value::Int(0))
}

fn f_readable(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let o = file_of(vm, &a[0])?;
    let r = matches!(&*o.kind.borrow(), ObjKind::File { mode, .. } if mode.contains('r') || mode.contains('+'));
    Ok(Value::Bool(r))
}

fn f_writable(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let o = file_of(vm, &a[0])?;
    let r = match &*o.kind.borrow() {
        ObjKind::File { writable, .. } => *writable,
        ObjKind::Other(_) => true,
        _ => false,
    };
    Ok(Value::Bool(r))
}

fn f_isatty(_vm: &mut Vm, _a: &[Value], _k: Kw) -> PyResult<Value> {
    Ok(Value::Bool(false))
}

fn f_fileno(_vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    Ok(Value::Int(if is_stderr(&a[0]) {
        2
    } else if is_std_stream(&a[0]) {
        1
    } else {
        3
    }))
}

fn f_repr(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let o = file_of(vm, &a[0])?;
    let r = match &*o.kind.borrow() {
        ObjKind::File { path, mode, binary, encoding, .. } => {
            if *binary {
                format!("<_io.BufferedReader name='{}'>", path)
            } else {
                format!("<_io.TextIOWrapper name='{}' mode='{}' encoding='{}'>", path, mode, encoding)
            }
        }
        ObjKind::Other(Value::Str(s)) => format!("<_io.TextIOWrapper name='{}' mode='w' encoding='utf-8'>", s.s),
        _ => "<file>".into(),
    };
    Ok(Value::string(r))
}

/// attributes of native objects (files)
pub fn obj_attr(_vm: &mut Vm, _obj: &Value, o: &Rc<NativeObj>, name: &str) -> PyResult<Option<Value>> {
    Ok(match &*o.kind.borrow() {
        ObjKind::File { path, mode, closed, encoding, errors, .. } => match name {
            "name" => Some(Value::str(path)),
            "mode" => Some(Value::str(mode)),
            "closed" => Some(Value::Bool(*closed)),
            "encoding" => Some(Value::str(encoding)),
            "errors" => Some(Value::str(errors)),
            "newlines" => Some(Value::None),
            "line_buffering" => Some(Value::Bool(false)),
            _ => None,
        },
        ObjKind::TupleGetter(_, d) if name == "__doc__" => Some(d.clone()),
        ObjKind::Other(Value::Str(s)) if s.s.starts_with('<') => match name {
            "name" => Some(Value::Str(s.clone())),
            "mode" => Some(Value::str("w")),
            "closed" => Some(Value::Bool(false)),
            "encoding" => Some(Value::str("utf-8")),
            "errors" => Some(Value::str("strict")),
            _ => None,
        },
        _ => None,
    })
}

// -------------------------------------------------------------------------
// sys

/// sys's struct sequences, made by pylib/_structseq.py once sys exists
fn sys_structseqs(vm: &mut Vm, d: &Rc<RefCell<Dict>>) {
    let g = vm.main_globals.clone();
    let m = match vm.import("_structseq", &Value::None, 0, &g) {
        Ok(m) => m,
        Err(_) => return,
    };
    let make = match vm.getattr(&m, &ps("make")) {
        Ok(f) => f,
        Err(_) => return,
    };
    let f = |x: f64| Value::Float(x);
    let i = |x: i64| Value::Int(x);
    let s = |x: &str| Value::str(x);
    let specs: Vec<(&str, Vec<&str>, Vec<Value>)> = vec![
        ("version_info", vec!["major", "minor", "micro", "releaselevel", "serial"], vec![i(3), i(13), i(0), s("final"), i(0)]),
        (
            "float_info",
            vec!["max", "max_exp", "max_10_exp", "min", "min_exp", "min_10_exp", "dig", "mant_dig", "epsilon", "radix", "rounds"],
            vec![f(f64::MAX), i(1024), i(308), f(f64::MIN_POSITIVE), i(-1021), i(-307), i(15), i(53), f(f64::EPSILON), i(2), i(1)],
        ),
        ("int_info", vec!["bits_per_digit", "sizeof_digit", "default_max_str_digits", "str_digits_check_threshold"], vec![i(30), i(4), i(4300), i(640)]),
        (
            "hash_info",
            vec!["width", "modulus", "inf", "nan", "imag", "algorithm", "hash_bits", "seed_bits", "cutoff"],
            vec![i(64), i((1 << 61) - 1), i(314159), i(0), i(1000003), s("siphash13"), i(64), i(128), i(0)],
        ),
        ("thread_info", vec!["name", "lock", "version"], vec![s("pthread"), s("mutex+cond"), Value::None]),
        (
            "flags",
            vec!["debug", "inspect", "interactive", "optimize", "dont_write_bytecode", "no_user_site", "no_site", "ignore_environment", "verbose", "bytes_warning", "quiet", "hash_randomization", "isolated", "dev_mode", "utf8_mode", "warn_default_encoding", "safe_path", "int_max_str_digits", "gil", "thread_inherit_context", "context_aware_warnings"],
            {
                let h = &vm.host;
                let b = |x: bool| i(x as i64);
                vec![
                    i(0),
                    b(h.inspect),
                    b(h.inspect),
                    i(h.optimize),
                    i(1),
                    i(1),
                    i(0),
                    b(h.ignore_environment),
                    i(h.verbose),
                    i(h.bytes_warning),
                    b(h.quiet),
                    i(1),
                    b(h.isolated),
                    Value::Bool(h.dev_mode),
                    i(1),
                    i(0),
                    Value::Bool(h.safe_path),
                    i(4300),
                    i(1),
                    i(0),
                    i(0),
                ]
            },
        ),
    ];
    for (name, fields, values) in specs {
        let fl = Value::list(fields.iter().map(|x| Value::str(x)).collect());
        let n = if name == "flags" { Value::Int(18) } else { Value::None };
        let kw = vec![(ps("module"), Value::str("sys")), (ps("n_sequence"), n)];
        if let Ok(v) = vm.call_vec(&make, vec![Value::str(name), fl, Value::tuple(values)], &kw) {
            d.borrow_mut().set_s(name, v);
        }
    }
    // sys.UnraisableHookArgs is the type, not an instance
    let fields = ["exc_type", "exc_value", "exc_traceback", "err_msg", "object"];
    let fl = Value::list(fields.iter().map(|x| Value::str(x)).collect());
    let kw = vec![(ps("module"), Value::str("sys"))];
    if let Ok(v) = vm.call_vec(&make, vec![Value::str("UnraisableHookArgs"), fl, Value::tuple(vec![Value::None; 5])], &kw) {
        d.borrow_mut().set_s("UnraisableHookArgs", Value::Class(vm.class_of(&v)));
    }
}

/// sys.stdlib_module_names: the modules of CPython 3.13's standard library
const STDLIB_MODULE_NAMES: &str = concat!(
    "__future__ _abc _aix_support _android_support _apple_support _ast _asyncio _bisect _blake2 _bz2 ",
    "_codecs _codecs_cn _codecs_hk _codecs_iso2022 _codecs_jp _codecs_kr _codecs_tw _collections ",
    "_collections_abc _colorize _compat_pickle _compression _contextvars _csv _ctypes _curses ",
    "_curses_panel _datetime _dbm _decimal _elementtree _frozen_importlib _frozen_importlib_external ",
    "_functools _gdbm _hashlib _heapq _imp _interpchannels _interpqueues _interpreters _io ",
    "_ios_support _json _locale _lsprof _lzma _markupbase _md5 _multibytecodec _multiprocessing ",
    "_opcode _opcode_metadata _operator _osx_support _overlapped _pickle _posixshmem _posixsubprocess ",
    "_py_abc _pydatetime _pydecimal _pyio _pylong _pyrepl _queue _random _scproxy _sha1 _sha2 _sha3 ",
    "_signal _sitebuiltins _socket _sqlite3 _sre _ssl _stat _statistics _string _strptime _struct ",
    "_suggestions _symtable _sysconfig _thread _threading_local _tkinter _tokenize _tracemalloc ",
    "_typing _uuid _warnings _weakref _weakrefset _winapi _wmi _zoneinfo abc antigravity argparse ",
    "array ast asyncio atexit base64 bdb binascii bisect builtins bz2 cProfile calendar cmath cmd ",
    "code codecs codeop collections colorsys compileall concurrent configparser contextlib ",
    "contextvars copy copyreg csv ctypes curses dataclasses datetime dbm decimal difflib dis doctest ",
    "email encodings ensurepip enum errno faulthandler fcntl filecmp fileinput fnmatch fractions ",
    "ftplib functools gc genericpath getopt getpass gettext glob graphlib grp gzip hashlib heapq hmac ",
    "html http idlelib imaplib importlib inspect io ipaddress itertools json keyword linecache locale ",
    "logging lzma mailbox marshal math mimetypes mmap modulefinder msvcrt multiprocessing netrc nt ",
    "ntpath nturl2path numbers opcode operator optparse os pathlib pdb pickle pickletools pkgutil ",
    "platform plistlib poplib posix posixpath pprint profile pstats pty pwd py_compile pyclbr pydoc ",
    "pydoc_data pyexpat queue quopri random re readline reprlib resource rlcompleter runpy sched ",
    "secrets select selectors shelve shlex shutil signal site smtplib socket socketserver sqlite3 ",
    "sre_compile sre_constants sre_parse ssl stat statistics string stringprep struct subprocess ",
    "symtable sys sysconfig syslog tabnanny tarfile tempfile termios textwrap this threading time ",
    "timeit tkinter token tokenize tomllib trace traceback tracemalloc tty turtle turtledemo types ",
    "typing unicodedata unittest urllib uuid venv warnings wave weakref webbrowser winreg winsound ",
    "wsgiref xml xmlrpc zipapp zipfile zipimport zlib zoneinfo ",
);

fn sys_module(vm: &mut Vm) -> Value {
    install_file_class(vm);
    let (d, m) = module("sys");
    let argv: Vec<Value> = vm.argv.iter().map(|s| Value::str(s)).collect();
    let stdout = stream(vm, "<stdout>");
    let stderr = stream(vm, "<stderr>");
    let stdin = stream(vm, "<stdin>");
    {
        let mut b = d.borrow_mut();
        b.set_s("argv", Value::list(if argv.is_empty() { vec![Value::str("")] } else { argv }));
        b.set_s("version", Value::str("3.13.0 (Ranger Python, Rust)"));
        b.set_s("version_info", Value::tuple(vec![Value::Int(3), Value::Int(13), Value::Int(0), Value::str("final"), Value::Int(0)]));
        b.set_s("hexversion", Value::Int(0x030d00f0));
        b.set_s("platform", Value::str("linux"));
        b.set_s("byteorder", Value::str("little"));
        // repr(float) is the shortest string that reads back the same
        b.set_s("float_repr_style", Value::str("short"));
        b.set_s("maxsize", Value::Int(i64::MAX));
        b.set_s("maxunicode", Value::Int(0x10ffff));
        b.set_s("executable", Value::str(&vm.host.executable));
        b.set_s("warnoptions", Value::list(vm.host.warnoptions.iter().map(|w| Value::str(w)).collect()));
        let mut xo = Dict::new();
        for (k, v) in &vm.host.xoptions {
            xo.set_s(k, v.as_deref().map(Value::str).unwrap_or(Value::Bool(true)));
        }
        b.set_s("_xoptions", Value::dict(xo));
        for k in ["prefix", "base_prefix", "exec_prefix", "base_exec_prefix"] {
            b.set_s(k, Value::str("/"));
        }
        b.set_s("path", Value::list(vec![Value::str("/")]));
        b.set_s("modules", Value::Dict(vm.modules.clone()));
        b.set_s("stdout", stdout.clone());
        b.set_s("stderr", stderr.clone());
        b.set_s("stdin", stdin);
        b.set_s("__stdout__", stdout);
        b.set_s("__stderr__", stderr);
        let names = Rc::new(RefCell::new(Dict::new()));
        for n in STDLIB_MODULE_NAMES.split_whitespace() {
            // hashing a str cannot fail
            let _ = vm.set_add(&names, Value::str(n));
        }
        b.set_s("stdlib_module_names", Value::FrozenSet(names));
        b.set_s("builtin_module_names", Value::tuple(["builtins", "sys", "math", "time", "json"].iter().map(|s| Value::str(s)).collect()));
        b.set_s("dont_write_bytecode", Value::Bool(true));
        let mut fi = Dict::new();
        for (k, v) in [("max", Value::Float(f64::MAX)), ("min", Value::Float(f64::MIN_POSITIVE)), ("epsilon", Value::Float(f64::EPSILON)), ("dig", Value::Int(15)), ("mant_dig", Value::Int(53)), ("max_exp", Value::Int(1024)), ("min_exp", Value::Int(-1021)), ("radix", Value::Int(2))] {
            fi.set_s(k, v);
        }
        b.set_s("float_info", Value::Module(Rc::new(Module { name: ps("sys.float_info"), dict: Rc::new(RefCell::new(fi)) })));
        let mut ii = Dict::new();
        ii.set_s("bits_per_digit", Value::Int(30));
        ii.set_s("sizeof_digit", Value::Int(4));
        ii.set_s("default_max_str_digits", Value::Int(4300));
        ii.set_s("str_digits_check_threshold", Value::Int(640));
        b.set_s("int_info", Value::Module(Rc::new(Module { name: ps("sys.int_info"), dict: Rc::new(RefCell::new(ii)) })));
        let mut fl = Dict::new();
        for n in ["debug", "inspect", "interactive", "optimize", "dont_write_bytecode", "no_user_site", "no_site", "ignore_environment", "verbose", "bytes_warning", "quiet", "hash_randomization", "isolated", "dev_mode", "utf8_mode", "warn_default_encoding", "safe_path", "int_max_str_digits"] {
            fl.set_s(n, Value::Int(0));
        }
        b.set_s("flags", Value::Module(Rc::new(Module { name: ps("sys.flags"), dict: Rc::new(RefCell::new(fl)) })));
        let mut imp = Dict::new();
        imp.set_s("name", Value::str("rangerpython"));
        imp.set_s("cache_tag", Value::None);
        let ns = vm.namespace_class();
        let inst = Instance { class: RefCell::new(ns), dict: Rc::new(RefCell::new(imp)), base: RefCell::new(Value::None), extra: RefCell::new(Dict::new()), gc: Default::default() };
        let inst = Rc::new(inst);
        crate::gc::track_inst(&inst);
        b.set_s("implementation", Value::Instance(inst));
    }
    put_fns(
        &d,
        &[
            ("sys.__breakpointhook__", sys_breakpointhook),
            ("sys.exit", sys_exit),
            ("sys.breakpointhook", sys_breakpointhook),
            ("sys.getrecursionlimit", sys_getrecursionlimit),
            ("sys.get_int_max_str_digits", sys_get_int_max_str_digits),
            ("sys.set_int_max_str_digits", sys_set_int_max_str_digits),
            ("sys.setrecursionlimit", sys_setrecursionlimit),
            ("sys.exc_info", sys_exc_info),
            ("sys.excepthook", sys_excepthook),
            ("sys.unraisablehook", sys_unraisablehook),
            ("sys.__unraisablehook__", sys_unraisablehook),
            ("sys.__excepthook__", sys_excepthook),
            ("sys.unraisablehook", sys_unraisablehook),
            ("sys.__unraisablehook__", sys_unraisablehook),
            ("sys.exception", sys_exception),
            ("sys.getsizeof", sys_getsizeof),
            ("sys.intern", sys_intern),
            ("sys.getdefaultencoding", sys_encoding),
            ("sys.getfilesystemencoding", sys_encoding),
            ("sys._getframe", sys_getframe),
            ("sys.getrefcount", sys_getrefcount),
            ("sys.settrace", sys_settrace),
            ("sys.setprofile", sys_setprofile),
            ("sys.gettrace", sys_noop),
            ("sys.audit", sys_noop),
            ("sys.is_finalizing", sys_false),
        ],
    );
    sys_structseqs(vm, &d);
    m
}

fn sys_exit(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let cls = vm.exc_class("SystemExit");
    let v = vm.new_exc(&cls, a.to_vec());
    Err(Exc { value: v })
}
fn sys_get_int_max_str_digits(_vm: &mut Vm, _a: &[Value], _k: Kw) -> PyResult<Value> {
    Ok(Value::Int(crate::format::max_str_digits() as i64))
}
fn sys_set_int_max_str_digits(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let v = match crate::builtins::arg_or_kw(a, 0, k, "maxdigits") {
        Some(v) => v,
        None => return Err(vm.type_error("set_int_max_str_digits() missing required argument 'maxdigits' (pos 1)")),
    };
    let n = vm.to_index(&v)?;
    if n != 0 && n < 640 {
        return Err(vm.value_error("maxdigits must be >= 640 or 0 for unlimited"));
    }
    crate::format::MAX_STR_DIGITS.store(n as usize, std::sync::atomic::Ordering::Relaxed);
    Ok(Value::None)
}
fn sys_getrecursionlimit(vm: &mut Vm, _a: &[Value], _k: Kw) -> PyResult<Value> {
    Ok(Value::Int(vm.max_depth as i64))
}
fn sys_setrecursionlimit(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let n = vm.to_index(&a[0])?;
    if n < 1 {
        return Err(vm.value_error("recursion limit must be greater or equal than 1"));
    }
    // the Rust stack bounds how deep frames may go
    vm.max_depth = (n as u32).min(crate::MAX_DEPTH);
    Ok(Value::None)
}
fn sys_exc_info(vm: &mut Vm, _a: &[Value], _k: Kw) -> PyResult<Value> {
    match vm.handled.last().cloned() {
        Some(e) => {
            let c = vm.class_of(&e);
            let tb = vm.getattr(&e, &vm.n.tb.clone()).unwrap_or(Value::None);
            Ok(Value::tuple(vec![Value::Class(c), e, tb]))
        }
        None => Ok(Value::tuple(vec![Value::None, Value::None, Value::None])),
    }
}
/// prints the exception and its traceback to sys.stderr
fn sys_excepthook(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    if a.len() != 3 {
        return Err(vm.type_error(format!("excepthook expected 3 arguments, got {}", a.len())));
    }
    let m = vm.import("traceback", &Value::None, 0, &vm.main_globals.clone())?;
    let f = vm.getattr(&m, &ps("print_exception"))?;
    vm.call_vec(&f, a.to_vec(), &[])?;
    Ok(Value::None)
}
/// prints an exception that could not be raised, and what it came from
fn sys_unraisablehook(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let arg = |vm: &mut Vm, n: &str| vm.getattr(&a[0], &ps(n));
    let (msg, obj) = (arg(vm, "err_msg")?, arg(vm, "object")?);
    let head = if msg.is_none() { "Exception ignored in".to_string() } else { vm.str_of(&msg)? };
    let line = if obj.is_none() { format!("{}\n", head) } else { format!("{}: {}\n", head, vm.repr(&obj)?) };
    let sys = vm.import("sys", &Value::None, 0, &vm.main_globals.clone())?;
    let err = vm.getattr(&sys, &ps("stderr"))?;
    let w = vm.getattr(&err, &ps("write"))?;
    vm.call_vec(&w, vec![Value::string(line)], &[])?;
    let (t, v, tb) = (arg(vm, "exc_type")?, arg(vm, "exc_value")?, arg(vm, "exc_traceback")?);
    let m = vm.import("traceback", &Value::None, 0, &vm.main_globals.clone())?;
    let f = vm.getattr(&m, &ps("print_exception"))?;
    vm.call_vec(&f, vec![t, v, tb], &[])?;
    Ok(Value::None)
}
fn sys_exception(vm: &mut Vm, _a: &[Value], _k: Kw) -> PyResult<Value> {
    Ok(vm.handled.last().cloned().unwrap_or(Value::None))
}
fn sys_getsizeof(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let n = match &a[0] {
        Value::Str(s) => 49 + s.s.len(),
        Value::List(l) => 56 + 8 * l.borrow().len(),
        Value::Tuple(t) => 40 + 8 * t.len(),
        Value::Dict(d) => 64 + 24 * d.borrow().len(),
        Value::Int(_) => 28,
        Value::Float(_) => 24,
        _ => {
            let _ = vm;
            48
        }
    };
    Ok(Value::Int(n as i64))
}

fn sys_intern(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    match &a[0] {
        Value::Str(s) => Ok(Value::Str(crate::value::intern_str(s))),
        other => {
            let tn = vm.type_name(other);
            Err(vm.type_error(format!("intern() argument must be str, not {}", tn)))
        }
    }
}
fn sys_encoding(_vm: &mut Vm, _a: &[Value], _k: Kw) -> PyResult<Value> {
    Ok(Value::str("utf-8"))
}
/// the strong count of the argument's allocation; None, the booleans and
/// small ints are immortal as in CPython, and other values without an
/// allocation of their own count the variable and the argument
fn sys_getrefcount(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let n = match &a[0] {
        Value::None | Value::NotImplemented | Value::Ellipsis | Value::Bool(_) => u32::MAX as usize,
        Value::Int(i) if (-5..=256).contains(i) => u32::MAX as usize,
        v => crate::value::strong_count(v).unwrap_or(2),
    };
    Ok(Value::Int(n as i64))
}
fn sys_noop(_vm: &mut Vm, _a: &[Value], _k: Kw) -> PyResult<Value> {
    Ok(Value::None)
}
/// sys.settrace(f) and sys.setprofile(f): there are no trace or profile
/// events here, so the function is never called and gettrace() stays None
fn sys_settrace(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    one_hook(vm, "settrace", a, k)
}
fn sys_setprofile(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    one_hook(vm, "setprofile", a, k)
}
fn one_hook(vm: &mut Vm, name: &str, a: &[Value], k: Kw) -> PyResult<Value> {
    if !k.is_empty() {
        return Err(vm.type_error(format!("{}() takes no keyword arguments", name)));
    }
    if a.len() != 1 {
        return Err(vm.type_error(format!("{}() takes exactly one argument ({} given)", name, a.len())));
    }
    Ok(Value::None)
}
fn sys_false(_vm: &mut Vm, _a: &[Value], _k: Kw) -> PyResult<Value> {
    Ok(Value::Bool(false))
}

/// the running frame `depth` calls up, with a fresh f_locals snapshot
fn sys_getframe(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let depth = match a.first() {
        Some(v) => vm.to_index(v)?,
        None => 0,
    };
    let n = vm.frames.len();
    if depth < 0 || depth as usize >= n {
        return Err(vm.value_error("call stack is not deep enough"));
    }
    let idx = n - 1 - depth as usize;
    let v = vm.frame_value(idx);
    // SAFETY: see Vm::current_frame
    let f = unsafe { &*vm.frames[idx] };
    let cur = f.code.line_at(f.ip.saturating_sub(1)).unwrap_or(0);
    let snap = Value::Dict(frame_locals(f));
    if let Value::Obj(o) = &v {
        if let ObjKind::Frame { line, locals, .. } = &mut *o.kind.borrow_mut() {
            *line = cur;
            *locals = snap;
        }
    }
    Ok(v)
}

// -------------------------------------------------------------------------
// time

fn time_module() -> Value {
    let (d, m) = module("_time");
    put_fns(
        &d,
        &[
            ("time.time", t_time),
            ("time.time_ns", t_time_ns),
            ("time.perf_counter", t_perf),
            ("time.perf_counter_ns", t_perf_ns),
            ("time.monotonic", t_perf),
            ("time.monotonic_ns", t_perf_ns),
            ("time.process_time", t_perf),
            ("time.sleep", t_sleep),
            ("time.localtime", t_gmtime),
            ("time.gmtime", t_gmtime),
            ("time.strftime", t_strftime),
        ],
    );
    d.borrow_mut().set_s("timezone", Value::Int(0));
    d.borrow_mut().set_s("tzname", Value::tuple(vec![Value::str("UTC"), Value::str("UTC")]));
    m
}

fn now_secs() -> f64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs_f64()).unwrap_or(0.0)
}

fn t_time(_vm: &mut Vm, _a: &[Value], _k: Kw) -> PyResult<Value> {
    Ok(Value::Float(now_secs()))
}
fn t_time_ns(_vm: &mut Vm, _a: &[Value], _k: Kw) -> PyResult<Value> {
    Ok(Value::Int((now_secs() * 1e9) as i64))
}
fn t_perf(vm: &mut Vm, _a: &[Value], _k: Kw) -> PyResult<Value> {
    Ok(Value::Float(vm.start_time.elapsed().as_secs_f64()))
}
fn t_perf_ns(vm: &mut Vm, _a: &[Value], _k: Kw) -> PyResult<Value> {
    Ok(Value::Int(vm.start_time.elapsed().as_nanos() as i64))
}
fn t_sleep(_vm: &mut Vm, _a: &[Value], _k: Kw) -> PyResult<Value> {
    Ok(Value::None)
}

/// (year, month, day, hour, minute, second, weekday, yearday) of a UTC time
pub fn civil(secs: i64) -> [i64; 8] {
    let days = secs.div_euclid(86400);
    let rem = secs.rem_euclid(86400);
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    let wd = (days + 3).rem_euclid(7);
    let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    let cum = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    let yd = cum[(m - 1) as usize] + d + if leap && m > 2 { 1 } else { 0 };
    [y, m, d, rem / 3600, (rem % 3600) / 60, rem % 60, wd, yd]
}

fn t_gmtime(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let secs = match a.first() {
        Some(Value::None) | None => now_secs(),
        Some(v) => vm.to_f64(v)?,
    };
    let c = civil(secs.floor() as i64);
    let mut v: Vec<Value> = c.iter().map(|x| Value::Int(*x)).collect();
    v.push(Value::Int(0));
    Ok(Value::tuple(v))
}

fn t_strftime(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let fmt = as_pystr(vm, &a[0], "strftime")?;
    let t: Vec<i64> = match a.get(1) {
        Some(v) => {
            let items = vm.to_vec(v)?;
            let mut out = vec![];
            for x in items {
                out.push(vm.to_index(&x)?);
            }
            out
        }
        None => civil(now_secs() as i64).to_vec(),
    };
    let days = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];
    let months = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];
    let mut out = String::new();
    let mut cs = fmt.s.chars();
    while let Some(c) = cs.next() {
        if c != '%' {
            out.push(c);
            continue;
        }
        match cs.next() {
            Some('Y') => out.push_str(&format!("{:04}", t[0])),
            Some('y') => out.push_str(&format!("{:02}", t[0] % 100)),
            Some('m') => out.push_str(&format!("{:02}", t[1])),
            Some('d') => out.push_str(&format!("{:02}", t[2])),
            Some('H') => out.push_str(&format!("{:02}", t[3])),
            Some('I') => out.push_str(&format!("{:02}", if t[3] % 12 == 0 { 12 } else { t[3] % 12 })),
            Some('p') => out.push_str(if t[3] < 12 { "AM" } else { "PM" }),
            Some('M') => out.push_str(&format!("{:02}", t[4])),
            Some('S') => out.push_str(&format!("{:02}", t[5])),
            Some('j') => out.push_str(&format!("{:03}", t[7])),
            Some('A') => out.push_str(days[t[6].rem_euclid(7) as usize]),
            Some('a') => out.push_str(&days[t[6].rem_euclid(7) as usize][..3]),
            Some('B') => out.push_str(months[(t[1] - 1).rem_euclid(12) as usize]),
            Some('b') => out.push_str(&months[(t[1] - 1).rem_euclid(12) as usize][..3]),
            Some('Z') => out.push_str("UTC"),
            Some('z') => out.push_str("+0000"),
            Some('%') => out.push('%'),
            Some(o) => {
                out.push('%');
                out.push(o);
            }
            None => out.push('%'),
        }
    }
    Ok(Value::string(out))
}

// -------------------------------------------------------------------------
// os and os.path

fn os_module(vm: &mut Vm) -> PyResult<Value> {
    let (d, m) = module("os");
    {
        let mut b = d.borrow_mut();
        b.set_s("sep", Value::str("/"));
        b.set_s("altsep", Value::None);
        b.set_s("extsep", Value::str("."));
        b.set_s("pathsep", Value::str(":"));
        b.set_s("linesep", Value::str("\n"));
        b.set_s("curdir", Value::str("."));
        b.set_s("pardir", Value::str(".."));
        b.set_s("devnull", Value::str("/dev/null"));
        b.set_s("name", Value::str("posix"));
        let mut env = Dict::new();
        match &vm.host.environ {
            Some(vars) => {
                for (k, v) in vars {
                    env.set_s(k, Value::str(v));
                }
            }
            None => {
                env.set_s("HOME", Value::str("/"));
                env.set_s("PATH", Value::str("/bin"));
            }
        }
        b.set_s("environ", Value::dict(env));
    }
    put_fns(
        &d,
        &[
            ("os.getcwd", os_getcwd),
            ("os.chdir", os_chdir),
            ("os.listdir", os_listdir),
            ("os.mkdir", os_mkdir),
            ("os.makedirs", os_makedirs),
            ("os.remove", os_remove),
            ("os.unlink", os_remove),
            ("os.rmdir", os_rmdir),
            ("os.removedirs", os_rmdir),
            ("os.rename", os_rename),
            ("os.replace", os_rename),
            ("os.getenv", os_getenv),
            ("os.walk", os_walk),
            ("os.urandom", os_urandom),
            ("os.cpu_count", os_cpu_count),
            ("os.getpid", os_getpid),
            ("os.get_inheritable", os_get_inheritable),
            ("os.fspath", os_fspath),
            ("os.stat", os_stat),
            ("os.scandir", os_listdir),
        ],
    );
    let p = os_path_module();
    vm.modules.borrow_mut().set_s("os.path", p.clone());
    d.borrow_mut().set_s("path", p);
    Ok(m)
}

fn path_arg(vm: &mut Vm, a: &[Value], i: usize) -> PyResult<String> {
    match a.get(i) {
        Some(Value::Str(s)) => Ok(s.s.clone()),
        Some(v) => {
            if let Some(f) = vm.getattr_opt(v, &ps("__fspath__"))? {
                let r = vm.call_vec(&f, vec![], &[])?;
                return vm.str_of(&r);
            }
            match vm.base_of(v) {
                Value::Str(s) => Ok(s.s.clone()),
                _ => {
                    let tn = vm.type_name(v);
                    Err(vm.type_error(format!("expected str, bytes or os.PathLike object, not {}", tn)))
                }
            }
        }
        None => Ok(".".into()),
    }
}

fn os_error(vm: &mut Vm, kind: &str, errno: i64, msg: &str, path: &str) -> Exc {
    let cls = vm.exc_class(kind);
    let e = vm.new_exc(&cls, vec![Value::Int(errno), Value::str(msg), Value::str(path)]);
    Exc { value: e }
}

fn os_getcwd(vm: &mut Vm, _a: &[Value], _k: Kw) -> PyResult<Value> {
    Ok(Value::str(&vm.fs.cwd))
}
fn os_chdir(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let p = path_arg(vm, a, 0)?;
    if !vm.fs.is_dir(&p) {
        return Err(os_error(vm, "FileNotFoundError", 2, "No such file or directory", &p));
    }
    vm.fs.cwd = vm.fs.normalize(&p);
    Ok(Value::None)
}
fn os_listdir(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let p = path_arg(vm, a, 0)?;
    if !vm.fs.is_dir(&p) {
        if vm.fs.is_file(&p) {
            return Err(os_error(vm, "NotADirectoryError", 20, "Not a directory", &p));
        }
        return Err(os_error(vm, "FileNotFoundError", 2, "No such file or directory", &p));
    }
    Ok(Value::list(vm.fs.list_dir(&p).into_iter().map(Value::string).collect()))
}
fn os_mkdir(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let p = path_arg(vm, a, 0)?;
    if vm.fs.exists(&p) {
        return Err(os_error(vm, "FileExistsError", 17, "File exists", &p));
    }
    let abs = vm.fs.normalize(&p);
    if !vm.fs.is_dir(&parent(&abs)) {
        return Err(os_error(vm, "FileNotFoundError", 2, "No such file or directory", &p));
    }
    vm.fs.mkdirs(&p);
    Ok(Value::None)
}
fn os_makedirs(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let p = path_arg(vm, a, 0)?;
    let ok = match arg_or_kw(a, 2, k, "exist_ok") {
        Some(v) => vm.truthy(&v)?,
        None => false,
    };
    if vm.fs.is_dir(&p) && !ok {
        return Err(os_error(vm, "FileExistsError", 17, "File exists", &p));
    }
    vm.fs.mkdirs(&p);
    Ok(Value::None)
}
fn os_remove(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let p = path_arg(vm, a, 0)?;
    if vm.fs.is_dir(&p) && !vm.fs.is_file(&p) {
        return Err(os_error(vm, "IsADirectoryError", 21, "Is a directory", &p));
    }
    if !vm.fs.remove(&p) {
        return Err(os_error(vm, "FileNotFoundError", 2, "No such file or directory", &p));
    }
    Ok(Value::None)
}
fn os_rmdir(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let p = path_arg(vm, a, 0)?;
    if !vm.fs.is_dir(&p) {
        return Err(os_error(vm, "FileNotFoundError", 2, "No such file or directory", &p));
    }
    if !vm.fs.remove_dir(&p) {
        return Err(os_error(vm, "OSError", 39, "Directory not empty", &p));
    }
    Ok(Value::None)
}
fn os_rename(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let src = path_arg(vm, a, 0)?;
    let dst = path_arg(vm, a, 1)?;
    match vm.fs.read(&src) {
        Some(t) => {
            vm.fs.remove(&src);
            vm.fs.write(&dst, t);
            Ok(Value::None)
        }
        None => Err(os_error(vm, "FileNotFoundError", 2, "No such file or directory", &src)),
    }
}
fn os_getenv(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    // os.environ.get(key, default)
    let key = path_arg(vm, a, 0)?;
    let os = vm.import("os", &Value::None, 0, &vm.main_globals.clone())?;
    let env = vm.getattr(&os, &ps("environ"))?;
    let get = vm.getattr(&env, &ps("get"))?;
    vm.call_vec(&get, vec![Value::string(key), a.get(1).cloned().unwrap_or(Value::None)], &[])
}
fn os_walk(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let top = path_arg(vm, a, 0)?;
    let mut out = vec![];
    let mut stack = vec![top];
    while let Some(dir) = stack.pop() {
        if !vm.fs.is_dir(&dir) {
            continue;
        }
        let mut dirs = vec![];
        let mut files = vec![];
        for n in vm.fs.list_dir(&dir) {
            let full = if dir.ends_with('/') { format!("{}{}", dir, n) } else { format!("{}/{}", dir, n) };
            if vm.fs.is_file(&full) {
                files.push(Value::string(n));
            } else {
                dirs.push(n);
            }
        }
        for d in dirs.iter().rev() {
            stack.push(if dir.ends_with('/') { format!("{}{}", dir, d) } else { format!("{}/{}", dir, d) });
        }
        out.push(Value::tuple(vec![Value::string(dir), Value::list(dirs.into_iter().map(Value::string).collect()), Value::list(files)]));
    }
    Ok(mk_iter(Iter::Vec(out.into_iter())))
}
fn os_urandom(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let n = vm.to_index(&a[0])?;
    if n < 0 {
        return Err(vm.value_error("negative argument not allowed"));
    }
    let n = n as usize;
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        out.push((vm.random.genrand() >> 24) as u8);
    }
    Ok(Value::Bytes(Rc::new(out)))
}
fn os_cpu_count(_vm: &mut Vm, _a: &[Value], _k: Kw) -> PyResult<Value> {
    Ok(Value::Int(1))
}
fn os_getpid(_vm: &mut Vm, _a: &[Value], _k: Kw) -> PyResult<Value> {
    Ok(Value::Int(1))
}
/// Files are not inherited (PEP 446): a child gets the sandbox's files, not
/// the parent's open ones; only the standard streams are its own too.
fn os_get_inheritable(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let fd = vm.to_index(&a[0])?;
    if fd < 0 {
        let cls = vm.exc_class("OSError");
        let e = vm.new_exc(&cls, vec![Value::Int(9), Value::str("Bad file descriptor")]);
        return Err(Exc { value: e });
    }
    Ok(Value::Bool(fd <= 2))
}
fn os_fspath(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    Ok(Value::string(path_arg(vm, a, 0)?))
}
fn os_stat(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let p = path_arg(vm, a, 0)?;
    let size = match vm.fs.read(&p) {
        Some(t) => t.len(),
        None => {
            if vm.fs.is_dir(&p) {
                0
            } else {
                return Err(os_error(vm, "FileNotFoundError", 2, "No such file or directory", &p));
            }
        }
    };
    let d = new_dict();
    d.borrow_mut().set_s("st_size", Value::Int(size as i64));
    d.borrow_mut().set_s("st_mtime", Value::Float(0.0));
    d.borrow_mut().set_s("st_mode", Value::Int(if vm.fs.is_file(&p) { 0o100644 } else { 0o40755 }));
    Ok(Value::Module(Rc::new(Module { name: ps("os.stat_result"), dict: d })))
}

fn os_path_module() -> Value {
    let (d, m) = module("posixpath");
    d.borrow_mut().set_s("sep", Value::str("/"));
    put_fns(
        &d,
        &[
            ("posixpath.join", p_join),
            ("posixpath.exists", p_exists),
            ("posixpath.lexists", p_exists),
            ("posixpath.isfile", p_isfile),
            ("posixpath.isdir", p_isdir),
            ("posixpath.basename", p_basename),
            ("posixpath.dirname", p_dirname),
            ("posixpath.split", p_split),
            ("posixpath.splitext", p_splitext),
            ("posixpath.abspath", p_abspath),
            ("posixpath.normpath", p_normpath),
            ("posixpath.realpath", p_abspath),
            ("posixpath.relpath", p_relpath),
            ("posixpath.getsize", p_getsize),
            ("posixpath.expanduser", p_expanduser),
            ("posixpath.isabs", p_isabs),
            ("posixpath.commonprefix", p_commonprefix),
            ("posixpath.islink", p_islink),
        ],
    );
    m
}

fn p_join(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let mut out = path_arg(vm, a, 0)?;
    for i in 1..a.len() {
        let p = path_arg(vm, a, i)?;
        if p.starts_with('/') {
            out = p;
        } else if out.is_empty() || out.ends_with('/') {
            out.push_str(&p);
        } else {
            out.push('/');
            out.push_str(&p);
        }
    }
    Ok(Value::string(out))
}
fn p_exists(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let p = path_arg(vm, a, 0)?;
    Ok(Value::Bool(vm.fs.exists(&p)))
}
fn p_isfile(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let p = path_arg(vm, a, 0)?;
    Ok(Value::Bool(vm.fs.is_file(&p)))
}
fn p_isdir(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let p = path_arg(vm, a, 0)?;
    Ok(Value::Bool(vm.fs.is_dir(&p) && !vm.fs.is_file(&p)))
}
fn p_islink(_vm: &mut Vm, _a: &[Value], _k: Kw) -> PyResult<Value> {
    Ok(Value::Bool(false))
}
fn split_path(p: &str) -> (String, String) {
    match p.rfind('/') {
        Some(i) => {
            let head = &p[..i + 1];
            let head = if head.chars().all(|c| c == '/') { head.to_string() } else { head.trim_end_matches('/').to_string() };
            (head, p[i + 1..].to_string())
        }
        None => (String::new(), p.to_string()),
    }
}
fn p_basename(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let p = path_arg(vm, a, 0)?;
    Ok(Value::string(split_path(&p).1))
}
fn p_dirname(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let p = path_arg(vm, a, 0)?;
    Ok(Value::string(split_path(&p).0))
}
fn p_split(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let p = path_arg(vm, a, 0)?;
    let (h, t) = split_path(&p);
    Ok(Value::tuple(vec![Value::string(h), Value::string(t)]))
}
fn p_splitext(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let p = path_arg(vm, a, 0)?;
    let base_start = p.rfind('/').map(|i| i + 1).unwrap_or(0);
    let base = &p[base_start..];
    let lead = base.len() - base.trim_start_matches('.').len();
    match base[lead..].rfind('.') {
        Some(i) => {
            let at = base_start + lead + i;
            Ok(Value::tuple(vec![Value::str(&p[..at]), Value::str(&p[at..])]))
        }
        None => Ok(Value::tuple(vec![Value::string(p.clone()), Value::str("")])),
    }
}
fn norm(p: &str) -> String {
    if p.is_empty() {
        return ".".into();
    }
    let abs = p.starts_with('/');
    let mut out: Vec<&str> = vec![];
    for part in p.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                if !out.is_empty() && *out.last().unwrap() != ".." {
                    out.pop();
                } else if !abs {
                    out.push("..");
                }
            }
            x => out.push(x),
        }
    }
    let s = out.join("/");
    if abs {
        format!("/{}", s)
    } else if s.is_empty() {
        ".".into()
    } else {
        s
    }
}
fn p_normpath(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let p = path_arg(vm, a, 0)?;
    Ok(Value::string(norm(&p)))
}
fn p_abspath(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let p = path_arg(vm, a, 0)?;
    Ok(Value::string(vm.fs.normalize(&p)))
}
fn p_relpath(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let p0 = path_arg(vm, a, 0)?;
    let p = vm.fs.normalize(&p0);
    let start = match a.get(1) {
        Some(_) => {
            let s0 = path_arg(vm, a, 1)?;
            vm.fs.normalize(&s0)
        }
        None => vm.fs.cwd.clone(),
    };
    let ps_: Vec<&str> = p.split('/').filter(|x| !x.is_empty()).collect();
    let ss: Vec<&str> = start.split('/').filter(|x| !x.is_empty()).collect();
    let common = ps_.iter().zip(ss.iter()).take_while(|(x, y)| x == y).count();
    let mut out: Vec<&str> = vec![".."; ss.len() - common];
    out.extend(&ps_[common..]);
    Ok(Value::string(if out.is_empty() { ".".into() } else { out.join("/") }))
}
fn p_getsize(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let p = path_arg(vm, a, 0)?;
    match vm.fs.read(&p) {
        Some(t) => Ok(Value::Int(t.len() as i64)),
        None => Err(os_error(vm, "FileNotFoundError", 2, "No such file or directory", &p)),
    }
}
fn p_expanduser(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let p = path_arg(vm, a, 0)?;
    Ok(Value::string(if let Some(r) = p.strip_prefix('~') { format!("/{}", r.trim_start_matches('/')) } else { p }))
}
fn p_isabs(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let p = path_arg(vm, a, 0)?;
    Ok(Value::Bool(p.starts_with('/')))
}
fn p_commonprefix(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let items = vm.to_vec(&a[0])?;
    let strs: Vec<String> = items.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
    if strs.is_empty() {
        return Ok(Value::str(""));
    }
    let mut pre: Vec<char> = strs[0].chars().collect();
    for s in &strs[1..] {
        let n = pre.iter().zip(s.chars()).take_while(|(a, b)| **a == *b).count();
        pre.truncate(n);
    }
    Ok(Value::string(pre.into_iter().collect()))
}

// -------------------------------------------------------------------------
// json

fn json_module(vm: &mut Vm) -> Value {
    let (d, m) = module("json");
    put_fns(&d, &[("json.dumps", j_dumps), ("json.loads", j_loads), ("json.dump", j_dump), ("json.load", j_load)]);
    let ve = vm.exc_class("ValueError");
    let c = Class::new("JSONDecodeError", vec![ve.clone()], Builtin::BaseException);
    let mut mro = vec![c.clone()];
    mro.extend(ve.mro.borrow().iter().cloned());
    c.set_mro(mro);
    c.dict.borrow_mut().set_s("__module__", Value::str("json.decoder"));
    vm.exc.insert("JSONDecodeError", c.clone());
    d.borrow_mut().set_s("JSONDecodeError", Value::Class(c));
    m
}

struct JsonOpts {
    indent: Option<String>,
    item_sep: String,
    key_sep: String,
    sort_keys: bool,
    ensure_ascii: bool,
    allow_nan: bool,
    skipkeys: bool,
    default: Value,
}

fn json_str(s: &str, ascii: bool, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\x08' => out.push_str("\\b"),
            '\x0c' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c if ascii && (c as u32) > 0x7e => {
                let n = c as u32;
                if n > 0xffff {
                    let v = n - 0x10000;
                    out.push_str(&format!("\\u{:04x}\\u{:04x}", 0xd800 + (v >> 10), 0xdc00 + (v & 0x3ff)));
                } else {
                    out.push_str(&format!("\\u{:04x}", n));
                }
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

fn json_float(vm: &mut Vm, f: f64, o: &JsonOpts) -> PyResult<String> {
    if f.is_finite() {
        return Ok(crate::format::float_repr(f));
    }
    if !o.allow_nan {
        let r = crate::format::float_repr(f);
        return Err(vm.value_error(format!("Out of range float values are not JSON compliant: {}", r)));
    }
    Ok(if f.is_nan() {
        "NaN".into()
    } else if f > 0.0 {
        "Infinity".into()
    } else {
        "-Infinity".into()
    })
}

fn json_enc(vm: &mut Vm, v: &Value, o: &JsonOpts, level: usize, out: &mut String, stack: &mut Vec<usize>) -> PyResult<()> {
    match v {
        Value::None => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Int(i) => out.push_str(&i.to_string()),
        Value::Big(b) => out.push_str(&b.to_string()),
        Value::Float(f) => out.push_str(&json_float(vm, *f, o)?),
        Value::Str(s) => json_str(&s.s, o.ensure_ascii, out),
        Value::List(_) | Value::Tuple(_) => {
            let items = vm.to_vec(v)?;
            let id = ptr_id(v);
            if stack.contains(&id) {
                return Err(vm.value_error("Circular reference detected"));
            }
            if items.is_empty() {
                out.push_str("[]");
                return Ok(());
            }
            stack.push(id);
            out.push('[');
            for (i, x) in items.iter().enumerate() {
                if i > 0 {
                    out.push_str(&o.item_sep);
                }
                if let Some(ind) = &o.indent {
                    out.push('\n');
                    out.push_str(&ind.repeat(level + 1));
                }
                json_enc(vm, x, o, level + 1, out, stack)?;
            }
            if let Some(ind) = &o.indent {
                out.push('\n');
                out.push_str(&ind.repeat(level));
            }
            out.push(']');
            stack.pop();
        }
        Value::Dict(d) => {
            let id = ptr_id(v);
            if stack.contains(&id) {
                return Err(vm.value_error("Circular reference detected"));
            }
            let mut items = d.borrow().items();
            if items.is_empty() {
                out.push_str("{}");
                return Ok(());
            }
            if o.sort_keys {
                let mut keys: Vec<Value> = items.iter().map(|(k, _)| k.clone()).collect();
                sort_values(vm, &mut keys, &Value::None, false)?;
                let mut sorted = vec![];
                for k in keys {
                    let i = items.iter().position(|(kk, _)| kk.is(&k)).unwrap();
                    sorted.push(items.remove(i));
                }
                items = sorted;
            }
            stack.push(id);
            out.push('{');
            let mut first = true;
            for (k, x) in items.iter() {
                let ks = match k {
                    Value::Str(s) => s.s.clone(),
                    Value::Int(i) => i.to_string(),
                    Value::Big(b) => b.to_string(),
                    Value::Float(f) => json_float(vm, *f, o)?,
                    Value::Bool(b) => (if *b { "true" } else { "false" }).to_string(),
                    Value::None => "null".into(),
                    _ => {
                        if o.skipkeys {
                            continue;
                        }
                        let tn = vm.type_name(k);
                        return Err(vm.type_error(format!("keys must be str, int, float, bool or None, not {}", tn)));
                    }
                };
                if !first {
                    out.push_str(&o.item_sep);
                }
                first = false;
                if let Some(ind) = &o.indent {
                    out.push('\n');
                    out.push_str(&ind.repeat(level + 1));
                }
                json_str(&ks, o.ensure_ascii, out);
                out.push_str(&o.key_sep);
                json_enc(vm, x, o, level + 1, out, stack)?;
            }
            if let Some(ind) = &o.indent {
                out.push('\n');
                out.push_str(&ind.repeat(level));
            }
            out.push('}');
            stack.pop();
        }
        _ => {
            let b = vm.base_of(v);
            if matches!(v, Value::Instance(_)) && !matches!(b, Value::Instance(_)) {
                return json_enc(vm, &b, o, level, out, stack);
            }
            if o.default.is_none() {
                let tn = vm.type_name(v);
                return Err(vm.type_error(format!("Object of type {} is not JSON serializable", tn)));
            }
            let r = vm.call_vec(&o.default.clone(), vec![v.clone()], &[])?;
            json_enc(vm, &r, o, level, out, stack)?;
        }
    }
    Ok(())
}

fn json_opts(vm: &mut Vm, k: Kw) -> PyResult<JsonOpts> {
    let indent = match kwarg(k, "indent") {
        None | Some(Value::None) => None,
        Some(Value::Str(s)) => Some(s.s.clone()),
        Some(v) => Some(" ".repeat(vm.to_index(&v)?.max(0) as usize)),
    };
    let (mut item_sep, mut key_sep) = (if indent.is_some() { "," } else { ", " }.to_string(), ": ".to_string());
    if let Some(sep) = kwarg(k, "separators") {
        if !sep.is_none() {
            let parts = vm.to_vec(&sep)?;
            if parts.len() == 2 {
                item_sep = vm.str_of(&parts[0])?;
                key_sep = vm.str_of(&parts[1])?;
            }
        }
    }
    let flag = |vm: &mut Vm, n: &str, d: bool| -> PyResult<bool> {
        match kwarg(k, n) {
            Some(v) => vm.truthy(&v),
            None => Ok(d),
        }
    };
    Ok(JsonOpts {
        indent,
        item_sep,
        key_sep,
        sort_keys: flag(vm, "sort_keys", false)?,
        ensure_ascii: flag(vm, "ensure_ascii", true)?,
        allow_nan: flag(vm, "allow_nan", true)?,
        skipkeys: flag(vm, "skipkeys", false)?,
        default: kwarg(k, "default").unwrap_or(Value::None),
    })
}

fn j_dumps(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    check_nargs(vm, "dumps", a, 1, 1)?;
    let o = json_opts(vm, k)?;
    let mut out = String::new();
    json_enc(vm, &a[0], &o, 0, &mut out, &mut vec![])?;
    Ok(Value::string(out))
}

fn j_dump(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    check_nargs(vm, "dump", a, 2, 2)?;
    let s = j_dumps(vm, &a[..1], k)?;
    let w = vm.getattr(&a[1], &ps("write"))?;
    vm.call_vec(&w, vec![s], &[])?;
    Ok(Value::None)
}

struct JParser<'a> {
    s: &'a [u8],
    src: &'a str,
    i: usize,
    hooks: (Value, Value, Value, Value),
}

fn json_err(vm: &mut Vm, p: &JParser, msg: &str) -> Exc {
    let pos = p.src[..p.i.min(p.src.len())].chars().count();
    let line = p.src[..p.i.min(p.src.len())].matches('\n').count() + 1;
    let col = pos - p.src[..p.i.min(p.src.len())].rfind('\n').map(|j| p.src[..j + 1].chars().count()).unwrap_or(0) + 1;
    let cls = vm.exc_class("JSONDecodeError");
    let text = format!("{}: line {} column {} (char {})", msg, line, col, pos);
    let e = vm.new_exc(&cls, vec![Value::string(text)]);
    if let Value::Instance(i) = &e {
        let mut d = i.dict.borrow_mut();
        d.set_s("msg", Value::str(msg));
        d.set_s("pos", Value::Int(pos as i64));
        d.set_s("lineno", Value::Int(line as i64));
        d.set_s("colno", Value::Int(col as i64));
        d.set_s("doc", Value::str(p.src));
    }
    Exc { value: e }
}

impl JParser<'_> {
    fn ws(&mut self) {
        while self.i < self.s.len() && matches!(self.s[self.i], b' ' | b'\t' | b'\n' | b'\r') {
            self.i += 1;
        }
    }

    fn value(&mut self, vm: &mut Vm) -> PyResult<Value> {
        self.ws();
        if self.i >= self.s.len() {
            return Err(json_err(vm, self, "Expecting value"));
        }
        match self.s[self.i] {
            b'{' => {
                self.i += 1;
                let mut pairs: Vec<(Value, Value)> = vec![];
                self.ws();
                if self.i < self.s.len() && self.s[self.i] == b'}' {
                    self.i += 1;
                } else {
                    loop {
                        self.ws();
                        if self.i >= self.s.len() || self.s[self.i] != b'"' {
                            return Err(json_err(vm, self, "Expecting property name enclosed in double quotes"));
                        }
                        let k = self.string(vm)?;
                        self.ws();
                        if self.i >= self.s.len() || self.s[self.i] != b':' {
                            return Err(json_err(vm, self, "Expecting ':' delimiter"));
                        }
                        self.i += 1;
                        let v = self.value(vm)?;
                        pairs.push((Value::string(k), v));
                        self.ws();
                        match self.s.get(self.i) {
                            Some(b',') => {
                                self.i += 1;
                                self.ws();
                                if self.s.get(self.i) == Some(&b'}') {
                                    return Err(json_err(vm, self, "Illegal trailing comma before end of object"));
                                }
                            }
                            Some(b'}') => {
                                self.i += 1;
                                break;
                            }
                            _ => return Err(json_err(vm, self, "Expecting ',' delimiter")),
                        }
                    }
                }
                if !self.hooks.1.is_none() {
                    let items = Value::list(pairs.into_iter().map(|(k, v)| Value::tuple(vec![k, v])).collect());
                    return vm.call_vec(&self.hooks.1.clone(), vec![items], &[]);
                }
                let d = new_dict();
                for (k, v) in pairs {
                    vm.dict_set(&d, k, v)?;
                }
                if !self.hooks.0.is_none() {
                    return vm.call_vec(&self.hooks.0.clone(), vec![Value::Dict(d)], &[]);
                }
                Ok(Value::Dict(d))
            }
            b'[' => {
                self.i += 1;
                let mut items = vec![];
                self.ws();
                if self.s.get(self.i) == Some(&b']') {
                    self.i += 1;
                    return Ok(Value::list(items));
                }
                loop {
                    items.push(self.value(vm)?);
                    self.ws();
                    match self.s.get(self.i) {
                        Some(b',') => {
                            self.i += 1;
                            self.ws();
                            if self.s.get(self.i) == Some(&b']') {
                                return Err(json_err(vm, self, "Illegal trailing comma before end of array"));
                            }
                        }
                        Some(b']') => {
                            self.i += 1;
                            break;
                        }
                        _ => return Err(json_err(vm, self, "Expecting ',' delimiter")),
                    }
                }
                Ok(Value::list(items))
            }
            b'"' => Ok(Value::string(self.string(vm)?)),
            b't' if self.s[self.i..].starts_with(b"true") => {
                self.i += 4;
                Ok(Value::Bool(true))
            }
            b'f' if self.s[self.i..].starts_with(b"false") => {
                self.i += 5;
                Ok(Value::Bool(false))
            }
            b'n' if self.s[self.i..].starts_with(b"null") => {
                self.i += 4;
                Ok(Value::None)
            }
            b'N' if self.s[self.i..].starts_with(b"NaN") => {
                self.i += 3;
                Ok(Value::Float(f64::NAN))
            }
            b'I' if self.s[self.i..].starts_with(b"Infinity") => {
                self.i += 8;
                Ok(Value::Float(f64::INFINITY))
            }
            b'-' if self.s[self.i..].starts_with(b"-Infinity") => {
                self.i += 9;
                Ok(Value::Float(f64::NEG_INFINITY))
            }
            b'-' | b'0'..=b'9' => self.number(vm),
            _ => Err(json_err(vm, self, "Expecting value")),
        }
    }

    fn number(&mut self, vm: &mut Vm) -> PyResult<Value> {
        let st = self.i;
        if self.s[self.i] == b'-' {
            self.i += 1;
        }
        let ds = self.i;
        while self.i < self.s.len() && self.s[self.i].is_ascii_digit() {
            self.i += 1;
        }
        if self.i == ds {
            self.i = st;
            return Err(json_err(vm, self, "Expecting value"));
        }
        let mut is_float = false;
        if self.i < self.s.len() && self.s[self.i] == b'.' && self.s.get(self.i + 1).map(|c| c.is_ascii_digit()).unwrap_or(false) {
            is_float = true;
            self.i += 1;
            while self.i < self.s.len() && self.s[self.i].is_ascii_digit() {
                self.i += 1;
            }
        }
        if self.i < self.s.len() && (self.s[self.i] == b'e' || self.s[self.i] == b'E') {
            let save = self.i;
            self.i += 1;
            if self.i < self.s.len() && (self.s[self.i] == b'+' || self.s[self.i] == b'-') {
                self.i += 1;
            }
            let es = self.i;
            while self.i < self.s.len() && self.s[self.i].is_ascii_digit() {
                self.i += 1;
            }
            if self.i == es {
                self.i = save;
            } else {
                is_float = true;
            }
        }
        let text = &self.src[st..self.i];
        if is_float {
            if !self.hooks.2.is_none() {
                return vm.call_vec(&self.hooks.2.clone(), vec![Value::str(text)], &[]);
            }
            return Ok(Value::Float(text.parse().unwrap_or(0.0)));
        }
        if !self.hooks.3.is_none() {
            return vm.call_vec(&self.hooks.3.clone(), vec![Value::str(text)], &[]);
        }
        Ok(match text.parse::<i64>() {
            Ok(i) => Value::Int(i),
            Err(_) => Value::from_big(crate::methods::parse_int(text, 10).unwrap()),
        })
    }

    fn string(&mut self, vm: &mut Vm) -> PyResult<String> {
        let start = self.i;
        self.i += 1;
        let mut out = String::new();
        loop {
            let run = self.i;
            while self.i < self.s.len() && self.s[self.i] != b'"' && self.s[self.i] != b'\\' && self.s[self.i] >= 0x20 {
                self.i += 1;
            }
            out.push_str(&self.src[run..self.i]);
            if self.i >= self.s.len() {
                self.i = start;
                return Err(json_err(vm, self, "Unterminated string starting at"));
            }
            match self.s[self.i] {
                b'"' => {
                    self.i += 1;
                    return Ok(out);
                }
                b'\\' => {
                    self.i += 1;
                    let c = match self.s.get(self.i) {
                        Some(c) => *c,
                        None => {
                            self.i = start;
                            return Err(json_err(vm, self, "Unterminated string starting at"));
                        }
                    };
                    self.i += 1;
                    match c {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\x08'),
                        b'f' => out.push('\x0c'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => {
                            let cp = self.hex4(vm)?;
                            if (0xd800..0xdc00).contains(&cp) && self.s[self.i..].starts_with(b"\\u") {
                                let save = self.i;
                                self.i += 2;
                                let lo = self.hex4(vm)?;
                                if (0xdc00..0xe000).contains(&lo) {
                                    let v = 0x10000 + ((cp - 0xd800) << 10) + (lo - 0xdc00);
                                    out.push(char::from_u32(v).unwrap_or('\u{fffd}'));
                                    continue;
                                }
                                self.i = save;
                            }
                            out.push(char::from_u32(cp).unwrap_or('\u{fffd}'));
                        }
                        _ => {
                            self.i -= 2;
                            return Err(json_err(vm, self, "Invalid \\escape"));
                        }
                    }
                }
                _ => return Err(json_err(vm, self, "Invalid control character at")),
            }
        }
    }

    fn hex4(&mut self, vm: &mut Vm) -> PyResult<u32> {
        if self.i + 4 > self.s.len() {
            return Err(json_err(vm, self, "Invalid \\uXXXX escape"));
        }
        let h = &self.src[self.i..self.i + 4];
        match u32::from_str_radix(h, 16) {
            Ok(v) => {
                self.i += 4;
                Ok(v)
            }
            Err(_) => Err(json_err(vm, self, "Invalid \\uXXXX escape")),
        }
    }
}

fn j_loads(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    check_nargs(vm, "loads", a, 1, 1)?;
    let text = match &a[0] {
        Value::Str(s) => s.s.clone(),
        Value::Bytes(_) | Value::ByteArray(_) => String::from_utf8_lossy(&bytes_of(vm, &a[0])?).into_owned(),
        v => {
            let tn = vm.type_name(v);
            return Err(vm.type_error(format!("the JSON object must be str, bytes or bytearray, not {}", tn)));
        }
    };
    let hooks = (kwarg(k, "object_hook").unwrap_or(Value::None), kwarg(k, "object_pairs_hook").unwrap_or(Value::None), kwarg(k, "parse_float").unwrap_or(Value::None), kwarg(k, "parse_int").unwrap_or(Value::None));
    let mut p = JParser { s: text.as_bytes(), src: &text, i: 0, hooks };
    let v = p.value(vm)?;
    p.ws();
    if p.i < p.s.len() {
        return Err(json_err(vm, &p, "Extra data"));
    }
    Ok(v)
}

fn j_load(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let r = vm.getattr(&a[0], &ps("read"))?;
    let s = vm.call_vec(&r, vec![], &[])?;
    j_loads(vm, &[s], k)
}

// -------------------------------------------------------------------------
// _random: CPython's Mersenne Twister

#[derive(Clone)]
pub struct Random {
    mt: Vec<u32>,
    mti: usize,
}

impl Default for Random {
    fn default() -> Self {
        Self::new()
    }
}

impl Random {
    pub fn new() -> Random {
        let mut r = Random { mt: vec![0; 624], mti: 625 };
        let t = (now_secs() * 1e6) as u64;
        r.init_by_array(&[t as u32, (t >> 32) as u32]);
        r
    }

    fn init_genrand(&mut self, s: u32) {
        self.mt[0] = s;
        for i in 1..624 {
            self.mt[i] = 1812433253u32.wrapping_mul(self.mt[i - 1] ^ (self.mt[i - 1] >> 30)).wrapping_add(i as u32);
        }
        self.mti = 624;
    }

    pub fn init_by_array(&mut self, key: &[u32]) {
        self.init_genrand(19650218);
        let n = 624;
        let (mut i, mut j) = (1usize, 0usize);
        let klen = key.len().max(1);
        let mut k = n.max(klen);
        while k > 0 {
            let prev = self.mt[i - 1] ^ (self.mt[i - 1] >> 30);
            self.mt[i] = (self.mt[i] ^ prev.wrapping_mul(1664525)).wrapping_add(*key.get(j).unwrap_or(&0)).wrapping_add(j as u32);
            i += 1;
            j += 1;
            if i >= n {
                self.mt[0] = self.mt[n - 1];
                i = 1;
            }
            if j >= klen {
                j = 0;
            }
            k -= 1;
        }
        k = n - 1;
        while k > 0 {
            let prev = self.mt[i - 1] ^ (self.mt[i - 1] >> 30);
            self.mt[i] = (self.mt[i] ^ prev.wrapping_mul(1566083941)).wrapping_sub(i as u32);
            i += 1;
            if i >= n {
                self.mt[0] = self.mt[n - 1];
                i = 1;
            }
            k -= 1;
        }
        self.mt[0] = 0x80000000;
        self.mti = 624;
    }

    pub fn genrand(&mut self) -> u32 {
        if self.mti >= 624 {
            for kk in 0..624 {
                let y = (self.mt[kk] & 0x80000000) | (self.mt[(kk + 1) % 624] & 0x7fffffff);
                let mut v = self.mt[(kk + 397) % 624] ^ (y >> 1);
                if y & 1 != 0 {
                    v ^= 0x9908b0df;
                }
                self.mt[kk] = v;
            }
            self.mti = 0;
        }
        let mut y = self.mt[self.mti];
        self.mti += 1;
        y ^= y >> 11;
        y ^= (y << 7) & 0x9d2c5680;
        y ^= (y << 15) & 0xefc60000;
        y ^= y >> 18;
        y
    }

    pub fn random(&mut self) -> f64 {
        let a = (self.genrand() >> 5) as f64;
        let b = (self.genrand() >> 6) as f64;
        (a * 67108864.0 + b) * (1.0 / 9007199254740992.0)
    }

    pub fn getrandbits(&mut self, k: usize) -> BigInt {
        if k == 0 {
            return BigInt::zero();
        }
        if k <= 32 {
            return BigInt::from_i64((self.genrand() >> (32 - k)) as i64);
        }
        let words = k.div_ceil(32);
        let mut mag = Vec::with_capacity(words);
        let mut left = k;
        for _ in 0..words {
            let mut r = self.genrand();
            if left < 32 {
                r >>= 32 - left;
            }
            mag.push(r);
            left = left.saturating_sub(32);
        }
        let mut b = BigInt::zero();
        for w in mag.iter().rev() {
            b = b.shl(32).add(&BigInt::from_i64(*w as i64));
        }
        b
    }

    pub fn seed_big(&mut self, v: &BigInt) {
        let a = v.abs();
        let mut key: Vec<u32> = a.mag.clone();
        while key.len() > 1 && *key.last().unwrap() == 0 {
            key.pop();
        }
        if key.is_empty() {
            key.push(0);
        }
        self.init_by_array(&key);
    }
}

fn random_module() -> Value {
    let (d, m) = module("_rpyrandom");
    put_fns(&d, &[("_rpyrandom.new", r_new), ("_rpyrandom.seed", r_seed), ("_rpyrandom.random", r_random), ("_rpyrandom.getrandbits", r_getrandbits), ("_rpyrandom.getstate", r_getstate), ("_rpyrandom.setstate", r_setstate)]);
    m
}

fn rng_of(vm: &mut Vm, v: &Value) -> PyResult<Rc<NativeObj>> {
    match v {
        Value::Obj(o) if matches!(&*o.kind.borrow(), ObjKind::Rng(_)) => Ok(o.clone()),
        _ => Err(vm.type_error("expected a random generator")),
    }
}

fn r_new(vm: &mut Vm, _a: &[Value], _k: Kw) -> PyResult<Value> {
    let r = Random::new();
    Ok(Value::Obj(Rc::new(NativeObj { class: vm.t.obj.clone(), gc: Default::default(), kind: RefCell::new(ObjKind::Rng(Box::new(r))) })))
}

fn r_seed(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let o = rng_of(vm, &a[0])?;
    let s = a.get(1).cloned().unwrap_or(Value::None);
    let key: BigInt = match &s {
        Value::None => {
            let t = (now_secs() * 1e6) as i64;
            BigInt::from_i64(t)
        }
        Value::Int(_) | Value::Big(_) | Value::Bool(_) => crate::ops::big_of(&s).unwrap(),
        Value::Float(f) => {
            // hash(float), as CPython
            BigInt::from_i64(crate::value::hash_float(*f) as i64)
        }
        Value::Str(_) | Value::Bytes(_) | Value::ByteArray(_) => {
            let mut bytes = match &s {
                Value::Str(t) => t.s.as_bytes().to_vec(),
                other => bytes_of(vm, other)?,
            };
            let digest = sha512(&bytes);
            bytes.extend_from_slice(&digest);
            let mut b = BigInt::zero();
            for byte in bytes {
                b = b.shl(8).add(&BigInt::from_i64(byte as i64));
            }
            b
        }
        other => {
            let tn = vm.type_name(other);
            return Err(vm.type_error(format!("The only supported seed types are:\nNone, int, float, str, bytes, and bytearray.").replace("{tn}", &tn)));
        }
    };
    if let ObjKind::Rng(r) = &mut *o.kind.borrow_mut() {
        r.seed_big(&key);
    }
    Ok(Value::None)
}

fn r_random(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let o = rng_of(vm, &a[0])?;
    if let ObjKind::Rng(r) = &mut *o.kind.borrow_mut() {
        return Ok(Value::Float(r.random()));
    }
    Ok(Value::Float(0.0))
}

fn r_getrandbits(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 2)?;
    let o = rng_of(vm, &a[0])?;
    let k = vm.to_index(&a[1])?;
    if k < 0 {
        return Err(vm.value_error("number of bits must be non-negative"));
    }
    if let ObjKind::Rng(r) = &mut *o.kind.borrow_mut() {
        if k <= 32 {
            return Ok(Value::Int(if k == 0 { 0 } else { (r.genrand() >> (32 - k)) as i64 }));
        }
        return Ok(Value::from_big(r.getrandbits(k as usize)));
    }
    Ok(Value::Int(0))
}

fn r_getstate(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let o = rng_of(vm, &a[0])?;
    if let ObjKind::Rng(r) = &*o.kind.borrow() {
        let mut v: Vec<Value> = r.mt.iter().map(|x| Value::Int(*x as i64)).collect();
        v.push(Value::Int(r.mti as i64));
        return Ok(Value::tuple(v));
    }
    Ok(Value::None)
}

fn r_setstate(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 2)?;
    let o = rng_of(vm, &a[0])?;
    let Value::Tuple(items) = &a[1] else {
        return Err(vm.type_error("state vector must be a tuple"));
    };
    if items.len() != 625 {
        return Err(vm.value_error("state vector is the wrong size"));
    }
    let mut mt = Vec::with_capacity(624);
    for x in &items[..624] {
        mt.push(vm.to_index(x)? as u32);
    }
    let mti = vm.to_index(&items[624])?;
    if !(0..=624).contains(&mti) {
        return Err(vm.value_error("invalid state"));
    }
    let mti = mti as usize;
    if let ObjKind::Rng(r) = &mut *o.kind.borrow_mut() {
        r.mt = mt;
        r.mti = mti;
    }
    Ok(Value::None)
}

// -------------------------------------------------------------------------
// hashing (random seeds from strings, hashlib)

const K512: [u64; 80] = [
    0x428a2f98d728ae22, 0x7137449123ef65cd, 0xb5c0fbcfec4d3b2f, 0xe9b5dba58189dbbc, 0x3956c25bf348b538, 0x59f111f1b605d019, 0x923f82a4af194f9b, 0xab1c5ed5da6d8118, 0xd807aa98a3030242, 0x12835b0145706fbe, 0x243185be4ee4b28c, 0x550c7dc3d5ffb4e2, 0x72be5d74f27b896f, 0x80deb1fe3b1696b1, 0x9bdc06a725c71235, 0xc19bf174cf692694, 0xe49b69c19ef14ad2, 0xefbe4786384f25e3, 0x0fc19dc68b8cd5b5, 0x240ca1cc77ac9c65, 0x2de92c6f592b0275, 0x4a7484aa6ea6e483, 0x5cb0a9dcbd41fbd4, 0x76f988da831153b5, 0x983e5152ee66dfab, 0xa831c66d2db43210, 0xb00327c898fb213f, 0xbf597fc7beef0ee4, 0xc6e00bf33da88fc2, 0xd5a79147930aa725, 0x06ca6351e003826f, 0x142929670a0e6e70, 0x27b70a8546d22ffc, 0x2e1b21385c26c926, 0x4d2c6dfc5ac42aed, 0x53380d139d95b3df, 0x650a73548baf63de, 0x766a0abb3c77b2a8, 0x81c2c92e47edaee6, 0x92722c851482353b, 0xa2bfe8a14cf10364, 0xa81a664bbc423001, 0xc24b8b70d0f89791, 0xc76c51a30654be30, 0xd192e819d6ef5218, 0xd69906245565a910, 0xf40e35855771202a, 0x106aa07032bbd1b8, 0x19a4c116b8d2d0c8, 0x1e376c085141ab53, 0x2748774cdf8eeb99, 0x34b0bcb5e19b48a8, 0x391c0cb3c5c95a63, 0x4ed8aa4ae3418acb, 0x5b9cca4f7763e373, 0x682e6ff3d6b2b8a3, 0x748f82ee5defb2fc, 0x78a5636f43172f60, 0x84c87814a1f0ab72, 0x8cc702081a6439ec, 0x90befffa23631e28, 0xa4506cebde82bde9, 0xbef9a3f7b2c67915, 0xc67178f2e372532b, 0xca273eceea26619c, 0xd186b8c721c0c207, 0xeada7dd6cde0eb1e, 0xf57d4f7fee6ed178, 0x06f067aa72176fba, 0x0a637dc5a2c898a6, 0x113f9804bef90dae, 0x1b710b35131c471b, 0x28db77f523047d84, 0x32caab7b40c72493, 0x3c9ebe0a15c9bebc, 0x431d67c49c100d4c, 0x4cc5d4becb3e42b6, 0x597f299cfc657e2a, 0x5fcb6fab3ad6faec, 0x6c44198c4a475817,
];

pub fn sha512(data: &[u8]) -> [u8; 64] {
    let mut h: [u64; 8] = [0x6a09e667f3bcc908, 0xbb67ae8584caa73b, 0x3c6ef372fe94f82b, 0xa54ff53a5f1d36f1, 0x510e527fade682d1, 0x9b05688c2b3e6c1f, 0x1f83d9abfb41bd6b, 0x5be0cd19137e2179];
    let mut msg = data.to_vec();
    let bitlen = (data.len() as u128) * 8;
    msg.push(0x80);
    while msg.len() % 128 != 112 {
        msg.push(0);
    }
    msg.extend_from_slice(&bitlen.to_be_bytes());
    for chunk in msg.chunks(128) {
        let mut w = [0u64; 80];
        for i in 0..16 {
            w[i] = u64::from_be_bytes(chunk[i * 8..i * 8 + 8].try_into().unwrap());
        }
        for i in 16..80 {
            let s0 = w[i - 15].rotate_right(1) ^ w[i - 15].rotate_right(8) ^ (w[i - 15] >> 7);
            let s1 = w[i - 2].rotate_right(19) ^ w[i - 2].rotate_right(61) ^ (w[i - 2] >> 6);
            w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
        }
        let mut v = h;
        for i in 0..80 {
            let s1 = v[4].rotate_right(14) ^ v[4].rotate_right(18) ^ v[4].rotate_right(41);
            let ch = (v[4] & v[5]) ^ (!v[4] & v[6]);
            let t1 = v[7].wrapping_add(s1).wrapping_add(ch).wrapping_add(K512[i]).wrapping_add(w[i]);
            let s0 = v[0].rotate_right(28) ^ v[0].rotate_right(34) ^ v[0].rotate_right(39);
            let maj = (v[0] & v[1]) ^ (v[0] & v[2]) ^ (v[1] & v[2]);
            let t2 = s0.wrapping_add(maj);
            v[7] = v[6];
            v[6] = v[5];
            v[5] = v[4];
            v[4] = v[3].wrapping_add(t1);
            v[3] = v[2];
            v[2] = v[1];
            v[1] = v[0];
            v[0] = t1.wrapping_add(t2);
        }
        for i in 0..8 {
            h[i] = h[i].wrapping_add(v[i]);
        }
    }
    let mut out = [0u8; 64];
    for i in 0..8 {
        out[i * 8..i * 8 + 8].copy_from_slice(&h[i].to_be_bytes());
    }
    out
}

pub fn sha256(data: &[u8]) -> [u8; 32] {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3, 0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ];
    let mut h: [u32; 8] = [0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19];
    let mut msg = data.to_vec();
    let bitlen = (data.len() as u64) * 8;
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bitlen.to_be_bytes());
    for chunk in msg.chunks(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes(chunk[i * 4..i * 4 + 4].try_into().unwrap());
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
        }
        let mut v = h;
        for i in 0..64 {
            let s1 = v[4].rotate_right(6) ^ v[4].rotate_right(11) ^ v[4].rotate_right(25);
            let ch = (v[4] & v[5]) ^ (!v[4] & v[6]);
            let t1 = v[7].wrapping_add(s1).wrapping_add(ch).wrapping_add(K[i]).wrapping_add(w[i]);
            let s0 = v[0].rotate_right(2) ^ v[0].rotate_right(13) ^ v[0].rotate_right(22);
            let maj = (v[0] & v[1]) ^ (v[0] & v[2]) ^ (v[1] & v[2]);
            let t2 = s0.wrapping_add(maj);
            v[7] = v[6];
            v[6] = v[5];
            v[5] = v[4];
            v[4] = v[3].wrapping_add(t1);
            v[3] = v[2];
            v[2] = v[1];
            v[1] = v[0];
            v[0] = t1.wrapping_add(t2);
        }
        for i in 0..8 {
            h[i] = h[i].wrapping_add(v[i]);
        }
    }
    let mut out = [0u8; 32];
    for i in 0..8 {
        out[i * 4..i * 4 + 4].copy_from_slice(&h[i].to_be_bytes());
    }
    out
}

fn hashlib_module() -> Value {
    let (d, m) = module("_hashlib");
    put_fns(&d, &[("_hashlib.sha256", h_sha256), ("_hashlib.sha512", h_sha512)]);
    m
}

fn h_sha256(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let b = bytes_of(vm, &a[0])?;
    Ok(Value::Bytes(Rc::new(sha256(&b).to_vec())))
}

fn h_sha512(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let b = bytes_of(vm, &a[0])?;
    Ok(Value::Bytes(Rc::new(sha512(&b).to_vec())))
}

// -------------------------------------------------------------------------
// _struct: IEEE bit patterns for struct.py

fn fl(vm: &mut Vm, a: &[Value], i: usize, name: &str) -> PyResult<f64> {
    match a.get(i) {
        Some(v) => match v {
            Value::Str(_) | Value::None | Value::List(_) | Value::Tuple(_) => {
                let tn = vm.type_name(v);
                Err(vm.type_error(format!("must be real number, not {}", tn)))
            }
            _ => vm.to_f64(v),
        },
        None => Err(vm.type_error(format!("{}() missing required argument", name))),
    }
}

fn struct_module() -> Value {
    let (d, m) = module("_struct");
    put_fns(&d, &[("_struct.double_bits", st_double_bits), ("_struct.bits_double", st_bits_double), ("_struct.float_bits", st_float_bits), ("_struct.bits_float", st_bits_float), ("_struct.half_bits", st_half_bits), ("_struct.bits_half", st_bits_half)]);
    m
}

fn st_double_bits(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = fl(vm, a, 0, "pack")?;
    Ok(Value::from_big(BigInt::from_u128(x.to_bits() as u128, false)))
}

fn st_bits_double(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let b = crate::ops::big_of(&a[0]).ok_or_else(|| vm.type_error("expected int"))?;
    let lo = b.mag.first().copied().unwrap_or(0) as u64;
    let hi = b.mag.get(1).copied().unwrap_or(0) as u64;
    Ok(Value::Float(f64::from_bits((hi << 32) | lo)))
}

fn st_float_bits(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = fl(vm, a, 0, "pack")?;
    let f = x as f32;
    if f.is_infinite() && x.is_finite() {
        return Err(vm.err("OverflowError", "float too large to pack with f format"));
    }
    Ok(Value::Int(f.to_bits() as i64))
}

fn st_bits_float(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let b = vm.to_index(&a[0])? as u32;
    Ok(Value::Float(f32::from_bits(b) as f64))
}

fn st_half_bits(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = fl(vm, a, 0, "pack")?;
    let sign: u16 = if x.is_sign_negative() { 0x8000 } else { 0 };
    let ax = x.abs();
    let h: u16 = if x.is_nan() {
        0x7e00
    } else if ax.is_infinite() {
        0x7c00
    } else if ax == 0.0 {
        0
    } else {
        // round to nearest even at 11 significant bits
        let (m, e) = frexp(ax);
        // ax = m * 2^e, 0.5 <= m < 1
        let mut e = e - 1; // ax = (2m) * 2^e, 1 <= 2m < 2
        if e < -14 {
            // subnormal: units of 2^-24
            let q = (ax * 2f64.powi(24)).round_ties_even();
            if q >= 1024.0 {
                0x0400
            } else {
                q as u16
            }
        } else {
            let mut frac = ((2.0 * m - 1.0) * 1024.0).round_ties_even();
            if frac >= 1024.0 {
                frac = 0.0;
                e += 1;
            }
            if e > 15 {
                return Err(vm.err("OverflowError", "float too large to pack with e format"));
            }
            (((e + 15) as u16) << 10) | frac as u16
        }
    };
    Ok(Value::Int((sign | h) as i64))
}

fn frexp(x: f64) -> (f64, i32) {
    if x == 0.0 || !x.is_finite() {
        return (x, 0);
    }
    let bits = x.to_bits();
    let exp = ((bits >> 52) & 0x7ff) as i32;
    if exp == 0 {
        let (m, e) = frexp(x * 2f64.powi(64));
        return (m, e - 64);
    }
    let m = f64::from_bits((bits & !(0x7ff << 52)) | (1022 << 52));
    (m, exp - 1022)
}

fn st_bits_half(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let h = vm.to_index(&a[0])? as u16;
    let sign = if h & 0x8000 != 0 { -1.0 } else { 1.0 };
    let e = ((h >> 10) & 0x1f) as i32;
    let f = (h & 0x3ff) as f64;
    let v = if e == 0 {
        f * 2f64.powi(-24)
    } else if e == 31 {
        if f == 0.0 {
            f64::INFINITY
        } else {
            f64::NAN
        }
    } else {
        (1.0 + f / 1024.0) * 2f64.powi(e - 15)
    };
    Ok(Value::Float(sign * v))
}

// -------------------------------------------------------------------------
// _unicodedata: character names; the rest of unicodedata is Python

fn unicodedata_module() -> Value {
    let (d, m) = module("_unicodedata");
    put_fns(&d, &[("_unicodedata.name", ud_name), ("_unicodedata.lookup", ud_lookup)]);
    m
}

/// the code point of a one-character str
fn ud_char(vm: &mut Vm, v: &Value, f: &str) -> PyResult<u32> {
    if let Value::Str(s) = v {
        let mut cs = s.code_points();
        if let (Some(c), None) = (cs.next(), cs.next()) {
            return Ok(c);
        }
    }
    let tn = vm.type_name(v);
    Err(vm.type_error(format!("{}() argument must be a unicode character, not {}", f, tn)))
}

fn ud_name(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let c = ud_char(vm, &a[0], "name")?;
    match char::from_u32(c).and_then(crate::unicode_name_of) {
        Some(n) => Ok(Value::str(n)),
        None => match a.get(1) {
            Some(d) => Ok(d.clone()),
            None => Err(vm.value_error("no such name")),
        },
    }
}

fn ud_lookup(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let n = as_pystr(vm, &a[0], "lookup")?;
    match crate::unicode_name(&n.s) {
        Some(c) => Ok(Value::string(c.to_string())),
        None => Err(vm.err("KeyError", format!("undefined character name '{}'", n.s))),
    }
}

/// sys.breakpointhook: PYTHONBREAKPOINT names the callable, pdb.set_trace by default
fn sys_breakpointhook(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let m = vm.import("_breakpoint", &Value::None, 0, &vm.main_globals.clone())?;
    let h = vm.getattr(&m, &ps("hook"))?;
    vm.call_vec(&h, a.to_vec(), k)
}

pub fn bi_breakpoint(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let sys = vm.import("sys", &Value::None, 0, &vm.main_globals.clone())?;
    match vm.getattr_opt(&sys, &ps("breakpointhook"))? {
        Some(h) => vm.call_vec(&h, a.to_vec(), k),
        None => Err(vm.err("RuntimeError", "lost sys.breakpointhook")),
    }
}
