// SPDX-License-Identifier: MIT
//! Bytecode: the instruction set and compiled code objects.

use crate::ast::{BinOp, CmpOp, Pos, UnOp};
use crate::value::{Name, PyStr, Value};
use std::cell::Cell;
use std::rc::Rc;

#[derive(Clone, Copy, Debug)]
pub enum Op {
    Nop,
    LoadConst(u32),
    LoadFast(u32),
    StoreFast(u32),
    DelFast(u32),
    /// pushes a fast local, bound or not, and unbinds it (an inlined
    /// comprehension saving its host's variable)
    LoadFastAndClear(u32),
    /// puts a new empty cell in a cell slot (an inlined comprehension's variable)
    MakeCell(u32),
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
    /// swaps the top of the stack with the item `n` - 1 below it
    Swap(u32),
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
    // Pairs fused into the first of them by fuse(), which leaves the
    // second where it was, for jumps to it: the pair runs as one and the
    // second is skipped
    /// LoadFast(a) LoadFast(b)
    LoadFast2(u16, u16),
    /// LoadFast(a) LoadConst(k)
    LoadFastConst(u16, u16),
    /// LoadFast(a) LoadAttr(n)
    LoadFastAttr(u16, u16),
    /// Compare(op) PopJumpIfFalse(t)
    CompareJumpIfFalse(crate::ast::CmpOp, u32),
    /// Compare(op) PopJumpIfTrue(t)
    CompareJumpIfTrue(crate::ast::CmpOp, u32),
}

/// fuses the common pairs of instructions (see Op::LoadFast2)
pub fn fuse(ops: &mut [Op]) {
    let small = |v: u32| v <= u16::MAX as u32;
    for i in 0..ops.len().saturating_sub(1) {
        ops[i] = match (ops[i], ops[i + 1]) {
            (Op::LoadFast(a), Op::LoadFast(b)) if small(a) && small(b) => Op::LoadFast2(a as u16, b as u16),
            (Op::LoadFast(a), Op::LoadConst(k)) if small(a) && small(k) => Op::LoadFastConst(a as u16, k as u16),
            (Op::LoadFast(a), Op::LoadAttr(n)) if small(a) && small(n) => Op::LoadFastAttr(a as u16, n as u16),
            (Op::Compare(c), Op::PopJumpIfFalse(t)) => Op::CompareJumpIfFalse(c, t),
            (Op::Compare(c), Op::PopJumpIfTrue(t)) => Op::CompareJumpIfTrue(c, t),
            (op, _) => op,
        };
    }
}

/// the instruction a fused one stands for (see fuse)
pub fn unfused(op: Op) -> Op {
    match op {
        Op::LoadFast2(a, _) | Op::LoadFastConst(a, _) | Op::LoadFastAttr(a, _) => Op::LoadFast(a as u32),
        Op::CompareJumpIfFalse(c, _) | Op::CompareJumpIfTrue(c, _) => Op::Compare(c),
        op => op,
    }
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
    /// the source span of each instruction
    pub positions: Vec<Pos>,
    pub consts: Vec<Value>,
    pub names: Vec<Rc<PyStr>>,
    /// for each name, where the attribute of that name was last found on
    /// an instance (CPython's LOAD_ATTR_INSTANCE_VALUE cache)
    pub attr_hints: Box<[Cell<AttrHint>]>,
    /// for each name, where LoadGlobal last found it
    pub global_hints: Box<[Cell<GlobalHint>]>,
    pub doc: Value,
    /// fast argument binding: no defaults/kwonly/varargs/varkw/cells
    pub simple: bool,
}

/// An attribute that the instances of a class keep inline, which nothing
/// on the class shadows: its slot among the class's shared keys, while no
/// class has gained or lost a name since (Class::lookup's version)
#[derive(Clone, Copy, Default)]
pub struct AttrHint {
    pub class: u64,
    pub version: u64,
    pub slot: u32,
}

/// Where a global was last found: entry `ix` of the globals (while that
/// entry still has the name as its key), or entry `ix` of the builtins
/// while the globals have the key stamp `stamp`, which they had when the
/// name was not among them
#[derive(Clone, Copy, Default)]
pub struct GlobalHint {
    pub stamp: u64,
    pub ix: u32,
    /// 0: nothing yet, 1: in the globals, 2: in the builtins
    pub kind: u8,
}

impl AttrHint {
    /// the slot of a class whose instances never keep the name inline
    pub const NONE: u32 = u32::MAX;
}

impl Code {
    pub fn ncells(&self) -> usize {
        self.cellvars.len() + self.freevars.len()
    }

    /// the line of the instruction at `ip`
    pub fn line_at(&self, ip: usize) -> Option<u32> {
        self.positions.get(ip).map(|p| p.line)
    }
}

// co_linetable in CPython's location table format, taking each op here as
// one code unit; a line of 0 is "no location"

/// a position without columns, as the location table can say
fn no_columns(p: &Pos) -> bool {
    (p.col, p.end_line, p.end_col) == (0, p.line, 0)
}

/// entries of "no location" (code 15), "line, no columns" (code 13) and the
/// long form (code 14)
pub fn encode_linetable(c: &Code) -> Vec<u8> {
    fn varint(out: &mut Vec<u8>, mut v: u64) {
        while v >= 64 {
            out.push(0x40 | (v & 63) as u8);
            v >>= 6;
        }
        out.push(v as u8);
    }
    fn svarint(out: &mut Vec<u8>, d: i64) {
        varint(out, if d < 0 { ((-d as u64) << 1) | 1 } else { (d as u64) << 1 });
    }
    let mut out = vec![];
    let mut prev = c.firstline as i64;
    let mut i = 0;
    while i < c.positions.len() {
        let p = c.positions[i];
        let mut n = 1;
        while n < 8 && i + n < c.positions.len() && c.positions[i + n] == p {
            n += 1;
        }
        if p.line == 0 {
            out.push(0x80 | (15 << 3) | (n as u8 - 1));
        } else {
            let code = if no_columns(&p) { 13 } else { 14 };
            out.push(0x80 | (code << 3) | (n as u8 - 1));
            svarint(&mut out, p.line as i64 - prev);
            if code == 14 {
                varint(&mut out, p.end_line.saturating_sub(p.line) as u64);
                varint(&mut out, p.col as u64 + 1);
                varint(&mut out, p.end_col as u64 + 1);
            }
            prev = p.line as i64;
        }
        i += n;
    }
    out
}

/// the position of each op from a location table; ops past its end have none
pub fn decode_linetable(table: &[u8], firstline: u32, nops: usize) -> Vec<Pos> {
    let mut positions = Vec::with_capacity(nops);
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
    let byte = |i: &mut usize| -> u32 {
        let b = table.get(*i).copied().unwrap_or(0);
        *i += 1;
        b as u32
    };
    while i < table.len() {
        let b = table[i];
        i += 1;
        let (code, len) = ((b >> 3) & 15, (b & 7) as usize + 1);
        let at = |line: i64| line.max(0) as u32;
        let here = match code {
            15 => Pos::default(),
            13 => {
                line += svarint(&mut i);
                Pos { line: at(line), col: 0, end_line: at(line), end_col: 0 }
            }
            14 => {
                line += svarint(&mut i);
                let end_line = at(line) + varint(&mut i) as u32;
                // columns are stored plus one, 0 being "unknown"
                let col = (varint(&mut i) as u32).saturating_sub(1);
                let end_col = (varint(&mut i) as u32).saturating_sub(1);
                Pos { line: at(line), col, end_line, end_col }
            }
            10..=12 => {
                line += (code - 10) as i64;
                let (col, end_col) = (byte(&mut i), byte(&mut i));
                Pos { line: at(line), col, end_line: at(line), end_col }
            }
            _ => {
                // the short form: a column in 0..80 and a width under 16
                let b = byte(&mut i);
                let col = code as u32 * 8 + (b >> 4);
                Pos { line: at(line), col, end_line: at(line), end_col: col + (b & 15) }
            }
        };
        for _ in 0..len {
            positions.push(here);
        }
    }
    positions.resize(nops, Pos::default());
    positions
}
