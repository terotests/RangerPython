// SPDX-License-Identifier: MIT
// Runs the wasm32-wasip1 build of rpy in Node:
//   node --stack-size=7000 rust/wasi-run.mjs rpy.wasm script.py [outdir]
// The host filesystem is visible at / so the CLI contract is the same as
// the native binary's.
import { WASI } from "node:wasi";
import fs from "node:fs";

process.removeAllListeners("warning");
const [wasm, ...rest] = process.argv.slice(2);
const wasi = new WASI({ version: "preview1", args: ["rpy", ...rest], env: {}, preopens: { "/": "/" }, returnOnExit: true });
const module = await WebAssembly.compile(fs.readFileSync(wasm));
const instance = await WebAssembly.instantiate(module, wasi.getImportObject());
process.exitCode = wasi.start(instance);
