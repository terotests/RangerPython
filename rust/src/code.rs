// SPDX-License-Identifier: MIT
//! Bytecode: the instruction set and compiled code objects.

use crate::ast::{BinOp, CmpOp, UnOp};
use crate::value::{Name, PyStr, Value};
use std::rc::Rc;

#[derive(Clone, Copy, Debug)]
pub enum Op {
    Nop,
    LoadConst(u32),
    LoadFast(u32),
    StoreFast(u32),
    DelFast(u32),
    LoadDeref(u32),
    StoreDeref(u32),
    DelDeref(u32),
    /// class body: the class namespace first, then the enclosing cell
    LoadClassDeref(u32),
    LoadGlobal(u32),
    StoreGlobal(u32),
    DelGlobal(u32),
    LoadName(u32),
    StoreName(u32),
    DelName(u32),
    LoadAttr(u32),
    StoreAttr(u32),
    DelAttr(u32),
    /// pushes [callable, self-or-Undef]
    LoadMethod(u32),
    CallMethod(u32),
    Subscr,
    StoreSubscr,
    DelSubscr,
    BuildSlice(u8),
    Binary(BinOp),
    Inplace(BinOp),
    Unary(UnOp),
    Compare(CmpOp),
    Pop,
    Dup,
    Dup2,
    Rot2,
    Rot3,
    Rot4,
    Jump(u32),
    PopJumpIfFalse(u32),
    PopJumpIfTrue(u32),
    JumpIfFalseOrPop(u32),
    JumpIfTrueOrPop(u32),
    /// pops and jumps when the top is not None
    PopJumpIfNotNone(u32),
    BuildList(u32),
    BuildTuple(u32),
    BuildSet(u32),
    BuildMap(u32),
    BuildString(u32),
    ListAppend(u32),
    SetAdd(u32),
    MapAdd(u32),
    ListExtend(u32),
    SetUpdate(u32),
    DictUpdate(u32),
    /// like DictUpdate, but a repeated keyword is an error (f(**a, **b))
    DictMerge(u32),
    ListToTuple,
    FormatValue(u8, bool),
    GetIter,
    /// the iterator `await x` drives: x itself for a coroutine, else x.__await__()
    GetAwaitable,
    ForIter(u32),
    Unpack(u32),
    UnpackEx(u16, u16),
    Call(u32),
    /// top of stack is a tuple of keyword names
    CallKw(u32),
    /// [func, args tuple, kwargs dict or Undef]
    CallEx,
    MakeFunction(u32, u8),
    LoadClosure(u32),
    Return,
    Yield,
    YieldFrom,
    SetupExcept(u32),
    SetupFinally(u32),
    SetupWith(u32),
    PopBlock,
    PopExcept,
    Reraise,
    Raise(u8),
    ExcMatch,
    /// [exit, exc]: call __exit__ with the exception
    WithExcept,
    /// call the __exit__ found `n` below the top with no exception
    WithExit(u32),
    ImportName(u32),
    ImportFrom(u32),
    ImportStar,
    LoadBuildClass,
    SetupAnnotations,
    LoadAssertionError,
    /// [subject, values tuple] -> captures tuple or None; operand is the
    /// pattern constant
    MatchPattern(u32),
    /// except* support: [exc, type] -> [rest, matched-or-None]
    ExcStarMatch,
    /// [original, rest, raised list] -> raises or continues
    ExcStarFinish,
    /// [list] append the exception on top to the list below
    ExcStarCollect,
    /// pushes the exception being handled
    LoadExc,
}

#[derive(Clone)]
pub struct Code {
    pub name: Rc<PyStr>,
    pub qualname: Rc<PyStr>,
    pub filename: Rc<str>,
    pub firstline: u32,
    pub argcount: u32,
    pub posonly: u32,
    pub kwonly: u32,
    pub varargs: bool,
    pub varkw: bool,
    pub generator: bool,
    pub coroutine: bool,
    /// CO_ITERABLE_COROUTINE: a generator types.coroutine made awaitable
    pub iterable_coroutine: bool,
    pub is_class: bool,
    pub varnames: Vec<Name>,
    /// cells this code creates, then the free variables it receives
    pub cellvars: Vec<Name>,
    pub freevars: Vec<Name>,
    /// for each cell var: the argument slot it starts from
    pub cell_from_arg: Vec<Option<u32>>,
    pub ops: Vec<Op>,
    pub lines: Vec<u32>,
    pub consts: Vec<Value>,
    pub names: Vec<Rc<PyStr>>,
    pub doc: Value,
    /// fast argument binding: no defaults/kwonly/varargs/varkw/cells
    pub simple: bool,
}

impl Code {
    pub fn ncells(&self) -> usize {
        self.cellvars.len() + self.freevars.len()
    }
}

// co_linetable in CPython's location table format, taking each op here as
// one code unit; a line of 0 is "no location"

/// entries of "line, no columns" (code 13) and "no location" (code 15)
pub fn encode_linetable(c: &Code) -> Vec<u8> {
    let mut out = vec![];
    let mut prev = c.firstline as i64;
    let mut i = 0;
    while i < c.lines.len() {
        let line = c.lines[i];
        let mut n = 1;
        while n < 8 && i + n < c.lines.len() && c.lines[i + n] == line {
            n += 1;
        }
        if line == 0 {
            out.push(0x80 | (15 << 3) | (n as u8 - 1));
        } else {
            out.push(0x80 | (13 << 3) | (n as u8 - 1));
            let d = line as i64 - prev;
            let mut v = if d < 0 { ((-d as u64) << 1) | 1 } else { (d as u64) << 1 };
            while v >= 64 {
                out.push(0x40 | (v & 63) as u8);
                v >>= 6;
            }
            out.push(v as u8);
            prev = line as i64;
        }
        i += n;
    }
    out
}

/// the line of each op from a location table; ops past its end have none
pub fn decode_linetable(table: &[u8], firstline: u32, nops: usize) -> Vec<u32> {
    let mut lines = Vec::with_capacity(nops);
    let mut line = firstline as i64;
    let mut i = 0;
    let varint = |i: &mut usize| -> u64 {
        let (mut v, mut shift) = (0u64, 0);
        while *i < table.len() {
            let b = table[*i];
            *i += 1;
            v |= ((b & 63) as u64) << shift;
            shift += 6;
            if b & 64 == 0 {
                break;
            }
        }
        v
    };
    let svarint = |i: &mut usize| -> i64 {
        let v = varint(i);
        if v & 1 != 0 { -((v >> 1) as i64) } else { (v >> 1) as i64 }
    };
    while i < table.len() {
        let b = table[i];
        i += 1;
        let (code, len) = ((b >> 3) & 15, (b & 7) as usize + 1);
        let here = match code {
            15 => None,
            13 => {
                line += svarint(&mut i);
                Some(line)
            }
            14 => {
                line += svarint(&mut i);
                for _ in 0..3 {
                    varint(&mut i);
                }
                Some(line)
            }
            10..=12 => {
                line += (code - 10) as i64;
                i += 2;
                Some(line)
            }
            _ => {
                i += 1;
                Some(line)
            }
        };
        for _ in 0..len {
            lines.push(here.map_or(0, |l| l.max(0) as u32));
        }
    }
    lines.resize(nops, 0);
    lines
}
