#!/usr/bin/env node
// SPDX-License-Identifier: MIT
//
//   node js/cli.mjs script.py [args…] [--mount DIR] [--max-steps N]
//
// Runs a script on the in-memory filesystem. With --mount, DIR's files are
// the filesystem's root before the run and what the script wrote is saved
// back under DIR afterwards; without it the files written are listed.
import fs from "node:fs";
import path from "node:path";
import { runPython } from "./index.mjs";

const argv = process.argv.slice(2);
let mount = null;
let maxSteps = 0;
const rest = [];
for (let i = 0; i < argv.length; i++) {
  if (argv[i] === "--mount") mount = argv[++i];
  else if (argv[i] === "--max-steps") maxSteps = Number(argv[++i]);
  else rest.push(argv[i]);
}
if (rest.length === 0) {
  console.error("usage: pyrun script.py [args…] [--mount DIR] [--max-steps N]");
  process.exit(2);
}
const [script, ...args] = rest;
const code = fs.readFileSync(script, "utf8");

const files = {};
if (mount) {
  const walk = (dir) => {
    for (const ent of fs.readdirSync(dir, { withFileTypes: true })) {
      const full = path.join(dir, ent.name);
      if (ent.isDirectory()) walk(full);
      else if (ent.isFile() && fs.statSync(full).size < 16 * 1024 * 1024) {
        files["/" + path.relative(mount, full).split(path.sep).join("/")] = fs.readFileSync(full, "utf8");
      }
    }
  };
  walk(mount);
}

const r = runPython(code, { files, argv: args, maxSteps, scriptName: path.basename(script) });
process.stdout.write(r.stdout);
for (const [p, text] of Object.entries(r.files)) {
  if (mount) {
    const out = path.join(mount, ...p.split("/").filter(Boolean));
    fs.mkdirSync(path.dirname(out), { recursive: true });
    fs.writeFileSync(out, text);
    console.error("[saved " + path.relative(process.cwd(), out) + "]");
  } else {
    console.error("[wrote " + p + ", " + text.length + " characters; --mount DIR saves it]");
  }
}
if (!r.ok) {
  console.error(r.traceback);
  process.exit(1);
}
