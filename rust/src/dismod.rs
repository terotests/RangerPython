// SPDX-License-Identifier: MIT
//! _rpydis: this runtime's own bytecode, laid out for the dis module
//! (pylib/dis.py). Each instruction gets an opname: CPython's where the
//! instruction does what CPython's of that name does, else one of its own
//! in the same style. Offsets count two bytes per instruction, as
//! co_lines() and co_positions() do.

use crate::ast::{BinOp, CmpOp, UnOp};
use crate::builtins::*;
use crate::code::{Code, Op};
use crate::modules::{module, put_fns};
use crate::value::*;
use crate::vm::Vm;
use std::rc::Rc;

/// every opname, in opcode order
const OPNAMES: &[&str] = &[
    "NOP", "LOAD_CONST", "LOAD_FAST", "STORE_FAST", "DELETE_FAST", "LOAD_FAST_AND_CLEAR", "MAKE_CELL",
    "LOAD_DEREF", "STORE_DEREF", "DELETE_DEREF", "LOAD_CLASSDEREF", "LOAD_GLOBAL", "STORE_GLOBAL",
    "DELETE_GLOBAL", "LOAD_NAME", "STORE_NAME", "DELETE_NAME", "LOAD_ATTR", "STORE_ATTR", "DELETE_ATTR",
    "LOAD_METHOD", "CALL_METHOD", "BINARY_SUBSCR", "STORE_SUBSCR", "DELETE_SUBSCR", "BUILD_SLICE",
    "BINARY_OP", "UNARY_NEGATIVE", "UNARY_POSITIVE", "UNARY_NOT", "UNARY_INVERT", "COMPARE_OP", "IS_OP",
    "CONTAINS_OP", "POP_TOP", "COPY", "DUP_TOP_TWO", "ROT_TWO", "ROT_THREE", "ROT_FOUR", "SWAP", "JUMP",
    "POP_JUMP_IF_FALSE", "POP_JUMP_IF_TRUE", "JUMP_IF_FALSE_OR_POP", "JUMP_IF_TRUE_OR_POP",
    "POP_JUMP_IF_NOT_NONE", "BUILD_LIST", "BUILD_TUPLE", "BUILD_SET", "BUILD_MAP", "BUILD_STRING",
    "LIST_APPEND", "SET_ADD", "MAP_ADD", "LIST_EXTEND", "SET_UPDATE", "DICT_UPDATE", "DICT_MERGE",
    "LIST_TO_TUPLE", "FORMAT_VALUE", "GET_ITER", "GET_AWAITABLE", "FOR_ITER", "UNPACK_SEQUENCE",
    "UNPACK_EX", "CALL", "CALL_KW", "CALL_FUNCTION_EX", "MAKE_FUNCTION", "LOAD_CLOSURE", "RETURN_VALUE",
    "YIELD_VALUE", "YIELD_FROM", "SETUP_EXCEPT", "SETUP_FINALLY", "SETUP_WITH", "POP_BLOCK", "POP_EXCEPT",
    "RERAISE", "RAISE_VARARGS", "CHECK_EXC_MATCH", "WITH_EXCEPT_START", "WITH_EXIT", "IMPORT_NAME",
    "IMPORT_FROM", "IMPORT_STAR", "LOAD_BUILD_CLASS", "SETUP_ANNOTATIONS", "LOAD_ASSERTION_ERROR",
    "MATCH_PATTERN", "CHECK_EG_MATCH", "EXC_STAR_FINISH", "EXC_STAR_COLLECT", "LOAD_EXC",
];

/// what an instruction's operand refers to
#[derive(Clone, Copy)]
enum Arg {
    None,
    /// a plain count or flag word
    Num,
    Const,
    Name,
    Local,
    /// a cell or free variable
    Cell,
    /// an instruction index
    Jump,
    Binary,
    Compare,
}

/// CPython's NB_* operand of BINARY_OP for `op`
fn nb(op: BinOp) -> (i64, &'static str) {
    match op {
        BinOp::Add => (0, "+"),
        BinOp::BitAnd => (1, "&"),
        BinOp::FloorDiv => (2, "//"),
        BinOp::LShift => (3, "<<"),
        BinOp::MatMul => (4, "@"),
        BinOp::Mul => (5, "*"),
        BinOp::Mod => (6, "%"),
        BinOp::BitOr => (7, "|"),
        BinOp::Pow => (8, "**"),
        BinOp::RShift => (9, ">>"),
        BinOp::Sub => (10, "-"),
        BinOp::Div => (11, "/"),
        BinOp::BitXor => (12, "^"),
    }
}

/// the opname, operand and operand kind of `op`
fn decode(op: &Op) -> (&'static str, i64, Arg) {
    use Arg as A;
    let n = |v: u32| v as i64;
    match *op {
        Op::Nop => ("NOP", 0, A::None),
        Op::LoadConst(k) => ("LOAD_CONST", n(k), A::Const),
        Op::LoadFast(i) => ("LOAD_FAST", n(i), A::Local),
        Op::StoreFast(i) => ("STORE_FAST", n(i), A::Local),
        Op::DelFast(i) => ("DELETE_FAST", n(i), A::Local),
        Op::LoadFastAndClear(i) => ("LOAD_FAST_AND_CLEAR", n(i), A::Local),
        Op::MakeCell(i) => ("MAKE_CELL", n(i), A::Cell),
        Op::LoadDeref(i) => ("LOAD_DEREF", n(i), A::Cell),
        Op::StoreDeref(i) => ("STORE_DEREF", n(i), A::Cell),
        Op::DelDeref(i) => ("DELETE_DEREF", n(i), A::Cell),
        Op::LoadClassDeref(i) => ("LOAD_CLASSDEREF", n(i), A::Cell),
        Op::LoadGlobal(i) => ("LOAD_GLOBAL", n(i), A::Name),
        Op::StoreGlobal(i) => ("STORE_GLOBAL", n(i), A::Name),
        Op::DelGlobal(i) => ("DELETE_GLOBAL", n(i), A::Name),
        Op::LoadName(i) => ("LOAD_NAME", n(i), A::Name),
        Op::StoreName(i) => ("STORE_NAME", n(i), A::Name),
        Op::DelName(i) => ("DELETE_NAME", n(i), A::Name),
        Op::LoadAttr(i) => ("LOAD_ATTR", n(i), A::Name),
        Op::StoreAttr(i) => ("STORE_ATTR", n(i), A::Name),
        Op::DelAttr(i) => ("DELETE_ATTR", n(i), A::Name),
        Op::LoadMethod(i) => ("LOAD_METHOD", n(i), A::Name),
        Op::CallMethod(c) => ("CALL_METHOD", n(c), A::Num),
        Op::Subscr => ("BINARY_SUBSCR", 0, A::None),
        Op::StoreSubscr => ("STORE_SUBSCR", 0, A::None),
        Op::DelSubscr => ("DELETE_SUBSCR", 0, A::None),
        Op::BuildSlice(c) => ("BUILD_SLICE", c as i64, A::Num),
        Op::Binary(b) => ("BINARY_OP", nb(b).0, A::Binary),
        Op::Inplace(b) => ("BINARY_OP", nb(b).0 + 13, A::Binary),
        Op::Unary(UnOp::Neg) => ("UNARY_NEGATIVE", 0, A::None),
        Op::Unary(UnOp::Pos) => ("UNARY_POSITIVE", 0, A::None),
        Op::Unary(UnOp::Not) => ("UNARY_NOT", 0, A::None),
        Op::Unary(UnOp::Invert) => ("UNARY_INVERT", 0, A::None),
        Op::Compare(c) => match c {
            CmpOp::Lt => ("COMPARE_OP", 0, A::Compare),
            CmpOp::Le => ("COMPARE_OP", 1, A::Compare),
            CmpOp::Eq => ("COMPARE_OP", 2, A::Compare),
            CmpOp::Ne => ("COMPARE_OP", 3, A::Compare),
            CmpOp::Gt => ("COMPARE_OP", 4, A::Compare),
            CmpOp::Ge => ("COMPARE_OP", 5, A::Compare),
            CmpOp::Is => ("IS_OP", 0, A::Num),
            CmpOp::IsNot => ("IS_OP", 1, A::Num),
            CmpOp::In => ("CONTAINS_OP", 0, A::Num),
            CmpOp::NotIn => ("CONTAINS_OP", 1, A::Num),
        },
        Op::Pop => ("POP_TOP", 0, A::None),
        Op::Dup => ("COPY", 1, A::Num),
        Op::Dup2 => ("DUP_TOP_TWO", 0, A::None),
        Op::Rot2 => ("ROT_TWO", 0, A::None),
        Op::Rot3 => ("ROT_THREE", 0, A::None),
        Op::Rot4 => ("ROT_FOUR", 0, A::None),
        Op::Swap(c) => ("SWAP", n(c), A::Num),
        Op::Jump(t) => ("JUMP", n(t), A::Jump),
        Op::PopJumpIfFalse(t) => ("POP_JUMP_IF_FALSE", n(t), A::Jump),
        Op::PopJumpIfTrue(t) => ("POP_JUMP_IF_TRUE", n(t), A::Jump),
        Op::JumpIfFalseOrPop(t) => ("JUMP_IF_FALSE_OR_POP", n(t), A::Jump),
        Op::JumpIfTrueOrPop(t) => ("JUMP_IF_TRUE_OR_POP", n(t), A::Jump),
        Op::PopJumpIfNotNone(t) => ("POP_JUMP_IF_NOT_NONE", n(t), A::Jump),
        Op::BuildList(c) => ("BUILD_LIST", n(c), A::Num),
        Op::BuildTuple(c) => ("BUILD_TUPLE", n(c), A::Num),
        Op::BuildSet(c) => ("BUILD_SET", n(c), A::Num),
        Op::BuildMap(c) => ("BUILD_MAP", n(c), A::Num),
        Op::BuildString(c) => ("BUILD_STRING", n(c), A::Num),
        Op::ListAppend(c) => ("LIST_APPEND", n(c), A::Num),
        Op::SetAdd(c) => ("SET_ADD", n(c), A::Num),
        Op::MapAdd(c) => ("MAP_ADD", n(c), A::Num),
        Op::ListExtend(c) => ("LIST_EXTEND", n(c), A::Num),
        Op::SetUpdate(c) => ("SET_UPDATE", n(c), A::Num),
        Op::DictUpdate(c) => ("DICT_UPDATE", n(c), A::Num),
        Op::DictMerge(c) => ("DICT_MERGE", n(c), A::Num),
        Op::ListToTuple => ("LIST_TO_TUPLE", 0, A::None),
        // CPython 3.12's FORMAT_VALUE operand: the conversion, then 4 for a spec
        Op::FormatValue(conv, spec) => ("FORMAT_VALUE", conv as i64 | (spec as i64) << 2, A::Num),
        Op::GetIter => ("GET_ITER", 0, A::None),
        Op::GetAwaitable => ("GET_AWAITABLE", 0, A::None),
        Op::ForIter(t) => ("FOR_ITER", n(t), A::Jump),
        Op::Unpack(c) => ("UNPACK_SEQUENCE", n(c), A::Num),
        Op::UnpackEx(before, after) => ("UNPACK_EX", before as i64 | (after as i64) << 8, A::Num),
        Op::Call(c) => ("CALL", n(c), A::Num),
        Op::CallKw(c) => ("CALL_KW", n(c), A::Num),
        Op::CallEx => ("CALL_FUNCTION_EX", 0, A::None),
        Op::MakeFunction(k, _) => ("MAKE_FUNCTION", n(k), A::Const),
        Op::LoadClosure(i) => ("LOAD_CLOSURE", n(i), A::Cell),
        Op::Return => ("RETURN_VALUE", 0, A::None),
        Op::Yield => ("YIELD_VALUE", 0, A::None),
        Op::YieldFrom => ("YIELD_FROM", 0, A::None),
        Op::SetupExcept(t) => ("SETUP_EXCEPT", n(t), A::Jump),
        Op::SetupFinally(t) => ("SETUP_FINALLY", n(t), A::Jump),
        Op::SetupWith(t) => ("SETUP_WITH", n(t), A::Jump),
        Op::PopBlock => ("POP_BLOCK", 0, A::None),
        Op::PopExcept => ("POP_EXCEPT", 0, A::None),
        Op::Reraise => ("RERAISE", 0, A::None),
        Op::Raise(c) => ("RAISE_VARARGS", c as i64, A::Num),
        Op::ExcMatch => ("CHECK_EXC_MATCH", 0, A::None),
        Op::WithExcept => ("WITH_EXCEPT_START", 0, A::None),
        Op::WithExit(c) => ("WITH_EXIT", n(c), A::Num),
        Op::ImportName(i) => ("IMPORT_NAME", n(i), A::Name),
        Op::ImportFrom(i) => ("IMPORT_FROM", n(i), A::Name),
        Op::ImportStar => ("IMPORT_STAR", 0, A::None),
        Op::LoadBuildClass => ("LOAD_BUILD_CLASS", 0, A::None),
        Op::SetupAnnotations => ("SETUP_ANNOTATIONS", 0, A::None),
        Op::LoadAssertionError => ("LOAD_ASSERTION_ERROR", 0, A::None),
        Op::MatchPattern(k) => ("MATCH_PATTERN", n(k), A::Const),
        Op::ExcStarMatch => ("CHECK_EG_MATCH", 0, A::None),
        Op::ExcStarFinish => ("EXC_STAR_FINISH", 0, A::None),
        Op::ExcStarCollect => ("EXC_STAR_COLLECT", 0, A::None),
        Op::LoadExc => ("LOAD_EXC", 0, A::None),
    }
}

pub fn dis_module() -> Value {
    let (d, m) = module("_rpydis");
    d.borrow_mut().set_s("opnames", Value::tuple(OPNAMES.iter().map(|s| Value::str(s)).collect()));
    put_fns(&d, &[("_rpydis.instructions", instructions)]);
    m
}

/// instructions(code): (opname, opcode, arg, argval, argrepr, offset, line)
/// for each instruction; arg is None for one without an operand
fn instructions(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    need(vm, a, 1)?;
    let c: Rc<Code> = match &a[0] {
        Value::Code(c) => c.clone(),
        o => {
            let t = vm.type_name(o);
            return Err(vm.type_error(format!("expected a code object, not {}", t)));
        }
    };
    let mut out = Vec::with_capacity(c.ops.len());
    for (i, op) in c.ops.iter().enumerate() {
        let (name, arg, kind) = decode(op);
        let opcode = OPNAMES.iter().position(|n| *n == name).unwrap() as i64;
        let cell = |i: i64| c.cellvars.iter().chain(c.freevars.iter()).nth(i as usize).map(|n| n.to_string());
        let (argval, argrepr) = match kind {
            Arg::None => (Value::None, String::new()),
            Arg::Num => (Value::Int(arg), String::new()),
            Arg::Const => {
                let v = c.consts[arg as usize].clone();
                let r = vm.repr(&v)?;
                (v, r)
            }
            Arg::Name => {
                let s = c.names[arg as usize].clone();
                let r = s.s.to_string();
                (Value::Str(s), r)
            }
            Arg::Local => {
                let s = c.varnames[arg as usize].to_string();
                (Value::str(&s), s)
            }
            Arg::Cell => {
                let s = cell(arg).unwrap_or_default();
                (Value::str(&s), s)
            }
            Arg::Jump => (Value::Int(2 * arg), format!("to {}", 2 * arg)),
            Arg::Binary => {
                let b = match op {
                    Op::Binary(b) | Op::Inplace(b) => *b,
                    _ => unreachable!(),
                };
                let sym = nb(b).1;
                let r = if matches!(op, Op::Inplace(_)) { format!("{}=", sym) } else { sym.to_string() };
                (Value::Int(arg), r)
            }
            Arg::Compare => {
                let r = ["<", "<=", "==", "!=", ">", ">="][arg as usize];
                (Value::str(r), r.to_string())
            }
        };
        let arg = if matches!(kind, Arg::None) { Value::None } else { Value::Int(arg) };
        let line = c.positions.get(i).map_or(0, |p| p.line);
        let line = if line > 0 { Value::Int(line as i64) } else { Value::None };
        out.push(Value::tuple(vec![
            Value::str(name),
            Value::Int(opcode),
            arg,
            argval,
            Value::str(&argrepr),
            Value::Int(2 * i as i64),
            line,
        ]));
    }
    Ok(Value::list(out))
}
