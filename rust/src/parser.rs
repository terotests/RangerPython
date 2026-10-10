// SPDX-License-Identifier: MIT
//! Recursive-descent parser: tokens to the syntax tree in ast.rs.

use crate::ast::*;
use crate::lexer::{tokenize, unescape, Tok, Token};
use std::rc::Rc;

pub struct SyntaxError {
    pub msg: String,
    pub line: u32,
}

type PResult<T> = Result<T, SyntaxError>;

pub struct Parser {
    toks: Vec<Token>,
    pos: usize,
}

pub fn parse_module(src: &str) -> PResult<Vec<Stmt>> {
    let toks = tokenize(src).map_err(|e| SyntaxError { msg: e.msg, line: e.line })?;
    let mut p = Parser { toks, pos: 0 };
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
    let toks = tokenize(src.trim()).map_err(|e| SyntaxError { msg: e.msg, line: e.line })?;
    let mut p = Parser { toks, pos: 0 };
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
        SyntaxError { msg: msg.to_string(), line: self.line() }
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
                _ => Err(self.err(if o == ":" { "expected ':'" } else { "invalid syntax" })),
            }
        }
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
                return Err(self.err("expected an indented block"));
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
                    out.push(Stmt { line, kind: StmtKind::FuncDef(Rc::new(f)) });
                    return Ok(());
                }
                "class" => {
                    self.advance();
                    let c = self.classdef(vec![], line)?;
                    out.push(Stmt { line, kind: StmtKind::ClassDef(Rc::new(c)) });
                    return Ok(());
                }
                "async" => {
                    self.advance();
                    if self.eat_kw("def") {
                        let f = self.funcdef(vec![], line, true)?;
                        out.push(Stmt { line, kind: StmtKind::FuncDef(Rc::new(f)) });
                        return Ok(());
                    }
                    if self.eat_kw("for") {
                        out.push(self.for_rest(line)?);
                        return Ok(());
                    }
                    if self.eat_kw("with") {
                        out.push(self.with_rest(line)?);
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
                out.push(Stmt { line, kind: StmtKind::FuncDef(Rc::new(f)) });
            } else if self.eat_kw("class") {
                let c = self.classdef(decos, line)?;
                out.push(Stmt { line, kind: StmtKind::ClassDef(Rc::new(c)) });
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
            return Err(self.err("invalid syntax"));
        }
        Ok(())
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

    fn funcdef(&mut self, decorators: Vec<Expr>, line: u32, is_async: bool) -> PResult<FuncDef> {
        let name = self.name()?;
        if self.at_op("[") {
            // PEP 695 type parameters: skipped
            let mut depth = 0;
            loop {
                if self.at_op("[") {
                    depth += 1;
                } else if self.at_op("]") {
                    depth -= 1;
                    if depth == 0 {
                        self.advance();
                        break;
                    }
                }
                self.advance();
            }
        }
        self.expect_op("(")?;
        let params = self.params(")", true)?;
        self.expect_op(")")?;
        let returns = if self.eat_op("->") { Some(self.expression()?) } else { None };
        let body = self.block()?;
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
        if self.at_op("[") {
            let mut depth = 0;
            loop {
                if self.at_op("[") {
                    depth += 1;
                } else if self.at_op("]") {
                    depth -= 1;
                    if depth == 0 {
                        self.advance();
                        break;
                    }
                }
                self.advance();
            }
        }
        let bases = if self.eat_op("(") {
            let a = self.call_args()?;
            self.expect_op(")")?;
            a
        } else {
            vec![]
        };
        let body = self.block()?;
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
                        let name = self.name()?;
                        if self.at_op("[") {
                            while !self.eat_op("]") {
                                self.advance();
                            }
                        }
                        self.expect_op("=")?;
                        let v = self.expression()?;
                        return Ok(StmtKind::Assign(vec![Expr::Name(name)], v));
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
                args.push(Arg::Star(self.expression()?));
            } else if matches!(self.peek(), Tok::Name(_)) && is_op(self.peek_at(1), "=") {
                let n = self.name()?;
                self.advance();
                args.push(Arg::Kw(n, self.expression()?));
            } else {
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
                    return Err(SyntaxError { msg: "cannot mix bytes and nonbytes literals".into(), line });
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
                let u = unescape(raw_lit, false).map_err(|m| SyntaxError { msg: m, line })?;
                lit.push_str(&u);
                raw_lit.clear();
            }
            Ok(())
        };
        while i < cs.len() {
            let c = cs[i];
            if c == '{' {
                if i + 1 < cs.len() && cs[i + 1] == '{' {
                    raw_lit.push('{');
                    i += 2;
                    continue;
                }
                flush(&mut raw_lit, lit)?;
                // find the end of the expression
                let (expr_src, conv, spec_src, debug, ni) = split_field(&cs, i + 1).ok_or(SyntaxError { msg: "f-string: expecting '}'".into(), line })?;
                i = ni;
                if debug {
                    lit.push_str(&expr_src);
                    lit.push('=');
                }
                if !lit.is_empty() {
                    parts.push(Expr::Const(Const::Str(Rc::from(lit.as_str()))));
                    lit.clear();
                }
                let trimmed = expr_src.trim();
                if trimmed.is_empty() {
                    return Err(SyntaxError { msg: "f-string: valid expression required before '}'".into(), line });
                }
                let value = parse_expression(&format!("({})", trimmed)).map_err(|e| SyntaxError { msg: format!("f-string: {}", e.msg), line })?;
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
                return Err(SyntaxError { msg: "f-string: single '}' is not allowed".into(), line });
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
                let j = i + 2;
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
