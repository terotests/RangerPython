// SPDX-License-Identifier: MIT
//! rpy script.py [outdir]: runs a script and prints its output; the files it
//! wrote are saved under outdir as a__b.txt for /a/b.txt.

fn run() -> i32 {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("usage: rpy script.py [outdir]");
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
    let mut rt = rpy::Runtime::new();
    rt.argv = vec![script.clone()];
    let name = script.rsplit('/').next().unwrap_or(script).to_string();
    let r = rt.run(&src, &name);
    use std::io::Write;
    let mut out = std::io::stdout().lock();
    if !r.stdout.is_empty() {
        let s = r.stdout.strip_suffix('\n').unwrap_or(&r.stdout);
        let _ = writeln!(out, "{}", s);
    }
    if !r.stderr.is_empty() {
        let _ = write!(out, "{}", r.stderr);
    }
    if !r.ok && !r.traceback.is_empty() {
        let _ = writeln!(out, "{}", r.traceback);
    }
    let _ = out.flush();
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
