// SPDX-License-Identifier: MIT
//! Recursive-descent parser: tokens to the syntax tree in ast.rs.

use crate::ast::*;
use crate::lexer::{tokenize, Tok, Token};
use std::rc::Rc;

pub struct SyntaxError {
    pub msg: String,
    pub line: u32,
    /// 1-based column, 0 when unknown
    pub col: u32,
    /// where the bad part ends: the line, and the 1-based column after it;
    /// 0 when it is only the start that is known
    pub end_line: u32,
    pub end_col: u32,
}

impl SyntaxError {
    pub fn new(msg: impl Into<String>, line: u32, col: u32) -> SyntaxError {
        SyntaxError { msg: msg.into(), line, col, end_line: 0, end_col: 0 }
    }
}

type PResult<T> = Result<T, SyntaxError>;

pub struct Parser {
    toks: Vec<Token>,
    pos: usize,
    /// the start of the last `or` expression parsed, and the token after it
    last_disj: Option<(Pos, usize)>,
}

pub fn parse_module(src: &str) -> PResult<Vec<Stmt>> {
    let mut p = Parser { toks: tokenize(src), pos: 0, last_disj: None };
    let mut body = Vec::new();
    while !p.at_eof() {
        if p.eat_tok(&Tok::Newline) {
            continue;
        }
        p.statement(&mut body).map_err(|e| p.settle(e))?;
    }
    Ok(body)
}

pub fn parse_expression(src: &str) -> PResult<Expr> {
    let mut p = Parser { toks: tokenize(src.trim_matches([' ', '\t', '\n', '\r', '\x0c'])), pos: 0, last_disj: None };
    let e = p.expressions().map_err(|e| p.settle(e))?;
    while p.eat_tok(&Tok::Newline) {}
    if !p.at_eof() {
        let e = p.err("invalid syntax");
        return Err(p.settle(e));
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
    /// where the current token starts
    fn start(&self) -> Pos {
        let t = &self.toks[self.pos];
        Pos { line: t.line, col: t.bcol, end_line: t.line, end_col: t.bcol }
    }
    /// from `start` to the end of the last token taken
    fn span(&self, start: Pos) -> Pos {
        let t = &self.toks[self.pos.saturating_sub(1)];
        Pos { end_line: t.end_line, end_col: t.end_bcol, ..start }
    }
    /// an expression node spanning from `start` to the last token taken
    fn node(&self, start: Pos, kind: ExprKind) -> Expr {
        Expr::new(kind, self.span(start))
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
    /// an error at the current token; at a tokenizer error, that error
    fn err(&self, msg: &str) -> SyntaxError {
        let t = &self.toks[self.pos];
        match &t.tok {
            Tok::Error(m) => Self::token_error(m, t),
            _ => SyntaxError { msg: msg.to_string(), line: t.line, col: t.col, end_line: t.end_line, end_col: t.end_col },
        }
    }
    /// an error at the first part of target `t` that is not one, which
    /// the statement would `verb` ("assign to", "delete")
    fn check_store(&self, t: &Expr, verb: &str) -> PResult<()> {
        match t.invalid_target() {
            Some(bad) => Err(self.err_at(&format!("cannot {} {}", verb, bad.describe()), bad.pos)),
            None => Ok(()),
        }
    }
    /// whether the current token closes a bracket opened before it
    fn closes_bracket(&self) -> bool {
        let depth = self.toks[..self.pos].iter().fold(0i32, |d, t| match t.tok {
            Tok::Op("(" | "[" | "{") => d + 1,
            Tok::Op(")" | "]" | "}") => d - 1,
            _ => d,
        });
        depth > 0
    }
    /// the tokenizer's error that token `t` stands for
    fn token_error(msg: &str, t: &Token) -> SyntaxError {
        let mut e = SyntaxError::new(msg, t.line, t.col);
        if t.end_col != t.col {
            (e.end_line, e.end_col) = (t.end_line, t.end_col);
        }
        e
    }
    /// an error spanning the source at `p`, a node's span
    fn err_at(&self, msg: &str, p: Pos) -> SyntaxError {
        // a node starts and ends where tokens do: their character columns
        let col = self.toks.iter().find(|t| t.line == p.line && t.bcol == p.col).map_or(p.col + 1, |t| t.col);
        let end = self.toks.iter().find(|t| t.end_line == p.end_line && t.end_bcol == p.end_col);
        SyntaxError { msg: msg.to_string(), line: p.line, col, end_line: p.end_line, end_col: end.map_or(p.end_col + 1, |t| t.end_col) }
    }
    /// As CPython does, lets a tokenizer error later in the source replace
    /// the parser's error: an unclosed bracket only when the error is on a
    /// later line than the bracket, and not an error in an f-string.
    fn settle(&self, e: SyntaxError) -> SyntaxError {
        let Some(at) = self.toks.iter().position(|t| matches!(t.tok, Tok::Error(_))) else {
            return e;
        };
        let t = &self.toks[at];
        let Token { tok: Tok::Error(m), line, .. } = t else { unreachable!() };
        let in_fstring = self.toks[..self.pos.min(at)].iter().fold(0i32, |d, t| match t.tok {
            Tok::FStart => d + 1,
            Tok::FEnd => d - 1,
            _ => d,
        }) > 0;
        let replace = if m.ends_with("was never closed") { e.line.max(e.end_line) > *line } else { !in_fstring };
        if replace {
            Self::token_error(m, t)
        } else {
            e
        }
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
                _ if (o == ")" || o == "]" || o == "}") && self.starts_expr() && !self.after_name_before_str() => Err(self.missing_comma()),
                _ => Err(self.err(if o == ":" { "expected ':'" } else { "invalid syntax" })),
            }
        }
    }
    /// "Perhaps you forgot a comma?", spanning the expression before the
    /// current token and the one at it, as CPython's invalid_expression rule
    fn missing_comma(&mut self) -> SyntaxError {
        let msg = "invalid syntax. Perhaps you forgot a comma?";
        let Some((a, _)) = self.last_disj.filter(|(_, end)| *end == self.pos) else {
            return self.err(msg);
        };
        let save = (self.pos, self.last_disj);
        let span = self.expression().ok().map(|_| self.span(a));
        (self.pos, self.last_disj) = save;
        match span {
            Some(p) => self.err_at(msg, p),
            None => self.err(msg),
        }
    }
    /// `name "string"`, which CPython does not take for a missing comma
    fn after_name_before_str(&self) -> bool {
        let prev = if self.pos > 0 { self.toks.get(self.pos - 1).map(|t| &t.tok) } else { None };
        matches!(prev, Some(Tok::Name(_))) && matches!(self.peek(), Tok::Str { .. } | Tok::FStart)
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
        let start = self.start();
        if let Tok::Name(n) = self.peek().clone() {
            match &*n {
                "if" => {
                    self.advance();
                    let s = self.if_rest(start)?;
                    out.push(s);
                    return Ok(());
                }
                "while" => {
                    self.advance();
                    let cond = self.named_expr()?;
                    let body = self.block()?;
                    let orelse = if self.eat_kw("else") { self.block()? } else { vec![] };
                    let pos = block_end(start, &body, &orelse);
                    out.push(Stmt { pos, kind: StmtKind::While(cond, body, orelse) });
                    return Ok(());
                }
                "for" => {
                    self.advance();
                    out.push(self.for_rest(start, false)?);
                    return Ok(());
                }
                "try" => {
                    self.advance();
                    out.push(self.try_rest(start)?);
                    return Ok(());
                }
                "with" => {
                    self.advance();
                    out.push(self.with_rest(start, false)?);
                    return Ok(());
                }
                "def" => {
                    self.advance();
                    let f = self.funcdef(vec![], start.line, false)?;
                    out.push(func_stmt(start, f));
                    return Ok(());
                }
                "class" => {
                    self.advance();
                    let c = self.classdef(vec![], start.line)?;
                    out.push(class_stmt(start, c));
                    return Ok(());
                }
                "async" => {
                    self.advance();
                    if self.eat_kw("def") {
                        let f = self.funcdef(vec![], start.line, true)?;
                        out.push(func_stmt(start, f));
                        return Ok(());
                    }
                    if self.eat_kw("for") {
                        out.push(self.for_rest(start, true)?);
                        return Ok(());
                    }
                    if self.eat_kw("with") {
                        out.push(self.with_rest(start, true)?);
                        return Ok(());
                    }
                    return Err(self.err("invalid syntax"));
                }
                "match" => {
                    if let Some(s) = self.try_match(start)? {
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
            let start = self.start();
            let is_async = self.eat_kw("async");
            if self.eat_kw("def") {
                let f = self.funcdef(decos, start.line, is_async)?;
                out.push(func_stmt(start, f));
            } else if self.eat_kw("class") {
                let c = self.classdef(decos, start.line)?;
                out.push(class_stmt(start, c));
            } else {
                return Err(self.err("invalid syntax"));
            }
            return Ok(());
        }
        self.simple_statements(out)
    }

    fn simple_statements(&mut self, out: &mut Vec<Stmt>) -> PResult<()> {
        loop {
            let start = self.start();
            let s = self.small_statement()?;
            out.push(Stmt { pos: self.span(start), kind: s });
            if self.eat_op(";") {
                if matches!(self.peek(), Tok::Newline | Tok::Eof) {
                    break;
                }
                continue;
            }
            break;
        }
        if !self.eat_tok(&Tok::Newline) && !self.at_eof() {
            if let Some(Stmt { kind: StmtKind::Expr(Expr { kind: ExprKind::Name(n), .. }), .. }) = out.last() {
                if (&**n == "print" || &**n == "exec") && self.starts_expr() {
                    // only when what follows is an expression
                    let save = self.pos;
                    let legacy = self.star_expressions().is_ok();
                    self.pos = save;
                    if legacy {
                        return Err(self.err(&format!("Missing parentheses in call to '{}'. Did you mean {}(...)?", n, n)));
                    }
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
            Tok::Int(_) | Tok::BigInt(_) | Tok::Float(_) | Tok::Imag(_) | Tok::Str { .. } | Tok::FStart => true,
            Tok::Op(o) => matches!(*o, "(" | "[" | "{" | "-" | "+" | "~" | "..."),
            _ => false,
        }
    }

    fn if_rest(&mut self, start: Pos) -> PResult<Stmt> {
        let cond = self.named_expr()?;
        let body = self.block()?;
        let orelse = if self.at_kw("elif") {
            let s = self.start();
            self.advance();
            vec![self.if_rest(s)?]
        } else if self.eat_kw("else") {
            self.block()?
        } else {
            vec![]
        };
        Ok(Stmt { pos: block_end(start, &body, &orelse), kind: StmtKind::If(cond, body, orelse) })
    }

    fn for_rest(&mut self, start: Pos, is_async: bool) -> PResult<Stmt> {
        let target = self.target_list()?;
        self.check_store(&target, "assign to")?;
        self.expect_kw("in")?;
        let iter = self.expressions()?;
        let body = self.block()?;
        let orelse = if self.eat_kw("else") { self.block()? } else { vec![] };
        Ok(Stmt { pos: block_end(start, &body, &orelse), kind: StmtKind::For(target, iter, body, orelse, is_async) })
    }

    fn try_rest(&mut self, start: Pos) -> PResult<Stmt> {
        let body = self.block()?;
        let mut handlers = Vec::new();
        let mut star = false;
        while self.at_kw("except") {
            let hs = self.start();
            self.advance();
            if self.eat_op("*") {
                star = true;
                if self.at_op(":") {
                    return Err(self.err("expected one or more exception types"));
                }
            }
            let mut typ = None;
            let mut name = None;
            if !self.at_op(":") {
                let ts = self.start();
                let t = self.expression()?;
                let t = if self.at_op(",") {
                    let mut items = vec![t];
                    while self.eat_op(",") {
                        if self.at_kw("as") || self.at_op(":") {
                            break;
                        }
                        items.push(self.expression()?);
                    }
                    self.node(ts, ExprKind::Tuple(items))
                } else {
                    t
                };
                typ = Some(t);
                if self.eat_kw("as") {
                    name = Some(self.name()?);
                }
            }
            let hbody = self.block()?;
            let pos = block_end(hs, &hbody, &[]);
            handlers.push(Handler { typ, name, body: hbody, pos });
        }
        let orelse = if self.eat_kw("else") { self.block()? } else { vec![] };
        let finally = if self.eat_kw("finally") { self.block()? } else { vec![] };
        if handlers.is_empty() && finally.is_empty() {
            return Err(self.err("expected 'except' or 'finally' block"));
        }
        let mut pos = block_end(start, &body, &[]);
        if let Some(h) = handlers.last() {
            pos = pos.to(h.pos);
        }
        let pos = block_end(pos, &orelse, &finally);
        Ok(Stmt { pos, kind: StmtKind::Try { body, handlers, orelse, finally, star } })
    }

    fn with_rest(&mut self, start: Pos, is_async: bool) -> PResult<Stmt> {
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
            if let Some(t) = &t {
                self.check_store(t, "assign to")?;
            }
            items.push((e, t));
            if !self.eat_op(",") {
                break;
            }
        }
        if paren {
            self.expect_op(")")?;
        }
        let body = self.block()?;
        Ok(Stmt { pos: block_end(start, &body, &[]), kind: StmtKind::With(items, body, is_async) })
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
            let start = self.start();
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
            out.push(TypeParam { kind, name, bound, default, pos: self.span(start) });
            if !self.eat_op(",") {
                self.expect_op("]")?;
                break;
            }
        }
        Ok(out)
    }

    fn funcdef(&mut self, decorators: Vec<Expr>, line: u32, is_async: bool) -> PResult<FuncDef> {
        let name = self.name()?;
        let type_params = if self.at_op("[") { self.type_params()? } else { vec![] };
        self.expect_op("(")?;
        let params = self.params(")", true)?;
        self.expect_op(")")?;
        let returns = if self.eat_op("->") { Some(self.expression()?) } else { None };
        let body = self.block()?;
        let line = decorators.first().map_or(line, |d| d.pos.line);
        Ok(FuncDef { name, params, body, decorators, returns, is_lambda: false, is_async, line, type_params })
    }

    fn params(&mut self, end: &str, annotations: bool) -> PResult<Params> {
        let mut p = Params::default();
        let mut seen_star = false;
        let mut seen_default = false;
        while !self.at_op(end) {
            if self.eat_op("/") {
                p.posonly = p.args.len();
            } else if self.eat_op("**") {
                let start = self.start();
                let name = self.name()?;
                let annotation = if annotations && self.eat_op(":") { Some(self.expression()?) } else { None };
                p.kwarg = Some(Param { name, default: None, annotation, pos: self.span(start) });
            } else if self.at_op("*") {
                let star = self.start();
                self.advance();
                seen_star = true;
                let bare_end = |t: &Tok| is_op(t, end) || is_op(t, "**");
                if self.at_op(end) || (self.at_op(",") && bare_end(self.peek_at(1))) {
                    return Err(self.err_at("named arguments must follow bare *", self.span(star)));
                }
                if !self.at_op(",") && !self.at_op(end) {
                    let start = self.start();
                    let name = self.name()?;
                    let annotation = if annotations && self.eat_op(":") { Some(self.star_or_expr()?) } else { None };
                    p.vararg = Some(Param { name, default: None, annotation, pos: self.span(start) });
                }
            } else {
                let start = self.start();
                let name = self.name()?;
                let annotation = if annotations && self.eat_op(":") { Some(self.expression()?) } else { None };
                let pos = self.span(start);
                let default = if self.eat_op("=") { Some(self.expression()?) } else { None };
                if seen_star {
                    p.kwonly.push(Param { name, default, annotation, pos });
                } else {
                    if default.is_some() {
                        seen_default = true;
                    } else if seen_default {
                        return Err(self.err("non-default argument follows default argument"));
                    }
                    p.args.push(Param { name, default, annotation, pos });
                }
            }
            if !self.eat_op(",") {
                break;
            }
        }
        let all: Vec<&Param> = p.args.iter().chain(p.vararg.iter()).chain(p.kwonly.iter()).chain(p.kwarg.iter()).collect();
        if let Some(d) = all.iter().enumerate().find(|(i, a)| all[..*i].iter().any(|b| b.name == a.name)).map(|(_, a)| a) {
            return Err(self.err_at(&format!("duplicate argument '{}' in function definition", d.name), d.pos));
        }
        Ok(p)
    }

    fn star_or_expr(&mut self) -> PResult<Expr> {
        let start = self.start();
        if self.eat_op("*") {
            let e = self.expression()?;
            Ok(self.node(start, ExprKind::Starred(Box::new(e))))
        } else {
            self.expression()
        }
    }

    fn classdef(&mut self, decorators: Vec<Expr>, line: u32) -> PResult<ClassDef> {
        let name = self.name()?;
        let type_params = if self.at_op("[") { self.type_params()? } else { vec![] };
        let bases = if self.at_op("(") {
            let open = self.start();
            self.advance();
            let a = self.call_args(open)?;
            self.expect_op(")")?;
            a
        } else {
            vec![]
        };
        let body = self.block()?;
        let line = decorators.first().map_or(line, |d| d.pos.line);
        Ok(ClassDef { name, bases, body, decorators, line, type_params })
    }

    /// `name [as asname]`, a dotted name for `import`
    fn alias(&mut self, dotted: bool) -> PResult<Alias> {
        let start = self.start();
        let name = if dotted { self.dotted_name()? } else { self.name()? };
        let asname = if self.eat_kw("as") { Some(self.name()?) } else { None };
        Ok(Alias { name, asname, pos: self.span(start) })
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
                    for t in &ts {
                        self.check_store(t, "delete")?;
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
                    let mut names = vec![self.alias(true)?];
                    while self.eat_op(",") {
                        names.push(self.alias(true)?);
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
                    let star = self.start();
                    if self.eat_op("*") {
                        names.push(Alias { name: Rc::from("*"), asname: None, pos: self.span(star) });
                    } else {
                        let paren = self.eat_op("(");
                        loop {
                            if paren && self.at_op(")") {
                                break;
                            }
                            names.push(self.alias(false)?);
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
                        let start = self.start();
                        let name = self.name()?;
                        let name = self.node(start, ExprKind::Name(name));
                        let tps = if self.at_op("[") { self.type_params()? } else { vec![] };
                        self.expect_op("=")?;
                        let v = self.expression()?;
                        return Ok(StmtKind::TypeAlias(name, tps, v));
                    }
                }
                _ => {}
            }
        }
        // expression statement / assignment
        let paren = self.at_op("(");
        let first = self.star_expressions_or_yield()?;
        if self.eat_op(":") {
            let ann = self.expression()?;
            let value = if self.eat_op("=") { Some(self.star_expressions_or_yield()?) } else { None };
            match &first.kind {
                ExprKind::Name(_) | ExprKind::Attr(..) | ExprKind::Subscript(..) => {}
                // CPython points at the first of an unparenthesized tuple
                ExprKind::Tuple(items) => {
                    let at = if paren { first.pos } else { items[0].pos };
                    return Err(self.err_at("only single target (not tuple) can be annotated", at));
                }
                ExprKind::List(_) => return Err(self.err_at("only single target (not list) can be annotated", first.pos)),
                _ => return Err(self.err_at("illegal target for annotation", first.pos)),
            }
            let simple = !paren && matches!(first.kind, ExprKind::Name(_));
            return Ok(StmtKind::AnnAssign(first, ann, value, simple));
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
                match &first.kind {
                    ExprKind::Name(_) | ExprKind::Attr(..) | ExprKind::Subscript(..) => {}
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
                if let Some(bad) = t.invalid_target() {
                    // `f() = 1` reads as a mistyped comparison, as in CPython's
                    // invalid_named_expression rule
                    let here = targets.len() == 1 && std::ptr::eq(bad, t) && reads_as_comparison(t) && !matches!(value.kind, ExprKind::Lambda(_) | ExprKind::Yield(_) | ExprKind::YieldFrom(_) | ExprKind::Unary(UnOp::Not, _));
                    let suffix = if here { " here. Maybe you meant '==' instead of '='?" } else { "" };
                    return Err(self.err_at(&format!("cannot assign to {}{}", bad.describe(), suffix), bad.pos));
                }
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

    fn try_match(&mut self, start: Pos) -> PResult<Option<Stmt>> {
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
        let mut pos = start;
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
            pos = block_end(pos, &body, &[]);
            cases.push(MatchCase { pat, guard, body });
        }
        Ok(Some(Stmt { pos, kind: StmtKind::Match(subject, cases) }))
    }

    /// a pattern node spanning from `start` to the last token taken
    fn pat(&self, start: Pos, kind: PatternKind) -> Pattern {
        Pattern { kind, pos: self.span(start) }
    }

    fn pattern_top(&mut self) -> PResult<Pattern> {
        let start = self.start();
        let p = self.pattern_as()?;
        if self.at_op(",") {
            let mut items = vec![p];
            while self.eat_op(",") {
                if self.at_op(":") || self.at_kw("if") {
                    break;
                }
                items.push(self.pattern_as()?);
            }
            return Ok(self.pat(start, PatternKind::Sequence(items)));
        }
        Ok(p)
    }

    fn pattern_as(&mut self) -> PResult<Pattern> {
        let start = self.start();
        let p = self.pattern_or()?;
        if self.eat_kw("as") {
            let n = self.name()?;
            return Ok(self.pat(start, PatternKind::As(Box::new(p), n)));
        }
        Ok(p)
    }

    fn pattern_or(&mut self) -> PResult<Pattern> {
        let start = self.start();
        let p = self.pattern_closed()?;
        if self.at_op("|") {
            let mut alts = vec![p];
            while self.eat_op("|") {
                alts.push(self.pattern_closed()?);
            }
            return Ok(self.pat(start, PatternKind::Or(alts)));
        }
        Ok(p)
    }

    fn pattern_closed(&mut self) -> PResult<Pattern> {
        let start = self.start();
        if self.eat_op("*") {
            let n = self.name()?;
            return Ok(self.pat(start, PatternKind::Star(if &*n == "_" { None } else { Some(n) })));
        }
        if self.eat_op("(") {
            if self.eat_op(")") {
                return Ok(self.pat(start, PatternKind::Sequence(vec![])));
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
                return Ok(self.pat(start, PatternKind::Sequence(items)));
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
            return Ok(self.pat(start, PatternKind::Sequence(items)));
        }
        if self.eat_op("{") {
            let mut keys = Vec::new();
            let mut pats = Vec::new();
            let mut rest = None;
            while !self.at_op("}") {
                if self.eat_op("**") {
                    rest = Some(self.name()?);
                    // the rest comes last
                    self.eat_op(",");
                    if !self.at_op("}") {
                        return Err(self.err("invalid syntax"));
                    }
                    break;
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
            return Ok(self.pat(start, PatternKind::Mapping { keys, pats, rest }));
        }
        if let Tok::Name(n) = self.peek().clone() {
            let singleton = match &*n {
                "None" => Some(Const::None),
                "True" => Some(Const::True),
                "False" => Some(Const::False),
                _ => None,
            };
            if let Some(c) = singleton {
                self.advance();
                return Ok(self.pat(start, PatternKind::Singleton(c)));
            }
            // capture, value (dotted) or class pattern
            self.advance();
            let mut e = self.node(start, ExprKind::Name(n.clone()));
            let mut dotted = false;
            while self.eat_op(".") {
                let a = self.name()?;
                e = self.node(start, ExprKind::Attr(Box::new(e), a));
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
                return Ok(self.pat(start, PatternKind::Class { cls: e, args, kw }));
            }
            if dotted {
                return Ok(self.pat(start, PatternKind::Value(e)));
            }
            return Ok(self.pat(start, PatternKind::Capture(if &*n == "_" { None } else { Some(n) })));
        }
        let v = self.pattern_value_expr()?;
        Ok(self.pat(start, PatternKind::Value(v)))
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
        let start = self.start();
        if self.eat_op("*") {
            let e = self.bitor()?;
            return Ok(self.node(start, ExprKind::Starred(Box::new(e))));
        }
        self.bitor()
    }

    fn target_list(&mut self) -> PResult<Expr> {
        let start = self.start();
        let first = self.star_or_bitor()?;
        if self.at_op(",") {
            let mut items = vec![first];
            while self.eat_op(",") {
                if self.at_kw("in") || self.at_op("=") {
                    break;
                }
                items.push(self.star_or_bitor()?);
            }
            return Ok(self.node(start, ExprKind::Tuple(items)));
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
        let start = self.start();
        self.expect_kw("yield")?;
        if self.eat_kw("from") {
            let e = self.expression()?;
            return Ok(self.node(start, ExprKind::YieldFrom(Box::new(e))));
        }
        if matches!(self.peek(), Tok::Newline | Tok::Eof) || self.at_op(")") || self.at_op("=") || self.at_op(";") || self.at_op("]") || self.at_op("}") {
            return Ok(self.node(start, ExprKind::Yield(None)));
        }
        let e = self.star_expressions()?;
        Ok(self.node(start, ExprKind::Yield(Some(Box::new(e)))))
    }

    /// a, *b, c — a bare tuple allowed
    fn star_expressions(&mut self) -> PResult<Expr> {
        let start = self.start();
        let first = self.star_named()?;
        if self.at_op(",") {
            let mut items = vec![first];
            while self.eat_op(",") {
                if self.ends_expr_list() {
                    break;
                }
                items.push(self.star_named()?);
            }
            return Ok(self.node(start, ExprKind::Tuple(items)));
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

    /// an element of a display in brackets, where an `=` after it is taken
    /// for a mistyped `==`
    fn element(&mut self) -> PResult<Expr> {
        let e = self.star_named()?;
        if self.at_op("=") {
            if let ExprKind::Name(_) = e.kind {
                self.advance();
                self.bitor()?;
                return Err(self.err_at("invalid syntax. Maybe you meant '==' or ':=' instead of '='?", self.span(e.pos)));
            }
            if reads_as_comparison(&e) {
                return Err(self.err_at(&format!("cannot assign to {} here. Maybe you meant '==' instead of '='?", e.describe()), e.pos));
            }
        }
        Ok(e)
    }

    /// the element of a comprehension, which cannot be unpacked
    fn comp_elt(&self, e: &Expr) -> PResult<()> {
        match e.kind {
            ExprKind::Starred(_) => Err(self.err_at("iterable unpacking cannot be used in comprehension", e.pos)),
            _ => Ok(()),
        }
    }

    fn star_named(&mut self) -> PResult<Expr> {
        let start = self.start();
        if self.eat_op("*") {
            let e = self.bitor()?;
            return Ok(self.node(start, ExprKind::Starred(Box::new(e))));
        }
        self.named_expr()
    }

    fn named_expr(&mut self) -> PResult<Expr> {
        let start = self.start();
        if let Tok::Name(n) = self.peek().clone() {
            if is_op(self.peek_at(1), ":=") && !is_keyword(&n) {
                self.advance();
                self.advance();
                let v = self.expression()?;
                return Ok(self.node(start, ExprKind::NamedExpr(n, Box::new(v))));
            }
        }
        self.expression()
    }

    pub fn expression(&mut self) -> PResult<Expr> {
        if self.at_kw("lambda") {
            return self.lambda();
        }
        let start = self.start();
        let e = self.disjunction()?;
        if self.at_kw("if") {
            // conditional expression (not a comprehension `if`)
            let save = self.pos;
            self.advance();
            let cond = self.disjunction()?;
            if self.eat_kw("else") {
                let other = self.expression()?;
                return Ok(self.node(start, ExprKind::IfExp(Box::new(cond), Box::new(e), Box::new(other))));
            }
            if !matches!(self.peek(), Tok::Error(_)) {
                self.pos = save;
            }
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
        let start = self.start();
        self.expect_kw("lambda")?;
        // in a replacement field, the ':' starts the format spec
        let no_lambda = "f-string: lambda expressions are not allowed without parentheses";
        let params = match self.params(":", false) {
            Err(_) if self.peek() == &Tok::FSpec => return Err(self.err(no_lambda)),
            r => r?,
        };
        if self.peek() == &Tok::FSpec {
            return Err(self.err(no_lambda));
        }
        self.expect_op(":")?;
        let body = self.expression()?;
        let ret = Stmt { pos: body.pos, kind: StmtKind::Return(Some(body)) };
        let f = FuncDef {
            name: Rc::from("<lambda>"),
            params,
            body: vec![ret],
            decorators: vec![],
            returns: None,
            is_lambda: true,
            is_async: false,
            line: start.line,
            type_params: vec![],
        };
        Ok(self.node(start, ExprKind::Lambda(Rc::new(f))))
    }

    fn disjunction(&mut self) -> PResult<Expr> {
        let start = self.start();
        let e = self.conjunction()?;
        let e = if self.at_kw("or") {
            let mut values = vec![e];
            while self.eat_kw("or") {
                values.push(self.conjunction()?);
            }
            self.node(start, ExprKind::BoolOr(values))
        } else {
            e
        };
        self.last_disj = Some((start, self.pos));
        Ok(e)
    }

    fn conjunction(&mut self) -> PResult<Expr> {
        let start = self.start();
        let e = self.inversion()?;
        if !self.at_kw("and") {
            return Ok(e);
        }
        let mut values = vec![e];
        while self.eat_kw("and") {
            values.push(self.inversion()?);
        }
        Ok(self.node(start, ExprKind::BoolAnd(values)))
    }

    fn inversion(&mut self) -> PResult<Expr> {
        let start = self.start();
        if self.eat_kw("not") {
            let e = self.inversion()?;
            return Ok(self.node(start, ExprKind::Unary(UnOp::Not, Box::new(e))));
        }
        self.comparison()
    }

    fn comparison(&mut self) -> PResult<Expr> {
        let start = self.start();
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
            Ok(self.node(start, ExprKind::Compare(Box::new(e), ops)))
        }
    }

    /// a left-associative run of binary operators over `operand`
    fn binary(&mut self, operand: fn(&mut Self) -> PResult<Expr>, op_of: fn(&Tok) -> Option<BinOp>) -> PResult<Expr> {
        let start = self.start();
        let mut e = operand(self)?;
        while let Some(op) = op_of(self.peek()) {
            self.advance();
            let r = operand(self)?;
            e = self.node(start, ExprKind::Bin(op, Box::new(e), Box::new(r)));
        }
        Ok(e)
    }
    fn bitor(&mut self) -> PResult<Expr> {
        self.binary(Self::bitxor, |t| is_op(t, "|").then_some(BinOp::BitOr))
    }
    fn bitxor(&mut self) -> PResult<Expr> {
        self.binary(Self::bitand, |t| is_op(t, "^").then_some(BinOp::BitXor))
    }
    fn bitand(&mut self) -> PResult<Expr> {
        self.binary(Self::shift, |t| is_op(t, "&").then_some(BinOp::BitAnd))
    }
    fn shift(&mut self) -> PResult<Expr> {
        self.binary(Self::sum_expr, |t| match t {
            Tok::Op("<<") => Some(BinOp::LShift),
            Tok::Op(">>") => Some(BinOp::RShift),
            _ => None,
        })
    }
    fn sum_expr(&mut self) -> PResult<Expr> {
        self.binary(Self::term, |t| match t {
            Tok::Op("+") => Some(BinOp::Add),
            Tok::Op("-") => Some(BinOp::Sub),
            _ => None,
        })
    }
    fn term(&mut self) -> PResult<Expr> {
        self.binary(Self::factor, |t| match t {
            Tok::Op("*") => Some(BinOp::Mul),
            Tok::Op("/") => Some(BinOp::Div),
            Tok::Op("//") => Some(BinOp::FloorDiv),
            Tok::Op("%") => Some(BinOp::Mod),
            Tok::Op("@") => Some(BinOp::MatMul),
            _ => None,
        })
    }
    fn factor(&mut self) -> PResult<Expr> {
        let start = self.start();
        let op = match self.peek() {
            Tok::Op("-") => UnOp::Neg,
            Tok::Op("+") => UnOp::Pos,
            Tok::Op("~") => UnOp::Invert,
            _ => return self.power(),
        };
        self.advance();
        let e = self.factor()?;
        Ok(self.node(start, ExprKind::Unary(op, Box::new(e))))
    }
    fn power(&mut self) -> PResult<Expr> {
        let start = self.start();
        let e = self.await_primary()?;
        if self.eat_op("**") {
            let r = self.factor()?;
            return Ok(self.node(start, ExprKind::Bin(BinOp::Pow, Box::new(e), Box::new(r))));
        }
        Ok(e)
    }
    fn await_primary(&mut self) -> PResult<Expr> {
        let start = self.start();
        if self.eat_kw("await") {
            let e = self.primary()?;
            return Ok(self.node(start, ExprKind::Await(Box::new(e))));
        }
        self.primary()
    }

    fn primary(&mut self) -> PResult<Expr> {
        let start = self.start();
        let mut e = self.atom()?;
        loop {
            if self.eat_op(".") {
                let n = match self.advance() {
                    Tok::Name(n) => n,
                    _ => return Err(self.err("invalid syntax")),
                };
                e = self.node(start, ExprKind::Attr(Box::new(e), n));
            } else if self.at_op("(") {
                let open = self.start();
                self.advance();
                let args = self.call_args(open)?;
                self.expect_op(")")?;
                e = self.node(start, ExprKind::Call { func: Box::new(e), args });
            } else if self.eat_op("[") {
                let idx = self.slices()?;
                self.expect_op("]")?;
                e = self.node(start, ExprKind::Subscript(Box::new(e), Box::new(idx)));
            } else {
                break;
            }
        }
        Ok(e)
    }

    fn slices(&mut self) -> PResult<Expr> {
        let start = self.start();
        let first = self.slice_item()?;
        if self.at_op(",") {
            let mut items = vec![first];
            while self.eat_op(",") {
                if self.at_op("]") {
                    break;
                }
                items.push(self.slice_item()?);
            }
            return Ok(self.node(start, ExprKind::Tuple(items)));
        }
        Ok(first)
    }

    fn slice_item(&mut self) -> PResult<Expr> {
        let start = self.start();
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
        Ok(self.node(start, ExprKind::Slice(lo, hi, step)))
    }

    /// the arguments of a call or class bases after the `(` at `open`
    fn call_args(&mut self, open: Pos) -> PResult<Vec<Arg>> {
        let mut args = Vec::new();
        while !self.at_op(")") {
            let start = self.start();
            if self.eat_op("**") {
                let e = self.expression()?;
                args.push(Arg::StarStar(e, self.span(start)));
            } else if self.eat_op("*") {
                if args.iter().any(|a| matches!(a, Arg::StarStar(..))) {
                    return Err(self.err("iterable argument unpacking follows keyword argument unpacking"));
                }
                let e = self.expression()?;
                args.push(Arg::Star(self.node(start, ExprKind::Starred(Box::new(e)))));
            } else if matches!(self.peek(), Tok::Name(_)) && is_op(self.peek_at(1), "=") {
                let n = self.name()?;
                if args.iter().any(|a| matches!(a, Arg::Kw(k, ..) if *k == n)) {
                    return Err(self.err(&format!("keyword argument repeated: {}", n)));
                }
                self.advance();
                let e = self.expression()?;
                args.push(Arg::Kw(n, e, self.span(start)));
            } else {
                if args.iter().any(|a| matches!(a, Arg::StarStar(..))) {
                    return Err(self.err("positional argument follows keyword argument unpacking"));
                }
                if args.iter().any(|a| matches!(a, Arg::Kw(..))) {
                    return Err(self.err("positional argument follows keyword argument"));
                }
                let e = self.named_expr()?;
                if self.at_op("=") {
                    self.advance();
                    return Err(self.err_at("expression cannot contain assignment, perhaps you meant \"==\"?", self.span(e.pos)));
                }
                if self.at_kw("for") || self.at_kw("async") {
                    let comps = self.comp_for()?;
                    if !args.is_empty() || self.at_op(",") {
                        let bare = self.span(e.pos);
                        if self.at_op(",") && matches!(self.peek_at(1), Tok::Error(_)) {
                            // what follows the comma is a tokenizer error, seen first
                            self.advance();
                            return Err(self.err(""));
                        }
                        return Err(self.err_at("Generator expression must be parenthesized", bare));
                    }
                    // the generator spans the call's parentheses
                    let mut g = self.node(open, ExprKind::GenExp(Box::new(e), comps));
                    if self.at_op(")") {
                        let t = &self.toks[self.pos];
                        (g.pos.end_line, g.pos.end_col) = (t.end_line, t.end_bcol);
                    }
                    args.push(Arg::Pos(g));
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
            self.check_store(&target, "assign to")?;
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
        let start = self.start();
        let t = self.peek().clone();
        let leaf = |p: &mut Self, k: ExprKind| {
            p.advance();
            Ok(p.node(start, k))
        };
        match t {
            Tok::Name(n) => match &*n {
                "None" => leaf(self, ExprKind::Const(Const::None)),
                "True" => leaf(self, ExprKind::Const(Const::True)),
                "False" => leaf(self, ExprKind::Const(Const::False)),
                _ => {
                    if is_keyword(&n) && &*n != "match" && &*n != "case" && &*n != "type" && &*n != "_" {
                        return Err(self.err("invalid syntax"));
                    }
                    leaf(self, ExprKind::Name(n))
                }
            },
            Tok::Int(v) => leaf(self, ExprKind::Const(Const::Int(v))),
            Tok::BigInt(s) => leaf(self, ExprKind::Const(Const::BigInt(s))),
            Tok::Float(v) => leaf(self, ExprKind::Const(Const::Float(v))),
            Tok::Imag(v) => leaf(self, ExprKind::Const(Const::Imag(v))),
            Tok::Str { .. } | Tok::FStart => self.strings(),
            Tok::Op("...") => leaf(self, ExprKind::Const(Const::Ellipsis)),
            Tok::Op("(") => {
                self.advance();
                if self.eat_op(")") {
                    return Ok(self.node(start, ExprKind::Tuple(vec![])));
                }
                if self.at_kw("yield") {
                    let y = self.yield_expr()?;
                    self.expect_op(")")?;
                    return Ok(y);
                }
                let first = self.element()?;
                if self.at_kw("for") || self.at_kw("async") {
                    self.comp_elt(&first)?;
                    let comps = self.comp_for()?;
                    self.expect_op(")")?;
                    return Ok(self.node(start, ExprKind::GenExp(Box::new(first), comps)));
                }
                if self.at_op(",") {
                    let mut items = vec![first];
                    while self.eat_op(",") {
                        if self.at_op(")") {
                            break;
                        }
                        items.push(self.element()?);
                    }
                    self.expect_op(")")?;
                    return Ok(self.node(start, ExprKind::Tuple(items)));
                }
                self.expect_op(")")?;
                Ok(first)
            }
            Tok::Op("[") => {
                self.advance();
                if self.eat_op("]") {
                    return Ok(self.node(start, ExprKind::List(vec![])));
                }
                let first = self.element()?;
                if self.at_kw("for") || self.at_kw("async") {
                    self.comp_elt(&first)?;
                    let comps = self.comp_for()?;
                    self.expect_op("]")?;
                    return Ok(self.node(start, ExprKind::ListComp(Box::new(first), comps)));
                }
                let mut items = vec![first];
                while self.eat_op(",") {
                    if self.at_op("]") {
                        break;
                    }
                    items.push(self.element()?);
                }
                self.expect_op("]")?;
                Ok(self.node(start, ExprKind::List(items)))
            }
            Tok::Op("{") => {
                self.advance();
                if self.eat_op("}") {
                    return Ok(self.node(start, ExprKind::Dict(vec![])));
                }
                if self.eat_op("**") {
                    let v = self.bitor()?;
                    if self.at_kw("for") || self.at_kw("async") {
                        return Err(self.err_at("dict unpacking cannot be used in dict comprehension", self.span(start.to(self.start()))));
                    }
                    let mut items = vec![(None, v)];
                    self.dict_rest(&mut items)?;
                    return Ok(self.node(start, ExprKind::Dict(items)));
                }
                let first = self.element()?;
                if self.eat_op(":") {
                    let v = self.expression()?;
                    if self.at_kw("for") || self.at_kw("async") {
                        let comps = self.comp_for()?;
                        self.expect_op("}")?;
                        return Ok(self.node(start, ExprKind::DictComp(Box::new(first), Box::new(v), comps)));
                    }
                    let mut items = vec![(Some(first), v)];
                    self.dict_rest(&mut items)?;
                    return Ok(self.node(start, ExprKind::Dict(items)));
                }
                if self.at_kw("for") || self.at_kw("async") {
                    self.comp_elt(&first)?;
                    let comps = self.comp_for()?;
                    self.expect_op("}")?;
                    return Ok(self.node(start, ExprKind::SetComp(Box::new(first), comps)));
                }
                let mut items = vec![first];
                while self.eat_op(",") {
                    if self.at_op("}") {
                        break;
                    }
                    items.push(self.element()?);
                }
                self.expect_op("}")?;
                Ok(self.node(start, ExprKind::Set(items)))
            }
            Tok::Op("}") | Tok::Op(")") | Tok::Op("]") if self.closes_bracket() => Err(self.err("invalid syntax")),
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

    /// adjacent string literals, joined; an f-string among them makes a JoinedStr
    fn strings(&mut self) -> PResult<Expr> {
        let start = self.start();
        let mut parts: Vec<Expr> = Vec::new();
        let mut lit = Lit::default();
        let mut any_f = false;
        let mut bytes = None;
        loop {
            let (b, f) = match self.peek() {
                Tok::Str { bytes, .. } => (*bytes, false),
                Tok::FStart => (false, true),
                _ => break,
            };
            if bytes.is_some_and(|pb| pb != b) {
                return Err(self.err_at("cannot mix bytes and nonbytes literals", self.span(start)));
            }
            bytes = Some(b);
            let at = self.start();
            if let Tok::Str { s, .. } = self.advance() {
                lit.push(&s, self.span(at));
            }
            if f {
                any_f = true;
                self.fstring(&mut parts, &mut lit)?;
            }
        }
        let pos = self.span(start);
        if bytes == Some(true) {
            let v: Vec<u8> = lit.text.chars().map(|c| c as u32 as u8).collect();
            return Ok(Expr::new(ExprKind::Const(Const::Bytes(Rc::from(v))), pos));
        }
        if !any_f {
            return Ok(Expr::new(ExprKind::Const(Const::Str(Rc::from(lit.text.as_str()))), pos));
        }
        lit.flush(&mut parts);
        Ok(Expr::new(ExprKind::JoinedStr(parts), pos))
    }

    /// the rest of an f-string after FStart: its text and replacement fields
    fn fstring(&mut self, parts: &mut Vec<Expr>, lit: &mut Lit) -> PResult<()> {
        loop {
            let at = self.start();
            match self.peek().clone() {
                Tok::FMiddle(s) => {
                    self.advance();
                    lit.push(&s, self.span(at));
                }
                Tok::FEnd => {
                    self.advance();
                    return Ok(());
                }
                Tok::Op("{") => {
                    self.advance();
                    let (debug, field) = self.replacement_field(at)?;
                    lit.push(&debug, debug_pos(&debug, &field));
                    lit.flush(parts);
                    parts.push(field);
                }
                _ => return Err(self.err("invalid syntax")),
            }
        }
    }

    /// `expr [=] [!conv] [:spec] }` after a field's `{` at `open`; the text
    /// a self-documenting field shows, and the formatted value
    fn replacement_field(&mut self, open: Pos) -> PResult<(String, Expr)> {
        let before = match self.peek() {
            Tok::FDebug(_) => Some('='),
            Tok::Op("!") => Some('!'),
            Tok::FSpec => Some(':'),
            Tok::Op("}") => Some('}'),
            _ => None,
        };
        if let Some(c) = before {
            return Err(self.err(&format!("f-string: valid expression required before '{}'", c)));
        }
        let after_brace = "f-string: expecting a valid expression after '{'";
        if !self.starts_expr() && !self.at_op("*") && !self.at_kw("yield") {
            return Err(self.err(after_brace));
        }
        let value = if self.at_kw("yield") { self.yield_expr() } else { self.star_expressions() };
        let value = match value {
            Err(e) if e.msg == "invalid syntax" && !matches!(self.peek(), Tok::Error(_)) => return Err(self.err(after_brace)),
            v => v?,
        };
        if matches!(value.kind, ExprKind::Starred(_)) {
            return Err(self.err("can't use starred expression here"));
        }
        let mut debug = String::new();
        if let Tok::FDebug(t) = self.peek().clone() {
            self.advance();
            debug = t;
        }
        let mut conv = '\0';
        if self.at_op("!") {
            let bang = self.toks[self.pos].col;
            self.advance();
            match self.peek().clone() {
                Tok::FSpec | Tok::Op("}") => return Err(self.err("f-string: missing conversion character")),
                Tok::Name(n) => {
                    if self.toks[self.pos].col != bang + 1 {
                        return Err(self.err("f-string: conversion type must come right after the exclamanation mark"));
                    }
                    if matches!(self.peek_at(1), Tok::Error(_)) {
                        self.advance();
                        return Err(self.err(""));
                    }
                    if !matches!(&*n, "s" | "r" | "a") {
                        return Err(self.err(&format!("f-string: invalid conversion character '{}': expected 's', 'r', or 'a'", n)));
                    }
                    self.advance();
                    conv = n.chars().next().unwrap();
                }
                _ => return Err(self.err("f-string: invalid conversion character")),
            }
        }
        let mut spec = None;
        let spec_start = self.start();
        if self.eat_tok(&Tok::FSpec) {
            let mut sp = Vec::new();
            loop {
                let at = self.start();
                match self.peek().clone() {
                    Tok::FMiddle(s) => {
                        self.advance();
                        sp.push(self.node(at, ExprKind::Const(Const::Str(Rc::from(s.as_str())))));
                    }
                    Tok::Op("{") => {
                        self.advance();
                        let (d, f) = self.replacement_field(at)?;
                        if !d.is_empty() {
                            sp.push(Expr::new(ExprKind::Const(Const::Str(Rc::from(d.as_str()))), debug_pos(&d, &f)));
                        }
                        sp.push(f);
                    }
                    _ => break,
                }
            }
            spec = Some(Box::new(self.node(spec_start, ExprKind::JoinedStr(sp))));
        }
        if !self.eat_op("}") {
            let msg = if spec.is_some() {
                "f-string: expecting '}'"
            } else if conv != '\0' {
                "f-string: expecting ':' or '}'"
            } else if !debug.is_empty() {
                "f-string: expecting '!', or ':', or '}'"
            } else if self.starts_expr() {
                return Err(self.missing_comma());
            } else {
                "f-string: expecting '=', or '!', or ':', or '}'"
            };
            return Err(self.err(msg));
        }
        let conv = if conv == '\0' && !debug.is_empty() && spec.is_none() { 'r' } else { conv };
        Ok((debug, self.node(open, ExprKind::FormattedValue { value: Box::new(value), conv, spec })))
    }
}

/// the literal text of a JoinedStr being gathered, and its span
#[derive(Default)]
struct Lit {
    text: String,
    pos: Option<Pos>,
}

impl Lit {
    fn push(&mut self, s: &str, pos: Pos) {
        if s.is_empty() {
            return;
        }
        self.text.push_str(s);
        self.pos = Some(self.pos.map_or(pos, |p| p.to(pos)));
    }

    /// ends the text gathered so far as a Constant part
    fn flush(&mut self, parts: &mut Vec<Expr>) {
        if let Some(pos) = self.pos.take() {
            parts.push(Expr::new(ExprKind::Const(Const::Str(Rc::from(std::mem::take(&mut self.text).as_str()))), pos));
        }
    }
}

/// the span of a self-documenting field's text: from its expression on
fn debug_pos(text: &str, field: &Expr) -> Pos {
    let start = match &field.kind {
        ExprKind::FormattedValue { value, .. } => value.pos,
        _ => field.pos,
    };
    let (end_line, end_col) = match text.rfind('\n') {
        Some(i) => (start.line + text.matches('\n').count() as u32, (text.len() - i - 1) as u32),
        None => (start.line, start.col + text.len() as u32),
    };
    Pos { end_line, end_col, ..start }
}

/// a compound statement's span: from `start` to the end of its last body
fn block_end(start: Pos, body: &[Stmt], orelse: &[Stmt]) -> Pos {
    match orelse.last().or(body.last()) {
        Some(s) => start.to(s.pos),
        None => start,
    }
}

fn func_stmt(start: Pos, f: FuncDef) -> Stmt {
    Stmt { pos: block_end(start, &f.body, &[]), kind: StmtKind::FuncDef(Rc::new(f)) }
}

fn class_stmt(start: Pos, c: ClassDef) -> Stmt {
    Stmt { pos: block_end(start, &c.body, &[]), kind: StmtKind::ClassDef(Rc::new(c)) }
}

/// whether an expression is one CPython's grammar takes for the left of a
/// mistyped `==`: a bitwise_or, but not a display or True/False/None
fn reads_as_comparison(e: &Expr) -> bool {
    !matches!(
        e.kind,
        ExprKind::Lambda(_)
            | ExprKind::IfExp(..)
            | ExprKind::BoolAnd(_)
            | ExprKind::BoolOr(_)
            | ExprKind::Unary(UnOp::Not, _)
            | ExprKind::Compare(..)
            | ExprKind::Tuple(_)
            | ExprKind::List(_)
            | ExprKind::GenExp(..)
            | ExprKind::Starred(_)
            | ExprKind::Const(Const::True | Const::False | Const::None)
    )
}

pub fn is_keyword(s: &str) -> bool {
    matches!(
        s,
        "False" | "None" | "True" | "and" | "as" | "assert" | "async" | "await" | "break" | "class" | "continue" | "def" | "del" | "elif" | "else"
            | "except" | "finally" | "for" | "from" | "global" | "if" | "import" | "in" | "is" | "lambda" | "nonlocal" | "not" | "or" | "pass"
            | "raise" | "return" | "try" | "while" | "with" | "yield"
    )
}
