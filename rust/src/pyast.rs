// SPDX-License-Identifier: MIT
//! The ast module's view of the syntax tree: the parser's tree as instances
//! of the node classes in _ast (lib/_ast.py), and such instances back as a
//! tree that compile() can turn into code.

use crate::ast::*;
use crate::ast::Name;
use crate::builtins::ps;
use crate::code::Code;
use crate::value::{Class, Exc, Module, PyResult, Value};
use crate::vm::Vm;
use std::collections::HashMap;
use std::rc::Rc;

pub const PYCF_ONLY_AST: i64 = 0x400;
pub const PYCF_OPTIMIZED_AST: i64 = 0x8000 | PYCF_ONLY_AST;
pub const PYCF_ALLOW_TOP_LEVEL_AWAIT: i64 = 0x2000;
/// the flags compile() takes: the CO_FUTURE_* bits, the obsolete CO_NESTED,
/// and the PyCF_* ones
pub const PYCF_COMPILE_MASK: i64 = 0x1fe_0000 | 0x10 | PYCF_OPTIMIZED_AST | 0x200 | 0x1000 | PYCF_ALLOW_TOP_LEVEL_AWAIT | 0x4000;

/// what compile() makes of a source or a tree, by its mode
pub enum Tree {
    Module(Vec<Stmt>),
    Expression(Expr),
    Interactive(Vec<Stmt>),
}

/// whether `v` is an ast.AST instance
pub fn is_ast(vm: &mut Vm, v: &Value) -> PyResult<bool> {
    if !matches!(v, Value::Instance(_)) {
        return Ok(false);
    }
    let m = vm.import_one("_ast")?;
    match vm.getattr(&m, &ps("AST"))? {
        Value::Class(c) => Ok(vm.isinstance(v, &c)),
        _ => Ok(false),
    }
}

/// ast.parse(): `src` parsed in `mode` as a tree of node instances
pub fn parse(vm: &mut Vm, src: &str, file: &str, mode: &str, flags: i64) -> PyResult<Value> {
    let tree = vm.from_source(src, file, || {
        Ok(Ok(match mode {
            "eval" => Tree::Expression(crate::parser::parse_expression(src)?),
            "single" => Tree::Interactive(crate::parser::parse_module(src)?),
            _ => Tree::Module(crate::parser::parse_module(src)?),
        }))
    })?;
    let tree = if flags & PYCF_OPTIMIZED_AST == PYCF_OPTIMIZED_AST { fold_tree(tree) } else { tree };
    ToPy::new(vm)?.tree(&tree)
}

/// compile() of a tree: code, or with PyCF_ONLY_AST the tree again
pub fn compile(vm: &mut Vm, node: &Value, file: &str, mode: &str, flags: i64, optimize: u8) -> PyResult<Value> {
    let tree = FromPy { vm }.tree(node, mode)?;
    if flags & PYCF_ONLY_AST != 0 {
        let tree = if flags & PYCF_OPTIMIZED_AST == PYCF_OPTIMIZED_AST { fold_tree(tree) } else { tree };
        return ToPy::new(vm)?.tree(&tree);
    }
    let top_await = flags & PYCF_ALLOW_TOP_LEVEL_AWAIT != 0;
    let code: Result<Rc<Code>, _> = match tree {
        Tree::Expression(e) => crate::compile::compile_eval_opt(&e, file, optimize, top_await),
        Tree::Module(b) | Tree::Interactive(b) => crate::compile::compile_module_opt(b, file, optimize, top_await),
    };
    code.map(Value::Code).map_err(|e| vm.parse_error(&e.syntax_error(None), file))
}

// ---------------------------------------------------------------------------
// to node instances

struct ToPy<'v> {
    vm: &'v mut Vm,
    module: Rc<Module>,
    classes: HashMap<&'static str, Rc<Class>>,
    /// the shared Load, Store and Del contexts
    ctx: [Value; 3],
}

#[derive(Clone, Copy)]
enum Ctx {
    Load,
    Store,
    Del,
}

impl<'v> ToPy<'v> {
    fn new(vm: &'v mut Vm) -> PyResult<ToPy<'v>> {
        let module = match vm.import_one("_ast")? {
            Value::Module(m) => m,
            _ => return Err(vm.err("ImportError", "_ast is not a module")),
        };
        let mut t = ToPy { vm, module, classes: HashMap::new(), ctx: [Value::None, Value::None, Value::None] };
        t.ctx = [t.node("Load", vec![], None)?, t.node("Store", vec![], None)?, t.node("Del", vec![], None)?];
        Ok(t)
    }

    fn class(&mut self, name: &'static str) -> PyResult<Rc<Class>> {
        if let Some(c) = self.classes.get(name) {
            return Ok(c.clone());
        }
        match self.module.dict.borrow().get_s(name) {
            Some(Value::Class(c)) => {
                self.classes.insert(name, c.clone());
                Ok(c)
            }
            _ => Err(self.vm.err("AttributeError", format!("module '_ast' has no attribute '{}'", name))),
        }
    }

    /// an instance of the node class `name` with these fields and position
    fn node(&mut self, name: &'static str, fields: Vec<(&str, Value)>, pos: Option<Pos>) -> PyResult<Value> {
        let cls = self.class(name)?;
        let v = self.vm.new_instance(&cls);
        if let Value::Instance(i) = &v {
            let mut d = i.dict.borrow_mut();
            for (k, f) in fields {
                d.set_s(k, f);
            }
            if let Some(p) = pos {
                d.set_s("lineno", Value::Int(p.line as i64));
                d.set_s("col_offset", Value::Int(p.col as i64));
                d.set_s("end_lineno", Value::Int(p.end_line as i64));
                d.set_s("end_col_offset", Value::Int(p.end_col as i64));
            }
        }
        Ok(v)
    }

    fn tree(&mut self, t: &Tree) -> PyResult<Value> {
        match t {
            Tree::Module(b) => {
                let body = self.stmts(b)?;
                self.node("Module", vec![("body", body), ("type_ignores", Value::list(vec![]))], None)
            }
            Tree::Interactive(b) => {
                let body = self.stmts(b)?;
                self.node("Interactive", vec![("body", body)], None)
            }
            Tree::Expression(e) => {
                let body = self.expr(e)?;
                self.node("Expression", vec![("body", body)], None)
            }
        }
    }

    fn stmts(&mut self, body: &[Stmt]) -> PyResult<Value> {
        let items = body.iter().map(|s| self.stmt(s)).collect::<PyResult<Vec<_>>>()?;
        Ok(Value::list(items))
    }

    fn exprs(&mut self, items: &[Expr], ctx: Ctx) -> PyResult<Value> {
        let items = items.iter().map(|e| self.expr_ctx(e, ctx)).collect::<PyResult<Vec<_>>>()?;
        Ok(Value::list(items))
    }

    fn opt(&mut self, e: &Option<Expr>) -> PyResult<Value> {
        match e {
            Some(e) => self.expr(e),
            None => Ok(Value::None),
        }
    }

    fn names(names: &[Name]) -> Value {
        Value::list(names.iter().map(|n| Value::str(n)).collect())
    }

    fn stmt(&mut self, s: &Stmt) -> PyResult<Value> {
        let none = || Value::None;
        let (name, fields): (&'static str, Vec<(&str, Value)>) = match &s.kind {
            StmtKind::Expr(e) => ("Expr", vec![("value", self.expr(e)?)]),
            StmtKind::Assign(targets, value) => {
                ("Assign", vec![("targets", self.exprs(targets, Ctx::Store)?), ("value", self.expr(value)?), ("type_comment", none())])
            }
            StmtKind::AugAssign(t, op, v) => {
                ("AugAssign", vec![("target", self.expr_ctx(t, Ctx::Store)?), ("op", self.node(binop_name(*op), vec![], None)?), ("value", self.expr(v)?)])
            }
            StmtKind::AnnAssign(t, ann, v, simple) => (
                "AnnAssign",
                vec![("target", self.expr_ctx(t, Ctx::Store)?), ("annotation", self.expr(ann)?), ("value", self.opt(v)?), ("simple", Value::Int(*simple as i64))],
            ),
            StmtKind::If(test, body, orelse) => ("If", vec![("test", self.expr(test)?), ("body", self.stmts(body)?), ("orelse", self.stmts(orelse)?)]),
            StmtKind::While(test, body, orelse) => ("While", vec![("test", self.expr(test)?), ("body", self.stmts(body)?), ("orelse", self.stmts(orelse)?)]),
            StmtKind::For(target, iter, body, orelse, is_async) => (
                if *is_async { "AsyncFor" } else { "For" },
                vec![
                    ("target", self.expr_ctx(target, Ctx::Store)?),
                    ("iter", self.expr(iter)?),
                    ("body", self.stmts(body)?),
                    ("orelse", self.stmts(orelse)?),
                    ("type_comment", none()),
                ],
            ),
            StmtKind::Break => ("Break", vec![]),
            StmtKind::Continue => ("Continue", vec![]),
            StmtKind::Pass => ("Pass", vec![]),
            StmtKind::Return(v) => ("Return", vec![("value", self.opt(v)?)]),
            StmtKind::FuncDef(f) => (
                if f.is_async { "AsyncFunctionDef" } else { "FunctionDef" },
                vec![
                    ("name", Value::str(&f.name)),
                    ("args", self.arguments(&f.params)?),
                    ("body", self.stmts(&f.body)?),
                    ("decorator_list", self.exprs(&f.decorators, Ctx::Load)?),
                    ("returns", self.opt(&f.returns)?),
                    ("type_comment", none()),
                    ("type_params", self.type_params(&f.type_params)?),
                ],
            ),
            StmtKind::ClassDef(c) => {
                let (bases, keywords) = self.call_args(&c.bases)?;
                (
                    "ClassDef",
                    vec![
                        ("name", Value::str(&c.name)),
                        ("bases", bases),
                        ("keywords", keywords),
                        ("body", self.stmts(&c.body)?),
                        ("decorator_list", self.exprs(&c.decorators, Ctx::Load)?),
                        ("type_params", self.type_params(&c.type_params)?),
                    ],
                )
            }
            StmtKind::Global(ns) => ("Global", vec![("names", Self::names(ns))]),
            StmtKind::Nonlocal(ns) => ("Nonlocal", vec![("names", Self::names(ns))]),
            StmtKind::Del(targets) => ("Delete", vec![("targets", self.exprs(targets, Ctx::Del)?)]),
            StmtKind::Raise(exc, cause) => ("Raise", vec![("exc", self.opt(exc)?), ("cause", self.opt(cause)?)]),
            StmtKind::Try { body, handlers, orelse, finally, star } => {
                let mut hs = vec![];
                for h in handlers {
                    let fields = vec![
                        ("type", self.opt(&h.typ)?),
                        ("name", h.name.as_ref().map_or(Value::None, |n| Value::str(n))),
                        ("body", self.stmts(&h.body)?),
                    ];
                    hs.push(self.node("ExceptHandler", fields, Some(h.pos))?);
                }
                (
                    if *star { "TryStar" } else { "Try" },
                    vec![("body", self.stmts(body)?), ("handlers", Value::list(hs)), ("orelse", self.stmts(orelse)?), ("finalbody", self.stmts(finally)?)],
                )
            }
            StmtKind::With(items, body, is_async) => {
                let mut its = vec![];
                for (e, t) in items {
                    let vars = match t {
                        Some(t) => self.expr_ctx(t, Ctx::Store)?,
                        None => Value::None,
                    };
                    let fields = vec![("context_expr", self.expr(e)?), ("optional_vars", vars)];
                    its.push(self.node("withitem", fields, None)?);
                }
                (if *is_async { "AsyncWith" } else { "With" }, vec![("items", Value::list(its)), ("body", self.stmts(body)?), ("type_comment", none())])
            }
            StmtKind::Import(names) => ("Import", vec![("names", self.aliases(names)?)]),
            StmtKind::ImportFrom { module, names, level } => {
                let m = if module.is_empty() { Value::None } else { Value::str(module) };
                ("ImportFrom", vec![("module", m), ("names", self.aliases(names)?), ("level", Value::Int(*level as i64))])
            }
            StmtKind::Assert(test, msg) => ("Assert", vec![("test", self.expr(test)?), ("msg", self.opt(msg)?)]),
            StmtKind::Match(subject, cases) => {
                let mut cs = vec![];
                for c in cases {
                    let fields = vec![("pattern", self.pattern(&c.pat)?), ("guard", self.opt(&c.guard)?), ("body", self.stmts(&c.body)?)];
                    cs.push(self.node("match_case", fields, None)?);
                }
                ("Match", vec![("subject", self.expr(subject)?), ("cases", Value::list(cs))])
            }
            StmtKind::TypeAlias(n, tps, v) => {
                ("TypeAlias", vec![("name", self.expr_ctx(n, Ctx::Store)?), ("type_params", self.type_params(tps)?), ("value", self.expr(v)?)])
            }
        };
        self.node(name, fields, Some(s.pos))
    }

    fn aliases(&mut self, names: &[Alias]) -> PyResult<Value> {
        let mut out = vec![];
        for a in names {
            let asname = a.asname.as_ref().map_or(Value::None, |n| Value::str(n));
            out.push(self.node("alias", vec![("name", Value::str(&a.name)), ("asname", asname)], Some(a.pos))?);
        }
        Ok(Value::list(out))
    }

    fn type_params(&mut self, tps: &[TypeParam]) -> PyResult<Value> {
        let mut out = vec![];
        for t in tps {
            let mut fields = vec![("name", Value::str(&t.name))];
            let cls = match t.kind {
                1 => "TypeVarTuple",
                2 => "ParamSpec",
                _ => {
                    fields.push(("bound", self.opt(&t.bound)?));
                    "TypeVar"
                }
            };
            fields.push(("default_value", self.opt(&t.default)?));
            out.push(self.node(cls, fields, Some(t.pos))?);
        }
        Ok(Value::list(out))
    }

    fn arg(&mut self, p: &Param) -> PyResult<Value> {
        let fields = vec![("arg", Value::str(&p.name)), ("annotation", self.opt(&p.annotation)?), ("type_comment", Value::None)];
        self.node("arg", fields, Some(p.pos))
    }

    fn arguments(&mut self, p: &Params) -> PyResult<Value> {
        let posonly = p.args[..p.posonly].iter().map(|a| self.arg(a)).collect::<PyResult<Vec<_>>>()?;
        let args = p.args[p.posonly..].iter().map(|a| self.arg(a)).collect::<PyResult<Vec<_>>>()?;
        let defaults = p.args.iter().filter_map(|a| a.default.as_ref()).map(|d| self.expr(d)).collect::<PyResult<Vec<_>>>()?;
        let kwonly = p.kwonly.iter().map(|a| self.arg(a)).collect::<PyResult<Vec<_>>>()?;
        let kw_defaults = p.kwonly.iter().map(|a| self.opt(&a.default)).collect::<PyResult<Vec<_>>>()?;
        let vararg = match &p.vararg {
            Some(a) => self.arg(a)?,
            None => Value::None,
        };
        let kwarg = match &p.kwarg {
            Some(a) => self.arg(a)?,
            None => Value::None,
        };
        let fields = vec![
            ("posonlyargs", Value::list(posonly)),
            ("args", Value::list(args)),
            ("vararg", vararg),
            ("kwonlyargs", Value::list(kwonly)),
            ("kw_defaults", Value::list(kw_defaults)),
            ("kwarg", kwarg),
            ("defaults", Value::list(defaults)),
        ];
        self.node("arguments", fields, None)
    }

    /// a call's positional arguments and its keywords
    fn call_args(&mut self, args: &[Arg]) -> PyResult<(Value, Value)> {
        let (mut pos, mut kws) = (vec![], vec![]);
        for a in args {
            match a {
                Arg::Pos(e) | Arg::Star(e) => pos.push(self.expr(e)?),
                Arg::Kw(n, e, p) => {
                    let fields = vec![("arg", Value::str(n)), ("value", self.expr(e)?)];
                    kws.push(self.node("keyword", fields, Some(*p))?);
                }
                Arg::StarStar(e, p) => {
                    let fields = vec![("arg", Value::None), ("value", self.expr(e)?)];
                    kws.push(self.node("keyword", fields, Some(*p))?);
                }
            }
        }
        Ok((Value::list(pos), Value::list(kws)))
    }

    fn comprehensions(&mut self, comps: &[Comp]) -> PyResult<Value> {
        let mut out = vec![];
        for c in comps {
            let fields = vec![
                ("target", self.expr_ctx(&c.target, Ctx::Store)?),
                ("iter", self.expr(&c.iter)?),
                ("ifs", self.exprs(&c.ifs, Ctx::Load)?),
                ("is_async", Value::Int(c.is_async as i64)),
            ];
            out.push(self.node("comprehension", fields, None)?);
        }
        Ok(Value::list(out))
    }

    fn expr(&mut self, e: &Expr) -> PyResult<Value> {
        self.expr_ctx(e, Ctx::Load)
    }

    fn expr_ctx(&mut self, e: &Expr, ctx: Ctx) -> PyResult<Value> {
        let c = self.ctx[ctx as usize].clone();
        let (name, fields): (&'static str, Vec<(&str, Value)>) = match &e.kind {
            ExprKind::Name(n) => ("Name", vec![("id", Value::str(n)), ("ctx", c)]),
            ExprKind::Const(k) => ("Constant", vec![("value", const_value(k)), ("kind", Value::None)]),
            ExprKind::JoinedStr(parts) => ("JoinedStr", vec![("values", self.exprs(parts, Ctx::Load)?)]),
            ExprKind::FormattedValue { value, conv, spec } => {
                let conv = if *conv == '\0' { -1 } else { *conv as i64 };
                let spec = match spec {
                    Some(s) => self.expr(s)?,
                    None => Value::None,
                };
                ("FormattedValue", vec![("value", self.expr(value)?), ("conversion", Value::Int(conv)), ("format_spec", spec)])
            }
            ExprKind::Bin(op, a, b) => ("BinOp", vec![("left", self.expr(a)?), ("op", self.node(binop_name(*op), vec![], None)?), ("right", self.expr(b)?)]),
            ExprKind::Unary(op, a) => {
                let op = match op {
                    UnOp::Neg => "USub",
                    UnOp::Pos => "UAdd",
                    UnOp::Not => "Not",
                    UnOp::Invert => "Invert",
                };
                ("UnaryOp", vec![("op", self.node(op, vec![], None)?), ("operand", self.expr(a)?)])
            }
            ExprKind::BoolAnd(values) | ExprKind::BoolOr(values) => {
                let op = self.node(if matches!(e.kind, ExprKind::BoolAnd(_)) { "And" } else { "Or" }, vec![], None)?;
                ("BoolOp", vec![("op", op), ("values", self.exprs(values, Ctx::Load)?)])
            }
            ExprKind::Compare(left, rest) => {
                let mut ops = vec![];
                let mut comparators = vec![];
                for (op, r) in rest {
                    ops.push(self.node(cmpop_name(*op), vec![], None)?);
                    comparators.push(self.expr(r)?);
                }
                ("Compare", vec![("left", self.expr(left)?), ("ops", Value::list(ops)), ("comparators", Value::list(comparators))])
            }
            ExprKind::IfExp(test, body, orelse) => ("IfExp", vec![("test", self.expr(test)?), ("body", self.expr(body)?), ("orelse", self.expr(orelse)?)]),
            ExprKind::Call { func, args } => {
                let func = self.expr(func)?;
                let (args, keywords) = self.call_args(args)?;
                ("Call", vec![("func", func), ("args", args), ("keywords", keywords)])
            }
            ExprKind::Attr(obj, attr) => ("Attribute", vec![("value", self.expr(obj)?), ("attr", Value::str(attr)), ("ctx", c)]),
            ExprKind::Subscript(obj, idx) => ("Subscript", vec![("value", self.expr(obj)?), ("slice", self.expr(idx)?), ("ctx", c)]),
            ExprKind::Slice(lo, hi, step) => {
                let mut part = |x: &Option<Box<Expr>>| match x {
                    Some(x) => self.expr(x),
                    None => Ok(Value::None),
                };
                ("Slice", vec![("lower", part(lo)?), ("upper", part(hi)?), ("step", part(step)?)])
            }
            ExprKind::List(items) => ("List", vec![("elts", self.exprs(items, ctx)?), ("ctx", c)]),
            ExprKind::Tuple(items) => ("Tuple", vec![("elts", self.exprs(items, ctx)?), ("ctx", c)]),
            ExprKind::Set(items) => ("Set", vec![("elts", self.exprs(items, Ctx::Load)?)]),
            ExprKind::Dict(items) => {
                let mut keys = vec![];
                let mut values = vec![];
                for (k, v) in items {
                    keys.push(self.opt(k)?);
                    values.push(self.expr(v)?);
                }
                ("Dict", vec![("keys", Value::list(keys)), ("values", Value::list(values))])
            }
            ExprKind::Starred(inner) => ("Starred", vec![("value", self.expr_ctx(inner, ctx)?), ("ctx", c)]),
            ExprKind::Lambda(f) => {
                let body = match f.body.first().map(|s| &s.kind) {
                    Some(StmtKind::Return(Some(b))) => self.expr(b)?,
                    _ => Value::None,
                };
                ("Lambda", vec![("args", self.arguments(&f.params)?), ("body", body)])
            }
            ExprKind::ListComp(elt, comps) => ("ListComp", vec![("elt", self.expr(elt)?), ("generators", self.comprehensions(comps)?)]),
            ExprKind::SetComp(elt, comps) => ("SetComp", vec![("elt", self.expr(elt)?), ("generators", self.comprehensions(comps)?)]),
            ExprKind::GenExp(elt, comps) => ("GeneratorExp", vec![("elt", self.expr(elt)?), ("generators", self.comprehensions(comps)?)]),
            ExprKind::DictComp(k, v, comps) => {
                ("DictComp", vec![("key", self.expr(k)?), ("value", self.expr(v)?), ("generators", self.comprehensions(comps)?)])
            }
            ExprKind::NamedExpr(n, v) => {
                let end_col = e.pos.col + n.len() as u32;
                let tpos = Pos { end_line: e.pos.line, end_col, ..e.pos };
                let store = self.ctx[Ctx::Store as usize].clone();
                let target = self.node("Name", vec![("id", Value::str(n)), ("ctx", store)], Some(tpos))?;
                ("NamedExpr", vec![("target", target), ("value", self.expr(v)?)])
            }
            ExprKind::Yield(v) => {
                let v = match v {
                    Some(v) => self.expr(v)?,
                    None => Value::None,
                };
                ("Yield", vec![("value", v)])
            }
            ExprKind::YieldFrom(v) => ("YieldFrom", vec![("value", self.expr(v)?)]),
            ExprKind::Await(v) => ("Await", vec![("value", self.expr(v)?)]),
        };
        self.node(name, fields, Some(e.pos))
    }

    fn pattern(&mut self, p: &Pattern) -> PyResult<Value> {
        let opt_name = |n: &Option<Name>| n.as_ref().map_or(Value::None, |n| Value::str(n));
        let (name, fields): (&'static str, Vec<(&str, Value)>) = match &p.kind {
            PatternKind::Capture(n) => ("MatchAs", vec![("pattern", Value::None), ("name", opt_name(n))]),
            PatternKind::Value(e) => ("MatchValue", vec![("value", self.expr(e)?)]),
            PatternKind::Singleton(c) => ("MatchSingleton", vec![("value", const_value(c))]),
            PatternKind::Sequence(ps) => ("MatchSequence", vec![("patterns", self.patterns(ps)?)]),
            PatternKind::Star(n) => ("MatchStar", vec![("name", opt_name(n))]),
            PatternKind::Mapping { keys, pats, rest } => {
                ("MatchMapping", vec![("keys", self.exprs(keys, Ctx::Load)?), ("patterns", self.patterns(pats)?), ("rest", opt_name(rest))])
            }
            PatternKind::Class { cls, args, kw } => {
                let attrs = Value::list(kw.iter().map(|(n, _)| Value::str(n)).collect());
                let kps = kw.iter().map(|(_, q)| self.pattern(q)).collect::<PyResult<Vec<_>>>()?;
                ("MatchClass", vec![("cls", self.expr(cls)?), ("patterns", self.patterns(args)?), ("kwd_attrs", attrs), ("kwd_patterns", Value::list(kps))])
            }
            PatternKind::Or(ps) => ("MatchOr", vec![("patterns", self.patterns(ps)?)]),
            PatternKind::As(q, n) => ("MatchAs", vec![("pattern", self.pattern(q)?), ("name", Value::str(n))]),
        };
        self.node(name, fields, Some(p.pos))
    }

    fn patterns(&mut self, ps: &[Pattern]) -> PyResult<Value> {
        let items = ps.iter().map(|q| self.pattern(q)).collect::<PyResult<Vec<_>>>()?;
        Ok(Value::list(items))
    }
}

fn const_value(c: &Const) -> Value {
    match c {
        Const::None => Value::None,
        Const::True => Value::Bool(true),
        Const::False => Value::Bool(false),
        Const::Ellipsis => Value::Ellipsis,
        Const::Int(i) => Value::Int(*i),
        Const::BigInt(s) => Value::from_big(crate::bigint::BigInt::from_radix(s, 10).unwrap()),
        Const::Float(f) => Value::Float(*f),
        Const::Imag(f) => Value::Complex(Rc::new((0.0, *f))),
        Const::Str(s) => Value::Str(s.clone()),
        Const::Bytes(b) => Value::Bytes(Rc::new(b.to_vec())),
    }
}

const BINOPS: [(BinOp, &str); 13] = [
    (BinOp::Add, "Add"),
    (BinOp::Sub, "Sub"),
    (BinOp::Mul, "Mult"),
    (BinOp::Div, "Div"),
    (BinOp::FloorDiv, "FloorDiv"),
    (BinOp::Mod, "Mod"),
    (BinOp::Pow, "Pow"),
    (BinOp::LShift, "LShift"),
    (BinOp::RShift, "RShift"),
    (BinOp::BitOr, "BitOr"),
    (BinOp::BitXor, "BitXor"),
    (BinOp::BitAnd, "BitAnd"),
    (BinOp::MatMul, "MatMult"),
];

const CMPOPS: [(CmpOp, &str); 10] = [
    (CmpOp::Eq, "Eq"),
    (CmpOp::Ne, "NotEq"),
    (CmpOp::Lt, "Lt"),
    (CmpOp::Le, "LtE"),
    (CmpOp::Gt, "Gt"),
    (CmpOp::Ge, "GtE"),
    (CmpOp::Is, "Is"),
    (CmpOp::IsNot, "IsNot"),
    (CmpOp::In, "In"),
    (CmpOp::NotIn, "NotIn"),
];

fn binop_name(op: BinOp) -> &'static str {
    BINOPS.iter().find(|(o, _)| *o == op).unwrap().1
}

fn cmpop_name(op: CmpOp) -> &'static str {
    CMPOPS.iter().find(|(o, _)| *o == op).unwrap().1
}

// ---------------------------------------------------------------------------
// from node instances

struct FromPy<'v> {
    vm: &'v mut Vm,
}

impl FromPy<'_> {
    /// the name of the _ast class `v` is an instance of
    fn kind(&mut self, v: &Value) -> Option<String> {
        let Value::Instance(i) = v else { return None };
        let cls = i.class.borrow().clone();
        let mro = cls.mro.borrow().clone();
        for c in std::iter::once(cls).chain(mro) {
            if matches!(c.dict.borrow().get_s("__module__"), Some(Value::Str(m)) if m.s == "ast") {
                return Some(c.name().to_string());
            }
        }
        None
    }

    fn type_error(&mut self, msg: String) -> Exc {
        self.vm.type_error(msg)
    }

    /// a field that must be present
    fn field(&mut self, v: &Value, name: &str, owner: &str) -> PyResult<Value> {
        match self.vm.getattr(v, &ps(name)) {
            Err(e) if self.vm.exc_is(&e, "AttributeError") => Err(self.type_error(format!("required field \"{}\" missing from {}", name, owner))),
            r => r,
        }
    }

    /// a field that may be missing or None
    fn opt_field(&mut self, v: &Value, name: &str) -> PyResult<Option<Value>> {
        match self.vm.getattr(v, &ps(name)) {
            Err(e) if self.vm.exc_is(&e, "AttributeError") => Ok(None),
            Ok(Value::None) => Ok(None),
            r => r.map(Some),
        }
    }

    fn list_field(&mut self, v: &Value, name: &str, owner: &str) -> PyResult<Vec<Value>> {
        match self.opt_field(v, name)? {
            None => Ok(vec![]),
            Some(Value::List(l)) => Ok(l.borrow().clone()),
            Some(_) => Err(self.type_error(format!("{} field \"{}\" must be a list, not a {}", owner, name, "non-list"))),
        }
    }

    fn ident(&mut self, v: &Value, name: &str, owner: &str) -> PyResult<Name> {
        match self.field(v, name, owner)? {
            Value::Str(s) => Ok(Rc::from(s.s.as_str())),
            other => {
                let t = self.vm.type_name(&other);
                Err(self.type_error(format!("{} field \"{}\" must be str, not {}", owner, name, t)))
            }
        }
    }

    fn opt_ident(&mut self, v: &Value, name: &str) -> PyResult<Option<Name>> {
        match self.opt_field(v, name)? {
            None => Ok(None),
            Some(Value::Str(s)) => Ok(Some(Rc::from(s.s.as_str()))),
            Some(other) => {
                let t = self.vm.type_name(&other);
                Err(self.type_error(format!("field \"{}\" must be str, not {}", name, t)))
            }
        }
    }

    fn int_field(&mut self, v: &Value, name: &str, owner: &str) -> PyResult<i64> {
        let f = self.field(v, name, owner)?;
        self.vm.to_index(&f)
    }

    /// a node's span; `owner` names its kind in errors
    fn pos(&mut self, v: &Value, owner: &str) -> PyResult<Pos> {
        let line = self.int_field(v, "lineno", owner)? as u32;
        let col = self.int_field(v, "col_offset", owner)? as u32;
        let end_line = match self.opt_field(v, "end_lineno")? {
            Some(x) => self.vm.to_index(&x)? as u32,
            None => line,
        };
        let end_col = match self.opt_field(v, "end_col_offset")? {
            Some(x) => self.vm.to_index(&x)? as u32,
            None => col,
        };
        Ok(Pos { line, col, end_line, end_col })
    }

    fn tree(&mut self, v: &Value, mode: &str) -> PyResult<Tree> {
        let kind = self.kind(v).unwrap_or_default();
        let want = match mode {
            "eval" => "Expression",
            "single" => "Interactive",
            _ => "Module",
        };
        if kind != want {
            let got = if kind.is_empty() { self.vm.type_name(v) } else { kind };
            return Err(self.type_error(format!("expected {} node, got {}", want, got)));
        }
        Ok(match mode {
            "eval" => {
                let b = self.field(v, "body", "Expression")?;
                Tree::Expression(self.expr(&b)?)
            }
            "single" => Tree::Interactive(self.stmts(v, "body", "Interactive")?),
            _ => Tree::Module(self.stmts(v, "body", "Module")?),
        })
    }

    fn stmts(&mut self, v: &Value, name: &str, owner: &str) -> PyResult<Vec<Stmt>> {
        let items = self.list_field(v, name, owner)?;
        items.iter().map(|s| self.stmt(s)).collect()
    }

    fn exprs(&mut self, v: &Value, name: &str, owner: &str) -> PyResult<Vec<Expr>> {
        let items = self.list_field(v, name, owner)?;
        items.iter().map(|e| self.expr(e)).collect()
    }

    fn opt_expr(&mut self, v: &Value, name: &str) -> PyResult<Option<Expr>> {
        match self.opt_field(v, name)? {
            Some(e) => Ok(Some(self.expr(&e)?)),
            None => Ok(None),
        }
    }

    fn sub_expr(&mut self, v: &Value, name: &str, owner: &str) -> PyResult<Expr> {
        let e = self.field(v, name, owner)?;
        if e.is_none() {
            return Err(self.type_error(format!("required field \"{}\" missing from {}", name, owner)));
        }
        self.expr(&e)
    }

    fn boxed(&mut self, v: &Value, name: &str, owner: &str) -> PyResult<Box<Expr>> {
        self.sub_expr(v, name, owner).map(Box::new)
    }

    fn op_kind(&mut self, v: &Value, name: &str, owner: &str) -> PyResult<String> {
        let op = self.field(v, name, owner)?;
        match self.kind(&op) {
            Some(k) => Ok(k),
            None => {
                let t = self.vm.type_name(&op);
                Err(self.type_error(format!("expected some sort of {}, but got {}", name, t)))
            }
        }
    }

    fn binop(&mut self, v: &Value, owner: &str) -> PyResult<BinOp> {
        let k = self.op_kind(v, "op", owner)?;
        match BINOPS.iter().find(|(_, n)| *n == k) {
            Some((op, _)) => Ok(*op),
            None => Err(self.type_error(format!("expected some sort of operator, but got {}", k))),
        }
    }

    fn stmt(&mut self, v: &Value) -> PyResult<Stmt> {
        let kind = match self.kind(v) {
            Some(k) => k,
            None => {
                let t = self.vm.type_name(v);
                return Err(self.type_error(format!("expected some sort of stmt, but got {}", t)));
            }
        };
        let k = kind.as_str();
        let pos = self.pos(v, "stmt")?;
        let kind = match k {
            "Expr" => StmtKind::Expr(self.sub_expr(v, "value", k)?),
            "Assign" => StmtKind::Assign(self.exprs(v, "targets", k)?, self.sub_expr(v, "value", k)?),
            "AugAssign" => {
                let t = self.sub_expr(v, "target", k)?;
                let op = self.binop(v, k)?;
                StmtKind::AugAssign(t, op, self.sub_expr(v, "value", k)?)
            }
            "AnnAssign" => {
                let t = self.sub_expr(v, "target", k)?;
                let ann = self.sub_expr(v, "annotation", k)?;
                let value = self.opt_expr(v, "value")?;
                let simple = self.int_field(v, "simple", k)? != 0;
                StmtKind::AnnAssign(t, ann, value, simple)
            }
            "If" | "While" => {
                let test = self.sub_expr(v, "test", k)?;
                let body = self.stmts(v, "body", k)?;
                let orelse = self.stmts(v, "orelse", k)?;
                if k == "If" {
                    StmtKind::If(test, body, orelse)
                } else {
                    StmtKind::While(test, body, orelse)
                }
            }
            "For" | "AsyncFor" => {
                let target = self.sub_expr(v, "target", k)?;
                let iter = self.sub_expr(v, "iter", k)?;
                StmtKind::For(target, iter, self.stmts(v, "body", k)?, self.stmts(v, "orelse", k)?, k == "AsyncFor")
            }
            "Break" => StmtKind::Break,
            "Continue" => StmtKind::Continue,
            "Pass" => StmtKind::Pass,
            "Return" => StmtKind::Return(self.opt_expr(v, "value")?),
            "FunctionDef" | "AsyncFunctionDef" => {
                let name = self.ident(v, "name", k)?;
                let a = self.field(v, "args", k)?;
                let params = self.arguments(&a)?;
                let body = self.stmts(v, "body", k)?;
                let decorators = self.exprs(v, "decorator_list", k)?;
                let returns = self.opt_expr(v, "returns")?;
                let type_params = self.type_params(v)?;
                let line = decorators.first().map_or(pos.line, |d| d.pos.line);
                let f = FuncDef { name, params, body, decorators, returns, is_lambda: false, is_async: k == "AsyncFunctionDef", line, type_params };
                StmtKind::FuncDef(Rc::new(f))
            }
            "ClassDef" => {
                let name = self.ident(v, "name", k)?;
                let mut bases: Vec<Arg> = self.exprs(v, "bases", k)?.into_iter().map(pos_arg).collect();
                bases.extend(self.keywords(v, k)?);
                let body = self.stmts(v, "body", k)?;
                let decorators = self.exprs(v, "decorator_list", k)?;
                let type_params = self.type_params(v)?;
                let line = decorators.first().map_or(pos.line, |d| d.pos.line);
                StmtKind::ClassDef(Rc::new(ClassDef { name, bases, body, decorators, line, type_params }))
            }
            "Global" | "Nonlocal" => {
                let names = self.list_field(v, "names", k)?;
                let mut out = vec![];
                for n in names {
                    match n {
                        Value::Str(s) => out.push(Rc::from(s.s.as_str())),
                        _ => return Err(self.type_error(format!("{} field \"names\" must be a list of str", k))),
                    }
                }
                if k == "Global" {
                    StmtKind::Global(out)
                } else {
                    StmtKind::Nonlocal(out)
                }
            }
            "Delete" => StmtKind::Del(self.exprs(v, "targets", k)?),
            "Raise" => StmtKind::Raise(self.opt_expr(v, "exc")?, self.opt_expr(v, "cause")?),
            "Try" | "TryStar" => {
                let body = self.stmts(v, "body", k)?;
                let mut handlers = vec![];
                for h in self.list_field(v, "handlers", k)? {
                    let hpos = self.pos(&h, "excepthandler")?;
                    let typ = self.opt_expr(&h, "type")?;
                    let name = self.opt_ident(&h, "name")?;
                    let hbody = self.stmts(&h, "body", "ExceptHandler")?;
                    handlers.push(Handler { typ, name, body: hbody, pos: hpos });
                }
                let orelse = self.stmts(v, "orelse", k)?;
                let finally = self.stmts(v, "finalbody", k)?;
                StmtKind::Try { body, handlers, orelse, finally, star: k == "TryStar" }
            }
            "With" | "AsyncWith" => {
                let mut items = vec![];
                for it in self.list_field(v, "items", k)? {
                    let e = self.sub_expr(&it, "context_expr", "withitem")?;
                    items.push((e, self.opt_expr(&it, "optional_vars")?));
                }
                StmtKind::With(items, self.stmts(v, "body", k)?, k == "AsyncWith")
            }
            "Import" => StmtKind::Import(self.aliases(v, k)?),
            "ImportFrom" => {
                let module = self.opt_ident(v, "module")?.unwrap_or_else(|| Rc::from(""));
                let names = self.aliases(v, k)?;
                let level = match self.opt_field(v, "level")? {
                    Some(l) => self.vm.to_index(&l)? as usize,
                    None => 0,
                };
                StmtKind::ImportFrom { module, names, level }
            }
            "Assert" => StmtKind::Assert(self.sub_expr(v, "test", k)?, self.opt_expr(v, "msg")?),
            "Match" => {
                let subject = self.sub_expr(v, "subject", k)?;
                let mut cases = vec![];
                for c in self.list_field(v, "cases", k)? {
                    let p = self.field(&c, "pattern", "match_case")?;
                    let pat = self.pattern(&p)?;
                    let guard = self.opt_expr(&c, "guard")?;
                    cases.push(MatchCase { pat, guard, body: self.stmts(&c, "body", "match_case")? });
                }
                StmtKind::Match(subject, cases)
            }
            "TypeAlias" => {
                let name = self.sub_expr(v, "name", k)?;
                let tps = self.type_params(v)?;
                StmtKind::TypeAlias(name, tps, self.sub_expr(v, "value", k)?)
            }
            _ => return Err(self.type_error(format!("expected some sort of stmt, but got <{}>", k))),
        };
        Ok(Stmt { pos, kind })
    }

    fn aliases(&mut self, v: &Value, owner: &str) -> PyResult<Vec<Alias>> {
        let mut out = vec![];
        for a in self.list_field(v, "names", owner)? {
            let pos = self.pos(&a, "alias").unwrap_or_default();
            let name = self.ident(&a, "name", "alias")?;
            out.push(Alias { name, asname: self.opt_ident(&a, "asname")?, pos });
        }
        Ok(out)
    }

    fn type_params(&mut self, v: &Value) -> PyResult<Vec<TypeParam>> {
        let mut out = vec![];
        for t in self.list_field(v, "type_params", "node")? {
            let k = self.kind(&t).unwrap_or_default();
            let kind = match k.as_str() {
                "TypeVar" => 0,
                "TypeVarTuple" => 1,
                "ParamSpec" => 2,
                _ => return Err(self.type_error(format!("expected some sort of type_param, but got {}", k))),
            };
            let pos = self.pos(&t, "type_param")?;
            let name = self.ident(&t, "name", &k)?;
            let bound = if kind == 0 { self.opt_expr(&t, "bound")? } else { None };
            let default = self.opt_expr(&t, "default_value")?;
            out.push(TypeParam { kind, name, bound, default, pos });
        }
        Ok(out)
    }

    fn arg(&mut self, a: &Value) -> PyResult<Param> {
        let name = self.ident(a, "arg", "arg")?;
        let annotation = self.opt_expr(a, "annotation")?;
        let pos = self.pos(a, "arg").unwrap_or_default();
        Ok(Param { name, default: None, annotation, pos })
    }

    fn arguments(&mut self, a: &Value) -> PyResult<Params> {
        let o = "arguments";
        let posonly = self.list_field(a, "posonlyargs", o)?;
        let plain = self.list_field(a, "args", o)?;
        let mut p = Params { posonly: posonly.len(), ..Params::default() };
        for x in posonly.iter().chain(plain.iter()) {
            p.args.push(self.arg(x)?);
        }
        let defaults = self.exprs(a, "defaults", o)?;
        if defaults.len() > p.args.len() {
            return Err(self.vm.value_error("more positional defaults than args on arguments"));
        }
        let first = p.args.len() - defaults.len();
        for (param, d) in p.args[first..].iter_mut().zip(defaults) {
            param.default = Some(d);
        }
        if let Some(v) = self.opt_field(a, "vararg")? {
            p.vararg = Some(self.arg(&v)?);
        }
        let kwonly = self.list_field(a, "kwonlyargs", o)?;
        let kw_defaults = self.list_field(a, "kw_defaults", o)?;
        if kw_defaults.len() != kwonly.len() {
            return Err(self.vm.value_error("length of kwonlyargs is not the same as kw_defaults on arguments"));
        }
        for (x, d) in kwonly.iter().zip(kw_defaults) {
            let mut param = self.arg(x)?;
            if !d.is_none() {
                param.default = Some(self.expr(&d)?);
            }
            p.kwonly.push(param);
        }
        if let Some(v) = self.opt_field(a, "kwarg")? {
            p.kwarg = Some(self.arg(&v)?);
        }
        Ok(p)
    }

    fn keywords(&mut self, v: &Value, owner: &str) -> PyResult<Vec<Arg>> {
        let mut out = vec![];
        for kw in self.list_field(v, "keywords", owner)? {
            let pos = self.pos(&kw, "keyword").unwrap_or_default();
            let value = self.sub_expr(&kw, "value", "keyword")?;
            out.push(match self.opt_ident(&kw, "arg")? {
                Some(n) => Arg::Kw(n, value, pos),
                None => Arg::StarStar(value, pos),
            });
        }
        Ok(out)
    }

    fn comprehensions(&mut self, v: &Value, owner: &str) -> PyResult<Vec<Comp>> {
        let mut out = vec![];
        for c in self.list_field(v, "generators", owner)? {
            let o = "comprehension";
            let target = self.sub_expr(&c, "target", o)?;
            let iter = self.sub_expr(&c, "iter", o)?;
            let ifs = self.exprs(&c, "ifs", o)?;
            let is_async = self.int_field(&c, "is_async", o)? != 0;
            out.push(Comp { target, iter, ifs, is_async });
        }
        if out.is_empty() {
            return Err(self.vm.value_error("comprehension with no generators"));
        }
        Ok(out)
    }

    fn constant(&mut self, c: &Value, pos: Pos) -> PyResult<ExprKind> {
        Ok(ExprKind::Const(match c {
            Value::None => Const::None,
            Value::Bool(true) => Const::True,
            Value::Bool(false) => Const::False,
            Value::Ellipsis => Const::Ellipsis,
            Value::Int(i) => Const::Int(*i),
            Value::Big(b) => Const::BigInt(b.to_string()),
            Value::Float(f) => Const::Float(*f),
            Value::Complex(z) if z.0 == 0.0 && z.0.is_sign_positive() => Const::Imag(z.1),
            Value::Complex(z) => {
                // real + imag j
                let re = Expr::new(ExprKind::Const(Const::Float(z.0)), pos);
                let im = Expr::new(ExprKind::Const(Const::Imag(z.1)), pos);
                return Ok(ExprKind::Bin(BinOp::Add, Box::new(re), Box::new(im)));
            }
            Value::Str(s) => Const::Str(s.clone()),
            Value::Bytes(b) => Const::Bytes(Rc::from(b.as_slice())),
            Value::Tuple(items) => {
                let elts = items.iter().map(|x| Ok(Expr::new(self.constant(x, pos)?, pos))).collect::<PyResult<Vec<_>>>()?;
                return Ok(ExprKind::Tuple(elts));
            }
            other => {
                let t = self.vm.type_name(other);
                return Err(self.type_error(format!("got an invalid type in Constant: {}", t)));
            }
        }))
    }

    fn expr(&mut self, v: &Value) -> PyResult<Expr> {
        let kind = match self.kind(v) {
            Some(k) => k,
            None => {
                let t = self.vm.type_name(v);
                return Err(self.type_error(format!("expected some sort of expr, but got {}", t)));
            }
        };
        let k = kind.as_str();
        let pos = self.pos(v, "expr")?;
        let kind = match k {
            "Name" => ExprKind::Name(self.ident(v, "id", k)?),
            "Constant" => {
                let c = self.field(v, "value", k)?;
                self.constant(&c, pos)?
            }
            "JoinedStr" => ExprKind::JoinedStr(self.exprs(v, "values", k)?),
            "FormattedValue" => {
                let value = self.boxed(v, "value", k)?;
                let conv = match self.opt_field(v, "conversion")? {
                    Some(c) => self.vm.to_index(&c)?,
                    None => -1,
                };
                let conv = if conv < 0 { '\0' } else { char::from_u32(conv as u32).unwrap_or('\0') };
                let spec = self.opt_expr(v, "format_spec")?.map(Box::new);
                ExprKind::FormattedValue { value, conv, spec }
            }
            "BinOp" => {
                let left = self.boxed(v, "left", k)?;
                let op = self.binop(v, k)?;
                ExprKind::Bin(op, left, self.boxed(v, "right", k)?)
            }
            "UnaryOp" => {
                let op = match self.op_kind(v, "op", k)?.as_str() {
                    "USub" => UnOp::Neg,
                    "UAdd" => UnOp::Pos,
                    "Not" => UnOp::Not,
                    "Invert" => UnOp::Invert,
                    other => return Err(self.type_error(format!("expected some sort of unaryop, but got {}", other))),
                };
                ExprKind::Unary(op, self.boxed(v, "operand", k)?)
            }
            "BoolOp" => {
                let op = self.op_kind(v, "op", k)?;
                let values = self.exprs(v, "values", k)?;
                if values.len() < 2 {
                    return Err(self.vm.value_error("BoolOp with less than 2 values"));
                }
                match op.as_str() {
                    "And" => ExprKind::BoolAnd(values),
                    "Or" => ExprKind::BoolOr(values),
                    other => return Err(self.type_error(format!("expected some sort of boolop, but got {}", other))),
                }
            }
            "Compare" => {
                let left = self.boxed(v, "left", k)?;
                let ops = self.list_field(v, "ops", k)?;
                let comparators = self.exprs(v, "comparators", k)?;
                if ops.is_empty() {
                    return Err(self.vm.value_error("Compare with no comparators"));
                }
                if ops.len() != comparators.len() {
                    return Err(self.vm.value_error("Compare has a different number of comparators and operands"));
                }
                let mut rest = vec![];
                for (op, c) in ops.iter().zip(comparators) {
                    let name = self.kind(op).unwrap_or_default();
                    match CMPOPS.iter().find(|(_, n)| *n == name) {
                        Some((o, _)) => rest.push((*o, c)),
                        None => return Err(self.type_error(format!("expected some sort of cmpop, but got {}", name))),
                    }
                }
                ExprKind::Compare(left, rest)
            }
            "IfExp" => {
                let test = self.boxed(v, "test", k)?;
                let body = self.boxed(v, "body", k)?;
                ExprKind::IfExp(test, body, self.boxed(v, "orelse", k)?)
            }
            "Call" => {
                let func = self.boxed(v, "func", k)?;
                let mut args: Vec<Arg> = self.exprs(v, "args", k)?.into_iter().map(pos_arg).collect();
                args.extend(self.keywords(v, k)?);
                ExprKind::Call { func, args }
            }
            "Attribute" => {
                let obj = self.boxed(v, "value", k)?;
                ExprKind::Attr(obj, self.ident(v, "attr", k)?)
            }
            "Subscript" => {
                let obj = self.boxed(v, "value", k)?;
                ExprKind::Subscript(obj, self.boxed(v, "slice", k)?)
            }
            "Slice" => {
                let lo = self.opt_expr(v, "lower")?.map(Box::new);
                let hi = self.opt_expr(v, "upper")?.map(Box::new);
                ExprKind::Slice(lo, hi, self.opt_expr(v, "step")?.map(Box::new))
            }
            "List" => ExprKind::List(self.exprs(v, "elts", k)?),
            "Tuple" => ExprKind::Tuple(self.exprs(v, "elts", k)?),
            "Set" => ExprKind::Set(self.exprs(v, "elts", k)?),
            "Dict" => {
                let keys = self.list_field(v, "keys", k)?;
                let values = self.exprs(v, "values", k)?;
                if keys.len() != values.len() {
                    return Err(self.vm.value_error("Dict doesn't have the same number of keys as values"));
                }
                let mut items = vec![];
                for (key, value) in keys.iter().zip(values) {
                    let key = if key.is_none() { None } else { Some(self.expr(key)?) };
                    items.push((key, value));
                }
                ExprKind::Dict(items)
            }
            "Starred" => ExprKind::Starred(self.boxed(v, "value", k)?),
            "Lambda" => {
                let a = self.field(v, "args", k)?;
                let params = self.arguments(&a)?;
                let body = self.sub_expr(v, "body", k)?;
                let ret = Stmt { pos: body.pos, kind: StmtKind::Return(Some(body)) };
                let f = FuncDef {
                    name: Rc::from("<lambda>"),
                    params,
                    body: vec![ret],
                    decorators: vec![],
                    returns: None,
                    is_lambda: true,
                    is_async: false,
                    line: pos.line,
                    type_params: vec![],
                };
                ExprKind::Lambda(Rc::new(f))
            }
            "ListComp" | "SetComp" | "GeneratorExp" => {
                let elt = self.boxed(v, "elt", k)?;
                let comps = self.comprehensions(v, k)?;
                match k {
                    "ListComp" => ExprKind::ListComp(elt, comps),
                    "SetComp" => ExprKind::SetComp(elt, comps),
                    _ => ExprKind::GenExp(elt, comps),
                }
            }
            "DictComp" => {
                let key = self.boxed(v, "key", k)?;
                let value = self.boxed(v, "value", k)?;
                ExprKind::DictComp(key, value, self.comprehensions(v, k)?)
            }
            "NamedExpr" => {
                let t = self.field(v, "target", k)?;
                let name = self.ident(&t, "id", "Name")?;
                ExprKind::NamedExpr(name, self.boxed(v, "value", k)?)
            }
            "Yield" => ExprKind::Yield(self.opt_expr(v, "value")?.map(Box::new)),
            "YieldFrom" => ExprKind::YieldFrom(self.boxed(v, "value", k)?),
            "Await" => ExprKind::Await(self.boxed(v, "value", k)?),
            _ => return Err(self.type_error(format!("expected some sort of expr, but got <{}>", k))),
        };
        Ok(Expr::new(kind, pos))
    }

    fn patterns(&mut self, v: &Value, name: &str, owner: &str) -> PyResult<Vec<Pattern>> {
        let items = self.list_field(v, name, owner)?;
        items.iter().map(|p| self.pattern(p)).collect()
    }

    fn pattern(&mut self, v: &Value) -> PyResult<Pattern> {
        let k = self.kind(v).unwrap_or_default();
        let k = k.as_str();
        let pos = self.pos(v, "pattern")?;
        let kind = match k {
            "MatchAs" => {
                let name = self.opt_ident(v, "name")?;
                match self.opt_field(v, "pattern")? {
                    Some(p) => match name {
                        Some(n) => PatternKind::As(Box::new(self.pattern(&p)?), n),
                        None => return Err(self.vm.value_error("MatchAs must specify a target name if a pattern is given")),
                    },
                    None => PatternKind::Capture(name),
                }
            }
            "MatchValue" => PatternKind::Value(self.sub_expr(v, "value", k)?),
            "MatchSingleton" => {
                let c = self.field(v, "value", k)?;
                match c {
                    Value::None => PatternKind::Singleton(Const::None),
                    Value::Bool(true) => PatternKind::Singleton(Const::True),
                    Value::Bool(false) => PatternKind::Singleton(Const::False),
                    _ => return Err(self.vm.value_error("MatchSingleton can only contain True, False and None")),
                }
            }
            "MatchSequence" => PatternKind::Sequence(self.patterns(v, "patterns", k)?),
            "MatchStar" => PatternKind::Star(self.opt_ident(v, "name")?),
            "MatchMapping" => {
                let keys = self.exprs(v, "keys", k)?;
                let pats = self.patterns(v, "patterns", k)?;
                PatternKind::Mapping { keys, pats, rest: self.opt_ident(v, "rest")? }
            }
            "MatchClass" => {
                let cls = self.sub_expr(v, "cls", k)?;
                let args = self.patterns(v, "patterns", k)?;
                let attrs = self.list_field(v, "kwd_attrs", k)?;
                let kps = self.patterns(v, "kwd_patterns", k)?;
                let mut kw = vec![];
                for (a, p) in attrs.iter().zip(kps) {
                    match a {
                        Value::Str(s) => kw.push((Rc::from(s.s.as_str()), p)),
                        _ => return Err(self.type_error("MatchClass field \"kwd_attrs\" must be a list of str".into())),
                    }
                }
                PatternKind::Class { cls, args, kw }
            }
            "MatchOr" => PatternKind::Or(self.patterns(v, "patterns", k)?),
            _ => {
                let t = self.vm.type_name(v);
                return Err(self.type_error(format!("expected some sort of pattern, but got {}", t)));
            }
        };
        Ok(Pattern { kind, pos })
    }
}

fn pos_arg(e: Expr) -> Arg {
    if matches!(e.kind, ExprKind::Starred(_)) {
        Arg::Star(e)
    } else {
        Arg::Pos(e)
    }
}

// ---------------------------------------------------------------------------
// constant folding, for PyCF_OPTIMIZED_AST

fn fold_tree(t: Tree) -> Tree {
    match t {
        Tree::Module(mut b) => {
            crate::lower::fold_stmts(&mut b);
            Tree::Module(b)
        }
        Tree::Interactive(mut b) => {
            crate::lower::fold_stmts(&mut b);
            Tree::Interactive(b)
        }
        Tree::Expression(mut e) => {
            crate::lower::fold_expr(&mut e);
            Tree::Expression(e)
        }
    }
}
