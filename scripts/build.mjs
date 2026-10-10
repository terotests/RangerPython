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
console.log("built", path.relative(REPO, path.join(dist, "pyruntime.mjs")), "and pyruntime.cjs");

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
  fs.writeFileSync(file, src);
  if (!fs.existsSync(path.join(dir, "go.mod"))) fs.writeFileSync(path.join(dir, "go.mod"), "module pyrun\n\ngo 1.21\n");
  const r = spawnSync("go", ["build", "-o", "pyrun", "."], { cwd: dir, stdio: "inherit" });
  if (r.status !== 0) throw new Error("go build failed");
  console.log("built build/go/pyrun");
}
