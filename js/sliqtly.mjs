// SPDX-License-Identifier: MIT
//
// Runs a Python script against a Sliqtly presentation's files.
//
//   import { openVfs } from "./vfs.js";
//   import { runInDeck } from "ranger-python/js/sliqtly.mjs";
//   const vfs = await openVfs();
//   const r = await runInDeck(vfs, doc.id, code);
//   r.saved   [{ from: "/tmp/chart3d.svg", path: "media/chart3d.svg" }]
//   r.removed ["data/old.csv"]  (deck files the script deleted)
//
// The deck's text files are mounted at the root: media/logo.svg is
// /media/logo.svg, data/sales.csv is /data/sales.csv, and a relative path
// reads from there too. After the run every file the script wrote is put
// back in the deck: a path in one of the deck's folders (media/, data/,
// charts/, apps/) as it is, anything else (/tmp/chart3d.svg, out.csv) by
// its name where Sliqtly puts a file of that kind (`place`, default
// placeFor below: pictures under media/, data under data/).
//
// The Python runtime is synchronous; `run` lets a host run it elsewhere
// (a Worker, see js/worker.mjs) and takes { code, files, options } and
// gives runPython's result.
import { runPython } from "./index.mjs";

const DECK_FOLDERS = ["media/", "data/", "charts/", "apps/"];

export function placeFor(name) {
  const clean = name.replace(/[\\/:*?"<>|]+/g, "-");
  const p = clean.toLowerCase();
  if (/\.(png|jpe?g|gif|webp|svg|xml)$/.test(p)) return "media/" + clean;
  if (/\.(vl|vg)\.json$/.test(p)) return "charts/" + clean;
  if (/\.(tsx|jsx)(\.css)?$/.test(p)) return "apps/" + clean;
  return "data/" + clean;
}

export function deckPathFor(vfsPath, place = placeFor) {
  const bare = vfsPath.replace(/^\/+/, "");
  if (DECK_FOLDERS.some((f) => bare.startsWith(f))) return bare;
  return place(bare.split("/").pop());
}

function textOf(rec) {
  if (typeof rec.data === "string") return rec.data;
  return null;
}

export async function runInDeck(vfs, docId, code, { place = placeFor, run, save = true, ...options } = {}) {
  const files = {};
  for (const rec of await vfs.listFiles(docId)) {
    let text = textOf(rec);
    if (text === null && rec.data && typeof rec.data.text === "function" && /\.(svg|xml|csv|tsv|json|txt|md|css)$/i.test(rec.path)) {
      text = await rec.data.text();
    }
    if (text !== null) files["/" + rec.path] = text;
  }
  const result = run ? await run({ code, files, options }) : runPython(code, { ...options, files });
  const saved = [];
  for (const [from, text] of Object.entries(result.files)) {
    const path = deckPathFor(from, place);
    if (save) {
      await vfs.putFile({
        doc: docId,
        path,
        type: /\.svg$/i.test(path) ? "image/svg+xml" : "",
        size: new TextEncoder().encode(text).length,
        data: text,
        updated: Date.now(),
      });
    }
    saved.push({ from, path });
  }
  // a deck file the script deleted (os.remove) leaves the deck too
  const removed = [];
  for (const from of result.removed || []) {
    const path = from.replace(/^\/+/, "");
    if (save) await vfs.deleteFile(docId, path);
    removed.push(path);
  }
  return { ...result, saved, removed };
}
