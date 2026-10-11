// SPDX-License-Identifier: MIT
// Runs scripts off the page's thread so a long or endless script never
// freezes it; the page stops one by terminating this worker.
import { runPython } from "./run.js";

const ready = WebAssembly.compileStreaming(fetch("rpy.wasm"));

ready.then(
  () => postMessage({ type: "ready" }),
  (e) => postMessage({ type: "fatal", message: String(e) }),
);

onmessage = async ({ data: { code, files } }) => {
  const module = await ready;
  // output is sent in batches: one message per write is slow for chatty loops
  let buf = [], last = performance.now();
  const flush = () => {
    if (buf.length) postMessage({ type: "out", chunks: buf });
    buf = [];
    last = performance.now();
  };
  const t0 = performance.now();
  try {
    const r = runPython(module, code, {
      files,
      out(fd, text) {
        buf.push([fd, text]);
        if (performance.now() - last > 50) flush();
      },
    });
    flush();
    postMessage({ type: "done", exitCode: r.exitCode, files: r.files, ms: performance.now() - t0 });
  } catch (e) {
    flush();
    // wasm calls run on the browser's own stack, which holds a few hundred
    // nested Python calls; the native rpy goes much deeper
    const msg = e instanceof RangeError ? "RecursionError: the browser's stack ran out (it holds a few hundred nested Python calls)" : String(e);
    postMessage({ type: "done", exitCode: -1, crash: msg, files: {}, ms: performance.now() - t0 });
  }
};
