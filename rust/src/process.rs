// SPDX-License-Identifier: MIT
//! Child processes for the subprocess module (`_rpyprocess`), and what an
//! interpreter knows about how it was started (sys.executable, sys.flags).
//!
//! A script's filesystem is the in-memory sandbox, so a child rpy (one
//! started through sys.executable) is given the same sandbox: the parent
//! writes its mounts, search path, working directory and the files it
//! changed to a file the child finds through RPY_SANDBOX, and when the child
//! exits the parent takes back the files the child changed. Other programs
//! run as they are. A WebAssembly build has no processes.

use crate::builtins::*;
use crate::fs::Fs;
use crate::value::*;
use crate::vm::Vm;

/// the environment variable that names a child's sandbox file
pub const SANDBOX_VAR: &str = "RPY_SANDBOX";

/// how the interpreter was started: the command line options CPython has
#[derive(Clone, Default)]
pub struct Host {
    /// the interpreter's own program, "" when it cannot start itself
    pub executable: String,
    /// host folders and where they are in the sandbox (--mount)
    pub mounts: Vec<(String, String)>,
    pub isolated: bool,
    pub ignore_environment: bool,
    pub no_user_site: bool,
    pub safe_path: bool,
    pub dev_mode: bool,
    pub utf8_mode: bool,
    pub inspect: bool,
    pub optimize: i64,
    pub verbose: i64,
    pub quiet: bool,
    pub bytes_warning: i64,
    pub unbuffered: bool,
    /// -W options, in order
    pub warnoptions: Vec<String>,
    /// -X options: name, and the value after `=`
    pub xoptions: Vec<(String, Option<String>)>,
    /// the environment os.environ starts with; None for the runtime's own
    pub environ: Option<Vec<(String, String)>>,
}

impl Host {
    /// takes one CPython option (`-I`, `-X dev`, ...); `next` gives an
    /// option's argument when it is not attached. False for an unknown one.
    pub fn option(&mut self, flag: char, arg: &mut dyn FnMut() -> Option<String>) -> bool {
        match flag {
            'I' => {
                self.isolated = true;
                self.ignore_environment = true;
                self.no_user_site = true;
                self.safe_path = true;
            }
            'E' => self.ignore_environment = true,
            's' => self.no_user_site = true,
            'P' => self.safe_path = true,
            'S' | 'B' | 'd' | 'R' => {}
            'i' => self.inspect = true,
            'O' => self.optimize += 1,
            'v' => self.verbose += 1,
            'q' => self.quiet = true,
            'b' => self.bytes_warning += 1,
            'u' => self.unbuffered = true,
            'W' => self.warnoptions.push(arg().unwrap_or_default()),
            'X' => {
                let x = arg().unwrap_or_default();
                let (name, value) = match x.split_once('=') {
                    Some((n, v)) => (n.to_string(), Some(v.to_string())),
                    None => (x.clone(), None),
                };
                match name.as_str() {
                    "dev" => self.dev_mode = true,
                    "utf8" => self.utf8_mode = value.as_deref() != Some("0"),
                    _ => {}
                }
                self.xoptions.push((name, value));
            }
            _ => return false,
        }
        true
    }
}

// -------------------------------------------------------------------------
// the sandbox a child shares

/// what a parent hands a child rpy, and what the child hands back
#[derive(Default)]
pub struct Sandbox {
    pub cwd: String,
    pub mounts: Vec<(String, String)>,
    pub search_path: Vec<String>,
    pub dirs: Vec<String>,
    pub files: Vec<(String, String)>,
    /// files that are no longer there
    pub gone: Vec<String>,
}

impl Sandbox {
    /// the folders of `fs`, the files the script wrote and the host files it removed
    pub fn changes(fs: &Fs) -> Sandbox {
        Sandbox {
            cwd: fs.cwd.clone(),
            dirs: fs.dirs.iter().cloned().collect(),
            files: fs.changed_files().into_iter().filter_map(|p| fs.read(&p).map(|t| (p, t))).collect(),
            gone: fs.removed_files(),
            ..Default::default()
        }
    }

    /// One record per line, `kind<TAB>field...`; a file's text follows its
    /// line, its length in bytes given on the line.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = String::from("rpy-sandbox 1\n");
        out.push_str(&format!("cwd\t{}\n", self.cwd));
        for (h, v) in &self.mounts {
            out.push_str(&format!("mount\t{}\t{}\n", h, v));
        }
        for p in &self.search_path {
            out.push_str(&format!("path\t{}\n", p));
        }
        for d in &self.dirs {
            out.push_str(&format!("dir\t{}\n", d));
        }
        for (p, t) in &self.files {
            out.push_str(&format!("file\t{}\t{}\n", p, t.len()));
            out.push_str(t);
            out.push('\n');
        }
        for p in &self.gone {
            out.push_str(&format!("gone\t{}\n", p));
        }
        out.into_bytes()
    }

    pub fn decode(text: &str) -> Option<Sandbox> {
        let mut rest = text.strip_prefix("rpy-sandbox 1\n")?;
        let mut sb = Sandbox::default();
        while !rest.is_empty() {
            let (line, after) = rest.split_once('\n')?;
            rest = after;
            let mut f = line.split('\t');
            match (f.next()?, f.next()?) {
                ("cwd", p) => sb.cwd = p.to_string(),
                ("mount", h) => sb.mounts.push((h.to_string(), f.next()?.to_string())),
                ("path", p) => sb.search_path.push(p.to_string()),
                ("dir", p) => sb.dirs.push(p.to_string()),
                ("gone", p) => sb.gone.push(p.to_string()),
                ("file", p) => {
                    let n: usize = f.next()?.parse().ok()?;
                    let t = rest.get(..n)?;
                    sb.files.push((p.to_string(), t.to_string()));
                    rest = rest.get(n + 1..)?;
                }
                _ => return None,
            }
        }
        Some(sb)
    }

    /// Puts the folders and files into `fs`: as the host's own files when
    /// a child starts, or as the script's changes when a parent takes them back.
    pub fn apply(&self, fs: &mut Fs, as_host: bool) {
        for d in &self.dirs {
            fs.mkdirs(d);
        }
        for (p, t) in &self.files {
            if as_host {
                fs.put(p, t.clone());
            } else if fs.read(p).as_deref() != Some(t.as_str()) {
                fs.write(p, t.clone());
            }
        }
        for p in &self.gone {
            fs.remove(p);
        }
        if !self.cwd.is_empty() && as_host {
            fs.cwd = self.cwd.clone();
        }
    }
}

// -------------------------------------------------------------------------
// _rpyprocess: what subprocess.Popen runs on

pub fn module() -> Value {
    let (d, m) = crate::modules::module("_rpyprocess");
    crate::modules::put_fns(
        &d,
        &[
            ("_rpyprocess.spawn", p_spawn),
            ("_rpyprocess.communicate", p_communicate),
            ("_rpyprocess.wait", p_wait),
            ("_rpyprocess.poll", p_poll),
            ("_rpyprocess.kill", p_kill),
        ],
    );
    d.borrow_mut().set_s("available", Value::Bool(cfg!(not(target_family = "wasm"))));
    m
}

#[cfg(target_family = "wasm")]
mod imp {
    use super::*;

    fn unsupported(vm: &mut Vm) -> Exc {
        let cls = vm.exc_class("OSError");
        let v = vm.new_exc(&cls, vec![Value::Int(52), Value::str("subprocesses are not supported on this platform")]);
        Exc { value: v }
    }

    pub fn spawn(vm: &mut Vm, _a: &[Value], _k: Kw) -> PyResult<Value> {
        Err(unsupported(vm))
    }
    pub fn communicate(vm: &mut Vm, _a: &[Value], _k: Kw) -> PyResult<Value> {
        Err(unsupported(vm))
    }
    pub fn wait(vm: &mut Vm, _a: &[Value], _k: Kw) -> PyResult<Value> {
        Err(unsupported(vm))
    }
    pub fn poll(vm: &mut Vm, _a: &[Value], _k: Kw) -> PyResult<Value> {
        Err(unsupported(vm))
    }
    pub fn kill(vm: &mut Vm, _a: &[Value], _k: Kw) -> PyResult<Value> {
        Err(unsupported(vm))
    }
}

#[cfg(not(target_family = "wasm"))]
mod imp {
    use super::*;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::io::{Read, Write};
    use std::process::{Child, Command, Stdio};
    use std::rc::Rc;

    /// the strings of a list of str
    fn strings(vm: &mut Vm, v: &Value) -> PyResult<Vec<String>> {
        let mut out = vec![];
        for x in vm.to_vec(v)? {
            out.push(as_pystr(vm, &x, "spawn")?.s.clone());
        }
        Ok(out)
    }

    /// the handle a native call is given, as an int
    fn handle(vm: &mut Vm, a: &[Value]) -> PyResult<i64> {
        need(vm, a, 1)?;
        vm.to_index(&a[0])
    }

    fn os_error(vm: &mut Vm, e: &std::io::Error, what: &str) -> Exc {
        let (kind, errno) = match e.kind() {
            std::io::ErrorKind::NotFound => ("FileNotFoundError", 2),
            std::io::ErrorKind::PermissionDenied => ("PermissionError", 13),
            _ => ("OSError", e.raw_os_error().unwrap_or(0) as i64),
        };
        let cls = vm.exc_class(kind);
        let msg = match kind {
            "FileNotFoundError" => "No such file or directory".to_string(),
            "PermissionError" => "Permission denied".to_string(),
            _ => e.to_string(),
        };
        let v = vm.new_exc(&cls, vec![Value::Int(errno), Value::string(msg), Value::str(what)]);
        Exc { value: v }
    }

    // subprocess's PIPE, STDOUT and DEVNULL
    const PIPE: i64 = -1;
    const STDOUT: i64 = -2;
    const DEVNULL: i64 = -3;

    struct Running {
        child: Child,
        /// stdout and stderr together, for stderr=STDOUT
        merged: Option<std::io::PipeReader>,
        /// the sandbox file of a child rpy
        sandbox: Option<std::path::PathBuf>,
        status: Option<i64>,
    }

    thread_local! {
        static CHILDREN: RefCell<HashMap<i64, Running>> = RefCell::new(HashMap::new());
    }

    fn stdio(mode: &Value) -> Stdio {
        match mode {
            Value::Int(PIPE) => Stdio::piped(),
            Value::Int(DEVNULL) => Stdio::null(),
            _ => Stdio::inherit(),
        }
    }

    /// spawn(args, env, cwd, stdin, stdout, stderr) -> (handle, pid)
    pub fn spawn(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
        need(vm, a, 6)?;
        let args = strings(vm, &a[0])?;
        let Some(program) = args.first().cloned() else {
            return Err(vm.value_error("args must not be empty"));
        };
        let mut cmd = Command::new(&program);
        cmd.args(&args[1..]);
        if let Value::Dict(d) = &a[1] {
            cmd.env_clear();
            for (key, v) in d.borrow().items() {
                let (key, v) = (as_pystr(vm, &key, "env")?, as_pystr(vm, &v, "env")?);
                cmd.env(&*key.s, &*v.s);
            }
        }
        let cwd = match &a[2] {
            Value::None => vm.fs.cwd.clone(),
            v => {
                let dir = as_pystr(vm, v, "cwd")?;
                vm.fs.normalize(&dir.s)
            }
        };
        // our own interpreter runs in this sandbox; anything else in the host's
        let own = !vm.host.executable.is_empty() && program == vm.host.executable;
        let sandbox = if own {
            let mut sb = Sandbox::changes(&vm.fs);
            sb.cwd = cwd;
            sb.mounts = vm.host.mounts.clone();
            sb.search_path = vm.search_path.clone();
            let n = CHILDREN.with(|c| c.borrow().len());
            let path = std::env::temp_dir().join(format!("rpy-sandbox-{}-{}-{}", std::process::id(), n, vm.start_time.elapsed().as_nanos()));
            if let Err(e) = std::fs::write(&path, sb.encode()) {
                return Err(os_error(vm, &e, &path.to_string_lossy()));
            }
            cmd.env(SANDBOX_VAR, &path);
            Some(path)
        } else {
            None
        };
        cmd.stdin(stdio(&a[3]));
        let mut merged = None;
        if matches!(a[5], Value::Int(STDOUT)) && matches!(a[4], Value::Int(PIPE)) {
            let (r, w) = std::io::pipe().map_err(|e| os_error(vm, &e, &program))?;
            let w2 = w.try_clone().map_err(|e| os_error(vm, &e, &program))?;
            cmd.stdout(w).stderr(w2);
            merged = Some(r);
        } else {
            cmd.stdout(stdio(&a[4]));
            cmd.stderr(stdio(&a[5]));
        }
        let r = cmd.spawn();
        // the write ends of a merged pipe are the child's alone now
        drop(cmd);
        let child = match r {
            Ok(c) => c,
            Err(e) => {
                if let Some(p) = &sandbox {
                    let _ = std::fs::remove_file(p);
                }
                return Err(os_error(vm, &e, &program));
            }
        };
        let pid = child.id() as i64;
        CHILDREN.with(|c| c.borrow_mut().insert(pid, Running { child, merged, sandbox, status: None }));
        Ok(Value::tuple(vec![Value::Int(pid), Value::Int(pid)]))
    }

    fn code(st: std::process::ExitStatus) -> i64 {
        use std::os::unix::process::ExitStatusExt;
        match st.code() {
            Some(c) => c as i64,
            None => -(st.signal().unwrap_or(0) as i64),
        }
    }

    /// the child's exit status once it is known: a child rpy's changes
    /// to the sandbox are taken back then
    fn finished(vm: &mut Vm, h: i64, st: std::process::ExitStatus) -> i64 {
        let rc = code(st);
        let sandbox = CHILDREN.with(|c| {
            let mut c = c.borrow_mut();
            let r = c.get_mut(&h)?;
            r.status = Some(rc);
            r.sandbox.take()
        });
        if let Some(p) = sandbox {
            let back = p.with_extension("out");
            if let Some(sb) = std::fs::read_to_string(&back).ok().and_then(|t| Sandbox::decode(&t)) {
                sb.apply(&mut vm.fs, false);
            }
            let _ = std::fs::remove_file(&p);
            let _ = std::fs::remove_file(&back);
        }
        rc
    }

    fn take(vm: &mut Vm, h: i64) -> PyResult<Running> {
        match CHILDREN.with(|c| c.borrow_mut().remove(&h)) {
            Some(r) => Ok(r),
            None => Err(vm.value_error("no such child process")),
        }
    }

    fn put(h: i64, r: Running) {
        CHILDREN.with(|c| c.borrow_mut().insert(h, r));
    }

    fn read_all(r: Option<impl Read>) -> Option<Vec<u8>> {
        r.map(|mut r| {
            let mut v = vec![];
            let _ = r.read_to_end(&mut v);
            v
        })
    }

    /// communicate(handle, input) -> (stdout, stderr, returncode): feeds
    /// stdin, reads the pipes to the end and waits for the exit
    pub fn communicate(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
        let h = handle(vm, a)?;
        let input = match a.get(1) {
            None | Some(Value::None) => None,
            Some(v) => Some(bytes_of(vm, v)?),
        };
        let mut r = take(vm, h)?;
        // stdin is written while the pipes are read, so that neither side blocks
        let feeder = r.child.stdin.take().map(|mut w| {
            std::thread::spawn(move || {
                if let Some(b) = input {
                    let _ = w.write_all(&b);
                }
            })
        });
        let err_pipe = r.child.stderr.take();
        let err_reader = err_pipe.map(|p| std::thread::spawn(move || read_all(Some(p))));
        let out = match r.merged.take() {
            Some(m) => read_all(Some(m)),
            None => read_all(r.child.stdout.take()),
        };
        let err = err_reader.and_then(|t| t.join().ok().flatten());
        if let Some(t) = feeder {
            let _ = t.join();
        }
        let st = r.child.wait();
        put(h, r);
        let rc = match st {
            Ok(st) => finished(vm, h, st),
            Err(e) => return Err(os_error(vm, &e, "wait")),
        };
        let b = |x: Option<Vec<u8>>| x.map(|v| Value::Bytes(Rc::new(v))).unwrap_or(Value::None);
        Ok(Value::tuple(vec![b(out), b(err), Value::Int(rc)]))
    }

    pub fn wait(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
        let h = handle(vm, a)?;
        let mut r = take(vm, h)?;
        if let Some(rc) = r.status {
            put(h, r);
            return Ok(Value::Int(rc));
        }
        drop(r.child.stdin.take());
        let st = r.child.wait();
        put(h, r);
        match st {
            Ok(st) => Ok(Value::Int(finished(vm, h, st))),
            Err(e) => Err(os_error(vm, &e, "wait")),
        }
    }

    pub fn poll(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
        let h = handle(vm, a)?;
        let mut r = take(vm, h)?;
        if let Some(rc) = r.status {
            put(h, r);
            return Ok(Value::Int(rc));
        }
        let st = r.child.try_wait();
        put(h, r);
        match st {
            Ok(Some(st)) => Ok(Value::Int(finished(vm, h, st))),
            Ok(None) => Ok(Value::None),
            Err(e) => Err(os_error(vm, &e, "poll")),
        }
    }

    pub fn kill(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
        let h = handle(vm, a)?;
        let mut r = take(vm, h)?;
        if r.status.is_none() {
            let _ = r.child.kill();
        }
        put(h, r);
        Ok(Value::None)
    }
}

fn p_spawn(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    imp::spawn(vm, a, k)
}
fn p_communicate(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    imp::communicate(vm, a, k)
}
fn p_wait(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    imp::wait(vm, a, k)
}
fn p_poll(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    imp::poll(vm, a, k)
}
fn p_kill(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    imp::kill(vm, a, k)
}
