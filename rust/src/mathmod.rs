// SPDX-License-Identifier: MIT
//! The math module, following CPython's Modules/mathmodule.c: errno-style
//! error mapping, exactly rounded fsum/hypot/sumprod, its gamma and lgamma.

use crate::ast::BinOp;
use crate::bigint::BigInt;
use crate::builtins::*;
use crate::modules::{module, put_fns};
use crate::value::*;
use crate::vm::Vm;
use std::cmp::Ordering;

mod c {
    extern "C" {
        pub fn erf(x: f64) -> f64;
        pub fn erfc(x: f64) -> f64;
        pub fn asinh(x: f64) -> f64;
        pub fn acosh(x: f64) -> f64;
        pub fn atanh(x: f64) -> f64;
    }
}

pub fn erf(x: f64) -> f64 {
    unsafe { c::erf(x) }
}
pub fn erfc(x: f64) -> f64 {
    unsafe { c::erfc(x) }
}

pub fn math_module() -> Value {
    let (d, m) = module("math");
    {
        let mut b = d.borrow_mut();
        b.set_s("pi", Value::Float(std::f64::consts::PI));
        b.set_s("e", Value::Float(std::f64::consts::E));
        b.set_s("tau", Value::Float(std::f64::consts::TAU));
        b.set_s("inf", Value::Float(f64::INFINITY));
        b.set_s("nan", Value::Float(f64::NAN));
    }
    put_fns(
        &d,
        &[
            ("math.sqrt", m_sqrt),
            ("math.isqrt", m_isqrt),
            ("math.floor", m_floor),
            ("math.ceil", m_ceil),
            ("math.trunc", m_trunc),
            ("math.fabs", m_fabs),
            ("math.pow", m_pow),
            ("math.exp", m_exp),
            ("math.exp2", m_exp2),
            ("math.expm1", m_expm1),
            ("math.log", m_log),
            ("math.log2", m_log2),
            ("math.log10", m_log10),
            ("math.log1p", m_log1p),
            ("math.sin", m_sin),
            ("math.cos", m_cos),
            ("math.tan", m_tan),
            ("math.asin", m_asin),
            ("math.acos", m_acos),
            ("math.atan", m_atan),
            ("math.atan2", m_atan2),
            ("math.sinh", m_sinh),
            ("math.cosh", m_cosh),
            ("math.tanh", m_tanh),
            ("math.asinh", m_asinh),
            ("math.acosh", m_acosh),
            ("math.atanh", m_atanh),
            ("math.degrees", m_degrees),
            ("math.radians", m_radians),
            ("math.hypot", m_hypot),
            ("math.dist", m_dist),
            ("math.gcd", m_gcd),
            ("math.lcm", m_lcm),
            ("math.factorial", m_factorial),
            ("math.comb", m_comb),
            ("math.perm", m_perm),
            ("math.prod", m_prod),
            ("math.fsum", m_fsum),
            ("math.sumprod", m_sumprod),
            ("math.copysign", m_copysign),
            ("math.isclose", m_isclose),
            ("math.isfinite", m_isfinite),
            ("math.isinf", m_isinf),
            ("math.isnan", m_isnan),
            ("math.modf", m_modf),
            ("math.frexp", m_frexp),
            ("math.ldexp", m_ldexp),
            ("math.fmod", m_fmod),
            ("math.remainder", m_remainder),
            ("math.erf", m_erf),
            ("math.erfc", m_erfc),
            ("math.gamma", m_gamma),
            ("math.lgamma", m_lgamma),
            ("math.nextafter", m_nextafter),
            ("math.ulp", m_ulp),
            ("math.cbrt", m_cbrt),
            ("math.fma", m_fma),
        ],
    );
    m
}

// -------------------------------------------------------------------------
// argument helpers

fn nokw(vm: &mut Vm, name: &str, k: Kw) -> PyResult<()> {
    if !k.is_empty() {
        return Err(vm.type_error(format!("{}() takes no keyword arguments", name)));
    }
    Ok(())
}

/// positional-only arguments, no keywords
fn args(vm: &mut Vm, name: &str, a: &[Value], k: Kw, min: usize, max: usize) -> PyResult<()> {
    nokw(vm, name, k)?;
    check_nargs(vm, name, a, min, max)
}

/// a special method looked up on the type; descriptors get `__get__` called
fn special(vm: &mut Vm, obj: &Value, name: &str) -> PyResult<Option<Value>> {
    let cls = vm.class_of(obj);
    let v = match cls.lookup_s(name) {
        Some(v) => v,
        None => return Ok(None),
    };
    if let Value::Instance(_) = &v {
        let vc = vm.class_of(&v);
        if let Some(g) = vc.lookup_s("__get__") {
            let g = vm.bind(g, &v, &vc);
            return vm.call_vec(&g, vec![obj.clone(), Value::Class(cls)], &[]).map(Some);
        }
    }
    Ok(Some(vm.bind(v, obj, &cls)))
}

fn big_to_f64(vm: &mut Vm, b: &BigInt) -> PyResult<f64> {
    let f = b.to_f64();
    if f.is_infinite() {
        return Err(vm.err("OverflowError", "int too large to convert to float"));
    }
    Ok(f)
}

fn user_defined(v: &Option<Value>) -> bool {
    matches!(v, Some(Value::Func(_)) | Some(Value::Instance(_)) | Some(Value::Method(_)))
}

/// PyFloat_AsDouble: float, int, __float__ (which must return a float), __index__
fn as_double(vm: &mut Vm, v: &Value) -> PyResult<f64> {
    match v {
        Value::Float(f) => Ok(*f),
        Value::Int(i) => Ok(*i as f64),
        Value::Bool(b) => Ok(*b as i64 as f64),
        Value::Big(b) => big_to_f64(vm, b),
        Value::Instance(_) => {
            let base = vm.base_of(v);
            let cls = vm.class_of(v);
            let own = cls.lookup_s("__float__");
            if let Value::Float(f) = base {
                if !user_defined(&own) {
                    return Ok(f);
                }
            }
            if own.is_some() {
                if let Some(m) = special(vm, v, "__float__")? {
                    let r = vm.call_vec(&m, vec![], &[])?;
                    return match vm.base_of(&r) {
                        Value::Float(f) => Ok(f),
                        _ => {
                            let (tn, rn) = (vm.type_name(v), vm.type_name(&r));
                            Err(vm.type_error(format!("{}.__float__ returned non-float (type {})", tn, rn)))
                        }
                    };
                }
            }
            if !matches!(base, Value::Instance(_)) && !matches!(base, Value::Float(_)) {
                if let Some(b) = crate::ops::big_of(&base) {
                    return big_to_f64(vm, &b);
                }
            }
            if cls.lookup_s("__index__").is_some() {
                let b = index_big(vm, v)?;
                return big_to_f64(vm, &b);
            }
            let tn = vm.type_name(v);
            Err(vm.type_error(format!("must be real number, not {}", tn)))
        }
        _ => {
            let tn = vm.type_name(v);
            Err(vm.type_error(format!("must be real number, not {}", tn)))
        }
    }
}

/// PyNumber_Index as a big integer
fn index_big(vm: &mut Vm, v: &Value) -> PyResult<BigInt> {
    if let Some(b) = crate::ops::big_of(&vm.base_of(v)) {
        return Ok(b);
    }
    if let Some(m) = special(vm, v, "__index__")? {
        let r = vm.call_vec(&m, vec![], &[])?;
        return match crate::ops::big_of(&vm.base_of(&r)) {
            Some(b) => Ok(b),
            None => {
                let tn = vm.type_name(&r);
                Err(vm.type_error(format!("__index__ returned non-int (type {})", tn)))
            }
        };
    }
    let tn = vm.type_name(v);
    Err(vm.type_error(format!("'{}' object cannot be interpreted as an integer", tn)))
}

fn is_int(vm: &Vm, v: &Value) -> bool {
    matches!(vm.base_of(v), Value::Int(_) | Value::Big(_) | Value::Bool(_))
}

// -------------------------------------------------------------------------
// errno-style error mapping (math_1, math_2, is_error)

#[derive(Clone, Copy, PartialEq)]
enum Errno {
    Ok,
    Dom,
    Range,
}

fn domain(vm: &mut Vm) -> Exc {
    vm.value_error("math domain error")
}

fn range_err(vm: &mut Vm) -> Exc {
    vm.err("OverflowError", "math range error")
}

/// is_error: ERANGE on a result under 1.5 is an underflow and not an error
fn result(vm: &mut Vm, r: f64, e: Errno) -> PyResult<Value> {
    match e {
        Errno::Dom => Err(domain(vm)),
        Errno::Range if r.abs() >= 1.5 => Err(range_err(vm)),
        _ => Ok(Value::Float(r)),
    }
}

fn math_1(vm: &mut Vm, name: &str, a: &[Value], k: Kw, f: fn(f64) -> f64, can_overflow: bool) -> PyResult<Value> {
    args(vm, name, a, k, 1, 1)?;
    let x = as_double(vm, &a[0])?;
    let r = f(x);
    if r.is_nan() && !x.is_nan() {
        return Err(domain(vm));
    }
    if r.is_infinite() && x.is_finite() {
        return Err(if can_overflow { range_err(vm) } else { domain(vm) });
    }
    Ok(Value::Float(r))
}

/// math_1a: the function reports its own errno
fn math_1a(vm: &mut Vm, name: &str, a: &[Value], k: Kw, f: fn(f64) -> (f64, Errno)) -> PyResult<Value> {
    args(vm, name, a, k, 1, 1)?;
    let x = as_double(vm, &a[0])?;
    let (r, e) = f(x);
    result(vm, r, e)
}

fn two(vm: &mut Vm, name: &str, a: &[Value], k: Kw) -> PyResult<(f64, f64)> {
    nokw(vm, name, k)?;
    if a.len() != 2 {
        return Err(vm.type_error(format!("{} expected 2 arguments, got {}", name, a.len())));
    }
    let x = as_double(vm, &a[0])?;
    let y = as_double(vm, &a[1])?;
    Ok((x, y))
}

fn math_2(vm: &mut Vm, name: &str, a: &[Value], k: Kw, f: fn(f64, f64) -> f64) -> PyResult<Value> {
    let (x, y) = two(vm, name, a, k)?;
    let r = f(x, y);
    let e = if r.is_nan() && !x.is_nan() && !y.is_nan() {
        Errno::Dom
    } else if r.is_infinite() && x.is_finite() && y.is_finite() {
        Errno::Range
    } else {
        Errno::Ok
    };
    result(vm, r, e)
}

macro_rules! math1 {
    ($( $name:ident, $py:literal, $f:expr, $ov:expr );* $(;)?) => {
        $(
            fn $name(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
                math_1(vm, $py, a, k, $f, $ov)
            }
        )*
    };
}

math1! {
    m_acos, "acos", f64::acos, false;
    m_acosh, "acosh", |x| unsafe { c::acosh(x) }, false;
    m_asin, "asin", f64::asin, false;
    m_asinh, "asinh", |x| unsafe { c::asinh(x) }, false;
    m_atan, "atan", f64::atan, false;
    m_atanh, "atanh", |x| unsafe { c::atanh(x) }, false;
    m_cbrt, "cbrt", f64::cbrt, false;
    m_cos, "cos", f64::cos, false;
    m_cosh, "cosh", f64::cosh, true;
    m_exp, "exp", f64::exp, true;
    m_exp2, "exp2", f64::exp2, true;
    m_expm1, "expm1", f64::exp_m1, true;
    m_fabs, "fabs", f64::abs, false;
    m_log1p, "log1p", |x| if x == 0.0 { x } else { x.ln_1p() }, false;
    m_sin, "sin", f64::sin, false;
    m_sinh, "sinh", f64::sinh, true;
    m_sqrt, "sqrt", f64::sqrt, false;
    m_tan, "tan", f64::tan, false;
    m_tanh, "tanh", f64::tanh, false;
}

fn m_erf(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    math_1a(vm, "erf", a, k, |x| (erf(x), Errno::Ok))
}
fn m_erfc(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    math_1a(vm, "erfc", a, k, |x| (erfc(x), Errno::Ok))
}
fn m_gamma(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    math_1a(vm, "gamma", a, k, tgamma)
}
fn m_lgamma(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    math_1a(vm, "lgamma", a, k, lgamma)
}

fn atan2(y: f64, x: f64) -> f64 {
    use std::f64::consts::PI;
    if x.is_nan() || y.is_nan() {
        return f64::NAN;
    }
    if y.is_infinite() {
        if x.is_infinite() {
            return if x.is_sign_positive() { (0.25 * PI).copysign(y) } else { (0.75 * PI).copysign(y) };
        }
        return (0.5 * PI).copysign(y);
    }
    if x.is_infinite() || y == 0.0 {
        return if x.is_sign_positive() { 0.0f64.copysign(y) } else { PI.copysign(y) };
    }
    y.atan2(x)
}

/// IEEE 754 remainder: x - n*y with n the nearest integer, ties to even; exact
fn remainder(x: f64, y: f64) -> f64 {
    if x.is_finite() && y.is_finite() {
        if y == 0.0 {
            return f64::NAN;
        }
        let absx = x.abs();
        let absy = y.abs();
        let m = absx % absy;
        let c = absy - m;
        let r = if m < c {
            m
        } else if m > c {
            -c
        } else {
            m - 2.0 * ((0.5 * (absx - m)) % absy)
        };
        return 1.0f64.copysign(x) * r;
    }
    if x.is_nan() {
        return x;
    }
    if y.is_nan() {
        return y;
    }
    if x.is_infinite() {
        return f64::NAN;
    }
    x
}

fn m_atan2(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    math_2(vm, "atan2", a, k, atan2)
}
fn m_copysign(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    math_2(vm, "copysign", a, k, f64::copysign)
}
fn m_remainder(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    math_2(vm, "remainder", a, k, remainder)
}

fn m_fmod(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let (x, y) = two(vm, "fmod", a, k)?;
    if y.is_infinite() && x.is_finite() {
        return Ok(Value::Float(x));
    }
    let r = x % y;
    if r.is_nan() && !x.is_nan() && !y.is_nan() {
        return Err(domain(vm));
    }
    Ok(Value::Float(r))
}

fn m_pow(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let (x, y) = two(vm, "pow", a, k)?;
    let mut e = Errno::Ok;
    let r = if !x.is_finite() || !y.is_finite() {
        if x.is_nan() {
            if y == 0.0 {
                1.0
            } else {
                x
            }
        } else if y.is_nan() {
            if x == 1.0 {
                1.0
            } else {
                y
            }
        } else if x.is_infinite() {
            let odd_y = y.is_finite() && (y.abs() % 2.0) == 1.0;
            if y > 0.0 {
                if odd_y {
                    x
                } else {
                    x.abs()
                }
            } else if y == 0.0 {
                1.0
            } else if odd_y {
                0.0f64.copysign(x)
            } else {
                0.0
            }
        } else if x.abs() == 1.0 {
            1.0
        } else if y > 0.0 && x.abs() > 1.0 {
            y
        } else if y < 0.0 && x.abs() < 1.0 {
            -y
        } else {
            0.0
        }
    } else {
        let r = x.powf(y);
        if r.is_nan() {
            e = Errno::Dom;
        } else if r.is_infinite() {
            e = if x == 0.0 { Errno::Dom } else { Errno::Range };
        }
        r
    };
    result(vm, r, e)
}

fn m_degrees(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    args(vm, "degrees", a, k, 1, 1)?;
    let x = as_double(vm, &a[0])?;
    Ok(Value::Float(x * (180.0 / std::f64::consts::PI)))
}
fn m_radians(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    args(vm, "radians", a, k, 1, 1)?;
    let x = as_double(vm, &a[0])?;
    Ok(Value::Float(x * (std::f64::consts::PI / 180.0)))
}

fn m_fma(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    args(vm, "fma", a, k, 3, 3)?;
    let x = as_double(vm, &a[0])?;
    let y = as_double(vm, &a[1])?;
    let z = as_double(vm, &a[2])?;
    let r = x.mul_add(y, z);
    if !r.is_finite() {
        if r.is_nan() {
            if !x.is_nan() && !y.is_nan() && !z.is_nan() {
                return Err(vm.value_error("invalid operation in fma"));
            }
        } else if x.is_finite() && y.is_finite() && z.is_finite() {
            return Err(vm.err("OverflowError", "overflow in fma"));
        }
    }
    Ok(Value::Float(r))
}

// -------------------------------------------------------------------------
// predicates, frexp/ldexp/modf, nextafter, ulp

fn one_f(vm: &mut Vm, name: &str, a: &[Value], k: Kw) -> PyResult<f64> {
    args(vm, name, a, k, 1, 1)?;
    as_double(vm, &a[0])
}

fn m_isfinite(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    Ok(Value::Bool(one_f(vm, "isfinite", a, k)?.is_finite()))
}
fn m_isinf(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    Ok(Value::Bool(one_f(vm, "isinf", a, k)?.is_infinite()))
}
fn m_isnan(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    Ok(Value::Bool(one_f(vm, "isnan", a, k)?.is_nan()))
}

fn m_modf(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let x = one_f(vm, "modf", a, k)?;
    let (f, i) = if x.is_infinite() {
        (0.0f64.copysign(x), x)
    } else if x.is_nan() {
        (x, x)
    } else {
        let i = x.trunc();
        ((x - i).copysign(x), i)
    };
    Ok(Value::tuple(vec![Value::Float(f), Value::Float(i)]))
}

/// x * 2**n with a single rounding (musl's scalbn)
pub fn scalbn(mut x: f64, mut n: i64) -> f64 {
    let p1023 = f64::from_bits(0x7fe0_0000_0000_0000);
    let pm969 = f64::from_bits(0x0360_0000_0000_0000);
    if n > 1023 {
        x *= p1023;
        n -= 1023;
        if n > 1023 {
            x *= p1023;
            n -= 1023;
            if n > 1023 {
                n = 1023;
            }
        }
    } else if n < -1022 {
        x *= pm969;
        n += 1022 - 53;
        if n < -1022 {
            x *= pm969;
            n += 1022 - 53;
            if n < -1022 {
                n = -1022;
            }
        }
    }
    x * f64::from_bits(((0x3ff + n) as u64) << 52)
}

/// mantissa in [0.5, 1) and exponent; zeros and non-finite values unchanged
pub fn frexp(x: f64) -> (f64, i64) {
    if x == 0.0 || !x.is_finite() {
        return (x, 0);
    }
    let (x, adj) = if x.abs() < f64::MIN_POSITIVE { (x * f64::from_bits(0x4350_0000_0000_0000), -54) } else { (x, 0) };
    let bits = x.to_bits();
    let e = ((bits >> 52) & 0x7ff) as i64 - 1022;
    let m = f64::from_bits((bits & !(0x7ffu64 << 52)) | (1022u64 << 52));
    (m, e + adj)
}

fn m_frexp(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let x = one_f(vm, "frexp", a, k)?;
    let (m, e) = frexp(x);
    Ok(Value::tuple(vec![Value::Float(m), Value::Int(e)]))
}

fn m_ldexp(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    args(vm, "ldexp", a, k, 2, 2)?;
    let x = as_double(vm, &a[0])?;
    if !is_int(vm, &a[1]) {
        return Err(vm.type_error("Expected an int as second argument to ldexp."));
    }
    let b = index_big(vm, &a[1])?;
    let exp = b.to_i64().unwrap_or(if b.neg { i64::MIN } else { i64::MAX });
    if x == 0.0 || !x.is_finite() {
        return Ok(Value::Float(x));
    }
    if exp > i32::MAX as i64 {
        return Err(range_err(vm));
    }
    if exp < i32::MIN as i64 {
        return Ok(Value::Float(0.0f64.copysign(x)));
    }
    let r = scalbn(x, exp);
    if r.is_infinite() {
        return Err(range_err(vm));
    }
    Ok(Value::Float(r))
}

fn next_toward(x: f64, y: f64) -> f64 {
    if x.is_nan() || y.is_nan() {
        return x + y;
    }
    if x == y {
        return y;
    }
    if x == 0.0 {
        return f64::from_bits(1).copysign(y);
    }
    let b = x.to_bits();
    let up = (y > x) == (x > 0.0);
    f64::from_bits(if up { b + 1 } else { b - 1 })
}

fn m_nextafter(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let mut steps = Value::None;
    for (n, v) in k {
        if &*n.s == "steps" {
            steps = v.clone();
        } else {
            return Err(vm.type_error(format!("nextafter() got an unexpected keyword argument '{}'", n.s)));
        }
    }
    let (x, y) = two(vm, "nextafter", a, &[])?;
    if let Value::None = steps {
        return Ok(Value::Float(next_toward(x, y)));
    }
    let s = index_big(vm, &steps)?;
    if s.neg {
        return Err(vm.value_error("steps must be a non-negative integer"));
    }
    let usteps: u64 = if s.bit_length() > 64 {
        u64::MAX
    } else {
        s.mag.iter().enumerate().fold(0u64, |acc, (i, &d)| acc | ((d as u64) << (32 * i)))
    };
    if usteps == 0 || x.is_nan() {
        return Ok(Value::Float(x));
    }
    if y.is_nan() {
        return Ok(Value::Float(y));
    }
    let (ux, uy) = (x.to_bits(), y.to_bits());
    if ux == uy {
        return Ok(Value::Float(x));
    }
    let sign = 1u64 << 63;
    let (ax, ay) = (ux & !sign, uy & !sign);
    let r = if (ux ^ uy) & sign != 0 {
        if ax + ay <= usteps {
            uy
        } else if ax < usteps {
            (uy & sign) | (usteps - ax)
        } else {
            ux - usteps
        }
    } else if ax > ay {
        if ax - ay >= usteps {
            ux - usteps
        } else {
            uy
        }
    } else if ay - ax >= usteps {
        ux + usteps
    } else {
        uy
    };
    Ok(Value::Float(f64::from_bits(r)))
}

fn m_ulp(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let x = one_f(vm, "ulp", a, k)?;
    if x.is_nan() {
        return Ok(Value::Float(x));
    }
    let x = x.abs();
    if x.is_infinite() {
        return Ok(Value::Float(x));
    }
    let x2 = next_toward(x, f64::INFINITY);
    if x2.is_infinite() {
        return Ok(Value::Float(x - next_toward(x, f64::NEG_INFINITY)));
    }
    Ok(Value::Float(x2 - x))
}

fn m_isclose(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    if a.len() > 2 {
        return Err(vm.type_error(format!("isclose() takes exactly 2 positional arguments ({} given)", a.len())));
    }
    let mut slots: [Option<Value>; 4] = [a.first().cloned(), a.get(1).cloned(), None, None];
    for (n, v) in k {
        let i = match &*n.s {
            "a" => 0,
            "b" => 1,
            "rel_tol" => 2,
            "abs_tol" => 3,
            _ => return Err(vm.type_error(format!("isclose() got an unexpected keyword argument '{}'", n.s))),
        };
        if slots[i].is_some() {
            return Err(vm.type_error(format!("argument for isclose() given by name ('{}') and position ({})", n.s, i + 1)));
        }
        slots[i] = Some(v.clone());
    }
    let mut f = [0.0, 0.0, 1e-9, 0.0];
    for i in 0..4 {
        match &slots[i] {
            Some(v) => f[i] = as_double(vm, v)?,
            None if i < 2 => {
                let name = if i == 0 { "a" } else { "b" };
                return Err(vm.type_error(format!("isclose() missing required argument '{}' (pos {})", name, i + 1)));
            }
            None => {}
        }
    }
    let [x, y, rel, abs] = f;
    if rel < 0.0 || abs < 0.0 {
        return Err(vm.value_error("tolerances must be non-negative"));
    }
    if x == y {
        return Ok(Value::Bool(true));
    }
    if x.is_infinite() || y.is_infinite() {
        return Ok(Value::Bool(false));
    }
    let d = (y - x).abs();
    Ok(Value::Bool(d <= (rel * y).abs() || d <= (rel * x).abs() || d <= abs))
}

// -------------------------------------------------------------------------
// floor, ceil, trunc

fn float_to_int(vm: &mut Vm, f: f64) -> PyResult<Value> {
    if f.is_nan() {
        return Err(vm.value_error("cannot convert float NaN to integer"));
    }
    if f.is_infinite() {
        return Err(vm.err("OverflowError", "cannot convert float infinity to integer"));
    }
    Ok(if f.abs() < 9.2e18 { Value::Int(f as i64) } else { Value::from_big(BigInt::from_f64(f).unwrap()) })
}

fn rounding(vm: &mut Vm, name: &str, a: &[Value], k: Kw, f: fn(f64) -> f64) -> PyResult<Value> {
    args(vm, name, a, k, 1, 1)?;
    match &a[0] {
        Value::Float(x) => float_to_int(vm, f(*x)),
        Value::Int(_) | Value::Big(_) => Ok(a[0].clone()),
        Value::Bool(b) => Ok(Value::Int(*b as i64)),
        v => {
            if let Some(m) = special(vm, v, &format!("__{}__", name))? {
                return vm.call_vec(&m, vec![], &[]);
            }
            if let Some(b) = crate::ops::big_of(&vm.base_of(v)) {
                return Ok(Value::from_big(b));
            }
            let x = as_double(vm, v)?;
            float_to_int(vm, f(x))
        }
    }
}

fn m_floor(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    rounding(vm, "floor", a, k, f64::floor)
}
fn m_ceil(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    rounding(vm, "ceil", a, k, f64::ceil)
}

fn m_trunc(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    args(vm, "trunc", a, k, 1, 1)?;
    match &a[0] {
        Value::Float(x) => float_to_int(vm, x.trunc()),
        Value::Int(_) | Value::Big(_) => Ok(a[0].clone()),
        Value::Bool(b) => Ok(Value::Int(*b as i64)),
        v => {
            if let Some(m) = special(vm, v, "__trunc__")? {
                return vm.call_vec(&m, vec![], &[]);
            }
            match vm.base_of(v) {
                Value::Float(x) => float_to_int(vm, x.trunc()),
                b if crate::ops::big_of(&b).is_some() => Ok(Value::from_big(crate::ops::big_of(&b).unwrap())),
                _ => {
                    let tn = vm.type_name(v);
                    Err(vm.type_error(format!("type {} doesn't define __trunc__ method", tn)))
                }
            }
        }
    }
}

// -------------------------------------------------------------------------
// logarithms

/// a positive big int as m * 2**e with m in [0.5, 1), correctly rounded
fn big_frexp(b: &BigInt) -> (f64, i64) {
    let bits = b.bit_length();
    if bits <= 63 {
        return frexp(b.to_f64().abs());
    }
    let a = b.abs();
    let shift = bits - 63;
    let top = a.shr(shift);
    let sticky = top.shl(shift) != a;
    let t = top.to_i64().unwrap_or(0) as u64 | sticky as u64;
    let m = t as f64 * f64::from_bits(0x3c00_0000_0000_0000); // 2**-63
    if m >= 1.0 {
        (0.5, bits as i64 + 1)
    } else {
        (m, bits as i64)
    }
}

fn loghelper(vm: &mut Vm, name: &str, v: &Value, f: fn(f64) -> f64) -> PyResult<f64> {
    let base = vm.base_of(v);
    if let Some(b) = crate::ops::big_of(&base) {
        if b.neg || b.is_zero() {
            return Err(domain(vm));
        }
        let x = b.to_f64();
        if x.is_finite() {
            return Ok(f(x));
        }
        let (m, e) = big_frexp(&b);
        return Ok(f(m) + f(2.0) * e as f64);
    }
    match math_1(vm, name, std::slice::from_ref(v), &[], f, false)? {
        Value::Float(r) => Ok(r),
        _ => unreachable!(),
    }
}

fn m_log(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    nokw(vm, "log", k)?;
    if a.is_empty() || a.len() > 2 {
        return Err(vm.type_error(format!("log expected {} arguments, got {}", if a.is_empty() { "at least 1" } else { "at most 2" }, a.len())));
    }
    let num = loghelper(vm, "log", &a[0], f64::ln)?;
    if a.len() == 1 {
        return Ok(Value::Float(num));
    }
    let den = loghelper(vm, "log", &a[1], f64::ln)?;
    if den == 0.0 {
        return Err(vm.err("ZeroDivisionError", "float division by zero"));
    }
    Ok(Value::Float(num / den))
}
fn m_log2(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    args(vm, "log2", a, k, 1, 1)?;
    loghelper(vm, "log2", &a[0], f64::log2).map(Value::Float)
}
fn m_log10(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    args(vm, "log10", a, k, 1, 1)?;
    loghelper(vm, "log10", &a[0], f64::log10).map(Value::Float)
}

// -------------------------------------------------------------------------
// gamma and lgamma (CPython's Lanczos approximation, N=13)

#[allow(clippy::excessive_precision)]
const LANCZOS_G: f64 = 6.024680040776729583740234375;
#[allow(clippy::excessive_precision)]
const LANCZOS_G_MINUS_HALF: f64 = 5.524680040776729583740234375;
#[allow(clippy::excessive_precision)]
const LANCZOS_NUM: [f64; 13] = [
    23531376880.410759688572007674451636754734846804940,
    42919803642.649098768957899047001988850926355848959,
    35711959237.355668049440185451547166705960488635843,
    17921034426.037209699919755754458931112671403265390,
    6039542586.3520280050642916443072979210699388420708,
    1439720407.3117216736632230727949123939715485786772,
    248874557.86205415651146038641322942321632125127801,
    31426415.585400194380614231628318205362874684987640,
    2876370.6289353724412254090516208496135991145378768,
    186056.26539522349504029498971604569928220784236328,
    8071.6720023658162106380029022722506138218516325024,
    210.82427775157934587250973392071336271166969580291,
    2.5066282746310002701649081771338373386264310793408,
];
const LANCZOS_DEN: [f64; 13] = [0.0, 39916800.0, 120543840.0, 150917976.0, 105258076.0, 45995730.0, 13339535.0, 2637558.0, 357423.0, 32670.0, 1925.0, 66.0, 1.0];
const GAMMA_INTEGRAL: [f64; 23] = [
    1.0,
    1.0,
    2.0,
    6.0,
    24.0,
    120.0,
    720.0,
    5040.0,
    40320.0,
    362880.0,
    3628800.0,
    39916800.0,
    479001600.0,
    6227020800.0,
    87178291200.0,
    1307674368000.0,
    20922789888000.0,
    355687428096000.0,
    6402373705728000.0,
    121645100408832000.0,
    2432902008176640000.0,
    51090942171709440000.0,
    1124000727777607680000.0,
];
const PI: f64 = std::f64::consts::PI;
#[allow(clippy::excessive_precision)]
const LOGPI: f64 = 1.144729885849400174143427351353058711647;

/// sin(pi*x), accurate near integers
fn sinpi(x: f64) -> f64 {
    let y = x.abs() % 2.0;
    let n = (2.0 * y).round() as i32;
    let r = match n {
        0 => (PI * y).sin(),
        1 => (PI * (y - 0.5)).cos(),
        2 => (PI * (1.0 - y)).sin(),
        3 => -(PI * (y - 1.5)).cos(),
        _ => (PI * (y - 2.0)).sin(),
    };
    1.0f64.copysign(x) * r
}

fn lanczos_sum(x: f64) -> f64 {
    let (mut num, mut den) = (0.0, 0.0);
    if x < 5.0 {
        for i in (0..13).rev() {
            num = num * x + LANCZOS_NUM[i];
            den = den * x + LANCZOS_DEN[i];
        }
    } else {
        for i in 0..13 {
            num = num / x + LANCZOS_NUM[i];
            den = den / x + LANCZOS_DEN[i];
        }
    }
    num / den
}

fn tgamma(x: f64) -> (f64, Errno) {
    if !x.is_finite() {
        if x.is_nan() || x > 0.0 {
            return (x, Errno::Ok);
        }
        return (f64::NAN, Errno::Dom);
    }
    if x == 0.0 {
        return (f64::INFINITY.copysign(x), Errno::Dom);
    }
    if x == x.floor() {
        if x < 0.0 {
            return (f64::NAN, Errno::Dom);
        }
        if x <= 23.0 {
            return (GAMMA_INTEGRAL[x as usize - 1], Errno::Ok);
        }
    }
    let absx = x.abs();
    if absx < 1e-20 {
        let r = 1.0 / x;
        return (r, if r.is_infinite() { Errno::Range } else { Errno::Ok });
    }
    if absx > 200.0 {
        if x < 0.0 {
            return (0.0 / sinpi(x), Errno::Ok);
        }
        return (f64::INFINITY, Errno::Range);
    }
    let y = absx + LANCZOS_G_MINUS_HALF;
    let z = if absx > LANCZOS_G_MINUS_HALF {
        let q = y - absx;
        q - LANCZOS_G_MINUS_HALF
    } else {
        let q = y - LANCZOS_G_MINUS_HALF;
        q - absx
    };
    let z = z * LANCZOS_G / y;
    let mut r;
    if x < 0.0 {
        r = -PI / sinpi(absx) / absx * y.exp() / lanczos_sum(absx);
        r -= z * r;
        if absx < 140.0 {
            r /= y.powf(absx - 0.5);
        } else {
            let sqrtpow = y.powf(absx / 2.0 - 0.25);
            r /= sqrtpow;
            r /= sqrtpow;
        }
    } else {
        r = lanczos_sum(absx) / y.exp();
        r += z * r;
        if absx < 140.0 {
            r *= y.powf(absx - 0.5);
        } else {
            let sqrtpow = y.powf(absx / 2.0 - 0.25);
            r *= sqrtpow;
            r *= sqrtpow;
        }
    }
    (r, if r.is_infinite() { Errno::Range } else { Errno::Ok })
}

fn lgamma(x: f64) -> (f64, Errno) {
    if !x.is_finite() {
        return (if x.is_nan() { x } else { f64::INFINITY }, Errno::Ok);
    }
    if x == x.floor() && x <= 2.0 {
        if x <= 0.0 {
            return (f64::INFINITY, Errno::Dom);
        }
        return (0.0, Errno::Ok);
    }
    let absx = x.abs();
    if absx < 1e-20 {
        return (-absx.ln(), Errno::Ok);
    }
    let mut r = lanczos_sum(absx).ln() - LANCZOS_G;
    r += (absx - 0.5) * ((absx + LANCZOS_G - 0.5).ln() - 1.0);
    if x < 0.0 {
        r = LOGPI - sinpi(absx).abs().ln() - absx.ln() - r;
    }
    (r, if r.is_infinite() { Errno::Range } else { Errno::Ok })
}

// -------------------------------------------------------------------------
// hypot, dist, fsum, sumprod, prod

/// error-free product hi + lo == x * y
fn dl_mul(x: f64, y: f64) -> (f64, f64) {
    let z = x * y;
    (z, x.mul_add(y, -z))
}
fn dl_fast_sum(a: f64, b: f64) -> (f64, f64) {
    let x = a + b;
    (x, (a - x) + b)
}
fn dl_sum(a: f64, b: f64) -> (f64, f64) {
    let x = a + b;
    let z = x - a;
    (x, (a - (x - z)) + (b - z))
}

/// sqrt(sum(x**2)) with a correctly rounded result in nearly all cases
fn vector_norm(vec: &mut [f64], max: f64, found_nan: bool) -> f64 {
    if max.is_infinite() {
        return max;
    }
    if found_nan {
        return f64::NAN;
    }
    if max == 0.0 || vec.len() <= 1 {
        return max;
    }
    let (_, max_e) = frexp(max);
    if max_e < -1023 {
        for x in vec.iter_mut() {
            *x /= f64::MIN_POSITIVE;
        }
        return f64::MIN_POSITIVE * vector_norm(vec, max / f64::MIN_POSITIVE, found_nan);
    }
    let scale = scalbn(1.0, -max_e);
    let (mut csum, mut frac1, mut frac2) = (1.0, 0.0, 0.0);
    for &x in vec.iter() {
        let x = x * scale;
        let (hi, lo) = dl_mul(x, x);
        let (s, slo) = dl_fast_sum(csum, hi);
        csum = s;
        frac1 += lo;
        frac2 += slo;
    }
    let mut h = (csum - 1.0 + (frac1 + frac2)).sqrt();
    let (hi, lo) = dl_mul(-h, h);
    let (s, slo) = dl_fast_sum(csum, hi);
    csum = s;
    frac1 += lo;
    frac2 += slo;
    let x = csum - 1.0 + (frac1 + frac2);
    h += x / (2.0 * h);
    h / scale
}

fn m_hypot(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    nokw(vm, "hypot", k)?;
    let mut xs = Vec::with_capacity(a.len());
    let (mut max, mut nan) = (0.0f64, false);
    for v in a {
        let x = as_double(vm, v)?.abs();
        nan |= x.is_nan();
        if x > max {
            max = x;
        }
        xs.push(x);
    }
    Ok(Value::Float(vector_norm(&mut xs, max, nan)))
}

fn m_dist(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    args(vm, "dist", a, k, 2, 2)?;
    let p = vm.to_vec(&a[0])?;
    let q = vm.to_vec(&a[1])?;
    if p.len() != q.len() {
        return Err(vm.value_error("both points must have the same number of dimensions"));
    }
    let mut xs = Vec::with_capacity(p.len());
    let (mut max, mut nan) = (0.0f64, false);
    for (px, qx) in p.iter().zip(q.iter()) {
        let px = as_double(vm, px)?;
        let qx = as_double(vm, qx)?;
        let x = (px - qx).abs();
        nan |= x.is_nan();
        if x > max {
            max = x;
        }
        xs.push(x);
    }
    Ok(Value::Float(vector_norm(&mut xs, max, nan)))
}

/// Shewchuk's exact partials, as CPython's fsum
#[derive(Default)]
struct Fsum {
    p: Vec<f64>,
    special: f64,
    inf: f64,
}

impl Fsum {
    fn add(&mut self, xsave: f64) -> Result<(), &'static str> {
        let mut x = xsave;
        let mut i = 0;
        for j in 0..self.p.len() {
            let mut y = self.p[j];
            if x.abs() < y.abs() {
                std::mem::swap(&mut x, &mut y);
            }
            let hi = x + y;
            let lo = y - (hi - x);
            if lo != 0.0 {
                self.p[i] = lo;
                i += 1;
            }
            x = hi;
        }
        self.p.truncate(i);
        if x != 0.0 {
            if !x.is_finite() {
                if xsave.is_finite() {
                    return Err("intermediate overflow in fsum");
                }
                if xsave.is_infinite() {
                    self.inf += xsave;
                }
                self.special += xsave;
                self.p.clear();
            } else {
                self.p.push(x);
            }
        }
        Ok(())
    }

    fn finish(&self) -> Result<f64, &'static str> {
        if self.special != 0.0 {
            if self.inf.is_nan() {
                return Err("-inf + inf in fsum");
            }
            return Ok(self.special);
        }
        let p = &self.p;
        let mut n = p.len();
        let mut hi = 0.0;
        if n > 0 {
            n -= 1;
            hi = p[n];
            let mut lo = 0.0;
            while n > 0 {
                let x = hi;
                n -= 1;
                let y = p[n];
                hi = x + y;
                let yr = hi - x;
                lo = y - yr;
                if lo != 0.0 {
                    break;
                }
            }
            if n > 0 && ((lo < 0.0 && p[n - 1] < 0.0) || (lo > 0.0 && p[n - 1] > 0.0)) {
                let y = lo * 2.0;
                let x = hi + y;
                let yr = x - hi;
                if y == yr {
                    hi = x;
                }
            }
        }
        Ok(hi)
    }
}

/// exactly rounded sum (Shewchuk's algorithm, as CPython's fsum)
pub fn fsum(xs: &[f64]) -> Result<f64, &'static str> {
    let mut s = Fsum::default();
    for &x in xs {
        s.add(x)?;
    }
    s.finish()
}

fn fsum_err(vm: &mut Vm, m: &str) -> Exc {
    if m.starts_with("intermediate") {
        vm.err("OverflowError", m)
    } else {
        vm.value_error(m)
    }
}

fn m_fsum(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    args(vm, "fsum", a, k, 1, 1)?;
    let it = vm.iter(&a[0])?;
    let mut s = Fsum::default();
    while let Some(v) = vm.next(&it)? {
        let x = as_double(vm, &v)?;
        if let Err(m) = s.add(x) {
            return Err(fsum_err(vm, m));
        }
    }
    match s.finish() {
        Ok(v) => Ok(Value::Float(v)),
        Err(m) => Err(fsum_err(vm, m)),
    }
}

/// a running sum of products in triple-length precision
#[derive(Clone, Copy, Default)]
struct Triple {
    hi: f64,
    lo: f64,
    tiny: f64,
}

impl Triple {
    fn fma(self, x: f64, y: f64) -> Triple {
        let (phi, plo) = dl_mul(x, y);
        let (shi, slo) = dl_sum(self.hi, phi);
        let (r1hi, r1lo) = dl_sum(self.lo, plo);
        let (r2hi, r2lo) = dl_sum(r1hi, slo);
        Triple { hi: shi, lo: r2hi, tiny: self.tiny + r1lo + r2lo }
    }
    fn to_f64(self) -> f64 {
        let (hi, lo) = dl_sum(self.lo, self.hi);
        self.tiny + lo + hi
    }
}

/// a float of an exact float/int/bool pair, as sumprod's float path takes it
fn flt_pair(p: &Value, q: &Value) -> Option<(f64, f64)> {
    let num = |v: &Value| match v {
        Value::Int(i) => Some(*i as f64),
        Value::Bool(b) => Some(*b as i64 as f64),
        Value::Big(b) => Some(b.to_f64()).filter(|f| f.is_finite()),
        _ => None,
    };
    match (p, q) {
        (Value::Float(x), Value::Float(y)) => Some((*x, *y)),
        (Value::Float(x), o) => num(o).map(|y| (*x, y)),
        (o, Value::Float(y)) => num(o).map(|x| (x, *y)),
        _ => None,
    }
}

fn m_sumprod(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    args(vm, "sumprod", a, k, 2, 2)?;
    let pit = vm.iter(&a[0])?;
    let qit = vm.iter(&a[1])?;
    let mut total = Value::Int(0);
    let (mut int_on, mut int_used, mut int_total) = (true, false, 0i64);
    let (mut flt_on, mut flt_used, mut flt_total) = (true, false, Triple::default());
    loop {
        let p = vm.next(&pit)?;
        let q = vm.next(&qit)?;
        if p.is_some() != q.is_some() {
            return Err(vm.value_error("Inputs are not the same length"));
        }
        let pair = p.zip(q);
        if int_on {
            if let Some((Value::Int(x), Value::Int(y))) = &pair {
                if let Some(t) = x.checked_mul(*y).and_then(|m| int_total.checked_add(m)) {
                    int_total = t;
                    int_used = true;
                    continue;
                }
            }
            int_on = false;
            if int_used {
                total = vm.binop(BinOp::Add, &total, &Value::Int(int_total))?;
                int_used = false;
            }
        }
        if flt_on {
            if let Some((x, y)) = pair.as_ref().and_then(|(p, q)| flt_pair(p, q)) {
                let t = flt_total.fma(x, y);
                if t.hi.is_finite() {
                    flt_total = t;
                    flt_used = true;
                    continue;
                }
            }
            flt_on = false;
            if flt_used {
                total = vm.binop(BinOp::Add, &total, &Value::Float(flt_total.to_f64()))?;
                flt_used = false;
            }
        }
        let Some((p, q)) = pair else { return Ok(total) };
        let term = vm.binop(BinOp::Mul, &p, &q)?;
        total = vm.binop(BinOp::Add, &total, &term)?;
    }
}

fn m_prod(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let mut acc = Value::Int(1);
    for (n, v) in k {
        if &*n.s == "start" {
            acc = v.clone();
        } else {
            return Err(vm.type_error(format!("prod() got an unexpected keyword argument '{}'", n.s)));
        }
    }
    if a.len() != 1 {
        return Err(vm.type_error(format!("prod() takes exactly 1 positional argument ({} given)", a.len())));
    }
    let it = vm.iter(&a[0])?;
    while let Some(x) = vm.next(&it)? {
        acc = vm.binop(BinOp::Mul, &acc, &x)?;
    }
    Ok(acc)
}

// -------------------------------------------------------------------------
// integer functions

pub fn isqrt_big(n: &BigInt) -> BigInt {
    if n.is_zero() {
        return BigInt::zero();
    }
    // Newton from above
    let bits = n.bit_length();
    let mut x = BigInt::from_i64(1).shl(bits.div_ceil(2));
    loop {
        let y = x.add(&n.divmod(&x).unwrap().0).shr(1);
        if y.cmp(&x) != Ordering::Less {
            return x;
        }
        x = y;
    }
}

fn m_isqrt(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    args(vm, "isqrt", a, k, 1, 1)?;
    let n = index_big(vm, &a[0])?;
    if n.neg {
        return Err(vm.value_error("isqrt() argument must be nonnegative"));
    }
    if let Some(i) = n.to_i64() {
        let i = i as u128;
        let mut r = (i as f64).sqrt() as u128;
        while r * r > i {
            r -= 1;
        }
        while (r + 1) * (r + 1) <= i {
            r += 1;
        }
        return Ok(Value::Int(r as i64));
    }
    Ok(Value::from_big(isqrt_big(&n)))
}

fn gcd_big(a: &BigInt, b: &BigInt) -> BigInt {
    let (mut x, mut y) = (a.abs(), b.abs());
    while !y.is_zero() {
        if let (Some(p), Some(q)) = (x.to_i64(), y.to_i64()) {
            let (mut p, mut q) = (p as u64, q as u64);
            while q != 0 {
                (p, q) = (q, p % q);
            }
            return BigInt::from_u128(p as u128, false);
        }
        let r = x.divmod(&y).unwrap().1;
        x = y;
        y = r;
    }
    x
}

fn m_gcd(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    nokw(vm, "gcd", k)?;
    let mut acc = BigInt::zero();
    for v in a {
        let b = index_big(vm, v)?;
        if acc != BigInt::from_i64(1) {
            acc = gcd_big(&acc, &b);
        }
    }
    Ok(Value::from_big(acc))
}

fn m_lcm(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    nokw(vm, "lcm", k)?;
    if a.is_empty() {
        return Ok(Value::Int(1));
    }
    let mut acc = index_big(vm, &a[0])?.abs();
    for v in &a[1..] {
        let b = index_big(vm, v)?;
        if acc.is_zero() {
            continue;
        }
        if b.is_zero() {
            acc = BigInt::zero();
            continue;
        }
        let g = gcd_big(&acc, &b);
        acc = acc.divmod(&g).unwrap().0.mul(&b).abs();
    }
    Ok(Value::from_big(acc))
}

/// product(range(start, stop, 2)) by divide and conquer
fn partial_product(start: u64, stop: u64, max_bits: u32) -> BigInt {
    let n = (stop - start) / 2;
    if n <= 64 && n * max_bits as u64 <= 64 {
        let mut total = start;
        let mut j = start + 2;
        while j < stop {
            total *= j;
            j += 2;
        }
        return BigInt::from_u128(total as u128, false);
    }
    let mid = (start + n) | 1;
    let left = partial_product(start, mid, bit_len(mid - 2));
    let right = partial_product(mid, stop, max_bits);
    left.mul(&right)
}

fn bit_len(n: u64) -> u32 {
    64 - n.leading_zeros()
}

/// n! as odd_part << (n - popcount(n))
fn factorial_big(n: u64) -> BigInt {
    if n < 21 {
        return BigInt::from_u128((1..=n as u128).product(), false);
    }
    let mut inner = BigInt::from_i64(1);
    let mut outer = BigInt::from_i64(1);
    let mut upper = 3u64;
    for i in (0..=(bit_len(n) as i64 - 2)).rev() {
        let v = n >> i;
        if v <= 2 {
            continue;
        }
        let lower = upper;
        upper = (v + 1) | 1;
        inner = inner.mul(&partial_product(lower, upper, bit_len(upper - 2)));
        outer = outer.mul(&inner);
    }
    outer.shl((n - n.count_ones() as u64) as usize)
}

fn m_factorial(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    args(vm, "factorial", a, k, 1, 1)?;
    let n = index_big(vm, &a[0])?;
    if n.neg {
        return Err(vm.value_error("factorial() not defined for negative values"));
    }
    let Some(n) = n.to_i64() else {
        return Err(vm.err("OverflowError", format!("factorial() argument should not exceed {}", i64::MAX)));
    };
    Ok(Value::from_big(factorial_big(n as u64)))
}

/// P(n, k) or C(n, k) for n below 2**63
fn perm_comb_small(n: u64, k: u64, comb: bool) -> BigInt {
    // direct products while they fit 128 bits
    let mut acc: Option<u128> = Some(1);
    for i in 0..k {
        acc = acc.and_then(|r| r.checked_mul((n - i) as u128)).map(|r| if comb { r / (i as u128 + 1) } else { r });
        if acc.is_none() {
            break;
        }
    }
    if let Some(r) = acc {
        return BigInt::from_u128(r, false);
    }
    let j = k / 2;
    let a = perm_comb_small(n, j, comb).mul(&perm_comb_small(n - j, k - j, comb));
    if comb {
        return a.divmod(&perm_comb_small(k, j, true)).unwrap().0;
    }
    a
}

fn perm_comb(n: &BigInt, k: u64, comb: bool) -> BigInt {
    if let Some(ni) = n.to_i64() {
        return perm_comb_small(ni as u64, k, comb);
    }
    if k == 0 {
        return BigInt::from_i64(1);
    }
    if k == 1 {
        return n.clone();
    }
    let j = k / 2;
    let a = perm_comb(n, j, comb);
    let b = perm_comb(&n.sub(&BigInt::from_i64(j as i64)), k - j, comb);
    let a = a.mul(&b);
    if comb {
        return a.divmod(&perm_comb_small(k, j, true)).unwrap().0;
    }
    a
}

fn nonneg(vm: &mut Vm, n: &BigInt, k: &BigInt) -> PyResult<()> {
    if n.neg {
        return Err(vm.value_error("n must be a non-negative integer"));
    }
    if k.neg {
        return Err(vm.value_error("k must be a non-negative integer"));
    }
    Ok(())
}

fn m_perm(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    args(vm, "perm", a, k, 1, 2)?;
    if matches!(a.get(1), None | Some(Value::None)) {
        return m_factorial(vm, &a[..1], &[]);
    }
    let n = index_big(vm, &a[0])?;
    let kk = index_big(vm, &a[1])?;
    nonneg(vm, &n, &kk)?;
    if n.cmp(&kk) == Ordering::Less {
        return Ok(Value::Int(0));
    }
    let Some(ki) = kk.to_i64() else {
        return Err(vm.err("OverflowError", format!("k must not exceed {}", i64::MAX)));
    };
    Ok(Value::from_big(perm_comb(&n, ki as u64, false)))
}

fn m_comb(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    args(vm, "comb", a, k, 2, 2)?;
    let n = index_big(vm, &a[0])?;
    let mut kk = index_big(vm, &a[1])?;
    nonneg(vm, &n, &kk)?;
    let rest = n.sub(&kk);
    if rest.neg {
        return Ok(Value::Int(0));
    }
    if rest.cmp(&kk) == Ordering::Less {
        kk = rest;
    }
    let Some(ki) = kk.to_i64() else {
        return Err(vm.err("OverflowError", format!("min(n - k, k) must not exceed {}", i64::MAX)));
    };
    Ok(Value::from_big(perm_comb(&n, ki as u64, true)))
}
