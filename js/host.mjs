// SPDX-License-Identifier: MIT
//
// runOnHost(code, host, options) runs a script against a filesystem the
// host keeps. The host needs three functions, each may be async:
//
//   list()              the files to mount: { path: text } or [[path, text]]
//   write(path, text)   called for each file the script wrote
//   remove(path)        called for each mounted file the script deleted
//
// Files are copied in before the run and the changes handed back after it,
// so a script never waits on the host. Options are runPython's, plus `run`
// to run elsewhere (see js/worker.mjs).
import { runPython } from "./index.mjs";

export async function runOnHost(code, host, { run, ...options } = {}) {
  const listed = await host.list();
  const files = {};
  for (const [p, text] of Array.isArray(listed) ? listed : Object.entries(listed || {})) {
    if (typeof text === "string") files[p.startsWith("/") ? p : "/" + p] = text;
  }
  const result = run ? await run({ code, files, options }) : runPython(code, { ...options, files });
  for (const [p, text] of Object.entries(result.files || {})) await host.write(p, text);
  for (const p of result.removed || []) await host.remove(p);
  return result;
}
