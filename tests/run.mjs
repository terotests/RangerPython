// SPDX-License-Identifier: MIT
//
//   node tests/run.mjs                 the JavaScript build (dist/)
//   node tests/run.mjs --go --python   also the Go and Python builds (build/)
//   node tests/run.mjs --update        rewrite tests/expected from CPython
//
// Each tests/scripts/NAME.py runs on the runtime; its output must equal
// tests/expected/NAME.out and each file it wrote must equal
// tests/expected/NAME.files/<path>, both recorded from CPython (python3).
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { runPython } from "../js/index.mjs";

const HERE = path.dirname(fileURLToPath(import.meta.url));
const REPO = path.resolve(HERE, "..");
const SCRIPTS = path.join(HERE, "scripts");
const EXPECTED = path.join(HERE, "expected");
const args = new Set(process.argv.slice(2));
const only = process.argv.slice(2).filter((a) => !a.startsWith("--"));

const names = fs.readdirSync(SCRIPTS).filter((f) => f.endsWith(".py")).map((f) => f.slice(0, -3)).sort()
  .filter((n) => only.length === 0 || only.includes(n));

function runJs(name) {
  const code = fs.readFileSync(path.join(SCRIPTS, name + ".py"), "utf8");
  const r = runPython(code, { scriptName: name + ".py", maxSteps: 50_000_000 });
  return { out: r.stdout + (r.ok ? "" : r.traceback + "\n"), files: r.files };
}

// the command line builds save what the script wrote as outdir/a__b.txt for /a/b.txt
function runCli(cmd, cmdArgs, name) {
  const outdir = fs.mkdtempSync(path.join(os.tmpdir(), "pyrt-"));
  const r = spawnSync(cmd, [...cmdArgs, path.join(SCRIPTS, name + ".py"), outdir], { encoding: "utf8", maxBuffer: 64 * 1024 * 1024 });
  const files = {};
  for (const f of fs.readdirSync(outdir)) files["/" + f.split("__").join("/")] = fs.readFileSync(path.join(outdir, f), "utf8");
  fs.rmSync(outdir, { recursive: true, force: true });
  let out = (r.stdout || "") + (r.stderr || "");
  if (out.length > 0 && !out.endsWith("\n")) out += "\n";
  return { out, files };
}

function expectedFiles(name) {
  const dir = path.join(EXPECTED, name + ".files");
  const files = {};
  const walk = (d) => {
    if (!fs.existsSync(d)) return;
    for (const ent of fs.readdirSync(d, { withFileTypes: true })) {
      const full = path.join(d, ent.name);
      if (ent.isDirectory()) walk(full);
      else files["/" + path.relative(dir, full).split(path.sep).join("/")] = fs.readFileSync(full, "utf8");
    }
  };
  walk(dir);
  return files;
}

if (args.has("--update")) {
  for (const name of names) {
    const r = spawnSync("python3", [path.join(SCRIPTS, name + ".py")], { encoding: "utf8", cwd: os.tmpdir() });
    if (r.status !== 0) throw new Error(name + ": CPython failed\n" + r.stderr);
    fs.writeFileSync(path.join(EXPECTED, name + ".out"), r.stdout);
    const dir = path.join(EXPECTED, name + ".files");
    fs.rmSync(dir, { recursive: true, force: true });
    // the files the runtime writes, as CPython wrote them
    for (const p of Object.keys(runJs(name).files)) {
      const dest = path.join(dir, ...p.split("/").filter(Boolean));
      fs.mkdirSync(path.dirname(dest), { recursive: true });
      fs.copyFileSync(p, dest);
    }
    console.log("recorded", name);
  }
  process.exit(0);
}

const targets = [["js", runJs]];
if (args.has("--python")) targets.push(["python", (n) => runCli("python3", [path.join(REPO, "build/python/pyrun.py")], n)]);
if (args.has("--go")) targets.push(["go", (n) => runCli(path.join(REPO, "build/go/pyrun"), [], n)]);
if (args.has("--rust")) targets.push(["rust", (n) => runCli(path.join(REPO, "build/rust/pyrun"), [], n)]);

let failed = 0;
for (const [target, run] of targets) {
  for (const name of names) {
    const want = fs.readFileSync(path.join(EXPECTED, name + ".out"), "utf8");
    const wantFiles = expectedFiles(name);
    const t0 = Date.now();
    const got = run(name);
    const problems = [];
    if (got.out !== want) {
      const a = want.split("\n");
      const b = got.out.split("\n");
      const i = a.findIndex((l, k) => l !== b[k]);
      problems.push(`output differs at line ${i + 1}:\n    want: ${a[i]}\n    got:  ${b[i]}`);
    }
    for (const [p, text] of Object.entries(wantFiles)) {
      if (!(p in got.files)) problems.push(`did not write ${p}`);
      else if (got.files[p] !== text) problems.push(`${p} differs`);
    }
    for (const p of Object.keys(got.files)) if (!(p in wantFiles)) problems.push(`wrote ${p}, CPython did not`);
    const ms = Date.now() - t0;
    if (problems.length) {
      failed++;
      console.log(`FAIL ${target} ${name}\n  ${problems.join("\n  ")}`);
    } else {
      console.log(`ok   ${target} ${name} (${ms} ms)`);
    }
  }
}
if (failed) {
  console.log(`${failed} failed`);
  process.exit(1);
}
