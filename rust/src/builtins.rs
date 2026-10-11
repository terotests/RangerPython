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

crate::gil_static! {
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
            // the buffer protocol: memoryview and array
            if matches!(vm.type_name(v).as_str(), "memoryview" | "array") {
                if let Some(f) = vm.getattr_opt(v, &ps("tobytes"))? {
                    let r = vm.call_vec(&f, vec![], &[])?;
                    return bytes_of(vm, &r);
                }
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
                if what == "string" {
                    return Err(vm.type_error(format!("string indices must be integers, not '{}'", tn)));
                }
                let what = what.trim_end_matches(" assignment");
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

/// reversed() of a dict or one of its views
pub fn dict_rev_iter(d: &Rc<RefCell<Dict>>, kind: u8) -> Value {
    let (n, end) = {
        let db = d.borrow();
        (db.len(), db.entries.len())
    };
    mk_iter(Iter::Dict(d.clone(), end, kind | 4, n, n))
}

/// an index clipped to i64 instead of overflowing (CPython's _PyEval_SliceIndex);
/// None when `v` has no __index__
pub fn clamp_index(vm: &mut Vm, v: &Value) -> PyResult<Option<i64>> {
    match v {
        Value::Int(i) => Ok(Some(*i)),
        Value::Bool(b) => Ok(Some(*b as i64)),
        Value::Big(b) => Ok(Some(if b.neg { i64::MIN } else { i64::MAX })),
        Value::Instance(_) => {
            if let Value::Int(i) = vm.base_of(v) {
                return Ok(Some(i));
            }
            match vm.lookup_special(v, &vm.n.index.clone()) {
                Some(f) => {
                    let r = vm.call_vec(&f, vec![], &[])?;
                    match r {
                        Value::Int(_) | Value::Bool(_) | Value::Big(_) => clamp_index(vm, &r),
                        _ => vm.to_index(&r).map(Some),
                    }
                }
                None => Ok(None),
            }
        }
        _ => Ok(None),
    }
}

/// _PyEval_SliceIndex: an index clamped to the i64 range
pub fn slice_index(vm: &mut Vm, v: &Value) -> PyResult<i64> {
    let ok = match v {
        Value::Int(_) | Value::Big(_) | Value::Bool(_) => true,
        _ => vm.class_of(v).lookup(&vm.n.index).is_some(),
    };
    if !ok {
        return Err(vm.type_error("slice indices must be integers or None or have an __index__ method"));
    }
    match vm.number_index(v)? {
        Value::Int(i) => Ok(i),
        Value::Big(b) => Ok(if b.neg { i64::MIN } else { i64::MAX }),
        _ => Ok(0),
    }
}

/// start, stop, step of a slice over `len` items (CPython's PySlice_AdjustIndices)
pub fn slice_indices(vm: &mut Vm, s: &(Value, Value, Value), len: usize) -> PyResult<(i64, i64, i64)> {
    let len = len as i64;
    let step = if s.2.is_none() { 1 } else { slice_index(vm, &s.2)?.max(-i64::MAX) };
    if step == 0 {
        return Err(vm.value_error("slice step cannot be zero"));
    }
    let conv = |vm: &mut Vm, v: &Value, dflt: i64| -> PyResult<i64> {
        if v.is_none() {
            return Ok(dflt);
        }
        slice_index(vm, v).map(|i| i.clamp(i64::MIN / 2, i64::MAX / 2))
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
            match i.checked_add(step) {
                Some(n) => i = n,
                None => break,
            }
        }
    } else {
        while i > stop {
            out.push(i as usize);
            match i.checked_add(step) {
                Some(n) => i = n,
                None => break,
            }
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
                if s.has_surrogates() {
                    let cps: Vec<u32> = s.code_points().collect();
                    let (a, b, st) = slice_indices(vm, sl, cps.len())?;
                    let picked = slice_positions(a, b, st).into_iter().map(|i| cps[i]);
                    return Ok(Value::Str(Rc::new(PyStr::from_code_points(picked))));
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
            Value::Range(_) => match crate::range::parts(obj) {
                Some((start, stop, step, len)) => crate::range::getitem(vm, &crate::range::BigRange { start, stop, step, len }, idx),
                None => Err(vm.type_error("unsliceable")),
            },
            Value::BigRange(r) => crate::range::getitem(vm, r, idx),
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
            Ok(Value::from_cp(s.code_points().nth(i).unwrap()))
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
        Value::BigRange(r) => crate::range::getitem(vm, r, idx),
        _ => Err(vm.type_error("not subscriptable")),
    }
}

pub fn seq_setitem(vm: &mut Vm, obj: &Value, idx: &Value, v: Value) -> PyResult<()> {
    match obj {
        Value::List(l) => {
            if let Value::Slice(sl) = idx {
                let items = match vm.to_vec(&v) {
                    Ok(items) => items,
                    Err(e) if vm.exc_is(&e, "TypeError") && vm.iter(&v).is_err() => return Err(vm.type_error("must assign iterable to extended slice")),
                    Err(e) => return Err(e),
                };
                if let Some(x) = items.iter().find(|x| x.holds_refs()) {
                    crate::gc::list_store(l, x);
                }
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
            crate::gc::list_store(l, &v);
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
            let d = new_dict();
            vm.set_extend(&d, v)?;
            Ok(Value::Set(d))
        }
    }
}

fn keyed(d: &Rc<RefCell<Dict>>) -> Vec<(u64, Value)> {
    d.borrow().iter().map(|e| (e.hash, e.key.clone())).collect()
}

fn set_has(vm: &mut Vm, d: &Rc<RefCell<Dict>>, h: u64, k: &Value) -> PyResult<bool> {
    Ok(vm.dict_find_h(d, h, k)?.1.is_some())
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
            for (h, e) in keyed(&db) {
                vm.set_add_h(&out, h, e)?;
            }
        }
        BinOp::BitAnd => {
            for (h, e) in keyed(&da) {
                if set_has(vm, &db, h, &e)? {
                    vm.set_add_h(&out, h, e)?;
                }
            }
        }
        BinOp::Sub => {
            for (h, e) in keyed(&da) {
                if !set_has(vm, &db, h, &e)? {
                    vm.set_add_h(&out, h, e)?;
                }
            }
        }
        _ => {
            for (h, e) in keyed(&da) {
                if !set_has(vm, &db, h, &e)? {
                    vm.set_add_h(&out, h, e)?;
                }
            }
            for (h, e) in keyed(&db) {
                if !set_has(vm, &da, h, &e)? {
                    vm.set_add_h(&out, h, e)?;
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
    for (h, k) in keyed(a) {
        if !set_has(vm, b, h, &k)? {
            return Ok(false);
        }
    }
    Ok(true)
}

pub fn set_compare(vm: &mut Vm, op: CmpOp, a: &Value, b: &Value) -> PyResult<Value> {
    if crate::ops::set_view_pair(a, b) {
        return Ok(Value::Bool(vm.view_cmp(op, a, b)?));
    }
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
// generic aliases and unions (list[int], int | None), which live in
// pylib/_genericalias.py

fn alias_helper(vm: &mut Vm, f: &str, args: Vec<Value>) -> PyResult<Value> {
    let m = vm.import("_genericalias", &Value::None, 0, &vm.main_globals.clone())?;
    let h = vm.getattr(&m, &Rc::new(PyStr::new(f.to_string())))?;
    vm.call_vec(&h, args, &[])
}

/// origin[idx] as a types.GenericAlias
pub fn generic_alias(vm: &mut Vm, origin: &Value, idx: &Value) -> PyResult<Value> {
    alias_helper(vm, "_generic_alias", vec![origin.clone(), idx.clone()])
}

/// a | b on types: a types.UnionType, or NotImplemented
pub fn make_union(vm: &mut Vm, a: &Value, b: &Value) -> PyResult<Value> {
    alias_helper(vm, "_union_or", vec![a.clone(), b.clone()])
}

/// the members of a types.UnionType, which isinstance treats as a tuple
fn union_args(vm: &mut Vm, v: &Value) -> PyResult<Option<Value>> {
    if let Value::Instance(i) = v {
        let c = i.class.borrow().clone();
        if c.name() == "UnionType" && matches!(c.dict.borrow().get_s("__module__"), Some(Value::Str(m)) if &*m.s == "types") {
            return Ok(Some(vm.getattr(v, &Rc::new(PyStr::new("__args__".to_string())))?));
        }
    }
    Ok(None)
}

// -------------------------------------------------------------------------
// attributes of built-in values that are not in a type's dict

pub fn func_attr(vm: &mut Vm, f: &Rc<Function>, name: &str) -> Option<Value> {
    Some(match name {
        "__name__" => Value::Str(f.name.borrow().clone()),
        "__qualname__" => Value::Str(f.qualname.borrow().clone()),
        "__module__" => f.globals.borrow().get_s("__name__").unwrap_or(Value::None),
        "__doc__" => f.code.borrow().doc.clone(),
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
        "__code__" => Value::Code(f.code.borrow().clone()),
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
        // the builtins the function runs with, as a dict
        "__builtins__" => match f.globals.borrow().get_s("__builtins__") {
            Some(Value::Module(m)) => Value::Dict(m.dict.clone()),
            Some(d @ Value::Dict(_)) => d,
            _ => Value::Dict(vm.builtins.clone()),
        },
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
        "co_flags" => {
            // CO_GENERATOR, CO_COROUTINE, or both as CO_ASYNC_GENERATOR
            let kind = match (c.generator, c.coroutine) {
                (true, true) => 0x200,
                (g, co) => ((g as i64) << 5) | ((co as i64) << 7),
            };
            Value::Int(((c.varargs as i64) << 2) | ((c.varkw as i64) << 3) | kind | ((c.iterable_coroutine as i64) << 8))
        }
        "co_linetable" => Value::Bytes(Rc::new(crate::code::encode_linetable(c))),
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
        Value::BigRange(r) => match name {
            "start" => Some(Value::from_big(r.start.clone())),
            "stop" => Some(Value::from_big(r.stop.clone())),
            "step" => Some(Value::from_big(r.step.clone())),
            _ => None,
        },
        Value::Native(n) => match name {
            // a type's method is qualified by the type ("int.__add__")
            "__qualname__" if n.bind => Some(Value::str(n.name)),
            "__name__" | "__qualname__" => Some(Value::str(n.name.rsplit('.').next().unwrap_or(n.name))),
            "__module__" => Some(Value::str(native_module_name(n.name))),
            "__self__" => Some(Value::None),
            // a method of a type is a descriptor
            "__get__" if n.bind => Some(Value::Method(Rc::new((obj.clone(), native("wrapper_descriptor.__get__", descr_get))))),
            "__doc__" => Some(builtin_doc(n.name).map(|(_, d)| Value::str(d)).unwrap_or(Value::None)),
            "__text_signature__" => Some(builtin_doc(n.name).and_then(|(s, _)| (!s.is_empty()).then(|| Value::str(s))).unwrap_or(Value::None)),
            _ => None,
        },
        Value::Method(m) => match name {
            "__self__" => Some(m.0.clone()),
            "__func__" => Some(m.1.clone()),
            "__name__" | "__qualname__" | "__doc__" | "__module__" | "__wrapped__" | "__defaults__" | "__code__" | "__annotations__" | "__dict__" | "__text_signature__" => match &m.1 {
                Value::Native(n) if name == "__qualname__" && n.bind => Some(Value::str(n.name)),
                Value::Native(n) if name == "__name__" || name == "__qualname__" => Some(Value::str(n.name.rsplit('.').next().unwrap_or(n.name))),
                f => vm.getattr_opt(&f.clone(), &ps(name))?,
            },
            _ => match &m.1 {
                Value::Func(f) => f.dict.borrow().as_ref().and_then(|d| d.borrow().get_s(name)),
                _ => None,
            },
        },
        Value::Gen(g) if matches!(name, "gi_frame" | "cr_frame" | "ag_frame") => {
            let mut g = g.borrow_mut();
            let g = &mut *g;
            match &g.frame {
                Some(f) => {
                    let line = f.code.line_at(f.ip.saturating_sub(1)).unwrap_or(f.code.firstline);
                    // one frame object per generator, its line kept current
                    if let Some(Value::Obj(o)) = &g.frame_obj {
                        if let ObjKind::Frame { line: l, .. } = &mut *o.kind.borrow_mut() {
                            *l = line;
                        }
                    }
                    if g.frame_obj.is_none() {
                        g.frame_obj = Some(vm.frame_obj(f, line));
                    }
                    g.frame_obj.clone()
                }
                None => Some(Value::None),
            }
        }
        Value::Gen(g) => {
            let g = g.borrow();
            match name {
                "__name__" => Some(Value::Str(g.name.clone())),
                "__qualname__" => Some(Value::Str(g.qualname.clone())),
                "gi_running" | "cr_running" => Some(Value::Bool(g.running)),
                "gi_code" | "cr_code" => Some(Value::Code(g.code.clone())),
                "gi_yieldfrom" | "cr_await" => Some(match &g.frame {
                    Some(f) if !g.running && matches!(f.code.ops.get(f.ip), Some(crate::code::Op::YieldFrom)) => f.stack.last().cloned().unwrap_or(Value::None),
                    _ => Value::None,
                }),
                "gi_suspended" | "cr_suspended" => Some(Value::Bool(g.started && !g.running && g.frame.is_some())),
                _ => None,
            }
        }
        Value::Property(p) => match name {
            "fget" => Some(p.fget.clone()),
            "fset" => Some(p.fset.clone()),
            "fdel" => Some(p.fdel.clone()),
            "__doc__" => Some(p.doc.borrow().clone()),
            "__name__" => {
                let n = p.name.borrow().clone();
                if !n.is_undef() {
                    Some(n)
                } else {
                    let f = p.fget.clone();
                    match vm.getattr_opt(&f, &ps("__name__"))? {
                        Some(v) if !f.is_none() => Some(v),
                        _ => return Err(vm.attr_error("'property' object has no attribute '__name__'")),
                    }
                }
            }
            "__isabstractmethod__" => {
                let fs = [p.fget.clone(), p.fset.clone(), p.fdel.clone()];
                let mut r = false;
                for f in &fs {
                    if f.is_none() {
                        continue;
                    }
                    let v = match vm.getattr_opt(f, &ps("__isabstractmethod__")) {
                        Ok(v) => v,
                        Err(e) if vm.exc_is(&e, "AttributeError") => None,
                        Err(e) => return Err(e),
                    };
                    if let Some(v) = v {
                        if vm.truthy(&v)? {
                            r = true;
                            break;
                        }
                    }
                }
                Some(Value::Bool(r))
            }
            _ => None,
        },
        Value::StaticMethod(f) | Value::ClassMethod(f) => match name {
            "__func__" | "__wrapped__" => Some((**f).clone()),
            "__isabstractmethod__" | "__name__" | "__qualname__" | "__doc__" | "__module__" | "__annotations__" => {
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
            "mapping" => Some(crate::methods::mappingproxy(vm, Value::Dict(dv.0.clone()))),
            _ => None,
        },
        Value::Obj(o) => {
            if let Some(v) = crate::tb::obj_attr(vm, o, name)? {
                return Ok(Some(v));
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
            "__traceback__" | "__cause__" | "__context__" => Some(crate::tb::exc_extra(vm, inst, name)),
            "__suppress_context__" => Some(inst.extra.borrow().get_str(&vm.n.suppress).cloned().unwrap_or(Value::Bool(false))),
            _ if exc_fields(vm, &cls).contains(&name) => Some(inst.extra.borrow().get_s(name).unwrap_or(Value::None)),
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

/// type.__flags__, as CPython's Py_TPFLAGS_* bits
fn type_flags(vm: &mut Vm, c: &Rc<Class>) -> PyResult<i64> {
    let heap = !c.immutable.get() && matches!(c.dict.borrow().get_s("__module__"), Some(Value::Str(m)) if &*m.s != "builtins");
    let mut f: i64 = 1 << 12;
    if heap {
        f |= (1 << 9) | (1 << 14);
        if matches!(c.builtin.get(), Builtin::User | Builtin::Object) && crate::ops::has_dict(c) {
            f |= (1 << 2) | (1 << 3);
        }
    } else {
        f |= 1 << 8;
    }
    if !matches!(c.builtin.get(), Builtin::Bool | Builtin::NoneType | Builtin::NotImplementedType | Builtin::Ellipsis | Builtin::Range | Builtin::Slice | Builtin::Function | Builtin::Code | Builtin::Cell) {
        f |= 1 << 10;
    }
    f |= match c.builtin.get() {
        Builtin::Int | Builtin::Bool => 1 << 24,
        Builtin::List => (1 << 25) | (1 << 5),
        Builtin::Tuple => (1 << 26) | (1 << 5),
        Builtin::Bytes => 1 << 27,
        Builtin::Str => 1 << 28,
        Builtin::Dict => (1 << 29) | (1 << 6),
        Builtin::BaseException => 1 << 30,
        Builtin::Type => 1 << 31,
        Builtin::Range => 1 << 5,
        _ => 0,
    };
    // the sequence/mapping flags an ABC declares, inherited down the MRO
    for k in c.mro.borrow().iter() {
        if let Some(Value::Int(v)) = k.dict.borrow().get_s("__abc_tpflags__") {
            f |= v & ((1 << 5) | (1 << 6));
            break;
        }
    }
    let am = c.dict.borrow().get_s("__abstractmethods__");
    if let Some(am) = am {
        if vm.truthy(&am)? {
            f |= 1 << 20;
        }
    }
    Ok(f)
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
        "__base__" => {
            // the base carrying the built-in layout, else the first
            let bs = c.bases.borrow();
            let solid = bs.iter().find(|b| b.mro.borrow().iter().any(|m| !matches!(m.builtin.get(), Builtin::User | Builtin::Object)));
            match solid.or(bs.first()) {
                Some(b) => Value::Class(b.clone()),
                None => Value::None,
            }
        }
        "__dict__" => crate::methods::mappingproxy(vm, Value::Dict(c.dict.clone())),
        "__module__" => Value::str("builtins"),
        // a class of the runtime's own has CPython's docstring
        "__doc__" => match c.dict.borrow().get_s("__module__") {
            None => builtin_doc(&c.name.borrow().s).map(|(_, d)| Value::str(d)).unwrap_or(Value::None),
            Some(_) => Value::None,
        },
        "__class__" => Value::Class(vm.class_of(&Value::Class(c.clone()))),
        "__annotations__" => {
            let d = Value::dict(Dict::new());
            if c.builtin.get() == Builtin::User || c.dict.borrow().get_s("__module__").is_some() {
                c.dict.borrow_mut().set_s("__annotations__", d.clone());
            }
            d
        }
        "__type_params__" => Value::tuple(vec![]),
        "__flags__" => Value::Int(type_flags(vm, c)?),
        "__basicsize__" => Value::Int(match c.builtin.get() {
            Builtin::Object | Builtin::NoneType => 16,
            Builtin::Int | Builtin::Bool | Builtin::Tuple | Builtin::Bytes => 24,
            Builtin::Float => 24,
            _ => 32,
        }),
        "__itemsize__" => Value::Int(match c.builtin.get() {
            Builtin::Int | Builtin::Bool => 4,
            Builtin::Tuple => 8,
            Builtin::Bytes => 1,
            _ => 0,
        }),
        "__abstractmethods__" => return Ok(None),
        "__text_signature__" => Value::None,
        _ => return Ok(None),
    }))
}

// -------------------------------------------------------------------------
// exceptions

/// a UnicodeDecodeError with CPython's fields
pub fn decode_error(vm: &mut Vm, enc: &str, obj: &[u8], start: usize, end: usize, reason: &str) -> Exc {
    let cls = vm.exc_class("UnicodeDecodeError");
    let v = vm.new_exc(&cls, vec![Value::str(enc), Value::Bytes(Rc::new(obj.to_vec())), Value::Int(start as i64), Value::Int(end as i64), Value::str(reason)]);
    Exc { value: v }
}

/// a UnicodeEncodeError with CPython's fields
pub fn encode_error(vm: &mut Vm, enc: &str, obj: &Rc<PyStr>, start: usize, end: usize, reason: &str) -> Exc {
    let cls = vm.exc_class("UnicodeEncodeError");
    let v = vm.new_exc(&cls, vec![Value::str(enc), Value::Str(obj.clone()), Value::Int(start as i64), Value::Int(end as i64), Value::str(reason)]);
    Exc { value: v }
}

/// the attributes an exception family keeps outside __dict__ (C slots in CPython)
pub fn exc_fields(vm: &Vm, cls: &Rc<Class>) -> &'static [&'static str] {
    let is = |n: &str| match vm.exc.get(n) {
        Some(k) => cls.is_subclass(k),
        None => false,
    };
    if is("OSError") {
        &["errno", "strerror", "filename", "filename2"]
    } else if is("StopIteration") {
        &["value"]
    } else if is("SystemExit") {
        &["code"]
    } else if is("ImportError") {
        &["msg", "name", "path", "name_from"]
    } else if is("AttributeError") {
        &["name", "obj"]
    } else if is("NameError") {
        &["name"]
    } else if is("SyntaxError") {
        &["msg", "filename", "lineno", "offset", "text", "end_lineno", "end_offset", "print_file_and_line", "_metadata"]
    } else if is("UnicodeError") {
        &["encoding", "object", "start", "end", "reason"]
    } else {
        &[]
    }
}

fn exc_field(inst: &Instance, name: &str) -> Value {
    inst.extra.borrow().get_s(name).unwrap_or(Value::None)
}

pub fn exc_set_field(v: &Value, name: &str, val: Value) {
    if let Value::Instance(i) = v {
        i.extra.borrow_mut().set_s(name, val);
    }
}

fn kw_check(vm: &mut Vm, tname: &str, k: Kw, allowed: &[&str]) -> PyResult<()> {
    for (n, _) in k {
        if !allowed.contains(&&*n.s) {
            return Err(vm.type_error(if allowed.is_empty() {
                format!("{}() takes no keyword arguments", tname)
            } else {
                format!("{}() got an unexpected keyword argument '{}'", tname, n.s)
            }));
        }
    }
    Ok(())
}

fn kw_get(k: Kw, name: &str) -> Value {
    k.iter().find(|(n, _)| &*n.s == name).map(|(_, v)| v.clone()).unwrap_or(Value::None)
}

/// BaseException.__init__ and the subclass slots it fills; returns the args to keep
/// the built-in exceptions exc_setup gives fields
const EXC_WITH_FIELDS: [&str; 11] = ["ImportError", "AttributeError", "NameError", "OSError", "StopIteration", "SystemExit", "SyntaxError", "UnicodeError", "UnicodeTranslateError", "UnicodeDecodeError", "UnicodeEncodeError"];

fn exc_setup(vm: &mut Vm, inst: &Instance, args: &[Value], k: Kw) -> PyResult<Vec<Value>> {
    let cls = inst.class.borrow().clone();
    // the built-in exception classes with fields of their own along the
    // MRO, found once: by name, then checked to be the built-in class
    let bases: Vec<&'static str> = cls
        .mro
        .borrow()
        .iter()
        .filter_map(|k| {
            let name = k.name.borrow();
            let n = *EXC_WITH_FIELDS.iter().find(|n| **n == &*name.s)?;
            vm.exc.get(n).filter(|c| Rc::ptr_eq(c, k)).map(|_| n)
        })
        .collect();
    let is = |_: &Vm, n: &str| bases.contains(&n);
    let mut keep = args.to_vec();
    let mut set: Vec<(&str, Value)> = vec![];
    let first = args.first().cloned().unwrap_or(Value::None);
    if is(vm, "ImportError") {
        kw_check(vm, "ImportError", k, &["name", "path"])?;
        let msg = if args.len() == 1 { first.clone() } else { Value::None };
        set = vec![("msg", msg), ("name", kw_get(k, "name")), ("path", kw_get(k, "path")), ("name_from", Value::None)];
    } else if is(vm, "AttributeError") {
        kw_check(vm, "AttributeError", k, &["name", "obj"])?;
        set = vec![("name", kw_get(k, "name")), ("obj", kw_get(k, "obj"))];
    } else if is(vm, "NameError") {
        kw_check(vm, "NameError", k, &["name"])?;
        set = vec![("name", kw_get(k, "name"))];
    } else if !k.is_empty() {
        let tn = cls.name().to_string();
        kw_check(vm, &tn, k, &[])?;
    }
    if is(vm, "OSError") {
        let mut f = [Value::None, Value::None, Value::None, Value::None];
        if (2..=5).contains(&args.len()) {
            f[0] = args[0].clone();
            f[1] = args[1].clone();
            if args.len() >= 3 {
                f[2] = args[2].clone();
                if args.len() == 5 {
                    f[3] = args[4].clone();
                }
                keep.truncate(2);
            }
        }
        let [e, s, n, n2] = f;
        set = vec![("errno", e), ("strerror", s), ("filename", n), ("filename2", n2)];
    } else if is(vm, "StopIteration") {
        set = vec![("value", first)];
    } else if is(vm, "SystemExit") {
        let code = if args.len() > 1 { Value::tuple(args.to_vec()) } else { first };
        set = vec![("code", code)];
    } else if is(vm, "SyntaxError") {
        set = vec![("msg", first), ("print_file_and_line", Value::None)];
        let mut info = [Value::None, Value::None, Value::None, Value::None, Value::None, Value::None];
        if args.len() == 2 {
            let t = vm.to_vec(&args[1])?;
            if !(4..=6).contains(&t.len()) {
                return Err(vm.type_error(format!("function takes at {} arguments ({} given)", if t.len() < 4 { "least 4" } else { "most 6" }, t.len())));
            }
            if t.len() == 5 {
                return Err(vm.type_error("end_offset must be provided when end_lineno is provided"));
            }
            for (i, v) in t.into_iter().enumerate() {
                info[i] = v;
            }
        }
        for (n, v) in ["filename", "lineno", "offset", "text", "end_lineno", "end_offset"].into_iter().zip(info) {
            set.push((n, v));
        }
    } else if is(vm, "UnicodeError") {
        let translate = is(vm, "UnicodeTranslateError");
        let decode = is(vm, "UnicodeDecodeError");
        if translate || decode || is(vm, "UnicodeEncodeError") {
            let want = if translate { 4 } else { 5 };
            if args.len() != want {
                return Err(vm.type_error(format!("function takes exactly {} arguments ({} given)", want, args.len())));
            }
            let off = if translate { 0 } else { 1 };
            let mut v = args.to_vec();
            if !translate && !matches!(v[0], Value::Str(_)) {
                let tn = vm.type_name(&v[0]);
                return Err(vm.type_error(format!("argument 1 must be str, not {}", tn)));
            }
            if decode {
                v[1] = match &v[1] {
                    Value::Bytes(b) => Value::Bytes(b.clone()),
                    Value::ByteArray(b) => Value::Bytes(Rc::new(b.borrow().clone())),
                    o => {
                        let tn = vm.type_name(o);
                        return Err(vm.type_error(format!("a bytes-like object is required, not '{}'", tn)));
                    }
                };
            } else if !matches!(v[off], Value::Str(_)) {
                let tn = vm.type_name(&v[off]);
                return Err(vm.type_error(format!("argument {} must be str, not {}", off + 1, tn)));
            }
            for i in [off + 1, off + 2] {
                if !matches!(v[i], Value::Int(_) | Value::Big(_)) {
                    let tn = vm.type_name(&v[i]);
                    return Err(vm.type_error(format!("'{}' object cannot be interpreted as an integer", tn)));
                }
            }
            if !matches!(v[off + 3], Value::Str(_)) {
                let tn = vm.type_name(&v[off + 3]);
                return Err(vm.type_error(format!("argument {} must be str, not {}", off + 4, tn)));
            }
            let enc = if translate { Value::None } else { v[0].clone() };
            set = vec![("encoding", enc), ("object", v[off].clone()), ("start", v[off + 1].clone()), ("end", v[off + 2].clone()), ("reason", v[off + 3].clone())];
        }
    }
    let mut ex = inst.extra.borrow_mut();
    for (n, v) in set {
        ex.set_str(exc_field_name(n), v);
    }
    Ok(keep)
}

/// the str for an exception field name, made once
fn exc_field_name(n: &str) -> Rc<PyStr> {
    crate::gil_static! {
        static NAMES: RefCell<Vec<Rc<PyStr>>> = const { RefCell::new(Vec::new()) };
    }
    NAMES.with(|names| {
        let mut names = names.borrow_mut();
        if let Some(s) = names.iter().find(|s| s.s == n) {
            return s.clone();
        }
        let s = ps(n);
        names.push(s.clone());
        s
    })
}

/// fills the subclass slots of an exception made natively from `args`
pub fn exc_init_native(vm: &mut Vm, v: &Value) {
    if let Value::Instance(i) = v {
        let args = vm.exc_args(v);
        if let Ok(keep) = exc_setup(vm, i, &args, &[]) {
            if keep.len() != args.len() {
                *i.base.borrow_mut() = Value::tuple(keep);
            }
        }
    }
}

fn unicode_error_str(vm: &mut Vm, cls: &Rc<Class>, inst: &Instance) -> PyResult<Option<String>> {
    let decode = cls.is_subclass(&vm.exc_class("UnicodeDecodeError"));
    let encode = cls.is_subclass(&vm.exc_class("UnicodeEncodeError"));
    let translate = cls.is_subclass(&vm.exc_class("UnicodeTranslateError"));
    if !(decode || encode || translate) {
        return Ok(None);
    }
    let obj = exc_field(inst, "object");
    if obj.is_none() {
        return Ok(Some(String::new()));
    }
    let (start, end) = match (exc_field(inst, "start"), exc_field(inst, "end")) {
        (Value::Int(a), Value::Int(b)) => (a, b),
        _ => (0, 0),
    };
    let reason = exc_field(inst, "reason");
    let reason = vm.str_of(&reason)?;
    let enc = if translate { String::new() } else {
        let e = exc_field(inst, "encoding");
        vm.str_of(&e)?
    };
    let len = match &obj {
        Value::Bytes(b) => b.len() as i64,
        Value::Str(s) => s.s.chars().count() as i64,
        _ => 0,
    };
    let single = start >= 0 && start < len && end == start + 1;
    if decode {
        let byte = match &obj {
            Value::Bytes(b) if single => b.get(start as usize).copied(),
            _ => None,
        };
        return Ok(Some(match byte {
            Some(c) => format!("'{}' codec can't decode byte 0x{:02x} in position {}: {}", enc, c, start, reason),
            None => format!("'{}' codec can't decode bytes in position {}-{}: {}", enc, start, end - 1, reason),
        }));
    }
    let ch = match &obj {
        Value::Str(s) if single => s.code_points().nth(start as usize),
        _ => None,
    };
    let what = if translate { "can't translate".to_string() } else { format!("'{}' codec can't encode", enc) };
    Ok(Some(match ch {
        Some(n) => {
            let esc = if n <= 0xff { format!("\\x{:02x}", n) } else if n <= 0xffff { format!("\\u{:04x}", n) } else { format!("\\U{:08x}", n) };
            format!("{} character '{}' in position {}: {}", what, esc, start, reason)
        }
        None => format!("{} characters in position {}-{}: {}", what, start, end - 1, reason),
    }))
}

/// str(exc) as a str; exact when its sole argument has lone surrogates
pub fn exc_str_value(vm: &mut Vm, v: &Value) -> PyResult<Value> {
    match &vm.exc_args(v)[..] {
        [a @ Value::Str(p)] if p.has_surrogates() && !vm.class_of(v).is_subclass(&vm.exc_class("KeyError")) => Ok(a.clone()),
        _ => exc_str(vm, v).map(Value::string),
    }
}

pub fn exc_str(vm: &mut Vm, v: &Value) -> PyResult<String> {
    let args = vm.exc_args(v);
    let cls = vm.class_of(v);
    if let Value::Instance(inst) = v {
        if let Some(s) = unicode_error_str(vm, &cls, inst)? {
            return Ok(s);
        }
        if cls.is_subclass(&vm.exc_class("OSError")) {
            let (e, s, f, f2) = (exc_field(inst, "errno"), exc_field(inst, "strerror"), exc_field(inst, "filename"), exc_field(inst, "filename2"));
            if !f.is_none() {
                let (e, s, f) = (vm.str_of(&e)?, vm.str_of(&s)?, vm.repr(&f)?);
                if !f2.is_none() {
                    let f2 = vm.repr(&f2)?;
                    return Ok(format!("[Errno {}] {}: {} -> {}", e, s, f, f2));
                }
                return Ok(format!("[Errno {}] {}: {}", e, s, f));
            }
            if !e.is_none() && !s.is_none() {
                let (e, s) = (vm.str_of(&e)?, vm.str_of(&s)?);
                return Ok(format!("[Errno {}] {}", e, s));
            }
        }
        if cls.is_subclass(&vm.exc_class("SyntaxError")) {
            let msg = exc_field(inst, "msg");
            let mut out = vm.str_of(&msg)?;
            let file = match exc_field(inst, "filename") {
                Value::Str(s) => Some(s.s.rsplit('/').next().unwrap_or("").to_string()),
                _ => None,
            };
            let line = match exc_field(inst, "lineno") {
                Value::Int(n) => Some(n),
                _ => None,
            };
            match (file, line) {
                (Some(f), Some(l)) => out = format!("{} ({}, line {})", out, f, l),
                (Some(f), None) => out = format!("{} ({})", out, f),
                (None, Some(l)) => out = format!("{} (line {})", out, l),
                _ => {}
            }
            return Ok(out);
        }
    }
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

/// the OSError subclass CPython picks for an errno
fn errno_class(errno: i64) -> Option<&'static str> {
    Some(match errno {
        11 | 114 | 115 => "BlockingIOError",
        10 => "ChildProcessError",
        32 | 108 => "BrokenPipeError",
        103 => "ConnectionAbortedError",
        111 => "ConnectionRefusedError",
        104 => "ConnectionResetError",
        17 => "FileExistsError",
        2 => "FileNotFoundError",
        21 => "IsADirectoryError",
        20 => "NotADirectoryError",
        4 => "InterruptedError",
        1 | 13 => "PermissionError",
        3 => "ProcessLookupError",
        110 => "TimeoutError",
        _ => return None,
    })
}

fn exc_new(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let mut cls = match &a[0] {
        Value::Class(c) => c.clone(),
        _ => return Err(vm.type_error("BaseException.__new__(X): X is not a type object")),
    };
    if !cls.is_subclass(&vm.t.base_exception) {
        let cn = cls.name().to_string();
        return Err(vm.type_error(format!("BaseException.__new__({}): {} is not a subtype of BaseException", cn, cn)));
    }
    let args = &a[1..];
    if Rc::ptr_eq(&cls, &vm.exc_class("OSError")) && args.len() >= 2 {
        if let Some(n) = errno_class(match &args[0] { Value::Int(n) => *n, _ => -1 }) {
            cls = vm.exc_class(n);
        }
    }
    let v = vm.new_instance(&cls);
    if let Value::Instance(i) = &v {
        *i.base.borrow_mut() = Value::tuple(args.to_vec());
        if let Ok(keep) = exc_setup(vm, i, args, &[]) {
            *i.base.borrow_mut() = Value::tuple(keep);
        }
    }
    Ok(v)
}

fn exc_init(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    if let Value::Instance(i) = &a[0] {
        let keep = exc_setup(vm, i, &a[1..], k)?;
        *i.base.borrow_mut() = Value::tuple(keep);
    }
    Ok(Value::None)
}

fn exc_str_native(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    exc_str_value(vm, &a[0])
}

fn exc_repr_native(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let cls = vm.class_of(&a[0]);
    let args = vm.exc_args(&a[0]);
    let inner = if args.len() == 1 { vm.repr(&args[0])? } else { let r = vm.repr(&Value::tuple(args.clone()))?; r[1..r.len() - 1].trim_end_matches(',').to_string() };
    Ok(Value::string(format!("{}({})", cls.name(), inner)))
}

fn exc_with_traceback(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 2)?;
    margs(vm, "with_traceback", a, 1, 1)?;
    vm.setattr(&a[0], &vm.n.tb.clone(), a[1].clone())?;
    Ok(a[0].clone())
}

fn exc_add_note(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    margs(vm, "add_note", a, 1, 1)?;
    if !matches!(a[1], Value::Str(_)) {
        let tn = vm.type_name(&a[1]);
        return Err(vm.type_error(format!("note must be a str, not '{}'", tn)));
    }
    if let Value::Instance(i) = &a[0] {
        let cur = i.attr(&vm.n.notes);
        match cur {
            Some(Value::List(l)) => l.borrow_mut().push(a[1].clone()),
            Some(_) => return Err(vm.type_error("Cannot add notes to a non-list __notes__")),
            None => i.set_attr(vm.n.notes.clone(), Value::list(vec![a[1].clone()])),
        }
    }
    Ok(Value::None)
}

fn exc_reduce(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let cls = vm.class_of(&a[0]);
    let mut args = vm.exc_args(&a[0]);
    let Value::Instance(inst) = &a[0] else {
        return Ok(Value::tuple(vec![Value::Class(cls), Value::tuple(args)]));
    };
    let dict = inst.with_attrs(Dict::copy);
    let mut state = if dict.is_empty() { None } else { Some(dict) };
    if cls.is_subclass(&vm.exc_class("OSError")) {
        let f = exc_field(inst, "filename");
        if args.len() == 2 && !f.is_none() {
            args.push(f);
            let f2 = exc_field(inst, "filename2");
            if !f2.is_none() {
                args.push(Value::None);
                args.push(f2);
            }
        }
    } else if cls.is_subclass(&vm.exc_class("ImportError")) {
        let mut d = state.take().unwrap_or_else(Dict::new);
        for n in ["name", "path", "name_from"] {
            let v = exc_field(inst, n);
            if !v.is_none() {
                d.set_s(n, v);
            }
        }
        state = if d.is_empty() { None } else { Some(d) };
    } else if cls.is_subclass(&vm.exc_class("AttributeError")) {
        let mut d = state.take().unwrap_or_else(Dict::new);
        let n = exc_field(inst, "name");
        if !n.is_none() {
            d.set_s("name", n);
        }
        d.set_s("args", Value::tuple(args.clone()));
        state = Some(d);
    }
    let mut out = vec![Value::Class(cls), Value::tuple(args)];
    if let Some(d) = state {
        out.push(Value::dict(d));
    }
    Ok(Value::tuple(out))
}

fn exc_setstate(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    margs(vm, "__setstate__", a, 1, 1)?;
    match &a[1] {
        Value::None => {}
        Value::Dict(d) => {
            let items = d.borrow().items();
            for (k, v) in items {
                let k = if let Value::Instance(_) = &k { vm.base_of(&k) } else { k };
                match &k {
                    Value::Str(s) => vm.setattr(&a[0], s, v)?,
                    _ => return Err(vm.type_error("attribute name must be string")),
                }
            }
        }
        _ => return Err(vm.type_error("state is not a dictionary")),
    }
    Ok(Value::None)
}

// -------------------------------------------------------------------------
// the types

/// registers a native method on a class; `name` is "type.method"
pub fn def(c: &Rc<Class>, name: &'static str, f: NativeFnPtr) {
    let short = name.rsplit('.').next().unwrap();
    c.dict.borrow_mut().set_s(short, Value::Native(Rc::new(NativeFn::new(name, f, Value::None))));
}

pub fn def_static(c: &Rc<Class>, name: &'static str, f: NativeFnPtr) {
    let short = name.rsplit('.').next().unwrap();
    c.dict.borrow_mut().set_s(short, Value::StaticMethod(Rc::new(Value::Native(Rc::new(NativeFn::new(name, f, Value::None))))));
}

pub fn def_class(c: &Rc<Class>, name: &'static str, f: NativeFnPtr) {
    let short = name.rsplit('.').next().unwrap();
    c.dict.borrow_mut().set_s(short, Value::ClassMethod(Rc::new(Value::Native(Rc::new(NativeFn::new(name, f, Value::None))))));
}

/// a method that takes lo..=hi positional arguments besides self
pub fn def_arity(c: &Rc<Class>, name: &'static str, f: NativeFnPtr, lo: u8, hi: u8) {
    let short = name.rsplit('.').next().unwrap();
    let mut n = NativeFn::new(name, f, Value::None);
    n.arity = Some((lo, hi));
    c.dict.borrow_mut().set_s(short, Value::Native(Rc::new(n)));
}

pub fn native(name: &'static str, f: NativeFnPtr) -> Value {
    Value::Native(Rc::new(NativeFn { name, f, data: Value::None, bind: false, arity: None }))
}

fn mk(name: &str, base: Option<&Rc<Class>>, b: Builtin) -> Rc<Class> {
    let bases = base.map(|x| vec![x.clone()]).unwrap_or_default();
    let c = Class::new(name, bases, b);
    let mut mro = vec![c.clone()];
    if let Some(x) = base {
        mro.extend(x.mro.borrow().iter().cloned());
    }
    c.set_mro(mro);
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
        obj.set_mro(vec![obj.clone(), object.clone()]);
    }
    let iterator = mk("iterator", o, Builtin::Iterator);
    Types {
        enumerate: mk("enumerate", Some(&iterator), Builtin::Iterator),
        zip: mk("zip", Some(&iterator), Builtin::Iterator),
        map: mk("map", Some(&iterator), Builtin::Iterator),
        filter: mk("filter", Some(&iterator), Builtin::Iterator),
        reversed: mk("reversed", Some(&iterator), Builtin::Iterator),
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
        wrapper_descriptor: mk("wrapper_descriptor", o, Builtin::BuiltinFunction),
        method_wrapper: mk("method-wrapper", o, Builtin::BuiltinFunction),
        method: mk("method", o, Builtin::Method),
        module: mk("module", o, Builtin::Module),
        iterator,
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
    def(&be, "BaseException.__setstate__", exc_setstate);

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
        ("enumerate", &t.enumerate),
        ("zip", &t.zip),
        ("map", &t.map),
        ("filter", &t.filter),
        ("reversed", &t.reversed),
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
        ("breakpoint", crate::modules::bi_breakpoint),
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
    vm.write_out(s);
    Ok(())
}

fn bi_print(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let mut sep = ps(" ");
    let mut end = ps("\n");
    let mut file = Value::None;
    for (key, v) in k {
        match &*key.s {
            "sep" => {
                if !v.is_none() {
                    sep = match v {
                        Value::Str(s) => s.clone(),
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
                        Value::Str(s) => s.clone(),
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
    let mut s = crate::wtf8::Wtf8Buf::default();
    for (i, x) in a.iter().enumerate() {
        if i > 0 {
            s.push_pystr(&sep);
        }
        match x {
            Value::Str(t) => s.push_pystr(t),
            Value::Int(n) => s.push_str(&n.to_string()),
            _ => s.push_str(&vm.str_of(x)?),
        }
    }
    s.push_pystr(&end);
    let Value::Str(s) = s.into_value() else { unreachable!() };
    if file.is_none() || crate::modules::is_std_stream(&file) {
        crate::strmethods::check_utf8(vm, &s)?;
        if crate::modules::is_stderr(&file) {
            vm.write_err(&s.s);
            return Ok(Value::None);
        }
        write_stdout(vm, &s.s)?;
    } else {
        let w = vm.getattr(&file, &ps("write"))?;
        vm.call_vec(&w, vec![Value::Str(s)], &[])?;
    }
    Ok(Value::None)
}

fn bi_len(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "len", a, 1, 1)?;
    Ok(Value::Int(vm.len(&a[0])? as i64))
}

/// what a user-defined __repr__ or __str__ returns, kept as the str
/// subclass it may be
pub fn user_str_result(vm: &mut Vm, v: &Value, dunder: &str) -> PyResult<Option<Value>> {
    if !matches!(v, Value::Instance(_)) {
        return Ok(None);
    }
    let f = match vm.lookup_special(v, &ps(dunder)) {
        Some(f @ Value::Method(_)) if matches!(&f, Value::Method(m) if matches!(m.1, Value::Func(_))) => f,
        _ => return Ok(None),
    };
    let r = vm.call_vec(&f, vec![], &[])?;
    if matches!(r, Value::Str(_)) || (matches!(r, Value::Instance(_)) && matches!(vm.base_of(&r), Value::Str(_))) {
        return Ok(Some(r));
    }
    let rn = vm.type_name(&r);
    Err(vm.type_error(format!("{} returned non-string (type {})", dunder, rn)))
}

fn bi_repr(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "repr", a, 1, 1)?;
    if let Some(r) = user_str_result(vm, &a[0], "__repr__")? {
        return Ok(r);
    }
    Ok(Value::string(vm.repr(&a[0])?))
}

fn bi_ascii(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "ascii", a, 1, 1)?;
    if let Some(r) = user_str_result(vm, &a[0], "__repr__")? {
        if let Value::Str(t) = vm.base_of(&r) {
            if t.ascii {
                return Ok(r);
            }
            return Ok(Value::string(crate::format::ascii_escape(&t.s)));
        }
    }
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
            if matches!(v, Value::Instance(_)) && !matches!(b, Value::Instance(_)) {
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
        v => match vm.to_index(v) {
            Ok(i) => i,
            Err(e) if vm.exc_is(&e, "OverflowError") => -1,
            Err(e) => return Err(e),
        },
    };
    match u32::try_from(i) {
        Ok(cp) if cp <= 0x10ffff => Ok(Value::from_cp(cp)),
        _ => Err(vm.value_error("chr() arg not in range(0x110000)")),
    }
}

fn bi_ord(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "ord", a, 1, 1)?;
    match &vm.base_of(&a[0]) {
        Value::Str(s) => {
            let mut cs = s.code_points();
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
        // module.__dir__ for a module subclass: the keys of its __dict__ only
        Value::Instance(i) if i.class.borrow().is_subclass(&vm.t.module) => match vm.getattr(v, &ps("__dict__"))? {
            Value::Dict(d) => add(&mut names, &d.borrow()),
            _ => {
                let mn = match vm.getattr(v, &ps("__name__")) {
                    Ok(n) => vm.str_of(&n)?,
                    Err(_) => "?".into(),
                };
                return Err(vm.type_error(format!("{mn}.__dict__ is not a dictionary")));
            }
        },
        Value::Instance(_) => {
            // object.__dir__: the instance's __dict__ and its __class__'s
            // attributes, each skipped when the lookup fails
            match vm.getattr(v, &ps("__dict__")) {
                Ok(Value::Dict(d)) => add(&mut names, &d.borrow()),
                Ok(_) => {}
                Err(e) if vm.exc_is(&e, "AttributeError") => {}
                Err(e) => return Err(e),
            }
            match vm.getattr(v, &ps("__class__")) {
                Ok(Value::Class(c)) => {
                    for k in c.mro.borrow().iter() {
                        add(&mut names, &k.dict.borrow());
                    }
                }
                Ok(_) => {}
                Err(e) if vm.exc_is(&e, "AttributeError") => {}
                Err(e) => return Err(e),
            }
        }
        // traceback.__dir__ lists only its four attributes
        Value::Obj(o) if matches!(&*o.kind.borrow(), ObjKind::Traceback { .. }) => {
            names.extend(["tb_frame", "tb_lasti", "tb_lineno", "tb_next"].map(String::from));
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
    if let Some(l) = a.is_empty().then(|| vm.current_frame().and_then(|f| f.ext.as_ref()).and_then(|x| x.locals.clone())).flatten() {
        let km = vm.getattr(&l, &ps("keys"))?;
        let keys = vm.call(&km, &[])?;
        let v = vm.to_vec(&keys)?;
        let r = Value::list(v);
        let sm = vm.getattr(&r, &ps("sort"))?;
        vm.call(&sm, &[])?;
        return Ok(r);
    }
    if a.is_empty() {
        let mut names: Vec<String> = match vm.current_frame() {
            Some(f) => frame_locals(f).borrow().keys().iter().filter_map(|k| k.as_str().map(|s| s.to_string())).collect(),
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
        let (x, y) = (vm.type_name(&a[0]), vm.type_name(&a[1]));
        return Err(vm.type_error(format!("unsupported operand type(s) for divmod(): '{}' and '{}'", x, y)));
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

pub fn bi_enumerate(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    if a.len() + k.len() > 2 {
        return Err(vm.type_error(format!("enumerate() takes at most 2 arguments ({} given)", a.len() + k.len())));
    }
    for (name, _) in k {
        if &*name.s != "iterable" && &*name.s != "start" {
            return Err(vm.type_error(format!("'{}' is an invalid keyword argument for enumerate()", name.s)));
        }
    }
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
    Ok(mk_iter(Iter::Enumerate(inner, start, None)))
}

pub fn bi_zip(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    if let Some((name, _)) = k.iter().find(|(n, _)| &*n.s != "strict") {
        return Err(vm.type_error(format!("zip() got an unexpected keyword argument '{}'", name.s)));
    }
    let strict = match kwarg(k, "strict") {
        Some(v) => vm.truthy(&v)?,
        None => false,
    };
    let mut its = Vec::with_capacity(a.len());
    for (i, x) in a.iter().enumerate() {
        let _ = i;
        its.push(vm.iter(x)?);
    }
    Ok(mk_iter(Iter::Zip(its, strict, None)))
}

impl Exc {
    fn with_note(self, _i: usize) -> Exc {
        self
    }
}

pub fn bi_map(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    if a.len() < 2 {
        return Err(vm.type_error("map() must have at least two arguments."));
    }
    let mut its = vec![];
    for x in &a[1..] {
        its.push(vm.iter(x)?);
    }
    Ok(mk_iter(Iter::Map(a[0].clone(), its)))
}

pub fn bi_filter(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "filter", a, 2, 2)?;
    let it = vm.iter(&a[1])?;
    Ok(mk_iter(Iter::Filter(a[0].clone(), it, false)))
}

pub fn bi_reversed(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    if !k.is_empty() {
        return Err(vm.type_error("reversed() takes no keyword arguments"));
    }
    check_nargs(vm, "reversed", a, 1, 1)?;
    let v = &a[0];
    match v {
        Value::List(l) => {
            let n = l.borrow().len();
            Ok(mk_iter(Iter::ListRev(l.clone(), n)))
        }
        Value::Tuple(t) => Ok(mk_iter(Iter::Vec(t.iter().rev().cloned().collect::<Vec<_>>().into_iter()))),
        Value::Str(s) => Ok(mk_iter(Iter::Vec(s.code_points().rev().map(Value::from_cp).collect::<Vec<_>>().into_iter()))),
        Value::Range(r) => {
            let n = range_len(r) as i128;
            let last = r.0 as i128 + (n - 1).max(0) * r.2 as i128;
            Ok(mk_iter(crate::range::iter_i128(last, -(r.2 as i128), n)))
        }
        Value::BigRange(r) => Ok(mk_iter(crate::range::reversed(r))),
        Value::Bytes(b) => Ok(mk_iter(Iter::Vec(b.iter().rev().map(|&x| Value::Int(x as i64)).collect::<Vec<_>>().into_iter()))),
        Value::Dict(d) => Ok(dict_rev_iter(d, 0)),
        Value::DictView(dv) => Ok(dict_rev_iter(&dv.0, dv.1)),
        _ => {
            if let Some(f) = vm.lookup_special(v, &vm.n.reversed.clone()) {
                let is_native = matches!(&f, Value::Method(m) if matches!(m.1, Value::Native(_)));
                if !is_native || matches!(v, Value::Obj(_)) {
                    return vm.call_vec(&f, vec![], &[]);
                }
            }
            let b = vm.base_of(v);
            if matches!(v, Value::Instance(_)) && !matches!(b, Value::Instance(_) | Value::Obj(_)) {
                return bi_reversed(vm, &[b], &[]);
            }
            if let Value::Obj(o) = v {
                if let ObjKind::Deque(d, _) = &*o.kind.borrow() {
                    return Ok(mk_iter(Iter::Vec(d.iter().rev().cloned().collect::<Vec<_>>().into_iter())));
                }
            }
            if vm.lookup_special(v, &vm.n.len.clone()).is_some() && vm.lookup_special(v, &vm.n.getitem.clone()).is_some() {
                let n = vm.len(v)?;
                return Ok(mk_iter(Iter::SeqRev(v.clone(), n as i64 - 1)));
            }
            let tn = vm.type_name(v);
            Err(vm.type_error(format!("'{}' object is not reversible", tn)))
        }
    }
}

fn bi_iter(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "iter", a, 1, 2)?;
    if a.len() == 2 {
        if !is_callable(vm, &a[0]) {
            return Err(vm.type_error("iter(v, w): v must be callable"));
        }
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
        Some(Value::Instance(i)) if matches!(&*i.base.borrow(), Value::Str(_)) => i.base.borrow().as_str().unwrap().to_string(),
        Some(v) => {
            let tn = vm.type_name(v);
            return Err(vm.type_error(format!("format() argument 2 must be str, not {}", tn)));
        }
        None => String::new(),
    };
    vm.format_to_str(&a[0], &spec)
}

fn bi_hash(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "hash", a, 1, 1)?;
    let h = vm.hash(&a[0])?;
    Ok(Value::Int(h as i64))
}

fn bi_id(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "id", a, 1, 1)?;
    Ok(Value::Int(ptr_id(&a[0]) as i64 & 0x7fff_ffff_ffff))
}

/// input(): the prompt to sys.stdout, then a line from sys.stdin.readline()
fn bi_input(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "input", a, 0, 1)?;
    let sys = vm.modules.borrow().get_s("sys");
    let stream = |name: &str| match &sys {
        Some(Value::Module(m)) => m.dict.borrow().get_s(name).filter(|v| !v.is_none()),
        _ => None,
    };
    let (fin, fout) = (stream("stdin"), stream("stdout"));
    let Some(fin) = fin else {
        return Err(vm.err("RuntimeError", "input(): lost sys.stdin"));
    };
    let Some(fout) = fout else {
        return Err(vm.err("RuntimeError", "input(): lost sys.stdout"));
    };
    if let Some(p) = a.first() {
        let s = vm.str_of(p)?;
        write_stdout(vm, &s)?;
    }
    // a failing flush is ignored, as CPython does
    if let Ok(f) = vm.getattr(&fout, &ps("flush")) {
        let _ = vm.call_vec(&f, vec![], &[]);
    }
    let rl = vm.getattr(&fin, &ps("readline"))?;
    let line = vm.call_vec(&rl, vec![], &[])?;
    let Value::Str(line) = line else {
        return Err(vm.type_error("object.readline() returned non-string"));
    };
    match line.s.strip_suffix('\n') {
        _ if line.s.is_empty() => Err(vm.err("EOFError", "EOF when reading a line")),
        Some(l) => Ok(Value::str(l)),
        None => Ok(Value::Str(line.clone())),
    }
}

/// isinstance(v, spec) with tuples, unions and __instancecheck__
pub fn isinstance_of(vm: &mut Vm, v: &Value, spec: &Value) -> PyResult<bool> {
    isinstance_nested(vm, v, spec, 0)
}

/// isinstance_of within `nest` levels of tuples, which CPython walks in C and
/// so bounds by its C recursion limit
fn isinstance_nested(vm: &mut Vm, v: &Value, spec: &Value, nest: usize) -> PyResult<bool> {
    if let Value::Class(c) = spec {
        if Rc::ptr_eq(&vm.class_of(v), c) {
            return Ok(true);
        }
        let plain = match &*c.meta.borrow() {
            None => true,
            Some(m) => Rc::ptr_eq(m, &vm.t.type_),
        };
        if plain {
            return real_isinstance(vm, v, spec);
        }
    }
    if let Some(args) = union_args(vm, spec)? {
        return isinstance_nested(vm, v, &args, nest);
    }
    if let Value::Tuple(t) = spec {
        if nest >= crate::format::C_RECURSION_LIMIT {
            return Err(vm.err("RecursionError", "maximum recursion depth exceeded in __instancecheck__"));
        }
        vm.enter(" in __instancecheck__")?;
        let mut r = Ok(false);
        for x in t.iter() {
            match isinstance_nested(vm, v, x, nest + 1) {
                Ok(false) => {}
                other => {
                    r = other;
                    break;
                }
            }
        }
        vm.depth -= 1;
        return r;
    }
    let name = vm.n.instancecheck.clone();
    match vm.lookup_special_get(spec, &name)? {
        Some(Value::Method(m)) if matches!(&m.1, Value::Native(n) if n.name == "type.__instancecheck__") => real_isinstance(vm, v, spec),
        Some(f) => {
            vm.enter(" in __instancecheck__")?;
            let r = vm.call_vec(&f, vec![v.clone()], &[]);
            vm.depth -= 1;
            let r = r?;
            vm.truthy(&r)
        }
        None => real_isinstance(vm, v, spec),
    }
}

/// type.__instancecheck__: isinstance without the hook on the class
pub fn real_isinstance(vm: &mut Vm, v: &Value, spec: &Value) -> PyResult<bool> {
    let name = vm.n.class.clone();
    if let Value::Class(c) = spec {
        if vm.isinstance(v, c) {
            return Ok(true);
        }
        // an instance whose class overrides __class__ (mock objects)
        if let Value::Instance(i) = v {
            let own = i.class.borrow().clone();
            if own.lookup(&name).is_some() {
                if let Some(Value::Class(ic)) = get_optional(vm, v, &name)? {
                    if !Rc::ptr_eq(&ic, &own) {
                        return Ok(ic.is_subclass(c));
                    }
                }
            }
        }
        return Ok(false);
    }
    check_class(vm, spec, "isinstance() arg 2 must be a type, a tuple of types, or a union")?;
    match get_optional(vm, v, &name)? {
        Some(ic) => abstract_issubclass(vm, &ic, spec),
        None => Ok(false),
    }
}

/// getattr that turns AttributeError into None
fn get_optional(vm: &mut Vm, v: &Value, name: &Rc<PyStr>) -> PyResult<Option<Value>> {
    match vm.getattr_opt(v, name) {
        Ok(r) => Ok(r),
        Err(e) if vm.exc_is(&e, "AttributeError") => Ok(None),
        Err(e) => Err(e),
    }
}

fn abstract_get_bases(vm: &mut Vm, v: &Value) -> PyResult<Option<Rc<Vec<Value>>>> {
    if let Value::Class(c) = v {
        let b: Vec<Value> = c.bases.borrow().iter().map(|b| Value::Class(b.clone())).collect();
        return Ok(Some(Rc::new(b)));
    }
    let name = PyStr::new("__bases__".to_string());
    match get_optional(vm, v, &Rc::new(name))? {
        Some(Value::Tuple(t)) => Ok(Some(t)),
        _ => Ok(None),
    }
}

fn check_class(vm: &mut Vm, v: &Value, msg: &str) -> PyResult<()> {
    match abstract_get_bases(vm, v)? {
        Some(_) => Ok(()),
        None => Err(vm.type_error(msg)),
    }
}

fn abstract_issubclass(vm: &mut Vm, derived: &Value, cls: &Value) -> PyResult<bool> {
    let mut derived = derived.clone();
    let bases = loop {
        if derived.is(cls) {
            return Ok(true);
        }
        let bases = match abstract_get_bases(vm, &derived)? {
            Some(b) => b,
            None => return Ok(false),
        };
        match bases.len() {
            0 => return Ok(false),
            1 => derived = bases[0].clone(),
            _ => break bases,
        }
    };
    vm.enter(" in __subclasscheck__")?;
    let mut r = Ok(false);
    for b in bases.iter() {
        match abstract_issubclass(vm, b, cls) {
            Ok(false) => {}
            other => {
                r = other;
                break;
            }
        }
    }
    vm.depth -= 1;
    r
}

/// type.__subclasscheck__: issubclass without the hook on the class
pub fn real_issubclass(vm: &mut Vm, c: &Value, spec: &Value) -> PyResult<bool> {
    if let (Value::Class(a), Value::Class(b)) = (c, spec) {
        return Ok(a.is_subclass(b));
    }
    check_class(vm, c, "issubclass() arg 1 must be a class")?;
    if union_args(vm, spec)?.is_none() {
        check_class(vm, spec, "issubclass() arg 2 must be a class, a tuple of classes, or a union")?;
    }
    abstract_issubclass(vm, c, spec)
}

fn bi_isinstance(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "isinstance", a, 2, 2)?;
    Ok(Value::Bool(isinstance_of(vm, &a[0], &a[1])?))
}

pub fn issubclass_of(vm: &mut Vm, c: &Value, spec: &Value) -> PyResult<bool> {
    issubclass_nested(vm, c, spec, 0)
}

/// issubclass_of within `nest` levels of tuples, bounded like isinstance_nested
fn issubclass_nested(vm: &mut Vm, c: &Value, spec: &Value, nest: usize) -> PyResult<bool> {
    if let Value::Class(s) = spec {
        let plain = match &*s.meta.borrow() {
            None => true,
            Some(m) => Rc::ptr_eq(m, &vm.t.type_),
        };
        if plain {
            if c.is(spec) {
                return Ok(true);
            }
            return real_issubclass(vm, c, spec);
        }
    }
    if let Some(args) = union_args(vm, spec)? {
        return issubclass_nested(vm, c, &args, nest);
    }
    if let Value::Tuple(t) = spec {
        if nest >= crate::format::C_RECURSION_LIMIT {
            return Err(vm.err("RecursionError", "maximum recursion depth exceeded in __subclasscheck__"));
        }
        vm.enter(" in __subclasscheck__")?;
        let mut r = Ok(false);
        for x in t.iter() {
            match issubclass_nested(vm, c, x, nest + 1) {
                Ok(false) => {}
                other => {
                    r = other;
                    break;
                }
            }
        }
        vm.depth -= 1;
        return r;
    }
    let name = vm.n.subclasscheck.clone();
    match vm.lookup_special_get(spec, &name)? {
        Some(Value::Method(m)) if matches!(&m.1, Value::Native(n) if n.name == "type.__subclasscheck__") => real_issubclass(vm, c, spec),
        Some(f) => {
            vm.enter(" in __subclasscheck__")?;
            let r = vm.call_vec(&f, vec![c.clone()], &[]);
            vm.depth -= 1;
            let r = r?;
            vm.truthy(&r)
        }
        None => real_issubclass(vm, c, spec),
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
    if let Value::Bytes(_) = start {
        return Err(vm.type_error("sum() can't sum bytes [use b''.join(seq) instead]"));
    }
    if let Value::ByteArray(_) = start {
        return Err(vm.type_error("sum() can't sum bytearray [use b''.join(seq) instead]"));
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
            acc = Value::Float(if c != 0.0 && c.is_finite() { s + c } else { s });
        }
        acc = vm.binop(BinOp::Add, &acc, &x)?;
    }
    if let Some((s, c)) = fsum {
        // like CPython, the compensation is only added when it is non-zero
        return Ok(Value::Float(if c != 0.0 && c.is_finite() { s + c } else { s }));
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
        // code point order, lone surrogates too
        let sk: Vec<&[u8]> = keys.iter().filter_map(|k| if let Value::Str(s) = k { Some(s.wtf8()) } else { None }).collect();
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
    // work modulo |m| so the remainders stay non-negative, then take the
    // result's sign from m
    let am = m.abs();
    let (mut old_r, mut r) = (a.divmod(&am).unwrap().1, am);
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
/// round(x, nd) as CPython: correctly rounded, ties to even; None when the
/// result overflows
pub fn round_float(x: f64, nd: i64) -> Option<f64> {
    if !x.is_finite() || nd > 323 {
        return Some(x);
    }
    if nd < -308 {
        return Some(0.0f64.copysign(x));
    }
    let r: f64 = if nd >= 0 {
        // the formatter rounds the exact binary value, ties to even
        format!("{:.*}", nd as usize, x).parse().unwrap_or(x)
    } else {
        // exactly: x = num/den, q = round(x / 10**k), result q * 10**k
        let bits = x.abs();
        let (num, den) = float_ratio_exact(bits);
        let p = BigInt::from_i64(10).pow((-nd) as u64);
        let d = den.mul(&p);
        let (q, rem) = num.divmod(&d).unwrap();
        let q = match rem.shl(1).cmp(&d) {
            std::cmp::Ordering::Greater => q.add(&BigInt::from_i64(1)),
            std::cmp::Ordering::Equal if q.is_odd() => q.add(&BigInt::from_i64(1)),
            _ => q,
        };
        format!("{}e{}", q, -nd).parse::<f64>().unwrap_or(f64::INFINITY)
    };
    if r.is_infinite() {
        return None;
    }
    Some(if r == 0.0 || nd < 0 { r.copysign(x) } else { r })
}

/// a finite non-negative float as an exact fraction
fn float_ratio_exact(f: f64) -> (BigInt, BigInt) {
    let bits = f.to_bits();
    let exp = ((bits >> 52) & 0x7ff) as i64;
    let mant = if exp == 0 { (bits & ((1 << 52) - 1)) << 1 } else { (bits & ((1 << 52) - 1)) | (1 << 52) };
    let e = exp - 1075;
    let num = BigInt::from_i64(mant as i64);
    if e >= 0 {
        (num.shl(e as usize), BigInt::from_i64(1))
    } else {
        (num, BigInt::from_i64(1).shl((-e) as usize))
    }
}

/// an index, clamped to the i64 range when it is larger
fn index_clamped(vm: &mut Vm, v: &Value) -> PyResult<i64> {
    if let Value::Big(b) = vm.base_of(v) {
        return Ok(if b.neg { i64::MIN } else { i64::MAX });
    }
    vm.to_index(v)
}

fn bi_round(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    check_nargs(vm, "round", a, 0, 2)?;
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
            let n = index_clamped(vm, &nd)?;
            match round_float(*f, n) {
                Some(r) => Ok(Value::Float(r)),
                None => Err(vm.err("OverflowError", "rounded value too large to represent")),
            }
        }
        Value::Int(_) | Value::Bool(_) | Value::Big(_) => {
            if nd.is_none() {
                return Ok(match &x {
                    Value::Bool(b) => Value::Int(*b as i64),
                    o => o.clone(),
                });
            }
            let n = index_clamped(vm, &nd)?;
            if n >= 0 {
                return Ok(match &x {
                    Value::Bool(b) => Value::Int(*b as i64),
                    o => o.clone(),
                });
            }
            let b = crate::ops::big_of(&x).unwrap();
            // more digits rounded off than there are: zero
            if (-(n as i128)) as u128 > b.bit_length() as u128 / 3 + 2 {
                return Ok(Value::Int(0));
            }
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
    let code = &f.code;
    if let Some(ns) = &f.ns {
        // the slots of a module or class body are those of the comprehensions
        // inlined there: while one runs, a copy of the namespace with them
        let slots = code.varnames.iter().zip(f.stack.iter().cloned());
        let cells = code.cellvars.iter().zip(f.cells.iter()).filter(|(n, _)| !n.starts_with("__class")).map(|(n, c)| (n, c.borrow().clone()));
        let bound: Vec<(&Name, Value)> = slots.chain(cells).filter(|(_, v)| !v.is_undef()).collect();
        if !bound.is_empty() {
            let mut d = ns.borrow().copy();
            for (n, v) in bound {
                d.set_s(n, v);
            }
            return Rc::new(RefCell::new(d));
        }
        return ns.clone();
    }
    let mut d = Dict::new();
    // a local that is a cell has both a slot and a cell; the cell holds it
    for (i, n) in code.varnames.iter().enumerate() {
        if let Some(ci) = code.cell_from_arg.iter().position(|s| *s == Some(i as u32)) {
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
        if code.cell_from_arg.get(ci).is_some_and(Option::is_some) {
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
    if let Some(l) = vm.current_frame().and_then(|f| f.ext.as_ref()).and_then(|x| x.locals.clone()) {
        return Ok(l);
    }
    match vm.current_frame() {
        Some(f) => Ok(Value::Dict(frame_locals(f))),
        None => Ok(Value::Dict(vm.main_globals.clone())),
    }
}

fn bi_globals(vm: &mut Vm, _a: &[Value], _k: Kw) -> PyResult<Value> {
    if let Some(g) = vm.current_frame().and_then(|f| f.ext.as_ref()).and_then(|x| x.globals.clone()) {
        return Ok(g);
    }
    match vm.current_frame() {
        Some(f) => Ok(Value::Dict(f.globals.clone())),
        None => Ok(Value::Dict(vm.main_globals.clone())),
    }
}

fn bi_vars(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    check_nargs(vm, "vars", a, 0, 1)?;
    if a.is_empty() {
        return bi_locals(vm, a, k);
    }
    match vm.getattr_opt(&a[0], &vm.n.dict.clone())? {
        Some(d) => Ok(d),
        None => Err(vm.type_error("vars() argument must have __dict__ attribute")),
    }
}

type Scopes = (Rc<RefCell<Dict>>, Rc<RefCell<Dict>>, Option<Rc<crate::vm::FrameExt>>);

fn eval_scopes(vm: &mut Vm, a: &[Value], k: Kw, what: &str) -> PyResult<Scopes> {
    let (cg, cl, cext) = match vm.current_frame() {
        Some(f) => (f.globals.clone(), frame_locals(f), f.ext.clone()),
        None => (vm.main_globals.clone(), vm.main_globals.clone(), None),
    };
    let ga = arg_or_kw(a, 1, k, "globals").filter(|v| !v.is_none());
    let la = arg_or_kw(a, 2, k, "locals").filter(|v| !v.is_none());
    let mut gobj = None;
    let g = match &ga {
        Some(Value::Dict(d)) => d.clone(),
        Some(v @ Value::Instance(i)) if matches!(&*i.base.borrow(), Value::Dict(_)) => {
            gobj = Some(v.clone());
            match &*i.base.borrow() {
                Value::Dict(d) => d.clone(),
                _ => unreachable!(),
            }
        }
        Some(v) => {
            let tn = vm.type_name(v);
            let msg = if what == "exec" { format!("exec() globals must be a dict, not {}", tn) } else { format!("globals must be a dict, not {}", tn) };
            return Err(vm.type_error(msg));
        }
        None => cg.clone(),
    };
    let mut lobj = None;
    let l = match &la {
        Some(Value::Dict(d)) => d.clone(),
        Some(v) => {
            let has = match v {
                Value::Instance(_) => vm.getattr_opt(v, &ps("__getitem__"))?.is_some(),
                Value::Obj(_) => vm.getattr_opt(v, &ps("__getitem__"))?.is_some(),
                _ => false,
            };
            if !has {
                let tn = vm.type_name(v);
                return Err(vm.type_error(format!("locals must be a mapping, not {}", tn)));
            }
            lobj = Some(v.clone());
            match v {
                Value::Instance(i) => match &*i.base.borrow() {
                    Value::Dict(d) => d.clone(),
                    _ => Rc::new(RefCell::new(Dict::new())),
                },
                _ => Rc::new(RefCell::new(Dict::new())),
            }
        }
        None if ga.is_some() => {
            lobj = gobj.clone();
            g.clone()
        }
        None => {
            if let Some(x) = &cext {
                lobj = x.locals.clone();
            }
            cl
        }
    };
    if ga.is_none() {
        if let Some(x) = &cext {
            gobj = x.globals.clone();
        }
    }
    let bv = g.borrow().get_s("__builtins__");
    let bobj = match bv {
        None => {
            // exec() gives globals without builtins the builtins' dict, as CPython does
            g.borrow_mut().set_s("__builtins__", Value::Dict(vm.builtins.clone()));
            None
        }
        Some(Value::Module(m)) if Rc::ptr_eq(&m.dict, &vm.builtins) => None,
        Some(Value::Dict(d)) if Rc::ptr_eq(&d, &vm.builtins) => None,
        Some(Value::Module(m)) => Some(Value::Dict(m.dict.clone())),
        Some(v) => Some(v),
    };
    let ext = if gobj.is_some() || lobj.is_some() || bobj.is_some() {
        Some(Rc::new(crate::vm::FrameExt { globals: gobj, locals: lobj, builtins: bobj }))
    } else {
        None
    };
    Ok((g, l, ext))
}

fn source_of(vm: &mut Vm, v: &Value, what: &str) -> PyResult<Option<String>> {
    match v {
        Value::Str(s) => Ok(Some(s.s.clone())),
        Value::Bytes(b) => decode_source(vm, b).map(Some),
        Value::Code(_) => Ok(None),
        Value::ByteArray(_) | Value::Obj(_) | Value::Instance(_) => match bytes_of(vm, v) {
            Ok(b) => source_of(vm, &Value::Bytes(Rc::new(b)), what),
            Err(_) => Err(vm.type_error(format!("{}() arg 1 must be a string, bytes or code object", what))),
        },
        _ => Err(vm.type_error(format!("{}() arg 1 must be a string, bytes or code object", what))),
    }
}

/// source code from bytes as PEP 263 reads it: UTF-8, or the encoding a
/// coding comment on one of the first two lines names
fn decode_source(vm: &mut Vm, b: &[u8]) -> PyResult<String> {
    let (bom, b) = match b.strip_prefix(b"\xef\xbb\xbf".as_slice()) {
        Some(rest) => (true, rest),
        None => (false, b),
    };
    if let Some(enc) = coding_cookie(b) {
        // CPython's get_normal_name
        let low = enc.to_ascii_lowercase().replace('_', "-");
        let utf8 = low == "utf-8" || low.starts_with("utf-8-");
        if bom && !utf8 {
            let e = vm.err("SyntaxError", format!("encoding problem: {} with BOM", enc));
            exc_set_field(&e.value, "lineno", Value::Int(0));
            exc_set_field(&e.value, "offset", Value::Int(-1));
            return Err(e);
        }
        if !utf8 {
            return Ok(crate::strmethods::decode_bytes(vm, b, &enc, "strict")?.into_string_lossy());
        }
    }
    match std::str::from_utf8(b) {
        Ok(s) => Ok(s.to_string()),
        Err(e) => Err(undecodable_source(vm, b, e.valid_up_to())),
    }
}

/// A source file's text as PEP 263 reads it. Bytes that are not UTF-8 in a
/// file without a coding comment are the error CPython's tokenizer gives.
pub fn decode_file_source(vm: &mut Vm, b: &[u8], file: &str) -> PyResult<String> {
    let body = b.strip_prefix(b"\xef\xbb\xbf".as_slice()).unwrap_or(b);
    if coding_cookie(body).is_none() {
        if let Err(e) = std::str::from_utf8(body) {
            let bad = e.valid_up_to();
            let line = body[..bad].iter().filter(|c| **c == b'\n').count() + 1;
            let msg = format!(
                "Non-UTF-8 code starting with '\\x{:02x}' in file {} on line {}, but no encoding declared; see https://peps.python.org/pep-0263/ for details",
                body[bad], file, line
            );
            return Err(vm.err("SyntaxError", msg));
        }
    }
    decode_source(vm, b)
}

/// a module's source as the import system reads it: compile() of its bytes
pub fn decode_module_source(vm: &mut Vm, b: &[u8], file: &str) -> PyResult<String> {
    decode_source(vm, b).map_err(|e| {
        if let Value::Instance(i) = &e.value {
            if matches!(exc_field(i, "filename"), Value::Str(f) if &*f.s == "<string>") {
                exc_set_field(&e.value, "filename", Value::str(file));
            }
        }
        e
    })
}

/// the encoding a `# coding: name` comment names on the first line, or on the
/// second after a comment or blank first line
fn coding_cookie(b: &[u8]) -> Option<String> {
    let mut lines = b.split(|c| *c == b'\n');
    let cookie = |line: &[u8]| -> Option<String> {
        let line = std::str::from_utf8(line).ok()?.trim_start_matches([' ', '\t', '\x0c']);
        let comment = line.strip_prefix('#')?;
        let at = comment.find("coding")?;
        let rest = comment[at + 6..].strip_prefix([':', '='])?.trim_start_matches([' ', '\t']);
        let name: String = rest.chars().take_while(|c| c.is_alphanumeric() || matches!(c, '-' | '_' | '.')).collect();
        (!name.is_empty()).then_some(name)
    };
    let first = lines.next()?;
    if let Some(c) = cookie(first) {
        return Some(c);
    }
    let blank_or_comment = |l: &[u8]| {
        let t = String::from_utf8_lossy(l);
        let t = t.trim_start_matches([' ', '\t', '\x0c', '\r']);
        t.is_empty() || t.starts_with('#')
    };
    if blank_or_comment(first) {
        return lines.next().and_then(cookie);
    }
    None
}

/// source that is not UTF-8 from byte `bad` on: when that is inside a string
/// literal, the error CPython gives decoding the literal, there
fn undecodable_source(vm: &mut Vm, b: &[u8], bad: usize) -> Exc {
    let prefix = std::str::from_utf8(&b[..bad]).unwrap_or_default();
    let toks = crate::lexer::tokenize(prefix);
    let open = toks.iter().rev().find_map(|t| match &t.tok {
        crate::lexer::Tok::Error(m) if m.starts_with("unterminated") => Some((t.line, t.col)),
        _ => None,
    });
    let Some((line, col)) = open else {
        return vm.err("SyntaxError", "source code cannot be decoded as UTF-8");
    };
    // the literal's body: after its prefix letters and quotes, up to its closing quote
    let line_start: usize = prefix.split_inclusive('\n').take(line as usize - 1).map(str::len).sum();
    let start = line_start + prefix[line_start..].chars().take(col as usize - 1).map(char::len_utf8).sum::<usize>();
    let mut i = start;
    while i < b.len() && b[i].is_ascii_alphabetic() {
        i += 1;
    }
    let quote = b.get(i).copied().unwrap_or(b'"');
    let n = if b[i..].starts_with(&[quote; 3]) { 3 } else { 1 };
    let body = i + n;
    let end = b[body..].windows(n).position(|w| w.iter().all(|c| *c == quote)).map_or(b.len(), |k| body + k);
    let msg = match crate::strmethods::decode_bytes(vm, &b[body..end], "utf-8", "strict") {
        Err(e) => crate::builtins::exc_str(vm, &e.value).unwrap_or_default(),
        Ok(_) => return vm.err("SyntaxError", "source code cannot be decoded as UTF-8"),
    };
    let e = vm.syntax_error(&format!("(unicode error) {}", msg), line, "<string>");
    exc_set_field(&e.value, "offset", Value::Int(col as i64));
    e
}

fn bi_eval(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    check_nargs(vm, "eval", a, 1, 3)?;
    let (g, l, ext) = eval_scopes(vm, a, k, "eval")?;
    let code = match source_of(vm, &a[0], "eval")? {
        Some(src) => {
            let src = src.trim_start_matches([' ', '\t']).to_string();
            vm.compile_source(&src, "<string>", true)?
        }
        None => match &a[0] {
            Value::Code(c) => {
                if !c.freevars.is_empty() {
                    return Err(vm.type_error("code object passed to eval() may not contain free variables"));
                }
                c.clone()
            }
            _ => unreachable!(),
        },
    };
    vm.run_code_ext(code, g, l, ext)
}

fn bi_exec(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    check_nargs(vm, "exec", a, 1, 3)?;
    let closure = kwarg(k, "closure").filter(|v| !v.is_none());
    let k: Vec<_> = k.iter().filter(|(n, _)| &*n.s != "closure").cloned().collect();
    let k = &k[..];
    let (g, l, ext) = eval_scopes(vm, a, k, "exec")?;
    if let Value::Code(c) = &a[0] {
        if closure.is_some() || !c.freevars.is_empty() {
            let cells = exec_closure(vm, c, closure)?;
            vm.run_code_closure(c.clone(), g, l, ext, cells)?;
            return Ok(Value::None);
        }
    } else if closure.is_some() {
        return Err(vm.type_error("closure can only be used when source is a code object"));
    }
    let code = match source_of(vm, &a[0], "exec")? {
        Some(src) => {
            vm.compile_source(&src, "<string>", false)?
        }
        None => match &a[0] {
            Value::Code(c) => {
                if !c.freevars.is_empty() {
                    return Err(vm.type_error("code object passed to exec() may not contain free variables"));
                }
                c.clone()
            }
            _ => unreachable!(),
        },
    };
    vm.run_code_ext(code, g, l, ext)?;
    Ok(Value::None)
}

/// checks exec()'s `closure=` against the free variables of `code`
fn exec_closure(vm: &mut Vm, code: &crate::code::Code, closure: Option<Value>) -> PyResult<Vec<Cellv>> {
    let n = code.freevars.len();
    let Some(closure) = closure else {
        return Err(vm.type_error(format!("code object requires a closure of exactly length {n}")));
    };
    if n == 0 {
        return Err(vm.type_error("cannot use a closure with this code object"));
    }
    let Value::Tuple(items) = &closure else {
        return Err(vm.type_error("closure must be a tuple of cells"));
    };
    if items.len() != n {
        return Err(vm.type_error(format!("code object requires a closure of exactly length {n}")));
    }
    items
        .iter()
        .map(|c| match c {
            Value::Cell(c) => Ok(c.clone()),
            _ => Err(vm.type_error("closure can only contain cells")),
        })
        .collect()
}

fn bi_compile(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    use crate::pyast::{PYCF_ALLOW_TOP_LEVEL_AWAIT, PYCF_COMPILE_MASK, PYCF_ONLY_AST};
    const PARAMS: [&str; 7] = ["source", "filename", "mode", "flags", "dont_inherit", "optimize", "_feature_version"];
    check_nargs(vm, "compile", a, 0, 6)?;
    for (n, _) in k {
        match PARAMS.iter().position(|p| *p == &*n.s) {
            Some(i) if i < a.len() => {
                return Err(vm.type_error(format!("argument for compile() given by name ('{}') and position ({})", n.s, i + 1)));
            }
            Some(_) => {}
            None => return Err(vm.type_error(format!("compile() got an unexpected keyword argument '{}'", n.s))),
        }
    }
    for (i, p) in PARAMS[..3].iter().enumerate() {
        if arg_or_kw(a, i, k, p).is_none() {
            return Err(vm.type_error(format!("compile() missing required argument '{}' (pos {})", p, i + 1)));
        }
    }
    let src = arg_or_kw(a, 0, k, "source").unwrap_or(Value::None);
    let file = arg_or_kw(a, 1, k, "filename").map(|v| v.as_str().unwrap_or("<string>").to_string()).unwrap_or("<string>".into());
    let mode = arg_or_kw(a, 2, k, "mode").map(|v| v.as_str().unwrap_or("exec").to_string()).unwrap_or("exec".into());
    let flags = match arg_or_kw(a, 3, k, "flags") {
        Some(f) => vm.to_index(&f)?,
        None => 0,
    };
    let optimize = match arg_or_kw(a, 5, k, "optimize") {
        Some(o) => vm.to_index(&o)?,
        None => -1,
    };
    if flags & !PYCF_COMPILE_MASK != 0 {
        return Err(vm.value_error("compile(): unrecognised flags"));
    }
    if !(-1..=2).contains(&optimize) {
        return Err(vm.value_error("compile(): invalid optimize value"));
    }
    if !matches!(&*mode, "exec" | "eval" | "single" | "func_type") {
        return Err(vm.value_error("compile() mode must be 'exec', 'eval' or 'single'"));
    }
    // -1 is the interpreter's own level, which is 0
    let optimize = optimize.max(0) as u8;
    let top_await = flags & PYCF_ALLOW_TOP_LEVEL_AWAIT != 0;
    if crate::pyast::is_ast(vm, &src)? {
        return crate::pyast::compile(vm, &src, &file, &mode, flags, optimize);
    }
    let s = match source_of(vm, &src, "compile")? {
        Some(s) => s,
        None => return Ok(src),
    };
    let s = if mode == "eval" { s.trim() } else { &s };
    if flags & PYCF_ONLY_AST != 0 {
        return crate::pyast::parse(vm, s, &file, &mode, flags);
    }
    let code = vm.from_source(s, &file, || {
        if mode == "eval" {
            crate::parser::parse_expression(s).map(|e| crate::compile::compile_eval_opt(&e, &file, optimize, top_await))
        } else {
            crate::parser::parse_module(s).map(|b| crate::compile::compile_module_opt(b, &file, optimize, top_await))
        }
    })?;
    Ok(Value::Code(code))
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
    const PARAMS: [&str; 5] = ["name", "globals", "locals", "fromlist", "level"];
    check_nargs(vm, "__import__", a, 0, 5)?;
    for (n, _) in k {
        match PARAMS.iter().position(|p| *p == &*n.s) {
            Some(i) if i < a.len() => {
                return Err(vm.type_error(format!("argument for __import__() given by name ('{}') and position ({})", n.s, i + 1)));
            }
            Some(_) => {}
            None => return Err(vm.type_error(format!("__import__() got an unexpected keyword argument '{}'", n.s))),
        }
    }
    let name = match arg_or_kw(a, 0, k, "name") {
        Some(Value::Str(s)) => s.s.clone(),
        Some(v) => return Err(vm.type_error(format!("module name must be str, not {}", vm.type_name(&v)))),
        None => return Err(vm.type_error("__import__() missing required argument 'name' (pos 1)")),
    };
    let fromlist = arg_or_kw(a, 3, k, "fromlist").unwrap_or(Value::None);
    let level = match arg_or_kw(a, 4, k, "level") {
        Some(v) => vm.to_index(&v)?,
        None => 0,
    };
    if level < 0 {
        return Err(vm.value_error("level must be >= 0"));
    }
    if name.is_empty() && level == 0 {
        return Err(vm.value_error("Empty module name"));
    }
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
    let Some(f) = vm.lookup_special(&a[0], &ps("__anext__")) else {
        let tn = vm.type_name(&a[0]);
        return Err(vm.type_error(format!("'{tn}' object is not an async iterator")));
    };
    let r = vm.call_vec(&f, vec![], &[])?;
    match a.get(1) {
        // with a default: pylib/_asyncgen.py's wrapper of the awaitable
        Some(d) => {
            let g = vm.main_globals.clone();
            let m = vm.import("_asyncgen", &Value::None, 0, &g)?;
            let w = vm.getattr(&m, &ps("_ANextDefault"))?;
            vm.call_vec(&w, vec![r, d.clone()], &[])
        }
        None => Ok(r),
    }
}

/// a native method's __get__: bound to the instance, or itself from the class
fn descr_get(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    need(vm, a, 2)?;
    match (&a[1], a.get(2)) {
        (Value::None, None | Some(Value::None)) => Err(vm.type_error("__get__(None, None) is invalid")),
        (Value::None, Some(_)) => Ok(a[0].clone()),
        (obj, _) => Ok(Value::Method(Rc::new((obj.clone(), a[0].clone())))),
    }
}

static DOCS: &str = include_str!("docs.txt");

/// CPython's __text_signature__ ("" when it has none) and __doc__ of a
/// built-in function, type or method ("type.method"), from tools/docgen.py
pub fn builtin_doc(name: &str) -> Option<(&'static str, &'static str)> {
    let key = format!("{}\x1f", name);
    let at = if DOCS.starts_with(&key) { 0 } else { DOCS.find(&format!("\x1e{}", key))? + 1 };
    let rest = &DOCS[at + key.len()..];
    let (sig, rest) = rest.split_once('\x1f')?;
    Some((sig, &rest[..rest.find('\x1e')?]))
}

/// the module a native function belongs to, from its dotted name
pub fn native_module_name(name: &str) -> &str {
    match name.rsplit_once('.') {
        Some((m, _)) if matches!(m, "math" | "sys" | "_time" | "os" | "posixpath" | "json" | "_struct" | "_hashlib" | "_abc" | "_statistics") => m,
        _ => "builtins",
    }
}

/// natives that read a[n-1] check first that it is there
pub fn need(vm: &mut Vm, a: &[Value], n: usize) -> PyResult<()> {
    if a.len() < n {
        let got = a.len().saturating_sub(1);
        let want = n - 1;
        return Err(vm.type_error(format!("expected {} argument{}, got {}", want, if want == 1 { "" } else { "s" }, got)));
    }
    Ok(())
}
