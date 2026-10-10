// SPDX-License-Identifier: MIT
//! Syntax tree produced by the parser.

use std::rc::Rc;

pub type Name = Rc<str>;

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
pub enum Expr {
    Name(Name),
    Const(Const),
    /// f-string: literal pieces and formatted values
    JoinedStr(Vec<Expr>),
    FormattedValue { value: Box<Expr>, conv: char, spec: Option<Box<Expr>> },
    Bin(BinOp, Box<Expr>, Box<Expr>),
    Unary(UnOp, Box<Expr>),
    BoolAnd(Box<Expr>, Box<Expr>),
    BoolOr(Box<Expr>, Box<Expr>),
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
    Str(Rc<str>),
    Bytes(Rc<[u8]>),
}

#[derive(Clone, Debug)]
pub enum Arg {
    Pos(Expr),
    Star(Expr),
    Kw(Name, Expr),
    StarStar(Expr),
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
    pub line: u32,
}

#[derive(Clone, Debug)]
pub struct ClassDef {
    pub name: Name,
    pub bases: Vec<Arg>,
    pub body: Vec<Stmt>,
    pub decorators: Vec<Expr>,
    pub line: u32,
}

#[derive(Clone, Debug)]
pub struct Handler {
    pub typ: Option<Expr>,
    pub name: Option<Name>,
    pub body: Vec<Stmt>,
    pub line: u32,
}

#[derive(Clone, Debug)]
pub enum Pattern {
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
    pub line: u32,
    pub kind: StmtKind,
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
    For(Expr, Expr, Vec<Stmt>, Vec<Stmt>),
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
    With(Vec<(Expr, Option<Expr>)>, Vec<Stmt>),
    Import(Vec<(Name, Option<Name>)>),
    ImportFrom { module: Name, names: Vec<(Name, Option<Name>)>, level: usize },
    Assert(Expr, Option<Expr>),
    Match(Expr, Vec<MatchCase>),
}
