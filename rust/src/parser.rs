// SPDX-License-Identifier: MIT
//! Recursive-descent parser: tokens to the syntax tree in ast.rs.

use crate::ast::*;
use crate::lexer::{tokenize, unescape, Tok, Token};
use std::rc::Rc;

pub struct SyntaxError {
    pub msg: String,
    pub line: u32,
    /// 1-based column, 0 when unknown
    pub col: u32,
}

type PResult<T> = Result<T, SyntaxError>;

pub struct Parser {
    toks: Vec<Token>,
    pos: usize,
    /// PEP 695 type parameters of the def or class just parsed
    tparams: Option<Vec<TypeParam>>,
}

/// `T`, `T: bound`, `T = default`, `*Ts` (kind 1) or `**P` (kind 2)
struct TypeParam {
    kind: u8,
    name: Name,
    bound: Option<Expr>,
    default: Option<Expr>,
}

pub fn parse_module(src: &str) -> PResult<Vec<Stmt>> {
    let toks = tokenize(src).map_err(|e| SyntaxError { msg: e.msg, line: e.line, col: e.col })?;
    let mut p = Parser { toks, pos: 0, tparams: None };
    let mut body = Vec::new();
    while !p.at_eof() {
        if p.eat_tok(&Tok::Newline) {
            continue;
        }
        p.statement(&mut body)?;
    }
    Ok(body)
}

pub fn parse_expression(src: &str) -> PResult<Expr> {
    let toks = tokenize(src.trim()).map_err(|e| SyntaxError { msg: e.msg, line: e.line, col: e.col })?;
    let mut p = Parser { toks, pos: 0, tparams: None };
    let e = p.expressions()?;
    while p.eat_tok(&Tok::Newline) {}
    if !p.at_eof() {
        return Err(p.err("invalid syntax"));
    }
    Ok(e)
}

fn is_op(t: &Tok, o: &str) -> bool {
    matches!(t, Tok::Op(x) if *x == o)
}

impl Parser {
    fn peek(&self) -> &Tok {
        &self.toks[self.pos].tok
    }
    fn peek_at(&self, k: usize) -> &Tok {
        let i = (self.pos + k).min(self.toks.len() - 1);
        &self.toks[i].tok
    }
    fn line(&self) -> u32 {
        self.toks[self.pos].line
    }
    fn at_eof(&self) -> bool {
        matches!(self.peek(), Tok::Eof)
    }
    fn advance(&mut self) -> Tok {
        let t = self.toks[self.pos].tok.clone();
        if self.pos < self.toks.len() - 1 {
            self.pos += 1;
        }
        t
    }
    fn err(&self, msg: &str) -> SyntaxError {
        SyntaxError { msg: msg.to_string(), line: self.line(), col: 0 }
    }
    fn at_op(&self, o: &str) -> bool {
        is_op(self.peek(), o)
    }
    fn eat_op(&mut self, o: &str) -> bool {
        if self.at_op(o) {
            self.advance();
            true
        } else {
            false
        }
    }
    fn eat_tok(&mut self, t: &Tok) -> bool {
        if self.peek() == t {
            self.advance();
            true
        } else {
            false
        }
    }
    fn expect_op(&mut self, o: &str) -> PResult<()> {
        if self.eat_op(o) {
            Ok(())
        } else {
            match self.peek() {
                Tok::Newline | Tok::Eof if o == ")" || o == "]" || o == "}" => Err(self.err(&format!("'{}' was never closed", match o {
                    ")" => "(",
                    "]" => "[",
                    _ => "{",
                }))),
                _ if (o == ")" || o == "]" || o == "}") && self.starts_expr() && !self.after_name_before_str() => Err(self.err("invalid syntax. Perhaps you forgot a comma?")),
                _ => Err(self.err(if o == ":" { "expected ':'" } else { "invalid syntax" })),
            }
        }
    }
    /// `name "string"`, which CPython does not take for a missing comma
    fn after_name_before_str(&self) -> bool {
        let prev = if self.pos > 0 { self.toks.get(self.pos - 1).map(|t| &t.tok) } else { None };
        matches!(prev, Some(Tok::Name(_))) && matches!(self.peek(), Tok::Str { .. })
    }
    fn at_kw(&self, k: &str) -> bool {
        matches!(self.peek(), Tok::Name(n) if &**n == k)
    }
    fn eat_kw(&mut self, k: &str) -> bool {
        if self.at_kw(k) {
            self.advance();
            true
        } else {
            false
        }
    }
    fn expect_kw(&mut self, k: &str) -> PResult<()> {
        if self.eat_kw(k) {
            Ok(())
        } else {
            Err(self.err("invalid syntax"))
        }
    }
    fn name(&mut self) -> PResult<Name> {
        match self.peek().clone() {
            Tok::Name(n) if !is_keyword(&n) => {
                self.advance();
                Ok(n)
            }
            _ => Err(self.err("invalid syntax")),
        }
    }

    // ---------------- statements ----------------

    fn block(&mut self) -> PResult<Vec<Stmt>> {
        self.expect_op(":")?;
        let mut body = Vec::new();
        if self.eat_tok(&Tok::Newline) {
            if !self.eat_tok(&Tok::Indent) {
                return Err(self.err(&self.indent_msg()));
            }
            while !self.eat_tok(&Tok::Dedent) {
                if self.at_eof() {
                    break;
                }
                if self.eat_tok(&Tok::Newline) {
                    continue;
                }
                self.statement(&mut body)?;
            }
        } else {
            self.simple_statements(&mut body)?;
        }
        Ok(body)
    }

    /// CPython's message for a block header with no indented body
    fn indent_msg(&self) -> String {
        let mut i = self.pos.saturating_sub(1);
        while i > 0 && !matches!(self.toks[i - 1].tok, Tok::Newline | Tok::Indent | Tok::Dedent) {
            i -= 1;
        }
        let kw = |k: usize| match self.toks.get(k).map(|t| &t.tok) {
            Some(Tok::Name(n)) => n.to_string(),
            _ => String::new(),
        };
        let mut k = kw(i);
        if k == "async" {
            k = kw(i + 1);
        }
        let line = self.toks.get(i).map(|t| t.line).unwrap_or(0);
        let what = match &*k {
            "def" => "function definition".to_string(),
            "class" => "class definition".to_string(),
            "if" | "elif" | "else" | "while" | "for" | "try" | "except" | "finally" | "with" | "match" | "case" => format!("'{}' statement", k),
            _ => return "expected an indented block".into(),
        };
        format!("expected an indented block after {} on line {}", what, line)
    }

    fn statement(&mut self, out: &mut Vec<Stmt>) -> PResult<()> {
        let line = self.line();
        if let Tok::Name(n) = self.peek().clone() {
            match &*n {
                "if" => {
                    self.advance();
                    let s = self.if_rest(line)?;
                    out.push(s);
                    return Ok(());
                }
                "while" => {
                    self.advance();
                    let cond = self.named_expr()?;
                    let body = self.block()?;
                    let orelse = if self.eat_kw("else") { self.block()? } else { vec![] };
                    out.push(Stmt { line, kind: StmtKind::While(cond, body, orelse) });
                    return Ok(());
                }
                "for" => {
                    self.advance();
                    out.push(self.for_rest(line)?);
                    return Ok(());
                }
                "try" => {
                    self.advance();
                    out.push(self.try_rest(line)?);
                    return Ok(());
                }
                "with" => {
                    self.advance();
                    out.push(self.with_rest(line)?);
                    return Ok(());
                }
                "def" => {
                    self.advance();
                    let f = self.funcdef(vec![], line, false)?;
                    self.push_func(out, f, line);
                    return Ok(());
                }
                "class" => {
                    self.advance();
                    let c = self.classdef(vec![], line)?;
                    self.push_class(out, c, line);
                    return Ok(());
                }
                "async" => {
                    self.advance();
                    if self.eat_kw("def") {
                        let f = self.funcdef(vec![], line, true)?;
                        self.push_func(out, f, line);
                        return Ok(());
                    }
                    if self.eat_kw("for") {
                        let st = self.for_rest(line)?;
                        out.push(self.async_for(st));
                        return Ok(());
                    }
                    if self.eat_kw("with") {
                        let st = self.with_rest(line)?;
                        out.push(self.async_with(st));
                        return Ok(());
                    }
                    return Err(self.err("invalid syntax"));
                }
                "match" => {
                    if let Some(s) = self.try_match(line)? {
                        out.push(s);
                        return Ok(());
                    }
                }
                _ => {}
            }
        }
        if self.at_op("@") {
            let mut decos = Vec::new();
            while self.eat_op("@") {
                decos.push(self.named_expr()?);
                if !self.eat_tok(&Tok::Newline) {
                    return Err(self.err("invalid syntax"));
                }
            }
            let line = self.line();
            let is_async = self.eat_kw("async");
            if self.eat_kw("def") {
                let f = self.funcdef(decos, line, is_async)?;
                self.push_func(out, f, line);
            } else if self.eat_kw("class") {
                let c = self.classdef(decos, line)?;
                self.push_class(out, c, line);
            } else {
                return Err(self.err("invalid syntax"));
            }
            return Ok(());
        }
        self.simple_statements(out)
    }

    fn simple_statements(&mut self, out: &mut Vec<Stmt>) -> PResult<()> {
        loop {
            let line = self.line();
            let s = self.small_statement()?;
            out.push(Stmt { line, kind: s });
            if self.eat_op(";") {
                if matches!(self.peek(), Tok::Newline | Tok::Eof) {
                    break;
                }
                continue;
            }
            break;
        }
        if !self.eat_tok(&Tok::Newline) && !self.at_eof() {
            if let Some(Stmt { kind: StmtKind::Expr(Expr::Name(n)), .. }) = out.last() {
                if (&**n == "print" || &**n == "exec") && self.starts_expr() {
                    return Err(self.err(&format!("Missing parentheses in call to '{}'. Did you mean {}(...)?", n, n)));
                }
            }
            return Err(self.err("invalid syntax"));
        }
        Ok(())
    }

    /// whether the next token can begin an expression
    fn starts_expr(&self) -> bool {
        match self.peek() {
            Tok::Name(n) => !is_keyword(n) || matches!(&**n, "None" | "True" | "False" | "not" | "lambda" | "await"),
            Tok::Int(_) | Tok::BigInt(_) | Tok::Float(_) | Tok::Imag(_) | Tok::Str { .. } => true,
            Tok::Op(o) => matches!(*o, "(" | "[" | "{" | "-" | "+" | "~" | "..."),
            _ => false,
        }
    }

    fn if_rest(&mut self, line: u32) -> PResult<Stmt> {
        let cond = self.named_expr()?;
        let body = self.block()?;
        let orelse = if self.at_kw("elif") {
            let l = self.line();
            self.advance();
            vec![self.if_rest(l)?]
        } else if self.eat_kw("else") {
            self.block()?
        } else {
            vec![]
        };
        Ok(Stmt { line, kind: StmtKind::If(cond, body, orelse) })
    }

    fn for_rest(&mut self, line: u32) -> PResult<Stmt> {
        let target = self.target_list()?;
        self.expect_kw("in")?;
        let iter = self.expressions()?;
        let body = self.block()?;
        let orelse = if self.eat_kw("else") { self.block()? } else { vec![] };
        Ok(Stmt { line, kind: StmtKind::For(target, iter, body, orelse) })
    }

    /// `async for` as a loop over `await anext(it)`
    fn async_for(&self, st: Stmt) -> Stmt {
        let line = st.line;
        let StmtKind::For(target, iter, body, orelse) = st.kind else { return st };
        let it: Name = Rc::from(format!(".aiter{}", self.pos));
        let name = |n: &str| Expr::Name(Rc::from(n));
        let call = |f: &str, a: Expr| Expr::Call { func: Box::new(name(f)), args: vec![Arg::Pos(a)] };
        let stmt = |kind| Stmt { line, kind };
        let mut on_end = orelse;
        on_end.push(stmt(StmtKind::Break));
        let get = stmt(StmtKind::Try {
            body: vec![stmt(StmtKind::Assign(vec![target], Expr::Await(Box::new(call("anext", Expr::Name(it.clone()))))))],
            handlers: vec![Handler { typ: Some(name("StopAsyncIteration")), name: None, body: on_end, line }],
            orelse: vec![],
            finally: vec![],
            star: false,
        });
        let mut loop_body = vec![get];
        loop_body.extend(body);
        stmt(StmtKind::If(
            Expr::Const(Const::True),
            vec![
                stmt(StmtKind::Assign(vec![Expr::Name(it.clone())], call("aiter", iter))),
                stmt(StmtKind::While(Expr::Const(Const::True), loop_body, vec![])),
            ],
            vec![],
        ))
    }

    /// `async with` as awaited `__aenter__`/`__aexit__` calls around a try
    fn async_with(&self, st: Stmt) -> Stmt {
        let line = st.line;
        let StmtKind::With(items, body) = st.kind else { return st };
        let mut inner = body;
        for (k, (mgr, target)) in items.into_iter().enumerate().rev() {
            let tag = format!("{}_{}", self.pos, k);
            let m: Name = Rc::from(format!(".amgr{}", tag));
            let ex: Name = Rc::from(format!(".aexit{}", tag));
            let ok: Name = Rc::from(format!(".aok{}", tag));
            let e: Name = Rc::from(format!(".aexc{}", tag));
            let n = |x: &Name| Expr::Name(x.clone());
            let stmt = |kind| Stmt { line, kind };
            let type_of = |x: Expr| Expr::Call { func: Box::new(Expr::Name(Rc::from("type"))), args: vec![Arg::Pos(x)] };
            let call = |f: Expr, args: Vec<Expr>| Expr::Call { func: Box::new(f), args: args.into_iter().map(Arg::Pos).collect() };
            let none = || Expr::Const(Const::None);
            let mut s = vec![
                stmt(StmtKind::Assign(vec![n(&m)], mgr)),
                stmt(StmtKind::Assign(vec![n(&ex)], Expr::Attr(Box::new(type_of(n(&m))), Rc::from("__aexit__")))),
            ];
            let enter = Expr::Await(Box::new(call(Expr::Attr(Box::new(type_of(n(&m))), Rc::from("__aenter__")), vec![n(&m)])));
            match target {
                Some(t) => s.push(stmt(StmtKind::Assign(vec![t], enter))),
                None => s.push(stmt(StmtKind::Expr(enter))),
            }
            s.push(stmt(StmtKind::Assign(vec![n(&ok)], Expr::Const(Const::True))));
            let exit_exc = Expr::Await(Box::new(call(n(&ex), vec![n(&m), type_of(n(&e)), n(&e), Expr::Attr(Box::new(n(&e)), Rc::from("__traceback__"))])));
            let handler = vec![
                stmt(StmtKind::Assign(vec![n(&ok)], Expr::Const(Const::False))),
                stmt(StmtKind::If(Expr::Unary(UnOp::Not, Box::new(exit_exc)), vec![stmt(StmtKind::Raise(None, None))], vec![])),
            ];
            let guarded = stmt(StmtKind::Try {
                body: inner,
                handlers: vec![Handler { typ: Some(Expr::Name(Rc::from("BaseException"))), name: Some(e.clone()), body: handler, line }],
                orelse: vec![],
                finally: vec![],
                star: false,
            });
            let exit_ok = stmt(StmtKind::Expr(Expr::Await(Box::new(call(n(&ex), vec![n(&m), none(), none(), none()])))));
            s.push(stmt(StmtKind::Try {
                body: vec![guarded],
                handlers: vec![],
                orelse: vec![],
                finally: vec![stmt(StmtKind::If(n(&ok), vec![exit_ok], vec![]))],
                star: false,
            }));
            inner = s;
        }
        Stmt { line, kind: StmtKind::If(Expr::Const(Const::True), inner, vec![]) }
    }

    fn try_rest(&mut self, line: u32) -> PResult<Stmt> {
        let body = self.block()?;
        let mut handlers = Vec::new();
        let mut star = false;
        while self.at_kw("except") {
            let hl = self.line();
            self.advance();
            if self.eat_op("*") {
                star = true;
            }
            let mut typ = None;
            let mut name = None;
            if !self.at_op(":") {
                let t = self.expression()?;
                let t = if self.at_op(",") {
                    let mut items = vec![t];
                    while self.eat_op(",") {
                        if self.at_kw("as") || self.at_op(":") {
                            break;
                        }
                        items.push(self.expression()?);
                    }
                    Expr::Tuple(items)
                } else {
                    t
                };
                typ = Some(t);
                if self.eat_kw("as") {
                    name = Some(self.name()?);
                }
            }
            let hbody = self.block()?;
            handlers.push(Handler { typ, name, body: hbody, line: hl });
        }
        let orelse = if self.eat_kw("else") { self.block()? } else { vec![] };
        let finally = if self.eat_kw("finally") { self.block()? } else { vec![] };
        if handlers.is_empty() && finally.is_empty() {
            return Err(self.err("expected 'except' or 'finally' block"));
        }
        Ok(Stmt { line, kind: StmtKind::Try { body, handlers, orelse, finally, star } })
    }

    fn with_rest(&mut self, line: u32) -> PResult<Stmt> {
        let mut items = Vec::new();
        let paren = self.at_op("(") && self.paren_with_items();
        if paren {
            self.advance();
        }
        loop {
            if paren && self.at_op(")") {
                break;
            }
            let e = self.expression()?;
            let t = if self.eat_kw("as") { Some(self.target()?) } else { None };
            items.push((e, t));
            if !self.eat_op(",") {
                break;
            }
        }
        if paren {
            self.expect_op(")")?;
        }
        let body = self.block()?;
        Ok(Stmt { line, kind: StmtKind::With(items, body) })
    }

    /// `with (a as b, c as d):` — a parenthesised item list, told apart from a
    /// parenthesised expression by an `as` at depth one
    fn paren_with_items(&self) -> bool {
        let mut depth = 0;
        let mut k = self.pos;
        while k < self.toks.len() {
            match &self.toks[k].tok {
                Tok::Op("(") | Tok::Op("[") | Tok::Op("{") => depth += 1,
                Tok::Op(")") | Tok::Op("]") | Tok::Op("}") => {
                    depth -= 1;
                    if depth == 0 {
                        return matches!(self.toks.get(k + 1).map(|t| &t.tok), Some(Tok::Op(":")));
                    }
                }
                Tok::Name(n) if &**n == "as" && depth == 1 => return true,
                Tok::Newline | Tok::Eof => return false,
                _ => {}
            }
            k += 1;
        }
        false
    }

    fn type_params(&mut self) -> PResult<Vec<TypeParam>> {
        self.expect_op("[")?;
        let mut out = vec![];
        while !self.eat_op("]") {
            let kind = if self.eat_op("**") {
                2
            } else if self.eat_op("*") {
                1
            } else {
                0
            };
            let name = self.name()?;
            let bound = if kind == 0 && self.eat_op(":") { Some(self.expression()?) } else { None };
            let default = if self.eat_op("=") { Some(self.expression()?) } else { None };
            out.push(TypeParam { kind, name, bound, default });
            if !self.eat_op(",") {
                self.expect_op("]")?;
                break;
            }
        }
        Ok(out)
    }

    fn push_func(&mut self, out: &mut Vec<Stmt>, mut f: FuncDef, line: u32) {
        match self.tparams.take() {
            None => out.push(Stmt { line, kind: StmtKind::FuncDef(Rc::new(f)) }),
            Some(tps) => {
                let decos = std::mem::take(&mut f.decorators);
                let name = f.name.clone();
                let body = generic_body(&name, &tps, Stmt { line, kind: StmtKind::FuncDef(Rc::new(f)) }, line);
                generic_wrap(out, &name, tps, body, decos, line);
            }
        }
    }

    fn push_class(&mut self, out: &mut Vec<Stmt>, mut c: ClassDef, line: u32) {
        match self.tparams.take() {
            None => out.push(Stmt { line, kind: StmtKind::ClassDef(Rc::new(c)) }),
            Some(tps) => {
                let decos = std::mem::take(&mut c.decorators);
                let name = c.name.clone();
                let plain: Vec<Expr> = tps.iter().map(|t| if t.kind == 1 { Expr::Starred(Box::new(Expr::Name(t.name.clone()))) } else { Expr::Name(t.name.clone()) }).collect();
                let idx = if plain.len() == 1 { plain.into_iter().next().unwrap() } else { Expr::Tuple(plain) };
                c.bases.push(Arg::Pos(Expr::Subscript(Box::new(typing_attr("Generic")), Box::new(idx))));
                let body = generic_body(&name, &tps, Stmt { line, kind: StmtKind::ClassDef(Rc::new(c)) }, line);
                generic_wrap(out, &name, tps, body, decos, line);
            }
        }
    }

    fn funcdef(&mut self, decorators: Vec<Expr>, line: u32, is_async: bool) -> PResult<FuncDef> {
        let name = self.name()?;
        let tparams = if self.at_op("[") { Some(self.type_params()?) } else { None };
        self.expect_op("(")?;
        let params = self.params(")", true)?;
        self.expect_op(")")?;
        let returns = if self.eat_op("->") { Some(self.expression()?) } else { None };
        let body = self.block()?;
        self.tparams = tparams;
        Ok(FuncDef { name, params, body, decorators, returns, is_lambda: false, is_async, line })
    }

    fn params(&mut self, end: &str, annotations: bool) -> PResult<Params> {
        let mut p = Params::default();
        let mut seen_star = false;
        let mut seen_default = false;
        while !self.at_op(end) {
            if self.eat_op("/") {
                p.posonly = p.args.len();
            } else if self.eat_op("**") {
                let name = self.name()?;
                let annotation = if annotations && self.eat_op(":") { Some(self.expression()?) } else { None };
                p.kwarg = Some(Param { name, default: None, annotation });
            } else if self.eat_op("*") {
                seen_star = true;
                if !self.at_op(",") && !self.at_op(end) {
                    let name = self.name()?;
                    let annotation = if annotations && self.eat_op(":") { Some(self.star_or_expr()?) } else { None };
                    p.vararg = Some(Param { name, default: None, annotation });
                }
            } else {
                let name = self.name()?;
                let annotation = if annotations && self.eat_op(":") { Some(self.expression()?) } else { None };
                let default = if self.eat_op("=") { Some(self.expression()?) } else { None };
                if seen_star {
                    p.kwonly.push(Param { name, default, annotation });
                } else {
                    if default.is_some() {
                        seen_default = true;
                    } else if seen_default {
                        return Err(self.err("non-default argument follows default argument"));
                    }
                    p.args.push(Param { name, default, annotation });
                }
            }
            if !self.eat_op(",") {
                break;
            }
        }
        Ok(p)
    }

    fn star_or_expr(&mut self) -> PResult<Expr> {
        if self.eat_op("*") {
            Ok(Expr::Starred(Box::new(self.expression()?)))
        } else {
            self.expression()
        }
    }

    fn classdef(&mut self, decorators: Vec<Expr>, line: u32) -> PResult<ClassDef> {
        let name = self.name()?;
        let tparams = if self.at_op("[") { Some(self.type_params()?) } else { None };
        let bases = if self.eat_op("(") {
            let a = self.call_args()?;
            self.expect_op(")")?;
            a
        } else {
            vec![]
        };
        let body = self.block()?;
        self.tparams = tparams;
        Ok(ClassDef { name, bases, body, decorators, line })
    }

    fn small_statement(&mut self) -> PResult<StmtKind> {
        if let Tok::Name(n) = self.peek().clone() {
            match &*n {
                "pass" => {
                    self.advance();
                    return Ok(StmtKind::Pass);
                }
                "break" => {
                    self.advance();
                    return Ok(StmtKind::Break);
                }
                "continue" => {
                    self.advance();
                    return Ok(StmtKind::Continue);
                }
                "return" => {
                    self.advance();
                    if matches!(self.peek(), Tok::Newline | Tok::Eof) || self.at_op(";") {
                        return Ok(StmtKind::Return(None));
                    }
                    return Ok(StmtKind::Return(Some(self.star_expressions()?)));
                }
                "global" | "nonlocal" => {
                    self.advance();
                    let mut names = vec![self.name()?];
                    while self.eat_op(",") {
                        names.push(self.name()?);
                    }
                    return Ok(if &*n == "global" { StmtKind::Global(names) } else { StmtKind::Nonlocal(names) });
                }
                "del" => {
                    self.advance();
                    let mut ts = vec![self.target()?];
                    while self.eat_op(",") {
                        if matches!(self.peek(), Tok::Newline) {
                            break;
                        }
                        ts.push(self.target()?);
                    }
                    return Ok(StmtKind::Del(ts));
                }
                "raise" => {
                    self.advance();
                    if matches!(self.peek(), Tok::Newline | Tok::Eof) || self.at_op(";") {
                        return Ok(StmtKind::Raise(None, None));
                    }
                    let e = self.expression()?;
                    let cause = if self.eat_kw("from") { Some(self.expression()?) } else { None };
                    return Ok(StmtKind::Raise(Some(e), cause));
                }
                "assert" => {
                    self.advance();
                    let e = self.expression()?;
                    let msg = if self.eat_op(",") { Some(self.expression()?) } else { None };
                    return Ok(StmtKind::Assert(e, msg));
                }
                "import" => {
                    self.advance();
                    let mut names = Vec::new();
                    loop {
                        let m = self.dotted_name()?;
                        let alias = if self.eat_kw("as") { Some(self.name()?) } else { None };
                        names.push((m, alias));
                        if !self.eat_op(",") {
                            break;
                        }
                    }
                    return Ok(StmtKind::Import(names));
                }
                "from" => {
                    self.advance();
                    let mut level = 0;
                    loop {
                        if self.eat_op(".") {
                            level += 1;
                        } else if self.eat_op("...") {
                            level += 3;
                        } else {
                            break;
                        }
                    }
                    let module: Name = if self.at_kw("import") { Rc::from("") } else { self.dotted_name()? };
                    self.expect_kw("import")?;
                    let mut names = Vec::new();
                    if self.eat_op("*") {
                        names.push((Rc::from("*"), None));
                    } else {
                        let paren = self.eat_op("(");
                        loop {
                            if paren && self.at_op(")") {
                                break;
                            }
                            let nm = self.name()?;
                            let alias = if self.eat_kw("as") { Some(self.name()?) } else { None };
                            names.push((nm, alias));
                            if !self.eat_op(",") {
                                break;
                            }
                        }
                        if paren {
                            self.expect_op(")")?;
                        }
                    }
                    return Ok(StmtKind::ImportFrom { module, names, level });
                }
                "type" => {
                    // `type X = ...` (PEP 695)
                    if matches!(self.peek_at(1), Tok::Name(_)) && matches!(self.peek_at(2), Tok::Op("=") | Tok::Op("[")) {
                        self.advance();
                        let line = self.line();
                        let name = self.name()?;
                        let tps = if self.at_op("[") { self.type_params()? } else { vec![] };
                        self.expect_op("=")?;
                        let v = self.expression()?;
                        let mut args = vec![Arg::Pos(str_const(&name)), Arg::Pos(v)];
                        if tps.is_empty() {
                            return Ok(StmtKind::Assign(vec![Expr::Name(name)], call(typing_attr("TypeAliasType"), args)));
                        }
                        args.push(Arg::Kw(Rc::from("type_params"), Expr::Tuple(tps.iter().map(|t| Expr::Name(t.name.clone())).collect())));
                        let ret = Stmt { line, kind: StmtKind::Return(Some(call(typing_attr("TypeAliasType"), args))) };
                        let mut out = vec![];
                        generic_wrap(&mut out, &name, tps, vec![ret], vec![], line);
                        return Ok(StmtKind::If(Expr::Const(Const::True), out, vec![]));
                    }
                }
                _ => {}
            }
        }
        // expression statement / assignment
        let first = self.star_expressions_or_yield()?;
        if self.eat_op(":") {
            let ann = self.expression()?;
            let value = if self.eat_op("=") { Some(self.star_expressions_or_yield()?) } else { None };
            check_target(&first).map_err(|m| self.err(&m))?;
            return Ok(StmtKind::AnnAssign(first, ann, value));
        }
        if let Tok::Op(o) = self.peek().clone() {
            let aug = match o {
                "+=" => Some(BinOp::Add),
                "-=" => Some(BinOp::Sub),
                "*=" => Some(BinOp::Mul),
                "/=" => Some(BinOp::Div),
                "//=" => Some(BinOp::FloorDiv),
                "%=" => Some(BinOp::Mod),
                "**=" => Some(BinOp::Pow),
                "<<=" => Some(BinOp::LShift),
                ">>=" => Some(BinOp::RShift),
                "|=" => Some(BinOp::BitOr),
                "^=" => Some(BinOp::BitXor),
                "&=" => Some(BinOp::BitAnd),
                "@=" => Some(BinOp::MatMul),
                _ => None,
            };
            if let Some(op) = aug {
                self.advance();
                match &first {
                    Expr::Name(_) | Expr::Attr(..) | Expr::Subscript(..) => {}
                    _ => return Err(self.err("'expression' is an illegal expression for augmented assignment")),
                }
                let v = self.star_expressions_or_yield()?;
                return Ok(StmtKind::AugAssign(first, op, v));
            }
        }
        if self.at_op("=") {
            let mut targets = vec![first];
            let mut value;
            loop {
                self.advance();
                value = self.star_expressions_or_yield()?;
                if self.at_op("=") {
                    targets.push(value);
                    continue;
                }
                break;
            }
            for t in &targets {
                check_target(t).map_err(|m| self.err(&m))?;
            }
            return Ok(StmtKind::Assign(targets, value));
        }
        Ok(StmtKind::Expr(first))
    }

    fn dotted_name(&mut self) -> PResult<Name> {
        let mut s = self.name()?.to_string();
        while self.eat_op(".") {
            s.push('.');
            s.push_str(&self.name()?);
        }
        Ok(Rc::from(s.as_str()))
    }

    // ---------------- match ----------------

    fn try_match(&mut self, line: u32) -> PResult<Option<Stmt>> {
        // `match` is a soft keyword: only a statement when a block follows
        let save = self.pos;
        self.advance();
        if matches!(self.peek(), Tok::Op("=") | Tok::Op(".") | Tok::Op("(") | Tok::Newline) && !self.at_op("(") {
            self.pos = save;
            return Ok(None);
        }
        let subject = match self.star_expressions() {
            Ok(e) => e,
            Err(_) => {
                self.pos = save;
                return Ok(None);
            }
        };
        if !self.at_op(":") {
            self.pos = save;
            return Ok(None);
        }
        self.advance();
        if !self.eat_tok(&Tok::Newline) || !self.eat_tok(&Tok::Indent) {
            return Err(self.err("expected an indented block"));
        }
        let mut cases = Vec::new();
        while !self.eat_tok(&Tok::Dedent) {
            if self.eat_tok(&Tok::Newline) {
                continue;
            }
            if !self.eat_kw("case") {
                return Err(self.err("invalid syntax"));
            }
            let pat = self.pattern_top()?;
            let guard = if self.eat_kw("if") { Some(self.named_expr()?) } else { None };
            let body = self.block()?;
            cases.push(MatchCase { pat, guard, body });
        }
        Ok(Some(Stmt { line, kind: StmtKind::Match(subject, cases) }))
    }

    fn pattern_top(&mut self) -> PResult<Pattern> {
        let p = self.pattern_as()?;
        if self.at_op(",") {
            let mut items = vec![p];
            while self.eat_op(",") {
                if self.at_op(":") || self.at_kw("if") {
                    break;
                }
                items.push(self.pattern_as()?);
            }
            return Ok(Pattern::Sequence(items));
        }
        Ok(p)
    }

    fn pattern_as(&mut self) -> PResult<Pattern> {
        let p = self.pattern_or()?;
        if self.eat_kw("as") {
            let n = self.name()?;
            return Ok(Pattern::As(Box::new(p), n));
        }
        Ok(p)
    }

    fn pattern_or(&mut self) -> PResult<Pattern> {
        let p = self.pattern_closed()?;
        if self.at_op("|") {
            let mut alts = vec![p];
            while self.eat_op("|") {
                alts.push(self.pattern_closed()?);
            }
            return Ok(Pattern::Or(alts));
        }
        Ok(p)
    }

    fn pattern_closed(&mut self) -> PResult<Pattern> {
        if self.eat_op("*") {
            let n = self.name()?;
            return Ok(Pattern::Star(if &*n == "_" { None } else { Some(n) }));
        }
        if self.eat_op("(") {
            if self.eat_op(")") {
                return Ok(Pattern::Sequence(vec![]));
            }
            let p = self.pattern_as()?;
            if self.at_op(",") {
                let mut items = vec![p];
                while self.eat_op(",") {
                    if self.at_op(")") {
                        break;
                    }
                    items.push(self.pattern_as()?);
                }
                self.expect_op(")")?;
                return Ok(Pattern::Sequence(items));
            }
            self.expect_op(")")?;
            return Ok(p);
        }
        if self.eat_op("[") {
            let mut items = Vec::new();
            while !self.at_op("]") {
                items.push(self.pattern_as()?);
                if !self.eat_op(",") {
                    break;
                }
            }
            self.expect_op("]")?;
            return Ok(Pattern::Sequence(items));
        }
        if self.eat_op("{") {
            let mut keys = Vec::new();
            let mut pats = Vec::new();
            let mut rest = None;
            while !self.at_op("}") {
                if self.eat_op("**") {
                    rest = Some(self.name()?);
                } else {
                    let k = self.pattern_value_expr()?;
                    self.expect_op(":")?;
                    keys.push(k);
                    pats.push(self.pattern_as()?);
                }
                if !self.eat_op(",") {
                    break;
                }
            }
            self.expect_op("}")?;
            return Ok(Pattern::Mapping { keys, pats, rest });
        }
        if let Tok::Name(n) = self.peek().clone() {
            match &*n {
                "None" => {
                    self.advance();
                    return Ok(Pattern::Singleton(Const::None));
                }
                "True" => {
                    self.advance();
                    return Ok(Pattern::Singleton(Const::True));
                }
                "False" => {
                    self.advance();
                    return Ok(Pattern::Singleton(Const::False));
                }
                _ => {}
            }
            // capture, value (dotted) or class pattern
            self.advance();
            let mut e = Expr::Name(n.clone());
            let mut dotted = false;
            while self.eat_op(".") {
                let a = self.name()?;
                e = Expr::Attr(Box::new(e), a);
                dotted = true;
            }
            if self.eat_op("(") {
                let mut args = Vec::new();
                let mut kw = Vec::new();
                while !self.at_op(")") {
                    if matches!(self.peek(), Tok::Name(_)) && is_op(self.peek_at(1), "=") {
                        let k = self.name()?;
                        self.advance();
                        kw.push((k, self.pattern_as()?));
                    } else {
                        args.push(self.pattern_as()?);
                    }
                    if !self.eat_op(",") {
                        break;
                    }
                }
                self.expect_op(")")?;
                return Ok(Pattern::Class { cls: e, args, kw });
            }
            if dotted {
                return Ok(Pattern::Value(e));
            }
            return Ok(Pattern::Capture(if &*n == "_" { None } else { Some(n) }));
        }
        let v = self.pattern_value_expr()?;
        Ok(Pattern::Value(v))
    }

    fn pattern_value_expr(&mut self) -> PResult<Expr> {
        // literals, negative numbers and complex literals
        let e = self.sum_expr()?;
        Ok(e)
    }

    // ---------------- expressions ----------------

    fn target(&mut self) -> PResult<Expr> {
        let e = self.star_or_bitor()?;
        Ok(e)
    }

    fn star_or_bitor(&mut self) -> PResult<Expr> {
        if self.eat_op("*") {
            return Ok(Expr::Starred(Box::new(self.bitor()?)));
        }
        self.bitor()
    }

    fn target_list(&mut self) -> PResult<Expr> {
        let first = self.star_or_bitor()?;
        if self.at_op(",") {
            let mut items = vec![first];
            while self.eat_op(",") {
                if self.at_kw("in") || self.at_op("=") {
                    break;
                }
                items.push(self.star_or_bitor()?);
            }
            return Ok(Expr::Tuple(items));
        }
        Ok(first)
    }

    fn star_expressions_or_yield(&mut self) -> PResult<Expr> {
        if self.at_kw("yield") {
            return self.yield_expr();
        }
        self.star_expressions()
    }

    fn yield_expr(&mut self) -> PResult<Expr> {
        self.expect_kw("yield")?;
        if self.eat_kw("from") {
            return Ok(Expr::YieldFrom(Box::new(self.expression()?)));
        }
        if matches!(self.peek(), Tok::Newline | Tok::Eof) || self.at_op(")") || self.at_op("=") || self.at_op(";") || self.at_op("]") || self.at_op("}") {
            return Ok(Expr::Yield(None));
        }
        Ok(Expr::Yield(Some(Box::new(self.star_expressions()?))))
    }

    /// a, *b, c — a bare tuple allowed
    fn star_expressions(&mut self) -> PResult<Expr> {
        let first = self.star_named()?;
        if self.at_op(",") {
            let mut items = vec![first];
            while self.eat_op(",") {
                if self.ends_expr_list() {
                    break;
                }
                items.push(self.star_named()?);
            }
            return Ok(Expr::Tuple(items));
        }
        Ok(first)
    }

    fn ends_expr_list(&self) -> bool {
        matches!(self.peek(), Tok::Newline | Tok::Eof | Tok::Op("=") | Tok::Op(")") | Tok::Op("]") | Tok::Op("}") | Tok::Op(":") | Tok::Op(";"))
            || matches!(self.peek(), Tok::Op(o) if o.ends_with('=') && o.len() >= 2 && *o != "==" && *o != "!=" && *o != "<=" && *o != ">=")
            || self.at_kw("in")
    }

    fn expressions(&mut self) -> PResult<Expr> {
        self.star_expressions()
    }

    fn star_named(&mut self) -> PResult<Expr> {
        if self.eat_op("*") {
            return Ok(Expr::Starred(Box::new(self.bitor()?)));
        }
        self.named_expr()
    }

    fn named_expr(&mut self) -> PResult<Expr> {
        if let Tok::Name(n) = self.peek().clone() {
            if is_op(self.peek_at(1), ":=") && !is_keyword(&n) {
                self.advance();
                self.advance();
                let v = self.expression()?;
                return Ok(Expr::NamedExpr(n, Box::new(v)));
            }
        }
        self.expression()
    }

    pub fn expression(&mut self) -> PResult<Expr> {
        if self.at_kw("lambda") {
            return self.lambda();
        }
        let e = self.disjunction()?;
        if self.at_kw("if") {
            // conditional expression (not a comprehension `if`)
            let save = self.pos;
            self.advance();
            let cond = self.disjunction()?;
            if self.eat_kw("else") {
                let other = self.expression()?;
                return Ok(Expr::IfExp(Box::new(cond), Box::new(e), Box::new(other)));
            }
            self.pos = save;
            return Err(self.err("expected 'else' after 'if' expression"));
        }
        Ok(e)
    }

    fn expression_nocond(&mut self) -> PResult<Expr> {
        if self.at_kw("lambda") {
            return self.lambda();
        }
        self.disjunction()
    }

    fn lambda(&mut self) -> PResult<Expr> {
        let line = self.line();
        self.expect_kw("lambda")?;
        let params = self.params(":", false)?;
        self.expect_op(":")?;
        let body = self.expression()?;
        let f = FuncDef {
            name: Rc::from("<lambda>"),
            params,
            body: vec![Stmt { line, kind: StmtKind::Return(Some(body)) }],
            decorators: vec![],
            returns: None,
            is_lambda: true,
            is_async: false,
            line,
        };
        Ok(Expr::Lambda(Rc::new(f)))
    }

    fn disjunction(&mut self) -> PResult<Expr> {
        let mut e = self.conjunction()?;
        while self.eat_kw("or") {
            let r = self.conjunction()?;
            e = Expr::BoolOr(Box::new(e), Box::new(r));
        }
        Ok(e)
    }

    fn conjunction(&mut self) -> PResult<Expr> {
        let mut e = self.inversion()?;
        while self.eat_kw("and") {
            let r = self.inversion()?;
            e = Expr::BoolAnd(Box::new(e), Box::new(r));
        }
        Ok(e)
    }

    fn inversion(&mut self) -> PResult<Expr> {
        if self.eat_kw("not") {
            let e = self.inversion()?;
            return Ok(Expr::Unary(UnOp::Not, Box::new(e)));
        }
        self.comparison()
    }

    fn comparison(&mut self) -> PResult<Expr> {
        let e = self.bitor()?;
        let mut ops = Vec::new();
        loop {
            let op = match self.peek() {
                Tok::Op("==") => CmpOp::Eq,
                Tok::Op("!=") => CmpOp::Ne,
                Tok::Op("<") => CmpOp::Lt,
                Tok::Op("<=") => CmpOp::Le,
                Tok::Op(">") => CmpOp::Gt,
                Tok::Op(">=") => CmpOp::Ge,
                Tok::Name(n) if &**n == "in" => CmpOp::In,
                Tok::Name(n) if &**n == "is" => {
                    if matches!(self.peek_at(1), Tok::Name(m) if &**m == "not") {
                        self.advance();
                        CmpOp::IsNot
                    } else {
                        CmpOp::Is
                    }
                }
                Tok::Name(n) if &**n == "not" && matches!(self.peek_at(1), Tok::Name(m) if &**m == "in") => {
                    self.advance();
                    CmpOp::NotIn
                }
                _ => break,
            };
            self.advance();
            ops.push((op, self.bitor()?));
        }
        if ops.is_empty() {
            Ok(e)
        } else {
            Ok(Expr::Compare(Box::new(e), ops))
        }
    }

    fn bitor(&mut self) -> PResult<Expr> {
        let mut e = self.bitxor()?;
        while self.eat_op("|") {
            let r = self.bitxor()?;
            e = Expr::Bin(BinOp::BitOr, Box::new(e), Box::new(r));
        }
        Ok(e)
    }
    fn bitxor(&mut self) -> PResult<Expr> {
        let mut e = self.bitand()?;
        while self.eat_op("^") {
            let r = self.bitand()?;
            e = Expr::Bin(BinOp::BitXor, Box::new(e), Box::new(r));
        }
        Ok(e)
    }
    fn bitand(&mut self) -> PResult<Expr> {
        let mut e = self.shift()?;
        while self.eat_op("&") {
            let r = self.shift()?;
            e = Expr::Bin(BinOp::BitAnd, Box::new(e), Box::new(r));
        }
        Ok(e)
    }
    fn shift(&mut self) -> PResult<Expr> {
        let mut e = self.sum_expr()?;
        loop {
            let op = if self.at_op("<<") {
                BinOp::LShift
            } else if self.at_op(">>") {
                BinOp::RShift
            } else {
                break;
            };
            self.advance();
            let r = self.sum_expr()?;
            e = Expr::Bin(op, Box::new(e), Box::new(r));
        }
        Ok(e)
    }
    fn sum_expr(&mut self) -> PResult<Expr> {
        let mut e = self.term()?;
        loop {
            let op = if self.at_op("+") {
                BinOp::Add
            } else if self.at_op("-") {
                BinOp::Sub
            } else {
                break;
            };
            self.advance();
            let r = self.term()?;
            e = Expr::Bin(op, Box::new(e), Box::new(r));
        }
        Ok(e)
    }
    fn term(&mut self) -> PResult<Expr> {
        let mut e = self.factor()?;
        loop {
            let op = match self.peek() {
                Tok::Op("*") => BinOp::Mul,
                Tok::Op("/") => BinOp::Div,
                Tok::Op("//") => BinOp::FloorDiv,
                Tok::Op("%") => BinOp::Mod,
                Tok::Op("@") => BinOp::MatMul,
                _ => break,
            };
            self.advance();
            let r = self.factor()?;
            e = Expr::Bin(op, Box::new(e), Box::new(r));
        }
        Ok(e)
    }
    fn factor(&mut self) -> PResult<Expr> {
        let op = match self.peek() {
            Tok::Op("-") => UnOp::Neg,
            Tok::Op("+") => UnOp::Pos,
            Tok::Op("~") => UnOp::Invert,
            _ => return self.power(),
        };
        self.advance();
        let e = self.factor()?;
        // fold negative numeric literals
        if op == UnOp::Neg {
            match &e {
                Expr::Const(Const::Int(v)) if *v != i64::MIN => return Ok(Expr::Const(Const::Int(-v))),
                Expr::Const(Const::Float(v)) => return Ok(Expr::Const(Const::Float(-v))),
                _ => {}
            }
        }
        Ok(Expr::Unary(op, Box::new(e)))
    }
    fn power(&mut self) -> PResult<Expr> {
        let e = self.await_primary()?;
        if self.eat_op("**") {
            let r = self.factor()?;
            return Ok(Expr::Bin(BinOp::Pow, Box::new(e), Box::new(r)));
        }
        Ok(e)
    }
    fn await_primary(&mut self) -> PResult<Expr> {
        if self.eat_kw("await") {
            return Ok(Expr::Await(Box::new(self.primary()?)));
        }
        self.primary()
    }

    fn primary(&mut self) -> PResult<Expr> {
        let mut e = self.atom()?;
        loop {
            if self.eat_op(".") {
                let n = match self.advance() {
                    Tok::Name(n) => n,
                    _ => return Err(self.err("invalid syntax")),
                };
                e = Expr::Attr(Box::new(e), n);
            } else if self.eat_op("(") {
                let args = self.call_args()?;
                self.expect_op(")")?;
                e = Expr::Call { func: Box::new(e), args };
            } else if self.eat_op("[") {
                let idx = self.slices()?;
                self.expect_op("]")?;
                e = Expr::Subscript(Box::new(e), Box::new(idx));
            } else {
                break;
            }
        }
        Ok(e)
    }

    fn slices(&mut self) -> PResult<Expr> {
        let first = self.slice_item()?;
        if self.at_op(",") {
            let mut items = vec![first];
            while self.eat_op(",") {
                if self.at_op("]") {
                    break;
                }
                items.push(self.slice_item()?);
            }
            return Ok(Expr::Tuple(items));
        }
        Ok(first)
    }

    fn slice_item(&mut self) -> PResult<Expr> {
        let lo = if self.at_op(":") { None } else { Some(Box::new(self.star_named()?)) };
        if !self.eat_op(":") {
            return Ok(*lo.unwrap());
        }
        let hi = if self.at_op(":") || self.at_op("]") || self.at_op(",") { None } else { Some(Box::new(self.expression()?)) };
        let step = if self.eat_op(":") {
            if self.at_op("]") || self.at_op(",") {
                None
            } else {
                Some(Box::new(self.expression()?))
            }
        } else {
            None
        };
        Ok(Expr::Slice(lo, hi, step))
    }

    fn call_args(&mut self) -> PResult<Vec<Arg>> {
        let mut args = Vec::new();
        while !self.at_op(")") {
            if self.eat_op("**") {
                args.push(Arg::StarStar(self.expression()?));
            } else if self.eat_op("*") {
                if args.iter().any(|a| matches!(a, Arg::StarStar(_))) {
                    return Err(self.err("iterable argument unpacking follows keyword argument unpacking"));
                }
                args.push(Arg::Star(self.expression()?));
            } else if matches!(self.peek(), Tok::Name(_)) && is_op(self.peek_at(1), "=") {
                let n = self.name()?;
                if args.iter().any(|a| matches!(a, Arg::Kw(k, _) if *k == n)) {
                    return Err(self.err(&format!("keyword argument repeated: {}", n)));
                }
                self.advance();
                args.push(Arg::Kw(n, self.expression()?));
            } else {
                if args.iter().any(|a| matches!(a, Arg::StarStar(_))) {
                    return Err(self.err("positional argument follows keyword argument unpacking"));
                }
                if args.iter().any(|a| matches!(a, Arg::Kw(..))) {
                    return Err(self.err("positional argument follows keyword argument"));
                }
                let e = self.named_expr()?;
                if self.at_kw("for") || self.at_kw("async") {
                    let comps = self.comp_for()?;
                    args.push(Arg::Pos(Expr::GenExp(Box::new(e), comps)));
                } else {
                    args.push(Arg::Pos(e));
                }
            }
            if !self.eat_op(",") {
                break;
            }
        }
        Ok(args)
    }

    fn comp_for(&mut self) -> PResult<Vec<Comp>> {
        let mut comps = Vec::new();
        loop {
            let is_async = self.eat_kw("async");
            if !self.eat_kw("for") {
                break;
            }
            let target = self.target_list()?;
            self.expect_kw("in")?;
            let iter = self.disjunction()?;
            let mut ifs = Vec::new();
            while self.at_kw("if") {
                self.advance();
                ifs.push(self.expression_nocond()?);
            }
            comps.push(Comp { target, iter, ifs, is_async });
        }
        Ok(comps)
    }

    fn atom(&mut self) -> PResult<Expr> {
        let line = self.line();
        let t = self.peek().clone();
        match t {
            Tok::Name(n) => {
                self.advance();
                Ok(match &*n {
                    "None" => Expr::Const(Const::None),
                    "True" => Expr::Const(Const::True),
                    "False" => Expr::Const(Const::False),
                    _ => {
                        if is_keyword(&n) && &*n != "match" && &*n != "case" && &*n != "type" && &*n != "_" {
                            self.pos -= 1;
                            return Err(self.err("invalid syntax"));
                        }
                        Expr::Name(n)
                    }
                })
            }
            Tok::Int(v) => {
                self.advance();
                Ok(Expr::Const(Const::Int(v)))
            }
            Tok::BigInt(s) => {
                self.advance();
                Ok(Expr::Const(Const::BigInt(s)))
            }
            Tok::Float(v) => {
                self.advance();
                Ok(Expr::Const(Const::Float(v)))
            }
            Tok::Imag(v) => {
                self.advance();
                Ok(Expr::Const(Const::Imag(v)))
            }
            Tok::Str { .. } => self.strings(line),
            Tok::Op("...") => {
                self.advance();
                Ok(Expr::Const(Const::Ellipsis))
            }
            Tok::Op("(") => {
                self.advance();
                if self.eat_op(")") {
                    return Ok(Expr::Tuple(vec![]));
                }
                if self.at_kw("yield") {
                    let y = self.yield_expr()?;
                    self.expect_op(")")?;
                    return Ok(y);
                }
                let first = self.star_named()?;
                if self.at_kw("for") || self.at_kw("async") {
                    let comps = self.comp_for()?;
                    self.expect_op(")")?;
                    return Ok(Expr::GenExp(Box::new(first), comps));
                }
                if self.at_op(",") {
                    let mut items = vec![first];
                    while self.eat_op(",") {
                        if self.at_op(")") {
                            break;
                        }
                        items.push(self.star_named()?);
                    }
                    self.expect_op(")")?;
                    return Ok(Expr::Tuple(items));
                }
                self.expect_op(")")?;
                Ok(first)
            }
            Tok::Op("[") => {
                self.advance();
                if self.eat_op("]") {
                    return Ok(Expr::List(vec![]));
                }
                let first = self.star_named()?;
                if self.at_kw("for") || self.at_kw("async") {
                    let comps = self.comp_for()?;
                    self.expect_op("]")?;
                    return Ok(Expr::ListComp(Box::new(first), comps));
                }
                let mut items = vec![first];
                while self.eat_op(",") {
                    if self.at_op("]") {
                        break;
                    }
                    items.push(self.star_named()?);
                }
                self.expect_op("]")?;
                Ok(Expr::List(items))
            }
            Tok::Op("{") => {
                self.advance();
                if self.eat_op("}") {
                    return Ok(Expr::Dict(vec![]));
                }
                if self.eat_op("**") {
                    let v = self.bitor()?;
                    let mut items = vec![(None, v)];
                    self.dict_rest(&mut items)?;
                    return Ok(Expr::Dict(items));
                }
                let first = self.star_named()?;
                if self.eat_op(":") {
                    let v = self.expression()?;
                    if self.at_kw("for") || self.at_kw("async") {
                        let comps = self.comp_for()?;
                        self.expect_op("}")?;
                        return Ok(Expr::DictComp(Box::new(first), Box::new(v), comps));
                    }
                    let mut items = vec![(Some(first), v)];
                    self.dict_rest(&mut items)?;
                    return Ok(Expr::Dict(items));
                }
                if self.at_kw("for") || self.at_kw("async") {
                    let comps = self.comp_for()?;
                    self.expect_op("}")?;
                    return Ok(Expr::SetComp(Box::new(first), comps));
                }
                let mut items = vec![first];
                while self.eat_op(",") {
                    if self.at_op("}") {
                        break;
                    }
                    items.push(self.star_named()?);
                }
                self.expect_op("}")?;
                Ok(Expr::Set(items))
            }
            Tok::Op("}") | Tok::Op(")") | Tok::Op("]") => Err(self.err(&format!("unmatched '{}'", match t {
                Tok::Op(o) => o,
                _ => "",
            }))),
            Tok::Indent => Err(self.err("unexpected indent")),
            Tok::Newline | Tok::Eof => Err(self.err("invalid syntax")),
            _ => Err(self.err("invalid syntax")),
        }
    }

    fn dict_rest(&mut self, items: &mut Vec<(Option<Expr>, Expr)>) -> PResult<()> {
        while self.eat_op(",") {
            if self.at_op("}") {
                break;
            }
            if self.eat_op("**") {
                let v = self.bitor()?;
                items.push((None, v));
                continue;
            }
            let k = self.expression()?;
            self.expect_op(":")?;
            let v = self.expression()?;
            items.push((Some(k), v));
        }
        self.expect_op("}")
    }

    fn strings(&mut self, line: u32) -> PResult<Expr> {
        let mut parts: Vec<Expr> = Vec::new();
        let mut lit = String::new();
        let mut any_f = false;
        let mut bytes = None;
        while let Tok::Str { s, bytes: b, f } = self.peek().clone() {
            self.advance();
            if let Some(pb) = bytes {
                if pb != b {
                    return Err(SyntaxError { msg: "cannot mix bytes and nonbytes literals".into(), line, col: 0 });
                }
            }
            bytes = Some(b);
            if f {
                any_f = true;
                self.fstring(&s, &mut parts, &mut lit, line)?;
            } else {
                lit.push_str(&s);
            }
        }
        if bytes == Some(true) {
            let v: Vec<u8> = lit.chars().map(|c| c as u32 as u8).collect();
            return Ok(Expr::Const(Const::Bytes(Rc::from(v))));
        }
        if !any_f {
            return Ok(Expr::Const(Const::Str(Rc::from(lit.as_str()))));
        }
        if !lit.is_empty() {
            parts.push(Expr::Const(Const::Str(Rc::from(lit.as_str()))));
        }
        Ok(Expr::JoinedStr(parts))
    }

    /// splits an f-string body into literal text and replacement fields
    fn fstring(&mut self, body: &str, parts: &mut Vec<Expr>, lit: &mut String, line: u32) -> PResult<()> {
        let cs: Vec<char> = body.chars().collect();
        let mut i = 0;
        let mut raw_lit = String::new();
        let flush = |raw_lit: &mut String, lit: &mut String| -> PResult<()> {
            if !raw_lit.is_empty() {
                let u = unescape(raw_lit, false).map_err(|m| SyntaxError { msg: m, line, col: 0 })?;
                lit.push_str(&u);
                raw_lit.clear();
            }
            Ok(())
        };
        while i < cs.len() {
            let c = cs[i];
            if c == '\\' && i + 2 < cs.len() && cs[i + 1] == 'N' && cs[i + 2] == '{' {
                while i < cs.len() && cs[i] != '}' {
                    raw_lit.push(cs[i]);
                    i += 1;
                }
                if i < cs.len() {
                    raw_lit.push(cs[i]);
                    i += 1;
                }
                continue;
            }
            if c == '\\' && i + 1 < cs.len() && cs[i + 1] == '\\' {
                raw_lit.push_str("\\\\");
                i += 2;
                continue;
            }
            if c == '{' {
                if i + 1 < cs.len() && cs[i + 1] == '{' {
                    raw_lit.push('{');
                    i += 2;
                    continue;
                }
                flush(&mut raw_lit, lit)?;
                // find the end of the expression
                let (expr_src, conv, spec_src, debug, ni) = split_field(&cs, i + 1).ok_or(SyntaxError { msg: "f-string: expecting '}'".into(), line, col: 0 })?;
                i = ni;
                // a debug field's source comes as "expr\0spaces after ="
                let (expr_src, after_eq) = match expr_src.split_once('\0') {
                    Some((a, b)) => (a.to_string(), b.to_string()),
                    None => (expr_src, String::new()),
                };
                if debug {
                    lit.push_str(&expr_src);
                    lit.push('=');
                    lit.push_str(&after_eq);
                }
                if !lit.is_empty() {
                    parts.push(Expr::Const(Const::Str(Rc::from(lit.as_str()))));
                    lit.clear();
                }
                let trimmed = expr_src.trim();
                if trimmed.is_empty() {
                    return Err(SyntaxError { msg: "f-string: valid expression required before '}'".into(), line, col: 0 });
                }
                let value = parse_expression(&format!("({})", trimmed)).map_err(|e| SyntaxError { msg: if e.msg.contains("Perhaps") { e.msg } else { format!("f-string: {}", e.msg) }, line, col: 0 })?;
                let spec = match spec_src {
                    Some(s) => {
                        let mut sp = Vec::new();
                        let mut sl = String::new();
                        self.fstring(&s, &mut sp, &mut sl, line)?;
                        if !sl.is_empty() {
                            sp.push(Expr::Const(Const::Str(Rc::from(sl.as_str()))));
                        }
                        Some(Box::new(Expr::JoinedStr(sp)))
                    }
                    None => None,
                };
                let conv = if conv == '\0' && debug && spec.is_none() { 'r' } else { conv };
                parts.push(Expr::FormattedValue { value: Box::new(value), conv, spec });
                continue;
            }
            if c == '}' {
                if i + 1 < cs.len() && cs[i + 1] == '}' {
                    raw_lit.push('}');
                    i += 2;
                    continue;
                }
                return Err(SyntaxError { msg: "f-string: single '}' is not allowed".into(), line, col: 0 });
            }
            raw_lit.push(c);
            i += 1;
        }
        flush(&mut raw_lit, lit)?;
        Ok(())
    }
}

/// `expr[=][!c][:spec]}` starting after `{`; returns the parts and the index
/// after the closing brace
fn split_field(cs: &[char], mut i: usize) -> Option<(String, char, Option<String>, bool, usize)> {
    let mut depth = 0;
    let mut expr = String::new();
    while i < cs.len() {
        let c = cs[i];
        if c == '\'' || c == '"' {
            let q = c;
            let triple = i + 2 < cs.len() && cs[i + 1] == q && cs[i + 2] == q;
            let qlen = if triple { 3 } else { 1 };
            for _ in 0..qlen {
                expr.push(q);
            }
            i += qlen;
            loop {
                if i >= cs.len() {
                    return None;
                }
                if cs[i] == '\\' {
                    expr.push(cs[i]);
                    if i + 1 < cs.len() {
                        expr.push(cs[i + 1]);
                    }
                    i += 2;
                    continue;
                }
                if cs[i] == q && (!triple || (i + 2 < cs.len() && cs[i + 1] == q && cs[i + 2] == q)) {
                    for _ in 0..qlen {
                        expr.push(q);
                    }
                    i += qlen;
                    break;
                }
                expr.push(cs[i]);
                i += 1;
            }
            continue;
        }
        if c == '(' || c == '[' || c == '{' {
            depth += 1;
        } else if c == ')' || c == ']' || (c == '}' && depth > 0) {
            depth -= 1;
        } else if depth == 0 {
            if c == '}' {
                return Some((expr, '\0', None, false, i + 1));
            }
            if c == '=' && i + 1 < cs.len() && cs[i + 1] != '=' && !expr.ends_with(['=', '!', '<', '>']) {
                // self-documenting expression
                let mut j = i + 1;
                let mut conv = '\0';
                while j < cs.len() && cs[j].is_whitespace() {
                    j += 1;
                }
                let ws: String = cs[i + 1..j].iter().collect();
                let expr = format!("{}\0{}", expr, ws);
                if j < cs.len() && cs[j] == '!' {
                    conv = cs[j + 1];
                    j += 2;
                }
                if j < cs.len() && cs[j] == '}' {
                    return Some((expr, conv, None, true, j + 1));
                }
                if j < cs.len() && cs[j] == ':' {
                    let (spec, nj) = read_spec(cs, j + 1)?;
                    return Some((expr, conv, Some(spec), true, nj));
                }
            }
            if c == '!' && i + 1 < cs.len() && cs[i + 1] != '=' {
                let conv = cs[i + 1];
                let mut j = i + 2;
                while j < cs.len() && cs[j].is_whitespace() {
                    j += 1;
                }
                if j < cs.len() && cs[j] == '}' {
                    return Some((expr, conv, None, false, j + 1));
                }
                if j < cs.len() && cs[j] == ':' {
                    let (spec, nj) = read_spec(cs, j + 1)?;
                    return Some((expr, conv, Some(spec), false, nj));
                }
                return None;
            }
            if c == ':' {
                let (spec, nj) = read_spec(cs, i + 1)?;
                return Some((expr, '\0', Some(spec), false, nj));
            }
        }
        expr.push(c);
        i += 1;
    }
    None
}

fn read_spec(cs: &[char], mut i: usize) -> Option<(String, usize)> {
    let mut depth = 0;
    let mut s = String::new();
    while i < cs.len() {
        let c = cs[i];
        if depth > 0 && (c == '\'' || c == '"') {
            // a string inside a nested replacement field
            s.push(c);
            i += 1;
            while i < cs.len() && cs[i] != c {
                s.push(cs[i]);
                i += 1;
            }
            if i < cs.len() {
                s.push(c);
                i += 1;
            }
            continue;
        }
        if c == '{' {
            depth += 1;
        } else if c == '}' {
            if depth == 0 {
                return Some((s, i + 1));
            }
            depth -= 1;
        }
        s.push(c);
        i += 1;
    }
    None
}

fn check_target(e: &Expr) -> Result<(), String> {
    match e {
        Expr::Name(_) | Expr::Attr(..) | Expr::Subscript(..) => Ok(()),
        Expr::Tuple(items) | Expr::List(items) => {
            for it in items {
                check_target(it)?;
            }
            Ok(())
        }
        Expr::Starred(inner) => check_target(inner),
        Expr::Call { .. } => Err("cannot assign to function call here. Maybe you meant '==' instead of '='?".into()),
        Expr::Const(_) => Err("cannot assign to literal here. Maybe you meant '==' instead of '='?".into()),
        _ => Err("cannot assign to expression here. Maybe you meant '==' instead of '='?".into()),
    }
}

pub fn is_keyword(s: &str) -> bool {
    matches!(
        s,
        "False" | "None" | "True" | "and" | "as" | "assert" | "async" | "await" | "break" | "class" | "continue" | "def" | "del" | "elif" | "else"
            | "except" | "finally" | "for" | "from" | "global" | "if" | "import" | "in" | "is" | "lambda" | "nonlocal" | "not" | "or" | "pass"
            | "raise" | "return" | "try" | "while" | "with" | "yield"
    )
}

fn str_const(s: &str) -> Expr {
    Expr::Const(Const::Str(Rc::from(s)))
}

fn call(func: Expr, args: Vec<Arg>) -> Expr {
    Expr::Call { func: Box::new(func), args }
}

/// typing.NAME, through __import__ so no name is bound
fn typing_attr(name: &str) -> Expr {
    let m = call(Expr::Name(Rc::from("__import__")), vec![Arg::Pos(str_const("typing"))]);
    Expr::Attr(Box::new(m), Rc::from(name))
}

/// the definition, then NAME.__type_params__ = (T, ...) and return NAME
fn generic_body(name: &Name, tps: &[TypeParam], def: Stmt, line: u32) -> Vec<Stmt> {
    let tuple = Expr::Tuple(tps.iter().map(|t| Expr::Name(t.name.clone())).collect());
    vec![
        def,
        Stmt { line, kind: StmtKind::Assign(vec![Expr::Attr(Box::new(Expr::Name(name.clone())), Rc::from("__type_params__"))], tuple) },
        Stmt { line, kind: StmtKind::Return(Some(Expr::Name(name.clone()))) },
    ]
}

/// PEP 695: the type parameters become the parameters of a hidden function
/// that is called with new TypeVars; its result, decorated, is bound to NAME
fn generic_wrap(out: &mut Vec<Stmt>, name: &Name, tps: Vec<TypeParam>, body: Vec<Stmt>, decos: Vec<Expr>, line: u32) {
    let wname: Name = Rc::from(format!("<generic parameters of {}>", name).as_str());
    let mut params = Params::default();
    let mut vars = vec![];
    for t in tps {
        params.args.push(Param { name: t.name.clone(), default: None, annotation: None });
        let cls = match t.kind {
            1 => "TypeVarTuple",
            2 => "ParamSpec",
            _ => "TypeVar",
        };
        let mut args = vec![Arg::Pos(str_const(&t.name)), Arg::Kw(Rc::from("infer_variance"), Expr::Const(Const::True))];
        if let Some(b) = t.bound {
            args.push(Arg::Kw(Rc::from("bound"), b));
        }
        if let Some(d) = t.default {
            args.push(Arg::Kw(Rc::from("default"), d));
        }
        vars.push(Arg::Pos(call(typing_attr(cls), args)));
    }
    let wrapper = FuncDef { name: wname.clone(), params, body, decorators: vec![], returns: None, is_lambda: false, is_async: false, line };
    out.push(Stmt { line, kind: StmtKind::FuncDef(Rc::new(wrapper)) });
    let mut v = call(Expr::Name(wname.clone()), vars);
    for d in decos.into_iter().rev() {
        v = call(d, vec![Arg::Pos(v)]);
    }
    out.push(Stmt { line, kind: StmtKind::Assign(vec![Expr::Name(name.clone())], v) });
    out.push(Stmt { line, kind: StmtKind::Del(vec![Expr::Name(wname)]) });
}
