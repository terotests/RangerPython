// SPDX-License-Identifier: MIT
// The playground page: an editor with several files, a Run button that
// sends them to worker.js, and the output and written files.
import { examples } from "./examples.js";

const $ = (id) => document.getElementById(id);
const editor = $("editor"), output = $("output"), filesEl = $("files"), tabsEl = $("tabs");
const runBtn = $("run"), stopBtn = $("stop"), status = $("status"), timing = $("timing");

let files = {};      // name -> text; "main.py" is the script
let current = "main.py";
let worker = null, running = false, loadStart = 0;

// ---- state: the URL hash (a shared link) first, then the last session, then the first example

function encode(obj) {
  const bytes = new TextEncoder().encode(JSON.stringify(obj));
  let s = "";
  for (const b of bytes) s += String.fromCharCode(b);
  return btoa(s).replaceAll("+", "-").replaceAll("/", "_").replace(/=+$/, "");
}
function decode(text) {
  const s = atob(text.replaceAll("-", "+").replaceAll("_", "/"));
  return JSON.parse(new TextDecoder().decode(Uint8Array.from(s, (c) => c.charCodeAt(0))));
}
function save() {
  try { localStorage.setItem("rpy-playground", JSON.stringify(files)); } catch {}
}
function load() {
  const m = location.hash.match(/^#code=(.+)$/);
  if (m) {
    try { return decode(m[1]); } catch {}
  }
  try {
    const saved = JSON.parse(localStorage.getItem("rpy-playground"));
    if (saved && typeof saved["main.py"] === "string") return saved;
  } catch {}
  return { ...Object.values(examples)[0] };
}

// ---- tabs and editor

function renderTabs() {
  tabsEl.textContent = "";
  for (const name of Object.keys(files)) {
    const t = document.createElement("button");
    t.className = "tab" + (name === current ? " active" : "");
    t.textContent = name;
    t.onclick = () => select(name);
    if (name !== "main.py") {
      const x = document.createElement("span");
      x.className = "x";
      x.textContent = "×";
      x.title = "Delete " + name;
      x.onclick = (e) => {
        e.stopPropagation();
        if (!confirm(`Delete ${name}?`)) return;
        delete files[name];
        if (current === name) current = "main.py";
        select(current);
      };
      t.append(x);
    }
    tabsEl.append(t);
  }
  const add = document.createElement("button");
  add.className = "add";
  add.textContent = "+ file";
  add.title = "Add a file the script can open or import";
  add.onclick = () => {
    const name = (prompt("File name (e.g. data/input.csv or helper.py)") || "").trim().replace(/^\/+/, "");
    if (!name) return;
    if (!(name in files)) files[name] = "";
    select(name);
  };
  tabsEl.append(add);
}

function select(name) {
  current = name;
  editor.value = files[name];
  renderTabs();
  save();
}

editor.addEventListener("input", () => {
  files[current] = editor.value;
  save();
});

// insertText keeps the browser's undo history; setRangeText where it is missing
function insert(text, from, to) {
  editor.setSelectionRange(from, to);
  if (!document.execCommand("insertText", false, text)) {
    editor.setRangeText(text, from, to, "end");
    editor.dispatchEvent(new Event("input"));
  }
}

// Tab indents, Shift+Tab dedents, Enter keeps the indent (and adds one after ":")
editor.addEventListener("keydown", (e) => {
  if ((e.ctrlKey || e.metaKey) && e.key === "Enter") {
    e.preventDefault();
    run();
    return;
  }
  const { selectionStart: s, selectionEnd: end, value: v } = editor;
  if (e.key === "Tab") {
    e.preventDefault();
    const lineStart = v.lastIndexOf("\n", s - 1) + 1;
    if (s === end && !e.shiftKey) {
      insert("    ", s, end);
    } else {
      const block = v.slice(lineStart, end);
      const changed = e.shiftKey ? block.replace(/^ {1,4}/gm, "") : block.replace(/^/gm, "    ");
      insert(changed, lineStart, end);
      editor.setSelectionRange(lineStart, lineStart + changed.length);
    }
  } else if (e.key === "Enter" && s === end && !e.shiftKey) {
    const line = v.slice(v.lastIndexOf("\n", s - 1) + 1, s);
    let indent = line.match(/^ */)[0];
    if (/:\s*$/.test(line)) indent += "    ";
    e.preventDefault();
    insert("\n" + indent, s, end);
  }
});

// ---- examples

for (const name of Object.keys(examples)) $("examples").append(new Option(name, name));
$("examples").onchange = (e) => {
  const ex = examples[e.target.value];
  e.target.value = "";
  if (!ex) return;
  files = { ...ex };
  history.replaceState(null, "", location.pathname);
  select("main.py");
};

// ---- running

function print(text, cls) {
  const atEnd = output.scrollTop + output.clientHeight >= output.scrollHeight - 4;
  if (cls) {
    const span = document.createElement("span");
    span.className = cls;
    span.textContent = text;
    output.append(span);
  } else {
    output.append(text);
  }
  if (atEnd) output.scrollTop = output.scrollHeight;
}

function startWorker() {
  worker = new Worker("worker.js", { type: "module" });
  loadStart = performance.now();
  worker.onmessage = ({ data: m }) => {
    if (m.type === "ready") {
      status.textContent = `rpy.wasm ready (${Math.round(performance.now() - loadStart)} ms)`;
      runBtn.disabled = running;
    } else if (m.type === "fatal") {
      status.innerHTML = "";
      status.append(Object.assign(document.createElement("span"), { className: "bad", textContent: "Could not load rpy.wasm: " + m.message }));
    } else if (m.type === "out") {
      for (const [fd, text] of m.chunks) print(text, fd === 2 ? "err" : "");
    } else if (m.type === "done") {
      if (m.crash) print("\n" + m.crash + "\n", "err");
      finish(m.exitCode, m.ms);
      showFiles(m.files);
    }
  };
}

function run() {
  if (running || runBtn.disabled) return;
  running = true;
  runBtn.disabled = true;
  stopBtn.disabled = false;
  output.textContent = "";
  filesEl.textContent = "";
  timing.textContent = "running…";
  const { "main.py": code, ...rest } = files;
  worker.postMessage({ code, files: rest });
}

function finish(code, ms) {
  running = false;
  runBtn.disabled = false;
  stopBtn.disabled = true;
  timing.innerHTML = "";
  const span = document.createElement("span");
  span.className = code === 0 ? "ok" : "bad";
  span.textContent = (code === 0 ? "finished" : `exited with ${code}`) + (ms != null ? ` in ${Math.round(ms)} ms` : "");
  timing.append(span);
}

$("stop").onclick = () => {
  worker.terminate();
  print("\n[stopped]\n", "note");
  finish(130);
  runBtn.disabled = true;
  status.textContent = "Restarting rpy.wasm…";
  startWorker();
};
runBtn.onclick = run;
$("clear").onclick = () => { output.textContent = ""; filesEl.textContent = ""; };

function showFiles(written) {
  for (const [path, data] of Object.entries(written).sort()) {
    const d = document.createElement("details");
    d.className = "file";
    const isSvg = path.endsWith(".svg");
    const mime = isSvg ? "image/svg+xml" : "text/plain";
    const url = URL.createObjectURL(new Blob([data], { type: mime }));
    const sum = document.createElement("summary");
    sum.append(path, Object.assign(document.createElement("span"), { className: "size", textContent: `${data.length} bytes` }));
    const dl = Object.assign(document.createElement("a"), { href: url, download: path.split("/").pop(), textContent: "download" });
    sum.append(dl);
    const body = document.createElement("div");
    body.className = "body";
    if (isSvg) {
      const box = Object.assign(document.createElement("div"), { className: "svg" });
      box.append(Object.assign(document.createElement("img"), { src: url, alt: path }));
      body.append(box);
      d.open = true;
    } else {
      body.append(Object.assign(document.createElement("pre"), { textContent: new TextDecoder().decode(data) }));
    }
    d.append(sum, body);
    filesEl.append(d);
  }
}

$("share").onclick = async () => {
  const url = location.origin + location.pathname + "#code=" + encode(files);
  history.replaceState(null, "", url);
  try {
    await navigator.clipboard.writeText(url);
    status.textContent = "Link copied";
  } catch {
    status.textContent = "Link is in the address bar";
  }
};

files = load();
select("main.py");
print("Press Run (Ctrl+Enter). Files you add are next to main.py; files the script writes appear below.\n", "note");
startWorker();
