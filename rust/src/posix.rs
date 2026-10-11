// SPDX-License-Identifier: MIT
//! What a native build has of the host's POSIX system: processes (os.fork,
//! os.waitpid), file descriptors, pipes and pseudo-terminals, signals with
//! Python handlers (_signal), select and termios.
//!
//! A signal's C handler only marks the signal tripped; the interpreter runs
//! the Python handler at its next check, as CPython's eval breaker does:
//! when a function starts, when a loop jumps back, when a generator resumes
//! at a `yield`, and when a blocking call is interrupted (PEP 475).
//!
//! A script's filesystem is the in-memory sandbox, so a forked child works
//! on its own copy of it; the files the child changes go back to the parent
//! when the parent reaps it with waitpid, as a child rpy's do (process.rs).
//! A WebAssembly build has none of this, and no signal is ever pending there.

use crate::builtins::*;
use crate::value::*;
use crate::vm::Vm;
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};

/// one past the highest signal number
#[cfg(any(target_os = "linux", target_os = "android"))]
pub const NSIG: usize = 65;
#[cfg(not(any(target_os = "linux", target_os = "android")))]
pub const NSIG: usize = 33;

/// some signal is tripped: the one flag the interpreter loop reads
static PENDING: AtomicBool = AtomicBool::new(false);
static TRIPPED: [AtomicBool; NSIG] = [const { AtomicBool::new(false) }; NSIG];

crate::gil_static! {
    /// each signal's Python handler: a callable, SIG_DFL or SIG_IGN as an
    /// int, or None for an action Python did not set
    static HANDLERS: RefCell<Vec<Value>> = RefCell::new(vec![Value::None; NSIG]);
}

/// whether a signal came that no Python handler has seen yet
#[inline]
pub fn signals_pending() -> bool {
    PENDING.load(Ordering::Relaxed)
}

/// Runs the Python handlers of the signals that came since the last check
/// (PyErr_CheckSignals); a handler's exception is raised where the
/// interpreter checked.
pub fn check_signals(vm: &mut Vm) -> PyResult<()> {
    // handlers run in the main thread only, as in CPython
    #[cfg(not(target_family = "wasm"))]
    if crate::gil::ident() != crate::gil::MAIN_IDENT.load(Ordering::Relaxed) {
        return Ok(());
    }
    if !PENDING.swap(false, Ordering::Acquire) {
        return Ok(());
    }
    for (sig, tripped) in TRIPPED.iter().enumerate().skip(1) {
        if !tripped.swap(false, Ordering::Acquire) {
            continue;
        }
        let h = HANDLERS.with(|h| h.borrow()[sig].clone());
        if !is_callable(vm, &h) {
            continue;
        }
        // the frame that was running, as sys._getframe() gives it
        let frame = if vm.frames.is_empty() { Value::None } else { crate::modules::sys_getframe(vm, &[], &[])? };
        if let Err(e) = vm.call_vec(&h, vec![Value::Int(sig as i64), frame], &[]) {
            // the signals still tripped wait for the next check
            if TRIPPED.iter().any(|t| t.load(Ordering::Relaxed)) {
                PENDING.store(true, Ordering::Release);
            }
            return Err(e);
        }
    }
    Ok(())
}

/// the system's message for an errno
pub fn strerror(errno: i32) -> String {
    let msg = std::io::Error::from_raw_os_error(errno).to_string();
    match msg.rfind(" (os error ") {
        Some(i) => msg[..i].to_string(),
        None => msg,
    }
}

/// OSError (or the subclass for the errno) with the system's message
pub fn os_err(vm: &mut Vm, errno: i32) -> Exc {
    let cls = vm.exc_class("OSError");
    let e = vm.new_exc(&cls, vec![Value::Int(errno as i64), Value::string(strerror(errno))]);
    Exc { value: e }
}

/// a file descriptor argument: an int, or an object with fileno()
pub fn fd_of(vm: &mut Vm, v: &Value) -> PyResult<i32> {
    let n = match v {
        Value::Int(_) | Value::Bool(_) | Value::Big(_) => vm.to_index(v)?,
        _ => match vm.getattr_opt(v, &ps("fileno"))? {
            Some(f) => {
                let r = vm.call_vec(&f, vec![], &[])?;
                match r {
                    Value::Int(n) => n,
                    _ => return Err(vm.type_error("fileno() returned a non-integer")),
                }
            }
            None => return Err(vm.type_error("argument must be an int, or have a fileno() method.")),
        },
    };
    if n < 0 {
        return Err(vm.value_error(format!("file descriptor cannot be a negative integer ({})", n)));
    }
    i32::try_from(n).map_err(|_| vm.err("OverflowError", "fd is greater than maximum"))
}

#[cfg(unix)]
pub use imp::*;

/// what a build without file descriptors says about one
#[cfg(not(unix))]
mod fallback {
    use super::*;

    pub fn fd_read(vm: &mut Vm, _fd: i32, _n: usize) -> PyResult<Vec<u8>> {
        Err(os_err(vm, 9))
    }
    pub fn fd_write_all(vm: &mut Vm, _fd: i32, _data: &[u8]) -> PyResult<()> {
        Err(os_err(vm, 9))
    }
    pub fn fd_close(_fd: i32) {}
    pub fn fd_isatty(_fd: i32) -> bool {
        false
    }
    pub fn fd_seek(vm: &mut Vm, _fd: i32, _off: i64, _whence: i64) -> PyResult<i64> {
        Err(os_err(vm, 9))
    }
}
#[cfg(not(unix))]
pub use fallback::*;

#[cfg(unix)]
mod imp {
    use super::*;
    use crate::modules::{module, put_fns};
    use crate::process::Sandbox;
    use std::collections::HashSet;
    use std::rc::Rc;
    use std::sync::atomic::AtomicI32;

    fn errno() -> i32 {
        std::io::Error::last_os_error().raw_os_error().unwrap_or(0)
    }

    /// Calls `f` until it is not interrupted by a signal; the Python
    /// handlers run in between, and their exception ends the call (PEP 475).
    fn retry(vm: &mut Vm, mut f: impl FnMut() -> isize) -> PyResult<isize> {
        loop {
            let r = f();
            if r >= 0 {
                return Ok(r);
            }
            let e = errno();
            if e != libc::EINTR {
                return Err(os_err(vm, e));
            }
            check_signals(vm)?;
        }
    }

    fn int_arg(vm: &mut Vm, a: &[Value], i: usize) -> PyResult<i64> {
        vm.to_index(&a[i])
    }

    fn c_int_arg(vm: &mut Vm, a: &[Value], i: usize) -> PyResult<i32> {
        let n = int_arg(vm, a, i)?;
        i32::try_from(n).map_err(|_| vm.err("OverflowError", "signed integer is greater than maximum"))
    }

    // ---------------------------------------------------------------------
    // file descriptors

    /// one read of at most `n` bytes from `fd`; empty at the end
    pub fn fd_read(vm: &mut Vm, fd: i32, n: usize) -> PyResult<Vec<u8>> {
        let mut buf = vec![0u8; n];
        // SAFETY: buf has room for n bytes
        let got = retry(vm, || unsafe { libc::read(fd, buf.as_mut_ptr() as *mut libc::c_void, n) })?;
        buf.truncate(got as usize);
        Ok(buf)
    }

    fn fd_write(vm: &mut Vm, fd: i32, data: &[u8]) -> PyResult<usize> {
        // SAFETY: data is valid for its length
        let n = retry(vm, || unsafe { libc::write(fd, data.as_ptr() as *const libc::c_void, data.len()) })?;
        Ok(n as usize)
    }

    pub fn fd_write_all(vm: &mut Vm, fd: i32, mut data: &[u8]) -> PyResult<()> {
        while !data.is_empty() {
            let n = fd_write(vm, fd, data)?;
            data = &data[n..];
        }
        Ok(())
    }

    pub fn fd_close(fd: i32) {
        // SAFETY: closing a descriptor has no memory effects
        unsafe { libc::close(fd) };
    }

    pub fn fd_isatty(fd: i32) -> bool {
        // SAFETY: a query on a descriptor
        unsafe { libc::isatty(fd) == 1 }
    }

    pub fn fd_seek(vm: &mut Vm, fd: i32, off: i64, whence: i64) -> PyResult<i64> {
        // SAFETY: a query on a descriptor
        let r = unsafe { libc::lseek(fd, off as libc::off_t, whence as libc::c_int) };
        if r < 0 {
            return Err(os_err(vm, errno()));
        }
        Ok(r as i64)
    }

    /// EBADF unless `fd` is an open descriptor
    pub fn fd_check(vm: &mut Vm, fd: i32) -> PyResult<()> {
        // SAFETY: st is written by fstat
        let mut st: libc::stat = unsafe { std::mem::zeroed() };
        if unsafe { libc::fstat(fd, &mut st) } != 0 {
            return Err(os_err(vm, errno()));
        }
        if st.st_mode & libc::S_IFMT == libc::S_IFDIR {
            return Err(os_err(vm, libc::EISDIR));
        }
        Ok(())
    }

    fn set_cloexec(fd: i32) {
        // SAFETY: flag changes on a descriptor
        unsafe {
            let fl = libc::fcntl(fd, libc::F_GETFD);
            if fl >= 0 {
                libc::fcntl(fd, libc::F_SETFD, fl | libc::FD_CLOEXEC);
            }
        }
    }

    fn os_pipe(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
        check_nargs(vm, "pipe", a, 0, 0)?;
        let mut fds = [0i32; 2];
        // SAFETY: fds has room for two descriptors
        #[cfg(any(target_os = "linux", target_os = "android"))]
        let r = unsafe { libc::pipe2(fds.as_mut_ptr(), libc::O_CLOEXEC) };
        #[cfg(not(any(target_os = "linux", target_os = "android")))]
        let r = unsafe { libc::pipe(fds.as_mut_ptr()) };
        if r != 0 {
            return Err(os_err(vm, errno()));
        }
        // PEP 446: new descriptors are not inherited
        fds.iter().for_each(|fd| set_cloexec(*fd));
        Ok(Value::tuple(vec![Value::Int(fds[0] as i64), Value::Int(fds[1] as i64)]))
    }

    fn os_read(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
        check_nargs(vm, "read", a, 2, 2)?;
        let fd = c_int_arg(vm, a, 0)?;
        let n = int_arg(vm, a, 1)?;
        if n < 0 {
            return Err(os_err(vm, libc::EINVAL));
        }
        let b = fd_read(vm, fd, n as usize)?;
        Ok(Value::Bytes(Rc::new(b)))
    }

    fn os_write(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
        check_nargs(vm, "write", a, 2, 2)?;
        let fd = c_int_arg(vm, a, 0)?;
        if matches!(a[1], Value::Str(_)) {
            return Err(vm.type_error("a bytes-like object is required, not 'str'"));
        }
        let data = bytes_of(vm, &a[1])?;
        Ok(Value::Int(fd_write(vm, fd, &data)? as i64))
    }

    fn os_close(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
        check_nargs(vm, "close", a, 1, 1)?;
        let fd = c_int_arg(vm, a, 0)?;
        // SAFETY: closing a descriptor has no memory effects
        if unsafe { libc::close(fd) } != 0 {
            let e = errno();
            // the descriptor is closed even when close was interrupted
            if e != libc::EINTR {
                return Err(os_err(vm, e));
            }
        }
        Ok(Value::None)
    }

    fn os_closerange(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
        check_nargs(vm, "closerange", a, 2, 2)?;
        let (lo, hi) = (c_int_arg(vm, a, 0)?, c_int_arg(vm, a, 1)?);
        for fd in lo.max(0)..hi {
            fd_close(fd);
        }
        Ok(Value::None)
    }

    fn os_dup(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
        check_nargs(vm, "dup", a, 1, 1)?;
        let fd = c_int_arg(vm, a, 0)?;
        // SAFETY: a new descriptor for an open one
        let r = unsafe { libc::fcntl(fd, libc::F_DUPFD_CLOEXEC, 0) };
        if r < 0 {
            return Err(os_err(vm, errno()));
        }
        Ok(Value::Int(r as i64))
    }

    fn os_dup2(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
        check_nargs(vm, "dup2", a, 2, 3)?;
        let (fd, fd2) = (c_int_arg(vm, a, 0)?, c_int_arg(vm, a, 1)?);
        let inheritable = match arg_or_kw(a, 2, k, "inheritable") {
            Some(v) => vm.truthy(&v)?,
            None => true,
        };
        // SAFETY: descriptor operations
        let r = retry(vm, || unsafe { libc::dup2(fd, fd2) as isize })?;
        if !inheritable {
            set_cloexec(r as i32);
        }
        Ok(Value::Int(r as i64))
    }

    fn os_isatty(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
        check_nargs(vm, "isatty", a, 1, 1)?;
        let fd = c_int_arg(vm, a, 0)?;
        Ok(Value::Bool(fd_isatty(fd)))
    }

    fn os_lseek(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
        check_nargs(vm, "lseek", a, 3, 3)?;
        let fd = c_int_arg(vm, a, 0)?;
        let (off, how) = (int_arg(vm, a, 1)?, int_arg(vm, a, 2)?);
        Ok(Value::Int(fd_seek(vm, fd, off, how)?))
    }

    fn os_get_blocking(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
        check_nargs(vm, "get_blocking", a, 1, 1)?;
        let fd = c_int_arg(vm, a, 0)?;
        // SAFETY: a query on a descriptor
        let fl = unsafe { libc::fcntl(fd, libc::F_GETFL) };
        if fl < 0 {
            return Err(os_err(vm, errno()));
        }
        Ok(Value::Bool(fl & libc::O_NONBLOCK == 0))
    }

    pub fn set_blocking(vm: &mut Vm, fd: i32, blocking: bool) -> PyResult<()> {
        // SAFETY: flag changes on a descriptor
        let fl = unsafe { libc::fcntl(fd, libc::F_GETFL) };
        if fl < 0 {
            return Err(os_err(vm, errno()));
        }
        let fl = if blocking { fl & !libc::O_NONBLOCK } else { fl | libc::O_NONBLOCK };
        if unsafe { libc::fcntl(fd, libc::F_SETFL, fl) } < 0 {
            return Err(os_err(vm, errno()));
        }
        Ok(())
    }

    fn os_set_blocking(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
        check_nargs(vm, "set_blocking", a, 2, 2)?;
        let fd = c_int_arg(vm, a, 0)?;
        let b = vm.truthy(&a[1])?;
        set_blocking(vm, fd, b)?;
        Ok(Value::None)
    }

    fn os_strerror(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
        check_nargs(vm, "strerror", a, 1, 1)?;
        let e = c_int_arg(vm, a, 0)?;
        Ok(Value::string(strerror(e)))
    }

    fn os_urandom(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
        check_nargs(vm, "urandom", a, 1, 1)?;
        let n = int_arg(vm, a, 0)?;
        if n < 0 {
            return Err(vm.value_error("negative argument not allowed"));
        }
        let mut out = vec![0u8; n as usize];
        use std::io::Read;
        let r = std::fs::File::open("/dev/urandom").and_then(|mut f| f.read_exact(&mut out));
        if let Err(e) = r {
            return Err(os_err(vm, e.raw_os_error().unwrap_or(libc::EIO)));
        }
        Ok(Value::Bytes(Rc::new(out)))
    }

    // ---------------------------------------------------------------------
    // pseudo-terminals

    /// a new pseudo-terminal pair: (master, slave), neither inherited
    fn open_pty(vm: &mut Vm) -> PyResult<(i32, i32)> {
        // SAFETY: the calls of posix_openpt(3); the name buffer is ours
        unsafe {
            let master = libc::posix_openpt(libc::O_RDWR | libc::O_NOCTTY);
            if master < 0 {
                return Err(os_err(vm, errno()));
            }
            let fail = |vm: &mut Vm| {
                let e = os_err(vm, errno());
                libc::close(master);
                e
            };
            if libc::grantpt(master) != 0 || libc::unlockpt(master) != 0 {
                return Err(fail(vm));
            }
            #[cfg(any(target_os = "linux", target_os = "android"))]
            let name = {
                let mut buf = [0 as libc::c_char; 128];
                if libc::ptsname_r(master, buf.as_mut_ptr(), buf.len()) != 0 {
                    return Err(fail(vm));
                }
                std::ffi::CStr::from_ptr(buf.as_ptr()).to_owned()
            };
            #[cfg(not(any(target_os = "linux", target_os = "android")))]
            let name = {
                let p = libc::ptsname(master);
                if p.is_null() {
                    return Err(fail(vm));
                }
                std::ffi::CStr::from_ptr(p).to_owned()
            };
            let slave = libc::open(name.as_ptr(), libc::O_RDWR | libc::O_NOCTTY);
            if slave < 0 {
                return Err(fail(vm));
            }
            set_cloexec(master);
            set_cloexec(slave);
            Ok((master, slave))
        }
    }

    fn os_openpty(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
        check_nargs(vm, "openpty", a, 0, 0)?;
        let (m, s) = open_pty(vm)?;
        Ok(Value::tuple(vec![Value::Int(m as i64), Value::Int(s as i64)]))
    }

    /// makes `fd` the controlling terminal and the standard streams of a
    /// new session, as login_tty(3)
    fn login_tty(fd: i32) -> Result<(), i32> {
        // SAFETY: descriptor and session operations
        unsafe {
            if libc::setsid() < 0 {
                return Err(errno());
            }
            if libc::ioctl(fd, libc::TIOCSCTTY as _, 0) < 0 {
                return Err(errno());
            }
            for std in 0..3 {
                if libc::dup2(fd, std) < 0 {
                    return Err(errno());
                }
            }
            if fd > 2 {
                libc::close(fd);
            }
        }
        Ok(())
    }

    fn os_login_tty(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
        check_nargs(vm, "login_tty", a, 1, 1)?;
        let fd = fd_of(vm, &a[0])?;
        login_tty(fd).map_err(|e| os_err(vm, e))?;
        Ok(Value::None)
    }

    fn os_forkpty(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
        check_nargs(vm, "forkpty", a, 0, 0)?;
        let (master, slave) = open_pty(vm)?;
        let pid = match fork(vm) {
            Ok(p) => p,
            Err(e) => {
                fd_close(master);
                fd_close(slave);
                return Err(e);
            }
        };
        if pid == 0 {
            fd_close(master);
            if login_tty(slave).is_err() {
                // SAFETY: the child has nowhere to report to
                unsafe { libc::_exit(1) };
            }
        } else {
            fd_close(slave);
        }
        Ok(Value::tuple(vec![Value::Int(pid as i64), Value::Int(if pid == 0 { -1 } else { master as i64 })]))
    }

    // ---------------------------------------------------------------------
    // processes

    crate::gil_static! {
        /// os.register_at_fork's callables: before, after_in_child, after_in_parent
        static AT_FORK: RefCell<[Vec<Value>; 3]> = const { RefCell::new([vec![], vec![], vec![]]) };
        /// the forked children whose changes to the files come back on waitpid
        static FORKED: RefCell<HashSet<i32>> = RefCell::new(HashSet::new());
        /// in a forked child: the file its changes go to
        static HAND_BACK: RefCell<Option<std::path::PathBuf>> = const { RefCell::new(None) };
    }

    fn hand_back_path(parent: i32, child: i32) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("rpy-fork-{}-{}", parent, child))
    }

    /// In a forked child that is ending: the files it changed, for the
    /// parent to take back when it reaps the child.
    pub fn hand_back(fs: &crate::fs::Fs) {
        let Some(p) = HAND_BACK.with(|h| h.borrow_mut().take()) else { return };
        if fs.changed_files().is_empty() && fs.removed_files().is_empty() {
            return;
        }
        let _ = std::fs::write(p, Sandbox::changes(fs).encode());
    }

    /// whether this process is a child forked by os.fork
    pub fn is_forked_child() -> bool {
        HAND_BACK.with(|h| h.borrow().is_some())
    }

    fn take_back(vm: &mut Vm, pid: i32) {
        if !FORKED.with(|f| f.borrow_mut().remove(&pid)) {
            return;
        }
        // SAFETY: getpid has no side effects
        let p = hand_back_path(unsafe { libc::getpid() }, pid);
        if let Ok(t) = std::fs::read(&p) {
            if let Some(sb) = Sandbox::decode(&t) {
                sb.apply(&mut vm.fs, false);
            }
            let _ = std::fs::remove_file(&p);
        }
    }

    fn run_at_fork(vm: &mut Vm, which: usize) {
        let mut hooks = AT_FORK.with(|h| h.borrow()[which].clone());
        // the `before` callables run last registered first
        if which == 0 {
            hooks.reverse();
        }
        for h in hooks {
            if let Err(e) = vm.call_vec(&h, vec![], &[]) {
                vm.unraisable(e, &h);
            }
        }
    }

    /// fork(2) with the at-fork callables; the child starts with no output
    /// of the parent's waiting to be written and no signal pending
    fn fork(vm: &mut Vm) -> PyResult<i32> {
        run_at_fork(vm, 0);
        {
            use std::io::Write;
            let _ = std::io::stdout().flush();
            let _ = std::io::stderr().flush();
        }
        // SAFETY: getpid has no side effects
        let parent = unsafe { libc::getpid() };
        // SAFETY: fork; the child only goes on with this thread, and the
        // interpreter's own locks are held across it
        let pid = crate::gil::fork_with(|| unsafe { libc::fork() });
        if pid < 0 {
            return Err(os_err(vm, errno()));
        }
        if pid == 0 {
            vm.out.clear();
            vm.err_out.clear();
            vm.fs.clear_changes();
            // SAFETY: getpid has no side effects
            let me = unsafe { libc::getpid() };
            HAND_BACK.with(|h| *h.borrow_mut() = Some(hand_back_path(parent, me)));
            FORKED.with(|f| f.borrow_mut().clear());
            crate::threadmod::after_fork_child();
            PENDING.store(false, Ordering::Relaxed);
            TRIPPED.iter().for_each(|t| t.store(false, Ordering::Relaxed));
            run_at_fork(vm, 1);
        } else {
            FORKED.with(|f| f.borrow_mut().insert(pid));
            run_at_fork(vm, 2);
        }
        Ok(pid)
    }

    fn os_fork(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
        check_nargs(vm, "fork", a, 0, 0)?;
        Ok(Value::Int(fork(vm)? as i64))
    }

    fn os_register_at_fork(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
        if !a.is_empty() {
            return Err(vm.type_error("register_at_fork() takes no positional arguments"));
        }
        let names = ["before", "after_in_child", "after_in_parent"];
        let mut got: Vec<(usize, Value)> = vec![];
        for (n, v) in k {
            let Some(i) = names.iter().position(|x| *x == &*n.s) else {
                return Err(vm.type_error(format!("register_at_fork() got an unexpected keyword argument '{}'", n.s)));
            };
            if v.is_none() {
                continue;
            }
            if !is_callable(vm, v) {
                let tn = vm.type_name(v);
                return Err(vm.type_error(format!("'{}' must be callable, not {}", names[i], tn)));
            }
            got.push((i, v.clone()));
        }
        if got.is_empty() {
            return Err(vm.type_error("At least one argument is required."));
        }
        AT_FORK.with(|h| {
            let mut h = h.borrow_mut();
            for (i, v) in got {
                h[i].push(v);
            }
        });
        Ok(Value::None)
    }

    fn os_waitpid(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
        check_nargs(vm, "waitpid", a, 2, 2)?;
        let (pid, options) = (c_int_arg(vm, a, 0)?, c_int_arg(vm, a, 1)?);
        let mut status = 0;
        // SAFETY: status is written by waitpid
        let r = retry(vm, || unsafe { libc::waitpid(pid, &mut status, options) as isize })? as i32;
        if r > 0 && (libc::WIFEXITED(status) || libc::WIFSIGNALED(status)) {
            take_back(vm, r);
        }
        Ok(Value::tuple(vec![Value::Int(r as i64), Value::Int(status as i64)]))
    }

    fn os_wait(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
        check_nargs(vm, "wait", a, 0, 0)?;
        os_waitpid(vm, &[Value::Int(-1), Value::Int(0)], &[])
    }

    fn os_exit(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
        check_nargs(vm, "_exit", a, 1, 1)?;
        let code = c_int_arg(vm, a, 0)?;
        hand_back(&vm.fs);
        // rpy keeps the program's output until it ends, flushed or not: what
        // was written goes out before the process does
        {
            use std::io::Write;
            let _ = std::io::stdout().write_all(std::mem::take(&mut vm.out).as_bytes());
            let _ = std::io::stdout().flush();
            let _ = std::io::stderr().write_all(std::mem::take(&mut vm.err_out).as_bytes());
        }
        // SAFETY: ends the process at once, as CPython's os._exit
        unsafe { libc::_exit(code) }
    }

    fn status_arg(vm: &mut Vm, a: &[Value], name: &str) -> PyResult<i32> {
        check_nargs(vm, name, a, 1, 1)?;
        c_int_arg(vm, a, 0)
    }

    fn os_waitstatus_to_exitcode(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
        let a = match (a.first(), kwarg(k, "status")) {
            (None, Some(v)) => vec![v],
            _ => a.to_vec(),
        };
        let st = status_arg(vm, &a, "waitstatus_to_exitcode")?;
        if libc::WIFEXITED(st) {
            return Ok(Value::Int(libc::WEXITSTATUS(st) as i64));
        }
        if libc::WIFSIGNALED(st) {
            return Ok(Value::Int(-(libc::WTERMSIG(st) as i64)));
        }
        if libc::WIFSTOPPED(st) {
            return Err(vm.value_error(format!("process stopped by delivery of signal {}", libc::WSTOPSIG(st))));
        }
        Err(vm.value_error(format!("invalid wait status: {}", st)))
    }

    macro_rules! wait_macro {
        ($name:ident, $py:literal, $f:expr) => {
            fn $name(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
                let st = status_arg(vm, a, $py)?;
                Ok($f(st))
            }
        };
    }
    wait_macro!(os_wifexited, "WIFEXITED", |s| Value::Bool(libc::WIFEXITED(s)));
    wait_macro!(os_wexitstatus, "WEXITSTATUS", |s| Value::Int(libc::WEXITSTATUS(s) as i64));
    wait_macro!(os_wifsignaled, "WIFSIGNALED", |s| Value::Bool(libc::WIFSIGNALED(s)));
    wait_macro!(os_wtermsig, "WTERMSIG", |s| Value::Int(libc::WTERMSIG(s) as i64));
    wait_macro!(os_wifstopped, "WIFSTOPPED", |s| Value::Bool(libc::WIFSTOPPED(s)));
    wait_macro!(os_wstopsig, "WSTOPSIG", |s| Value::Int(libc::WSTOPSIG(s) as i64));
    wait_macro!(os_wifcontinued, "WIFCONTINUED", |s| Value::Bool(libc::WIFCONTINUED(s)));
    wait_macro!(os_wcoredump, "WCOREDUMP", |s| Value::Bool(libc::WCOREDUMP(s)));

    fn os_getpid(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
        check_nargs(vm, "getpid", a, 0, 0)?;
        // SAFETY: no side effects
        Ok(Value::Int(unsafe { libc::getpid() } as i64))
    }

    fn os_getppid(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
        check_nargs(vm, "getppid", a, 0, 0)?;
        // SAFETY: no side effects
        Ok(Value::Int(unsafe { libc::getppid() } as i64))
    }

    fn os_getpgrp(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
        check_nargs(vm, "getpgrp", a, 0, 0)?;
        // SAFETY: no side effects
        Ok(Value::Int(unsafe { libc::getpgrp() } as i64))
    }

    fn os_setsid(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
        check_nargs(vm, "setsid", a, 0, 0)?;
        // SAFETY: a session operation
        if unsafe { libc::setsid() } < 0 {
            return Err(os_err(vm, errno()));
        }
        Ok(Value::None)
    }

    fn os_kill(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
        check_nargs(vm, "kill", a, 2, 2)?;
        let (pid, sig) = (c_int_arg(vm, a, 0)?, c_int_arg(vm, a, 1)?);
        // SAFETY: sends a signal
        if unsafe { libc::kill(pid, sig) } != 0 {
            return Err(os_err(vm, errno()));
        }
        // a signal sent to this process is handled before kill returns
        check_signals(vm)?;
        Ok(Value::None)
    }

    fn os_killpg(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
        check_nargs(vm, "killpg", a, 2, 2)?;
        let (pg, sig) = (c_int_arg(vm, a, 0)?, c_int_arg(vm, a, 1)?);
        // SAFETY: sends a signal
        if unsafe { libc::killpg(pg, sig) } != 0 {
            return Err(os_err(vm, errno()));
        }
        check_signals(vm)?;
        Ok(Value::None)
    }

    /// the os functions a POSIX host has
    pub fn extend_os(d: &Rc<RefCell<Dict>>) {
        put_fns(
            d,
            &[
                ("os.fork", os_fork),
                ("os.forkpty", os_forkpty),
                ("os.register_at_fork", os_register_at_fork),
                ("os.waitpid", os_waitpid),
                ("os.wait", os_wait),
                ("os._exit", os_exit),
                ("os.waitstatus_to_exitcode", os_waitstatus_to_exitcode),
                ("os.WIFEXITED", os_wifexited),
                ("os.WEXITSTATUS", os_wexitstatus),
                ("os.WIFSIGNALED", os_wifsignaled),
                ("os.WTERMSIG", os_wtermsig),
                ("os.WIFSTOPPED", os_wifstopped),
                ("os.WSTOPSIG", os_wstopsig),
                ("os.WIFCONTINUED", os_wifcontinued),
                ("os.WCOREDUMP", os_wcoredump),
                ("os.pipe", os_pipe),
                ("os.read", os_read),
                ("os.write", os_write),
                ("os.close", os_close),
                ("os.closerange", os_closerange),
                ("os.dup", os_dup),
                ("os.dup2", os_dup2),
                ("os.isatty", os_isatty),
                ("os.lseek", os_lseek),
                ("os.get_blocking", os_get_blocking),
                ("os.set_blocking", os_set_blocking),
                ("os.strerror", os_strerror),
                ("os.urandom", os_urandom),
                ("os.openpty", os_openpty),
                ("os.login_tty", os_login_tty),
                ("os.getpid", os_getpid),
                ("os.getppid", os_getppid),
                ("os.getpgrp", os_getpgrp),
                ("os.setsid", os_setsid),
                ("os.kill", os_kill),
                ("os.killpg", os_killpg),
            ],
        );
        let mut d = d.borrow_mut();
        for (n, v) in [
            ("WNOHANG", libc::WNOHANG),
            ("WUNTRACED", libc::WUNTRACED),
            ("WCONTINUED", libc::WCONTINUED),
            ("SEEK_SET", libc::SEEK_SET),
            ("SEEK_CUR", libc::SEEK_CUR),
            ("SEEK_END", libc::SEEK_END),
            ("O_RDONLY", libc::O_RDONLY),
            ("O_WRONLY", libc::O_WRONLY),
            ("O_RDWR", libc::O_RDWR),
            ("O_NONBLOCK", libc::O_NONBLOCK),
            ("O_APPEND", libc::O_APPEND),
            ("O_CREAT", libc::O_CREAT),
            ("O_EXCL", libc::O_EXCL),
            ("O_TRUNC", libc::O_TRUNC),
            ("O_NOCTTY", libc::O_NOCTTY),
            ("O_CLOEXEC", libc::O_CLOEXEC),
        ] {
            d.set_s(n, Value::Int(v as i64));
        }
    }

    // ---------------------------------------------------------------------
    // signals

    crate::gil_static! {
        /// signal.default_int_handler, the same object whoever asks for it
        static DEFAULT_INT: Value = native("default_int_handler", default_int_handler);
    }

    fn default_int_handler(vm: &mut Vm, _a: &[Value], _k: Kw) -> PyResult<Value> {
        let cls = vm.exc_class("KeyboardInterrupt");
        let e = vm.new_exc(&cls, vec![]);
        Err(Exc { value: e })
    }

    /// the descriptor signal.set_wakeup_fd names, -1 for none
    static WAKEUP_FD: AtomicI32 = AtomicI32::new(-1);

    #[cfg(any(target_os = "linux", target_os = "android"))]
    fn errno_ptr() -> *mut libc::c_int {
        // SAFETY: the thread's errno
        unsafe { libc::__errno_location() }
    }
    #[cfg(any(target_os = "macos", target_os = "ios", target_os = "freebsd"))]
    fn errno_ptr() -> *mut libc::c_int {
        // SAFETY: the thread's errno
        unsafe { libc::__error() }
    }

    /// the C handler of a signal with a Python handler: it marks the signal
    /// and wakes the wakeup descriptor, and nothing else (async-signal-safe)
    extern "C" fn trip(sig: libc::c_int) {
        if let Some(t) = TRIPPED.get(sig as usize) {
            t.store(true, Ordering::Release);
        }
        PENDING.store(true, Ordering::Release);
        let fd = WAKEUP_FD.load(Ordering::Relaxed);
        if fd >= 0 {
            let b = sig as u8;
            // SAFETY: write(2) is async-signal-safe; errno is kept for the code interrupted
            unsafe {
                let saved = *errno_ptr();
                libc::write(fd, &b as *const u8 as *const libc::c_void, 1);
                *errno_ptr() = saved;
            }
        }
    }

    fn trip_handler() -> libc::sighandler_t {
        trip as extern "C" fn(libc::c_int) as libc::sighandler_t
    }

    /// the current action of `sig`
    fn action(sig: i32) -> Option<libc::sigaction> {
        // SAFETY: old is written by sigaction
        unsafe {
            let mut old: libc::sigaction = std::mem::zeroed();
            if libc::sigaction(sig, std::ptr::null(), &mut old) != 0 {
                return None;
            }
            Some(old)
        }
    }

    /// sets the C handler of `sig`; Python-handled signals interrupt system calls
    fn set_action(sig: i32, handler: libc::sighandler_t) -> Result<(), i32> {
        // SAFETY: a zeroed sigaction with its handler and an empty mask
        unsafe {
            let mut new: libc::sigaction = std::mem::zeroed();
            new.sa_sigaction = handler;
            new.sa_flags = libc::SA_ONSTACK;
            libc::sigemptyset(&mut new.sa_mask);
            if libc::sigaction(sig, &new, std::ptr::null_mut()) != 0 {
                return Err(errno());
            }
        }
        Ok(())
    }

    /// What CPython does at startup: SIGINT raises KeyboardInterrupt
    /// (unless the process was started with it ignored), SIGXFSZ is ignored.
    pub fn install_default_handlers() {
        if action(libc::SIGINT).is_some_and(|a| a.sa_sigaction == libc::SIG_DFL) && set_action(libc::SIGINT, trip_handler()).is_ok() {
            let h = DEFAULT_INT.with(|d| d.clone());
            HANDLERS.with(|t| t.borrow_mut()[libc::SIGINT as usize] = h);
        }
        let _ = set_action(libc::SIGXFSZ, libc::SIG_IGN);
    }

    /// ends the process by SIGINT's default action (exit status 130 if it cannot)
    pub fn exit_by_sigint() {
        let _ = set_action(libc::SIGINT, libc::SIG_DFL);
        // SAFETY: the signal ends the process
        unsafe {
            libc::raise(libc::SIGINT);
            libc::_exit(130);
        }
    }

    /// Blocks the asynchronous signals in the calling thread, so that the
    /// process's signals reach the interpreter's thread.
    pub fn block_signals() {
        // SAFETY: the calling thread's mask
        unsafe {
            let mut set: libc::sigset_t = std::mem::zeroed();
            libc::sigfillset(&mut set);
            for s in [libc::SIGSEGV, libc::SIGBUS, libc::SIGFPE, libc::SIGILL, libc::SIGABRT] {
                libc::sigdelset(&mut set, s);
            }
            libc::pthread_sigmask(libc::SIG_BLOCK, &set, std::ptr::null_mut());
        }
    }

    /// raise(3), leaving a Python handler to the next check
    pub fn raise_only(sig: i32) -> Result<(), i32> {
        // SAFETY: sends a signal to this thread
        if unsafe { libc::raise(sig) } != 0 {
            return Err(errno());
        }
        Ok(())
    }

    /// raise(3): the signal is handled before it returns
    pub fn raise_signal(vm: &mut Vm, sig: i32) -> PyResult<()> {
        raise_only(sig).map_err(|e| os_err(vm, e))?;
        check_signals(vm)
    }

    fn signum(vm: &mut Vm, v: &Value) -> PyResult<i32> {
        let n = vm.to_index(v)?;
        if n < 1 || n >= NSIG as i64 {
            return Err(vm.value_error("signal number out of range"));
        }
        Ok(n as i32)
    }

    /// the Python handler table, filled from the actions in place on first use
    fn init_handlers() {
        HANDLERS.with(|t| {
            let mut t = t.borrow_mut();
            for (sig, h) in t.iter_mut().enumerate().skip(1) {
                if !h.is_none() {
                    continue;
                }
                if let Some(a) = action(sig as i32) {
                    if a.sa_sigaction == libc::SIG_DFL {
                        *h = Value::Int(0);
                    } else if a.sa_sigaction == libc::SIG_IGN {
                        *h = Value::Int(1);
                    }
                }
            }
        });
    }

    fn sig_signal(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
        check_nargs(vm, "signal", a, 2, 2)?;
        let sig = signum(vm, &a[0])?;
        let h = a[1].clone();
        let c = match &h {
            Value::Int(0) => libc::SIG_DFL,
            Value::Int(1) => libc::SIG_IGN,
            v if is_callable(vm, v) => trip_handler(),
            _ => return Err(vm.type_error("signal handler must be signal.SIG_IGN, signal.SIG_DFL, or a callable object")),
        };
        set_action(sig, c).map_err(|e| os_err(vm, e))?;
        let old = HANDLERS.with(|t| std::mem::replace(&mut t.borrow_mut()[sig as usize], h));
        Ok(old)
    }

    fn sig_getsignal(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
        check_nargs(vm, "getsignal", a, 1, 1)?;
        let sig = signum(vm, &a[0])?;
        Ok(HANDLERS.with(|t| t.borrow()[sig as usize].clone()))
    }

    fn sig_raise_signal(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
        check_nargs(vm, "raise_signal", a, 1, 1)?;
        let sig = signum(vm, &a[0])?;
        raise_signal(vm, sig)?;
        Ok(Value::None)
    }

    fn sig_alarm(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
        check_nargs(vm, "alarm", a, 1, 1)?;
        let secs = int_arg(vm, a, 0)?;
        // SAFETY: arms the process's alarm timer
        Ok(Value::Int(unsafe { libc::alarm(secs.clamp(0, u32::MAX as i64) as libc::c_uint) } as i64))
    }

    fn sig_pause(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
        check_nargs(vm, "pause", a, 0, 0)?;
        // SAFETY: waits for a signal
        unsafe { libc::pause() };
        check_signals(vm)?;
        Ok(Value::None)
    }

    /// set_wakeup_fd(fd, *, warn_on_full_buffer=True): a full buffer is never warned about here
    fn sig_set_wakeup_fd(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
        check_nargs(vm, "set_wakeup_fd", a, 1, 1)?;
        let fd = vm.to_index(&a[0])?;
        if fd != -1 {
            let fd = i32::try_from(fd).map_err(|_| vm.value_error(format!("invalid fd: {}", fd)))?;
            // SAFETY: a query on a descriptor
            let fl = unsafe { libc::fcntl(fd, libc::F_GETFL) };
            if fl < 0 {
                return Err(os_err(vm, errno()));
            }
            if fl & libc::O_NONBLOCK == 0 {
                return Err(vm.value_error(format!("the fd {} must be in non-blocking mode", fd)));
            }
        }
        Ok(Value::Int(WAKEUP_FD.swap(fd as i32, Ordering::Relaxed) as i64))
    }

    fn sig_siginterrupt(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
        check_nargs(vm, "siginterrupt", a, 2, 2)?;
        let sig = signum(vm, &a[0])?;
        let flag = vm.truthy(&a[1])?;
        let Some(mut act) = action(sig) else { return Err(os_err(vm, errno())) };
        if flag {
            act.sa_flags &= !libc::SA_RESTART;
        } else {
            act.sa_flags |= libc::SA_RESTART;
        }
        // SAFETY: act is the action read above
        if unsafe { libc::sigaction(sig, &act, std::ptr::null_mut()) } != 0 {
            return Err(os_err(vm, errno()));
        }
        Ok(Value::None)
    }

    fn sig_strsignal(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
        check_nargs(vm, "strsignal", a, 1, 1)?;
        let sig = signum(vm, &a[0])?;
        // SAFETY: strsignal returns a string we copy at once
        let p = unsafe { libc::strsignal(sig) };
        if p.is_null() {
            return Ok(Value::None);
        }
        let s = unsafe { std::ffi::CStr::from_ptr(p) }.to_string_lossy().into_owned();
        if s.starts_with("Unknown signal") || s.starts_with("Real-time signal") {
            return Ok(Value::None);
        }
        Ok(Value::string(s))
    }

    fn signal_set(vm: &mut Vm, set: &libc::sigset_t) -> PyResult<Value> {
        let out = Rc::new(RefCell::new(Dict::new()));
        for sig in 1..NSIG as i32 {
            // SAFETY: a query on a filled set
            if unsafe { libc::sigismember(set, sig) } == 1 {
                vm.set_add(&out, Value::Int(sig as i64))?;
            }
        }
        Ok(Value::Set(out))
    }

    fn sigset_of(vm: &mut Vm, v: &Value) -> PyResult<libc::sigset_t> {
        // SAFETY: an emptied set filled with checked signal numbers
        let mut set: libc::sigset_t = unsafe { std::mem::zeroed() };
        unsafe { libc::sigemptyset(&mut set) };
        for x in vm.to_vec(v)? {
            let sig = signum(vm, &x)?;
            unsafe { libc::sigaddset(&mut set, sig) };
        }
        Ok(set)
    }

    fn sig_valid_signals(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
        check_nargs(vm, "valid_signals", a, 0, 0)?;
        // SAFETY: a filled set
        let mut set: libc::sigset_t = unsafe { std::mem::zeroed() };
        unsafe { libc::sigfillset(&mut set) };
        signal_set(vm, &set)
    }

    fn sig_pthread_sigmask(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
        check_nargs(vm, "pthread_sigmask", a, 2, 2)?;
        let how = c_int_arg(vm, a, 0)?;
        let set = sigset_of(vm, &a[1])?;
        // SAFETY: old is written by pthread_sigmask
        let mut old: libc::sigset_t = unsafe { std::mem::zeroed() };
        let r = unsafe { libc::pthread_sigmask(how, &set, &mut old) };
        if r != 0 {
            return Err(os_err(vm, r));
        }
        // the signals it unblocked are handled now
        check_signals(vm)?;
        signal_set(vm, &old)
    }

    fn sig_sigpending(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
        check_nargs(vm, "sigpending", a, 0, 0)?;
        // SAFETY: set is written by sigpending
        let mut set: libc::sigset_t = unsafe { std::mem::zeroed() };
        if unsafe { libc::sigpending(&mut set) } != 0 {
            return Err(os_err(vm, errno()));
        }
        signal_set(vm, &set)
    }

    fn timeval(t: f64) -> libc::timeval {
        let secs = t.trunc();
        libc::timeval {
            tv_sec: secs as libc::time_t,
            tv_usec: ((t - secs) * 1e6).round() as libc::suseconds_t,
        }
    }

    fn itimer_tuple(v: &libc::itimerval) -> Value {
        let f = |t: &libc::timeval| Value::Float(t.tv_sec as f64 + t.tv_usec as f64 / 1e6);
        Value::tuple(vec![f(&v.it_value), f(&v.it_interval)])
    }

    fn sig_setitimer(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
        check_nargs(vm, "setitimer", a, 2, 3)?;
        let which = c_int_arg(vm, a, 0)?;
        let delay = vm.to_f64(&a[1])?;
        let interval = match a.get(2) {
            Some(v) => vm.to_f64(v)?,
            None => 0.0,
        };
        let new = libc::itimerval {
            it_value: timeval(delay),
            it_interval: timeval(interval),
        };
        // SAFETY: old is written by setitimer
        let mut old: libc::itimerval = unsafe { std::mem::zeroed() };
        if unsafe { libc::setitimer(which as _, &new, &mut old) } != 0 {
            return Err(os_err(vm, errno()));
        }
        Ok(itimer_tuple(&old))
    }

    fn sig_getitimer(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
        check_nargs(vm, "getitimer", a, 1, 1)?;
        let which = c_int_arg(vm, a, 0)?;
        // SAFETY: cur is written by getitimer
        let mut cur: libc::itimerval = unsafe { std::mem::zeroed() };
        if unsafe { libc::getitimer(which as _, &mut cur) } != 0 {
            return Err(os_err(vm, errno()));
        }
        Ok(itimer_tuple(&cur))
    }

    /// the signal numbers of the host, by name
    fn signal_names() -> Vec<(&'static str, i32)> {
        let mut v = vec![
            ("SIGHUP", libc::SIGHUP),
            ("SIGINT", libc::SIGINT),
            ("SIGQUIT", libc::SIGQUIT),
            ("SIGILL", libc::SIGILL),
            ("SIGTRAP", libc::SIGTRAP),
            ("SIGABRT", libc::SIGABRT),
            ("SIGIOT", libc::SIGIOT),
            ("SIGBUS", libc::SIGBUS),
            ("SIGFPE", libc::SIGFPE),
            ("SIGKILL", libc::SIGKILL),
            ("SIGUSR1", libc::SIGUSR1),
            ("SIGSEGV", libc::SIGSEGV),
            ("SIGUSR2", libc::SIGUSR2),
            ("SIGPIPE", libc::SIGPIPE),
            ("SIGALRM", libc::SIGALRM),
            ("SIGTERM", libc::SIGTERM),
            ("SIGCHLD", libc::SIGCHLD),
            ("SIGCONT", libc::SIGCONT),
            ("SIGSTOP", libc::SIGSTOP),
            ("SIGTSTP", libc::SIGTSTP),
            ("SIGTTIN", libc::SIGTTIN),
            ("SIGTTOU", libc::SIGTTOU),
            ("SIGURG", libc::SIGURG),
            ("SIGXCPU", libc::SIGXCPU),
            ("SIGXFSZ", libc::SIGXFSZ),
            ("SIGVTALRM", libc::SIGVTALRM),
            ("SIGPROF", libc::SIGPROF),
            ("SIGWINCH", libc::SIGWINCH),
            ("SIGIO", libc::SIGIO),
            ("SIGSYS", libc::SIGSYS),
        ];
        #[cfg(any(target_os = "linux", target_os = "android"))]
        v.extend([
            ("SIGSTKFLT", libc::SIGSTKFLT),
            ("SIGPOLL", libc::SIGPOLL),
            ("SIGPWR", libc::SIGPWR),
            ("SIGRTMIN", libc::SIGRTMIN()),
            ("SIGRTMAX", libc::SIGRTMAX()),
        ]);
        v
    }

    pub fn signal_module(vm: &mut Vm) -> Value {
        init_handlers();
        let (d, m) = module("_signal");
        put_fns(
            &d,
            &[
                ("_signal.signal", sig_signal),
                ("_signal.getsignal", sig_getsignal),
                ("_signal.raise_signal", sig_raise_signal),
                ("_signal.alarm", sig_alarm),
                ("_signal.pause", sig_pause),
                ("_signal.set_wakeup_fd", sig_set_wakeup_fd),
                ("_signal.siginterrupt", sig_siginterrupt),
                ("_signal.strsignal", sig_strsignal),
                ("_signal.valid_signals", sig_valid_signals),
                ("_signal.pthread_sigmask", sig_pthread_sigmask),
                ("_signal.sigpending", sig_sigpending),
                ("_signal.setitimer", sig_setitimer),
                ("_signal.getitimer", sig_getitimer),
            ],
        );
        let oserror = vm.exc_class("OSError");
        let itimer = Class::new("itimer_error", vec![oserror.clone()], Builtin::BaseException);
        let mut mro = vec![itimer.clone()];
        mro.extend(oserror.mro.borrow().iter().cloned());
        itimer.set_mro(mro);
        itimer.dict.borrow_mut().set_s("__module__", Value::str("signal"));
        let mut b = d.borrow_mut();
        b.set_s("ItimerError", Value::Class(itimer));
        b.set_s("default_int_handler", DEFAULT_INT.with(|h| h.clone()));
        for (n, v) in signal_names() {
            b.set_s(n, Value::Int(v as i64));
        }
        for (n, v) in [
            ("SIG_DFL", 0),
            ("SIG_IGN", 1),
            ("NSIG", NSIG as i32),
            ("SIG_BLOCK", libc::SIG_BLOCK),
            ("SIG_UNBLOCK", libc::SIG_UNBLOCK),
            ("SIG_SETMASK", libc::SIG_SETMASK),
            ("ITIMER_REAL", libc::ITIMER_REAL as i32),
            ("ITIMER_VIRTUAL", libc::ITIMER_VIRTUAL as i32),
            ("ITIMER_PROF", libc::ITIMER_PROF as i32),
        ] {
            b.set_s(n, Value::Int(v as i64));
        }
        drop(b);
        m
    }

    // ---------------------------------------------------------------------
    // select

    /// a timeout in seconds (None: none) as poll's milliseconds
    pub fn timeout_ms(vm: &mut Vm, v: Option<&Value>) -> PyResult<Option<f64>> {
        match v {
            None | Some(Value::None) => Ok(None),
            Some(v) => {
                let t = vm.to_f64(v)?;
                if t.is_nan() {
                    return Err(vm.value_error("Invalid value NaN (not a number)"));
                }
                if t < 0.0 {
                    return Err(vm.value_error("timeout must be non-negative"));
                }
                Ok(Some(t * 1000.0))
            }
        }
    }

    /// poll(2) on `fds`, until something is ready or `ms` milliseconds
    /// passed; interrupted waits run the signal handlers and go on
    pub fn poll_fds(vm: &mut Vm, fds: &mut [libc::pollfd], ms: Option<f64>) -> PyResult<usize> {
        let deadline = ms.map(|ms| std::time::Instant::now() + std::time::Duration::from_secs_f64(ms / 1000.0));
        loop {
            let wait = match deadline {
                None => -1,
                Some(d) => {
                    let left = d.saturating_duration_since(std::time::Instant::now());
                    (left.as_secs_f64() * 1000.0).ceil().min(i32::MAX as f64) as libc::c_int
                }
            };
            // SAFETY: fds is a valid array of pollfd
            let r = unsafe { libc::poll(fds.as_mut_ptr(), fds.len() as libc::nfds_t, wait) };
            if r >= 0 {
                return Ok(r as usize);
            }
            let e = errno();
            if e != libc::EINTR {
                return Err(os_err(vm, e));
            }
            check_signals(vm)?;
        }
    }

    fn select_select(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
        check_nargs(vm, "select", a, 3, 4)?;
        let ms = timeout_ms(vm, arg_or_kw(a, 3, k, "timeout").as_ref())?;
        let events = [libc::POLLIN, libc::POLLOUT, libc::POLLPRI];
        let mut lists: Vec<Vec<(Value, i32)>> = vec![];
        let mut fds: Vec<libc::pollfd> = vec![];
        for (i, l) in a[..3].iter().enumerate() {
            let mut items = vec![];
            for x in vm.to_vec(l)? {
                let fd = fd_of(vm, &x)?;
                match fds.iter_mut().find(|p| p.fd == fd) {
                    Some(p) => p.events |= events[i],
                    None => fds.push(libc::pollfd { fd, events: events[i], revents: 0 }),
                }
                items.push((x, fd));
            }
            lists.push(items);
        }
        poll_fds(vm, &mut fds, ms)?;
        if fds.iter().any(|p| p.revents & libc::POLLNVAL != 0) {
            return Err(os_err(vm, libc::EBADF));
        }
        let ready = [libc::POLLIN | libc::POLLHUP | libc::POLLERR, libc::POLLOUT | libc::POLLERR, libc::POLLPRI];
        let mut out = vec![];
        for (i, items) in lists.into_iter().enumerate() {
            let got = items
                .into_iter()
                .filter(|(_, fd)| fds.iter().any(|p| p.fd == *fd && p.revents & ready[i] != 0))
                .map(|(v, _)| v)
                .collect();
            out.push(Value::list(got));
        }
        Ok(Value::tuple(out))
    }

    pub fn select_module(vm: &mut Vm) -> Value {
        let (d, m) = module("select");
        put_fns(&d, &[("select.select", select_select)]);
        d.borrow_mut().set_s("error", Value::Class(vm.exc_class("OSError")));
        m
    }

    // ---------------------------------------------------------------------
    // termios

    fn termios_error(vm: &mut Vm, errno: i32) -> Exc {
        let cls = vm.exc_class("termios.error");
        let e = vm.new_exc(&cls, vec![Value::Int(errno as i64), Value::string(strerror(errno))]);
        Exc { value: e }
    }

    fn get_termios(vm: &mut Vm, fd: i32) -> PyResult<libc::termios> {
        // SAFETY: t is written by tcgetattr
        let mut t: libc::termios = unsafe { std::mem::zeroed() };
        if unsafe { libc::tcgetattr(fd, &mut t) } != 0 {
            return Err(termios_error(vm, errno()));
        }
        Ok(t)
    }

    fn tc_getattr(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
        check_nargs(vm, "tcgetattr", a, 1, 1)?;
        let fd = fd_of(vm, &a[0])?;
        let t = get_termios(vm, fd)?;
        // SAFETY: speed queries on a read termios
        let (ispeed, ospeed) = unsafe { (libc::cfgetispeed(&t), libc::cfgetospeed(&t)) };
        let raw = t.c_lflag & libc::ICANON == 0;
        let cc = t
            .c_cc
            .iter()
            .enumerate()
            .map(|(i, c)| {
                if raw && (i == libc::VMIN || i == libc::VTIME) {
                    Value::Int(*c as i64)
                } else {
                    Value::Bytes(Rc::new(vec![*c]))
                }
            })
            .collect();
        let n = |x: libc::tcflag_t| Value::Int(x as i64);
        Ok(Value::list(vec![
            n(t.c_iflag),
            n(t.c_oflag),
            n(t.c_cflag),
            n(t.c_lflag),
            Value::Int(ispeed as i64),
            Value::Int(ospeed as i64),
            Value::list(cc),
        ]))
    }

    fn tc_setattr(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
        check_nargs(vm, "tcsetattr", a, 3, 3)?;
        let fd = fd_of(vm, &a[0])?;
        let when = c_int_arg(vm, a, 1)?;
        let attrs = match &a[2] {
            Value::List(l) if l.borrow().len() == 7 => l.borrow().clone(),
            _ => return Err(vm.type_error("tcsetattr, arg 3: must be 7 element list")),
        };
        let mut t = get_termios(vm, fd)?;
        let flag = |vm: &mut Vm, v: &Value| vm.to_index(v).map(|x| x as libc::tcflag_t);
        t.c_iflag = flag(vm, &attrs[0])?;
        t.c_oflag = flag(vm, &attrs[1])?;
        t.c_cflag = flag(vm, &attrs[2])?;
        t.c_lflag = flag(vm, &attrs[3])?;
        let ispeed = vm.to_index(&attrs[4])? as libc::speed_t;
        let ospeed = vm.to_index(&attrs[5])? as libc::speed_t;
        let cc = match &attrs[6] {
            Value::List(l) if l.borrow().len() == libc::NCCS => l.borrow().clone(),
            _ => return Err(vm.type_error(format!("tcsetattr: attributes[6] must be {} element list", libc::NCCS))),
        };
        for (i, c) in cc.iter().enumerate() {
            t.c_cc[i] = match c {
                Value::Bytes(b) if b.len() == 1 => b[0],
                Value::Int(n) => *n as libc::cc_t,
                _ => return Err(vm.type_error("tcsetattr: elements of attributes must be characters or integers")),
            };
        }
        // SAFETY: t is a complete termios
        unsafe {
            if libc::cfsetispeed(&mut t, ispeed) != 0 || libc::cfsetospeed(&mut t, ospeed) != 0 {
                return Err(termios_error(vm, errno()));
            }
            if libc::tcsetattr(fd, when, &t) != 0 {
                return Err(termios_error(vm, errno()));
            }
        }
        Ok(Value::None)
    }

    macro_rules! tc_call {
        ($name:ident, $py:literal, $f:ident) => {
            fn $name(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
                check_nargs(vm, $py, a, 2, 2)?;
                let fd = fd_of(vm, &a[0])?;
                let n = c_int_arg(vm, a, 1)?;
                // SAFETY: a terminal operation on a descriptor
                if unsafe { libc::$f(fd, n) } != 0 {
                    return Err(termios_error(vm, errno()));
                }
                Ok(Value::None)
            }
        };
    }
    tc_call!(tc_sendbreak, "tcsendbreak", tcsendbreak);
    tc_call!(tc_flush, "tcflush", tcflush);
    tc_call!(tc_flow, "tcflow", tcflow);

    fn tc_drain(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
        check_nargs(vm, "tcdrain", a, 1, 1)?;
        let fd = fd_of(vm, &a[0])?;
        // SAFETY: a terminal operation on a descriptor
        if unsafe { libc::tcdrain(fd) } != 0 {
            return Err(termios_error(vm, errno()));
        }
        Ok(Value::None)
    }

    fn tc_getwinsize(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
        check_nargs(vm, "tcgetwinsize", a, 1, 1)?;
        let fd = fd_of(vm, &a[0])?;
        // SAFETY: w is written by the ioctl
        let mut w: libc::winsize = unsafe { std::mem::zeroed() };
        if unsafe { libc::ioctl(fd, libc::TIOCGWINSZ as _, &mut w) } != 0 {
            return Err(termios_error(vm, errno()));
        }
        Ok(Value::tuple(vec![Value::Int(w.ws_row as i64), Value::Int(w.ws_col as i64)]))
    }

    fn tc_setwinsize(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
        check_nargs(vm, "tcsetwinsize", a, 2, 2)?;
        let fd = fd_of(vm, &a[0])?;
        let v = vm.to_vec(&a[1])?;
        if v.len() != 2 {
            return Err(vm.type_error("tcsetwinsize, arg 2: must be a two-item sequence"));
        }
        // SAFETY: w is read, changed and written back by the ioctls
        let mut w: libc::winsize = unsafe { std::mem::zeroed() };
        if unsafe { libc::ioctl(fd, libc::TIOCGWINSZ as _, &mut w) } != 0 {
            return Err(termios_error(vm, errno()));
        }
        w.ws_row = vm.to_index(&v[0])? as u16;
        w.ws_col = vm.to_index(&v[1])? as u16;
        if unsafe { libc::ioctl(fd, libc::TIOCSWINSZ as _, &w) } != 0 {
            return Err(termios_error(vm, errno()));
        }
        Ok(Value::None)
    }

    pub fn termios_module(vm: &mut Vm) -> Value {
        let (d, m) = module("termios");
        put_fns(
            &d,
            &[
                ("termios.tcgetattr", tc_getattr),
                ("termios.tcsetattr", tc_setattr),
                ("termios.tcsendbreak", tc_sendbreak),
                ("termios.tcdrain", tc_drain),
                ("termios.tcflush", tc_flush),
                ("termios.tcflow", tc_flow),
                ("termios.tcgetwinsize", tc_getwinsize),
                ("termios.tcsetwinsize", tc_setwinsize),
            ],
        );
        let base = vm.exc_class("Exception");
        let c = Class::new("error", vec![base.clone()], Builtin::BaseException);
        let mut mro = vec![c.clone()];
        mro.extend(base.mro.borrow().iter().cloned());
        c.set_mro(mro);
        c.dict.borrow_mut().set_s("__module__", Value::str("termios"));
        vm.exc.insert("termios.error", c.clone());
        let mut b = d.borrow_mut();
        b.set_s("error", Value::Class(c));
        let flags: &[(&str, libc::tcflag_t)] = &[
            ("IGNBRK", libc::IGNBRK),
            ("BRKINT", libc::BRKINT),
            ("IGNPAR", libc::IGNPAR),
            ("PARMRK", libc::PARMRK),
            ("INPCK", libc::INPCK),
            ("ISTRIP", libc::ISTRIP),
            ("INLCR", libc::INLCR),
            ("IGNCR", libc::IGNCR),
            ("ICRNL", libc::ICRNL),
            ("IXON", libc::IXON),
            ("IXANY", libc::IXANY),
            ("IXOFF", libc::IXOFF),
            ("IMAXBEL", libc::IMAXBEL),
            ("OPOST", libc::OPOST),
            ("ONLCR", libc::ONLCR),
            ("OCRNL", libc::OCRNL),
            ("ONOCR", libc::ONOCR),
            ("ONLRET", libc::ONLRET),
            ("CSIZE", libc::CSIZE),
            ("CS5", libc::CS5),
            ("CS6", libc::CS6),
            ("CS7", libc::CS7),
            ("CS8", libc::CS8),
            ("CSTOPB", libc::CSTOPB),
            ("CREAD", libc::CREAD),
            ("PARENB", libc::PARENB),
            ("PARODD", libc::PARODD),
            ("HUPCL", libc::HUPCL),
            ("CLOCAL", libc::CLOCAL),
            ("ISIG", libc::ISIG),
            ("ICANON", libc::ICANON),
            ("ECHO", libc::ECHO),
            ("ECHOE", libc::ECHOE),
            ("ECHOK", libc::ECHOK),
            ("ECHONL", libc::ECHONL),
            ("NOFLSH", libc::NOFLSH),
            ("TOSTOP", libc::TOSTOP),
            ("IEXTEN", libc::IEXTEN),
            ("ECHOCTL", libc::ECHOCTL),
            ("ECHOKE", libc::ECHOKE),
        ];
        for (n, v) in flags {
            b.set_s(n, Value::Int(*v as i64));
        }
        #[cfg(any(target_os = "linux", target_os = "android"))]
        b.set_s("IUTF8", Value::Int(libc::IUTF8 as i64));
        let ints: &[(&str, i64)] = &[
            ("TCSANOW", libc::TCSANOW as i64),
            ("TCSADRAIN", libc::TCSADRAIN as i64),
            ("TCSAFLUSH", libc::TCSAFLUSH as i64),
            ("TCIFLUSH", libc::TCIFLUSH as i64),
            ("TCOFLUSH", libc::TCOFLUSH as i64),
            ("TCIOFLUSH", libc::TCIOFLUSH as i64),
            ("TCOOFF", libc::TCOOFF as i64),
            ("TCOON", libc::TCOON as i64),
            ("TCIOFF", libc::TCIOFF as i64),
            ("TCION", libc::TCION as i64),
            ("VEOF", libc::VEOF as i64),
            ("VEOL", libc::VEOL as i64),
            ("VERASE", libc::VERASE as i64),
            ("VINTR", libc::VINTR as i64),
            ("VKILL", libc::VKILL as i64),
            ("VMIN", libc::VMIN as i64),
            ("VQUIT", libc::VQUIT as i64),
            ("VSTART", libc::VSTART as i64),
            ("VSTOP", libc::VSTOP as i64),
            ("VSUSP", libc::VSUSP as i64),
            ("VTIME", libc::VTIME as i64),
            ("NCCS", libc::NCCS as i64),
            ("B0", libc::B0 as i64),
            ("B50", libc::B50 as i64),
            ("B75", libc::B75 as i64),
            ("B110", libc::B110 as i64),
            ("B134", libc::B134 as i64),
            ("B150", libc::B150 as i64),
            ("B200", libc::B200 as i64),
            ("B300", libc::B300 as i64),
            ("B600", libc::B600 as i64),
            ("B1200", libc::B1200 as i64),
            ("B1800", libc::B1800 as i64),
            ("B2400", libc::B2400 as i64),
            ("B4800", libc::B4800 as i64),
            ("B9600", libc::B9600 as i64),
            ("B19200", libc::B19200 as i64),
            ("B38400", libc::B38400 as i64),
            ("B57600", libc::B57600 as i64),
            ("B115200", libc::B115200 as i64),
            ("B230400", libc::B230400 as i64),
            ("TIOCGWINSZ", libc::TIOCGWINSZ as i64),
            ("TIOCSWINSZ", libc::TIOCSWINSZ as i64),
            ("TIOCSCTTY", libc::TIOCSCTTY as i64),
        ];
        for (n, v) in ints {
            b.set_s(n, Value::Int(*v));
        }
        drop(b);
        m
    }
}
