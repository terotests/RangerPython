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

/// the positional arguments a str or bytes method takes besides self
fn str_arity(n: &str) -> Option<(u8, u8)> {
    Some(match n {
        "__len__" | "__iter__" | "__hash__" | "__repr__" | "__str__" | "__getnewargs__" | "upper" | "lower" | "casefold" | "capitalize" | "title" | "swapcase" | "isdigit" | "isdecimal" | "isnumeric" | "isalpha" | "isalnum" | "isspace" | "isupper" | "islower" | "istitle" | "isidentifier" | "isprintable" | "isascii" => (0, 0),
        "__getitem__" | "__contains__" | "__add__" | "__mul__" | "__rmul__" | "__mod__" | "__rmod__" | "__eq__" | "__ne__" | "__lt__" | "__le__" | "__gt__" | "__ge__" | "__format__" | "format_map" | "join" | "zfill" | "partition" | "rpartition" | "removeprefix" | "removesuffix" | "translate" => (1, 1),
        "split" | "rsplit" | "encode" => (0, 2),
        "splitlines" | "strip" | "lstrip" | "rstrip" | "expandtabs" => (0, 1),
        "startswith" | "endswith" | "find" | "rfind" | "index" | "rindex" | "count" => (1, 3),
        "replace" => (2, 3),
        "center" | "ljust" | "rjust" => (1, 2),
        _ => return None,
    })
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
        ("casefold", str_casefold),
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
        match str_arity(n) {
            Some((lo, hi)) => def_arity(&c, leak!("str", n), *f, lo, hi),
            None => def(&c, leak!("str", n), *f),
        }
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
            ("startswith", bs_startswith),
            ("endswith", bs_endswith),
            ("find", bs_find),
            ("rfind", bs_rfind),
            ("index", bs_index),
            ("rindex", bs_rindex),
            ("count", bs_count),
            ("replace", bs_replace),
            ("partition", bs_partition),
            ("rpartition", bs_rpartition),
            ("center", bs_center),
            ("ljust", bs_ljust),
            ("rjust", bs_rjust),
            ("zfill", bs_zfill),
            ("expandtabs", bs_expandtabs),
            ("removeprefix", bs_removeprefix),
            ("removesuffix", bs_removesuffix),
            ("splitlines", bs_splitlines),
            ("rsplit", bs_rsplit),
            ("upper", b_upper),
            ("lower", b_lower),
            ("swapcase", b_swapcase),
            ("capitalize", b_capitalize),
            ("title", b_title),
            ("isdigit", b_isdigit),
            ("isalpha", b_isalpha),
            ("isalnum", b_isalnum),
            ("isascii", b_isascii),
            ("islower", b_islower),
            ("isupper", b_isupper),
            ("istitle", b_istitle),
            ("isspace", b_isspace),
            ("translate", b_translate),
        ];
        for (n, f) in fs {
            def(&cls, leak!(tname, n), *f);
        }
        def_class(&cls, leak!(tname, "fromhex"), b_fromhex);
        def_static(&cls, leak!(tname, "maketrans"), b_maketrans);
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

/// UTF-8 as CPython decodes it: each maximal bad subpart is one error
fn decode_utf8(vm: &mut Vm, b: &[u8], errors: &str) -> PyResult<String> {
    let mut out = String::with_capacity(b.len());
    let mut rest = b;
    let mut pos = 0;
    loop {
        match std::str::from_utf8(rest) {
            Ok(s) => {
                out.push_str(s);
                return Ok(out);
            }
            Err(e) => {
                let ok = e.valid_up_to();
                out.push_str(std::str::from_utf8(&rest[..ok]).unwrap());
                let lead = rest[ok];
                let (n, reason) = match e.error_len() {
                    None => (rest.len() - ok, "unexpected end of data"),
                    Some(n) if lead < 0xc2 || lead > 0xf4 => (n, "invalid start byte"),
                    Some(n) => (n, "invalid continuation byte"),
                };
                let start = pos + ok;
                match errors {
                    "replace" => out.push('\u{fffd}'),
                    "ignore" => {}
                    "backslashreplace" => {
                        for c in &rest[ok..ok + n] {
                            out.push_str(&format!("\\x{:02x}", c));
                        }
                    }
                    _ => return Err(crate::builtins::decode_error(vm, "utf-8", b, start, start + n, reason)),
                }
                rest = &rest[ok + n..];
                pos = start + n;
            }
        }
    }
}

fn decode_bytes(vm: &mut Vm, b: &[u8], enc: &str, errors: &str) -> PyResult<String> {
    let enc = enc.to_ascii_lowercase().replace('_', "-");
    match enc.as_str() {
        "utf-8" | "utf8" | "u8" | "utf-8-sig" => {
            let b = if enc == "utf-8-sig" && b.starts_with(&[0xef, 0xbb, 0xbf]) { &b[3..] } else { b };
            decode_utf8(vm, b, errors)
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
                    return Err(crate::builtins::decode_error(vm, "ascii", b, i, i + 1, "ordinal not in range(128)"));
                }
            }
            Ok(out)
        }
        "latin-1" | "latin1" | "iso-8859-1" | "iso8859-1" | "l1" | "8859" | "cp819" | "latin" => Ok(b.iter().map(|&c| c as char).collect()),
        "raw-unicode-escape" | "unicode-escape" => unescape_bytes(vm, b, enc == "raw-unicode-escape", errors),
        "utf-16" | "utf16" | "utf-16-le" | "utf-16le" | "utf-16-be" | "utf-16be" | "utf-32" | "utf32" | "utf-32-le" | "utf-32le" | "utf-32-be" | "utf-32be" => {
            let wide = enc.contains("32");
            let unit = if wide { 4 } else { 2 };
            let mut big = enc.ends_with("be");
            let mut b = b;
            if !enc.contains('l') && !enc.ends_with("be") {
                // a byte order mark decides, little-endian without one
                if wide && b.starts_with(&[0xff, 0xfe, 0, 0]) {
                    b = &b[4..];
                } else if wide && b.starts_with(&[0, 0, 0xfe, 0xff]) {
                    b = &b[4..];
                    big = true;
                } else if !wide && b.starts_with(&[0xff, 0xfe]) {
                    b = &b[2..];
                } else if !wide && b.starts_with(&[0xfe, 0xff]) {
                    b = &b[2..];
                    big = true;
                }
            }
            let name = if wide { "utf-32" } else { "utf-16" };
            let mut units = vec![];
            for (i, ch) in b.chunks(unit).enumerate() {
                if ch.len() < unit {
                    if errors == "ignore" {
                        break;
                    }
                    if errors == "replace" {
                        units.push(0xfffd);
                        break;
                    }
                    return Err(vm.err("UnicodeDecodeError", format!("'{}-{}' codec can't decode byte 0x{:02x} in position {}: truncated data", name, if big { "be" } else { "le" }, ch[0], i * unit)));
                }
                let mut v: u32 = 0;
                for k in 0..unit {
                    let byte = if big { ch[k] } else { ch[unit - 1 - k] } as u32;
                    v = (v << 8) | byte;
                }
                units.push(v);
            }
            let mut out = String::new();
            let mut i = 0;
            while i < units.len() {
                let u = units[i];
                let c = if !wide && (0xd800..0xdc00).contains(&u) && i + 1 < units.len() && (0xdc00..0xe000).contains(&units[i + 1]) {
                    i += 1;
                    char::from_u32(0x10000 + ((u - 0xd800) << 10) + (units[i] - 0xdc00))
                } else {
                    char::from_u32(u)
                };
                match c {
                    Some(c) => out.push(c),
                    None if errors == "replace" => out.push('\u{fffd}'),
                    None if errors == "ignore" => {}
                    None => return Err(vm.err("UnicodeDecodeError", format!("'{}' codec can't decode bytes in position {}-{}: {}", name, i * unit, i * unit + unit - 1, if wide { "code point not in range(0x110000)" } else { "illegal encoding" }))),
                }
                i += 1;
            }
            Ok(out)
        }
        "utf-7" | "utf7" | "u7" | "unicode-1-1-utf-7" => decode_utf7(vm, b, errors),
        _ => match registry_codec(vm, &enc, Value::Bytes(Rc::new(b.to_vec())), errors, false)? {
            Value::Str(r) => Ok(r.s.clone()),
            _ => unreachable!(),
        },
    }
}

/// the unicode-escape and raw-unicode-escape decoders
fn unescape_bytes(vm: &mut Vm, b: &[u8], raw: bool, errors: &str) -> PyResult<String> {
    let codec = if raw { "rawunicodeescape" } else { "unicodeescape" };
    let mut out = String::new();
    let mut i = 0;
    let bad = |vm: &mut Vm, start: usize, end: usize, msg: &str| crate::builtins::decode_error(vm, codec, b, start, end, msg);
    while i < b.len() {
        let c = b[i];
        if c != b'\\' || i + 1 >= b.len() {
            if c == b'\\' && !raw {
                if errors == "ignore" {
                    i += 1;
                    continue;
                }
                if errors == "replace" {
                    out.push('\u{fffd}');
                    i += 1;
                    continue;
                }
                return Err(crate::builtins::decode_error(vm, codec, b, i, i + 1, "\\ at end of string"));
            }
            out.push(c as char);
            i += 1;
            continue;
        }
        let e = b[i + 1];
        let hex = |n: usize, at: usize| -> Option<u32> {
            if at + n > b.len() {
                return None;
            }
            let s = std::str::from_utf8(&b[at..at + n]).ok()?;
            if !s.chars().all(|c| c.is_ascii_hexdigit()) {
                return None;
            }
            u32::from_str_radix(s, 16).ok()
        };
        if raw {
            let n = match e {
                b'u' => 4,
                b'U' => 8,
                _ => {
                    out.push('\\');
                    i += 1;
                    continue;
                }
            };
            match hex(n, i + 2).and_then(char::from_u32) {
                Some(ch) => {
                    out.push(ch);
                    i += 2 + n;
                }
                None => {
                    if errors == "ignore" || errors == "replace" {
                        if errors == "replace" {
                            out.push('\u{fffd}');
                        }
                        i += 2;
                        while i < b.len() && (b[i] as char).is_ascii_hexdigit() {
                            i += 1;
                        }
                        continue;
                    }
                    let msg = if hex(n, i + 2).is_some() { "\\Uxxxxxxxx out of range" } else if n == 4 { "truncated \\uXXXX escape" } else { "truncated \\UXXXXXXXX escape" };
                    return Err(bad(vm, i, (i + 2 + n).min(b.len()), msg));
                }
            }
            continue;
        }
        i += 2;
        match e {
            b'\n' => {}
            b'\\' => out.push('\\'),
            b'\'' => out.push('\''),
            b'"' => out.push('"'),
            b'a' => out.push('\x07'),
            b'b' => out.push('\x08'),
            b'f' => out.push('\x0c'),
            b'n' => out.push('\n'),
            b'r' => out.push('\r'),
            b't' => out.push('\t'),
            b'v' => out.push('\x0b'),
            b'0'..=b'7' => {
                let mut v = (e - b'0') as u32;
                let mut k = 0;
                while k < 2 && i < b.len() && (b'0'..=b'7').contains(&b[i]) {
                    v = v * 8 + (b[i] - b'0') as u32;
                    i += 1;
                    k += 1;
                }
                out.push(char::from_u32(v).unwrap_or('\u{fffd}'));
            }
            b'x' | b'u' | b'U' => {
                let n = match e {
                    b'x' => 2,
                    b'u' => 4,
                    _ => 8,
                };
                match hex(n, i).and_then(char::from_u32) {
                    Some(ch) => {
                        out.push(ch);
                        i += n;
                    }
                    None => {
                        if errors == "ignore" || errors == "replace" {
                            if errors == "replace" {
                                out.push('\u{fffd}');
                            }
                            while i < b.len() && (b[i] as char).is_ascii_hexdigit() {
                                i += 1;
                            }
                            continue;
                        }
                        let msg = match (e, hex(n, i).is_some()) {
                            (b'U', true) => "illegal Unicode character".to_string(),
                            (b'x', _) => "truncated \\xXX escape".to_string(),
                            (b'u', _) => "truncated \\uXXXX escape".to_string(),
                            _ => "truncated \\UXXXXXXXX escape".to_string(),
                        };
                        return Err(bad(vm, i - 2, (i + n).min(b.len()), &msg));
                    }
                }
            }
            b'N' => {
                let end = if i < b.len() && b[i] == b'{' { b[i..].iter().position(|&x| x == b'}').map(|p| i + p) } else { None };
                let ch = end.and_then(|end| std::str::from_utf8(&b[i + 1..end]).ok().and_then(|n| crate::unicode_name(n)).map(|c| (c, end)));
                match ch {
                    Some((c, end)) => {
                        out.push(c);
                        i = end + 1;
                    }
                    None => {
                        if errors == "ignore" || errors == "replace" {
                            if errors == "replace" {
                                out.push('\u{fffd}');
                            }
                            if let Some(end) = end {
                                i = end + 1;
                            }
                            continue;
                        }
                        return Err(bad(vm, i - 2, end.map_or(b.len(), |e| e + 1), "malformed \\N character escape"));
                    }
                }
            }
            other => {
                out.push('\\');
                out.push(other as char);
            }
        }
    }
    Ok(out)
}

/// \\xhh, \\uhhhh or \\Uhhhhhhhh for one character
fn escape_char(c: char) -> String {
    let n = c as u32;
    if n < 0x100 {
        format!("\\x{:02x}", n)
    } else if n < 0x10000 {
        format!("\\u{:04x}", n)
    } else {
        format!("\\U{:08x}", n)
    }
}

/// what an error handler puts for a character the codec can't encode
fn encode_replacement(errors: &str, c: char, ascii_q: bool) -> Option<String> {
    match errors {
        "replace" => Some(if ascii_q { "?".into() } else { "?".into() }),
        "ignore" => Some(String::new()),
        "backslashreplace" => Some(escape_char(c)),
        "xmlcharrefreplace" => Some(format!("&#{};", c as u32)),
        "namereplace" => Some(match crate::unicode_name_of(c) {
            Some(n) => format!("\\N{{{}}}", n),
            None => escape_char(c),
        }),
        _ => None,
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
        "ascii" | "us-ascii" | "latin-1" | "latin1" | "iso-8859-1" | "iso8859-1" | "l1" | "8859" | "cp819" | "latin" => {
            let limit = if e.contains("ascii") { 128 } else { 256 };
            let mut out = vec![];
            for (i, c) in st.chars().enumerate() {
                if (c as u32) < limit {
                    out.push(c as u32 as u8);
                } else if let Some(r) = encode_replacement(errors, c, true) {
                    out.extend_from_slice(r.as_bytes());
                } else {
                    let name = if limit == 128 { "ascii" } else { "latin-1" };
                    return Err(crate::builtins::encode_error(vm, name, st, i, i + 1, &format!("ordinal not in range({})", limit)));
                }
            }
            Ok(out)
        }
        "raw-unicode-escape" => {
            let mut out = vec![];
            for c in st.chars() {
                if (c as u32) < 0x100 {
                    out.push(c as u32 as u8);
                } else {
                    out.extend_from_slice(escape_char(c).as_bytes());
                }
            }
            Ok(out)
        }
        "unicode-escape" => {
            let mut out = String::new();
            for c in st.chars() {
                match c {
                    '\\' => out.push_str("\\\\"),
                    '\t' => out.push_str("\\t"),
                    '\n' => out.push_str("\\n"),
                    '\r' => out.push_str("\\r"),
                    ' '..='~' => out.push(c),
                    _ => out.push_str(&escape_char(c)),
                }
            }
            Ok(out.into_bytes())
        }
        "utf-16" | "utf16" | "utf-16-le" | "utf-16le" | "utf-16-be" | "utf-16be" | "utf-32" | "utf32" | "utf-32-le" | "utf-32le" | "utf-32-be" | "utf-32be" => {
            let wide = e.contains("32");
            let big = e.ends_with("be");
            let mut out = vec![];
            if !e.contains('l') && !big {
                out.extend_from_slice(if wide { &[0xff, 0xfe, 0, 0][..] } else { &[0xff, 0xfe][..] });
            }
            let mut push = |u: u32, n: usize| {
                let bytes: Vec<u8> = (0..n).map(|k| (u >> (8 * k)) as u8).collect();
                if big {
                    out.extend(bytes.iter().rev());
                } else {
                    out.extend(bytes.iter());
                }
            };
            for c in st.chars() {
                if wide {
                    push(c as u32, 4);
                } else {
                    let mut buf = [0u16; 2];
                    for u in c.encode_utf16(&mut buf) {
                        push(*u as u32, 2);
                    }
                }
            }
            Ok(out)
        }
        "utf-7" | "utf7" | "u7" | "unicode-1-1-utf-7" => Ok(encode_utf7(st)),
        _ => match registry_codec(vm, enc, Value::str(st), errors, true)? {
            Value::Bytes(b) => Ok(b.to_vec()),
            _ => unreachable!(),
        },
    }
}

/// an encoding the native codecs do not know goes through the codecs
/// registry, as str.encode/bytes.decode do in CPython; the result is a str
/// for decoding and bytes for encoding
fn registry_codec(vm: &mut Vm, enc: &str, input: Value, errors: &str, encode: bool) -> PyResult<Value> {
    let g = vm.main_globals.clone();
    let m = vm.import("codecs", &Value::None, 0, &g)?;
    let lookup = vm.getattr(&m, &ps("lookup"))?;
    let info = vm.call_vec(&lookup, vec![Value::str(enc)], &[])?;
    // search functions may return plain 4-tuples: (encode, decode, reader, writer)
    let f = match vm.base_of(&info) {
        Value::Tuple(t) if t.len() >= 2 => t[if encode { 0 } else { 1 }].clone(),
        _ => return Err(vm.type_error("codec search functions must return 4-tuples")),
    };
    let r = vm.call_vec(&f, vec![input, Value::str(errors)], &[])?;
    let first = match vm.base_of(&r) {
        Value::Tuple(t) if t.len() == 2 => t[0].clone(),
        _ => {
            let what = if encode { "encoder" } else { "decoder" };
            return Err(vm.type_error(format!("{} must return a tuple (object, integer)", what)));
        }
    };
    let (ok, what, want, verb) = if encode {
        (matches!(first, Value::Bytes(_) | Value::ByteArray(_)), "encoder", "bytes", "encode")
    } else {
        (matches!(vm.base_of(&first), Value::Str(_)), "decoder", "str", "decode")
    };
    if !ok {
        let tn = vm.type_name(&first);
        return Err(vm.type_error(format!("'{}' {} returned '{}' instead of '{}'; use codecs.{}() to {} to arbitrary types", enc, what, tn, want, verb, verb)));
    }
    Ok(if encode { Value::Bytes(Rc::new(bytes_of(vm, &first)?)) } else { vm.base_of(&first) })
}

/// RFC 2152 character classes as CPython's utf7_category: 0 direct, 1
/// optional direct, 2 whitespace, 3 special
fn utf7_category(c: u32) -> u8 {
    match c {
        0x09 | 0x0a | 0x0d | 0x20 => 2,
        0x27..=0x29 | 0x2c..=0x3a | 0x3f | 0x41..=0x5a | 0x61..=0x7a => 0,
        0x21..=0x26 | 0x2a | 0x3b..=0x3e | 0x40 | 0x5b | 0x5d..=0x60 | 0x7b..=0x7d => 1,
        _ => 3,
    }
}

const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn from_b64(c: u8) -> Option<u32> {
    B64.iter().position(|&x| x == c).map(|i| i as u32)
}

/// UTF-7 the way CPython writes it: only the special characters are encoded
fn encode_utf7(st: &str) -> Vec<u8> {
    let direct = |c: u32| c < 128 && c > 0 && utf7_category(c) != 3;
    let mut out = vec![];
    let (mut in_shift, mut bits, mut buf) = (false, 0u32, 0u64);
    for ch in st.chars() {
        let c = ch as u32;
        if in_shift {
            if direct(c) {
                // shift out, flushing the partial sextet
                if bits > 0 {
                    out.push(B64[((buf << (6 - bits)) & 0x3f) as usize]);
                    bits = 0;
                    buf = 0;
                }
                in_shift = false;
                if from_b64(c as u8).is_some() || c == u32::from(b'-') {
                    out.push(b'-');
                }
                out.push(c as u8);
                continue;
            }
        } else if ch == '+' {
            out.extend_from_slice(b"+-");
            continue;
        } else if direct(c) {
            out.push(c as u8);
            continue;
        } else {
            out.push(b'+');
            in_shift = true;
        }
        let mut buf16 = [0u16; 2];
        for u in ch.encode_utf16(&mut buf16) {
            bits += 16;
            buf = (buf << 16) | u64::from(*u);
            while bits >= 6 {
                out.push(B64[((buf >> (bits - 6)) & 0x3f) as usize]);
                bits -= 6;
            }
            buf &= (1 << bits) - 1;
        }
    }
    if bits > 0 {
        out.push(B64[((buf << (6 - bits)) & 0x3f) as usize]);
    }
    if in_shift {
        out.push(b'-');
    }
    out
}

/// CPython's UTF-7 decoder; a lone surrogate cannot be a Rust char, so one
/// decodes to U+FFFD
fn decode_utf7(vm: &mut Vm, b: &[u8], errors: &str) -> PyResult<String> {
    let mut out = String::new();
    let (mut in_shift, mut bits, mut buf, mut surrogate) = (false, 0u32, 0u64, 0u32);
    let mut shift_start = 0;
    let mut i = 0;
    let push = |out: &mut String, u: u32| out.push(char::from_u32(u).unwrap_or('\u{fffd}'));
    while i < b.len() {
        let ch = b[i];
        let start = i;
        let err: &str;
        if in_shift {
            if let Some(v) = from_b64(ch) {
                buf = (buf << 6) | u64::from(v);
                bits += 6;
                i += 1;
                if bits >= 16 {
                    let u = ((buf >> (bits - 16)) & 0xffff) as u32;
                    bits -= 16;
                    buf &= (1 << bits) - 1;
                    if surrogate != 0 {
                        if (0xdc00..0xe000).contains(&u) {
                            push(&mut out, 0x10000 + ((surrogate - 0xd800) << 10) + (u - 0xdc00));
                            surrogate = 0;
                            continue;
                        }
                        push(&mut out, surrogate);
                        surrogate = 0;
                    }
                    if (0xd800..0xdc00).contains(&u) {
                        surrogate = u;
                    } else {
                        push(&mut out, u);
                    }
                }
                continue;
            }
            // leaving a base-64 section
            in_shift = false;
            if bits >= 6 {
                i += 1;
                err = "partial character in shift sequence";
            } else if bits > 0 && buf != 0 {
                i += 1;
                err = "non-zero padding bits in shift sequence";
            } else {
                if surrogate != 0 && ch <= 127 && ch != b'+' {
                    push(&mut out, surrogate);
                }
                surrogate = 0;
                if ch == b'-' {
                    i += 1;
                }
                continue;
            }
        } else if ch == b'+' {
            i += 1;
            if i < b.len() && b[i] == b'-' {
                i += 1;
                out.push('+');
                continue;
            }
            if i < b.len() && from_b64(b[i]).is_none() {
                i += 1;
                err = "ill-formed sequence";
            } else {
                in_shift = true;
                surrogate = 0;
                shift_start = start;
                bits = 0;
                buf = 0;
                continue;
            }
        } else if ch <= 127 {
            i += 1;
            out.push(ch as char);
            continue;
        } else {
            i += 1;
            err = "unexpected special character";
        }
        match errors {
            "ignore" => {}
            "replace" => out.push('\u{fffd}'),
            _ => return Err(crate::builtins::decode_error(vm, "utf7", b, start, i, err)),
        }
    }
    if in_shift && (surrogate != 0 || bits >= 6 || (bits > 0 && buf != 0)) {
        match errors {
            "ignore" => {}
            "replace" => out.push('\u{fffd}'),
            _ => return Err(crate::builtins::decode_error(vm, "utf7", b, shift_start, b.len(), "unterminated shift sequence")),
        }
    }
    Ok(out)
}

fn str_new(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let npos = a.len() - 1;
    if npos > 3 {
        return Err(vm.type_error(format!("str expected at most 3 arguments, got {}", npos)));
    }
    if npos + k.len() > 3 {
        return Err(vm.type_error(format!("str() takes at most 3 arguments ({} given)", npos + k.len())));
    }
    if let Some((n, _)) = k.iter().find(|(n, _)| !matches!(&*n.s, "object" | "encoding" | "errors")) {
        return Err(vm.type_error(format!("str() got an unexpected keyword argument '{}'", n.s)));
    }
    for (pos, name) in [(1, "object"), (2, "encoding"), (3, "errors")] {
        if npos >= pos && kwarg(k, name).is_some() {
            return Err(vm.type_error(format!("argument for str() given by name ('{}') and position ({})", name, pos)));
        }
    }
    let obj = arg_or_kw(a, 1, k, "object");
    let enc = arg_or_kw(a, 2, k, "encoding");
    let errors = arg_or_kw(a, 3, k, "errors");
    for (name, v) in [("encoding", &enc), ("errors", &errors)] {
        if let Some(v) = v {
            if !matches!(vm.base_of(v), Value::Str(_)) {
                let tn = vm.type_name(v);
                return Err(vm.type_error(format!("str() argument '{}' must be str, not {}", name, tn)));
            }
        }
    }
    let v = match obj {
        None => Value::str(""),
        Some(o) => {
            if enc.is_some() || errors.is_some() {
                let b = bytes_of(vm, &o)?;
                let e = enc.as_ref().and_then(|v| vm.base_of(v).as_str().map(|s| s.to_string())).unwrap_or("utf-8".into());
                let er = errors.as_ref().and_then(|v| vm.base_of(v).as_str().map(|s| s.to_string())).unwrap_or("strict".into());
                Value::string(decode_bytes(vm, &b, &e, &er)?)
            } else {
                match &o {
                    Value::Str(_) => o.clone(),
                    _ => {
                        let exact = matches!(&a[0], Value::Class(c) if Rc::ptr_eq(c, &vm.t.str_));
                        let mut r = crate::builtins::user_str_result(vm, &o, "__str__")?;
                        if r.is_none() && !matches!(vm.base_of(&o), Value::Str(_)) {
                            // object.__str__ is repr(): keep a str subclass it returns
                            let native_str = vm.lookup_special(&o, &ps("__str__")).is_none_or(|f| !matches!(&f, Value::Method(m) if matches!(m.1, Value::Func(_))));
                            if native_str {
                                r = crate::builtins::user_str_result(vm, &o, "__repr__")?;
                            }
                        }
                        match r {
                            Some(r) if exact => return Ok(r),
                            Some(r) => vm.base_of(&r),
                            None => match vm.base_of(&o) {
                                b @ Value::Str(_) if vm.lookup_special(&o, &ps("__str__")).is_some_and(|f| matches!(&f, Value::Method(m) if matches!(m.1, Value::Native(_)))) => b,
                                _ => Value::string(vm.str_of(&o)?),
                            },
                        }
                    }
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
    crate::builtins::need(vm, a, 2)?;
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
    crate::builtins::need(vm, a, 2)?;
    let x = me(vm, a, "__add__")?;
    match vm.base_of(&a[1]) {
        Value::Str(y) => Ok(s(format!("{}{}", x.s, y.s))),
        _ => Ok(Value::NotImplemented),
    }
}
fn str_mul(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 2)?;
    let x = me(vm, a, "__mul__")?;
    match vm.base_of(&a[1]) {
        Value::Int(n) => { let n = vm.repeat_count(x.s.len(), n)?; Ok(s(x.s.repeat(n))) }
        Value::Bool(b) => Ok(s(if b { x.s.clone() } else { String::new() })),
        Value::Big(_) => Err(vm.err("OverflowError", "cannot fit 'int' into an index-sized integer")),
        _ => match vm.lookup_special(&a[1], &vm.n.index.clone()) {
            Some(_) => {
                let n = vm.to_index(&a[1])?;
                let n = vm.repeat_count(x.s.len(), n)?;
                Ok(s(x.s.repeat(n)))
            }
            None => {
                let tn = vm.type_name(&a[1]);
                Err(vm.type_error(format!("can't multiply sequence by non-int of type '{}'", tn)))
            }
        },
    }
}
fn str_mod(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 2)?;
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
    Ok(Value::Int(x.hash() as i64))
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
        if matches!(a[0], Value::Instance(_)) {
            return Ok(Value::string(vm.str_of(&a[0])?));
        }
        return Ok(Value::Str(x));
    }
    Ok(s(crate::format::format_spec(vm, &Value::Str(x), &spec.s)?))
}
fn str_getnewargs(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = me(vm, a, "__getnewargs__")?;
    // a copy, as CPython's
    Ok(Value::tuple(vec![Value::string(x.s.clone())]))
}

// -------------------------------------------------------------------------
// str.format

/// the value a replacement field names: "0", "name", "a.b", "x[0]"
fn field_value(vm: &mut Vm, field: &str, args: Option<&[Value]>, kw: &Value, auto: &mut usize, manual: &mut bool) -> PyResult<Value> {
    let end = field.find(['.', '[']).unwrap_or(field.len());
    let head = &field[..end];
    let numeric = head.is_empty() || head.bytes().all(|b| b.is_ascii_digit());
    let args = match args {
        Some(a) => a,
        None if numeric => return Err(vm.value_error("Format string contains positional fields")),
        None => &[],
    };
    if numeric && !head.is_empty() && head.parse::<i64>().is_err() {
        return Err(vm.value_error("Too many decimal digits in format string"));
    }
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
            if name.is_empty() {
                return Err(vm.value_error("Empty attribute in format string"));
            }
            v = vm.getattr(&v, &ps(name))?;
            rest = &r[e..];
        } else if let Some(r) = rest.strip_prefix('[') {
            let e = match r.find(']') {
                Some(e) => e,
                None => return Err(vm.value_error("Missing ']' in format string")),
            };
            let key = &r[..e];
            if key.is_empty() {
                return Err(vm.value_error("Empty attribute in format string"));
            }
            if key.bytes().all(|b| b.is_ascii_digit()) && key.parse::<i64>().is_err() {
                return Err(vm.value_error("Too many decimal digits in format string"));
            }
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
    format_inner(vm, fmt, Some(args), kw, &mut auto, &mut manual, 0)
}

fn format_inner(vm: &mut Vm, fmt: &str, args: Option<&[Value]>, kw: &Value, auto: &mut usize, manual: &mut bool, depth: u32) -> PyResult<String> {
    if depth >= 2 {
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
            let mut inb = false;
            for b in field.chars() {
                match b {
                    '[' => inb = true,
                    ']' => inb = false,
                    '{' | '}' if !inb => return Err(vm.value_error("unexpected '{' in field name")),
                    _ => {}
                }
            }
            let mut rest = &body[field_end..];
            let mut conv = None;
            if let Some(r) = rest.strip_prefix('!') {
                let cc = r.chars().next();
                if cc.is_none() || cc == Some(':') {
                    return Err(vm.value_error("end of string while looking for conversion specifier"));
                }
                conv = cc;
                rest = &r[cc.map(|x| x.len_utf8()).unwrap_or(0)..];
                if !rest.is_empty() && !rest.starts_with(':') {
                    return Err(vm.value_error("expected ':' after conversion specifier"));
                }
            }
            let spec = rest.strip_prefix(':').unwrap_or("");
            let v = field_value(vm, field, args, kw, auto, manual)?;
            let spec = if spec.contains('{') { format_inner(vm, spec, args, kw, auto, manual, depth + 1)? } else { spec.to_string() };
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
    let (mut auto, mut manual) = (0usize, false);
    Ok(s(format_inner(vm, &x.s, None, &a[1], &mut auto, &mut manual, 0)?))
}

// -------------------------------------------------------------------------
// the rest of str

fn str_join(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let sep = me(vm, a, "join")?;
    margs(vm, "str.join", a, 1, 1)?;
    let items = vm.to_vec(&a[1])?;
    if let [x @ Value::Str(_)] = &items[..] {
        return Ok(x.clone());
    }
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
/// a slice index as CPython's _PyEval_SliceIndex: big values clamp
fn slice_index(vm: &mut Vm, v: &Value) -> PyResult<i64> {
    match vm.base_of(v) {
        Value::Big(b) => Ok(if b.neg { i64::MIN / 2 } else { i64::MAX / 2 }),
        _ => match vm.to_index(v) {
            Ok(i) => Ok(i),
            Err(_) => Err(vm.type_error("slice indices must be integers or None or have an __index__ method")),
        },
    }
}

/// start and end as find/count take them: (start byte, end byte, start
/// char, end char); the start char may lie past the end of the string
fn sub_range(vm: &mut Vm, x: &PyStr, a: &[Value], i: usize) -> PyResult<(usize, usize, usize, usize)> {
    let n = x.char_len() as i64;
    let adjust = |v: i64| -> i64 { if v < 0 { (v + n).max(0) } else { v } };
    let st = match a.get(i) {
        Some(Value::None) | None => 0,
        Some(v) => adjust(slice_index(vm, v)?),
    };
    let en = match a.get(i + 1) {
        Some(Value::None) | None => n,
        Some(v) => adjust(slice_index(vm, v)?).min(n),
    };
    let to_byte = |ci: i64| -> usize {
        let ci = ci.min(n);
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
    // nothing to replace gives back the string itself, as CPython does
    if matches!(&a[0], Value::Str(_)) && (count == 0 || Rc::ptr_eq(&old, &new) || (!old.s.is_empty() && !x.s.contains(&*old.s))) {
        return Ok(a[0].clone());
    }
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

use crate::unidata as ud;

fn cased(c: char) -> bool {
    ud::is_in(&ud::CASED, c)
}

/// a capital sigma at i ends a word: cased before it, none after
/// (case-ignorable characters skipped)
fn final_sigma(cs: &[char], i: usize) -> bool {
    let before = cs[..i].iter().rev().find(|c| !ud::is_in(&ud::CASE_IGNORABLE, **c));
    let after = cs[i + 1..].iter().find(|c| !ud::is_in(&ud::CASE_IGNORABLE, **c));
    before.is_some_and(|c| cased(*c)) && !after.is_some_and(|c| cased(*c))
}

fn push_lower(out: &mut String, cs: &[char], i: usize) {
    if cs[i] == 'Σ' {
        out.push(if final_sigma(cs, i) { 'ς' } else { 'σ' });
    } else {
        ud::push_lower(out, cs[i]);
    }
}

fn push_title(out: &mut String, c: char) {
    match ud::mapped(&ud::TITLE_MAP, c) {
        Some(t) => out.push_str(t),
        None => ud::push_upper(out, c),
    }
}

fn str_upper(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = me(vm, a, "upper")?;
    if x.ascii {
        return Ok(s(x.s.to_ascii_uppercase()));
    }
    let mut out = String::with_capacity(x.s.len());
    for c in x.s.chars() {
        ud::push_upper(&mut out, c);
    }
    Ok(s(out))
}
fn str_lower(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = me(vm, a, "lower")?;
    if x.ascii {
        return Ok(s(x.s.to_ascii_lowercase()));
    }
    let cs: Vec<char> = x.s.chars().collect();
    let mut out = String::with_capacity(x.s.len());
    for i in 0..cs.len() {
        push_lower(&mut out, &cs, i);
    }
    Ok(s(out))
}
fn str_casefold(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = me(vm, a, "casefold")?;
    let mut out = String::with_capacity(x.s.len());
    for c in x.s.chars() {
        match ud::mapped(&ud::FOLD_MAP, c) {
            Some(f) => out.push_str(f),
            None => ud::push_lower(&mut out, c),
        }
    }
    Ok(s(out))
}
fn str_capitalize(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = me(vm, a, "capitalize")?;
    let cs: Vec<char> = x.s.chars().collect();
    let mut out = String::with_capacity(x.s.len());
    for i in 0..cs.len() {
        if i == 0 {
            push_title(&mut out, cs[0]);
        } else {
            push_lower(&mut out, &cs, i);
        }
    }
    Ok(s(out))
}
fn str_title(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = me(vm, a, "title")?;
    let cs: Vec<char> = x.s.chars().collect();
    let mut out = String::with_capacity(x.s.len());
    let mut prev_cased = false;
    for i in 0..cs.len() {
        if prev_cased {
            push_lower(&mut out, &cs, i);
        } else {
            push_title(&mut out, cs[i]);
        }
        prev_cased = cased(cs[i]);
    }
    Ok(s(out))
}
fn str_swapcase(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = me(vm, a, "swapcase")?;
    let cs: Vec<char> = x.s.chars().collect();
    let mut out = String::with_capacity(x.s.len());
    for i in 0..cs.len() {
        let c = cs[i];
        if ud::is_in(&ud::UPPER, c) {
            push_lower(&mut out, &cs, i);
        } else if ud::is_in(&ud::LOWER, c) {
            ud::push_upper(&mut out, c);
        } else {
            out.push(c);
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
    if matches!(&a[0], Value::Str(_)) && !x.s.contains('\t') {
        return Ok(a[0].clone());
    }
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

/// The zero of each run of ten Unicode decimal digits (category Nd)
const DECIMAL_ZEROS: [u32; 68] = [0x30, 0x660, 0x6f0, 0x7c0, 0x966, 0x9e6, 0xa66, 0xae6, 0xb66, 0xbe6, 0xc66, 0xce6, 0xd66, 0xde6, 0xe50, 0xed0, 0xf20, 0x1040, 0x1090, 0x17e0, 0x1810, 0x1946, 0x19d0, 0x1a80, 0x1a90, 0x1b50, 0x1bb0, 0x1c40, 0x1c50, 0xa620, 0xa8d0, 0xa900, 0xa9d0, 0xa9f0, 0xaa50, 0xabf0, 0xff10, 0x104a0, 0x10d30, 0x11066, 0x110f0, 0x11136, 0x111d0, 0x112f0, 0x11450, 0x114d0, 0x11650, 0x116c0, 0x11730, 0x118e0, 0x11950, 0x11c50, 0x11d50, 0x11da0, 0x11f50, 0x16a60, 0x16ac0, 0x16b50, 0x1d7ce, 0x1d7d8, 0x1d7e2, 0x1d7ec, 0x1d7f6, 0x1e140, 0x1e2f0, 0x1e4f0, 0x1e950, 0x1fbf0];

/// The value of a Unicode decimal digit
pub fn decimal_value(c: char) -> Option<u32> {
    let u = c as u32;
    let i = DECIMAL_ZEROS.partition_point(|&z| z <= u);
    if i == 0 {
        return None;
    }
    let d = u - DECIMAL_ZEROS[i - 1];
    (d < 10).then_some(d)
}

fn is_decimal(c: char) -> bool {
    decimal_value(c).is_some()
}

fn str_isdigit(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    char_test(vm, a, "isdigit", |c| ud::is_in(&ud::DIGIT, c))
}
fn str_isdecimal(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    char_test(vm, a, "isdecimal", is_decimal)
}
fn str_isnumeric(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    char_test(vm, a, "isnumeric", |c| ud::is_in(&ud::NUMERIC, c))
}
fn str_isalpha(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    char_test(vm, a, "isalpha", |c| ud::is_in(&ud::ALPHA, c))
}
fn str_isalnum(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    char_test(vm, a, "isalnum", |c| ud::is_in(&ud::ALPHA, c) || ud::is_in(&ud::NUMERIC, c))
}
fn str_isspace(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    char_test(vm, a, "isspace", |c| ud::is_in(&ud::SPACE, c))
}
fn str_isupper(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = me(vm, a, "isupper")?;
    let mut any = false;
    for c in x.s.chars() {
        if ud::is_in(&ud::LOWER, c) || ud::is_in(&ud::TITLE, c) {
            return Ok(Value::Bool(false));
        }
        any |= ud::is_in(&ud::UPPER, c);
    }
    Ok(Value::Bool(any))
}
fn str_islower(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = me(vm, a, "islower")?;
    let mut any = false;
    for c in x.s.chars() {
        if ud::is_in(&ud::UPPER, c) || ud::is_in(&ud::TITLE, c) {
            return Ok(Value::Bool(false));
        }
        any |= ud::is_in(&ud::LOWER, c);
    }
    Ok(Value::Bool(any))
}
fn str_istitle(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = me(vm, a, "istitle")?;
    let mut prev_cased = false;
    let mut any = false;
    for c in x.s.chars() {
        if ud::is_in(&ud::UPPER, c) || ud::is_in(&ud::TITLE, c) {
            if prev_cased {
                return Ok(Value::Bool(false));
            }
            prev_cased = true;
            any = true;
        } else if ud::is_in(&ud::LOWER, c) {
            if !prev_cased {
                return Ok(Value::Bool(false));
            }
            prev_cased = true;
            any = true;
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
        Some(f) => ud::is_in(&ud::XID_START, f) && cs.all(|c| ud::is_in(&ud::XID_CONTINUE, c)),
        None => false,
    }))
}
fn str_isprintable(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = me(vm, a, "isprintable")?;
    Ok(Value::Bool(x.s.chars().all(|c| ud::is_in(&ud::PRINTABLE, c))))
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
                    Value::Str(_) => return Err(vm.value_error("string keys in translate table must be of length 1")),
                    _ => return Err(vm.type_error("keys in translate table must be strings or integers")),
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
            Some(Value::Int(i)) => match u32::try_from(i).ok().filter(|&u| u < 0x110000) {
                Some(u) => out.push(char::from_u32(u).unwrap_or('\u{fffd}')),
                None => return Err(vm.value_error("character mapping must be in range(0x110000)")),
            },
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
    crate::builtins::need(vm, a, 1)?;
    let src = arg_or_kw(a, 1, k, "source");
    let v = bytes_from(vm, src.as_ref(), arg_or_kw(a, 2, k, "encoding"), arg_or_kw(a, 3, k, "errors"))?;
    let t = vm.t.bytes.clone();
    Ok(wrap_new(vm, &a[0], &t, Value::Bytes(Rc::new(v))))
}

fn bytearray_new(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let t = vm.t.bytearray.clone();
    Ok(wrap_new(vm, &a[0], &t, Value::ByteArray(Rc::new(RefCell::new(vec![])))))
}

fn bytearray_init(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
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
    crate::builtins::need(vm, a, 2)?;
    let x = vm.base_of(&a[0]);
    seq_getitem(vm, &x, &a[1])
}
fn b_contains(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 2)?;
    let x = vm.base_of(&a[0]);
    Ok(Value::Bool(vm.contains(&x, &a[1])?))
}
fn b_iter(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = vm.base_of(&a[0]);
    vm.iter(&x)
}
fn b_add(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 2)?;
    let mut x = bme(vm, a)?;
    let y = bytes_of(vm, &a[1])?;
    x.extend(y);
    Ok(same_type(&vm.base_of(&a[0]), x))
}
fn b_eq(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 2)?;
    let x = bme(vm, a)?;
    match bytes_of(vm, &a[1]) {
        Ok(y) => Ok(Value::Bool(x == y)),
        Err(_) => Ok(Value::NotImplemented),
    }
}
fn b_repr(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let x = vm.base_of(&a[0]);
    Ok(s(vm.repr(&x)?))
}
fn b_hash(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = vm.base_of(&a[0]);
    Ok(Value::Int(vm.hash(&x)? as i64))
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
    crate::builtins::need(vm, a, 2)?;
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
    crate::builtins::need(vm, a, 2)?;
    let sep = bme(vm, a)?;
    // bytearray.join holds an export of itself while it consumes the iterable
    let held = match vm.base_of(&a[0]) {
        Value::ByteArray(b) => Some(Rc::as_ptr(&b) as usize),
        _ => None,
    };
    if let Some(p) = held {
        BA_EXPORTS.with(|e| e.borrow_mut().push(p));
    }
    let items = vm.to_vec(&a[1]);
    if let Some(p) = held {
        BA_EXPORTS.with(|e| {
            let mut e = e.borrow_mut();
            if let Some(i) = e.iter().rposition(|x| *x == p) {
                e.remove(i);
            }
        });
    }
    let items = items?;
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
    crate::builtins::need(vm, a, 2)?;
    let x = bme(vm, a)?;
    let n = b_needle(vm, &a[1])?;
    Ok(Value::Int(find_sub(&x, &n).map(|p| p as i64).unwrap_or(-1)))
}
fn b_index(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 2)?;
    let x = bme(vm, a)?;
    let n = b_needle(vm, &a[1])?;
    match find_sub(&x, &n) {
        Some(p) => Ok(Value::Int(p as i64)),
        None => Err(vm.value_error("subsection not found")),
    }
}
fn b_count(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 2)?;
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
    crate::builtins::need(vm, a, 3)?;
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
/// bytes.translate(table, /, delete=b''): drop the bytes in `delete`, map the rest through `table`
fn b_translate(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let x = bme(vm, a)?;
    if a.len() > 3 {
        return Err(vm.type_error(format!("translate() takes at most 2 arguments ({} given)", a.len() - 1)));
    }
    let table = match a.get(1) {
        Some(Value::None) => None,
        Some(t) => {
            let t = bytes_of(vm, t)?;
            if t.len() != 256 {
                return Err(vm.value_error("translation table must be 256 characters long"));
            }
            Some(t)
        }
        None => return Err(vm.type_error("translate() takes at least 1 argument (0 given)")),
    };
    let delete = match a.get(2).cloned().or_else(|| kwarg(k, "delete")) {
        Some(d) => bytes_of(vm, &d)?,
        None => vec![],
    };
    let out = x.iter().filter(|c| !delete.contains(c)).map(|&c| table.as_ref().map_or(c, |t| t[c as usize])).collect();
    Ok(same_type(&vm.base_of(&a[0]), out))
}

/// bytes.maketrans(frm, to): a 256-byte table mapping each byte of `frm` to the one of `to`
fn b_maketrans(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    if a.len() != 2 {
        return Err(vm.type_error(format!("maketrans expected 2 arguments, got {}", a.len())));
    }
    let (frm, to) = (bytes_of(vm, &a[0])?, bytes_of(vm, &a[1])?);
    if frm.len() != to.len() {
        return Err(vm.value_error("maketrans arguments must have same length"));
    }
    let mut table: Vec<u8> = (0..=255).collect();
    for (f, t) in frm.iter().zip(&to) {
        table[*f as usize] = *t;
    }
    Ok(Value::Bytes(Rc::new(table)))
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

thread_local! {
    /// bytearrays whose buffer is exported (held by a running join), by address
    static BA_EXPORTS: RefCell<Vec<usize>> = const { RefCell::new(Vec::new()) };
}

/// the bytearray of `a[0]`, for an operation that changes its size
fn ba_resizable(vm: &mut Vm, a: &[Value]) -> PyResult<Rc<RefCell<Vec<u8>>>> {
    let b = ba_of(vm, a)?;
    if BA_EXPORTS.with(|e| e.borrow().contains(&(Rc::as_ptr(&b) as usize))) {
        return Err(vm.err("BufferError", "Existing exports of data: object cannot be re-sized"));
    }
    Ok(b)
}
fn ba_setitem(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 3)?;
    let x = vm.base_of(&a[0]);
    seq_setitem(vm, &x, &a[1], a[2].clone())?;
    Ok(Value::None)
}
fn ba_delitem(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 2)?;
    let x = vm.base_of(&a[0]);
    seq_delitem(vm, &x, &a[1])?;
    Ok(Value::None)
}
fn ba_iadd(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 2)?;
    let b = ba_resizable(vm, a)?;
    let y = bytes_of(vm, &a[1])?;
    b.borrow_mut().extend(y);
    Ok(a[0].clone())
}
fn ba_append(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 2)?;
    let b = ba_resizable(vm, a)?;
    let i = vm.to_index(&a[1])?;
    if !(0..256).contains(&i) {
        return Err(vm.value_error("byte must be in range(0, 256)"));
    }
    b.borrow_mut().push(i as u8);
    Ok(Value::None)
}
fn ba_extend(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 2)?;
    let b = ba_resizable(vm, a)?;
    let y = bytes_from(vm, Some(&a[1]), None, None)?;
    b.borrow_mut().extend(y);
    Ok(Value::None)
}
fn ba_pop(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let b = ba_resizable(vm, a)?;
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
    let b = ba_resizable(vm, a)?;
    b.borrow_mut().clear();
    Ok(Value::None)
}
fn ba_insert(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 3)?;
    let b = ba_resizable(vm, a)?;
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

// -------------------------------------------------------------------------
// bytes methods done by the str method of the same name over latin-1 text

fn latin1(b: &[u8]) -> String {
    b.iter().map(|&c| c as char).collect()
}

fn unlatin1(s: &str) -> Vec<u8> {
    s.chars().map(|c| c as u32 as u8).collect()
}

fn via_str(vm: &mut Vm, a: &[Value], k: Kw, name: &str, needle_int: bool) -> PyResult<Value> {
    let me_v = vm.base_of(&a[0]);
    let x = bytes_of(vm, &me_v)?;
    let mut args = vec![Value::string(latin1(&x))];
    for (i, v) in a[1..].iter().enumerate() {
        args.push(match v {
            Value::Bytes(b) => Value::string(latin1(b)),
            Value::ByteArray(b) => Value::string(latin1(&b.borrow())),
            Value::Int(n) if i == 0 && needle_int => {
                if !(0..256).contains(n) {
                    return Err(vm.value_error("byte must be in range(0, 256)"));
                }
                Value::string(latin1(&[*n as u8]))
            }
            Value::Tuple(t) if t.iter().all(|e| matches!(e, Value::Bytes(_) | Value::ByteArray(_))) => {
                let mut items = vec![];
                for e in t.iter() {
                    let b = bytes_of(vm, e)?;
                    items.push(Value::string(latin1(&b)));
                }
                Value::tuple(items)
            }
            Value::Str(_) if i == 0 || name == "replace" || name.ends_with("just") || name == "center" => {
                let tn = vm.type_name(v);
                return Err(vm.type_error(format!("a bytes-like object is required, not '{}'", tn)));
            }
            Value::Instance(_) => {
                let b = vm.base_of(v);
                match &b {
                    Value::Bytes(bb) => Value::string(latin1(bb)),
                    Value::ByteArray(bb) => Value::string(latin1(&bb.borrow())),
                    _ => v.clone(),
                }
            }
            other => other.clone(),
        });
    }
    let mut kw = vec![];
    for (n, v) in k {
        let v = match v {
            Value::Bytes(b) => Value::string(latin1(b)),
            Value::ByteArray(b) => Value::string(latin1(&b.borrow())),
            o => o.clone(),
        };
        kw.push((n.clone(), v));
    }
    let f = vm.t.str_.dict.borrow().get_s(name).unwrap();
    let r = vm.call_vec(&f, args, &kw)?;
    Ok(from_text(&me_v, r))
}

fn from_text(me: &Value, r: Value) -> Value {
    match r {
        Value::Str(s) => same_type(me, unlatin1(&s.s)),
        Value::List(l) => {
            let items: Vec<Value> = l.borrow().iter().map(|v| from_text(me, v.clone())).collect();
            Value::list(items)
        }
        Value::Tuple(t) => Value::tuple(t.iter().map(|v| from_text(me, v.clone())).collect()),
        other => other,
    }
}

macro_rules! bytes_via_str {
    ($( $fname:ident, $name:expr, $needle:expr );* $(;)?) => {
        $(
            fn $fname(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
                via_str(vm, a, k, $name, $needle)
            }
        )*
    };
}

bytes_via_str! {
    bs_startswith, "startswith", false;
    bs_endswith, "endswith", false;
    bs_find, "find", true;
    bs_rfind, "rfind", true;
    bs_index, "index", true;
    bs_rindex, "rindex", true;
    bs_count, "count", true;
    bs_replace, "replace", false;
    bs_partition, "partition", false;
    bs_rpartition, "rpartition", false;
    bs_center, "center", false;
    bs_ljust, "ljust", false;
    bs_rjust, "rjust", false;
    bs_zfill, "zfill", false;
    bs_expandtabs, "expandtabs", false;
    bs_removeprefix, "removeprefix", false;
    bs_removesuffix, "removesuffix", false;
    bs_splitlines, "splitlines", false;
}

fn bs_rsplit(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let sep = arg_or_kw(a, 1, k, "sep").filter(|v| !v.is_none());
    if sep.is_some() {
        return via_str(vm, a, k, "rsplit", false);
    }
    // ASCII whitespace only
    let me_v = vm.base_of(&a[0]);
    let x = bytes_of(vm, &me_v)?;
    let max = match arg_or_kw(a, 2, k, "maxsplit") {
        Some(v) => vm.to_index(&v)?,
        None => -1,
    };
    let ws = |c: u8| matches!(c, b' ' | b'\t' | b'\n' | b'\r' | 0x0b | 0x0c);
    let mut out = vec![];
    let mut end = x.len();
    let mut n = 0;
    loop {
        while end > 0 && ws(x[end - 1]) {
            end -= 1;
        }
        if end == 0 {
            break;
        }
        if max >= 0 && n >= max {
            out.push(same_type(&me_v, x[..end].to_vec()));
            break;
        }
        let mut start = end;
        while start > 0 && !ws(x[start - 1]) {
            start -= 1;
        }
        out.push(same_type(&me_v, x[start..end].to_vec()));
        end = start;
        n += 1;
    }
    out.reverse();
    Ok(Value::list(out))
}

fn b_map(vm: &mut Vm, a: &[Value], f: impl Fn(&[u8]) -> Vec<u8>) -> PyResult<Value> {
    let me_v = vm.base_of(&a[0]);
    let x = bytes_of(vm, &me_v)?;
    Ok(same_type(&me_v, f(&x)))
}

fn b_swapcase(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    b_map(vm, a, |x| x.iter().map(|c| if c.is_ascii_lowercase() { c.to_ascii_uppercase() } else { c.to_ascii_lowercase() }).collect())
}

fn b_capitalize(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    b_map(vm, a, |x| x.iter().enumerate().map(|(i, c)| if i == 0 { c.to_ascii_uppercase() } else { c.to_ascii_lowercase() }).collect())
}

fn b_title(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    b_map(vm, a, |x| {
        let mut prev = false;
        x.iter()
            .map(|&c| {
                let r = if prev { c.to_ascii_lowercase() } else { c.to_ascii_uppercase() };
                prev = c.is_ascii_alphabetic();
                r
            })
            .collect()
    })
}

fn b_test(vm: &mut Vm, a: &[Value], f: impl Fn(&[u8]) -> bool) -> PyResult<Value> {
    let x = bme(vm, a)?;
    Ok(Value::Bool(f(&x)))
}

fn b_isalnum(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    b_test(vm, a, |x| !x.is_empty() && x.iter().all(|c| c.is_ascii_alphanumeric()))
}

fn b_isascii(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    b_test(vm, a, |x| x.is_ascii())
}

fn b_islower(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    b_test(vm, a, |x| x.iter().any(|c| c.is_ascii_lowercase()) && !x.iter().any(|c| c.is_ascii_uppercase()))
}

fn b_isupper(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    b_test(vm, a, |x| x.iter().any(|c| c.is_ascii_uppercase()) && !x.iter().any(|c| c.is_ascii_lowercase()))
}

fn b_istitle(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    b_test(vm, a, |x| {
        let mut prev = false;
        let mut any = false;
        for &c in x {
            if c.is_ascii_uppercase() {
                if prev {
                    return false;
                }
                any = true;
                prev = true;
            } else if c.is_ascii_lowercase() {
                if !prev {
                    return false;
                }
                prev = true;
            } else {
                prev = false;
            }
        }
        any
    })
}
