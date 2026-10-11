// SPDX-License-Identifier: MIT
//! Tokenizer: Python source to a token list with INDENT/DEDENT. F-strings
//! are tokenized as in PEP 701: FStart, literal FMiddle pieces and the
//! tokens of each replacement field, then FEnd.

use crate::wtf8::Wtf8Buf;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub enum Tok {
    Name(Rc<str>),
    Int(i64),
    /// an integer literal too large for i64, kept as its digits
    BigInt(String),
    Float(f64),
    Imag(f64),
    /// a string or bytes literal, escapes decoded (bytes as chars < 256)
    Str { s: Wtf8Buf, bytes: bool },
    /// the start and end of an f-string
    FStart,
    FEnd,
    /// literal text of an f-string or of a format spec, escapes decoded
    FMiddle(Wtf8Buf),
    /// the `:` that starts a replacement field's format spec
    FSpec,
    /// a self-documenting `=`, holding the field's source up to and after it
    FDebug(String),
    Op(&'static str),
    Newline,
    Indent,
    Dedent,
    /// a tokenizer error; the parser raises it when it gets this far
    Error(Rc<str>),
    Eof,
}

#[derive(Clone, Debug)]
pub struct Token {
    pub tok: Tok,
    pub line: u32,
    /// 1-based column
    pub col: u32,
    /// where the token ends: the line, and the 1-based column after it
    pub end_line: u32,
    pub end_col: u32,
    /// the start and end columns as 0-based UTF-8 byte offsets, as the ast has them
    pub bcol: u32,
    pub end_bcol: u32,
}

pub struct LexError {
    pub msg: String,
    pub line: u32,
    /// 1-based column, 0 when unknown
    pub col: u32,
    /// the column after the bad part, on the error's own line; 0 when only its start is known
    pub end_col: u32,
}

/// a SyntaxWarning found while tokenizing or compiling
pub struct SyntaxWarn {
    pub msg: String,
    pub line: u32,
    pub col: u32,
}

crate::gil_static! {
    static WARNINGS: RefCell<Vec<SyntaxWarn>> = const { RefCell::new(Vec::new()) };
}

/// records a SyntaxWarning for whoever compiles the source to emit
pub fn push_warning(msg: String, line: u32, col: u32) {
    WARNINGS.with(|w| w.borrow_mut().push(SyntaxWarn { msg, line, col }));
}

/// the SyntaxWarnings recorded since the last call
pub fn take_warnings() -> Vec<SyntaxWarn> {
    WARNINGS.with(|w| std::mem::take(&mut *w.borrow_mut()))
}

const OPS3: [&str; 6] = ["**=", "//=", ">>=", "<<=", "...", "!="];
const OPS2: [&str; 22] = [
    "**", "//", "==", "!=", "<=", ">=", "<<", ">>", "+=", "-=", "*=", "/=", "%=", "&=", "|=", "^=", "->", ":=", "@=",
    "<>", "!=", "==",
];
/// single-character operators, and the ASCII characters that are no
/// operator but still tokens the parser rejects
const ONE: [&str; 27] = [
    "+", "-", "*", "/", "%", "@", "&", "|", "^", "~", "<", ">", "(", ")", "[", "]", "{", "}", ",", ":", ".", ";", "=", "!", "$", "?", "`",
];

fn op_str(s: &str) -> Option<&'static str> {
    OPS3.iter().chain(OPS2.iter()).chain(ONE.iter()).find(|o| **o == s).copied()
}

/// the valid string prefixes, lowercased
fn string_prefix(w: &str) -> bool {
    matches!(w, "r" | "u" | "b" | "f" | "br" | "rb" | "fr" | "rf")
}

/// what CPython calls a non-printable character in source
fn non_printable(c: char) -> bool {
    c.is_control() || c.is_whitespace() || matches!(c, '\u{ad}' | '\u{200b}'..='\u{200f}' | '\u{2028}'..='\u{202e}' | '\u{2060}'..='\u{2064}' | '\u{feff}')
}

/// an f-string being tokenized
struct FStr {
    quote: char,
    triple: bool,
    raw: bool,
    line: u32,
    col: u32,
    /// the replacement fields open in it, innermost last
    fields: Vec<Field>,
}

struct Field {
    /// the length of `opens` with this field's `{` on top
    base: usize,
    /// whether the format spec has started
    spec: bool,
    /// the index after the `{`, for a self-documenting field's text
    start: usize,
}

/// f-strings nested deeper than this are refused, as by CPython
const MAX_FSTRINGS: usize = 150;
/// and brackets nested deeper than this
const MAX_PARENS: usize = 200;

struct Lexer {
    chars: Vec<char>,
    i: usize,
    line: u32,
    /// the index the current line starts at
    line_start: usize,
    toks: Vec<Token>,
    indents: Vec<usize>,
    /// the same widths with a tab counted as one column: TabError when they disagree
    alts: Vec<usize>,
    /// open brackets: which, and the line and column they opened at
    opens: Vec<(&'static str, u32, u32)>,
    at_line_start: bool,
    fstrs: Vec<FStr>,
}

/// Tokenizes `src`. A tokenizer error ends the list as an Error token, so
/// that a syntax error the parser finds before it is reported first.
pub fn tokenize(src: &str) -> Vec<Token> {
    let src = if src.contains('\r') { src.replace("\r\n", "\n").replace('\r', "\n") } else { src.to_string() };
    let mut lx = Lexer {
        chars: src.chars().collect(),
        i: 0,
        line: 1,
        line_start: 0,
        toks: vec![],
        indents: vec![0],
        alts: vec![0],
        opens: vec![],
        at_line_start: true,
        fstrs: vec![],
    };
    if let Err(e) = lx.run() {
        lx.push(Tok::Error(Rc::from(e.msg.as_str())), e.line, e.col);
        if e.end_col > 0 {
            lx.toks.last_mut().unwrap().end_col = e.end_col;
        }
        lx.push(Tok::Eof, e.line, e.col);
    }
    lx.byte_columns();
    lx.toks
}

impl Lexer {
    fn col(&self) -> u32 {
        (self.i - self.line_start + 1) as u32
    }

    fn err(&self, msg: &str) -> LexError {
        LexError { msg: msg.to_string(), line: self.line, col: self.col(), end_col: 0 }
    }

    /// an error whose offset is the character before index `at`, as CPython's
    /// tokenizer reports one with the bad character put back
    fn err_before(&self, msg: &str, at: usize) -> LexError {
        LexError { msg: msg.to_string(), line: self.line, col: (at - self.line_start) as u32, end_col: 0 }
    }

    fn push(&mut self, tok: Tok, line: u32, col: u32) {
        // set_ends gives the real end; a newline or the end is one character
        let end_col = if matches!(tok, Tok::Newline | Tok::Eof) { col + 1 } else { col };
        self.toks.push(Token { tok, line, col, end_line: line, end_col, bcol: 0, end_bcol: 0 });
    }

    /// sets each token's byte columns from its character columns
    fn byte_columns(&mut self) {
        if self.chars.iter().all(char::is_ascii) {
            for t in &mut self.toks {
                (t.bcol, t.end_bcol) = (t.col.saturating_sub(1), t.end_col.saturating_sub(1));
            }
            return;
        }
        // the byte offset of each character, and where each line starts
        let mut offs = Vec::with_capacity(self.chars.len() + 1);
        let mut starts = vec![0];
        let mut b = 0u32;
        for (i, c) in self.chars.iter().enumerate() {
            offs.push(b);
            b += c.len_utf8() as u32;
            if *c == '\n' {
                starts.push(i + 1);
            }
        }
        offs.push(b);
        let at = |line: u32, col: u32| {
            let start = starts.get((line as usize).saturating_sub(1)).copied().unwrap_or(0);
            let i = (start + (col as usize).saturating_sub(1)).min(offs.len() - 1);
            offs[i] - offs[start.min(offs.len() - 1)]
        };
        for t in &mut self.toks {
            (t.bcol, t.end_bcol) = (at(t.line, t.col), at(t.end_line, t.end_col));
        }
    }

    /// Ends the tokens from `first` on, pushed by one step: each where the
    /// next begins, the last where the step stopped.
    fn set_ends(&mut self, first: usize) {
        let (line, col) = (self.line, self.col());
        for k in first..self.toks.len() {
            let (l, c) = self.toks.get(k + 1).map_or((line, col), |t| (t.line, t.col));
            let t = &mut self.toks[k];
            // a newline, or the end, is one character wide on its own line
            (t.end_line, t.end_col) = if matches!(t.tok, Tok::Newline | Tok::Eof) { (t.line, t.col + 1) } else { (l, c) };
        }
    }

    /// steps over a newline at `i`
    fn newline_at(&mut self, i: usize) {
        self.line += 1;
        self.line_start = i + 1;
    }

    fn peek_at(&self, k: usize) -> char {
        self.chars.get(self.i + k).copied().unwrap_or('\0')
    }

    /// whether the innermost f-string is in its literal text or a format spec
    fn in_fstring_text(&self) -> bool {
        self.fstrs.last().map_or(false, |f| f.fields.last().map_or(true, |fd| fd.spec))
    }

    /// the innermost replacement field, when its bracket is the innermost open one
    fn field_top(&self) -> Option<&Field> {
        let fd = self.fstrs.last()?.fields.last()?;
        (fd.base == self.opens.len()).then_some(fd)
    }

    fn run(&mut self) -> Result<(), LexError> {
        let n = self.chars.len();
        loop {
            let first = self.toks.len();
            if self.in_fstring_text() {
                self.fstring_text()?;
                self.set_ends(first);
                continue;
            }
            if self.i >= n {
                break;
            }
            if self.at_line_start && self.opens.is_empty() {
                if !self.indentation()? {
                    break;
                }
                continue;
            }
            self.token()?;
            self.set_ends(first);
        }
        if let Some((o, l, c)) = self.opens.last() {
            return Err(LexError { msg: format!("'{}' was never closed", o), line: *l, col: *c, end_col: 0 });
        }
        if let Some(t) = self.toks.last() {
            if t.tok != Tok::Newline && t.tok != Tok::Dedent {
                let (line, col) = (self.line, self.col());
                self.push(Tok::Newline, line, col);
            }
        }
        while self.indents.len() > 1 {
            self.indents.pop();
            let line = self.line;
            self.push(Tok::Dedent, line, 1);
        }
        let line = self.line;
        self.push(Tok::Eof, line, 1);
        Ok(())
    }

    /// Measures a line's indentation, skipping blank and comment lines.
    /// Returns false at the end of the source.
    fn indentation(&mut self) -> Result<bool, LexError> {
        let n = self.chars.len();
        let chars = &self.chars;
        let mut col = 0usize;
        let mut alt = 0usize;
        let mut j = self.i;
        while j < n && (chars[j] == ' ' || chars[j] == '\t' || chars[j] == '\x0c') {
            if chars[j] == '\t' {
                col = (col / 8 + 1) * 8;
                alt += 1;
            } else if chars[j] == ' ' {
                col += 1;
                alt += 1;
            } else {
                col = 0;
                alt = 0;
            }
            j += 1;
        }
        if j >= n {
            self.i = j;
            return Ok(false);
        }
        let c = chars[j];
        if c == '\n' || c == '#' || (c == '\\' && j + 1 < n && chars[j + 1] == '\n') {
            // blank or comment line
            if c == '#' {
                while j < n && chars[j] != '\n' {
                    j += 1;
                }
            }
            if c == '\\' {
                j += 1;
            }
            if j < n && chars[j] == '\n' {
                self.newline_at(j);
                j += 1;
            }
            self.i = j;
            return Ok(true);
        }
        self.i = j;
        let line = self.line;
        let cur = *self.indents.last().unwrap();
        let tab_err = "inconsistent use of tabs and spaces in indentation";
        if col > cur {
            if alt <= *self.alts.last().unwrap_or(&0) {
                return Err(self.err(tab_err));
            }
            self.indents.push(col);
            self.alts.push(alt);
            self.push(Tok::Indent, line, 1);
        } else {
            while col < *self.indents.last().unwrap() {
                self.indents.pop();
                self.alts.pop();
                self.push(Tok::Dedent, line, 1);
            }
            if col != *self.indents.last().unwrap() {
                // CPython puts this one after the end of the line
                let len = self.chars[self.line_start..].iter().take_while(|c| **c != '\n').count();
                return Err(LexError { msg: "unindent does not match any outer indentation level".into(), line: self.line, col: len as u32 + 1, end_col: 0 });
            }
            if alt != *self.alts.last().unwrap_or(&0) {
                return Err(self.err(tab_err));
            }
        }
        self.at_line_start = false;
        Ok(true)
    }

    /// one token of ordinary code (also inside a replacement field)
    fn token(&mut self) -> Result<(), LexError> {
        let n = self.chars.len();
        let c = self.chars[self.i];
        let (line, col) = (self.line, self.col());
        if c == '\n' {
            if self.opens.is_empty() {
                if let Some(t) = self.toks.last() {
                    if t.tok != Tok::Newline {
                        self.push(Tok::Newline, line, col);
                    }
                }
                self.at_line_start = true;
            }
            self.newline_at(self.i);
            self.i += 1;
            return Ok(());
        }
        if c == ' ' || c == '\t' || c == '\x0c' {
            self.i += 1;
            return Ok(());
        }
        if c == '#' {
            while self.i < n && self.chars[self.i] != '\n' {
                self.i += 1;
            }
            return Ok(());
        }
        if c == '\\' {
            if self.i + 1 == n && !self.opens.is_empty() {
                // the unclosed bracket is the error
                self.i += 1;
                return Ok(());
            }
            if self.peek_at(1) == '\n' {
                self.newline_at(self.i + 1);
                self.i += 2;
                return Ok(());
            }
            self.i += 1;
            return Err(self.err("unexpected character after line continuation"));
        }
        if c.is_alphabetic() || c == '_' {
            let start = self.i;
            while self.i < n && (self.chars[self.i].is_alphanumeric() || self.chars[self.i] == '_') {
                self.i += 1;
            }
            let word: String = self.chars[start..self.i].iter().collect();
            if self.i < n && (self.chars[self.i] == '\'' || self.chars[self.i] == '"') {
                let lw = word.to_ascii_lowercase();
                if string_prefix(&lw) {
                    return self.string(lw.contains('r'), lw.contains('b'), lw.contains('f'), line, col);
                }
            }
            self.push(Tok::Name(Rc::from(word.as_str())), line, col);
            return Ok(());
        }
        if c == '\'' || c == '"' {
            return self.string(false, false, false, line, col);
        }
        if c.is_ascii_digit() || (c == '.' && self.peek_at(1).is_ascii_digit()) {
            let (t, ni) = read_number(&self.chars, self.i).map_err(|(m, at)| self.err_before(&m, at))?;
            let kind = match (self.peek_at(1).to_ascii_lowercase(), &t) {
                (_, Tok::Imag(_)) => "imaginary",
                ('x', _) if c == '0' => "hexadecimal",
                ('o', _) if c == '0' => "octal",
                ('b', _) if c == '0' => "binary",
                _ => "decimal",
            };
            self.push(t, line, col);
            self.i = ni;
            // a letter right after a number: a keyword is only deprecated
            let after = |s: &str| self.chars[ni..].iter().take(s.len()).copied().eq(s.chars());
            if after("and") || after("else") || after("for") || after("if") || after("in") || after("is") || after("or") || after("not") {
                push_warning(format!("invalid {} literal", kind), line, col);
            } else if self.i < n && (self.chars[self.i].is_ascii_alphanumeric() || self.chars[self.i] == '_') {
                return Err(self.err_before(&format!("invalid {} literal", kind), self.i));
            }
            return Ok(());
        }
        // in a replacement field, ':' starts the format spec and a lone '=' is self-documenting
        if let Some(start) = self.field_top().map(|fd| fd.start) {
            if c == ':' {
                let fs = self.fstrs.last_mut().unwrap();
                fs.fields.last_mut().unwrap().spec = true;
                self.push(Tok::FSpec, line, col);
                self.i += 1;
                return Ok(());
            }
            if c == '=' && !matches!(self.peek_at(1), '=') {
                // the text shown keeps the whitespace after the '=', not comments
                let mut text: String = self.chars[start..=self.i].iter().collect();
                let mut j = self.i + 1;
                while j < n {
                    match self.chars[j] {
                        ' ' | '\t' | '\n' | '\x0c' => text.push(self.chars[j]),
                        '#' => {
                            while j + 1 < n && self.chars[j + 1] != '\n' {
                                j += 1;
                            }
                        }
                        _ => break,
                    }
                    j += 1;
                }
                self.push(Tok::FDebug(text), line, col);
                self.i += 1;
                return Ok(());
            }
        }
        for len in [3usize, 2, 1] {
            if self.i + len > n {
                continue;
            }
            let s: String = self.chars[self.i..self.i + len].iter().collect();
            let Some(o) = op_str(&s) else { continue };
            match o {
                "(" | "[" | "{" => {
                    if self.opens.len() >= MAX_PARENS {
                        return Err(self.err("too many nested parentheses"));
                    }
                    self.opens.push((o, line, col));
                }
                ")" | "]" | "}" => self.close_bracket(o)?,
                _ => {}
            }
            self.push(Tok::Op(if o == "<>" { "!=" } else { o }), line, col);
            self.i += len;
            return Ok(());
        }
        if non_printable(c) {
            return Err(self.err(&format!("invalid non-printable character U+{:04X}", c as u32)));
        }
        Err(self.err(&format!("invalid character '{}' (U+{:04X})", c, c as u32)))
    }

    /// checks a closing bracket against the open one; a `}` may end a replacement field
    fn close_bracket(&mut self, o: &'static str) -> Result<(), LexError> {
        let Some(&(open, oline, _)) = self.opens.last() else {
            return Err(self.err(&format!("unmatched '{}'", o)));
        };
        let field = self.field_top().is_some();
        if field && o != "}" {
            return Err(self.err(&format!("f-string: unmatched '{}'", o)));
        }
        let want = match open {
            "(" => ")",
            "[" => "]",
            _ => "}",
        };
        if want != o {
            let on = if oline != self.line { format!(" on line {}", oline) } else { String::new() };
            return Err(self.err(&format!("closing parenthesis '{}' does not match opening parenthesis '{}'{}", o, open, on)));
        }
        self.opens.pop();
        if field {
            self.fstrs.last_mut().unwrap().fields.pop();
        }
        Ok(())
    }

    /// a string literal whose prefix is read; `i` is at the quote
    fn string(&mut self, raw: bool, bytes: bool, f: bool, line: u32, col: u32) -> Result<(), LexError> {
        let n = self.chars.len();
        let q = self.chars[self.i];
        let triple = self.peek_at(1) == q && self.peek_at(2) == q;
        self.i += if triple { 3 } else { 1 };
        if f {
            if self.fstrs.len() >= MAX_FSTRINGS {
                return Err(LexError { msg: "too many nested f-strings".into(), line, col, end_col: 0 });
            }
            self.fstrs.push(FStr { quote: q, triple, raw, line, col, fields: vec![] });
            self.push(Tok::FStart, line, col);
            return Ok(());
        }
        let mut body = String::new();
        loop {
            if self.i >= n || (!triple && self.chars[self.i] == '\n') {
                // in a replacement field, the f-string's own quote means a missing '}'
                if let Some(fs) = self.fstrs.last() {
                    if fs.quote == q && fs.triple == triple {
                        return Err(LexError { msg: "f-string: expecting '}'".into(), line, col, end_col: 0 });
                    }
                }
                let msg = if triple { "unterminated triple-quoted string literal" } else { "unterminated string literal" };
                let msg = format!("{} (detected at line {})", msg, self.line);
                return Err(LexError { msg, line, col, end_col: 0 });
            }
            let c = self.chars[self.i];
            if c == '\\' && self.i + 1 < n {
                body.push(c);
                body.push(self.chars[self.i + 1]);
                if self.chars[self.i + 1] == '\n' {
                    self.newline_at(self.i + 1);
                }
                self.i += 2;
                continue;
            }
            if c == q && (!triple || (self.peek_at(1) == q && self.peek_at(2) == q)) {
                self.i += if triple { 3 } else { 1 };
                break;
            }
            if c == '\n' {
                self.newline_at(self.i);
            }
            body.push(c);
            self.i += 1;
        }
        if bytes {
            if body.chars().any(|c| !c.is_ascii()) {
                // the whole literal, when it is on one line
                let end_col = if self.line == line { self.col() } else { 0 };
                return Err(LexError { msg: "bytes can only contain ASCII literal characters".into(), line, col, end_col });
            }
        }
        let s = if raw { Wtf8Buf::from(body) } else { self.decode(&body, bytes, line, col)? };
        self.push(Tok::Str { s, bytes }, line, col);
        Ok(())
    }

    /// decodes a literal's escapes, recording an invalid escape's warning
    fn decode(&self, body: &str, bytes: bool, line: u32, col: u32) -> Result<Wtf8Buf, LexError> {
        let (s, warn) = unescape(body, bytes).map_err(|msg| LexError { msg, line, col, end_col: 0 })?;
        if let Some(w) = warn {
            push_warning(w, line, col);
        }
        Ok(s)
    }

    /// The literal text of the innermost f-string, or of the format spec of
    /// its innermost field, up to a replacement field's brace or the end.
    fn fstring_text(&mut self) -> Result<(), LexError> {
        let n = self.chars.len();
        let fs = self.fstrs.last().unwrap();
        let (q, triple, raw, fline, fcol) = (fs.quote, fs.triple, fs.raw, fs.line, fs.col);
        let in_spec = !fs.fields.is_empty();
        let nfields = fs.fields.len();
        let (line, col) = (self.line, self.col());
        // decoded text, and the text still to decode
        let mut out = Wtf8Buf::default();
        let mut pending = String::new();
        let flush = |lx: &Lexer, out: &mut Wtf8Buf, pending: &mut String| -> Result<(), LexError> {
            if raw {
                out.push_str(pending);
            } else if !pending.is_empty() {
                out.push_buf(&lx.decode(pending, false, line, col)?);
            }
            pending.clear();
            Ok(())
        };
        loop {
            if self.i >= n {
                let msg = if triple { "unterminated triple-quoted f-string literal" } else { "unterminated f-string literal" };
                return Err(LexError { msg: format!("{} (detected at line {})", msg, self.line), line: fline, col: fcol, end_col: 0 });
            }
            let c = self.chars[self.i];
            if c == q && (!triple || (self.peek_at(1) == q && self.peek_at(2) == q)) {
                if in_spec {
                    return Err(self.err("f-string: expecting '}'"));
                }
                flush(self, &mut out, &mut pending)?;
                if !out.is_empty() {
                    self.push(Tok::FMiddle(out), line, col);
                }
                let (l, cl) = (self.line, self.col());
                self.push(Tok::FEnd, l, cl);
                self.i += if triple { 3 } else { 1 };
                self.fstrs.pop();
                return Ok(());
            }
            if c == '\n' {
                if !triple {
                    let msg = format!("unterminated f-string literal (detected at line {})", self.line);
                    return Err(LexError { msg, line: fline, col: fcol, end_col: 0 });
                }
                self.newline_at(self.i);
                pending.push(c);
                self.i += 1;
                continue;
            }
            if c == '{' && !in_spec && self.peek_at(1) == '{' {
                flush(self, &mut out, &mut pending)?;
                out.push('{');
                self.i += 2;
                continue;
            }
            if c == '}' && !in_spec {
                if self.peek_at(1) != '}' {
                    return Err(self.err("f-string: single '}' is not allowed"));
                }
                flush(self, &mut out, &mut pending)?;
                out.push('}');
                self.i += 2;
                continue;
            }
            if c == '{' || c == '}' {
                flush(self, &mut out, &mut pending)?;
                if !out.is_empty() {
                    self.push(Tok::FMiddle(out), line, col);
                }
                let (l, cl) = (self.line, self.col());
                if c == '{' {
                    if in_spec && nfields >= 3 {
                        return Err(self.err("f-string: expressions nested too deeply"));
                    }
                    self.opens.push(("{", l, cl));
                    let base = self.opens.len();
                    self.fstrs.last_mut().unwrap().fields.push(Field { base, spec: false, start: self.i + 1 });
                } else {
                    // the end of the field whose format spec this is
                    self.opens.pop();
                    self.fstrs.last_mut().unwrap().fields.pop();
                }
                self.push(Tok::Op(if c == '{' { "{" } else { "}" }), l, cl);
                self.i += 1;
                return Ok(());
            }
            if c == '\\' {
                let nx = self.peek_at(1);
                if nx == '{' || nx == '}' {
                    // a backslash does not escape a brace
                    flush(self, &mut out, &mut pending)?;
                    out.push('\\');
                    if !raw {
                        push_warning(format!("invalid escape sequence '\\{}'", nx), line, col);
                    }
                    self.i += 1;
                    continue;
                }
                if !raw && nx == 'N' && self.peek_at(2) == '{' {
                    // \N{NAME} is an escape, not a replacement field
                    while self.i < n && self.chars[self.i] != '}' && self.chars[self.i] != q && self.chars[self.i] != '\n' {
                        pending.push(self.chars[self.i]);
                        self.i += 1;
                    }
                    if self.i < n && self.chars[self.i] == '}' {
                        pending.push('}');
                        self.i += 1;
                    }
                    continue;
                }
                if self.i + 1 < n {
                    pending.push(c);
                    pending.push(nx);
                    if nx == '\n' {
                        self.newline_at(self.i + 1);
                    }
                    self.i += 2;
                    continue;
                }
            }
            pending.push(c);
            self.i += 1;
        }
    }
}

/// a position in a literal's body for a decoding error: the byte offsets
fn span(offs: &[usize], from: usize, to: usize) -> String {
    format!("{}-{}", offs[from], offs[to.max(from + 1)] - 1)
}

/// Decodes the escapes of a string or bytes literal body (\uXXXX may be a
/// lone surrogate). Also returns the SyntaxWarning for the first invalid
/// escape sequence, if any.
pub fn unescape(s: &str, bytes: bool) -> Result<(Wtf8Buf, Option<String>), String> {
    if !s.contains('\\') {
        return Ok((Wtf8Buf::from(s.to_string()), None));
    }
    let cs: Vec<char> = s.chars().collect();
    let mut offs: Vec<usize> = s.char_indices().map(|(o, _)| o).collect();
    offs.push(s.len());
    let mut out = Wtf8Buf::from(String::with_capacity(s.len()));
    let mut warn = None;
    let err = |from: usize, to: usize, what: &str| {
        if bytes {
            format!("(value error) {} at position {}", what, offs[from])
        } else {
            format!("(unicode error) 'unicodeescape' codec can't decode bytes in position {}: {}", span(&offs, from, to), what)
        }
    };
    let mut i = 0;
    while i < cs.len() {
        let c = cs[i];
        if c != '\\' || i + 1 >= cs.len() {
            out.push(c);
            i += 1;
            continue;
        }
        let start = i;
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
                while i - start < 4 && i < cs.len() && cs[i].is_digit(8) {
                    v = v * 8 + cs[i].to_digit(8).unwrap();
                    i += 1;
                }
                if v > 0o377 && warn.is_none() {
                    let digits: String = cs[start + 1..i].iter().collect();
                    warn = Some(format!("invalid octal escape sequence '\\{}'", digits));
                }
                if bytes {
                    v &= 0xff;
                }
                out.push_cp(v);
            }
            'x' | 'u' | 'U' if !(bytes && e != 'x') => {
                let len = match e {
                    'x' => 2,
                    'u' => 4,
                    _ => 8,
                };
                let digits = cs[i..].iter().take(len).take_while(|c| c.is_ascii_hexdigit()).count();
                if digits < len {
                    let what = match e {
                        'x' if bytes => "invalid \\x escape".to_string(),
                        'x' => "truncated \\xXX escape".to_string(),
                        'u' => "truncated \\uXXXX escape".to_string(),
                        _ => "truncated \\UXXXXXXXX escape".to_string(),
                    };
                    return Err(err(start, i + digits, &what));
                }
                let hex: String = cs[i..i + len].iter().collect();
                let v = u32::from_str_radix(&hex, 16).unwrap_or(0);
                if v > 0x10ffff {
                    return Err(err(start, i + len, "illegal Unicode character"));
                }
                out.push_cp(v);
                i += len;
            }
            'N' if !bytes => {
                let close = if i < cs.len() && cs[i] == '{' { cs[i..].iter().position(|&c| c == '}').map(|p| i + p) } else { None };
                let Some(j) = close.filter(|&j| j > i + 1) else {
                    return Err(err(start, cs.len().min(i + 1), "malformed \\N character escape"));
                };
                let name: String = cs[i + 1..j].iter().collect();
                match crate::unicode_name(&name) {
                    Some(ch) => out.push(ch),
                    None => return Err(err(start, j + 1, "unknown Unicode character name")),
                }
                i = j + 1;
            }
            _ => {
                if warn.is_none() {
                    warn = Some(format!("invalid escape sequence '\\{}'", e));
                }
                out.push('\\');
                out.push(e);
            }
        }
    }
    Ok((out, warn))
}

/// digit (['_'] digit)*, from i; the index after it
/// a bad number literal: the message, and the index CPython's tokenizer
/// stands at, whose column is the error's offset
type NumError = (String, usize);

fn digit_run(chars: &[char], mut i: usize, radix: u32, what: &str) -> Result<usize, NumError> {
    let n = chars.len();
    if i >= n || !chars[i].is_digit(radix) {
        return Err((format!("invalid {} literal", what), i));
    }
    while i < n {
        if chars[i].is_digit(radix) {
            i += 1;
        } else if chars[i] == '_' {
            if i + 1 < n && chars[i + 1].is_digit(radix) {
                i += 1;
            } else {
                return Err((format!("invalid {} literal", what), i + 1));
            }
        } else {
            break;
        }
    }
    Ok(i)
}

fn read_number(chars: &[char], start: usize) -> Result<(Tok, usize), NumError> {
    let n = chars.len();
    let mut i = start;
    if chars[i] == '0' && i + 1 < n && "xXoObB".contains(chars[i + 1]) {
        let (radix, what) = match chars[i + 1] {
            'x' | 'X' => (16, "hexadecimal"),
            'o' | 'O' => (8, "octal"),
            _ => (2, "binary"),
        };
        i += 2;
        if i < n && chars[i] == '_' {
            i += 1;
        }
        let s0 = i;
        // a decimal digit past the radix's own
        let bad_digit = |i: usize| (format!("invalid digit '{}' in {} literal", chars[i], what), i + 1);
        if i < n && radix < 10 && chars[i].is_ascii_digit() && !chars[i].is_digit(radix) {
            return Err(bad_digit(i));
        }
        i = digit_run(chars, i, radix, what)?;
        if i < n && chars[i].is_ascii_digit() {
            return Err(bad_digit(i));
        }
        let digits: String = chars[s0..i].iter().filter(|c| **c != '_').collect();
        return match i64::from_str_radix(&digits, radix) {
            Ok(v) => Ok((Tok::Int(v), i)),
            Err(_) => {
                let big = crate::bigint::BigInt::from_radix(&digits, radix).ok_or(("invalid digit".to_string(), i))?;
                Ok((Tok::BigInt(big.to_string()), i))
            }
        };
    }
    let mut is_float = false;
    if chars[i] != '.' {
        i = digit_run(chars, i, 10, "decimal")?;
    }
    if i < n && chars[i] == '.' {
        is_float = true;
        i += 1;
        if i < n && chars[i].is_ascii_digit() {
            i = digit_run(chars, i, 10, "decimal")?;
        } else if i < n && chars[i] == '_' {
            return Err(("invalid decimal literal".into(), i));
        }
    }
    if i < n && (chars[i] == 'e' || chars[i] == 'E') {
        let save = i;
        i += 1;
        let signed = i < n && (chars[i] == '+' || chars[i] == '-');
        if signed {
            i += 1;
        }
        if i < n && chars[i].is_ascii_digit() {
            is_float = true;
            i = digit_run(chars, i, 10, "decimal")?;
        } else if signed || (i < n && chars[i] == '_') {
            // an exponent's sign must have digits after it
            return Err(("invalid decimal literal".into(), i));
        } else {
            i = save;
        }
    }
    let text: String = chars[start..i].iter().filter(|c| **c != '_').collect();
    if i < n && (chars[i] == 'j' || chars[i] == 'J') {
        return Ok((Tok::Imag(text.parse::<f64>().map_err(|_| ("invalid number".to_string(), i))?), i + 1));
    }
    if is_float {
        Ok((Tok::Float(text.parse::<f64>().map_err(|_| ("invalid number".to_string(), i))?), i))
    } else {
        if text.len() > 1 && text.starts_with('0') && text.chars().any(|c| c != '0') {
            return Err(("leading zeros in decimal integer literals are not permitted; use an 0o prefix for octal integers".into(), start + 1));
        }
        match text.parse::<i64>() {
            Ok(v) => Ok((Tok::Int(v), i)),
            Err(_) => Ok((Tok::BigInt(text), i)),
        }
    }
}
