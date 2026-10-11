// SPDX-License-Identifier: MIT
//
// rpy's pandas against real pandas: each pd_*.py prints what an analysis
// script prints, and must print exactly what pandas printed (recorded in
// pd_*.out).
//
//   node tests/pandas/run.mjs            the native build (rust/target/release/rpy)
//   node tests/pandas/run.mjs --wasm     the wasm build, through rust/wasi-run.mjs
//   node tests/pandas/run.mjs --record   record pandas' output in pd_*.out
//                                        (needs python3 with pandas)
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const HERE = path.dirname(fileURLToPath(import.meta.url));
const RUST = path.resolve(HERE, "../../rust");
const args = new Set(process.argv.slice(2));
const cases = fs.readdirSync(HERE).filter((f) => /^pd_.*\.py$/.test(f)).sort();

function run(cmd, argv) {
  const r = spawnSync(cmd, argv, { cwd: HERE, encoding: "utf8", maxBuffer: 1 << 26 });
  return (r.stdout || "") + (r.stderr || "");
}

let failed = 0;
for (const f of cases) {
  const out = path.join(HERE, f.replace(/\.py$/, ".out"));
  if (args.has("--record")) {
    fs.writeFileSync(out, run("python3", ["-P", f]));
    console.log("recorded", f);
    continue;
  }
  const got = args.has("--wasm")
    ? run("node", ["--stack-size=7000", path.join(RUST, "wasi-run.mjs"), path.join(RUST, "target/wasm32-wasip1/release/rpy.wasm"), path.join(HERE, f)])
    : run(path.join(RUST, "target/release/rpy"), [f]);
  const want = fs.readFileSync(out, "utf8");
  if (got === want) {
    console.log("ok  ", f);
  } else {
    failed++;
    const a = want.split("\n");
    const b = got.split("\n");
    let i = 0;
    while (i < a.length && a[i] === b[i]) i++;
    console.log("FAIL", f, "first difference at line", i + 1);
    console.log("  pandas: " + JSON.stringify(a[i]));
    console.log("  rpy:    " + JSON.stringify(b[i]));
  }
}
if (!args.has("--record")) {
  console.log(`${cases.length - failed} of ${cases.length} match pandas`);
  process.exit(failed ? 1 : 0);
}
