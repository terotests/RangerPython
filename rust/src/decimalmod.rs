// SPDX-License-Identifier: MIT
//! _rpydecimal: native versions of the hot methods of the pure-Python
//! decimal module (pylib/_pydecimal.py), as CPython has its C _decimal.
//! They work on the same representation (the _sign, _int, _exp and
//! _is_special slots), follow the Python code step by step, and hand the
//! cases they leave out (NaNs, infinities, unusual operands) to the Python
//! methods they replace. Signals still go through Context._raise_error.

use crate::bigint::BigInt;
use crate::builtins::*;
use crate::modules::{module, put_fns};
use crate::value::*;
use crate::vm::Vm;
use std::cell::RefCell;
use std::rc::Rc;

/// what install() found in the decimal module
struct State {
    decimal: Rc<Class>,
    context: Rc<Class>,
    /// the decimal module's namespace
    ns: Rc<RefCell<Dict>>,
    getcontext: Value,
    /// _current_context_var, ContextVar.get and the contextvars namespace
    /// getcontext() reads
    var: Value,
    var_get: Value,
    var_ns: Rc<RefCell<Dict>>,
    /// _SignedInfinity
    infinities: Value,
    /// the Python methods replaced, by name
    py: Vec<(&'static str, Value)>,
    /// the signal classes, by name
    signals: Vec<(&'static str, Value)>,
}

thread_local! {
    static STATE: RefCell<Option<Rc<State>>> = const { RefCell::new(None) };
}

fn state(vm: &mut Vm) -> PyResult<Rc<State>> {
    match STATE.with(|s| s.borrow().clone()) {
        Some(s) => Ok(s),
        None => Err(vm.err("RuntimeError", "_rpydecimal is not installed")),
    }
}

/// the native replacements, by method name
const METHODS: [(&str, NativeFnPtr); 11] = [
    ("__new__", d_new),
    ("__add__", d_add),
    ("__radd__", d_add),
    ("__sub__", d_sub),
    ("__rsub__", d_rsub),
    ("__mul__", d_mul),
    ("__rmul__", d_mul),
    ("__bool__", d_bool),
    ("__str__", d_str),
    ("__repr__", d_repr),
    ("_fix", d_fix),
];

const SIGNALS: [&str; 8] = ["Clamped", "Inexact", "Rounded", "Subnormal", "Underflow", "Overflow", "InvalidOperation", "ConversionSyntax"];

pub fn decimal_module() -> Value {
    let (d, m) = module("_rpydecimal");
    put_fns(&d, &[("_rpydecimal.install", install)]);
    m
}

/// install(Decimal, namespace): puts the native methods on Decimal, keeping
/// the Python ones they fall back on
fn install(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    need(vm, a, 2)?;
    let (Value::Class(cls), Value::Dict(ns)) = (&a[0], &a[1]) else {
        return Err(vm.type_error("install(Decimal, namespace)"));
    };
    let get = |vm: &mut Vm, name: &str| ns.borrow().get_s(name).ok_or_else(|| vm.type_error(format!("no {}", name)));
    let getcontext = get(vm, "getcontext")?;
    let infinities = get(vm, "_SignedInfinity")?;
    let var = get(vm, "_current_context_var")?;
    let Value::Class(context) = get(vm, "Context")? else {
        return Err(vm.type_error("no Context class"));
    };
    let var_get = vm.class_of(&var).lookup_s("get").ok_or_else(|| vm.type_error("no ContextVar.get"))?;
    let Value::Func(f) = &var_get else {
        return Err(vm.type_error("ContextVar.get is not a function"));
    };
    let var_ns = f.globals.clone();
    let mut signals = vec![];
    for name in SIGNALS {
        let v = ns.borrow().get_s(name).ok_or_else(|| vm.type_error(format!("no {}", name)))?;
        signals.push((name, v));
    }
    let mut py = vec![];
    for (name, f) in METHODS {
        let old = cls.dict.borrow().get_s(name).ok_or_else(|| vm.type_error(format!("no Decimal.{}", name)))?;
        py.push((name, old));
        let full: &'static str = Box::leak(format!("Decimal.{}", name).into_boxed_str());
        let native = Value::Native(Rc::new(NativeFn::new(full, f, Value::None)));
        let v = if name == "__new__" { Value::StaticMethod(Rc::new(native)) } else { native };
        cls.dict.borrow_mut().set_s(name, v);
    }
    let st = State { decimal: cls.clone(), context, ns: ns.clone(), getcontext, var, var_get, var_ns, infinities, py, signals };
    STATE.with(|s| *s.borrow_mut() = Some(Rc::new(st)));
    Ok(Value::None)
}

/// the Python method `name` replaced, called with the same arguments
fn fallback(vm: &mut Vm, st: &State, name: &str, a: &[Value], k: Kw) -> PyResult<Value> {
    let f = st.py.iter().find(|(n, _)| *n == name).map(|(_, f)| f.clone()).unwrap();
    let f = match f {
        Value::StaticMethod(inner) => (*inner).clone(),
        other => other,
    };
    vm.call_vec(&f, a.to_vec(), k)
}

fn signal(st: &State, name: &str) -> Value {
    st.signals.iter().find(|(n, _)| *n == name).map(|(_, v)| v.clone()).unwrap()
}

/// a finite decimal: sign 0 or 1, coefficient digits, exponent
#[derive(Clone)]
struct Fin {
    sign: i64,
    int: Rc<PyStr>,
    exp: i64,
}

impl Fin {
    fn is_zero(&self) -> bool {
        &*self.int.s == "0"
    }
    fn len(&self) -> i64 {
        self.int.s.len() as i64
    }
}

/// a NaN, sNaN or infinity: its sign, payload and the _exp marking it
#[derive(Clone)]
struct Spec {
    sign: i64,
    int: Value,
    exp: Value,
    kind: u8,
}

/// what a Decimal's slots hold
#[derive(Clone)]
enum Dec {
    Fin(Fin),
    Spec(Spec),
}

impl Dec {
    fn sign(&self) -> i64 {
        match self {
            Dec::Fin(f) => f.sign,
            Dec::Spec(s) => s.sign,
        }
    }
    /// _isnan(): 0, 1 for a quiet NaN, 2 for a signaling one
    fn nan(&self) -> u8 {
        match self {
            Dec::Spec(Spec { kind: b'n', .. }) => 1,
            Dec::Spec(Spec { kind: b'N', .. }) => 2,
            _ => 0,
        }
    }
    fn is_inf(&self) -> bool {
        matches!(self, Dec::Spec(Spec { kind: b'F', .. }))
    }
    /// `not self`
    fn is_zero(&self) -> bool {
        matches!(self, Dec::Fin(f) if f.is_zero())
    }
    /// copy_negate()
    fn negated(&self) -> Dec {
        match self {
            Dec::Fin(f) => Dec::Fin(Fin { sign: 1 - f.sign, ..f.clone() }),
            Dec::Spec(s) => Dec::Spec(Spec { sign: 1 - s.sign, ..s.clone() }),
        }
    }
}

/// the slots of a Decimal instance; None when it is not one
fn read(st: &State, v: &Value) -> Option<Dec> {
    // a subclass may override the methods the Python code calls on it
    let Value::Instance(i) = v else { return None };
    if !Rc::ptr_eq(&*i.class.borrow(), &st.decimal) {
        return None;
    }
    let ex = i.extra.borrow();
    let special = match ex.get_s("_is_special")? {
        Value::Bool(b) => b,
        _ => return None,
    };
    let sign = match ex.get_s("_sign")? {
        Value::Int(s @ (0 | 1)) => s,
        Value::Bool(b) => b as i64,
        _ => return None,
    };
    let Value::Str(int) = ex.get_s("_int")? else { return None };
    let exp = ex.get_s("_exp")?;
    if special {
        let kind = match exp.as_str()? {
            "F" => b'F',
            "n" => b'n',
            "N" => b'N',
            _ => return None,
        };
        return Some(Dec::Spec(Spec { sign, int: Value::Str(int), exp, kind }));
    }
    let Value::Int(exp) = exp else { return None };
    Some(Dec::Fin(Fin { sign, int, exp }))
}

/// a new instance of `cls` with the given slots (_dec_from_triple)
fn make(vm: &mut Vm, cls: &Rc<Class>, sign: i64, int: Value, exp: Value, special: bool) -> Value {
    let v = vm.new_instance(cls);
    if let Value::Instance(i) = &v {
        let mut ex = i.extra.borrow_mut();
        ex.set_s("_sign", Value::Int(sign));
        ex.set_s("_int", int);
        ex.set_s("_exp", exp);
        ex.set_s("_is_special", Value::Bool(special));
    }
    v
}

fn triple(vm: &mut Vm, st: &State, sign: i64, int: &str, exp: i64) -> Value {
    make(vm, &st.decimal, sign, Value::str(int), Value::Int(exp), false)
}

fn fin_value(vm: &mut Vm, st: &State, f: &Fin) -> Value {
    make(vm, &st.decimal, f.sign, Value::Str(f.int.clone()), Value::Int(f.exp), false)
}

/// Decimal(d)
fn dec_value(vm: &mut Vm, st: &State, d: &Dec) -> Value {
    match d {
        Dec::Fin(f) => fin_value(vm, st, f),
        Dec::Spec(s) => make(vm, &st.decimal, s.sign, s.int.clone(), s.exp.clone(), true),
    }
}

/// the sign and digits of an int, as Decimal(int) takes them with
/// `value >= 0` and `str(abs(value))`: an exact int, or a subclass that
/// keeps int's __ge__ and __abs__
fn int_digits(vm: &Vm, v: &Value) -> Option<(i64, String)> {
    match v {
        Value::Int(i) => Some(((*i < 0) as i64, i.unsigned_abs().to_string())),
        Value::Bool(b) => Some((0, (*b as u8).to_string())),
        Value::Big(b) => Some((b.neg as i64, b.abs().to_radix(10))),
        Value::Instance(i) => {
            let base = i.base.borrow().clone();
            if !matches!(base, Value::Int(_) | Value::Big(_) | Value::Bool(_)) {
                return None;
            }
            let cls = i.class.borrow().clone();
            let int_method = |name: &str| cls.lookup_s(name).zip(vm.t.int.lookup_s(name)).is_some_and(|(x, y)| x.is(&y));
            if int_method("__ge__") && int_method("__abs__") {
                int_digits(vm, &base)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// _convert_other for the operands handled here: a Decimal, an exact int,
/// or NotImplemented; None for anything the Python code should see
enum Other {
    Dec(Dec),
    NotImpl,
}

fn convert_other(vm: &Vm, st: &State, v: &Value) -> Option<Other> {
    if let Some(d) = read(st, v) {
        return Some(Other::Dec(d));
    }
    if let Some((sign, digits)) = int_digits(vm, v) {
        return Some(Other::Dec(Dec::Fin(Fin { sign, int: Rc::new(PyStr::new(digits)), exp: 0 })));
    }
    match v {
        Value::Float(_) | Value::Complex(_) | Value::Str(_) | Value::None | Value::List(_) | Value::Tuple(_) | Value::Dict(_) => Some(Other::NotImpl),
        // an instance that is neither a Decimal nor an int, and whose
        // __class__ is its type (a Fraction, say)
        Value::Instance(i) => {
            let cls = i.class.borrow().clone();
            let own_class = match (cls.lookup(&vm.n.class), vm.t.object.lookup(&vm.n.class)) {
                (Some(x), Some(y)) => x.is(&y),
                (x, y) => x.is_none() && y.is_none(),
            };
            let plain = own_class && cls.builtin.get() == Builtin::Object && !cls.is_subclass(&st.decimal);
            plain.then_some(Other::NotImpl)
        }
        _ => None,
    }
}

/// the context attributes the arithmetic reads
struct Ctx {
    obj: Value,
    prec: i64,
    emin: i64,
    emax: i64,
    clamp: i64,
    rounding: Rounding,
}

#[derive(Clone, Copy, PartialEq)]
enum Rounding {
    Down,
    Up,
    HalfUp,
    HalfDown,
    HalfEven,
    Ceiling,
    Floor,
    Up05,
}

/// an attribute of a context: read from the instance dict of a plain
/// Context, where Context.__init__ keeps them
fn ctx_attr(vm: &mut Vm, st: &State, o: &Value, name: &str) -> PyResult<Value> {
    if let Value::Instance(i) = o {
        if Rc::ptr_eq(&*i.class.borrow(), &st.context) {
            if let Some(v) = i.dict.borrow().get_s(name) {
                return Ok(v);
            }
        }
    }
    vm.getattr(o, &ps(name))
}

fn int_attr(vm: &mut Vm, st: &State, o: &Value, name: &str) -> PyResult<Option<i64>> {
    Ok(match ctx_attr(vm, st, o, name)? {
        Value::Int(i) => Some(i),
        Value::Bool(b) => Some(b as i64),
        _ => None,
    })
}

/// getcontext(). While the module's getcontext and ContextVar.get are the
/// ones install() saw, this is the value contextvars' current mapping
/// holds for _current_context_var, when it holds one
fn getcontext(vm: &mut Vm, st: &State) -> PyResult<Value> {
    let same = |d: &Rc<RefCell<Dict>>, name: &str, v: &Value| d.borrow().get_s(name).is_some_and(|x| x.is(v));
    if same(&st.ns, "getcontext", &st.getcontext) && same(&st.ns, "_current_context_var", &st.var) && vm.class_of(&st.var).lookup_s("get").is_some_and(|g| g.is(&st.var_get)) {
        let data = match st.var_ns.borrow().get_s("_current") {
            Some(Value::Instance(cur)) => cur.dict.borrow().get_s("_data"),
            _ => None,
        };
        if let Some(Value::Dict(d)) = data {
            if let Some(c) = vm.dict_get(&d, &st.var)? {
                return Ok(c);
            }
        }
    }
    // the Python methods call whatever the module's getcontext now is
    let f = st.ns.borrow().get_s("getcontext");
    match f {
        Some(f) => vm.call_vec(&f, vec![], &[]),
        None => Err(vm.err("NameError", "name 'getcontext' is not defined")),
    }
}

/// the context argument or the current one; None when its attributes are
/// not plain enough for the native code
fn context(vm: &mut Vm, st: &State, a: &[Value], at: usize, k: Kw) -> PyResult<Option<Ctx>> {
    let given = a.get(at).cloned().or_else(|| k.iter().find(|(n, _)| &*n.s == "context").map(|(_, v)| v.clone()));
    let obj = match given {
        Some(c) if !c.is_none() => c,
        _ => getcontext(vm, st)?,
    };
    let (Some(prec), Some(emin), Some(emax), Some(clamp)) = (int_attr(vm, st, &obj, "prec")?, int_attr(vm, st, &obj, "Emin")?, int_attr(vm, st, &obj, "Emax")?, int_attr(vm, st, &obj, "clamp")?) else {
        return Ok(None);
    };
    let r = ctx_attr(vm, st, &obj, "rounding")?;
    let rounding = match r.as_str() {
        Some("ROUND_DOWN") => Rounding::Down,
        Some("ROUND_UP") => Rounding::Up,
        Some("ROUND_HALF_UP") => Rounding::HalfUp,
        Some("ROUND_HALF_DOWN") => Rounding::HalfDown,
        Some("ROUND_HALF_EVEN") => Rounding::HalfEven,
        Some("ROUND_CEILING") => Rounding::Ceiling,
        Some("ROUND_FLOOR") => Rounding::Floor,
        Some("ROUND_05UP") => Rounding::Up05,
        _ => return Ok(None),
    };
    Ok(Some(Ctx { obj, prec, emin, emax, clamp, rounding }))
}

/// context._raise_error(condition, *args)
fn raise_error(vm: &mut Vm, st: &State, ctx: &Ctx, cond: &str, args: Vec<Value>) -> PyResult<Value> {
    let f = vm.getattr(&ctx.obj, &ps("_raise_error"))?;
    let mut a = vec![signal(st, cond)];
    a.extend(args);
    vm.call_vec(&f, a, &[])
}

// rounding: whether digits from `prec` on are dropped exactly (0), are
// rounded away (1) or truncated with a loss (-1), as _pydecimal's _round_*

fn all_zeros(s: &[u8], from: usize) -> bool {
    s.get(from..).is_none_or(|t| t.iter().all(|&c| c == b'0'))
}

fn exact_half(s: &[u8], from: usize) -> bool {
    s.get(from) == Some(&b'5') && all_zeros(s, from + 1)
}

fn round_digits(mode: Rounding, sign: i64, s: &[u8], prec: usize) -> i64 {
    let down = || if all_zeros(s, prec) { 0 } else { -1 };
    let half_up = || {
        if b"56789".contains(&s[prec]) {
            1
        } else if all_zeros(s, prec) {
            0
        } else {
            -1
        }
    };
    match mode {
        Rounding::Down => down(),
        Rounding::Up => -down(),
        Rounding::HalfUp => half_up(),
        Rounding::HalfDown => {
            if exact_half(s, prec) {
                -1
            } else {
                half_up()
            }
        }
        Rounding::HalfEven => {
            if exact_half(s, prec) && (prec == 0 || b"02468".contains(&s[prec - 1])) {
                -1
            } else {
                half_up()
            }
        }
        Rounding::Ceiling => {
            if sign == 1 {
                down()
            } else {
                -down()
            }
        }
        Rounding::Floor => {
            if sign == 0 {
                down()
            } else {
                -down()
            }
        }
        Rounding::Up05 => {
            if prec > 0 && !b"05".contains(&s[prec - 1]) {
                down()
            } else {
                -down()
            }
        }
    }
}

/// str(int(digits) + 1)
fn increment(digits: &str) -> String {
    let mut b = digits.as_bytes().to_vec();
    let mut i = b.len();
    loop {
        if i == 0 {
            b.insert(0, b'1');
            break;
        }
        i -= 1;
        if b[i] == b'9' {
            b[i] = b'0';
        } else {
            b[i] += 1;
            break;
        }
    }
    String::from_utf8(b).unwrap()
}

/// Decimal._fix for a finite value: round to the context's precision and
/// bring the exponent into range, raising the signals the Python code does
fn fix(vm: &mut Vm, st: &State, ctx: &Ctx, d: &Fin) -> PyResult<Value> {
    let etiny = ctx.emin - ctx.prec + 1;
    let etop = ctx.emax - ctx.prec + 1;
    if d.is_zero() {
        let exp_max = if ctx.clamp == 0 { ctx.emax } else { etop };
        let new_exp = d.exp.max(etiny).min(exp_max);
        if new_exp != d.exp {
            raise_error(vm, st, ctx, "Clamped", vec![])?;
            return Ok(triple(vm, st, d.sign, "0", new_exp));
        }
        return Ok(fin_value(vm, st, d));
    }
    let mut exp_min = d.len() + d.exp - ctx.prec;
    if exp_min > etop {
        let ans = raise_error(vm, st, ctx, "Overflow", vec![Value::str("above Emax"), Value::Int(d.sign)])?;
        raise_error(vm, st, ctx, "Inexact", vec![])?;
        raise_error(vm, st, ctx, "Rounded", vec![])?;
        return Ok(ans);
    }
    let subnormal = exp_min < etiny;
    if subnormal {
        exp_min = etiny;
    }
    if d.exp < exp_min {
        let mut digits = d.len() + d.exp - exp_min;
        let mut int: &str = &d.int.s;
        if digits < 0 {
            int = "1";
            digits = 0;
        }
        let digits = digits as usize;
        let changed = round_digits(ctx.rounding, d.sign, int.as_bytes(), digits);
        let mut coeff = if digits == 0 { "0".to_string() } else { int[..digits].to_string() };
        if changed > 0 {
            coeff = increment(&coeff);
            if coeff.len() as i64 > ctx.prec {
                coeff.pop();
                exp_min += 1;
            }
        }
        let ans = if exp_min > etop {
            raise_error(vm, st, ctx, "Overflow", vec![Value::str("above Emax"), Value::Int(d.sign)])?
        } else {
            triple(vm, st, d.sign, &coeff, exp_min)
        };
        if changed != 0 && subnormal {
            raise_error(vm, st, ctx, "Underflow", vec![])?;
        }
        if subnormal {
            raise_error(vm, st, ctx, "Subnormal", vec![])?;
        }
        if changed != 0 {
            raise_error(vm, st, ctx, "Inexact", vec![])?;
        }
        raise_error(vm, st, ctx, "Rounded", vec![])?;
        if !vm.truthy(&ans)? {
            raise_error(vm, st, ctx, "Clamped", vec![])?;
        }
        return Ok(ans);
    }
    if subnormal {
        raise_error(vm, st, ctx, "Subnormal", vec![])?;
    }
    if ctx.clamp == 1 && d.exp > etop {
        raise_error(vm, st, ctx, "Clamped", vec![])?;
        let padded = format!("{}{}", d.int.s, "0".repeat((d.exp - etop) as usize));
        return Ok(triple(vm, st, d.sign, &padded, etop));
    }
    Ok(fin_value(vm, st, d))
}

/// Decimal._rescale for a finite value
fn rescale(d: &Fin, exp: i64, rounding: Rounding) -> Fin {
    let mk = |int: String| Fin { sign: d.sign, int: Rc::new(PyStr::new(int)), exp };
    if d.is_zero() {
        return mk("0".into());
    }
    if d.exp >= exp {
        return mk(format!("{}{}", d.int.s, "0".repeat((d.exp - exp) as usize)));
    }
    let mut digits = d.len() + d.exp - exp;
    let mut int: &str = &d.int.s;
    if digits < 0 {
        int = "1";
        digits = 0;
    }
    let digits = digits as usize;
    let changed = round_digits(rounding, d.sign, int.as_bytes(), digits);
    let mut coeff = if digits == 0 { "0".to_string() } else { int[..digits].to_string() };
    if changed == 1 {
        coeff = increment(&coeff);
    }
    mk(coeff)
}

fn big(digits: &str) -> BigInt {
    BigInt::from_radix(digits, 10).unwrap_or_else(BigInt::zero)
}

/// a coefficient times a power of ten
fn scaled(digits: &str, by: i64) -> BigInt {
    if by <= 0 {
        return big(digits);
    }
    big(&format!("{}{}", digits, "0".repeat(by as usize)))
}

/// Decimal._fix_nan
fn fix_nan(vm: &mut Vm, st: &State, ctx: &Ctx, s: &Spec) -> Value {
    let payload = s.int.as_str().unwrap_or("");
    let max = (ctx.prec - ctx.clamp).max(0) as usize;
    if payload.len() > max {
        let kept = payload[payload.len() - max..].trim_start_matches('0');
        return make(vm, &st.decimal, s.sign, Value::str(kept), s.exp.clone(), true);
    }
    dec_value(vm, st, &Dec::Spec(s.clone()))
}

fn d_fix(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let st = state(vm)?;
    if a.len() == 2 && k.is_empty() {
        if let Some(d) = read(&st, &a[0]) {
            if let Some(ctx) = context(vm, &st, a, 1, k)? {
                return match &d {
                    Dec::Fin(f) => fix(vm, &st, &ctx, f),
                    Dec::Spec(s) if d.nan() != 0 => Ok(fix_nan(vm, &st, &ctx, s)),
                    Dec::Spec(_) => Ok(dec_value(vm, &st, &d)),
                };
            }
        }
    }
    fallback(vm, &st, "_fix", a, k)
}

/// self + other for finite operands, as Decimal.__add__
fn add(vm: &mut Vm, st: &State, ctx: &Ctx, x: &Fin, y: &Fin) -> PyResult<Value> {
    let mut exp = x.exp.min(y.exp);
    let negativezero = ctx.rounding == Rounding::Floor && x.sign != y.sign;
    if x.is_zero() && y.is_zero() {
        let sign = if negativezero { 1 } else { x.sign.min(y.sign) };
        return fix(vm, st, ctx, &Fin { sign, int: Rc::new(PyStr::new("0".into())), exp });
    }
    if x.is_zero() || y.is_zero() {
        let nz = if x.is_zero() { y } else { x };
        exp = exp.max(nz.exp - ctx.prec - 1);
        let ans = rescale(nz, exp, ctx.rounding);
        return fix(vm, st, ctx, &ans);
    }
    // _normalize: line the coefficients up on the smaller exponent, with
    // a much smaller operand replaced by one below the rounding point
    let (mut tmp, mut other, x_is_tmp) = if x.exp < y.exp { (y.clone(), x.clone(), false) } else { (x.clone(), y.clone(), true) };
    let n = tmp.exp + (-1i64).min(tmp.len() - ctx.prec - 2);
    if other.len() + other.exp - 1 < n {
        other.int = Rc::new(PyStr::new("1".into()));
        other.exp = n;
    }
    let tmp_int = scaled(&tmp.int.s, tmp.exp - other.exp);
    tmp.exp = other.exp;
    let other_int = big(&other.int.s);
    let ((s1, i1), (s2, i2)) = if x_is_tmp { ((tmp.sign, tmp_int), (other.sign, other_int)) } else { ((other.sign, other_int), (tmp.sign, tmp_int)) };
    let result_exp = other.exp;
    let (mut s1, mut i1, mut s2, mut i2) = (s1, i1, s2, i2);
    let sign;
    if s1 != s2 {
        if i1 == i2 {
            let zero = Fin { sign: negativezero as i64, int: Rc::new(PyStr::new("0".into())), exp };
            return fix(vm, st, ctx, &zero);
        }
        if i1.cmp(&i2) == std::cmp::Ordering::Less {
            std::mem::swap(&mut i1, &mut i2);
            std::mem::swap(&mut s1, &mut s2);
        }
        if s1 == 1 {
            sign = 1;
            std::mem::swap(&mut s1, &mut s2);
        } else {
            sign = 0;
        }
    } else if s1 == 1 {
        sign = 1;
        s2 = 0;
    } else {
        sign = 0;
    }
    let total = if s2 == 0 { i1.add(&i2) } else { i1.sub(&i2) };
    let r = Fin { sign, int: Rc::new(PyStr::new(total.to_radix(10))), exp: result_exp };
    fix(vm, st, ctx, &r)
}

fn mul(vm: &mut Vm, st: &State, ctx: &Ctx, x: &Fin, y: &Fin) -> PyResult<Value> {
    let sign = x.sign ^ y.sign;
    let exp = x.exp + y.exp;
    let int = if x.is_zero() || y.is_zero() {
        "0".to_string()
    } else if &*x.int.s == "1" {
        y.int.s.clone()
    } else if &*y.int.s == "1" {
        x.int.s.clone()
    } else if x.int.s.len() + y.int.s.len() <= 38 {
        let (a, b): (u128, u128) = (x.int.s.parse().unwrap(), y.int.s.parse().unwrap());
        (a * b).to_string()
    } else {
        big(&x.int.s).mul(&big(&y.int.s)).to_radix(10)
    };
    fix(vm, st, ctx, &Fin { sign, int: Rc::new(PyStr::new(int)), exp })
}

/// keyword arguments the operators take: at most context
fn only_context(k: Kw) -> bool {
    k.iter().all(|(n, _)| &*n.s == "context") && k.len() <= 1
}

#[derive(Clone, Copy, PartialEq)]
enum Op {
    Add,
    Sub,
    Mul,
}

/// Decimal._check_nans: the NaN result when either operand is one
fn check_nans(vm: &mut Vm, st: &State, ctx: &Ctx, x: (&Dec, &Value), y: (&Dec, &Value)) -> PyResult<Option<Value>> {
    let (xn, yn) = (x.0.nan(), y.0.nan());
    if xn == 0 && yn == 0 {
        return Ok(None);
    }
    let ans = if xn == 2 {
        raise_error(vm, st, ctx, "InvalidOperation", vec![Value::str("sNaN"), x.1.clone()])?
    } else if yn == 2 {
        raise_error(vm, st, ctx, "InvalidOperation", vec![Value::str("sNaN"), y.1.clone()])?
    } else {
        let Dec::Spec(s) = (if xn != 0 { x.0 } else { y.0 }) else { unreachable!() };
        fix_nan(vm, st, ctx, s)
    };
    Ok(if vm.truthy(&ans)? { Some(ans) } else { None })
}

/// x op y, as Decimal.__add__, __sub__ and __mul__ once the other operand
/// is converted and the context known
fn arith(vm: &mut Vm, st: &State, ctx: &Ctx, op: Op, x: (&Dec, &Value), y: (&Dec, &Value)) -> PyResult<Value> {
    if let (Dec::Fin(a), Dec::Fin(b)) = (x.0, y.0) {
        return match op {
            Op::Add => add(vm, st, ctx, a, b),
            Op::Sub => add(vm, st, ctx, a, &Fin { sign: 1 - b.sign, ..b.clone() }),
            Op::Mul => mul(vm, st, ctx, a, b),
        };
    }
    if let Some(ans) = check_nans(vm, st, ctx, x, y)? {
        return Ok(ans);
    }
    let (a, b) = (x.0, y.0);
    match op {
        Op::Add | Op::Sub => {
            // self - other is self + other.copy_negate()
            let b = if op == Op::Sub { b.negated() } else { b.clone() };
            if a.is_inf() {
                if a.sign() != b.sign() && b.is_inf() {
                    return raise_error(vm, st, ctx, "InvalidOperation", vec![Value::str("-INF + INF")]);
                }
                return Ok(dec_value(vm, st, a));
            }
            Ok(dec_value(vm, st, &b))
        }
        Op::Mul => {
            let sign = a.sign() ^ b.sign();
            let (msg, zero) = if a.is_inf() { ("(+-)INF * 0", b.is_zero()) } else { ("0 * (+-)INF", a.is_zero()) };
            if zero {
                return raise_error(vm, st, ctx, "InvalidOperation", vec![Value::str(msg)]);
            }
            vm.getitem(&st.infinities, &Value::Int(sign))
        }
    }
}

/// a binary operator: the native code when the operands and the context are
/// plain, else the Python method; `reflected` computes other op self
fn binary(vm: &mut Vm, a: &[Value], k: Kw, name: &str, op: Op, reflected: bool) -> PyResult<Value> {
    let st = state(vm)?;
    if (2..=3).contains(&a.len()) && only_context(k) {
        if let Some(x) = read(&st, &a[0]) {
            match convert_other(vm, &st, &a[1]) {
                Some(Other::NotImpl) => return Ok(Value::NotImplemented),
                Some(Other::Dec(y)) => {
                    if let Some(ctx) = context(vm, &st, a, 2, k)? {
                        let (x, y) = ((&x, &a[0]), (&y, &a[1]));
                        return if reflected { arith(vm, &st, &ctx, op, y, x) } else { arith(vm, &st, &ctx, op, x, y) };
                    }
                }
                None => {}
            }
        }
    }
    fallback(vm, &st, name, a, k)
}

fn d_add(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    binary(vm, a, k, "__add__", Op::Add, false)
}

fn d_sub(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    binary(vm, a, k, "__sub__", Op::Sub, false)
}

fn d_mul(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    binary(vm, a, k, "__mul__", Op::Mul, false)
}

/// other - self: the converted other's __sub__
fn d_rsub(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    binary(vm, a, k, "__rsub__", Op::Sub, true)
}

fn d_bool(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let st = state(vm)?;
    if a.len() == 1 {
        if let Some(d) = read(&st, &a[0]) {
            return Ok(Value::Bool(!d.is_zero()));
        }
    }
    fallback(vm, &st, "__bool__", a, k)
}

/// Decimal.__str__ in scientific notation (eng=False) of a finite value
fn sci(d: &Fin, capitals: bool) -> String {
    let int = &*d.int.s;
    let left = d.exp + d.len();
    let dot = if d.exp <= 0 && left > -6 { left } else { 1 };
    let mut s = String::with_capacity(int.len() + 8);
    if d.sign == 1 {
        s.push('-');
    }
    if dot <= 0 {
        s.push_str("0.");
        s.push_str(&"0".repeat((-dot) as usize));
        s.push_str(int);
    } else if dot >= d.len() {
        s.push_str(int);
        s.push_str(&"0".repeat((dot - d.len()) as usize));
    } else {
        s.push_str(&int[..dot as usize]);
        s.push('.');
        s.push_str(&int[dot as usize..]);
    }
    if left != dot {
        s.push(if capitals { 'E' } else { 'e' });
        s.push_str(&format!("{:+}", left - dot));
    }
    s
}

/// whether the exponent needs writing, which is when the context matters
fn needs_exp(d: &Fin) -> bool {
    !(d.exp <= 0 && d.exp + d.len() > -6)
}

fn capitals(vm: &mut Vm, st: &State, ctx: Option<Value>) -> PyResult<Option<bool>> {
    let ctx = match ctx {
        Some(c) if !c.is_none() => c,
        _ => getcontext(vm, st)?,
    };
    Ok(int_attr(vm, st, &ctx, "capitals")?.map(|c| c != 0))
}

/// str(d), or None when the context's capitals is not plain
fn dec_str(vm: &mut Vm, st: &State, d: &Dec) -> PyResult<Option<String>> {
    match d {
        Dec::Fin(f) => {
            let caps = if needs_exp(f) { capitals(vm, st, None)? } else { Some(true) };
            Ok(caps.map(|caps| sci(f, caps)))
        }
        Dec::Spec(s) => {
            let sign = if s.sign == 1 { "-" } else { "" };
            let payload = s.int.as_str().unwrap_or("");
            Ok(Some(match s.kind {
                b'F' => format!("{}Infinity", sign),
                b'n' => format!("{}NaN{}", sign, payload),
                _ => format!("{}sNaN{}", sign, payload),
            }))
        }
    }
}

fn d_str(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let st = state(vm)?;
    if a.len() == 1 && k.is_empty() {
        if let Some(d) = read(&st, &a[0]) {
            if let Some(s) = dec_str(vm, &st, &d)? {
                return Ok(Value::string(s));
            }
        }
    }
    fallback(vm, &st, "__str__", a, k)
}

fn d_repr(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let st = state(vm)?;
    // read() takes only Decimal itself, whose __str__ is the native one
    if a.len() == 1 && k.is_empty() {
        if let Some(d) = read(&st, &a[0]) {
            if let Some(s) = dec_str(vm, &st, &d)? {
                return Ok(Value::string(format!("Decimal('{}')", s)));
            }
        }
    }
    fallback(vm, &st, "__repr__", a, k)
}

/// a numeric string as the decimal regex reads it, ASCII only: sign,
/// coefficient digits (leading zeros stripped) and exponent of a finite
/// number, or None for anything else
fn parse_finite(s: &str) -> Option<(i64, String, i64)> {
    // str.strip(): the ASCII characters str.isspace() accepts
    let t = s.trim_matches(|c: char| matches!(c, ' ' | '\t'..='\r' | '\x1c'..='\x1f'));
    let b: Vec<u8> = t.bytes().filter(|&c| c != b'_').collect();
    let (sign, mut rest) = match b.first() {
        Some(b'-') => (1, &b[1..]),
        Some(b'+') => (0, &b[1..]),
        _ => (0, &b[..]),
    };
    let int_end = rest.iter().position(|c| !c.is_ascii_digit()).unwrap_or(rest.len());
    let int_part = &rest[..int_end];
    rest = &rest[int_end..];
    let mut frac: &[u8] = &[];
    if rest.first() == Some(&b'.') {
        let f = &rest[1..];
        let end = f.iter().position(|c| !c.is_ascii_digit()).unwrap_or(f.len());
        frac = &f[..end];
        rest = &f[end..];
    }
    if int_part.is_empty() && frac.is_empty() {
        return None;
    }
    let mut exp: i64 = 0;
    if let Some(&e) = rest.first() {
        if e != b'e' && e != b'E' {
            return None;
        }
        let digits = std::str::from_utf8(&rest[1..]).ok()?;
        let body = digits.strip_prefix(['+', '-']).unwrap_or(digits);
        if body.is_empty() || !body.bytes().all(|c| c.is_ascii_digit()) {
            return None;
        }
        exp = digits.parse().ok()?;
    }
    let mut coeff: String = int_part.iter().chain(frac.iter()).map(|&c| c as char).collect();
    let nz = coeff.find(|c| c != '0').unwrap_or(coeff.len().saturating_sub(1));
    coeff.drain(..nz);
    if coeff.is_empty() {
        coeff.push('0');
    }
    Some((sign, coeff, exp.checked_sub(frac.len() as i64)?))
}

/// Decimal(value, context): strings, exact ints and Decimals here
fn d_new(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let st = state(vm)?;
    if (1..=3).contains(&a.len()) && k.is_empty() {
        if let Value::Class(cls) = &a[0] {
            let value = a.get(1).cloned().unwrap_or_else(|| Value::str("0"));
            match &value {
                Value::Str(s) if s.ascii => {
                    if let Some((sign, int, exp)) = parse_finite(&s.s) {
                        return Ok(make(vm, cls, sign, Value::string(int), Value::Int(exp), false));
                    }
                }
                v => {
                    if let Some((sign, int)) = int_digits(vm, v) {
                        return Ok(make(vm, cls, sign, Value::string(int), Value::Int(0), false));
                    }
                    match read(&st, v) {
                        Some(Dec::Fin(d)) => return Ok(make(vm, cls, d.sign, Value::Str(d.int), Value::Int(d.exp), false)),
                        Some(Dec::Spec(d)) => return Ok(make(vm, cls, d.sign, d.int, d.exp, true)),
                        None => {}
                    }
                }
            }
        }
    }
    fallback(vm, &st, "__new__", a, k)
}
