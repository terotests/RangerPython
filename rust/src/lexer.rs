// SPDX-License-Identifier: MIT
//! Tokenizer: Python source to a token list with INDENT/DEDENT.

use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub enum Tok {
    Name(Rc<str>),
    Int(i64),
    /// an integer literal too large for i64, kept as its digits
    BigInt(String),
    Float(f64),
    Imag(f64),
    /// a string literal; `f` holds the raw body of an f-string
    Str { s: String, bytes: bool, f: bool },
    Op(&'static str),
    Newline,
    Indent,
    Dedent,
    Eof,
}

#[derive(Clone, Debug)]
pub struct Token {
    pub tok: Tok,
    pub line: u32,
}

pub struct LexError {
    pub msg: String,
    pub line: u32,
}

const OPS3: [&str; 6] = ["**=", "//=", ">>=", "<<=", "...", "!="];
const OPS2: [&str; 22] = [
    "**", "//", "==", "!=", "<=", ">=", "<<", ">>", "+=", "-=", "*=", "/=", "%=", "&=", "|=", "^=", "->", ":=", "@=",
    "<>", "!=", "==",
];
const OPS1: &str = "+-*/%@&|^~<>()[]{},:.;=!";

fn op_str(s: &str) -> &'static str {
    for o in OPS3.iter().chain(OPS2.iter()) {
        if *o == s {
            return o;
        }
    }
    const ONE: [&str; 24] = [
        "+", "-", "*", "/", "%", "@", "&", "|", "^", "~", "<", ">", "(", ")", "[", "]", "{", "}", ",", ":", ".", ";", "=", "!",
    ];
    for o in ONE.iter() {
        if *o == s {
            return o;
        }
    }
    "?"
}

pub fn tokenize(src: &str) -> Result<Vec<Token>, LexError> {
    let chars: Vec<char> = src.chars().collect();
    let mut toks: Vec<Token> = Vec::new();
    let mut indents: Vec<usize> = vec![0];
    let mut i = 0usize;
    let mut line: u32 = 1;
    let mut depth = 0i32;
    let mut at_line_start = true;
    let n = chars.len();
    let err = |msg: &str, line: u32| LexError { msg: msg.to_string(), line };
    while i < n {
        if at_line_start && depth == 0 {
            // measure indentation
            let mut col = 0usize;
            let mut j = i;
            while j < n && (chars[j] == ' ' || chars[j] == '\t' || chars[j] == '\x0c') {
                if chars[j] == '\t' {
                    col = (col / 8 + 1) * 8;
                } else if chars[j] == ' ' {
                    col += 1;
                }
                j += 1;
            }
            if j >= n {
                i = j;
                break;
            }
            let c = chars[j];
            if c == '\n' || c == '#' || c == '\r' || (c == '\\' && j + 1 < n && chars[j + 1] == '\n') {
                // blank or comment line
                if c == '#' {
                    while j < n && chars[j] != '\n' {
                        j += 1;
                    }
                }
                if c == '\\' {
                    j += 1;
                }
                if j < n && chars[j] == '\r' {
                    j += 1;
                }
                if j < n && chars[j] == '\n' {
                    j += 1;
                    line += 1;
                }
                i = j;
                continue;
            }
            let cur = *indents.last().unwrap();
            if col > cur {
                indents.push(col);
                toks.push(Token { tok: Tok::Indent, line });
            } else {
                while col < *indents.last().unwrap() {
                    indents.pop();
                    toks.push(Token { tok: Tok::Dedent, line });
                }
                if col != *indents.last().unwrap() {
                    return Err(err("unindent does not match any outer indentation level", line));
                }
            }
            i = j;
            at_line_start = false;
        }
        let c = chars[i];
        if c == '\n' {
            if depth == 0 {
                if let Some(t) = toks.last() {
                    if t.tok != Tok::Newline {
                        toks.push(Token { tok: Tok::Newline, line });
                    }
                }
                at_line_start = true;
            }
            line += 1;
            i += 1;
            continue;
        }
        if c == ' ' || c == '\t' || c == '\r' || c == '\x0c' {
            i += 1;
            continue;
        }
        if c == '#' {
            while i < n && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }
        if c == '\\' && i + 1 < n && (chars[i + 1] == '\n' || chars[i + 1] == '\r') {
            i += 1;
            if chars[i] == '\r' {
                i += 1;
            }
            if i < n && chars[i] == '\n' {
                i += 1;
            }
            line += 1;
            continue;
        }
        // string prefixes
        if c.is_alphabetic() || c == '_' {
            let start = i;
            while i < n && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            if i < n && (chars[i] == '\'' || chars[i] == '"') {
                let lw = word.to_ascii_lowercase();
                if lw.len() <= 2 && lw.chars().all(|ch| "rbfu".contains(ch)) {
                    let raw = lw.contains('r');
                    let bytes = lw.contains('b');
                    let f = lw.contains('f');
                    let (body, ni, nl) = read_string(&chars, i, line, raw || f, f)?;
                    i = ni;
                    line = nl;
                    let s = if raw || f { body } else { unescape(&body, bytes).map_err(|m| err(&m, line))? };
                    // the parser unescapes an f-string's literal text; a raw
                    // one keeps its backslashes through that
                    let s = if f && raw { s.replace('\\', "\\\\") } else { s };
                    toks.push(Token { tok: Tok::Str { s, bytes, f }, line });
                    continue;
                }
            }
            toks.push(Token { tok: Tok::Name(Rc::from(word.as_str())), line });
            continue;
        }
        if c == '\'' || c == '"' {
            let (body, ni, nl) = read_string(&chars, i, line, false, false)?;
            let s = unescape(&body, false).map_err(|m| err(&m, line))?;
            toks.push(Token { tok: Tok::Str { s, bytes: false, f: false }, line });
            i = ni;
            line = nl;
            continue;
        }
        if c.is_ascii_digit() || (c == '.' && i + 1 < n && chars[i + 1].is_ascii_digit()) {
            let (t, ni) = read_number(&chars, i).map_err(|m| err(&m, line))?;
            toks.push(Token { tok: t, line });
            i = ni;
            continue;
        }
        // operators
        let mut matched = false;
        for len in [3usize, 2] {
            if i + len <= n {
                let s: String = chars[i..i + len].iter().collect();
                let o = op_str(&s);
                if o != "?" && o.len() == len {
                    toks.push(Token { tok: Tok::Op(if o == "<>" { "!=" } else { o }), line });
                    i += len;
                    matched = true;
                    break;
                }
            }
        }
        if matched {
            continue;
        }
        if OPS1.contains(c) {
            let o = op_str(&c.to_string());
            match o {
                "(" | "[" | "{" => depth += 1,
                ")" | "]" | "}" => depth -= 1,
                _ => {}
            }
            toks.push(Token { tok: Tok::Op(o), line });
            i += 1;
            continue;
        }
        return Err(err(&format!("invalid character '{}' (U+{:04X})", c, c as u32), line));
    }
    if let Some(t) = toks.last() {
        if t.tok != Tok::Newline && t.tok != Tok::Dedent {
            toks.push(Token { tok: Tok::Newline, line });
        }
    }
    while indents.len() > 1 {
        indents.pop();
        toks.push(Token { tok: Tok::Dedent, line });
    }
    toks.push(Token { tok: Tok::Eof, line });
    Ok(toks)
}

/// Reads a quoted string starting at `i` (the quote). Returns the raw body,
/// the index after the closing quote and the line after it.
fn read_string(chars: &[char], mut i: usize, mut line: u32, _raw: bool, fstr: bool) -> Result<(String, usize, u32), LexError> {
    let q = chars[i];
    let n = chars.len();
    let triple = i + 2 < n && chars[i + 1] == q && chars[i + 2] == q;
    i += if triple { 3 } else { 1 };
    let mut body = String::new();
    let mut brace = 0i32;
    loop {
        if i >= n {
            return Err(LexError {
                msg: if triple { "unterminated triple-quoted string literal".into() } else { "unterminated string literal".into() },
                line,
            });
        }
        let c = chars[i];
        if fstr && brace > 0 && (c == '\'' || c == '"') {
            // a string nested in an f-string expression, copied as written
            let iq = c;
            let it = i + 2 < n && chars[i + 1] == iq && chars[i + 2] == iq;
            let ql = if it { 3 } else { 1 };
            for _ in 0..ql {
                body.push(iq);
            }
            i += ql;
            loop {
                if i >= n {
                    return Err(LexError { msg: "unterminated string literal".into(), line });
                }
                if chars[i] == '\\' && i + 1 < n {
                    body.push(chars[i]);
                    body.push(chars[i + 1]);
                    i += 2;
                    continue;
                }
                if chars[i] == iq && (!it || (i + 2 < n && chars[i + 1] == iq && chars[i + 2] == iq)) {
                    for _ in 0..ql {
                        body.push(iq);
                    }
                    i += ql;
                    break;
                }
                if chars[i] == '\n' {
                    line += 1;
                }
                body.push(chars[i]);
                i += 1;
            }
            continue;
        }
        if c == '\\' && i + 1 < n {
            body.push(c);
            body.push(chars[i + 1]);
            if chars[i + 1] == '\n' {
                line += 1;
            }
            i += 2;
            continue;
        }
        if c == q {
            if triple {
                if i + 2 < n && chars[i + 1] == q && chars[i + 2] == q {
                    return Ok((body, i + 3, line));
                }
            } else {
                return Ok((body, i + 1, line));
            }
        }
        if c == '\n' {
            if !triple {
                return Err(LexError { msg: "unterminated string literal".into(), line });
            }
            line += 1;
        }
        if fstr {
            if c == '{' {
                if brace == 0 && i + 1 < n && chars[i + 1] == '{' {
                    body.push_str("{{");
                    i += 2;
                    continue;
                }
                brace += 1;
            } else if c == '}' && brace > 0 {
                brace -= 1;
            }
        }
        body.push(c);
        i += 1;
    }
}

pub fn unescape(s: &str, bytes: bool) -> Result<String, String> {
    if !s.contains('\\') {
        return Ok(s.to_string());
    }
    let cs: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < cs.len() {
        let c = cs[i];
        if c != '\\' || i + 1 >= cs.len() {
            out.push(c);
            i += 1;
            continue;
        }
        let e = cs[i + 1];
        i += 2;
        match e {
            '\n' => {}
            '\\' => out.push('\\'),
            '\'' => out.push('\''),
            '"' => out.push('"'),
            'a' => out.push('\x07'),
            'b' => out.push('\x08'),
            'f' => out.push('\x0c'),
            'n' => out.push('\n'),
            'r' => out.push('\r'),
            't' => out.push('\t'),
            'v' => out.push('\x0b'),
            '0'..='7' => {
                let mut v = e.to_digit(8).unwrap();
                let mut k = 0;
                while k < 2 && i < cs.len() && cs[i].is_digit(8) {
                    v = v * 8 + cs[i].to_digit(8).unwrap();
                    i += 1;
                    k += 1;
                }
                out.push(char::from_u32(v).unwrap_or('\u{fffd}'));
            }
            'x' | 'u' | 'U' if !(bytes && e != 'x') => {
                let len = match e {
                    'x' => 2,
                    'u' => 4,
                    _ => 8,
                };
                if i + len > cs.len() {
                    return Err(format!("truncated \\{} escape", e));
                }
                let hex: String = cs[i..i + len].iter().collect();
                let v = u32::from_str_radix(&hex, 16).map_err(|_| format!("truncated \\{} escape", e))?;
                out.push(char::from_u32(v).unwrap_or('\u{fffd}'));
                i += len;
            }
            'N' if !bytes && i < cs.len() && cs[i] == '{' => {
                let mut j = i + 1;
                while j < cs.len() && cs[j] != '}' {
                    j += 1;
                }
                let name: String = cs[i + 1..j].iter().collect();
                out.push(crate::unicode_name(&name).ok_or_else(|| "unknown Unicode character name".to_string())?);
                i = j + 1;
            }
            _ => {
                out.push('\\');
                out.push(e);
            }
        }
    }
    Ok(out)
}

fn read_number(chars: &[char], start: usize) -> Result<(Tok, usize), String> {
    let n = chars.len();
    let mut i = start;
    if chars[i] == '0' && i + 1 < n && "xXoObB".contains(chars[i + 1]) {
        let radix = match chars[i + 1] {
            'x' | 'X' => 16,
            'o' | 'O' => 8,
            _ => 2,
        };
        i += 2;
        let s0 = i;
        while i < n && (chars[i].is_digit(radix) || chars[i] == '_') {
            i += 1;
        }
        let digits: String = chars[s0..i].iter().filter(|c| **c != '_').collect();
        return match i64::from_str_radix(&digits, radix) {
            Ok(v) => Ok((Tok::Int(v), i)),
            Err(_) => {
                let big = crate::bigint::BigInt::from_radix(&digits, radix).ok_or("invalid digit")?;
                Ok((Tok::BigInt(big.to_string()), i))
            }
        };
    }
    let mut is_float = false;
    while i < n && (chars[i].is_ascii_digit() || chars[i] == '_') {
        i += 1;
    }
    if i < n && chars[i] == '.' {
        is_float = true;
        i += 1;
        while i < n && (chars[i].is_ascii_digit() || chars[i] == '_') {
            i += 1;
        }
    }
    if i < n && (chars[i] == 'e' || chars[i] == 'E') {
        let save = i;
        i += 1;
        if i < n && (chars[i] == '+' || chars[i] == '-') {
            i += 1;
        }
        if i < n && chars[i].is_ascii_digit() {
            is_float = true;
            while i < n && chars[i].is_ascii_digit() {
                i += 1;
            }
        } else {
            i = save;
        }
    }
    let text: String = chars[start..i].iter().filter(|c| **c != '_').collect();
    if i < n && (chars[i] == 'j' || chars[i] == 'J') {
        return Ok((Tok::Imag(text.parse::<f64>().map_err(|_| "invalid number")?), i + 1));
    }
    if is_float {
        Ok((Tok::Float(text.parse::<f64>().map_err(|_| "invalid number")?), i))
    } else {
        if text.len() > 1 && text.starts_with('0') && text.chars().any(|c| c != '0') {
            return Err("leading zeros in decimal integer literals are not permitted; use an 0o prefix for octal integers".into());
        }
        match text.parse::<i64>() {
            Ok(v) => Ok((Tok::Int(v), i)),
            Err(_) => Ok((Tok::BigInt(text), i)),
        }
    }
}
