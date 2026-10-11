// SPDX-License-Identifier: MIT
//! The part of CPython's C-API test modules (_testcapi, _testlimitedcapi)
//! whose meaning is Python-level: the C limits of the build, raising and
//! creating exceptions the way the C API does, and attribute probes.

use crate::bigint::BigInt;
use crate::builtins::*;
use crate::modules::{module, put_fns};
use crate::value::*;
use crate::vm::Vm;
use std::cell::Cell;
use std::rc::Rc;

crate::gil_static! {
    /// set_nomemory's fault injection: (allocations counted, start, stop)
    static NOMEM: Cell<Option<(i64, i64, i64)>> = const { Cell::new(None) };
}

/// whether the allocation being made fails: allocations start+1 to stop
/// (or on without end when stop <= 0) counted from set_nomemory, as
/// CPython's hooked allocators count them
#[inline]
pub fn alloc_fails() -> bool {
    NOMEM.with(|n| match n.get() {
        None => false,
        Some((count, start, stop)) => {
            let count = count + 1;
            n.set(Some((count, start, stop)));
            count > start && (stop <= 0 || count <= stop)
        }
    })
}

pub fn testcapi_module(vm: &mut Vm) -> PyResult<Value> {
    let (d, m) = module("_testcapi");
    let big = |v: i128| Value::from_big(BigInt::from_i128(v));
    let limits = [
        ("CHAR_MAX", Value::Int(i8::MAX as i64)),
        ("CHAR_MIN", Value::Int(i8::MIN as i64)),
        ("UCHAR_MAX", Value::Int(u8::MAX as i64)),
        ("SHRT_MAX", Value::Int(i16::MAX as i64)),
        ("SHRT_MIN", Value::Int(i16::MIN as i64)),
        ("USHRT_MAX", Value::Int(u16::MAX as i64)),
        ("INT_MAX", Value::Int(i32::MAX as i64)),
        ("INT_MIN", Value::Int(i32::MIN as i64)),
        ("UINT_MAX", Value::Int(u32::MAX as i64)),
        ("INT32_MAX", Value::Int(i32::MAX as i64)),
        ("INT32_MIN", Value::Int(i32::MIN as i64)),
        ("UINT32_MAX", Value::Int(u32::MAX as i64)),
        ("LONG_MAX", Value::Int(i64::MAX)),
        ("LONG_MIN", Value::Int(i64::MIN)),
        ("ULONG_MAX", big(u64::MAX as i128)),
        ("LLONG_MAX", Value::Int(i64::MAX)),
        ("LLONG_MIN", Value::Int(i64::MIN)),
        ("ULLONG_MAX", big(u64::MAX as i128)),
        ("INT64_MAX", Value::Int(i64::MAX)),
        ("INT64_MIN", Value::Int(i64::MIN)),
        ("UINT64_MAX", big(u64::MAX as i128)),
        ("PY_SSIZE_T_MAX", Value::Int(i64::MAX)),
        ("PY_SSIZE_T_MIN", Value::Int(i64::MIN)),
        ("SIZE_MAX", big(u64::MAX as i128)),
        ("FLT_MAX", Value::Float(f32::MAX as f64)),
        ("FLT_MIN", Value::Float(f32::MIN_POSITIVE as f64)),
        ("DBL_MAX", Value::Float(f64::MAX)),
        ("DBL_MIN", Value::Float(f64::MIN_POSITIVE)),
        ("SIZEOF_INT", Value::Int(4)),
        ("SIZEOF_LONG", Value::Int(8)),
        ("SIZEOF_VOID_P", Value::Int(8)),
        ("SIZEOF_TIME_T", Value::Int(8)),
        ("SIZEOF_WCHAR_T", Value::Int(4)),
    ];
    for (n, v) in limits {
        d.borrow_mut().set_s(n, v);
    }
    put_fns(
        &d,
        &[
            ("_testcapi.raise_exception", raise_exception),
            ("_testcapi.raise_memoryerror", raise_memoryerror),
            ("_testcapi.make_exception_with_doc", make_exception_with_doc),
            ("_testcapi.set_nomemory", set_nomemory),
            ("_testcapi.remove_mem_hooks", remove_mem_hooks),
            ("_testcapi.run_in_subinterp", run_in_subinterp),
        ],
    );
    let rie = recursing_infinitely_error(vm)?;
    d.borrow_mut().set_s("RecursingInfinitelyError", rie);
    Ok(m)
}

pub fn testlimitedcapi_module() -> Value {
    let (d, m) = module("_testlimitedcapi");
    put_fns(&d, &[("_testlimitedcapi.object_hasattrstring", object_hasattrstring)]);
    m
}

/// type(name, bases, ns), as PyType_Type is called from C
fn make_type(vm: &mut Vm, name: &str, bases: Vec<Value>, ns: Value) -> PyResult<Value> {
    let ty = vm.builtins.borrow().get_s("type").unwrap_or(Value::None);
    vm.call(&ty, &[Value::str(name), Value::tuple(bases), ns])
}

/// the exception class whose instantiation raises itself again, so that
/// creating it ends in a RecursionError
fn recursing_infinitely_error(vm: &mut Vm) -> PyResult<Value> {
    let mut ns = Dict::new();
    ns.set_s("__module__", Value::str("_testcapi"));
    ns.set_s("__init__", native("RecursingInfinitelyError.__init__", recursing_init));
    let base = Value::Class(vm.exc_class("Exception"));
    make_type(vm, "RecursingInfinitelyError", vec![base], Value::dict(ns))
}

fn recursing_init(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let cls = Value::Class(vm.class_of(&a[0]));
    let e = vm.make_raisable(cls)?;
    Err(Exc { value: e })
}

/// raise_exception(exc, nargs): PyErr_SetObject(exc, (0, 1, ..., nargs-1))
/// and its normalization, which instantiates exc in the caller's frame
fn raise_exception(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "raise_exception", a, 2, 2)?;
    let n = vm.to_index(&a[1])?;
    if n > i32::MAX as i64 || n < i32::MIN as i64 {
        return Err(vm.err("OverflowError", "signed integer is greater than maximum"));
    }
    let is_exc = matches!(&a[0], Value::Class(c) if c.is_subclass(&vm.t.base_exception));
    if !is_exc {
        let r = vm.repr(&a[0])?;
        return Err(vm.err("SystemError", format!("_PyErr_SetObject: exception {} is not a BaseException subclass", r)));
    }
    let args: Vec<Value> = (0..n.max(0)).map(Value::Int).collect();
    let v = vm.call_vec(&a[0], args, &[])?;
    if !vm.class_of(&v).is_subclass(&vm.t.base_exception) {
        let (cn, tn) = (vm.repr(&a[0])?, vm.type_name(&v));
        return Err(vm.type_error(format!("calling {} should have returned an instance of BaseException, not {}", cn, tn)));
    }
    Err(Exc { value: v })
}

/// raise_memoryerror(): PyErr_NoMemory()
fn raise_memoryerror(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "raise_memoryerror", a, 0, 0)?;
    let cls = vm.exc_class("MemoryError");
    Err(Exc { value: vm.new_exc(&cls, vec![]) })
}

/// set_nomemory(start, stop=0): the allocations of objects after the
/// start-th one, up to the stop-th, fail with MemoryError
fn set_nomemory(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "set_nomemory", a, 1, 2)?;
    let start = vm.to_index(&a[0])?;
    let stop = match a.get(1) {
        Some(v) => vm.to_index(v)?,
        None => 0,
    };
    NOMEM.with(|n| n.set(Some((0, start, stop))));
    Ok(Value::None)
}

/// remove_mem_hooks(): allocations no longer fail
fn remove_mem_hooks(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "remove_mem_hooks", a, 0, 0)?;
    NOMEM.with(|n| n.set(None));
    Ok(Value::None)
}

/// run_in_subinterp(code): runs code in an interpreter of its own, with
/// its own builtins, types, sys and modules, sharing only the filesystem
/// and the standard streams; what it does not catch is printed. 0, or -1
/// when the code raised.
fn run_in_subinterp(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "run_in_subinterp", a, 1, 1)?;
    let code = match &a[0] {
        Value::Str(s) => s.s.to_string(),
        v => {
            let tn = vm.type_name(v);
            return Err(vm.type_error(format!("run_in_subinterp() argument must be str, not {}", tn)));
        }
    };
    use crate::format::MAX_STR_DIGITS;
    use std::sync::atomic::Ordering::Relaxed;
    // the integer string limit is the interpreter's: a new one starts at the default
    let digits = MAX_STR_DIGITS.swap(4300, Relaxed);
    let mut sub = Vm::new(std::mem::replace(&mut vm.fs, crate::fs::Fs::new()));
    sub.echo = vm.echo;
    sub.search_path = vm.search_path.clone();
    sub.host = vm.host.clone();
    sub.argv = vec![String::new()];
    let g = sub.main_globals.clone();
    g.borrow_mut().remove_str(&PyStr::new("__file__".to_string()));
    let status = match sub.run_source(&code, "<string>", g) {
        Ok(_) => 0,
        Err(e) => {
            crate::print_uncaught(&mut sub, &e.value);
            -1
        }
    };
    sub.finalize();
    vm.fs = std::mem::replace(&mut sub.fs, crate::fs::Fs::new());
    let (out, err) = (std::mem::take(&mut sub.out), std::mem::take(&mut sub.err_out));
    drop(sub);
    vm.out.push_str(&out);
    vm.err_out.push_str(&err);
    MAX_STR_DIGITS.store(digits, Relaxed);
    Ok(Value::Int(status))
}

/// make_exception_with_doc(name, doc=None, base=None, dict=None):
/// PyErr_NewExceptionWithDoc
fn make_exception_with_doc(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    if a.len() > 4 {
        return Err(vm.type_error(format!("make_exception_with_doc() takes at most 4 arguments ({} given)", a.len())));
    }
    let opt = |i: usize, n: &str| arg_or_kw(a, i, k, n).filter(|v| !v.is_none());
    let name = match opt(0, "name") {
        Some(Value::Str(s)) => s.s.to_string(),
        _ => return Err(vm.type_error("make_exception_with_doc() argument 'name' must be str")),
    };
    let doc = opt(1, "doc");
    if let Some(d) = &doc {
        if !matches!(d, Value::Str(_)) {
            return Err(vm.type_error("make_exception_with_doc() argument 'doc' must be str"));
        }
    }
    let Some(dot) = name.rfind('.') else {
        return Err(vm.err("SystemError", "PyErr_NewException: name must be module.class"));
    };
    let base = opt(2, "base").unwrap_or_else(|| Value::Class(vm.exc_class("Exception")));
    let ns = match opt(3, "dict") {
        Some(d @ Value::Dict(_)) => d,
        Some(_) => return Err(vm.err("SystemError", "bad argument to internal function")),
        None => Value::dict(Dict::new()),
    };
    if let Value::Dict(nd) = &ns {
        let mut nd = nd.borrow_mut();
        if let Some(d) = doc {
            nd.set_s("__doc__", d);
        }
        if nd.get_s("__module__").is_none() {
            nd.set_s("__module__", Value::str(&name[..dot]));
        }
    }
    let bases = match &base {
        Value::Tuple(t) => t.to_vec(),
        b => vec![b.clone()],
    };
    make_type(vm, &name[dot + 1..], bases, ns)
}

/// object_hasattrstring(obj, name: bytes): PyObject_HasAttrString, which
/// answers 1 or 0 and leaves no exception set
fn object_hasattrstring(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "object_hasattrstring", a, 2, 2)?;
    let name = match &a[1] {
        Value::None => return Ok(Value::Int(0)),
        b => bytes_of(vm, b)?,
    };
    let Ok(name) = String::from_utf8(name) else {
        return Ok(Value::Int(0));
    };
    match vm.getattr(&a[0], &Rc::new(PyStr::new(name))) {
        Ok(_) => Ok(Value::Int(1)),
        Err(e) if vm.exc_is(&e, "AttributeError") => Ok(Value::Int(0)),
        Err(e) => {
            vm.unraisable(e, &a[0]);
            Ok(Value::Int(0))
        }
    }
}
