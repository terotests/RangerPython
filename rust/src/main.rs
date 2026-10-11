// SPDX-License-Identifier: MIT
//! rpy [options] (script.py | -c command) [outdir | args...]: runs a script
//! and prints its output; the files it wrote are saved under outdir as
//! a__b.txt for /a/b.txt.
//!
//!   --mount HOST=VFS   put HOST's files (recursively) at VFS
//!   --path VFS         a folder imports look in after /
//!   --arg X            append X to sys.argv
//!   --echo             stream output as it is written
//!
//! and CPython's own options (-c, -I, -E, -X, -W, -u, ...). An rpy started
//! by a script (subprocess with sys.executable) runs in its parent's
//! sandbox, named by RPY_SANDBOX: the script path is a sandbox path, the
//! words after it are sys.argv, and the files it changes go back to the
//! parent.

use rpy::process::{Sandbox, SANDBOX_VAR};

#[global_allocator]
static ALLOC: rpy::poolalloc::PoolAlloc = rpy::poolalloc::PoolAlloc;

fn mount(rt: &mut rpy::Runtime, host: &std::path::Path, vfs: &str) {
    let Ok(rd) = std::fs::read_dir(host) else { return };
    for e in rd.flatten() {
        let p = e.path();
        let name = e.file_name().to_string_lossy().to_string();
        let target = format!("{}/{}", vfs.trim_end_matches('/'), name);
        if p.is_dir() {
            mount(rt, &p, &target);
        } else if let Ok(data) = std::fs::read(&p) {
            rt.write_file(&target, data);
        }
    }
}

fn add_mount(rt: &mut rpy::Runtime, host: &str, vfs: &str) {
    let abs = std::fs::canonicalize(host).map(|p| p.to_string_lossy().to_string()).unwrap_or_else(|_| host.to_string());
    mount(rt, std::path::Path::new(&abs), vfs);
    rt.host.mounts.push((abs, vfs.to_string()));
}

fn run() -> i32 {
    #[cfg(unix)]
    rpy::posix::install_default_handlers();
    let mut rt = rpy::Runtime::new();
    #[cfg(not(target_family = "wasm"))]
    {
        rt.host.executable = std::env::current_exe().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
    }
    // a child of an rpy script: its parent's sandbox
    let sandbox = std::env::var(SANDBOX_VAR).ok().filter(|p| !p.is_empty());
    if let Some(p) = &sandbox {
        let sb = std::fs::read(p).ok().and_then(|t| Sandbox::decode(&t)).unwrap_or_default();
        for (h, v) in &sb.mounts {
            add_mount(&mut rt, h, v);
        }
        rt.search_path.extend(sb.search_path.iter().cloned());
        sb.apply(&mut rt.fs, true);
        rt.host.environ = Some(std::env::vars().filter(|(k, _)| k != SANDBOX_VAR).collect());
        rt.echo = true;
    }
    let mut extra = vec![];
    let mut args: Vec<String> = vec![];
    let mut command = None;
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--mount" => {
                let m = it.next().unwrap_or_default();
                let (h, v) = m.split_once('=').unwrap_or((&m, "/"));
                add_mount(&mut rt, h, v);
            }
            "--path" => rt.search_path.push(it.next().unwrap_or_default()),
            "--arg" => extra.push(it.next().unwrap_or_default()),
            "--echo" => rt.echo = true,
            "--" => {
                args.extend(it.by_ref());
                break;
            }
            _ if a.len() > 1 && a.starts_with('-') && !a.starts_with("--") => {
                // CPython's short options: -IE, -Wd, -X dev, -c command
                let flags: Vec<char> = a[1..].chars().collect();
                for (i, &f) in flags.iter().enumerate() {
                    let attached: String = flags[i + 1..].iter().collect();
                    let takes_arg = matches!(f, 'c' | 'W' | 'X');
                    let mut value = || if attached.is_empty() { it.next() } else { Some(attached.clone()) };
                    if f == 'c' {
                        command = Some(value().unwrap_or_default());
                    } else if !rt.host.option(f, &mut value) {
                        eprintln!("Unknown option: -{}", f);
                        return 2;
                    }
                    if takes_arg {
                        break;
                    }
                }
                if command.is_some() {
                    args.extend(it.by_ref());
                    break;
                }
            }
            _ => {
                args.push(a);
                if sandbox.is_some() {
                    args.extend(it.by_ref());
                    break;
                }
            }
        }
    }
    let (src, name, outdir) = if let Some(c) = command {
        rt.argv = vec!["-c".into()];
        rt.argv.extend(args);
        (c.into_bytes(), "<string>".to_string(), None)
    } else {
        let Some(script) = args.first().cloned() else {
            println!("usage: rpy [--mount HOST=VFS] [--path VFS] [--arg X] [--echo] script.py [outdir]");
            return 2;
        };
        // a child's script is in the sandbox; the host's is on disk
        let text = if sandbox.is_some() { rt.read_file(&script) } else { None };
        let src = match text.or_else(|| std::fs::read(&script).ok()) {
            Some(s) => s,
            None if sandbox.is_some() => {
                eprintln!("{}: can't open file '{}': [Errno 2] No such file or directory", rt.host.executable, script);
                return 2;
            }
            None => {
                println!("cannot read {}", script);
                return 2;
            }
        };
        let (name, outdir) = if sandbox.is_some() {
            rt.argv = args.clone();
            (script.clone(), None)
        } else {
            rt.argv = vec![script.clone()];
            (script.rsplit('/').next().unwrap_or(&script).to_string(), args.get(1).cloned())
        };
        (src, name, outdir)
    };
    rt.argv.extend(extra);
    // a -c command is text already; a script's coding comment says how to read it
    let r = if name == "<string>" { rt.run(&String::from_utf8_lossy(&src), &name) } else { rt.run_bytes(&src, &name) };
    use std::io::Write;
    let mut out = std::io::stdout().lock();
    if !r.stdout.is_empty() {
        let s = r.stdout.strip_suffix('\n').unwrap_or(&r.stdout);
        let _ = writeln!(out, "{}", s);
    }
    let _ = out.flush();
    let mut err = std::io::stderr().lock();
    if !r.stderr.is_empty() {
        let _ = write!(err, "{}", r.stderr);
    }
    if !r.ok && !r.traceback.is_empty() {
        let _ = writeln!(err, "{}", r.traceback);
    }
    // a child of os.fork ends here, its changes to the files left for its parent
    #[cfg(unix)]
    if rpy::posix::is_forked_child() {
        rpy::posix::hand_back(&rt.fs);
        let code = if r.ok { 0 } else { r.exit_code };
        let _ = out.flush();
        drop(out);
        if !r.ok && r.error_type == "KeyboardInterrupt" {
            rpy::posix::exit_by_sigint();
        }
        std::process::exit(code);
    }
    if let Some(dir) = outdir {
        for p in rt.changed_files() {
            if let Some(text) = rt.read_file(&p) {
                let fname = p.trim_start_matches('/').replace('/', "__");
                let _ = std::fs::write(format!("{}/{}", dir, fname), text);
            }
        }
    }
    // what the child changed goes back to its parent
    if let Some(p) = &sandbox {
        let back = std::path::Path::new(p).with_extension("out");
        let _ = std::fs::write(back, Sandbox::changes(&rt.fs).encode());
    }
    // an uncaught KeyboardInterrupt ends the process as SIGINT would have, as in CPython
    #[cfg(unix)]
    if !r.ok && r.error_type == "KeyboardInterrupt" {
        let _ = out.flush();
        rpy::posix::exit_by_sigint();
    }
    if r.ok {
        0
    } else {
        r.exit_code
    }
}

#[cfg(not(target_family = "wasm"))]
fn main() {
    // deep Python recursion runs on the Rust stack
    let h = std::thread::Builder::new().stack_size(1 << 30).spawn(run).unwrap();
    // the process's signals go to the interpreter's thread
    #[cfg(unix)]
    rpy::posix::block_signals();
    let code = h.join().unwrap_or(1);
    std::process::exit(code);
}

#[cfg(target_family = "wasm")]
fn main() {
    std::process::exit(run());
}
