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
    used: HashSet<Name>,
    is_gen: bool,
    is_comp: bool,
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
            used: HashSet::new(),
            is_gen: false,
            is_comp: false,
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
}

fn key<T>(r: &T) -> usize {
    r as *const T as usize
}

struct Collector<'a> {
    st: &'a mut SymTable,
    cur: usize,
    /// enclosing class names, for private name mangling
    cls: Vec<String>,
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
        match e {
            Expr::Name(n) => {
                let n = self.mg(n);
                self.sc().assign(&n);
            }
            Expr::Tuple(items) | Expr::List(items) => {
                for i in items {
                    self.target(i)?;
                }
            }
            Expr::Starred(inner) => self.target(inner)?,
            Expr::Attr(obj, _) => self.expr(obj)?,
            Expr::Subscript(obj, idx) => {
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
                if let Expr::Name(n) = t {
                    let n = n.clone();
                    self.sc().used.insert(n.clone());
                    self.sc().assign(&n);
                } else {
                    self.target(t)?;
                }
            }
            StmtKind::AnnAssign(t, ann, v) => {
                if !self.st.future_annotations {
                    self.expr(ann)?;
                }
                if let Some(v) = v {
                    self.expr(v)?;
                    self.target(t)?;
                } else if let Expr::Name(n) = t {
                    // a bare annotation still makes the name local
                    if self.sc().kind == ScopeKind::Function {
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
            StmtKind::For(t, it, a, b) => {
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
                        Arg::Pos(e) | Arg::Star(e) | Arg::Kw(_, e) | Arg::StarStar(e) => self.expr(e)?,
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
            StmtKind::Global(ns) => {
                for n in ns {
                    let n = &self.mg(n);
                    if self.sc().assigned.contains(n) || self.sc().used.contains(n) {
                        return Err(CompileError { msg: format!("name '{}' is used prior to global declaration", n), line: s.line });
                    }
                    self.sc().globals.insert(n.clone());
                }
            }
            StmtKind::Nonlocal(ns) => {
                if self.sc().kind == ScopeKind::Module {
                    return Err(CompileError { msg: "nonlocal declaration not allowed at module level".into(), line: s.line });
                }
                for n in ns {
                    let n = self.mg(n);
                    self.sc().nonlocals.insert(n);
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
            StmtKind::With(items, body) => {
                for (e, t) in items {
                    self.expr(e)?;
                    if let Some(t) = t {
                        self.target(t)?;
                    }
                }
                self.stmts(body)?;
            }
            StmtKind::Import(names) => {
                for (m, alias) in names {
                    let n: Name = match alias {
                        Some(a) => a.clone(),
                        None => Rc::from(m.split('.').next().unwrap()),
                    };
                    self.sc().assign(&n);
                }
            }
            StmtKind::ImportFrom { module, names, .. } => {
                if &**module == "__future__" && names.iter().any(|(n, _)| &**n == "annotations") {
                    self.st.future_annotations = true;
                }
                for (n, alias) in names {
                    if &**n == "*" {
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
        match p {
            Pattern::Capture(Some(n)) | Pattern::Star(Some(n)) => {
                let n = n.clone();
                self.sc().assign(&n);
            }
            Pattern::Capture(None) | Pattern::Star(None) | Pattern::Singleton(_) => {}
            Pattern::Value(e) => self.expr(e)?,
            Pattern::Sequence(ps) | Pattern::Or(ps) => {
                for q in ps {
                    self.pattern(q)?;
                }
            }
            Pattern::Mapping { keys, pats, rest } => {
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
            Pattern::Class { cls, args, kw } => {
                self.expr(cls)?;
                for q in args {
                    self.pattern(q)?;
                }
                for (_, q) in kw {
                    self.pattern(q)?;
                }
            }
            Pattern::As(q, n) => {
                self.pattern(q)?;
                let n = n.clone();
                self.sc().assign(&n);
            }
        }
        Ok(())
    }

    fn comp(&mut self, node: usize, elts: &[&Expr], comps: &[Comp]) -> CResult<()> {
        // the first iterable is evaluated in the enclosing scope
        self.expr(&comps[0].iter)?;
        let id = self.new_scope(ScopeKind::Function, node);
        let save = self.cur;
        self.cur = id;
        self.sc().is_comp = true;
        self.sc().params = vec![Rc::from(".0")];
        for (i, c) in comps.iter().enumerate() {
            if i > 0 {
                self.expr(&c.iter)?;
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
        Ok(())
    }

    fn expr(&mut self, e: &Expr) -> CResult<()> {
        match e {
            Expr::Name(n) => {
                let n = self.mg(n);
                if &*n == "super" {
                    self.sc().used.insert(Rc::from("__class__"));
                }
                self.sc().used.insert(n);
            }
            Expr::Const(_) => {}
            Expr::JoinedStr(parts) => {
                for p in parts {
                    self.expr(p)?;
                }
            }
            Expr::FormattedValue { value, spec, .. } => {
                self.expr(value)?;
                if let Some(s) = spec {
                    self.expr(s)?;
                }
            }
            Expr::Bin(_, a, b) | Expr::BoolAnd(a, b) | Expr::BoolOr(a, b) | Expr::Subscript(a, b) => {
                self.expr(a)?;
                self.expr(b)?;
            }
            Expr::Unary(_, a) | Expr::Starred(a) | Expr::Await(a) | Expr::Attr(a, _) => self.expr(a)?,
            Expr::Compare(a, rest) => {
                self.expr(a)?;
                for (_, b) in rest {
                    self.expr(b)?;
                }
            }
            Expr::IfExp(c, a, b) => {
                self.expr(c)?;
                self.expr(a)?;
                self.expr(b)?;
            }
            Expr::Call { func, args } => {
                self.expr(func)?;
                for a in args {
                    match a {
                        Arg::Pos(e) | Arg::Star(e) | Arg::Kw(_, e) | Arg::StarStar(e) => self.expr(e)?,
                    }
                }
            }
            Expr::Slice(a, b, c) => {
                for x in [a, b, c].into_iter().flatten() {
                    self.expr(x)?;
                }
            }
            Expr::List(items) | Expr::Tuple(items) | Expr::Set(items) => {
                for i in items {
                    self.expr(i)?;
                }
            }
            Expr::Dict(items) => {
                for (k, v) in items {
                    if let Some(k) = k {
                        self.expr(k)?;
                    }
                    self.expr(v)?;
                }
            }
            Expr::Lambda(f) => self.func(f)?,
            Expr::ListComp(elt, comps) | Expr::SetComp(elt, comps) => self.comp(key(e), &[elt], comps)?,
            Expr::GenExp(elt, comps) => {
                self.comp(key(e), &[elt], comps)?;
                let id = self.st.by_node[&key(e)];
                self.st.scopes[id].is_gen = true;
            }
            Expr::DictComp(k, v, comps) => self.comp(key(e), &[k, v], comps)?,
            Expr::NamedExpr(n, v) => {
                self.expr(v)?;
                // in a comprehension the name binds in the enclosing function
                let mut s = self.cur;
                while self.st.scopes[s].is_comp {
                    let n2 = n.clone();
                    self.st.scopes[s].nonlocals.insert(n2);
                    s = self.st.scopes[s].parent.unwrap();
                }
                if s != self.cur {
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
            Expr::Yield(v) => {
                self.sc().is_gen = true;
                if let Some(v) = v {
                    self.expr(v)?;
                }
            }
            Expr::YieldFrom(v) => {
                self.sc().is_gen = true;
                self.expr(v)?;
            }
        }
        Ok(())
    }
}

impl SymTable {
    fn build(body: &[Stmt]) -> CResult<SymTable> {
        let mut st = SymTable { scopes: vec![Scope::new(ScopeKind::Module, None)], by_node: HashMap::new(), future_annotations: false };
        {
            let mut c = Collector { st: &mut st, cur: 0, cls: vec![] };
            c.stmts(body)?;
        }
        st.resolve()?;
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
                    return Err(CompileError { msg: format!("no binding for nonlocal '{}' found", n), line: 0 });
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
    With,
}

struct Unit<'a> {
    scope: usize,
    kind: ScopeKind,
    ops: Vec<Op>,
    lines: Vec<u32>,
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
    line: u32,
    qualname: String,
    tmp: u32,
}

pub struct Compiler<'a> {
    st: SymTable,
    units: Vec<Unit<'a>>,
    filename: Rc<str>,
}

pub fn compile_module(body: &[Stmt], filename: &str) -> CResult<Rc<Code>> {
    let st = SymTable::build(body)?;
    let mut c = Compiler { st, units: vec![], filename: Rc::from(filename) };
    c.push_unit(0, "<module>".into());
    let has_ann = body.iter().any(|s| matches!(s.kind, StmtKind::AnnAssign(..)));
    if has_ann {
        c.emit(Op::SetupAnnotations);
    }
    // a docstring
    if let Some(Stmt { kind: StmtKind::Expr(Expr::Const(Const::Str(d))), .. }) = body.first() {
        let k = c.const_str(d);
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

/// compiles an expression for eval()
pub fn compile_eval(e: &Expr, filename: &str) -> CResult<Rc<Code>> {
    let body = vec![Stmt { line: 1, kind: StmtKind::Return(Some(e.clone())) }];
    let st = SymTable::build(&body)?;
    let mut c = Compiler { st, units: vec![], filename: Rc::from(filename) };
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
        for (i, n) in cellvars.iter().chain(freevars.iter()).enumerate() {
            cell_map.insert(n.clone(), i as u32);
        }
        let kind = s.kind;
        self.units.push(Unit {
            scope,
            kind,
            ops: vec![],
            lines: vec![],
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
            line: 1,
            qualname,
            tmp: 0,
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
            coroutine: false,
            is_class: u.kind == ScopeKind::Class,
            cell_from_arg: u.cellvars.iter().map(|c| u.var_map.get(c).copied()).collect(),
            varnames: u.varnames,
            cellvars: u.cellvars,
            freevars: u.freevars,
            ops: u.ops,
            lines: u.lines,
            consts: u.consts,
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
        let line = self.u().line;
        let u = self.u();
        u.ops.push(op);
        u.lines.push(line);
        u.ops.len() - 1
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
        let i = self.const_val(Value::str(s));
        self.u().str_consts.insert(s.to_string(), i);
        i
    }

    fn name_idx(&mut self, n: &Name) -> u32 {
        if let Some(&i) = self.u().name_map.get(n) {
            return i;
        }
        let u = self.u();
        u.names.push(Rc::new(PyStr::new(n.to_string())));
        let i = (u.names.len() - 1) as u32;
        u.name_map.insert(n.clone(), i);
        i
    }

    fn err(&self, msg: &str) -> CompileError {
        CompileError { msg: msg.into(), line: self.units.last().map(|u| u.line).unwrap_or(0) }
    }

    // ------------------------------------------------------------- names

    fn load_name(&mut self, n: &Name) {
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
                } else {
                    let i = self.name_idx(n);
                    self.emit(Op::LoadGlobal(i));
                }
            }
            ScopeKind::Class => {
                if s.globals.contains(n) {
                    let i = self.name_idx(n);
                    self.emit(Op::LoadGlobal(i));
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
        self.u().line = s.line;
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
            StmtKind::AugAssign(t, op, v) => match t {
                Expr::Name(n) => {
                    self.load_name(n);
                    self.expr(v)?;
                    self.emit(Op::Inplace(*op));
                    self.store_name(n);
                }
                Expr::Attr(obj, attr) => {
                    self.expr(obj)?;
                    self.emit(Op::Dup);
                    let i = self.name_idx(attr);
                    self.emit(Op::LoadAttr(i));
                    self.expr(v)?;
                    self.emit(Op::Inplace(*op));
                    self.emit(Op::Rot2);
                    self.emit(Op::StoreAttr(i));
                }
                Expr::Subscript(obj, idx) => {
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
            StmtKind::AnnAssign(t, ann, v) => {
                if let Some(v) = v {
                    self.expr(v)?;
                    self.store(t)?;
                }
                let kind = self.u().kind;
                if let (Expr::Name(n), ScopeKind::Class | ScopeKind::Module) = (t, kind) {
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
            StmtKind::For(target, iter, body, orelse) => {
                self.expr(iter)?;
                self.emit(Op::GetIter);
                let start = self.here();
                let fi = self.emit(Op::ForIter(0));
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
                    self.u().blocks.push(FBlock::Handler);
                    self.stmts(finally)?;
                    self.u().blocks.pop();
                    self.emit(Op::Reraise);
                    let h = self.here();
                    self.patch(end, h);
                }
            }
            StmtKind::With(items, body) => self.with(items, 0, body)?,
            StmtKind::Import(names) => {
                for (m, alias) in names {
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
                let fl: Vec<Value> = names.iter().map(|(n, _)| Value::str(n)).collect();
                let k = self.const_val(Value::tuple(fl));
                self.emit(Op::LoadConst(k));
                let i = self.name_idx(module);
                self.emit(Op::ImportName(i));
                if names.len() == 1 && &*names[0].0 == "*" {
                    self.emit(Op::ImportStar);
                } else {
                    for (n, alias) in names {
                        let i = self.name_idx(n);
                        self.emit(Op::ImportFrom(i));
                        self.store_name(alias.as_ref().unwrap_or(n));
                    }
                    self.emit(Op::Pop);
                }
            }
            StmtKind::Assert(cond, msg) => {
                self.jump_if(cond, true)?;
                let j = self.emit(Op::PopJumpIfTrue(0));
                self.emit(Op::LoadAssertionError);
                if let Some(m) = msg {
                    self.expr(m)?;
                    self.emit(Op::Call(1));
                }
                self.emit(Op::Raise(1));
                let h = self.here();
                self.patch(j, h);
            }
            StmtKind::Match(subject, cases) => self.match_stmt(subject, cases)?,
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
                    self.stmts(body)?;
                    self.u().blocks.extend(saved);
                }
                FBlock::Handler => {
                    self.emit(Op::PopExcept);
                }
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
            self.u().line = hd.line;
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
            self.emit(Op::Reraise);
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
            self.u().line = hd.line;
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
        let setup = self.emit(Op::SetupWith(0));
        match t {
            Some(t) => self.store(t)?,
            None => {
                self.emit(Op::Pop);
            }
        }
        self.u().blocks.push(FBlock::With);
        self.with(items, i + 1, body)?;
        self.u().blocks.pop();
        self.emit(Op::PopBlock);
        self.emit(Op::WithExit(0));
        let end = self.emit(Op::Jump(0));
        let h = self.here();
        self.patch(setup, h);
        self.emit(Op::WithExcept);
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
        self.u().line = c.line;
        let mn = self.name_idx(&Rc::from("__name__"));
        self.emit(Op::LoadGlobal(mn));
        let mm = self.name_idx(&Rc::from("__module__"));
        self.emit(Op::StoreName(mm));
        let q = self.const_str(&qual);
        self.emit(Op::LoadConst(q));
        let qn = self.name_idx(&Rc::from("__qualname__"));
        self.emit(Op::StoreName(qn));
        let mut doc = Value::None;
        if let Some(Stmt { kind: StmtKind::Expr(Expr::Const(Const::Str(d))), .. }) = c.body.first() {
            doc = Value::str(d);
            let k = self.const_str(d);
            self.emit(Op::LoadConst(k));
            let dn = self.name_idx(&Rc::from("__doc__"));
            self.emit(Op::StoreName(dn));
        }
        if c.body.iter().any(|s| matches!(s.kind, StmtKind::AnnAssign(..))) {
            self.emit(Op::SetupAnnotations);
        }
        self.stmts(&c.body)?;
        let cls_cell: Name = Rc::from("__class__");
        if let Some(&ci) = self.u().cell_map.get(&cls_cell) {
            if self.u().cellvars.contains(&cls_cell) {
                self.emit(Op::LoadClosure(ci));
                self.emit(Op::Return);
            } else {
                let k = self.const_val(Value::None);
                self.emit(Op::LoadConst(k));
                self.emit(Op::Return);
            }
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

    /// pushes the function object for `code`, with `flags` already pushed
    fn emit_closure_and_make(&mut self, code: Code, mut flags: u8) -> CResult<()> {
        if !code.freevars.is_empty() {
            for fv in code.freevars.clone() {
                let ci = match self.u().cell_map.get(&fv) {
                    Some(&c) => c,
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
                let k = self.const_str(&p.name);
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
                let k = self.const_str(name);
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
        self.u().line = f.line;
        let mut doc = Value::None;
        let mut body: &'a [Stmt] = &f.body;
        if !f.is_lambda {
            if let Some(Stmt { kind: StmtKind::Expr(Expr::Const(Const::Str(d))), .. }) = f.body.first() {
                doc = Value::str(d);
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
        let name = match kind {
            0 => "<listcomp>",
            1 => "<setcomp>",
            2 => "<dictcomp>",
            _ => "<genexpr>",
        };
        let qual = self.child_qualname(name);
        self.expr(&comps[0].iter)?;
        self.emit(Op::GetIter);
        let line = self.u().line;
        self.push_unit(scope, qual);
        self.u().line = line;
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
        self.emit_closure_and_make(code, 0)?;
        self.emit(Op::Rot2);
        self.emit(Op::Call(1));
        Ok(())
    }

    fn comp_loop(&mut self, comps: &'a [Comp], i: usize, kind: u8, elt: &'a Expr, val: Option<&'a Expr>) -> CResult<()> {
        let c = &comps[i];
        if i == 0 {
            self.emit(Op::LoadFast(0));
        } else {
            self.expr(&c.iter)?;
            self.emit(Op::GetIter);
        }
        let start = self.here();
        let fi = self.emit(Op::ForIter(0));
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
        self.patch(fi, h);
        Ok(())
    }

    // ------------------------------------------------------------- targets

    fn store(&mut self, t: &'a Expr) -> CResult<()> {
        match t {
            Expr::Name(n) => self.store_name(n),
            Expr::Attr(obj, attr) => {
                self.expr(obj)?;
                let i = self.name_idx(&mangle_attr(&self.class_name(), attr));
                self.emit(Op::StoreAttr(i));
            }
            Expr::Subscript(obj, idx) => {
                self.expr(obj)?;
                self.expr(idx)?;
                self.emit(Op::StoreSubscr);
            }
            Expr::Tuple(items) | Expr::List(items) => {
                let star = items.iter().position(|e| matches!(e, Expr::Starred(_)));
                match star {
                    Some(si) => {
                        if items.iter().filter(|e| matches!(e, Expr::Starred(_))).count() > 1 {
                            return Err(self.err("multiple starred expressions in assignment"));
                        }
                        self.emit(Op::UnpackEx(si as u16, (items.len() - si - 1) as u16));
                    }
                    None => {
                        self.emit(Op::Unpack(items.len() as u32));
                    }
                }
                for it in items {
                    match it {
                        Expr::Starred(inner) => self.store(inner)?,
                        _ => self.store(it)?,
                    }
                }
            }
            Expr::Starred(_) => return Err(self.err("starred assignment target must be in a list or tuple")),
            _ => return Err(self.err("cannot assign to expression")),
        }
        Ok(())
    }

    fn del(&mut self, t: &'a Expr) -> CResult<()> {
        match t {
            Expr::Name(n) => self.del_name(n),
            Expr::Attr(obj, attr) => {
                self.expr(obj)?;
                let i = self.name_idx(attr);
                self.emit(Op::DelAttr(i));
            }
            Expr::Subscript(obj, idx) => {
                self.expr(obj)?;
                self.expr(idx)?;
                self.emit(Op::DelSubscr);
            }
            Expr::Tuple(items) | Expr::List(items) => {
                for i in items {
                    self.del(i)?;
                }
            }
            _ => return Err(self.err("cannot delete expression")),
        }
        Ok(())
    }

    // ------------------------------------------------------------- expressions

    fn constant(&mut self, c: &Const) -> u32 {
        match c {
            Const::Str(s) => self.const_str(s),
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

    fn expr(&mut self, e: &'a Expr) -> CResult<()> {
        match e {
            Expr::Name(n) => self.load_name(n),
            Expr::Const(c) => {
                let k = self.constant(c);
                self.emit(Op::LoadConst(k));
            }
            Expr::JoinedStr(parts) => {
                for p in parts {
                    self.expr(p)?;
                }
                if parts.len() != 1 || !matches!(parts[0], Expr::Const(_)) {
                    self.emit(Op::BuildString(parts.len() as u32));
                }
            }
            Expr::FormattedValue { value, conv, spec } => {
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
            Expr::Bin(op, a, b) => {
                self.expr(a)?;
                self.expr(b)?;
                self.emit(Op::Binary(*op));
            }
            Expr::Unary(op, a) => {
                self.expr(a)?;
                self.emit(Op::Unary(*op));
            }
            Expr::BoolAnd(a, b) => {
                self.expr(a)?;
                let j = self.emit(Op::JumpIfFalseOrPop(0));
                self.expr(b)?;
                let h = self.here();
                self.patch(j, h);
            }
            Expr::BoolOr(a, b) => {
                self.expr(a)?;
                let j = self.emit(Op::JumpIfTrueOrPop(0));
                self.expr(b)?;
                let h = self.here();
                self.patch(j, h);
            }
            Expr::Compare(first, rest) => {
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
            Expr::IfExp(c, a, b) => {
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
            Expr::Call { func, args } => {
                let simple = args.iter().all(|a| matches!(a, Arg::Pos(_)));
                if let (Expr::Attr(obj, attr), true) = (&**func, simple) {
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
            Expr::Attr(obj, attr) => {
                self.expr(obj)?;
                let i = self.name_idx(&mangle_attr(&self.class_name(), attr));
                self.emit(Op::LoadAttr(i));
            }
            Expr::Subscript(obj, idx) => {
                self.expr(obj)?;
                self.expr(idx)?;
                self.emit(Op::Subscr);
            }
            Expr::Slice(a, b, c) => {
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
            Expr::List(items) => {
                if items.iter().any(|e| matches!(e, Expr::Starred(_))) {
                    self.starred_list(items)?;
                } else {
                    for i in items {
                        self.expr(i)?;
                    }
                    self.emit(Op::BuildList(items.len() as u32));
                }
            }
            Expr::Tuple(items) => {
                if items.iter().any(|e| matches!(e, Expr::Starred(_))) {
                    self.starred_list(items)?;
                    self.emit(Op::ListToTuple);
                } else if !items.is_empty() && items.iter().all(|e| matches!(e, Expr::Const(_))) {
                    let vals: Vec<Value> = items
                        .iter()
                        .map(|e| match e {
                            Expr::Const(c) => {
                                let k = self.constant(c);
                                self.u().consts[k as usize].clone()
                            }
                            _ => unreachable!(),
                        })
                        .collect();
                    let k = self.const_val(Value::tuple(vals));
                    self.emit(Op::LoadConst(k));
                } else {
                    for i in items {
                        self.expr(i)?;
                    }
                    self.emit(Op::BuildTuple(items.len() as u32));
                }
            }
            Expr::Set(items) => {
                if items.iter().any(|e| matches!(e, Expr::Starred(_))) {
                    self.emit(Op::BuildSet(0));
                    for i in items {
                        match i {
                            Expr::Starred(inner) => {
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
            Expr::Dict(items) => {
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
            Expr::Starred(_) => return Err(self.err("can't use starred expression here")),
            Expr::Lambda(f) => self.make_function(f)?,
            Expr::ListComp(elt, comps) => self.comprehension(e, comps, 0, elt, None)?,
            Expr::SetComp(elt, comps) => self.comprehension(e, comps, 1, elt, None)?,
            Expr::DictComp(k, v, comps) => self.comprehension(e, comps, 2, k, Some(v))?,
            Expr::GenExp(elt, comps) => self.comprehension(e, comps, 3, elt, None)?,
            Expr::NamedExpr(n, v) => {
                self.expr(v)?;
                self.emit(Op::Dup);
                self.store_name(n);
            }
            Expr::Yield(v) => {
                if self.u().kind != ScopeKind::Function {
                    return Err(self.err("'yield' outside function"));
                }
                match v {
                    Some(v) => self.expr(v)?,
                    None => {
                        let k = self.const_val(Value::None);
                        self.emit(Op::LoadConst(k));
                    }
                }
                self.emit(Op::Yield);
            }
            Expr::YieldFrom(v) => {
                self.expr(v)?;
                self.emit(Op::GetIter);
                let k = self.const_val(Value::None);
                self.emit(Op::LoadConst(k));
                self.emit(Op::YieldFrom);
            }
            Expr::Await(v) => {
                self.expr(v)?;
                self.emit(Op::GetAwaitable);
                let k = self.const_val(Value::None);
                self.emit(Op::LoadConst(k));
                self.emit(Op::YieldFrom);
            }
        }
        Ok(())
    }

    fn starred_list(&mut self, items: &'a [Expr]) -> CResult<()> {
        self.emit(Op::BuildList(0));
        for i in items {
            match i {
                Expr::Starred(inner) => {
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
        let has_star = args.iter().any(|a| matches!(a, Arg::Star(_) | Arg::StarStar(_)));
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
                    Arg::Kw(n, e) => {
                        if kwnames.iter().any(|k: &Value| k.as_str() == Some(n)) {
                            return Err(self.err(&format!("keyword argument repeated: {}", n)));
                        }
                        self.expr(e)?;
                        kwnames.push(Value::str(n));
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
                    self.expr(e)?;
                    self.emit(Op::ListExtend(1));
                }
                _ => {}
            }
        }
        self.emit(Op::ListToTuple);
        for a in args {
            match a {
                Arg::Kw(n, e) => {
                    if !have_kw {
                        self.emit(Op::BuildMap(0));
                        have_kw = true;
                    }
                    let k = self.const_str(n);
                    self.emit(Op::LoadConst(k));
                    self.expr(e)?;
                    self.emit(Op::MapAdd(1));
                }
                Arg::StarStar(e) => {
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

fn collect_pattern<'a>(p: &'a Pattern, caps: &mut Vec<Name>, vals: &mut Vec<&'a Expr>) {
    match p {
        Pattern::Capture(Some(n)) | Pattern::Star(Some(n)) => caps.push(n.clone()),
        Pattern::Capture(None) | Pattern::Star(None) | Pattern::Singleton(_) => {}
        Pattern::Value(e) => vals.push(e),
        Pattern::Sequence(ps) => {
            for q in ps {
                collect_pattern(q, caps, vals);
            }
        }
        Pattern::Or(ps) => {
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
        Pattern::Mapping { keys, pats, rest } => {
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
        Pattern::Class { cls, args, kw } => {
            vals.push(cls);
            for q in args {
                collect_pattern(q, caps, vals);
            }
            for (_, q) in kw {
                collect_pattern(q, caps, vals);
            }
        }
        Pattern::As(q, n) => {
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
    match e {
        Expr::Name(n) => n.to_string(),
        Expr::Attr(o, a) => format!("{}.{}", unparse(o), a),
        Expr::Const(Const::Str(s)) => format!("'{}'", s),
        Expr::Const(Const::None) => "None".into(),
        Expr::Const(Const::Int(i)) => i.to_string(),
        Expr::Const(Const::Ellipsis) => "...".into(),
        Expr::Subscript(o, i) => format!("{}[{}]", unparse(o), unparse(i)),
        Expr::Tuple(items) => items.iter().map(unparse).collect::<Vec<_>>().join(", "),
        Expr::List(items) => format!("[{}]", items.iter().map(unparse).collect::<Vec<_>>().join(", ")),
        Expr::Bin(BinOp::BitOr, a, b) => format!("{} | {}", unparse(a), unparse(b)),
        Expr::Call { func, .. } => format!("{}(...)", unparse(func)),
        _ => "...".into(),
    }
}
