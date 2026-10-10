// SPDX-License-Identifier: MIT
//! Rewrites, before code generation, the statements the compiler has no
//! instructions of its own for: `async for`, `async with`, and PEP 695
//! generic definitions and type aliases. The parser keeps them as written,
//! so that the ast module sees the source's own tree.

use crate::ast::*;
use crate::compile::CompileError;
use crate::value::PyStr;
use std::rc::Rc;

type CResult<T> = Result<T, CompileError>;

/// lowers the statements of a module, and of every body nested in them;
/// with `top_await` (PyCF_ALLOW_TOP_LEVEL_AWAIT) the module's own may be async
pub fn lower(body: &mut Vec<Stmt>, top_await: bool) -> CResult<()> {
    Lower { n: 0, in_async: top_await }.stmts(body)
}

struct Lower {
    /// numbers the hidden names of each rewrite apart
    n: u32,
    /// whether the statements are an async function's, or may await at top level
    in_async: bool,
}

impl Lower {
    fn stmts(&mut self, body: &mut Vec<Stmt>) -> CResult<()> {
        if !body.iter().any(needs_lowering) {
            for s in body.iter_mut() {
                self.inner(s)?;
            }
            return Ok(());
        }
        for s in std::mem::take(body) {
            self.stmt(s, body)?;
        }
        Ok(())
    }

    /// lowers the bodies inside `s`
    fn inner(&mut self, s: &mut Stmt) -> CResult<()> {
        match &mut s.kind {
            StmtKind::If(_, a, b) | StmtKind::While(_, a, b) | StmtKind::For(_, _, a, b, _) => {
                self.stmts(a)?;
                self.stmts(b)?;
            }
            StmtKind::With(_, a, _) => self.stmts(a)?,
            StmtKind::Try { body, handlers, orelse, finally, .. } => {
                self.stmts(body)?;
                for h in handlers {
                    self.stmts(&mut h.body)?;
                }
                self.stmts(orelse)?;
                self.stmts(finally)?;
            }
            StmtKind::Match(_, cases) => {
                for c in cases {
                    self.stmts(&mut c.body)?;
                }
            }
            StmtKind::FuncDef(f) => {
                let outer = std::mem::replace(&mut self.in_async, f.is_async);
                self.stmts(&mut Rc::make_mut(f).body)?;
                self.in_async = outer;
            }
            StmtKind::ClassDef(c) => {
                let outer = std::mem::replace(&mut self.in_async, false);
                self.stmts(&mut Rc::make_mut(c).body)?;
                self.in_async = outer;
            }
            _ => {}
        }
        Ok(())
    }

    /// `s`, lowered, onto `out`
    fn stmt(&mut self, mut s: Stmt, out: &mut Vec<Stmt>) -> CResult<()> {
        self.inner(&mut s)?;
        self.n += 1;
        let pos = s.pos;
        let what = match &s.kind {
            StmtKind::For(.., true) => "for",
            StmtKind::With(.., true) => "with",
            _ => "",
        };
        if !what.is_empty() && !self.in_async {
            return Err(CompileError::at(format!("'async {}' outside async function", what), pos));
        }
        match s.kind {
            StmtKind::For(target, iter, body, orelse, true) => out.push(self.async_for(pos, target, iter, body, orelse)),
            StmtKind::With(items, body, true) => out.push(self.async_with(pos, items, body)),
            StmtKind::FuncDef(mut f) if !f.type_params.is_empty() => {
                let fm = Rc::make_mut(&mut f);
                let tps = std::mem::take(&mut fm.type_params);
                let decos = std::mem::take(&mut fm.decorators);
                let name = f.name.clone();
                let body = generic_body(&name, &tps, Stmt { pos, kind: StmtKind::FuncDef(f) });
                generic_wrap(out, &name, tps, body, decos, pos);
            }
            StmtKind::ClassDef(mut c) if !c.type_params.is_empty() => {
                let cm = Rc::make_mut(&mut c);
                let tps = std::mem::take(&mut cm.type_params);
                let decos = std::mem::take(&mut cm.decorators);
                let plain: Vec<Expr> = tps
                    .iter()
                    .map(|t| if t.kind == 1 { at(pos, ExprKind::Starred(Box::new(name(pos, &t.name)))) } else { name(pos, &t.name) })
                    .collect();
                let idx = if plain.len() == 1 { plain.into_iter().next().unwrap() } else { at(pos, ExprKind::Tuple(plain)) };
                cm.bases.push(Arg::Pos(at(pos, ExprKind::Subscript(Box::new(typing_attr("Generic", pos)), Box::new(idx)))));
                let cname = c.name.clone();
                let body = generic_body(&cname, &tps, Stmt { pos, kind: StmtKind::ClassDef(c) });
                generic_wrap(out, &cname, tps, body, decos, pos);
            }
            StmtKind::TypeAlias(target, tps, value) => {
                let ExprKind::Name(n) = &target.kind else { return Ok(()) };
                let mut args = vec![Arg::Pos(str_const(n, pos)), Arg::Pos(value)];
                let make = |args| call(typing_attr("TypeAliasType", pos), args, pos);
                if tps.is_empty() {
                    out.push(Stmt { pos, kind: StmtKind::Assign(vec![target.clone()], make(args)) });
                    return Ok(());
                }
                let params = at(pos, ExprKind::Tuple(tps.iter().map(|t| name(pos, &t.name)).collect()));
                args.push(Arg::Kw(Rc::from("type_params"), params, pos));
                let ret = Stmt { pos, kind: StmtKind::Return(Some(make(args))) };
                let mut wrapped = vec![];
                generic_wrap(&mut wrapped, n, tps, vec![ret], vec![], pos);
                out.push(Stmt { pos, kind: StmtKind::If(at(pos, ExprKind::Const(Const::True)), wrapped, vec![]) });
            }
            kind => out.push(Stmt { pos, kind }),
        }
        Ok(())
    }

    /// `async for` as a loop over `await anext(it)`
    fn async_for(&mut self, pos: Pos, target: Expr, iter: Expr, body: Vec<Stmt>, orelse: Vec<Stmt>) -> Stmt {
        let it: Name = Rc::from(format!(".aiter{}", self.n));
        let call1 = |f: &str, a: Expr| call(name(pos, f), vec![Arg::Pos(a)], pos);
        let stmt = |kind| Stmt { pos, kind };
        let mut on_end = orelse;
        on_end.push(stmt(StmtKind::Break));
        let next = at(pos, ExprKind::Await(Box::new(call1("anext", name(pos, &it)))));
        let get = stmt(StmtKind::Try {
            body: vec![stmt(StmtKind::Assign(vec![target], next))],
            handlers: vec![Handler { typ: Some(name(pos, "StopAsyncIteration")), name: None, body: on_end, pos }],
            orelse: vec![],
            finally: vec![],
            star: false,
        });
        let mut loop_body = vec![get];
        loop_body.extend(body);
        let yes = || at(pos, ExprKind::Const(Const::True));
        stmt(StmtKind::If(
            yes(),
            vec![stmt(StmtKind::Assign(vec![name(pos, &it)], call1("aiter", iter))), stmt(StmtKind::While(yes(), loop_body, vec![]))],
            vec![],
        ))
    }

    /// `async with` as awaited `__aenter__`/`__aexit__` calls around a try
    fn async_with(&mut self, pos: Pos, items: Vec<(Expr, Option<Expr>)>, body: Vec<Stmt>) -> Stmt {
        let mut inner = body;
        for (k, (mgr, target)) in items.into_iter().enumerate().rev() {
            let tag = format!("{}_{}", self.n, k);
            let m: Name = Rc::from(format!(".amgr{}", tag));
            let ex: Name = Rc::from(format!(".aexit{}", tag));
            let ok: Name = Rc::from(format!(".aok{}", tag));
            let e: Name = Rc::from(format!(".aexc{}", tag));
            let n = |x: &Name| name(pos, x);
            let stmt = |kind| Stmt { pos, kind };
            let type_of = |x: Expr| call(name(pos, "type"), vec![Arg::Pos(x)], pos);
            let attr = |x: Expr, a: &str| at(pos, ExprKind::Attr(Box::new(x), Rc::from(a)));
            let callp = |f: Expr, args: Vec<Expr>| call(f, args.into_iter().map(Arg::Pos).collect(), pos);
            let awaited = |x: Expr| at(pos, ExprKind::Await(Box::new(x)));
            let cst = |c: Const| at(pos, ExprKind::Const(c));
            let mut s = vec![
                stmt(StmtKind::Assign(vec![n(&m)], mgr)),
                stmt(StmtKind::Assign(vec![n(&ex)], attr(type_of(n(&m)), "__aexit__"))),
            ];
            let enter = awaited(callp(attr(type_of(n(&m)), "__aenter__"), vec![n(&m)]));
            match target {
                Some(t) => s.push(stmt(StmtKind::Assign(vec![t], enter))),
                None => s.push(stmt(StmtKind::Expr(enter))),
            }
            s.push(stmt(StmtKind::Assign(vec![n(&ok)], cst(Const::True))));
            let exit_exc = awaited(callp(n(&ex), vec![n(&m), type_of(n(&e)), n(&e), attr(n(&e), "__traceback__")]));
            let handler = vec![
                stmt(StmtKind::Assign(vec![n(&ok)], cst(Const::False))),
                stmt(StmtKind::If(at(pos, ExprKind::Unary(UnOp::Not, Box::new(exit_exc))), vec![stmt(StmtKind::Raise(None, None))], vec![])),
            ];
            let guarded = stmt(StmtKind::Try {
                body: inner,
                handlers: vec![Handler { typ: Some(name(pos, "BaseException")), name: Some(e.clone()), body: handler, pos }],
                orelse: vec![],
                finally: vec![],
                star: false,
            });
            let exit_ok = stmt(StmtKind::Expr(awaited(callp(n(&ex), vec![n(&m), cst(Const::None), cst(Const::None), cst(Const::None)]))));
            s.push(stmt(StmtKind::Try {
                body: vec![guarded],
                handlers: vec![],
                orelse: vec![],
                finally: vec![stmt(StmtKind::If(n(&ok), vec![exit_ok], vec![]))],
                star: false,
            }));
            inner = s;
        }
        Stmt { pos, kind: StmtKind::If(at(pos, ExprKind::Const(Const::True)), inner, vec![]) }
    }
}

fn needs_lowering(s: &Stmt) -> bool {
    match &s.kind {
        StmtKind::For(.., true) | StmtKind::With(_, _, true) | StmtKind::TypeAlias(..) => true,
        StmtKind::FuncDef(f) => !f.type_params.is_empty(),
        StmtKind::ClassDef(c) => !c.type_params.is_empty(),
        _ => false,
    }
}

fn at(pos: Pos, kind: ExprKind) -> Expr {
    Expr::new(kind, pos)
}

fn name(pos: Pos, n: &str) -> Expr {
    at(pos, ExprKind::Name(Rc::from(n)))
}

fn str_const(s: &str, pos: Pos) -> Expr {
    at(pos, ExprKind::Const(Const::Str(Rc::new(PyStr::new(s.to_string())))))
}

fn call(func: Expr, args: Vec<Arg>, pos: Pos) -> Expr {
    at(pos, ExprKind::Call { func: Box::new(func), args })
}

/// typing.NAME, through __import__ so no name is bound
fn typing_attr(attr: &str, pos: Pos) -> Expr {
    let m = call(name(pos, "__import__"), vec![Arg::Pos(str_const("typing", pos))], pos);
    at(pos, ExprKind::Attr(Box::new(m), Rc::from(attr)))
}

/// the definition, then NAME.__type_params__ = (T, ...) and return NAME
fn generic_body(n: &Name, tps: &[TypeParam], def: Stmt) -> Vec<Stmt> {
    let pos = def.pos;
    let tuple = at(pos, ExprKind::Tuple(tps.iter().map(|t| name(pos, &t.name)).collect()));
    let target = at(pos, ExprKind::Attr(Box::new(name(pos, n)), Rc::from("__type_params__")));
    vec![def, Stmt { pos, kind: StmtKind::Assign(vec![target], tuple) }, Stmt { pos, kind: StmtKind::Return(Some(name(pos, n))) }]
}

/// PEP 695: the type parameters become the parameters of a hidden function
/// that is called with new TypeVars; its result, decorated, is bound to NAME
fn generic_wrap(out: &mut Vec<Stmt>, n: &Name, tps: Vec<TypeParam>, body: Vec<Stmt>, decos: Vec<Expr>, pos: Pos) {
    let wname: Name = Rc::from(format!("<generic parameters of {}>", n).as_str());
    let mut params = Params::default();
    let mut vars = vec![];
    for t in tps {
        params.args.push(Param { name: t.name.clone(), default: None, annotation: None, pos: t.pos });
        let cls = match t.kind {
            1 => "TypeVarTuple",
            2 => "ParamSpec",
            _ => "TypeVar",
        };
        let mut args = vec![Arg::Pos(str_const(&t.name, pos)), Arg::Kw(Rc::from("infer_variance"), at(pos, ExprKind::Const(Const::True)), pos)];
        if let Some(b) = t.bound {
            args.push(Arg::Kw(Rc::from("bound"), b, pos));
        }
        if let Some(d) = t.default {
            args.push(Arg::Kw(Rc::from("default"), d, pos));
        }
        vars.push(Arg::Pos(call(typing_attr(cls, pos), args, pos)));
    }
    let line = decos.first().map_or(pos.line, |d| d.pos.line);
    let wrapper = FuncDef {
        name: wname.clone(),
        params,
        body,
        decorators: vec![],
        returns: None,
        is_lambda: false,
        is_async: false,
        line,
        type_params: vec![],
    };
    out.push(Stmt { pos, kind: StmtKind::FuncDef(Rc::new(wrapper)) });
    let mut v = call(name(pos, &wname), vars, pos);
    for d in decos.into_iter().rev() {
        v = call(d, vec![Arg::Pos(v)], pos);
    }
    out.push(Stmt { pos, kind: StmtKind::Assign(vec![name(pos, n)], v) });
    out.push(Stmt { pos, kind: StmtKind::Del(vec![name(pos, &wname)]) });
}

/// calls `f` on every expression in `body`, children before parents
pub fn walk_stmts(body: &mut [Stmt], f: &mut dyn FnMut(&mut Expr)) {
    for s in body {
        walk_stmt(s, f);
    }
}

fn walk_opt(e: &mut Option<Expr>, f: &mut dyn FnMut(&mut Expr)) {
    if let Some(e) = e {
        walk_expr(e, f);
    }
}

fn walk_all(es: &mut [Expr], f: &mut dyn FnMut(&mut Expr)) {
    for e in es {
        walk_expr(e, f);
    }
}

fn walk_stmt(s: &mut Stmt, f: &mut dyn FnMut(&mut Expr)) {
    match &mut s.kind {
        StmtKind::Expr(e) => walk_expr(e, f),
        StmtKind::Return(v) => walk_opt(v, f),
        StmtKind::Assign(ts, v) => {
            walk_all(ts, f);
            walk_expr(v, f);
        }
        StmtKind::AugAssign(t, _, v) => {
            walk_expr(t, f);
            walk_expr(v, f);
        }
        StmtKind::AnnAssign(t, a, v, _) => {
            walk_expr(t, f);
            walk_expr(a, f);
            walk_opt(v, f);
        }
        StmtKind::If(c, a, b) | StmtKind::While(c, a, b) => {
            walk_expr(c, f);
            walk_stmts(a, f);
            walk_stmts(b, f);
        }
        StmtKind::For(t, it, a, b, _) => {
            walk_expr(t, f);
            walk_expr(it, f);
            walk_stmts(a, f);
            walk_stmts(b, f);
        }
        StmtKind::FuncDef(fd) => walk_func(Rc::make_mut(fd), f),
        StmtKind::ClassDef(c) => {
            let c = Rc::make_mut(c);
            walk_all(&mut c.decorators, f);
            walk_args(&mut c.bases, f);
            walk_stmts(&mut c.body, f);
        }
        StmtKind::Del(ts) => walk_all(ts, f),
        StmtKind::Raise(a, b) => {
            walk_opt(a, f);
            walk_opt(b, f);
        }
        StmtKind::Assert(a, b) => {
            walk_expr(a, f);
            walk_opt(b, f);
        }
        StmtKind::Try { body, handlers, orelse, finally, .. } => {
            walk_stmts(body, f);
            for h in handlers {
                walk_opt(&mut h.typ, f);
                walk_stmts(&mut h.body, f);
            }
            walk_stmts(orelse, f);
            walk_stmts(finally, f);
        }
        StmtKind::With(items, body, _) => {
            for (e, t) in items {
                walk_expr(e, f);
                walk_opt(t, f);
            }
            walk_stmts(body, f);
        }
        StmtKind::Match(subject, cases) => {
            walk_expr(subject, f);
            for c in cases {
                walk_opt(&mut c.guard, f);
                walk_stmts(&mut c.body, f);
            }
        }
        StmtKind::TypeAlias(_, _, v) => walk_expr(v, f),
        StmtKind::Break | StmtKind::Continue | StmtKind::Pass | StmtKind::Global(_) | StmtKind::Nonlocal(_) => {}
        StmtKind::Import(_) | StmtKind::ImportFrom { .. } => {}
    }
}

fn walk_func(fd: &mut FuncDef, f: &mut dyn FnMut(&mut Expr)) {
    walk_all(&mut fd.decorators, f);
    let p = &mut fd.params;
    for a in p.args.iter_mut().chain(p.vararg.iter_mut()).chain(p.kwonly.iter_mut()).chain(p.kwarg.iter_mut()) {
        walk_opt(&mut a.default, f);
        walk_opt(&mut a.annotation, f);
    }
    walk_opt(&mut fd.returns, f);
    walk_stmts(&mut fd.body, f);
}

fn walk_args(args: &mut [Arg], f: &mut dyn FnMut(&mut Expr)) {
    for a in args {
        match a {
            Arg::Pos(e) | Arg::Star(e) | Arg::Kw(_, e, _) | Arg::StarStar(e, _) => walk_expr(e, f),
        }
    }
}

fn walk_comps(comps: &mut [Comp], f: &mut dyn FnMut(&mut Expr)) {
    for c in comps {
        walk_expr(&mut c.target, f);
        walk_expr(&mut c.iter, f);
        walk_all(&mut c.ifs, f);
    }
}

/// calls `f` on `e` and every expression inside it, children first
pub fn walk_expr(e: &mut Expr, f: &mut dyn FnMut(&mut Expr)) {
    match &mut e.kind {
        ExprKind::Name(_) | ExprKind::Const(_) => {}
        ExprKind::JoinedStr(xs) | ExprKind::BoolAnd(xs) | ExprKind::BoolOr(xs) => walk_all(xs, f),
        ExprKind::List(xs) | ExprKind::Tuple(xs) | ExprKind::Set(xs) => walk_all(xs, f),
        ExprKind::FormattedValue { value, spec, .. } => {
            walk_expr(value, f);
            if let Some(s) = spec {
                walk_expr(s, f);
            }
        }
        ExprKind::Bin(_, a, b) | ExprKind::Subscript(a, b) => {
            walk_expr(a, f);
            walk_expr(b, f);
        }
        ExprKind::Unary(_, a) | ExprKind::Starred(a) | ExprKind::Await(a) | ExprKind::Attr(a, _) | ExprKind::YieldFrom(a) | ExprKind::NamedExpr(_, a) => {
            walk_expr(a, f)
        }
        ExprKind::Yield(a) => {
            if let Some(a) = a {
                walk_expr(a, f);
            }
        }
        ExprKind::Compare(a, rest) => {
            walk_expr(a, f);
            for (_, b) in rest {
                walk_expr(b, f);
            }
        }
        ExprKind::IfExp(c, a, b) => {
            walk_expr(c, f);
            walk_expr(a, f);
            walk_expr(b, f);
        }
        ExprKind::Call { func, args } => {
            walk_expr(func, f);
            walk_args(args, f);
        }
        ExprKind::Slice(a, b, c) => {
            for x in [a, b, c].into_iter().flatten() {
                walk_expr(x, f);
            }
        }
        ExprKind::Dict(items) => {
            for (k, v) in items {
                walk_opt(k, f);
                walk_expr(v, f);
            }
        }
        ExprKind::Lambda(fd) => walk_func(Rc::make_mut(fd), f),
        ExprKind::ListComp(elt, comps) | ExprKind::SetComp(elt, comps) | ExprKind::GenExp(elt, comps) => {
            walk_expr(elt, f);
            walk_comps(comps, f);
        }
        ExprKind::DictComp(k, v, comps) => {
            walk_expr(k, f);
            walk_expr(v, f);
            walk_comps(comps, f);
        }
    }
    f(e);
}

/// folds the constant arithmetic in `body`, as CPython's AST optimizer does
pub fn fold_stmts(body: &mut [Stmt]) {
    walk_stmts(body, &mut fold);
}

pub fn fold_expr(e: &mut Expr) {
    walk_expr(e, &mut fold);
}

/// a number, as an operand of folding
#[derive(Clone, Copy)]
enum Num {
    Int(i64),
    Float(f64),
}

fn num(e: &Expr) -> Option<Num> {
    match &e.kind {
        ExprKind::Const(Const::Int(i)) => Some(Num::Int(*i)),
        ExprKind::Const(Const::Float(f)) => Some(Num::Float(*f)),
        _ => None,
    }
}

/// `e` replaced by its value when its operands are constants and the
/// operation cannot fail; anything else is left to run
fn fold(e: &mut Expr) {
    let c = match &e.kind {
        ExprKind::Unary(op, a) => match (op, num(a), &a.kind) {
            (UnOp::Neg, Some(Num::Int(i)), _) => i.checked_neg().map(Const::Int),
            (UnOp::Neg, Some(Num::Float(x)), _) => Some(Const::Float(-x)),
            (UnOp::Pos, Some(Num::Int(i)), _) => Some(Const::Int(i)),
            (UnOp::Pos, Some(Num::Float(x)), _) => Some(Const::Float(x)),
            (UnOp::Invert, Some(Num::Int(i)), _) => Some(Const::Int(!i)),
            (UnOp::Not, _, ExprKind::Const(k)) => truth(k).map(|t| if t { Const::False } else { Const::True }),
            _ => None,
        },
        ExprKind::Bin(op, a, b) => match (&a.kind, &b.kind) {
            (ExprKind::Const(Const::Str(x)), ExprKind::Const(Const::Str(y))) if *op == BinOp::Add => {
                Some(Const::Str(Rc::new(PyStr::from_wtf8([x.wtf8(), y.wtf8()].concat()))))
            }
            _ => match (num(a), num(b)) {
                (Some(x), Some(y)) => fold_num(*op, x, y),
                _ => None,
            },
        },
        _ => None,
    };
    if let Some(c) = c {
        e.kind = ExprKind::Const(c);
    }
}

fn truth(k: &Const) -> Option<bool> {
    Some(match k {
        Const::None | Const::False => false,
        Const::True | Const::Ellipsis => true,
        Const::Int(i) => *i != 0,
        Const::Float(f) => *f != 0.0,
        Const::Str(s) => !s.s.is_empty(),
        Const::Bytes(b) => !b.is_empty(),
        _ => return None,
    })
}

fn fold_num(op: BinOp, x: Num, y: Num) -> Option<Const> {
    if let (Num::Int(a), Num::Int(b)) = (x, y) {
        return match op {
            BinOp::Add => a.checked_add(b),
            BinOp::Sub => a.checked_sub(b),
            BinOp::Mul => a.checked_mul(b),
            BinOp::FloorDiv if b != 0 => a.checked_div_euclid(b).map(|q| if b < 0 && a.rem_euclid(b) != 0 { q - 1 } else { q }),
            BinOp::Mod if b != 0 => a.checked_rem(b).map(|r| if r != 0 && (r < 0) != (b < 0) { r + b } else { r }),
            BinOp::Pow if (0..=64).contains(&b) => a.checked_pow(b as u32),
            BinOp::LShift if (0..63).contains(&b) => a.checked_shl(b as u32).filter(|v| v >> b == a),
            BinOp::RShift if b >= 0 => Some(a >> b.min(63)),
            BinOp::BitAnd => Some(a & b),
            BinOp::BitOr => Some(a | b),
            BinOp::BitXor => Some(a ^ b),
            _ => None,
        }
        .map(Const::Int);
    }
    let f = |n: Num| match n {
        Num::Int(i) => i as f64,
        Num::Float(x) => x,
    };
    let (a, b) = (f(x), f(y));
    let v = match op {
        BinOp::Add => a + b,
        BinOp::Sub => a - b,
        BinOp::Mul => a * b,
        BinOp::Div if b != 0.0 => a / b,
        _ => return None,
    };
    Some(Const::Float(v))
}
