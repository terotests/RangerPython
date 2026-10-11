// SPDX-License-Identifier: MIT
// Runs a Python script with rpy.wasm on in-memory files:
//   const r = runPython(module, code, { files: { "data.csv": text }, out: (fd, text) => … });
//   r.exitCode, r.files  ({ "/tmp/chart.svg": Uint8Array }: the files the script wrote)
import { WasiHost } from "./wasi.js";

export function runPython(module, code, { files = {}, args = [], out = () => {} } = {}) {
  const host = new WasiHost({ out });
  // the user's files are mounted at the interpreter's "/", next to main.py
  for (const [name, data] of Object.entries(files)) host.writeFile("/in/" + name, data);
  host.writeFile("/in/main.py", code);
  host.dirs.add("/out");
  host.args = ["rpy", "--echo", "--mount", "/in=/", "/in/main.py", "/out", ...args.flatMap((a) => ["--arg", a])];
  const instance = new WebAssembly.Instance(module, host.imports());
  const exitCode = host.start(instance);
  // rpy saves /a/b.txt as a__b.txt
  const written = {};
  for (const [name, data] of Object.entries(host.listFiles("/out"))) written["/" + name.replaceAll("__", "/")] = data;
  return { exitCode, files: written };
}
