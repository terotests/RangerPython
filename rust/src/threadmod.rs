// SPDX-License-Identifier: MIT
//! _thread, following CPython's Modules/_threadmodule.c: OS threads that
//! take turns under the GIL (see gil.rs), locks whose waits run without the
//! GIL, and the thread handles threading.Thread joins. Builds without OS
//! threads (wasm32-wasip1) have no such module.

use crate::builtins::*;
use crate::gil;
use crate::modules::{module, put_fns};
use crate::value::*;
use crate::vm::Vm;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering::Relaxed};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

/// the longest wait a lock takes (TIMEOUT_MAX), in seconds
const TIMEOUT_MAX: f64 = 9223372036.0;

/// The objects of the module, held by a NativeObj.
pub enum ThreadObj {
    Lock(Arc<RawLock>),
    /// the owner's ident (0 when free) and how often it holds it
    RLock { raw: Arc<RawLock>, owner: u64, count: u64 },
    Handle(Arc<Handle>),
}

/// A lock any thread may wait for without the GIL.
#[derive(Default)]
pub struct RawLock {
    locked: Mutex<bool>,
    freed: Condvar,
}

impl RawLock {
    fn try_acquire(&self) -> bool {
        let mut l = self.locked.lock().unwrap_or_else(|e| e.into_inner());
        !std::mem::replace(&mut *l, true)
    }

    /// waits until the lock is free, for at most `timeout` (None: no limit)
    fn acquire(&self, timeout: Option<Duration>) -> bool {
        let deadline = timeout.map(|t| Instant::now() + t);
        let mut l = self.locked.lock().unwrap_or_else(|e| e.into_inner());
        while *l {
            match deadline {
                None => l = self.freed.wait(l).unwrap_or_else(|e| e.into_inner()),
                Some(d) => {
                    let now = Instant::now();
                    if now >= d {
                        return false;
                    }
                    l = self.freed.wait_timeout(l, d - now).unwrap_or_else(|e| e.into_inner()).0;
                }
            }
        }
        *l = true;
        true
    }

    /// false when it was not locked
    fn release(&self) -> bool {
        let mut l = self.locked.lock().unwrap_or_else(|e| e.into_inner());
        if !*l {
            return false;
        }
        *l = false;
        self.freed.notify_one();
        true
    }

    fn is_locked(&self) -> bool {
        *self.locked.lock().unwrap_or_else(|e| e.into_inner())
    }
}

const NOT_STARTED: u8 = 0;
const RUNNING: u8 = 1;
const DONE: u8 = 2;

/// What _ThreadHandle wraps: a thread's ident and whether it has finished.
#[derive(Default)]
pub struct Handle {
    ident: AtomicU64,
    state: Mutex<u8>,
    changed: Condvar,
}

impl Handle {
    fn state(&self) -> u8 {
        *self.state.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn set_state(&self, s: u8) {
        *self.state.lock().unwrap_or_else(|e| e.into_inner()) = s;
        self.changed.notify_all();
    }

    /// waits for the thread to finish, at most `timeout`; whether it did
    fn wait(&self, timeout: Option<Duration>) -> bool {
        let deadline = timeout.map(|t| Instant::now() + t);
        let mut s = self.state.lock().unwrap_or_else(|e| e.into_inner());
        while *s != DONE {
            match deadline {
                None => s = self.changed.wait(s).unwrap_or_else(|e| e.into_inner()),
                Some(d) => {
                    let now = Instant::now();
                    if now >= d {
                        return false;
                    }
                    s = self.changed.wait_timeout(s, d - now).unwrap_or_else(|e| e.into_inner()).0;
                }
            }
        }
        true
    }
}

/// the handles of the non-daemon threads, which _shutdown() waits for
static NON_DAEMON: Mutex<Vec<Arc<Handle>>> = Mutex::new(Vec::new());
/// every handle there is, for a forked child to mark its threads done
static HANDLES: Mutex<Vec<std::sync::Weak<Handle>>> = Mutex::new(Vec::new());

/// In the child of a fork: only the forking thread lives on, so the handles
/// of the others are done (as CPython's _PyThread_AfterFork has it).
#[cfg(unix)]
pub fn after_fork_child() {
    let me = gil::ident();
    for h in HANDLES.lock().unwrap_or_else(|e| e.into_inner()).iter().filter_map(|w| w.upgrade()) {
        if h.ident.load(Relaxed) != me && h.state() != NOT_STARTED {
            h.set_state(DONE);
        }
    }
    NON_DAEMON.lock().unwrap_or_else(|e| e.into_inner()).clear();
}

/// the stack size new threads get (stack_size()); 0 for the default
static STACK_SIZE: AtomicUsize = AtomicUsize::new(0);
/// the Rust stack of a thread: room for the recursion limit's frames
const DEFAULT_STACK: usize = 128 << 20;

/// _rpythread, the native part of pylib/threads/_thread.py
pub fn thread_module(vm: &mut Vm) -> Value {
    let (d, m) = module("_rpythread");
    put_fns(
        &d,
        &[
            ("_thread.start_new_thread", start_new_thread),
            ("_thread.start_new", start_new_thread),
            ("_thread.start_joinable_thread", start_joinable_thread),
            ("_thread.allocate_lock", allocate_lock),
            ("_thread.allocate", allocate_lock),
            ("_thread.get_ident", get_ident),
            ("_thread.get_native_id", get_native_id),
            ("_thread._count", count),
            ("_thread.stack_size", stack_size),
            ("_thread.interrupt_main", interrupt_main),
            ("_thread.exit", exit),
            ("_thread.exit_thread", exit),
            ("_thread.daemon_threads_allowed", true_fn),
            ("_thread._is_main_interpreter", true_fn),
            ("_thread._get_main_thread_ident", main_thread_ident),
            ("_thread._make_thread_handle", make_thread_handle),
            ("_thread._shutdown", shutdown),
            ("_thread._local_switch", local_switch),
        ],
    );
    let mut d = d.borrow_mut();
    let lock = lock_class(vm);
    d.set_s("LockType", Value::Class(lock.clone()));
    d.set_s("lock", Value::Class(lock));
    d.set_s("RLock", Value::Class(rlock_class(vm)));
    d.set_s("_ThreadHandle", Value::Class(handle_class(vm)));
    d.set_s("error", Value::Class(vm.exc_class("RuntimeError")));
    d.set_s("TIMEOUT_MAX", Value::Float(TIMEOUT_MAX));
    drop(d);
    m
}

/// a class of the module, made once
fn class(vm: &mut Vm, key: &'static str, name: &str, methods: &[(&'static str, NativeFnPtr)], new: NativeFnPtr) -> Rc<Class> {
    if let Some(c) = vm.classes.get(key) {
        return c.clone();
    }
    let c = vm.native_class(key);
    *c.name.borrow_mut() = ps(name);
    *c.qualname.borrow_mut() = ps(name);
    c.dict.borrow_mut().set_s("__module__", Value::str("_thread"));
    def_static(&c, "__new__", new);
    for (n, f) in methods {
        def(&c, n, *f);
    }
    c
}

fn lock_class(vm: &mut Vm) -> Rc<Class> {
    let methods: &[(&'static str, NativeFnPtr)] = &[
        ("lock.acquire", lock_acquire),
        ("lock.acquire_lock", lock_acquire),
        ("lock.__enter__", lock_acquire),
        ("lock.release", lock_release),
        ("lock.release_lock", lock_release),
        ("lock.__exit__", lock_release),
        ("lock.locked", lock_locked),
        ("lock.locked_lock", lock_locked),
        ("lock._at_fork_reinit", lock_reinit),
        ("lock.__repr__", lock_repr),
    ];
    class(vm, "_thread.lock", "lock", methods, lock_new)
}

fn rlock_class(vm: &mut Vm) -> Rc<Class> {
    let methods: &[(&'static str, NativeFnPtr)] = &[
        ("RLock.acquire", rlock_acquire),
        ("RLock.__enter__", rlock_acquire),
        ("RLock.release", rlock_release),
        ("RLock.__exit__", rlock_release),
        ("RLock._is_owned", rlock_is_owned),
        ("RLock._release_save", rlock_release_save),
        ("RLock._acquire_restore", rlock_acquire_restore),
        ("RLock._recursion_count", rlock_recursion_count),
        ("RLock._at_fork_reinit", rlock_reinit),
        ("RLock.__repr__", rlock_repr),
    ];
    class(vm, "_thread.RLock", "RLock", methods, rlock_new)
}

fn handle_class(vm: &mut Vm) -> Rc<Class> {
    let methods: &[(&'static str, NativeFnPtr)] = &[
        ("_ThreadHandle.join", handle_join),
        ("_ThreadHandle.is_done", handle_is_done),
        ("_ThreadHandle._set_done", handle_set_done),
        ("_ThreadHandle.__repr__", handle_repr),
    ];
    let fresh = vm.classes.get("_thread._ThreadHandle").is_none();
    let c = class(vm, "_thread._ThreadHandle", "_ThreadHandle", methods, handle_new);
    if fresh {
        // ident, a read-only property
        def(&c, "_thread._ThreadHandle.ident", handle_ident);
        let getter = c.dict.borrow_mut().remove_str(&ps("ident"));
        let prop = vm.builtins.borrow().get_s("property");
        if let (Some(g), Some(prop)) = (getter, prop) {
            if let Ok(p) = vm.call_vec(&prop, vec![g], &[]) {
                c.dict.borrow_mut().set_s("ident", p);
            }
        }
    }
    c
}

fn handle_ident(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let h = handle_of(vm, a)?;
    Ok(Value::Int(h.ident.load(Relaxed) as i64))
}

fn new_obj(cls: Rc<Class>, kind: ThreadObj) -> Value {
    let o = Rc::new(NativeObj { class: cls, gc: Default::default(), kind: std::cell::RefCell::new(ObjKind::Thread(kind)) });
    crate::gc::track_obj(&o);
    Value::Obj(o)
}

fn new_lock(vm: &mut Vm) -> Value {
    let c = lock_class(vm);
    new_obj(c, ThreadObj::Lock(Arc::default()))
}

fn new_handle(vm: &mut Vm, h: Arc<Handle>) -> Value {
    {
        let mut l = HANDLES.lock().unwrap_or_else(|e| e.into_inner());
        l.retain(|w| w.strong_count() > 0);
        l.push(Arc::downgrade(&h));
    }
    let c = handle_class(vm);
    new_obj(c, ThreadObj::Handle(h))
}

/// the ThreadObj of `self`, or a TypeError naming what was wanted
fn this<R>(vm: &mut Vm, a: &[Value], what: &str, f: impl FnOnce(&mut ThreadObj) -> Option<R>) -> PyResult<R> {
    if let Some(Value::Obj(o)) = a.first() {
        if let ObjKind::Thread(t) = &mut *o.kind.borrow_mut() {
            if let Some(r) = f(t) {
                return Ok(r);
            }
        }
    }
    let got = a.first().map(|v| vm.type_name(v)).unwrap_or_default();
    Err(vm.type_error(format!("descriptor requires a '_thread.{}' object but received a '{}'", what, got)))
}

fn raw_lock(vm: &mut Vm, a: &[Value]) -> PyResult<Arc<RawLock>> {
    this(vm, a, "lock", |t| match t {
        ThreadObj::Lock(l) => Some(l.clone()),
        _ => None,
    })
}

/// acquire(blocking=True, timeout=-1)'s arguments: None for a call that
/// must not wait, else how long it may (None: without limit)
fn wait_args(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Option<Option<Duration>>> {
    let blocking = match arg_or_kw(a, 1, k, "blocking") {
        Some(b) => vm.truthy(&b)?,
        None => true,
    };
    let timeout = match arg_or_kw(a, 2, k, "timeout") {
        Some(t) => vm.to_f64(&t)?,
        None => -1.0,
    };
    if !blocking && timeout != -1.0 {
        return Err(vm.value_error("can't specify a timeout for a non-blocking call"));
    }
    if timeout < 0.0 && timeout != -1.0 {
        return Err(vm.value_error("timeout value must be a non-negative number"));
    }
    if !blocking {
        return Ok(None);
    }
    if timeout == -1.0 {
        return Ok(Some(None));
    }
    if timeout > TIMEOUT_MAX {
        return Err(vm.err("OverflowError", "timeout value is too large"));
    }
    Ok(Some(Some(Duration::from_secs_f64(timeout))))
}

/// takes `raw`, waiting without the GIL as `wait` allows
fn acquire(vm: &mut Vm, raw: &Arc<RawLock>, wait: Option<Option<Duration>>) -> bool {
    if raw.try_acquire() {
        return true;
    }
    match wait {
        None => false,
        Some(Some(d)) if d.is_zero() => false,
        Some(t) => {
            let raw = raw.clone();
            vm.allow_threads(move || raw.acquire(t))
        }
    }
}

fn lock_new(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    if a.len() > 1 || !k.is_empty() {
        return Err(vm.type_error("lock() takes no arguments"));
    }
    Ok(new_lock(vm))
}

fn allocate_lock(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "allocate_lock", a, 0, 0)?;
    Ok(new_lock(vm))
}

fn lock_acquire(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let raw = raw_lock(vm, a)?;
    let wait = wait_args(vm, a, k)?;
    Ok(Value::Bool(acquire(vm, &raw, wait)))
}

fn lock_release(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let raw = raw_lock(vm, a)?;
    if !raw.release() {
        return Err(vm.err("RuntimeError", "release unlocked lock"));
    }
    Ok(Value::None)
}

fn lock_locked(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let raw = raw_lock(vm, a)?;
    Ok(Value::Bool(raw.is_locked()))
}

fn lock_reinit(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    this(vm, a, "lock", |t| match t {
        ThreadObj::Lock(l) => {
            *l = Arc::default();
            Some(())
        }
        _ => None,
    })?;
    Ok(Value::None)
}

fn type_label(vm: &mut Vm, v: &Value) -> String {
    let c = vm.class_of(v);
    let m = match c.dict.borrow().get_s("__module__") {
        Some(Value::Str(m)) => m.s.to_string(),
        _ => String::new(),
    };
    format!("{}.{}", m, c.qualname.borrow().s)
}

fn lock_repr(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let raw = raw_lock(vm, a)?;
    let state = if raw.is_locked() { "locked" } else { "unlocked" };
    let label = type_label(vm, &a[0]);
    Ok(Value::string(format!("<{} {} object at 0x{:012x}>", state, label, ptr_id(&a[0]))))
}

// -------------------------------------------------------------------------
// RLock: owned by a thread, which may take it again

fn rlock_new(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let c = match a.first() {
        Some(Value::Class(c)) => c.clone(),
        _ => rlock_class(vm),
    };
    Ok(new_obj(c, ThreadObj::RLock { raw: Arc::default(), owner: 0, count: 0 }))
}

/// (lock, owner, count) of an RLock
fn rlock_parts(vm: &mut Vm, a: &[Value]) -> PyResult<(Arc<RawLock>, u64, u64)> {
    this(vm, a, "RLock", |t| match t {
        ThreadObj::RLock { raw, owner, count } => Some((raw.clone(), *owner, *count)),
        _ => None,
    })
}

fn rlock_set(a: &[Value], new_owner: u64, new_count: u64) {
    if let Some(Value::Obj(o)) = a.first() {
        if let ObjKind::Thread(ThreadObj::RLock { owner, count, .. }) = &mut *o.kind.borrow_mut() {
            *owner = new_owner;
            *count = new_count;
        }
    }
}

fn rlock_acquire(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let (raw, owner, count) = rlock_parts(vm, a)?;
    let wait = wait_args(vm, a, k)?;
    let me = gil::ident();
    if owner == me && count > 0 {
        if count == u64::MAX {
            return Err(vm.err("OverflowError", "Internal lock count overflowed"));
        }
        rlock_set(a, me, count + 1);
        return Ok(Value::Bool(true));
    }
    if !acquire(vm, &raw, wait) {
        return Ok(Value::Bool(false));
    }
    rlock_set(a, me, 1);
    Ok(Value::Bool(true))
}

fn rlock_release(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let (raw, owner, count) = rlock_parts(vm, a)?;
    if count == 0 || owner != gil::ident() {
        return Err(vm.err("RuntimeError", "cannot release un-acquired lock"));
    }
    if count == 1 {
        rlock_set(a, 0, 0);
        raw.release();
    } else {
        rlock_set(a, owner, count - 1);
    }
    Ok(Value::None)
}

fn rlock_is_owned(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let (_, owner, count) = rlock_parts(vm, a)?;
    Ok(Value::Bool(count > 0 && owner == gil::ident()))
}

fn rlock_recursion_count(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let (_, owner, count) = rlock_parts(vm, a)?;
    Ok(Value::Int(if owner == gil::ident() { count as i64 } else { 0 }))
}

/// releases the lock however often it is held: (count, owner) for
/// _acquire_restore (what Condition.wait does)
fn rlock_release_save(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let (raw, owner, count) = rlock_parts(vm, a)?;
    if count == 0 {
        return Err(vm.err("RuntimeError", "cannot release un-acquired lock"));
    }
    rlock_set(a, 0, 0);
    raw.release();
    Ok(Value::tuple(vec![Value::Int(count as i64), Value::Int(owner as i64)]))
}

fn rlock_acquire_restore(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let (raw, _, _) = rlock_parts(vm, a)?;
    need(vm, a, 2)?;
    let state = vm.to_vec(&a[1])?;
    if state.len() != 2 {
        return Err(vm.type_error("_acquire_restore() argument must be a (count, owner) pair"));
    }
    let count = vm.to_index(&state[0])? as u64;
    let owner = vm.to_index(&state[1])? as u64;
    acquire(vm, &raw, Some(None));
    rlock_set(a, owner, count);
    Ok(Value::None)
}

fn rlock_reinit(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    this(vm, a, "RLock", |t| match t {
        ThreadObj::RLock { raw, owner, count } => {
            *raw = Arc::default();
            *owner = 0;
            *count = 0;
            Some(())
        }
        _ => None,
    })?;
    Ok(Value::None)
}

fn rlock_repr(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let (raw, owner, count) = rlock_parts(vm, a)?;
    let state = if raw.is_locked() { "locked" } else { "unlocked" };
    let label = type_label(vm, &a[0]);
    Ok(Value::string(format!("<{} {} object owner={} count={} at 0x{:012x}>", state, label, owner, count, ptr_id(&a[0]))))
}

// -------------------------------------------------------------------------
// thread handles

fn handle_of(vm: &mut Vm, a: &[Value]) -> PyResult<Arc<Handle>> {
    this(vm, a, "_ThreadHandle", |t| match t {
        ThreadObj::Handle(h) => Some(h.clone()),
        _ => None,
    })
}

fn handle_new(vm: &mut Vm, _a: &[Value], _k: Kw) -> PyResult<Value> {
    Ok(new_handle(vm, Arc::default()))
}

fn handle_join(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let h = handle_of(vm, a)?;
    let timeout = match arg_or_kw(a, 1, k, "timeout") {
        None | Some(Value::None) => None,
        Some(t) => {
            let t = vm.to_f64(&t)?;
            (t >= 0.0).then(|| Duration::from_secs_f64(t.min(TIMEOUT_MAX)))
        }
    };
    match h.state() {
        NOT_STARTED => return Err(vm.err("RuntimeError", "thread not started")),
        DONE => return Ok(Value::None),
        _ => {}
    }
    if h.ident.load(Relaxed) == gil::ident() {
        return Err(vm.err("RuntimeError", "Cannot join current thread"));
    }
    vm.allow_threads(move || h.wait(timeout));
    Ok(Value::None)
}

fn handle_is_done(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let h = handle_of(vm, a)?;
    Ok(Value::Bool(h.state() == DONE))
}

fn handle_set_done(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let h = handle_of(vm, a)?;
    h.set_state(DONE);
    Ok(Value::None)
}

fn handle_repr(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let h = handle_of(vm, a)?;
    Ok(Value::string(format!("<_thread._ThreadHandle object: ident={}>", h.ident.load(Relaxed))))
}

fn make_thread_handle(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "_make_thread_handle", a, 1, 1)?;
    let h = Arc::new(Handle::default());
    h.ident.store(vm.to_index(&a[0])? as u64, Relaxed);
    h.set_state(RUNNING);
    Ok(new_handle(vm, h))
}

/// waits for the threads that are not daemons
fn shutdown(vm: &mut Vm, _a: &[Value], _k: Kw) -> PyResult<Value> {
    let me = gil::ident();
    loop {
        let next = {
            let mut l = NON_DAEMON.lock().unwrap_or_else(|e| e.into_inner());
            l.retain(|h| h.state() != DONE && h.ident.load(Relaxed) != me);
            l.first().cloned()
        };
        let Some(h) = next else { break };
        vm.allow_threads(move || h.wait(None));
    }
    Ok(Value::None)
}

// -------------------------------------------------------------------------
// starting threads

/// What a new thread runs, made by its creator under the GIL.
struct Job {
    vm: Vm,
    func: Value,
    args: Vec<Value>,
    kwargs: Vec<(Rc<PyStr>, Value)>,
    handle: Arc<Handle>,
    ident: u64,
}

/// A Job crossing to its thread.
struct SendJob(Job);

// SAFETY: the creator gives up the Job's Rcs when the thread starts and
// the thread touches them only once it holds the GIL, so they are never
// used by two threads at once.
unsafe impl Send for SendJob {}

fn start(vm: &mut Vm, func: Value, args: Vec<Value>, kwargs: Vec<(Rc<PyStr>, Value)>, handle: Arc<Handle>) -> PyResult<u64> {
    let ident = gil::new_ident();
    if handle.state() != NOT_STARTED {
        return Err(vm.err("RuntimeError", "thread already started"));
    }
    handle.ident.store(ident, Relaxed);
    handle.set_state(RUNNING);
    let job = SendJob(Job { vm: vm.new_thread(), func, args, kwargs, handle: handle.clone(), ident });
    let size = match STACK_SIZE.load(Relaxed) {
        0 => DEFAULT_STACK,
        n => n,
    };
    gil::THREADS.fetch_add(1, Relaxed);
    gil::set_alive(ident, true);
    let spawned = std::thread::Builder::new().stack_size(size).spawn(move || run_thread(job));
    if spawned.is_err() {
        gil::THREADS.fetch_sub(1, Relaxed);
        gil::set_alive(ident, false);
        handle.set_state(NOT_STARTED);
        return Err(vm.err("RuntimeError", "can't start new thread"));
    }
    Ok(ident)
}

/// the body of every thread: runs the function under the GIL, reporting
/// what it raised as CPython's t_bootstrap does
fn run_thread(job: SendJob) {
    let Job { mut vm, func, args, kwargs, handle, ident } = job.0;
    gil::set_ident(ident);
    vm.take_gil();
    let r = vm.call_vec(&func, args, &kwargs);
    if let Err(e) = r {
        if !vm.exc_is(&e, "SystemExit") {
            let f = vm.repr(&func).unwrap_or_default();
            vm.unraisable_msg(e, &Value::None, &format!("Exception ignored in thread started by {}", f));
        }
    }
    drop((func, kwargs));
    vm.run_finalizers();
    gil::THREADS.fetch_sub(1, Relaxed);
    gil::set_alive(ident, false);
    vm.exit_thread();
    handle.set_state(DONE);
}

/// the function and its arguments of a start_new_thread call
fn call_parts(vm: &mut Vm, f: &Value, args: &Value, kwargs: Option<&Value>) -> PyResult<(Vec<Value>, Vec<(Rc<PyStr>, Value)>)> {
    if !is_callable(vm, f) {
        return Err(vm.type_error("first arg must be callable"));
    }
    let Value::Tuple(args) = args else { return Err(vm.type_error("2nd arg must be a tuple")) };
    let mut kw = vec![];
    match kwargs {
        None => {}
        Some(Value::Dict(d)) => {
            for (k, v) in d.borrow().items() {
                let Value::Str(k) = k else { return Err(vm.type_error("keywords must be strings")) };
                kw.push((k, v));
            }
        }
        Some(_) => return Err(vm.type_error("optional 3rd arg must be a dictionary")),
    }
    Ok((args.to_vec(), kw))
}

fn start_new_thread(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    if !k.is_empty() {
        return Err(vm.type_error("start_new_thread() takes no keyword arguments"));
    }
    check_nargs(vm, "start_new_thread", a, 2, 3)?;
    let (args, kw) = call_parts(vm, &a[0], &a[1], a.get(2))?;
    let ident = start(vm, a[0].clone(), args, kw, Arc::default())?;
    Ok(Value::Int(ident as i64))
}

/// start_joinable_thread(function, handle=None, daemon=True)
fn start_joinable_thread(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let Some(func) = arg_or_kw(a, 0, k, "function") else {
        return Err(vm.type_error("start_joinable_thread() missing required argument 'function' (pos 1)"));
    };
    if !is_callable(vm, &func) {
        return Err(vm.type_error("thread function must be callable"));
    }
    let handle = match arg_or_kw(a, 1, k, "handle") {
        None | Some(Value::None) => new_handle(vm, Arc::default()),
        Some(h) => h,
    };
    let h = handle_of(vm, std::slice::from_ref(&handle))?;
    let daemon = match arg_or_kw(a, 2, k, "daemon") {
        Some(d) => vm.truthy(&d)?,
        None => true,
    };
    start(vm, func, vec![], vec![], h.clone())?;
    if !daemon {
        NON_DAEMON.lock().unwrap_or_else(|e| e.into_inner()).push(h);
    }
    Ok(handle)
}

// -------------------------------------------------------------------------
// the rest of the module

fn get_ident(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "get_ident", a, 0, 0)?;
    Ok(Value::Int(gil::ident() as i64))
}

/// the OS's number for the thread (Linux's tid), else get_ident()'s
fn get_native_id(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "get_native_id", a, 0, 0)?;
    let tid = std::fs::read_link("/proc/thread-self").ok().and_then(|p| p.file_name()?.to_str()?.parse::<i64>().ok());
    Ok(Value::Int(tid.unwrap_or(gil::ident() as i64)))
}

fn count(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "_count", a, 0, 0)?;
    Ok(Value::Int(gil::THREADS.load(Relaxed) as i64))
}

/// stack_size([size]): the size new threads get; the previous one
fn stack_size(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "stack_size", a, 0, 1)?;
    let old = STACK_SIZE.load(Relaxed);
    if let Some(v) = a.first() {
        let n = vm.to_index(v)?;
        if n != 0 && n < 32768 {
            return Err(vm.value_error(format!("size not valid: {} bytes", n)));
        }
        if n < 0 {
            return Err(vm.value_error("size must be 0 or a positive value"));
        }
        STACK_SIZE.store(n as usize, Relaxed);
    }
    Ok(Value::Int(old as i64))
}

/// interrupt_main(signum=SIGINT): the main thread raises KeyboardInterrupt
/// as the default SIGINT handler would
fn interrupt_main(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let signum = match arg_or_kw(a, 0, k, "signum") {
        Some(s) => vm.to_index(&s)?,
        None => 2,
    };
    if !(1..65).contains(&signum) {
        return Err(vm.value_error("signal number out of range"));
    }
    if signum == 2 {
        gil::interrupt_main();
    }
    Ok(Value::None)
}

fn exit(vm: &mut Vm, _a: &[Value], _k: Kw) -> PyResult<Value> {
    let cls = vm.exc_class("SystemExit");
    Err(Exc { value: vm.new_exc(&cls, vec![]) })
}

fn true_fn(_vm: &mut Vm, _a: &[Value], _k: Kw) -> PyResult<Value> {
    Ok(Value::Bool(true))
}

fn main_thread_ident(_vm: &mut Vm, _a: &[Value], _k: Kw) -> PyResult<Value> {
    Ok(Value::Int(gil::MAIN_IDENT.load(Relaxed) as i64))
}

/// _local_switch(local, impl): makes the attributes of a _thread._local the
/// running thread's. `impl` is [owner ident, {ident: attributes} of the
/// other threads, ...]: the instance's own dict holds the owner's, which
/// are swapped for this thread's (new ones when it has none, which is
/// returned so that __init__ runs for them). Those of finished threads go.
fn local_switch(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    need(vm, a, 2)?;
    let (Value::Instance(inst), Value::List(state)) = (&a[0], &a[1]) else {
        return Err(vm.type_error("_local_switch() needs a local and its state"));
    };
    let me = gil::ident() as i64;
    let (owner, dicts) = {
        let s = state.borrow();
        match (s.first(), s.get(1)) {
            (Some(Value::Int(o)), Some(Value::Dict(d))) => (*o, d.clone()),
            _ => return Err(vm.type_error("_local_switch(): bad state")),
        }
    };
    if owner == me {
        return Ok(Value::Bool(false));
    }
    let gone: Vec<Value> = dicts.borrow().keys().into_iter().filter(|k| !matches!(k, Value::Int(i) if gil::alive(*i as u64))).collect();
    for k in gone {
        vm.dict_del(&dicts, &k)?;
    }
    let (mine, fresh) = match vm.dict_del(&dicts, &Value::Int(me))? {
        Some(Value::Dict(d)) => (d, false),
        _ => (Rc::new(std::cell::RefCell::new(Dict::new())), true),
    };
    std::mem::swap(&mut *inst.dict().borrow_mut(), &mut *mine.borrow_mut());
    if gil::alive(owner as u64) {
        vm.dict_set(&dicts, Value::Int(owner), Value::Dict(mine))?;
    }
    state.borrow_mut()[0] = Value::Int(me);
    Ok(Value::Bool(fresh))
}
