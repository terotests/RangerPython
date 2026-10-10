// SPDX-License-Identifier: MIT
//
// runInDeck against a store with Sliqtly's vfs API (listFiles, putFile,
// deleteFile): the example script's SVG lands in the deck as media/…, a
// deck file is readable at /data/…, and a deleted deck file leaves.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { runInDeck } from "../js/sliqtly.mjs";

const HERE = path.dirname(fileURLToPath(import.meta.url));

function store() {
  const files = new Map();
  const key = (doc, p) => doc + "\u0000" + p;
  return {
    files,
    listFiles: async (doc) => [...files.values()].filter((f) => f.doc === doc),
    getFile: async (doc, p) => files.get(key(doc, p)),
    putFile: async (f) => { files.set(key(f.doc, f.path), f); },
    deleteFile: async (doc, p) => { files.delete(key(doc, p)); },
  };
}

const vfs = store();
await vfs.putFile({ doc: "d1", path: "data/sales.csv", data: "q,v\nQ1,12\nQ2,15\n" });
await vfs.putFile({ doc: "d1", path: "data/old.txt", data: "x" });
await vfs.putFile({ doc: "d2", path: "data/other.csv", data: "not this deck" });

const chart = fs.readFileSync(path.join(HERE, "scripts/chart3d.py"), "utf8");
const r1 = await runInDeck(vfs, "d1", chart);
assert.equal(r1.ok, true, r1.traceback);
assert.deepEqual(r1.saved, [{ from: "/tmp/chart3d.svg", path: "media/chart3d.svg" }]);
const svg = vfs.files.get("d1\u0000media/chart3d.svg");
assert.equal(svg.data, fs.readFileSync(path.join(HERE, "expected/chart3d.files/tmp/chart3d.svg"), "utf8"));
assert.equal(svg.type, "image/svg+xml");

const r2 = await runInDeck(vfs, "d1", `import os
rows = open("data/sales.csv").read().split()[1:]
total = sum(int(r.split(",")[1]) for r in rows)
print(total, os.listdir("/data"))
with open("/data/total.txt", "w") as f:
    f.write(str(total))
os.remove("/data/old.txt")
`);
assert.equal(r2.ok, true, r2.traceback);
assert.equal(r2.stdout, "27 ['old.txt', 'sales.csv']\n");
assert.deepEqual(r2.saved, [{ from: "/data/total.txt", path: "data/total.txt" }]);
assert.deepEqual(r2.removed, ["data/old.txt"]);
assert.equal(vfs.files.has("d1\u0000data/old.txt"), false);
assert.equal(vfs.files.get("d1\u0000data/total.txt").data, "27");

const r3 = await runInDeck(vfs, "d1", "1/0");
assert.equal(r3.ok, false);
assert.equal(r3.errorType, "ZeroDivisionError");
assert.match(r3.traceback, /ZeroDivisionError: division by zero$/);

console.log("ok   sliqtly adapter");
