// SPDX-License-Identifier: MIT
//! Methods of str, bytes and bytearray, and str.format.

use crate::builtins::*;
use crate::methods::wrap_new;
use crate::value::*;
use crate::vm::Vm;
use std::cell::RefCell;
use std::rc::Rc;

fn me(vm: &mut Vm, a: &[Value], m: &str) -> PyResult<Rc<PyStr>> {
    match a.first() {
        Some(Value::Str(s)) => Ok(s.clone()),
        Some(v) => match vm.base_of(v) {
            Value::Str(s) => Ok(s),
            _ => {
                let tn = vm.type_name(v);
                Err(vm.type_error(format!("descriptor '{}' for 'str' objects doesn't apply to a '{}' object", m, tn)))
            }
        },
        None => Err(vm.type_error(format!("unbound method str.{}() needs an argument", m))),
    }
}

fn str_arg(vm: &mut Vm, a: &[Value], i: usize, m: &str) -> PyResult<Rc<PyStr>> {
    match a.get(i) {
        Some(Value::Str(s)) => Ok(s.clone()),
        Some(v) => match vm.base_of(v) {
            Value::Str(s) => Ok(s),
            _ => {
                let tn = vm.type_name(v);
                Err(vm.type_error(format!("{}() argument {} must be str, not {}", m, i, tn)))
            }
        },
        None => Err(vm.type_error(format!("{}() missing required argument", m))),
    }
}

fn opt_str(vm: &mut Vm, a: &[Value], i: usize, m: &str) -> PyResult<Option<Rc<PyStr>>> {
    match a.get(i) {
        None | Some(Value::None) => Ok(None),
        Some(_) => Ok(Some(str_arg(vm, a, i, m)?)),
    }
}

fn s(v: String) -> Value {
    Value::string(v)
}

macro_rules! leak {
    ($t:expr, $n:expr) => {
        Box::leak(format!("{}.{}", $t, $n).into_boxed_str()) as &'static str
    };
}

pub fn install(vm: &mut Vm) {
    let c = vm.t.str_.clone();
    let fs: &[(&str, NativeFnPtr)] = &[
        ("__new__", str_new),
        ("__len__", str_len),
        ("__getitem__", str_getitem),
        ("__contains__", str_contains),
        ("__iter__", str_iter),
        ("__add__", str_add),
        ("__mul__", str_mul),
        ("__rmul__", str_mul),
        ("__mod__", str_mod),
        ("__rmod__", str_rmod),
        ("__eq__", str_eq),
        ("__ne__", str_ne),
        ("__lt__", str_lt),
        ("__le__", str_le),
        ("__gt__", str_gt),
        ("__ge__", str_ge),
        ("__hash__", str_hash),
        ("__repr__", str_repr_m),
        ("__str__", str_str),
        ("__format__", str_format_m),
        ("__getnewargs__", str_getnewargs),
        ("format", str_format),
        ("format_map", str_format_map),
        ("join", str_join),
        ("split", str_split),
        ("rsplit", str_rsplit),
        ("splitlines", str_splitlines),
        ("strip", str_strip),
        ("lstrip", str_lstrip),
        ("rstrip", str_rstrip),
        ("startswith", str_startswith),
        ("endswith", str_endswith),
        ("find", str_find),
        ("rfind", str_rfind),
        ("index", str_index),
        ("rindex", str_rindex),
        ("count", str_count),
        ("replace", str_replace),
        ("upper", str_upper),
        ("lower", str_lower),
        ("casefold", str_lower),
        ("capitalize", str_capitalize),
        ("title", str_title),
        ("swapcase", str_swapcase),
        ("center", str_center),
        ("ljust", str_ljust),
        ("rjust", str_rjust),
        ("zfill", str_zfill),
        ("expandtabs", str_expandtabs),
        ("partition", str_partition),
        ("rpartition", str_rpartition),
        ("isdigit", str_isdigit),
        ("isdecimal", str_isdecimal),
        ("isnumeric", str_isnumeric),
        ("isalpha", str_isalpha),
        ("isalnum", str_isalnum),
        ("isspace", str_isspace),
        ("isupper", str_isupper),
        ("islower", str_islower),
        ("istitle", str_istitle),
        ("isidentifier", str_isidentifier),
        ("isprintable", str_isprintable),
        ("isascii", str_isascii),
        ("removeprefix", str_removeprefix),
        ("removesuffix", str_removesuffix),
        ("encode", str_encode),
        ("translate", str_translate),
    ];
    for (n, f) in fs {
        def(&c, leak!("str", n), *f);
    }
    def_static(&c, "str.maketrans", str_maketrans);
    for (cls, tname) in [(vm.t.bytes.clone(), "bytes"), (vm.t.bytearray.clone(), "bytearray")] {
        let fs: &[(&str, NativeFnPtr)] = &[
            ("__len__", b_len),
            ("__getitem__", b_getitem),
            ("__contains__", b_contains),
            ("__iter__", b_iter),
            ("__add__", b_add),
            ("__eq__", b_eq),
            ("__repr__", b_repr),
            ("__str__", b_repr),
            ("__hash__", b_hash),
            ("decode", b_decode),
            ("hex", b_hex),
            ("join", b_join),
            ("split", b_split),
            ("strip", b_strip),
            ("lstrip", b_lstrip),
            ("rstrip", b_rstrip),
            ("startswith", b_startswith),
            ("endswith", b_endswith),
            ("find", b_find),
            ("index", b_index),
            ("count", b_count),
            ("replace", b_replace),
            ("upper", b_upper),
            ("lower", b_lower),
            ("isdigit", b_isdigit),
            ("isalpha", b_isalpha),
            ("isspace", b_isspace),
        ];
        for (n, f) in fs {
            def(&cls, leak!(tname, n), *f);
        }
        def_class(&cls, leak!(tname, "fromhex"), b_fromhex);
    }
    def(&vm.t.bytes.clone(), "bytes.__new__", bytes_new);
    let ba = vm.t.bytearray.clone();
    ba.dict.borrow_mut().set_s("__hash__", Value::None);
    def(&ba, "bytearray.__new__", bytearray_new);
    def(&ba, "bytearray.__init__", bytearray_init);
    def(&ba, "bytearray.__setitem__", ba_setitem);
    def(&ba, "bytearray.__delitem__", ba_delitem);
    def(&ba, "bytearray.__iadd__", ba_iadd);
    def(&ba, "bytearray.append", ba_append);
    def(&ba, "bytearray.extend", ba_extend);
    def(&ba, "bytearray.pop", ba_pop);
    def(&ba, "bytearray.clear", ba_clear);
    def(&ba, "bytearray.insert", ba_insert);
    def(&ba, "bytearray.reverse", ba_reverse);
    def(&ba, "bytearray.copy", ba_copy);
}

// -------------------------------------------------------------------------
// construction

fn decode_bytes(vm: &mut Vm, b: &[u8], enc: &str, errors: &str) -> PyResult<String> {
    let enc = enc.to_ascii_lowercase().replace('_', "-");
    match enc.as_str() {
        "utf-8" | "utf8" | "u8" | "utf-8-sig" => {
            let b = if enc == "utf-8-sig" && b.starts_with(&[0xef, 0xbb, 0xbf]) { &b[3..] } else { b };
            match std::str::from_utf8(b) {
                Ok(s) => Ok(s.to_string()),
                Err(e) => {
                    if errors == "replace" || errors == "ignore" {
                        let s = String::from_utf8_lossy(b).into_owned();
                        return Ok(if errors == "ignore" { s.replace('\u{fffd}', "") } else { s });
                    }
                    let pos = e.valid_up_to();
                    let byte = b[pos];
                    Err(vm.err("UnicodeDecodeError", format!("'utf-8' codec can't decode byte 0x{:02x} in position {}: {}", byte, pos, if byte >= 0x80 && byte < 0xc0 { "invalid start byte" } else { "unexpected end of data" })))
                }
            }
        }
        "ascii" | "us-ascii" => {
            let mut out = String::new();
            for (i, &c) in b.iter().enumerate() {
                if c < 128 {
                    out.push(c as char);
                } else if errors == "replace" {
                    out.push('\u{fffd}');
                } else if errors == "ignore" {
                } else {
                    return Err(vm.err("UnicodeDecodeError", format!("'ascii' codec can't decode byte 0x{:02x} in position {}: ordinal not in range(128)", c, i)));
                }
            }
            Ok(out)
        }
        "latin-1" | "latin1" | "iso-8859-1" | "iso8859-1" | "l1" | "cp1252" => Ok(b.iter().map(|&c| c as char).collect()),
        _ => Err(vm.err("LookupError", format!("unknown encoding: {}", enc))),
    }
}

pub fn encode_str(vm: &mut Vm, st: &str, enc: &str, errors: &str) -> PyResult<Vec<u8>> {
    let e = enc.to_ascii_lowercase().replace('_', "-");
    match e.as_str() {
        "utf-8" | "utf8" | "u8" => Ok(st.as_bytes().to_vec()),
        "utf-8-sig" => {
            let mut v = vec![0xef, 0xbb, 0xbf];
            v.extend_from_slice(st.as_bytes());
            Ok(v)
        }
        "ascii" | "us-ascii" | "latin-1" | "latin1" | "iso-8859-1" | "iso8859-1" | "l1" => {
            let limit = if e.contains("ascii") { 128 } else { 256 };
            let mut out = vec![];
            for (i, c) in st.chars().enumerate() {
                if (c as u32) < limit {
                    out.push(c as u32 as u8);
                } else if errors == "replace" {
                    out.push(b'?');
                } else if errors == "ignore" {
                } else {
                    let name = if limit == 128 { "ascii" } else { "latin-1" };
                    return Err(vm.err("UnicodeEncodeError", format!("'{}' codec can't encode character {} in position {}: ordinal not in range({})", name, crate::format::str_repr(&c.to_string()).replace('\'', "'"), i, limit)));
                }
            }
            Ok(out)
        }
        _ => Err(vm.err("LookupError", format!("unknown encoding: {}", enc))),
    }
}

fn str_new(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let obj = arg_or_kw(a, 1, k, "object");
    let enc = arg_or_kw(a, 2, k, "encoding");
    let errors = arg_or_kw(a, 3, k, "errors");
    let v = match obj {
        None => Value::str(""),
        Some(o) => {
            if enc.is_some() || errors.is_some() {
                let b = bytes_of(vm, &o)?;
                let e = enc.as_ref().and_then(|v| v.as_str().map(|s| s.to_string())).unwrap_or("utf-8".into());
                let er = errors.as_ref().and_then(|v| v.as_str().map(|s| s.to_string())).unwrap_or("strict".into());
                Value::string(decode_bytes(vm, &b, &e, &er)?)
            } else {
                match &o {
                    Value::Str(_) => o.clone(),
                    _ => Value::string(vm.str_of(&o)?),
                }
            }
        }
    };
    let t = vm.t.str_.clone();
    Ok(wrap_new(vm, &a[0], &t, v))
}

fn str_len(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = me(vm, a, "__len__")?;
    Ok(Value::Int(x.char_len() as i64))
}
fn str_getitem(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = me(vm, a, "__getitem__")?;
    seq_getitem(vm, &Value::Str(x), &a[1])
}
fn str_contains(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = me(vm, a, "__contains__")?;
    let y = str_arg(vm, a, 1, "__contains__")?;
    Ok(Value::Bool(x.s.contains(&*y.s)))
}
fn str_iter(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = me(vm, a, "__iter__")?;
    Ok(mk_iter(Iter::Str(x, 0)))
}
fn str_add(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = me(vm, a, "__add__")?;
    match vm.base_of(&a[1]) {
        Value::Str(y) => Ok(s(format!("{}{}", x.s, y.s))),
        _ => Ok(Value::NotImplemented),
    }
}
fn str_mul(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = me(vm, a, "__mul__")?;
    match vm.base_of(&a[1]) {
        Value::Int(n) => { let n = vm.repeat_count(x.s.len(), n)?; Ok(s(x.s.repeat(n))) }
        Value::Bool(b) => Ok(s(if b { x.s.clone() } else { String::new() })),
        _ => Ok(Value::NotImplemented),
    }
}
fn str_mod(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = me(vm, a, "__mod__")?;
    Ok(s(crate::format::percent_format(vm, &Value::Str(x), &a[1])?))
}
fn str_rmod(_vm: &mut Vm, _a: &[Value], _k: Kw) -> PyResult<Value> {
    Ok(Value::NotImplemented)
}
fn str_cmp(vm: &mut Vm, a: &[Value], f: fn(&str, &str) -> bool) -> PyResult<Value> {
    let x = me(vm, a, "compare")?;
    match vm.base_of(&a[1]) {
        Value::Str(y) => Ok(Value::Bool(f(&x.s, &y.s))),
        _ => Ok(Value::NotImplemented),
    }
}
fn str_eq(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    str_cmp(vm, a, |x, y| x == y)
}
fn str_ne(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    str_cmp(vm, a, |x, y| x != y)
}
fn str_lt(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    str_cmp(vm, a, |x, y| x < y)
}
fn str_le(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    str_cmp(vm, a, |x, y| x <= y)
}
fn str_gt(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    str_cmp(vm, a, |x, y| x > y)
}
fn str_ge(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    str_cmp(vm, a, |x, y| x >= y)
}
fn str_hash(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = me(vm, a, "__hash__")?;
    Ok(Value::Int((x.hash() >> 1) as i64))
}
fn str_repr_m(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = me(vm, a, "__repr__")?;
    Ok(s(crate::format::str_repr(&x.s)))
}
fn str_str(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = me(vm, a, "__str__")?;
    Ok(Value::Str(x))
}
fn str_format_m(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = me(vm, a, "__format__")?;
    let spec = str_arg(vm, a, 1, "__format__")?;
    if spec.s.is_empty() {
        return Ok(Value::Str(x));
    }
    Ok(s(crate::format::format_spec(vm, &Value::Str(x), &spec.s)?))
}
fn str_getnewargs(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = me(vm, a, "__getnewargs__")?;
    Ok(Value::tuple(vec![Value::Str(x)]))
}

// -------------------------------------------------------------------------
// str.format

/// the value a replacement field names: "0", "name", "a.b", "x[0]"
fn field_value(vm: &mut Vm, field: &str, args: &[Value], kw: &Value, auto: &mut usize, manual: &mut bool) -> PyResult<Value> {
    let end = field.find(['.', '[']).unwrap_or(field.len());
    let head = &field[..end];
    let mut v = if head.is_empty() {
        if *manual {
            return Err(vm.value_error("cannot switch from manual field specification to automatic field numbering"));
        }
        let i = *auto;
        *auto += 1;
        match args.get(i) {
            Some(v) => v.clone(),
            None => return Err(vm.index_error(format!("Replacement index {} out of range for positional args tuple", i))),
        }
    } else if let Ok(i) = head.parse::<usize>() {
        if *auto > 0 {
            return Err(vm.value_error("cannot switch from automatic field numbering to manual field specification"));
        }
        *manual = true;
        match args.get(i) {
            Some(v) => v.clone(),
            None => return Err(vm.index_error(format!("Replacement index {} out of range for positional args tuple", i))),
        }
    } else {
        match kw {
            Value::Dict(d) => match d.borrow().get_s(head) {
                Some(v) => v,
                None => return Err(vm.key_error(Value::str(head))),
            },
            Value::None => return Err(vm.key_error(Value::str(head))),
            m => match vm.getitem(m, &Value::str(head)) {
                Ok(v) => v,
                Err(e) => return Err(e),
            },
        }
    };
    let mut rest = &field[end..];
    while !rest.is_empty() {
        if let Some(r) = rest.strip_prefix('.') {
            let e = r.find(['.', '[']).unwrap_or(r.len());
            let name = &r[..e];
            v = vm.getattr(&v, &ps(name))?;
            rest = &r[e..];
        } else if let Some(r) = rest.strip_prefix('[') {
            let e = match r.find(']') {
                Some(e) => e,
                None => return Err(vm.value_error("Missing ']' in format string")),
            };
            let key = &r[..e];
            let kv = match key.parse::<i64>() {
                Ok(i) => Value::Int(i),
                Err(_) => Value::str(key),
            };
            v = vm.getitem(&v, &kv)?;
            rest = &r[e + 1..];
        } else {
            return Err(vm.value_error("Only '.' or '[' may follow ']' in format field specifier"));
        }
    }
    Ok(v)
}

pub fn format_str(vm: &mut Vm, fmt: &str, args: &[Value], kw: &Value) -> PyResult<String> {
    let mut auto = 0usize;
    let mut manual = false;
    format_inner(vm, fmt, args, kw, &mut auto, &mut manual, 0)
}

fn format_inner(vm: &mut Vm, fmt: &str, args: &[Value], kw: &Value, auto: &mut usize, manual: &mut bool, depth: u32) -> PyResult<String> {
    if depth > 2 {
        return Err(vm.value_error("Max string recursion exceeded"));
    }
    let c: Vec<char> = fmt.chars().collect();
    let mut out = String::with_capacity(fmt.len() + 16);
    let mut i = 0;
    while i < c.len() {
        let ch = c[i];
        if ch == '{' {
            if i + 1 < c.len() && c[i + 1] == '{' {
                out.push('{');
                i += 2;
                continue;
            }
            // the field runs to the matching brace
            let mut j = i + 1;
            let mut nest = 1;
            let mut in_bracket = false;
            while j < c.len() {
                match c[j] {
                    '[' => in_bracket = true,
                    ']' => in_bracket = false,
                    '{' if !in_bracket => nest += 1,
                    '}' if !in_bracket => {
                        nest -= 1;
                        if nest == 0 {
                            break;
                        }
                    }
                    _ => {}
                }
                j += 1;
            }
            if j >= c.len() {
                return Err(vm.value_error("Single '{' encountered in format string"));
            }
            let body: String = c[i + 1..j].iter().collect();
            i = j + 1;
            // field[!conv][:spec]
            let mut field_end = body.len();
            let mut bracket = false;
            for (k, b) in body.char_indices() {
                match b {
                    '[' => bracket = true,
                    ']' => bracket = false,
                    '!' | ':' if !bracket => {
                        field_end = k;
                        break;
                    }
                    _ => {}
                }
            }
            let field = &body[..field_end];
            let mut rest = &body[field_end..];
            let mut conv = None;
            if let Some(r) = rest.strip_prefix('!') {
                let cc = r.chars().next();
                conv = cc;
                rest = &r[cc.map(|x| x.len_utf8()).unwrap_or(0)..];
                if !rest.is_empty() && !rest.starts_with(':') {
                    return Err(vm.value_error("expected ':' after conversion specifier"));
                }
            }
            let spec = rest.strip_prefix(':').unwrap_or("");
            let spec = if spec.contains('{') { format_inner(vm, spec, args, kw, auto, manual, depth + 1)? } else { spec.to_string() };
            let v = field_value(vm, field, args, kw, auto, manual)?;
            let v = match conv {
                None => v,
                Some('r') => Value::string(vm.repr(&v)?),
                Some('s') => Value::string(vm.str_of(&v)?),
                Some('a') => Value::string(crate::format::ascii_escape(&vm.repr(&v)?)),
                Some(x) => return Err(vm.value_error(format!("Unknown conversion specifier {}", x))),
            };
            out.push_str(&vm.format_value(&v, &spec)?);
        } else if ch == '}' {
            if i + 1 < c.len() && c[i + 1] == '}' {
                out.push('}');
                i += 2;
                continue;
            }
            return Err(vm.value_error("Single '}' encountered in format string"));
        } else {
            out.push(ch);
            i += 1;
        }
    }
    Ok(out)
}

fn str_format(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let x = me(vm, a, "format")?;
    let mut d = Dict::new();
    for (key, v) in k {
        d.set_str(key.clone(), v.clone());
    }
    Ok(s(format_str(vm, &x.s, &a[1..], &Value::dict(d))?))
}

fn str_format_map(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = me(vm, a, "format_map")?;
    margs(vm, "format_map", a, 1, 1)?;
    Ok(s(format_str(vm, &x.s, &[], &a[1])?))
}

// -------------------------------------------------------------------------
// the rest of str

fn str_join(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let sep = me(vm, a, "join")?;
    margs(vm, "str.join", a, 1, 1)?;
    let items = vm.to_vec(&a[1])?;
    let mut out = String::new();
    for (i, x) in items.iter().enumerate() {
        if i > 0 {
            out.push_str(&sep.s);
        }
        match x {
            Value::Str(t) => out.push_str(&t.s),
            other => match vm.base_of(other) {
                Value::Str(t) => out.push_str(&t.s),
                _ => {
                    let tn = vm.type_name(other);
                    return Err(vm.type_error(format!("sequence item {}: expected str instance, {} found", i, tn)));
                }
            },
        }
    }
    Ok(s(out))
}

fn maxsplit_arg(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<i64> {
    match arg_or_kw(a, 2, k, "maxsplit") {
        Some(v) => vm.to_index(&v),
        None => Ok(-1),
    }
}

fn sep_arg(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Option<Rc<PyStr>>> {
    match arg_or_kw(a, 1, k, "sep") {
        None | Some(Value::None) => Ok(None),
        Some(v) => {
            let p = as_pystr(vm, &v, "sep")?;
            if p.s.is_empty() {
                return Err(vm.value_error("empty separator"));
            }
            Ok(Some(p))
        }
    }
}

fn str_split(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let x = me(vm, a, "split")?;
    let sep = sep_arg(vm, a, k)?;
    let max = maxsplit_arg(vm, a, k)?;
    let parts: Vec<Value> = match sep {
        Some(sp) => {
            if max < 0 {
                x.s.split(&*sp.s).map(Value::str).collect()
            } else {
                x.s.splitn(max as usize + 1, &*sp.s).map(Value::str).collect()
            }
        }
        None => {
            let mut out = vec![];
            let mut rest = x.s.trim_start();
            let mut n = 0;
            while !rest.is_empty() {
                if max >= 0 && n >= max {
                    out.push(Value::str(rest));
                    break;
                }
                let e = rest.find(char::is_whitespace).unwrap_or(rest.len());
                out.push(Value::str(&rest[..e]));
                rest = rest[e..].trim_start();
                n += 1;
            }
            out
        }
    };
    Ok(Value::list(parts))
}

fn str_rsplit(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let x = me(vm, a, "rsplit")?;
    let sep = sep_arg(vm, a, k)?;
    let max = maxsplit_arg(vm, a, k)?;
    let mut parts: Vec<Value> = match sep {
        Some(sp) => {
            if max < 0 {
                x.s.rsplit(&*sp.s).map(Value::str).collect()
            } else {
                x.s.rsplitn(max as usize + 1, &*sp.s).map(Value::str).collect()
            }
        }
        None => {
            let mut out = vec![];
            let mut rest = x.s.trim_end();
            let mut n = 0;
            while !rest.is_empty() {
                if max >= 0 && n >= max {
                    out.push(Value::str(rest));
                    break;
                }
                let e = rest.rfind(char::is_whitespace).map(|i| i + rest[i..].chars().next().unwrap().len_utf8()).unwrap_or(0);
                out.push(Value::str(&rest[e..]));
                rest = rest[..e].trim_end();
                n += 1;
            }
            out
        }
    };
    parts.reverse();
    Ok(Value::list(parts))
}

fn str_splitlines(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let x = me(vm, a, "splitlines")?;
    let keep = match arg_or_kw(a, 1, k, "keepends") {
        Some(v) => vm.truthy(&v)?,
        None => false,
    };
    let mut out = vec![];
    let cs: Vec<char> = x.s.chars().collect();
    let mut start = 0;
    let mut i = 0;
    while i < cs.len() {
        let c = cs[i];
        let is_break = matches!(c, '\n' | '\r' | '\x0b' | '\x0c' | '\x1c' | '\x1d' | '\x1e' | '\u{85}' | '\u{2028}' | '\u{2029}');
        if is_break {
            let mut end = i + 1;
            if c == '\r' && i + 1 < cs.len() && cs[i + 1] == '\n' {
                end = i + 2;
            }
            let line: String = if keep { cs[start..end].iter().collect() } else { cs[start..i].iter().collect() };
            out.push(Value::string(line));
            start = end;
            i = end;
        } else {
            i += 1;
        }
    }
    if start < cs.len() {
        out.push(Value::string(cs[start..].iter().collect()));
    }
    Ok(Value::list(out))
}

fn strip_set(vm: &mut Vm, a: &[Value]) -> PyResult<Option<Vec<char>>> {
    Ok(opt_str(vm, a, 1, "strip")?.map(|c| c.s.chars().collect()))
}

fn str_strip(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = me(vm, a, "strip")?;
    Ok(match strip_set(vm, a)? {
        Some(cs) => Value::str(x.s.trim_matches(|c| cs.contains(&c))),
        None => Value::str(x.s.trim()),
    })
}
fn str_lstrip(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = me(vm, a, "lstrip")?;
    Ok(match strip_set(vm, a)? {
        Some(cs) => Value::str(x.s.trim_start_matches(|c| cs.contains(&c))),
        None => Value::str(x.s.trim_start()),
    })
}
fn str_rstrip(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = me(vm, a, "rstrip")?;
    Ok(match strip_set(vm, a)? {
        Some(cs) => Value::str(x.s.trim_end_matches(|c| cs.contains(&c))),
        None => Value::str(x.s.trim_end()),
    })
}

/// the char range [start, end) of optional start/end arguments, as byte offsets
fn sub_range(vm: &mut Vm, x: &PyStr, a: &[Value], i: usize) -> PyResult<(usize, usize, usize, usize)> {
    let n = x.char_len() as i64;
    let norm = |v: i64| -> i64 { if v < 0 { (v + n).max(0) } else { v.min(n) } };
    let st = match a.get(i) {
        Some(Value::None) | None => 0,
        Some(v) => norm(vm.to_index(v)?),
    };
    let en = match a.get(i + 1) {
        Some(Value::None) | None => n,
        Some(v) => norm(vm.to_index(v)?),
    };
    let to_byte = |ci: i64| -> usize {
        if x.ascii {
            ci as usize
        } else {
            x.s.char_indices().nth(ci as usize).map(|(b, _)| b).unwrap_or(x.s.len())
        }
    };
    Ok((to_byte(st), to_byte(en.max(st)), st as usize, en as usize))
}

fn byte_to_char(x: &PyStr, b: usize) -> usize {
    if x.ascii {
        b
    } else {
        x.s[..b].chars().count()
    }
}

fn affix_test(vm: &mut Vm, a: &[Value], m: &str, ends: bool) -> PyResult<Value> {
    let x = me(vm, a, m)?;
    margs(vm, m, a, 1, 3)?;
    let (bs, be, cs, ce) = sub_range(vm, &x, a, 2)?;
    let hay = if cs > ce { "" } else { &x.s[bs..be] };
    let test = |p: &str| if ends { hay.ends_with(p) } else { hay.starts_with(p) };
    if cs > x.char_len() {
        return Ok(Value::Bool(false));
    }
    match &a[1] {
        Value::Tuple(t) => {
            for p in t.iter() {
                let p = as_pystr(vm, p, m)?;
                if test(&p.s) {
                    return Ok(Value::Bool(true));
                }
            }
            Ok(Value::Bool(false))
        }
        v => match vm.base_of(v) {
            Value::Str(p) => Ok(Value::Bool(test(&p.s))),
            _ => {
                let tn = vm.type_name(v);
                Err(vm.type_error(format!("{} first arg must be str or a tuple of str, not {}", m, tn)))
            }
        },
    }
}

fn str_startswith(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    affix_test(vm, a, "startswith", false)
}
fn str_endswith(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    affix_test(vm, a, "endswith", true)
}

fn find_impl(vm: &mut Vm, a: &[Value], m: &str, rev: bool) -> PyResult<i64> {
    let x = me(vm, a, m)?;
    margs(vm, m, a, 1, 3)?;
    let sub = str_arg(vm, a, 1, m)?;
    let (bs, be, cs, ce) = sub_range(vm, &x, a, 2)?;
    if cs > ce || cs > x.char_len() {
        return Ok(-1);
    }
    let hay = &x.s[bs..be];
    let r = if rev { hay.rfind(&*sub.s) } else { hay.find(&*sub.s) };
    Ok(match r {
        Some(p) => byte_to_char(&x, bs + p) as i64,
        None => -1,
    })
}

fn str_find(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    Ok(Value::Int(find_impl(vm, a, "find", false)?))
}
fn str_rfind(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    Ok(Value::Int(find_impl(vm, a, "rfind", true)?))
}
fn str_index(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let r = find_impl(vm, a, "index", false)?;
    if r < 0 {
        return Err(vm.value_error("substring not found"));
    }
    Ok(Value::Int(r))
}
fn str_rindex(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let r = find_impl(vm, a, "rindex", true)?;
    if r < 0 {
        return Err(vm.value_error("substring not found"));
    }
    Ok(Value::Int(r))
}

fn str_count(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = me(vm, a, "count")?;
    margs(vm, "count", a, 1, 3)?;
    let sub = str_arg(vm, a, 1, "count")?;
    let (bs, be, cs, ce) = sub_range(vm, &x, a, 2)?;
    if cs > ce || cs > x.char_len() {
        return Ok(Value::Int(0));
    }
    let hay = &x.s[bs..be];
    if sub.s.is_empty() {
        return Ok(Value::Int(hay.chars().count() as i64 + 1));
    }
    Ok(Value::Int(hay.matches(&*sub.s).count() as i64))
}

fn str_replace(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let x = me(vm, a, "replace")?;
    let old = str_arg(vm, a, 1, "replace")?;
    let new = str_arg(vm, a, 2, "replace")?;
    let count = match arg_or_kw(a, 3, k, "count") {
        Some(v) => vm.to_index(&v)?,
        None => -1,
    };
    if count < 0 {
        if old.s.is_empty() {
            // between every character
            let mut out = String::new();
            out.push_str(&new.s);
            for c in x.s.chars() {
                out.push(c);
                out.push_str(&new.s);
            }
            return Ok(s(out));
        }
        return Ok(s(x.s.replace(&*old.s, &new.s)));
    }
    if old.s.is_empty() {
        let mut out = String::new();
        let mut n = 0;
        for c in x.s.chars() {
            if n < count {
                out.push_str(&new.s);
                n += 1;
            }
            out.push(c);
        }
        if n < count {
            out.push_str(&new.s);
        }
        return Ok(s(out));
    }
    Ok(s(x.s.replacen(&*old.s, &new.s, count as usize)))
}

fn str_upper(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = me(vm, a, "upper")?;
    Ok(s(x.s.to_uppercase()))
}
fn str_lower(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = me(vm, a, "lower")?;
    Ok(s(x.s.to_lowercase()))
}
fn str_capitalize(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = me(vm, a, "capitalize")?;
    let mut cs = x.s.chars();
    Ok(s(match cs.next() {
        Some(f) => {
            let mut out: String = f.to_uppercase().collect();
            out.push_str(&cs.as_str().to_lowercase());
            out
        }
        None => String::new(),
    }))
}
fn str_title(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = me(vm, a, "title")?;
    let mut out = String::new();
    let mut prev_cased = false;
    for c in x.s.chars() {
        if prev_cased {
            out.extend(c.to_lowercase());
        } else {
            out.extend(c.to_uppercase());
        }
        prev_cased = c.is_alphabetic();
    }
    Ok(s(out))
}
fn str_swapcase(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = me(vm, a, "swapcase")?;
    let mut out = String::new();
    for c in x.s.chars() {
        if c.is_uppercase() {
            out.extend(c.to_lowercase());
        } else {
            out.extend(c.to_uppercase());
        }
    }
    Ok(s(out))
}

fn fill_arg(vm: &mut Vm, a: &[Value], m: &str) -> PyResult<(usize, char)> {
    margs(vm, m, a, 1, 2)?;
    let w = vm.to_index(&a[1])?.max(0) as usize;
    let fill = match a.get(2) {
        Some(v) => {
            let f = as_pystr(vm, v, m)?;
            let mut cs = f.s.chars();
            match (cs.next(), cs.next()) {
                (Some(c), None) => c,
                _ => return Err(vm.type_error("The fill character must be exactly one character long")),
            }
        }
        None => ' ',
    };
    Ok((w, fill))
}

fn str_center(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = me(vm, a, "center")?;
    let (w, f) = fill_arg(vm, a, "center")?;
    let n = x.char_len();
    if w <= n {
        return Ok(Value::Str(x));
    }
    let total = w - n;
    let left = total / 2 + (total & w & 1);
    let right = total - left;
    let fs = |k: usize| std::iter::repeat(f).take(k).collect::<String>();
    Ok(s(format!("{}{}{}", fs(left), x.s, fs(right))))
}
fn str_ljust(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = me(vm, a, "ljust")?;
    let (w, f) = fill_arg(vm, a, "ljust")?;
    let n = x.char_len();
    if w <= n {
        return Ok(Value::Str(x));
    }
    Ok(s(format!("{}{}", x.s, std::iter::repeat(f).take(w - n).collect::<String>())))
}
fn str_rjust(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = me(vm, a, "rjust")?;
    let (w, f) = fill_arg(vm, a, "rjust")?;
    let n = x.char_len();
    if w <= n {
        return Ok(Value::Str(x));
    }
    Ok(s(format!("{}{}", std::iter::repeat(f).take(w - n).collect::<String>(), x.s)))
}
fn str_zfill(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = me(vm, a, "zfill")?;
    margs(vm, "zfill", a, 1, 1)?;
    let w = vm.to_index(&a[1])?.max(0) as usize;
    let n = x.char_len();
    if w <= n {
        return Ok(Value::Str(x));
    }
    let zeros = "0".repeat(w - n);
    Ok(s(if x.s.starts_with(['+', '-']) { format!("{}{}{}", &x.s[..1], zeros, &x.s[1..]) } else { format!("{}{}", zeros, x.s) }))
}
fn str_expandtabs(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let x = me(vm, a, "expandtabs")?;
    let ts = match arg_or_kw(a, 1, k, "tabsize") {
        Some(v) => vm.to_index(&v)?,
        None => 8,
    };
    let mut out = String::new();
    let mut col = 0i64;
    for c in x.s.chars() {
        match c {
            '\t' => {
                if ts > 0 {
                    let n = ts - (col % ts);
                    out.push_str(&" ".repeat(n as usize));
                    col += n;
                }
            }
            '\n' | '\r' => {
                out.push(c);
                col = 0;
            }
            _ => {
                out.push(c);
                col += 1;
            }
        }
    }
    Ok(s(out))
}

fn str_partition(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = me(vm, a, "partition")?;
    let sep = str_arg(vm, a, 1, "partition")?;
    if sep.s.is_empty() {
        return Err(vm.value_error("empty separator"));
    }
    Ok(match x.s.find(&*sep.s) {
        Some(i) => Value::tuple(vec![Value::str(&x.s[..i]), Value::Str(sep.clone()), Value::str(&x.s[i + sep.s.len()..])]),
        None => Value::tuple(vec![Value::Str(x.clone()), Value::str(""), Value::str("")]),
    })
}
fn str_rpartition(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = me(vm, a, "rpartition")?;
    let sep = str_arg(vm, a, 1, "rpartition")?;
    if sep.s.is_empty() {
        return Err(vm.value_error("empty separator"));
    }
    Ok(match x.s.rfind(&*sep.s) {
        Some(i) => Value::tuple(vec![Value::str(&x.s[..i]), Value::Str(sep.clone()), Value::str(&x.s[i + sep.s.len()..])]),
        None => Value::tuple(vec![Value::str(""), Value::str(""), Value::Str(x.clone())]),
    })
}

fn char_test(vm: &mut Vm, a: &[Value], m: &str, f: fn(char) -> bool) -> PyResult<Value> {
    let x = me(vm, a, m)?;
    Ok(Value::Bool(!x.s.is_empty() && x.s.chars().all(f)))
}

fn is_decimal(c: char) -> bool {
    c.is_ascii_digit() || (c.is_numeric() && c.to_digit(10).is_some()) || ('\u{660}'..='\u{669}').contains(&c) || ('\u{ff10}'..='\u{ff19}').contains(&c) || ('\u{966}'..='\u{96f}').contains(&c)
}

fn str_isdigit(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    char_test(vm, a, "isdigit", |c| is_decimal(c) || matches!(c, '²' | '³' | '¹' | '\u{2070}'..='\u{2079}' | '\u{2080}'..='\u{2089}' | '\u{2460}'..='\u{2468}'))
}
fn str_isdecimal(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    char_test(vm, a, "isdecimal", is_decimal)
}
fn str_isnumeric(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    char_test(vm, a, "isnumeric", |c| c.is_numeric())
}
fn str_isalpha(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    char_test(vm, a, "isalpha", |c| c.is_alphabetic())
}
fn str_isalnum(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    char_test(vm, a, "isalnum", |c| c.is_alphanumeric())
}
fn str_isspace(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    char_test(vm, a, "isspace", |c| c.is_whitespace() || matches!(c, '\x1c'..='\x1f'))
}
fn str_isupper(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = me(vm, a, "isupper")?;
    Ok(Value::Bool(x.s.chars().any(|c| c.is_uppercase()) && !x.s.chars().any(|c| c.is_lowercase())))
}
fn str_islower(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = me(vm, a, "islower")?;
    Ok(Value::Bool(x.s.chars().any(|c| c.is_lowercase()) && !x.s.chars().any(|c| c.is_uppercase())))
}
fn str_istitle(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = me(vm, a, "istitle")?;
    let mut prev_cased = false;
    let mut any = false;
    for c in x.s.chars() {
        if c.is_uppercase() {
            if prev_cased {
                return Ok(Value::Bool(false));
            }
            prev_cased = true;
            any = true;
        } else if c.is_lowercase() {
            if !prev_cased {
                return Ok(Value::Bool(false));
            }
            prev_cased = true;
        } else {
            prev_cased = false;
        }
    }
    Ok(Value::Bool(any))
}
fn str_isidentifier(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = me(vm, a, "isidentifier")?;
    let mut cs = x.s.chars();
    Ok(Value::Bool(match cs.next() {
        Some(f) => (f == '_' || f.is_alphabetic()) && cs.all(|c| c == '_' || c.is_alphanumeric()),
        None => false,
    }))
}
fn str_isprintable(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = me(vm, a, "isprintable")?;
    Ok(Value::Bool(x.s.chars().all(|c| c == ' ' || (!c.is_control() && !c.is_whitespace()))))
}
fn str_isascii(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = me(vm, a, "isascii")?;
    Ok(Value::Bool(x.ascii))
}
fn str_removeprefix(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = me(vm, a, "removeprefix")?;
    let p = str_arg(vm, a, 1, "removeprefix")?;
    Ok(match x.s.strip_prefix(&*p.s) {
        Some(r) => Value::str(r),
        None => Value::Str(x),
    })
}
fn str_removesuffix(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = me(vm, a, "removesuffix")?;
    let p = str_arg(vm, a, 1, "removesuffix")?;
    Ok(match x.s.strip_suffix(&*p.s) {
        Some(r) if !p.s.is_empty() => Value::str(r),
        _ => Value::Str(x),
    })
}
fn str_encode(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let x = me(vm, a, "encode")?;
    let enc = arg_or_kw(a, 1, k, "encoding").and_then(|v| v.as_str().map(|s| s.to_string())).unwrap_or("utf-8".into());
    let errors = arg_or_kw(a, 2, k, "errors").and_then(|v| v.as_str().map(|s| s.to_string())).unwrap_or("strict".into());
    Ok(Value::Bytes(Rc::new(encode_str(vm, &x.s, &enc, &errors)?)))
}

fn str_maketrans(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let d = new_dict();
    match a.len() {
        1 => {
            let items = vm.mapping_items(&a[0])?;
            for (key, v) in items {
                let kk = match &key {
                    Value::Str(s) if s.char_len() == 1 => Value::Int(s.s.chars().next().unwrap() as i64),
                    Value::Int(_) => key.clone(),
                    _ => return Err(vm.value_error("string keys in translate table must be of length 1")),
                };
                vm.dict_set(&d, kk, v)?;
            }
        }
        2 | 3 => {
            let x = as_pystr(vm, &a[0], "maketrans")?;
            let y = as_pystr(vm, &a[1], "maketrans")?;
            if x.char_len() != y.char_len() {
                return Err(vm.value_error("the first two maketrans arguments must have equal length"));
            }
            for (p, q) in x.s.chars().zip(y.s.chars()) {
                vm.dict_set(&d, Value::Int(p as i64), Value::Int(q as i64))?;
            }
            if let Some(z) = a.get(2) {
                let z = as_pystr(vm, z, "maketrans")?;
                for c in z.s.chars() {
                    vm.dict_set(&d, Value::Int(c as i64), Value::None)?;
                }
            }
        }
        _ => return Err(vm.type_error("maketrans() takes 1 to 3 arguments")),
    }
    Ok(Value::Dict(d))
}

fn str_translate(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = me(vm, a, "translate")?;
    margs(vm, "translate", a, 1, 1)?;
    let mut out = String::new();
    for c in x.s.chars() {
        let r = match vm.getitem(&a[1], &Value::Int(c as i64)) {
            Ok(v) => Some(v),
            Err(e) if vm.exc_is(&e, "LookupError") => None,
            Err(e) => return Err(e),
        };
        match r {
            None => out.push(c),
            Some(Value::None) => {}
            Some(Value::Int(i)) => out.push(char::from_u32(i as u32).unwrap_or('\u{fffd}')),
            Some(Value::Str(t)) => out.push_str(&t.s),
            Some(_) => return Err(vm.type_error("character mapping must return integer, None or str")),
        }
    }
    Ok(s(out))
}

// -------------------------------------------------------------------------
// bytes and bytearray

fn bme(vm: &mut Vm, a: &[Value]) -> PyResult<Vec<u8>> {
    let v = vm.base_of(&a[0]);
    bytes_of(vm, &v)
}

fn same_type(a: &Value, v: Vec<u8>) -> Value {
    match a {
        Value::ByteArray(_) => Value::ByteArray(Rc::new(RefCell::new(v))),
        _ => Value::Bytes(Rc::new(v)),
    }
}

fn bytes_from(vm: &mut Vm, src: Option<&Value>, enc: Option<Value>, errors: Option<Value>) -> PyResult<Vec<u8>> {
    Ok(match src {
        None => vec![],
        Some(Value::Str(st)) => {
            let e = match enc {
                Some(Value::Str(e)) => e.s.clone(),
                _ => return Err(vm.type_error("string argument without an encoding")),
            };
            let er = errors.and_then(|v| v.as_str().map(|s| s.to_string())).unwrap_or("strict".into());
            encode_str(vm, &st.s, &e, &er)?
        }
        Some(Value::Int(n)) => {
            if *n < 0 {
                return Err(vm.value_error("negative count"));
            }
            vec![0; *n as usize]
        }
        Some(v @ (Value::Bytes(_) | Value::ByteArray(_))) => bytes_of(vm, v)?,
        Some(v) => {
            if let Some(f) = vm.lookup_special(v, &ps("__bytes__")) {
                let r = vm.call_vec(&f, vec![], &[])?;
                return bytes_of(vm, &r);
            }
            let items = vm.to_vec(v)?;
            let mut out = Vec::with_capacity(items.len());
            for x in items {
                let i = vm.to_index(&x)?;
                if !(0..256).contains(&i) {
                    return Err(vm.value_error("bytes must be in range(0, 256)"));
                }
                out.push(i as u8);
            }
            out
        }
    })
}

fn bytes_new(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let src = arg_or_kw(a, 1, k, "source");
    let v = bytes_from(vm, src.as_ref(), arg_or_kw(a, 2, k, "encoding"), arg_or_kw(a, 3, k, "errors"))?;
    let t = vm.t.bytes.clone();
    Ok(wrap_new(vm, &a[0], &t, Value::Bytes(Rc::new(v))))
}

fn bytearray_new(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let t = vm.t.bytearray.clone();
    Ok(wrap_new(vm, &a[0], &t, Value::ByteArray(Rc::new(RefCell::new(vec![])))))
}

fn bytearray_init(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let src = arg_or_kw(a, 1, k, "source");
    let v = bytes_from(vm, src.as_ref(), arg_or_kw(a, 2, k, "encoding"), arg_or_kw(a, 3, k, "errors"))?;
    if let Value::ByteArray(b) = vm.base_of(&a[0]) {
        *b.borrow_mut() = v;
    }
    Ok(Value::None)
}

fn b_len(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    Ok(Value::Int(bme(vm, a)?.len() as i64))
}
fn b_getitem(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = vm.base_of(&a[0]);
    seq_getitem(vm, &x, &a[1])
}
fn b_contains(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = vm.base_of(&a[0]);
    Ok(Value::Bool(vm.contains(&x, &a[1])?))
}
fn b_iter(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = vm.base_of(&a[0]);
    vm.iter(&x)
}
fn b_add(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let mut x = bme(vm, a)?;
    let y = bytes_of(vm, &a[1])?;
    x.extend(y);
    Ok(same_type(&vm.base_of(&a[0]), x))
}
fn b_eq(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = bme(vm, a)?;
    match bytes_of(vm, &a[1]) {
        Ok(y) => Ok(Value::Bool(x == y)),
        Err(_) => Ok(Value::NotImplemented),
    }
}
fn b_repr(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = vm.base_of(&a[0]);
    Ok(s(vm.repr(&x)?))
}
fn b_hash(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = vm.base_of(&a[0]);
    Ok(Value::Int((vm.hash(&x)? >> 1) as i64))
}
fn b_decode(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let x = bme(vm, a)?;
    let enc = arg_or_kw(a, 1, k, "encoding").and_then(|v| v.as_str().map(|s| s.to_string())).unwrap_or("utf-8".into());
    let errors = arg_or_kw(a, 2, k, "errors").and_then(|v| v.as_str().map(|s| s.to_string())).unwrap_or("strict".into());
    Ok(s(decode_bytes(vm, &x, &enc, &errors)?))
}
fn b_hex(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let x = bme(vm, a)?;
    let sep = arg_or_kw(a, 1, k, "sep").and_then(|v| v.as_str().map(|s| s.to_string()));
    let parts: Vec<String> = x.iter().map(|b| format!("{:02x}", b)).collect();
    Ok(s(parts.join(sep.as_deref().unwrap_or(""))))
}
fn b_fromhex(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let st = as_pystr(vm, &a[1], "fromhex")?;
    let digits: Vec<char> = st.s.chars().filter(|c| !c.is_whitespace()).collect();
    if digits.len() % 2 != 0 {
        return Err(vm.value_error("non-hexadecimal number found in fromhex() arg"));
    }
    let mut out = vec![];
    for p in digits.chunks(2) {
        let h: String = p.iter().collect();
        match u8::from_str_radix(&h, 16) {
            Ok(b) => out.push(b),
            Err(_) => return Err(vm.value_error("non-hexadecimal number found in fromhex() arg")),
        }
    }
    let is_ba = matches!(&a[0], Value::Class(c) if Rc::ptr_eq(c, &vm.t.bytearray));
    Ok(if is_ba { Value::ByteArray(Rc::new(RefCell::new(out))) } else { Value::Bytes(Rc::new(out)) })
}
fn b_join(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let sep = bme(vm, a)?;
    let items = vm.to_vec(&a[1])?;
    let mut out = vec![];
    for (i, x) in items.iter().enumerate() {
        if i > 0 {
            out.extend_from_slice(&sep);
        }
        out.extend(bytes_of(vm, x)?);
    }
    Ok(same_type(&vm.base_of(&a[0]), out))
}
fn find_sub(h: &[u8], n: &[u8]) -> Option<usize> {
    if n.is_empty() {
        return Some(0);
    }
    h.windows(n.len()).position(|w| w == n)
}
fn b_split(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let x = bme(vm, a)?;
    let me_v = vm.base_of(&a[0]);
    let sep = arg_or_kw(a, 1, k, "sep").filter(|v| !v.is_none());
    let max = match arg_or_kw(a, 2, k, "maxsplit") {
        Some(v) => vm.to_index(&v)?,
        None => -1,
    };
    let mut out = vec![];
    match sep {
        Some(sv) => {
            let sp = bytes_of(vm, &sv)?;
            if sp.is_empty() {
                return Err(vm.value_error("empty separator"));
            }
            let mut rest = &x[..];
            let mut n = 0;
            while let Some(p) = find_sub(rest, &sp) {
                if max >= 0 && n >= max {
                    break;
                }
                out.push(same_type(&me_v, rest[..p].to_vec()));
                rest = &rest[p + sp.len()..];
                n += 1;
            }
            out.push(same_type(&me_v, rest.to_vec()));
        }
        None => {
            for part in x.split(|c| c.is_ascii_whitespace()).filter(|p| !p.is_empty()) {
                out.push(same_type(&me_v, part.to_vec()));
            }
        }
    }
    Ok(Value::list(out))
}
fn b_strip_impl(vm: &mut Vm, a: &[Value], left: bool, right: bool) -> PyResult<Value> {
    let x = bme(vm, a)?;
    let chars = match a.get(1) {
        Some(Value::None) | None => b" \t\n\r\x0b\x0c".to_vec(),
        Some(v) => bytes_of(vm, v)?,
    };
    let mut st = 0;
    let mut en = x.len();
    if left {
        while st < en && chars.contains(&x[st]) {
            st += 1;
        }
    }
    if right {
        while en > st && chars.contains(&x[en - 1]) {
            en -= 1;
        }
    }
    Ok(same_type(&vm.base_of(&a[0]), x[st..en].to_vec()))
}
fn b_strip(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    b_strip_impl(vm, a, true, true)
}
fn b_lstrip(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    b_strip_impl(vm, a, true, false)
}
fn b_rstrip(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    b_strip_impl(vm, a, false, true)
}
fn b_affix(vm: &mut Vm, a: &[Value], ends: bool) -> PyResult<Value> {
    let x = bme(vm, a)?;
    let pats: Vec<Value> = match &a[1] {
        Value::Tuple(t) => t.to_vec(),
        v => vec![v.clone()],
    };
    for p in pats {
        let p = bytes_of(vm, &p)?;
        if (ends && x.ends_with(&p)) || (!ends && x.starts_with(&p)) {
            return Ok(Value::Bool(true));
        }
    }
    Ok(Value::Bool(false))
}
fn b_startswith(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    b_affix(vm, a, false)
}
fn b_endswith(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    b_affix(vm, a, true)
}
fn b_needle(vm: &mut Vm, v: &Value) -> PyResult<Vec<u8>> {
    match v {
        Value::Int(i) => Ok(vec![*i as u8]),
        o => bytes_of(vm, o),
    }
}
fn b_find(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = bme(vm, a)?;
    let n = b_needle(vm, &a[1])?;
    Ok(Value::Int(find_sub(&x, &n).map(|p| p as i64).unwrap_or(-1)))
}
fn b_index(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = bme(vm, a)?;
    let n = b_needle(vm, &a[1])?;
    match find_sub(&x, &n) {
        Some(p) => Ok(Value::Int(p as i64)),
        None => Err(vm.value_error("subsection not found")),
    }
}
fn b_count(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = bme(vm, a)?;
    let n = b_needle(vm, &a[1])?;
    if n.is_empty() {
        return Ok(Value::Int(x.len() as i64 + 1));
    }
    let mut c = 0;
    let mut i = 0;
    while i + n.len() <= x.len() {
        if x[i..i + n.len()] == n[..] {
            c += 1;
            i += n.len();
        } else {
            i += 1;
        }
    }
    Ok(Value::Int(c))
}
fn b_replace(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = bme(vm, a)?;
    let old = bytes_of(vm, &a[1])?;
    let new = bytes_of(vm, &a[2])?;
    let mut out = vec![];
    let mut i = 0;
    if old.is_empty() {
        out.extend_from_slice(&x);
    } else {
        while i < x.len() {
            if x[i..].starts_with(&old) {
                out.extend_from_slice(&new);
                i += old.len();
            } else {
                out.push(x[i]);
                i += 1;
            }
        }
    }
    Ok(same_type(&vm.base_of(&a[0]), out))
}
fn b_upper(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = bme(vm, a)?;
    Ok(same_type(&vm.base_of(&a[0]), x.to_ascii_uppercase()))
}
fn b_lower(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = bme(vm, a)?;
    Ok(same_type(&vm.base_of(&a[0]), x.to_ascii_lowercase()))
}
fn b_isdigit(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = bme(vm, a)?;
    Ok(Value::Bool(!x.is_empty() && x.iter().all(|c| c.is_ascii_digit())))
}
fn b_isalpha(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = bme(vm, a)?;
    Ok(Value::Bool(!x.is_empty() && x.iter().all(|c| c.is_ascii_alphabetic())))
}
fn b_isspace(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = bme(vm, a)?;
    Ok(Value::Bool(!x.is_empty() && x.iter().all(|c| c.is_ascii_whitespace())))
}

fn ba_of(vm: &mut Vm, a: &[Value]) -> PyResult<Rc<RefCell<Vec<u8>>>> {
    match vm.base_of(&a[0]) {
        Value::ByteArray(b) => Ok(b),
        _ => Err(vm.type_error("descriptor requires a 'bytearray' object")),
    }
}
fn ba_setitem(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = vm.base_of(&a[0]);
    seq_setitem(vm, &x, &a[1], a[2].clone())?;
    Ok(Value::None)
}
fn ba_delitem(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = vm.base_of(&a[0]);
    seq_delitem(vm, &x, &a[1])?;
    Ok(Value::None)
}
fn ba_iadd(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let b = ba_of(vm, a)?;
    let y = bytes_of(vm, &a[1])?;
    b.borrow_mut().extend(y);
    Ok(a[0].clone())
}
fn ba_append(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let b = ba_of(vm, a)?;
    let i = vm.to_index(&a[1])?;
    if !(0..256).contains(&i) {
        return Err(vm.value_error("byte must be in range(0, 256)"));
    }
    b.borrow_mut().push(i as u8);
    Ok(Value::None)
}
fn ba_extend(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let b = ba_of(vm, a)?;
    let y = bytes_from(vm, Some(&a[1]), None, None)?;
    b.borrow_mut().extend(y);
    Ok(Value::None)
}
fn ba_pop(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let b = ba_of(vm, a)?;
    let n = b.borrow().len();
    if n == 0 {
        return Err(vm.index_error("pop from empty bytearray"));
    }
    let i = match a.get(1) {
        Some(v) => {
            let i = vm.to_index(v)?;
            (if i < 0 { i + n as i64 } else { i }) as usize
        }
        None => n - 1,
    };
    if i >= n {
        return Err(vm.index_error("pop index out of range"));
    }
    let v = b.borrow_mut().remove(i);
    Ok(Value::Int(v as i64))
}
fn ba_clear(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let b = ba_of(vm, a)?;
    b.borrow_mut().clear();
    Ok(Value::None)
}
fn ba_insert(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let b = ba_of(vm, a)?;
    let n = b.borrow().len() as i64;
    let i = vm.to_index(&a[1])?;
    let i = if i < 0 { (i + n).max(0) } else { i.min(n) } as usize;
    let v = vm.to_index(&a[2])? as u8;
    b.borrow_mut().insert(i, v);
    Ok(Value::None)
}
fn ba_reverse(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let b = ba_of(vm, a)?;
    b.borrow_mut().reverse();
    Ok(Value::None)
}
fn ba_copy(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let b = ba_of(vm, a)?;
    let v = b.borrow().clone();
    Ok(Value::ByteArray(Rc::new(RefCell::new(v))))
}
