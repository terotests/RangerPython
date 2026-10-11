// SPDX-License-Identifier: MIT
//! _decimal: the interface of CPython's C decimal module
//! (Modules/_decimal/_decimal.c) over the arithmetic of mpdec.rs.
//! Decimal, Context, the signal dicts and localcontext's manager are
//! native objects; DecimalTuple, the signal exceptions and SignalDict are
//! made through Python, as the C module makes them. Each import makes a
//! module with its own types and its own context variable, as CPython's
//! multi-phase module does, so a fresh import (import_fresh_module) stands
//! apart from the first one.

use crate::bigint::BigInt;
use crate::builtins::*;
use crate::mpdec::{self, Ctx, Dec, Kind, Round};
use crate::value::*;
use crate::vm::Vm;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

/// CPython's docstrings and text signatures of the module's types and
/// functions, in the format of docs.txt (see builtins::builtin_doc)
pub static DOCS: &str = include_str!("decimal_docs.txt");

/// a context: the arithmetic settings, the status and trap bits (shared
/// with its flags and traps dicts) and capitals
pub struct DecContext {
    pub ctx: Ctx,
    pub status: Rc<Cell<u32>>,
    pub traps: Rc<Cell<u32>>,
    pub capitals: bool,
    pub flags: Value,
    pub traps_dict: Value,
}

/// one imported _decimal module
struct State {
    /// the module's namespace, by address
    ns: usize,
    decimal: Rc<Class>,
    context: Rc<Class>,
    manager: Rc<Class>,
    mixin: Rc<Class>,
    signal_dict: Rc<Class>,
    tuple: Value,
    rational: Value,
    /// the signal classes in the order of _decimal.c's signal_map
    signals: Vec<Value>,
    signal_tuple: Value,
    /// ConversionSyntax, DivisionImpossible, DivisionUndefined, InvalidContext
    conds: Vec<Value>,
    /// the rounding names, in Round::ALL order
    rounds: Vec<Value>,
    var: Value,
    /// the contextvars module's namespace
    cv_ns: Rc<RefCell<Dict>>,
    /// DefaultContext, BasicContext, ExtendedContext
    templates: RefCell<Vec<Value>>,
    pydecimal: RefCell<Value>,
}

impl State {
    fn owns(&self, c: &Rc<Class>) -> bool {
        [&self.decimal, &self.context, &self.manager, &self.mixin, &self.signal_dict].iter().any(|x| Rc::ptr_eq(x, c))
    }
}

crate::gil_static! {
    static STATES: RefCell<Vec<Rc<State>>> = const { RefCell::new(Vec::new()) };
}

/// the module a value's type, a class or a module function belongs to
fn state(vm: &Vm, v: &Value) -> Rc<State> {
    STATES.with(|s| {
        let s = s.borrow();
        if let Value::Module(m) = v {
            let id = Rc::as_ptr(&m.dict) as usize;
            if let Some(st) = s.iter().find(|st| st.ns == id) {
                return st.clone();
            }
        }
        let cls = match v {
            Value::Class(c) => c.clone(),
            _ => vm.class_of(v),
        };
        if let Some(st) = s.iter().rev().find(|st| st.owns(&cls)) {
            return st.clone();
        }
        let mro = cls.mro.borrow();
        s.iter().rev().find(|st| mro.iter().any(|c| st.owns(c))).unwrap_or_else(|| s.last().expect("_decimal state")).clone()
    })
}

// the signals, in signal_map order
const SIGNAL_FLAGS: [u32; 9] = [mpdec::IEEE_INVALID, mpdec::FLOAT_OPERATION, mpdec::DIVISION_BY_ZERO, mpdec::OVERFLOW, mpdec::UNDERFLOW, mpdec::SUBNORMAL, mpdec::INEXACT, mpdec::ROUNDED, mpdec::CLAMPED];
const SIGNAL_NAMES: [&str; 9] = ["InvalidOperation", "FloatOperation", "DivisionByZero", "Overflow", "Underflow", "Subnormal", "Inexact", "Rounded", "Clamped"];
const COND_FLAGS: [u32; 4] = [mpdec::CONVERSION_SYNTAX, mpdec::DIVISION_IMPOSSIBLE, mpdec::DIVISION_UNDEFINED, mpdec::INVALID_CONTEXT];
const COND_NAMES: [&str; 4] = ["ConversionSyntax", "DivisionImpossible", "DivisionUndefined", "InvalidContext"];
/// the signal each status bit belongs to, for Context.__repr__
const BIT_NAMES: [&str; 15] = ["Clamped", "InvalidOperation", "DivisionByZero", "InvalidOperation", "InvalidOperation", "InvalidOperation", "Inexact", "InvalidOperation", "InvalidOperation", "InvalidOperation", "FloatOperation", "Overflow", "Rounded", "Subnormal", "Underflow"];

const INVALID_ROUNDING: &str = "valid values for rounding are:\n  [ROUND_CEILING, ROUND_FLOOR, ROUND_UP, ROUND_DOWN,\n   ROUND_HALF_UP, ROUND_HALF_DOWN, ROUND_HALF_EVEN,\n   ROUND_05UP]";
const INVALID_SIGNALS: &str = "valid values for signals are:\n  [InvalidOperation, FloatOperation, DivisionByZero,\n   Overflow, Underflow, Subnormal, Inexact, Rounded,\n   Clamped]";

// -------------------------------------------------------------------------
// the native objects

fn new_obj(cls: &Rc<Class>, kind: ObjKind) -> Rc<NativeObj> {
    Rc::new(NativeObj { class: cls.clone(), gc: Default::default(), kind: RefCell::new(kind) })
}

/// the native object behind a value or an instance of a subclass
fn native_of(v: &Value) -> Option<Rc<NativeObj>> {
    match v {
        Value::Obj(o) => Some(o.clone()),
        Value::Instance(i) => match &*i.base.borrow() {
            Value::Obj(o) => Some(o.clone()),
            _ => None,
        },
        _ => None,
    }
}

fn dec_val(v: &Value) -> Option<Rc<Dec>> {
    let o = native_of(v)?;
    let k = o.kind.borrow();
    match &*k {
        ObjKind::Decimal(d) => Some(d.clone()),
        _ => None,
    }
}

/// a Decimal of this module
fn as_dec(vm: &Vm, st: &State, v: &Value) -> Option<Rc<Dec>> {
    if !vm.class_of(v).is_subclass(&st.decimal) {
        return None;
    }
    dec_val(v)
}

/// a native value of `cls`, or an instance of a subclass wrapping it
fn typed(vm: &mut Vm, base: &Rc<Class>, cls: &Rc<Class>, o: Rc<NativeObj>) -> Value {
    if Rc::ptr_eq(base, cls) {
        return Value::Obj(o);
    }
    let inst = vm.new_instance(cls);
    if let Value::Instance(i) = &inst {
        *i.base.borrow_mut() = Value::Obj(o);
    }
    inst
}

fn new_dec(st: &State, d: Dec) -> Value {
    Value::Obj(new_obj(&st.decimal, ObjKind::Decimal(Rc::new(d))))
}

fn new_dec_of(vm: &mut Vm, st: &State, cls: &Rc<Class>, d: Rc<Dec>) -> Value {
    let o = new_obj(&st.decimal, ObjKind::Decimal(d));
    typed(vm, &st.decimal, cls, o)
}

/// self of a Decimal method
/// METH_NOARGS methods reject keyword arguments, which `def_arity` lets through.
fn no_kw(vm: &mut Vm, k: Kw, name: &str) -> PyResult<()> {
    if k.is_empty() {
        Ok(())
    } else {
        Err(vm.type_error(&format!("{name}() takes no keyword arguments")))
    }
}

fn this(vm: &mut Vm, a: &[Value], name: &str) -> PyResult<Rc<Dec>> {
    if let Some(d) = a.first().and_then(dec_val) {
        return Ok(d);
    }
    let tn = a.first().map(|v| vm.type_name(v)).unwrap_or_default();
    Err(vm.type_error(format!("descriptor '{}' for 'decimal.Decimal' objects doesn't apply to a '{}' object", name, tn)))
}

/// what an operation reads from a context, and where its status goes
struct Cur {
    obj: Rc<NativeObj>,
    ctx: Ctx,
    caps: bool,
    status: Rc<Cell<u32>>,
    traps: Rc<Cell<u32>>,
}

fn ctx_native(v: &Value) -> Option<Rc<NativeObj>> {
    let o = native_of(v)?;
    let ok = matches!(&*o.kind.borrow(), ObjKind::DecContext(_));
    ok.then_some(o)
}

fn cur_of(o: &Rc<NativeObj>) -> Cur {
    match &*o.kind.borrow() {
        ObjKind::DecContext(c) => Cur { obj: o.clone(), ctx: c.ctx, caps: c.capitals, status: c.status.clone(), traps: c.traps.clone() },
        _ => unreachable!("not a context"),
    }
}

/// a Context of this module
fn as_ctx(vm: &Vm, st: &State, v: &Value) -> Option<Cur> {
    if !vm.class_of(v).is_subclass(&st.context) {
        return None;
    }
    ctx_native(v).map(|o| cur_of(&o))
}

/// self of a Context method
fn this_ctx(vm: &mut Vm, a: &[Value], name: &str) -> PyResult<Cur> {
    if let Some(o) = a.first().and_then(ctx_native) {
        return Ok(cur_of(&o));
    }
    let tn = a.first().map(|v| vm.type_name(v)).unwrap_or_default();
    Err(vm.type_error(format!("descriptor '{}' for 'decimal.Context' objects doesn't apply to a '{}' object", name, tn)))
}

fn with_ctx<R>(o: &Rc<NativeObj>, f: impl FnOnce(&mut DecContext) -> R) -> R {
    match &mut *o.kind.borrow_mut() {
        ObjKind::DecContext(c) => f(c),
        _ => unreachable!("not a context"),
    }
}

// -------------------------------------------------------------------------
// the current context

/// contextvars' current mapping, while it is the plain one contextvars.py
/// keeps
fn cv_data(vm: &mut Vm, st: &State) -> Option<Rc<RefCell<Dict>>> {
    let f = st.cv_ns.borrow().get_s("_cur")?;
    let cur = vm.call_vec(&f, vec![], &[]).ok()?;
    let Value::Instance(i) = cur else {
        return None;
    };
    let d = i.with_attrs(|d| d.get_s("_data"));
    match d {
        Some(Value::Dict(d)) => Some(d),
        _ => None,
    }
}

fn set_current(vm: &mut Vm, st: &State, v: Value) -> PyResult<()> {
    match cv_data(vm, st) {
        Some(d) => vm.dict_set(&d, st.var.clone(), v),
        None => {
            let f = vm.getattr(&st.var, &ps("set"))?;
            vm.call_vec(&f, vec![v], &[])?;
            Ok(())
        }
    }
}

/// getcontext(): the context variable's value, set to a copy of
/// DefaultContext on first use
fn current(vm: &mut Vm, st: &State) -> PyResult<Value> {
    let found = match cv_data(vm, st) {
        Some(d) => vm.dict_get(&d, &st.var)?,
        None => {
            let f = vm.getattr(&st.var, &ps("get"))?;
            Some(vm.call_vec(&f, vec![Value::None], &[])?).filter(|v| !v.is_none())
        }
    };
    if let Some(c) = found {
        return Ok(c);
    }
    let tpl = st.templates.borrow()[0].clone();
    let c = context_copy(vm, st, &tpl)?;
    with_ctx(&ctx_native(&c).unwrap(), |x| x.status.set(0));
    set_current(vm, st, c.clone())?;
    Ok(c)
}

fn cur(vm: &mut Vm, st: &State) -> PyResult<Cur> {
    let c = current(vm, st)?;
    match ctx_native(&c) {
        Some(o) => Ok(cur_of(&o)),
        None => Err(vm.type_error("the current decimal context is not a Context")),
    }
}

/// an optional context argument: None for the current context
fn ctx_arg(vm: &mut Vm, st: &State, v: Option<&Value>) -> PyResult<Cur> {
    match v {
        None | Some(Value::None) => cur(vm, st),
        Some(v) => match as_ctx(vm, st, v) {
            Some(c) => Ok(c),
            None => Err(vm.type_error("optional argument must be a context")),
        },
    }
}

/// dec_addstatus: records the status and raises the first trapped signal
fn add_status(vm: &mut Vm, st: &State, c: &Cur, status: u32) -> PyResult<()> {
    c.status.set(c.status.get() | status);
    if status & mpdec::MALLOC_ERROR != 0 {
        return Err(no_memory(vm));
    }
    let trapped = status & c.traps.get();
    if trapped == 0 {
        return Ok(());
    }
    let ex = SIGNAL_FLAGS.iter().position(|f| trapped & f != 0).map(|i| st.signals[i].clone()).unwrap();
    let list = Value::list(flags_as_list(st, trapped));
    let e = vm.call_vec(&ex, vec![list], &[])?;
    Err(Exc { value: e })
}

fn no_memory(vm: &mut Vm) -> Exc {
    let cls = vm.exc_class("MemoryError");
    Exc { value: vm.new_exc(&cls, vec![]) }
}

/// the conditions and signals of `flags`, as an exception's argument
fn flags_as_list(st: &State, flags: u32) -> Vec<Value> {
    let mut out = vec![];
    if flags & mpdec::INVALID_OPERATION != 0 {
        out.push(st.signals[0].clone());
    }
    for (i, f) in COND_FLAGS.iter().enumerate() {
        if flags & f != 0 {
            out.push(st.conds[i].clone());
        }
    }
    for i in 1..9 {
        if flags & SIGNAL_FLAGS[i] != 0 {
            out.push(st.signals[i].clone());
        }
    }
    out
}

fn signals_as_list(st: &State, flags: u32) -> Value {
    Value::list((0..9).filter(|&i| flags & SIGNAL_FLAGS[i] != 0).map(|i| st.signals[i].clone()).collect())
}

/// the flag of a signal class; KeyError for anything else
fn signal_flag(vm: &mut Vm, st: &State, v: &Value) -> PyResult<u32> {
    match st.signals.iter().position(|s| s.is(v)) {
        Some(i) => Ok(SIGNAL_FLAGS[i]),
        None => Err(vm.key_error(Value::str(INVALID_SIGNALS))),
    }
}

fn list_as_flags(vm: &mut Vm, st: &State, v: &Value) -> PyResult<u32> {
    let mut flags = 0;
    for x in vm.to_vec(v)? {
        flags |= signal_flag(vm, st, &x)?;
    }
    Ok(flags)
}

/// the flags a signal dict or a {signal: bool} dict of all nine holds
fn dict_as_flags(vm: &mut Vm, st: &State, v: &Value) -> PyResult<u32> {
    if let Some(cell) = signals_of(vm, st, v) {
        return match cell {
            Some(c) => Ok(c.get()),
            None => Err(vm.value_error("invalid signal dict")),
        };
    }
    let Value::Dict(d) = vm.base_of(v) else {
        return Err(vm.type_error("argument must be a signal dict"));
    };
    if d.borrow().len() != 9 {
        return Err(vm.key_error(Value::str("invalid signal dict")));
    }
    let mut flags = 0;
    for i in 0..9 {
        let b = match vm.dict_get(&d, &st.signals[i])? {
            Some(b) => b,
            None => return Err(vm.key_error(Value::str("invalid signal dict"))),
        };
        if vm.truthy(&b)? {
            flags |= SIGNAL_FLAGS[i];
        }
    }
    Ok(flags)
}

// -------------------------------------------------------------------------
// arguments

/// PyArg_ParseTupleAndKeywords: the arguments by name, the positional ones
/// first, `req` of them required
fn parse_args(vm: &mut Vm, a: &[Value], k: Kw, names: &[&str], req: usize) -> PyResult<Vec<Option<Value>>> {
    let mut out: Vec<Option<Value>> = vec![None; names.len()];
    if a.len() > names.len() {
        let n = names.len();
        return Err(vm.type_error(format!("function takes at most {} argument{} ({} given)", n, if n == 1 { "" } else { "s" }, a.len() + k.len())));
    }
    for (i, v) in a.iter().enumerate() {
        out[i] = Some(v.clone());
    }
    for (n, v) in k {
        match names.iter().position(|x| *x == &*n.s) {
            Some(i) if out[i].is_some() => return Err(vm.type_error(format!("argument for function given by name ('{}') and position ({})", n.s, i + 1))),
            Some(i) => out[i] = Some(v.clone()),
            None => return Err(vm.type_error(format!("'{}' is an invalid keyword argument for this function", n.s))),
        }
    }
    for i in 0..req {
        if out[i].is_none() {
            return Err(vm.type_error(format!("function missing required argument '{}' (pos {})", names[i], i + 1)));
        }
    }
    Ok(out)
}

/// PyArg_ParseTuple with exactly `n` arguments besides self
fn exact_args(vm: &mut Vm, a: &[Value], k: Kw, name: &str, n: usize) -> PyResult<()> {
    if !k.is_empty() {
        return Err(vm.type_error(format!("{}() takes no keyword arguments", name)));
    }
    let got = a.len().saturating_sub(1);
    if got != n {
        return Err(vm.type_error(format!("function takes exactly {} argument{} ({} given)", n, if n == 1 { "" } else { "s" }, got)));
    }
    Ok(())
}

/// PyArg_ParseTuple with lo..=hi arguments besides self
fn args_range(vm: &mut Vm, a: &[Value], k: Kw, lo: usize, hi: usize) -> PyResult<()> {
    if !k.is_empty() {
        return Err(vm.type_error("function takes no keyword arguments"));
    }
    let got = a.len().saturating_sub(1);
    if got < lo || got > hi {
        let (what, n) = if got < lo { ("at least", lo) } else { ("at most", hi) };
        return Err(vm.type_error(format!("function takes {} {} argument{} ({} given)", what, n, if n == 1 { "" } else { "s" }, got)));
    }
    Ok(())
}

/// PyLong_AsSsize_t
fn ssize(vm: &mut Vm, v: &Value) -> PyResult<i64> {
    match vm.base_of(v) {
        Value::Int(i) => Ok(i),
        Value::Bool(b) => Ok(b as i64),
        Value::Big(_) => Err(vm.err("OverflowError", "Python int too large to convert to C ssize_t")),
        _ => {
            let tn = vm.type_name(v);
            Err(vm.type_error(format!("'{}' object cannot be interpreted as an integer", tn)))
        }
    }
}

fn is_int(vm: &Vm, v: &Value) -> bool {
    matches!(vm.base_of(v), Value::Int(_) | Value::Big(_) | Value::Bool(_))
}

/// a rounding name: ROUND_UP .. ROUND_05UP
fn get_round(vm: &mut Vm, st: &State, v: &Value) -> PyResult<Round> {
    if let Value::Str(s) = vm.base_of(v) {
        if let Some(i) = st.rounds.iter().position(|r| r.is(v)) {
            return Ok(Round::ALL[i]);
        }
        if let Some(r) = Round::ALL.iter().find(|r| r.name() == s.s) {
            return Ok(*r);
        }
    }
    Err(vm.type_error(INVALID_ROUNDING))
}

// -------------------------------------------------------------------------
// conversions to Decimal

fn int_dec(vm: &Vm, v: &Value) -> Option<Dec> {
    match vm.base_of(v) {
        Value::Int(i) => Some(Dec::from_i64(i)),
        Value::Bool(b) => Some(Dec::from_i64(b as i64)),
        Value::Big(b) => Some(Dec::from_bigint(&b)),
        _ => None,
    }
}

fn float_of(vm: &Vm, v: &Value) -> Option<f64> {
    match vm.base_of(v) {
        Value::Float(f) => Some(f),
        _ => None,
    }
}

/// a float, exactly: its integer ratio n / 2**k as n * 5**k at exponent -k
fn float_dec(x: f64) -> Dec {
    if x.is_nan() {
        return Dec::nan();
    }
    if x.is_infinite() {
        return Dec::inf(x < 0.0);
    }
    let sign = x.is_sign_negative();
    let bits = x.abs().to_bits();
    let (eb, frac) = ((bits >> 52) as i64, bits & ((1 << 52) - 1));
    let (mut m, mut e) = if eb == 0 { (frac, -1074) } else { (frac | 1 << 52, eb - 1075) };
    if m == 0 {
        return Dec::triple(sign, 0, 0);
    }
    while e < 0 && m & 1 == 0 {
        m >>= 1;
        e += 1;
    }
    let n = BigInt::from_u128(m as u128, false);
    if e >= 0 {
        return Dec::fin(sign, n.shl(e as usize), 0);
    }
    Dec::fin(sign, n.mul(&BigInt::from_i64(5).pow(-e as u64)), e)
}

/// numeric_as_ascii: Unicode digits and spaces made ASCII; anything else
/// outside ASCII makes the string empty, a syntax error
fn numeric_as_ascii(s: &str, strip_ws: bool, ignore_underscores: bool) -> String {
    let is_space = |c: char| crate::unidata::is_in(&crate::unidata::SPACE, c);
    let s = if strip_ws { s.trim_matches(is_space) } else { s };
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        if ignore_underscores && ch == '_' {
            continue;
        }
        if ch != '\0' && ch.is_ascii() {
            out.push(ch);
        } else if ch != '\0' && is_space(ch) {
            out.push(' ');
        } else {
            match crate::strmethods::decimal_value(ch) {
                Some(d) if ch != '\0' => out.push((b'0' + d as u8) as char),
                _ => return String::new(),
            }
        }
    }
    out
}

/// dectuple_as_str: (sign, digits, exponent) as a numeric string
fn tuple_as_str(vm: &mut Vm, v: &Value) -> PyResult<String> {
    let t = vm.to_vec(v)?;
    if t.len() != 3 {
        return Err(vm.value_error("argument must be a sequence of length 3"));
    }
    let sign = match vm.base_of(&t[0]) {
        Value::Int(0) | Value::Bool(false) => false,
        Value::Int(1) | Value::Bool(true) => true,
        _ => return Err(vm.value_error("sign must be an integer with the value 0 or 1")),
    };
    let mut s = String::from(if sign { "-" } else { "+" });
    let mut special = true;
    let mut inf = false;
    let mut exp = 0;
    match vm.base_of(&t[2]) {
        Value::Str(e) => match &*e.s {
            "F" => {
                s.push_str("Inf");
                inf = true;
            }
            "n" => s.push_str("NaN"),
            "N" => s.push_str("sNaN"),
            _ => return Err(vm.value_error("string argument in the third position must be 'F', 'n' or 'N'")),
        },
        e @ (Value::Int(_) | Value::Big(_) | Value::Bool(_)) => {
            special = false;
            exp = ssize(vm, &e)?;
        }
        _ => return Err(vm.value_error("exponent must be an integer")),
    }
    if !is_seq(vm, &t[1]) {
        return Err(vm.value_error("coefficient must be a tuple of digits"));
    }
    let digits = vm.to_vec(&t[1])?;
    if digits.is_empty() && !special {
        s.push('0');
    }
    for d in &digits {
        let l = match vm.base_of(d) {
            Value::Int(l) => l,
            Value::Bool(b) => b as i64,
            _ => return Err(vm.value_error("coefficient must be a tuple of digits")),
        };
        if !(0..=9).contains(&l) {
            return Err(vm.value_error("coefficient must be a tuple of digits"));
        }
        if !inf {
            s.push((b'0' + l as u8) as char);
        }
    }
    if !special {
        s.push_str(&format!("E{}", exp));
    }
    Ok(s)
}

fn str_of_value(vm: &Vm, v: &Value) -> Option<Rc<PyStr>> {
    match vm.base_of(v) {
        Value::Str(s) => Some(s),
        _ => None,
    }
}

fn is_seq(vm: &Vm, v: &Value) -> bool {
    matches!(vm.base_of(v), Value::Tuple(_) | Value::List(_))
}

fn unsupported(vm: &mut Vm, v: &Value) -> Exc {
    let tn = vm.type_name(v);
    vm.type_error(format!("conversion from {} to Decimal is not supported", tn))
}

/// PyDecType_FromObjectExact: Decimal(v, context)
fn from_object_exact(vm: &mut Vm, st: &State, v: Option<&Value>, c: &Cur) -> PyResult<Rc<Dec>> {
    let Some(v) = v else {
        return Ok(Rc::new(Dec::from_i64(0)));
    };
    if let Some(d) = as_dec(vm, st, v) {
        return Ok(d);
    }
    let mut status = 0;
    let d = if let Some(s) = str_of_value(vm, v) {
        mpdec::parse_exact(&numeric_as_ascii(&s.s, true, true), &mut status)
    } else if let Some(d) = int_dec(vm, v) {
        d
    } else if is_seq(vm, v) {
        let s = tuple_as_str(vm, v)?;
        mpdec::parse_exact(&s, &mut status)
    } else if let Some(f) = float_of(vm, v) {
        add_status(vm, st, c, mpdec::FLOAT_OPERATION)?;
        float_dec(f)
    } else {
        return Err(unsupported(vm, v));
    };
    add_status(vm, st, c, status)?;
    Ok(Rc::new(d))
}

/// PyDec_FromObject: context.create_decimal(v)
fn from_object(vm: &mut Vm, st: &State, v: Option<&Value>, c: &Cur) -> PyResult<Dec> {
    let Some(v) = v else {
        return Ok(Dec::from_i64(0));
    };
    let mut status = 0;
    let d = if let Some(d) = as_dec(vm, st, v) {
        if d.is_nan() && d.digits > c.ctx.prec - c.ctx.clamp as i64 {
            // too many payload digits
            add_status(vm, st, c, mpdec::CONVERSION_SYNTAX)?;
            return Ok(Dec::nan());
        }
        let mut r = (*d).clone();
        mpdec::finalize(&mut r, &c.ctx, &mut status);
        r
    } else if let Some(s) = str_of_value(vm, v) {
        mpdec::parse(&numeric_as_ascii(&s.s, false, false), &c.ctx, &mut status)
    } else if let Some(mut d) = int_dec(vm, v) {
        mpdec::finalize(&mut d, &c.ctx, &mut status);
        d
    } else if is_seq(vm, v) {
        let s = tuple_as_str(vm, v)?;
        mpdec::parse(&s, &c.ctx, &mut status)
    } else if let Some(f) = float_of(vm, v) {
        add_status(vm, st, c, mpdec::FLOAT_OPERATION)?;
        let mut d = float_dec(f);
        mpdec::finalize(&mut d, &c.ctx, &mut status);
        d
    } else {
        return Err(unsupported(vm, v));
    };
    add_status(vm, st, c, status)?;
    Ok(d)
}

/// convert_op: a Decimal or an int; None (NotImplemented) or a TypeError
/// for anything else
fn convert(vm: &mut Vm, st: &State, v: &Value, type_err: bool) -> PyResult<Option<Rc<Dec>>> {
    if let Some(d) = as_dec(vm, st, v) {
        return Ok(Some(d));
    }
    if let Some(d) = int_dec(vm, v) {
        return Ok(Some(Rc::new(d)));
    }
    if type_err {
        return Err(unsupported(vm, v));
    }
    Ok(None)
}

fn convert_raise(vm: &mut Vm, st: &State, v: &Value) -> PyResult<Rc<Dec>> {
    Ok(convert(vm, st, v, true)?.unwrap())
}

#[derive(Clone, Copy, PartialEq)]
enum Cmp {
    Lt,
    Le,
    Eq,
    Ne,
    Gt,
    Ge,
}

/// convert_op_cmp: v (self) and w made comparable; floats, complex numbers
/// with no imaginary part and Rationals too. None for NotImplemented.
fn convert_cmp(vm: &mut Vm, st: &State, c: &Cur, v: &Rc<Dec>, w: &Value, op: Cmp) -> PyResult<Option<(Rc<Dec>, Rc<Dec>)>> {
    let eq = matches!(op, Cmp::Eq | Cmp::Ne);
    if let Some(d) = as_dec(vm, st, w) {
        return Ok(Some((v.clone(), d)));
    }
    if let Some(d) = int_dec(vm, w) {
        return Ok(Some((v.clone(), Rc::new(d))));
    }
    if let Some(f) = float_of(vm, w) {
        if !eq {
            add_status(vm, st, c, mpdec::FLOAT_OPERATION)?;
        } else {
            c.status.set(c.status.get() | mpdec::FLOAT_OPERATION);
        }
        return Ok(Some((v.clone(), Rc::new(float_dec(f)))));
    }
    if let Value::Complex(z) = vm.base_of(w) {
        if !eq || z.1 != 0.0 {
            return Ok(None);
        }
        c.status.set(c.status.get() | mpdec::FLOAT_OPERATION);
        return Ok(Some((v.clone(), Rc::new(float_dec(z.0)))));
    }
    if crate::builtins::isinstance_of(vm, w, &st.rational)? {
        let n = vm.getattr(w, &ps("numerator"))?;
        let num = match int_dec(vm, &n) {
            Some(d) => d,
            None => return Err(unsupported(vm, &n)),
        };
        if v.is_special() {
            return Ok(Some((v.clone(), Rc::new(num))));
        }
        let den = vm.getattr(w, &ps("denominator"))?;
        let den = match int_dec(vm, &den) {
            Some(d) => d,
            None => return Err(unsupported(vm, &den)),
        };
        // v * denominator at v's exponent: compared, never rounded
        let d = Dec::fin(v.sign, v.coef.mul(&den.coef), v.exp);
        return Ok(Some((Rc::new(d), Rc::new(num))));
    }
    Ok(None)
}

// -------------------------------------------------------------------------
// Decimal

fn d_new(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let Some(Value::Class(cls)) = a.first().cloned() else {
        return Err(vm.type_error("Decimal.__new__(X): X is not a type object"));
    };
    let st = state(vm, &a[0]);
    let p = parse_args(vm, &a[1..], k, &["value", "context"], 0)?;
    let c = ctx_arg(vm, &st, p[1].as_ref())?;
    let v = p[0].as_ref();
    if let Some(v) = v {
        // Decimal(d) is d itself
        if Rc::ptr_eq(&cls, &st.decimal) && matches!(v, Value::Obj(o) if Rc::ptr_eq(&o.class, &st.decimal)) {
            return Ok(v.clone());
        }
    }
    let d = from_object_exact(vm, &st, v, &c)?;
    Ok(new_dec_of(vm, &st, &cls, d))
}

fn d_from_float(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    no_kw(vm, k, "from_float")?;
    let Value::Class(cls) = a[0].clone() else {
        return Err(vm.type_error("from_float() needs a class"));
    };
    let st = state(vm, &a[0]);
    let d = from_float_exact(vm, &a[1])?;
    let r = new_dec(&st, d);
    if Rc::ptr_eq(&cls, &st.decimal) {
        return Ok(r);
    }
    vm.call_vec(&a[0], vec![r], &[])
}

/// PyDecType_FromFloatExact: an int or a float, exactly
fn from_float_exact(vm: &mut Vm, v: &Value) -> PyResult<Dec> {
    if let Some(d) = int_dec(vm, v) {
        return Ok(d);
    }
    match float_of(vm, v) {
        Some(f) => Ok(float_dec(f)),
        None => Err(vm.type_error("argument must be int or float")),
    }
}

fn d_str(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let d = this(vm, a, "__str__")?;
    let st = state(vm, &a[0]);
    let c = cur(vm, &st)?;
    Ok(Value::string(mpdec::to_sci(&d, c.caps)))
}

fn d_repr(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let d = this(vm, a, "__repr__")?;
    let st = state(vm, &a[0]);
    let c = cur(vm, &st)?;
    Ok(Value::string(format!("Decimal('{}')", mpdec::to_sci(&d, c.caps))))
}

/// the arithmetic of an operation on its converted operands
type Unary = fn(&Dec, &Ctx, &mut u32) -> Dec;
type Binary = fn(&Dec, &Dec, &Ctx, &mut u32) -> Dec;

fn run1(vm: &mut Vm, st: &State, c: &Cur, x: &Dec, op: Unary) -> PyResult<Value> {
    let mut status = 0;
    let r = op(x, &c.ctx, &mut status);
    add_status(vm, st, c, status)?;
    Ok(new_dec(st, r))
}

fn run2(vm: &mut Vm, st: &State, c: &Cur, x: &Dec, y: &Dec, op: Binary) -> PyResult<Value> {
    let mut status = 0;
    let r = op(x, y, &c.ctx, &mut status);
    add_status(vm, st, c, status)?;
    Ok(new_dec(st, r))
}

/// the operands of a number method in operator order; None when one does
/// not convert (NotImplemented)
fn operands(vm: &mut Vm, st: &State, a: &[Value], reflected: bool) -> PyResult<Option<(Rc<Dec>, Rc<Dec>)>> {
    let (x, y) = if reflected { (&a[1], &a[0]) } else { (&a[0], &a[1]) };
    let Some(x) = convert(vm, st, x, false)? else {
        return Ok(None);
    };
    let Some(y) = convert(vm, st, y, false)? else {
        return Ok(None);
    };
    Ok(Some((x, y)))
}

fn number2(vm: &mut Vm, a: &[Value], op: Binary, reflected: bool) -> PyResult<Value> {
    let st = state(vm, &a[0]);
    let Some((x, y)) = operands(vm, &st, a, reflected)? else {
        return Ok(Value::NotImplemented);
    };
    let c = cur(vm, &st)?;
    run2(vm, &st, &c, &x, &y, op)
}

fn number1(vm: &mut Vm, a: &[Value], op: Unary, name: &str) -> PyResult<Value> {
    let d = this(vm, a, name)?;
    let st = state(vm, &a[0]);
    let c = cur(vm, &st)?;
    run1(vm, &st, &c, &d, op)
}

macro_rules! number_methods {
    ($( $f:ident, $r:ident => $op:path; )*) => {
        $(
            fn $f(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
                number2(vm, a, $op, false)
            }
            fn $r(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
                number2(vm, a, $op, true)
            }
        )*
    };
}

number_methods! {
    d_add, d_radd => mpdec::add;
    d_sub, d_rsub => mpdec::sub;
    d_mul, d_rmul => mpdec::mul;
    d_truediv, d_rtruediv => mpdec::div;
    d_floordiv, d_rfloordiv => mpdec::divint;
    d_mod, d_rmod => mpdec::rem;
}

fn d_neg(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    number1(vm, a, mpdec::minus, "__neg__")
}
fn d_pos(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    number1(vm, a, mpdec::plus, "__pos__")
}
fn d_abs(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    number1(vm, a, mpdec::abs, "__abs__")
}

fn divmod2(vm: &mut Vm, a: &[Value], reflected: bool) -> PyResult<Value> {
    let st = state(vm, &a[0]);
    let Some((x, y)) = operands(vm, &st, a, reflected)? else {
        return Ok(Value::NotImplemented);
    };
    let c = cur(vm, &st)?;
    let mut status = 0;
    let (q, r) = mpdec::divmod(&x, &y, &c.ctx, &mut status);
    add_status(vm, &st, &c, status)?;
    Ok(Value::tuple(vec![new_dec(&st, q), new_dec(&st, r)]))
}

fn d_divmod(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    divmod2(vm, a, false)
}
fn d_rdivmod(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    divmod2(vm, a, true)
}

/// nm_mpd_qpow: base ** exp, or pow(base, exp, mod)
fn pow3(vm: &mut Vm, a: &[Value], reflected: bool) -> PyResult<Value> {
    let st = state(vm, &a[0]);
    let Some((x, y)) = operands(vm, &st, a, reflected)? else {
        return Ok(Value::NotImplemented);
    };
    let m = match a.get(2) {
        None | Some(Value::None) => None,
        Some(m) => match convert(vm, &st, m, false)? {
            Some(m) => Some(m),
            None => return Ok(Value::NotImplemented),
        },
    };
    let c = cur(vm, &st)?;
    let mut status = 0;
    let r = match m {
        None => mpdec::pow(&x, &y, &c.ctx, &mut status),
        Some(m) => mpdec::powmod(&x, &y, &m, &c.ctx, &mut status),
    };
    add_status(vm, &st, &c, status)?;
    Ok(new_dec(&st, r))
}

fn d_pow(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    pow3(vm, a, false)
}
fn d_rpow(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    pow3(vm, a, true)
}

fn d_bool(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let d = this(vm, a, "__bool__")?;
    Ok(Value::Bool(!d.is_zero()))
}

fn richcompare(vm: &mut Vm, a: &[Value], op: Cmp) -> PyResult<Value> {
    let v = this(vm, a, "__eq__")?;
    let st = state(vm, &a[0]);
    let c = cur(vm, &st)?;
    let Some((x, y)) = convert_cmp(vm, &st, &c, &v, &a[1], op)? else {
        return Ok(Value::NotImplemented);
    };
    let mut status = 0;
    let Some(r) = mpdec::qcmp(&x, &y, &mut status) else {
        // sNaNs and ordering comparisons signal
        if x.is_snan() || y.is_snan() || !matches!(op, Cmp::Eq | Cmp::Ne) {
            add_status(vm, &st, &c, status)?;
        }
        return Ok(Value::Bool(op == Cmp::Ne));
    };
    Ok(Value::Bool(match op {
        Cmp::Eq => r == 0,
        Cmp::Ne => r != 0,
        Cmp::Le => r <= 0,
        Cmp::Ge => r >= 0,
        Cmp::Lt => r == -1,
        Cmp::Gt => r == 1,
    }))
}

fn d_eq(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    richcompare(vm, a, Cmp::Eq)
}
fn d_ne(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    richcompare(vm, a, Cmp::Ne)
}
fn d_lt(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    richcompare(vm, a, Cmp::Lt)
}
fn d_le(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    richcompare(vm, a, Cmp::Le)
}
fn d_gt(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    richcompare(vm, a, Cmp::Gt)
}
fn d_ge(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    richcompare(vm, a, Cmp::Ge)
}

/// the modulus of numeric hashes, 2**61 - 1
const HASH_P: u64 = (1 << 61) - 1;
/// the inverse of 10 modulo HASH_P
const HASH_INV10: u64 = 2075258708292324556;

fn mulmod(a: u64, b: u64) -> u64 {
    ((a as u128 * b as u128) % HASH_P as u128) as u64
}

fn powmod(mut b: u64, mut e: u64) -> u64 {
    let mut r = 1;
    while e > 0 {
        if e & 1 == 1 {
            r = mulmod(r, b);
        }
        b = mulmod(b, b);
        e >>= 1;
    }
    r
}

/// the hash ints, floats and Fractions of the same value have
fn d_hash(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let d = this(vm, a, "__hash__")?;
    match d.kind {
        Kind::SNaN => return Err(vm.type_error("Cannot hash a signaling NaN value")),
        Kind::NaN => return Ok(Value::Int((ptr_id(&a[0]) >> 4) as i64)),
        Kind::Inf => return Ok(Value::Int(if d.sign { -314159 } else { 314159 })),
        Kind::Fin => {}
    }
    let e = if d.exp >= 0 { powmod(10, d.exp as u64) } else { powmod(HASH_INV10, d.exp.unsigned_abs()) };
    let p = BigInt::from_u128(HASH_P as u128, false);
    let c = d.coef.divmod(&p).unwrap().1.to_i64().unwrap() as u64;
    let r = mulmod(c, e) as i64;
    let r = if d.sign { -r } else { r };
    Ok(Value::Int(if r == -1 { -2 } else { r }))
}

/// dec_as_long: the integer the value rounds to by `round`
fn as_long(vm: &mut Vm, a: &[Value], round: Round, name: &str) -> PyResult<Value> {
    let d = this(vm, a, name)?;
    if d.is_nan() {
        return Err(vm.value_error("cannot convert NaN to integer"));
    }
    if d.is_inf() {
        return Err(vm.err("OverflowError", "cannot convert Infinity to integer"));
    }
    let st = state(vm, &a[0]);
    let c = cur(vm, &st)?;
    let mut status = 0;
    let r = mpdec::to_integral_by(&d, round, &c.ctx, &mut status);
    add_status(vm, &st, &c, status)?;
    Ok(Value::from_big(r.int_value()))
}

fn d_int(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    as_long(vm, a, Round::Down, "__int__")
}
fn d_trunc(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    as_long(vm, a, Round::Down, "__trunc__")
}
fn d_floor(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    as_long(vm, a, Round::Floor, "__floor__")
}
fn d_ceil(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    as_long(vm, a, Round::Ceiling, "__ceil__")
}

fn as_f64(vm: &mut Vm, d: &Dec) -> PyResult<f64> {
    if d.is_snan() {
        return Err(vm.value_error("cannot convert signaling NaN to float"));
    }
    if d.is_nan() {
        return Ok(if d.sign { -f64::NAN } else { f64::NAN });
    }
    Ok(mpdec::to_sci(d, false).parse::<f64>().unwrap_or(f64::NAN))
}

fn d_float(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let d = this(vm, a, "__float__")?;
    Ok(Value::Float(as_f64(vm, &d)?))
}

fn d_complex(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let d = this(vm, a, "__complex__")?;
    Ok(Value::Complex(Rc::new((as_f64(vm, &d)?, 0.0))))
}

fn d_round(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let d = this(vm, a, "__round__")?;
    args_range(vm, a, k, 0, 1)?;
    let Some(x) = a.get(1) else {
        return as_long(vm, a, Round::HalfEven, "__round__");
    };
    if !is_int(vm, x) {
        return Err(vm.type_error("optional arg must be an integer"));
    }
    let y = ssize(vm, x)?;
    let st = state(vm, &a[0]);
    let c = cur(vm, &st)?;
    let q = Dec::triple(false, 1, if y == i64::MIN { i64::MAX } else { -y });
    let mut status = 0;
    let r = mpdec::quantize(&d, &q, &c.ctx, &mut status);
    add_status(vm, &st, &c, status)?;
    Ok(new_dec(&st, r))
}

fn digit_values(c: &BigInt) -> Vec<Value> {
    c.to_radix(10).bytes().map(|b| Value::Int((b - b'0') as i64)).collect()
}

fn d_as_tuple(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    no_kw(vm, k, "as_tuple")?;
    let d = this(vm, a, "as_tuple")?;
    let st = state(vm, &a[0]);
    let (digits, exp) = match d.kind {
        Kind::Inf => (vec![Value::Int(0)], Value::str("F")),
        Kind::NaN | Kind::SNaN => {
            let digits = if d.digits > 0 { digit_values(&d.coef) } else { vec![] };
            (digits, Value::str(if d.kind == Kind::NaN { "n" } else { "N" }))
        }
        Kind::Fin => (digit_values(&d.coef), Value::Int(d.exp)),
    };
    vm.call_vec(&st.tuple, vec![Value::Int(d.sign as i64), Value::tuple(digits), exp], &[])
}

fn d_as_integer_ratio(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let d = this(vm, a, "as_integer_ratio")?;
    if d.is_nan() {
        return Err(vm.value_error("cannot convert NaN to integer ratio"));
    }
    if d.is_inf() {
        return Err(vm.err("OverflowError", "cannot convert Infinity to integer ratio"));
    }
    if d.exp >= 0 {
        return Ok(Value::tuple(vec![Value::from_big(d.int_value()), Value::Int(1)]));
    }
    let den = mpdec::pow10(-d.exp);
    let g = crate::mathmod::gcd_big(&d.coef, &den);
    let n = d.coef.divmod(&g).unwrap().0;
    let n = if d.sign { n.neg() } else { n };
    Ok(Value::tuple(vec![Value::from_big(n), Value::from_big(den.divmod(&g).unwrap().0)]))
}

fn d_copy(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    this(vm, a, "__copy__")?;
    Ok(a[0].clone())
}

fn d_reduce(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let d = this(vm, a, "__reduce__")?;
    let st = state(vm, &a[0]);
    let c = cur(vm, &st)?;
    let cls = Value::Class(vm.class_of(&a[0]));
    Ok(Value::tuple(vec![cls, Value::tuple(vec![Value::string(mpdec::to_sci(&d, c.caps))])]))
}

fn d_sizeof(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let d = this(vm, a, "__sizeof__")?;
    // the words of a coefficient longer than the four kept inline
    let words = (d.digits.max(1) + 18) / 19;
    Ok(Value::Int(104 + if words > 4 { words * 8 } else { 0 }))
}

fn d_real(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    this(vm, a, "real")?;
    Ok(a[0].clone())
}

fn d_imag(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    this(vm, a, "imag")?;
    let st = state(vm, &a[0]);
    Ok(new_dec(&st, Dec::triple(false, 0, 0)))
}

fn d_radix(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    no_kw(vm, k, "radix")?;
    let st = state(vm, &a[0]);
    Ok(new_dec(&st, Dec::triple(false, 10, 0)))
}

fn d_adjusted(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    no_kw(vm, k, "adjusted")?;
    let d = this(vm, a, "adjusted")?;
    Ok(Value::Int(if d.is_special() { 0 } else { d.adjexp() }))
}

fn d_self(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    no_kw(vm, k, "canonical")?;
    this(vm, a, "canonical")?;
    Ok(a[0].clone())
}

fn d_copy_abs(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    no_kw(vm, k, "copy_abs")?;
    let d = this(vm, a, "copy_abs")?;
    let st = state(vm, &a[0]);
    Ok(new_dec(&st, mpdec::copy_abs(&d)))
}

fn d_copy_negate(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    no_kw(vm, k, "copy_negate")?;
    let d = this(vm, a, "copy_negate")?;
    let st = state(vm, &a[0]);
    Ok(new_dec(&st, mpdec::copy_negate(&d)))
}

/// Dec_UnaryFuncVA: method(context=None)
fn unary_va(vm: &mut Vm, a: &[Value], k: Kw, op: Unary, name: &str) -> PyResult<Value> {
    let d = this(vm, a, name)?;
    let st = state(vm, &a[0]);
    let p = parse_args(vm, &a[1..], k, &["context"], 0)?;
    let c = ctx_arg(vm, &st, p[0].as_ref())?;
    run1(vm, &st, &c, &d, op)
}

/// Dec_BinaryFuncVA: method(other, context=None)
fn binary_va(vm: &mut Vm, a: &[Value], k: Kw, op: Binary, name: &str) -> PyResult<Value> {
    let d = this(vm, a, name)?;
    let st = state(vm, &a[0]);
    let p = parse_args(vm, &a[1..], k, &["other", "context"], 1)?;
    let c = ctx_arg(vm, &st, p[1].as_ref())?;
    let other = convert_raise(vm, &st, p[0].as_ref().unwrap())?;
    run2(vm, &st, &c, &d, &other, op)
}

macro_rules! unary_va_methods {
    ($( $f:ident => $op:path, $name:literal; )*) => {
        $(
            fn $f(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
                unary_va(vm, a, k, $op, $name)
            }
        )*
    };
}

macro_rules! binary_va_methods {
    ($( $f:ident => $op:path, $name:literal; )*) => {
        $(
            fn $f(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
                binary_va(vm, a, k, $op, $name)
            }
        )*
    };
}

fn logical_and(a: &Dec, b: &Dec, ctx: &Ctx, st: &mut u32) -> Dec {
    mpdec::logical(a, b, '&', ctx, st)
}
fn logical_or(a: &Dec, b: &Dec, ctx: &Ctx, st: &mut u32) -> Dec {
    mpdec::logical(a, b, '|', ctx, st)
}
fn logical_xor(a: &Dec, b: &Dec, ctx: &Ctx, st: &mut u32) -> Dec {
    mpdec::logical(a, b, '^', ctx, st)
}
fn copy_sign(a: &Dec, b: &Dec, _ctx: &Ctx, _st: &mut u32) -> Dec {
    mpdec::copy_sign(a, b)
}
fn compare_total(a: &Dec, b: &Dec, _ctx: &Ctx, _st: &mut u32) -> Dec {
    mpdec::compare_total(a, b)
}
fn compare_total_mag(a: &Dec, b: &Dec, _ctx: &Ctx, _st: &mut u32) -> Dec {
    mpdec::compare_total_mag(a, b)
}
fn copy_abs(a: &Dec, _ctx: &Ctx, _st: &mut u32) -> Dec {
    mpdec::copy_abs(a)
}
fn copy_negate(a: &Dec, _ctx: &Ctx, _st: &mut u32) -> Dec {
    mpdec::copy_negate(a)
}
fn apply(a: &Dec, ctx: &Ctx, st: &mut u32) -> Dec {
    let mut r = a.clone();
    mpdec::finalize(&mut r, ctx, st);
    r
}

unary_va_methods! {
    d_exp => mpdec::exp, "exp";
    d_ln => mpdec::ln, "ln";
    d_log10 => mpdec::log10, "log10";
    d_next_minus => mpdec::next_minus, "next_minus";
    d_next_plus => mpdec::next_plus, "next_plus";
    d_normalize => mpdec::reduce, "normalize";
    d_sqrt => mpdec::sqrt, "sqrt";
    d_logb => mpdec::logb, "logb";
    d_logical_invert => mpdec::invert, "logical_invert";
}

binary_va_methods! {
    d_compare => mpdec::compare, "compare";
    d_compare_signal => mpdec::compare_signal, "compare_signal";
    d_max => mpdec::max, "max";
    d_max_mag => mpdec::max_mag, "max_mag";
    d_min => mpdec::min, "min";
    d_min_mag => mpdec::min_mag, "min_mag";
    d_next_toward => mpdec::next_toward, "next_toward";
    d_remainder_near => mpdec::rem_near, "remainder_near";
    d_logical_and => logical_and, "logical_and";
    d_logical_or => logical_or, "logical_or";
    d_logical_xor => logical_xor, "logical_xor";
    d_rotate => mpdec::rotate, "rotate";
    d_scaleb => mpdec::scaleb, "scaleb";
    d_shift => mpdec::shift, "shift";
    d_copy_sign => copy_sign, "copy_sign";
    d_compare_total => compare_total, "compare_total";
    d_compare_total_mag => compare_total_mag, "compare_total_mag";
}

fn d_fma(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let d = this(vm, a, "fma")?;
    let st = state(vm, &a[0]);
    let p = parse_args(vm, &a[1..], k, &["other", "third", "context"], 2)?;
    let c = ctx_arg(vm, &st, p[2].as_ref())?;
    let b = convert_raise(vm, &st, p[0].as_ref().unwrap())?;
    let t = convert_raise(vm, &st, p[1].as_ref().unwrap())?;
    let mut status = 0;
    let r = mpdec::fma(&d, &b, &t, &c.ctx, &mut status);
    add_status(vm, &st, &c, status)?;
    Ok(new_dec(&st, r))
}

fn d_quantize(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let d = this(vm, a, "quantize")?;
    let st = state(vm, &a[0]);
    let p = parse_args(vm, &a[1..], k, &["exp", "rounding", "context"], 1)?;
    let c = ctx_arg(vm, &st, p[2].as_ref())?;
    let mut work = c.ctx;
    if let Some(r) = p[1].as_ref().filter(|r| !r.is_none()) {
        work.round = get_round(vm, &st, r)?;
    }
    let b = convert_raise(vm, &st, p[0].as_ref().unwrap())?;
    let mut status = 0;
    let r = mpdec::quantize(&d, &b, &work, &mut status);
    add_status(vm, &st, &c, status)?;
    Ok(new_dec(&st, r))
}

fn to_integral(vm: &mut Vm, a: &[Value], k: Kw, exact: bool, name: &str) -> PyResult<Value> {
    let d = this(vm, a, name)?;
    let st = state(vm, &a[0]);
    let p = parse_args(vm, &a[1..], k, &["rounding", "context"], 0)?;
    let c = ctx_arg(vm, &st, p[1].as_ref())?;
    let mut work = c.ctx;
    if let Some(r) = p[0].as_ref().filter(|r| !r.is_none()) {
        work.round = get_round(vm, &st, r)?;
    }
    let mut status = 0;
    let r = if exact { mpdec::round_to_intx(&d, &work, &mut status) } else { mpdec::round_to_int(&d, &work, &mut status) };
    add_status(vm, &st, &c, status)?;
    Ok(new_dec(&st, r))
}

fn d_to_integral(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    to_integral(vm, a, k, false, "to_integral")
}
fn d_to_integral_value(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    to_integral(vm, a, k, false, "to_integral_value")
}
fn d_to_integral_exact(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    to_integral(vm, a, k, true, "to_integral_exact")
}

fn d_same_quantum(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let d = this(vm, a, "same_quantum")?;
    let st = state(vm, &a[0]);
    let p = parse_args(vm, &a[1..], k, &["other", "context"], 1)?;
    ctx_arg(vm, &st, p[1].as_ref())?;
    let b = convert_raise(vm, &st, p[0].as_ref().unwrap())?;
    Ok(Value::Bool(mpdec::same_quantum(&d, &b)))
}

fn bool_va(vm: &mut Vm, a: &[Value], k: Kw, name: &str, f: fn(&Dec, &Ctx) -> bool) -> PyResult<Value> {
    let d = this(vm, a, name)?;
    let st = state(vm, &a[0]);
    let p = parse_args(vm, &a[1..], k, &["context"], 0)?;
    let c = ctx_arg(vm, &st, p[0].as_ref())?;
    Ok(Value::Bool(f(&d, &c.ctx)))
}

fn d_is_normal(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    bool_va(vm, a, k, "is_normal", |d, c| d.is_normal(c))
}
fn d_is_subnormal(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    bool_va(vm, a, k, "is_subnormal", |d, c| d.is_subnormal(c))
}

macro_rules! bool_methods {
    ($( $f:ident => $name:literal, $test:expr; )*) => {
        $(
            fn $f(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
                no_kw(vm, k, $name)?;
                let d = this(vm, a, $name)?;
                let t: fn(&Dec) -> bool = $test;
                Ok(Value::Bool(t(&d)))
            }
        )*
    };
}

bool_methods! {
    d_is_canonical => "is_canonical", |_| true;
    d_is_finite => "is_finite", |d| d.is_finite();
    d_is_infinite => "is_infinite", |d| d.is_inf();
    d_is_nan => "is_nan", |d| d.is_nan();
    d_is_qnan => "is_qnan", |d| d.is_qnan();
    d_is_snan => "is_snan", |d| d.is_snan();
    d_is_signed => "is_signed", |d| d.sign;
    d_is_zero => "is_zero", |d| d.is_zero();
}

fn d_number_class(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let d = this(vm, a, "number_class")?;
    let st = state(vm, &a[0]);
    let p = parse_args(vm, &a[1..], k, &["context"], 0)?;
    let c = ctx_arg(vm, &st, p[0].as_ref())?;
    Ok(Value::str(mpdec::class(&d, &c.ctx)))
}

fn d_to_eng_string(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let d = this(vm, a, "to_eng_string")?;
    let st = state(vm, &a[0]);
    let p = parse_args(vm, &a[1..], k, &["context"], 0)?;
    let c = ctx_arg(vm, &st, p[0].as_ref())?;
    Ok(Value::string(mpdec::to_eng(&d, c.caps)))
}

/// an override dict's entry as UTF-8
fn override_str(vm: &mut Vm, d: &Rc<RefCell<Dict>>, key: &str) -> PyResult<Option<String>> {
    let Some(v) = vm.dict_get(d, &Value::str(key))? else {
        return Ok(None);
    };
    match str_of_value(vm, &v) {
        Some(s) => Ok(Some(s.s.clone())),
        None => Err(vm.type_error("bad argument type for built-in operation")),
    }
}

/// a C string: up to its first NUL
fn c_str(s: &str) -> &str {
    s.split('\0').next().unwrap_or("")
}

fn d_format(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let d = this(vm, a, "__format__")?;
    args_range(vm, a, k, 1, 2)?;
    let st = state(vm, &a[0]);
    let c = cur(vm, &st)?;
    let Some(fmt) = str_of_value(vm, &a[1]) else {
        return Err(vm.type_error("format arg must be str"));
    };
    if fmt.s.ends_with('N') {
        crate::tb::warn(vm, "DeprecationWarning", "Format specifier 'N' is deprecated")?;
    }
    let Some(mut spec) = mpdec::parse_fmt(&fmt.s, c.caps) else {
        return pydec_format(vm, &st, &a[0], &a[1], &c);
    };
    if let Some(o) = a.get(2) {
        let Value::Dict(od) = vm.base_of(o) else {
            return Err(vm.type_error("optional argument must be a dict"));
        };
        if let Some(s) = override_str(vm, &od, "decimal_point")? {
            spec.dot = c_str(&s).to_string();
        }
        if let Some(s) = override_str(vm, &od, "thousands_sep")? {
            spec.sep = c_str(&s).to_string();
        }
        if let Some(s) = override_str(vm, &od, "grouping")? {
            spec.grouping = c_str(&s).bytes().collect();
        }
        // mpd_validate_lconv
        if spec.grouping.iter().any(|&g| g > 127) || spec.dot.is_empty() || spec.dot.len() > 4 || spec.sep.len() > 4 {
            return Err(vm.value_error("invalid override dict"));
        }
    }
    let mut status = 0;
    match mpdec::format(&d, &spec, &c.ctx, &mut status) {
        Some(s) => Ok(Value::string(s)),
        None if status & mpdec::MALLOC_ERROR != 0 => Err(no_memory(vm)),
        None => Err(vm.value_error("format specification exceeds internal limits of _decimal")),
    }
}

/// the format specifiers libmpdec does not take go to _pydecimal
fn pydec_format(vm: &mut Vm, st: &State, v: &Value, fmt: &Value, c: &Cur) -> PyResult<Value> {
    if st.pydecimal.borrow().is_none() {
        let m = vm.import_one("_pydecimal")?;
        let d = vm.getattr(&m, &ps("Decimal"))?;
        *st.pydecimal.borrow_mut() = d;
    }
    let d = dec_val(v).unwrap();
    let pd = st.pydecimal.borrow().clone();
    let p = vm.call_vec(&pd, vec![Value::string(mpdec::to_sci(&d, c.caps))], &[])?;
    let f = vm.getattr(&p, &ps("__format__"))?;
    match vm.call_vec(&f, vec![fmt.clone(), Value::Obj(c.obj.clone())], &[]) {
        Err(e) if vm.exc_is(&e, "ValueError") => Err(vm.value_error("invalid format string")),
        r => r,
    }
}

// -------------------------------------------------------------------------
// Context

/// a new context of `cls` with `ctx`, its flags and traps dicts fresh
fn new_context(vm: &mut Vm, st: &State, cls: &Rc<Class>, ctx: Ctx, status: u32, traps: u32, capitals: bool) -> Value {
    let status = Rc::new(Cell::new(status));
    let traps = Rc::new(Cell::new(traps));
    let flags = signal_dict(vm, st, Some(status.clone()));
    let traps_dict = signal_dict(vm, st, Some(traps.clone()));
    let o = new_obj(&st.context, ObjKind::DecContext(Box::new(DecContext { ctx, status, traps, capitals, flags, traps_dict })));
    crate::gc::track_obj(&o);
    typed(vm, &st.context, cls, o)
}

const DEFAULT_TRAPS: u32 = mpdec::IEEE_INVALID | mpdec::DIVISION_BY_ZERO | mpdec::OVERFLOW;

fn default_ctx() -> Ctx {
    Ctx { prec: 28, emax: 999_999, emin: -999_999, round: Round::HalfEven, clamp: false, allcr: true }
}

/// context_new: the settings of DefaultContext
fn c_new(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let Some(Value::Class(cls)) = a.first().cloned() else {
        return Err(vm.type_error("Context.__new__(X): X is not a type object"));
    };
    let st = state(vm, &a[0]);
    let tpl = st.templates.borrow().first().cloned();
    let (ctx, status, traps) = match tpl.as_ref().and_then(ctx_native) {
        Some(o) => {
            let t = cur_of(&o);
            (t.ctx, t.status.get(), t.traps.get())
        }
        None => (default_ctx(), 0, DEFAULT_TRAPS),
    };
    Ok(new_context(vm, &st, &cls, ctx, status, traps, true))
}

const CTX_KWS: [&str; 8] = ["prec", "rounding", "Emin", "Emax", "capitals", "clamp", "flags", "traps"];

fn c_init(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    this_ctx(vm, a, "__init__")?;
    let p = parse_args(vm, &a[1..], k, &CTX_KWS, 0)?;
    set_attrs(vm, &a[0], &p)?;
    Ok(Value::None)
}

/// context_setattrs: each setting given and not None
fn set_attrs(vm: &mut Vm, ctx: &Value, p: &[Option<Value>]) -> PyResult<()> {
    let st = state(vm, ctx);
    let c = cur_of(&ctx_native(ctx).unwrap());
    let setters: [NativeFnPtr; 6] = [c_set_prec, c_set_rounding, c_set_emin, c_set_emax, c_set_capitals, c_set_clamp];
    for (i, f) in setters.iter().enumerate() {
        if let Some(v) = p[i].as_ref().filter(|v| !v.is_none()) {
            f(vm, &[ctx.clone(), v.clone()], &[])?;
        }
    }
    // traps before flags, as the C module does
    for (i, bits) in [(7, &c.traps), (6, &c.status)] {
        if let Some(v) = p[i].as_ref().filter(|v| !v.is_none()) {
            let flags = if matches!(vm.base_of(v), Value::List(_)) { list_as_flags(vm, &st, v)? } else { dict_as_flags(vm, &st, v)? };
            bits.set(flags);
        }
    }
    Ok(())
}

fn c_get_prec(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    Ok(Value::Int(this_ctx(vm, a, "prec")?.ctx.prec))
}
fn c_get_emin(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    Ok(Value::Int(this_ctx(vm, a, "Emin")?.ctx.emin))
}
fn c_get_emax(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    Ok(Value::Int(this_ctx(vm, a, "Emax")?.ctx.emax))
}
fn c_get_clamp(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    Ok(Value::Int(this_ctx(vm, a, "clamp")?.ctx.clamp as i64))
}
fn c_get_capitals(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    Ok(Value::Int(this_ctx(vm, a, "capitals")?.caps as i64))
}
fn c_get_rounding(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let c = this_ctx(vm, a, "rounding")?;
    let st = state(vm, &a[0]);
    Ok(st.rounds[Round::ALL.iter().position(|r| *r == c.ctx.round).unwrap_or(6)].clone())
}
fn c_get_flags(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let c = this_ctx(vm, a, "flags")?;
    Ok(with_ctx(&c.obj, |c| c.flags.clone()))
}
fn c_get_traps(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let c = this_ctx(vm, a, "traps")?;
    Ok(with_ctx(&c.obj, |c| c.traps_dict.clone()))
}

/// a setting checked by `ok`, or a ValueError
fn set_ssize(vm: &mut Vm, a: &[Value], name: &str, ok: fn(i64) -> bool, msg: &str, set: fn(&mut DecContext, i64)) -> PyResult<Value> {
    let c = this_ctx(vm, a, name)?;
    let x = ssize(vm, &a[1])?;
    if !ok(x) {
        return Err(vm.value_error(msg));
    }
    with_ctx(&c.obj, |c| set(c, x));
    Ok(Value::None)
}

fn c_set_prec(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    set_ssize(vm, a, "prec", |x| (1..=mpdec::MAX_PREC).contains(&x), "valid range for prec is [1, MAX_PREC]", |c, x| c.ctx.prec = x)
}
fn c_set_emin(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    set_ssize(vm, a, "Emin", |x| (mpdec::MIN_EMIN..=0).contains(&x), "valid range for Emin is [MIN_EMIN, 0]", |c, x| c.ctx.emin = x)
}
fn c_set_emax(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    set_ssize(vm, a, "Emax", |x| (0..=mpdec::MAX_EMAX).contains(&x), "valid range for Emax is [0, MAX_EMAX]", |c, x| c.ctx.emax = x)
}
fn c_set_capitals(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    set_ssize(vm, a, "capitals", |x| x == 0 || x == 1, "valid values for capitals are 0 or 1", |c, x| c.capitals = x == 1)
}
fn c_set_clamp(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    set_ssize(vm, a, "clamp", |x| x == 0 || x == 1, "valid values for clamp are 0 or 1", |c, x| c.ctx.clamp = x == 1)
}
fn c_set_rounding(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let c = this_ctx(vm, a, "rounding")?;
    let st = state(vm, &a[0]);
    let r = get_round(vm, &st, &a[1])?;
    with_ctx(&c.obj, |c| c.ctx.round = r);
    Ok(Value::None)
}
fn c_set_flags(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let c = this_ctx(vm, a, "flags")?;
    let st = state(vm, &a[0]);
    let f = dict_as_flags(vm, &st, &a[1])?;
    c.status.set(f);
    Ok(Value::None)
}
fn c_set_traps(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let c = this_ctx(vm, a, "traps")?;
    let st = state(vm, &a[0]);
    let f = dict_as_flags(vm, &st, &a[1])?;
    c.traps.set(f);
    Ok(Value::None)
}
fn c_del(vm: &mut Vm, _a: &[Value], _k: Kw) -> PyResult<Value> {
    Err(vm.attr_error("context attributes cannot be deleted"))
}

/// the signals of `flags` as Context.__repr__ lists them
fn signal_names(flags: u32) -> String {
    let mut out = vec![];
    let mut invalid_done = false;
    for (j, name) in BIT_NAMES.iter().enumerate() {
        let f = flags & (1 << j);
        if f == 0 {
            continue;
        }
        if f & mpdec::IEEE_INVALID != 0 {
            if invalid_done {
                continue;
            }
            invalid_done = true;
        }
        out.push(*name);
    }
    format!("[{}]", out.join(", "))
}

fn c_repr(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let c = this_ctx(vm, a, "__repr__")?;
    let x = c.ctx;
    Ok(Value::string(format!(
        "Context(prec={}, rounding={}, Emin={}, Emax={}, capitals={}, clamp={}, flags={}, traps={})",
        x.prec,
        x.round.name(),
        x.emin,
        x.emax,
        c.caps as i32,
        x.clamp as i32,
        signal_names(c.status.get()),
        signal_names(c.traps.get())
    )))
}

/// context_copy: a plain Context with the same settings and bits
fn context_copy(vm: &mut Vm, st: &State, v: &Value) -> PyResult<Value> {
    let Some(c) = ctx_native(v).map(|o| cur_of(&o)) else {
        return Err(vm.type_error("argument must be a context"));
    };
    Ok(new_context(vm, st, &st.context.clone(), c.ctx, c.status.get(), c.traps.get(), c.caps))
}

fn c_copy(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    this_ctx(vm, a, "copy")?;
    let st = state(vm, &a[0]);
    context_copy(vm, &st, &a[0])
}

fn c_reduce(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let c = this_ctx(vm, a, "__reduce__")?;
    let st = state(vm, &a[0]);
    let x = c.ctx;
    let args = vec![Value::Int(x.prec), Value::str(x.round.name()), Value::Int(x.emin), Value::Int(x.emax), Value::Int(c.caps as i64), Value::Int(x.clamp as i64), signals_as_list(&st, c.status.get()), signals_as_list(&st, c.traps.get())];
    let cls = Value::Class(vm.class_of(&a[0]));
    Ok(Value::tuple(vec![cls, Value::tuple(args)]))
}

fn c_clear_flags(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    this_ctx(vm, a, "clear_flags")?.status.set(0);
    Ok(Value::None)
}
fn c_clear_traps(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    this_ctx(vm, a, "clear_traps")?.traps.set(0);
    Ok(Value::None)
}
fn c_etiny(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    Ok(Value::Int(this_ctx(vm, a, "Etiny")?.ctx.etiny()))
}
fn c_etop(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    Ok(Value::Int(this_ctx(vm, a, "Etop")?.ctx.etop()))
}

fn c_create_decimal(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let c = this_ctx(vm, a, "create_decimal")?;
    args_range(vm, a, k, 0, 1)?;
    let st = state(vm, &a[0]);
    let d = from_object(vm, &st, a.get(1), &c)?;
    Ok(new_dec(&st, d))
}

fn c_create_decimal_from_float(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let c = this_ctx(vm, a, "create_decimal_from_float")?;
    let st = state(vm, &a[0]);
    let mut d = from_float_exact(vm, &a[1])?;
    let mut status = 0;
    mpdec::finalize(&mut d, &c.ctx, &mut status);
    add_status(vm, &st, &c, status)?;
    Ok(new_dec(&st, d))
}

/// a Context method of one operand
fn ctx_unary(vm: &mut Vm, a: &[Value], op: Unary, name: &str) -> PyResult<Value> {
    let c = this_ctx(vm, a, name)?;
    let st = state(vm, &a[0]);
    let x = convert_raise(vm, &st, &a[1])?;
    run1(vm, &st, &c, &x, op)
}

/// a Context method of two operands
fn ctx_binary(vm: &mut Vm, a: &[Value], k: Kw, op: Binary, name: &str) -> PyResult<Value> {
    let c = this_ctx(vm, a, name)?;
    exact_args(vm, a, k, name, 2)?;
    let st = state(vm, &a[0]);
    let x = convert_raise(vm, &st, &a[1])?;
    let y = convert_raise(vm, &st, &a[2])?;
    run2(vm, &st, &c, &x, &y, op)
}

macro_rules! ctx_unary_methods {
    ($( $f:ident => $op:path, $name:literal; )*) => {
        $(
            fn $f(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
                ctx_unary(vm, a, $op, $name)
            }
        )*
    };
}

macro_rules! ctx_binary_methods {
    ($( $f:ident => $op:path, $name:literal; )*) => {
        $(
            fn $f(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
                ctx_binary(vm, a, k, $op, $name)
            }
        )*
    };
}

ctx_unary_methods! {
    c_abs => mpdec::abs, "abs";
    c_exp => mpdec::exp, "exp";
    c_ln => mpdec::ln, "ln";
    c_log10 => mpdec::log10, "log10";
    c_minus => mpdec::minus, "minus";
    c_next_minus => mpdec::next_minus, "next_minus";
    c_next_plus => mpdec::next_plus, "next_plus";
    c_normalize => mpdec::reduce, "normalize";
    c_plus => mpdec::plus, "plus";
    c_to_integral => mpdec::round_to_int, "to_integral";
    c_to_integral_exact => mpdec::round_to_intx, "to_integral_exact";
    c_to_integral_value => mpdec::round_to_int, "to_integral_value";
    c_sqrt => mpdec::sqrt, "sqrt";
    c_logb => mpdec::logb, "logb";
    c_logical_invert => mpdec::invert, "logical_invert";
    c_copy_abs => copy_abs, "copy_abs";
    c_copy_negate => copy_negate, "copy_negate";
    c_apply => apply, "_apply";
}

ctx_binary_methods! {
    c_add => mpdec::add, "add";
    c_compare => mpdec::compare, "compare";
    c_compare_signal => mpdec::compare_signal, "compare_signal";
    c_divide => mpdec::div, "divide";
    c_divide_int => mpdec::divint, "divide_int";
    c_max => mpdec::max, "max";
    c_max_mag => mpdec::max_mag, "max_mag";
    c_min => mpdec::min, "min";
    c_min_mag => mpdec::min_mag, "min_mag";
    c_multiply => mpdec::mul, "multiply";
    c_next_toward => mpdec::next_toward, "next_toward";
    c_quantize => mpdec::quantize, "quantize";
    c_remainder => mpdec::rem, "remainder";
    c_remainder_near => mpdec::rem_near, "remainder_near";
    c_subtract => mpdec::sub, "subtract";
    c_compare_total => compare_total, "compare_total";
    c_compare_total_mag => compare_total_mag, "compare_total_mag";
    c_copy_sign => copy_sign, "copy_sign";
    c_logical_and => logical_and, "logical_and";
    c_logical_or => logical_or, "logical_or";
    c_logical_xor => logical_xor, "logical_xor";
    c_rotate => mpdec::rotate, "rotate";
    c_scaleb => mpdec::scaleb, "scaleb";
    c_shift => mpdec::shift, "shift";
}

fn c_divmod(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let c = this_ctx(vm, a, "divmod")?;
    exact_args(vm, a, k, "divmod", 2)?;
    let st = state(vm, &a[0]);
    let x = convert_raise(vm, &st, &a[1])?;
    let y = convert_raise(vm, &st, &a[2])?;
    let mut status = 0;
    let (q, r) = mpdec::divmod(&x, &y, &c.ctx, &mut status);
    add_status(vm, &st, &c, status)?;
    Ok(Value::tuple(vec![new_dec(&st, q), new_dec(&st, r)]))
}

fn c_power(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let c = this_ctx(vm, a, "power")?;
    let st = state(vm, &a[0]);
    let p = parse_args(vm, &a[1..], k, &["a", "b", "modulo"], 2)?;
    let x = convert_raise(vm, &st, p[0].as_ref().unwrap())?;
    let y = convert_raise(vm, &st, p[1].as_ref().unwrap())?;
    let m = match p[2].as_ref().filter(|m| !m.is_none()) {
        Some(m) => Some(convert_raise(vm, &st, m)?),
        None => None,
    };
    let mut status = 0;
    let r = match m {
        None => mpdec::pow(&x, &y, &c.ctx, &mut status),
        Some(m) => mpdec::powmod(&x, &y, &m, &c.ctx, &mut status),
    };
    add_status(vm, &st, &c, status)?;
    Ok(new_dec(&st, r))
}

fn c_fma(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let c = this_ctx(vm, a, "fma")?;
    exact_args(vm, a, k, "fma", 3)?;
    let st = state(vm, &a[0]);
    let x = convert_raise(vm, &st, &a[1])?;
    let y = convert_raise(vm, &st, &a[2])?;
    let z = convert_raise(vm, &st, &a[3])?;
    let mut status = 0;
    let r = mpdec::fma(&x, &y, &z, &c.ctx, &mut status);
    add_status(vm, &st, &c, status)?;
    Ok(new_dec(&st, r))
}

fn c_same_quantum(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    this_ctx(vm, a, "same_quantum")?;
    exact_args(vm, a, k, "same_quantum", 2)?;
    let st = state(vm, &a[0]);
    let x = convert_raise(vm, &st, &a[1])?;
    let y = convert_raise(vm, &st, &a[2])?;
    Ok(Value::Bool(mpdec::same_quantum(&x, &y)))
}

fn c_radix(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    this_ctx(vm, a, "radix")?;
    d_radix(vm, a, &[])
}

/// a Context predicate of one operand
fn ctx_bool(vm: &mut Vm, a: &[Value], name: &str, f: fn(&Dec, &Ctx) -> bool) -> PyResult<Value> {
    let c = this_ctx(vm, a, name)?;
    let st = state(vm, &a[0]);
    let x = convert_raise(vm, &st, &a[1])?;
    Ok(Value::Bool(f(&x, &c.ctx)))
}

macro_rules! ctx_bool_methods {
    ($( $f:ident => $name:literal, $test:expr; )*) => {
        $(
            fn $f(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
                ctx_bool(vm, a, $name, $test)
            }
        )*
    };
}

ctx_bool_methods! {
    c_is_normal => "is_normal", |d, c| d.is_normal(c);
    c_is_subnormal => "is_subnormal", |d, c| d.is_subnormal(c);
    c_is_finite => "is_finite", |d, _| d.is_finite();
    c_is_infinite => "is_infinite", |d, _| d.is_inf();
    c_is_nan => "is_nan", |d, _| d.is_nan();
    c_is_qnan => "is_qnan", |d, _| d.is_qnan();
    c_is_signed => "is_signed", |d, _| d.sign;
    c_is_snan => "is_snan", |d, _| d.is_snan();
    c_is_zero => "is_zero", |d, _| d.is_zero();
}

/// is_canonical and canonical take only a Decimal
fn ctx_decimal_arg(vm: &mut Vm, a: &[Value], name: &str) -> PyResult<()> {
    this_ctx(vm, a, name)?;
    let st = state(vm, &a[0]);
    if as_dec(vm, &st, &a[1]).is_none() {
        return Err(vm.type_error("argument must be a Decimal"));
    }
    Ok(())
}

fn c_is_canonical(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    ctx_decimal_arg(vm, a, "is_canonical")?;
    Ok(Value::Bool(true))
}
fn c_canonical(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    ctx_decimal_arg(vm, a, "canonical")?;
    Ok(a[1].clone())
}

fn c_copy_decimal(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    this_ctx(vm, a, "copy_decimal")?;
    let st = state(vm, &a[0]);
    if as_dec(vm, &st, &a[1]).is_some() {
        return Ok(a[1].clone());
    }
    let d = convert_raise(vm, &st, &a[1])?;
    Ok(new_dec(&st, (*d).clone()))
}

fn c_number_class(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let c = this_ctx(vm, a, "number_class")?;
    let st = state(vm, &a[0]);
    let x = convert_raise(vm, &st, &a[1])?;
    Ok(Value::str(mpdec::class(&x, &c.ctx)))
}

fn c_to_sci_string(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let c = this_ctx(vm, a, "to_sci_string")?;
    let st = state(vm, &a[0]);
    let x = convert_raise(vm, &st, &a[1])?;
    Ok(Value::string(mpdec::to_sci(&x, c.caps)))
}

fn c_to_eng_string(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let c = this_ctx(vm, a, "to_eng_string")?;
    let st = state(vm, &a[0]);
    let x = convert_raise(vm, &st, &a[1])?;
    Ok(Value::string(mpdec::to_eng(&x, c.caps)))
}

// -------------------------------------------------------------------------
// signal dicts

/// a SignalDict over `bits`
fn signal_dict(vm: &mut Vm, st: &State, bits: Option<Rc<Cell<u32>>>) -> Value {
    let o = new_obj(&st.mixin, ObjKind::DecSignals(bits));
    typed(vm, &st.mixin, &st.signal_dict.clone(), o)
}

/// the bits behind a signal dict of this module; Some(None) for one tied
/// to no context
fn signals_of(vm: &Vm, st: &State, v: &Value) -> Option<Option<Rc<Cell<u32>>>> {
    if !vm.class_of(v).is_subclass(&st.mixin) {
        return None;
    }
    let o = native_of(v)?;
    let k = o.kind.borrow();
    match &*k {
        ObjKind::DecSignals(b) => Some(b.clone()),
        _ => None,
    }
}

/// self of a SignalDict method: its bits
fn this_sd(vm: &mut Vm, a: &[Value]) -> PyResult<(Rc<State>, Rc<Cell<u32>>)> {
    let st = state(vm, &a[0]);
    match signals_of(vm, &st, &a[0]) {
        Some(Some(b)) => Ok((st, b)),
        _ => Err(vm.value_error("invalid signal dict")),
    }
}

fn sd_new(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let Some(Value::Class(cls)) = a.first().cloned() else {
        return Err(vm.type_error("SignalDictMixin.__new__(X): X is not a type object"));
    };
    let st = state(vm, &a[0]);
    let o = new_obj(&st.mixin, ObjKind::DecSignals(None));
    Ok(typed(vm, &st.mixin, &cls, o))
}

fn sd_init(_vm: &mut Vm, _a: &[Value], _k: Kw) -> PyResult<Value> {
    Ok(Value::None)
}

fn sd_len(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    this_sd(vm, a)?;
    Ok(Value::Int(9))
}

fn sd_iter(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let (st, _) = this_sd(vm, a)?;
    vm.iter(&st.signal_tuple)
}

fn sd_getitem(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let (st, b) = this_sd(vm, a)?;
    let f = signal_flag(vm, &st, &a[1])?;
    Ok(Value::Bool(b.get() & f != 0))
}

fn sd_setitem(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let (st, b) = this_sd(vm, a)?;
    let f = signal_flag(vm, &st, &a[1])?;
    if vm.truthy(&a[2])? {
        b.set(b.get() | f);
    } else {
        b.set(b.get() & !f);
    }
    Ok(Value::None)
}

fn sd_delitem(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    this_sd(vm, a)?;
    Err(vm.value_error("signal keys cannot be deleted"))
}

fn sd_repr(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let (_, b) = this_sd(vm, a)?;
    let items: Vec<String> = (0..9).map(|i| format!("<class 'decimal.{}'>:{}", SIGNAL_NAMES[i], if b.get() & SIGNAL_FLAGS[i] != 0 { "True" } else { "False" })).collect();
    Ok(Value::string(format!("{{{}}}", items.join(", "))))
}

fn sd_compare(vm: &mut Vm, a: &[Value], ne: bool) -> PyResult<Value> {
    let (st, b) = this_sd(vm, a)?;
    let other = match signals_of(vm, &st, &a[1]) {
        Some(Some(o)) => o.get(),
        Some(None) => return Err(vm.value_error("invalid signal dict")),
        None => {
            if !matches!(vm.base_of(&a[1]), Value::Dict(_)) {
                return Ok(Value::NotImplemented);
            }
            match dict_as_flags(vm, &st, &a[1]) {
                Ok(f) => f,
                // a dict of other keys does not compare
                Err(e) if vm.exc_is(&e, "KeyError") => return Ok(Value::NotImplemented),
                Err(e) => return Err(e),
            }
        }
    };
    Ok(Value::Bool((b.get() == other) != ne))
}

fn sd_eq(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    sd_compare(vm, a, false)
}
fn sd_ne(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    sd_compare(vm, a, true)
}

fn sd_copy(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let (st, b) = this_sd(vm, a)?;
    let d = new_dict();
    for i in 0..9 {
        vm.dict_set(&d, st.signals[i].clone(), Value::Bool(b.get() & SIGNAL_FLAGS[i] != 0))?;
    }
    Ok(Value::Dict(d))
}

// -------------------------------------------------------------------------
// module functions and localcontext()

fn m_getcontext(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    no_kw(vm, k, "_decimal.getcontext")?;
    if a.len() != 1 {
        return Err(vm.type_error(format!("_decimal.getcontext() takes no arguments ({} given)", a.len() - 1)));
    }
    let st = state(vm, &a[0]);
    current(vm, &st)
}

fn m_setcontext(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    no_kw(vm, k, "_decimal.setcontext")?;
    if a.len() != 2 {
        return Err(vm.type_error(format!("_decimal.setcontext() takes exactly one argument ({} given)", a.len() - 1)));
    }
    let st = state(vm, &a[0]);
    setcontext(vm, &st, &a[1])?;
    Ok(Value::None)
}

fn setcontext(vm: &mut Vm, st: &State, v: &Value) -> PyResult<()> {
    if as_ctx(vm, st, v).is_none() {
        return Err(vm.type_error("argument must be a context"));
    }
    // a template is copied, as decimal.py does
    let is_template = st.templates.borrow().iter().any(|t| t.is(v));
    let v = if is_template {
        let c = context_copy(vm, st, v)?;
        with_ctx(&ctx_native(&c).unwrap(), |x| x.status.set(0));
        c
    } else {
        v.clone()
    };
    set_current(vm, st, v)
}

fn m_localcontext(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let st = state(vm, &a[0]);
    let global = current(vm, &st)?;
    let mut names = vec!["ctx"];
    names.extend(CTX_KWS);
    let p = parse_args(vm, &a[1..], k, &names, 0)?;
    let local = match p[0].as_ref().filter(|v| !v.is_none()) {
        None => global.clone(),
        Some(l) if as_ctx(vm, &st, l).is_some() => l.clone(),
        Some(_) => return Err(vm.type_error("optional argument must be a context")),
    };
    let local = context_copy(vm, &st, &local)?;
    set_attrs(vm, &local, &p[1..])?;
    let o = new_obj(&st.manager, ObjKind::Other(Value::tuple(vec![local, global])));
    crate::gc::track_obj(&o);
    Ok(Value::Obj(o))
}

fn manager_parts(vm: &mut Vm, a: &[Value]) -> PyResult<(Value, Value)> {
    if let Some(o) = native_of(&a[0]) {
        if let ObjKind::Other(Value::Tuple(t)) = &*o.kind.borrow() {
            if t.len() == 2 {
                return Ok((t[0].clone(), t[1].clone()));
            }
        }
    }
    Err(vm.type_error("not a decimal context manager"))
}

fn cm_enter(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let (local, _) = manager_parts(vm, a)?;
    let st = state(vm, &a[0]);
    setcontext(vm, &st, &local)?;
    Ok(local)
}

fn cm_exit(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let (_, global) = manager_parts(vm, a)?;
    let st = state(vm, &a[0]);
    setcontext(vm, &st, &global)?;
    Ok(Value::None)
}

fn cm_new(vm: &mut Vm, _a: &[Value], _k: Kw) -> PyResult<Value> {
    Err(vm.type_error("cannot create 'decimal.ContextManager' instances"))
}

// -------------------------------------------------------------------------
// the module

fn native_type(vm: &Vm, name: &str) -> Rc<Class> {
    let object = vm.t.object.clone();
    let c = Class::new(name, vec![object.clone()], Builtin::Obj);
    c.set_mro(vec![c.clone(), object]);
    let mut d = c.dict.borrow_mut();
    d.set_s("__module__", Value::str("decimal"));
    if let Some((_, doc)) = doc_in(DOCS, name) {
        d.set_s("__doc__", Value::str(doc));
    }
    drop(d);
    c
}

fn property(get: (&'static str, NativeFnPtr), set: Option<(&'static str, NativeFnPtr)>) -> Value {
    let (fset, fdel) = match set {
        Some((n, f)) => (native(n, f), native("Context.__delattr__", c_del)),
        None => (Value::None, Value::None),
    };
    Value::Property(Rc::new(Property { fget: native(get.0, get.1), fset, fdel, doc: RefCell::new(Value::None), name: RefCell::new(Value::Undef), getter_doc: Cell::new(false) }))
}

fn decimal_type(vm: &Vm) -> Rc<Class> {
    let c = native_type(vm, "Decimal");
    def_static(&c, "Decimal.__new__", d_new);
    def_class(&c, "Decimal.from_float", d_from_float);
    let one: &[(&'static str, NativeFnPtr)] = &[
        ("Decimal.__add__", d_add),
        ("Decimal.__radd__", d_radd),
        ("Decimal.__sub__", d_sub),
        ("Decimal.__rsub__", d_rsub),
        ("Decimal.__mul__", d_mul),
        ("Decimal.__rmul__", d_rmul),
        ("Decimal.__truediv__", d_truediv),
        ("Decimal.__rtruediv__", d_rtruediv),
        ("Decimal.__floordiv__", d_floordiv),
        ("Decimal.__rfloordiv__", d_rfloordiv),
        ("Decimal.__mod__", d_mod),
        ("Decimal.__rmod__", d_rmod),
        ("Decimal.__divmod__", d_divmod),
        ("Decimal.__rdivmod__", d_rdivmod),
        ("Decimal.__eq__", d_eq),
        ("Decimal.__ne__", d_ne),
        ("Decimal.__lt__", d_lt),
        ("Decimal.__le__", d_le),
        ("Decimal.__gt__", d_gt),
        ("Decimal.__ge__", d_ge),
        ("Decimal.__deepcopy__", d_copy),
    ];
    for (n, f) in one {
        def_arity(&c, n, *f, 1, 1);
    }
    def_arity(&c, "Decimal.__pow__", d_pow, 1, 2);
    def_arity(&c, "Decimal.__rpow__", d_rpow, 1, 2);
    let none: &[(&'static str, NativeFnPtr)] = &[
        ("Decimal.__neg__", d_neg),
        ("Decimal.__pos__", d_pos),
        ("Decimal.__abs__", d_abs),
        ("Decimal.__bool__", d_bool),
        ("Decimal.__hash__", d_hash),
        ("Decimal.__str__", d_str),
        ("Decimal.__repr__", d_repr),
        ("Decimal.__int__", d_int),
        ("Decimal.__float__", d_float),
        ("Decimal.__complex__", d_complex),
        ("Decimal.__trunc__", d_trunc),
        ("Decimal.__floor__", d_floor),
        ("Decimal.__ceil__", d_ceil),
        ("Decimal.__copy__", d_copy),
        ("Decimal.__reduce__", d_reduce),
        ("Decimal.__sizeof__", d_sizeof),
        ("Decimal.as_tuple", d_as_tuple),
        ("Decimal.as_integer_ratio", d_as_integer_ratio),
        ("Decimal.adjusted", d_adjusted),
        ("Decimal.canonical", d_self),
        ("Decimal.conjugate", d_self),
        ("Decimal.radix", d_radix),
        ("Decimal.copy_abs", d_copy_abs),
        ("Decimal.copy_negate", d_copy_negate),
        ("Decimal.is_canonical", d_is_canonical),
        ("Decimal.is_finite", d_is_finite),
        ("Decimal.is_infinite", d_is_infinite),
        ("Decimal.is_nan", d_is_nan),
        ("Decimal.is_qnan", d_is_qnan),
        ("Decimal.is_snan", d_is_snan),
        ("Decimal.is_signed", d_is_signed),
        ("Decimal.is_zero", d_is_zero),
    ];
    for (n, f) in none {
        def_arity(&c, n, *f, 0, 0);
    }
    let kw: &[(&'static str, NativeFnPtr)] = &[
        ("Decimal.__round__", d_round),
        ("Decimal.__format__", d_format),
        ("Decimal.exp", d_exp),
        ("Decimal.ln", d_ln),
        ("Decimal.log10", d_log10),
        ("Decimal.next_minus", d_next_minus),
        ("Decimal.next_plus", d_next_plus),
        ("Decimal.normalize", d_normalize),
        ("Decimal.to_integral", d_to_integral),
        ("Decimal.to_integral_exact", d_to_integral_exact),
        ("Decimal.to_integral_value", d_to_integral_value),
        ("Decimal.sqrt", d_sqrt),
        ("Decimal.compare", d_compare),
        ("Decimal.compare_signal", d_compare_signal),
        ("Decimal.max", d_max),
        ("Decimal.max_mag", d_max_mag),
        ("Decimal.min", d_min),
        ("Decimal.min_mag", d_min_mag),
        ("Decimal.next_toward", d_next_toward),
        ("Decimal.quantize", d_quantize),
        ("Decimal.remainder_near", d_remainder_near),
        ("Decimal.fma", d_fma),
        ("Decimal.is_normal", d_is_normal),
        ("Decimal.is_subnormal", d_is_subnormal),
        ("Decimal.logb", d_logb),
        ("Decimal.logical_invert", d_logical_invert),
        ("Decimal.number_class", d_number_class),
        ("Decimal.to_eng_string", d_to_eng_string),
        ("Decimal.compare_total", d_compare_total),
        ("Decimal.compare_total_mag", d_compare_total_mag),
        ("Decimal.copy_sign", d_copy_sign),
        ("Decimal.same_quantum", d_same_quantum),
        ("Decimal.logical_and", d_logical_and),
        ("Decimal.logical_or", d_logical_or),
        ("Decimal.logical_xor", d_logical_xor),
        ("Decimal.rotate", d_rotate),
        ("Decimal.scaleb", d_scaleb),
        ("Decimal.shift", d_shift),
    ];
    for (n, f) in kw {
        def(&c, n, *f);
    }
    c.dict.borrow_mut().set_s("real", property(("Decimal.real", d_real), None));
    c.dict.borrow_mut().set_s("imag", property(("Decimal.imag", d_imag), None));
    c
}

fn context_type(vm: &Vm) -> Rc<Class> {
    let c = native_type(vm, "Context");
    def_static(&c, "Context.__new__", c_new);
    def(&c, "Context.__init__", c_init);
    let one: &[(&'static str, NativeFnPtr)] = &[
        ("Context.abs", c_abs),
        ("Context.exp", c_exp),
        ("Context.ln", c_ln),
        ("Context.log10", c_log10),
        ("Context.minus", c_minus),
        ("Context.next_minus", c_next_minus),
        ("Context.next_plus", c_next_plus),
        ("Context.normalize", c_normalize),
        ("Context.plus", c_plus),
        ("Context.to_integral", c_to_integral),
        ("Context.to_integral_exact", c_to_integral_exact),
        ("Context.to_integral_value", c_to_integral_value),
        ("Context.sqrt", c_sqrt),
        ("Context.is_canonical", c_is_canonical),
        ("Context.is_finite", c_is_finite),
        ("Context.is_infinite", c_is_infinite),
        ("Context.is_nan", c_is_nan),
        ("Context.is_normal", c_is_normal),
        ("Context.is_qnan", c_is_qnan),
        ("Context.is_signed", c_is_signed),
        ("Context.is_snan", c_is_snan),
        ("Context.is_subnormal", c_is_subnormal),
        ("Context.is_zero", c_is_zero),
        ("Context._apply", c_apply),
        ("Context.canonical", c_canonical),
        ("Context.copy_abs", c_copy_abs),
        ("Context.copy_decimal", c_copy_decimal),
        ("Context.copy_negate", c_copy_negate),
        ("Context.logb", c_logb),
        ("Context.logical_invert", c_logical_invert),
        ("Context.number_class", c_number_class),
        ("Context.to_sci_string", c_to_sci_string),
        ("Context.to_eng_string", c_to_eng_string),
        ("Context.create_decimal_from_float", c_create_decimal_from_float),
    ];
    for (n, f) in one {
        def_arity(&c, n, *f, 1, 1);
    }
    let none: &[(&'static str, NativeFnPtr)] = &[
        ("Context.Etiny", c_etiny),
        ("Context.Etop", c_etop),
        ("Context.radix", c_radix),
        ("Context.clear_flags", c_clear_flags),
        ("Context.clear_traps", c_clear_traps),
        ("Context.__copy__", c_copy),
        ("Context.__reduce__", c_reduce),
        ("Context.__repr__", c_repr),
        ("Context.copy", c_copy),
    ];
    for (n, f) in none {
        def_arity(&c, n, *f, 0, 0);
    }
    let var: &[(&'static str, NativeFnPtr)] = &[
        ("Context.add", c_add),
        ("Context.compare", c_compare),
        ("Context.compare_signal", c_compare_signal),
        ("Context.divide", c_divide),
        ("Context.divide_int", c_divide_int),
        ("Context.divmod", c_divmod),
        ("Context.max", c_max),
        ("Context.max_mag", c_max_mag),
        ("Context.min", c_min),
        ("Context.min_mag", c_min_mag),
        ("Context.multiply", c_multiply),
        ("Context.next_toward", c_next_toward),
        ("Context.quantize", c_quantize),
        ("Context.remainder", c_remainder),
        ("Context.remainder_near", c_remainder_near),
        ("Context.subtract", c_subtract),
        ("Context.power", c_power),
        ("Context.fma", c_fma),
        ("Context.compare_total", c_compare_total),
        ("Context.compare_total_mag", c_compare_total_mag),
        ("Context.copy_sign", c_copy_sign),
        ("Context.logical_and", c_logical_and),
        ("Context.logical_or", c_logical_or),
        ("Context.logical_xor", c_logical_xor),
        ("Context.rotate", c_rotate),
        ("Context.same_quantum", c_same_quantum),
        ("Context.scaleb", c_scaleb),
        ("Context.shift", c_shift),
        ("Context.create_decimal", c_create_decimal),
    ];
    for (n, f) in var {
        def(&c, n, *f);
    }
    let props: [(&str, (&'static str, NativeFnPtr), (&'static str, NativeFnPtr)); 8] = [
        ("prec", ("Context.prec", c_get_prec), ("Context.prec", c_set_prec)),
        ("Emax", ("Context.Emax", c_get_emax), ("Context.Emax", c_set_emax)),
        ("Emin", ("Context.Emin", c_get_emin), ("Context.Emin", c_set_emin)),
        ("rounding", ("Context.rounding", c_get_rounding), ("Context.rounding", c_set_rounding)),
        ("capitals", ("Context.capitals", c_get_capitals), ("Context.capitals", c_set_capitals)),
        ("clamp", ("Context.clamp", c_get_clamp), ("Context.clamp", c_set_clamp)),
        ("flags", ("Context.flags", c_get_flags), ("Context.flags", c_set_flags)),
        ("traps", ("Context.traps", c_get_traps), ("Context.traps", c_set_traps)),
    ];
    for (name, get, set) in props {
        c.dict.borrow_mut().set_s(name, property(get, Some(set)));
    }
    c
}

fn mixin_type(vm: &Vm) -> Rc<Class> {
    let c = native_type(vm, "SignalDictMixin");
    def_static(&c, "SignalDictMixin.__new__", sd_new);
    def(&c, "SignalDictMixin.__init__", sd_init);
    def_arity(&c, "SignalDictMixin.__len__", sd_len, 0, 0);
    def_arity(&c, "SignalDictMixin.__iter__", sd_iter, 0, 0);
    def_arity(&c, "SignalDictMixin.__repr__", sd_repr, 0, 0);
    def_arity(&c, "SignalDictMixin.copy", sd_copy, 0, 0);
    def_arity(&c, "SignalDictMixin.__getitem__", sd_getitem, 1, 1);
    def_arity(&c, "SignalDictMixin.__delitem__", sd_delitem, 1, 1);
    def_arity(&c, "SignalDictMixin.__eq__", sd_eq, 1, 1);
    def_arity(&c, "SignalDictMixin.__ne__", sd_ne, 1, 1);
    def_arity(&c, "SignalDictMixin.__setitem__", sd_setitem, 2, 2);
    c.dict.borrow_mut().set_s("__hash__", Value::None);
    c
}

fn manager_type(vm: &Vm) -> Rc<Class> {
    let c = native_type(vm, "ContextManager");
    def_static(&c, "ContextManager.__new__", cm_new);
    def_arity(&c, "ContextManager.__enter__", cm_enter, 0, 0);
    def(&c, "ContextManager.__exit__", cm_exit);
    c
}

/// an exception class of the module: type(name, bases, {...})
fn exception(vm: &mut Vm, name: &str, bases: Vec<Value>) -> PyResult<Value> {
    let ns = new_dict();
    ns.borrow_mut().set_s("__module__", Value::str("decimal"));
    let t = Value::Class(vm.t.type_.clone());
    vm.call_vec(&t, vec![Value::str(name), Value::tuple(bases), Value::Dict(ns)], &[])
}

pub fn decimal_module(vm: &mut Vm) -> PyResult<Value> {
    let (d, m) = crate::modules::module("decimal");
    let decimal = decimal_type(vm);
    let context = context_type(vm);
    let manager = manager_type(vm);
    let mixin = mixin_type(vm);
    for c in [&decimal, &context, &manager, &mixin] {
        vm.update_flags(c);
    }

    // numbers.Number.register(Decimal)
    let numbers = vm.import_one("numbers")?;
    let number = vm.getattr(&numbers, &ps("Number"))?;
    let register = vm.getattr(&number, &ps("register"))?;
    vm.call_vec(&register, vec![Value::Class(decimal.clone())], &[])?;
    let rational = vm.getattr(&numbers, &ps("Rational"))?;

    let collections = vm.import_one("collections")?;
    let namedtuple = vm.getattr(&collections, &ps("namedtuple"))?;
    let tuple = vm.call_vec(&namedtuple, vec![Value::str("DecimalTuple"), Value::str("sign digits exponent")], &[])?;
    vm.setattr(&tuple, &ps("__module__"), Value::str("decimal"))?;

    let abc = vm.import_one("collections.abc")?;
    let mm = vm.getattr(&abc, &ps("MutableMapping"))?;
    let t = Value::Class(vm.t.type_.clone());
    let sd = vm.call_vec(&t, vec![Value::str("SignalDict"), Value::tuple(vec![Value::Class(mixin.clone()), mm]), Value::Dict(new_dict())], &[])?;
    let Value::Class(signal_dict) = sd else {
        return Err(vm.type_error("SignalDict is not a class"));
    };

    // the signals, made from the last: Overflow and Underflow derive from
    // Inexact, Rounded and Subnormal
    let arith = Value::Class(vm.exc_class("ArithmeticError"));
    let base_exc = exception(vm, "DecimalException", vec![arith])?;
    let mut signals = vec![Value::None; 9];
    for i in (0..9).rev() {
        let bases = match SIGNAL_NAMES[i] {
            "FloatOperation" => vec![base_exc.clone(), Value::Class(vm.exc_class("TypeError"))],
            "DivisionByZero" => vec![base_exc.clone(), Value::Class(vm.exc_class("ZeroDivisionError"))],
            "Overflow" => vec![signals[6].clone(), signals[7].clone()],
            "Underflow" => vec![signals[6].clone(), signals[7].clone(), signals[5].clone()],
            _ => vec![base_exc.clone()],
        };
        signals[i] = exception(vm, SIGNAL_NAMES[i], bases)?;
    }
    let mut conds = vec![];
    for name in COND_NAMES {
        let mut bases = vec![signals[0].clone()];
        if name == "DivisionUndefined" {
            bases.push(Value::Class(vm.exc_class("ZeroDivisionError")));
        }
        conds.push(exception(vm, name, bases)?);
    }

    let cv = vm.import_one("contextvars")?;
    let Value::Module(cvm) = &cv else {
        return Err(vm.type_error("contextvars is not a module"));
    };
    let cv_ns = cvm.dict.clone();
    let cvar = vm.getattr(&cv, &ps("ContextVar"))?;
    let var = vm.call_vec(&cvar, vec![Value::str("decimal_context")], &[])?;

    // interned, so they are the same objects as _pydecimal's
    let rounds: Vec<Value> = Round::ALL.iter().map(|r| Value::Str(intern_str(&ps(r.name())))).collect();
    let st = Rc::new(State {
        ns: Rc::as_ptr(&d) as usize,
        decimal: decimal.clone(),
        context: context.clone(),
        manager: manager.clone(),
        mixin: mixin.clone(),
        signal_dict,
        tuple: tuple.clone(),
        rational,
        signal_tuple: Value::tuple(signals.clone()),
        signals: signals.clone(),
        conds: conds.clone(),
        rounds: rounds.clone(),
        var,
        cv_ns,
        templates: RefCell::new(vec![]),
        pydecimal: RefCell::new(Value::None),
    });
    STATES.with(|s| s.borrow_mut().push(st.clone()));

    // DefaultContext, BasicContext, ExtendedContext
    let dflt = default_ctx();
    let default = new_context(vm, &st, &context, dflt, 0, DEFAULT_TRAPS, true);
    let basic = new_context(vm, &st, &context, Ctx { prec: 9, round: Round::HalfUp, ..dflt }, 0, DEFAULT_TRAPS | mpdec::UNDERFLOW | mpdec::CLAMPED, true);
    let extended = new_context(vm, &st, &context, Ctx { prec: 9, ..dflt }, 0, 0, true);
    *st.templates.borrow_mut() = vec![default.clone(), basic.clone(), extended.clone()];

    for c in [&decimal, &context, &manager, &mixin] {
        c.immutable.set(true);
    }
    let mut ns = d.borrow_mut();
    ns.set_s("Decimal", Value::Class(decimal));
    ns.set_s("Context", Value::Class(context));
    ns.set_s("DecimalTuple", tuple);
    ns.set_s("DecimalException", base_exc);
    for i in 0..9 {
        ns.set_s(SIGNAL_NAMES[i], signals[i].clone());
    }
    for (i, name) in COND_NAMES.iter().enumerate() {
        ns.set_s(name, conds[i].clone());
    }
    ns.set_s("DefaultContext", default);
    ns.set_s("BasicContext", basic);
    ns.set_s("ExtendedContext", extended);
    ns.set_s("HAVE_CONTEXTVAR", Value::Bool(true));
    ns.set_s("HAVE_THREADS", Value::Bool(true));
    ns.set_s("MAX_PREC", Value::Int(mpdec::MAX_PREC));
    ns.set_s("MAX_EMAX", Value::Int(mpdec::MAX_EMAX));
    ns.set_s("MIN_EMIN", Value::Int(mpdec::MIN_EMIN));
    ns.set_s("MIN_ETINY", Value::Int(mpdec::MIN_ETINY));
    for (i, r) in Round::ALL.iter().enumerate() {
        ns.set_s(r.name(), rounds[i].clone());
    }
    ns.set_s("__version__", Value::str("1.70"));
    ns.set_s("__libmpdec_version__", Value::str("2.5.1"));
    // the module functions take the module as self, to find its state
    let fns: [(&'static str, NativeFnPtr); 3] = [("_decimal.getcontext", m_getcontext), ("_decimal.setcontext", m_setcontext), ("_decimal.localcontext", m_localcontext)];
    for (name, f) in fns {
        ns.set_s(&name[9..], Value::Method(Rc::new((m.clone(), native(name, f)))));
    }
    drop(ns);
    Ok(m)
}
