// SPDX-License-Identifier: MIT
//! rpy [options] script.py [outdir]: runs a script and prints its output;
//! the files it wrote are saved under outdir as a__b.txt for /a/b.txt.
//!
//!   --mount HOST=VFS   put HOST's text files (recursively) at VFS
//!   --path VFS         a folder imports look in after /
//!   --arg X            append X to sys.argv
//!   --echo             stream output as it is written

fn mount(rt: &mut rpy::Runtime, host: &std::path::Path, vfs: &str) {
    let Ok(rd) = std::fs::read_dir(host) else { return };
    for e in rd.flatten() {
        let p = e.path();
        let name = e.file_name().to_string_lossy().to_string();
        let target = format!("{}/{}", vfs.trim_end_matches('/'), name);
        if p.is_dir() {
            mount(rt, &p, &target);
        } else if let Ok(t) = std::fs::read_to_string(&p) {
            // modules and the data files tests read; the VFS holds text only
            rt.write_file(&target, &t);
        }
    }
}

fn run() -> i32 {
    let mut rt = rpy::Runtime::new();
    let mut extra = vec![];
    let mut args: Vec<String> = vec![];
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--mount" => {
                let m = it.next().unwrap_or_default();
                let (h, v) = m.split_once('=').unwrap_or((&m, "/"));
                mount(&mut rt, std::path::Path::new(h), v);
            }
            "--path" => rt.search_path.push(it.next().unwrap_or_default()),
            "--arg" => extra.push(it.next().unwrap_or_default()),
            "--echo" => rt.echo = true,
            _ => args.push(a),
        }
    }
    args.insert(0, String::new());
    if args.len() < 2 {
        println!("usage: rpy [--mount HOST=VFS] [--path VFS] [--arg X] [--echo] script.py [outdir]");
        return 2;
    }
    let script = &args[1];
    let src = match std::fs::read_to_string(script) {
        Ok(s) => s,
        Err(_) => {
            println!("cannot read {}", script);
            return 2;
        }
    };
    rt.argv = vec![script.clone()];
    rt.argv.extend(extra);
    let name = script.rsplit('/').next().unwrap_or(script).to_string();
    let r = rt.run(&src, &name);
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
    if let Some(dir) = args.get(2) {
        for p in rt.changed_files() {
            if let Some(text) = rt.read_file(&p) {
                let fname = p.trim_start_matches('/').replace('/', "__");
                let _ = std::fs::write(format!("{}/{}", dir, fname), text);
            }
        }
    }
    if r.ok { 0 } else { 1 }
}

#[cfg(not(target_family = "wasm"))]
fn main() {
    // deep Python recursion runs on the Rust stack
    let h = std::thread::Builder::new().stack_size(1 << 30).spawn(run).unwrap();
    let code = h.join().unwrap_or(1);
    std::process::exit(code);
}

#[cfg(target_family = "wasm")]
fn main() {
    std::process::exit(run());
}
