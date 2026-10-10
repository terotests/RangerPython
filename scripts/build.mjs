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
