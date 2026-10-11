// SPDX-License-Identifier: MIT
// A minimal WASI preview1 host for rpy.wasm in a browser (or any JS engine).
// The filesystem is a Map of absolute paths to bytes, preopened at "/".
// Only the calls rpy imports are implemented.

const OK = 0, EBADF = 8, EEXIST = 20, EINVAL = 28, EISDIR = 31, ENOENT = 44, ENOTDIR = 54;
const T_CHAR = 2, T_DIR = 3, T_FILE = 4;
const O_CREAT = 1, O_DIRECTORY = 2, O_EXCL = 4, O_TRUNC = 8, FD_APPEND = 1;

export class Exit extends Error {
  constructor(code) { super("exit " + code); this.code = code; }
}

function norm(path) {
  const out = [];
  for (const p of path.split("/")) {
    if (p === "" || p === ".") continue;
    if (p === "..") out.pop(); else out.push(p);
  }
  return "/" + out.join("/");
}

export class WasiHost {
  // files: { "/path": Uint8Array | string }; out(fd, text) receives stdout (1) and stderr (2)
  constructor({ args = [], env = {}, files = {}, out = () => {} } = {}) {
    this.args = args;
    this.env = Object.entries(env).map(([k, v]) => `${k}=${v}`);
    this.files = new Map();
    this.dirs = new Set(["/"]);
    for (const [p, data] of Object.entries(files)) this.writeFile(p, data);
    this.out = out;
    this.decoders = { 1: new TextDecoder(), 2: new TextDecoder() };
    this.fds = new Map([[3, { dir: true, path: "/" }]]);
    this.nextFd = 4;
    this.memory = null;
  }

  writeFile(path, data) {
    path = norm(path);
    this.files.set(path, typeof data === "string" ? new TextEncoder().encode(data) : data);
    for (let d = path; d !== "/"; ) {
      d = d.slice(0, d.lastIndexOf("/")) || "/";
      this.dirs.add(d);
    }
  }

  // the files under dir, relative to it
  listFiles(dir) {
    const pre = norm(dir) + "/";
    const r = {};
    for (const [p, data] of this.files) if (p.startsWith(pre)) r[p.slice(pre.length)] = data;
    return r;
  }

  start(instance) {
    this.memory = instance.exports.memory;
    try {
      instance.exports._start();
      return 0;
    } catch (e) {
      if (e instanceof Exit) return e.code;
      throw e;
    } finally {
      for (const fd of [1, 2]) {
        const rest = this.decoders[fd].decode();
        if (rest) this.out(fd, rest);
      }
    }
  }

  get view() { return new DataView(this.memory.buffer); }
  get bytes() { return new Uint8Array(this.memory.buffer); }
  str(ptr, len) { return new TextDecoder().decode(this.bytes.subarray(ptr, ptr + len)); }

  resolve(fd, ptr, len) {
    const d = this.fds.get(fd);
    if (!d || !d.dir) return null;
    return norm(d.path + "/" + this.str(ptr, len));
  }

  // the entries directly in dir: [name, type]
  entries(dir) {
    const pre = dir === "/" ? "/" : dir + "/";
    const seen = new Map();
    const add = (p, t) => {
      if (!p.startsWith(pre) || p === dir) return;
      const name = p.slice(pre.length).split("/")[0];
      if (!seen.has(name)) seen.set(name, p.slice(pre.length).includes("/") ? T_DIR : t);
    };
    for (const p of this.dirs) add(p, T_DIR);
    for (const p of this.files.keys()) add(p, T_FILE);
    return [...seen];
  }

  writeStat(ptr, type, size) {
    const v = this.view;
    for (let i = 0; i < 64; i += 8) v.setBigUint64(ptr + i, 0n, true);
    v.setUint8(ptr + 16, type);
    v.setBigUint64(ptr + 24, 1n, true);
    v.setBigUint64(ptr + 32, BigInt(size), true);
  }

  imports() {
    const self = this;
    const strings = (list, ptrs, buf) => {
      const enc = new TextEncoder();
      for (const s of list) {
        self.view.setUint32(ptrs, buf, true);
        ptrs += 4;
        const b = enc.encode(s + "\0");
        self.bytes.set(b, buf);
        buf += b.length;
      }
      return OK;
    };
    const sizes = (list, count, size) => {
      self.view.setUint32(count, list.length, true);
      self.view.setUint32(size, list.reduce((n, s) => n + new TextEncoder().encode(s).length + 1, 0), true);
      return OK;
    };
    const iovs = (ptr, n) => {
      const r = [];
      for (let i = 0; i < n; i++) r.push([self.view.getUint32(ptr + 8 * i, true), self.view.getUint32(ptr + 8 * i + 4, true)]);
      return r;
    };
    return {
      wasi_snapshot_preview1: {
        args_sizes_get: (c, s) => sizes(self.args, c, s),
        args_get: (p, b) => strings(self.args, p, b),
        environ_sizes_get: (c, s) => sizes(self.env, c, s),
        environ_get: (p, b) => strings(self.env, p, b),
        random_get(buf, len) {
          for (let i = 0; i < len; i += 65536) crypto.getRandomValues(self.bytes.subarray(buf + i, buf + Math.min(len, i + 65536)));
          return OK;
        },
        clock_time_get(id, _precision, out) {
          const ms = id === 0 ? Date.now() : performance.now();
          self.view.setBigUint64(out, BigInt(Math.round(ms * 1e6)), true);
          return OK;
        },
        // only clock subscriptions (time.sleep): wait them out
        poll_oneoff(inp, out, n, nevents) {
          let wait = 0;
          for (let i = 0; i < n; i++) {
            const s = inp + 48 * i;
            if (self.view.getUint8(s + 8) !== 0) continue;
            const ns = Number(self.view.getBigUint64(s + 24, true));
            const abs = self.view.getUint16(s + 40, true) & 1;
            wait = Math.max(wait, abs ? ns / 1e6 - Date.now() : ns / 1e6);
          }
          const end = performance.now() + wait;
          while (performance.now() < end);
          for (let i = 0; i < n; i++) {
            const e = out + 32 * i;
            self.view.setBigUint64(e, self.view.getBigUint64(inp + 48 * i, true), true);
            self.view.setUint16(e + 8, 0, true);
            self.view.setUint8(e + 10, self.view.getUint8(inp + 48 * i + 8));
          }
          self.view.setUint32(nevents, n, true);
          return OK;
        },
        proc_exit(code) { throw new Exit(code); },
        fd_close(fd) { return self.fds.delete(fd) ? OK : EBADF; },
        fd_fdstat_get(fd, out) {
          const d = self.fds.get(fd);
          if (fd > 2 && !d) return EBADF;
          const v = self.view;
          v.setUint8(out, fd <= 2 ? T_CHAR : d.dir ? T_DIR : T_FILE);
          v.setUint16(out + 2, d && d.append ? FD_APPEND : 0, true);
          v.setBigUint64(out + 8, 0xffffffffn, true);
          v.setBigUint64(out + 16, 0xffffffffn, true);
          return OK;
        },
        fd_filestat_get(fd, out) {
          const d = self.fds.get(fd);
          if (fd <= 2) self.writeStat(out, T_CHAR, 0);
          else if (!d) return EBADF;
          else self.writeStat(out, d.dir ? T_DIR : T_FILE, d.dir ? 0 : self.files.get(d.path).length);
          return OK;
        },
        fd_prestat_get(fd, out) {
          if (fd !== 3) return EBADF;
          self.view.setUint8(out, 0);
          self.view.setUint32(out + 4, 1, true);
          return OK;
        },
        fd_prestat_dir_name(fd, ptr, len) {
          if (fd !== 3) return EBADF;
          self.bytes.set(new TextEncoder().encode("/").subarray(0, len), ptr);
          return OK;
        },
        fd_read(fd, iov, n, nread) {
          let total = 0;
          const d = self.fds.get(fd);
          if (fd !== 0) {
            if (!d || d.dir) return EBADF;
            const data = self.files.get(d.path);
            for (const [buf, len] of iovs(iov, n)) {
              const chunk = data.subarray(d.pos, d.pos + len);
              self.bytes.set(chunk, buf);
              d.pos += chunk.length;
              total += chunk.length;
              if (chunk.length < len) break;
            }
          }
          self.view.setUint32(nread, total, true);
          return OK;
        },
        fd_write(fd, iov, n, nwritten) {
          let total = 0;
          for (const [buf, len] of iovs(iov, n)) {
            const chunk = self.bytes.slice(buf, buf + len);
            total += len;
            if (fd === 1 || fd === 2) {
              const text = self.decoders[fd].decode(chunk, { stream: true });
              if (text) self.out(fd, text);
              continue;
            }
            const d = self.fds.get(fd);
            if (!d || d.dir) return EBADF;
            const old = self.files.get(d.path);
            const pos = d.append ? old.length : d.pos;
            const next = new Uint8Array(Math.max(old.length, pos + len));
            next.set(old);
            next.set(chunk, pos);
            self.files.set(d.path, next);
            d.pos = pos + len;
          }
          self.view.setUint32(nwritten, total, true);
          return OK;
        },
        fd_readdir(fd, buf, len, cookie, used) {
          const d = self.fds.get(fd);
          if (!d || !d.dir) return EBADF;
          const enc = new TextEncoder();
          const list = [[".", T_DIR], ["..", T_DIR], ...self.entries(d.path)];
          let off = 0;
          for (let i = Number(cookie); i < list.length && off < len; i++) {
            const name = enc.encode(list[i][0]);
            const ent = new Uint8Array(24 + name.length);
            const v = new DataView(ent.buffer);
            v.setBigUint64(0, BigInt(i + 1), true);
            v.setBigUint64(8, BigInt(i + 1), true);
            v.setUint32(16, name.length, true);
            v.setUint8(20, list[i][1]);
            ent.set(name, 24);
            const part = ent.subarray(0, Math.min(ent.length, len - off));
            self.bytes.set(part, buf + off);
            off += part.length;
          }
          self.view.setUint32(used, off, true);
          return OK;
        },
        path_filestat_get(fd, _flags, ptr, plen, out) {
          const p = self.resolve(fd, ptr, plen);
          if (p === null) return EBADF;
          if (self.files.has(p)) self.writeStat(out, T_FILE, self.files.get(p).length);
          else if (self.dirs.has(p)) self.writeStat(out, T_DIR, 0);
          else return ENOENT;
          return OK;
        },
        path_open(fd, _dirflags, ptr, plen, oflags, _rb, _ri, fdflags, out) {
          const p = self.resolve(fd, ptr, plen);
          if (p === null) return EBADF;
          const isDir = self.dirs.has(p), isFile = self.files.has(p);
          if (oflags & O_DIRECTORY && !isDir) return isFile ? ENOTDIR : ENOENT;
          if (isDir) {
            if (oflags & O_CREAT) return EISDIR;
          } else if (!isFile) {
            if (!(oflags & O_CREAT)) return ENOENT;
            const parent = p.slice(0, p.lastIndexOf("/")) || "/";
            if (!self.dirs.has(parent)) return ENOENT;
            self.files.set(p, new Uint8Array(0));
          } else if (oflags & O_CREAT && oflags & O_EXCL) {
            return EEXIST;
          }
          if (!isDir && oflags & O_TRUNC) self.files.set(p, new Uint8Array(0));
          const n = self.nextFd++;
          self.fds.set(n, { dir: isDir, path: p, pos: 0, append: !!(fdflags & FD_APPEND) });
          self.view.setUint32(out, n, true);
          return OK;
        },
        path_readlink: () => EINVAL,
      },
    };
  }
}
