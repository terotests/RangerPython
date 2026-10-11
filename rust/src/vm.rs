// SPDX-License-Identifier: MIT
//! The virtual machine: frames, the instruction loop, calls, classes,
//! exceptions, generators and imports.

use crate::ast::{BinOp, CmpOp, UnOp};
use crate::code::{Code, Op};
use crate::value::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub const BLOCK_EXCEPT: u8 = 0;
pub const BLOCK_FINALLY: u8 = 1;
pub const BLOCK_WITH: u8 = 2;
pub const BLOCK_HANDLER: u8 = 3;

#[derive(Clone, Copy)]
pub struct Block {
    pub kind: u8,
    pub handler: u32,
    pub level: u32,
    pub line: u32,
}

pub struct Frame {
    pub code: Rc<Code>,
    pub ip: usize,
    /// the fast locals, then the value stack
    pub stack: Vec<Value>,
    pub cells: Vec<Cellv>,
    pub blocks: Vec<Block>,
    pub globals: Rc<RefCell<Dict>>,
    /// the namespace LoadName/StoreName use (module globals or class body)
    pub ns: Option<Rc<RefCell<Dict>>>,
    pub reraise_line: Option<u32>,
    /// exec/eval namespaces that are not plain dicts
    pub ext: Option<Rc<FrameExt>>,
}

/// namespaces handed to exec()/eval() that need item protocol access
pub struct FrameExt {
    /// a dict subclass instance used as globals
    pub globals: Option<Value>,
    /// any mapping used as locals
    pub locals: Option<Value>,
    /// a `__builtins__` mapping other than the builtins module
    pub builtins: Option<Value>,
}

pub enum Exit {
    Return(Value),
    Yield(Value),
}

pub struct Generator {
    pub frame: Option<Box<Frame>>,
    pub started: bool,
    pub running: bool,
    pub name: Rc<PyStr>,
    pub qualname: Rc<PyStr>,
    pub coroutine: bool,
    /// the exceptions its frame was handling when it last yielded
    pub handled: Vec<Value>,
    /// gi_code / cr_code, kept after the frame is gone
    pub code: Rc<Code>,
    /// gi_frame, made on first access so it keeps its identity
    pub frame_obj: Option<Value>,
}

impl Drop for Generator {
    fn drop(&mut self) {
        // a generator suspended inside try or with is closed when it goes
        // away: it comes back to life once, so that its handlers can run
        let resumable = self.started && !self.running && !self.coroutine;
        if !resumable || !self.frame.as_ref().is_some_and(|f| !f.blocks.is_empty()) {
            return;
        }
        let queue_open = crate::value::PENDING_DEL.try_with(|p| p.try_borrow_mut().is_ok()).unwrap_or(false);
        if !queue_open {
            return;
        }
        let g = Generator {
            frame: self.frame.take(),
            started: true,
            running: false,
            name: self.name.clone(),
            qualname: self.qualname.clone(),
            coroutine: false,
            handled: std::mem::take(&mut self.handled),
            code: self.code.clone(),
            frame_obj: None,
        };
        let g = Rc::new(RefCell::new(g));
        crate::gc::track(crate::gc::Tracked::Gen(Rc::downgrade(&g)));
        crate::value::PENDING_DEL.with(|p| p.borrow_mut().push(Value::Gen(g)));
        crate::value::HAS_PENDING_DEL.with(|h| h.set(true));
    }
}

/// interned names the VM looks up often
#[derive(Clone)]
pub struct Names {
    pub init: Rc<PyStr>,
    pub new: Rc<PyStr>,
    pub call: Rc<PyStr>,
    pub iter: Rc<PyStr>,
    pub next: Rc<PyStr>,
    pub len: Rc<PyStr>,
    pub getitem: Rc<PyStr>,
    pub setitem: Rc<PyStr>,
    pub delitem: Rc<PyStr>,
    pub contains: Rc<PyStr>,
    pub repr: Rc<PyStr>,
    pub str_: Rc<PyStr>,
    pub bool_: Rc<PyStr>,
    pub hash: Rc<PyStr>,
    pub eq: Rc<PyStr>,
    pub ne: Rc<PyStr>,
    pub lt: Rc<PyStr>,
    pub le: Rc<PyStr>,
    pub gt: Rc<PyStr>,
    pub ge: Rc<PyStr>,
    pub getattr: Rc<PyStr>,
    pub getattribute: Rc<PyStr>,
    pub setattr: Rc<PyStr>,
    pub delattr: Rc<PyStr>,
    pub get: Rc<PyStr>,
    pub set: Rc<PyStr>,
    pub delete: Rc<PyStr>,
    pub enter: Rc<PyStr>,
    pub exit: Rc<PyStr>,
    pub name: Rc<PyStr>,
    pub qualname: Rc<PyStr>,
    pub module: Rc<PyStr>,
    pub doc: Rc<PyStr>,
    pub dict: Rc<PyStr>,
    pub class: Rc<PyStr>,
    pub format: Rc<PyStr>,
    pub index: Rc<PyStr>,
    pub missing: Rc<PyStr>,
    pub tb: Rc<PyStr>,
    pub cause: Rc<PyStr>,
    pub context: Rc<PyStr>,
    pub suppress: Rc<PyStr>,
    pub annotations: Rc<PyStr>,
    pub init_subclass: Rc<PyStr>,
    pub set_name: Rc<PyStr>,
    pub prepare: Rc<PyStr>,
    pub mro_entries: Rc<PyStr>,
    pub class_getitem: Rc<PyStr>,
    pub instancecheck: Rc<PyStr>,
    pub subclasscheck: Rc<PyStr>,
    pub float_: Rc<PyStr>,
    pub int_: Rc<PyStr>,
    pub reversed: Rc<PyStr>,
    pub notes: Rc<PyStr>,
    pub match_args: Rc<PyStr>,
    pub slots: Rc<PyStr>,
    pub abs: Rc<PyStr>,
    pub neg: Rc<PyStr>,
    pub pos: Rc<PyStr>,
    pub invert: Rc<PyStr>,
    pub round: Rc<PyStr>,
    pub classcell: Rc<PyStr>,
    binops: [[Rc<PyStr>; 3]; 13],
}

fn ps(s: &str) -> Rc<PyStr> {
    crate::value::intern(s)
}

impl Names {
    fn new() -> Names {
        Names {
            init: ps("__init__"),
            new: ps("__new__"),
            call: ps("__call__"),
            iter: ps("__iter__"),
            next: ps("__next__"),
            len: ps("__len__"),
            getitem: ps("__getitem__"),
            setitem: ps("__setitem__"),
            delitem: ps("__delitem__"),
            contains: ps("__contains__"),
            repr: ps("__repr__"),
            str_: ps("__str__"),
            bool_: ps("__bool__"),
            hash: ps("__hash__"),
            eq: ps("__eq__"),
            ne: ps("__ne__"),
            lt: ps("__lt__"),
            le: ps("__le__"),
            gt: ps("__gt__"),
            ge: ps("__ge__"),
            getattr: ps("__getattr__"),
            getattribute: ps("__getattribute__"),
            setattr: ps("__setattr__"),
            delattr: ps("__delattr__"),
            get: ps("__get__"),
            set: ps("__set__"),
            delete: ps("__delete__"),
            enter: ps("__enter__"),
            exit: ps("__exit__"),
            name: ps("__name__"),
            qualname: ps("__qualname__"),
            module: ps("__module__"),
            doc: ps("__doc__"),
            dict: ps("__dict__"),
            class: ps("__class__"),
            format: ps("__format__"),
            index: ps("__index__"),
            missing: ps("__missing__"),
            tb: ps("__traceback__"),
            cause: ps("__cause__"),
            context: ps("__context__"),
            suppress: ps("__suppress_context__"),
            annotations: ps("__annotations__"),
            init_subclass: ps("__init_subclass__"),
            set_name: ps("__set_name__"),
            prepare: ps("__prepare__"),
            mro_entries: ps("__mro_entries__"),
            class_getitem: ps("__class_getitem__"),
            instancecheck: ps("__instancecheck__"),
            subclasscheck: ps("__subclasscheck__"),
            float_: ps("__float__"),
            int_: ps("__int__"),
            reversed: ps("__reversed__"),
            notes: ps("__notes__"),
            match_args: ps("__match_args__"),
            slots: ps("__slots__"),
            abs: ps("__abs__"),
            neg: ps("__neg__"),
            pos: ps("__pos__"),
            invert: ps("__invert__"),
            round: ps("__round__"),
            classcell: ps("__classcell__"),
            binops: crate::ops::BINOPS.map(|op| {
                let (f, r, i) = crate::ops::op_dunder(op);
                [ps(f), ps(r), ps(i)]
            }),
        }
    }

    /// the forward, reflected and in-place method names of a binary operator
    pub fn binop(&self, op: crate::ast::BinOp) -> &[Rc<PyStr>; 3] {
        &self.binops[op as usize]
    }
}

#[derive(Clone)]
pub struct Types {
    pub object: Rc<Class>,
    pub type_: Rc<Class>,
    pub int: Rc<Class>,
    pub bool_: Rc<Class>,
    pub float: Rc<Class>,
    pub complex: Rc<Class>,
    pub str_: Rc<Class>,
    pub bytes: Rc<Class>,
    pub bytearray: Rc<Class>,
    pub list: Rc<Class>,
    pub tuple: Rc<Class>,
    pub dict: Rc<Class>,
    pub set: Rc<Class>,
    pub frozenset: Rc<Class>,
    pub range: Rc<Class>,
    pub slice: Rc<Class>,
    pub none: Rc<Class>,
    pub function: Rc<Class>,
    pub builtin_function: Rc<Class>,
    /// a builtin type's slot method, and one bound to an instance
    pub wrapper_descriptor: Rc<Class>,
    pub method_wrapper: Rc<Class>,
    pub method: Rc<Class>,
    pub module: Rc<Class>,
    pub iterator: Rc<Class>,
    pub enumerate: Rc<Class>,
    pub zip: Rc<Class>,
    pub map: Rc<Class>,
    pub filter: Rc<Class>,
    pub reversed: Rc<Class>,
    pub generator: Rc<Class>,
    pub coroutine: Rc<Class>,
    pub dict_keys: Rc<Class>,
    pub dict_values: Rc<Class>,
    pub dict_items: Rc<Class>,
    pub property: Rc<Class>,
    pub staticmethod: Rc<Class>,
    pub classmethod: Rc<Class>,
    pub super_: Rc<Class>,
    pub notimpl: Rc<Class>,
    pub ellipsis: Rc<Class>,
    pub cell: Rc<Class>,
    pub code: Rc<Class>,
    pub obj: Rc<Class>,
    pub file: Rc<Class>,
    pub base_exception: Rc<Class>,
}

crate::gil_static! {
    static OBJ_CLASS: RefCell<Option<Rc<Class>>> = const { RefCell::new(None) };
}

/// the class of internal objects (match patterns) made at compile time
pub fn obj_class() -> Rc<Class> {
    OBJ_CLASS.with(|c| {
        let mut c = c.borrow_mut();
        if c.is_none() {
            *c = Some(Class::new("object", vec![], Builtin::Obj));
        }
        c.clone().unwrap()
    })
}

/// a table keyed by a built-in name, hashed with the fast FxHasher: the
/// exception and native classes are looked up on every raise
pub type NameMap<V> = HashMap<&'static str, V, std::hash::BuildHasherDefault<crate::value::FxHasher>>;

pub struct Vm {
    pub t: Types,
    pub n: Names,
    pub builtins: Rc<RefCell<Dict>>,
    pub exc: NameMap<Rc<Class>>,
    pub modules: Rc<RefCell<Dict>>,
    /// exceptions being handled, innermost last (sys.exc_info)
    pub handled: Vec<Value>,
    pub out: String,
    /// what the script wrote to sys.stderr
    pub err_out: String,
    pub fs: crate::fs::Fs,
    pub depth: u32,
    pub max_depth: u32,
    pub argv: Vec<String>,
    pub steps: u64,
    pub max_steps: u64,
    pub main_globals: Rc<RefCell<Dict>>,
    pub repr_stack: Vec<usize>,
    pub random: crate::modules::Random,
    pub start_time: std::time::Instant,
    /// the value a finished generator or iterator returned
    pub last_return: Option<Value>,
    /// the frames running now, innermost last (locals(), super(), eval)
    pub frames: Vec<*mut Frame>,
    /// the frame object of each running frame once something asked for it
    pub frame_objs: Vec<Value>,
    /// value stacks of finished frames, reused by the next calls
    pub stack_pool: Vec<Vec<Value>>,
    /// write output to the process's stdout/stderr as it comes
    pub echo: bool,
    /// folders imports look in after /
    pub search_path: Vec<String>,
    /// set by a bare `raise`: the traceback gets no new entry
    pub bare_reraise: bool,
    /// classes of native objects (traceback, frame...), by name
    pub classes: NameMap<Rc<Class>>,
    /// how the interpreter was started (sys.executable, sys.flags)
    pub host: crate::process::Host,
    /// one of the interpreter's threads, switching the GIL (a
    /// sub-interpreter's Vm runs inside its creator's turn)
    pub in_gil: bool,
}

impl Vm {
    pub fn new(fs: crate::fs::Fs) -> Vm {
        let t = crate::builtins::make_types();
        let mut vm = Vm {
            t,
            n: Names::new(),
            builtins: Rc::new(RefCell::new(Dict::new())),
            exc: NameMap::default(),
            modules: Rc::new(RefCell::new(Dict::new())),
            handled: vec![],
            out: String::new(),
            err_out: String::new(),
            fs,
            depth: 0,
            max_depth: 1000,
            argv: vec![],
            steps: 0,
            max_steps: 0,
            main_globals: Rc::new(RefCell::new(Dict::new())),
            repr_stack: vec![],
            random: crate::modules::Random::new(),
            start_time: std::time::Instant::now(),
            last_return: None,
            frames: Vec::with_capacity(64),
            frame_objs: Vec::with_capacity(64),
            stack_pool: Vec::new(),
            classes: NameMap::default(),
            bare_reraise: false,
            echo: false,
            search_path: vec![],
            host: Default::default(),
            in_gil: false,
        };
        crate::builtins::install(&mut vm);
        let fc = vm.t.function.clone();
        crate::tb::add_getsets(&mut vm, &fc, &["__code__", "__defaults__", "__kwdefaults__", "__annotations__", "__dict__", "__name__", "__qualname__", "__type_params__"], &["__globals__", "__closure__", "__builtins__"]);
        let g = vm.main_globals.clone();
        g.borrow_mut().set_s("__name__", Value::str("__main__"));
        g.borrow_mut().set_s("__file__", Value::str("<script>"));
        g.borrow_mut().set_s("__builtins__", Value::Module(Rc::new(Module { name: ps("builtins"), dict: vm.builtins.clone() })));
        g.borrow_mut().set_s("__doc__", Value::None);
        let main = Value::Module(Rc::new(Module { name: ps("__main__"), dict: g.clone() }));
        vm.modules.borrow_mut().set_s("__main__", main);
        vm.load_py_builtins();
        vm
    }

    /// the built-ins written in Python (pylib/_pybuiltins.py)
    fn load_py_builtins(&mut self) {
        let g = self.main_globals.clone();
        let m = match self.import("_pybuiltins", &Value::None, 0, &g) {
            Ok(m) => m,
            Err(_) => return,
        };
        let names = self.getattr(&m, &ps("__all__")).ok().and_then(|v| self.to_vec(&v).ok()).unwrap_or_default();
        for n in names {
            if let Some(s) = n.as_str() {
                if let Ok(v) = self.getattr(&m, &ps(s)) {
                    self.builtins.borrow_mut().set_s(s, v);
                }
            }
        }
    }

    pub fn write_out(&mut self, s: &str) {
        if self.echo {
            use std::io::Write;
            let _ = std::io::stdout().write_all(s.as_bytes());
        } else {
            self.out.push_str(s);
        }
    }

    pub fn write_err(&mut self, s: &str) {
        if self.echo {
            use std::io::Write;
            let _ = std::io::stderr().write_all(s.as_bytes());
        } else {
            self.err_out.push_str(s);
        }
    }

    // ------------------------------------------------------------------
    // errors

    pub fn exc_class(&self, name: &str) -> Rc<Class> {
        match self.exc.get(name) {
            Some(c) => c.clone(),
            None => self.t.base_exception.clone(),
        }
    }

    pub fn new_exc(&mut self, cls: &Rc<Class>, args: Vec<Value>) -> Value {
        let inst = Rc::new(Instance::new(cls.clone(), Value::tuple(args)));
        crate::gc::track_inst(&inst);
        let v = Value::Instance(inst);
        crate::builtins::exc_init_native(self, &v);
        v
    }

    pub fn err(&mut self, kind: &str, msg: impl Into<String>) -> Exc {
        let cls = self.exc_class(kind);
        let m: String = msg.into();
        let v = self.new_exc(&cls, vec![Value::string(m)]);
        Exc { value: v }
    }

    pub fn type_error(&mut self, msg: impl Into<String>) -> Exc {
        self.err("TypeError", msg)
    }
    pub fn value_error(&mut self, msg: impl Into<String>) -> Exc {
        self.err("ValueError", msg)
    }
    pub fn index_error(&mut self, msg: impl Into<String>) -> Exc {
        self.err("IndexError", msg)
    }
    pub fn key_error(&mut self, key: Value) -> Exc {
        let cls = self.exc_class("KeyError");
        Exc { value: self.new_exc(&cls, vec![key]) }
    }
    pub fn attr_error(&mut self, msg: impl Into<String>) -> Exc {
        self.err("AttributeError", msg)
    }
    pub fn stop_iteration(&mut self, v: Value) -> Exc {
        let cls = self.exc_class("StopIteration");
        let args = if v.is_none() { vec![] } else { vec![v] };
        Exc { value: self.new_exc(&cls, args) }
    }

    pub fn exc_is(&self, e: &Exc, name: &str) -> bool {
        match &e.value {
            Value::Instance(i) => {
                let c = i.class.borrow().clone();
                match self.exc.get(name) {
                    Some(k) => c.is_subclass(k),
                    None => false,
                }
            }
            _ => false,
        }
    }

    pub fn exc_args(&self, v: &Value) -> Vec<Value> {
        if let Value::Instance(i) = v {
            if let Value::Tuple(t) = &*i.base.borrow() {
                return t.to_vec();
            }
        }
        vec![]
    }

    // ------------------------------------------------------------------
    // running code

    pub fn run_module_code(&mut self, code: Rc<Code>, globals: Rc<RefCell<Dict>>) -> PyResult<Value> {
        let n = code.varnames.len();
        let mut stack = Vec::with_capacity(16 + n);
        stack.resize(n, Value::Undef);
        let mut f = Frame {
            code: code.clone(),
            ip: 0,
            stack,
            cells: code.cellvars.iter().map(|_| Rc::new(RefCell::new(Value::Undef))).collect(),
            blocks: vec![],
            globals: globals.clone(),
            ns: Some(globals),
            reraise_line: None,
            ext: None,
        };
        match self.run_frame(&mut f)? {
            Exit::Return(v) | Exit::Yield(v) => Ok(v),
        }
    }

    /// code with separate globals and a locals namespace (eval/exec)
    pub fn run_code_ns(&mut self, code: Rc<Code>, globals: Rc<RefCell<Dict>>, ns: Rc<RefCell<Dict>>) -> PyResult<Value> {
        self.run_code_ext(code, globals, ns, None)
    }

    pub fn run_code_ext(&mut self, code: Rc<Code>, globals: Rc<RefCell<Dict>>, ns: Rc<RefCell<Dict>>, ext: Option<Rc<FrameExt>>) -> PyResult<Value> {
        self.run_code_closure(code, globals, ns, ext, vec![])
    }

    /// runs `code` with `closure` as the cells of its free variables (exec's `closure=`)
    pub fn run_code_closure(&mut self, code: Rc<Code>, globals: Rc<RefCell<Dict>>, ns: Rc<RefCell<Dict>>, ext: Option<Rc<FrameExt>>, closure: Vec<Cellv>) -> PyResult<Value> {
        let n = code.varnames.len();
        let mut stack = Vec::with_capacity(16 + n);
        stack.resize(n, Value::Undef);
        let mut cells: Vec<Cellv> = code.cellvars.iter().map(|_| Rc::new(RefCell::new(Value::Undef))).collect();
        cells.extend(closure);
        let mut f = Frame {
            code: code.clone(),
            ip: 0,
            stack,
            cells,
            blocks: vec![],
            globals,
            ns: Some(ns),
            reraise_line: None,
            ext,
        };
        // code compiled with PyCF_ALLOW_TOP_LEVEL_AWAIT gives a coroutine to await
        if code.coroutine && !code.generator {
            let g = Generator { frame: Some(Box::new(f)), started: false, running: false, name: code.name.clone(), qualname: code.qualname.clone(), coroutine: true, handled: vec![], code: code.clone(), frame_obj: None };
            return Ok(crate::gc::new_gen(g));
        }
        match self.run_frame(&mut f)? {
            Exit::Return(v) | Exit::Yield(v) => Ok(v),
        }
    }

    pub fn run_frame(&mut self, f: &mut Frame) -> PyResult<Exit> {
        self.eval_breaker()?;
        if crate::value::HAS_PENDING_DEL.with(|h| h.get()) {
            self.run_finalizers();
        }
        self.frames.push(f as *mut Frame);
        self.frame_objs.push(Value::Undef);
        // the signals that came run their handlers as a function starts
        let mut first = if f.ip == 0 && crate::posix::signals_pending() { crate::posix::check_signals(self).err() } else { None };
        let r = loop {
            let step = match first.take() {
                Some(e) => Err(e),
                None => self.exec(f),
            };
            match step {
                Ok(x) => break Ok(x),
                Err(e) => {
                    if let Err(e) = self.handle_exc(f, e) {
                        break Err(e);
                    }
                }
            }
        };
        // a frame object (a traceback's, f_locals) keeps the locals the frame left
        if let Some(Value::Obj(o)) = self.frame_objs.last() {
            if let ObjKind::Frame { locals: l, .. } = &mut *o.kind.borrow_mut() {
                if f.ns.is_none() {
                    *l = Value::Dict(crate::builtins::frame_locals(f));
                }
            }
        }
        self.frames.pop();
        self.frame_objs.pop();
        r
    }

    /// runs the finalizers that a store or delete just made due, so that
    /// __del__ follows the last reference going as closely as in CPython
    #[inline]
    pub fn finalize_pending(&mut self) {
        if crate::value::HAS_PENDING_DEL.with(|h| h.get()) {
            self.run_finalizers();
        }
    }

    /// runs __del__ of the instances whose last reference went away
    pub fn run_finalizers(&mut self) {
        crate::value::HAS_PENDING_DEL.with(|h| h.set(false));
        if crate::gc::due() {
            crate::gc::collect(self);
        }
        loop {
            let v = crate::value::PENDING_DEL.with(|p| p.borrow_mut().pop());
            let v = match v {
                Some(v) => v,
                None => break,
            };
            if let Value::Gen(_) = &v {
                // a lost generator is closed
                let r = self.getattr(&v, &ps("close")).and_then(|c| self.call_vec(&c, vec![], &[]));
                if let Err(e) = r {
                    self.unraisable(e, &v);
                }
                continue;
            }
            if let Value::Obj(o) = &v {
                // an open file nothing refers to any more
                crate::modules::file_finalize(self, o);
                continue;
            }
            let name = Rc::new(PyStr::new("__del__".to_string()));
            if let Some(f) = self.lookup_special(&v, &name) {
                if let Err(e) = self.call_vec(&f, vec![], &[]) {
                    // the report names the function, as CPython's does
                    let raw = self.class_of(&v).lookup(&name).unwrap_or(f);
                    self.unraisable(e, &raw);
                }
            }
        }
    }

    /// What CPython does to __main__ when the interpreter exits: the names
    /// that start with an underscore are set to None, then the others but
    /// __builtins__, and the objects that lose their last reference are
    /// finalized.
    pub fn finalize(&mut self) {
        self.run_finalizers();
        let g = self.main_globals.clone();
        for underscore in [true, false] {
            let keys = g.borrow().keys();
            for k in keys {
                let Value::Str(s) = &k else { continue };
                if underscore != (s.s.starts_with('_') && !s.s.starts_with("__")) || &*s.s == "__builtins__" {
                    continue;
                }
                g.borrow_mut().set_str(s.clone(), Value::None);
                self.run_finalizers();
            }
        }
    }

    /// reports an exception nothing can catch through sys.unraisablehook
    pub fn unraisable(&mut self, e: Exc, obj: &Value) {
        self.unraisable_with(e, obj, None)
    }

    /// unraisable() with the hook's err_msg, which says what was running
    pub fn unraisable_msg(&mut self, e: Exc, obj: &Value, msg: &str) {
        self.unraisable_with(e, obj, Some(msg))
    }

    fn unraisable_with(&mut self, e: Exc, obj: &Value, msg: Option<&str>) {
        let hook = match self.modules.borrow().get_s("sys") {
            Some(Value::Module(m)) => m.dict.borrow().get_s("unraisablehook"),
            _ => None,
        };
        let tb = self.getattr(&e.value, &self.n.tb.clone()).unwrap_or(Value::None);
        let mut d = Dict::new();
        d.set_s("exc_type", Value::Class(self.class_of(&e.value)));
        d.set_s("exc_value", e.value.clone());
        d.set_s("exc_traceback", tb);
        d.set_s("err_msg", msg.map_or(Value::None, Value::str));
        d.set_s("object", obj.clone());
        let ns = self.namespace_class();
        let args = Rc::new(Instance::with_dict(ns, d, Value::None));
        crate::gc::track_inst(&args);
        let args = Value::Instance(args);
        let r = match hook {
            Some(h) => self.call_vec(&h, vec![args], &[]).map(|_| ()).map_err(|_| ()),
            None => Err(()),
        };
        if r.is_err() {
            let fr = self.repr(obj).unwrap_or_default();
            let tn = self.type_name(&e.value);
            let msg = self.str_of(&e.value).unwrap_or_default();
            eprintln!("Exception ignored in: {}\n{}: {}", fr, tn, msg);
        }
    }

    /// the innermost running frame
    /// `m[name]`, None on KeyError
    pub fn map_get(&mut self, m: &Value, name: &Rc<PyStr>) -> PyResult<Option<Value>> {
        if let Value::Dict(d) = m {
            return Ok(d.borrow().get_str(name).cloned());
        }
        match self.getitem(m, &Value::Str(name.clone())) {
            Ok(v) => Ok(Some(v)),
            Err(e) if self.exc_is(&e, "KeyError") => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// global then builtin lookup for an exec/eval frame
    fn ext_global(&mut self, f: &Frame, ext: &FrameExt, name: &Rc<PyStr>) -> PyResult<Value> {
        let v = match &ext.globals {
            Some(g) => self.map_get(g, name)?,
            None => f.globals.borrow().get_str(name).cloned(),
        };
        if let Some(v) = v {
            return Ok(v);
        }
        let v = match &ext.builtins {
            Some(b) => self.map_get(b, name)?,
            None => self.builtins.borrow().get_str(name).cloned(),
        };
        match v {
            Some(v) => Ok(v),
            None => self.unbound_name(name),
        }
    }

    /// a name no namespace holds: the builtins made on first use, or a NameError
    fn unbound_name(&mut self, name: &Rc<PyStr>) -> PyResult<Value> {
        if &*name.s == "ExceptionGroup" || &*name.s == "BaseExceptionGroup" {
            self.lazy_builtin(&name.s)
        } else {
            Err(self.name_error(name))
        }
    }

    /// a builtin as the running frame sees it (its `__builtins__` mapping);
    /// AttributeError when missing, as CPython's _PyEval_GetBuiltin
    pub fn frame_builtin(&mut self, name: &str) -> PyResult<Value> {
        let key = crate::builtins::ps(name);
        let v = match self.current_frame().and_then(|f| f.ext.as_ref()).and_then(|x| x.builtins.clone()) {
            Some(b) => self.map_get(&b, &key)?,
            // a full lookup: the key may be a str subclass or str-alike
            None => {
                let b = self.builtins.clone();
                self.dict_get(&b, &Value::Str(key))?
            }
        };
        v.ok_or_else(|| self.err("AttributeError", name))
    }

    pub fn current_frame(&self) -> Option<&Frame> {
        // SAFETY: frames are pushed by run_frame and popped before it
        // returns, so every pointer refers to a live frame further up the
        // Rust stack; natives only read through it
        self.frames.last().map(|p| unsafe { &**p })
    }

    /// finds a handler in `f` for `e`; Err when the exception leaves the frame
    fn handle_exc(&mut self, f: &mut Frame, e: Exc) -> PyResult<()> {
        // a re-raise (bare `raise`, the end of a finally or with block)
        // leaves the traceback as it is
        let reraised = f.reraise_line.is_some() || std::mem::take(&mut self.bare_reraise);
        let line = match f.reraise_line.take() {
            Some(l) => l,
            None => {
                let ip = f.ip.saturating_sub(1);
                f.code.line_at(ip).unwrap_or(0)
            }
        };
        if !reraised {
            self.tb_here(&e.value, f, line);
        } else if let Some(Value::Obj(o)) = self.frame_objs.last() {
            // the frame's f_lineno follows the re-raise
            let with_exit = matches!(f.code.ops.get(f.ip.saturating_sub(1)), Some(Op::WithExcept));
            if !with_exit && self.frames.last().map(|p| std::ptr::eq(*p as *const Frame, f)).unwrap_or(false) {
                if let ObjKind::Frame { line: l, .. } = &mut *o.kind.borrow_mut() {
                    *l = f.code.line_at(f.ip.saturating_sub(1)).unwrap_or(line);
                }
            }
        }
        // chain onto an exception being handled
        if let Some(h) = self.handled.last().cloned() {
            if !h.is(&e.value) {
                if let Value::Instance(i) = &e.value {
                    let has = i.extra.borrow().get_str(&self.n.context).is_some();
                    if !has {
                        // a cycle through __context__ is cut where it would close
                        let mut cur = h.clone();
                        let mut steps = 0;
                        while let Value::Instance(ci) = &cur {
                            let next = ci.extra.borrow().get_str(&self.n.context).cloned().unwrap_or(Value::None);
                            if next.is(&e.value) {
                                ci.extra.borrow_mut().remove_str(&self.n.context);
                                break;
                            }
                            cur = next;
                            steps += 1;
                            if steps > 10000 {
                                break;
                            }
                        }
                        i.extra.borrow_mut().set_str(self.n.context.clone(), h);
                    }
                }
            }
        }
        while let Some(b) = f.blocks.pop() {
            if b.kind == BLOCK_HANDLER {
                self.handled.pop();
                continue;
            }
            let nlocals = f.code.varnames.len();
            f.stack.truncate((b.level as usize).max(nlocals));
            f.blocks.push(Block { kind: BLOCK_HANDLER, handler: 0, level: f.stack.len() as u32, line });
            self.handled.push(e.value.clone());
            f.stack.push(e.value);
            f.ip = b.handler as usize;
            return Ok(());
        }
        Err(e)
    }

    #[inline]
    fn pop(f: &mut Frame) -> Value {
        f.stack.pop().unwrap()
    }

    fn exec(&mut self, f: &mut Frame) -> PyResult<Exit> {
        let code = f.code.clone();
        let ops = &code.ops[..];
        loop {
            let op = ops[f.ip];
            f.ip += 1;
            match op {
                Op::Nop => {}
                Op::LoadConst(i) => f.stack.push(code.consts[i as usize].clone()),
                Op::LoadFast(i) => {
                    let v = &f.stack[i as usize];
                    if v.is_undef() {
                        let n = code.varnames[i as usize].clone();
                        return Err(self.err("UnboundLocalError", format!("cannot access local variable '{}' where it is not associated with a value", n)));
                    }
                    let v = v.clone();
                    f.stack.push(v);
                }
                Op::StoreFast(i) => {
                    let v = Self::pop(f);
                    f.stack[i as usize] = v;
                    self.finalize_pending();
                }
                Op::DelFast(i) => {
                    if f.stack[i as usize].is_undef() {
                        let n = code.varnames[i as usize].clone();
                        return Err(self.err("UnboundLocalError", format!("cannot access local variable '{}' where it is not associated with a value", n)));
                    }
                    f.stack[i as usize] = Value::Undef;
                    self.finalize_pending();
                }
                Op::LoadFastAndClear(i) => {
                    let v = std::mem::replace(&mut f.stack[i as usize], Value::Undef);
                    f.stack.push(v);
                }
                Op::Swap(n) => {
                    let top = f.stack.len() - 1;
                    f.stack.swap(top, top + 1 - n as usize);
                }
                Op::MakeCell(i) => {
                    f.cells[i as usize] = Rc::new(RefCell::new(Value::Undef));
                    self.finalize_pending();
                }
                Op::LoadDeref(i) => {
                    let v = f.cells[i as usize].borrow().clone();
                    if v.is_undef() {
                        return Err(self.unbound_deref(&code, i));
                    }
                    f.stack.push(v);
                }
                Op::StoreDeref(i) => {
                    let v = Self::pop(f);
                    *f.cells[i as usize].borrow_mut() = v;
                    self.finalize_pending();
                }
                Op::DelDeref(i) => {
                    *f.cells[i as usize].borrow_mut() = Value::Undef;
                }
                Op::LoadClassDeref(i) => {
                    let name = self.cell_name(&code, i);
                    let found = f.ns.as_ref().and_then(|ns| ns.borrow().get_str(&PyStr::new(name.clone())).cloned());
                    match found {
                        Some(v) => f.stack.push(v),
                        None => {
                            let v = f.cells[i as usize].borrow().clone();
                            if v.is_undef() {
                                return Err(self.unbound_deref(&code, i));
                            }
                            f.stack.push(v);
                        }
                    }
                }
                Op::LoadGlobal(i) if f.ext.is_some() => {
                    let name = &code.names[i as usize];
                    let ext = f.ext.clone().unwrap();
                    let v = self.ext_global(f, &ext, name)?;
                    f.stack.push(v);
                }
                Op::LoadGlobal(i) => {
                    let name = &code.names[i as usize];
                    let v = f.globals.borrow().get_str(name).cloned();
                    let v = match v {
                        Some(v) => v,
                        None => {
                            let b = self.builtins.borrow().get_str(name).cloned();
                            match b {
                                Some(v) => v,
                                None => {
                                    // a key that is not a str but equals one
                                    let bd = self.builtins.clone();
                                    match self.dict_get(&bd, &Value::Str(name.clone()))? {
                                        Some(v) => v,
                                        None => self.unbound_name(name)?,
                                    }
                                }
                            }
                        }
                    };
                    f.stack.push(v);
                }
                Op::StoreGlobal(i) if f.ext.as_ref().map(|x| x.globals.is_some()).unwrap_or(false) => {
                    let v = Self::pop(f);
                    let g = f.ext.as_ref().unwrap().globals.clone().unwrap();
                    self.setitem(&g, Value::Str(code.names[i as usize].clone()), v)?;
                    self.finalize_pending();
                }
                Op::StoreGlobal(i) => {
                    let v = Self::pop(f);
                    f.globals.borrow_mut().set_str(code.names[i as usize].clone(), v);
                    self.finalize_pending();
                }
                Op::DelGlobal(i) => {
                    let name = &code.names[i as usize];
                    if f.globals.borrow_mut().remove_str(name).is_none() {
                        return Err(self.name_error(name));
                    }
                    self.finalize_pending();
                }
                Op::LoadName(i) if f.ext.is_some() => {
                    let name = &code.names[i as usize];
                    let ext = f.ext.clone().unwrap();
                    let v = match &ext.locals {
                        Some(m) => self.map_get(m, name)?,
                        None => f.ns.as_ref().and_then(|ns| ns.borrow().get_str(name).cloned()),
                    };
                    let v = match v {
                        Some(v) => v,
                        None => self.ext_global(f, &ext, name)?,
                    };
                    f.stack.push(v);
                }
                Op::LoadName(i) => {
                    let name = &code.names[i as usize];
                    let mut v = f.ns.as_ref().and_then(|ns| ns.borrow().get_str(name).cloned());
                    if v.is_none() {
                        let same = f.ns.as_ref().map(|ns| Rc::ptr_eq(ns, &f.globals)).unwrap_or(false);
                        if !same {
                            v = f.globals.borrow().get_str(name).cloned();
                        }
                    }
                    if v.is_none() {
                        v = self.builtins.borrow().get_str(name).cloned();
                    }
                    match v {
                        Some(v) => f.stack.push(v),
                        None => {
                            let v = self.unbound_name(name)?;
                            f.stack.push(v);
                        }
                    }
                }
                Op::StoreName(i) if f.ext.as_ref().map(|x| x.locals.is_some()).unwrap_or(false) => {
                    let v = Self::pop(f);
                    let m = f.ext.as_ref().unwrap().locals.clone().unwrap();
                    self.setitem(&m, Value::Str(code.names[i as usize].clone()), v)?;
                    self.finalize_pending();
                }
                Op::StoreName(i) => {
                    let v = Self::pop(f);
                    let name = code.names[i as usize].clone();
                    match &f.ns {
                        Some(ns) => ns.borrow_mut().set_str(name, v),
                        None => f.globals.borrow_mut().set_str(name, v),
                    }
                    self.finalize_pending();
                }
                Op::DelName(i) if f.ext.as_ref().map(|x| x.locals.is_some()).unwrap_or(false) => {
                    let m = f.ext.as_ref().unwrap().locals.clone().unwrap();
                    let name = &code.names[i as usize];
                    if let Err(e) = self.delitem(&m, &Value::Str(name.clone())) {
                        return Err(if self.exc_is(&e, "KeyError") { self.name_error(name) } else { e });
                    }
                    self.finalize_pending();
                }
                Op::DelName(i) => {
                    let name = &code.names[i as usize];
                    let ns = f.ns.clone().unwrap_or_else(|| f.globals.clone());
                    if ns.borrow_mut().remove_str(name).is_none() {
                        return Err(self.name_error(name));
                    }
                    self.finalize_pending();
                }
                Op::LoadAttr(i) => {
                    let obj = Self::pop(f);
                    let v = self.load_attr(&obj, &code.names[i as usize], &code.attr_hints[i as usize])?;
                    f.stack.push(v);
                }
                Op::StoreAttr(i) => {
                    let obj = Self::pop(f);
                    let v = Self::pop(f);
                    self.store_attr(&obj, &code.names[i as usize], v, &code.attr_hints[i as usize])?;
                    self.finalize_pending();
                }
                Op::DelAttr(i) => {
                    let obj = Self::pop(f);
                    self.delattr(&obj, &code.names[i as usize])?;
                    self.finalize_pending();
                }
                Op::LoadMethod(i) => {
                    let obj = Self::pop(f);
                    let (m, s) = self.load_method(obj, &code.names[i as usize])?;
                    f.stack.push(m);
                    f.stack.push(s);
                }
                Op::CallMethod(n) => {
                    let n = n as usize;
                    let len = f.stack.len();
                    let base = len - n - 2;
                    let first = if f.stack[base + 1].is_undef() { base + 2 } else { base + 1 };
                    let func = f.stack[base].clone();
                    let r = match &func {
                        Value::Native(nf) => {
                            let r = self.call_native(nf, &f.stack[first..]);
                            f.stack.truncate(base);
                            r?
                        }
                        Value::Func(func) if func.code.borrow().simple && func.code.borrow().argcount as usize == len - first => {
                            let mut args = self.stack_pool.pop().unwrap_or_default();
                            args.extend(f.stack.drain(first..));
                            f.stack.truncate(base);
                            self.call_simple(func, args)?
                        }
                        _ => {
                            let args: Vec<Value> = f.stack.drain(first..).collect();
                            f.stack.truncate(base);
                            self.call_vec(&func, args, &[])?
                        }
                    };
                    f.stack.push(r);
                }
                Op::Subscr => {
                    let idx = Self::pop(f);
                    let obj = Self::pop(f);
                    let v = match (&obj, &idx) {
                        (Value::List(l), Value::Int(i)) => {
                            let l = l.borrow();
                            let n = l.len() as i64;
                            let j = if *i < 0 { *i + n } else { *i };
                            if j < 0 || j >= n {
                                drop(l);
                                return Err(self.index_error("list index out of range"));
                            }
                            l[j as usize].clone()
                        }
                        _ => self.getitem(&obj, &idx)?,
                    };
                    f.stack.push(v);
                }
                Op::StoreSubscr => {
                    let idx = Self::pop(f);
                    let obj = Self::pop(f);
                    let v = Self::pop(f);
                    if let (Value::List(l), Value::Int(i)) = (&obj, &idx) {
                        crate::gc::list_store(l, &v);
                        let mut lb = l.borrow_mut();
                        let n = lb.len() as i64;
                        let j = if *i < 0 { *i + n } else { *i };
                        if j >= 0 && j < n {
                            lb[j as usize] = v;
                            drop(lb);
                            self.finalize_pending();
                            continue;
                        }
                        drop(lb);
                        return Err(self.index_error("list assignment index out of range"));
                    }
                    self.setitem(&obj, idx, v)?;
                    self.finalize_pending();
                }
                Op::DelSubscr => {
                    let idx = Self::pop(f);
                    let obj = Self::pop(f);
                    self.delitem(&obj, &idx)?;
                    self.finalize_pending();
                }
                Op::BuildSlice(n) => {
                    let step = if n == 3 { Self::pop(f) } else { Value::None };
                    let hi = Self::pop(f);
                    let lo = Self::pop(f);
                    f.stack.push(Value::Slice(Rc::new((lo, hi, step))));
                }
                Op::Binary(op) => {
                    let b = Self::pop(f);
                    let a = Self::pop(f);
                    let r = match (&a, &b) {
                        (Value::Int(x), Value::Int(y)) => match op {
                            BinOp::Add => match x.checked_add(*y) {
                                Some(v) => Value::Int(v),
                                None => self.binop(op, &a, &b)?,
                            },
                            BinOp::Sub => match x.checked_sub(*y) {
                                Some(v) => Value::Int(v),
                                None => self.binop(op, &a, &b)?,
                            },
                            BinOp::Mul => match x.checked_mul(*y) {
                                Some(v) => Value::Int(v),
                                None => self.binop(op, &a, &b)?,
                            },
                            _ => self.binop(op, &a, &b)?,
                        },
                        (Value::Float(x), Value::Float(y)) => match op {
                            BinOp::Add => Value::Float(x + y),
                            BinOp::Sub => Value::Float(x - y),
                            BinOp::Mul => Value::Float(x * y),
                            _ => self.binop(op, &a, &b)?,
                        },
                        _ => self.binop(op, &a, &b)?,
                    };
                    f.stack.push(r);
                }
                Op::Inplace(op) => {
                    let b = Self::pop(f);
                    let a = Self::pop(f);
                    let r = match (&a, &b) {
                        (Value::Int(x), Value::Int(y)) => match op {
                            BinOp::Add => match x.checked_add(*y) {
                                Some(v) => Value::Int(v),
                                None => self.binop(op, &a, &b)?,
                            },
                            BinOp::Sub => match x.checked_sub(*y) {
                                Some(v) => Value::Int(v),
                                None => self.binop(op, &a, &b)?,
                            },
                            _ => self.binop(op, &a, &b)?,
                        },
                        (Value::Float(x), Value::Float(y)) => match op {
                            BinOp::Add => Value::Float(x + y),
                            BinOp::Sub => Value::Float(x - y),
                            BinOp::Mul => Value::Float(x * y),
                            _ => self.binop(op, &a, &b)?,
                        },
                        _ => self.inplace(op, &a, &b)?,
                    };
                    f.stack.push(r);
                }
                Op::Unary(op) => {
                    let a = Self::pop(f);
                    let r = self.unary(op, &a)?;
                    f.stack.push(r);
                }
                Op::Compare(op) => {
                    let b = Self::pop(f);
                    let a = Self::pop(f);
                    let r = match (&a, &b) {
                        (Value::Int(x), Value::Int(y)) => Value::Bool(match op {
                            CmpOp::Lt => x < y,
                            CmpOp::Le => x <= y,
                            CmpOp::Gt => x > y,
                            CmpOp::Ge => x >= y,
                            CmpOp::Eq => x == y,
                            CmpOp::Ne => x != y,
                            _ => {
                                let r = self.compare(op, &a, &b)?;
                                f.stack.push(r);
                                continue;
                            }
                        }),
                        (Value::Float(x), Value::Float(y)) => Value::Bool(match op {
                            CmpOp::Lt => x < y,
                            CmpOp::Le => x <= y,
                            CmpOp::Gt => x > y,
                            CmpOp::Ge => x >= y,
                            CmpOp::Eq => x == y,
                            CmpOp::Ne => x != y,
                            _ => {
                                let r = self.compare(op, &a, &b)?;
                                f.stack.push(r);
                                continue;
                            }
                        }),
                        _ => self.compare(op, &a, &b)?,
                    };
                    f.stack.push(r);
                }
                Op::Pop => {
                    f.stack.pop();
                    self.finalize_pending();
                }
                Op::Dup => {
                    let v = f.stack.last().unwrap().clone();
                    f.stack.push(v);
                }
                Op::Dup2 => {
                    let n = f.stack.len();
                    let a = f.stack[n - 2].clone();
                    let b = f.stack[n - 1].clone();
                    f.stack.push(a);
                    f.stack.push(b);
                }
                Op::Rot2 => {
                    let n = f.stack.len();
                    f.stack.swap(n - 1, n - 2);
                }
                Op::Rot3 => {
                    let v = Self::pop(f);
                    let n = f.stack.len();
                    f.stack.insert(n - 2, v);
                }
                Op::Rot4 => {
                    let v = Self::pop(f);
                    let n = f.stack.len();
                    f.stack.insert(n - 3, v);
                }
                Op::Jump(t) => {
                    // a loop's way back: where other threads get their turn,
                    // and signal handlers run
                    if (t as usize) < f.ip {
                        self.eval_breaker()?;
                        if crate::posix::signals_pending() {
                            crate::posix::check_signals(self)?;
                        }
                    }
                    f.ip = t as usize
                }
                Op::PopJumpIfFalse(t) => {
                    let v = Self::pop(f);
                    let b = match v {
                        Value::Bool(b) => b,
                        _ => self.truthy(&v)?,
                    };
                    if !b {
                        f.ip = t as usize;
                    }
                }
                Op::PopJumpIfTrue(t) => {
                    let v = Self::pop(f);
                    let b = match v {
                        Value::Bool(b) => b,
                        _ => self.truthy(&v)?,
                    };
                    if b {
                        f.ip = t as usize;
                    }
                }
                Op::JumpIfFalseOrPop(t) => {
                    let v = f.stack.last().unwrap().clone();
                    if !self.truthy(&v)? {
                        f.ip = t as usize;
                    } else {
                        f.stack.pop();
                    }
                }
                Op::JumpIfTrueOrPop(t) => {
                    let v = f.stack.last().unwrap().clone();
                    if self.truthy(&v)? {
                        f.ip = t as usize;
                    } else {
                        f.stack.pop();
                    }
                }
                Op::PopJumpIfNotNone(t) => {
                    let v = Self::pop(f);
                    if !v.is_none() {
                        f.ip = t as usize;
                    }
                }
                Op::BuildList(n) => {
                    let at = f.stack.len() - n as usize;
                    let items: Vec<Value> = f.stack.drain(at..).collect();
                    f.stack.push(Value::list(items));
                }
                Op::BuildTuple(n) => {
                    let at = f.stack.len() - n as usize;
                    let items: Vec<Value> = f.stack.drain(at..).collect();
                    f.stack.push(Value::tuple(items));
                }
                Op::BuildSet(n) => {
                    let at = f.stack.len() - n as usize;
                    let items: Vec<Value> = f.stack.drain(at..).collect();
                    let s = Rc::new(RefCell::new(Dict::new()));
                    for it in items {
                        self.set_add(&s, it)?;
                    }
                    f.stack.push(Value::Set(s));
                }
                Op::BuildMap(n) => {
                    let at = f.stack.len() - 2 * n as usize;
                    let items: Vec<Value> = f.stack.drain(at..).collect();
                    let d = Rc::new(RefCell::new(Dict::with_capacity(n as usize)));
                    let mut it = items.into_iter();
                    while let (Some(k), Some(v)) = (it.next(), it.next()) {
                        self.dict_set(&d, k, v)?;
                    }
                    f.stack.push(Value::Dict(d));
                }
                Op::BuildString(n) => {
                    let at = f.stack.len() - n as usize;
                    let size = f.stack[at..].iter().map(|v| if let Value::Str(p) = v { p.s.len() } else { 0 }).sum();
                    let mut s = crate::wtf8::Wtf8Buf::from(String::with_capacity(size));
                    for v in f.stack.drain(at..) {
                        if let Value::Str(p) = v {
                            s.push_pystr(&p);
                        }
                    }
                    f.stack.push(s.into_value());
                }
                Op::ListAppend(d) => {
                    let v = Self::pop(f);
                    let n = f.stack.len();
                    if let Value::List(l) = &f.stack[n - d as usize] {
                        l.borrow_mut().push(v);
                    }
                }
                Op::SetAdd(d) => {
                    let v = Self::pop(f);
                    let n = f.stack.len();
                    let s = match &f.stack[n - d as usize] {
                        Value::Set(s) => s.clone(),
                        _ => unreachable!(),
                    };
                    self.set_add(&s, v)?;
                }
                Op::MapAdd(d) => {
                    let v = Self::pop(f);
                    let k = Self::pop(f);
                    let n = f.stack.len();
                    let m = match &f.stack[n - d as usize] {
                        Value::Dict(m) => m.clone(),
                        _ => unreachable!(),
                    };
                    self.dict_set(&m, k, v)?;
                }
                Op::ListExtend(d) => {
                    let v = Self::pop(f);
                    let items = self.to_vec(&v)?;
                    let n = f.stack.len();
                    if let Value::List(l) = &f.stack[n - d as usize] {
                        l.borrow_mut().extend(items);
                    }
                }
                Op::SetUpdate(d) => {
                    let v = Self::pop(f);
                    let items = self.to_vec(&v)?;
                    let n = f.stack.len();
                    let s = match &f.stack[n - d as usize] {
                        Value::Set(s) => s.clone(),
                        _ => unreachable!(),
                    };
                    for it in items {
                        self.set_add(&s, it)?;
                    }
                }
                Op::DictUpdate(d) | Op::DictMerge(d) => {
                    let v = Self::pop(f);
                    let n = f.stack.len();
                    let m = match &f.stack[n - d as usize] {
                        Value::Dict(m) => m.clone(),
                        _ => unreachable!(),
                    };
                    let pairs = self.mapping_items(&v).map_err(|e| e)?;
                    for (k, val) in pairs {
                        if matches!(op, Op::DictMerge(_)) {
                            if self.dict_get(&m, &k)?.is_some() {
                                let ks = self.str_of(&k)?;
                                return Err(self.type_error(format!("got multiple values for keyword argument '{}'", ks)));
                            }
                        }
                        self.dict_set(&m, k, val)?;
                    }
                }
                Op::ListToTuple => {
                    let v = Self::pop(f);
                    if let Value::List(l) = v {
                        let items = std::mem::take(&mut *l.borrow_mut());
                        f.stack.push(Value::tuple(items));
                    }
                }
                Op::FormatValue(conv, has_spec) => {
                    let spec = if has_spec { Self::pop(f) } else { Value::str("") };
                    let v = Self::pop(f);
                    let v = match conv {
                        1 if matches!(v, Value::Str(_)) => v,
                        1 => Value::string(self.str_of(&v)?),
                        2 => Value::string(self.repr(&v)?),
                        3 => Value::string(crate::format::ascii_escape(&self.repr(&v)?)),
                        _ => v,
                    };
                    let spec_s = match &spec {
                        Value::Str(s) => s.s.clone(),
                        _ => String::new(),
                    };
                    let r = match &v {
                        Value::Str(_) if spec_s.is_empty() => v,
                        _ => self.format_to_str(&v, &spec_s)?,
                    };
                    f.stack.push(r);
                }
                Op::GetIter => {
                    let v = Self::pop(f);
                    let it = self.iter(&v)?;
                    f.stack.push(it);
                }
                Op::GetAwaitable => {
                    let v = Self::pop(f);
                    let it = match &v {
                        Value::Gen(g) => {
                            let (coro, iterable, agen) = {
                                let g = g.borrow();
                                (g.coroutine, g.code.iterable_coroutine, g.code.coroutine)
                            };
                            if !coro && !iterable {
                                let msg = if agen { "'async_generator' object can't be awaited" } else { "object generator can't be used in 'await' expression" };
                                return Err(self.type_error(msg));
                            }
                            v
                        }
                        _ => match self.lookup_special(&v, &ps("__await__")) {
                            Some(m) => {
                                let r = self.call_vec(&m, vec![], &[])?;
                                if !matches!(r, Value::Gen(_) | Value::Iter(_)) && self.lookup_special(&r, &self.n.next.clone()).is_none() {
                                    let tn = self.type_name(&r);
                                    return Err(self.type_error(format!("__await__() returned non-iterator of type '{}'", tn)));
                                }
                                r
                            }
                            None => {
                                let tn = self.type_name(&v);
                                return Err(self.type_error(format!("'{}' object can't be awaited", tn)));
                            }
                        },
                    };
                    f.stack.push(it);
                }
                Op::ForIter(t) => {
                    let it = f.stack.last().unwrap().clone();
                    // fast paths for the common iterators
                    let next = match &it {
                        Value::Iter(r) => {
                            let mut b = r.borrow_mut();
                            match &mut *b {
                                Iter::Range(cur, left, step) => {
                                    if *left > 0 {
                                        let v = *cur;
                                        *left -= 1;
                                        *cur = cur.wrapping_add(*step);
                                        Some(Some(Value::Int(v)))
                                    } else {
                                        Some(None)
                                    }
                                }
                                Iter::List(l, i) => {
                                    let lb = l.borrow();
                                    if *i < lb.len() {
                                        let v = lb[*i].clone();
                                        *i += 1;
                                        Some(Some(v))
                                    } else {
                                        *i = usize::MAX / 2;
                                        Some(None)
                                    }
                                }
                                Iter::Tuple(l, i) => {
                                    if *i < l.len() {
                                        let v = l[*i].clone();
                                        *i += 1;
                                        Some(Some(v))
                                    } else {
                                        Some(None)
                                    }
                                }
                                _ => None,
                            }
                        }
                        _ => None,
                    };
                    let next = match next {
                        Some(n) => n,
                        None => self.next(&it)?,
                    };
                    match next {
                        Some(v) => f.stack.push(v),
                        None => {
                            f.stack.pop();
                            f.ip = t as usize;
                        }
                    }
                }
                Op::Unpack(n) => {
                    let v = Self::pop(f);
                    let n = n as usize;
                    match &v {
                        Value::Tuple(t) if t.len() == n => {
                            for x in t.iter().rev() {
                                f.stack.push(x.clone());
                            }
                        }
                        Value::List(l) if l.borrow().len() == n => {
                            for x in l.borrow().iter().rev() {
                                f.stack.push(x.clone());
                            }
                        }
                        _ => {
                            let items = self.unpack_n(&v, n)?;
                            for x in items.into_iter().rev() {
                                f.stack.push(x);
                            }
                        }
                    }
                }
                Op::UnpackEx(before, after) => {
                    let v = Self::pop(f);
                    let items = self.to_vec(&v)?;
                    let (b, a) = (before as usize, after as usize);
                    if items.len() < b + a {
                        return Err(self.value_error(format!("not enough values to unpack (expected at least {}, got {})", b + a, items.len())));
                    }
                    let mid: Vec<Value> = items[b..items.len() - a].to_vec();
                    for x in items[items.len() - a..].iter().rev() {
                        f.stack.push(x.clone());
                    }
                    f.stack.push(Value::list(mid));
                    for x in items[..b].iter().rev() {
                        f.stack.push(x.clone());
                    }
                }
                Op::Call(n) => {
                    let n = n as usize;
                    let base = f.stack.len() - n - 1;
                    let func = f.stack[base].clone();
                    let r = if let Value::Func(func) = &func {
                        if func.code.borrow().simple && func.code.borrow().argcount as usize == n {
                            let mut args = self.stack_pool.pop().unwrap_or_default();
                            args.extend(f.stack.drain(base + 1..));
                            f.stack.pop();
                            self.call_simple(func, args)?
                        } else {
                            let args: Vec<Value> = f.stack.drain(base + 1..).collect();
                            f.stack.pop();
                            self.call_func(func, args, &[])?
                        }
                    } else if let Value::Native(nf) = &func {
                        let r = self.call_native(nf, &f.stack[base + 1..]);
                        f.stack.truncate(base);
                        r?
                    } else {
                        let args: Vec<Value> = f.stack.drain(base + 1..).collect();
                        f.stack.pop();
                        self.call_vec(&func, args, &[])?
                    };
                    f.stack.push(r);
                }
                Op::CallKw(n) => {
                    let names = Self::pop(f);
                    let names = match names {
                        Value::Tuple(t) => t,
                        _ => unreachable!(),
                    };
                    let n = n as usize;
                    let base = f.stack.len() - n - 1;
                    let mut args: Vec<Value> = f.stack.drain(base + 1..).collect();
                    let func = Self::pop(f);
                    let nkw = names.len();
                    let kwvals = args.split_off(n - nkw);
                    let kw: Vec<(Rc<PyStr>, Value)> = names
                        .iter()
                        .zip(kwvals)
                        .map(|(k, v)| match k {
                            Value::Str(s) => (s.clone(), v),
                            _ => unreachable!(),
                        })
                        .collect();
                    let r = self.call_vec(&func, args, &kw)?;
                    f.stack.push(r);
                }
                Op::CallEx => {
                    let kwargs = Self::pop(f);
                    let args = Self::pop(f);
                    let func = Self::pop(f);
                    let args = match args {
                        Value::Tuple(t) => t.to_vec(),
                        other => self.to_vec(&other)?,
                    };
                    let kw = match kwargs {
                        Value::Dict(d) => {
                            let mut kw = vec![];
                            for e in d.borrow().iter() {
                                match &e.key {
                                    Value::Str(s) => kw.push((s.clone(), e.value.clone())),
                                    _ => {
                                        return Err(self.type_error("keywords must be strings"));
                                    }
                                }
                            }
                            kw
                        }
                        _ => vec![],
                    };
                    let r = self.call_vec(&func, args, &kw)?;
                    f.stack.push(r);
                }
                Op::MakeFunction(k, flags) => {
                    let closure = if flags & 8 != 0 {
                        match Self::pop(f) {
                            Value::Tuple(t) => t
                                .iter()
                                .map(|c| match c {
                                    Value::Cell(c) => c.clone(),
                                    _ => unreachable!(),
                                })
                                .collect(),
                            _ => unreachable!(),
                        }
                    } else {
                        vec![]
                    };
                    let annotations = if flags & 4 != 0 { Self::pop(f) } else { Value::Undef };
                    let kwdefaults = if flags & 2 != 0 {
                        match Self::pop(f) {
                            Value::Dict(d) => d.borrow().iter().map(|e| (Rc::from(e.key.as_str().unwrap()), e.value.clone())).collect(),
                            _ => vec![],
                        }
                    } else {
                        vec![]
                    };
                    let defaults = if flags & 1 != 0 {
                        match Self::pop(f) {
                            Value::Tuple(t) => t.to_vec(),
                            _ => vec![],
                        }
                    } else {
                        vec![]
                    };
                    let c = match &code.consts[k as usize] {
                        Value::Code(c) => c.clone(),
                        _ => unreachable!(),
                    };
                    let func = self.make_function(c, f.globals.clone(), defaults, kwdefaults, closure);
                    if let (Value::Func(fu), false) = (&func, annotations.is_undef()) {
                        *fu.annotations.borrow_mut() = annotations;
                    }
                    f.stack.push(func);
                }
                Op::LoadClosure(i) => f.stack.push(Value::Cell(f.cells[i as usize].clone())),
                Op::Return => {
                    let v = Self::pop(f);
                    return Ok(Exit::Return(v));
                }
                Op::Yield => {
                    let v = Self::pop(f);
                    return Ok(Exit::Yield(v));
                }
                Op::YieldFrom => {
                    let sent = Self::pop(f);
                    let recv = f.stack.last().unwrap().clone();
                    match self.send(&recv, sent)? {
                        Some(v) => {
                            f.ip -= 1;
                            return Ok(Exit::Yield(v));
                        }
                        None => {
                            f.stack.pop();
                            let rv = self.last_return.take().unwrap_or(Value::None);
                            f.stack.push(rv);
                        }
                    }
                }
                Op::SetupExcept(h) => f.blocks.push(Block { kind: BLOCK_EXCEPT, handler: h, level: f.stack.len() as u32, line: 0 }),
                Op::SetupFinally(h) => f.blocks.push(Block { kind: BLOCK_FINALLY, handler: h, level: f.stack.len() as u32, line: 0 }),
                Op::SetupWith(h) => {
                    let mgr = Self::pop(f);
                    let enter = match self.lookup_special(&mgr, &self.n.enter.clone()) {
                        Some(e) => e,
                        None => {
                            let tn = self.type_name(&mgr);
                            return Err(self.type_error(format!("'{}' object does not support the context manager protocol", tn)));
                        }
                    };
                    let exit = match self.lookup_special(&mgr, &self.n.exit.clone()) {
                        Some(e) => e,
                        None => {
                            let tn = self.type_name(&mgr);
                            return Err(self.type_error(format!("'{}' object does not support the context manager protocol (missed __exit__ method)", tn)));
                        }
                    };
                    f.stack.push(exit);
                    let r = self.call_vec(&enter, vec![], &[])?;
                    f.blocks.push(Block { kind: BLOCK_WITH, handler: h, level: f.stack.len() as u32, line: 0 });
                    f.stack.push(r);
                }
                Op::PopBlock => {
                    f.blocks.pop();
                }
                Op::PopExcept => {
                    if let Some(b) = f.blocks.pop() {
                        if b.kind == BLOCK_HANDLER {
                            self.handled.pop();
                        }
                    }
                    self.finalize_pending();
                }
                Op::Reraise => {
                    let exc = Self::pop(f);
                    let line = f.blocks.last().map(|b| b.line);
                    f.reraise_line = line;
                    return Err(Exc { value: exc });
                }
                Op::Raise(n) => {
                    return Err(self.do_raise(f, n)?);
                }
                Op::ExcMatch => {
                    let t = Self::pop(f);
                    let e = Self::pop(f);
                    let r = self.exc_matches(&e, &t)?;
                    f.stack.push(Value::Bool(r));
                }
                Op::WithExcept => {
                    let exc = Self::pop(f);
                    let exit = Self::pop(f);
                    let typ = Value::Class(self.class_of(&exc));
                    let tb = self.getattr(&exc, &self.n.tb.clone()).unwrap_or(Value::None);
                    let r = self.call_vec(&exit, vec![typ, exc.clone(), tb], &[])?;
                    if self.truthy(&r)? {
                        // suppressed: leave the handler
                        if let Some(b) = f.blocks.pop() {
                            if b.kind == BLOCK_HANDLER {
                                self.handled.pop();
                            }
                        }
                    } else {
                        f.stack.push(exit);
                        f.stack.push(exc.clone());
                        f.stack.pop();
                        f.stack.pop();
                        f.reraise_line = f.blocks.last().map(|b| b.line);
                        return Err(Exc { value: exc });
                    }
                }
                Op::WithExit(d) => {
                    let n = f.stack.len();
                    let exit = f.stack.remove(n - 1 - d as usize);
                    self.call_vec(&exit, vec![Value::None, Value::None, Value::None], &[])?;
                }
                Op::ImportName(i) => {
                    let fromlist = Self::pop(f);
                    let level = Self::pop(f);
                    let level = match level {
                        Value::Int(l) => l,
                        _ => 0,
                    };
                    let name = code.names[i as usize].s.clone();
                    if let Some(b) = f.ext.as_ref().and_then(|x| x.builtins.clone()) {
                        let imp = match self.map_get(&b, &crate::builtins::ps("__import__"))? {
                            Some(v) => v,
                            None => {
                                return Err(self.err("ImportError", "__import__ not found"));
                            }
                        };
                        let g = f.ext.as_ref().unwrap().globals.clone().unwrap_or_else(|| Value::Dict(f.globals.clone()));
                        let l = match (&f.ext.as_ref().unwrap().locals, &f.ns) {
                            (Some(l), _) => l.clone(),
                            (None, Some(ns)) => Value::Dict(ns.clone()),
                            _ => Value::None,
                        };
                        let m = self.call_vec(&imp, vec![Value::Str(crate::builtins::ps(&name)), g, l, fromlist, Value::Int(level)], &[])?;
                        f.stack.push(m);
                        continue;
                    }
                    let m = self.import(&name, &fromlist, level, &f.globals)?;
                    f.stack.push(m);
                }
                Op::ImportFrom(i) => {
                    let m = f.stack.last().unwrap().clone();
                    let name = code.names[i as usize].clone();
                    let v = self.import_from(&m, &name)?;
                    f.stack.push(v);
                }
                Op::ImportStar => {
                    let m = Self::pop(f);
                    self.import_star(&m, f)?;
                }
                Op::LoadBuildClass => {
                    let v = match f.ext.as_ref().and_then(|x| x.builtins.clone()) {
                        Some(b) => match self.map_get(&b, &crate::builtins::ps("__build_class__"))? {
                            Some(v) => v,
                            None => {
                                return Err(self.err("NameError", "__build_class__ not found"));
                            }
                        },
                        None => self.builtins.borrow().get_s("__build_class__").unwrap(),
                    };
                    f.stack.push(v);
                }
                Op::SetupAnnotations if f.ext.as_ref().is_some_and(|x| x.locals.is_some()) => {
                    let m = f.ext.as_ref().unwrap().locals.clone().unwrap();
                    if self.map_get(&m, &self.n.annotations.clone())?.is_none() {
                        self.setitem(&m, Value::Str(self.n.annotations.clone()), Value::dict(Dict::new()))?;
                    }
                }
                Op::SetupAnnotations => {
                    let ns = f.ns.clone().unwrap_or_else(|| f.globals.clone());
                    let has = ns.borrow().get_str(&self.n.annotations).is_some();
                    if !has {
                        ns.borrow_mut().set_str(self.n.annotations.clone(), Value::dict(Dict::new()));
                    }
                }
                Op::LoadAssertionError => {
                    let c = self.exc_class("AssertionError");
                    f.stack.push(Value::Class(c));
                }
                Op::MatchPattern(k) => {
                    let vals = Self::pop(f);
                    let subject = Self::pop(f);
                    let pat = match &code.consts[k as usize] {
                        Value::Obj(o) => match &*o.kind.borrow() {
                            ObjKind::MatchPat(p) => p.clone(),
                            _ => unreachable!(),
                        },
                        _ => unreachable!(),
                    };
                    let vals = match vals {
                        Value::Tuple(t) => t,
                        _ => unreachable!(),
                    };
                    let r = crate::matching::match_pattern(self, &pat, &subject, &vals)?;
                    f.stack.push(r);
                }
                Op::ExcStarMatch => {
                    let typ = Self::pop(f);
                    let rest = Self::pop(f);
                    let (m, r) = self.exc_star_split(&rest, &typ)?;
                    f.stack.push(r);
                    f.stack.push(m);
                }
                Op::ExcStarCollect => {
                    let exc = Self::pop(f);
                    // leave the handler entered for this exception
                    if let Some(b) = f.blocks.pop() {
                        if b.kind == BLOCK_HANDLER {
                            self.handled.pop();
                        }
                    }
                    let n = f.stack.len();
                    if let Value::List(l) = &f.stack[n - 2] {
                        l.borrow_mut().push(exc);
                    }
                }
                Op::ExcStarFinish => {
                    let raised = Self::pop(f);
                    let rest = Self::pop(f);
                    let orig = Self::pop(f);
                    let r = self.exc_star_finish(orig, rest, raised)?;
                    if !r.is_none() {
                        f.reraise_line = f.blocks.last().map(|b| b.line);
                        return Err(Exc { value: r });
                    }
                }
                Op::LoadExc => {
                    let v = self.handled.last().cloned().unwrap_or(Value::None);
                    f.stack.push(v);
                }
            }
        }
    }

    fn cell_name(&self, code: &Code, i: u32) -> String {
        let i = i as usize;
        if i < code.cellvars.len() {
            code.cellvars[i].to_string()
        } else {
            code.freevars[i - code.cellvars.len()].to_string()
        }
    }

    fn unbound_deref(&mut self, code: &Code, i: u32) -> Exc {
        let n = self.cell_name(code, i);
        if (i as usize) < code.cellvars.len() {
            self.err("UnboundLocalError", format!("cannot access local variable '{}' where it is not associated with a value", n))
        } else {
            self.err("NameError", format!("cannot access free variable '{}' where it is not associated with a value in enclosing scope", n))
        }
    }

    pub fn name_error(&mut self, name: &PyStr) -> Exc {
        let e = self.err("NameError", format!("name '{}' is not defined", name.s));
        if let Value::Instance(i) = &e.value {
            i.extra.borrow_mut().set_s("name", Value::str(&name.s));
        }
        e
    }

    fn do_raise(&mut self, f: &mut Frame, n: u8) -> PyResult<Exc> {
        if n == 0 {
            return match self.handled.last().cloned() {
                Some(v) => {
                    f.reraise_line = None;
                    self.bare_reraise = true;
                    Ok(Exc { value: v })
                }
                None => Ok(self.err("RuntimeError", "No active exception to reraise")),
            };
        }
        let cause = if n == 2 { Some(Self::pop(f)) } else { None };
        let e = Self::pop(f);
        let exc = self.make_raisable(e)?;
        if let Some(c) = cause {
            let c = match c {
                Value::None => Value::None,
                Value::Class(ref k) if k.is_subclass(&self.t.base_exception) => self.make_raisable(c)?,
                Value::Instance(ref i) if i.class.borrow().is_subclass(&self.t.base_exception) => c,
                _ => return Err(self.type_error("exception causes must derive from BaseException")),
            };
            if let Value::Instance(i) = &exc {
                i.extra.borrow_mut().set_str(self.n.cause.clone(), c);
                i.extra.borrow_mut().set_str(self.n.suppress.clone(), Value::Bool(true));
            }
        }
        Ok(Exc { value: exc })
    }

    /// an exception class is instantiated; anything else must be an exception
    pub fn make_raisable(&mut self, e: Value) -> PyResult<Value> {
        match &e {
            Value::Class(c) if c.is_subclass(&self.t.base_exception) => {
                let v = self.call_vec(&e, vec![], &[])?;
                if !self.class_of(&v).is_subclass(&self.t.base_exception) {
                    let tn = self.type_name(&v);
                    return Err(self.type_error(format!("calling {} should have returned an instance of BaseException, not {}", crate::format::class_repr(c), tn)));
                }
                Ok(v)
            }
            Value::Instance(i) if i.class.borrow().is_subclass(&self.t.base_exception) => Ok(e),
            _ => Err(self.type_error("exceptions must derive from BaseException")),
        }
    }

    pub fn exc_matches(&mut self, e: &Value, t: &Value) -> PyResult<bool> {
        let cls = self.class_of(e);
        match t {
            Value::Class(c) => {
                if !c.is_subclass(&self.t.base_exception) {
                    return Err(self.type_error("catching classes that do not inherit from BaseException is not allowed"));
                }
                Ok(cls.is_subclass(c))
            }
            Value::Tuple(items) => {
                if items.iter().any(|it| !matches!(it, Value::Class(c) if c.is_subclass(&self.t.base_exception))) {
                    return Err(self.type_error("catching classes that do not inherit from BaseException is not allowed"));
                }
                Ok(items.iter().any(|it| matches!(it, Value::Class(c) if cls.is_subclass(c))))
            }
            _ => Err(self.type_error("catching classes that do not inherit from BaseException is not allowed")),
        }
    }

    fn exc_star_split(&mut self, rest: &Value, typ: &Value) -> PyResult<(Value, Value)> {
        if rest.is_none() {
            return Ok((Value::None, Value::None));
        }
        let m = self.import("_exceptiongroup", &Value::None, 0, &self.main_globals.clone())?;
        let as_group = self.getattr(&m, &ps("_as_group"))?;
        let g = self.call_vec(&as_group, vec![rest.clone()], &[])?;
        let split = self.getattr(&g, &ps("split"))?;
        let r = self.call_vec(&split, vec![typ.clone()], &[])?;
        let parts = self.to_vec(&r)?;
        Ok((parts[0].clone(), parts[1].clone()))
    }

    fn exc_star_finish(&mut self, orig: Value, rest: Value, raised: Value) -> PyResult<Value> {
        let m = self.import("_exceptiongroup", &Value::None, 0, &self.main_globals.clone())?;
        let fin = self.getattr(&m, &ps("_finish"))?;
        self.call_vec(&fin, vec![orig, rest, raised], &[])
    }

    pub fn lazy_builtin(&mut self, name: &str) -> PyResult<Value> {
        let m = self.import("_exceptiongroup", &Value::None, 0, &self.main_globals.clone())?;
        let v = self.getattr(&m, &ps(name))?;
        self.builtins.borrow_mut().set_s(name, v.clone());
        if let Value::Class(c) = &v {
            self.exc.insert(if name == "ExceptionGroup" { "ExceptionGroup" } else { "BaseExceptionGroup" }, c.clone());
        }
        Ok(v)
    }

    // ------------------------------------------------------------------
    // functions and calls

    pub fn make_function(&mut self, code: Rc<Code>, globals: Rc<RefCell<Dict>>, defaults: Vec<Value>, kwdefaults: Vec<(Name, Value)>, closure: Vec<Cellv>) -> Value {
        let name = code.name.clone();
        let qualname = code.qualname.clone();
        let f = Rc::new(Function {
            code: RefCell::new(code),
            globals,
            defaults: RefCell::new(defaults),
            kwdefaults: RefCell::new(kwdefaults),
            closure,
            name: RefCell::new(name),
            qualname: RefCell::new(qualname),
            dict: RefCell::new(None),
            annotations: RefCell::new(Value::Undef),
            gc: Default::default(),
        });
        crate::gc::track_func(&f);
        Value::Func(f)
    }

    #[inline]
    pub fn call_native(&mut self, n: &NativeFn, args: &[Value]) -> PyResult<Value> {
        self.depth += 1;
        if self.depth > self.max_depth + 50 {
            self.depth -= 1;
            return Err(self.err("RecursionError", "maximum recursion depth exceeded"));
        }
        let r = self.run_native(n, args, &[]);
        self.depth -= 1;
        r
    }

    /// runs a native; a bug that panics in it becomes a SystemError
    #[inline]
    pub fn run_native(&mut self, n: &NativeFn, args: &[Value], kw: crate::builtins::Kw) -> PyResult<Value> {
        if let Some((lo, hi)) = n.arity {
            let got = args.len().saturating_sub(1);
            if got < lo as usize || got > hi as usize {
                return Err(self.arity_error(n.name, lo, hi, got));
            }
        }
        #[cfg(panic = "unwind")]
        {
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| (n.f)(self, args, kw))) {
                Ok(r) => r,
                Err(p) => {
                    let msg = p.downcast_ref::<String>().cloned().or_else(|| p.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default();
                    Err(self.err("SystemError", format!("internal error in {}: {}", n.name, msg)))
                }
            }
        }
        #[cfg(not(panic = "unwind"))]
        {
            (n.f)(self, args, kw)
        }
    }

    #[cold]
    fn arity_error(&mut self, name: &str, lo: u8, hi: u8, got: usize) -> Exc {
        let short = name.rsplit('.').next().unwrap_or(name);
        let msg = if hi == 0 {
            format!("{}() takes no arguments ({} given)", name, got)
        } else if lo == hi && hi == 1 {
            format!("{}() takes exactly one argument ({} given)", name, got)
        } else if lo == hi {
            format!("{} expected {} arguments, got {}", short, lo, got)
        } else if got < lo as usize {
            format!("{} expected at least {} argument{}, got {}", short, lo, if lo == 1 { "" } else { "s" }, got)
        } else {
            format!("{} expected at most {} argument{}, got {}", short, hi, if hi == 1 { "" } else { "s" }, got)
        };
        self.type_error(msg)
    }

    pub fn call(&mut self, f: &Value, args: &[Value]) -> PyResult<Value> {
        self.call_vec(f, args.to_vec(), &[])
    }

    pub fn call_vec(&mut self, f: &Value, args: Vec<Value>, kw: &[(Rc<PyStr>, Value)]) -> PyResult<Value> {
        match f {
            Value::Func(func) => {
                if kw.is_empty() && func.code.borrow().simple && func.code.borrow().argcount as usize == args.len() {
                    self.call_simple(func, args)
                } else {
                    self.call_func(func, args, kw)
                }
            }
            Value::Native(n) => {
                self.depth += 1;
                if self.depth > self.max_depth + 50 {
                    self.depth -= 1;
                    return Err(self.err("RecursionError", "maximum recursion depth exceeded"));
                }
                let r = self.run_native(n, &args, kw);
                self.depth -= 1;
                r
            }
            Value::Method(m) => {
                let (s, func) = (&m.0, &m.1);
                let mut a = Vec::with_capacity(args.len() + 1);
                a.push(s.clone());
                a.extend(args);
                self.call_vec(func, a, kw)
            }
            Value::Class(c) => self.call_class(c, args, kw),
            Value::Instance(_) | Value::Obj(_) => {
                let call = self.lookup_special(f, &self.n.call.clone());
                match call {
                    Some(c) => {
                        self.enter("")?;
                        let r = self.call_vec(&c, args, kw);
                        self.depth -= 1;
                        r
                    }
                    None => {
                        let tn = self.type_name(f);
                        Err(self.type_error(format!("'{}' object is not callable", tn)))
                    }
                }
            }
            Value::StaticMethod(inner) => self.call_vec(&inner.clone(), args, kw),
            _ => {
                let tn = self.type_name(f);
                Err(self.type_error(format!("'{}' object is not callable", tn)))
            }
        }
    }

    /// a call with exactly the positional parameters, nothing else to bind
    #[inline]
    pub fn call_simple(&mut self, func: &Rc<Function>, mut args: Vec<Value>) -> PyResult<Value> {
        let code = func.code.borrow().clone();
        let n = code.varnames.len();
        // room for the locals and the operand stack, as bind_args leaves
        if args.capacity() < n + 16 {
            args.reserve_exact(n + 16 - args.len());
        }
        args.resize(n, Value::Undef);
        let mut f = Frame { code, ip: 0, stack: args, cells: func.closure.clone(), blocks: vec![], globals: func.globals.clone(), ns: None, reraise_line: None, ext: None };
        self.depth += 1;
        if self.depth > self.max_depth {
            self.depth -= 1;
            return Err(self.err("RecursionError", "maximum recursion depth exceeded"));
        }
        let r = self.run_frame(&mut f);
        self.depth -= 1;
        let mut st = std::mem::take(&mut f.stack);
        st.clear();
        if self.stack_pool.len() < 64 {
            self.stack_pool.push(st);
        }
        match r? {
            Exit::Return(v) => Ok(v),
            Exit::Yield(v) => Ok(v),
        }
    }

    pub fn call_func(&mut self, func: &Rc<Function>, args: Vec<Value>, kw: &[(Rc<PyStr>, Value)]) -> PyResult<Value> {
        let code = func.code.borrow().clone();
        let locals = self.bind_args(func, &code, args, kw)?;
        let mut cells: Vec<Cellv> = Vec::with_capacity(code.ncells());
        for (i, _) in code.cellvars.iter().enumerate() {
            let init = match code.cell_from_arg[i] {
                Some(slot) => locals[slot as usize].clone(),
                None => Value::Undef,
            };
            cells.push(Rc::new(RefCell::new(init)));
        }
        cells.extend(func.closure.iter().cloned());
        let f = Frame { code: code.clone(), ip: 0, stack: locals, cells, blocks: vec![], globals: func.globals.clone(), ns: None, reraise_line: None, ext: None };
        if code.generator || code.coroutine {
            let g = Generator { frame: Some(Box::new(f)), started: false, running: false, name: func.name.borrow().clone(), qualname: func.qualname.borrow().clone(), coroutine: code.coroutine && !code.generator, handled: vec![], code: code.clone(), frame_obj: None };
            let g = crate::gc::new_gen(g);
            if code.coroutine && code.generator {
                // an async generator: pylib/_asyncgen.py drives the generator
                let g0 = self.main_globals.clone();
                let m = self.import("_asyncgen", &Value::None, 0, &g0)?;
                let cls = self.getattr(&m, &crate::builtins::ps("async_generator"))?;
                return self.call_vec(&cls, vec![g], &[]);
            }
            return Ok(g);
        }
        let mut f = f;
        self.depth += 1;
        if self.depth > self.max_depth {
            self.depth -= 1;
            return Err(self.err("RecursionError", "maximum recursion depth exceeded"));
        }
        let r = self.run_frame(&mut f);
        self.depth -= 1;
        match r? {
            Exit::Return(v) => Ok(v),
            Exit::Yield(v) => Ok(v),
        }
    }

    fn bind_args(&mut self, func: &Function, code: &Code, args: Vec<Value>, kw: &[(Rc<PyStr>, Value)]) -> PyResult<Vec<Value>> {
        let nloc = code.varnames.len();
        let mut locals: Vec<Value> = Vec::with_capacity(nloc + 16);
        let argc = code.argcount as usize;
        let fname = || func.qualname.borrow().s.clone();
        let nargs = args.len();
        if nargs > argc && !code.varargs {
            let defaults = func.defaults.borrow().len();
            let msg = if defaults > 0 {
                format!("{}() takes from {} to {} positional arguments but {} {} given", fname(), argc - defaults, argc, nargs, if nargs == 1 { "was" } else { "were" })
            } else {
                format!("{}() takes {} positional argument{} but {} {} given", fname(), argc, if argc == 1 { "" } else { "s" }, nargs, if nargs == 1 { "was" } else { "were" })
            };
            return Err(self.type_error(msg));
        }
        let mut it = args.into_iter();
        for _ in 0..argc.min(nargs) {
            locals.push(it.next().unwrap());
        }
        locals.resize(nloc, Value::Undef);
        let mut slot = argc;
        if code.varargs {
            locals[slot] = Value::tuple(it.collect());
            slot += 1;
        }
        let kwonly_start = slot;
        let kwonly_end = slot + code.kwonly as usize;
        let mut kwdict: Option<Dict> = if code.varkw { Some(Dict::new()) } else { None };
        let mut posonly_kw: Vec<String> = vec![];
        for (k, v) in kw {
            let mut found = None;
            for i in (code.posonly as usize)..argc {
                if *code.varnames[i] == *k.s {
                    found = Some(i);
                    break;
                }
            }
            if found.is_none() {
                for i in kwonly_start..kwonly_end {
                    if *code.varnames[i] == *k.s {
                        found = Some(i);
                        break;
                    }
                }
            }
            match found {
                Some(i) => {
                    if !locals[i].is_undef() {
                        return Err(self.type_error(format!("{}() got multiple values for argument '{}'", fname(), k.s)));
                    }
                    locals[i] = v.clone();
                }
                None => match &mut kwdict {
                    Some(d) => {
                        d.set_str(k.clone(), v.clone());
                    }
                    None => {
                        if (0..code.posonly as usize).any(|i| *code.varnames[i] == *k.s) {
                            posonly_kw.push(k.s.clone());
                            continue;
                        }
                        return Err(self.type_error(format!("{}() got an unexpected keyword argument '{}'", fname(), k.s)));
                    }
                },
            }
        }
        if !posonly_kw.is_empty() {
            return Err(self.type_error(format!("{}() got some positional-only arguments passed as keyword arguments: '{}'", fname(), posonly_kw.join(", "))));
        }
        if code.varkw {
            locals[kwonly_end] = Value::dict(kwdict.take().unwrap());
        }
        // defaults
        let defaults = func.defaults.borrow();
        let nd = defaults.len();
        let mut missing = vec![];
        for i in 0..argc {
            if locals[i].is_undef() {
                if i + nd >= argc {
                    locals[i] = defaults[i + nd - argc].clone();
                } else {
                    missing.push(code.varnames[i].to_string());
                }
            }
        }
        if !missing.is_empty() {
            let names: Vec<String> = missing.iter().map(|m| format!("'{}'", m)).collect();
            let list = match names.len() {
                1 => names[0].clone(),
                2 => format!("{} and {}", names[0], names[1]),
                _ => format!("{}, and {}", names[..names.len() - 1].join(", "), names[names.len() - 1]),
            };
            return Err(self.type_error(format!("{}() missing {} required positional argument{}: {}", fname(), missing.len(), if missing.len() == 1 { "" } else { "s" }, list)));
        }
        let kwd = func.kwdefaults.borrow();
        let mut kmissing = vec![];
        for i in kwonly_start..kwonly_end {
            if locals[i].is_undef() {
                match kwd.iter().find(|(n, _)| **n == *code.varnames[i]) {
                    Some((_, v)) => locals[i] = v.clone(),
                    None => kmissing.push(format!("'{}'", code.varnames[i])),
                }
            }
        }
        if !kmissing.is_empty() {
            let list = match kmissing.len() {
                1 => kmissing[0].clone(),
                2 => format!("{} and {}", kmissing[0], kmissing[1]),
                _ => format!("{}, and {}", kmissing[..kmissing.len() - 1].join(", "), kmissing[kmissing.len() - 1]),
            };
            return Err(self.type_error(format!("{}() missing {} required keyword-only argument{}: {}", fname(), kmissing.len(), if kmissing.len() == 1 { "" } else { "s" }, list)));
        }
        Ok(locals)
    }

    // ------------------------------------------------------------------
    // generators

    /// resumes a generator; Ok(None) when it returned (value in last_return)
    pub fn resume(&mut self, g: &Rc<RefCell<Generator>>, sent: Value, throw: Option<Value>) -> PyResult<Option<Value>> {
        let base = self.handled.len();
        let mut frame = {
            let mut gb = g.borrow_mut();
            if gb.running {
                drop(gb);
                return Err(self.value_error("generator already executing"));
            }
            match gb.frame.take() {
                Some(f) => {
                    if !gb.started && !sent.is_none() && throw.is_none() {
                        gb.frame = Some(f);
                        drop(gb);
                        return Err(self.type_error("can't send non-None value to a just-started generator"));
                    }
                    let started = gb.started;
                    gb.started = true;
                    gb.running = true;
                    let saved = std::mem::take(&mut gb.handled);
                    self.handled.extend(saved);
                    if started && throw.is_none() {
                        let mut f = f;
                        f.stack.push(sent);
                        f
                    } else {
                        f
                    }
                }
                None => {
                    drop(gb);
                    if let Some(t) = throw {
                        return Err(Exc { value: t });
                    }
                    self.last_return = Some(Value::None);
                    return Ok(None);
                }
            }
        };
        // and as a generator resumes at a `yield` (not in `yield from`,
        // where the iterator it delegates to resumes): a handler's
        // exception is raised at the `yield`
        let throw = match throw {
            None if crate::posix::signals_pending() && frame.ip > 0 && !matches!(frame.code.ops.get(frame.ip), Some(Op::YieldFrom)) => {
                crate::posix::check_signals(self).err().map(|e| e.value)
            }
            t => t,
        };
        self.depth += 1;
        let r = match throw {
            Some(t) => self.throw_into(&mut frame, t),
            None => self.run_frame(&mut frame),
        };
        self.depth -= 1;
        let mut gb = g.borrow_mut();
        gb.running = false;
        let mine = if base <= self.handled.len() { self.handled.split_off(base) } else { vec![] };
        match r {
            Ok(Exit::Yield(v)) => {
                gb.handled = mine;
                gb.frame = Some(frame);
                Ok(Some(v))
            }
            Ok(Exit::Return(v)) => {
                drop(gb);
                self.last_return = Some(v);
                Ok(None)
            }
            Err(e) => {
                drop(gb);
                // a StopIteration escaping a generator becomes RuntimeError
                if self.exc_is(&e, "StopIteration") {
                    let re = self.err("RuntimeError", "generator raised StopIteration");
                    if let Value::Instance(i) = &re.value {
                        i.extra.borrow_mut().set_str(self.n.cause.clone(), e.value.clone());
                        i.extra.borrow_mut().set_str(self.n.context.clone(), e.value.clone());
                        i.extra.borrow_mut().set_str(self.n.suppress.clone(), Value::Bool(true));
                    }
                    return Err(re);
                }
                Err(e)
            }
        }
    }

    /// raises `exc` inside a resumed generator frame; a frame suspended in
    /// `yield from` hands it to the iterator it delegates to first
    fn throw_into(&mut self, f: &mut Frame, exc: Value) -> PyResult<Exit> {
        let mut exc = Exc { value: exc };
        if matches!(f.code.ops.get(f.ip), Some(Op::YieldFrom)) {
            if let Some(recv) = f.stack.last().cloned() {
                // the delegating frame is running while the iterator handles it
                self.frames.push(f as *mut Frame);
                self.frame_objs.push(Value::Undef);
                let r = self.throw_delegate(&recv, exc.value.clone());
                self.frames.pop();
                self.frame_objs.pop();
                match r {
                    Ok(Some(v)) => return Ok(Exit::Yield(v)),
                    Ok(None) => {
                        // the iterator finished: its value is the result of `yield from`
                        f.stack.pop();
                        let rv = self.last_return.take().unwrap_or(Value::None);
                        f.stack.push(rv);
                        f.ip += 1;
                        return self.run_frame(f);
                    }
                    Err(e) => exc = e,
                }
            }
        }
        self.handle_exc(f, exc)?;
        self.run_frame(f)
    }

    /// `yield from`'s part of throw() and close(): GeneratorExit closes the
    /// iterator and is then raised anyway; anything else goes to its throw()
    fn throw_delegate(&mut self, recv: &Value, exc: Value) -> PyResult<Option<Value>> {
        if self.exc_is(&Exc { value: exc.clone() }, "GeneratorExit") {
            if let Some(close) = self.getattr_opt(recv, &ps("close"))? {
                self.call_vec(&close, vec![], &[])?;
            }
            return Err(Exc { value: exc });
        }
        if let Value::Gen(g) = recv {
            return self.resume(g, Value::None, Some(exc));
        }
        let Some(throw) = self.getattr_opt(recv, &ps("throw"))? else {
            return Err(Exc { value: exc });
        };
        match self.call_vec(&throw, vec![exc], &[]) {
            Ok(v) => Ok(Some(v)),
            Err(e) if self.exc_is(&e, "StopIteration") => {
                let a = self.exc_args(&e.value);
                self.last_return = Some(a.into_iter().next().unwrap_or(Value::None));
                Ok(None)
            }
            Err(e) => Err(e),
        }
    }

    /// one step of `yield from`: Some(value yielded) or None when finished
    pub fn send(&mut self, recv: &Value, sent: Value) -> PyResult<Option<Value>> {
        match recv {
            Value::Gen(g) => self.resume(g, sent, None),
            _ => {
                if sent.is_none() {
                    match self.next(recv)? {
                        Some(v) => Ok(Some(v)),
                        None => {
                            if self.last_return.is_none() {
                                self.last_return = Some(Value::None);
                            }
                            Ok(None)
                        }
                    }
                } else {
                    let m = self.getattr(recv, &ps("send"))?;
                    match self.call_vec(&m, vec![sent], &[]) {
                        Ok(v) => Ok(Some(v)),
                        Err(e) if self.exc_is(&e, "StopIteration") => {
                            let a = self.exc_args(&e.value);
                            self.last_return = Some(a.into_iter().next().unwrap_or(Value::None));
                            Ok(None)
                        }
                        Err(e) => Err(e),
                    }
                }
            }
        }
    }

    // ------------------------------------------------------------------
    // classes

    pub fn call_class(&mut self, cls: &Rc<Class>, args: Vec<Value>, kw: &[(Rc<PyStr>, Value)]) -> PyResult<Value> {
        // a metaclass with its own __call__
        if let Some(meta) = cls.meta.borrow().clone() {
            if !Rc::ptr_eq(&meta, &self.t.type_) {
                if let Some(c) = meta.lookup(&self.n.call) {
                    if matches!(c, Value::Func(_)) {
                        let mut a = Vec::with_capacity(args.len() + 1);
                        a.push(Value::Class(cls.clone()));
                        a.extend(args);
                        return self.call_vec(&c, a, kw);
                    }
                    if !matches!(c, Value::Native(_)) {
                        // a non-function __call__ (TypedDict's `__call__ = dict`) is not bound
                        let f = self.bind(c, &Value::Class(cls.clone()), &meta);
                        return self.call_vec(&f, args, kw);
                    }
                }
            }
        }
        self.type_call(cls, args, kw)
    }

    /// type.__call__: __new__ then __init__
    pub fn type_call(&mut self, cls: &Rc<Class>, args: Vec<Value>, kw: &[(Rc<PyStr>, Value)]) -> PyResult<Value> {
        if Rc::ptr_eq(cls, &self.t.type_) && args.len() == 1 && kw.is_empty() {
            return Ok(Value::Class(self.class_of(&args[0])));
        }
        let new = cls.lookup(&self.n.new);
        let obj = match &new {
            Some(Value::Native(n)) if n.name == "object.__new__" && cls.dict.borrow().get_s("__abstractmethods__").is_none() => self.alloc_instance(cls)?,
            Some(f @ (Value::StaticMethod(_) | Value::Native(_) | Value::Func(_))) => {
                let f = match f {
                    Value::StaticMethod(inner) => (**inner).clone(),
                    other => other.clone(),
                };
                let mut a = Vec::with_capacity(args.len() + 1);
                a.push(Value::Class(cls.clone()));
                a.extend(args.iter().cloned());
                self.call_vec(&f, a, kw)?
            }
            Some(Value::ClassMethod(f)) => {
                let mut a = Vec::with_capacity(args.len() + 2);
                a.push(Value::Class(cls.clone()));
                a.push(Value::Class(cls.clone()));
                a.extend(args.iter().cloned());
                self.call_vec(&f, a, kw)?
            }
            _ => self.alloc_instance(cls)?,
        };
        // __init__ runs when __new__ returned an instance of cls
        let ocls = self.class_of(&obj);
        if !ocls.is_subclass(cls) {
            return Ok(obj);
        }
        if let Some(init) = ocls.lookup(&self.n.init) {
            match &init {
                Value::Native(n) if n.name == "object.__init__" => {
                    let new_is_object = matches!(&new, Some(Value::Native(n)) if n.name == "object.__new__") || new.is_none();
                    if (!args.is_empty() || !kw.is_empty()) && new_is_object {
                        let cn = cls.name();
                        return Err(self.type_error(format!("{}() takes no arguments", cn)));
                    }
                }
                _ => {
                    let r = if matches!(init, Value::Func(_) | Value::Native(_)) {
                        let mut a = Vec::with_capacity(args.len() + 1);
                        a.push(obj.clone());
                        a.extend(args);
                        self.call_vec(&init, a, kw)?
                    } else {
                        // a descriptor: call what it gives for the instance
                        let ini = self.n.init.clone();
                        match self.lookup_special_get(&obj, &ini)? {
                            Some(f) => self.call_vec(&f, args, kw)?,
                            None => Value::None,
                        }
                    };
                    if !r.is_none() {
                        let tn = self.type_name(&r);
                        return Err(self.type_error(format!("__init__() should return None, not '{}'", tn)));
                    }
                }
            }
        }
        Ok(obj)
    }

    /// a new instance of a class, made where an allocation can fail: a
    /// MemoryError when _testcapi.set_nomemory says this one does
    pub fn alloc_instance(&mut self, cls: &Rc<Class>) -> PyResult<Value> {
        if crate::testcapi::alloc_fails() {
            let cls = self.exc_class("MemoryError");
            return Err(Exc { value: self.new_exc(&cls, vec![]) });
        }
        Ok(self.new_instance(cls))
    }

    pub fn new_instance(&mut self, cls: &Rc<Class>) -> Value {
        let base = match cls.builtin.get() {
            Builtin::List => Value::list(vec![]),
            Builtin::Dict => Value::dict(Dict::new()),
            Builtin::Set => Value::Set(Rc::new(RefCell::new(Dict::new()))),
            Builtin::BaseException => Value::tuple(vec![]),
            _ => Value::Undef,
        };
        let inst = Rc::new(Instance::new(cls.clone(), base));
        crate::gc::track_inst(&inst);
        Value::Instance(inst)
    }

    /// C3 linearisation
    pub fn compute_mro(&mut self, cls: &Rc<Class>) -> PyResult<()> {
        let bases = cls.bases.borrow().clone();
        let mut seqs: Vec<Vec<Rc<Class>>> = bases.iter().map(|b| b.mro.borrow().clone()).collect();
        seqs.push(bases.clone());
        let mut out = vec![cls.clone()];
        loop {
            seqs.retain(|s| !s.is_empty());
            if seqs.is_empty() {
                break;
            }
            let mut cand = None;
            for s in &seqs {
                let h = &s[0];
                if !seqs.iter().any(|t| t[1..].iter().any(|x| Rc::ptr_eq(x, h))) {
                    cand = Some(h.clone());
                    break;
                }
            }
            let c = match cand {
                Some(c) => c,
                None => {
                    let names: Vec<String> = bases.iter().map(|b| b.name()).collect();
                    return Err(self.type_error(format!("Cannot create a consistent method resolution order (MRO) for bases {}", names.join(", "))));
                }
            };
            for s in seqs.iter_mut() {
                if Rc::ptr_eq(&s[0], &c) {
                    s.remove(0);
                }
            }
            out.push(c);
        }
        cls.set_mro(out);
        Ok(())
    }

    /// type(name, bases, ns) with metaclass `meta`
    pub fn type_new(&mut self, meta: &Rc<Class>, name: &str, bases: Vec<Rc<Class>>, ns: &Rc<RefCell<Dict>>) -> PyResult<Rc<Class>> {
        let bases = if bases.is_empty() { vec![self.t.object.clone()] } else { bases };
        // the built-in kind comes from the first base that has one
        let mut kind = Builtin::User;
        for b in &bases {
            let k = b.builtin.get();
            if k != Builtin::Object && k != Builtin::User {
                kind = match k {
                    Builtin::Type => Builtin::Type,
                    other => other,
                };
                break;
            }
            if k == Builtin::User {
                // a user class derived from a built-in carries its kind
            }
        }
        if kind == Builtin::User {
            for b in &bases {
                for m in b.mro.borrow().iter() {
                    let k = m.builtin.get();
                    if k != Builtin::Object && k != Builtin::User {
                        kind = k;
                        break;
                    }
                }
                if kind != Builtin::User {
                    break;
                }
            }
        }
        let cls = Class::new(name, bases.clone(), Builtin::User);
        cls.builtin.set(kind);
        {
            let d = ns.borrow().copy();
            cls.set_dict(d);
        }
        if let Some(q) = cls.dict.borrow().get_str(&self.n.qualname) {
            if let Value::Str(s) = q {
                *cls.qualname.borrow_mut() = s.clone();
            }
        }
        cls.dict.borrow_mut().remove_str(&self.n.qualname);
        let cc = cls.dict.borrow_mut().remove_str(&self.n.classcell);
        match cc {
            Some(Value::Cell(c)) => *c.borrow_mut() = Value::Class(cls.clone()),
            Some(other) => {
                let r = format!("<class '{}'>", self.type_name(&other));
                return Err(self.type_error(format!("__classcell__ must be a nonlocal cell, not {}", r)));
            }
            None => {}
        }
        // implicit staticmethod / classmethod
        for (n, wrap) in [("__new__", 0), ("__init_subclass__", 1), ("__class_getitem__", 1)] {
            let key = PyStr::new(n.to_string());
            let cur = cls.dict.borrow().get_str(&key).cloned();
            if let Some(v @ Value::Func(_)) = cur {
                let w = if wrap == 0 { Value::StaticMethod(Rc::new(v)) } else { Value::ClassMethod(Rc::new(v)) };
                cls.dict.borrow_mut().set_s(n, w);
            }
        }
        *cls.meta.borrow_mut() = Some(meta.clone());
        self.compute_mro(&cls)?;
        if !Rc::ptr_eq(meta, &self.t.type_) {
            // a metaclass's own mro()
            if let Some(f @ Value::Func(_)) = meta.lookup(&PyStr::new("mro".to_string())) {
                let r = self.call_vec(&f, vec![Value::Class(cls.clone())], &[])?;
                let mut mro = vec![];
                for x in self.to_vec(&r)? {
                    match x {
                        Value::Class(c) => mro.push(c),
                        other => {
                            let tn = self.type_name(&other);
                            return Err(self.type_error(format!("mro() returned a non-class ('{}')", tn)));
                        }
                    }
                }
                cls.set_mro(mro);
            }
        }
        self.update_flags(&cls);
        // __slots__
        let slots = cls.dict.borrow().get_str(&self.n.slots).cloned();
        if let Some(s) = slots {
            let items = match &s {
                Value::Str(_) => vec![s.clone()],
                other => self.to_vec(other)?,
            };
            let mut names: Vec<Name> = vec![];
            for it in &items {
                let n = match it.as_str() {
                    Some(n) => n.to_string(),
                    None => {
                        let tn = self.type_name(it);
                        return Err(self.type_error(format!("__slots__ items must be strings, not '{}'", tn)));
                    }
                };
                let mut cs = n.chars();
                let ident = cs.next().map_or(false, |f| f == '_' || f.is_alphabetic()) && cs.all(|c| c == '_' || c.is_alphanumeric());
                if !ident {
                    return Err(self.type_error("__slots__ must be identifiers"));
                }
                if n == "__dict__" || n == "__weakref__" {
                    // one each, and none when a base already provides it
                    let bases = cls.bases.borrow().clone();
                    let inherited = bases.iter().any(|b| {
                        b.mro.borrow().iter().any(|c| match &*c.slots.borrow() {
                            Some(s) => s.iter().any(|x| **x == *n),
                            None => matches!(c.dict.borrow().get_s("__module__"), Some(Value::Str(m)) if &*m.s != "builtins"),
                        })
                    });
                    if inherited || names.iter().any(|x| **x == *n) {
                        return Err(self.type_error(format!("{} slot disallowed: we already got one", n)));
                    }
                    names.push(Rc::from(n.as_str()));
                    continue;
                }
                if names.iter().any(|x| **x == *n) {
                    return Err(self.type_error(format!("duplicate slot name '{}'", n)));
                }
                // variable-size instances have no room for slots (CPython also
                // refuses tuple, but the struct sequences here are tuples with slots)
                let varsize = cls.mro.borrow().iter().rev().find(|c| matches!(c.builtin.get(), Builtin::Int | Builtin::Bytes)).cloned();
                if let Some(b) = varsize {
                    return Err(self.type_error(format!("nonempty __slots__ not supported for subtype of '{}'", b.name())));
                }
                names.push(Rc::from(n.as_str()));
                let m = crate::compile::mangle(name, &n);
                let key = Rc::new(PyStr::new(m.clone()));
                if cls.dict.borrow().get_str(&key).is_some() {
                    return Err(self.value_error(format!("'{}' in __slots__ conflicts with class variable", n)));
                }
                let member = crate::tb::new_member(self, key.clone(), &cls);
                cls.dict.borrow_mut().set_str(key, member);
            }
            *cls.slots.borrow_mut() = Some(names);
        }
        // an ordinary class's instances keep their attributes inline, as
        // CPython's do when neither __slots__ nor a built-in base adds to
        // their layout
        if kind == Builtin::User && cls.mro.borrow().iter().all(|c| c.slots.borrow().is_none()) {
            *cls.shared_keys.borrow_mut() = Some(Rc::new(SharedKeys::default()));
        }
        if cls.dict.borrow().get_str(&self.n.doc).is_none() {
            cls.dict.borrow_mut().set_str(self.n.doc.clone(), Value::None);
        }
        Ok(cls)
    }

    pub fn update_flags(&mut self, cls: &Rc<Class>) {
        let ga = cls.lookup(&self.n.getattr).is_some();
        let gab = matches!(cls.lookup(&self.n.getattribute), Some(Value::Func(_)));
        let sa = matches!(cls.lookup(&self.n.setattr), Some(Value::Func(_)));
        cls.has_getattr.set(ga);
        cls.has_del.set(cls.lookup(&PyStr::new("__del__".to_string())).is_some());
        cls.has_getattribute.set(gab);
        cls.has_setattr.set(sa || matches!(cls.lookup(&self.n.delattr), Some(Value::Func(_))));
    }

    /// __build_class__(func, name, *bases, metaclass=None, **kw)
    pub fn build_class(&mut self, args: &[Value], kw: &[(Rc<PyStr>, Value)]) -> PyResult<Value> {
        if args.len() < 2 {
            return Err(self.type_error("__build_class__: not enough arguments"));
        }
        let body = args[0].clone();
        let name = match &args[1] {
            Value::Str(s) => s.s.clone(),
            _ => return Err(self.type_error("__build_class__: name is not a string")),
        };
        let orig_bases: Vec<Value> = args[2..].to_vec();
        // __mro_entries__ of bases that are not types
        let mut bases_v = vec![];
        let mut changed = false;
        for b in &orig_bases {
            if let Value::Class(_) = b {
                bases_v.push(b.clone());
                continue;
            }
            let me = match self.getattr_opt(b, &self.n.mro_entries.clone()) {
                Ok(m) => m,
                Err(e) if self.exc_is(&e, "AttributeError") => None,
                Err(e) => return Err(e),
            };
            match me {
                Some(f) => {
                    let r = self.call_vec(&f, vec![Value::tuple(orig_bases.clone())], &[])?;
                    match r {
                        Value::Tuple(t) => bases_v.extend(t.iter().cloned()),
                        _ => return Err(self.type_error("__mro_entries__ must return a tuple")),
                    }
                    changed = true;
                }
                None => bases_v.push(b.clone()),
            }
        }
        let mut meta: Option<Value> = None;
        let mut rest_kw = vec![];
        for (k, v) in kw {
            if &*k.s == "metaclass" {
                meta = Some(v.clone());
            } else {
                rest_kw.push((k.clone(), v.clone()));
            }
        }
        let meta = match meta {
            Some(m) => m,
            None => match bases_v.first() {
                Some(b) => Value::Class(self.class_of(b)),
                None => Value::Class(self.t.type_.clone()),
            },
        };
        // the most derived metaclass among the bases
        let meta = match &meta {
            Value::Class(mc) => {
                let mut winner = mc.clone();
                for b in &bases_v {
                    let bm = self.class_of(b);
                    if winner.is_subclass(&bm) {
                        continue;
                    }
                    if bm.is_subclass(&winner) {
                        winner = bm;
                        continue;
                    }
                    return Err(self.type_error("metaclass conflict: the metaclass of a derived class must be a (non-strict) subclass of the metaclasses of all its bases"));
                }
                Value::Class(winner)
            }
            other => other.clone(),
        };
        let bases_t = Value::tuple(bases_v.clone());
        let ns = Rc::new(RefCell::new(Dict::new()));
        let plain = matches!(&meta, Value::Class(mc) if Rc::ptr_eq(mc, &self.t.type_));
        let ns = if plain {
            ns
        } else {
            let prep = match self.getattr_opt(&meta, &self.n.prepare.clone()) {
                Ok(p) => p,
                Err(e) if self.exc_is(&e, "AttributeError") => None,
                Err(e) => return Err(e),
            };
            match prep {
                Some(p) => {
                    let r = self.call_vec(&p, vec![Value::str(&name), bases_t.clone()], &rest_kw)?;
                    match r {
                        Value::Dict(d) => d,
                        Value::Instance(i) if matches!(&*i.base.borrow(), Value::Dict(_)) => match &*i.base.borrow() {
                            Value::Dict(d) => d.clone(),
                            _ => ns,
                        },
                        other => {
                            if self.class_of(&other).lookup(&self.n.getitem).is_none() {
                                let mn = match &meta {
                                    Value::Class(c) => c.name(),
                                    _ => "<metaclass>".to_string(),
                                };
                                let tn = self.type_name(&other);
                                return Err(self.type_error(format!("{}.__prepare__() must return a mapping, not {}", mn, tn)));
                            }
                            ns
                        }
                    }
                }
                None => ns,
            }
        };
        if changed {
            ns.borrow_mut().set_s("__orig_bases__", Value::tuple(orig_bases.clone()));
        }
        // run the body
        let cell = match &body {
            Value::Func(f) => {
                let code = f.code.borrow().clone();
                let n = code.varnames.len();
                let mut stack = Vec::with_capacity(16 + n);
                stack.resize(n, Value::Undef);
                let mut cells: Vec<Cellv> = code.cellvars.iter().map(|_| Rc::new(RefCell::new(Value::Undef))).collect();
                cells.extend(f.closure.iter().cloned());
                let mut fr = Frame { code, ip: 0, stack, cells, blocks: vec![], globals: f.globals.clone(), ns: Some(ns.clone()), reraise_line: None, ext: None };
                match self.run_frame(&mut fr)? {
                    Exit::Return(v) | Exit::Yield(v) => v,
                }
            }
            _ => Value::None,
        };
        let cls_v = match &meta {
            Value::Class(mc) if Rc::ptr_eq(mc, &self.t.type_) => {
                let mut bases = vec![];
                for b in &bases_v {
                    match b {
                        Value::Class(c) => bases.push(c.clone()),
                        _ => return Err(self.type_error("bases must be types")),
                    }
                }
                crate::methods::check_bases(self, &bases)?;
                let c = self.type_new(mc, &name, bases, &ns)?;
                Value::Class(c)
            }
            _ => {
                let nsv = Value::Dict(ns.clone());
                self.call_vec(&meta, vec![Value::str(&name), bases_t, nsv], &rest_kw)?
            }
        };
        if let (Value::Cell(c), Value::Class(_)) = (&cell, &cls_v) {
            let got = c.borrow().clone();
            if !got.is(&cls_v) {
                let cr = self.repr(&cls_v)?;
                if got.is_undef() {
                    return Err(self.err("RuntimeError", format!("__class__ not set defining '{}' as {}. Was __classcell__ propagated to type.__new__?", name, cr)));
                }
                let gr = self.repr(&got)?;
                return Err(self.type_error(format!("__class__ set to {} defining '{}' as {}", gr, name, cr)));
            }
        }
        // a metaclass's type.__new__ already ran class_created
        if let (Value::Class(cls), Value::Class(mc)) = (&cls_v, &meta) {
            if Rc::ptr_eq(mc, &self.t.type_) {
                self.class_created(cls, &rest_kw, true)?;
            }
        }
        Ok(cls_v)
    }

    /// __set_name__ on the namespace, then the parent's __init_subclass__
    pub fn class_created(&mut self, cls: &Rc<Class>, kw: &[(Rc<PyStr>, Value)], plain_type: bool) -> PyResult<()> {
        if cls.dict.borrow().get_s("__ranger_created__").is_some() {
            return Ok(());
        }
        cls.dict.borrow_mut().set_s("__ranger_created__", Value::Bool(true));
        let items = cls.dict.borrow().items();
        cls.dict.borrow_mut().remove_str(&PyStr::new("__ranger_created__".into()));
        for (k, v) in items {
            if let Value::Instance(_) | Value::Property(_) | Value::Obj(_) = &v {
                if let Some(sn) = self.lookup_special_get(&v, &self.n.set_name.clone())? {
                    self.call_vec(&sn, vec![Value::Class(cls.clone()), k.clone()], &[])?;
                }
            }
        }
        // the parent's __init_subclass__ (classmethod)
        let mro = cls.mro.borrow().clone();
        for c in mro.iter().skip(1) {
            if let Some(v) = c.dict.borrow().get_str(&self.n.init_subclass).cloned() {
                let f = self.bind_classmethod(&v, &Value::Class(cls.clone()));
                self.call_vec(&f, vec![], kw)?;
                return Ok(());
            }
        }
        if !kw.is_empty() && plain_type {
            return Err(self.type_error(format!("{}.__init_subclass__() takes no keyword arguments", cls.name())));
        }
        Ok(())
    }

    pub fn bind_classmethod(&mut self, v: &Value, owner: &Value) -> Value {
        match v {
            Value::ClassMethod(f) => Value::Method(Rc::new((owner.clone(), (**f).clone()))),
            Value::StaticMethod(f) => (**f).clone(),
            Value::Func(_) => Value::Method(Rc::new((owner.clone(), v.clone()))),
            other => other.clone(),
        }
    }

    // ------------------------------------------------------------------
    // imports

    pub fn import(&mut self, name: &str, fromlist: &Value, level: i64, globals: &Rc<RefCell<Dict>>) -> PyResult<Value> {
        let full = if level > 0 {
            let pkg = globals.borrow().get_s("__package__").filter(|v| !v.is_none());
            let mut base = match pkg {
                Some(p) => match p.as_str() {
                    Some(p) => p.to_string(),
                    None => return Err(self.type_error("package must be a string")),
                },
                None => {
                    // importlib's _calc___package__ without a usable __spec__
                    let spec = globals.borrow().get_s("__spec__").filter(|v| !v.is_none());
                    let parent = match spec {
                        Some(spec) => self.getattr(&spec, &crate::builtins::ps("parent")).ok().and_then(|v| v.as_str().map(|s| s.to_string())),
                        None => None,
                    };
                    match parent {
                        Some(p) => p,
                        None => {
                            crate::tb::warn(self, "ImportWarning", "can't resolve package from __spec__ or __package__, falling back on __name__ and __path__")?;
                            let modname = globals.borrow().get_s("__name__").and_then(|v| v.as_str().map(|s| s.to_string())).unwrap_or_default();
                            if globals.borrow().get_s("__path__").is_some() {
                                modname
                            } else {
                                modname.rsplit_once('.').map(|(p, _)| p.to_string()).unwrap_or_default()
                            }
                        }
                    }
                }
            };
            if base.is_empty() {
                return Err(self.err("ImportError", "attempted relative import with no known parent package"));
            }
            for _ in 1..level {
                base = match base.rfind('.') {
                    Some(i) => base[..i].to_string(),
                    None => return Err(self.err("ImportError", "attempted relative import beyond top-level package")),
                };
            }
            if name.is_empty() {
                base
            } else {
                format!("{}.{}", base, name)
            }
        } else {
            name.to_string()
        };
        // import each package along the dotted path
        let parts: Vec<&str> = full.split('.').collect();
        let mut cur = String::new();
        let mut top = Value::None;
        let mut leaf = Value::None;
        for (i, p) in parts.iter().enumerate() {
            if i > 0 {
                cur.push('.');
            }
            cur.push_str(p);
            let m = self.import_one(&cur)?;
            if i > 0 {
                if let Value::Module(parent) = &leaf {
                    let has = parent.dict.borrow().get_s(p).is_some();
                    if !has {
                        parent.dict.borrow_mut().set_s(p, m.clone());
                    }
                }
            }
            if i == 0 {
                top = m.clone();
            }
            leaf = m;
        }
        match fromlist {
            Value::None => Ok(if level > 0 { leaf } else { top }),
            _ => Ok(leaf),
        }
    }

    pub fn import_one(&mut self, name: &str) -> PyResult<Value> {
        let cached = self.modules.borrow().get_s(name);
        match cached {
            // a None entry blocks the import
            Some(Value::None) => {
                let e = self.err("ModuleNotFoundError", format!("import of {} halted; None in sys.modules", name));
                if let Value::Instance(i) = &e.value {
                    i.extra.borrow_mut().set_s("name", Value::str(name));
                }
                return Err(e);
            }
            Some(m) => return Ok(m),
            None => {}
        }
        if let Some(m) = crate::modules::native_module(self, name)? {
            self.modules.borrow_mut().set_s(name, m.clone());
            return Ok(m);
        }
        // a Python module: the deck's own files first, then the bundled library
        let (src, file, is_pkg) = match self.find_source(name) {
            Some(x) => x,
            None => {
                let e = self.err("ModuleNotFoundError", format!("No module named '{}'", name));
                if let Value::Instance(i) = &e.value {
                    i.extra.borrow_mut().set_s("name", Value::str(name));
                }
                return Err(e);
            }
        };
        let src = crate::builtins::decode_module_source(self, &src, &file)?;
        let globals = Rc::new(RefCell::new(Dict::new()));
        {
            let mut g = globals.borrow_mut();
            g.set_s("__name__", Value::str(name));
            g.set_s("__file__", Value::str(&file));
            g.set_s("__doc__", Value::None);
            let pkg = if is_pkg { name.to_string() } else { name.rsplit_once('.').map(|(a, _)| a.to_string()).unwrap_or_default() };
            g.set_s("__package__", Value::str(&pkg));
            // as in CPython, a module other than __main__ sees the builtins' dict
            g.set_s("__builtins__", Value::Dict(self.builtins.clone()));
        }
        let m = Value::Module(Rc::new(Module { name: ps(name), dict: globals.clone() }));
        self.modules.borrow_mut().set_s(name, m.clone());
        let r = self.run_source(&src, &file, globals);
        if let Err(e) = r {
            self.modules.borrow_mut().remove_str(&PyStr::new(name.to_string()));
            return Err(e);
        }
        // a module may put something else in sys.modules under its name
        Ok(self.modules.borrow().get_s(name).unwrap_or(m))
    }

    fn find_source(&mut self, name: &str) -> Option<(Vec<u8>, String, bool)> {
        let path = name.replace('.', "/");
        let mut dirs = vec![String::new()];
        dirs.extend(self.search_path.iter().map(|d| d.trim_end_matches('/').to_string()));
        // then the directories a program put on sys.path
        let sys_path = match self.modules.borrow().get_s("sys") {
            Some(Value::Module(m)) => m.dict.borrow().get_s("path"),
            _ => None,
        };
        if let Some(Value::List(l)) = sys_path {
            for v in l.borrow().iter() {
                if let Value::Str(s) = v {
                    let d = s.s.trim_end_matches('/').to_string();
                    if !dirs.contains(&d) {
                        dirs.push(d);
                    }
                }
            }
        }
        for d in dirs {
            for (cand, pkg) in [(format!("{}/{}.py", d, path), false), (format!("{}/{}/__init__.py", d, path), true)] {
                if let Some(s) = self.fs.read(&cand) {
                    return Some((s, cand, pkg));
                }
            }
        }
        for (cand, pkg) in [(format!("{}.py", path), false), (format!("{}/__init__.py", path), true)] {
            if let Some(s) = crate::stdlib::source(&cand) {
                return Some((s.as_bytes().to_vec(), format!("<lib>/{}", cand), pkg));
            }
        }
        None
    }

    pub fn run_source(&mut self, src: &str, file: &str, globals: Rc<RefCell<Dict>>) -> PyResult<Value> {
        let code = self.compile_source(src, file, false)?;
        self.run_module_code(code, globals)
    }

    /// Parses and compiles a module, or an expression when `eval`. The
    /// SyntaxWarnings found go through the warnings machinery first; one
    /// that a filter turns into an error is raised as a SyntaxError.
    pub fn compile_source(&mut self, src: &str, file: &str, eval: bool) -> PyResult<Rc<Code>> {
        self.from_source(src, file, || {
            if eval {
                crate::parser::parse_expression(src).map(|e| crate::compile::compile_eval(&e, file))
            } else {
                crate::parser::parse_module(src).map(|b| crate::compile::compile_module(b, file))
            }
        })
    }

    /// Runs `f`, which parses `src` and maybe compiles it, raising its errors
    /// as SyntaxError and emitting the SyntaxWarnings it records.
    pub fn from_source<T>(
        &mut self,
        src: &str,
        file: &str,
        f: impl FnOnce() -> Result<Result<T, crate::compile::CompileError>, crate::parser::SyntaxError>,
    ) -> PyResult<T> {
        crate::lexer::take_warnings();
        let code = f();
        for w in crate::lexer::take_warnings() {
            self.syntax_warning(&w, src, file)?;
        }
        match code {
            Ok(Ok(c)) => Ok(c),
            Ok(Err(e)) => {
                let x = self.parse_error(&e.syntax_error(Some(src)), file);
                set_error_text(&x, src, e.line);
                Err(x)
            }
            Err(e) => {
                let x = self.parse_error(&e, file);
                set_error_text(&x, src, e.line);
                Err(x)
            }
        }
    }

    /// warnings.warn_explicit() for a SyntaxWarning, as a SyntaxError when it raises one
    fn syntax_warning(&mut self, w: &crate::lexer::SyntaxWarn, src: &str, file: &str) -> PyResult<()> {
        let m = self.import_one("warnings")?;
        let f = self.getattr(&m, &ps("warn_explicit"))?;
        let cat = Value::Class(self.exc_class("SyntaxWarning"));
        let args = vec![Value::str(&w.msg), cat, Value::str(file), Value::Int(w.line as i64)];
        match self.call_vec(&f, args, &[]) {
            Err(e) if self.exc_is(&e, "SyntaxWarning") => {
                let e = crate::parser::SyntaxError::new(w.msg.clone(), w.line, w.col);
                let x = self.parse_error(&e, file);
                set_error_text(&x, src, w.line);
                Err(x)
            }
            r => r.map(|_| ()),
        }
    }

    /// a SyntaxError from the parser, with its column when known
    pub fn parse_error(&mut self, e: &crate::parser::SyntaxError, file: &str) -> Exc {
        let x = self.syntax_error(&e.msg, e.line, file);
        if e.col > 0 {
            // a span's end, or else the start again, as the tokenizer's errors have it
            let (end_line, end_col) = if e.end_col > 0 { (e.end_line, e.end_col) } else { (e.line, e.col) };
            for (k, v) in [("offset", e.col), ("end_lineno", end_line), ("end_offset", end_col)] {
                crate::builtins::exc_set_field(&x.value, k, Value::Int(v as i64));
            }
        }
        x
    }

    pub fn syntax_error(&mut self, msg: &str, line: u32, file: &str) -> Exc {
        let kind = if msg.starts_with("inconsistent use of tabs") {
            "TabError"
        } else if msg.contains("indent") {
            "IndentationError"
        } else {
            "SyntaxError"
        };
        let e = self.err(kind, msg.to_string());
        if let Value::Instance(i) = &e.value {
            let mut d = i.extra.borrow_mut();
            d.set_s("lineno", Value::Int(line as i64));
            d.set_s("filename", Value::str(file));
        }
        e
    }

    fn import_from(&mut self, m: &Value, name: &Rc<PyStr>) -> PyResult<Value> {
        match self.getattr(m, name) {
            Ok(v) => Ok(v),
            Err(e) if self.exc_is(&e, "AttributeError") => {
                // a submodule
                if let Value::Module(md) = m {
                    let full = format!("{}.{}", md.name.s, name.s);
                    if let Ok(v) = self.import_one(&full) {
                        md.dict.borrow_mut().set_str(name.clone(), v.clone());
                        return Ok(v);
                    }
                    let mn = md.name.s.clone();
                    return Err(self.err("ImportError", format!("cannot import name '{}' from '{}'", name.s, mn)));
                }
                Err(e)
            }
            Err(e) => Err(e),
        }
    }

    fn import_star(&mut self, m: &Value, f: &mut Frame) -> PyResult<()> {
        if let Value::Module(md) = m {
            let all = md.dict.borrow().get_s("__all__");
            let names: Vec<Value> = match all {
                Some(a) => self.to_vec(&a)?,
                None => md.dict.borrow().keys().into_iter().filter(|k| !k.as_str().map(|s| s.starts_with('_')).unwrap_or(true)).collect(),
            };
            let ns = f.ns.clone().unwrap_or_else(|| f.globals.clone());
            for n in names {
                if let Value::Str(s) = &n {
                    let v = self.getattr(m, s)?;
                    ns.borrow_mut().set_str(s.clone(), v);
                }
            }
        }
        Ok(())
    }
}

/// sets a SyntaxError's `text` to its line of the source
fn set_error_text(x: &Exc, src: &str, line: u32) {
    if let Some(t) = src.lines().nth((line as usize).wrapping_sub(1)) {
        crate::builtins::exc_set_field(&x.value, "text", Value::str(t));
    }
}
