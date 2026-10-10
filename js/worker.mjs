// SPDX-License-Identifier: MIT
//
// A module Worker that runs scripts off the page's thread, so a long or
// endless script does not freeze the page (terminate() the worker to stop it).
//
//   const w = new Worker(new URL("./worker.mjs", import.meta.url), { type: "module" });
//   const run = workerRunner(w);
//   await runInDeck(vfs, doc.id, code, { run });
import { runPython } from "./index.mjs";

if (typeof self !== "undefined" && typeof self.postMessage === "function" && typeof window === "undefined") {
  self.onmessage = (e) => {
    const { id, code, files, options } = e.data || {};
    let result;
    try {
      result = runPython(code, { ...(options || {}), files });
    } catch (err) {
      result = { ok: false, stdout: "", error: "RuntimeError: " + err.message, errorType: "RuntimeError", errorLine: 0, traceback: String(err.stack || err), files: {} };
    }
    self.postMessage({ id, result });
  };
}

export function workerRunner(worker) {
  let next = 1;
  const waiting = new Map();
  worker.addEventListener("message", (e) => {
    const w = waiting.get(e.data.id);
    if (w) {
      waiting.delete(e.data.id);
      w(e.data.result);
    }
  });
  return ({ code, files, options }) => new Promise((resolve) => {
    const id = next++;
    waiting.set(id, resolve);
    worker.postMessage({ id, code, files, options });
  });
}
