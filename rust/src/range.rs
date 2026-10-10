// SPDX-License-Identifier: MIT
//! range objects past i64: a range whose bounds or length do not fit a
//! machine word keeps them as big integers. Small ranges stay
//! `Value::Range` and take the i64 paths elsewhere.

use crate::bigint::BigInt;
use crate::value::*;
use crate::vm::Vm;
use std::cmp::Ordering;
use std::rc::Rc;

pub struct BigRange {
    pub start: BigInt,
    pub stop: BigInt,
    pub step: BigInt,
    pub len: BigInt,
}

fn bi(i: i64) -> BigInt {
    BigInt::from_i64(i)
}

fn is_neg(b: &BigInt) -> bool {
    b.neg && !b.is_zero()
}

pub fn big_len(start: &BigInt, stop: &BigInt, step: &BigInt) -> BigInt {
    let (lo, hi, st) = if is_neg(step) { (stop, start, step.neg()) } else { (start, stop, step.clone()) };
    if lo.cmp(hi) != Ordering::Less {
        return BigInt::zero();
    }
    match hi.sub(lo).sub(&bi(1)).divmod(&st) {
        Some((q, _)) => q.add(&bi(1)),
        None => BigInt::zero(),
    }
}

/// a range value, small when everything fits i64
pub fn make(start: BigInt, stop: BigInt, step: BigInt) -> Value {
    let len = big_len(&start, &stop, &step);
    if let (Some(a), Some(b), Some(c), Some(_)) = (start.to_i64(), stop.to_i64(), step.to_i64(), len.to_i64()) {
        return Value::Range(Rc::new((a, b, c)));
    }
    Value::BigRange(Rc::new(BigRange { start, stop, step, len }))
}

pub fn make_i128(start: i128, stop: i128, step: i128) -> Value {
    make(BigInt::from_i128(start), BigInt::from_i128(stop), BigInt::from_i128(step))
}

/// start, stop, step and length of either kind of range
pub fn parts(v: &Value) -> Option<(BigInt, BigInt, BigInt, BigInt)> {
    match v {
        Value::Range(r) => Some((bi(r.0), bi(r.1), bi(r.2), bi(crate::builtins::range_len(r) as i64))),
        Value::BigRange(r) => Some((r.start.clone(), r.stop.clone(), r.step.clone(), r.len.clone())),
        _ => None,
    }
}

pub fn len(vm: &mut Vm, r: &BigRange) -> PyResult<usize> {
    match r.len.to_i64() {
        Some(n) => Ok(n as usize),
        None => Err(vm.err("OverflowError", "Python int too large to convert to C ssize_t")),
    }
}

pub fn getitem(vm: &mut Vm, r: &BigRange, idx: &Value) -> PyResult<Value> {
    if let Value::Slice(sl) = idx {
        let (a, b, st) = slice_indices(vm, sl, &r.len)?;
        let start = r.start.add(&a.mul(&r.step));
        let stop = r.start.add(&b.mul(&r.step));
        return Ok(make(start, stop, r.step.mul(&st)));
    }
    let mut i = match idx {
        Value::Int(_) | Value::Bool(_) | Value::Big(_) | Value::Instance(_) => crate::mathmod::index_big(vm, idx).map_err(|e| {
            if vm.exc_is(&e, "TypeError") {
                let tn = vm.type_name(idx);
                vm.type_error(format!("range indices must be integers or slices, not {}", tn))
            } else {
                e
            }
        })?,
        _ => {
            let tn = vm.type_name(idx);
            return Err(vm.type_error(format!("range indices must be integers or slices, not {}", tn)));
        }
    };
    if is_neg(&i) {
        i = i.add(&r.len);
    }
    if is_neg(&i) || i.cmp(&r.len) != Ordering::Less {
        return Err(vm.index_error("range object index out of range"));
    }
    Ok(Value::from_big(r.start.add(&i.mul(&r.step))))
}

/// slice.indices over a length that may not fit i64
pub fn slice_indices(vm: &mut Vm, sl: &(Value, Value, Value), len: &BigInt) -> PyResult<(BigInt, BigInt, BigInt)> {
    let step = if sl.2.is_none() { bi(1) } else { crate::mathmod::index_big(vm, &sl.2)? };
    if step.is_zero() {
        return Err(vm.value_error("slice step cannot be zero"));
    }
    let neg = is_neg(&step);
    let (lower, upper) = if neg { (bi(-1), len.sub(&bi(1))) } else { (bi(0), len.clone()) };
    let clamp = |vm: &mut Vm, v: &Value, dflt: &BigInt| -> PyResult<BigInt> {
        if v.is_none() {
            return Ok(dflt.clone());
        }
        let mut s = crate::mathmod::index_big(vm, v)?;
        if is_neg(&s) {
            s = s.add(len);
            if s.cmp(&lower) == Ordering::Less {
                s = lower.clone();
            }
        } else if s.cmp(&upper) == Ordering::Greater {
            s = upper.clone();
        }
        Ok(s)
    };
    let (ds, de) = if neg { (upper.clone(), lower.clone()) } else { (lower.clone(), upper.clone()) };
    let start = clamp(vm, &sl.0, &ds)?;
    let stop = clamp(vm, &sl.1, &de)?;
    Ok((start, stop, step))
}

/// the position of the int `x` in the range, if it is an element
pub fn position(r: &(BigInt, BigInt, BigInt, BigInt), x: &BigInt) -> Option<BigInt> {
    let (start, _, step, len) = r;
    let (q, m) = x.sub(start).divmod(step)?;
    if !m.is_zero() || is_neg(&q) || q.cmp(len) != Ordering::Less {
        return None;
    }
    Some(q)
}

pub fn eq(a: &Value, b: &Value) -> bool {
    match (parts(a), parts(b)) {
        (Some(x), Some(y)) => {
            if x.3 != y.3 {
                return false;
            }
            if x.3.is_zero() {
                return true;
            }
            x.0 == y.0 && (x.3 == bi(1) || x.2 == y.2)
        }
        _ => false,
    }
}

/// hash((len, start, step)), with start and step None where they do not
/// matter to equality
pub fn hash(v: &Value) -> u64 {
    let (start, _, step, len) = match parts(v) {
        Some(p) => p,
        None => return 0,
    };
    let t = if len.is_zero() {
        vec![Value::Int(0), Value::None, Value::None]
    } else if len == bi(1) {
        vec![Value::Int(1), Value::from_big(start), Value::None]
    } else {
        vec![Value::from_big(len), Value::from_big(start), Value::from_big(step)]
    };
    fast_hash(&Value::tuple(t)).unwrap_or(0)
}

/// an iterator over `len` values from `start`: the i64 one when its
/// values and the stop it pickles with fit, the big one otherwise
pub fn iter_of(start: BigInt, step: BigInt, len: BigInt) -> Iter {
    let stop = start.add(&len.mul(&step));
    if let (Some(a), Some(c), Some(n), Some(_)) = (start.to_i64(), step.to_i64(), len.to_i64(), stop.to_i64()) {
        return Iter::Range(a, n, c);
    }
    Iter::BigRange(Box::new((start, step, len)))
}

pub fn iter_i128(start: i128, step: i128, len: i128) -> Iter {
    let fits = |x: i128| x >= i64::MIN as i128 && x <= i64::MAX as i128;
    if fits(start) && fits(step) && fits(len) && fits(start + len * step) {
        return Iter::Range(start as i64, len as i64, step as i64);
    }
    iter_of(BigInt::from_i128(start), BigInt::from_i128(step), BigInt::from_i128(len))
}

pub fn iter(r: &BigRange) -> Iter {
    iter_of(r.start.clone(), r.step.clone(), r.len.clone())
}

pub fn reversed(r: &BigRange) -> Iter {
    let last = if r.len.is_zero() { r.start.clone() } else { r.start.add(&r.len.sub(&bi(1)).mul(&r.step)) };
    iter_of(last, r.step.neg(), r.len.clone())
}

/// the index of `item` in a range, or with `count` how many items equal it
pub fn find(vm: &mut Vm, r: &Value, item: &Value, count: bool) -> PyResult<Option<BigInt>> {
    let p = match parts(r) {
        Some(p) => p,
        None => return Ok(None),
    };
    if let Some(x) = crate::ops::big_of(item) {
        let pos = position(&p, &x);
        if count {
            return Ok(Some(bi(pos.is_some() as i64)));
        }
        return Ok(pos);
    }
    let it = Value::Iter(Rc::new(std::cell::RefCell::new(iter_of(p.0, p.2, p.3))));
    let (mut i, mut n) = (0i64, 0i64);
    while let Some(x) = vm.next(&it)? {
        if vm.eq(&x, item)? {
            if !count {
                return Ok(Some(bi(i)));
            }
            n += 1;
        }
        i += 1;
    }
    Ok(if count { Some(bi(n)) } else { None })
}

/// the next value of a big range iterator: (current, step, remaining)
pub fn iter_next(st: &mut (BigInt, BigInt, BigInt)) -> Option<Value> {
    if st.2.is_zero() || is_neg(&st.2) {
        return None;
    }
    let v = Value::from_big(st.0.clone());
    st.0 = st.0.add(&st.1);
    st.2 = st.2.sub(&bi(1));
    Some(v)
}

pub fn items(r: &BigRange) -> Vec<Value> {
    let mut st = (r.start.clone(), r.step.clone(), r.len.clone());
    let mut out = vec![];
    while let Some(v) = iter_next(&mut st) {
        out.push(v);
    }
    out
}

pub fn repr(v: &Value) -> String {
    match parts(v) {
        Some((a, b, c, _)) if c == bi(1) => format!("range({}, {})", a, b),
        Some((a, b, c, _)) => format!("range({}, {}, {})", a, b, c),
        None => String::new(),
    }
}
