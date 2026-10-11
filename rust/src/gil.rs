// SPDX-License-Identifier: MIT
//! The global interpreter lock, as in CPython: Python code may run on many
//! OS threads, but only the thread holding the GIL touches interpreter
//! state. That is what makes the plain `Rc` and `RefCell` of the object
//! model sound with threads: no two threads ever use them at once.
//!
//! Each thread has a `Vm` of its own, holding what CPython keeps in a
//! thread state (frames, the exception being handled, recursion depth) and
//! clones of the `Rc`s every thread shares (types, modules, builtins). The
//! interpreter-wide state the `Vm` owns by value (the filesystem, output
//! buffers, class tables...) moves with the GIL: a thread dropping it parks
//! that state in the lock and the thread taking it moves it into its own
//! `Vm`, so no `Vm` is ever reached from two threads.
//!
//! The holder lets others run at a switch interval (sys.setswitchinterval):
//! a thread that waited that long for the GIL sets a drop request, which
//! the instruction loop sees at its next backward jump or call, and around
//! blocking calls (lock waits, sleeps, joins), which run without it.
//!
//! State that would be a `static` in C lives in `gil_static!` statics:
//! process-wide, read and written only by the GIL holder.

use crate::value::Value;
use crate::vm::{NameMap, Vm};
use std::cell::OnceCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, AtomicU8, AtomicUsize, Ordering::Relaxed};
use std::sync::{Condvar, Mutex, MutexGuard};
use std::time::Duration;

/// A process-wide static that only the GIL holder uses.
pub struct GilConst<T>(T);

// SAFETY: every thread running interpreter code holds the GIL, and the
// only code running without it (Vm::allow_threads' closures) touches no
// interpreter state, so the value is never used by two threads at once.
unsafe impl<T> Sync for GilConst<T> {}

impl<T> GilConst<T> {
    pub const fn new(v: T) -> Self {
        GilConst(v)
    }

    /// runs `f` on the value; the caller holds the GIL
    #[inline(always)]
    pub fn with<R>(&'static self, f: impl FnOnce(&T) -> R) -> R {
        f(&self.0)
    }

    /// `with`, for code written for a thread_local's try_with: never fails
    #[inline(always)]
    pub fn try_with<R>(&'static self, f: impl FnOnce(&T) -> R) -> Result<R, ()> {
        Ok(f(&self.0))
    }
}

/// A process-wide static made on first use, that only the GIL holder uses.
pub struct GilLazy<T> {
    cell: OnceCell<T>,
    init: fn() -> T,
}

// SAFETY: as for GilConst; the first use, which makes the value, is the
// GIL holder's too.
unsafe impl<T> Sync for GilLazy<T> {}

impl<T> GilLazy<T> {
    pub const fn new(init: fn() -> T) -> Self {
        GilLazy { cell: OnceCell::new(), init }
    }

    /// runs `f` on the value; the caller holds the GIL
    #[inline]
    pub fn with<R>(&'static self, f: impl FnOnce(&T) -> R) -> R {
        f(self.cell.get_or_init(self.init))
    }

    /// `with`, for code written for a thread_local's try_with: never fails
    #[inline]
    pub fn try_with<R>(&'static self, f: impl FnOnce(&T) -> R) -> Result<R, ()> {
        Ok(self.with(f))
    }
}

/// Declares statics shared by all the interpreter's threads under the GIL,
/// written as `thread_local!` declarations are: `X.with(|x| ...)`.
#[macro_export]
macro_rules! gil_static {
    () => {};
    ($(#[$m:meta])* $vis:vis static $name:ident: $t:ty = const { $e:expr }; $($rest:tt)*) => {
        $(#[$m])* $vis static $name: $crate::gil::GilConst<$t> = $crate::gil::GilConst::new($e);
        $crate::gil_static!($($rest)*);
    };
    ($(#[$m:meta])* $vis:vis static $name:ident: $t:ty = $e:expr; $($rest:tt)*) => {
        $(#[$m])* $vis static $name: $crate::gil::GilLazy<$t> = $crate::gil::GilLazy::new(|| $e);
        $crate::gil_static!($($rest)*);
    };
}

/// The interpreter-wide state a `Vm` owns by value, which travels with the
/// GIL from thread to thread.
#[derive(Default)]
pub struct Shared {
    out: String,
    err_out: String,
    fs: crate::fs::Fs,
    random: crate::modules::Random,
    max_depth: u32,
    argv: Vec<String>,
    search_path: Vec<String>,
    host: crate::process::Host,
    exc: NameMap<Rc<crate::value::Class>>,
    classes: NameMap<Rc<crate::value::Class>>,
}

impl Shared {
    fn swap(&mut self, vm: &mut Vm) {
        use std::mem::swap;
        swap(&mut self.out, &mut vm.out);
        swap(&mut self.err_out, &mut vm.err_out);
        swap(&mut self.fs, &mut vm.fs);
        swap(&mut self.random, &mut vm.random);
        swap(&mut self.max_depth, &mut vm.max_depth);
        swap(&mut self.argv, &mut vm.argv);
        swap(&mut self.search_path, &mut vm.search_path);
        swap(&mut self.host, &mut vm.host);
        swap(&mut self.exc, &mut vm.exc);
        swap(&mut self.classes, &mut vm.classes);
    }
}

/// The shared state between the thread that dropped the GIL and the one
/// that takes it.
struct Parked(Box<Shared>);

// SAFETY: the Rcs inside are handed from the thread dropping the GIL to the
// thread taking it, and only ever used by the holder.
unsafe impl Send for Parked {}

struct State {
    locked: bool,
    /// the GIL holder's thread identity (get_ident)
    holder: u64,
    /// bumped by each take, so a waiter sees whether the holder changed
    switches: u64,
    parked: Option<Parked>,
}

static GIL: Mutex<State> = Mutex::new(State { locked: false, holder: 0, switches: 0, parked: None });
/// signalled when the GIL is dropped
static DROPPED: Condvar = Condvar::new();
/// signalled when the GIL is taken
static TAKEN: Condvar = Condvar::new();

/// set when the instruction loop must look up from its work: bit 0, a
/// thread waiting for the GIL asks the holder to drop it; bit 1,
/// interrupt_main() was called
pub static EVAL_BREAKER: AtomicU8 = AtomicU8::new(0);
const DROP_REQUEST: u8 = 1;
const INTERRUPT: u8 = 2;

/// sys.setswitchinterval, in nanoseconds
static INTERVAL_NS: AtomicU64 = AtomicU64::new(5_000_000);
/// the threads started by _thread that have not finished (_count())
pub static THREADS: AtomicUsize = AtomicUsize::new(0);
/// get_ident() of the thread that runs the main program
pub static MAIN_IDENT: AtomicU64 = AtomicU64::new(0);

/// sys.getswitchinterval() in seconds
pub fn switch_interval() -> f64 {
    INTERVAL_NS.load(Relaxed) as f64 / 1e9
}

pub fn set_switch_interval(secs: f64) {
    INTERVAL_NS.store(((secs * 1e9) as u64).max(1), Relaxed);
}

static NEXT_IDENT: AtomicU64 = AtomicU64::new(1);

thread_local! {
    /// get_ident() of this OS thread, 0 until it is needed
    static IDENT: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// get_ident(): a number for the running OS thread, unique while it lives
pub fn ident() -> u64 {
    IDENT.with(|i| {
        if i.get() == 0 {
            i.set(new_ident());
        }
        i.get()
    })
}

/// a number for a thread about to start
pub fn new_ident() -> u64 {
    NEXT_IDENT.fetch_add(1, Relaxed)
}

/// gives the running thread the ident its creator chose
pub fn set_ident(n: u64) {
    IDENT.with(|i| i.set(n));
}

/// the idents of the threads _thread started that are still running
static ALIVE: Mutex<Vec<u64>> = Mutex::new(Vec::new());

pub fn set_alive(ident: u64, alive: bool) {
    let mut l = ALIVE.lock().unwrap_or_else(|e| e.into_inner());
    l.retain(|&i| i != ident);
    if alive {
        l.push(ident);
    }
}

/// whether the thread `ident` is running (the main one always is)
pub fn alive(ident: u64) -> bool {
    ident == MAIN_IDENT.load(Relaxed) || ALIVE.lock().unwrap_or_else(|e| e.into_inner()).contains(&ident)
}

/// Runs `fork` (which returns 0 in the child) holding the GIL's and the
/// thread table's locks, so that the child gets them unlocked; in the child
/// the forking thread becomes the main and only one.
#[cfg(unix)]
pub fn fork_with(fork: impl FnOnce() -> i32) -> i32 {
    let mut g = lock();
    let mut alive = ALIVE.lock().unwrap_or_else(|e| e.into_inner());
    let pid = fork();
    if pid == 0 {
        let me = ident();
        alive.clear();
        THREADS.store(0, Relaxed);
        MAIN_IDENT.store(me, Relaxed);
        if g.locked {
            g.holder = me;
        }
        EVAL_BREAKER.fetch_and(!DROP_REQUEST, Relaxed);
    }
    drop(alive);
    drop(g);
    pid
}

/// makes the main thread see KeyboardInterrupt (_thread.interrupt_main)
pub fn interrupt_main() {
    EVAL_BREAKER.fetch_or(INTERRUPT, Relaxed);
}

fn lock() -> MutexGuard<'static, State> {
    GIL.lock().unwrap_or_else(|e| e.into_inner())
}

/// waits for the GIL and takes it; the parked shared state, if any
fn take(mut g: MutexGuard<'static, State>) -> Option<Parked> {
    while g.locked {
        let seen = g.switches;
        let wait = Duration::from_nanos(INTERVAL_NS.load(Relaxed));
        let (ng, r) = DROPPED.wait_timeout(g, wait).unwrap_or_else(|e| e.into_inner());
        g = ng;
        // the holder kept it a whole interval: ask it to let go
        if r.timed_out() && g.locked && g.switches == seen {
            EVAL_BREAKER.fetch_or(DROP_REQUEST, Relaxed);
        }
    }
    g.locked = true;
    g.holder = ident();
    g.switches += 1;
    EVAL_BREAKER.fetch_and(!DROP_REQUEST, Relaxed);
    TAKEN.notify_all();
    g.parked.take()
}

/// drops the GIL with `parked` in it; the guard is kept for the caller
fn drop_gil(mut g: MutexGuard<'static, State>, parked: Option<Parked>) -> MutexGuard<'static, State> {
    g.parked = parked;
    g.locked = false;
    g.holder = 0;
    DROPPED.notify_one();
    g
}

/// The main program is done and its objects are gone. With no thread
/// left the GIL is dropped; otherwise it is kept for good, leaving daemon
/// threads waiting for it while the process exits, as CPython's
/// finalization does.
pub fn release_main() {
    if THREADS.load(Relaxed) == 0 {
        drop(drop_gil(lock(), None));
    }
}

impl Vm {
    /// A Vm for a new thread of this interpreter: the shared objects are
    /// this one's, the thread state is fresh, and the state that moves with
    /// the GIL arrives when the thread takes it.
    pub fn new_thread(&self) -> Vm {
        Vm {
            t: self.t.clone(),
            n: self.n.clone(),
            builtins: self.builtins.clone(),
            exc: NameMap::default(),
            modules: self.modules.clone(),
            handled: vec![],
            out: String::new(),
            err_out: String::new(),
            fs: Default::default(),
            depth: 0,
            max_depth: 0,
            argv: vec![],
            steps: 0,
            max_steps: self.max_steps,
            main_globals: self.main_globals.clone(),
            repr_stack: vec![],
            random: Default::default(),
            start_time: self.start_time,
            last_return: None,
            frames: Vec::with_capacity(64),
            frame_objs: Vec::with_capacity(64),
            stack_pool: Vec::new(),
            echo: self.echo,
            search_path: vec![],
            bare_reraise: false,
            classes: NameMap::default(),
            host: Default::default(),
            in_gil: true,
        }
    }

    /// Takes the GIL for this thread's Vm, with the state the last holder
    /// parked. The program's first Vm (the main thread's) finds none and
    /// keeps its own; taking it again on the thread that holds it is a no-op.
    pub fn take_gil(&mut self) {
        let g = lock();
        if g.locked && g.holder == ident() {
            return;
        }
        if let Some(Parked(mut s)) = take(g) {
            s.swap(self);
        }
    }

    /// Drops the GIL, parking the shared state for the next holder.
    pub fn drop_gil(&mut self) {
        let mut s = Box::<Shared>::default();
        s.swap(self);
        drop(drop_gil(lock(), Some(Parked(s))));
    }

    /// The end of a thread: the GIL is dropped with the shared state parked
    /// and the rest of this Vm freed while it is still held.
    pub fn exit_thread(mut self) {
        let mut s = Box::<Shared>::default();
        s.swap(&mut self);
        let g = lock();
        drop(self);
        drop(drop_gil(g, Some(Parked(s))));
    }

    /// What CPython's finalization does first: threading._shutdown() waits
    /// for the threads that are not daemons.
    pub fn shutdown_threads(&mut self) {
        let threading = self.modules.borrow().get_s("threading");
        let Some(m) = threading else { return };
        // (the single-thread threading of builds without threads has none)
        let r = match self.getattr_opt(&m, &crate::builtins::ps("_shutdown")) {
            Ok(Some(f)) => self.call_vec(&f, vec![], &[]).map(drop),
            Ok(None) => Ok(()),
            Err(e) => Err(e),
        };
        if let Err(e) = r {
            self.unraisable(e, &Value::None);
        }
    }

    /// Runs `f`, which must not touch any Python object or interpreter
    /// state, with the GIL dropped so that other threads run meanwhile (a
    /// blocking wait or a sleep).
    pub fn allow_threads<R>(&mut self, f: impl FnOnce() -> R) -> R {
        if !self.in_gil || THREADS.load(Relaxed) == 0 {
            return f();
        }
        self.drop_gil();
        let r = f();
        self.take_gil();
        r
    }

    /// The instruction loop's check at backward jumps and calls: costs one
    /// load while there is nothing to do.
    #[inline(always)]
    pub fn eval_breaker(&mut self) -> crate::value::PyResult<()> {
        if EVAL_BREAKER.load(Relaxed) != 0 {
            return self.handle_eval_breaker();
        }
        Ok(())
    }

    #[cold]
    #[inline(never)]
    fn handle_eval_breaker(&mut self) -> crate::value::PyResult<()> {
        if !self.in_gil {
            return Ok(());
        }
        if EVAL_BREAKER.load(Relaxed) & DROP_REQUEST != 0 {
            // let a waiting thread have it, and wait until one has (or the
            // same thread would just take it back)
            let mut s = Box::<Shared>::default();
            s.swap(self);
            let mut g = drop_gil(lock(), Some(Parked(s)));
            let seen = g.switches;
            while g.switches == seen && !g.locked {
                let (ng, r) = TAKEN.wait_timeout(g, Duration::from_millis(5)).unwrap_or_else(|e| e.into_inner());
                g = ng;
                if r.timed_out() {
                    break;
                }
            }
            if let Some(Parked(mut s)) = take(g) {
                s.swap(self);
            }
        }
        if EVAL_BREAKER.load(Relaxed) & INTERRUPT != 0 && ident() == MAIN_IDENT.load(Relaxed) {
            EVAL_BREAKER.fetch_and(!INTERRUPT, Relaxed);
            let cls = self.exc_class("KeyboardInterrupt");
            return Err(crate::value::Exc { value: self.new_exc(&cls, vec![]) });
        }
        Ok(())
    }
}
