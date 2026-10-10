// SPDX-License-Identifier: MIT
//! Lone surrogates in str.
//!
//! A Python str is a sequence of code points that may include the
//! surrogates U+D800..U+DFFF, which a Rust String cannot hold. A str with
//! one keeps its code points as generalized UTF-8 (each surrogate encoded
//! on its own in three bytes, as 'surrogatepass' writes it) in `PyStr::wtf`,
//! and `PyStr::s` is the same text with U+FFFD, also three bytes, in place
//! of each surrogate. A byte offset or range so means the same in both, and
//! code working on `s` turns its results exact through these helpers. The
//! byte order of generalized UTF-8 is code point order.

use crate::value::{PyStr, Value};
use std::ops::Range;

#[inline]
pub fn is_surrogate(cp: u32) -> bool {
    (0xd800..0xe000).contains(&cp)
}

/// appends one code point, a surrogate too
#[inline]
pub fn push_cp(out: &mut Vec<u8>, cp: u32) {
    match char::from_u32(cp) {
        Some(c) => out.extend_from_slice(c.encode_utf8(&mut [0; 4]).as_bytes()),
        None => out.extend_from_slice(&[0xe0 | (cp >> 12) as u8, 0x80 | ((cp >> 6) & 0x3f) as u8, 0x80 | (cp & 0x3f) as u8]),
    }
}

/// the code point that starts at byte i, and its length in bytes
#[inline]
pub fn cp_at(b: &[u8], i: usize) -> (u32, usize) {
    let c = b[i] as u32;
    let cont = |k: usize| b[i + k] as u32 & 0x3f;
    match b[i] {
        0..=0x7f => (c, 1),
        0xc0..=0xdf => (((c & 0x1f) << 6) | cont(1), 2),
        0xe0..=0xef => (((c & 0x0f) << 12) | (cont(1) << 6) | cont(2), 3),
        _ => (((c & 0x07) << 18) | (cont(1) << 12) | (cont(2) << 6) | cont(3), 4),
    }
}

/// the code points of generalized UTF-8
pub struct CodePoints<'a> {
    b: &'a [u8],
    i: usize,
}

impl<'a> CodePoints<'a> {
    pub fn new(b: &'a [u8]) -> CodePoints<'a> {
        CodePoints { b, i: 0 }
    }
}

impl Iterator for CodePoints<'_> {
    type Item = u32;
    #[inline]
    fn next(&mut self) -> Option<u32> {
        if self.i >= self.b.len() {
            return None;
        }
        let (cp, n) = cp_at(self.b, self.i);
        self.i += n;
        Some(cp)
    }
}

impl DoubleEndedIterator for CodePoints<'_> {
    fn next_back(&mut self) -> Option<u32> {
        if self.i >= self.b.len() {
            return None;
        }
        let mut j = self.b.len() - 1;
        while self.b[j] & 0xc0 == 0x80 {
            j -= 1;
        }
        let (cp, _) = cp_at(self.b, j);
        self.b = &self.b[..j];
        Some(cp)
    }
}

impl PyStr {
    /// the str of generalized UTF-8 code points
    pub fn from_wtf8(b: Vec<u8>) -> PyStr {
        match String::from_utf8(b) {
            Ok(s) => PyStr::new(s),
            Err(e) => {
                let b = e.into_bytes();
                let s = CodePoints::new(&b).map(|cp| char::from_u32(cp).unwrap_or('\u{fffd}')).collect();
                let mut p = PyStr::new(s);
                p.wtf = Some(b.into_boxed_slice());
                p
            }
        }
    }

    pub fn from_code_points(cps: impl IntoIterator<Item = u32>) -> PyStr {
        let mut b = Vec::new();
        for cp in cps {
            push_cp(&mut b, cp);
        }
        PyStr::from_wtf8(b)
    }

    #[inline]
    pub fn has_surrogates(&self) -> bool {
        self.wtf.is_some()
    }

    /// the exact code points as generalized UTF-8
    #[inline]
    pub fn wtf8(&self) -> &[u8] {
        self.wtf.as_deref().unwrap_or(self.s.as_bytes())
    }

    #[inline]
    pub fn code_points(&self) -> CodePoints<'_> {
        CodePoints::new(self.wtf8())
    }

    /// the code point at byte i of `s`, and its length
    #[inline]
    pub fn cp_at(&self, i: usize) -> (u32, usize) {
        cp_at(self.wtf8(), i)
    }

    /// equal as Python strs
    #[inline]
    pub fn same(&self, o: &PyStr) -> bool {
        self.s == o.s && self.wtf == o.wtf
    }

    /// the str of bytes r of `s`
    pub fn sub(&self, r: Range<usize>) -> Value {
        match &self.wtf {
            None => Value::str(&self.s[r]),
            Some(b) => Value::from_wtf8(b[r].to_vec()),
        }
    }

    /// the str of p, a slice of `s`
    pub fn piece(&self, p: &str) -> Value {
        if self.wtf.is_none() {
            return Value::str(p);
        }
        let at = p.as_ptr() as usize - self.s.as_ptr() as usize;
        debug_assert!(at + p.len() <= self.s.len());
        self.sub(at..at + p.len())
    }

    pub fn concat(&self, o: &PyStr) -> Value {
        if self.wtf.is_none() && o.wtf.is_none() {
            let mut s = String::with_capacity(self.s.len() + o.s.len());
            s.push_str(&self.s);
            s.push_str(&o.s);
            return Value::string(s);
        }
        Value::from_wtf8([self.wtf8(), o.wtf8()].concat())
    }

    pub fn repeat(&self, n: usize) -> Value {
        match &self.wtf {
            None => Value::string(self.s.repeat(n)),
            Some(b) => Value::from_wtf8(b.repeat(n)),
        }
    }

    /// whether n occurs in self
    pub fn contains(&self, n: &PyStr) -> bool {
        if self.wtf.is_none() && n.wtf.is_none() {
            return self.s.contains(&*n.s);
        }
        find_bytes(self.wtf8(), n.wtf8(), false).is_some()
    }
}

impl Value {
    pub fn from_wtf8(b: Vec<u8>) -> Value {
        Value::Str(std::rc::Rc::new(PyStr::from_wtf8(b)))
    }

    /// the one-character str of a code point
    #[inline]
    pub fn from_cp(cp: u32) -> Value {
        match char::from_u32(cp) {
            Some(c) => crate::builtins::char_value(c),
            None => Value::Str(std::rc::Rc::new(PyStr::from_code_points([cp]))),
        }
    }
}

/// Builds a str from text and code points; also the text of a string
/// literal's token. It stays a String until a lone surrogate comes.
#[derive(Clone, Debug, PartialEq)]
pub enum Wtf8Buf {
    Text(String),
    Wtf(Vec<u8>),
}

impl Default for Wtf8Buf {
    fn default() -> Wtf8Buf {
        Wtf8Buf::Text(String::new())
    }
}

impl From<String> for Wtf8Buf {
    fn from(s: String) -> Wtf8Buf {
        Wtf8Buf::Text(s)
    }
}

impl Wtf8Buf {
    /// the bytes, to append generalized UTF-8 to
    fn wtf(&mut self) -> &mut Vec<u8> {
        if let Wtf8Buf::Text(s) = self {
            *self = Wtf8Buf::Wtf(std::mem::take(s).into_bytes());
        }
        match self {
            Wtf8Buf::Wtf(b) => b,
            Wtf8Buf::Text(_) => unreachable!(),
        }
    }
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Wtf8Buf::Text(s) => s.as_bytes(),
            Wtf8Buf::Wtf(b) => b,
        }
    }
    #[inline]
    pub fn push_str(&mut self, s: &str) {
        match self {
            Wtf8Buf::Text(t) => t.push_str(s),
            Wtf8Buf::Wtf(b) => b.extend_from_slice(s.as_bytes()),
        }
    }
    #[inline]
    pub fn push(&mut self, c: char) {
        match self {
            Wtf8Buf::Text(t) => t.push(c),
            Wtf8Buf::Wtf(b) => b.extend_from_slice(c.encode_utf8(&mut [0; 4]).as_bytes()),
        }
    }
    #[inline]
    pub fn push_cp(&mut self, cp: u32) {
        match char::from_u32(cp) {
            Some(c) => self.push(c),
            None => push_cp(self.wtf(), cp),
        }
    }
    /// appends generalized UTF-8
    pub fn push_wtf8(&mut self, b: &[u8]) {
        match self {
            Wtf8Buf::Wtf(v) => v.extend_from_slice(b),
            Wtf8Buf::Text(s) => match std::str::from_utf8(b) {
                Ok(text) => s.push_str(text),
                Err(_) => self.wtf().extend_from_slice(b),
            },
        }
    }
    #[inline]
    pub fn push_pystr(&mut self, p: &PyStr) {
        match &p.wtf {
            None => self.push_str(&p.s),
            Some(b) => self.wtf().extend_from_slice(b),
        }
    }
    pub fn push_buf(&mut self, b: &Wtf8Buf) {
        match b {
            Wtf8Buf::Text(s) => self.push_str(s),
            Wtf8Buf::Wtf(b) => self.wtf().extend_from_slice(b),
        }
    }
    pub fn clear(&mut self) {
        *self = Wtf8Buf::default();
    }
    pub fn is_empty(&self) -> bool {
        self.as_bytes().is_empty()
    }
    pub fn code_points(&self) -> CodePoints<'_> {
        CodePoints::new(self.as_bytes())
    }
    /// the text, with U+FFFD for each lone surrogate
    pub fn into_string_lossy(self) -> String {
        match self {
            Wtf8Buf::Text(s) => s,
            Wtf8Buf::Wtf(b) => PyStr::from_wtf8(b).s,
        }
    }
    pub fn to_pystr(&self) -> PyStr {
        match self {
            Wtf8Buf::Text(s) => PyStr::new(s.clone()),
            Wtf8Buf::Wtf(b) => PyStr::from_wtf8(b.clone()),
        }
    }
    #[inline]
    pub fn into_value(self) -> Value {
        match self {
            Wtf8Buf::Text(s) => Value::string(s),
            Wtf8Buf::Wtf(b) => Value::from_wtf8(b),
        }
    }
}

/// the offset of n's first occurrence in h, or its last when rev
pub fn find_bytes(h: &[u8], n: &[u8], rev: bool) -> Option<usize> {
    if n.len() > h.len() {
        return None;
    }
    let mut at = (0..=h.len() - n.len()).filter(|&i| h[i..].starts_with(n));
    if rev {
        at.next_back()
    } else {
        at.next()
    }
}

/// the offsets of the non-overlapping occurrences of n (not empty) in h,
/// matched from the left, or from the right when rev
pub fn occurrences(h: &[u8], n: &[u8], rev: bool) -> Vec<usize> {
    debug_assert!(!n.is_empty());
    let mut out = vec![];
    if rev {
        let mut end = h.len();
        while let Some(i) = find_bytes(&h[..end], n, true) {
            out.push(i);
            end = i;
        }
    } else {
        let mut at = 0;
        while let Some(i) = find_bytes(&h[at..], n, false) {
            out.push(at + i);
            at += i + n.len();
        }
    }
    out
}
