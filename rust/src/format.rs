// SPDX-License-Identifier: MIT
//! repr/str of built-in values, float formatting, the format-spec
//! mini-language and %-formatting.

use crate::value::*;
use crate::vm::Vm;
use std::rc::Rc;

// -------------------------------------------------------------------------
// floats

/// shortest digits and decimal exponent: x = 0.d1d2d3... * 10^(exp+1)
fn shortest(x: f64) -> (String, i32) {
    let s = format!("{:e}", x.abs());
    let (m, e) = s.split_once('e').unwrap();
    let digits: String = m.chars().filter(|c| *c != '.').collect();
    (digits, e.parse().unwrap())
}

/// repr(float)
pub fn float_repr(x: f64) -> String {
    if x.is_nan() {
        return "nan".into();
    }
    if x.is_infinite() {
        return if x > 0.0 { "inf".into() } else { "-inf".into() };
    }
    if x == 0.0 {
        return if x.is_sign_negative() { "-0.0".into() } else { "0.0".into() };
    }
    let (digits, exp) = shortest(x);
    let sign = if x < 0.0 { "-" } else { "" };
    if (-4..16).contains(&exp) {
        let s = if exp >= 0 {
            let e = exp as usize;
            if digits.len() <= e + 1 {
                format!("{}{}.0", digits, "0".repeat(e + 1 - digits.len()))
            } else {
                format!("{}.{}", &digits[..e + 1], &digits[e + 1..])
            }
        } else {
            format!("0.{}{}", "0".repeat((-exp - 1) as usize), digits)
        };
        format!("{}{}", sign, s)
    } else {
        let mant = if digits.len() == 1 { digits.clone() } else { format!("{}.{}", &digits[..1], &digits[1..]) };
        format!("{}{}e{}{:02}", sign, mant, if exp < 0 { '-' } else { '+' }, exp.abs())
    }
}

/// repr of a float inside a complex: no ".0" on whole numbers
fn complex_part(x: f64) -> String {
    let r = float_repr(x);
    match r.strip_suffix(".0") {
        Some(s) => s.to_string(),
        None => r,
    }
}

pub fn complex_repr(re: f64, im: f64) -> String {
    if re == 0.0 && !re.is_sign_negative() {
        return format!("{}j", complex_part(im));
    }
    let ims = complex_part(im);
    let sign = if ims.starts_with('-') || im.is_nan() && ims.starts_with('-') { "" } else { "+" };
    format!("({}{}{}j)", complex_part(re), sign, ims)
}

fn exp_fix(s: String) -> String {
    // Rust writes 1.5e3 / 1.5e-3; Python writes 1.5e+03 / 1.5e-03
    match s.find('e') {
        Some(i) => {
            let (m, e) = s.split_at(i);
            let e = &e[1..];
            let (sign, digits) = if let Some(d) = e.strip_prefix('-') { ('-', d) } else { ('+', e) };
            format!("{}e{}{:0>2}", m, sign, digits)
        }
        None => s,
    }
}

fn strip_zeros(s: &str) -> String {
    if let Some(i) = s.find('e') {
        let (m, e) = s.split_at(i);
        return format!("{}{}", strip_zeros(m), e);
    }
    if s.contains('.') {
        let t = s.trim_end_matches('0');
        t.trim_end_matches('.').to_string()
    } else {
        s.to_string()
    }
}

/// a float in one of the e/f/g/% presentations (no sign handling for
/// negative values beyond what the digits carry)
/// x with p digits after the point; past 1100 digits a double's exact
/// expansion has ended, so the rest are zeros
fn fixed(x: f64, p: usize) -> String {
    if p <= 1100 {
        return format!("{:.*}", p, x);
    }
    let mut s = format!("{:.*}", 1100, x);
    s.push_str(&"0".repeat(p - 1100));
    s
}

fn sci(x: f64, p: usize) -> String {
    if p <= 800 {
        return format!("{:.*e}", p, x);
    }
    let s = format!("{:.*e}", 800, x);
    let i = s.find('e').unwrap();
    format!("{}{}{}", &s[..i], "0".repeat(p - 800), &s[i..])
}

pub fn fmt_float(x: f64, ty: char, prec: Option<usize>, alt: bool) -> String {
    if x.is_nan() {
        return if ty.is_ascii_uppercase() { "NAN".into() } else { "nan".into() };
    }
    if x.is_infinite() {
        let s = if x > 0.0 { "inf" } else { "-inf" };
        return if ty.is_ascii_uppercase() { s.to_uppercase() } else { s.into() };
    }
    match ty {
        'f' | 'F' => {
            let p = prec.unwrap_or(6);
            let mut s = fixed(x, p);
            if alt && p == 0 {
                s.push('.');
            }
            s
        }
        'e' | 'E' => {
            let p = prec.unwrap_or(6);
            let mut s = exp_fix(sci(x, p));
            if alt && p == 0 {
                let i = s.find('e').unwrap();
                s.insert(i, '.');
            }
            if ty == 'E' {
                s = s.to_uppercase();
            }
            s
        }
        '%' => {
            let p = prec.unwrap_or(6);
            format!("{}%", fixed(x * 100.0, p))
        }
        'g' | 'G' | 'n' => {
            let p = prec.unwrap_or(6).max(1);
            if x == 0.0 {
                let z = if x.is_sign_negative() { "-0" } else { "0" };
                return if alt { format!("{}.{}", z, "0".repeat(p - 1)) } else { z.into() };
            }
            let e = sci(x, p - 1);
            let exp: i32 = e[e.find('e').unwrap() + 1..].parse().unwrap();
            let s = if exp >= -4 && exp < p as i32 {
                fixed(x, (p as i32 - 1 - exp).max(0) as usize)
            } else {
                exp_fix(e)
            };
            let s = if alt { s } else { strip_zeros(&s) };
            if ty == 'G' {
                s.to_uppercase()
            } else {
                s
            }
        }
        // no type: like g with at least one digit after the point, or repr
        _ => match prec {
            None => float_repr(x),
            Some(p) => {
                let p = p.max(1);
                let e = sci(x, p - 1);
                let exp: i32 = e[e.find('e').unwrap() + 1..].parse().unwrap();
                if exp >= -4 && exp < p as i32 {
                    let s = strip_zeros(&fixed(x, (p as i32 - 1 - exp).max(0) as usize));
                    if s.contains('.') || s.contains("inf") || s.contains("nan") {
                        s
                    } else {
                        format!("{}.0", s)
                    }
                } else {
                    strip_zeros(&exp_fix(e))
                }
            }
        },
    }
}

// -------------------------------------------------------------------------
// repr

pub fn str_repr(s: &str) -> String {
    let q = if s.contains('\'') && !s.contains('"') { '"' } else { '\'' };
    let mut out = String::with_capacity(s.len() + 2);
    out.push(q);
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c == q => {
                out.push('\\');
                out.push(c);
            }
            c if (c as u32) < 0x20 || (c as u32) == 0x7f => out.push_str(&format!("\\x{:02x}", c as u32)),
            c if (0x80..0xa0).contains(&(c as u32)) || c == '\u{ad}' => out.push_str(&format!("\\x{:02x}", c as u32)),
            c if is_unprintable(c) => {
                let n = c as u32;
                if n <= 0xffff {
                    out.push_str(&format!("\\u{:04x}", n))
                } else {
                    out.push_str(&format!("\\U{:08x}", n))
                }
            }
            c => out.push(c),
        }
    }
    out.push(q);
    out
}

fn is_unprintable(c: char) -> bool {
    let n = c as u32;
    matches!(n, 0x2028 | 0x2029 | 0x200b..=0x200f | 0x202a..=0x202e | 0x2060..=0x2064 | 0xfeff | 0xd800..=0xdfff | 0xe000..=0xf8ff | 0xfff9..=0xfffb | 0xe0000..=0xe007f | 0xf0000..=0x10ffff)
        || (0x1680 == n || (0x2000..=0x200a).contains(&n) || n == 0x3000 || n == 0x205f || n == 0x202f || n == 0xa0)
}

pub fn bytes_repr(b: &[u8]) -> String {
    let q = if b.contains(&b'\'') && !b.contains(&b'"') { '"' } else { '\'' };
    let mut out = String::from("b");
    out.push(q);
    for &c in b {
        match c {
            b'\\' => out.push_str("\\\\"),
            b'\n' => out.push_str("\\n"),
            b'\r' => out.push_str("\\r"),
            b'\t' => out.push_str("\\t"),
            c if c as char == q => {
                out.push('\\');
                out.push(c as char);
            }
            0x20..=0x7e => out.push(c as char),
            c => out.push_str(&format!("\\x{:02x}", c)),
        }
    }
    out.push(q);
    out
}

pub fn ascii_escape(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        let n = c as u32;
        if n < 128 {
            out.push(c);
        } else if n < 256 {
            out.push_str(&format!("\\x{:02x}", n));
        } else if n < 0x10000 {
            out.push_str(&format!("\\u{:04x}", n));
        } else {
            out.push_str(&format!("\\U{:08x}", n));
        }
    }
    out
}

fn addr(v: &Value) -> String {
    format!("0x{:012x}", ptr_id(v))
}

fn iter_kind(it: &Iter) -> &'static str {
    match it {
        Iter::List(..) => "list_iterator",
        Iter::ListRev(..) => "list_reverseiterator",
        Iter::Tuple(..) => "tuple_iterator",
        Iter::Str(..) => "str_ascii_iterator",
        Iter::Bytes(..) => "bytes_iterator",
        Iter::Range(..) => "range_iterator",
        Iter::Dict(_, _, k, _) => match k {
            0 => "dict_keyiterator",
            1 => "dict_valueiterator",
            _ => "dict_itemiterator",
        },
        Iter::Enumerate(..) => "enumerate",
        Iter::Zip(..) => "zip",
        Iter::Map(..) => "map",
        Iter::Filter(..) => "filter",
        Iter::Seq(..) => "iterator",
        Iter::Callable(..) => "callable_iterator",
        Iter::Count(..) => "itertools.count",
        Iter::Repeat(..) => "itertools.repeat",
        Iter::Chain(..) => "itertools.chain",
        Iter::Empty | Iter::Vec(_) => "iterator",
    }
}

pub fn class_repr(c: &Rc<Class>) -> String {
    let module = c.dict.borrow().get_s("__module__");
    match module {
        Some(Value::Str(m)) if &*m.s != "builtins" => format!("<class '{}.{}'>", m.s, c.qualname.borrow().s),
        _ => format!("<class '{}'>", c.name()),
    }
}

/// guards against a container that holds itself
fn enter(vm: &mut Vm, v: &Value) -> bool {
    let id = ptr_id(v);
    if vm.repr_stack.contains(&id) {
        return false;
    }
    vm.repr_stack.push(id);
    true
}

fn leave(vm: &mut Vm) {
    vm.repr_stack.pop();
}

fn join_reprs(vm: &mut Vm, items: &[Value]) -> PyResult<String> {
    let mut out = String::new();
    for (i, x) in items.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        out.push_str(&repr(vm, x)?);
    }
    Ok(out)
}

pub fn repr(vm: &mut Vm, v: &Value) -> PyResult<String> {
    match v {
        Value::List(_) | Value::Tuple(_) | Value::Dict(_) | Value::Set(_) | Value::FrozenSet(_) | Value::DictView(_) | Value::Instance(_) => {
            vm.enter(" while getting the repr of an object")?;
            let r = repr_inner(vm, v);
            vm.depth -= 1;
            r
        }
        _ => repr_inner(vm, v),
    }
}

fn repr_inner(vm: &mut Vm, v: &Value) -> PyResult<String> {
    Ok(match v {
        Value::Undef | Value::None => "None".into(),
        Value::NotImplemented => "NotImplemented".into(),
        Value::Ellipsis => "Ellipsis".into(),
        Value::Bool(b) => if *b { "True" } else { "False" }.into(),
        Value::Int(i) => i.to_string(),
        Value::Big(b) => big_decimal(vm, b)?,
        Value::Float(f) => float_repr(*f),
        Value::Complex(c) => complex_repr(c.0, c.1),
        Value::Str(s) => str_repr(&s.s),
        Value::Bytes(b) => bytes_repr(b),
        Value::ByteArray(b) => format!("bytearray({})", bytes_repr(&b.borrow())),
        Value::List(l) => {
            if !enter(vm, v) {
                return Ok("[...]".into());
            }
            let items = l.borrow().clone();
            let r = join_reprs(vm, &items);
            leave(vm);
            format!("[{}]", r?)
        }
        Value::Tuple(t) => {
            if t.len() == 1 {
                format!("({},)", repr(vm, &t[0])?)
            } else {
                format!("({})", join_reprs(vm, t)?)
            }
        }
        Value::Dict(d) => {
            if !enter(vm, v) {
                return Ok("{...}".into());
            }
            let r = dict_body(vm, d);
            leave(vm);
            format!("{{{}}}", r?)
        }
        Value::Set(d) => {
            let keys = d.borrow().keys();
            if keys.is_empty() {
                "set()".into()
            } else {
                if !enter(vm, v) {
                    return Ok("{...}".into());
                }
                let r = join_reprs(vm, &keys);
                leave(vm);
                format!("{{{}}}", r?)
            }
        }
        Value::FrozenSet(d) => {
            let keys = d.borrow().keys();
            if keys.is_empty() {
                "frozenset()".into()
            } else {
                format!("frozenset({{{}}})", join_reprs(vm, &keys)?)
            }
        }
        Value::Range(r) => {
            if r.2 == 1 {
                format!("range({}, {})", r.0, r.1)
            } else {
                format!("range({}, {}, {})", r.0, r.1, r.2)
            }
        }
        Value::Slice(s) => format!("slice({}, {}, {})", repr(vm, &s.0)?, repr(vm, &s.1)?, repr(vm, &s.2)?),
        Value::Func(f) => format!("<function {} at {}>", f.qualname.borrow().s, addr(v)),
        Value::Native(n) => format!("<built-in function {}>", n.name.rsplit('.').next().unwrap_or(n.name)),
        Value::Method(m) => match &m.1 {
            Value::Func(f) => {
                let owner = match &m.0 {
                    Value::Class(_) => repr(vm, &m.0)?,
                    o => repr(vm, o)?,
                };
                format!("<bound method {} of {}>", f.qualname.borrow().s, owner)
            }
            Value::Native(n) => {
                let short = n.name.rsplit('.').next().unwrap_or(n.name);
                let tn = vm.type_name(&m.0);
                match &m.0 {
                    Value::Module(md) => format!("<built-in function {}>", if short.is_empty() { &md.name.s } else { short }),
                    _ => format!("<built-in method {} of {} object at {}>", short, tn, addr(&m.0)),
                }
            }
            other => repr(vm, other)?,
        },
        Value::Class(c) => {
            let meta = c.meta.borrow().clone();
            if let Some(m) = meta {
                if !Rc::ptr_eq(&m, &vm.t.type_) {
                    if let Some(f @ Value::Func(_)) = m.lookup(&vm.n.repr) {
                        let r = vm.call_vec(&f, vec![v.clone()], &[])?;
                        return vm.str_of(&r);
                    }
                }
            }
            class_repr(c)
        }
        Value::Instance(inst) => return instance_repr(vm, v, inst),
        Value::Module(m) => {
            let file = m.dict.borrow().get_s("__file__");
            match file {
                Some(Value::Str(f)) => format!("<module '{}' from '{}'>", m.name.s, f.s),
                _ => format!("<module '{}' (built-in)>", m.name.s),
            }
        }
        Value::Iter(it) => format!("<{} object at {}>", iter_kind(&it.borrow()), addr(v)),
        Value::Gen(g) => {
            let g = g.borrow();
            format!("<{} object {} at {}>", if g.coroutine { "coroutine" } else { "generator" }, g.qualname.s, addr(v))
        }
        Value::DictView(dv) => {
            let (name, items) = match dv.1 {
                0 => ("dict_keys", dv.0.borrow().keys()),
                1 => ("dict_values", dv.0.borrow().values()),
                _ => ("dict_items", dv.0.borrow().items().into_iter().map(|(k, v)| Value::tuple(vec![k, v])).collect()),
            };
            format!("{}([{}])", name, join_reprs(vm, &items)?)
        }
        Value::Property(_) => format!("<property object at {}>", addr(v)),
        Value::StaticMethod(f) => format!("<staticmethod({})>", repr(vm, f)?),
        Value::ClassMethod(f) => format!("<classmethod({})>", repr(vm, f)?),
        Value::Super(s) => format!("<super: <class '{}'>, <{} object>>", s.0.name(), vm.type_name(&s.1)),
        Value::Cell(_) => format!("<cell at {}>", addr(v)),
        Value::Code(c) => format!("<code object {} at {}, file \"{}\", line {}>", c.name.s, addr(v), c.filename, c.firstline),
        Value::Obj(o) => {
            if let Some(f) = o.class.lookup(&vm.n.repr) {
                let f = vm.bind(f, v, &o.class.clone());
                let r = vm.call_vec(&f, vec![], &[])?;
                return vm.str_of(&r);
            }
            format!("<{} object at {}>", o.class.name(), addr(v))
        }
    })
}

fn dict_body(vm: &mut Vm, d: &Rc<std::cell::RefCell<Dict>>) -> PyResult<String> {
    let items = d.borrow().items();
    let mut out = String::new();
    for (i, (k, val)) in items.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        out.push_str(&repr(vm, k)?);
        out.push_str(": ");
        out.push_str(&repr(vm, val)?);
    }
    Ok(out)
}

fn instance_repr(vm: &mut Vm, v: &Value, inst: &Rc<Instance>) -> PyResult<String> {
    let cls = inst.class.borrow().clone();
    if let Some(f) = cls.lookup(&vm.n.repr) {
        if !matches!(f, Value::Native(_)) {
            if !enter(vm, v) {
                return Ok("...".into());
            }
            let f = vm.bind(f, v, &cls);
            let r = vm.call_vec(&f, vec![], &[]);
            leave(vm);
            let r = r?;
            return match &r {
                Value::Str(s) => Ok(s.s.clone()),
                _ => match vm.base_of(&r) {
                    Value::Str(s) => Ok(s.s.clone()),
                    _ => {
                        let tn = vm.type_name(&r);
                        Err(vm.type_error(format!("__repr__ returned non-string (type {})", tn)))
                    }
                },
            };
        }
        if let Value::Native(n) = &f {
            if n.name != "object.__repr__" && n.name != "BaseException.__repr__" {
                let r = vm.call_vec(&f, vec![v.clone()], &[])?;
                return vm.str_of(&r);
            }
        }
    }
    if cls.builtin.get() == Builtin::BaseException {
        let args = vm.exc_args(v);
        let name = cls.name();
        return Ok(if args.len() == 1 { format!("{}({})", name, repr(vm, &args[0])?) } else { format!("{}({})", name, join_reprs(vm, &args)?) });
    }
    let b = inst.base.borrow().clone();
    if !b.is_undef() {
        // a subclass of a built-in shows as the built-in value
        return match &b {
            Value::List(_) | Value::Dict(_) | Value::Set(_) | Value::Tuple(_) | Value::Str(_) | Value::Int(_) | Value::Float(_) | Value::Bool(_) | Value::Big(_) | Value::FrozenSet(_) | Value::Bytes(_) | Value::Complex(_) => {
                let r = repr(vm, &b)?;
                if matches!(b, Value::Set(_) | Value::FrozenSet(_)) && !r.starts_with("set") {
                    Ok(format!("{}({})", cls.name(), r))
                } else {
                    Ok(r)
                }
            }
            _ => repr(vm, &b),
        };
    }
    Ok(object_repr(vm, v))
}

/// object.__repr__: <module.Class object at 0x...>
pub fn object_repr(vm: &mut Vm, v: &Value) -> String {
    let cls = vm.class_of(v);
    let module = cls.dict.borrow().get_s("__module__");
    let qn = cls.qualname.borrow().s.clone();
    let name = match module {
        Some(Value::Str(m)) if &*m.s != "builtins" => format!("{}.{}", m.s, qn),
        _ => qn,
    };
    format!("<{} object at {}>", name, addr(v))
}

/// str() of a value that is not an instance
pub fn str_builtin(vm: &mut Vm, v: &Value) -> PyResult<String> {
    match v {
        Value::Str(s) => Ok(s.s.clone()),
        Value::Float(f) => Ok(float_repr(*f)),
        Value::Int(i) => Ok(i.to_string()),
        _ => repr(vm, v),
    }
}

// -------------------------------------------------------------------------
// format specs

#[derive(Default, Debug)]
pub struct Spec {
    pub fill: char,
    pub align: Option<char>,
    pub sign: Option<char>,
    pub z: bool,
    pub alt: bool,
    pub zero: bool,
    pub width: usize,
    pub grouping: Option<char>,
    pub prec: Option<usize>,
    pub ty: Option<char>,
}

pub fn parse_spec(s: &str) -> Option<Spec> {
    let c: Vec<char> = s.chars().collect();
    let mut i = 0;
    let mut sp = Spec { fill: ' ', ..Default::default() };
    let is_align = |ch: char| matches!(ch, '<' | '>' | '^' | '=');
    if c.len() >= 2 && is_align(c[1]) {
        sp.fill = c[0];
        sp.align = Some(c[1]);
        i = 2;
    } else if !c.is_empty() && is_align(c[0]) {
        sp.align = Some(c[0]);
        i = 1;
    }
    if i < c.len() && matches!(c[i], '+' | '-' | ' ') {
        sp.sign = Some(c[i]);
        i += 1;
    }
    if i < c.len() && c[i] == 'z' {
        sp.z = true;
        i += 1;
    }
    if i < c.len() && c[i] == '#' {
        sp.alt = true;
        i += 1;
    }
    if i < c.len() && c[i] == '0' {
        sp.zero = true;
        i += 1;
    }
    let st = i;
    while i < c.len() && c[i].is_ascii_digit() {
        i += 1;
    }
    if i > st {
        sp.width = c[st..i].iter().collect::<String>().parse().ok()?;
        // CPython's widths are Py_ssize_t; past what memory could hold
        // the answer is an error either way
        if sp.width > 1 << 40 {
            return None;
        }
    }
    if i < c.len() && matches!(c[i], ',' | '_') {
        sp.grouping = Some(c[i]);
        i += 1;
    }
    if i < c.len() && c[i] == '.' {
        i += 1;
        let st = i;
        while i < c.len() && c[i].is_ascii_digit() {
            i += 1;
        }
        if i == st {
            return None;
        }
        let pv: usize = c[st..i].iter().collect::<String>().parse().ok()?;
        if pv > i32::MAX as usize {
            return None;
        }
        sp.prec = Some(pv);
    }
    if i < c.len() {
        sp.ty = Some(c[i]);
        i += 1;
    }
    if i != c.len() {
        return None;
    }
    Some(sp)
}

fn group(digits: &str, sep: char, every: usize) -> String {
    let (int, frac) = match digits.find(|c: char| !c.is_ascii_alphanumeric()) {
        Some(i) => (&digits[..i], &digits[i..]),
        None => (digits, ""),
    };
    let n = int.len();
    let mut out = String::new();
    for (i, ch) in int.chars().enumerate() {
        if i > 0 && (n - i) % every == 0 {
            out.push(sep);
        }
        out.push(ch);
    }
    out.push_str(frac);
    out
}

/// sign + body padded per the spec; `numeric` turns on '=' and 0-padding
fn pad(sp: &Spec, sign: &str, body: &str, numeric: bool, default_left: bool) -> String {
    let len = sign.chars().count() + body.chars().count();
    if sp.width <= len {
        return format!("{}{}", sign, body);
    }
    let n = sp.width - len;
    let (fill, align) = if sp.zero && sp.align.is_none() && numeric { ('0', '=') } else { (sp.fill, sp.align.unwrap_or(if default_left { '<' } else { '>' })) };
    let f = |k: usize| std::iter::repeat(fill).take(k).collect::<String>();
    match align {
        '<' => format!("{}{}{}", sign, body, f(n)),
        '^' => format!("{}{}{}{}", f(n / 2), sign, body, f(n - n / 2)),
        '=' => format!("{}{}{}", sign, f(n), body),
        _ => format!("{}{}{}", f(n), sign, body),
    }
}

fn sign_str(sp: &Spec, neg: bool) -> &'static str {
    if neg {
        "-"
    } else {
        match sp.sign {
            Some('+') => "+",
            Some(' ') => " ",
            _ => "",
        }
    }
}

/// sys.set_int_max_str_digits: 0 is no limit
pub static MAX_STR_DIGITS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(4300);

pub fn max_str_digits() -> usize {
    MAX_STR_DIGITS.load(std::sync::atomic::Ordering::Relaxed)
}

/// A big int in decimal, under the interpreter's digit limit
pub fn big_decimal(vm: &mut Vm, b: &crate::bigint::BigInt) -> PyResult<String> {
    let lim = max_str_digits();
    let err = |vm: &mut Vm| vm.value_error(format!("Exceeds the limit ({}) for integer string conversion; use sys.set_int_max_str_digits() to increase the limit", lim));
    if lim > 0 {
        // at least this many decimal digits: fail before the slow conversion
        let low = ((b.bit_length().saturating_sub(1)) as f64 * std::f64::consts::LOG10_2) as usize;
        if low > lim {
            return Err(err(vm));
        }
    }
    let s = b.to_string();
    if lim > 0 && s.trim_start_matches('-').len() > lim {
        return Err(err(vm));
    }
    Ok(s)
}

/// int(str) under the digit limit; power-of-two bases are not limited
pub fn check_str_digits(vm: &mut Vm, s: &str, base: u32) -> PyResult<()> {
    let lim = max_str_digits();
    if lim == 0 || base & (base.wrapping_sub(1)) == 0 || s.len() <= lim {
        return Ok(());
    }
    let n = s.chars().filter(|c| c.is_alphanumeric()).count();
    if n > lim {
        return Err(vm.value_error(format!("Exceeds the limit ({}) for integer string conversion: value has {} digits; use sys.set_int_max_str_digits() to increase the limit", lim, n)));
    }
    Ok(())
}

fn format_int(vm: &mut Vm, v: &crate::bigint::BigInt, sp: &Spec) -> PyResult<String> {
    let ty = sp.ty.unwrap_or('d');
    if sp.prec.is_some() && matches!(ty, 'd' | 'b' | 'o' | 'x' | 'X' | 'c' | 'n') {
        return Err(vm.value_error("Precision not allowed in integer format specifier"));
    }
    if matches!(ty, 'e' | 'E' | 'f' | 'F' | 'g' | 'G' | '%') {
        return format_float(vm, v.to_f64(), sp);
    }
    let neg = v.neg;
    let mag = v.abs();
    let (prefix, digits) = match ty {
        'd' | 'n' => ("", big_decimal(vm, &mag)?),
        'b' => ("0b", mag.to_radix(2)),
        'o' => ("0o", mag.to_radix(8)),
        'x' => ("0x", mag.to_radix(16)),
        'X' => ("0X", mag.to_radix(16).to_uppercase()),
        'c' => {
            let c = v.to_i64().and_then(|i| char::from_u32(i as u32)).ok_or_else(|| vm.err("OverflowError", "%c arg not in range(0x110000)"))?;
            return Ok(pad(sp, "", &c.to_string(), false, false));
        }
        other => return Err(vm.value_error(format!("Unknown format code '{}' for object of type 'int'", other))),
    };
    let digits = match sp.grouping {
        Some(g) => group(&digits, g, if matches!(ty, 'd' | 'n') { 3 } else { 4 }),
        None => digits,
    };
    let sign = format!("{}{}", sign_str(sp, neg), if sp.alt { prefix } else { "" });
    if sp.zero && sp.grouping.is_some() && sp.align.is_none() && sp.width > sign.len() + digits.len() {
        // zero padding takes part in the grouping
        let g = sp.grouping.unwrap();
        let every = if matches!(ty, 'd' | 'n') { 3 } else { 4 };
        let raw = match ty {
            'd' | 'n' => mag.to_radix(10),
            'b' => mag.to_radix(2),
            'o' => mag.to_radix(8),
            'x' => mag.to_radix(16),
            _ => mag.to_radix(16).to_uppercase(),
        };
        let mut z = raw;
        loop {
            let gz = group(&z, g, every);
            if sign.len() + gz.len() >= sp.width {
                return Ok(format!("{}{}", sign, gz));
            }
            z.insert(0, '0');
        }
    }
    Ok(pad(sp, &sign, &digits, true, false))
}

fn format_float(vm: &mut Vm, x: f64, sp: &Spec) -> PyResult<String> {
    let ty = sp.ty.unwrap_or('\0');
    if !matches!(ty, 'e' | 'E' | 'f' | 'F' | 'g' | 'G' | 'n' | '%' | '\0') {
        return Err(vm.value_error(format!("Unknown format code '{}' for object of type 'float'", ty)));
    }
    let neg = x.is_sign_negative() && !x.is_nan();
    let mut body = fmt_float(x.abs(), ty, sp.prec, sp.alt);
    let mut neg = neg;
    if sp.z && neg && body.chars().all(|c| matches!(c, '0' | '.' | '%' | 'e' | 'E' | '+' | '-')) {
        neg = false;
    }
    if x == 0.0 && neg && ty == '\0' && sp.prec.is_none() {
        body = "0.0".into();
    }
    if let Some(g) = sp.grouping {
        body = group(&body, g, 3);
    }
    Ok(pad(sp, sign_str(sp, neg), &body, true, false))
}

pub fn format_spec(vm: &mut Vm, v: &Value, spec: &str) -> PyResult<String> {
    let sp = match parse_spec(spec) {
        Some(s) => s,
        None => return Err(vm.value_error("Invalid format specifier")),
    };
    match v {
        Value::Bool(b) if sp.ty.is_none() => Ok(pad(&sp, "", if *b { "True" } else { "False" }, false, true)),
        Value::Int(_) | Value::Big(_) | Value::Bool(_) => {
            let b = crate::ops::big_of(v).unwrap();
            format_int(vm, &b, &sp)
        }
        Value::Float(f) => format_float(vm, *f, &sp),
        Value::Str(s) => {
            if let Some(t) = sp.ty {
                if t != 's' {
                    return Err(vm.value_error(format!("Unknown format code '{}' for object of type 'str'", t)));
                }
            }
            if sp.sign.is_some() {
                return Err(vm.value_error("Sign not allowed in string format specifier"));
            }
            if sp.align == Some('=') {
                return Err(vm.value_error("'=' alignment not allowed in string format specifier"));
            }
            let body: String = match sp.prec {
                Some(p) => s.s.chars().take(p).collect(),
                None => s.s.clone(),
            };
            Ok(pad(&sp, "", &body, false, true))
        }
        Value::Complex(c) => {
            let (re, im) = (c.0, c.1);
            let ty = sp.ty.unwrap_or('\0');
            if ty == '\0' && sp.prec.is_none() {
                let body = complex_repr(re, im);
                return Ok(pad(&sp, "", &body, false, false));
            }
            let t = if ty == '\0' { 'g' } else { ty };
            let r = fmt_float(re, t, sp.prec, sp.alt);
            let i = fmt_float(im.abs(), t, sp.prec, sp.alt);
            let body = format!("{}{}{}j", r, if im < 0.0 { '-' } else { '+' }, i);
            Ok(pad(&sp, "", &body, false, false))
        }
        Value::None | Value::List(_) | Value::Tuple(_) | Value::Dict(_) => {
            let tn = vm.type_name(v);
            Err(vm.type_error(format!("unsupported format string passed to {}.__format__", tn)))
        }
        _ => {
            let tn = vm.type_name(v);
            Err(vm.type_error(format!("unsupported format string passed to {}.__format__", tn)))
        }
    }
}

// -------------------------------------------------------------------------
// printf-style

pub fn percent_format(vm: &mut Vm, fmt: &Value, args: &Value) -> PyResult<String> {
    let f = match fmt {
        Value::Str(s) => s.s.clone(),
        _ => vm.str_of(fmt)?,
    };
    let mapping = match args {
        Value::Dict(_) => true,
        Value::Instance(_) => matches!(vm.base_of(args), Value::Dict(_)) || vm.getattr_opt(args, &Rc::new(PyStr::new("keys".into())))?.is_some(),
        _ => false,
    };
    let items: Vec<Value> = match args {
        Value::Tuple(t) => t.to_vec(),
        _ => vec![args.clone()],
    };
    let mut ai = 0;
    let mut out = String::with_capacity(f.len() + 16);
    let c: Vec<char> = f.chars().collect();
    let mut i = 0;
    let mut used_mapping = false;
    while i < c.len() {
        if c[i] != '%' {
            out.push(c[i]);
            i += 1;
            continue;
        }
        i += 1;
        if i >= c.len() {
            return Err(vm.value_error("incomplete format"));
        }
        let mut key: Option<String> = None;
        if c[i] == '(' {
            let mut depth = 1;
            let st = i + 1;
            i += 1;
            while i < c.len() && depth > 0 {
                if c[i] == '(' {
                    depth += 1;
                } else if c[i] == ')' {
                    depth -= 1;
                }
                i += 1;
            }
            key = Some(c[st..i - 1].iter().collect());
        }
        let mut sp = Spec { fill: ' ', ..Default::default() };
        let mut left = false;
        while i < c.len() && matches!(c[i], '-' | '+' | ' ' | '#' | '0') {
            match c[i] {
                '-' => left = true,
                '+' => sp.sign = Some('+'),
                ' ' => {
                    if sp.sign.is_none() {
                        sp.sign = Some(' ')
                    }
                }
                '#' => sp.alt = true,
                _ => sp.zero = true,
            }
            i += 1;
        }
        let mut next_arg = |vm: &mut Vm| -> PyResult<Value> {
            if ai < items.len() {
                ai += 1;
                Ok(items[ai - 1].clone())
            } else {
                Err(vm.type_error("not enough arguments for format string"))
            }
        };
        if i < c.len() && c[i] == '*' {
            let w = next_arg(vm)?;
            let w = vm.to_index(&w)?;
            if w < 0 {
                left = true;
            }
            sp.width = w.unsigned_abs() as usize;
            i += 1;
        } else {
            let st = i;
            while i < c.len() && c[i].is_ascii_digit() {
                i += 1;
            }
            if i > st {
                let w: Option<usize> = c[st..i].iter().collect::<String>().parse().ok();
                match w {
                    Some(w) if w <= 1 << 40 => sp.width = w,
                    _ => return Err(vm.value_error("width too big")),
                }
            }
        }
        if i < c.len() && c[i] == '.' {
            i += 1;
            if i < c.len() && c[i] == '*' {
                let p = next_arg(vm)?;
                sp.prec = Some(vm.to_index(&p)?.max(0) as usize);
                i += 1;
            } else {
                let st = i;
                while i < c.len() && c[i].is_ascii_digit() {
                    i += 1;
                }
                let p: Option<usize> = c[st..i].iter().collect::<String>().parse().ok();
                match p {
                    Some(p) if p <= i32::MAX as usize => sp.prec = Some(p),
                    None if i == st => sp.prec = Some(0),
                    _ => return Err(vm.value_error("precision too big")),
                }
            }
        }
        while i < c.len() && matches!(c[i], 'h' | 'l' | 'L') {
            i += 1;
        }
        if i >= c.len() {
            return Err(vm.value_error("incomplete format"));
        }
        let conv = c[i];
        i += 1;
        if conv == '%' {
            out.push('%');
            continue;
        }
        if left {
            sp.align = Some('<');
            sp.zero = false;
        }
        let arg = match &key {
            Some(k) => {
                used_mapping = true;
                if !mapping {
                    return Err(vm.type_error("format requires a mapping"));
                }
                vm.getitem(args, &Value::str(k))?
            }
            None => next_arg(vm)?,
        };
        let piece = match conv {
            's' | 'r' | 'a' => {
                let s = match conv {
                    's' => vm.str_of(&arg)?,
                    'r' => vm.repr(&arg)?,
                    _ => ascii_escape(&vm.repr(&arg)?),
                };
                let s: String = match sp.prec {
                    Some(p) => s.chars().take(p).collect(),
                    None => s,
                };
                sp.zero = false;
                pad(&sp, "", &s, false, false)
            }
            'd' | 'i' | 'u' => {
                let n = match &arg {
                    Value::Float(x) => {
                        if !x.is_finite() {
                            return Err(vm.err("OverflowError", "cannot convert float infinity to integer"));
                        }
                        crate::bigint::BigInt::from_f64(x.trunc()).unwrap()
                    }
                    other => match crate::ops::big_of(&vm.base_of(other)) {
                        Some(b) => b,
                        None => {
                            let tn = vm.type_name(other);
                            return Err(vm.type_error(format!("%{} format: a real number is required, not {}", conv, tn)));
                        }
                    },
                };
                let mut digits = n.abs().to_string();
                if let Some(p) = sp.prec {
                    while digits.len() < p {
                        digits.insert(0, '0');
                    }
                }
                pad(&sp, sign_str(&sp, n.neg), &digits, true, false)
            }
            'x' | 'X' | 'o' => {
                let n = match crate::ops::big_of(&vm.base_of(&arg)) {
                    Some(b) => b,
                    None => {
                        let tn = vm.type_name(&arg);
                        return Err(vm.type_error(format!("%{} format: an integer is required, not {}", conv, tn)));
                    }
                };
                let mut digits = n.abs().to_radix(if conv == 'o' { 8 } else { 16 });
                if conv == 'X' {
                    digits = digits.to_uppercase();
                }
                if let Some(p) = sp.prec {
                    while digits.len() < p {
                        digits.insert(0, '0');
                    }
                }
                let prefix = if sp.alt {
                    match conv {
                        'x' => "0x",
                        'X' => "0X",
                        _ => "0o",
                    }
                } else {
                    ""
                };
                pad(&sp, &format!("{}{}", sign_str(&sp, n.neg), prefix), &digits, true, false)
            }
            'e' | 'E' | 'f' | 'F' | 'g' | 'G' => {
                let x = match &arg {
                    Value::Str(_) => {
                        let tn = vm.type_name(&arg);
                        return Err(vm.type_error(format!("must be real number, not {}", tn)));
                    }
                    other => vm.to_f64(other)?,
                };
                let body = fmt_float(x.abs(), conv, Some(sp.prec.unwrap_or(6)), sp.alt);
                pad(&sp, sign_str(&sp, x.is_sign_negative() && !x.is_nan()), &body, true, false)
            }
            'c' => {
                let ch = match &arg {
                    Value::Str(s) if s.char_len() == 1 => s.s.clone(),
                    Value::Int(n) => match char::from_u32(*n as u32) {
                        Some(c) => c.to_string(),
                        None => return Err(vm.err("OverflowError", "%c arg not in range(0x110000)")),
                    },
                    _ => return Err(vm.type_error("%c requires an int or a unicode character")),
                };
                pad(&sp, "", &ch, false, false)
            }
            other => {
                return Err(vm.value_error(format!("unsupported format character '{}' (0x{:x}) at index {}", other, other as u32, i - 1)));
            }
        };
        out.push_str(&piece);
    }
    if ai < items.len() && !used_mapping && !(mapping && ai == 0) {
        return Err(vm.type_error("not all arguments converted during string formatting"));
    }
    Ok(out)
}
