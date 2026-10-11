// SPDX-License-Identifier: MIT
//! Syntax tree to bytecode: a symbol table pass that decides where each name
//! lives (fast local, cell, global, class namespace), then code generation.

use crate::ast::*;
use crate::code::{Code, Op};
use crate::value::{NativeObj, ObjKind, PyStr, Value};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

pub struct CompileError {
    pub msg: String,
    pub line: u32,
    /// the source of the error, when known
    pub span: Option<Pos>,
}

impl CompileError {
    pub fn new(msg: impl Into<String>, line: u32) -> CompileError {
        CompileError { msg: msg.into(), line, span: None }
    }

    /// an error spanning a node
    pub fn at(msg: impl Into<String>, pos: Pos) -> CompileError {
        CompileError { msg: msg.into(), line: pos.line, span: Some(pos) }
    }

    /// as the parser's error, the byte columns of the span made character
    /// columns of `src` when it is known
    pub fn syntax_error(&self, src: Option<&str>) -> crate::parser::SyntaxError {
        let mut e = crate::parser::SyntaxError::new(self.msg.clone(), self.line, 0);
        if let Some(p) = self.span {
            let col = |line: u32, b: u32| {
                let text = src.and_then(|s| s.lines().nth((line as usize).wrapping_sub(1)));
                match text.and_then(|t| t.get(..b as usize)) {
                    Some(head) => head.chars().count() as u32 + 1,
                    None => b + 1,
                }
            };
            (e.col, e.end_line, e.end_col) = (col(p.line, p.col), p.end_line, col(p.end_line, p.end_col));
        }
        e
    }
}

type CResult<T> = Result<T, CompileError>;

// ---------------------------------------------------------------------------
// symbol table

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ScopeKind {
    Module,
    Class,
    Function,
}

struct Scope {
    kind: ScopeKind,
    parent: Option<usize>,
    params: Vec<Name>,
    assigned: HashSet<Name>,
    globals: HashSet<Name>,
    nonlocals: HashSet<Name>,
    /// where each global or nonlocal name was first declared so
    directives: HashMap<Name, Pos>,
    used: HashSet<Name>,
    is_gen: bool,
    is_comp: bool,
    /// a list, set or dict comprehension: compiled into the code of its host,
    /// the nearest enclosing scope that is not one (PEP 709)
    inlined: bool,
    /// an `async def`
    is_async: bool,
    /// awaits (CPython's ste_coroutine): an async comprehension, or the
    /// module under PyCF_ALLOW_TOP_LEVEL_AWAIT
    coro: bool,
    // results
    cells: HashSet<Name>,
    frees: HashSet<Name>,
    order: Vec<Name>,
}

impl Scope {
    fn new(kind: ScopeKind, parent: Option<usize>) -> Scope {
        Scope {
            kind,
            parent,
            params: vec![],
            assigned: HashSet::new(),
            globals: HashSet::new(),
            nonlocals: HashSet::new(),
            directives: HashMap::new(),
            used: HashSet::new(),
            is_gen: false,
            is_comp: false,
            inlined: false,
            is_async: false,
            coro: false,
            cells: HashSet::new(),
            frees: HashSet::new(),
            order: vec![],
        }
    }
    fn is_local(&self, n: &Name) -> bool {
        (self.assigned.contains(n) || self.params.contains(n)) && !self.globals.contains(n) && !self.nonlocals.contains(n)
    }
    fn assign(&mut self, n: &Name) {
        if !self.assigned.contains(n) {
            self.order.push(n.clone());
        }
        self.assigned.insert(n.clone());
    }
}

struct SymTable {
    scopes: Vec<Scope>,
    by_node: HashMap<usize, usize>,
    future_annotations: bool,
    /// the inlined comprehensions of each host scope
    inlined: HashMap<usize, Vec<usize>>,
}

fn key<T>(r: &T) -> usize {
    r as *const T as usize
}

struct Collector<'a> {
    st: &'a mut SymTable,
    cur: usize,
    /// enclosing class names, for private name mangling
    cls: Vec<String>,
    /// compile()'s PyCF_ALLOW_TOP_LEVEL_AWAIT
    top_await: bool,
    /// the span of the node being visited, for errors
    pos: Pos,
}

impl<'a> Collector<'a> {
    fn mg(&self, n: &Name) -> Name {
        match self.cls.last() {
            Some(c) => mangle_attr(c, n),
            None => n.clone(),
        }
    }

    fn sc(&mut self) -> &mut Scope {
        &mut self.st.scopes[self.cur]
    }
    fn new_scope(&mut self, kind: ScopeKind, node: usize) -> usize {
        let id = self.st.scopes.len();
        self.st.scopes.push(Scope::new(kind, Some(self.cur)));
        self.st.by_node.insert(node, id);
        id
    }

    fn stmts(&mut self, body: &[Stmt]) -> CResult<()> {
        for s in body {
            self.stmt(s)?;
        }
        Ok(())
    }

    fn target(&mut self, e: &Expr) -> CResult<()> {
        match &e.kind {
            ExprKind::Name(n) => {
                let n = self.mg(n);
                self.sc().assign(&n);
            }
            ExprKind::Tuple(items) | ExprKind::List(items) => {
                for i in items {
                    self.target(i)?;
                }
            }
            ExprKind::Starred(inner) => self.target(inner)?,
            ExprKind::Attr(obj, _) => self.expr(obj)?,
            ExprKind::Subscript(obj, idx) => {
                self.expr(obj)?;
                self.expr(idx)?;
            }
            _ => self.expr(e)?,
        }
        Ok(())
    }

    fn params(&mut self, p: &Params) -> CResult<()> {
        for a in p.args.iter().chain(p.kwonly.iter()) {
            if let Some(d) = &a.default {
                self.expr(d)?;
            }
        }
        if !self.st.future_annotations {
            for a in p.args.iter().chain(p.vararg.iter()).chain(p.kwonly.iter()).chain(p.kwarg.iter()) {
                if let Some(e) = &a.annotation {
                    self.expr(e)?;
                }
            }
        }
        Ok(())
    }

    fn func(&mut self, f: &Rc<FuncDef>) -> CResult<()> {
        for d in &f.decorators {
            self.expr(d)?;
        }
        self.params(&f.params)?;
        if let (Some(r), false) = (&f.returns, self.st.future_annotations) {
            self.expr(r)?;
        }
        let id = self.new_scope(ScopeKind::Function, Rc::as_ptr(f) as usize);
        let save = self.cur;
        self.cur = id;
        self.sc().is_async = f.is_async;
        let p = &f.params;
        let mut names: Vec<Name> = p.args.iter().map(|a| self.mg(&a.name)).collect();
        if let Some(v) = &p.vararg {
            names.push(self.mg(&v.name));
        }
        names.extend(p.kwonly.iter().map(|a| self.mg(&a.name)));
        if let Some(v) = &p.kwarg {
            names.push(self.mg(&v.name));
        }
        self.sc().params = names;
        self.stmts(&f.body)?;
        self.cur = save;
        Ok(())
    }

    fn stmt(&mut self, s: &Stmt) -> CResult<()> {
        self.pos = s.pos;
        match &s.kind {
            StmtKind::Expr(e) => self.expr(e)?,
            StmtKind::Assign(ts, v) => {
                self.expr(v)?;
                for t in ts {
                    self.target(t)?;
                }
            }
            StmtKind::AugAssign(t, _, v) => {
                self.expr(v)?;
                if let ExprKind::Name(n) = &t.kind {
                    let n = n.clone();
                    self.sc().used.insert(n.clone());
                    self.sc().assign(&n);
                } else {
                    self.target(t)?;
                }
            }
            StmtKind::AnnAssign(t, ann, v, simple) => {
                if let ExprKind::Name(n) = &t.kind {
                    let n = self.mg(n);
                    let sc = self.sc();
                    let what = if sc.globals.contains(&n) { "global" } else if sc.nonlocals.contains(&n) { "nonlocal" } else { "" };
                    if !what.is_empty() {
                        return Err(CompileError::at(format!("annotated name '{}' can't be {}", n, what), s.pos));
                    }
                }
                if !self.st.future_annotations {
                    self.expr(ann)?;
                }
                if let Some(v) = v {
                    self.expr(v)?;
                    self.target(t)?;
                } else if let ExprKind::Name(n) = &t.kind {
                    // a bare annotation still makes the name local
                    if self.sc().kind == ScopeKind::Function && *simple {
                        let n = n.clone();
                        self.sc().assign(&n);
                    }
                } else {
                    self.target(t)?;
                }
            }
            StmtKind::If(c, a, b) | StmtKind::While(c, a, b) => {
                self.expr(c)?;
                self.stmts(a)?;
                self.stmts(b)?;
            }
            StmtKind::For(t, it, a, b, _) => {
                self.expr(it)?;
                self.target(t)?;
                self.stmts(a)?;
                self.stmts(b)?;
            }
            StmtKind::Break | StmtKind::Continue | StmtKind::Pass => {}
            StmtKind::Return(v) => {
                if let Some(v) = v {
                    self.expr(v)?;
                }
            }
            StmtKind::FuncDef(f) => {
                self.func(f)?;
                let n = self.mg(&f.name);
                self.sc().assign(&n);
            }
            StmtKind::ClassDef(c) => {
                for d in &c.decorators {
                    self.expr(d)?;
                }
                for b in &c.bases {
                    match b {
                        Arg::Pos(e) | Arg::Star(e) | Arg::Kw(_, e, _) | Arg::StarStar(e, _) => self.expr(e)?,
                    }
                }
                let id = self.new_scope(ScopeKind::Class, Rc::as_ptr(c) as usize);
                let save = self.cur;
                self.cur = id;
                self.cls.push(c.name.to_string());
                self.stmts(&c.body)?;
                self.cls.pop();
                self.cur = save;
                let n = self.mg(&c.name);
                self.sc().assign(&n);
            }
            StmtKind::Global(ns) | StmtKind::Nonlocal(ns) => {
                let what = if matches!(s.kind, StmtKind::Global(_)) { "global" } else { "nonlocal" };
                if what == "nonlocal" && self.sc().kind == ScopeKind::Module {
                    return Err(CompileError::at("nonlocal declaration not allowed at module level", s.pos));
                }
                for n in ns {
                    let n = &self.mg(n);
                    let sc = self.sc();
                    let prior = if sc.params.contains(n) {
                        Some(format!("name '{}' is parameter and {}", n, what))
                    } else if sc.used.contains(n) {
                        Some(format!("name '{}' is used prior to {} declaration", n, what))
                    } else if sc.assigned.contains(n) {
                        Some(format!("name '{}' is assigned to before {} declaration", n, what))
                    } else {
                        None
                    };
                    if let Some(msg) = prior {
                        return Err(CompileError::at(msg, s.pos));
                    }
                    // both: an error where the first of them is
                    let other = if what == "global" { &sc.nonlocals } else { &sc.globals };
                    if other.contains(n) {
                        return Err(CompileError::at(format!("name '{}' is nonlocal and global", n), sc.directives[n]));
                    }
                    sc.directives.entry(n.clone()).or_insert(s.pos);
                    if what == "nonlocal" {
                        sc.nonlocals.insert(n.clone());
                        continue;
                    }
                    sc.globals.insert(n.clone());
                    // as in CPython, the module's own uses of the name
                    // become explicitly global too
                    self.st.scopes[0].globals.insert(n.clone());
                }
            }
            StmtKind::Del(ts) => {
                for t in ts {
                    self.target(t)?;
                }
            }
            StmtKind::Raise(a, b) => {
                if let Some(a) = a {
                    self.expr(a)?;
                }
                if let Some(b) = b {
                    self.expr(b)?;
                }
            }
            StmtKind::Try { body, handlers, orelse, finally, .. } => {
                self.stmts(body)?;
                for h in handlers {
                    if let Some(t) = &h.typ {
                        self.expr(t)?;
                    }
                    if let Some(n) = &h.name {
                        let n = n.clone();
                        self.sc().assign(&n);
                    }
                    self.stmts(&h.body)?;
                }
                self.stmts(orelse)?;
                self.stmts(finally)?;
            }
            StmtKind::With(items, body, _) => {
                for (e, t) in items {
                    self.expr(e)?;
                    if let Some(t) = t {
                        self.target(t)?;
                    }
                }
                self.stmts(body)?;
            }
            StmtKind::Import(names) => {
                for Alias { name: m, asname: alias, .. } in names {
                    let n: Name = match alias {
                        Some(a) => a.clone(),
                        None => Rc::from(m.split('.').next().unwrap()),
                    };
                    self.sc().assign(&n);
                }
            }
            StmtKind::ImportFrom { module, names, .. } => {
                if &**module == "__future__" && names.iter().any(|a| &*a.name == "annotations") {
                    self.st.future_annotations = true;
                }
                for Alias { name: n, asname: alias, pos } in names {
                    if &**n == "*" {
                        if self.sc().kind != ScopeKind::Module {
                            return Err(CompileError::at("import * only allowed at module level", *pos));
                        }
                        continue;
                    }
                    let b = alias.clone().unwrap_or_else(|| n.clone());
                    self.sc().assign(&b);
                }
            }
            StmtKind::Assert(a, b) => {
                self.expr(a)?;
                if let Some(b) = b {
                    self.expr(b)?;
                }
            }
            StmtKind::TypeAlias(..) => unreachable!("lowered"),
            StmtKind::Match(subj, cases) => {
                self.expr(subj)?;
                for c in cases {
                    self.pattern(&c.pat)?;
                    if let Some(g) = &c.guard {
                        self.expr(g)?;
                    }
                    self.stmts(&c.body)?;
                }
            }
        }
        Ok(())
    }

    fn pattern(&mut self, p: &Pattern) -> CResult<()> {
        match &p.kind {
            PatternKind::Capture(Some(n)) | PatternKind::Star(Some(n)) => {
                let n = n.clone();
                self.sc().assign(&n);
            }
            PatternKind::Capture(None) | PatternKind::Star(None) | PatternKind::Singleton(_) => {}
            PatternKind::Value(e) => self.expr(e)?,
            PatternKind::Sequence(ps) | PatternKind::Or(ps) => {
                for q in ps {
                    self.pattern(q)?;
                }
            }
            PatternKind::Mapping { keys, pats, rest } => {
                for k in keys {
                    self.expr(k)?;
                }
                for q in pats {
                    self.pattern(q)?;
                }
                if let Some(r) = rest {
                    let r = r.clone();
                    self.sc().assign(&r);
                }
            }
            PatternKind::Class { cls, args, kw } => {
                self.expr(cls)?;
                for q in args {
                    self.pattern(q)?;
                }
                for (_, q) in kw {
                    self.pattern(q)?;
                }
            }
            PatternKind::As(q, n) => {
                self.pattern(q)?;
                let n = n.clone();
                self.sc().assign(&n);
            }
        }
        Ok(())
    }

    fn comp(&mut self, e: &Expr, elts: &[&Expr], comps: &[Comp], genexp: bool) -> CResult<()> {
        // the first iterable is evaluated in the enclosing scope
        self.expr(&comps[0].iter)?;
        let id = self.new_scope(ScopeKind::Function, key(e));
        let save = self.cur;
        self.cur = id;
        self.sc().is_comp = true;
        if genexp {
            self.sc().params = vec![Rc::from(".0")];
        } else {
            self.sc().inlined = true;
        }
        for (i, c) in comps.iter().enumerate() {
            if i > 0 {
                self.expr(&c.iter)?;
            }
            // a name an assignment expression in this comprehension bound
            let mut names = vec![];
            target_names(&c.target, &mut names);
            let sc = &self.st.scopes[id];
            if let Some(t) = names.into_iter().find(|t| matches!(&t.kind, ExprKind::Name(n) if sc.nonlocals.contains(n) || sc.globals.contains(n))) {
                let ExprKind::Name(n) = &t.kind else { unreachable!() };
                return Err(CompileError::at(format!("comprehension inner loop cannot rebind assignment expression target '{}'", n), t.pos));
            }
            self.target(&c.target)?;
            for f in &c.ifs {
                self.expr(f)?;
            }
        }
        for e in elts {
            self.expr(e)?;
        }
        self.cur = save;
        // an async comprehension is a coroutine the enclosing scope awaits;
        // an async generator expression is fine anywhere
        if comps.iter().any(|c| c.is_async) || self.st.scopes[id].coro {
            self.st.scopes[id].coro = true;
            if !genexp {
                let outer = &self.st.scopes[save];
                let ok = outer.is_async || outer.is_comp || (self.top_await && outer.kind == ScopeKind::Module);
                if !ok {
                    return Err(CompileError::at("asynchronous comprehension outside of an asynchronous function", e.pos));
                }
                self.st.scopes[save].coro = true;
            }
        }
        Ok(())
    }

    fn expr(&mut self, e: &Expr) -> CResult<()> {
        self.pos = e.pos;
        match &e.kind {
            ExprKind::Name(n) => {
                let n = self.mg(n);
                if &*n == "super" {
                    self.sc().used.insert(Rc::from("__class__"));
                }
                self.sc().used.insert(n);
            }
            ExprKind::Const(_) => {}
            ExprKind::JoinedStr(parts) => {
                for p in parts {
                    self.expr(p)?;
                }
            }
            ExprKind::FormattedValue { value, spec, .. } => {
                self.expr(value)?;
                if let Some(s) = spec {
                    self.expr(s)?;
                }
            }
            ExprKind::BoolAnd(values) | ExprKind::BoolOr(values) => {
                for v in values {
                    self.expr(v)?;
                }
            }
            ExprKind::Bin(_, a, b) | ExprKind::Subscript(a, b) => {
                self.expr(a)?;
                self.expr(b)?;
            }
            ExprKind::Await(a) => {
                let sc = &self.st.scopes[self.cur];
                if !(self.top_await && sc.kind == ScopeKind::Module) {
                    if sc.kind != ScopeKind::Function {
                        return Err(CompileError::at("'await' outside function", self.pos));
                    }
                    if !sc.is_async && !sc.is_comp {
                        return Err(CompileError::at("'await' outside async function", self.pos));
                    }
                }
                self.sc().coro = true;
                self.expr(a)?;
            }
            ExprKind::Unary(_, a) | ExprKind::Starred(a) | ExprKind::Attr(a, _) => self.expr(a)?,
            ExprKind::Compare(a, rest) => {
                self.expr(a)?;
                for (_, b) in rest {
                    self.expr(b)?;
                }
            }
            ExprKind::IfExp(c, a, b) => {
                self.expr(c)?;
                self.expr(a)?;
                self.expr(b)?;
            }
            ExprKind::Call { func, args } => {
                self.expr(func)?;
                for a in args {
                    match a {
                        Arg::Pos(e) | Arg::Star(e) | Arg::Kw(_, e, _) | Arg::StarStar(e, _) => self.expr(e)?,
                    }
                }
            }
            ExprKind::Slice(a, b, c) => {
                for x in [a, b, c].into_iter().flatten() {
                    self.expr(x)?;
                }
            }
            ExprKind::List(items) | ExprKind::Tuple(items) | ExprKind::Set(items) => {
                for i in items {
                    self.expr(i)?;
                }
            }
            ExprKind::Dict(items) => {
                for (k, v) in items {
                    if let Some(k) = k {
                        self.expr(k)?;
                    }
                    self.expr(v)?;
                }
            }
            ExprKind::Lambda(f) => self.func(f)?,
            ExprKind::ListComp(elt, comps) | ExprKind::SetComp(elt, comps) => self.comp(e, &[elt], comps, false)?,
            ExprKind::GenExp(elt, comps) => {
                self.comp(e, &[elt], comps, true)?;
                let id = self.st.by_node[&key(e)];
                self.st.scopes[id].is_gen = true;
            }
            ExprKind::DictComp(k, v, comps) => self.comp(e, &[k, v], comps, false)?,
            ExprKind::NamedExpr(n, v) => {
                self.expr(v)?;
                if self.sc().is_comp && self.sc().assigned.contains(n) {
                    return Err(CompileError::at(format!("assignment expression cannot rebind comprehension iteration variable '{}'", n), e.pos));
                }
                // in a comprehension the name binds in the enclosing function
                let mut s = self.cur;
                while self.st.scopes[s].is_comp {
                    let n2 = n.clone();
                    self.st.scopes[s].nonlocals.insert(n2);
                    s = self.st.scopes[s].parent.unwrap();
                }
                if s != self.cur {
                    if self.st.scopes[s].kind == ScopeKind::Class {
                        return Err(CompileError::at("assignment expression within a comprehension cannot be used in a class body", e.pos));
                    }
                    if self.st.scopes[s].kind == ScopeKind::Module {
                        let mut t = self.cur;
                        while t != s {
                            self.st.scopes[t].nonlocals.remove(n);
                            self.st.scopes[t].globals.insert(n.clone());
                            t = self.st.scopes[t].parent.unwrap();
                        }
                    }
                }
                self.st.scopes[s].assign(n);
            }
            ExprKind::Yield(v) => {
                self.sc().is_gen = true;
                if let Some(v) = v {
                    self.expr(v)?;
                }
            }
            ExprKind::YieldFrom(v) => {
                self.sc().is_gen = true;
                self.expr(v)?;
            }
        }
        Ok(())
    }
}

impl SymTable {
    fn build(body: &[Stmt], top_await: bool) -> CResult<SymTable> {
        let mut st = SymTable { scopes: vec![Scope::new(ScopeKind::Module, None)], by_node: HashMap::new(), future_annotations: false, inlined: HashMap::new() };
        {
            let mut c = Collector { st: &mut st, cur: 0, cls: vec![], top_await, pos: Pos::default() };
            c.stmts(body)?;
        }
        st.resolve()?;
        for i in 0..st.scopes.len() {
            let mut host = i;
            while st.scopes[host].inlined {
                host = st.scopes[host].parent.unwrap();
            }
            if host != i {
                st.inlined.entry(host).or_default().push(i);
            }
        }
        Ok(st)
    }

    /// marks cells in the scope that binds `n` and frees on the way there
    fn resolve_free(&mut self, from: usize, n: &Name) -> bool {
        let mut path = vec![from];
        let mut p = self.scopes[from].parent;
        while let Some(pi) = p {
            let s = &self.scopes[pi];
            match s.kind {
                ScopeKind::Module => return false,
                ScopeKind::Class => {
                    // never through a class body from code inlined into it
                    if &**n == "__class__" && path.iter().all(|&q| self.scopes[q].inlined) {
                        return false;
                    }
                    if &**n == "__class__" {
                        self.scopes[pi].cells.insert(n.clone());
                        for q in path {
                            self.scopes[q].frees.insert(n.clone());
                        }
                        return true;
                    }
                    // a class body passes the variable on to its methods
                    path.push(pi);
                }
                ScopeKind::Function => {
                    if s.globals.contains(n) {
                        return false;
                    }
                    if s.is_local(n) {
                        // inlined comprehensions use their host's variable directly
                        if path.iter().all(|&q| self.scopes[q].inlined) {
                            return true;
                        }
                        self.scopes[pi].cells.insert(n.clone());
                        for q in path {
                            self.scopes[q].frees.insert(n.clone());
                        }
                        return true;
                    }
                    path.push(pi);
                }
            }
            p = self.scopes[pi].parent;
        }
        false
    }

    fn resolve(&mut self) -> CResult<()> {
        for i in 0..self.scopes.len() {
            let kind = self.scopes[i].kind;
            if kind == ScopeKind::Module {
                continue;
            }
            let mut names: Vec<Name> = self.scopes[i].used.iter().cloned().collect();
            names.extend(self.scopes[i].nonlocals.iter().cloned());
            names.sort();
            names.dedup();
            for n in names {
                let s = &self.scopes[i];
                if s.globals.contains(&n) {
                    continue;
                }
                if kind == ScopeKind::Function && s.is_local(&n) {
                    continue;
                }
                if kind == ScopeKind::Class && s.assigned.contains(&n) && !s.nonlocals.contains(&n) {
                    continue;
                }
                let is_nonlocal = s.nonlocals.contains(&n);
                let found = self.resolve_free(i, &n);
                if is_nonlocal && !found {
                    return Err(CompileError::at(format!("no binding for nonlocal '{}' found", n), self.scopes[i].directives[&n]));
                }
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// code generation

#[derive(Clone)]
enum FBlock<'a> {
    Loop { start: usize, breaks: Vec<usize>, is_for: bool },
    TryExcept,
    TryFinally(&'a [Stmt]),
    Handler,
    /// a finally body run for an exception, which sits on the stack
    FinallyExc,
    /// a finally body run on the way out of a return, whose value sits on the stack
    PendingReturn,
    With,
}

struct Unit<'a> {
    scope: usize,
    kind: ScopeKind,
    ops: Vec<Op>,
    positions: Vec<Pos>,
    consts: Vec<Value>,
    str_consts: HashMap<String, u32>,
    names: Vec<Rc<PyStr>>,
    name_map: HashMap<Name, u32>,
    varnames: Vec<Name>,
    var_map: HashMap<Name, u32>,
    cellvars: Vec<Name>,
    freevars: Vec<Name>,
    cell_map: HashMap<Name, u32>,
    blocks: Vec<FBlock<'a>>,
    /// the source span the instructions emitted now come from
    pos: Pos,
    qualname: String,
    tmp: u32,
    /// where the variables of the comprehensions inlined here live
    hidden: HashMap<usize, Vec<(Name, Slot)>>,
    /// the cellvars before this index are the scope's own, the rest
    /// belong to inlined comprehensions
    own_cells: usize,
    /// the kinds of the comprehensions being inlined now, innermost last
    inl: Vec<u8>,
}

/// a variable of an inlined comprehension: a fast local or a cell of its host
#[derive(Clone, Copy, PartialEq)]
enum Slot {
    Fast(u32),
    Cell(u32),
}

pub struct Compiler<'a> {
    st: SymTable,
    units: Vec<Unit<'a>>,
    filename: Rc<str>,
    /// compile()'s optimize: from 1 asserts go and __debug__ is false, at 2 docstrings go too
    optimize: u8,
}

pub fn compile_module(body: Vec<Stmt>, filename: &str) -> CResult<Rc<Code>> {
    compile_module_opt(body, filename, 0, false)
}

pub fn compile_module_opt(mut body: Vec<Stmt>, filename: &str, optimize: u8, top_await: bool) -> CResult<Rc<Code>> {
    check_future(&body)?;
    crate::lower::lower(&mut body, top_await)?;
    let body = &body[..];
    let st = SymTable::build(body, top_await)?;
    let mut c = Compiler { st, units: vec![], filename: Rc::from(filename), optimize };
    c.push_unit(0, "<module>".into());
    let has_ann = has_annotations(body);
    if has_ann {
        c.emit(Op::SetupAnnotations);
    }
    // a docstring
    if let (Some(Stmt { kind: StmtKind::Expr(Expr { kind: ExprKind::Const(Const::Str(d)), .. }), .. }), true) = (body.first(), optimize < 2) {
        let k = c.const_pystr(d);
        c.emit(Op::LoadConst(k));
        let n = c.name_idx(&Rc::from("__doc__"));
        c.emit(Op::StoreName(n));
    }
    c.stmts(body)?;
    let none = c.const_val(Value::None);
    c.emit(Op::LoadConst(none));
    c.emit(Op::Return);
    let code = c.pop_unit("<module>", 1, None, Value::None);
    Ok(Rc::new(code))
}

/// the names an assignment target binds
fn target_names<'e>(t: &'e Expr, out: &mut Vec<&'e Expr>) {
    match &t.kind {
        ExprKind::Name(_) => out.push(t),
        ExprKind::Tuple(items) | ExprKind::List(items) => items.iter().for_each(|i| target_names(i, out)),
        ExprKind::Starred(inner) => target_names(inner, out),
        _ => {}
    }
}

/// `from __future__` imports: features that exist, before anything but the docstring
fn check_future(body: &[Stmt]) -> CResult<()> {
    const FEATURES: [&str; 10] = [
        "nested_scopes",
        "generators",
        "division",
        "absolute_import",
        "with_statement",
        "print_function",
        "unicode_literals",
        "barry_as_FLUFL",
        "generator_stop",
        "annotations",
    ];
    let mut done = false;
    for (i, s) in body.iter().enumerate() {
        match &s.kind {
            StmtKind::ImportFrom { module, names, level: 0 } if &**module == "__future__" => {
                if done {
                    return Err(CompileError::at("from __future__ imports must occur at the beginning of the file", s.pos));
                }
                for a in names {
                    if &*a.name == "braces" {
                        return Err(CompileError::at("not a chance", s.pos));
                    }
                    if !FEATURES.contains(&&*a.name) {
                        return Err(CompileError::at(format!("future feature {} is not defined", a.name), s.pos));
                    }
                }
            }
            StmtKind::Expr(Expr { kind: ExprKind::Const(Const::Str(_)), .. }) if i == 0 => {}
            _ => done = true,
        }
    }
    Ok(())
}

/// whether a module or class body has annotations to store, which
/// compound statements may hold
fn has_annotations(body: &[Stmt]) -> bool {
    body.iter().any(|s| match &s.kind {
        StmtKind::AnnAssign(..) => true,
        StmtKind::If(_, a, b) | StmtKind::While(_, a, b) | StmtKind::For(_, _, a, b, _) => has_annotations(a) || has_annotations(b),
        StmtKind::With(_, a, _) => has_annotations(a),
        StmtKind::Try { body, handlers, orelse, finally, .. } => {
            has_annotations(body) || handlers.iter().any(|h| has_annotations(&h.body)) || has_annotations(orelse) || has_annotations(finally)
        }
        StmtKind::Match(_, cases) => cases.iter().any(|c| has_annotations(&c.body)),
        _ => false,
    })
}

/// compiles an expression for eval()
pub fn compile_eval(e: &Expr, filename: &str) -> CResult<Rc<Code>> {
    compile_eval_opt(e, filename, 0, false)
}

/// compile_eval with compile()'s optimize and PyCF_ALLOW_TOP_LEVEL_AWAIT
pub fn compile_eval_opt(e: &Expr, filename: &str, optimize: u8, top_await: bool) -> CResult<Rc<Code>> {
    let body = vec![Stmt { pos: e.pos, kind: StmtKind::Return(Some(e.clone())) }];
    let st = SymTable::build(&body, top_await)?;
    let mut c = Compiler { st, units: vec![], filename: Rc::from(filename), optimize };
    c.push_unit(0, "<module>".into());
    // the symbol table is keyed by node address: compile the copy it saw
    let e = match &body[0].kind {
        StmtKind::Return(Some(e)) => e,
        _ => unreachable!(),
    };
    c.expr(e)?;
    c.emit(Op::Return);
    let code = c.pop_unit("<module>", 1, None, Value::None);
    Ok(Rc::new(code))
}

impl<'a> Compiler<'a> {
    fn u(&mut self) -> &mut Unit<'a> {
        self.units.last_mut().unwrap()
    }

    fn push_unit(&mut self, scope: usize, qualname: String) {
        let s = &self.st.scopes[scope];
        let mut varnames = Vec::new();
        let mut var_map = HashMap::new();
        let mut cellvars: Vec<Name> = s.cells.iter().cloned().collect();
        cellvars.sort();
        let mut freevars: Vec<Name> = s.frees.iter().cloned().collect();
        freevars.sort();
        if s.kind == ScopeKind::Function {
            for p in &s.params {
                var_map.insert(p.clone(), varnames.len() as u32);
                varnames.push(p.clone());
            }
            for n in &s.order {
                if s.is_local(n) && !var_map.contains_key(n) {
                    var_map.insert(n.clone(), varnames.len() as u32);
                    varnames.push(n.clone());
                }
            }
        }
        let mut cell_map = HashMap::new();
        for (i, n) in cellvars.iter().enumerate() {
            cell_map.insert(n.clone(), i as u32);
        }
        // The variables of the comprehensions inlined here: a fast local is
        // the scope's own slot of the name, or one after them, which the
        // comprehension saves and restores around its run. One that something
        // captures is a cell of its own, which comprehensions that never run
        // at the same time share.
        let own_cells = cellvars.len();
        let mut hidden: HashMap<usize, Vec<(Name, Slot)>> = HashMap::new();
        let scopes = &self.st.scopes;
        let encloses = |a: usize, mut b: usize| {
            while let Some(p) = scopes[b].parent.filter(|_| scopes[b].inlined) {
                if p == a {
                    return true;
                }
                b = p;
            }
            false
        };
        let mut taken: Vec<(usize, Name, Slot)> = vec![];
        for &c in self.st.inlined.get(&scope).into_iter().flatten() {
            let cs = &scopes[c];
            for n in cs.order.iter().filter(|n| cs.is_local(n)) {
                let cell = cs.cells.contains(n);
                let fast = var_map.get(n).filter(|_| !cell && !cell_map.contains_key(n));
                // a slot of the name no comprehension around this one holds
                let in_use = |slot: &Slot| matches!(slot, Slot::Cell(_)) && taken.iter().any(|(e, _, s2)| s2 == slot && encloses(*e, c));
                let free = taken.iter().find(|(_, m, slot)| m == n && matches!(slot, Slot::Cell(_)) == cell && !in_use(slot));
                let slot = match (fast, free) {
                    (Some(&v), _) => Slot::Fast(v),
                    (None, Some((_, _, slot))) => *slot,
                    (None, None) if cell => {
                        cellvars.push(n.clone());
                        Slot::Cell(cellvars.len() as u32 - 1)
                    }
                    (None, None) => {
                        varnames.push(n.clone());
                        Slot::Fast(varnames.len() as u32 - 1)
                    }
                };
                taken.push((c, n.clone(), slot));
                hidden.entry(c).or_default().push((n.clone(), slot));
            }
        }
        for (i, n) in freevars.iter().enumerate() {
            cell_map.insert(n.clone(), (cellvars.len() + i) as u32);
        }
        let kind = s.kind;
        self.units.push(Unit {
            scope,
            kind,
            ops: vec![],
            positions: vec![],
            consts: vec![],
            str_consts: HashMap::new(),
            names: vec![],
            name_map: HashMap::new(),
            varnames,
            var_map,
            cellvars,
            freevars,
            cell_map,
            blocks: vec![],
            pos: Pos { line: 1, col: 0, end_line: 1, end_col: 0 },
            qualname,
            tmp: 0,
            hidden,
            own_cells,
            inl: vec![],
        });
    }

    fn pop_unit(&mut self, name: &str, firstline: u32, params: Option<&Params>, doc: Value) -> Code {
        let u = self.units.pop().unwrap();
        let s = &self.st.scopes[u.scope];
        let mut code = Code {
            name: Rc::new(PyStr::new(name.to_string())),
            qualname: Rc::new(PyStr::new(u.qualname.clone())),
            filename: self.filename.clone(),
            firstline,
            argcount: 0,
            posonly: 0,
            kwonly: 0,
            varargs: false,
            varkw: false,
            generator: s.is_gen,
            // a module awaiting at top level, or an async comprehension (with
            // is_gen, an async generator expression)
            coroutine: s.coro && (s.is_comp || s.kind == ScopeKind::Module),
            iterable_coroutine: false,
            is_class: u.kind == ScopeKind::Class,
            cell_from_arg: u.cellvars.iter().enumerate().map(|(i, c)| if i < u.own_cells { u.var_map.get(c).copied() } else { None }).collect(),
            varnames: u.varnames,
            cellvars: u.cellvars,
            freevars: u.freevars,
            ops: u.ops,
            positions: u.positions,
            consts: u.consts,
            attr_hints: u.names.iter().map(|_| std::cell::Cell::default()).collect(),
            names: u.names,
            doc,
            simple: false,
        };
        if let Some(p) = params {
            code.argcount = p.args.len() as u32;
            code.posonly = p.posonly as u32;
            code.kwonly = p.kwonly.len() as u32;
            code.varargs = p.vararg.is_some();
            code.varkw = p.kwarg.is_some();
        }
        if s.is_comp {
            code.argcount = 1;
        }
        code.simple = !code.varargs && !code.varkw && code.kwonly == 0 && code.cellvars.is_empty() && !code.generator && !code.coroutine;
        code
    }

    fn emit(&mut self, op: Op) -> usize {
        let u = self.u();
        u.ops.push(op);
        u.positions.push(u.pos);
        u.ops.len() - 1
    }

    /// the line instructions come from now
    fn line(&self) -> u32 {
        self.units.last().map_or(0, |u| u.pos.line)
    }

    /// moves the current position to `pos`, returning the one it replaces
    fn set_pos(&mut self, pos: Pos) -> Pos {
        std::mem::replace(&mut self.u().pos, pos)
    }

    fn here(&mut self) -> usize {
        self.u().ops.len()
    }

    fn patch(&mut self, at: usize, target: usize) {
        let t = target as u32;
        let op = &mut self.u().ops[at];
        *op = match *op {
            Op::Jump(_) => Op::Jump(t),
            Op::PopJumpIfFalse(_) => Op::PopJumpIfFalse(t),
            Op::PopJumpIfTrue(_) => Op::PopJumpIfTrue(t),
            Op::JumpIfFalseOrPop(_) => Op::JumpIfFalseOrPop(t),
            Op::JumpIfTrueOrPop(_) => Op::JumpIfTrueOrPop(t),
            Op::PopJumpIfNotNone(_) => Op::PopJumpIfNotNone(t),
            Op::ForIter(_) => Op::ForIter(t),
            Op::SetupExcept(_) => Op::SetupExcept(t),
            Op::SetupFinally(_) => Op::SetupFinally(t),
            Op::SetupWith(_) => Op::SetupWith(t),
            other => other,
        };
    }

    fn const_val(&mut self, v: Value) -> u32 {
        let u = self.u();
        u.consts.push(v);
        (u.consts.len() - 1) as u32
    }

    fn const_str(&mut self, s: &str) -> u32 {
        if let Some(&i) = self.u().str_consts.get(s) {
            return i;
        }
        // identifier-like constants are interned, as CPython's are
        let v = if !s.is_empty() && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_') { Value::Str(crate::value::intern_str(&Rc::new(PyStr::new(s.to_string())))) } else { Value::str(s) };
        let i = self.const_val(v);
        self.u().str_consts.insert(s.to_string(), i);
        i
    }

    /// a literal's constant; one with lone surrogates is its own, as the
    /// table of shared ones goes by text
    fn const_pystr(&mut self, s: &Rc<PyStr>) -> u32 {
        if s.has_surrogates() {
            return self.const_val(Value::Str(s.clone()));
        }
        self.const_str(&s.s)
    }

    fn name_idx(&mut self, n: &Name) -> u32 {
        if let Some(&i) = self.u().name_map.get(n) {
            return i;
        }
        let u = self.u();
        u.names.push(crate::value::intern(n));
        let i = (u.names.len() - 1) as u32;
        u.name_map.insert(n.clone(), i);
        i
    }

    /// an error at the node being compiled
    fn err(&self, msg: &str) -> CompileError {
        CompileError::at(msg, self.units.last().map_or(Pos::default(), |u| u.pos))
    }

    // ------------------------------------------------------------- names

    fn load_name(&mut self, n: &Name) {
        if &**n == "__debug__" {
            let k = self.const_val(Value::Bool(self.optimize == 0));
            self.emit(Op::LoadConst(k));
            return;
        }
        let n = &mangle_attr(&self.class_name(), n);
        let scope = self.u().scope;
        let s = &self.st.scopes[scope];
        match s.kind {
            ScopeKind::Function => {
                if s.globals.contains(n) {
                    let i = self.name_idx(n);
                    self.emit(Op::LoadGlobal(i));
                } else if let Some(&c) = self.u().cell_map.get(n) {
                    self.emit(Op::LoadDeref(c));
                } else if let Some(&v) = self.u().var_map.get(n) {
                    self.emit(Op::LoadFast(v));
                } else if self.u().kind == ScopeKind::Module {
                    // a comprehension inlined at module level looks names up as the module does
                    let i = self.name_idx(n);
                    self.emit(Op::LoadName(i));
                } else {
                    let i = self.name_idx(n);
                    self.emit(Op::LoadGlobal(i));
                }
            }
            ScopeKind::Class => {
                if s.globals.contains(n) {
                    let i = self.name_idx(n);
                    self.emit(Op::LoadGlobal(i));
                } else if s.nonlocals.contains(n) {
                    let c = self.u().cell_map[n];
                    self.emit(Op::LoadDeref(c));
                } else if s.frees.contains(n) && !s.assigned.contains(n) {
                    let c = self.u().cell_map[n];
                    self.emit(Op::LoadClassDeref(c));
                } else {
                    let i = self.name_idx(n);
                    self.emit(Op::LoadName(i));
                }
            }
            ScopeKind::Module => {
                let g = s.globals.contains(n);
                let i = self.name_idx(n);
                // a module's `global` matters when exec runs it with separate locals
                self.emit(if g { Op::LoadGlobal(i) } else { Op::LoadName(i) });
            }
        }
    }

    fn store_name(&mut self, n: &Name) {
        let n = &mangle_attr(&self.class_name(), n);
        let scope = self.u().scope;
        let s = &self.st.scopes[scope];
        match s.kind {
            ScopeKind::Function => {
                if s.globals.contains(n) {
                    let i = self.name_idx(n);
                    self.emit(Op::StoreGlobal(i));
                } else if let Some(&c) = self.u().cell_map.get(n) {
                    self.emit(Op::StoreDeref(c));
                } else if let Some(&v) = self.u().var_map.get(n) {
                    self.emit(Op::StoreFast(v));
                } else {
                    let i = self.name_idx(n);
                    self.emit(Op::StoreGlobal(i));
                }
            }
            ScopeKind::Class => {
                if s.globals.contains(n) {
                    let i = self.name_idx(n);
                    self.emit(Op::StoreGlobal(i));
                } else if s.nonlocals.contains(n) {
                    let c = self.u().cell_map[n];
                    self.emit(Op::StoreDeref(c));
                } else {
                    let i = self.name_idx(n);
                    self.emit(Op::StoreName(i));
                }
            }
            ScopeKind::Module => {
                let g = s.globals.contains(n);
                let i = self.name_idx(n);
                self.emit(if g { Op::StoreGlobal(i) } else { Op::StoreName(i) });
            }
        }
    }

    fn del_name(&mut self, n: &Name) {
        let n = &mangle_attr(&self.class_name(), n);
        let scope = self.u().scope;
        let kind = self.st.scopes[scope].kind;
        let is_global = self.st.scopes[scope].globals.contains(n);
        match kind {
            ScopeKind::Function => {
                if is_global {
                    let i = self.name_idx(n);
                    self.emit(Op::DelGlobal(i));
                } else if let Some(&c) = self.u().cell_map.get(n) {
                    self.emit(Op::DelDeref(c));
                } else if let Some(&v) = self.u().var_map.get(n) {
                    self.emit(Op::DelFast(v));
                } else {
                    let i = self.name_idx(n);
                    self.emit(Op::DelGlobal(i));
                }
            }
            _ => {
                let i = self.name_idx(n);
                if is_global {
                    self.emit(Op::DelGlobal(i));
                } else {
                    self.emit(Op::DelName(i));
                }
            }
        }
    }

    // ------------------------------------------------------------- statements

    fn stmts(&mut self, body: &'a [Stmt]) -> CResult<()> {
        for s in body {
            self.stmt(s)?;
        }
        Ok(())
    }

    fn stmt(&mut self, s: &'a Stmt) -> CResult<()> {
        self.u().pos = s.pos;
        match &s.kind {
            StmtKind::Expr(e) => {
                self.expr(e)?;
                self.emit(Op::Pop);
            }
            StmtKind::Assign(targets, value) => {
                self.expr(value)?;
                for (i, t) in targets.iter().enumerate() {
                    if i + 1 < targets.len() {
                        self.emit(Op::Dup);
                    }
                    self.store(t)?;
                }
            }
            StmtKind::AugAssign(t, op, v) => match &t.kind {
                ExprKind::Name(n) => {
                    self.load_name(n);
                    self.expr(v)?;
                    self.emit(Op::Inplace(*op));
                    self.store_name(n);
                }
                ExprKind::Attr(obj, attr) => {
                    self.expr(obj)?;
                    self.emit(Op::Dup);
                    let i = self.name_idx(attr);
                    self.emit(Op::LoadAttr(i));
                    self.expr(v)?;
                    self.emit(Op::Inplace(*op));
                    self.emit(Op::Rot2);
                    self.emit(Op::StoreAttr(i));
                }
                ExprKind::Subscript(obj, idx) => {
                    self.expr(obj)?;
                    self.expr(idx)?;
                    self.emit(Op::Dup2);
                    self.emit(Op::Subscr);
                    self.expr(v)?;
                    self.emit(Op::Inplace(*op));
                    self.emit(Op::Rot3);
                    self.emit(Op::StoreSubscr);
                }
                _ => return Err(self.err("illegal expression for augmented assignment")),
            },
            StmtKind::AnnAssign(t, ann, v, simple) => {
                if let Some(v) = v {
                    self.expr(v)?;
                    self.store(t)?;
                } else {
                    // a bare annotation still evaluates the target's object and index
                    match &t.kind {
                        ExprKind::Attr(obj, _) => {
                            self.expr(obj)?;
                            self.emit(Op::Pop);
                        }
                        ExprKind::Subscript(obj, idx) => {
                            self.expr(obj)?;
                            self.emit(Op::Pop);
                            self.expr(idx)?;
                            self.emit(Op::Pop);
                        }
                        _ => {}
                    }
                }
                let kind = self.u().kind;
                let outer = matches!(kind, ScopeKind::Class | ScopeKind::Module);
                if !*simple && outer && !self.st.future_annotations {
                    // only a bare name's annotation is stored
                    self.expr(ann)?;
                    self.emit(Op::Pop);
                }
                if let (ExprKind::Name(n), true, true) = (&t.kind, outer, *simple) {
                    if self.st.future_annotations {
                        let k = self.const_str(&unparse(ann));
                        self.emit(Op::LoadConst(k));
                    } else {
                        self.expr(ann)?;
                    }
                    let a = self.name_idx(&Rc::from("__annotations__"));
                    self.emit(Op::LoadName(a));
                    let k = self.const_str(&mangle(&self.class_name(), n));
                    self.emit(Op::LoadConst(k));
                    self.emit(Op::StoreSubscr);
                }
            }
            StmtKind::If(cond, body, orelse) => {
                self.jump_if(cond, false)?;
                let j = self.emit(Op::PopJumpIfFalse(0));
                self.stmts(body)?;
                if orelse.is_empty() {
                    let h = self.here();
                    self.patch(j, h);
                } else {
                    let end = self.emit(Op::Jump(0));
                    let h = self.here();
                    self.patch(j, h);
                    self.stmts(orelse)?;
                    let h = self.here();
                    self.patch(end, h);
                }
            }
            StmtKind::While(cond, body, orelse) => {
                let start = self.here();
                self.jump_if(cond, false)?;
                let exit = self.emit(Op::PopJumpIfFalse(0));
                self.u().blocks.push(FBlock::Loop { start, breaks: vec![], is_for: false });
                self.stmts(body)?;
                self.emit(Op::Jump(start as u32));
                let b = self.u().blocks.pop().unwrap();
                let h = self.here();
                self.patch(exit, h);
                self.stmts(orelse)?;
                let end = self.here();
                if let FBlock::Loop { breaks, .. } = b {
                    for br in breaks {
                        self.patch(br, end);
                    }
                }
            }
            StmtKind::For(target, iter, body, orelse, _) => {
                self.expr(iter)?;
                // getting the iterator and each item are where the iterable is
                let saved = self.set_pos(iter.pos);
                self.emit(Op::GetIter);
                let start = self.here();
                let fi = self.emit(Op::ForIter(0));
                self.u().pos = saved;
                self.store(target)?;
                self.u().blocks.push(FBlock::Loop { start, breaks: vec![], is_for: true });
                self.stmts(body)?;
                self.emit(Op::Jump(start as u32));
                let b = self.u().blocks.pop().unwrap();
                let h = self.here();
                self.patch(fi, h);
                self.stmts(orelse)?;
                let end = self.here();
                if let FBlock::Loop { breaks, .. } = b {
                    for br in breaks {
                        self.patch(br, end);
                    }
                }
            }
            StmtKind::Break => {
                self.unwind_to_loop(true)?;
            }
            StmtKind::Continue => {
                self.unwind_to_loop(false)?;
            }
            StmtKind::Pass => {}
            StmtKind::Return(v) => {
                if self.u().kind != ScopeKind::Function {
                    return Err(self.err("'return' outside function"));
                }
                match v {
                    Some(v) => self.expr(v)?,
                    None => {
                        let k = self.const_val(Value::None);
                        self.emit(Op::LoadConst(k));
                    }
                }
                self.unwind_for_return()?;
                self.emit(Op::Return);
            }
            StmtKind::FuncDef(f) => {
                for d in &f.decorators {
                    self.expr(d)?;
                }
                self.make_function(f)?;
                for _ in &f.decorators {
                    self.emit(Op::Call(1));
                }
                self.store_name(&f.name);
            }
            StmtKind::ClassDef(c) => self.class_def(c)?,
            StmtKind::Global(_) | StmtKind::Nonlocal(_) => {}
            StmtKind::Del(targets) => {
                for t in targets {
                    self.del(t)?;
                }
            }
            StmtKind::Raise(e, cause) => {
                let mut n = 0;
                if let Some(e) = e {
                    self.expr(e)?;
                    n = 1;
                    if let Some(c) = cause {
                        self.expr(c)?;
                        n = 2;
                    }
                }
                self.emit(Op::Raise(n));
            }
            StmtKind::Try { body, handlers, orelse, finally, star } => {
                if let Some(h) = handlers.iter().rev().skip(1).find(|h| h.typ.is_none()) {
                    return Err(CompileError::at("default 'except:' must be last", h.pos));
                }
                if finally.is_empty() {
                    self.try_except(body, handlers, orelse, *star)?;
                } else {
                    let setup = self.emit(Op::SetupFinally(0));
                    self.u().blocks.push(FBlock::TryFinally(finally));
                    if handlers.is_empty() {
                        self.stmts(body)?;
                    } else {
                        self.try_except(body, handlers, orelse, *star)?;
                    }
                    self.u().blocks.pop();
                    self.emit(Op::PopBlock);
                    self.stmts(finally)?;
                    let end = self.emit(Op::Jump(0));
                    let h = self.here();
                    self.patch(setup, h);
                    self.u().blocks.push(FBlock::FinallyExc);
                    self.stmts(finally)?;
                    self.u().blocks.pop();
                    self.emit(Op::Reraise);
                    let h = self.here();
                    self.patch(end, h);
                }
            }
            StmtKind::With(items, body, _) => self.with(items, 0, body)?,
            StmtKind::Import(names) => {
                for Alias { name: m, asname: alias, .. } in names {
                    let z = self.const_val(Value::Int(0));
                    self.emit(Op::LoadConst(z));
                    let fl = if alias.is_some() && m.contains('.') { self.const_val(Value::tuple(vec![])) } else { self.const_val(Value::None) };
                    self.emit(Op::LoadConst(fl));
                    let i = self.name_idx(m);
                    self.emit(Op::ImportName(i));
                    match alias {
                        Some(a) => self.store_name(a),
                        None => {
                            let top: Name = Rc::from(m.split('.').next().unwrap());
                            self.store_name(&top);
                        }
                    }
                }
            }
            StmtKind::ImportFrom { module, names, level } => {
                let lv = self.const_val(Value::Int(*level as i64));
                self.emit(Op::LoadConst(lv));
                let fl: Vec<Value> = names.iter().map(|a| Value::str(&a.name)).collect();
                let k = self.const_val(Value::tuple(fl));
                self.emit(Op::LoadConst(k));
                let i = self.name_idx(module);
                self.emit(Op::ImportName(i));
                if names.len() == 1 && &*names[0].name == "*" {
                    self.emit(Op::ImportStar);
                } else {
                    for Alias { name: n, asname: alias, .. } in names {
                        let i = self.name_idx(n);
                        self.emit(Op::ImportFrom(i));
                        self.store_name(alias.as_ref().unwrap_or(n));
                    }
                    self.emit(Op::Pop);
                }
            }
            StmtKind::Assert(..) if self.optimize > 0 => {}
            StmtKind::Assert(cond, msg) => {
                if matches!(&cond.kind, ExprKind::Tuple(items) if !items.is_empty()) {
                    crate::lexer::push_warning("assertion is always true, perhaps remove parentheses?".into(), self.line(), 0);
                }
                self.jump_if(cond, true)?;
                let j = self.emit(Op::PopJumpIfTrue(0));
                // the error is raised where the condition is
                let saved = self.set_pos(cond.pos);
                self.emit(Op::LoadAssertionError);
                if let Some(m) = msg {
                    self.expr(m)?;
                    self.emit(Op::Call(1));
                }
                self.emit(Op::Raise(1));
                self.u().pos = saved;
                let h = self.here();
                self.patch(j, h);
            }
            StmtKind::Match(subject, cases) => self.match_stmt(subject, cases)?,
            StmtKind::TypeAlias(..) => unreachable!("lowered"),
        }
        Ok(())
    }

    fn class_name(&self) -> String {
        // the innermost enclosing class, for private name mangling
        for u in self.units.iter().rev() {
            if u.kind == ScopeKind::Class {
                return u.qualname.rsplit('.').next().unwrap_or("").to_string();
            }
        }
        String::new()
    }

    fn jump_if(&mut self, cond: &'a Expr, _want: bool) -> CResult<()> {
        self.expr(cond)
    }

    fn unwind_to_loop(&mut self, is_break: bool) -> CResult<()> {
        let n = self.u().blocks.len();
        let mut i = n;
        loop {
            if i == 0 {
                return Err(self.err(if is_break { "'break' outside loop" } else { "'continue' not properly in loop" }));
            }
            i -= 1;
            let b = self.u().blocks[i].clone();
            match b {
                FBlock::Loop { start, is_for, .. } => {
                    if is_break {
                        if is_for {
                            self.emit(Op::Pop);
                        }
                        let j = self.emit(Op::Jump(0));
                        if let FBlock::Loop { breaks, .. } = &mut self.u().blocks[i] {
                            breaks.push(j);
                        }
                    } else {
                        self.emit(Op::Jump(start as u32));
                    }
                    return Ok(());
                }
                FBlock::TryExcept => {
                    self.emit(Op::PopBlock);
                }
                FBlock::TryFinally(body) => {
                    self.emit(Op::PopBlock);
                    // compile the finally body without this block in view
                    let saved: Vec<FBlock> = self.u().blocks.drain(i..).collect();
                    self.stmts(body)?;
                    let u = self.u();
                    u.blocks.extend(saved);
                }
                FBlock::Handler => {
                    self.emit(Op::PopExcept);
                }
                FBlock::FinallyExc => {
                    self.emit(Op::PopExcept);
                    self.emit(Op::Pop);
                }
                FBlock::PendingReturn => {
                    self.emit(Op::Pop);
                }
                FBlock::With => {
                    self.emit(Op::PopBlock);
                    self.emit(Op::WithExit(0));
                }
            }
        }
    }

    fn unwind_for_return(&mut self) -> CResult<()> {
        let n = self.u().blocks.len();
        let mut i = n;
        while i > 0 {
            i -= 1;
            let b = self.u().blocks[i].clone();
            match b {
                // the frame's stack goes with it: loop iterators need no pop
                FBlock::Loop { .. } => {}
                FBlock::TryExcept => {
                    self.emit(Op::PopBlock);
                }
                FBlock::TryFinally(body) => {
                    self.emit(Op::PopBlock);
                    let saved: Vec<FBlock> = self.u().blocks.drain(i..).collect();
                    self.u().blocks.push(FBlock::PendingReturn);
                    self.stmts(body)?;
                    self.u().blocks.pop();
                    self.u().blocks.extend(saved);
                }
                FBlock::Handler | FBlock::FinallyExc => {
                    self.emit(Op::PopExcept);
                }
                FBlock::PendingReturn => {}
                FBlock::With => {
                    self.emit(Op::PopBlock);
                    self.emit(Op::WithExit(1));
                }
            }
        }
        Ok(())
    }

    fn try_except(&mut self, body: &'a [Stmt], handlers: &'a [Handler], orelse: &'a [Stmt], star: bool) -> CResult<()> {
        let setup = self.emit(Op::SetupExcept(0));
        self.u().blocks.push(FBlock::TryExcept);
        self.stmts(body)?;
        self.u().blocks.pop();
        self.emit(Op::PopBlock);
        self.stmts(orelse)?;
        let mut ends = vec![self.emit(Op::Jump(0))];
        let h = self.here();
        self.patch(setup, h);
        if star {
            return self.except_star(handlers, ends);
        }
        for hd in handlers {
            self.u().pos = hd.pos;
            let mut next = None;
            if let Some(t) = &hd.typ {
                self.emit(Op::Dup);
                self.expr(t)?;
                self.emit(Op::ExcMatch);
                next = Some(self.emit(Op::PopJumpIfFalse(0)));
            }
            match &hd.name {
                Some(n) => self.store_name(n),
                None => {
                    self.emit(Op::Pop);
                }
            }
            self.u().blocks.push(FBlock::Handler);
            self.stmts(&hd.body)?;
            self.u().blocks.pop();
            self.emit(Op::PopExcept);
            if let Some(n) = &hd.name {
                let k = self.const_val(Value::None);
                self.emit(Op::LoadConst(k));
                self.store_name(n);
                self.del_name(n);
            }
            ends.push(self.emit(Op::Jump(0)));
            match next {
                Some(j) => {
                    let h = self.here();
                    self.patch(j, h);
                }
                None => break,
            }
        }
        if handlers.last().map(|h| h.typ.is_some()).unwrap_or(true) {
            let saved = self.u().pos;
            if let Some(hd) = handlers.last() {
                self.u().pos = hd.pos;
            }
            self.emit(Op::Reraise);
            self.u().pos = saved;
        }
        let end = self.here();
        for e in ends {
            self.patch(e, end);
        }
        Ok(())
    }

    /// except*: the group is split by each clause in turn; what the clauses
    /// raise and what nobody matched are raised together at the end
    fn except_star(&mut self, handlers: &'a [Handler], mut ends: Vec<usize>) -> CResult<()> {
        // [exc] -> [orig, rest, raised]
        self.emit(Op::Dup);
        self.emit(Op::BuildList(0));
        for hd in handlers {
            self.u().pos = hd.pos;
            // [orig, rest, raised] -> [orig, raised, rest]
            self.emit(Op::Rot2);
            match &hd.typ {
                Some(t) => self.expr(t)?,
                None => return Err(self.err("expected exception type in except*")),
            }
            // [orig, raised, rest, type] -> [orig, raised, rest2, match]
            self.emit(Op::ExcStarMatch);
            self.emit(Op::Dup);
            let skip = self.emit(Op::PopJumpIfNotNone(0));
            self.emit(Op::Pop);
            let after = self.emit(Op::Jump(0));
            let h = self.here();
            self.patch(skip, h);
            match &hd.name {
                Some(n) => self.store_name(n),
                None => {
                    self.emit(Op::Pop);
                }
            }
            let setup = self.emit(Op::SetupExcept(0));
            self.u().blocks.push(FBlock::TryExcept);
            self.stmts(&hd.body)?;
            self.u().blocks.pop();
            self.emit(Op::PopBlock);
            let ok = self.emit(Op::Jump(0));
            let hh = self.here();
            self.patch(setup, hh);
            // [orig, raised, rest2, exc2]: collect into raised
            self.emit(Op::ExcStarCollect);
            let h2 = self.here();
            self.patch(ok, h2);
            self.patch(after, h2);
            // [orig, raised, rest2] -> [orig, rest2, raised]
            self.emit(Op::Rot2);
        }
        self.emit(Op::ExcStarFinish);
        self.emit(Op::PopExcept);
        ends.push(self.emit(Op::Jump(0)));
        let end = self.here();
        for e in ends {
            self.patch(e, end);
        }
        Ok(())
    }

    fn with(&mut self, items: &'a [(Expr, Option<Expr>)], i: usize, body: &'a [Stmt]) -> CResult<()> {
        if i == items.len() {
            return self.stmts(body);
        }
        let (e, t) = &items[i];
        self.expr(e)?;
        // entering and exiting are where the manager's expression is
        let saved = self.set_pos(e.pos);
        let setup = self.emit(Op::SetupWith(0));
        self.u().pos = saved;
        match t {
            Some(t) => self.store(t)?,
            None => {
                self.emit(Op::Pop);
            }
        }
        self.u().blocks.push(FBlock::With);
        self.with(items, i + 1, body)?;
        self.u().blocks.pop();
        let saved = self.set_pos(e.pos);
        self.emit(Op::PopBlock);
        self.emit(Op::WithExit(0));
        let end = self.emit(Op::Jump(0));
        let h = self.here();
        self.patch(setup, h);
        self.emit(Op::WithExcept);
        self.u().pos = saved;
        let h = self.here();
        self.patch(end, h);
        Ok(())
    }

    fn match_stmt(&mut self, subject: &'a Expr, cases: &'a [MatchCase]) -> CResult<()> {
        self.expr(subject)?;
        let mut ends = vec![];
        for c in cases {
            // capture names in pattern order, value expressions in order
            let mut caps = vec![];
            let mut vals: Vec<&'a Expr> = vec![];
            collect_pattern(&c.pat, &mut caps, &mut vals);
            self.emit(Op::Dup);
            for v in &vals {
                self.expr(v)?;
            }
            self.emit(Op::BuildTuple(vals.len() as u32));
            let pk = self.const_val(Value::Obj(Rc::new(NativeObj {
                class: crate::vm::obj_class(),
                gc: Default::default(),
                kind: std::cell::RefCell::new(ObjKind::MatchPat(Rc::new(c.pat.clone()))),
            })));
            self.emit(Op::MatchPattern(pk));
            self.emit(Op::Dup);
            let k = self.const_val(Value::None);
            self.emit(Op::LoadConst(k));
            self.emit(Op::Compare(CmpOp::Is));
            let skip_none = self.emit(Op::PopJumpIfFalse(0));
            self.emit(Op::Pop);
            let next = self.emit(Op::Jump(0));
            let h = self.here();
            self.patch(skip_none, h);
            self.emit(Op::Unpack(caps.len() as u32));
            for n in &caps {
                self.store_name(n);
            }
            let mut guard_fail = None;
            if let Some(g) = &c.guard {
                self.expr(g)?;
                guard_fail = Some(self.emit(Op::PopJumpIfFalse(0)));
            }
            self.emit(Op::Pop);
            self.stmts(&c.body)?;
            ends.push(self.emit(Op::Jump(0)));
            let h = self.here();
            self.patch(next, h);
            if let Some(g) = guard_fail {
                self.patch(g, h);
            }
        }
        self.emit(Op::Pop);
        let end = self.here();
        for e in ends {
            self.patch(e, end);
        }
        Ok(())
    }

    fn class_def(&mut self, c: &'a Rc<ClassDef>) -> CResult<()> {
        for d in &c.decorators {
            self.expr(d)?;
        }
        self.emit(Op::LoadBuildClass);
        let scope = self.st.by_node[&(Rc::as_ptr(c) as usize)];
        let qual = self.child_qualname(&c.name);
        self.push_unit(scope, qual.clone());
        self.u().pos = line_pos(c.line);
        let mn = self.name_idx(&Rc::from("__name__"));
        self.emit(Op::LoadGlobal(mn));
        let mm = self.name_idx(&Rc::from("__module__"));
        self.emit(Op::StoreName(mm));
        let q = self.const_str(&qual);
        self.emit(Op::LoadConst(q));
        let qn = self.name_idx(&Rc::from("__qualname__"));
        self.emit(Op::StoreName(qn));
        let mut doc = Value::None;
        if let (Some(Stmt { kind: StmtKind::Expr(Expr { kind: ExprKind::Const(Const::Str(d)), .. }), .. }), true) = (c.body.first(), self.optimize < 2) {
            doc = Value::Str(d.clone());
            let k = self.const_pystr(d);
            self.emit(Op::LoadConst(k));
            let dn = self.name_idx(&Rc::from("__doc__"));
            self.emit(Op::StoreName(dn));
        }
        if has_annotations(&c.body) {
            self.emit(Op::SetupAnnotations);
        }
        self.stmts(&c.body)?;
        if let Some(ci) = self.class_cell() {
            self.emit(Op::LoadClosure(ci));
            let cc = self.name_idx(&Rc::from("__classcell__"));
            self.emit(Op::StoreName(cc));
            self.emit(Op::LoadClosure(ci));
            self.emit(Op::Return);
        } else {
            let k = self.const_val(Value::None);
            self.emit(Op::LoadConst(k));
            self.emit(Op::Return);
        }
        let code = self.pop_unit(&c.name, c.line, None, doc);
        self.emit_closure_and_make(code, 0)?;
        let nk = self.const_str(&c.name);
        self.emit(Op::LoadConst(nk));
        self.call_args(&c.bases, 2)?;
        for _ in &c.decorators {
            self.emit(Op::Call(1));
        }
        self.store_name(&c.name);
        Ok(())
    }

    fn annotation(&mut self, a: &'a Expr) -> CResult<()> {
        if self.st.future_annotations {
            let k = self.const_str(&unparse(a));
            self.emit(Op::LoadConst(k));
            Ok(())
        } else {
            self.expr(a)
        }
    }

    fn child_qualname(&self, name: &str) -> String {
        let u = self.units.last().unwrap();
        // the hidden function of a PEP 695 definition does not show
        if let Some(pre) = u.qualname.strip_suffix(&format!("<generic parameters of {}>", name)) {
            return format!("{}{}", pre, name);
        }
        match u.kind {
            ScopeKind::Module => name.to_string(),
            ScopeKind::Class => format!("{}.{}", u.qualname, name),
            ScopeKind::Function => format!("{}.<locals>.{}", u.qualname, name),
        }
    }

    /// the implicit __class__ cell of the class body being compiled
    fn class_cell(&mut self) -> Option<u32> {
        let u = self.u();
        if u.kind != ScopeKind::Class {
            return None;
        }
        u.cellvars.iter().position(|n| &**n == "__class__").map(|i| i as u32)
    }

    /// pushes the function object for `code`, with `flags` already pushed
    fn emit_closure_and_make(&mut self, code: Code, mut flags: u8) -> CResult<()> {
        if !code.freevars.is_empty() {
            for fv in code.freevars.clone() {
                // a method's __class__ is the class's implicit cell, even
                // when the class body declares __class__ nonlocal
                let implicit = if &*fv == "__class__" { self.class_cell() } else { None };
                let ci = match implicit.or_else(|| self.u().cell_map.get(&fv).copied()) {
                    Some(c) => c,
                    None => return Err(self.err(&format!("internal: free variable {} not found", fv))),
                };
                self.emit(Op::LoadClosure(ci));
            }
            self.emit(Op::BuildTuple(code.freevars.len() as u32));
            flags |= 8;
        }
        let k = self.const_val(Value::Code(Rc::new(code)));
        self.emit(Op::MakeFunction(k, flags));
        Ok(())
    }

    fn make_function(&mut self, f: &'a Rc<FuncDef>) -> CResult<()> {
        let mut flags = 0u8;
        let defaults: Vec<&Expr> = f.params.args.iter().filter_map(|a| a.default.as_ref()).collect();
        if !defaults.is_empty() {
            for d in &defaults {
                self.expr(d)?;
            }
            self.emit(Op::BuildTuple(defaults.len() as u32));
            flags |= 1;
        }
        let kwd: Vec<&Param> = f.params.kwonly.iter().filter(|a| a.default.is_some()).collect();
        if !kwd.is_empty() {
            for p in &kwd {
                let k = self.const_str(&mangle(&self.class_name(), &p.name));
                self.emit(Op::LoadConst(k));
                self.expr(p.default.as_ref().unwrap())?;
            }
            self.emit(Op::BuildMap(kwd.len() as u32));
            flags |= 2;
        }
        // annotations are evaluated when the function is defined (3.13)
        let mut anns: Vec<(&Name, &'a Expr)> = vec![];
        for p in f.params.args.iter().chain(f.params.vararg.iter()).chain(f.params.kwonly.iter()).chain(f.params.kwarg.iter()) {
            if let Some(a) = &p.annotation {
                anns.push((&p.name, a));
            }
        }
        if !anns.is_empty() || f.returns.is_some() {
            let n = anns.len() + f.returns.is_some() as usize;
            for (name, a) in anns {
                let k = self.const_str(&mangle(&self.class_name(), name));
                self.emit(Op::LoadConst(k));
                self.annotation(a)?;
            }
            if let Some(r) = &f.returns {
                let k = self.const_str("return");
                self.emit(Op::LoadConst(k));
                self.annotation(r)?;
            }
            self.emit(Op::BuildMap(n as u32));
            flags |= 4;
        }
        let scope = self.st.by_node[&(Rc::as_ptr(f) as usize)];
        let qual = self.child_qualname(&f.name);
        self.push_unit(scope, qual);
        self.u().pos = line_pos(f.line);
        let mut doc = Value::None;
        let mut body: &'a [Stmt] = &f.body;
        if !f.is_lambda {
            if let Some(Stmt { kind: StmtKind::Expr(Expr { kind: ExprKind::Const(Const::Str(d)), .. }), .. }) = f.body.first() {
                if self.optimize < 2 {
                    doc = Value::Str(d.clone());
                }
                body = &f.body[1..];
            }
        }
        self.stmts(body)?;
        if f.is_lambda {
            // the body is a single return
        } else {
            let k = self.const_val(Value::None);
            self.emit(Op::LoadConst(k));
            self.emit(Op::Return);
        }
        let mut code = self.pop_unit(&f.name, f.line, Some(&f.params), doc);
        code.coroutine = f.is_async;
        if f.is_async {
            code.simple = false;
        }
        self.emit_closure_and_make(code, flags)
    }

    fn comprehension(&mut self, e: &'a Expr, comps: &'a [Comp], kind: u8, elt: &'a Expr, val: Option<&'a Expr>) -> CResult<()> {
        let scope = self.st.by_node[&key(e)];
        if self.st.scopes[scope].inlined {
            return self.inline_comprehension(scope, comps, kind, elt, val);
        }
        let name = match kind {
            0 => "<listcomp>",
            1 => "<setcomp>",
            2 => "<dictcomp>",
            _ => "<genexpr>",
        };
        let qual = self.child_qualname(name);
        let iter = &comps[0].iter;
        self.expr(iter)?;
        // the first iterator is got, and the loop runs, where the iterable is
        let pos = self.set_pos(iter.pos);
        if comps[0].is_async {
            self.call_dunder("__aiter__");
        } else {
            self.emit(Op::GetIter);
        }
        let line = pos.line;
        self.push_unit(scope, qual);
        self.u().pos = pos;
        match kind {
            0 => {
                self.emit(Op::BuildList(0));
            }
            1 => {
                self.emit(Op::BuildSet(0));
            }
            2 => {
                self.emit(Op::BuildMap(0));
            }
            _ => {}
        }
        self.comp_loop(comps, 0, kind, elt, val)?;
        if kind == 3 {
            let k = self.const_val(Value::None);
            self.emit(Op::LoadConst(k));
        }
        self.emit(Op::Return);
        let code = self.pop_unit(name, line, None, Value::None);
        let coro = code.coroutine && !code.generator;
        self.emit_closure_and_make(code, 0)?;
        self.emit(Op::Rot2);
        self.emit(Op::Call(1));
        if coro {
            self.emit(Op::GetAwaitable);
            let k = self.const_val(Value::None);
            self.emit(Op::LoadConst(k));
            self.emit(Op::YieldFrom);
        }
        self.u().pos = pos;
        Ok(())
    }

    /// A list, set or dict comprehension run in the frame of its host, as
    /// CPython does since 3.12: its variables are slots of the host that
    /// nothing outside the comprehension sees, unbound again when it ends.
    fn inline_comprehension(&mut self, scope: usize, comps: &'a [Comp], kind: u8, elt: &'a Expr, val: Option<&'a Expr>) -> CResult<()> {
        let iter = &comps[0].iter;
        self.expr(iter)?;
        let pos = self.set_pos(iter.pos);
        if comps[0].is_async {
            self.call_dunder("__aiter__");
        } else {
            self.emit(Op::GetIter);
        }
        let own = self.u().hidden.get(&scope).cloned().unwrap_or_default();
        // the values the fast locals had go under the iterator
        let fasts: Vec<u32> = own.iter().filter_map(|(_, s)| if let Slot::Fast(v) = s { Some(*v) } else { None }).collect();
        for &v in &fasts {
            self.emit(Op::LoadFastAndClear(v));
        }
        if !fasts.is_empty() {
            self.emit(Op::Swap(fasts.len() as u32 + 1));
        }
        let setup = self.emit(Op::SetupExcept(0));
        self.emit(match kind {
            0 => Op::BuildList(0),
            1 => Op::BuildSet(0),
            _ => Op::BuildMap(0),
        });
        self.emit(Op::Rot2);
        // closures made in one run keep their cells
        for (_, slot) in &own {
            if let Slot::Cell(c) = slot {
                self.emit(Op::MakeCell(*c));
            }
        }
        // the comprehension's names over the host's while it is compiled
        let mut saved = vec![];
        {
            let u = self.u();
            for (n, slot) in &own {
                saved.push((n.clone(), u.var_map.remove(n), u.cell_map.remove(n)));
                match slot {
                    Slot::Fast(v) => u.var_map.insert(n.clone(), *v),
                    Slot::Cell(c) => u.cell_map.insert(n.clone(), *c),
                };
            }
        }
        let host = std::mem::replace(&mut self.u().scope, scope);
        self.u().inl.push(kind);
        let r = self.comp_loop(comps, 0, kind, elt, val);
        self.u().inl.pop();
        self.u().scope = host;
        {
            let u = self.u();
            for (n, v, c) in saved.into_iter().rev() {
                u.var_map.remove(&n);
                u.cell_map.remove(&n);
                if let Some(v) = v {
                    u.var_map.insert(n.clone(), v);
                }
                if let Some(c) = c {
                    u.cell_map.insert(n, c);
                }
            }
        }
        r?;
        self.u().pos = pos;
        self.emit(Op::PopBlock);
        self.restore_slots(&own, &fasts);
        let end = self.emit(Op::Jump(0));
        // an exception leaving the comprehension restores them too
        let h = self.here();
        self.patch(setup, h);
        self.emit(Op::Rot2);
        self.emit(Op::Pop);
        self.restore_slots(&own, &fasts);
        self.emit(Op::Reraise);
        let h = self.here();
        self.patch(end, h);
        Ok(())
    }

    /// [saved..., result] -> [result], the saved values back in their slots
    fn restore_slots(&mut self, own: &[(Name, Slot)], fasts: &[u32]) {
        for (_, slot) in own {
            if let Slot::Cell(c) = slot {
                self.emit(Op::MakeCell(*c));
            }
        }
        if !fasts.is_empty() {
            self.emit(Op::Swap(fasts.len() as u32 + 1));
            for &v in fasts.iter().rev() {
                self.emit(Op::StoreFast(v));
            }
        }
    }

    /// [obj] -> [obj.<name>()]
    fn call_dunder(&mut self, name: &str) {
        let i = self.name_idx(&Rc::from(name));
        self.emit(Op::LoadAttr(i));
        self.emit(Op::Call(0));
    }

    fn comp_loop(&mut self, comps: &'a [Comp], i: usize, kind: u8, elt: &'a Expr, val: Option<&'a Expr>) -> CResult<()> {
        let c = &comps[i];
        if i == 0 {
            // an inlined comprehension finds the iterator on the stack already
            let scope = self.u().scope;
            if !self.st.scopes[scope].inlined {
                self.emit(Op::LoadFast(0));
            }
        } else {
            self.expr(&c.iter)?;
            if c.is_async {
                self.call_dunder("__aiter__");
            } else {
                self.emit(Op::GetIter);
            }
        }
        let saved = self.set_pos(c.iter.pos);
        let start = self.here();
        // `async for`: await __anext__() until StopAsyncIteration; the
        // handler leaves [iterator, exception] on the stack
        let (fi, setup) = if c.is_async {
            let setup = self.emit(Op::SetupExcept(0));
            self.emit(Op::Dup);
            self.call_dunder("__anext__");
            self.emit(Op::GetAwaitable);
            let k = self.const_val(Value::None);
            self.emit(Op::LoadConst(k));
            self.emit(Op::YieldFrom);
            self.emit(Op::PopBlock);
            (None, Some(setup))
        } else {
            (Some(self.emit(Op::ForIter(0))), None)
        };
        self.u().pos = saved;
        self.store(&c.target)?;
        let mut skips = vec![];
        for f in &c.ifs {
            self.expr(f)?;
            skips.push(self.emit(Op::PopJumpIfFalse(0)));
        }
        if i + 1 < comps.len() {
            self.comp_loop(comps, i + 1, kind, elt, val)?;
        } else {
            // depth: the result sits under one iterator per loop level
            let depth = (i + 2) as u32;
            match kind {
                0 => {
                    self.expr(elt)?;
                    self.emit(Op::ListAppend(depth));
                }
                1 => {
                    self.expr(elt)?;
                    self.emit(Op::SetAdd(depth));
                }
                2 => {
                    self.expr(elt)?;
                    self.expr(val.unwrap())?;
                    self.emit(Op::MapAdd(depth));
                }
                _ => {
                    self.expr(elt)?;
                    self.emit(Op::Yield);
                    self.emit(Op::Pop);
                }
            }
        }
        for s in skips {
            self.patch(s, start);
        }
        self.emit(Op::Jump(start as u32));
        let h = self.here();
        if let Some(fi) = fi {
            self.patch(fi, h);
        }
        if let Some(setup) = setup {
            self.patch(setup, h);
            self.emit(Op::Dup);
            let n = self.name_idx(&Rc::from("StopAsyncIteration"));
            self.emit(Op::LoadGlobal(n));
            self.emit(Op::ExcMatch);
            let other = self.emit(Op::PopJumpIfFalse(0));
            self.emit(Op::Pop);
            self.emit(Op::PopExcept);
            self.emit(Op::Pop);
            let end = self.emit(Op::Jump(0));
            let h = self.here();
            self.patch(other, h);
            self.emit(Op::Reraise);
            let h = self.here();
            self.patch(end, h);
        }
        Ok(())
    }

    // ------------------------------------------------------------- targets

    fn store(&mut self, t: &'a Expr) -> CResult<()> {
        match &t.kind {
            ExprKind::Name(n) => self.store_name(n),
            ExprKind::Attr(obj, attr) => {
                self.expr(obj)?;
                let i = self.name_idx(&mangle_attr(&self.class_name(), attr));
                self.emit(Op::StoreAttr(i));
            }
            ExprKind::Subscript(obj, idx) => {
                self.expr(obj)?;
                self.expr(idx)?;
                self.emit(Op::StoreSubscr);
            }
            ExprKind::Tuple(items) | ExprKind::List(items) => {
                let star = items.iter().position(|e| matches!(e.kind, ExprKind::Starred(_)));
                match star {
                    Some(si) => {
                        if items.iter().filter(|e| matches!(e.kind, ExprKind::Starred(_))).count() > 1 {
                            return Err(self.err("multiple starred expressions in assignment"));
                        }
                        self.emit(Op::UnpackEx(si as u16, (items.len() - si - 1) as u16));
                    }
                    None => {
                        self.emit(Op::Unpack(items.len() as u32));
                    }
                }
                for it in items {
                    match &it.kind {
                        ExprKind::Starred(inner) => self.store(inner)?,
                        _ => self.store(it)?,
                    }
                }
            }
            ExprKind::Starred(_) => return Err(self.err("starred assignment target must be in a list or tuple")),
            _ => return Err(CompileError::at(format!("cannot assign to {}", t.describe()), t.pos)),
        }
        Ok(())
    }

    fn del(&mut self, t: &'a Expr) -> CResult<()> {
        match &t.kind {
            ExprKind::Name(n) => self.del_name(n),
            ExprKind::Attr(obj, attr) => {
                self.expr(obj)?;
                let i = self.name_idx(attr);
                self.emit(Op::DelAttr(i));
            }
            ExprKind::Subscript(obj, idx) => {
                self.expr(obj)?;
                self.expr(idx)?;
                self.emit(Op::DelSubscr);
            }
            ExprKind::Tuple(items) | ExprKind::List(items) => {
                for i in items {
                    self.del(i)?;
                }
            }
            _ => return Err(CompileError::at(format!("cannot delete {}", t.describe()), t.pos)),
        }
        Ok(())
    }

    // ------------------------------------------------------------- expressions

    fn constant(&mut self, c: &Const) -> u32 {
        match c {
            Const::Str(s) => self.const_pystr(s),
            other => {
                let v = match other {
                    Const::None => Value::None,
                    Const::True => Value::Bool(true),
                    Const::False => Value::Bool(false),
                    Const::Ellipsis => Value::Ellipsis,
                    Const::Int(i) => Value::Int(*i),
                    Const::BigInt(s) => Value::from_big(crate::bigint::BigInt::from_radix(s, 10).unwrap()),
                    Const::Float(f) => Value::Float(*f),
                    Const::Imag(f) => Value::Complex(Rc::new((0.0, *f))),
                    Const::Bytes(b) => Value::Bytes(Rc::new(b.to_vec())),
                    Const::Str(_) => unreachable!(),
                };
                self.const_val(v)
            }
        }
    }

    /// a constant, or a negated number, as a constant index
    fn literal(&mut self, e: &Expr) -> Option<u32> {
        match &e.kind {
            ExprKind::Const(c) => Some(self.constant(c)),
            ExprKind::Unary(UnOp::Neg, a) => match &a.kind {
                ExprKind::Const(Const::Int(i)) => Some(self.const_val(Value::Int(-i))),
                ExprKind::Const(Const::Float(f)) => Some(self.const_val(Value::Float(-f))),
                _ => None,
            },
            _ => None,
        }
    }

    /// compiles `e`, its instructions placed at its span
    fn expr(&mut self, e: &'a Expr) -> CResult<()> {
        let saved = self.set_pos(e.pos);
        let r = self.expr_here(e);
        self.u().pos = saved;
        r
    }

    fn expr_here(&mut self, e: &'a Expr) -> CResult<()> {
        if let Some(k) = self.literal(e) {
            self.emit(Op::LoadConst(k));
            return Ok(());
        }
        match &e.kind {
            ExprKind::Name(n) => self.load_name(n),
            ExprKind::Const(_) => unreachable!("a literal"),
            ExprKind::JoinedStr(parts) => {
                // f'' is the constant '', as CPython folds it (gh-99606)
                if parts.is_empty() {
                    let k = self.const_str("");
                    self.emit(Op::LoadConst(k));
                    return Ok(());
                }
                for p in parts {
                    self.expr(p)?;
                }
                if parts.len() != 1 || !matches!(parts[0].kind, ExprKind::Const(_)) {
                    self.emit(Op::BuildString(parts.len() as u32));
                }
            }
            ExprKind::FormattedValue { value, conv, spec } => {
                self.expr(value)?;
                if let Some(s) = spec {
                    self.expr(s)?;
                }
                let c = match conv {
                    's' => 1,
                    'r' => 2,
                    'a' => 3,
                    _ => 0,
                };
                self.emit(Op::FormatValue(c, spec.is_some()));
            }
            ExprKind::Bin(op, a, b) => {
                self.expr(a)?;
                self.expr(b)?;
                self.emit(Op::Binary(*op));
            }
            ExprKind::Unary(op, a) => {
                self.expr(a)?;
                self.emit(Op::Unary(*op));
            }
            ExprKind::BoolAnd(values) | ExprKind::BoolOr(values) => {
                // each operand but the last decides or is dropped
                let and = matches!(e.kind, ExprKind::BoolAnd(_));
                let mut jumps = vec![];
                for (i, v) in values.iter().enumerate() {
                    self.expr(v)?;
                    if i + 1 < values.len() {
                        jumps.push(self.emit(if and { Op::JumpIfFalseOrPop(0) } else { Op::JumpIfTrueOrPop(0) }));
                    }
                }
                let h = self.here();
                for j in jumps {
                    self.patch(j, h);
                }
            }
            ExprKind::Compare(first, rest) => {
                self.expr(first)?;
                if rest.len() == 1 {
                    self.expr(&rest[0].1)?;
                    self.emit(Op::Compare(rest[0].0));
                } else {
                    let mut cleanups = vec![];
                    for (i, (op, rhs)) in rest.iter().enumerate() {
                        self.expr(rhs)?;
                        if i + 1 < rest.len() {
                            self.emit(Op::Dup);
                            self.emit(Op::Rot3);
                            self.emit(Op::Compare(*op));
                            cleanups.push(self.emit(Op::JumpIfFalseOrPop(0)));
                        } else {
                            self.emit(Op::Compare(*op));
                        }
                    }
                    let end = self.emit(Op::Jump(0));
                    let c = self.here();
                    for j in cleanups {
                        self.patch(j, c);
                    }
                    self.emit(Op::Rot2);
                    self.emit(Op::Pop);
                    let h = self.here();
                    self.patch(end, h);
                }
            }
            ExprKind::IfExp(c, a, b) => {
                self.expr(c)?;
                let j = self.emit(Op::PopJumpIfFalse(0));
                self.expr(a)?;
                let end = self.emit(Op::Jump(0));
                let h = self.here();
                self.patch(j, h);
                self.expr(b)?;
                let h = self.here();
                self.patch(end, h);
            }
            ExprKind::Call { func, args } => {
                let simple = args.iter().all(|a| matches!(a, Arg::Pos(_)));
                if let (ExprKind::Attr(obj, attr), true) = (&func.kind, simple) {
                    self.expr(obj)?;
                    let i = self.name_idx(&mangle_attr(&self.class_name(), attr));
                    self.emit(Op::LoadMethod(i));
                    for a in args {
                        if let Arg::Pos(e) = a {
                            self.expr(e)?;
                        }
                    }
                    self.emit(Op::CallMethod(args.len() as u32));
                } else {
                    self.expr(func)?;
                    self.call_args(args, 0)?;
                }
            }
            ExprKind::Attr(obj, attr) => {
                self.expr(obj)?;
                let i = self.name_idx(&mangle_attr(&self.class_name(), attr));
                self.emit(Op::LoadAttr(i));
            }
            ExprKind::Subscript(obj, idx) => {
                self.expr(obj)?;
                self.expr(idx)?;
                self.emit(Op::Subscr);
            }
            ExprKind::Slice(a, b, c) => {
                let mut n = 2;
                for x in [a, b] {
                    match x {
                        Some(x) => self.expr(x)?,
                        None => {
                            let k = self.const_val(Value::None);
                            self.emit(Op::LoadConst(k));
                        }
                    }
                }
                if let Some(c) = c {
                    self.expr(c)?;
                    n = 3;
                }
                self.emit(Op::BuildSlice(n));
            }
            ExprKind::List(items) => {
                if items.iter().any(|e| matches!(e.kind, ExprKind::Starred(_))) {
                    self.starred_list(items)?;
                } else {
                    for i in items {
                        self.expr(i)?;
                    }
                    self.emit(Op::BuildList(items.len() as u32));
                }
            }
            ExprKind::Tuple(items) => {
                if items.iter().any(|e| matches!(e.kind, ExprKind::Starred(_))) {
                    self.starred_list(items)?;
                    self.emit(Op::ListToTuple);
                } else if let Some(vals) = self.literals(items) {
                    let k = self.const_val(Value::tuple(vals));
                    self.emit(Op::LoadConst(k));
                } else {
                    for i in items {
                        self.expr(i)?;
                    }
                    self.emit(Op::BuildTuple(items.len() as u32));
                }
            }
            ExprKind::Set(items) => {
                if items.iter().any(|e| matches!(e.kind, ExprKind::Starred(_))) {
                    self.emit(Op::BuildSet(0));
                    for i in items {
                        match &i.kind {
                            ExprKind::Starred(inner) => {
                                self.expr(inner)?;
                                self.emit(Op::SetUpdate(1));
                            }
                            _ => {
                                self.expr(i)?;
                                self.emit(Op::SetAdd(1));
                            }
                        }
                    }
                } else {
                    for i in items {
                        self.expr(i)?;
                    }
                    self.emit(Op::BuildSet(items.len() as u32));
                }
            }
            ExprKind::Dict(items) => {
                if items.iter().any(|(k, _)| k.is_none()) {
                    self.emit(Op::BuildMap(0));
                    for (k, v) in items {
                        match k {
                            Some(k) => {
                                self.expr(k)?;
                                self.expr(v)?;
                                self.emit(Op::MapAdd(1));
                            }
                            None => {
                                self.expr(v)?;
                                self.emit(Op::DictUpdate(1));
                            }
                        }
                    }
                } else {
                    for (k, v) in items {
                        self.expr(k.as_ref().unwrap())?;
                        self.expr(v)?;
                    }
                    self.emit(Op::BuildMap(items.len() as u32));
                }
            }
            ExprKind::Starred(_) => return Err(self.err("can't use starred expression here")),
            ExprKind::Lambda(f) => self.make_function(f)?,
            ExprKind::ListComp(elt, comps) => self.comprehension(e, comps, 0, elt, None)?,
            ExprKind::SetComp(elt, comps) => self.comprehension(e, comps, 1, elt, None)?,
            ExprKind::DictComp(k, v, comps) => self.comprehension(e, comps, 2, k, Some(v))?,
            ExprKind::GenExp(elt, comps) => self.comprehension(e, comps, 3, elt, None)?,
            ExprKind::NamedExpr(n, v) => {
                self.expr(v)?;
                self.emit(Op::Dup);
                self.store_name(n);
            }
            ExprKind::Yield(v) => {
                self.check_yield()?;
                match v {
                    Some(v) => self.expr(v)?,
                    None => {
                        let k = self.const_val(Value::None);
                        self.emit(Op::LoadConst(k));
                    }
                }
                self.emit(Op::Yield);
            }
            ExprKind::YieldFrom(v) => {
                self.check_yield()?;
                self.expr(v)?;
                self.emit(Op::GetIter);
                let k = self.const_val(Value::None);
                self.emit(Op::LoadConst(k));
                self.emit(Op::YieldFrom);
            }
            ExprKind::Await(v) => {
                self.expr(v)?;
                self.emit(Op::GetAwaitable);
                let k = self.const_val(Value::None);
                self.emit(Op::LoadConst(k));
                self.emit(Op::YieldFrom);
            }
        }
        Ok(())
    }

    /// the values of a tuple display made only of literals
    fn literals(&mut self, items: &[Expr]) -> Option<Vec<Value>> {
        if items.is_empty() || !items.iter().all(|e| matches!(&e.kind, ExprKind::Const(_)) || matches!(&e.kind, ExprKind::Unary(UnOp::Neg, a) if matches!(a.kind, ExprKind::Const(Const::Int(_) | Const::Float(_))))) {
            return None;
        }
        Some(items.iter().map(|e| {
            let k = self.literal(e).unwrap();
            self.u().consts[k as usize].clone()
        }).collect())
    }

    /// where a yield may not be
    fn check_yield(&mut self) -> CResult<()> {
        const COMPS: [(&str, &str); 4] =
            [("<listcomp>", "list comprehension"), ("<setcomp>", "set comprehension"), ("<dictcomp>", "dict comprehension"), ("<genexpr>", "generator expression")];
        let u = self.u();
        let inlined = u.inl.last().map(|&k| COMPS[k as usize].1);
        if let Some(what) = inlined.or_else(|| COMPS.iter().find(|(s, _)| u.qualname.ends_with(s)).map(|(_, w)| *w)) {
            return Err(self.err(&format!("'yield' inside {}", what)));
        }
        if u.kind != ScopeKind::Function {
            return Err(self.err("'yield' outside function"));
        }
        Ok(())
    }

    fn starred_list(&mut self, items: &'a [Expr]) -> CResult<()> {
        self.emit(Op::BuildList(0));
        for i in items {
            match &i.kind {
                ExprKind::Starred(inner) => {
                    self.expr(inner)?;
                    self.emit(Op::ListExtend(1));
                }
                _ => {
                    self.expr(i)?;
                    self.emit(Op::ListAppend(1));
                }
            }
        }
        Ok(())
    }

    /// the arguments of a call whose callable (and `extra` leading positional
    /// arguments) are already pushed
    fn call_args(&mut self, args: &'a [Arg], extra: u32) -> CResult<()> {
        let has_star = args.iter().any(|a| matches!(a, Arg::Star(_) | Arg::StarStar(..)));
        if !has_star {
            let mut npos = extra;
            let mut kwnames = vec![];
            for a in args {
                match a {
                    Arg::Pos(e) => {
                        if !kwnames.is_empty() {
                            return Err(self.err("positional argument follows keyword argument"));
                        }
                        self.expr(e)?;
                        npos += 1;
                    }
                    Arg::Kw(n, e, _) => {
                        if kwnames.iter().any(|k: &Value| k.as_str() == Some(n)) {
                            return Err(self.err(&format!("keyword argument repeated: {}", n)));
                        }
                        self.expr(e)?;
                        kwnames.push(Value::str(&mangle(&self.class_name(), n)));
                    }
                    _ => unreachable!(),
                }
            }
            if kwnames.is_empty() {
                self.emit(Op::Call(npos));
            } else {
                let n = npos + kwnames.len() as u32;
                let k = self.const_val(Value::tuple(kwnames));
                self.emit(Op::LoadConst(k));
                self.emit(Op::CallKw(n));
            }
            return Ok(());
        }
        // general form: positional list -> tuple, keywords -> dict
        self.emit(Op::BuildList(extra));
        if extra > 0 {
            // the extra values were pushed before the list: rotate them in
        }
        let mut have_kw = false;
        for a in args {
            match a {
                Arg::Pos(e) => {
                    self.expr(e)?;
                    self.emit(Op::ListAppend(1));
                }
                Arg::Star(e) => {
                    self.expr(unstarred(e))?;
                    self.emit(Op::ListExtend(1));
                }
                _ => {}
            }
        }
        self.emit(Op::ListToTuple);
        for a in args {
            match a {
                Arg::Kw(n, e, _) => {
                    if !have_kw {
                        self.emit(Op::BuildMap(0));
                        have_kw = true;
                    }
                    let k = self.const_str(&mangle(&self.class_name(), n));
                    self.emit(Op::LoadConst(k));
                    self.expr(e)?;
                    self.emit(Op::MapAdd(1));
                }
                Arg::StarStar(e, _) => {
                    if !have_kw {
                        self.emit(Op::BuildMap(0));
                        have_kw = true;
                    }
                    self.expr(e)?;
                    self.emit(Op::DictMerge(1));
                }
                _ => {}
            }
        }
        if !have_kw {
            let k = self.const_val(Value::Undef);
            self.emit(Op::LoadConst(k));
        }
        self.emit(Op::CallEx);
        Ok(())
    }
}

/// the value of `*value`, which is a Starred node
fn unstarred(e: &Expr) -> &Expr {
    match &e.kind {
        ExprKind::Starred(inner) => inner,
        _ => e,
    }
}

/// the position of a whole line, for code that starts there
fn line_pos(line: u32) -> Pos {
    Pos { line, col: 0, end_line: line, end_col: 0 }
}

fn collect_pattern<'a>(p: &'a Pattern, caps: &mut Vec<Name>, vals: &mut Vec<&'a Expr>) {
    match &p.kind {
        PatternKind::Capture(Some(n)) | PatternKind::Star(Some(n)) => caps.push(n.clone()),
        PatternKind::Capture(None) | PatternKind::Star(None) | PatternKind::Singleton(_) => {}
        PatternKind::Value(e) => vals.push(e),
        PatternKind::Sequence(ps) => {
            for q in ps {
                collect_pattern(q, caps, vals);
            }
        }
        PatternKind::Or(ps) => {
            // every alternative binds the same names; the first one decides order
            let mut first_caps = vec![];
            for (i, q) in ps.iter().enumerate() {
                let mut c = vec![];
                collect_pattern(q, &mut c, vals);
                if i == 0 {
                    first_caps = c;
                }
            }
            caps.extend(first_caps);
        }
        PatternKind::Mapping { keys, pats, rest } => {
            for k in keys {
                vals.push(k);
            }
            for q in pats {
                collect_pattern(q, caps, vals);
            }
            if let Some(r) = rest {
                caps.push(r.clone());
            }
        }
        PatternKind::Class { cls, args, kw } => {
            vals.push(cls);
            for q in args {
                collect_pattern(q, caps, vals);
            }
            for (_, q) in kw {
                collect_pattern(q, caps, vals);
            }
        }
        PatternKind::As(q, n) => {
            collect_pattern(q, caps, vals);
            caps.push(n.clone());
        }
    }
}

/// private names (`__x`) inside a class become `_Class__x`
pub fn mangle(class: &str, n: &str) -> String {
    if class.is_empty() || !n.starts_with("__") || n.ends_with("__") || n.contains('.') {
        return n.to_string();
    }
    let c = class.trim_start_matches('_');
    if c.is_empty() {
        return n.to_string();
    }
    format!("_{}{}", c, n)
}

fn mangle_attr(class: &str, n: &Name) -> Name {
    let m = mangle(class, n);
    if m == **n {
        n.clone()
    } else {
        Rc::from(m.as_str())
    }
}

/// source text for an annotation under `from __future__ import annotations`
pub fn unparse(e: &Expr) -> String {
    match &e.kind {
        ExprKind::Name(n) => n.to_string(),
        ExprKind::Attr(o, a) => format!("{}.{}", unparse(o), a),
        ExprKind::Const(Const::Str(s)) => format!("'{}'", s.s),
        ExprKind::Const(Const::None) => "None".into(),
        ExprKind::Const(Const::Int(i)) => i.to_string(),
        ExprKind::Const(Const::Ellipsis) => "...".into(),
        ExprKind::Subscript(o, i) => format!("{}[{}]", unparse(o), unparse(i)),
        ExprKind::Tuple(items) => items.iter().map(unparse).collect::<Vec<_>>().join(", "),
        ExprKind::List(items) => format!("[{}]", items.iter().map(unparse).collect::<Vec<_>>().join(", ")),
        ExprKind::Bin(BinOp::BitOr, a, b) => format!("{} | {}", unparse(a), unparse(b)),
        ExprKind::Call { func, .. } => format!("{}(...)", unparse(func)),
        _ => "...".into(),
    }
}
