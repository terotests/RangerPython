// SPDX-License-Identifier: MIT
//! The built-in types, the builtins module, and the helpers the object
//! protocol in ops.rs calls for built-in values.

use crate::ast::{BinOp, CmpOp};
use crate::bigint::BigInt;
use crate::value::*;
use crate::vm::{Frame, Types, Vm};
use std::cell::RefCell;
use std::rc::Rc;

pub type Kw<'a> = &'a [(Rc<PyStr>, Value)];

pub fn ps(s: &str) -> Rc<PyStr> {
    Rc::new(PyStr::new(s.to_string()))
}

// -------------------------------------------------------------------------
// small helpers natives use

pub fn kwarg(kw: Kw, name: &str) -> Option<Value> {
    kw.iter().find(|(k, _)| &*k.s == name).map(|(_, v)| v.clone())
}

/// positional argument `i`, or the keyword `name`
pub fn arg_or_kw(args: &[Value], i: usize, kw: Kw, name: &str) -> Option<Value> {
    if i < args.len() {
        Some(args[i].clone())
    } else {
        kwarg(kw, name)
    }
}

pub fn check_nargs(vm: &mut Vm, name: &str, args: &[Value], min: usize, max: usize) -> PyResult<()> {
    let n = args.len();
    if n < min || n > max {
        let msg = if min == max {
            match min {
                0 => format!("{}() takes no arguments ({} given)", name, n),
                1 => format!("{}() takes exactly one argument ({} given)", name, n),
                _ => format!("{}() takes exactly {} arguments ({} given)", name, min, n),
            }
        } else if n < min {
            format!("{} expected at least {} argument{}, got {}", name, min, if min == 1 { "" } else { "s" }, n)
        } else {
            format!("{} expected at most {} argument{}, got {}", name, max, if max == 1 { "" } else { "s" }, n)
        };
        return Err(vm.type_error(msg));
    }
    Ok(())
}

/// the method's own argument count check (self excluded)
pub fn margs(vm: &mut Vm, name: &str, args: &[Value], min: usize, max: usize) -> PyResult<()> {
    let n = args.len().saturating_sub(1);
    if n < min || n > max {
        let msg = if max == 0 {
            format!("{}() takes no arguments ({} given)", name, n)
        } else if min == max && min == 1 {
            format!("{}() takes exactly one argument ({} given)", name, n)
        } else if n < min {
            format!("{} expected at least {} argument{}, got {}", name, min, if min == 1 { "" } else { "s" }, n)
        } else {
            format!("{} expected at most {} argument{}, got {}", name, max, if max == 1 { "" } else { "s" }, n)
        };
        return Err(vm.type_error(msg));
    }
    Ok(())
}

pub fn mk_iter(it: Iter) -> Value {
    Value::Iter(Rc::new(RefCell::new(it)))
}

pub fn new_dict() -> Rc<RefCell<Dict>> {
    Rc::new(RefCell::new(Dict::new()))
}

pub fn set_from(d: Dict) -> Value {
    Value::Set(Rc::new(RefCell::new(d)))
}

thread_local! {
    static CHARS: Vec<Rc<PyStr>> = (0u8..128).map(|b| Rc::new(PyStr::new((b as char).to_string()))).collect();
}

/// a one-character str (ASCII ones are shared)
#[inline]
pub fn char_value(c: char) -> Value {
    if (c as u32) < 128 {
        CHARS.with(|t| Value::Str(t[c as usize].clone()))
    } else {
        Value::string(c.to_string())
    }
}

pub fn range_len(r: &Rc<(i64, i64, i64)>) -> usize {
    let (start, stop, step) = (r.0 as i128, r.1 as i128, r.2 as i128);
    let n = if step > 0 {
        if stop > start {
            (stop - start + step - 1) / step
        } else {
            0
        }
    } else if start > stop {
        (start - stop - step - 1) / (-step)
    } else {
        0
    };
    n as usize
}

pub fn range_items(r: &Rc<(i64, i64, i64)>) -> Vec<i64> {
    let n = range_len(r);
    (0..n).map(|i| r.0 + r.2 * i as i64).collect()
}

pub fn bytes_to_latin1(v: &Value) -> String {
    match v {
        Value::Bytes(b) => b.iter().map(|&c| c as char).collect(),
        Value::ByteArray(b) => b.borrow().iter().map(|&c| c as char).collect(),
        _ => String::new(),
    }
}

/// the bytes of a bytes-like value
pub fn bytes_of(vm: &mut Vm, v: &Value) -> PyResult<Vec<u8>> {
    match v {
        Value::Bytes(b) => Ok((**b).clone()),
        Value::ByteArray(b) => Ok(b.borrow().clone()),
        Value::Instance(_) => {
            let b = vm.base_of(v);
            if matches!(b, Value::Bytes(_) | Value::ByteArray(_)) {
                return bytes_of(vm, &b);
            }
            let tn = vm.type_name(v);
            Err(vm.type_error(format!("a bytes-like object is required, not '{}'", tn)))
        }
        _ => {
            let tn = vm.type_name(v);
            Err(vm.type_error(format!("a bytes-like object is required, not '{}'", tn)))
        }
    }
}

/// the str inside a str or a str subclass
pub fn as_pystr(vm: &mut Vm, v: &Value, what: &str) -> PyResult<Rc<PyStr>> {
    match v {
        Value::Str(s) => Ok(s.clone()),
        Value::Instance(_) => match vm.base_of(v) {
            Value::Str(s) => Ok(s),
            _ => {
                let tn = vm.type_name(v);
                Err(vm.type_error(format!("{} must be str, not {}", what, tn)))
            }
        },
        _ => {
            let tn = vm.type_name(v);
            Err(vm.type_error(format!("{} must be str, not {}", what, tn)))
        }
    }
}

// -------------------------------------------------------------------------
// indexing

/// a normalised index into a sequence of `len`
pub fn norm_index(vm: &mut Vm, idx: &Value, len: usize, what: &str) -> PyResult<usize> {
    let i = match idx {
        Value::Int(i) => *i,
        Value::Bool(b) => *b as i64,
        Value::Big(_) => return Err(vm.index_error(format!("cannot fit 'int' into an index-sized integer"))),
        _ => match vm.to_index(idx) {
            Ok(i) => i,
            Err(_) => {
                let tn = vm.type_name(idx);
                return Err(vm.type_error(format!("{} indices must be integers or slices, not {}", what, tn)));
            }
        },
    };
    let j = if i < 0 { i + len as i64 } else { i };
    if j < 0 || j >= len as i64 {
        return Err(vm.index_error(format!("{} index out of range", what)));
    }
    Ok(j as usize)
}

/// start, stop, step of a slice over `len` items (CPython's PySlice_AdjustIndices)
pub fn slice_indices(vm: &mut Vm, s: &(Value, Value, Value), len: usize) -> PyResult<(i64, i64, i64)> {
    let len = len as i64;
    let step = if s.2.is_none() { 1 } else { vm.to_index(&s.2).or_else(|_| Err(vm.type_error("slice indices must be integers or None or have an __index__ method")))? };
    if step == 0 {
        return Err(vm.value_error("slice step cannot be zero"));
    }
    let conv = |vm: &mut Vm, v: &Value, dflt: i64| -> PyResult<i64> {
        if v.is_none() {
            return Ok(dflt);
        }
        match v {
            Value::Big(b) => Ok(if b.neg { i64::MIN / 2 } else { i64::MAX / 2 }),
            _ => vm.to_index(v).or_else(|_| Err(vm.type_error("slice indices must be integers or None or have an __index__ method"))),
        }
    };
    let (lo_d, hi_d) = if step < 0 { (len - 1, -1) } else { (0, len) };
    let mut start = conv(vm, &s.0, if step < 0 { i64::MAX } else { 0 })?;
    let mut stop = conv(vm, &s.1, if step < 0 { i64::MIN } else { len })?;
    let _ = (lo_d, hi_d);
    if start < 0 {
        start += len;
        if start < 0 {
            start = if step < 0 { -1 } else { 0 };
        }
    } else if start >= len {
        start = if step < 0 { len - 1 } else { len };
    }
    if stop < 0 {
        stop = stop.saturating_add(len);
        if stop < 0 {
            stop = if step < 0 { -1 } else { 0 };
        }
    } else if stop >= len {
        stop = if step < 0 { len - 1 } else { len };
    }
    Ok((start, stop, step))
}

fn slice_positions(start: i64, stop: i64, step: i64) -> Vec<usize> {
    let mut out = vec![];
    let mut i = start;
    if step > 0 {
        while i < stop {
            out.push(i as usize);
            i += step;
        }
    } else {
        while i > stop {
            out.push(i as usize);
            i += step;
        }
    }
    out
}

fn str_chars(s: &PyStr) -> Vec<char> {
    s.s.chars().collect()
}

pub fn seq_getitem(vm: &mut Vm, obj: &Value, idx: &Value) -> PyResult<Value> {
    if let Value::Slice(sl) = idx {
        return match obj {
            Value::List(l) => {
                let lb = l.borrow();
                let (a, b, st) = slice_indices(vm, sl, lb.len())?;
                if st == 1 {
                    return Ok(Value::list(if a < b { lb[a as usize..b as usize].to_vec() } else { vec![] }));
                }
                Ok(Value::list(slice_positions(a, b, st).into_iter().map(|i| lb[i].clone()).collect()))
            }
            Value::Tuple(t) => {
                let (a, b, st) = slice_indices(vm, sl, t.len())?;
                if st == 1 && a == 0 && b == t.len() as i64 {
                    return Ok(obj.clone());
                }
                Ok(Value::tuple(slice_positions(a, b, st).into_iter().map(|i| t[i].clone()).collect()))
            }
            Value::Str(s) => {
                if s.ascii {
                    let by = s.s.as_bytes();
                    let (a, b, st) = slice_indices(vm, sl, by.len())?;
                    if st == 1 {
                        return Ok(Value::str(if a < b { &s.s[a as usize..b as usize] } else { "" }));
                    }
                    let out: String = slice_positions(a, b, st).into_iter().map(|i| by[i] as char).collect();
                    return Ok(Value::string(out));
                }
                let cs = str_chars(s);
                let (a, b, st) = slice_indices(vm, sl, cs.len())?;
                Ok(Value::string(slice_positions(a, b, st).into_iter().map(|i| cs[i]).collect()))
            }
            Value::Bytes(by) => {
                let (a, b, st) = slice_indices(vm, sl, by.len())?;
                Ok(Value::Bytes(Rc::new(slice_positions(a, b, st).into_iter().map(|i| by[i]).collect())))
            }
            Value::ByteArray(by) => {
                let by = by.borrow();
                let (a, b, st) = slice_indices(vm, sl, by.len())?;
                Ok(Value::ByteArray(Rc::new(RefCell::new(slice_positions(a, b, st).into_iter().map(|i| by[i]).collect()))))
            }
            Value::Range(r) => {
                let n = range_len(r);
                let (a, b, st) = slice_indices(vm, sl, n)?;
                let start = r.0 + a * r.2;
                let stop = r.0 + b * r.2;
                Ok(Value::Range(Rc::new((start, stop, r.2 * st))))
            }
            _ => Err(vm.type_error("unsliceable")),
        };
    }
    match obj {
        Value::List(l) => {
            let n = l.borrow().len();
            if let Value::Int(i) = idx {
                let j = if *i < 0 { *i + n as i64 } else { *i };
                if j >= 0 && (j as usize) < n {
                    return Ok(l.borrow()[j as usize].clone());
                }
                return Err(vm.index_error("list index out of range"));
            }
            let i = norm_index(vm, idx, n, "list")?;
            Ok(l.borrow()[i].clone())
        }
        Value::Tuple(t) => {
            let i = norm_index(vm, idx, t.len(), "tuple")?;
            Ok(t[i].clone())
        }
        Value::Str(s) => {
            if s.ascii {
                let i = norm_index(vm, idx, s.s.len(), "string")?;
                return Ok(char_value(s.s.as_bytes()[i] as char));
            }
            let n = s.char_len();
            let i = norm_index(vm, idx, n, "string")?;
            Ok(char_value(s.s.chars().nth(i).unwrap()))
        }
        Value::Bytes(b) => {
            let i = norm_index(vm, idx, b.len(), "index")?;
            Ok(Value::Int(b[i] as i64))
        }
        Value::ByteArray(b) => {
            let n = b.borrow().len();
            let i = norm_index(vm, idx, n, "bytearray")?;
            Ok(Value::Int(b.borrow()[i] as i64))
        }
        Value::Range(r) => {
            let n = range_len(r);
            let i = norm_index(vm, idx, n, "range object")?;
            Ok(Value::Int(r.0 + r.2 * i as i64))
        }
        _ => Err(vm.type_error("not subscriptable")),
    }
}

pub fn seq_setitem(vm: &mut Vm, obj: &Value, idx: &Value, v: Value) -> PyResult<()> {
    match obj {
        Value::List(l) => {
            if let Value::Slice(sl) = idx {
                let items = vm.to_vec(&v)?;
                let n = l.borrow().len();
                let (a, b, st) = slice_indices(vm, sl, n)?;
                if st == 1 {
                    let b = b.max(a);
                    l.borrow_mut().splice(a as usize..b as usize, items);
                    return Ok(());
                }
                let pos = slice_positions(a, b, st);
                if pos.len() != items.len() {
                    return Err(vm.value_error(format!("attempt to assign sequence of size {} to extended slice of size {}", items.len(), pos.len())));
                }
                let mut lb = l.borrow_mut();
                for (p, x) in pos.into_iter().zip(items) {
                    lb[p] = x;
                }
                return Ok(());
            }
            let n = l.borrow().len();
            let i = norm_index(vm, idx, n, "list assignment")?;
            l.borrow_mut()[i] = v;
            Ok(())
        }
        Value::ByteArray(b) => {
            if let Value::Slice(sl) = idx {
                let items = match &v {
                    Value::Bytes(_) | Value::ByteArray(_) => bytes_of(vm, &v)?,
                    _ => {
                        let vs = vm.to_vec(&v)?;
                        let mut out = vec![];
                        for x in vs {
                            out.push(vm.to_index(&x)? as u8);
                        }
                        out
                    }
                };
                let n = b.borrow().len();
                let (a, e, st) = slice_indices(vm, sl, n)?;
                if st == 1 {
                    let e = e.max(a);
                    b.borrow_mut().splice(a as usize..e as usize, items);
                    return Ok(());
                }
                let pos = slice_positions(a, e, st);
                let mut bb = b.borrow_mut();
                for (p, x) in pos.into_iter().zip(items) {
                    bb[p] = x;
                }
                return Ok(());
            }
            let n = b.borrow().len();
            let i = norm_index(vm, idx, n, "bytearray")?;
            let x = vm.to_index(&v)?;
            if !(0..256).contains(&x) {
                return Err(vm.value_error("byte must be in range(0, 256)"));
            }
            b.borrow_mut()[i] = x as u8;
            Ok(())
        }
        _ => Err(vm.type_error("object does not support item assignment")),
    }
}

pub fn seq_delitem(vm: &mut Vm, obj: &Value, idx: &Value) -> PyResult<()> {
    match obj {
        Value::List(l) => {
            if let Value::Slice(sl) = idx {
                let n = l.borrow().len();
                let (a, b, st) = slice_indices(vm, sl, n)?;
                let mut pos = slice_positions(a, b, st);
                pos.sort_unstable();
                let mut lb = l.borrow_mut();
                for p in pos.into_iter().rev() {
                    lb.remove(p);
                }
                return Ok(());
            }
            let n = l.borrow().len();
            let i = norm_index(vm, idx, n, "list assignment")?;
            l.borrow_mut().remove(i);
            Ok(())
        }
        Value::ByteArray(b) => {
            if let Value::Slice(sl) = idx {
                let n = b.borrow().len();
                let (a, e, st) = slice_indices(vm, sl, n)?;
                let mut pos = slice_positions(a, e, st);
                pos.sort_unstable();
                let mut bb = b.borrow_mut();
                for p in pos.into_iter().rev() {
                    bb.remove(p);
                }
                return Ok(());
            }
            let n = b.borrow().len();
            let i = norm_index(vm, idx, n, "bytearray")?;
            b.borrow_mut().remove(i);
            Ok(())
        }
        _ => Err(vm.type_error("object does not support item deletion")),
    }
}

// -------------------------------------------------------------------------
// sets

pub fn to_set(vm: &mut Vm, v: &Value) -> PyResult<Value> {
    match v {
        Value::Set(_) | Value::FrozenSet(_) => Ok(v.clone()),
        _ => {
            let items = vm.to_vec(v)?;
            let d = new_dict();
            for x in items {
                vm.set_add(&d, x)?;
            }
            Ok(Value::Set(d))
        }
    }
}

fn set_dict(v: &Value) -> Rc<RefCell<Dict>> {
    match v {
        Value::Set(d) | Value::FrozenSet(d) => d.clone(),
        _ => new_dict(),
    }
}

/// a | b, a & b, a - b, a ^ b; the result has a's kind
pub fn set_op(vm: &mut Vm, op: BinOp, a: &Value, b: &Value) -> PyResult<Value> {
    let (da, db) = (set_dict(a), set_dict(b));
    let out = new_dict();
    match op {
        BinOp::BitOr => {
            *out.borrow_mut() = da.borrow().copy();
            for e in db.borrow().keys() {
                vm.set_add(&out, e)?;
            }
        }
        BinOp::BitAnd => {
            for e in da.borrow().keys() {
                if vm.set_contains(&db, &e)? {
                    vm.set_add(&out, e)?;
                }
            }
        }
        BinOp::Sub => {
            for e in da.borrow().keys() {
                if !vm.set_contains(&db, &e)? {
                    vm.set_add(&out, e)?;
                }
            }
        }
        _ => {
            for e in da.borrow().keys() {
                if !vm.set_contains(&db, &e)? {
                    vm.set_add(&out, e)?;
                }
            }
            for e in db.borrow().keys() {
                if !vm.set_contains(&da, &e)? {
                    vm.set_add(&out, e)?;
                }
            }
        }
    }
    Ok(match a {
        Value::FrozenSet(_) => Value::FrozenSet(out),
        _ => Value::Set(out),
    })
}

pub fn is_subset(vm: &mut Vm, a: &Rc<RefCell<Dict>>, b: &Rc<RefCell<Dict>>) -> PyResult<bool> {
    if a.borrow().len() > b.borrow().len() {
        return Ok(false);
    }
    for k in a.borrow().keys() {
        if !vm.set_contains(b, &k)? {
            return Ok(false);
        }
    }
    Ok(true)
}

pub fn set_compare(vm: &mut Vm, op: CmpOp, a: &Value, b: &Value) -> PyResult<Value> {
    let sa = to_set(vm, a)?;
    let sb = to_set(vm, b)?;
    let (da, db) = (set_dict(&sa), set_dict(&sb));
    let (la, lb) = (da.borrow().len(), db.borrow().len());
    Ok(Value::Bool(match op {
        CmpOp::Le => is_subset(vm, &da, &db)?,
        CmpOp::Lt => la < lb && is_subset(vm, &da, &db)?,
        CmpOp::Ge => is_subset(vm, &db, &da)?,
        _ => la > lb && is_subset(vm, &db, &da)?,
    }))
}

// -------------------------------------------------------------------------
// complex numbers

fn complex_parts(vm: &mut Vm, v: &Value) -> PyResult<Option<(f64, f64)>> {
    Ok(match v {
        Value::Complex(c) => Some((c.0, c.1)),
        Value::Int(_) | Value::Float(_) | Value::Bool(_) | Value::Big(_) => Some((vm.to_f64(v)?, 0.0)),
        _ => None,
    })
}

pub fn complex_div(vm: &mut Vm, a: (f64, f64), b: (f64, f64)) -> PyResult<(f64, f64)> {
    let d = b.0 * b.0 + b.1 * b.1;
    if d == 0.0 {
        return Err(vm.err("ZeroDivisionError", "division by zero"));
    }
    Ok(((a.0 * b.0 + a.1 * b.1) / d, (a.1 * b.0 - a.0 * b.1) / d))
}

pub fn complex_pow(vm: &mut Vm, a: (f64, f64), b: (f64, f64)) -> PyResult<(f64, f64)> {
    if b.1 == 0.0 && b.0.fract() == 0.0 && b.0.abs() < 100.0 {
        // integer powers by repeated multiplication, like CPython
        let n = b.0 as i64;
        let mut r = (1.0, 0.0);
        let mut base = a;
        let mut e = n.unsigned_abs();
        while e > 0 {
            if e & 1 == 1 {
                r = (r.0 * base.0 - r.1 * base.1, r.0 * base.1 + r.1 * base.0);
            }
            base = (base.0 * base.0 - base.1 * base.1, 2.0 * base.0 * base.1);
            e >>= 1;
        }
        if n < 0 {
            return complex_div(vm, (1.0, 0.0), r);
        }
        return Ok(r);
    }
    if a == (0.0, 0.0) {
        if b.1 != 0.0 || b.0 < 0.0 {
            return Err(vm.err("ZeroDivisionError", "0.0 to a negative or complex power"));
        }
        return Ok((0.0, 0.0));
    }
    let r = (a.0 * a.0 + a.1 * a.1).sqrt();
    let th = a.1.atan2(a.0);
    let lr = r.ln();
    let mag = (b.0 * lr - b.1 * th).exp();
    let ang = b.1 * lr + b.0 * th;
    Ok((mag * ang.cos(), mag * ang.sin()))
}

pub fn complex_binop(vm: &mut Vm, op: BinOp, a: &Value, b: &Value) -> PyResult<Option<Value>> {
    let (x, y) = match (complex_parts(vm, a)?, complex_parts(vm, b)?) {
        (Some(x), Some(y)) => (x, y),
        _ => return Ok(None),
    };
    let r = match op {
        BinOp::Add => (x.0 + y.0, x.1 + y.1),
        BinOp::Sub => (x.0 - y.0, x.1 - y.1),
        BinOp::Mul => (x.0 * y.0 - x.1 * y.1, x.0 * y.1 + x.1 * y.0),
        BinOp::Div => complex_div(vm, x, y)?,
        BinOp::Pow => complex_pow(vm, x, y)?,
        _ => return Ok(None),
    };
    Ok(Some(Value::Complex(Rc::new(r))))
}

// -------------------------------------------------------------------------
// generic aliases and unions (list[int], int | None)

pub fn alias_class(vm: &mut Vm, name: &str) -> Rc<Class> {
    let key = format!("__{}__", name);
    if let Some(Value::Class(c)) = vm.builtins.borrow().get_s(&key) {
        return c;
    }
    let c = Class::new(name, vec![vm.t.object.clone()], Builtin::Obj);
    *c.mro.borrow_mut() = vec![c.clone(), vm.t.object.clone()];
    c.dict.borrow_mut().set_s("__module__", Value::str("types"));
    let lname: &'static str = if name == "GenericAlias" { "GenericAlias.__repr__" } else { "UnionType.__repr__" };
    def(&c, lname, alias_repr);
    if name == "GenericAlias" {
        def(&c, "GenericAlias.__call__", alias_call);
        def(&c, "GenericAlias.__getitem__", alias_getitem);
    }
    def(&c, "alias.__eq__", alias_eq);
    def(&c, "alias.__hash__", alias_hash);
    def(&c, "alias.__or__", alias_or);
    def(&c, "alias.__ror__", alias_ror);
    vm.builtins.borrow_mut().set_s(&key, Value::Class(c.clone()));
    c
}

fn alias_parts(v: &Value) -> (Value, Vec<Value>) {
    if let Value::Obj(o) = v {
        if let ObjKind::Other(Value::Tuple(t)) = &*o.kind.borrow() {
            let args = match &t[1] {
                Value::Tuple(a) => a.to_vec(),
                _ => vec![],
            };
            return (t[0].clone(), args);
        }
    }
    (Value::None, vec![])
}

fn type_repr_short(vm: &mut Vm, v: &Value) -> PyResult<String> {
    match v {
        Value::Class(c) => {
            let m = c.dict.borrow().get_s("__module__");
            Ok(match m {
                Some(Value::Str(m)) if &*m.s != "builtins" => format!("{}.{}", m.s, c.qualname.borrow().s),
                _ => c.name(),
            })
        }
        Value::None => Ok("None".into()),
        Value::Ellipsis => Ok("...".into()),
        Value::List(l) => {
            let items = l.borrow().clone();
            let mut parts = vec![];
            for x in &items {
                parts.push(type_repr_short(vm, x)?);
            }
            Ok(format!("[{}]", parts.join(", ")))
        }
        _ => vm.repr(v),
    }
}

fn alias_repr(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let (origin, args) = alias_parts(&a[0]);
    let mut parts = vec![];
    for x in &args {
        parts.push(type_repr_short(vm, x)?);
    }
    if origin.is_none() {
        return Ok(Value::string(parts.join(" | ")));
    }
    let o = type_repr_short(vm, &origin)?;
    Ok(Value::string(format!("{}[{}]", o, if args.is_empty() { "()".to_string() } else { parts.join(", ") })))
}

fn alias_call(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let (origin, _) = alias_parts(&a[0]);
    vm.call_vec(&origin, a[1..].to_vec(), k)
}

fn alias_getitem(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let (origin, _) = alias_parts(&a[0]);
    generic_alias(vm, &origin, &a[1])
}

fn alias_eq(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let (o1, a1) = alias_parts(&a[0]);
    if !matches!(a[1], Value::Obj(_)) {
        return Ok(Value::NotImplemented);
    }
    let (o2, a2) = alias_parts(&a[1]);
    let r = vm.eq(&o1, &o2)? && vm.eq(&Value::tuple(a1), &Value::tuple(a2))?;
    Ok(Value::Bool(r))
}

fn alias_hash(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let (o, args) = alias_parts(&a[0]);
    let h = vm.hash(&Value::tuple(vec![o, Value::tuple(args)]))?;
    Ok(Value::Int(h as i64 >> 1))
}

fn alias_or(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    make_union(vm, &a[0], &a[1])
}

fn alias_ror(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    make_union(vm, &a[1], &a[0])
}

pub fn generic_alias(vm: &mut Vm, origin: &Value, idx: &Value) -> PyResult<Value> {
    let c = alias_class(vm, "GenericAlias");
    let args = match idx {
        Value::Tuple(t) => (**t).clone(),
        other => vec![other.clone()],
    };
    Ok(Value::Obj(Rc::new(NativeObj { class: c, kind: RefCell::new(ObjKind::Other(Value::tuple(vec![origin.clone(), Value::tuple(args)]))) })))
}

fn is_union(vm: &mut Vm, v: &Value) -> bool {
    if let Value::Obj(o) = v {
        return o.class.name() == "UnionType" && { let _ = vm; true };
    }
    false
}

pub fn make_union(vm: &mut Vm, a: &Value, b: &Value) -> PyResult<Value> {
    let c = alias_class(vm, "UnionType");
    let mut args: Vec<Value> = vec![];
    for x in [a, b] {
        let parts = if is_union(vm, x) { alias_parts(x).1 } else { vec![if x.is_none() { Value::Class(vm.t.none.clone()) } else { x.clone() }] };
        for p in parts {
            let mut dup = false;
            for q in &args {
                if q.is(&p) {
                    dup = true;
                }
            }
            if !dup {
                args.push(p);
            }
        }
    }
    let args: Vec<Value> = args.into_iter().map(|x| if let Value::Class(c) = &x { if Rc::ptr_eq(c, &vm.t.none) { Value::None } else { x } } else { x }).collect();
    Ok(Value::Obj(Rc::new(NativeObj { class: c, kind: RefCell::new(ObjKind::Other(Value::tuple(vec![Value::None, Value::tuple(args)]))) })))
}

// -------------------------------------------------------------------------
// attributes of built-in values that are not in a type's dict

pub fn func_attr(vm: &mut Vm, f: &Rc<Function>, name: &str) -> Option<Value> {
    Some(match name {
        "__name__" => Value::Str(f.name.borrow().clone()),
        "__qualname__" => Value::Str(f.qualname.borrow().clone()),
        "__module__" => f.globals.borrow().get_s("__name__").unwrap_or(Value::None),
        "__doc__" => f.code.doc.clone(),
        "__defaults__" => {
            let d = f.defaults.borrow();
            if d.is_empty() {
                Value::None
            } else {
                Value::tuple(d.clone())
            }
        }
        "__kwdefaults__" => {
            let kd = f.kwdefaults.borrow();
            if kd.is_empty() {
                Value::None
            } else {
                let mut d = Dict::new();
                for (k, v) in kd.iter() {
                    d.set_s(k, v.clone());
                }
                Value::dict(d)
            }
        }
        "__globals__" => Value::Dict(f.globals.clone()),
        "__code__" => Value::Code(f.code.clone()),
        "__closure__" => {
            if f.closure.is_empty() {
                Value::None
            } else {
                Value::tuple(f.closure.iter().map(|c| Value::Cell(c.clone())).collect())
            }
        }
        "__dict__" => {
            let mut d = f.dict.borrow_mut();
            if d.is_none() {
                *d = Some(new_dict());
            }
            Value::Dict(d.as_ref().unwrap().clone())
        }
        "__annotations__" => {
            let a = f.annotations.borrow().clone();
            if a.is_undef() {
                let d = Value::dict(Dict::new());
                *f.annotations.borrow_mut() = d.clone();
                d
            } else {
                a
            }
        }
        "__type_params__" => Value::tuple(vec![]),
        _ => {
            let _ = vm;
            return None;
        }
    })
}

fn code_attr(c: &Rc<crate::code::Code>, name: &str) -> Option<Value> {
    Some(match name {
        "co_name" => Value::Str(c.name.clone()),
        "co_qualname" => Value::Str(c.qualname.clone()),
        "co_filename" => Value::str(&c.filename),
        "co_firstlineno" => Value::Int(c.firstline as i64),
        "co_argcount" => Value::Int(c.argcount as i64),
        "co_posonlyargcount" => Value::Int(c.posonly as i64),
        "co_kwonlyargcount" => Value::Int(c.kwonly as i64),
        "co_nlocals" => Value::Int(c.varnames.len() as i64),
        "co_varnames" => Value::tuple(c.varnames.iter().map(|n| Value::str(n)).collect()),
        "co_cellvars" => Value::tuple(c.cellvars.iter().map(|n| Value::str(n)).collect()),
        "co_freevars" => Value::tuple(c.freevars.iter().map(|n| Value::str(n)).collect()),
        "co_names" => Value::tuple(c.names.iter().map(|n| Value::Str(n.clone())).collect()),
        "co_consts" => Value::tuple(c.consts.iter().filter(|v| !matches!(v, Value::Obj(_))).cloned().collect()),
        "co_flags" => Value::Int(((c.varargs as i64) << 2) | ((c.varkw as i64) << 3) | ((c.generator as i64) << 5) | ((c.coroutine as i64) << 7)),
        _ => return None,
    })
}

pub fn special_attr(vm: &mut Vm, obj: &Value, name: &str) -> PyResult<Option<Value>> {
    Ok(match obj {
        Value::Int(_) | Value::Bool(_) | Value::Big(_) => match name {
            "real" | "numerator" => Some(match obj {
                Value::Bool(b) => Value::Int(*b as i64),
                o => o.clone(),
            }),
            "imag" => Some(Value::Int(0)),
            "denominator" => Some(Value::Int(1)),
            _ => None,
        },
        Value::Float(f) => match name {
            "real" => Some(Value::Float(*f)),
            "imag" => Some(Value::Float(0.0)),
            _ => None,
        },
        Value::Complex(c) => match name {
            "real" => Some(Value::Float(c.0)),
            "imag" => Some(Value::Float(c.1)),
            _ => None,
        },
        Value::Slice(s) => match name {
            "start" => Some(s.0.clone()),
            "stop" => Some(s.1.clone()),
            "step" => Some(s.2.clone()),
            _ => None,
        },
        Value::Range(r) => match name {
            "start" => Some(Value::Int(r.0)),
            "stop" => Some(Value::Int(r.1)),
            "step" => Some(Value::Int(r.2)),
            _ => None,
        },
        Value::Native(n) => match name {
            "__name__" | "__qualname__" => Some(Value::str(n.name.rsplit('.').next().unwrap_or(n.name))),
            "__module__" => Some(Value::str("builtins")),
            "__self__" => Some(Value::None),
            "__doc__" => Some(Value::None),
            "__text_signature__" => Some(Value::None),
            _ => None,
        },
        Value::Method(m) => match name {
            "__self__" => Some(m.0.clone()),
            "__func__" => Some(m.1.clone()),
            "__name__" | "__qualname__" | "__doc__" | "__module__" | "__wrapped__" | "__defaults__" | "__code__" | "__annotations__" | "__dict__" => match &m.1 {
                Value::Native(n) if name == "__name__" || name == "__qualname__" => Some(Value::str(n.name.rsplit('.').next().unwrap_or(n.name))),
                f => vm.getattr_opt(&f.clone(), &ps(name))?,
            },
            _ => match &m.1 {
                Value::Func(f) => f.dict.borrow().as_ref().and_then(|d| d.borrow().get_s(name)),
                _ => None,
            },
        },
        Value::Gen(g) => {
            let g = g.borrow();
            match name {
                "__name__" => Some(Value::Str(g.name.clone())),
                "__qualname__" => Some(Value::Str(g.qualname.clone())),
                "gi_running" | "cr_running" => Some(Value::Bool(g.running)),
                "gi_frame" | "cr_frame" => Some(if g.frame.is_some() { Value::Bool(true) } else { Value::None }),
                "gi_code" | "cr_code" => g.frame.as_ref().map(|f| Value::Code(f.code.clone())),
                "gi_yieldfrom" => Some(Value::None),
                _ => None,
            }
        }
        Value::Property(p) => match name {
            "fget" => Some(p.fget.clone()),
            "fset" => Some(p.fset.clone()),
            "fdel" => Some(p.fdel.clone()),
            "__doc__" => Some(if p.doc.is_none() {
                match &p.fget {
                    Value::Func(f) => f.code.doc.clone(),
                    _ => Value::None,
                }
            } else {
                p.doc.clone()
            }),
            "__isabstractmethod__" => {
                let f = p.fget.clone();
                match vm.getattr_opt(&f, &ps("__isabstractmethod__"))? {
                    Some(v) => Some(v),
                    None => Some(Value::Bool(false)),
                }
            }
            _ => None,
        },
        Value::StaticMethod(f) | Value::ClassMethod(f) => match name {
            "__func__" | "__wrapped__" => Some((**f).clone()),
            "__isabstractmethod__" | "__name__" | "__qualname__" | "__doc__" => {
                let f = (**f).clone();
                vm.getattr_opt(&f, &ps(name))?
            }
            _ => None,
        },
        Value::Cell(c) => match name {
            "cell_contents" => {
                let v = c.borrow().clone();
                if v.is_undef() {
                    return Err(vm.value_error("Cell is empty"));
                }
                Some(v)
            }
            _ => None,
        },
        Value::Code(c) => code_attr(c, name),
        Value::DictView(dv) => match name {
            "mapping" => Some(Value::Dict(dv.0.clone())),
            _ => None,
        },
        Value::Obj(o) => {
            if let ObjKind::Other(_) = &*o.kind.borrow() {
                let cn = o.class.name();
                if cn == "GenericAlias" || cn == "UnionType" {
                    let (origin, args) = alias_parts(obj);
                    return Ok(match name {
                        "__origin__" => Some(origin),
                        "__args__" => Some(Value::tuple(args)),
                        "__parameters__" => Some(Value::tuple(vec![])),
                        _ => None,
                    });
                }
            }
            crate::modules::obj_attr(vm, obj, o, name)?
        }
        _ => None,
    })
}

pub fn instance_special(vm: &mut Vm, obj: &Value, inst: &Rc<Instance>, name: &str) -> PyResult<Option<Value>> {
    let cls = inst.class.borrow().clone();
    if cls.builtin.get() == Builtin::BaseException {
        let args = vm.exc_args(obj);
        return Ok(match name {
            "args" => Some(Value::tuple(args)),
            "__traceback__" | "__cause__" | "__context__" => Some(Value::None),
            "__suppress_context__" => Some(Value::Bool(inst.dict.borrow().get_str(&vm.n.cause).map(|c| !c.is_none()).unwrap_or(false))),
            "value" if cls.is_subclass(&vm.exc_class("StopIteration")) => Some(args.into_iter().next().unwrap_or(Value::None)),
            "code" if cls.is_subclass(&vm.exc_class("SystemExit")) => Some(match args.len() {
                0 => Value::None,
                1 => args[0].clone(),
                _ => Value::tuple(args),
            }),
            "errno" | "strerror" | "filename" | "filename2" if cls.is_subclass(&vm.exc_class("OSError")) => {
                let i = match name {
                    "errno" => 0,
                    "strerror" => 1,
                    "filename" => 2,
                    _ => 3,
                };
                Some(if args.len() >= 2 { args.get(i).cloned().unwrap_or(Value::None) } else { Value::None })
            }
            "name" | "path" | "obj" if cls.is_subclass(&vm.exc_class("ImportError")) || cls.is_subclass(&vm.exc_class("AttributeError")) || cls.is_subclass(&vm.exc_class("NameError")) => Some(Value::None),
            "msg" if cls.is_subclass(&vm.exc_class("SyntaxError")) => Some(args.into_iter().next().unwrap_or(Value::None)),
            "lineno" | "offset" | "text" | "filename" | "end_lineno" | "end_offset" if cls.is_subclass(&vm.exc_class("SyntaxError")) => Some(Value::None),
            "exceptions" | "message" => None,
            _ => None,
        });
    }
    let b = inst.base.borrow().clone();
    if !b.is_undef() {
        return special_attr(vm, &b, name);
    }
    Ok(None)
}

pub fn class_special(vm: &mut Vm, c: &Rc<Class>, name: &str) -> PyResult<Option<Value>> {
    Ok(Some(match name {
        "__name__" => Value::Str(c.name.borrow().clone()),
        "__qualname__" => Value::Str(c.qualname.borrow().clone()),
        "__mro__" => Value::tuple(c.mro.borrow().iter().map(|k| Value::Class(k.clone())).collect()),
        "__bases__" => {
            let bs = c.bases.borrow();
            Value::tuple(bs.iter().map(|k| Value::Class(k.clone())).collect())
        }
        "__base__" => match c.bases.borrow().first() {
            Some(b) => Value::Class(b.clone()),
            None => Value::None,
        },
        "__dict__" => Value::Dict(c.dict.clone()),
        "__module__" => Value::str("builtins"),
        "__doc__" => Value::None,
        "__class__" => Value::Class(vm.class_of(&Value::Class(c.clone()))),
        "__annotations__" => {
            let d = Value::dict(Dict::new());
            if c.builtin.get() == Builtin::User || c.dict.borrow().get_s("__module__").is_some() {
                c.dict.borrow_mut().set_s("__annotations__", d.clone());
            }
            d
        }
        "__type_params__" => Value::tuple(vec![]),
        "__abstractmethods__" => return Ok(None),
        "__text_signature__" => Value::None,
        _ => return Ok(None),
    }))
}

// -------------------------------------------------------------------------
// exceptions

pub fn exc_str(vm: &mut Vm, v: &Value) -> PyResult<String> {
    let args = vm.exc_args(v);
    let cls = vm.class_of(v);
    if cls.is_subclass(&vm.exc_class("KeyError")) && args.len() == 1 {
        return vm.repr(&args[0]);
    }
    match args.len() {
        0 => Ok(String::new()),
        1 => vm.str_of(&args[0]),
        _ => vm.repr(&Value::tuple(args)),
    }
}

/// name and parent of every built-in exception class
pub const EXCEPTIONS: &[(&str, &str)] = &[
    ("BaseException", ""),
    ("SystemExit", "BaseException"),
    ("KeyboardInterrupt", "BaseException"),
    ("GeneratorExit", "BaseException"),
    ("Exception", "BaseException"),
    ("StopIteration", "Exception"),
    ("StopAsyncIteration", "Exception"),
    ("ArithmeticError", "Exception"),
    ("FloatingPointError", "ArithmeticError"),
    ("OverflowError", "ArithmeticError"),
    ("ZeroDivisionError", "ArithmeticError"),
    ("AssertionError", "Exception"),
    ("AttributeError", "Exception"),
    ("BufferError", "Exception"),
    ("EOFError", "Exception"),
    ("ImportError", "Exception"),
    ("ModuleNotFoundError", "ImportError"),
    ("LookupError", "Exception"),
    ("IndexError", "LookupError"),
    ("KeyError", "LookupError"),
    ("MemoryError", "Exception"),
    ("NameError", "Exception"),
    ("UnboundLocalError", "NameError"),
    ("OSError", "Exception"),
    ("BlockingIOError", "OSError"),
    ("ChildProcessError", "OSError"),
    ("ConnectionError", "OSError"),
    ("BrokenPipeError", "ConnectionError"),
    ("ConnectionAbortedError", "ConnectionError"),
    ("ConnectionRefusedError", "ConnectionError"),
    ("ConnectionResetError", "ConnectionError"),
    ("FileExistsError", "OSError"),
    ("FileNotFoundError", "OSError"),
    ("InterruptedError", "OSError"),
    ("IsADirectoryError", "OSError"),
    ("NotADirectoryError", "OSError"),
    ("PermissionError", "OSError"),
    ("ProcessLookupError", "OSError"),
    ("TimeoutError", "OSError"),
    ("ReferenceError", "Exception"),
    ("RuntimeError", "Exception"),
    ("NotImplementedError", "RuntimeError"),
    ("RecursionError", "RuntimeError"),
    ("PythonFinalizationError", "RuntimeError"),
    ("SyntaxError", "Exception"),
    ("IndentationError", "SyntaxError"),
    ("TabError", "IndentationError"),
    ("SystemError", "Exception"),
    ("TypeError", "Exception"),
    ("ValueError", "Exception"),
    ("UnicodeError", "ValueError"),
    ("UnicodeDecodeError", "UnicodeError"),
    ("UnicodeEncodeError", "UnicodeError"),
    ("UnicodeTranslateError", "UnicodeError"),
    ("Warning", "Exception"),
    ("DeprecationWarning", "Warning"),
    ("PendingDeprecationWarning", "Warning"),
    ("RuntimeWarning", "Warning"),
    ("SyntaxWarning", "Warning"),
    ("UserWarning", "Warning"),
    ("FutureWarning", "Warning"),
    ("ImportWarning", "Warning"),
    ("UnicodeWarning", "Warning"),
    ("BytesWarning", "Warning"),
    ("ResourceWarning", "Warning"),
    ("EncodingWarning", "Warning"),
];

fn exc_new(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let cls = match &a[0] {
        Value::Class(c) => c.clone(),
        _ => return Err(vm.type_error("BaseException.__new__(X): X is not a type object")),
    };
    let v = vm.new_instance(&cls);
    if let Value::Instance(i) = &v {
        *i.base.borrow_mut() = Value::tuple(a[1..].to_vec());
    }
    Ok(v)
}

fn exc_init(_vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    if let Value::Instance(i) = &a[0] {
        *i.base.borrow_mut() = Value::tuple(a[1..].to_vec());
    }
    Ok(Value::None)
}

fn exc_str_native(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    Ok(Value::string(exc_str(vm, &a[0])?))
}

fn exc_repr_native(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let cls = vm.class_of(&a[0]);
    let args = vm.exc_args(&a[0]);
    let inner = if args.len() == 1 { vm.repr(&args[0])? } else { let r = vm.repr(&Value::tuple(args.clone()))?; r[1..r.len() - 1].trim_end_matches(',').to_string() };
    Ok(Value::string(format!("{}({})", cls.name(), inner)))
}

fn exc_with_traceback(_vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    Ok(a[0].clone())
}

fn exc_add_note(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    margs(vm, "add_note", a, 1, 1)?;
    if !matches!(a[1], Value::Str(_)) {
        let tn = vm.type_name(&a[1]);
        return Err(vm.type_error(format!("note must be a str, not '{}'", tn)));
    }
    if let Value::Instance(i) = &a[0] {
        let cur = i.dict.borrow().get_str(&vm.n.notes).cloned();
        match cur {
            Some(Value::List(l)) => l.borrow_mut().push(a[1].clone()),
            _ => i.dict.borrow_mut().set_str(vm.n.notes.clone(), Value::list(vec![a[1].clone()])),
        }
    }
    Ok(Value::None)
}

fn exc_reduce(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let cls = vm.class_of(&a[0]);
    let args = vm.exc_args(&a[0]);
    Ok(Value::tuple(vec![Value::Class(cls), Value::tuple(args)]))
}

// -------------------------------------------------------------------------
// the types

/// registers a native method on a class; `name` is "type.method"
pub fn def(c: &Rc<Class>, name: &'static str, f: NativeFnPtr) {
    let short = name.rsplit('.').next().unwrap();
    c.dict.borrow_mut().set_s(short, Value::Native(Rc::new(NativeFn { name, f, data: Value::None })));
}

pub fn def_static(c: &Rc<Class>, name: &'static str, f: NativeFnPtr) {
    let short = name.rsplit('.').next().unwrap();
    c.dict.borrow_mut().set_s(short, Value::StaticMethod(Rc::new(Value::Native(Rc::new(NativeFn { name, f, data: Value::None })))));
}

pub fn def_class(c: &Rc<Class>, name: &'static str, f: NativeFnPtr) {
    let short = name.rsplit('.').next().unwrap();
    c.dict.borrow_mut().set_s(short, Value::ClassMethod(Rc::new(Value::Native(Rc::new(NativeFn { name, f, data: Value::None })))));
}

pub fn native(name: &'static str, f: NativeFnPtr) -> Value {
    Value::Native(Rc::new(NativeFn { name, f, data: Value::None }))
}

fn mk(name: &str, base: Option<&Rc<Class>>, b: Builtin) -> Rc<Class> {
    let bases = base.map(|x| vec![x.clone()]).unwrap_or_default();
    let c = Class::new(name, bases, b);
    let mut mro = vec![c.clone()];
    if let Some(x) = base {
        mro.extend(x.mro.borrow().iter().cloned());
    }
    *c.mro.borrow_mut() = mro;
    c
}

pub fn make_types() -> Types {
    let object = mk("object", None, Builtin::Object);
    let o = Some(&object);
    let int = mk("int", o, Builtin::Int);
    let base_exception = mk("BaseException", o, Builtin::BaseException);
    let obj = crate::vm::obj_class();
    if obj.mro.borrow().is_empty() {
        *obj.bases.borrow_mut() = vec![object.clone()];
        *obj.mro.borrow_mut() = vec![obj.clone(), object.clone()];
    }
    Types {
        type_: mk("type", o, Builtin::Type),
        bool_: mk("bool", Some(&int), Builtin::Bool),
        int,
        float: mk("float", o, Builtin::Float),
        complex: mk("complex", o, Builtin::Complex),
        str_: mk("str", o, Builtin::Str),
        bytes: mk("bytes", o, Builtin::Bytes),
        bytearray: mk("bytearray", o, Builtin::ByteArray),
        list: mk("list", o, Builtin::List),
        tuple: mk("tuple", o, Builtin::Tuple),
        dict: mk("dict", o, Builtin::Dict),
        set: mk("set", o, Builtin::Set),
        frozenset: mk("frozenset", o, Builtin::FrozenSet),
        range: mk("range", o, Builtin::Range),
        slice: mk("slice", o, Builtin::Slice),
        none: mk("NoneType", o, Builtin::NoneType),
        function: mk("function", o, Builtin::Function),
        builtin_function: mk("builtin_function_or_method", o, Builtin::BuiltinFunction),
        method: mk("method", o, Builtin::Method),
        module: mk("module", o, Builtin::Module),
        iterator: mk("iterator", o, Builtin::Iterator),
        generator: mk("generator", o, Builtin::Generator),
        coroutine: mk("coroutine", o, Builtin::Generator),
        dict_keys: mk("dict_keys", o, Builtin::DictKeys),
        dict_values: mk("dict_values", o, Builtin::DictValues),
        dict_items: mk("dict_items", o, Builtin::DictItems),
        property: mk("property", o, Builtin::Property),
        staticmethod: mk("staticmethod", o, Builtin::StaticMethod),
        classmethod: mk("classmethod", o, Builtin::ClassMethod),
        super_: mk("super", o, Builtin::Super),
        notimpl: mk("NotImplementedType", o, Builtin::NotImplementedType),
        ellipsis: mk("ellipsis", o, Builtin::Ellipsis),
        cell: mk("cell", o, Builtin::Cell),
        code: mk("code", o, Builtin::Code),
        obj,
        file: mk("TextIOWrapper", o, Builtin::Obj),
        base_exception,
        object,
    }
}

fn exc_class_new(vm: &mut Vm, name: &'static str, parent: &str) -> Rc<Class> {
    let p = if parent.is_empty() { vm.t.object.clone() } else { vm.exc[parent].clone() };
    let c = if name == "BaseException" { vm.t.base_exception.clone() } else { mk(name, Some(&p), Builtin::BaseException) };
    vm.exc.insert(name, c.clone());
    c
}

pub fn install(vm: &mut Vm) {
    for (name, parent) in EXCEPTIONS {
        let c = exc_class_new(vm, name, parent);
        vm.builtins.borrow_mut().set_s(name, Value::Class(c));
    }
    vm.builtins.borrow_mut().set_s("EnvironmentError", Value::Class(vm.exc["OSError"].clone()));
    vm.builtins.borrow_mut().set_s("IOError", Value::Class(vm.exc["OSError"].clone()));
    let be = vm.t.base_exception.clone();
    def_static(&be, "BaseException.__new__", exc_new);
    def(&be, "BaseException.__init__", exc_init);
    def(&be, "BaseException.__str__", exc_str_native);
    def(&be, "BaseException.__repr__", exc_repr_native);
    def(&be, "BaseException.with_traceback", exc_with_traceback);
    def(&be, "BaseException.add_note", exc_add_note);
    def(&be, "BaseException.__reduce__", exc_reduce);

    crate::methods::install_types(vm);
    install_functions(vm);
    let b = vm.builtins.clone();
    let mut bd = b.borrow_mut();
    let t = &vm.t;
    for (n, c) in [
        ("object", &t.object),
        ("type", &t.type_),
        ("int", &t.int),
        ("bool", &t.bool_),
        ("float", &t.float),
        ("complex", &t.complex),
        ("str", &t.str_),
        ("bytes", &t.bytes),
        ("bytearray", &t.bytearray),
        ("list", &t.list),
        ("tuple", &t.tuple),
        ("dict", &t.dict),
        ("set", &t.set),
        ("frozenset", &t.frozenset),
        ("range", &t.range),
        ("slice", &t.slice),
        ("property", &t.property),
        ("staticmethod", &t.staticmethod),
        ("classmethod", &t.classmethod),
        ("super", &t.super_),
    ] {
        bd.set_s(n, Value::Class(c.clone()));
    }
    bd.set_s("__name__", Value::str("builtins"));
    bd.set_s("__doc__", Value::None);
    bd.set_s("None", Value::None);
    bd.set_s("True", Value::Bool(true));
    bd.set_s("False", Value::Bool(false));
    bd.set_s("NotImplemented", Value::NotImplemented);
    bd.set_s("Ellipsis", Value::Ellipsis);
    bd.set_s("__debug__", Value::Bool(true));
}

// -------------------------------------------------------------------------
// builtin functions

fn install_functions(vm: &mut Vm) {
    let fns: &[(&'static str, NativeFnPtr)] = &[
        ("print", bi_print),
        ("len", bi_len),
        ("repr", bi_repr),
        ("ascii", bi_ascii),
        ("abs", bi_abs),
        ("all", bi_all),
        ("any", bi_any),
        ("bin", bi_bin),
        ("oct", bi_oct),
        ("hex", bi_hex),
        ("callable", bi_callable),
        ("chr", bi_chr),
        ("ord", bi_ord),
        ("delattr", bi_delattr),
        ("getattr", bi_getattr),
        ("setattr", bi_setattr),
        ("hasattr", bi_hasattr),
        ("dir", bi_dir),
        ("divmod", bi_divmod),
        ("enumerate", bi_enumerate),
        ("zip", bi_zip),
        ("map", bi_map),
        ("filter", bi_filter),
        ("reversed", bi_reversed),
        ("iter", bi_iter),
        ("next", bi_next),
        ("format", bi_format),
        ("hash", bi_hash),
        ("id", bi_id),
        ("input", bi_input),
        ("isinstance", bi_isinstance),
        ("issubclass", bi_issubclass),
        ("max", bi_max),
        ("min", bi_min),
        ("sum", bi_sum),
        ("sorted", bi_sorted),
        ("pow", bi_pow),
        ("round", bi_round),
        ("vars", bi_vars),
        ("locals", bi_locals),
        ("globals", bi_globals),
        ("eval", bi_eval),
        ("exec", bi_exec),
        ("compile", bi_compile),
        ("exit", bi_exit),
        ("quit", bi_exit),
        ("open", bi_open),
        ("__import__", bi_import),
        ("__build_class__", bi_build_class),
        ("aiter", bi_aiter),
        ("anext", bi_anext),
        ("breakpoint", bi_noop),
        ("help", bi_noop),
    ];
    let mut b = vm.builtins.borrow_mut();
    for (n, f) in fns {
        b.set_s(n, native(n, *f));
    }
}

fn bi_noop(_vm: &mut Vm, _a: &[Value], _k: Kw) -> PyResult<Value> {
    Ok(Value::None)
}

pub fn write_stdout(vm: &mut Vm, s: &str) -> PyResult<()> {
    // sys.stdout may have been replaced (contextlib.redirect_stdout)
    let sys = vm.modules.borrow().get_s("sys");
    if let Some(Value::Module(m)) = sys {
        let out = m.dict.borrow().get_s("stdout");
        if let Some(o) = out {
            if !crate::modules::is_std_stream(&o) {
                let w = vm.getattr(&o, &ps("write"))?;
                vm.call_vec(&w, vec![Value::str(s)], &[])?;
                return Ok(());
            }
        }
    }
    vm.out.push_str(s);
    Ok(())
}

fn bi_print(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let mut sep = " ".to_string();
    let mut end = "\n".to_string();
    let mut file = Value::None;
    for (key, v) in k {
        match &*key.s {
            "sep" => {
                if !v.is_none() {
                    sep = match v {
                        Value::Str(s) => s.s.clone(),
                        _ => {
                            let tn = vm.type_name(v);
                            return Err(vm.type_error(format!("sep must be None or a string, not {}", tn)));
                        }
                    }
                }
            }
            "end" => {
                if !v.is_none() {
                    end = match v {
                        Value::Str(s) => s.s.clone(),
                        _ => {
                            let tn = vm.type_name(v);
                            return Err(vm.type_error(format!("end must be None or a string, not {}", tn)));
                        }
                    }
                }
            }
            "file" => file = v.clone(),
            "flush" => {}
            other => return Err(vm.type_error(format!("'{}' is an invalid keyword argument for print()", other))),
        }
    }
    let mut s = String::new();
    for (i, x) in a.iter().enumerate() {
        if i > 0 {
            s.push_str(&sep);
        }
        match x {
            Value::Str(t) => s.push_str(&t.s),
            Value::Int(n) => {
                use std::fmt::Write;
                let _ = write!(s, "{}", n);
            }
            _ => s.push_str(&vm.str_of(x)?),
        }
    }
    s.push_str(&end);
    if file.is_none() || crate::modules::is_std_stream(&file) {
        if crate::modules::is_stderr(&file) {
            vm.err_out.push_str(&s);
            return Ok(Value::None);
        }
        write_stdout(vm, &s)?;
    } else {
        let w = vm.getattr(&file, &ps("write"))?;
        vm.call_vec(&w, vec![Value::string(s)], &[])?;
    }
    Ok(Value::None)
}

fn bi_len(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "len", a, 1, 1)?;
    Ok(Value::Int(vm.len(&a[0])? as i64))
}

fn bi_repr(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "repr", a, 1, 1)?;
    Ok(Value::string(vm.repr(&a[0])?))
}

fn bi_ascii(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "ascii", a, 1, 1)?;
    let r = vm.repr(&a[0])?;
    Ok(Value::string(crate::format::ascii_escape(&r)))
}

fn bi_abs(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "abs", a, 1, 1)?;
    Ok(match &a[0] {
        Value::Int(i) => match i.checked_abs() {
            Some(v) => Value::Int(v),
            None => Value::from_big(BigInt::from_i64(*i).abs()),
        },
        Value::Bool(b) => Value::Int(*b as i64),
        Value::Float(f) => Value::Float(f.abs()),
        Value::Big(b) => Value::from_big(b.abs()),
        Value::Complex(c) => Value::Float(c.0.hypot(c.1)),
        v => {
            if let Some(f) = vm.lookup_special(v, &vm.n.abs.clone()) {
                return vm.call_vec(&f, vec![], &[]);
            }
            let b = vm.base_of(v);
            if !matches!(b, Value::Instance(_)) {
                return bi_abs(vm, &[b], &[]);
            }
            let tn = vm.type_name(v);
            return Err(vm.type_error(format!("bad operand type for abs(): '{}'", tn)));
        }
    })
}

fn bi_all(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "all", a, 1, 1)?;
    let it = vm.iter(&a[0])?;
    while let Some(x) = vm.next(&it)? {
        if !vm.truthy(&x)? {
            return Ok(Value::Bool(false));
        }
    }
    Ok(Value::Bool(true))
}

fn bi_any(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "any", a, 1, 1)?;
    let it = vm.iter(&a[0])?;
    while let Some(x) = vm.next(&it)? {
        if vm.truthy(&x)? {
            return Ok(Value::Bool(true));
        }
    }
    Ok(Value::Bool(false))
}

fn int_radix(vm: &mut Vm, v: &Value, radix: u32, prefix: &str) -> PyResult<Value> {
    let i = match v {
        Value::Int(_) | Value::Bool(_) | Value::Big(_) => crate::ops::big_of(v).unwrap(),
        _ => {
            let b = vm.base_of(v);
            match crate::ops::big_of(&b) {
                Some(x) => x,
                None => {
                    let n = vm.to_index(v)?;
                    BigInt::from_i64(n)
                }
            }
        }
    };
    let digits = i.abs().to_radix(radix);
    Ok(Value::string(format!("{}{}{}", if i.neg { "-" } else { "" }, prefix, digits)))
}

fn bi_bin(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "bin", a, 1, 1)?;
    int_radix(vm, &a[0], 2, "0b")
}

fn bi_oct(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "oct", a, 1, 1)?;
    int_radix(vm, &a[0], 8, "0o")
}

fn bi_hex(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "hex", a, 1, 1)?;
    int_radix(vm, &a[0], 16, "0x")
}

pub fn is_callable(vm: &mut Vm, v: &Value) -> bool {
    match v {
        Value::Func(_) | Value::Native(_) | Value::Method(_) | Value::Class(_) => true,
        Value::Instance(_) | Value::Obj(_) => vm.lookup_special(v, &vm.n.call.clone()).is_some(),
        _ => false,
    }
}

fn bi_callable(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "callable", a, 1, 1)?;
    Ok(Value::Bool(is_callable(vm, &a[0])))
}

fn bi_chr(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "chr", a, 1, 1)?;
    let i = match &a[0] {
        Value::Float(_) => return Err(vm.type_error("'float' object cannot be interpreted as an integer")),
        v => vm.to_index(v)?,
    };
    match u32::try_from(i).ok().and_then(char::from_u32) {
        Some(c) => Ok(char_value(c)),
        None => {
            if (0xd800..0xe000).contains(&i) {
                // a lone surrogate cannot be a Rust char; keep the replacement
                return Ok(Value::str("\u{fffd}"));
            }
            Err(vm.value_error("chr() arg not in range(0x110000)"))
        }
    }
}

fn bi_ord(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "ord", a, 1, 1)?;
    match &vm.base_of(&a[0]) {
        Value::Str(s) => {
            let mut cs = s.s.chars();
            match (cs.next(), cs.next()) {
                (Some(c), None) => Ok(Value::Int(c as i64)),
                _ => Err(vm.type_error(format!("ord() expected a character, but string of length {} found", s.char_len()))),
            }
        }
        Value::Bytes(b) if b.len() == 1 => Ok(Value::Int(b[0] as i64)),
        Value::Bytes(b) => Err(vm.type_error(format!("ord() expected a character, but string of length {} found", b.len()))),
        v => {
            let tn = vm.type_name(v);
            Err(vm.type_error(format!("ord() expected string of length 1, but {} found", tn)))
        }
    }
}

fn attr_name(vm: &mut Vm, v: &Value) -> PyResult<Rc<PyStr>> {
    match v {
        Value::Str(s) => Ok(s.clone()),
        _ => match vm.base_of(v) {
            Value::Str(s) => Ok(s),
            _ => {
                let tn = vm.type_name(v);
                Err(vm.type_error(format!("attribute name must be string, not '{}'", tn)))
            }
        },
    }
}

fn bi_getattr(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "getattr", a, 2, 3)?;
    let n = attr_name(vm, &a[1])?;
    if a.len() == 3 {
        match vm.getattr(&a[0], &n) {
            Ok(v) => Ok(v),
            Err(e) if vm.exc_is(&e, "AttributeError") => Ok(a[2].clone()),
            Err(e) => Err(e),
        }
    } else {
        vm.getattr(&a[0], &n)
    }
}

fn bi_setattr(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "setattr", a, 3, 3)?;
    let n = attr_name(vm, &a[1])?;
    vm.setattr(&a[0], &n, a[2].clone())?;
    Ok(Value::None)
}

fn bi_delattr(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "delattr", a, 2, 2)?;
    let n = attr_name(vm, &a[1])?;
    vm.delattr(&a[0], &n)?;
    Ok(Value::None)
}

fn bi_hasattr(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "hasattr", a, 2, 2)?;
    let n = attr_name(vm, &a[1])?;
    match vm.getattr(&a[0], &n) {
        Ok(_) => Ok(Value::Bool(true)),
        Err(e) if vm.exc_is(&e, "AttributeError") => Ok(Value::Bool(false)),
        Err(e) => Err(e),
    }
}

pub fn dir_of(vm: &mut Vm, v: &Value) -> PyResult<Vec<String>> {
    let mut names: Vec<String> = vec![];
    let mut add = |names: &mut Vec<String>, d: &Dict| {
        for k in d.keys() {
            if let Value::Str(s) = k {
                names.push(s.s.clone());
            }
        }
    };
    match v {
        Value::Module(m) => add(&mut names, &m.dict.borrow()),
        Value::Class(c) => {
            for k in c.mro.borrow().iter() {
                add(&mut names, &k.dict.borrow());
            }
        }
        Value::Instance(i) => {
            add(&mut names, &i.dict.borrow());
            let c = i.class.borrow().clone();
            for k in c.mro.borrow().iter() {
                add(&mut names, &k.dict.borrow());
            }
        }
        _ => {
            let c = vm.class_of(v);
            for k in c.mro.borrow().iter() {
                add(&mut names, &k.dict.borrow());
            }
        }
    }
    names.sort();
    names.dedup();
    Ok(names)
}

fn bi_dir(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "dir", a, 0, 1)?;
    if a.is_empty() {
        let mut names: Vec<String> = match vm.current_frame() {
            Some(f) => {
                if let Some(ns) = &f.ns {
                    ns.borrow().keys().iter().filter_map(|k| k.as_str().map(|s| s.to_string())).collect()
                } else {
                    f.code.varnames.iter().enumerate().filter(|(i, _)| !f.stack[*i].is_undef()).map(|(_, n)| n.to_string()).collect()
                }
            }
            None => vec![],
        };
        names.sort();
        return Ok(Value::list(names.into_iter().map(Value::string).collect()));
    }
    if let Value::Instance(_) = &a[0] {
        let cls = vm.class_of(&a[0]);
        if let Some(f @ Value::Func(_)) = cls.lookup(&ps("__dir__")) {
            let f = vm.bind(f, &a[0], &cls);
            let r = vm.call_vec(&f, vec![], &[])?;
            let mut items = vm.to_vec(&r)?;
            sort_values(vm, &mut items, &Value::None, false)?;
            return Ok(Value::list(items));
        }
    }
    let names = dir_of(vm, &a[0])?;
    Ok(Value::list(names.into_iter().map(Value::string).collect()))
}

fn bi_divmod(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "divmod", a, 2, 2)?;
    if let (Value::Instance(_), _) | (_, Value::Instance(_)) = (&a[0], &a[1]) {
        if let Some(f) = vm.lookup_special(&a[0], &ps("__divmod__")) {
            let r = vm.call_vec(&f, vec![a[1].clone()], &[])?;
            if !matches!(r, Value::NotImplemented) {
                return Ok(r);
            }
        }
        if let Some(f) = vm.lookup_special(&a[1], &ps("__rdivmod__")) {
            let r = vm.call_vec(&f, vec![a[0].clone()], &[])?;
            if !matches!(r, Value::NotImplemented) {
                return Ok(r);
            }
        }
    }
    let q = vm.binop(BinOp::FloorDiv, &a[0], &a[1]).map_err(|e| {
        if vm.exc_is(&e, "TypeError") {
            let (x, y) = (vm.type_name(&a[0]), vm.type_name(&a[1]));
            vm.type_error(format!("unsupported operand type(s) for divmod(): '{}' and '{}'", x, y))
        } else {
            e
        }
    })?;
    let r = vm.binop(BinOp::Mod, &a[0], &a[1])?;
    Ok(Value::tuple(vec![q, r]))
}

fn bi_enumerate(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let it = match arg_or_kw(a, 0, k, "iterable") {
        Some(v) => v,
        None => return Err(vm.type_error("enumerate() missing required argument 'iterable'")),
    };
    let start = arg_or_kw(a, 1, k, "start").unwrap_or(Value::Int(0));
    if !matches!(start, Value::Int(_) | Value::Big(_) | Value::Bool(_)) {
        let tn = vm.type_name(&start);
        return Err(vm.type_error(format!("'{}' object cannot be interpreted as an integer", tn)));
    }
    let inner = vm.iter(&it)?;
    Ok(mk_iter(Iter::Enumerate(inner, start)))
}

fn bi_zip(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let strict = match kwarg(k, "strict") {
        Some(v) => vm.truthy(&v)?,
        None => false,
    };
    let mut its = Vec::with_capacity(a.len());
    for (i, x) in a.iter().enumerate() {
        match vm.iter(x) {
            Ok(it) => its.push(it),
            Err(e) if vm.exc_is(&e, "TypeError") => {
                let tn = vm.type_name(x);
                return Err(vm.type_error(format!("'{}' object is not iterable", tn)).with_note(i));
            }
            Err(e) => return Err(e),
        }
    }
    Ok(mk_iter(Iter::Zip(its, strict)))
}

impl Exc {
    fn with_note(self, _i: usize) -> Exc {
        self
    }
}

fn bi_map(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    if a.len() < 2 {
        return Err(vm.type_error("map() must have at least two arguments."));
    }
    let mut its = vec![];
    for x in &a[1..] {
        its.push(vm.iter(x)?);
    }
    Ok(mk_iter(Iter::Map(a[0].clone(), its)))
}

fn bi_filter(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "filter", a, 2, 2)?;
    let it = vm.iter(&a[1])?;
    Ok(mk_iter(Iter::Filter(a[0].clone(), it, false)))
}

fn bi_reversed(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "reversed", a, 1, 1)?;
    let v = &a[0];
    match v {
        Value::List(l) => {
            let n = l.borrow().len();
            Ok(mk_iter(Iter::ListRev(l.clone(), n)))
        }
        Value::Tuple(t) => Ok(mk_iter(Iter::Vec(t.iter().rev().cloned().collect::<Vec<_>>().into_iter()))),
        Value::Str(s) => Ok(mk_iter(Iter::Vec(s.s.chars().rev().map(char_value).collect::<Vec<_>>().into_iter()))),
        Value::Range(r) => {
            let n = range_len(r) as i64;
            if n == 0 {
                return Ok(mk_iter(Iter::Empty));
            }
            let last = r.0 + (n - 1) * r.2;
            Ok(mk_iter(Iter::Range(last, r.0 - r.2, -r.2)))
        }
        Value::Bytes(b) => Ok(mk_iter(Iter::Vec(b.iter().rev().map(|&x| Value::Int(x as i64)).collect::<Vec<_>>().into_iter()))),
        Value::Dict(d) => {
            let keys = d.borrow().keys();
            Ok(mk_iter(Iter::Vec(keys.into_iter().rev().collect::<Vec<_>>().into_iter())))
        }
        Value::DictView(dv) => {
            let items: Vec<Value> = match dv.1 {
                0 => dv.0.borrow().keys(),
                1 => dv.0.borrow().values(),
                _ => dv.0.borrow().items().into_iter().map(|(k, v)| Value::tuple(vec![k, v])).collect(),
            };
            Ok(mk_iter(Iter::Vec(items.into_iter().rev().collect::<Vec<_>>().into_iter())))
        }
        _ => {
            if let Some(f) = vm.lookup_special(v, &vm.n.reversed.clone()) {
                let is_native = matches!(&f, Value::Method(m) if matches!(m.1, Value::Native(_)));
                if !is_native {
                    return vm.call_vec(&f, vec![], &[]);
                }
            }
            let b = vm.base_of(v);
            if !matches!(b, Value::Instance(_) | Value::Obj(_)) {
                return bi_reversed(vm, &[b], &[]);
            }
            if let Value::Obj(o) = v {
                if let ObjKind::Deque(d, _) = &*o.kind.borrow() {
                    return Ok(mk_iter(Iter::Vec(d.iter().rev().cloned().collect::<Vec<_>>().into_iter())));
                }
            }
            if vm.lookup_special(v, &vm.n.len.clone()).is_some() && vm.lookup_special(v, &vm.n.getitem.clone()).is_some() {
                let n = vm.len(v)?;
                let mut out = Vec::with_capacity(n);
                for i in (0..n).rev() {
                    out.push(vm.getitem(v, &Value::Int(i as i64))?);
                }
                return Ok(mk_iter(Iter::Vec(out.into_iter())));
            }
            let tn = vm.type_name(v);
            Err(vm.type_error(format!("'{}' object is not reversible", tn)))
        }
    }
}

fn bi_iter(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "iter", a, 1, 2)?;
    if a.len() == 2 {
        return Ok(mk_iter(Iter::Callable(a[0].clone(), a[1].clone())));
    }
    vm.iter(&a[0])
}

fn bi_next(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "next", a, 1, 2)?;
    match &a[0] {
        Value::Iter(_) | Value::Gen(_) => {}
        v => {
            if vm.lookup_special(v, &vm.n.next.clone()).is_none() {
                let tn = vm.type_name(v);
                return Err(vm.type_error(format!("'{}' object is not an iterator", tn)));
            }
        }
    }
    match vm.next(&a[0])? {
        Some(v) => Ok(v),
        None => {
            if a.len() == 2 {
                return Ok(a[1].clone());
            }
            let rv = vm.last_return.take().unwrap_or(Value::None);
            Err(vm.stop_iteration(rv))
        }
    }
}

fn bi_format(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "format", a, 1, 2)?;
    let spec = match a.get(1) {
        Some(Value::Str(s)) => s.s.clone(),
        Some(v) => {
            let tn = vm.type_name(v);
            return Err(vm.type_error(format!("format() argument 2 must be str, not {}", tn)));
        }
        None => String::new(),
    };
    Ok(Value::string(vm.format_value(&a[0], &spec)?))
}

fn bi_hash(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "hash", a, 1, 1)?;
    let h = vm.hash(&a[0])?;
    Ok(Value::Int(match &a[0] {
        Value::Int(_) | Value::Bool(_) | Value::Big(_) | Value::Float(_) => h as i64,
        Value::Instance(_) => h as i64,
        _ => (h >> 1) as i64,
    }))
}

fn bi_id(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "id", a, 1, 1)?;
    Ok(Value::Int(ptr_id(&a[0]) as i64 & 0x7fff_ffff_ffff))
}

fn bi_input(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    if let Some(p) = a.first() {
        let s = vm.str_of(p)?;
        write_stdout(vm, &s)?;
    }
    Err(vm.err("EOFError", "EOF when reading a line"))
}

/// isinstance(v, spec) with tuples, unions and __instancecheck__
pub fn isinstance_of(vm: &mut Vm, v: &Value, spec: &Value) -> PyResult<bool> {
    match spec {
        Value::Class(c) => {
            if vm.isinstance(v, c) {
                return Ok(true);
            }
            let meta = c.meta.borrow().clone();
            if let Some(m) = meta {
                if !Rc::ptr_eq(&m, &vm.t.type_) {
                    if let Some(f @ Value::Func(_)) = m.lookup(&vm.n.instancecheck) {
                        let r = vm.call_vec(&f, vec![spec.clone(), v.clone()], &[])?;
                        return vm.truthy(&r);
                    }
                }
            }
            // a class with a __class__ property (mock objects)
            Ok(false)
        }
        Value::Tuple(t) => {
            for x in t.iter() {
                if isinstance_of(vm, v, x)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        Value::Obj(_) if is_union(vm, spec) => {
            let (_, args) = alias_parts(spec);
            for x in &args {
                let x = if x.is_none() { Value::Class(vm.t.none.clone()) } else { x.clone() };
                if isinstance_of(vm, v, &x)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        Value::Obj(_) => {
            let (origin, _) = alias_parts(spec);
            if !origin.is_none() {
                return Err(vm.type_error("isinstance() argument 2 cannot be a parameterized generic"));
            }
            Err(vm.type_error("isinstance() arg 2 must be a type, a tuple of types, or a union"))
        }
        Value::Instance(_) => {
            if let Some(f) = vm.lookup_special(spec, &vm.n.instancecheck.clone()) {
                let r = vm.call_vec(&f, vec![v.clone()], &[])?;
                return vm.truthy(&r);
            }
            Err(vm.type_error("isinstance() arg 2 must be a type, a tuple of types, or a union"))
        }
        _ => Err(vm.type_error("isinstance() arg 2 must be a type, a tuple of types, or a union")),
    }
}

fn bi_isinstance(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "isinstance", a, 2, 2)?;
    Ok(Value::Bool(isinstance_of(vm, &a[0], &a[1])?))
}

pub fn issubclass_of(vm: &mut Vm, c: &Value, spec: &Value) -> PyResult<bool> {
    let cls = match c {
        Value::Class(c) => c.clone(),
        _ => return Err(vm.type_error("issubclass() arg 1 must be a class")),
    };
    match spec {
        Value::Class(s) => {
            if cls.is_subclass(s) {
                return Ok(true);
            }
            let meta = s.meta.borrow().clone();
            if let Some(m) = meta {
                if !Rc::ptr_eq(&m, &vm.t.type_) {
                    if let Some(f @ Value::Func(_)) = m.lookup(&vm.n.subclasscheck) {
                        let r = vm.call_vec(&f, vec![spec.clone(), c.clone()], &[])?;
                        return vm.truthy(&r);
                    }
                }
            }
            Ok(false)
        }
        Value::Tuple(t) => {
            for x in t.iter() {
                if issubclass_of(vm, c, x)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        Value::Obj(_) if is_union(vm, spec) => {
            let (_, args) = alias_parts(spec);
            for x in &args {
                let x = if x.is_none() { Value::Class(vm.t.none.clone()) } else { x.clone() };
                if issubclass_of(vm, c, &x)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        _ => Err(vm.type_error("issubclass() arg 2 must be a class, a tuple of classes, or a union")),
    }
}

fn bi_issubclass(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "issubclass", a, 2, 2)?;
    Ok(Value::Bool(issubclass_of(vm, &a[0], &a[1])?))
}

fn min_max(vm: &mut Vm, a: &[Value], k: Kw, is_max: bool) -> PyResult<Value> {
    let name = if is_max { "max" } else { "min" };
    let key = kwarg(k, "key").unwrap_or(Value::None);
    let default = kwarg(k, "default");
    for (kk, _) in k {
        if &*kk.s != "key" && &*kk.s != "default" {
            return Err(vm.type_error(format!("'{}' is an invalid keyword argument for {}()", kk.s, name)));
        }
    }
    let items = if a.len() == 1 {
        vm.to_vec(&a[0])?
    } else if a.is_empty() {
        return Err(vm.type_error(format!("{} expected at least 1 argument, got 0", name)));
    } else {
        if default.is_some() {
            return Err(vm.type_error(format!("Cannot specify a default for {}() with multiple positional arguments", name)));
        }
        a.to_vec()
    };
    if items.is_empty() {
        return match default {
            Some(d) => Ok(d),
            None => Err(vm.value_error(format!("{}() iterable argument is empty", name))),
        };
    }
    let mut best = items[0].clone();
    let mut best_k = if key.is_none() { best.clone() } else { vm.call_vec(&key, vec![best.clone()], &[])? };
    for x in items.into_iter().skip(1) {
        let kx = if key.is_none() { x.clone() } else { vm.call_vec(&key, vec![x.clone()], &[])? };
        let better = if is_max { vm.less(&best_k, &kx)? } else { vm.less(&kx, &best_k)? };
        if better {
            best = x;
            best_k = kx;
        }
    }
    Ok(best)
}

fn bi_max(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    min_max(vm, a, k, true)
}

fn bi_min(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    min_max(vm, a, k, false)
}

fn bi_sum(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    check_nargs(vm, "sum", a, 1, 2)?;
    let start = a.get(1).cloned().or_else(|| kwarg(k, "start")).unwrap_or(Value::Int(0));
    if let Value::Str(_) = start {
        return Err(vm.type_error("sum() can't sum strings [use ''.join(seq) instead]"));
    }
    let it = vm.iter(&a[0])?;
    let mut acc = start;
    // floats: compensated (Neumaier) summation like CPython 3.12
    let mut fsum: Option<(f64, f64)> = None;
    while let Some(x) = vm.next(&it)? {
        match (&acc, &x, &mut fsum) {
            (Value::Int(i), Value::Int(j), None) => {
                if let Some(s) = i.checked_add(*j) {
                    acc = Value::Int(s);
                    continue;
                }
            }
            (Value::Float(_), Value::Float(_) | Value::Int(_), _) => {
                let xf = vm.to_f64(&x)?;
                let (s, c) = fsum.get_or_insert_with(|| (if let Value::Float(f) = acc { f } else { 0.0 }, 0.0));
                let t = *s + xf;
                if s.abs() >= xf.abs() {
                    *c += (*s - t) + xf;
                } else {
                    *c += (xf - t) + *s;
                }
                *s = t;
                continue;
            }
            (Value::Int(i), Value::Float(f), None) => {
                fsum = Some((*i as f64, 0.0));
                let (s, c) = fsum.as_mut().unwrap();
                let t = *s + *f;
                if s.abs() >= f.abs() {
                    *c += (*s - t) + *f;
                } else {
                    *c += (*f - t) + *s;
                }
                *s = t;
                acc = Value::Float(0.0);
                continue;
            }
            _ => {}
        }
        if let Some((s, c)) = fsum.take() {
            acc = Value::Float(s + c);
        }
        acc = vm.binop(BinOp::Add, &acc, &x)?;
    }
    if let Some((s, c)) = fsum {
        let r = s + c;
        return Ok(Value::Float(if r.is_nan() && !s.is_nan() { s } else if s.is_infinite() { s } else { r }));
    }
    Ok(acc)
}

/// stable sort of `items` (by `key` when given)
pub fn sort_values(vm: &mut Vm, items: &mut Vec<Value>, key: &Value, reverse: bool) -> PyResult<()> {
    let keys: Vec<Value> = if key.is_none() {
        items.clone()
    } else {
        let mut ks = Vec::with_capacity(items.len());
        for x in items.iter() {
            ks.push(vm.call_vec(key, vec![x.clone()], &[])?);
        }
        ks
    };
    let n = items.len();
    let mut idx: Vec<usize> = (0..n).collect();
    if keys.iter().all(|k| matches!(k, Value::Int(_))) {
        let ik: Vec<i64> = keys.iter().map(|k| if let Value::Int(i) = k { *i } else { 0 }).collect();
        if reverse {
            idx.sort_by(|a, b| ik[*b].cmp(&ik[*a]));
        } else {
            idx.sort_by_key(|i| ik[*i]);
        }
    } else if keys.iter().all(|k| matches!(k, Value::Float(f) if !f.is_nan())) {
        let fk: Vec<f64> = keys.iter().map(|k| if let Value::Float(f) = k { *f } else { 0.0 }).collect();
        if reverse {
            idx.sort_by(|a, b| fk[*b].partial_cmp(&fk[*a]).unwrap());
        } else {
            idx.sort_by(|a, b| fk[*a].partial_cmp(&fk[*b]).unwrap());
        }
    } else if keys.iter().all(|k| matches!(k, Value::Str(_))) {
        let sk: Vec<&str> = keys.iter().map(|k| k.as_str().unwrap()).collect();
        if reverse {
            idx.sort_by(|a, b| sk[*b].cmp(sk[*a]));
        } else {
            idx.sort_by(|a, b| sk[*a].cmp(sk[*b]));
        }
    } else {
        // merge sort with fallible comparisons
        let mut buf = idx.clone();
        merge_sort(vm, &keys, &mut idx, &mut buf, reverse)?;
    }
    let old = std::mem::take(items);
    *items = idx.into_iter().map(|i| old[i].clone()).collect();
    Ok(())
}

fn merge_sort(vm: &mut Vm, keys: &[Value], idx: &mut [usize], buf: &mut [usize], reverse: bool) -> PyResult<()> {
    let n = idx.len();
    if n <= 1 {
        return Ok(());
    }
    if n <= 8 {
        // insertion sort
        for i in 1..n {
            let mut j = i;
            while j > 0 {
                let before = if reverse { vm.less(&keys[idx[j - 1]], &keys[idx[j]])? } else { vm.less(&keys[idx[j]], &keys[idx[j - 1]])? };
                if !before {
                    break;
                }
                idx.swap(j, j - 1);
                j -= 1;
            }
        }
        return Ok(());
    }
    let mid = n / 2;
    {
        let (l, r) = idx.split_at_mut(mid);
        let (bl, br) = buf.split_at_mut(mid);
        merge_sort(vm, keys, l, bl, reverse)?;
        merge_sort(vm, keys, r, br, reverse)?;
    }
    let (mut i, mut j, mut k) = (0, mid, 0);
    while i < mid && j < n {
        // take from the right only when strictly before (stability)
        let right_first = if reverse { vm.less(&keys[idx[i]], &keys[idx[j]])? } else { vm.less(&keys[idx[j]], &keys[idx[i]])? };
        if right_first {
            buf[k] = idx[j];
            j += 1;
        } else {
            buf[k] = idx[i];
            i += 1;
        }
        k += 1;
    }
    while i < mid {
        buf[k] = idx[i];
        i += 1;
        k += 1;
    }
    while j < n {
        buf[k] = idx[j];
        j += 1;
        k += 1;
    }
    idx.copy_from_slice(&buf[..n]);
    Ok(())
}

fn bi_sorted(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    check_nargs(vm, "sorted", a, 1, 1)?;
    let mut items = vm.to_vec(&a[0])?;
    let key = kwarg(k, "key").unwrap_or(Value::None);
    let rev = match kwarg(k, "reverse") {
        Some(v) => vm.truthy(&v)?,
        None => false,
    };
    sort_values(vm, &mut items, &key, rev)?;
    Ok(Value::list(items))
}

fn bi_pow(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let base = arg_or_kw(a, 0, k, "base");
    let exp = arg_or_kw(a, 1, k, "exp");
    let m = arg_or_kw(a, 2, k, "mod").unwrap_or(Value::None);
    let (base, exp) = match (base, exp) {
        (Some(b), Some(e)) => (b, e),
        _ => return Err(vm.type_error("pow() missing required argument")),
    };
    if m.is_none() {
        return vm.binop(BinOp::Pow, &base, &exp);
    }
    let (b, e, md) = match (crate::ops::big_of(&base), crate::ops::big_of(&exp), crate::ops::big_of(&m)) {
        (Some(b), Some(e), Some(md)) => (b, e, md),
        _ => return Err(vm.type_error("pow() 3rd argument not allowed unless all arguments are integers")),
    };
    if md.is_zero() {
        return Err(vm.value_error("pow() 3rd argument cannot be 0"));
    }
    let mut e = e;
    let mut b = b.divmod(&md).unwrap().1;
    if e.neg {
        // modular inverse
        b = match mod_inverse(&b, &md) {
            Some(x) => x,
            None => return Err(vm.value_error("base is not invertible for the given modulus")),
        };
        e = e.neg();
    }
    let mut r = BigInt::from_i64(1).divmod(&md).unwrap().1;
    let two = BigInt::from_i64(2);
    while !e.is_zero() {
        if e.is_odd() {
            r = r.mul(&b).divmod(&md).unwrap().1;
        }
        b = b.mul(&b).divmod(&md).unwrap().1;
        e = e.divmod(&two).unwrap().0;
    }
    Ok(Value::from_big(r))
}

fn mod_inverse(a: &BigInt, m: &BigInt) -> Option<BigInt> {
    let (mut old_r, mut r) = (a.clone(), m.clone());
    let (mut old_s, mut s) = (BigInt::from_i64(1), BigInt::zero());
    while !r.is_zero() {
        let (q, rem) = old_r.divmod(&r).unwrap();
        old_r = std::mem::replace(&mut r, rem);
        let ns = old_s.sub(&q.mul(&s));
        old_s = std::mem::replace(&mut s, ns);
    }
    if old_r.abs().to_i64() != Some(1) {
        return None;
    }
    Some(old_s.divmod(m).unwrap().1)
}

/// round half to even at `nd` decimals
pub fn round_float(x: f64, nd: i64) -> f64 {
    if !x.is_finite() {
        return x;
    }
    if nd > 300 {
        return x;
    }
    if nd >= 0 {
        // exact decimal rounding through the formatter
        let s = format!("{:.*}", nd as usize, x);
        let r: f64 = s.parse().unwrap_or(x);
        // Rust rounds ties on the exact binary value; recheck halfway cases
        let scaled = x * 10f64.powi(nd as i32);
        if scaled.is_finite() && (scaled - scaled.trunc()).abs() == 0.5 && scaled.abs() < 1e15 {
            let t = scaled.trunc();
            let even = if (t as i64) % 2 == 0 { t } else { t + scaled.signum() };
            return even / 10f64.powi(nd as i32);
        }
        r
    } else {
        let p = 10f64.powi((-nd) as i32);
        let y = x / p;
        let r = y.round();
        let r = if (y - y.trunc()).abs() == 0.5 { 2.0 * (y / 2.0).round() } else { r };
        r * p
    }
}

fn bi_round(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let x = match arg_or_kw(a, 0, k, "number") {
        Some(x) => x,
        None => return Err(vm.type_error("round() missing required argument 'number' (pos 1)")),
    };
    let nd = arg_or_kw(a, 1, k, "ndigits").unwrap_or(Value::None);
    match &x {
        Value::Float(f) => {
            if nd.is_none() {
                if f.is_nan() {
                    return Err(vm.value_error("cannot convert float NaN to integer"));
                }
                if f.is_infinite() {
                    return Err(vm.err("OverflowError", "cannot convert float infinity to integer"));
                }
                let r = f.round();
                let r = if (f - f.trunc()).abs() == 0.5 { 2.0 * (f / 2.0).round() } else { r };
                return Ok(if r.abs() < 9e18 { Value::Int(r as i64) } else { Value::from_big(BigInt::from_f64(r).unwrap()) });
            }
            let n = vm.to_index(&nd)?;
            Ok(Value::Float(round_float(*f, n)))
        }
        Value::Int(_) | Value::Bool(_) | Value::Big(_) => {
            if nd.is_none() {
                return Ok(match &x {
                    Value::Bool(b) => Value::Int(*b as i64),
                    o => o.clone(),
                });
            }
            let n = vm.to_index(&nd)?;
            if n >= 0 {
                return Ok(match &x {
                    Value::Bool(b) => Value::Int(*b as i64),
                    o => o.clone(),
                });
            }
            let b = crate::ops::big_of(&x).unwrap();
            let p = BigInt::from_i64(10).pow((-n) as u64);
            let (q, r) = b.divmod(&p).unwrap();
            // half to even
            let twice = r.mul(&BigInt::from_i64(2));
            let q = match twice.cmp(&p) {
                std::cmp::Ordering::Greater => q.add(&BigInt::from_i64(1)),
                std::cmp::Ordering::Equal if q.is_odd() => q.add(&BigInt::from_i64(1)),
                _ => q,
            };
            Ok(Value::from_big(q.mul(&p)))
        }
        _ => {
            if let Some(f) = vm.lookup_special(&x, &vm.n.round.clone()) {
                let args = if nd.is_none() { vec![] } else { vec![nd] };
                return vm.call_vec(&f, args, &[]);
            }
            let tn = vm.type_name(&x);
            Err(vm.type_error(format!("type {} doesn't define __round__ method", tn)))
        }
    }
}

/// the local variables of a frame as a dict
pub fn frame_locals(f: &Frame) -> Rc<RefCell<Dict>> {
    if let Some(ns) = &f.ns {
        return ns.clone();
    }
    let mut d = Dict::new();
    let code = &f.code;
    for (i, n) in code.varnames.iter().enumerate() {
        if let Some(ci) = code.cellvars.iter().position(|c| c == n) {
            let v = f.cells[ci].borrow().clone();
            if !v.is_undef() {
                d.set_s(n, v);
            }
            continue;
        }
        if i < f.stack.len() && !f.stack[i].is_undef() {
            d.set_s(n, f.stack[i].clone());
        }
    }
    for (ci, n) in code.cellvars.iter().enumerate() {
        if code.varnames.iter().any(|x| x == n) {
            continue;
        }
        let v = f.cells[ci].borrow().clone();
        if !v.is_undef() {
            d.set_s(n, v);
        }
    }
    for (fi, n) in code.freevars.iter().enumerate() {
        if let Some(c) = f.cells.get(code.cellvars.len() + fi) {
            let v = c.borrow().clone();
            if !v.is_undef() && &**n != "__class__" {
                d.set_s(n, v);
            }
        }
    }
    Rc::new(RefCell::new(d))
}

fn bi_locals(vm: &mut Vm, _a: &[Value], _k: Kw) -> PyResult<Value> {
    match vm.current_frame() {
        Some(f) => Ok(Value::Dict(frame_locals(f))),
        None => Ok(Value::Dict(vm.main_globals.clone())),
    }
}

fn bi_globals(vm: &mut Vm, _a: &[Value], _k: Kw) -> PyResult<Value> {
    match vm.current_frame() {
        Some(f) => Ok(Value::Dict(f.globals.clone())),
        None => Ok(Value::Dict(vm.main_globals.clone())),
    }
}

fn bi_vars(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    if a.is_empty() {
        return bi_locals(vm, a, k);
    }
    match vm.getattr_opt(&a[0], &vm.n.dict.clone())? {
        Some(d) => Ok(d),
        None => Err(vm.type_error("vars() argument must have __dict__ attribute")),
    }
}

fn eval_scopes(vm: &mut Vm, a: &[Value]) -> PyResult<(Rc<RefCell<Dict>>, Rc<RefCell<Dict>>)> {
    let (g, l) = match vm.current_frame() {
        Some(f) => (f.globals.clone(), frame_locals(f)),
        None => (vm.main_globals.clone(), vm.main_globals.clone()),
    };
    let g = match a.get(1) {
        Some(Value::Dict(d)) => d.clone(),
        Some(Value::None) | None => g,
        Some(v) => {
            let tn = vm.type_name(v);
            return Err(vm.type_error(format!("globals must be a dict, not {}", tn)));
        }
    };
    let l = match a.get(2) {
        Some(Value::Dict(d)) => d.clone(),
        Some(Value::Instance(i)) => match &*i.base.borrow() {
            Value::Dict(d) => d.clone(),
            _ => l,
        },
        _ => {
            if a.get(1).map(|v| !v.is_none()).unwrap_or(false) {
                g.clone()
            } else {
                l
            }
        }
    };
    if g.borrow().get_s("__builtins__").is_none() {
        let bm = Value::Module(Rc::new(Module { name: ps("builtins"), dict: vm.builtins.clone() }));
        g.borrow_mut().set_s("__builtins__", bm);
    }
    Ok((g, l))
}

fn source_of(vm: &mut Vm, v: &Value, what: &str) -> PyResult<Option<String>> {
    match v {
        Value::Str(s) => Ok(Some(s.s.clone())),
        Value::Bytes(b) => Ok(Some(String::from_utf8_lossy(b).into_owned())),
        Value::Code(_) => Ok(None),
        _ => Err(vm.type_error(format!("{}() arg 1 must be a string, bytes or code object", what))),
    }
}

fn bi_eval(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "eval", a, 1, 3)?;
    let (g, l) = eval_scopes(vm, a)?;
    let code = match source_of(vm, &a[0], "eval")? {
        Some(src) => {
            let src = src.trim_start_matches([' ', '\t']).to_string();
            let e = match crate::parser::parse_expression(&src) {
                Ok(e) => e,
                Err(e) => return Err(vm.syntax_error(&e.msg, e.line, "<string>")),
            };
            match crate::compile::compile_eval(&e, "<string>") {
                Ok(c) => c,
                Err(e) => return Err(vm.syntax_error(&e.msg, e.line, "<string>")),
            }
        }
        None => match &a[0] {
            Value::Code(c) => c.clone(),
            _ => unreachable!(),
        },
    };
    vm.run_code_ns(code, g, l)
}

fn bi_exec(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "exec", a, 1, 3)?;
    let (g, l) = eval_scopes(vm, a)?;
    let code = match source_of(vm, &a[0], "exec")? {
        Some(src) => {
            let body = match crate::parser::parse_module(&src) {
                Ok(b) => b,
                Err(e) => return Err(vm.syntax_error(&e.msg, e.line, "<string>")),
            };
            match crate::compile::compile_module(&body, "<string>") {
                Ok(c) => c,
                Err(e) => return Err(vm.syntax_error(&e.msg, e.line, "<string>")),
            }
        }
        None => match &a[0] {
            Value::Code(c) => c.clone(),
            _ => unreachable!(),
        },
    };
    vm.run_code_ns(code, g, l)?;
    Ok(Value::None)
}

fn bi_compile(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let src = arg_or_kw(a, 0, k, "source").unwrap_or(Value::None);
    let file = arg_or_kw(a, 1, k, "filename").map(|v| v.as_str().unwrap_or("<string>").to_string()).unwrap_or("<string>".into());
    let mode = arg_or_kw(a, 2, k, "mode").map(|v| v.as_str().unwrap_or("exec").to_string()).unwrap_or("exec".into());
    let s = match source_of(vm, &src, "compile")? {
        Some(s) => s,
        None => return Ok(src),
    };
    let code = if mode == "eval" {
        let e = match crate::parser::parse_expression(s.trim()) {
            Ok(e) => e,
            Err(e) => return Err(vm.syntax_error(&e.msg, e.line, &file)),
        };
        crate::compile::compile_eval(&e, &file)
    } else {
        let body = match crate::parser::parse_module(&s) {
            Ok(b) => b,
            Err(e) => return Err(vm.syntax_error(&e.msg, e.line, &file)),
        };
        crate::compile::compile_module(&body, &file)
    };
    match code {
        Ok(c) => Ok(Value::Code(c)),
        Err(e) => Err(vm.syntax_error(&e.msg, e.line, &file)),
    }
}

fn bi_exit(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let cls = vm.exc_class("SystemExit");
    let v = vm.new_exc(&cls, a.to_vec());
    Err(Exc { value: v })
}

fn bi_open(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    crate::modules::open_file(vm, a, k)
}

fn bi_import(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let name = match arg_or_kw(a, 0, k, "name") {
        Some(Value::Str(s)) => s.s.clone(),
        _ => return Err(vm.type_error("__import__() argument 1 must be str")),
    };
    let fromlist = arg_or_kw(a, 3, k, "fromlist").unwrap_or(Value::None);
    let level = match arg_or_kw(a, 4, k, "level") {
        Some(Value::Int(i)) => i,
        _ => 0,
    };
    let g = match arg_or_kw(a, 1, k, "globals") {
        Some(Value::Dict(d)) => d,
        _ => vm.main_globals.clone(),
    };
    let fl = match &fromlist {
        Value::None => Value::None,
        v => {
            if vm.truthy(v)? {
                v.clone()
            } else {
                Value::None
            }
        }
    };
    vm.import(&name, &fl, level, &g)
}

fn bi_build_class(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    if a.len() < 2 {
        return Err(vm.type_error("__build_class__: not enough arguments"));
    }
    vm.build_class(a, k)
}

fn bi_aiter(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "aiter", a, 1, 1)?;
    let f = vm.getattr(&a[0], &ps("__aiter__"))?;
    vm.call_vec(&f, vec![], &[])
}

fn bi_anext(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "anext", a, 1, 2)?;
    let f = vm.getattr(&a[0], &ps("__anext__"))?;
    vm.call_vec(&f, vec![], &[])
}
