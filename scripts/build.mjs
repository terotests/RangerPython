// SPDX-License-Identifier: MIT
//
//   node scripts/build.mjs            dist/pyruntime.mjs (ESM, browser and Node)
//                                     dist/pyruntime.cjs (CommonJS)
//   node scripts/build.mjs --go       build/go/pyrun      (needs Go)
//   node scripts/build.mjs --python   build/python/pyrun.py
//
// The Go and Python builds are command line runners (src/PyMain.rgr) used
// by tests/run.mjs to check that the runtime answers the same on every
// target Ranger compiles it to.
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { REPO, compile } from "./compiler.mjs";
import { writeStdlib } from "./stdlib.mjs";

const args = new Set(process.argv.slice(2));
const dist = path.join(REPO, "dist");

writeStdlib();
compile(["-es6", "-esm", "src/PyRuntime.rgr", "-d=dist", "-o=pyruntime.mjs"]);
compile(["-es6", "-nodemodule", "src/PyRuntime.rgr", "-d=dist", "-o=pyruntime.cjs"]);
for (const f of ["pyruntime.mjs", "pyruntime.cjs"]) lazyFields(path.join(dist, f));
console.log("built", path.relative(REPO, path.join(dist, "pyruntime.mjs")), "and pyruntime.cjs");

// Every value is a PyVal, and its constructor gave each one nine empty
// lists and maps that most values (numbers, strings) never touch. In the
// JavaScript build they are made on first use instead, which cuts the
// garbage an arithmetic loop makes by about ten times.
function lazyFields(file) {
  const fields = ["items", "vals", "index", "attrs", "akeys", "defs", "bases", "cps", "slotNames"];
  let src = fs.readFileSync(file, "utf8");
  const start = src.indexOf("class PyVal  {");
  const ctor = src.indexOf("constructor(tag) {", start);
  const end = src.indexOf("\n  }\n", ctor);
  if (start < 0 || ctor < 0 || end < 0) throw new Error("PyVal constructor not found in " + file);
  let body = src.slice(ctor, end);
  const accessors = [];
  for (const f of fields) {
    const m = new RegExp(`\\n    this\\.${f} = (\\[\\]|\\{\\});`).exec(body);
    if (!m) throw new Error(`PyVal.${f} initialiser not found in ` + file);
    body = body.replace(m[0], `\n    this._z_${f} = undefined;`);
    accessors.push(`  get ${f}() { const v = this._z_${f}; return v === undefined ? (this._z_${f} = ${m[1]}) : v; }\n  set ${f}(v) { this._z_${f} = v; }\n`);
  }
  src = src.slice(0, ctor) + body + "\n  }\n" + accessors.join("") + src.slice(end + 5);
  fs.writeFileSync(file, src);
}

if (args.has("--python")) {
  compile(["-l=python", "src/PyMain.rgr", "-d=build/python", "-o=pyrun.py"]);
  console.log("built build/python/pyrun.py");
}

if (args.has("--rust")) {
  // Rust has no exceptions; a copy of src/ with each try/catch rewritten
  // (rustTry) compiles to one file that rustc builds without Cargo
  const src = path.join(REPO, "build/rust/src");
  fs.mkdirSync(src, { recursive: true });
  for (const f of fs.readdirSync(path.join(REPO, "src"))) {
    if (!f.endsWith(".rgr")) continue;
    let text = fs.readFileSync(path.join(REPO, "src", f), "utf8");
    if (f !== "PyStdlib.rgr" && f !== "PyUnicodeNames.rgr") text = rustTry(text);
    fs.writeFileSync(path.join(src, f), text);
  }
  compile(["-l=rust", "build/rust/src/PyMain.rgr", "-d=build/rust", "-o=pyrun.rs"]);
  const file = path.join(REPO, "build/rust/pyrun.rs");
  fs.writeFileSync(file, rustFixups(fs.readFileSync(file, "utf8")));
  const r = spawnSync("rustc", ["--edition", "2021", "-O", "-C", "target-cpu=native", "pyrun.rs", "-o", "pyrun"], { cwd: path.dirname(file), stdio: "inherit" });
  if (r.status !== 0) throw new Error("rustc failed");
  console.log("built build/rust/pyrun");
}

if (args.has("--go")) {
  const dir = path.join(REPO, "build/go");
  compile(["-l=go", "src/PyMain.rgr", "-d=build/go", "-o=pyrun.go"]);
  // ranger-compiler 4.0.2 passes an enum-typed constructor argument as a
  // pointer in Go; the constructors take the value
  const file = path.join(dir, "pyrun.go");
  const src = fs.readFileSync(file, "utf8")
    .replace("func CreateNew_PyNode(k *PyK, ", "func CreateNew_PyNode(k PyK, ")
    .replace("func CreateNew_PyVal(tag *PyT)", "func CreateNew_PyVal(tag PyT)");
  fs.writeFileSync(file, inlineGoOptionals(lazyGoFields(src)));
  if (!fs.existsSync(path.join(dir, "go.mod"))) fs.writeFileSync(path.join(dir, "go.mod"), "module pyrun\n\ngo 1.21\n");
  goBuild(dir, file);
  console.log("built build/go/pyrun");
}

// The same for Go: a PyVal's lists start nil (append and len take nil
// slices) and its maps are made at the first write (reads take nil maps).
function lazyGoFields(src) {
  const maps = { index: "map[string]int64", attrs: "map[string]*PyVal", defs: "map[string]*PyVal", slotNames: "map[string]bool" };
  const ctor = src.indexOf("func CreateNew_PyVal(tag PyT) *PyVal {");
  const end = src.indexOf("\n}\n", ctor);
  if (ctor < 0 || end < 0) throw new Error("CreateNew_PyVal not found in the Go build");
  let body = src.slice(ctor, end);
  for (const f of ["items", "vals", "akeys", "bases", "cps", ...Object.keys(maps)]) {
    const line = new RegExp(`\\n  me\\.${f} = make\\([^\\n]*`);
    if (!line.test(body)) throw new Error(`PyVal.${f} initialiser not found in the Go build`);
    body = body.replace(line, "");
  }
  src = src.slice(0, ctor) + body + src.slice(end);
  // every write to one of those maps through a field makes the map first
  const write = /^(\s*)([\w.]+)\.(index|attrs|defs|slotNames)\[(.*)\] = (.*)$/gm;
  src = src.replace(write, (m, ind, obj, f) => `${ind}if ${obj}.${f} == nil { ${obj}.${f} = make(${maps[f]}) }\n${m}`);
  // and so does handing one of them on to something else that may write it
  const alias = /^(\s*)(.+) = ([\w.]+)\.(index|attrs|defs|slotNames);?\s*$/gm;
  return src.replace(alias, (m, ind, lhs, obj, f) => `${ind}if ${obj}.${f} == nil { ${obj}.${f} = make(${maps[f]}) }\n${m}`);
}

// A PyVal's optional fields were ten separately allocated *GoNullable
// boxes per value; they become plain GoNullable fields inside the struct.
// The places that hand such a field on as a *GoNullable then take its
// address, which keeps the sharing they had; go build names them.
function inlineGoOptionals(src) {
  const PYVAL_OPTIONALS = ["cls", "__self", "wrapped", "setter", "node", "scope", "gen", "cur", "dictv", "owner"];
  const a = src.indexOf("type PyVal struct");
  const b = src.indexOf("\n}\n", a);
  let st = src.slice(a, b);
  for (const f of PYVAL_OPTIONALS) st = st.replace(new RegExp(`(\\n  ${f} )\\*GoNullable`), "$1GoNullable");
  src = src.slice(0, a) + st + src.slice(b);
  const c = src.indexOf("func CreateNew_PyVal(tag PyT) *PyVal {");
  const d = src.indexOf("\n}\n", c);
  let body = src.slice(c, d);
  for (const f of PYVAL_OPTIONALS) body = body.replace(`\n  me.${f} = new(GoNullable);`, "");
  return src.slice(0, c) + body + src.slice(d);
}

function goBuild(dir, file) {
  for (let round = 0; round < 20; round++) {
    const r = spawnSync("go", ["build", "-gcflags=-e", "-o", "pyrun", "."], { cwd: dir, encoding: "utf8" });
    if (r.status === 0) return;
    const errs = [...r.stderr.matchAll(/pyrun\.go:(\d+):(\d+): cannot use (\S+) \(variable of struct type GoNullable\) as \*GoNullable/g)];
    if (errs.length === 0) {
      process.stderr.write(r.stderr);
      throw new Error("go build failed");
    }
    const lines = fs.readFileSync(file, "utf8").split("\n");
    errs.sort((x, y) => x[1] - y[1] || y[2] - x[2]);
    for (const [, ln, col, expr] of errs) {
      const line = lines[ln - 1];
      const at = col - 1;
      if (line.slice(at, at + expr.length) !== expr) throw new Error(`go build: no ${expr} at pyrun.go:${ln}:${col}`);
      lines[ln - 1] = line.slice(0, at) + "(&" + expr + ")" + line.slice(at + expr.length);
    }
    fs.writeFileSync(file, lines.join("\n"));
  }
  throw new Error("go build failed: too many rounds");
}

// `try { A } { B }` becomes
//   def __tryN:boolean false
//   py_rust_try __tryN { A }
//   if __tryN { B }
// with error_msg in B read from the caught panic. The fields TRUE and FALSE
// are renamed, since the Rust writer lower-cases them into keywords.
function rustTry(text) {
  text = text.replace(/\bTRUE\b/g, "PY_TRUE").replace(/\bFALSE\b/g, "PY_FALSE");
  text = oneElementLists(text);
  let n = 0;
  for (;;) {
    const m = /^([ \t]*)try \{[ \t]*$/m.exec(text);
    if (!m) return text;
    const ind = m[1];
    const bodyStart = m.index + m[0].length;
    const bodyEnd = closeBrace(text, bodyStart);
    const afterBody = text.slice(bodyEnd + 1);
    const cm = /^[ \t]*\{/.exec(afterBody);
    if (!cm) throw new Error("try without a catch block at " + text.slice(m.index, m.index + 80));
    const catchStart = bodyEnd + 1 + cm[0].length;
    const catchEnd = closeBrace(text, catchStart);
    const body = text.slice(bodyStart, bodyEnd);
    const handler = text.slice(catchStart, catchEnd).replace(/\(error_msg\)/g, "(py_rust_err)");
    const v = "__try" + n++;
    text = text.slice(0, m.index) +
      `${ind}def ${v}:boolean false\n${ind}py_rust_try ${v} {${body}}\n${ind}if ${v} {${handler}}` +
      text.slice(catchEnd + 1);
  }
}

function closeBrace(text, i) {
  let depth = 1;
  for (; i < text.length; i++) {
    const c = text[i];
    if (c === '"') {
      for (i++; i < text.length && text[i] !== '"'; i++) if (text[i] === "\\") i++;
    } else if (c === ";") {
      while (i < text.length && text[i] !== "\n") i++;
    } else if (c === "{") depth++;
    else if (c === "}" && --depth === 0) return i;
  }
  throw new Error("unbalanced braces");
}

// what the Rust writer leaves for rustc to refuse
function rustFixups(src) {
  // The runtime was written for shared, mutable objects (JavaScript, Go,
  // Python): the interpreter re-enters its builtins and they re-enter it,
  // and the Rust writer's RefCell turns each such overlap into a "already
  // borrowed" panic. This build swaps in a cell without the run-time check,
  // which gives the same aliasing the other targets have.
  src = src.replace("use std::cell::RefCell;", `use std::cell::UnsafeCell;
pub struct RefCell<T: ?Sized> { v: UnsafeCell<T> }
impl<T> RefCell<T> {
    #[inline(always)] pub fn new(v: T) -> RefCell<T> { RefCell { v: UnsafeCell::new(v) } }
}
impl<T: ?Sized> RefCell<T> {
    #[inline(always)] pub fn borrow(&self) -> &T { unsafe { &*self.v.get() } }
    #[inline(always)] #[allow(clippy::mut_from_ref)] pub fn borrow_mut(&self) -> &mut T { unsafe { &mut *self.v.get() } }
}
impl<T: Clone> Clone for RefCell<T> { fn clone(&self) -> Self { RefCell::new(self.borrow().clone()) } }
impl<T: Default> Default for RefCell<T> { fn default() -> Self { RefCell::new(T::default()) } }`);
  // Every Python value, scope and call frame is a small heap block, and the
  // system malloc was close to half the run time. Blocks up to 512 bytes
  // come from per-size free lists; nothing small goes back to the system.
  src += `
mod rg_alloc {
    use std::alloc::{GlobalAlloc, Layout, System};
    use std::cell::UnsafeCell;
    use std::sync::atomic::{AtomicBool, Ordering};
    const STEP: usize = 16;
    const CLASSES: usize = 32;
    const CHUNK: usize = 1 << 20;
    pub struct RgAlloc { lock: AtomicBool, free: UnsafeCell<[*mut u8; CLASSES]>, cur: UnsafeCell<(*mut u8, usize)> }
    unsafe impl Sync for RgAlloc {}
    impl RgAlloc {
        pub const fn new() -> Self {
            RgAlloc { lock: AtomicBool::new(false), free: UnsafeCell::new([std::ptr::null_mut(); CLASSES]), cur: UnsafeCell::new((std::ptr::null_mut(), 0)) }
        }
        #[inline(always)] fn lock(&self) { while self.lock.compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed).is_err() { std::hint::spin_loop(); } }
        #[inline(always)] fn unlock(&self) { self.lock.store(false, Ordering::Release); }
    }
    unsafe impl GlobalAlloc for RgAlloc {
        #[inline]
        unsafe fn alloc(&self, l: Layout) -> *mut u8 {
            let sz = l.size();
            if sz == 0 || sz > STEP * CLASSES || l.align() > STEP { return System.alloc(l); }
            let c = (sz - 1) / STEP;
            self.lock();
            let fl = &mut (*self.free.get())[c];
            if !fl.is_null() {
                let p = *fl;
                *fl = *(p as *mut *mut u8);
                self.unlock();
                return p;
            }
            let need = (c + 1) * STEP;
            let cur = &mut *self.cur.get();
            if cur.1 < need {
                let chunk = System.alloc(Layout::from_size_align_unchecked(CHUNK, STEP));
                if chunk.is_null() { self.unlock(); return chunk; }
                *cur = (chunk, CHUNK);
            }
            let p = cur.0;
            cur.0 = p.add(need);
            cur.1 -= need;
            self.unlock();
            p
        }
        #[inline]
        unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
            let sz = l.size();
            if sz == 0 || sz > STEP * CLASSES || l.align() > STEP { return System.dealloc(p, l); }
            let c = (sz - 1) / STEP;
            self.lock();
            let fl = &mut (*self.free.get())[c];
            *(p as *mut *mut u8) = *fl;
            *fl = p;
            self.unlock();
        }
    }
}
#[global_allocator]
static RG_GLOBAL: rg_alloc::RgAlloc = rg_alloc::RgAlloc::new();
`;
  return src
    .replace(/\.borrow\(\)\.(\w+)\.rg_remove\(/g, ".borrow_mut().$1.rg_remove(")
    // a non-optional class field read into a local stays an Option
    .replace(/PyModules::make\(&mdl, /g, "PyModules::make(mdl.as_ref().unwrap(), ");
}

// ([] _:PyVal ((call))) comes out of the Rust writer as the bare element
function oneElementLists(text) {
  const head = "([] _:PyVal (";
  let out = "";
  let i = 0;
  for (;;) {
    const at = text.indexOf(head, i);
    if (at < 0) return out + text.slice(i);
    const open = at + head.length - 1;
    const close = closeParen(text, open + 1);
    const inner = text.slice(open + 1, close);
    const end = closeParen(text, at + 1);
    if (inner.startsWith("(") && closeParen(inner, 1) === inner.length - 1 && text.slice(close + 1, end).trim() === "") {
      out += text.slice(i, at) + "(PyVal.one(" + oneElementLists(inner) + "))";
      i = end + 1;
    } else {
      out += text.slice(i, open + 1);
      i = open + 1;
    }
  }
}

function closeParen(text, i) {
  let depth = 1;
  for (; i < text.length; i++) {
    const c = text[i];
    if (c === '"') {
      for (i++; i < text.length && text[i] !== '"'; i++) if (text[i] === "\\") i++;
    } else if (c === "(") depth++;
    else if (c === ")" && --depth === 0) return i;
  }
  throw new Error("unbalanced parentheses");
}
