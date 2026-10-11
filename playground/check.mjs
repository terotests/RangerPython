// SPDX-License-Identifier: MIT
// Runs every example of the playground in Node with the playground's own
// WASI host: node playground/check.mjs path/to/rpy.wasm
import fs from "node:fs";
import { runPython } from "./run.js";
import { examples } from "./examples.js";

const module = new WebAssembly.Module(fs.readFileSync(process.argv[2] ?? "playground/rpy.wasm"));
let failed = 0;
for (const [name, { "main.py": code, ...files }] of Object.entries(examples)) {
  let out = "";
  const r = runPython(module, code, { files, out: (fd, text) => (out += text) });
  // the Exceptions example ends with an uncaught error on purpose
  const want = name === "Exceptions" ? 1 : 0;
  const ok = r.exitCode === want && out.length > 0;
  if (!ok) failed++;
  console.log(`${ok ? "ok  " : "FAIL"} ${name}`);
  if (!ok) console.log(out);
}
process.exitCode = failed ? 1 : 0;
