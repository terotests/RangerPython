// SPDX-License-Identifier: MIT
//
// runPython(code, options) runs a Python script on an in-memory filesystem.
//
//   const r = runPython(src, { files: { "/data/sales.csv": text } });
//   r.ok, r.stdout, r.error ("ValueError: …"), r.errorType, r.errorLine,
//   r.traceback (CPython-style), r.files ({ path: text } the script wrote),
//   r.removed (paths of the given files the script deleted)
//
// Options:
//   files     { path: text } put in the filesystem before the run
//   cwd       working folder (default "/")
//   argv      sys.argv[1:]
//   maxSteps  stop a script after this many statements (0: no limit)
//   echo      print the script's output to the console as it runs
import { PyRuntime } from "../dist/pyruntime.mjs";

export { PyRuntime };

export function runPython(code, { files = {}, cwd = "/", argv = [], maxSteps = 0, echo = false, scriptName = "main.py" } = {}) {
  const rt = new PyRuntime();
  for (const [p, text] of Object.entries(files)) rt.writeFile(abs(p), String(text));
  rt.clearChanges();
  if (cwd !== "/") rt.setCwd(cwd);
  rt.addArg(scriptName);
  for (const a of argv) rt.addArg(String(a));
  rt.maxSteps = maxSteps;
  rt.echo = echo;
  const ok = rt.run(code);
  const written = {};
  for (const p of rt.changedFiles()) if (rt.hasFile(p)) written[p] = rt.readFile(p);
  return {
    ok,
    stdout: rt.stdout,
    error: rt.error,
    errorType: rt.errorType,
    errorLine: rt.errorLine,
    traceback: rt.traceback,
    files: written,
    removed: rt.removedFiles(),
  };
}

function abs(p) {
  return p.startsWith("/") ? p : "/" + p;
}
