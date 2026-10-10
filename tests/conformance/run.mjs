// SPDX-License-Identifier: MIT
//
// Conformance: CPython's own tests, run test by test on this runtime.
//
//   node tests/conformance/run.mjs              run, write results.json and
//                                              docs/COMPATIBILITY.md
//   node tests/conformance/run.mjs --baseline   record what CPython (python3)
//                                              passes, in baseline.json
//   node tests/conformance/run.mjs test_int …   only these files
//
// The files in files.txt are fetched from CPython at TAG into cache/. Each
// runs in its own process on the JavaScript build with driver.py and the
// test.support stand-in in support/. A test counts as supported when it
// passes here; the score is out of the tests CPython itself passes.
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawn, spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

export const TAG = "v3.13.0";
const HERE = path.dirname(fileURLToPath(import.meta.url));
const REPO = path.resolve(HERE, "../..");
const CACHE = path.join(HERE, "cache", TAG);
const args = process.argv.slice(2);
const only = args.filter((a) => !a.startsWith("--"));
const TIMEOUT_MS = Number(process.env.CONFORMANCE_TIMEOUT_MS || 180000);

const entries = fs.readFileSync(path.join(HERE, "files.txt"), "utf8").split("\n")
  .map((l) => l.trim()).filter((l) => l && !l.startsWith("#"));
const testFiles = entries.filter((e) => !e.startsWith("helper:")).filter((n) => only.length === 0 || only.includes(n));
const helpers = entries.filter((e) => e.startsWith("helper:")).map((e) => e.slice(7));

async function fetchAll() {
  for (const rel of [...entries.filter((e) => !e.startsWith("helper:")), ...helpers]) {
    const dest = path.join(CACHE, "test", rel + ".py");
    if (fs.existsSync(dest)) continue;
    const url = `https://raw.githubusercontent.com/python/cpython/${TAG}/Lib/test/${rel}.py`;
    const res = await fetch(url);
    if (!res.ok) throw new Error(`fetch ${url}: HTTP ${res.status}`);
    fs.mkdirSync(path.dirname(dest), { recursive: true });
    fs.writeFileSync(dest, await res.text());
  }
  fs.writeFileSync(path.join(CACHE, "test", "__init__.py"), "");
  // test_types imports one decorator from the test_import package, whose
  // own tests are not part of the suite
  fs.mkdirSync(path.join(CACHE, "test", "test_import"), { recursive: true });
  fs.writeFileSync(path.join(CACHE, "test", "test_import", "__init__.py"), "def no_rerun(reason):\n    return lambda func: func\n");
}

function parse(out) {
  const tests = {};
  let importError = null;
  for (const line of out.split("\n")) {
    const parts = line.split("\t");
    if (parts[0] === "T" && parts.length >= 3) tests[parts[1]] = { status: parts[2], detail: parts[3] || "" };
    else if (parts[0] === "IMPORT") importError = parts[1];
  }
  return { tests, importError };
}

function pool(items, n, fn) {
  let next = 0;
  const results = new Array(items.length);
  const worker = async () => {
    while (next < items.length) {
      const i = next++;
      results[i] = await fn(items[i], i);
    }
  };
  return Promise.all(Array.from({ length: Math.min(n, items.length) }, worker)).then(() => results);
}

function runProcess(cmd, cmdArgs, opts) {
  return new Promise((resolve) => {
    const t0 = Date.now();
    const child = spawn(cmd, cmdArgs, { ...opts, stdio: ["ignore", "pipe", "pipe"] });
    let out = "";
    let err = "";
    child.stdout.on("data", (d) => (out += d));
    child.stderr.on("data", (d) => (err += d));
    const timer = setTimeout(() => child.kill("SIGKILL"), TIMEOUT_MS);
    child.on("close", (code, signal) => {
      clearTimeout(timer);
      resolve({ out, err, code, signal, ms: Date.now() - t0 });
    });
  });
}

async function baseline() {
  // CPython runs the fetched files with its own test.support
  const sandbox = fs.mkdtempSync(path.join(os.tmpdir(), "pyconf-"));
  fs.cpSync(path.join(CACHE, "test"), path.join(sandbox, "test"), { recursive: true });
  const sysSupport = spawnSync("python3", ["-c", "import test.support, os; print(os.path.dirname(test.support.__file__))"], { encoding: "utf8" }).stdout.trim();
  fs.cpSync(sysSupport, path.join(sandbox, "test", "support"), { recursive: true });
  fs.copyFileSync(path.join(HERE, "driver.py"), path.join(sandbox, "driver.py"));
  const version = spawnSync("python3", ["--version"], { encoding: "utf8" }).stdout.trim();
  const file = path.join(HERE, "baseline.json");
  const base = fs.existsSync(file) ? JSON.parse(fs.readFileSync(file, "utf8")) : { files: {} };
  base.cpython = version;
  base.tag = TAG;
  await pool(testFiles, os.cpus().length, async (name) => {
    const r = await runProcess("python3", ["driver.py", "test." + name], { cwd: sandbox, env: { ...process.env, PYTHONPATH: sandbox } });
    const { tests, importError } = parse(r.out);
    if (importError) {
      console.warn(`baseline ${name}: skipped, does not import on CPython: ${importError}`);
      delete base.files[name];
      return;
    }
    base.files[name] = Object.fromEntries(Object.entries(tests).map(([k, v]) => [k, v.status]));
    console.log(`baseline ${name}: ${Object.keys(tests).length} tests`);
  });
  fs.writeFileSync(file, JSON.stringify(base, null, 1) + "\n");
  fs.rmSync(sandbox, { recursive: true, force: true });
}

async function runOurs() {
  const base = JSON.parse(fs.readFileSync(path.join(HERE, "baseline.json"), "utf8"));
  const resultsFile = path.join(HERE, "results.json");
  const prev = fs.existsSync(resultsFile) ? JSON.parse(fs.readFileSync(resultsFile, "utf8")) : { files: {} };
  const results = { tag: TAG, cpython: base.cpython, files: only.length ? prev.files : {} };
  await pool(testFiles, Math.max(1, os.cpus().length - 1), async (name) => {
    const r = await runProcess(process.execPath, ["--stack-size=4000", path.join(HERE, "child.mjs"), CACHE, name], { cwd: REPO });
    const { tests, importError } = parse(r.out);
    const want = base.files[name] || {};
    const file = { ms: r.ms, importError, crashed: null, tests: {} };
    if (r.signal) file.crashed = `killed after ${Math.round(r.ms / 1000)} s`;
    else if (r.code !== 0) file.crashed = (r.err.trim().split("\n").pop() || "exit " + r.code).slice(0, 240);
    for (const [test, status] of Object.entries(want)) {
      const got = tests[test];
      file.tests[test] = { cpython: status, ours: got ? got.status : "missing", detail: got ? got.detail : (importError || file.crashed || "not run") };
    }
    results.files[name] = file;
    const s = summarize(file);
    console.log(`${name.padEnd(22)} ${String(s.pass).padStart(4)} / ${String(s.total).padStart(4)}  ${s.pct.toFixed(1).padStart(5)} %  ${(r.ms / 1000).toFixed(1)} s${importError ? "  import: " + importError.slice(0, 80) : ""}${file.crashed ? "  crashed: " + file.crashed.slice(0, 80) : ""}`);
  });
  fs.writeFileSync(resultsFile, JSON.stringify(results, null, 1) + "\n");
  writeReport(results);
}

function summarize(file) {
  let total = 0;
  let pass = 0;
  for (const t of Object.values(file.tests)) {
    if (t.cpython !== "pass") continue;
    total++;
    if (t.ours === "pass") pass++;
  }
  return { total, pass, pct: total ? (100 * pass) / total : 0 };
}

function writeReport(results) {
  const names = Object.keys(results.files).sort();
  let total = 0;
  let pass = 0;
  const rows = [];
  const reasons = new Map();
  for (const n of names) {
    const f = results.files[n];
    const s = summarize(f);
    total += s.total;
    pass += s.pass;
    let note = f.importError ? "does not import: " + f.importError : f.crashed ? "run stopped: " + f.crashed : "";
    rows.push(`| ${n} | ${s.pass} | ${s.total} | ${s.pct.toFixed(1)} % | ${note.replace(/\|/g, "\\|").slice(0, 120)} |`);
    for (const t of Object.values(f.tests)) {
      if (t.cpython !== "pass" || t.ours === "pass") continue;
      const key = reasonOf(t);
      reasons.set(key, (reasons.get(key) || 0) + 1);
    }
  }
  const top = [...reasons.entries()].sort((a, b) => b[1] - a[1]).slice(0, 40);
  const md = `# Python compatibility

Generated by \`node tests/conformance/run.mjs\` — do not edit.

CPython's own test files (Lib/test at ${results.tag}) run test by test on
the JavaScript build of this runtime. A test counts when it passes here; the
score is out of the tests CPython (${results.cpython}) passes with the same
driver. Tests that need CPython internals (reference counts, object sizes,
subprocesses) are skipped by the test.support stand-in and are not counted.

**${pass} of ${total} tests pass: ${total ? ((100 * pass) / total).toFixed(1) : 0} %**

| file | pass | of | score | note |
|---|---:|---:|---:|---|
${rows.join("\n")}

## Why tests fail

The most common reasons, by the error a failing test ends with:

| tests | reason |
|---:|---|
${top.map(([k, v]) => `| ${v} | ${k.replace(/\|/g, "\\|")} |`).join("\n")}

Run \`node tests/conformance/run.mjs\` for the full list in
tests/conformance/results.json.
`;
  fs.mkdirSync(path.join(REPO, "docs"), { recursive: true });
  fs.writeFileSync(path.join(REPO, "docs/COMPATIBILITY.md"), md);
  console.log(`\n${pass} of ${total} CPython-passing tests pass (${total ? ((100 * pass) / total).toFixed(1) : 0} %); report in docs/COMPATIBILITY.md`);
}

// a failure reason without the values in it, so alike failures group
function reasonOf(t) {
  let d = t.detail || t.ours;
  if (t.ours === "missing") return "not run: " + d.replace(/'[^']*'/g, "'…'").slice(0, 100);
  d = d.replace(/'[^']*'/g, "'…'").replace(/"[^"]*"/g, '"…"').replace(/\b\d+(\.\d+)?\b/g, "N");
  if (t.ours === "skip") return "skipped here: " + d.slice(0, 100);
  if (t.ours === "fail" && d.startsWith("AssertionError")) return "wrong result (AssertionError)";
  return d.slice(0, 110);
}

await fetchAll();
if (args.includes("--baseline")) await baseline();
else await runOurs();
