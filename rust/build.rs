// Embeds the Python standard library (../lib and pylib) into the binary.
use std::fs;
use std::path::{Path, PathBuf};

fn walk(root: &Path, dir: &Path, out: &mut Vec<(String, PathBuf)>) {
    let mut entries: Vec<_> = fs::read_dir(dir).unwrap().flatten().map(|e| e.path()).collect();
    entries.sort();
    for p in entries {
        if p.is_dir() {
            // modules that need OS threads have a folder of their own
            if p.ends_with("pylib/threads") {
                continue;
            }
            walk(root, &p, out);
        } else if p.extension().map_or(false, |e| e == "py") {
            let rel = p.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/");
            out.push((rel, fs::canonicalize(&p).unwrap()));
        }
    }
}

fn main() {
    let mut files = Vec::new();
    // threading on OS threads, ahead of lib's single-thread one, where the
    // target has threads (the native _thread module is built there too)
    let threads = std::env::var("CARGO_CFG_TARGET_FAMILY").map_or(true, |f| f != "wasm");
    let roots: &[&str] = if threads { &["pylib/threads", "pylib", "../lib"] } else { &["pylib", "../lib"] };
    for &root in roots {
        let r = Path::new(root);
        println!("cargo:rerun-if-changed={}", root);
        if r.is_dir() {
            walk(r, r, &mut files);
        }
    }
    let mut src = String::from("pub fn source(path: &str) -> Option<&'static str> {\n    match path {\n");
    let mut seen = std::collections::HashSet::new();
    for (rel, abs) in &files {
        if seen.insert(rel.clone()) {
            src.push_str(&format!("        {:?} => Some(include_str!({:?})),\n", rel, abs.to_string_lossy()));
        }
    }
    src.push_str("        _ => None,\n    }\n}\n");
    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("stdlib.rs");
    fs::write(out, src).unwrap();
    let names = Path::new("../src/PyUnicodeNames.rgr");
    println!("cargo:rerun-if-changed=../src/PyUnicodeNames.rgr");
    let text = fs::read_to_string(names).unwrap_or_default();
    let table = text.find("return \";").map(|i| {
        let rest = &text[i + 8..];
        rest[..rest.find('"').unwrap_or(0)].to_string()
    }).unwrap_or_default();
    fs::write(PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("unames.txt"), table).unwrap();
}

