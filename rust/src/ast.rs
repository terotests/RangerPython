// SPDX-License-Identifier: MIT
//! Syntax tree produced by the parser.

use crate::value::PyStr;
use std::rc::Rc;

pub type Name = Rc<str>;

/// A node's span in the source: 1-based lines and 0-based columns, the end
/// exclusive, as in CPython's ast. Columns count UTF-8 bytes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Pos {
    pub line: u32,
    pub col: u32,
    pub end_line: u32,
    pub end_col: u32,
}

impl Pos {
    /// from the start of `self` to the end of `other`
    pub fn to(self, other: Pos) -> Pos {
        Pos { end_line: other.end_line, end_col: other.end_col, ..self }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    FloorDiv,
    Mod,
    Pow,
    LShift,
    RShift,
    BitOr,
    BitXor,
    BitAnd,
    MatMul,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnOp {
    Neg,
    Pos,
    Not,
    Invert,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CmpOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Is,
    IsNot,
    In,
    NotIn,
}

#[derive(Clone, Debug)]
pub struct Expr {
    pub kind: ExprKind,
    pub pos: Pos,
}

impl Expr {
    pub fn new(kind: ExprKind, pos: Pos) -> Expr {
        Expr { kind, pos }
    }

    /// what CPython's error messages call this kind of expression
    pub fn describe(&self) -> &'static str {
        match &self.kind {
            ExprKind::Name(_) => "name",
            ExprKind::Const(Const::None) => "None",
            ExprKind::Const(Const::True) => "True",
            ExprKind::Const(Const::False) => "False",
            ExprKind::Const(Const::Ellipsis) => "ellipsis",
            ExprKind::Const(_) => "literal",
            ExprKind::JoinedStr(_) | ExprKind::FormattedValue { .. } => "f-string expression",
            ExprKind::Bin(..) | ExprKind::Unary(..) | ExprKind::BoolAnd(_) | ExprKind::BoolOr(_) => "expression",
            ExprKind::Compare(..) => "comparison",
            ExprKind::IfExp(..) => "conditional expression",
            ExprKind::Call { .. } => "function call",
            ExprKind::Attr(..) => "attribute",
            ExprKind::Subscript(..) => "subscript",
            ExprKind::Slice(..) => "slice",
            ExprKind::List(_) => "list",
            ExprKind::Tuple(_) => "tuple",
            ExprKind::Set(_) => "set display",
            ExprKind::Dict(_) => "dict literal",
            ExprKind::Starred(_) => "starred",
            ExprKind::Lambda(_) => "lambda",
            ExprKind::ListComp(..) => "list comprehension",
            ExprKind::SetComp(..) => "set comprehension",
            ExprKind::DictComp(..) => "dict comprehension",
            ExprKind::GenExp(..) => "generator expression",
            ExprKind::NamedExpr(..) => "named expression",
            ExprKind::Yield(_) | ExprKind::YieldFrom(_) => "yield expression",
            ExprKind::Await(_) => "await expression",
        }
    }

    /// the first part of this assignment target that cannot be assigned to
    pub fn invalid_target(&self) -> Option<&Expr> {
        match &self.kind {
            ExprKind::Name(_) | ExprKind::Attr(..) | ExprKind::Subscript(..) => None,
            ExprKind::Tuple(items) | ExprKind::List(items) => items.iter().find_map(Expr::invalid_target),
            ExprKind::Starred(inner) => inner.invalid_target(),
            _ => Some(self),
        }
    }
}

#[derive(Clone, Debug)]
pub enum ExprKind {
    Name(Name),
    Const(Const),
    /// f-string: literal pieces and formatted values
    JoinedStr(Vec<Expr>),
    FormattedValue { value: Box<Expr>, conv: char, spec: Option<Box<Expr>> },
    Bin(BinOp, Box<Expr>, Box<Expr>),
    Unary(UnOp, Box<Expr>),
    /// `a and b and ...`, two or more operands
    BoolAnd(Vec<Expr>),
    BoolOr(Vec<Expr>),
    Compare(Box<Expr>, Vec<(CmpOp, Expr)>),
    IfExp(Box<Expr>, Box<Expr>, Box<Expr>),
    Call { func: Box<Expr>, args: Vec<Arg> },
    Attr(Box<Expr>, Name),
    Subscript(Box<Expr>, Box<Expr>),
    Slice(Option<Box<Expr>>, Option<Box<Expr>>, Option<Box<Expr>>),
    List(Vec<Expr>),
    Tuple(Vec<Expr>),
    Set(Vec<Expr>),
    /// None key means `**value`
    Dict(Vec<(Option<Expr>, Expr)>),
    Starred(Box<Expr>),
    Lambda(Rc<FuncDef>),
    ListComp(Box<Expr>, Vec<Comp>),
    SetComp(Box<Expr>, Vec<Comp>),
    DictComp(Box<Expr>, Box<Expr>, Vec<Comp>),
    GenExp(Box<Expr>, Vec<Comp>),
    NamedExpr(Name, Box<Expr>),
    Yield(Option<Box<Expr>>),
    YieldFrom(Box<Expr>),
    Await(Box<Expr>),
}

#[derive(Clone, Debug)]
pub enum Const {
    None,
    True,
    False,
    Ellipsis,
    Int(i64),
    BigInt(String),
    Float(f64),
    Imag(f64),
    Str(Rc<PyStr>),
    Bytes(Rc<[u8]>),
}

#[derive(Clone, Debug)]
pub enum Arg {
    Pos(Expr),
    Star(Expr),
    /// a keyword argument and the span of `name=value`
    Kw(Name, Expr, Pos),
    /// `**value` and its span
    StarStar(Expr, Pos),
}

#[derive(Clone, Debug)]
pub struct Comp {
    pub target: Expr,
    pub iter: Expr,
    pub ifs: Vec<Expr>,
    pub is_async: bool,
}

#[derive(Clone, Debug)]
pub struct Param {
    pub name: Name,
    pub default: Option<Expr>,
    pub annotation: Option<Expr>,
    /// the name and its annotation
    pub pos: Pos,
}

#[derive(Clone, Debug, Default)]
pub struct Params {
    pub posonly: usize,
    /// positional (including positional-only first) parameters
    pub args: Vec<Param>,
    pub vararg: Option<Param>,
    pub kwonly: Vec<Param>,
    pub kwarg: Option<Param>,
}

#[derive(Clone, Debug)]
pub struct FuncDef {
    pub name: Name,
    pub params: Params,
    pub body: Vec<Stmt>,
    pub decorators: Vec<Expr>,
    pub returns: Option<Expr>,
    pub is_lambda: bool,
    pub is_async: bool,
    /// the first line, a decorator's when there is one
    pub line: u32,
    /// PEP 695 type parameters
    pub type_params: Vec<TypeParam>,
}

#[derive(Clone, Debug)]
pub struct ClassDef {
    pub name: Name,
    pub bases: Vec<Arg>,
    pub body: Vec<Stmt>,
    pub decorators: Vec<Expr>,
    pub line: u32,
    pub type_params: Vec<TypeParam>,
}

/// `T`, `T: bound`, `T = default`, `*Ts` (kind 1) or `**P` (kind 2)
#[derive(Clone, Debug)]
pub struct TypeParam {
    pub kind: u8,
    pub name: Name,
    pub bound: Option<Expr>,
    pub default: Option<Expr>,
    pub pos: Pos,
}

#[derive(Clone, Debug)]
pub struct Handler {
    pub typ: Option<Expr>,
    pub name: Option<Name>,
    pub body: Vec<Stmt>,
    pub pos: Pos,
}

/// `name [as asname]` of an import
#[derive(Clone, Debug)]
pub struct Alias {
    pub name: Name,
    pub asname: Option<Name>,
    pub pos: Pos,
}

#[derive(Clone, Debug)]
pub struct Pattern {
    pub kind: PatternKind,
    pub pos: Pos,
}

#[derive(Clone, Debug)]
pub enum PatternKind {
    /// `_` or a capture name (None for `_`)
    Capture(Option<Name>),
    Value(Expr),
    Singleton(Const),
    Sequence(Vec<Pattern>),
    /// `*name` inside a sequence
    Star(Option<Name>),
    Mapping { keys: Vec<Expr>, pats: Vec<Pattern>, rest: Option<Name> },
    Class { cls: Expr, args: Vec<Pattern>, kw: Vec<(Name, Pattern)> },
    Or(Vec<Pattern>),
    As(Box<Pattern>, Name),
}

#[derive(Clone, Debug)]
pub struct MatchCase {
    pub pat: Pattern,
    pub guard: Option<Expr>,
    pub body: Vec<Stmt>,
}

#[derive(Clone, Debug)]
pub struct Stmt {
    pub pos: Pos,
    pub kind: StmtKind,
}

impl Stmt {
    pub fn line(&self) -> u32 {
        self.pos.line
    }
}

#[derive(Clone, Debug)]
pub enum StmtKind {
    Expr(Expr),
    Assign(Vec<Expr>, Expr),
    AugAssign(Expr, BinOp, Expr),
    /// target, annotation, value, and whether the target is a bare name
    AnnAssign(Expr, Expr, Option<Expr>, bool),
    If(Expr, Vec<Stmt>, Vec<Stmt>),
    While(Expr, Vec<Stmt>, Vec<Stmt>),
    /// target, iterable, body, else, and whether it is `async for`
    For(Expr, Expr, Vec<Stmt>, Vec<Stmt>, bool),
    Break,
    Continue,
    Pass,
    Return(Option<Expr>),
    FuncDef(Rc<FuncDef>),
    ClassDef(Rc<ClassDef>),
    Global(Vec<Name>),
    Nonlocal(Vec<Name>),
    Del(Vec<Expr>),
    Raise(Option<Expr>, Option<Expr>),
    Try { body: Vec<Stmt>, handlers: Vec<Handler>, orelse: Vec<Stmt>, finally: Vec<Stmt>, star: bool },
    /// items, body, and whether it is `async with`
    With(Vec<(Expr, Option<Expr>)>, Vec<Stmt>, bool),
    Import(Vec<Alias>),
    ImportFrom { module: Name, names: Vec<Alias>, level: usize },
    /// `type name[params] = value` (PEP 695)
    TypeAlias(Expr, Vec<TypeParam>, Expr),
    Assert(Expr, Option<Expr>),
    Match(Expr, Vec<MatchCase>),
}
