// SPDX-License-Identifier: MIT
//! A Python interpreter for the Ranger Python runtime, written in Rust.
//!
//! ```ignore
//! let mut rt = rpy::Runtime::new();
//! rt.write_file("/data/in.csv", text);
//! let r = rt.run(source, "main.py");
//! r.stdout; r.traceback; rt.changed_files();
//! ```
//!
//! The filesystem is in memory: the host puts files in before the run and
//! reads the ones the script changed after it (Sliqtly's deck is the sandbox).

pub mod abcmod;
pub mod ast;
pub mod bigint;
pub mod builtins;
pub mod code;
pub mod compile;
pub mod cdecimal;
pub mod decimalmod;
pub mod dismod;
pub mod format;
pub mod interpmod;
pub mod fs;
pub mod gc;
pub mod gil;
pub mod lexer;
pub mod lower;
pub mod mathmod;
pub mod matching;
pub mod mpdec;
pub mod methods;
pub mod modules;
pub mod ops;
pub mod parser;
pub mod poolalloc;
pub mod posix;
pub mod process;
pub mod pyast;
pub mod range;
#[cfg(unix)]
pub mod socketmod;
pub mod stdlib;
pub mod strmethods;
pub mod value;
pub mod vm;
pub mod tb;
pub mod testcapi;
pub mod threadmod;
pub mod unidata;
pub mod wtf8;

use value::Value;

/// the deepest Python call nesting sys.setrecursionlimit may allow
pub const MAX_DEPTH: u32 = 20000;

static UNICODE_NAMES: &str = include_str!(concat!(env!("OUT_DIR"), "/unames.txt"));

/// the character a \N{NAME} escape names
pub fn unicode_name(name: &str) -> Option<char> {
    let key = format!(";{}=", name.to_uppercase());
    let at = UNICODE_NAMES.find(&key)? + key.len();
    let rest = &UNICODE_NAMES[at..];
    let end = rest.find(';').unwrap_or(rest.len());
    char::from_u32(u32::from_str_radix(&rest[..end], 16).ok()?)
}

/// the name of a character (unicodedata.name), when the table has it
pub fn unicode_name_of(c: char) -> Option<&'static str> {
    let key = format!("={:X};", c as u32);
    let at = UNICODE_NAMES.find(&key)?;
    let start = UNICODE_NAMES[..at].rfind(';')? + 1;
    Some(&UNICODE_NAMES[start..at])
}

pub struct RunResult {
    pub ok: bool,
    pub stdout: String,
    pub stderr: String,
    /// "ZeroDivisionError" when the script raised
    pub error_type: String,
    /// the last line of the traceback
    pub error: String,
    pub error_line: u32,
    /// CPython style, last line = error
    pub traceback: String,
    /// the status a process running the script exits with (SystemExit's code)
    pub exit_code: i32,
}

pub struct Runtime {
    pub fs: fs::Fs,
    pub argv: Vec<String>,
    pub max_steps: u64,
    /// stream output to the process's stdout/stderr instead of collecting it
    pub echo: bool,
    /// folders imports look in after /
    pub search_path: Vec<String>,
    /// how the interpreter was started (sys.executable, sys.flags)
    pub host: process::Host,
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

impl Runtime {
    pub fn new() -> Runtime {
        Runtime { fs: fs::Fs::new(), argv: vec![], max_steps: 0, echo: false, search_path: vec![], host: Default::default() }
    }

    pub fn write_file(&mut self, path: &str, data: impl AsRef<[u8]>) {
        self.fs.put(path, data.as_ref().to_vec());
    }

    pub fn read_file(&self, path: &str) -> Option<Vec<u8>> {
        self.fs.read(path)
    }

    pub fn changed_files(&self) -> Vec<String> {
        self.fs.changed_files()
    }

    pub fn removed_files(&self) -> Vec<String> {
        self.fs.removed_files()
    }

    /// runs a script; the filesystem keeps what it wrote
    pub fn run(&mut self, source: &str, filename: &str) -> RunResult {
        self.run_with(filename, |vm, globals| vm.run_source(source, filename, globals))
    }

    /// runs a script file's bytes, decoded as UTF-8 or as its coding
    /// comment says (PEP 263)
    pub fn run_bytes(&mut self, source: &[u8], filename: &str) -> RunResult {
        self.run_with(filename, |vm, globals| {
            let src = crate::builtins::decode_file_source(vm, source, filename)?;
            vm.run_source(&src, filename, globals)
        })
    }

    fn run_with(
        &mut self,
        filename: &str,
        run: impl FnOnce(&mut vm::Vm, std::rc::Rc<std::cell::RefCell<value::Dict>>) -> value::PyResult<Value>,
    ) -> RunResult {
        let fs = std::mem::replace(&mut self.fs, fs::Fs::new());
        let mut vm = vm::Vm::new(fs);
        vm.argv = self.argv.clone();
        vm.max_steps = self.max_steps;
        vm.echo = self.echo;
        vm.search_path = self.search_path.clone();
        vm.host = self.host.clone();
        vm.in_gil = true;
        vm.take_gil();
        gil::MAIN_IDENT.store(gil::ident(), std::sync::atomic::Ordering::Relaxed);
        let globals = vm.main_globals.clone();
        globals.borrow_mut().set_s("__file__", Value::str(filename));
        let r = run(&mut vm, globals);
        let mut res = RunResult {
            ok: true,
            stdout: String::new(),
            stderr: String::new(),
            error_type: String::new(),
            error: String::new(),
            error_line: 0,
            traceback: String::new(),
            exit_code: 0,
        };
        if let Err(e) = r {
            report(&mut vm, &e.value, &mut res);
        }
        vm.shutdown_threads();
        vm.finalize();
        res.stdout = std::mem::take(&mut vm.out);
        res.stderr = std::mem::take(&mut vm.err_out);
        self.fs = std::mem::replace(&mut vm.fs, fs::Fs::new());
        // the objects go while the GIL is still held
        drop(vm);
        gil::release_main();
        res
    }
}

/// the source line of a SyntaxError and the carets under the bad part, as
/// the traceback module shows them
fn syntax_error_lines(text: &str, at: [Option<Value>; 4]) -> Vec<String> {
    let int = |v: &Option<Value>| match v {
        Some(Value::Int(n)) => Some(*n),
        _ => None,
    };
    let [lineno, end_lineno, offset, end_offset] = at.each_ref().map(int);
    let rtext = text.trim_end_matches('\n');
    let ltext = rtext.trim_start_matches([' ', '\n', '\x0c']);
    let (rlen, tlen) = (rtext.chars().count() as i64, text.chars().count() as i64);
    let spaces = rlen - ltext.chars().count() as i64;
    let mut lines = vec![format!("    {}", ltext)];
    let Some(mut offset) = offset else { return lines };
    let mut end = if lineno == end_lineno { end_offset.filter(|e| *e != 0).unwrap_or(offset) } else { rlen + 1 };
    if tlen > 0 && offset > tlen {
        offset = rlen + 1;
    }
    if tlen > 0 && end > tlen {
        end = rlen + 1;
    }
    if offset >= end || end < 0 {
        end = offset + 1;
    }
    let (col, end_col) = (offset - 1 - spaces, end - 1 - spaces);
    if col >= 0 {
        let pad: String = ltext.chars().take(col as usize).map(|c| if c.is_whitespace() { c } else { ' ' }).collect();
        lines.push(format!("    {}{}", pad, "^".repeat((end_col - col).max(0) as usize)));
    }
    lines
}

fn report(vm: &mut vm::Vm, exc: &Value, res: &mut RunResult) {
    let cls = vm.class_of(exc);
    if cls.is_subclass(&vm.exc_class("SystemExit")) {
        let code = vm.getattr(exc, &builtins::ps("code")).unwrap_or(Value::None);
        res.exit_code = match code {
            Value::None => return,
            Value::Int(0) | Value::Bool(false) => return,
            Value::Int(n) => n as i32,
            Value::Bool(true) => 1,
            v => {
                let s = vm.str_of(&v).unwrap_or_default();
                vm.write_err(&s);
                vm.write_err("\n");
                1
            }
        };
        res.ok = false;
        res.error_type = "SystemExit".into();
        return;
    }
    res.ok = false;
    res.exit_code = 1;
    let mut lines = vec!["Traceback (most recent call last):".to_string()];
    let tb = vm.getattr(exc, &builtins::ps("__traceback__")).unwrap_or(Value::None);
    let mut entries = vm.tb_entries(&tb);
    let mut prev = String::new();
    let mut run = 0;
    for (file, line, name) in &entries {
        let w = format!("  File \"{}\", line {}, in {}", file, line, name);
        if w == prev {
            run += 1;
            if run >= 3 {
                continue;
            }
        } else {
            if run >= 3 {
                lines.push(format!("  [Previous line repeated {} more time{}]", run - 2, if run - 2 == 1 { "" } else { "s" }));
            }
            run = 0;
        }
        lines.push(w.clone());
        prev = w;
    }
    if run >= 3 {
        lines.push(format!("  [Previous line repeated {} more time{}]", run - 2, if run - 2 == 1 { "" } else { "s" }));
    }
    // an error in compiling the script has no traceback to show
    if entries.is_empty() {
        lines.clear();
    }
    if cls.is_subclass(&vm.exc_class("SyntaxError")) {
        let attr = |vm: &mut vm::Vm, n: &str| vm.getattr(exc, &builtins::ps(n)).ok();
        if let (Some(Value::Str(f)), Some(Value::Int(l))) = (attr(vm, "filename"), attr(vm, "lineno")) {
            lines.push(format!("  File \"{}\", line {}", f.s, l));
            entries.push((f.s.clone(), l, String::new()));
        }
        if let Some(Value::Str(t)) = attr(vm, "text") {
            let at = ["lineno", "end_lineno", "offset", "end_offset"].map(|n| attr(vm, n));
            lines.extend(syntax_error_lines(&t.s, at));
        }
    }
    res.error_line = entries.last().map(|e| e.1 as u32).unwrap_or(0);
    let mut name = cls.name().to_string();
    if let Some(Value::Str(m)) = cls.dict.borrow().get_s("__module__") {
        if m.s != "builtins" && m.s != "__main__" {
            name = format!("{}.{}", m.s, name);
        }
    }
    res.error_type = cls.name().to_string();
    let msg = if cls.is_subclass(&vm.exc_class("SyntaxError")) {
        // the traceback shows the location itself, so only the message
        match vm.getattr(exc, &builtins::ps("msg")) {
            Ok(Value::None) | Err(_) => String::new(),
            Ok(m) => vm.str_of(&m).unwrap_or_default(),
        }
    } else {
        builtins::exc_str(vm, exc).unwrap_or_default()
    };
    res.error = if msg.is_empty() { name } else { format!("{}: {}", name, msg) };
    lines.push(res.error.clone());
    res.traceback = python_traceback(vm, exc).unwrap_or_else(|| lines.join("\n"));
}

/// prints an exception nothing caught to the interpreter's stderr, as the
/// top level does
pub fn print_uncaught(vm: &mut vm::Vm, exc: &Value) {
    let mut res = RunResult { ok: true, stdout: String::new(), stderr: String::new(), error_type: String::new(), error: String::new(), error_line: 0, traceback: String::new(), exit_code: 0 };
    report(vm, exc, &mut res);
    if !res.traceback.is_empty() {
        vm.write_err(&res.traceback);
        vm.write_err("\n");
    }
}

/// the traceback module's rendering of an uncaught exception, with source
/// lines, carets and chained exceptions, as CPython prints it; None when the
/// module cannot run, leaving the plain report
fn python_traceback(vm: &mut vm::Vm, exc: &Value) -> Option<String> {
    let m = vm.import_one("traceback").ok()?;
    let f = vm.getattr(&m, &builtins::ps("format_exception")).ok()?;
    let parts = vm.call_vec(&f, vec![exc.clone()], &[]).ok()?;
    let parts = vm.to_vec(&parts).ok()?;
    let mut text = String::new();
    for p in &parts {
        text.push_str(p.as_str()?);
    }
    Some(text.trim_end_matches('\n').to_string())
}
