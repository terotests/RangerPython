export class PyToken  {
  constructor() {
    this.kind = 9;
    this.text = "";
    this.line = 0;
    this.bytes = false;
  }
}
export class PyNode  {
  constructor(k, ln) {
    this.kind = 49;
    this.line = 0;
    this.s = "";
    this.i = 0;
    this.d = 0.0;
    this.k1 = undefined;
    this.k2 = undefined;
    this.k3 = undefined;
    this.kids = [];
    this.strs = [];
    this.names = [];
    this.body = [];
    this.orelse = [];
    this.final = [];
    this.handlers = [];
    this.decos = [];
    this.tparams = [];
    this.qual = "";
    this.locals = {};     /* note: unused */
    this.localsKnown = false;     /* note: unused */
    this.kind = k;
    this.line = ln;
  }
}
export class PyVal  {
  constructor(tag) {
    this.t = 0;
    this.i = 0;
    this.i2 = 0;
    this.i3 = 0;
    this.d = 0.0;
    this.d2 = 0.0;
    this.s = "";
    this._z_items = undefined;
    this._z_vals = undefined;
    this._z_index = undefined;
    this._z_attrs = undefined;
    this._z_akeys = undefined;
    this._z_defs = undefined;
    this._z_bases = undefined;
    this.cls = undefined;
    this.__self = undefined;
    this.wrapped = undefined;
    this.setter = undefined;
    this.node = undefined;
    this.scope = undefined;
    this.id = 0;
    this.pos = 0;
    this.mode = "";
    this.closed = false;
    this.isExc = false;
    this._z_cps = undefined;
    this.cpsKnown = false;
    this.gen = undefined;
    this.cur = undefined;
    this.bt = "";
    this.slotsKnown = 0;
    this._z_slotNames = undefined;
    this.dictv = undefined;
    this.owner = undefined;
    this.t = tag;
  }
  get items() { const v = this._z_items; return v === undefined ? (this._z_items = []) : v; }
  set items(v) { this._z_items = v; }
  get vals() { const v = this._z_vals; return v === undefined ? (this._z_vals = []) : v; }
  set vals(v) { this._z_vals = v; }
  get index() { const v = this._z_index; return v === undefined ? (this._z_index = {}) : v; }
  set index(v) { this._z_index = v; }
  get attrs() { const v = this._z_attrs; return v === undefined ? (this._z_attrs = {}) : v; }
  set attrs(v) { this._z_attrs = v; }
  get akeys() { const v = this._z_akeys; return v === undefined ? (this._z_akeys = []) : v; }
  set akeys(v) { this._z_akeys = v; }
  get defs() { const v = this._z_defs; return v === undefined ? (this._z_defs = {}) : v; }
  set defs(v) { this._z_defs = v; }
  get bases() { const v = this._z_bases; return v === undefined ? (this._z_bases = []) : v; }
  set bases(v) { this._z_bases = v; }
  get cps() { const v = this._z_cps; return v === undefined ? (this._z_cps = []) : v; }
  set cps(v) { this._z_cps = v; }
  get slotNames() { const v = this._z_slotNames; return v === undefined ? (this._z_slotNames = {}) : v; }
  set slotNames(v) { this._z_slotNames = v; }
  putAttr (k, v) {
    if ( ( typeof(this.attrs[k] ) != "undefined" && Object.prototype.hasOwnProperty.call(this.attrs, k) ) == false ) {
      this.akeys.push(k);
    }
    this.attrs[k] = v;
    if ( (typeof(this.dictv) !== "undefined" && this.dictv != null )  ) {
      this.mirrorPut(k, v);
    }
  };
  delAttrKey (k) {
    delete this.attrs[k];
    let kept = [];
    // Loop start
    for ( const a of this.akeys) {
      if ( a != k ) {
        kept.push(a);
      }
    }
    this.akeys = kept;
    if ( (typeof(this.dictv) !== "undefined" && this.dictv != null )  ) {
      this.mirrorDel(k);
    }
  };
  mirrorPut (k, v) {
    const d_1 = this.dictv;
    const hk = "s" + k;
    const at = ( Object.prototype.hasOwnProperty.call(d_1.index, hk) ? d_1.index[hk] : undefined );
    if ( typeof(at) === "undefined" ) {
      d_1.index[hk] = d_1.items.length;
      const ks = new PyVal(4);
      ks.s = k;
      d_1.items.push(ks);
      d_1.vals.push(v);
      return;
    }
    d_1.vals[at] = v;
  };
  mirrorDel (k) {
    const d_1 = this.dictv;
    const hk = "s" + k;
    const at = ( Object.prototype.hasOwnProperty.call(d_1.index, hk) ? d_1.index[hk] : undefined );
    if ( typeof(at) === "undefined" ) {
      return;
    }
    let p = at;
    d_1.items.splice(p, 1).pop();
    d_1.vals.splice(p, 1).pop();
    delete d_1.index[hk];
    const n = d_1.items.length;
    while (p < n) {
      const it = d_1.items[p];
      if ( it.t == 4 ) {
        d_1.index["s" + it.s] = p;
      }
      p = p + 1;
    };
  };
  ownerSync () {
    const o = this.owner;
    let na = {};
    let nk = [];
    const n = this.items.length;
    let i_1 = 0;
    while (i_1 < n) {
      const it = this.items[i_1];
      if ( it.t == 4 ) {
        if ( ( typeof(na[it.s] ) != "undefined" && Object.prototype.hasOwnProperty.call(na, it.s) ) == false ) {
          nk.push(it.s);
        }
        na[it.s] = this.vals[i_1];
      }
      i_1 = i_1 + 1;
    };
    o.attrs = na;
    o.akeys = nk;
  };
  attrKeys () {
    let out = [];
    let seen = {};
    // Loop start
    for ( const a of this.akeys) {
      if ( ( typeof(this.attrs[a] ) != "undefined" && Object.prototype.hasOwnProperty.call(this.attrs, a) ) && ( typeof(seen[a] ) != "undefined" && Object.prototype.hasOwnProperty.call(seen, a) ) == false ) {
        out.push(a);
        seen[a] = true;
      }
    }
    if ( out.length != Object.keys(this.attrs).length ) {
      const rest = Object.keys(this.attrs);
      // Loop start
      for ( const r of rest) {
        if ( ( typeof(seen[r] ) != "undefined" && Object.prototype.hasOwnProperty.call(seen, r) ) == false ) {
          out.push(r);
          this.akeys.push(r);
        }
      }
    }
    return out;
  };
  codes () {
    if ( this.cpsKnown == false ) {
      this.cps = Array.from(this.s, (rg_c) => rg_c.codePointAt(0));
      this.cpsKnown = true;
    }
    return this.cps;
  };
  strLen () {
    const c = this.codes();
    return c.length;
  };
}
PyVal.one = function(v) {
  let r = [];
  r.push(v);
  return r;
};
export class PyScope  {
  constructor() {
    this.vars = {};
    this.parent = undefined;
    this.globals = undefined;
    this.decl = {};
    this.isClass = false;
    this.order = [];
  }
  put (name, v) {
    if ( this.isClass && ( typeof(this.vars[name] ) != "undefined" && Object.prototype.hasOwnProperty.call(this.vars, name) ) == false ) {
      this.order.push(name);
    }
    this.vars[name] = v;
  };
  orderedKeys () {
    if ( this.isClass == false ) {
      return Object.keys(this.vars);
    }
    let out = [];
    let seen = {};
    // Loop start
    for ( const k of this.order) {
      if ( ( typeof(this.vars[k] ) != "undefined" && Object.prototype.hasOwnProperty.call(this.vars, k) ) && ( typeof(seen[k] ) != "undefined" && Object.prototype.hasOwnProperty.call(seen, k) ) == false ) {
        seen[k] = true;
        out.push(k);
      }
    }
    const vk = Object.keys(this.vars);
    // Loop start
    for ( const k2 of vk) {
      if ( ( typeof(seen[k2] ) != "undefined" && Object.prototype.hasOwnProperty.call(seen, k2) ) == false ) {
        out.push(k2);
      }
    }
    return out;
  };
}
export class PyGenRes  {
  constructor() {
    this.kind = 0;
    this.v = new PyVal(0);
    this.send = new PyVal(0);
    this.sendKind = 0;
  }
}
export class PyGenSlot  {
  constructor() {
    this.i = 0;
    this.j = 0;
    this.sig = 0;
    this.v = new PyVal(0);
    this.w = undefined;
    this.keep = new PyVal(0);
    this.res = [];
    this.used = 0;
    this.resumed = false;
  }
}
export class PyGen  {
  constructor() {
    this.func = undefined;
    this.node = undefined;
    this.scope = undefined;
    this.selfv = undefined;
    this.name = "";
    this.live = [];
    this.depth = 0;
    this.started = false;
    this.done = false;
    this.running = false;
    this.out = new PyVal(0);
    this.line = 0;
  }
}
export class PySpec  {
  constructor() {
    this.fill = " ";
    this.align = "";
    this.sign = "-";
    this.alt = false;
    this.zero = false;
    this.width = 0;
    this.grouping = "";
    this.prec = -1;
    this.type = "";
    this.err = "";
  }
}
export class PyDigits  {
  constructor() {
    this.neg = false;
    this.digits = "";
    this.point = 0;
  }
}
export class PyFormat  {
}
PyFormat.isNegZero = function(x) {
  if ( x != 0.0 ) {
    return false;
  }
  const inv = 1.0 / x;
  return inv < 0.0;
};
PyFormat.zeros = function(n) {
  let s = "";
  let i = 0;
  while (i < n) {
    s = s + "0";
    i = i + 1;
  };
  return s;
};
PyFormat.rep = function(str, n) {
  let s = "";
  let i = 0;
  while (i < n) {
    s = s + str;
    i = i + 1;
  };
  return s;
};
PyFormat.intDigits = function(v) {
  if ( v < 1.0 ) {
    return "0";
  }
  const n = Math.trunc(v);
  return ((n).toString());
};
PyFormat.decompose = function(x) {
  const r = new PyDigits();
  if ( x < 0.0 ) {
    r.neg = true;
  }
  if ( PyFormat.isNegZero(x) ) {
    r.neg = true;
  }
  let a = x;
  if ( a < 0.0 ) {
    a = 0.0 - a;
  }
  if ( a == 0.0 ) {
    r.digits = "";
    r.point = 1;
    return r;
  }
  const txt = String(a);
  const cs = Array.from(txt, (rg_c) => rg_c.codePointAt(0));
  let mant = [];
  let before = 0;
  let seenPoint = false;
  let ex = 0;
  let exSign = 1;
  let inExp = false;
  // Loop start
  for ( const c of cs) {
    if ( inExp ) {
      if ( c == 45 ) {
        exSign = -1;
      } else {
        if ( c >= 48 && c <= 57 ) {
          ex = ex * 10 + (c - 48);
        }
      }
    } else {
      if ( c == 101 || c == 69 ) {
        inExp = true;
      } else {
        if ( c == 46 ) {
          seenPoint = true;
        } else {
          if ( c >= 48 && c <= 57 ) {
            mant.push(c);
            if ( seenPoint == false ) {
              before = before + 1;
            }
          }
        }
      }
    }
  }
  let point = before + ex * exSign;
  let start = 0;
  const n = mant.length;
  while (start < n && mant[start] == 48) {
    start = start + 1;
    point = point - 1;
  };
  let end = n;
  while (end > start && mant[(end - 1)] == 48) {
    end = end - 1;
  };
  r.digits = py_rt_codes(mant, start, end);
  r.point = point;
  return r;
};
PyFormat.floatRepr = function(x) {
  if ( Number.isNaN(x) ) {
    return "nan";
  }
  if ( (Math.abs(x) === Infinity) ) {
    if ( x < 0.0 ) {
      return "-inf";
    }
    return "inf";
  }
  const d = PyFormat.decompose(x);
  let sign = "";
  if ( d.neg ) {
    sign = "-";
  }
  if ( d.digits.length == 0 ) {
    return sign + "0.0";
  }
  const nd = d.digits.length;
  const e10 = d.point - 1;
  if ( e10 >= -4 && e10 < 16 ) {
    if ( d.point <= 0 ) {
      return ((sign + "0.") + PyFormat.zeros((0 - d.point))) + d.digits;
    }
    if ( d.point >= nd ) {
      return ((sign + d.digits) + PyFormat.zeros((d.point - nd))) + ".0";
    }
    return ((sign + d.digits.substring(0, d.point )) + ".") + d.digits.substring(d.point, nd );
  }
  let m = d.digits.substring(0, 1 );
  if ( nd > 1 ) {
    m = (m + ".") + d.digits.substring(1, nd );
  }
  return ((sign + m) + "e") + PyFormat.expText(e10);
};
PyFormat.expText = function(e) {
  let sg = "+";
  let v = e;
  if ( e < 0 ) {
    sg = "-";
    v = 0 - e;
  }
  let t = ((v).toString());
  if ( t.length < 2 ) {
    t = "0" + t;
  }
  return sg + t;
};
PyFormat.productError = function(a, b, p) {
  const sp = 134217729.0;
  const ca = sp * a;
  const ah = ca - (ca - a);
  const al = a - ah;
  const cb = sp * b;
  const bh = cb - (cb - b);
  const bl = b - bh;
  return (((ah * bh - p) + ah * bl) + al * bh) + al * bl;
};
PyFormat.pow10 = function(n) {
  return Math.pow(10.0, (n));
};
PyFormat.roundSide = function(m, side) {
  const f = Math.floor(m);
  const diff = m - f;
  if ( diff > 0.5 ) {
    return f + 1.0;
  }
  if ( diff < 0.5 ) {
    return f;
  }
  if ( side > 0.0 ) {
    return f + 1.0;
  }
  if ( side < 0.0 ) {
    return f;
  }
  const half = f / 2.0;
  if ( Math.floor(half) == half ) {
    return f;
  }
  return f + 1.0;
};
PyFormat.scaledRound = function(a, k) {
  if ( k >= 0 && k <= 22 ) {
    const q = PyFormat.pow10(k);
    const m = a * q;
    if ( m >= 4000000000000000.0 ) {
      return -1.0;
    }
    const err = PyFormat.productError(a, q, m);
    return PyFormat.roundSide(m, err);
  }
  if ( k < 0 && k >= -22 ) {
    const q2 = PyFormat.pow10((0 - k));
    const m2 = a / q2;
    if ( m2 >= 4000000000000000.0 ) {
      return -1.0;
    }
    const hi = m2 * q2;
    const lo = PyFormat.productError(m2, q2, hi);
    const rest = (a - hi) - lo;
    return PyFormat.roundSide(m2, rest);
  }
  const m3 = a * PyFormat.pow10(k);
  if ( m3 >= 4000000000000000.0 ) {
    return -1.0;
  }
  return PyFormat.roundSide(m3, 0.0);
};
PyFormat.fixedAbs = function(a, prec) {
  const r = PyFormat.scaledRound(a, prec);
  if ( r >= 0.0 ) {
    let ds = PyFormat.intDigits(r);
    if ( ds.length <= prec ) {
      ds = PyFormat.zeros(((prec + 1) - ds.length)) + ds;
    }
    if ( prec == 0 ) {
      return ds;
    }
    const n = ds.length;
    return (ds.substring(0, n - prec ) + ".") + ds.substring(n - prec, n );
  }
  const d = PyFormat.decompose(a);
  const nd = d.digits.length;
  let ip = "";
  let fp = "";
  if ( d.point <= 0 ) {
    ip = "0";
    fp = PyFormat.zeros((0 - d.point)) + d.digits;
  } else {
    if ( d.point >= nd ) {
      ip = d.digits + PyFormat.zeros((d.point - nd));
    } else {
      ip = d.digits.substring(0, d.point );
      fp = d.digits.substring(d.point, nd );
    }
  }
  if ( fp.length > prec ) {
    fp = fp.substring(0, prec );
  } else {
    fp = fp + PyFormat.zeros((prec - fp.length));
  }
  if ( prec == 0 ) {
    return ip;
  }
  return (ip + ".") + fp;
};
PyFormat.sciAbs = function(a, prec, upper) {
  let ech = "e";
  if ( upper ) {
    ech = "E";
  }
  if ( a == 0.0 ) {
    let z = "0";
    if ( prec > 0 ) {
      z = (z + ".") + PyFormat.zeros(prec);
    }
    return (z + ech) + "+00";
  }
  const d = PyFormat.decompose(a);
  let e = d.point - 1;
  const r = PyFormat.scaledRound(a, (prec - e));
  if ( r < 0.0 ) {
    const ds0 = d.digits + PyFormat.zeros((prec + 1));
    let body0 = ds0.substring(0, 1 );
    if ( prec > 0 ) {
      body0 = (body0 + ".") + ds0.substring(1, prec + 1 );
    }
    return (body0 + ech) + PyFormat.expText(e);
  }
  let ds = PyFormat.intDigits(r);
  if ( ds.length > prec + 1 ) {
    e = e + 1;
    ds = ds.substring(0, prec + 1 );
  }
  while (ds.length < prec + 1) {
    ds = ds + "0";
  };
  let body = ds.substring(0, 1 );
  if ( prec > 0 ) {
    body = (body + ".") + ds.substring(1, prec + 1 );
  }
  return (body + ech) + PyFormat.expText(e);
};
PyFormat.stripZeros = function(s) {
  const cs = Array.from(s, (rg_c) => rg_c.codePointAt(0));
  let epos = -1;
  let dot = -1;
  // Loop start
  for ( let i = 0; i < cs.length; i++) {
    var c = cs[i];
    if ( c == 101 || c == 69 ) {
      epos = i;
    }
    if ( c == 46 ) {
      dot = i;
    }
  }
  if ( dot < 0 ) {
    return s;
  }
  const n = cs.length;
  let mantEnd = n;
  if ( epos >= 0 ) {
    mantEnd = epos;
  }
  let k = mantEnd;
  while (k > dot && cs[(k - 1)] == 48) {
    k = k - 1;
  };
  if ( k == dot + 1 ) {
    k = dot;
  }
  return py_rt_codes(cs, 0, k) + py_rt_codes(cs, mantEnd, n);
};
PyFormat.generalAbs = function(a, prec0, upper, alt) {
  let prec = prec0;
  if ( prec == 0 ) {
    prec = 1;
  }
  if ( a == 0.0 ) {
    if ( alt ) {
      return "0." + PyFormat.zeros((prec - 1));
    }
    return "0";
  }
  const es = PyFormat.sciAbs(a, (prec - 1), upper);
  const ei = es.toLowerCase().indexOf("e");
  const ex = (isNaN( parseInt(PyFormat.plainInt(es.substring(ei + 1, es.length ))) ) ? undefined : parseInt(PyFormat.plainInt(es.substring(ei + 1, es.length ))));
  if ( ex >= -4 && ex < prec ) {
    const f = PyFormat.fixedAbs(a, ((prec - 1) - ex));
    if ( alt ) {
      if ( f.indexOf(".") < 0 ) {
        return f + ".";
      }
      return f;
    }
    return PyFormat.stripZeros(f);
  }
  if ( alt ) {
    return es;
  }
  return PyFormat.stripZeros(es);
};
PyFormat.plainInt = function(s) {
  if ( s.startsWith("+") ) {
    return s.substring(1, s.length );
  }
  return s;
};
PyFormat.noTypeAbs = function(a, prec) {
  const g = PyFormat.generalAbs(a, prec, false, false);
  if ( (g.indexOf(".") < 0 && g.indexOf("e") < 0) && g.indexOf("n") < 0 ) {
    return g + ".0";
  }
  return g;
};
PyFormat.intBase = function(v0, base, upper) {
  let v = v0;
  if ( v < 0 ) {
    v = 0 - v;
  }
  if ( v == 0 ) {
    return "0";
  }
  let digs = "0123456789abcdef";
  if ( upper ) {
    digs = "0123456789ABCDEF";
  }
  let out = [];
  while (v > 0) {
    const q = PyFormat.floorDivInt(v, base);
    const r = v - q * base;
    out.push(digs.charCodeAt(r ));
    v = q;
  };
  let rev = [];
  let i = out.length - 1;
  while (i >= 0) {
    rev.push(out[i]);
    i = i - 1;
  };
  return py_rt_codes(rev, 0, rev.length);
};
PyFormat.floorDivInt = function(a, b) {
  const q = Math.floor((a) / (b));
  let qi = Math.trunc(q);
  let r = a - qi * b;
  if ( b > 0 ) {
    while (r < 0) {
      qi = qi - 1;
      r = r + b;
    };
    while (r >= b) {
      qi = qi + 1;
      r = r - b;
    };
  } else {
    while (r > 0) {
      qi = qi - 1;
      r = r + b;
    };
    while (r <= b) {
      if ( r == b ) {
        qi = qi + 1;
        r = 0;
        break;
      }
      break;
    };
  }
  return qi;
};
PyFormat.floorModInt = function(a, b) {
  return a - b * PyFormat.floorDivInt(a, b);
};
PyFormat.parseSpec = function(spec) {
  const sp = new PySpec();
  const cs = Array.from(spec, (rg_c) => rg_c.codePointAt(0));
  const n = cs.length;
  let i = 0;
  if ( n >= 2 && PyFormat.isAlign(cs[1]) ) {
    sp.fill = String.fromCodePoint(cs[0]);
    sp.align = String.fromCodePoint(cs[1]);
    i = 2;
  } else {
    if ( n >= 1 && PyFormat.isAlign(cs[0]) ) {
      sp.align = String.fromCodePoint(cs[0]);
      i = 1;
    }
  }
  if ( i < n ) {
    const c = cs[i];
    if ( (c == 43 || c == 45) || c == 32 ) {
      sp.sign = String.fromCodePoint(c);
      i = i + 1;
    }
  }
  if ( i < n && cs[i] == 122 ) {
    i = i + 1;
  }
  if ( i < n && cs[i] == 35 ) {
    sp.alt = true;
    i = i + 1;
  }
  if ( i < n && cs[i] == 48 ) {
    sp.zero = true;
    i = i + 1;
  }
  let w = 0;
  while ((i < n && cs[i] >= 48) && cs[i] <= 57) {
    w = w * 10 + (cs[i] - 48);
    i = i + 1;
  };
  sp.width = w;
  if ( i < n && (cs[i] == 44 || cs[i] == 95) ) {
    sp.grouping = String.fromCodePoint(cs[i]);
    i = i + 1;
  }
  if ( i < n && cs[i] == 46 ) {
    i = i + 1;
    let pr = 0;
    let any = false;
    while ((i < n && cs[i] >= 48) && cs[i] <= 57) {
      pr = pr * 10 + (cs[i] - 48);
      i = i + 1;
      any = true;
    };
    if ( any == false ) {
      sp.err = "Format specifier missing precision";
      return sp;
    }
    sp.prec = pr;
  }
  if ( i < n ) {
    sp.type = String.fromCodePoint(cs[i]);
    i = i + 1;
  }
  if ( i < n ) {
    sp.err = ("Invalid format specifier '" + spec) + "'";
  }
  return sp;
};
PyFormat.isAlign = function(c) {
  return ((c == 60 || c == 62) || c == 94) || c == 61;
};
PyFormat.group = function(ds, sep, every) {
  if ( sep.length == 0 ) {
    return ds;
  }
  const cs = Array.from(ds, (rg_c) => rg_c.codePointAt(0));
  const n = cs.length;
  let out = "";
  let i = 0;
  while (i < n) {
    if ( i > 0 && PyFormat.floorModInt(n - i, every) == 0 ) {
      out = out + sep;
    }
    out = out + String.fromCodePoint(cs[i]);
    i = i + 1;
  };
  return out;
};
PyFormat.pad = function(sign, body, sp, numeric) {
  let align = sp.align;
  let fill = sp.fill;
  if ( align.length == 0 ) {
    if ( numeric ) {
      align = ">";
      if ( sp.zero ) {
        align = "=";
        fill = "0";
      }
    } else {
      align = "<";
    }
  }
  const __len = r_char_length(sign) + r_char_length(body);
  if ( __len >= sp.width ) {
    return sign + body;
  }
  const gap = sp.width - __len;
  if ( align == "<" ) {
    return (sign + body) + PyFormat.rep(fill, gap);
  }
  if ( align == ">" ) {
    return (PyFormat.rep(fill, gap) + sign) + body;
  }
  if ( align == "=" ) {
    return (sign + PyFormat.rep(fill, gap)) + body;
  }
  const left = ((gap / 2) | 0);
  return ((PyFormat.rep(fill, left) + sign) + body) + PyFormat.rep(fill, (gap - left));
};
PyFormat.signText = function(neg, sp) {
  if ( neg ) {
    return "-";
  }
  if ( sp.sign == "+" ) {
    return "+";
  }
  if ( sp.sign == " " ) {
    return " ";
  }
  return "";
};
PyFormat.formatInt = function(v, sp) {
  const t = sp.type;
  const neg = v < 0;
  let a = v;
  if ( neg ) {
    a = 0 - v;
  }
  let body = "";
  let prefix = "";
  if ( (t == "" || t == "d") || t == "n" ) {
    body = PyFormat.group(((a).toString()), sp.grouping, 3);
  } else {
    if ( t == "x" || t == "X" ) {
      body = PyFormat.group(
        PyFormat.intBase(a, 16, (t == "X")),
        sp.grouping,
        4
      );
      if ( sp.alt ) {
        prefix = "0" + t;
      }
    } else {
      if ( t == "o" ) {
        body = PyFormat.group(PyFormat.intBase(a, 8, false), sp.grouping, 4);
        if ( sp.alt ) {
          prefix = "0o";
        }
      } else {
        if ( t == "b" ) {
          body = PyFormat.group(PyFormat.intBase(a, 2, false), sp.grouping, 4);
          if ( sp.alt ) {
            prefix = "0b";
          }
        } else {
          if ( t == "c" ) {
            return PyFormat.pad("", String.fromCodePoint(v), sp, false);
          }
          return PyFormat.formatFloat((v), sp);
        }
      }
    }
  }
  if ( sp.prec >= 0 ) {
    return (String.fromCodePoint(0) + "!") + "Precision not allowed in integer format specifier";
  }
  return PyFormat.pad((PyFormat.signText(neg, sp) + prefix), body, sp, true);
};
PyFormat.formatFloat = function(x, sp) {
  const t = sp.type;
  let neg = x < 0.0 || PyFormat.isNegZero(x);
  let a = x;
  if ( a < 0.0 ) {
    a = 0.0 - a;
  }
  let body = "";
  const upper = (t == "F" || t == "E") || t == "G";
  if ( Number.isNaN(x) ) {
    neg = false;
    body = "nan";
  } else {
    if ( (Math.abs(x) === Infinity) ) {
      body = "inf";
    } else {
      let prec = sp.prec;
      if ( t == "f" || t == "F" ) {
        if ( prec < 0 ) {
          prec = 6;
        }
        body = PyFormat.fixedAbs(a, prec);
      } else {
        if ( t == "e" || t == "E" ) {
          if ( prec < 0 ) {
            prec = 6;
          }
          body = PyFormat.sciAbs(a, prec, upper);
        } else {
          if ( (t == "g" || t == "G") || t == "n" ) {
            if ( prec < 0 ) {
              prec = 6;
            }
            body = PyFormat.generalAbs(a, prec, upper, sp.alt);
          } else {
            if ( t == "%" ) {
              if ( prec < 0 ) {
                prec = 6;
              }
              body = PyFormat.fixedAbs(a * 100.0, prec) + "%";
            } else {
              if ( t == "" ) {
                if ( prec < 0 ) {
                  body = PyFormat.floatRepr(a);
                } else {
                  body = PyFormat.noTypeAbs(a, prec);
                }
              } else {
                return (((String.fromCodePoint(0) + "!") + "Unknown format code '") + t) + "' for object of type 'float'";
              }
            }
          }
        }
      }
    }
  }
  if ( upper ) {
    body = body.toUpperCase();
  }
  if ( sp.grouping.length > 0 ) {
    const dot = body.indexOf(".");
    const epos = body.indexOf("e");
    let cut = body.length;
    if ( dot >= 0 ) {
      cut = dot;
    } else {
      if ( epos >= 0 ) {
        cut = epos;
      }
    }
    body = PyFormat.group(
      body.substring(0, cut ),
      sp.grouping,
      3
    ) + body.substring(cut, body.length );
  }
  return PyFormat.pad(PyFormat.signText(neg, sp), body, sp, true);
};
PyFormat.formatStr = function(s, sp) {
  if ( sp.type != "" && sp.type != "s" ) {
    return (((String.fromCodePoint(0) + "!") + "Unknown format code '") + sp.type) + "' for object of type 'str'";
  }
  if ( sp.sign != "-" ) {
    return (String.fromCodePoint(0) + "!") + "Sign not allowed in string format specifier";
  }
  let body = s;
  if ( sp.prec >= 0 ) {
    const cs = Array.from(s, (rg_c) => rg_c.codePointAt(0));
    if ( cs.length > sp.prec ) {
      body = py_rt_codes(cs, 0, sp.prec);
    }
  }
  if ( sp.align == "=" ) {
    return (String.fromCodePoint(0) + "!") + "'=' alignment not allowed in string format specifier";
  }
  return PyFormat.pad("", body, sp, false);
};
PyFormat.strRepr = function(s) {
  const cs = Array.from(s, (rg_c) => rg_c.codePointAt(0));
  let hasSingle = false;
  let hasDouble = false;
  // Loop start
  for ( const c of cs) {
    if ( c == 39 ) {
      hasSingle = true;
    }
    if ( c == 34 ) {
      hasDouble = true;
    }
  }
  let q = 39;
  if ( hasSingle && hasDouble == false ) {
    q = 34;
  }
  let out = [];
  out.push(q);
  // Loop start
  for ( const c2 of cs) {
    if ( c2 == q || c2 == 92 ) {
      out.push(92);
      out.push(c2);
      continue;
    }
    if ( c2 == 10 ) {
      out.push(92);
      out.push(110);
      continue;
    }
    if ( c2 == 9 ) {
      out.push(92);
      out.push(116);
      continue;
    }
    if ( c2 == 13 ) {
      out.push(92);
      out.push(114);
      continue;
    }
    if ( ((c2 < 32 || c2 == 127) || c2 >= 128 && c2 < 160) || c2 == 173 ) {
      let hx = PyFormat.intBase(c2, 16, false);
      if ( hx.length < 2 ) {
        hx = "0" + hx;
      }
      const esc = Array.from("\\x" + hx, (rg_c) => rg_c.codePointAt(0));
      // Loop start
      for ( const ec of esc) {
        out.push(ec);
      }
      continue;
    }
    if ( c2 >= 55296 && c2 <= 57343 ) {
      const hx2 = PyFormat.intBase(c2, 16, false);
      const esc2 = Array.from("\\u" + hx2, (rg_c) => rg_c.codePointAt(0));
      // Loop start
      for ( const ec2 of esc2) {
        out.push(ec2);
      }
      continue;
    }
    out.push(c2);
  }
  out.push(q);
  return py_rt_codes(out, 0, out.length);
};
export class PyFileSystem  {
  constructor() {
    this.files = {};
    this.dirs = {};
    this.changed = [];
    this.changedSet = {};
    this.hostFiles = {};
    this.cwd = "/";
    this.dirs["/"] = true;
    this.dirs["/tmp"] = true;
  }
  normalize (path) {
    let p = path;
    if ( p.startsWith("/") == false ) {
      if ( this.cwd == "/" ) {
        p = "/" + p;
      } else {
        p = (this.cwd + "/") + p;
      }
    }
    const parts = p.split("/");
    let out = [];
    // Loop start
    for ( const part of parts) {
      if ( part.length == 0 || part == "." ) {
        continue;
      }
      if ( part == ".." ) {
        if ( out.length > 0 ) {
          out.pop();
        }
        continue;
      }
      out.push(part);
    }
    return "/" + out.join("/");
  };
  parentOf (abs) {
    const i = abs.lastIndexOf("/");
    if ( i <= 0 ) {
      return "/";
    }
    return abs.substring(0, i );
  };
  markChanged (abs) {
    if ( ( typeof(this.changedSet[abs] ) != "undefined" && Object.prototype.hasOwnProperty.call(this.changedSet, abs) ) == false ) {
      this.changedSet[abs] = true;
      this.changed.push(abs);
    }
  };
  isFile (path) {
    return ( typeof(this.files[this.normalize(path)] ) != "undefined" && Object.prototype.hasOwnProperty.call(this.files, this.normalize(path)) );
  };
  isDir (path) {
    const abs = this.normalize(path);
    if ( ( typeof(this.dirs[abs] ) != "undefined" && Object.prototype.hasOwnProperty.call(this.dirs, abs) ) ) {
      return true;
    }
    const prefix = abs + "/";
    if ( abs == "/" ) {
      return true;
    }
    const ks = Object.keys(this.files);
    // Loop start
    for ( const k of ks) {
      if ( k.startsWith(prefix) ) {
        return true;
      }
    }
    return false;
  };
  exists (path) {
    if ( this.isFile(path) ) {
      return true;
    }
    return this.isDir(path);
  };
  read (path) {
    const abs = this.normalize(path);
    return ( Object.prototype.hasOwnProperty.call(this.files, abs) ? this.files[abs] : undefined );
  };
  mkdirs (path) {
    let abs = this.normalize(path);
    while (abs != "/" && ( typeof(this.dirs[abs] ) != "undefined" && Object.prototype.hasOwnProperty.call(this.dirs, abs) ) == false) {
      this.dirs[abs] = true;
      abs = this.parentOf(abs);
    };
  };
  write (path, text) {
    const abs = this.normalize(path);
    this.mkdirs(this.parentOf(abs));
    this.files[abs] = text;
    this.markChanged(abs);
  };
  append (path, text) {
    const abs = this.normalize(path);
    let old = "";
    if ( ( typeof(this.files[abs] ) != "undefined" && Object.prototype.hasOwnProperty.call(this.files, abs) ) ) {
      old = ( Object.prototype.hasOwnProperty.call(this.files, abs) ? this.files[abs] : undefined );
    }
    this.write(abs, old + text);
  };
  remove (path) {
    const abs = this.normalize(path);
    if ( ( typeof(this.files[abs] ) != "undefined" && Object.prototype.hasOwnProperty.call(this.files, abs) ) ) {
      delete this.files[abs];
      this.markChanged(abs);
      return true;
    }
    return false;
  };
  removeDir (path) {
    const abs = this.normalize(path);
    if ( ( typeof(this.dirs[abs] ) != "undefined" && Object.prototype.hasOwnProperty.call(this.dirs, abs) ) == false ) {
      return false;
    }
    const kids = this.listDir(abs);
    if ( kids.length > 0 ) {
      return false;
    }
    delete this.dirs[abs];
    return true;
  };
  listDir (path) {
    const abs = this.normalize(path);
    let prefix = abs + "/";
    if ( abs == "/" ) {
      prefix = "/";
    }
    let seen = {};
    let out = [];
    let all = [];
    const fk = Object.keys(this.files);
    // Loop start
    for ( const f of fk) {
      all.push(f);
    }
    const dk = Object.keys(this.dirs);
    // Loop start
    for ( const d of dk) {
      all.push(d);
    }
    // Loop start
    for ( const a of all) {
      if ( a.startsWith(prefix) && a != abs ) {
        let rest = a.substring(prefix.length, a.length );
        const cut = rest.indexOf("/");
        if ( cut >= 0 ) {
          rest = rest.substring(0, cut );
        }
        if ( rest.length > 0 && ( typeof(seen[rest] ) != "undefined" && Object.prototype.hasOwnProperty.call(seen, rest) ) == false ) {
          seen[rest] = true;
          out.push(rest);
        }
      }
    }
    return PyFileSystem.sortStrings(out);
  };
  allFiles () {
    const ks = Object.keys(this.files);
    return PyFileSystem.sortStrings(ks);
  };
  put (path, text) {
    const abs = this.normalize(path);
    this.mkdirs(this.parentOf(abs));
    this.files[abs] = text;
    this.hostFiles[abs] = true;
  };
  get (path) {
    return this.read(path);
  };
  changedFiles () {
    let out = [];
    // Loop start
    for ( const p of this.changed) {
      if ( ( typeof(this.files[p] ) != "undefined" && Object.prototype.hasOwnProperty.call(this.files, p) ) ) {
        out.push(p);
      }
    }
    return out;
  };
  removedFiles () {
    let out = [];
    // Loop start
    for ( const p of this.changed) {
      if ( ( typeof(this.hostFiles[p] ) != "undefined" && Object.prototype.hasOwnProperty.call(this.hostFiles, p) ) && ( typeof(this.files[p] ) != "undefined" && Object.prototype.hasOwnProperty.call(this.files, p) ) == false ) {
        out.push(p);
      }
    }
    return out;
  };
  clearChanges () {
    let e = [];
    this.changed = e;
    let m = {};
    this.changedSet = m;
  };
}
PyFileSystem.lessStr = function(a, b) {
  const ca = Array.from(a, (rg_c) => rg_c.codePointAt(0));
  const cb = Array.from(b, (rg_c) => rg_c.codePointAt(0));
  const na = ca.length;
  const nb = cb.length;
  let i = 0;
  while (i < na && i < nb) {
    const x = ca[i];
    const y = cb[i];
    if ( x != y ) {
      return x < y;
    }
    i = i + 1;
  };
  return na < nb;
};
PyFileSystem.sortStrings = function(xs) {
  let out = [];
  // Loop start
  for ( const x of xs) {
    out.push(x);
  }
  const n = out.length;
  let k = 1;
  while (k < n) {
    const v = out[k];
    let j = k - 1;
    while (j >= 0 && PyFileSystem.lessStr(v, out[j])) {
      out[j + 1] = out[j];
      j = j - 1;
    };
    out[j + 1] = v;
    k = k + 1;
  };
  return out;
};
export class PyUnicodeNames  {
}
PyUnicodeNames.table = function() {
  return ";SPACE=20;EXCLAMATION MARK=21;QUOTATION MARK=22;NUMBER SIGN=23;DOLLAR SIGN=24;PERCENT SIGN=25;AMPERSAND=26;APOSTROPHE=27;LEFT PARENTHESIS=28;RIGHT PARENTHESIS=29;ASTERISK=2A;PLUS SIGN=2B;COMMA=2C;HYPHEN-MINUS=2D;FULL STOP=2E;SOLIDUS=2F;DIGIT ZERO=30;DIGIT ONE=31;DIGIT TWO=32;DIGIT THREE=33;DIGIT FOUR=34;DIGIT FIVE=35;DIGIT SIX=36;DIGIT SEVEN=37;DIGIT EIGHT=38;DIGIT NINE=39;COLON=3A;SEMICOLON=3B;LESS-THAN SIGN=3C;EQUALS SIGN=3D;GREATER-THAN SIGN=3E;QUESTION MARK=3F;COMMERCIAL AT=40;LATIN CAPITAL LETTER A=41;LATIN CAPITAL LETTER B=42;LATIN CAPITAL LETTER C=43;LATIN CAPITAL LETTER D=44;LATIN CAPITAL LETTER E=45;LATIN CAPITAL LETTER F=46;LATIN CAPITAL LETTER G=47;LATIN CAPITAL LETTER H=48;LATIN CAPITAL LETTER I=49;LATIN CAPITAL LETTER J=4A;LATIN CAPITAL LETTER K=4B;LATIN CAPITAL LETTER L=4C;LATIN CAPITAL LETTER M=4D;LATIN CAPITAL LETTER N=4E;LATIN CAPITAL LETTER O=4F;LATIN CAPITAL LETTER P=50;LATIN CAPITAL LETTER Q=51;LATIN CAPITAL LETTER R=52;LATIN CAPITAL LETTER S=53;LATIN CAPITAL LETTER T=54;LATIN CAPITAL LETTER U=55;LATIN CAPITAL LETTER V=56;LATIN CAPITAL LETTER W=57;LATIN CAPITAL LETTER X=58;LATIN CAPITAL LETTER Y=59;LATIN CAPITAL LETTER Z=5A;LEFT SQUARE BRACKET=5B;REVERSE SOLIDUS=5C;RIGHT SQUARE BRACKET=5D;CIRCUMFLEX ACCENT=5E;LOW LINE=5F;GRAVE ACCENT=60;LATIN SMALL LETTER A=61;LATIN SMALL LETTER B=62;LATIN SMALL LETTER C=63;LATIN SMALL LETTER D=64;LATIN SMALL LETTER E=65;LATIN SMALL LETTER F=66;LATIN SMALL LETTER G=67;LATIN SMALL LETTER H=68;LATIN SMALL LETTER I=69;LATIN SMALL LETTER J=6A;LATIN SMALL LETTER K=6B;LATIN SMALL LETTER L=6C;LATIN SMALL LETTER M=6D;LATIN SMALL LETTER N=6E;LATIN SMALL LETTER O=6F;LATIN SMALL LETTER P=70;LATIN SMALL LETTER Q=71;LATIN SMALL LETTER R=72;LATIN SMALL LETTER S=73;LATIN SMALL LETTER T=74;LATIN SMALL LETTER U=75;LATIN SMALL LETTER V=76;LATIN SMALL LETTER W=77;LATIN SMALL LETTER X=78;LATIN SMALL LETTER Y=79;LATIN SMALL LETTER Z=7A;LEFT CURLY BRACKET=7B;VERTICAL LINE=7C;RIGHT CURLY BRACKET=7D;TILDE=7E;NO-BREAK SPACE=A0;INVERTED EXCLAMATION MARK=A1;CENT SIGN=A2;POUND SIGN=A3;CURRENCY SIGN=A4;YEN SIGN=A5;BROKEN BAR=A6;SECTION SIGN=A7;DIAERESIS=A8;COPYRIGHT SIGN=A9;FEMININE ORDINAL INDICATOR=AA;LEFT-POINTING DOUBLE ANGLE QUOTATION MARK=AB;NOT SIGN=AC;SOFT HYPHEN=AD;REGISTERED SIGN=AE;MACRON=AF;DEGREE SIGN=B0;PLUS-MINUS SIGN=B1;SUPERSCRIPT TWO=B2;SUPERSCRIPT THREE=B3;ACUTE ACCENT=B4;MICRO SIGN=B5;PILCROW SIGN=B6;MIDDLE DOT=B7;CEDILLA=B8;SUPERSCRIPT ONE=B9;MASCULINE ORDINAL INDICATOR=BA;RIGHT-POINTING DOUBLE ANGLE QUOTATION MARK=BB;VULGAR FRACTION ONE QUARTER=BC;VULGAR FRACTION ONE HALF=BD;VULGAR FRACTION THREE QUARTERS=BE;INVERTED QUESTION MARK=BF;LATIN CAPITAL LETTER A WITH GRAVE=C0;LATIN CAPITAL LETTER A WITH ACUTE=C1;LATIN CAPITAL LETTER A WITH CIRCUMFLEX=C2;LATIN CAPITAL LETTER A WITH TILDE=C3;LATIN CAPITAL LETTER A WITH DIAERESIS=C4;LATIN CAPITAL LETTER A WITH RING ABOVE=C5;LATIN CAPITAL LETTER AE=C6;LATIN CAPITAL LETTER C WITH CEDILLA=C7;LATIN CAPITAL LETTER E WITH GRAVE=C8;LATIN CAPITAL LETTER E WITH ACUTE=C9;LATIN CAPITAL LETTER E WITH CIRCUMFLEX=CA;LATIN CAPITAL LETTER E WITH DIAERESIS=CB;LATIN CAPITAL LETTER I WITH GRAVE=CC;LATIN CAPITAL LETTER I WITH ACUTE=CD;LATIN CAPITAL LETTER I WITH CIRCUMFLEX=CE;LATIN CAPITAL LETTER I WITH DIAERESIS=CF;LATIN CAPITAL LETTER ETH=D0;LATIN CAPITAL LETTER N WITH TILDE=D1;LATIN CAPITAL LETTER O WITH GRAVE=D2;LATIN CAPITAL LETTER O WITH ACUTE=D3;LATIN CAPITAL LETTER O WITH CIRCUMFLEX=D4;LATIN CAPITAL LETTER O WITH TILDE=D5;LATIN CAPITAL LETTER O WITH DIAERESIS=D6;MULTIPLICATION SIGN=D7;LATIN CAPITAL LETTER O WITH STROKE=D8;LATIN CAPITAL LETTER U WITH GRAVE=D9;LATIN CAPITAL LETTER U WITH ACUTE=DA;LATIN CAPITAL LETTER U WITH CIRCUMFLEX=DB;LATIN CAPITAL LETTER U WITH DIAERESIS=DC;LATIN CAPITAL LETTER Y WITH ACUTE=DD;LATIN CAPITAL LETTER THORN=DE;LATIN SMALL LETTER SHARP S=DF;LATIN SMALL LETTER A WITH GRAVE=E0;LATIN SMALL LETTER A WITH ACUTE=E1;LATIN SMALL LETTER A WITH CIRCUMFLEX=E2;LATIN SMALL LETTER A WITH TILDE=E3;LATIN SMALL LETTER A WITH DIAERESIS=E4;LATIN SMALL LETTER A WITH RING ABOVE=E5;LATIN SMALL LETTER AE=E6;LATIN SMALL LETTER C WITH CEDILLA=E7;LATIN SMALL LETTER E WITH GRAVE=E8;LATIN SMALL LETTER E WITH ACUTE=E9;LATIN SMALL LETTER E WITH CIRCUMFLEX=EA;LATIN SMALL LETTER E WITH DIAERESIS=EB;LATIN SMALL LETTER I WITH GRAVE=EC;LATIN SMALL LETTER I WITH ACUTE=ED;LATIN SMALL LETTER I WITH CIRCUMFLEX=EE;LATIN SMALL LETTER I WITH DIAERESIS=EF;LATIN SMALL LETTER ETH=F0;LATIN SMALL LETTER N WITH TILDE=F1;LATIN SMALL LETTER O WITH GRAVE=F2;LATIN SMALL LETTER O WITH ACUTE=F3;LATIN SMALL LETTER O WITH CIRCUMFLEX=F4;LATIN SMALL LETTER O WITH TILDE=F5;LATIN SMALL LETTER O WITH DIAERESIS=F6;DIVISION SIGN=F7;LATIN SMALL LETTER O WITH STROKE=F8;LATIN SMALL LETTER U WITH GRAVE=F9;LATIN SMALL LETTER U WITH ACUTE=FA;LATIN SMALL LETTER U WITH CIRCUMFLEX=FB;LATIN SMALL LETTER U WITH DIAERESIS=FC;LATIN SMALL LETTER Y WITH ACUTE=FD;LATIN SMALL LETTER THORN=FE;LATIN SMALL LETTER Y WITH DIAERESIS=FF;LATIN CAPITAL LETTER A WITH MACRON=100;LATIN SMALL LETTER A WITH MACRON=101;LATIN CAPITAL LETTER A WITH BREVE=102;LATIN SMALL LETTER A WITH BREVE=103;LATIN CAPITAL LETTER A WITH OGONEK=104;LATIN SMALL LETTER A WITH OGONEK=105;LATIN CAPITAL LETTER C WITH ACUTE=106;LATIN SMALL LETTER C WITH ACUTE=107;LATIN CAPITAL LETTER C WITH CIRCUMFLEX=108;LATIN SMALL LETTER C WITH CIRCUMFLEX=109;LATIN CAPITAL LETTER C WITH DOT ABOVE=10A;LATIN SMALL LETTER C WITH DOT ABOVE=10B;LATIN CAPITAL LETTER C WITH CARON=10C;LATIN SMALL LETTER C WITH CARON=10D;LATIN CAPITAL LETTER D WITH CARON=10E;LATIN SMALL LETTER D WITH CARON=10F;LATIN CAPITAL LETTER D WITH STROKE=110;LATIN SMALL LETTER D WITH STROKE=111;LATIN CAPITAL LETTER E WITH MACRON=112;LATIN SMALL LETTER E WITH MACRON=113;LATIN CAPITAL LETTER E WITH BREVE=114;LATIN SMALL LETTER E WITH BREVE=115;LATIN CAPITAL LETTER E WITH DOT ABOVE=116;LATIN SMALL LETTER E WITH DOT ABOVE=117;LATIN CAPITAL LETTER E WITH OGONEK=118;LATIN SMALL LETTER E WITH OGONEK=119;LATIN CAPITAL LETTER E WITH CARON=11A;LATIN SMALL LETTER E WITH CARON=11B;LATIN CAPITAL LETTER G WITH CIRCUMFLEX=11C;LATIN SMALL LETTER G WITH CIRCUMFLEX=11D;LATIN CAPITAL LETTER G WITH BREVE=11E;LATIN SMALL LETTER G WITH BREVE=11F;LATIN CAPITAL LETTER G WITH DOT ABOVE=120;LATIN SMALL LETTER G WITH DOT ABOVE=121;LATIN CAPITAL LETTER G WITH CEDILLA=122;LATIN SMALL LETTER G WITH CEDILLA=123;LATIN CAPITAL LETTER H WITH CIRCUMFLEX=124;LATIN SMALL LETTER H WITH CIRCUMFLEX=125;LATIN CAPITAL LETTER H WITH STROKE=126;LATIN SMALL LETTER H WITH STROKE=127;LATIN CAPITAL LETTER I WITH TILDE=128;LATIN SMALL LETTER I WITH TILDE=129;LATIN CAPITAL LETTER I WITH MACRON=12A;LATIN SMALL LETTER I WITH MACRON=12B;LATIN CAPITAL LETTER I WITH BREVE=12C;LATIN SMALL LETTER I WITH BREVE=12D;LATIN CAPITAL LETTER I WITH OGONEK=12E;LATIN SMALL LETTER I WITH OGONEK=12F;LATIN CAPITAL LETTER I WITH DOT ABOVE=130;LATIN SMALL LETTER DOTLESS I=131;LATIN CAPITAL LIGATURE IJ=132;LATIN SMALL LIGATURE IJ=133;LATIN CAPITAL LETTER J WITH CIRCUMFLEX=134;LATIN SMALL LETTER J WITH CIRCUMFLEX=135;LATIN CAPITAL LETTER K WITH CEDILLA=136;LATIN SMALL LETTER K WITH CEDILLA=137;LATIN SMALL LETTER KRA=138;LATIN CAPITAL LETTER L WITH ACUTE=139;LATIN SMALL LETTER L WITH ACUTE=13A;LATIN CAPITAL LETTER L WITH CEDILLA=13B;LATIN SMALL LETTER L WITH CEDILLA=13C;LATIN CAPITAL LETTER L WITH CARON=13D;LATIN SMALL LETTER L WITH CARON=13E;LATIN CAPITAL LETTER L WITH MIDDLE DOT=13F;LATIN SMALL LETTER L WITH MIDDLE DOT=140;LATIN CAPITAL LETTER L WITH STROKE=141;LATIN SMALL LETTER L WITH STROKE=142;LATIN CAPITAL LETTER N WITH ACUTE=143;LATIN SMALL LETTER N WITH ACUTE=144;LATIN CAPITAL LETTER N WITH CEDILLA=145;LATIN SMALL LETTER N WITH CEDILLA=146;LATIN CAPITAL LETTER N WITH CARON=147;LATIN SMALL LETTER N WITH CARON=148;LATIN SMALL LETTER N PRECEDED BY APOSTROPHE=149;LATIN CAPITAL LETTER ENG=14A;LATIN SMALL LETTER ENG=14B;LATIN CAPITAL LETTER O WITH MACRON=14C;LATIN SMALL LETTER O WITH MACRON=14D;LATIN CAPITAL LETTER O WITH BREVE=14E;LATIN SMALL LETTER O WITH BREVE=14F;LATIN CAPITAL LETTER O WITH DOUBLE ACUTE=150;LATIN SMALL LETTER O WITH DOUBLE ACUTE=151;LATIN CAPITAL LIGATURE OE=152;LATIN SMALL LIGATURE OE=153;LATIN CAPITAL LETTER R WITH ACUTE=154;LATIN SMALL LETTER R WITH ACUTE=155;LATIN CAPITAL LETTER R WITH CEDILLA=156;LATIN SMALL LETTER R WITH CEDILLA=157;LATIN CAPITAL LETTER R WITH CARON=158;LATIN SMALL LETTER R WITH CARON=159;LATIN CAPITAL LETTER S WITH ACUTE=15A;LATIN SMALL LETTER S WITH ACUTE=15B;LATIN CAPITAL LETTER S WITH CIRCUMFLEX=15C;LATIN SMALL LETTER S WITH CIRCUMFLEX=15D;LATIN CAPITAL LETTER S WITH CEDILLA=15E;LATIN SMALL LETTER S WITH CEDILLA=15F;LATIN CAPITAL LETTER S WITH CARON=160;LATIN SMALL LETTER S WITH CARON=161;LATIN CAPITAL LETTER T WITH CEDILLA=162;LATIN SMALL LETTER T WITH CEDILLA=163;LATIN CAPITAL LETTER T WITH CARON=164;LATIN SMALL LETTER T WITH CARON=165;LATIN CAPITAL LETTER T WITH STROKE=166;LATIN SMALL LETTER T WITH STROKE=167;LATIN CAPITAL LETTER U WITH TILDE=168;LATIN SMALL LETTER U WITH TILDE=169;LATIN CAPITAL LETTER U WITH MACRON=16A;LATIN SMALL LETTER U WITH MACRON=16B;LATIN CAPITAL LETTER U WITH BREVE=16C;LATIN SMALL LETTER U WITH BREVE=16D;LATIN CAPITAL LETTER U WITH RING ABOVE=16E;LATIN SMALL LETTER U WITH RING ABOVE=16F;LATIN CAPITAL LETTER U WITH DOUBLE ACUTE=170;LATIN SMALL LETTER U WITH DOUBLE ACUTE=171;LATIN CAPITAL LETTER U WITH OGONEK=172;LATIN SMALL LETTER U WITH OGONEK=173;LATIN CAPITAL LETTER W WITH CIRCUMFLEX=174;LATIN SMALL LETTER W WITH CIRCUMFLEX=175;LATIN CAPITAL LETTER Y WITH CIRCUMFLEX=176;LATIN SMALL LETTER Y WITH CIRCUMFLEX=177;LATIN CAPITAL LETTER Y WITH DIAERESIS=178;LATIN CAPITAL LETTER Z WITH ACUTE=179;LATIN SMALL LETTER Z WITH ACUTE=17A;LATIN CAPITAL LETTER Z WITH DOT ABOVE=17B;LATIN SMALL LETTER Z WITH DOT ABOVE=17C;LATIN CAPITAL LETTER Z WITH CARON=17D;LATIN SMALL LETTER Z WITH CARON=17E;LATIN SMALL LETTER LONG S=17F;LATIN SMALL LETTER B WITH STROKE=180;LATIN CAPITAL LETTER B WITH HOOK=181;LATIN CAPITAL LETTER B WITH TOPBAR=182;LATIN SMALL LETTER B WITH TOPBAR=183;LATIN CAPITAL LETTER TONE SIX=184;LATIN SMALL LETTER TONE SIX=185;LATIN CAPITAL LETTER OPEN O=186;LATIN CAPITAL LETTER C WITH HOOK=187;LATIN SMALL LETTER C WITH HOOK=188;LATIN CAPITAL LETTER AFRICAN D=189;LATIN CAPITAL LETTER D WITH HOOK=18A;LATIN CAPITAL LETTER D WITH TOPBAR=18B;LATIN SMALL LETTER D WITH TOPBAR=18C;LATIN SMALL LETTER TURNED DELTA=18D;LATIN CAPITAL LETTER REVERSED E=18E;LATIN CAPITAL LETTER SCHWA=18F;LATIN CAPITAL LETTER OPEN E=190;LATIN CAPITAL LETTER F WITH HOOK=191;LATIN SMALL LETTER F WITH HOOK=192;LATIN CAPITAL LETTER G WITH HOOK=193;LATIN CAPITAL LETTER GAMMA=194;LATIN SMALL LETTER HV=195;LATIN CAPITAL LETTER IOTA=196;LATIN CAPITAL LETTER I WITH STROKE=197;LATIN CAPITAL LETTER K WITH HOOK=198;LATIN SMALL LETTER K WITH HOOK=199;LATIN SMALL LETTER L WITH BAR=19A;LATIN SMALL LETTER LAMBDA WITH STROKE=19B;LATIN CAPITAL LETTER TURNED M=19C;LATIN CAPITAL LETTER N WITH LEFT HOOK=19D;LATIN SMALL LETTER N WITH LONG RIGHT LEG=19E;LATIN CAPITAL LETTER O WITH MIDDLE TILDE=19F;LATIN CAPITAL LETTER O WITH HORN=1A0;LATIN SMALL LETTER O WITH HORN=1A1;LATIN CAPITAL LETTER OI=1A2;LATIN SMALL LETTER OI=1A3;LATIN CAPITAL LETTER P WITH HOOK=1A4;LATIN SMALL LETTER P WITH HOOK=1A5;LATIN LETTER YR=1A6;LATIN CAPITAL LETTER TONE TWO=1A7;LATIN SMALL LETTER TONE TWO=1A8;LATIN CAPITAL LETTER ESH=1A9;LATIN LETTER REVERSED ESH LOOP=1AA;LATIN SMALL LETTER T WITH PALATAL HOOK=1AB;LATIN CAPITAL LETTER T WITH HOOK=1AC;LATIN SMALL LETTER T WITH HOOK=1AD;LATIN CAPITAL LETTER T WITH RETROFLEX HOOK=1AE;LATIN CAPITAL LETTER U WITH HORN=1AF;LATIN SMALL LETTER U WITH HORN=1B0;LATIN CAPITAL LETTER UPSILON=1B1;LATIN CAPITAL LETTER V WITH HOOK=1B2;LATIN CAPITAL LETTER Y WITH HOOK=1B3;LATIN SMALL LETTER Y WITH HOOK=1B4;LATIN CAPITAL LETTER Z WITH STROKE=1B5;LATIN SMALL LETTER Z WITH STROKE=1B6;LATIN CAPITAL LETTER EZH=1B7;LATIN CAPITAL LETTER EZH REVERSED=1B8;LATIN SMALL LETTER EZH REVERSED=1B9;LATIN SMALL LETTER EZH WITH TAIL=1BA;LATIN LETTER TWO WITH STROKE=1BB;LATIN CAPITAL LETTER TONE FIVE=1BC;LATIN SMALL LETTER TONE FIVE=1BD;LATIN LETTER INVERTED GLOTTAL STOP WITH STROKE=1BE;LATIN LETTER WYNN=1BF;LATIN LETTER DENTAL CLICK=1C0;LATIN LETTER LATERAL CLICK=1C1;LATIN LETTER ALVEOLAR CLICK=1C2;LATIN LETTER RETROFLEX CLICK=1C3;LATIN CAPITAL LETTER DZ WITH CARON=1C4;LATIN CAPITAL LETTER D WITH SMALL LETTER Z WITH CARON=1C5;LATIN SMALL LETTER DZ WITH CARON=1C6;LATIN CAPITAL LETTER LJ=1C7;LATIN CAPITAL LETTER L WITH SMALL LETTER J=1C8;LATIN SMALL LETTER LJ=1C9;LATIN CAPITAL LETTER NJ=1CA;LATIN CAPITAL LETTER N WITH SMALL LETTER J=1CB;LATIN SMALL LETTER NJ=1CC;LATIN CAPITAL LETTER A WITH CARON=1CD;LATIN SMALL LETTER A WITH CARON=1CE;LATIN CAPITAL LETTER I WITH CARON=1CF;LATIN SMALL LETTER I WITH CARON=1D0;LATIN CAPITAL LETTER O WITH CARON=1D1;LATIN SMALL LETTER O WITH CARON=1D2;LATIN CAPITAL LETTER U WITH CARON=1D3;LATIN SMALL LETTER U WITH CARON=1D4;LATIN CAPITAL LETTER U WITH DIAERESIS AND MACRON=1D5;LATIN SMALL LETTER U WITH DIAERESIS AND MACRON=1D6;LATIN CAPITAL LETTER U WITH DIAERESIS AND ACUTE=1D7;LATIN SMALL LETTER U WITH DIAERESIS AND ACUTE=1D8;LATIN CAPITAL LETTER U WITH DIAERESIS AND CARON=1D9;LATIN SMALL LETTER U WITH DIAERESIS AND CARON=1DA;LATIN CAPITAL LETTER U WITH DIAERESIS AND GRAVE=1DB;LATIN SMALL LETTER U WITH DIAERESIS AND GRAVE=1DC;LATIN SMALL LETTER TURNED E=1DD;LATIN CAPITAL LETTER A WITH DIAERESIS AND MACRON=1DE;LATIN SMALL LETTER A WITH DIAERESIS AND MACRON=1DF;LATIN CAPITAL LETTER A WITH DOT ABOVE AND MACRON=1E0;LATIN SMALL LETTER A WITH DOT ABOVE AND MACRON=1E1;LATIN CAPITAL LETTER AE WITH MACRON=1E2;LATIN SMALL LETTER AE WITH MACRON=1E3;LATIN CAPITAL LETTER G WITH STROKE=1E4;LATIN SMALL LETTER G WITH STROKE=1E5;LATIN CAPITAL LETTER G WITH CARON=1E6;LATIN SMALL LETTER G WITH CARON=1E7;LATIN CAPITAL LETTER K WITH CARON=1E8;LATIN SMALL LETTER K WITH CARON=1E9;LATIN CAPITAL LETTER O WITH OGONEK=1EA;LATIN SMALL LETTER O WITH OGONEK=1EB;LATIN CAPITAL LETTER O WITH OGONEK AND MACRON=1EC;LATIN SMALL LETTER O WITH OGONEK AND MACRON=1ED;LATIN CAPITAL LETTER EZH WITH CARON=1EE;LATIN SMALL LETTER EZH WITH CARON=1EF;LATIN SMALL LETTER J WITH CARON=1F0;LATIN CAPITAL LETTER DZ=1F1;LATIN CAPITAL LETTER D WITH SMALL LETTER Z=1F2;LATIN SMALL LETTER DZ=1F3;LATIN CAPITAL LETTER G WITH ACUTE=1F4;LATIN SMALL LETTER G WITH ACUTE=1F5;LATIN CAPITAL LETTER HWAIR=1F6;LATIN CAPITAL LETTER WYNN=1F7;LATIN CAPITAL LETTER N WITH GRAVE=1F8;LATIN SMALL LETTER N WITH GRAVE=1F9;LATIN CAPITAL LETTER A WITH RING ABOVE AND ACUTE=1FA;LATIN SMALL LETTER A WITH RING ABOVE AND ACUTE=1FB;LATIN CAPITAL LETTER AE WITH ACUTE=1FC;LATIN SMALL LETTER AE WITH ACUTE=1FD;LATIN CAPITAL LETTER O WITH STROKE AND ACUTE=1FE;LATIN SMALL LETTER O WITH STROKE AND ACUTE=1FF;LATIN CAPITAL LETTER A WITH DOUBLE GRAVE=200;LATIN SMALL LETTER A WITH DOUBLE GRAVE=201;LATIN CAPITAL LETTER A WITH INVERTED BREVE=202;LATIN SMALL LETTER A WITH INVERTED BREVE=203;LATIN CAPITAL LETTER E WITH DOUBLE GRAVE=204;LATIN SMALL LETTER E WITH DOUBLE GRAVE=205;LATIN CAPITAL LETTER E WITH INVERTED BREVE=206;LATIN SMALL LETTER E WITH INVERTED BREVE=207;LATIN CAPITAL LETTER I WITH DOUBLE GRAVE=208;LATIN SMALL LETTER I WITH DOUBLE GRAVE=209;LATIN CAPITAL LETTER I WITH INVERTED BREVE=20A;LATIN SMALL LETTER I WITH INVERTED BREVE=20B;LATIN CAPITAL LETTER O WITH DOUBLE GRAVE=20C;LATIN SMALL LETTER O WITH DOUBLE GRAVE=20D;LATIN CAPITAL LETTER O WITH INVERTED BREVE=20E;LATIN SMALL LETTER O WITH INVERTED BREVE=20F;LATIN CAPITAL LETTER R WITH DOUBLE GRAVE=210;LATIN SMALL LETTER R WITH DOUBLE GRAVE=211;LATIN CAPITAL LETTER R WITH INVERTED BREVE=212;LATIN SMALL LETTER R WITH INVERTED BREVE=213;LATIN CAPITAL LETTER U WITH DOUBLE GRAVE=214;LATIN SMALL LETTER U WITH DOUBLE GRAVE=215;LATIN CAPITAL LETTER U WITH INVERTED BREVE=216;LATIN SMALL LETTER U WITH INVERTED BREVE=217;LATIN CAPITAL LETTER S WITH COMMA BELOW=218;LATIN SMALL LETTER S WITH COMMA BELOW=219;LATIN CAPITAL LETTER T WITH COMMA BELOW=21A;LATIN SMALL LETTER T WITH COMMA BELOW=21B;LATIN CAPITAL LETTER YOGH=21C;LATIN SMALL LETTER YOGH=21D;LATIN CAPITAL LETTER H WITH CARON=21E;LATIN SMALL LETTER H WITH CARON=21F;LATIN CAPITAL LETTER N WITH LONG RIGHT LEG=220;LATIN SMALL LETTER D WITH CURL=221;LATIN CAPITAL LETTER OU=222;LATIN SMALL LETTER OU=223;LATIN CAPITAL LETTER Z WITH HOOK=224;LATIN SMALL LETTER Z WITH HOOK=225;LATIN CAPITAL LETTER A WITH DOT ABOVE=226;LATIN SMALL LETTER A WITH DOT ABOVE=227;LATIN CAPITAL LETTER E WITH CEDILLA=228;LATIN SMALL LETTER E WITH CEDILLA=229;LATIN CAPITAL LETTER O WITH DIAERESIS AND MACRON=22A;LATIN SMALL LETTER O WITH DIAERESIS AND MACRON=22B;LATIN CAPITAL LETTER O WITH TILDE AND MACRON=22C;LATIN SMALL LETTER O WITH TILDE AND MACRON=22D;LATIN CAPITAL LETTER O WITH DOT ABOVE=22E;LATIN SMALL LETTER O WITH DOT ABOVE=22F;LATIN CAPITAL LETTER O WITH DOT ABOVE AND MACRON=230;LATIN SMALL LETTER O WITH DOT ABOVE AND MACRON=231;LATIN CAPITAL LETTER Y WITH MACRON=232;LATIN SMALL LETTER Y WITH MACRON=233;LATIN SMALL LETTER L WITH CURL=234;LATIN SMALL LETTER N WITH CURL=235;LATIN SMALL LETTER T WITH CURL=236;LATIN SMALL LETTER DOTLESS J=237;LATIN SMALL LETTER DB DIGRAPH=238;LATIN SMALL LETTER QP DIGRAPH=239;LATIN CAPITAL LETTER A WITH STROKE=23A;LATIN CAPITAL LETTER C WITH STROKE=23B;LATIN SMALL LETTER C WITH STROKE=23C;LATIN CAPITAL LETTER L WITH BAR=23D;LATIN CAPITAL LETTER T WITH DIAGONAL STROKE=23E;LATIN SMALL LETTER S WITH SWASH TAIL=23F;LATIN SMALL LETTER Z WITH SWASH TAIL=240;LATIN CAPITAL LETTER GLOTTAL STOP=241;LATIN SMALL LETTER GLOTTAL STOP=242;LATIN CAPITAL LETTER B WITH STROKE=243;LATIN CAPITAL LETTER U BAR=244;LATIN CAPITAL LETTER TURNED V=245;LATIN CAPITAL LETTER E WITH STROKE=246;LATIN SMALL LETTER E WITH STROKE=247;LATIN CAPITAL LETTER J WITH STROKE=248;LATIN SMALL LETTER J WITH STROKE=249;LATIN CAPITAL LETTER SMALL Q WITH HOOK TAIL=24A;LATIN SMALL LETTER Q WITH HOOK TAIL=24B;LATIN CAPITAL LETTER R WITH STROKE=24C;LATIN SMALL LETTER R WITH STROKE=24D;LATIN CAPITAL LETTER Y WITH STROKE=24E;LATIN SMALL LETTER Y WITH STROKE=24F;GREEK CAPITAL LETTER HETA=370;GREEK SMALL LETTER HETA=371;GREEK CAPITAL LETTER ARCHAIC SAMPI=372;GREEK SMALL LETTER ARCHAIC SAMPI=373;GREEK NUMERAL SIGN=374;GREEK LOWER NUMERAL SIGN=375;GREEK CAPITAL LETTER PAMPHYLIAN DIGAMMA=376;GREEK SMALL LETTER PAMPHYLIAN DIGAMMA=377;GREEK YPOGEGRAMMENI=37A;GREEK SMALL REVERSED LUNATE SIGMA SYMBOL=37B;GREEK SMALL DOTTED LUNATE SIGMA SYMBOL=37C;GREEK SMALL REVERSED DOTTED LUNATE SIGMA SYMBOL=37D;GREEK QUESTION MARK=37E;GREEK CAPITAL LETTER YOT=37F;GREEK TONOS=384;GREEK DIALYTIKA TONOS=385;GREEK CAPITAL LETTER ALPHA WITH TONOS=386;GREEK ANO TELEIA=387;GREEK CAPITAL LETTER EPSILON WITH TONOS=388;GREEK CAPITAL LETTER ETA WITH TONOS=389;GREEK CAPITAL LETTER IOTA WITH TONOS=38A;GREEK CAPITAL LETTER OMICRON WITH TONOS=38C;GREEK CAPITAL LETTER UPSILON WITH TONOS=38E;GREEK CAPITAL LETTER OMEGA WITH TONOS=38F;GREEK SMALL LETTER IOTA WITH DIALYTIKA AND TONOS=390;GREEK CAPITAL LETTER ALPHA=391;GREEK CAPITAL LETTER BETA=392;GREEK CAPITAL LETTER GAMMA=393;GREEK CAPITAL LETTER DELTA=394;GREEK CAPITAL LETTER EPSILON=395;GREEK CAPITAL LETTER ZETA=396;GREEK CAPITAL LETTER ETA=397;GREEK CAPITAL LETTER THETA=398;GREEK CAPITAL LETTER IOTA=399;GREEK CAPITAL LETTER KAPPA=39A;GREEK CAPITAL LETTER LAMDA=39B;GREEK CAPITAL LETTER MU=39C;GREEK CAPITAL LETTER NU=39D;GREEK CAPITAL LETTER XI=39E;GREEK CAPITAL LETTER OMICRON=39F;GREEK CAPITAL LETTER PI=3A0;GREEK CAPITAL LETTER RHO=3A1;GREEK CAPITAL LETTER SIGMA=3A3;GREEK CAPITAL LETTER TAU=3A4;GREEK CAPITAL LETTER UPSILON=3A5;GREEK CAPITAL LETTER PHI=3A6;GREEK CAPITAL LETTER CHI=3A7;GREEK CAPITAL LETTER PSI=3A8;GREEK CAPITAL LETTER OMEGA=3A9;GREEK CAPITAL LETTER IOTA WITH DIALYTIKA=3AA;GREEK CAPITAL LETTER UPSILON WITH DIALYTIKA=3AB;GREEK SMALL LETTER ALPHA WITH TONOS=3AC;GREEK SMALL LETTER EPSILON WITH TONOS=3AD;GREEK SMALL LETTER ETA WITH TONOS=3AE;GREEK SMALL LETTER IOTA WITH TONOS=3AF;GREEK SMALL LETTER UPSILON WITH DIALYTIKA AND TONOS=3B0;GREEK SMALL LETTER ALPHA=3B1;GREEK SMALL LETTER BETA=3B2;GREEK SMALL LETTER GAMMA=3B3;GREEK SMALL LETTER DELTA=3B4;GREEK SMALL LETTER EPSILON=3B5;GREEK SMALL LETTER ZETA=3B6;GREEK SMALL LETTER ETA=3B7;GREEK SMALL LETTER THETA=3B8;GREEK SMALL LETTER IOTA=3B9;GREEK SMALL LETTER KAPPA=3BA;GREEK SMALL LETTER LAMDA=3BB;GREEK SMALL LETTER MU=3BC;GREEK SMALL LETTER NU=3BD;GREEK SMALL LETTER XI=3BE;GREEK SMALL LETTER OMICRON=3BF;GREEK SMALL LETTER PI=3C0;GREEK SMALL LETTER RHO=3C1;GREEK SMALL LETTER FINAL SIGMA=3C2;GREEK SMALL LETTER SIGMA=3C3;GREEK SMALL LETTER TAU=3C4;GREEK SMALL LETTER UPSILON=3C5;GREEK SMALL LETTER PHI=3C6;GREEK SMALL LETTER CHI=3C7;GREEK SMALL LETTER PSI=3C8;GREEK SMALL LETTER OMEGA=3C9;GREEK SMALL LETTER IOTA WITH DIALYTIKA=3CA;GREEK SMALL LETTER UPSILON WITH DIALYTIKA=3CB;GREEK SMALL LETTER OMICRON WITH TONOS=3CC;GREEK SMALL LETTER UPSILON WITH TONOS=3CD;GREEK SMALL LETTER OMEGA WITH TONOS=3CE;GREEK CAPITAL KAI SYMBOL=3CF;GREEK BETA SYMBOL=3D0;GREEK THETA SYMBOL=3D1;GREEK UPSILON WITH HOOK SYMBOL=3D2;GREEK UPSILON WITH ACUTE AND HOOK SYMBOL=3D3;GREEK UPSILON WITH DIAERESIS AND HOOK SYMBOL=3D4;GREEK PHI SYMBOL=3D5;GREEK PI SYMBOL=3D6;GREEK KAI SYMBOL=3D7;GREEK LETTER ARCHAIC KOPPA=3D8;GREEK SMALL LETTER ARCHAIC KOPPA=3D9;GREEK LETTER STIGMA=3DA;GREEK SMALL LETTER STIGMA=3DB;GREEK LETTER DIGAMMA=3DC;GREEK SMALL LETTER DIGAMMA=3DD;GREEK LETTER KOPPA=3DE;GREEK SMALL LETTER KOPPA=3DF;GREEK LETTER SAMPI=3E0;GREEK SMALL LETTER SAMPI=3E1;COPTIC CAPITAL LETTER SHEI=3E2;COPTIC SMALL LETTER SHEI=3E3;COPTIC CAPITAL LETTER FEI=3E4;COPTIC SMALL LETTER FEI=3E5;COPTIC CAPITAL LETTER KHEI=3E6;COPTIC SMALL LETTER KHEI=3E7;COPTIC CAPITAL LETTER HORI=3E8;COPTIC SMALL LETTER HORI=3E9;COPTIC CAPITAL LETTER GANGIA=3EA;COPTIC SMALL LETTER GANGIA=3EB;COPTIC CAPITAL LETTER SHIMA=3EC;COPTIC SMALL LETTER SHIMA=3ED;COPTIC CAPITAL LETTER DEI=3EE;COPTIC SMALL LETTER DEI=3EF;GREEK KAPPA SYMBOL=3F0;GREEK RHO SYMBOL=3F1;GREEK LUNATE SIGMA SYMBOL=3F2;GREEK LETTER YOT=3F3;GREEK CAPITAL THETA SYMBOL=3F4;GREEK LUNATE EPSILON SYMBOL=3F5;GREEK REVERSED LUNATE EPSILON SYMBOL=3F6;GREEK CAPITAL LETTER SHO=3F7;GREEK SMALL LETTER SHO=3F8;GREEK CAPITAL LUNATE SIGMA SYMBOL=3F9;GREEK CAPITAL LETTER SAN=3FA;GREEK SMALL LETTER SAN=3FB;GREEK RHO WITH STROKE SYMBOL=3FC;GREEK CAPITAL REVERSED LUNATE SIGMA SYMBOL=3FD;GREEK CAPITAL DOTTED LUNATE SIGMA SYMBOL=3FE;GREEK CAPITAL REVERSED DOTTED LUNATE SIGMA SYMBOL=3FF;CYRILLIC CAPITAL LETTER IE WITH GRAVE=400;CYRILLIC CAPITAL LETTER IO=401;CYRILLIC CAPITAL LETTER DJE=402;CYRILLIC CAPITAL LETTER GJE=403;CYRILLIC CAPITAL LETTER UKRAINIAN IE=404;CYRILLIC CAPITAL LETTER DZE=405;CYRILLIC CAPITAL LETTER BYELORUSSIAN-UKRAINIAN I=406;CYRILLIC CAPITAL LETTER YI=407;CYRILLIC CAPITAL LETTER JE=408;CYRILLIC CAPITAL LETTER LJE=409;CYRILLIC CAPITAL LETTER NJE=40A;CYRILLIC CAPITAL LETTER TSHE=40B;CYRILLIC CAPITAL LETTER KJE=40C;CYRILLIC CAPITAL LETTER I WITH GRAVE=40D;CYRILLIC CAPITAL LETTER SHORT U=40E;CYRILLIC CAPITAL LETTER DZHE=40F;CYRILLIC CAPITAL LETTER A=410;CYRILLIC CAPITAL LETTER BE=411;CYRILLIC CAPITAL LETTER VE=412;CYRILLIC CAPITAL LETTER GHE=413;CYRILLIC CAPITAL LETTER DE=414;CYRILLIC CAPITAL LETTER IE=415;CYRILLIC CAPITAL LETTER ZHE=416;CYRILLIC CAPITAL LETTER ZE=417;CYRILLIC CAPITAL LETTER I=418;CYRILLIC CAPITAL LETTER SHORT I=419;CYRILLIC CAPITAL LETTER KA=41A;CYRILLIC CAPITAL LETTER EL=41B;CYRILLIC CAPITAL LETTER EM=41C;CYRILLIC CAPITAL LETTER EN=41D;CYRILLIC CAPITAL LETTER O=41E;CYRILLIC CAPITAL LETTER PE=41F;CYRILLIC CAPITAL LETTER ER=420;CYRILLIC CAPITAL LETTER ES=421;CYRILLIC CAPITAL LETTER TE=422;CYRILLIC CAPITAL LETTER U=423;CYRILLIC CAPITAL LETTER EF=424;CYRILLIC CAPITAL LETTER HA=425;CYRILLIC CAPITAL LETTER TSE=426;CYRILLIC CAPITAL LETTER CHE=427;CYRILLIC CAPITAL LETTER SHA=428;CYRILLIC CAPITAL LETTER SHCHA=429;CYRILLIC CAPITAL LETTER HARD SIGN=42A;CYRILLIC CAPITAL LETTER YERU=42B;CYRILLIC CAPITAL LETTER SOFT SIGN=42C;CYRILLIC CAPITAL LETTER E=42D;CYRILLIC CAPITAL LETTER YU=42E;CYRILLIC CAPITAL LETTER YA=42F;CYRILLIC SMALL LETTER A=430;CYRILLIC SMALL LETTER BE=431;CYRILLIC SMALL LETTER VE=432;CYRILLIC SMALL LETTER GHE=433;CYRILLIC SMALL LETTER DE=434;CYRILLIC SMALL LETTER IE=435;CYRILLIC SMALL LETTER ZHE=436;CYRILLIC SMALL LETTER ZE=437;CYRILLIC SMALL LETTER I=438;CYRILLIC SMALL LETTER SHORT I=439;CYRILLIC SMALL LETTER KA=43A;CYRILLIC SMALL LETTER EL=43B;CYRILLIC SMALL LETTER EM=43C;CYRILLIC SMALL LETTER EN=43D;CYRILLIC SMALL LETTER O=43E;CYRILLIC SMALL LETTER PE=43F;CYRILLIC SMALL LETTER ER=440;CYRILLIC SMALL LETTER ES=441;CYRILLIC SMALL LETTER TE=442;CYRILLIC SMALL LETTER U=443;CYRILLIC SMALL LETTER EF=444;CYRILLIC SMALL LETTER HA=445;CYRILLIC SMALL LETTER TSE=446;CYRILLIC SMALL LETTER CHE=447;CYRILLIC SMALL LETTER SHA=448;CYRILLIC SMALL LETTER SHCHA=449;CYRILLIC SMALL LETTER HARD SIGN=44A;CYRILLIC SMALL LETTER YERU=44B;CYRILLIC SMALL LETTER SOFT SIGN=44C;CYRILLIC SMALL LETTER E=44D;CYRILLIC SMALL LETTER YU=44E;CYRILLIC SMALL LETTER YA=44F;CYRILLIC SMALL LETTER IE WITH GRAVE=450;CYRILLIC SMALL LETTER IO=451;CYRILLIC SMALL LETTER DJE=452;CYRILLIC SMALL LETTER GJE=453;CYRILLIC SMALL LETTER UKRAINIAN IE=454;CYRILLIC SMALL LETTER DZE=455;CYRILLIC SMALL LETTER BYELORUSSIAN-UKRAINIAN I=456;CYRILLIC SMALL LETTER YI=457;CYRILLIC SMALL LETTER JE=458;CYRILLIC SMALL LETTER LJE=459;CYRILLIC SMALL LETTER NJE=45A;CYRILLIC SMALL LETTER TSHE=45B;CYRILLIC SMALL LETTER KJE=45C;CYRILLIC SMALL LETTER I WITH GRAVE=45D;CYRILLIC SMALL LETTER SHORT U=45E;CYRILLIC SMALL LETTER DZHE=45F;CYRILLIC CAPITAL LETTER OMEGA=460;CYRILLIC SMALL LETTER OMEGA=461;CYRILLIC CAPITAL LETTER YAT=462;CYRILLIC SMALL LETTER YAT=463;CYRILLIC CAPITAL LETTER IOTIFIED E=464;CYRILLIC SMALL LETTER IOTIFIED E=465;CYRILLIC CAPITAL LETTER LITTLE YUS=466;CYRILLIC SMALL LETTER LITTLE YUS=467;CYRILLIC CAPITAL LETTER IOTIFIED LITTLE YUS=468;CYRILLIC SMALL LETTER IOTIFIED LITTLE YUS=469;CYRILLIC CAPITAL LETTER BIG YUS=46A;CYRILLIC SMALL LETTER BIG YUS=46B;CYRILLIC CAPITAL LETTER IOTIFIED BIG YUS=46C;CYRILLIC SMALL LETTER IOTIFIED BIG YUS=46D;CYRILLIC CAPITAL LETTER KSI=46E;CYRILLIC SMALL LETTER KSI=46F;CYRILLIC CAPITAL LETTER PSI=470;CYRILLIC SMALL LETTER PSI=471;CYRILLIC CAPITAL LETTER FITA=472;CYRILLIC SMALL LETTER FITA=473;CYRILLIC CAPITAL LETTER IZHITSA=474;CYRILLIC SMALL LETTER IZHITSA=475;CYRILLIC CAPITAL LETTER IZHITSA WITH DOUBLE GRAVE ACCENT=476;CYRILLIC SMALL LETTER IZHITSA WITH DOUBLE GRAVE ACCENT=477;CYRILLIC CAPITAL LETTER UK=478;CYRILLIC SMALL LETTER UK=479;CYRILLIC CAPITAL LETTER ROUND OMEGA=47A;CYRILLIC SMALL LETTER ROUND OMEGA=47B;CYRILLIC CAPITAL LETTER OMEGA WITH TITLO=47C;CYRILLIC SMALL LETTER OMEGA WITH TITLO=47D;CYRILLIC CAPITAL LETTER OT=47E;CYRILLIC SMALL LETTER OT=47F;CYRILLIC CAPITAL LETTER KOPPA=480;CYRILLIC SMALL LETTER KOPPA=481;CYRILLIC THOUSANDS SIGN=482;COMBINING CYRILLIC TITLO=483;COMBINING CYRILLIC PALATALIZATION=484;COMBINING CYRILLIC DASIA PNEUMATA=485;COMBINING CYRILLIC PSILI PNEUMATA=486;COMBINING CYRILLIC POKRYTIE=487;COMBINING CYRILLIC HUNDRED THOUSANDS SIGN=488;COMBINING CYRILLIC MILLIONS SIGN=489;CYRILLIC CAPITAL LETTER SHORT I WITH TAIL=48A;CYRILLIC SMALL LETTER SHORT I WITH TAIL=48B;CYRILLIC CAPITAL LETTER SEMISOFT SIGN=48C;CYRILLIC SMALL LETTER SEMISOFT SIGN=48D;CYRILLIC CAPITAL LETTER ER WITH TICK=48E;CYRILLIC SMALL LETTER ER WITH TICK=48F;CYRILLIC CAPITAL LETTER GHE WITH UPTURN=490;CYRILLIC SMALL LETTER GHE WITH UPTURN=491;CYRILLIC CAPITAL LETTER GHE WITH STROKE=492;CYRILLIC SMALL LETTER GHE WITH STROKE=493;CYRILLIC CAPITAL LETTER GHE WITH MIDDLE HOOK=494;CYRILLIC SMALL LETTER GHE WITH MIDDLE HOOK=495;CYRILLIC CAPITAL LETTER ZHE WITH DESCENDER=496;CYRILLIC SMALL LETTER ZHE WITH DESCENDER=497;CYRILLIC CAPITAL LETTER ZE WITH DESCENDER=498;CYRILLIC SMALL LETTER ZE WITH DESCENDER=499;CYRILLIC CAPITAL LETTER KA WITH DESCENDER=49A;CYRILLIC SMALL LETTER KA WITH DESCENDER=49B;CYRILLIC CAPITAL LETTER KA WITH VERTICAL STROKE=49C;CYRILLIC SMALL LETTER KA WITH VERTICAL STROKE=49D;CYRILLIC CAPITAL LETTER KA WITH STROKE=49E;CYRILLIC SMALL LETTER KA WITH STROKE=49F;CYRILLIC CAPITAL LETTER BASHKIR KA=4A0;CYRILLIC SMALL LETTER BASHKIR KA=4A1;CYRILLIC CAPITAL LETTER EN WITH DESCENDER=4A2;CYRILLIC SMALL LETTER EN WITH DESCENDER=4A3;CYRILLIC CAPITAL LIGATURE EN GHE=4A4;CYRILLIC SMALL LIGATURE EN GHE=4A5;CYRILLIC CAPITAL LETTER PE WITH MIDDLE HOOK=4A6;CYRILLIC SMALL LETTER PE WITH MIDDLE HOOK=4A7;CYRILLIC CAPITAL LETTER ABKHASIAN HA=4A8;CYRILLIC SMALL LETTER ABKHASIAN HA=4A9;CYRILLIC CAPITAL LETTER ES WITH DESCENDER=4AA;CYRILLIC SMALL LETTER ES WITH DESCENDER=4AB;CYRILLIC CAPITAL LETTER TE WITH DESCENDER=4AC;CYRILLIC SMALL LETTER TE WITH DESCENDER=4AD;CYRILLIC CAPITAL LETTER STRAIGHT U=4AE;CYRILLIC SMALL LETTER STRAIGHT U=4AF;CYRILLIC CAPITAL LETTER STRAIGHT U WITH STROKE=4B0;CYRILLIC SMALL LETTER STRAIGHT U WITH STROKE=4B1;CYRILLIC CAPITAL LETTER HA WITH DESCENDER=4B2;CYRILLIC SMALL LETTER HA WITH DESCENDER=4B3;CYRILLIC CAPITAL LIGATURE TE TSE=4B4;CYRILLIC SMALL LIGATURE TE TSE=4B5;CYRILLIC CAPITAL LETTER CHE WITH DESCENDER=4B6;CYRILLIC SMALL LETTER CHE WITH DESCENDER=4B7;CYRILLIC CAPITAL LETTER CHE WITH VERTICAL STROKE=4B8;CYRILLIC SMALL LETTER CHE WITH VERTICAL STROKE=4B9;CYRILLIC CAPITAL LETTER SHHA=4BA;CYRILLIC SMALL LETTER SHHA=4BB;CYRILLIC CAPITAL LETTER ABKHASIAN CHE=4BC;CYRILLIC SMALL LETTER ABKHASIAN CHE=4BD;CYRILLIC CAPITAL LETTER ABKHASIAN CHE WITH DESCENDER=4BE;CYRILLIC SMALL LETTER ABKHASIAN CHE WITH DESCENDER=4BF;CYRILLIC LETTER PALOCHKA=4C0;CYRILLIC CAPITAL LETTER ZHE WITH BREVE=4C1;CYRILLIC SMALL LETTER ZHE WITH BREVE=4C2;CYRILLIC CAPITAL LETTER KA WITH HOOK=4C3;CYRILLIC SMALL LETTER KA WITH HOOK=4C4;CYRILLIC CAPITAL LETTER EL WITH TAIL=4C5;CYRILLIC SMALL LETTER EL WITH TAIL=4C6;CYRILLIC CAPITAL LETTER EN WITH HOOK=4C7;CYRILLIC SMALL LETTER EN WITH HOOK=4C8;CYRILLIC CAPITAL LETTER EN WITH TAIL=4C9;CYRILLIC SMALL LETTER EN WITH TAIL=4CA;CYRILLIC CAPITAL LETTER KHAKASSIAN CHE=4CB;CYRILLIC SMALL LETTER KHAKASSIAN CHE=4CC;CYRILLIC CAPITAL LETTER EM WITH TAIL=4CD;CYRILLIC SMALL LETTER EM WITH TAIL=4CE;CYRILLIC SMALL LETTER PALOCHKA=4CF;CYRILLIC CAPITAL LETTER A WITH BREVE=4D0;CYRILLIC SMALL LETTER A WITH BREVE=4D1;CYRILLIC CAPITAL LETTER A WITH DIAERESIS=4D2;CYRILLIC SMALL LETTER A WITH DIAERESIS=4D3;CYRILLIC CAPITAL LIGATURE A IE=4D4;CYRILLIC SMALL LIGATURE A IE=4D5;CYRILLIC CAPITAL LETTER IE WITH BREVE=4D6;CYRILLIC SMALL LETTER IE WITH BREVE=4D7;CYRILLIC CAPITAL LETTER SCHWA=4D8;CYRILLIC SMALL LETTER SCHWA=4D9;CYRILLIC CAPITAL LETTER SCHWA WITH DIAERESIS=4DA;CYRILLIC SMALL LETTER SCHWA WITH DIAERESIS=4DB;CYRILLIC CAPITAL LETTER ZHE WITH DIAERESIS=4DC;CYRILLIC SMALL LETTER ZHE WITH DIAERESIS=4DD;CYRILLIC CAPITAL LETTER ZE WITH DIAERESIS=4DE;CYRILLIC SMALL LETTER ZE WITH DIAERESIS=4DF;CYRILLIC CAPITAL LETTER ABKHASIAN DZE=4E0;CYRILLIC SMALL LETTER ABKHASIAN DZE=4E1;CYRILLIC CAPITAL LETTER I WITH MACRON=4E2;CYRILLIC SMALL LETTER I WITH MACRON=4E3;CYRILLIC CAPITAL LETTER I WITH DIAERESIS=4E4;CYRILLIC SMALL LETTER I WITH DIAERESIS=4E5;CYRILLIC CAPITAL LETTER O WITH DIAERESIS=4E6;CYRILLIC SMALL LETTER O WITH DIAERESIS=4E7;CYRILLIC CAPITAL LETTER BARRED O=4E8;CYRILLIC SMALL LETTER BARRED O=4E9;CYRILLIC CAPITAL LETTER BARRED O WITH DIAERESIS=4EA;CYRILLIC SMALL LETTER BARRED O WITH DIAERESIS=4EB;CYRILLIC CAPITAL LETTER E WITH DIAERESIS=4EC;CYRILLIC SMALL LETTER E WITH DIAERESIS=4ED;CYRILLIC CAPITAL LETTER U WITH MACRON=4EE;CYRILLIC SMALL LETTER U WITH MACRON=4EF;CYRILLIC CAPITAL LETTER U WITH DIAERESIS=4F0;CYRILLIC SMALL LETTER U WITH DIAERESIS=4F1;CYRILLIC CAPITAL LETTER U WITH DOUBLE ACUTE=4F2;CYRILLIC SMALL LETTER U WITH DOUBLE ACUTE=4F3;CYRILLIC CAPITAL LETTER CHE WITH DIAERESIS=4F4;CYRILLIC SMALL LETTER CHE WITH DIAERESIS=4F5;CYRILLIC CAPITAL LETTER GHE WITH DESCENDER=4F6;CYRILLIC SMALL LETTER GHE WITH DESCENDER=4F7;CYRILLIC CAPITAL LETTER YERU WITH DIAERESIS=4F8;CYRILLIC SMALL LETTER YERU WITH DIAERESIS=4F9;CYRILLIC CAPITAL LETTER GHE WITH STROKE AND HOOK=4FA;CYRILLIC SMALL LETTER GHE WITH STROKE AND HOOK=4FB;CYRILLIC CAPITAL LETTER HA WITH HOOK=4FC;CYRILLIC SMALL LETTER HA WITH HOOK=4FD;CYRILLIC CAPITAL LETTER HA WITH STROKE=4FE;CYRILLIC SMALL LETTER HA WITH STROKE=4FF;EN QUAD=2000;EM QUAD=2001;EN SPACE=2002;EM SPACE=2003;THREE-PER-EM SPACE=2004;FOUR-PER-EM SPACE=2005;SIX-PER-EM SPACE=2006;FIGURE SPACE=2007;PUNCTUATION SPACE=2008;THIN SPACE=2009;HAIR SPACE=200A;ZERO WIDTH SPACE=200B;ZERO WIDTH NON-JOINER=200C;ZERO WIDTH JOINER=200D;LEFT-TO-RIGHT MARK=200E;RIGHT-TO-LEFT MARK=200F;HYPHEN=2010;NON-BREAKING HYPHEN=2011;FIGURE DASH=2012;EN DASH=2013;EM DASH=2014;HORIZONTAL BAR=2015;DOUBLE VERTICAL LINE=2016;DOUBLE LOW LINE=2017;LEFT SINGLE QUOTATION MARK=2018;RIGHT SINGLE QUOTATION MARK=2019;SINGLE LOW-9 QUOTATION MARK=201A;SINGLE HIGH-REVERSED-9 QUOTATION MARK=201B;LEFT DOUBLE QUOTATION MARK=201C;RIGHT DOUBLE QUOTATION MARK=201D;DOUBLE LOW-9 QUOTATION MARK=201E;DOUBLE HIGH-REVERSED-9 QUOTATION MARK=201F;DAGGER=2020;DOUBLE DAGGER=2021;BULLET=2022;TRIANGULAR BULLET=2023;ONE DOT LEADER=2024;TWO DOT LEADER=2025;HORIZONTAL ELLIPSIS=2026;HYPHENATION POINT=2027;LINE SEPARATOR=2028;PARAGRAPH SEPARATOR=2029;LEFT-TO-RIGHT EMBEDDING=202A;RIGHT-TO-LEFT EMBEDDING=202B;POP DIRECTIONAL FORMATTING=202C;LEFT-TO-RIGHT OVERRIDE=202D;RIGHT-TO-LEFT OVERRIDE=202E;NARROW NO-BREAK SPACE=202F;PER MILLE SIGN=2030;PER TEN THOUSAND SIGN=2031;PRIME=2032;DOUBLE PRIME=2033;TRIPLE PRIME=2034;REVERSED PRIME=2035;REVERSED DOUBLE PRIME=2036;REVERSED TRIPLE PRIME=2037;CARET=2038;SINGLE LEFT-POINTING ANGLE QUOTATION MARK=2039;SINGLE RIGHT-POINTING ANGLE QUOTATION MARK=203A;REFERENCE MARK=203B;DOUBLE EXCLAMATION MARK=203C;INTERROBANG=203D;OVERLINE=203E;UNDERTIE=203F;CHARACTER TIE=2040;CARET INSERTION POINT=2041;ASTERISM=2042;HYPHEN BULLET=2043;FRACTION SLASH=2044;LEFT SQUARE BRACKET WITH QUILL=2045;RIGHT SQUARE BRACKET WITH QUILL=2046;DOUBLE QUESTION MARK=2047;QUESTION EXCLAMATION MARK=2048;EXCLAMATION QUESTION MARK=2049;TIRONIAN SIGN ET=204A;REVERSED PILCROW SIGN=204B;BLACK LEFTWARDS BULLET=204C;BLACK RIGHTWARDS BULLET=204D;LOW ASTERISK=204E;REVERSED SEMICOLON=204F;CLOSE UP=2050;TWO ASTERISKS ALIGNED VERTICALLY=2051;COMMERCIAL MINUS SIGN=2052;SWUNG DASH=2053;INVERTED UNDERTIE=2054;FLOWER PUNCTUATION MARK=2055;THREE DOT PUNCTUATION=2056;QUADRUPLE PRIME=2057;FOUR DOT PUNCTUATION=2058;FIVE DOT PUNCTUATION=2059;TWO DOT PUNCTUATION=205A;FOUR DOT MARK=205B;DOTTED CROSS=205C;TRICOLON=205D;VERTICAL FOUR DOTS=205E;MEDIUM MATHEMATICAL SPACE=205F;WORD JOINER=2060;FUNCTION APPLICATION=2061;INVISIBLE TIMES=2062;INVISIBLE SEPARATOR=2063;INVISIBLE PLUS=2064;LEFT-TO-RIGHT ISOLATE=2066;RIGHT-TO-LEFT ISOLATE=2067;FIRST STRONG ISOLATE=2068;POP DIRECTIONAL ISOLATE=2069;INHIBIT SYMMETRIC SWAPPING=206A;ACTIVATE SYMMETRIC SWAPPING=206B;INHIBIT ARABIC FORM SHAPING=206C;ACTIVATE ARABIC FORM SHAPING=206D;NATIONAL DIGIT SHAPES=206E;NOMINAL DIGIT SHAPES=206F;EURO-CURRENCY SIGN=20A0;COLON SIGN=20A1;CRUZEIRO SIGN=20A2;FRENCH FRANC SIGN=20A3;LIRA SIGN=20A4;MILL SIGN=20A5;NAIRA SIGN=20A6;PESETA SIGN=20A7;RUPEE SIGN=20A8;WON SIGN=20A9;NEW SHEQEL SIGN=20AA;DONG SIGN=20AB;EURO SIGN=20AC;KIP SIGN=20AD;TUGRIK SIGN=20AE;DRACHMA SIGN=20AF;GERMAN PENNY SIGN=20B0;PESO SIGN=20B1;GUARANI SIGN=20B2;AUSTRAL SIGN=20B3;HRYVNIA SIGN=20B4;CEDI SIGN=20B5;LIVRE TOURNOIS SIGN=20B6;SPESMILO SIGN=20B7;TENGE SIGN=20B8;INDIAN RUPEE SIGN=20B9;TURKISH LIRA SIGN=20BA;NORDIC MARK SIGN=20BB;MANAT SIGN=20BC;RUBLE SIGN=20BD;LARI SIGN=20BE;BITCOIN SIGN=20BF;SOM SIGN=20C0;ACCOUNT OF=2100;ADDRESSED TO THE SUBJECT=2101;DOUBLE-STRUCK CAPITAL C=2102;DEGREE CELSIUS=2103;CENTRE LINE SYMBOL=2104;CARE OF=2105;CADA UNA=2106;EULER CONSTANT=2107;SCRUPLE=2108;DEGREE FAHRENHEIT=2109;SCRIPT SMALL G=210A;SCRIPT CAPITAL H=210B;BLACK-LETTER CAPITAL H=210C;DOUBLE-STRUCK CAPITAL H=210D;PLANCK CONSTANT=210E;PLANCK CONSTANT OVER TWO PI=210F;SCRIPT CAPITAL I=2110;BLACK-LETTER CAPITAL I=2111;SCRIPT CAPITAL L=2112;SCRIPT SMALL L=2113;L B BAR SYMBOL=2114;DOUBLE-STRUCK CAPITAL N=2115;NUMERO SIGN=2116;SOUND RECORDING COPYRIGHT=2117;SCRIPT CAPITAL P=2118;DOUBLE-STRUCK CAPITAL P=2119;DOUBLE-STRUCK CAPITAL Q=211A;SCRIPT CAPITAL R=211B;BLACK-LETTER CAPITAL R=211C;DOUBLE-STRUCK CAPITAL R=211D;PRESCRIPTION TAKE=211E;RESPONSE=211F;SERVICE MARK=2120;TELEPHONE SIGN=2121;TRADE MARK SIGN=2122;VERSICLE=2123;DOUBLE-STRUCK CAPITAL Z=2124;OUNCE SIGN=2125;OHM SIGN=2126;INVERTED OHM SIGN=2127;BLACK-LETTER CAPITAL Z=2128;TURNED GREEK SMALL LETTER IOTA=2129;KELVIN SIGN=212A;ANGSTROM SIGN=212B;SCRIPT CAPITAL B=212C;BLACK-LETTER CAPITAL C=212D;ESTIMATED SYMBOL=212E;SCRIPT SMALL E=212F;SCRIPT CAPITAL E=2130;SCRIPT CAPITAL F=2131;TURNED CAPITAL F=2132;SCRIPT CAPITAL M=2133;SCRIPT SMALL O=2134;ALEF SYMBOL=2135;BET SYMBOL=2136;GIMEL SYMBOL=2137;DALET SYMBOL=2138;INFORMATION SOURCE=2139;ROTATED CAPITAL Q=213A;FACSIMILE SIGN=213B;DOUBLE-STRUCK SMALL PI=213C;DOUBLE-STRUCK SMALL GAMMA=213D;DOUBLE-STRUCK CAPITAL GAMMA=213E;DOUBLE-STRUCK CAPITAL PI=213F;DOUBLE-STRUCK N-ARY SUMMATION=2140;TURNED SANS-SERIF CAPITAL G=2141;TURNED SANS-SERIF CAPITAL L=2142;REVERSED SANS-SERIF CAPITAL L=2143;TURNED SANS-SERIF CAPITAL Y=2144;DOUBLE-STRUCK ITALIC CAPITAL D=2145;DOUBLE-STRUCK ITALIC SMALL D=2146;DOUBLE-STRUCK ITALIC SMALL E=2147;DOUBLE-STRUCK ITALIC SMALL I=2148;DOUBLE-STRUCK ITALIC SMALL J=2149;PROPERTY LINE=214A;TURNED AMPERSAND=214B;PER SIGN=214C;AKTIESELSKAB=214D;TURNED SMALL F=214E;SYMBOL FOR SAMARITAN SOURCE=214F;LEFTWARDS ARROW=2190;UPWARDS ARROW=2191;RIGHTWARDS ARROW=2192;DOWNWARDS ARROW=2193;LEFT RIGHT ARROW=2194;UP DOWN ARROW=2195;NORTH WEST ARROW=2196;NORTH EAST ARROW=2197;SOUTH EAST ARROW=2198;SOUTH WEST ARROW=2199;LEFTWARDS ARROW WITH STROKE=219A;RIGHTWARDS ARROW WITH STROKE=219B;LEFTWARDS WAVE ARROW=219C;RIGHTWARDS WAVE ARROW=219D;LEFTWARDS TWO HEADED ARROW=219E;UPWARDS TWO HEADED ARROW=219F;RIGHTWARDS TWO HEADED ARROW=21A0;DOWNWARDS TWO HEADED ARROW=21A1;LEFTWARDS ARROW WITH TAIL=21A2;RIGHTWARDS ARROW WITH TAIL=21A3;LEFTWARDS ARROW FROM BAR=21A4;UPWARDS ARROW FROM BAR=21A5;RIGHTWARDS ARROW FROM BAR=21A6;DOWNWARDS ARROW FROM BAR=21A7;UP DOWN ARROW WITH BASE=21A8;LEFTWARDS ARROW WITH HOOK=21A9;RIGHTWARDS ARROW WITH HOOK=21AA;LEFTWARDS ARROW WITH LOOP=21AB;RIGHTWARDS ARROW WITH LOOP=21AC;LEFT RIGHT WAVE ARROW=21AD;LEFT RIGHT ARROW WITH STROKE=21AE;DOWNWARDS ZIGZAG ARROW=21AF;UPWARDS ARROW WITH TIP LEFTWARDS=21B0;UPWARDS ARROW WITH TIP RIGHTWARDS=21B1;DOWNWARDS ARROW WITH TIP LEFTWARDS=21B2;DOWNWARDS ARROW WITH TIP RIGHTWARDS=21B3;RIGHTWARDS ARROW WITH CORNER DOWNWARDS=21B4;DOWNWARDS ARROW WITH CORNER LEFTWARDS=21B5;ANTICLOCKWISE TOP SEMICIRCLE ARROW=21B6;CLOCKWISE TOP SEMICIRCLE ARROW=21B7;NORTH WEST ARROW TO LONG BAR=21B8;LEFTWARDS ARROW TO BAR OVER RIGHTWARDS ARROW TO BAR=21B9;ANTICLOCKWISE OPEN CIRCLE ARROW=21BA;CLOCKWISE OPEN CIRCLE ARROW=21BB;LEFTWARDS HARPOON WITH BARB UPWARDS=21BC;LEFTWARDS HARPOON WITH BARB DOWNWARDS=21BD;UPWARDS HARPOON WITH BARB RIGHTWARDS=21BE;UPWARDS HARPOON WITH BARB LEFTWARDS=21BF;RIGHTWARDS HARPOON WITH BARB UPWARDS=21C0;RIGHTWARDS HARPOON WITH BARB DOWNWARDS=21C1;DOWNWARDS HARPOON WITH BARB RIGHTWARDS=21C2;DOWNWARDS HARPOON WITH BARB LEFTWARDS=21C3;RIGHTWARDS ARROW OVER LEFTWARDS ARROW=21C4;UPWARDS ARROW LEFTWARDS OF DOWNWARDS ARROW=21C5;LEFTWARDS ARROW OVER RIGHTWARDS ARROW=21C6;LEFTWARDS PAIRED ARROWS=21C7;UPWARDS PAIRED ARROWS=21C8;RIGHTWARDS PAIRED ARROWS=21C9;DOWNWARDS PAIRED ARROWS=21CA;LEFTWARDS HARPOON OVER RIGHTWARDS HARPOON=21CB;RIGHTWARDS HARPOON OVER LEFTWARDS HARPOON=21CC;LEFTWARDS DOUBLE ARROW WITH STROKE=21CD;LEFT RIGHT DOUBLE ARROW WITH STROKE=21CE;RIGHTWARDS DOUBLE ARROW WITH STROKE=21CF;LEFTWARDS DOUBLE ARROW=21D0;UPWARDS DOUBLE ARROW=21D1;RIGHTWARDS DOUBLE ARROW=21D2;DOWNWARDS DOUBLE ARROW=21D3;LEFT RIGHT DOUBLE ARROW=21D4;UP DOWN DOUBLE ARROW=21D5;NORTH WEST DOUBLE ARROW=21D6;NORTH EAST DOUBLE ARROW=21D7;SOUTH EAST DOUBLE ARROW=21D8;SOUTH WEST DOUBLE ARROW=21D9;LEFTWARDS TRIPLE ARROW=21DA;RIGHTWARDS TRIPLE ARROW=21DB;LEFTWARDS SQUIGGLE ARROW=21DC;RIGHTWARDS SQUIGGLE ARROW=21DD;UPWARDS ARROW WITH DOUBLE STROKE=21DE;DOWNWARDS ARROW WITH DOUBLE STROKE=21DF;LEFTWARDS DASHED ARROW=21E0;UPWARDS DASHED ARROW=21E1;RIGHTWARDS DASHED ARROW=21E2;DOWNWARDS DASHED ARROW=21E3;LEFTWARDS ARROW TO BAR=21E4;RIGHTWARDS ARROW TO BAR=21E5;LEFTWARDS WHITE ARROW=21E6;UPWARDS WHITE ARROW=21E7;RIGHTWARDS WHITE ARROW=21E8;DOWNWARDS WHITE ARROW=21E9;UPWARDS WHITE ARROW FROM BAR=21EA;UPWARDS WHITE ARROW ON PEDESTAL=21EB;UPWARDS WHITE ARROW ON PEDESTAL WITH HORIZONTAL BAR=21EC;UPWARDS WHITE ARROW ON PEDESTAL WITH VERTICAL BAR=21ED;UPWARDS WHITE DOUBLE ARROW=21EE;UPWARDS WHITE DOUBLE ARROW ON PEDESTAL=21EF;RIGHTWARDS WHITE ARROW FROM WALL=21F0;NORTH WEST ARROW TO CORNER=21F1;SOUTH EAST ARROW TO CORNER=21F2;UP DOWN WHITE ARROW=21F3;RIGHT ARROW WITH SMALL CIRCLE=21F4;DOWNWARDS ARROW LEFTWARDS OF UPWARDS ARROW=21F5;THREE RIGHTWARDS ARROWS=21F6;LEFTWARDS ARROW WITH VERTICAL STROKE=21F7;RIGHTWARDS ARROW WITH VERTICAL STROKE=21F8;LEFT RIGHT ARROW WITH VERTICAL STROKE=21F9;LEFTWARDS ARROW WITH DOUBLE VERTICAL STROKE=21FA;RIGHTWARDS ARROW WITH DOUBLE VERTICAL STROKE=21FB;LEFT RIGHT ARROW WITH DOUBLE VERTICAL STROKE=21FC;LEFTWARDS OPEN-HEADED ARROW=21FD;RIGHTWARDS OPEN-HEADED ARROW=21FE;LEFT RIGHT OPEN-HEADED ARROW=21FF;FOR ALL=2200;COMPLEMENT=2201;PARTIAL DIFFERENTIAL=2202;THERE EXISTS=2203;THERE DOES NOT EXIST=2204;EMPTY SET=2205;INCREMENT=2206;NABLA=2207;ELEMENT OF=2208;NOT AN ELEMENT OF=2209;SMALL ELEMENT OF=220A;CONTAINS AS MEMBER=220B;DOES NOT CONTAIN AS MEMBER=220C;SMALL CONTAINS AS MEMBER=220D;END OF PROOF=220E;N-ARY PRODUCT=220F;N-ARY COPRODUCT=2210;N-ARY SUMMATION=2211;MINUS SIGN=2212;MINUS-OR-PLUS SIGN=2213;DOT PLUS=2214;DIVISION SLASH=2215;SET MINUS=2216;ASTERISK OPERATOR=2217;RING OPERATOR=2218;BULLET OPERATOR=2219;SQUARE ROOT=221A;CUBE ROOT=221B;FOURTH ROOT=221C;PROPORTIONAL TO=221D;INFINITY=221E;RIGHT ANGLE=221F;ANGLE=2220;MEASURED ANGLE=2221;SPHERICAL ANGLE=2222;DIVIDES=2223;DOES NOT DIVIDE=2224;PARALLEL TO=2225;NOT PARALLEL TO=2226;LOGICAL AND=2227;LOGICAL OR=2228;INTERSECTION=2229;UNION=222A;INTEGRAL=222B;DOUBLE INTEGRAL=222C;TRIPLE INTEGRAL=222D;CONTOUR INTEGRAL=222E;SURFACE INTEGRAL=222F;VOLUME INTEGRAL=2230;CLOCKWISE INTEGRAL=2231;CLOCKWISE CONTOUR INTEGRAL=2232;ANTICLOCKWISE CONTOUR INTEGRAL=2233;THEREFORE=2234;BECAUSE=2235;RATIO=2236;PROPORTION=2237;DOT MINUS=2238;EXCESS=2239;GEOMETRIC PROPORTION=223A;HOMOTHETIC=223B;TILDE OPERATOR=223C;REVERSED TILDE=223D;INVERTED LAZY S=223E;SINE WAVE=223F;WREATH PRODUCT=2240;NOT TILDE=2241;MINUS TILDE=2242;ASYMPTOTICALLY EQUAL TO=2243;NOT ASYMPTOTICALLY EQUAL TO=2244;APPROXIMATELY EQUAL TO=2245;APPROXIMATELY BUT NOT ACTUALLY EQUAL TO=2246;NEITHER APPROXIMATELY NOR ACTUALLY EQUAL TO=2247;ALMOST EQUAL TO=2248;NOT ALMOST EQUAL TO=2249;ALMOST EQUAL OR EQUAL TO=224A;TRIPLE TILDE=224B;ALL EQUAL TO=224C;EQUIVALENT TO=224D;GEOMETRICALLY EQUIVALENT TO=224E;DIFFERENCE BETWEEN=224F;APPROACHES THE LIMIT=2250;GEOMETRICALLY EQUAL TO=2251;APPROXIMATELY EQUAL TO OR THE IMAGE OF=2252;IMAGE OF OR APPROXIMATELY EQUAL TO=2253;COLON EQUALS=2254;EQUALS COLON=2255;RING IN EQUAL TO=2256;RING EQUAL TO=2257;CORRESPONDS TO=2258;ESTIMATES=2259;EQUIANGULAR TO=225A;STAR EQUALS=225B;DELTA EQUAL TO=225C;EQUAL TO BY DEFINITION=225D;MEASURED BY=225E;QUESTIONED EQUAL TO=225F;NOT EQUAL TO=2260;IDENTICAL TO=2261;NOT IDENTICAL TO=2262;STRICTLY EQUIVALENT TO=2263;LESS-THAN OR EQUAL TO=2264;GREATER-THAN OR EQUAL TO=2265;LESS-THAN OVER EQUAL TO=2266;GREATER-THAN OVER EQUAL TO=2267;LESS-THAN BUT NOT EQUAL TO=2268;GREATER-THAN BUT NOT EQUAL TO=2269;MUCH LESS-THAN=226A;MUCH GREATER-THAN=226B;BETWEEN=226C;NOT EQUIVALENT TO=226D;NOT LESS-THAN=226E;NOT GREATER-THAN=226F;NEITHER LESS-THAN NOR EQUAL TO=2270;NEITHER GREATER-THAN NOR EQUAL TO=2271;LESS-THAN OR EQUIVALENT TO=2272;GREATER-THAN OR EQUIVALENT TO=2273;NEITHER LESS-THAN NOR EQUIVALENT TO=2274;NEITHER GREATER-THAN NOR EQUIVALENT TO=2275;LESS-THAN OR GREATER-THAN=2276;GREATER-THAN OR LESS-THAN=2277;NEITHER LESS-THAN NOR GREATER-THAN=2278;NEITHER GREATER-THAN NOR LESS-THAN=2279;PRECEDES=227A;SUCCEEDS=227B;PRECEDES OR EQUAL TO=227C;SUCCEEDS OR EQUAL TO=227D;PRECEDES OR EQUIVALENT TO=227E;SUCCEEDS OR EQUIVALENT TO=227F;DOES NOT PRECEDE=2280;DOES NOT SUCCEED=2281;SUBSET OF=2282;SUPERSET OF=2283;NOT A SUBSET OF=2284;NOT A SUPERSET OF=2285;SUBSET OF OR EQUAL TO=2286;SUPERSET OF OR EQUAL TO=2287;NEITHER A SUBSET OF NOR EQUAL TO=2288;NEITHER A SUPERSET OF NOR EQUAL TO=2289;SUBSET OF WITH NOT EQUAL TO=228A;SUPERSET OF WITH NOT EQUAL TO=228B;MULTISET=228C;MULTISET MULTIPLICATION=228D;MULTISET UNION=228E;SQUARE IMAGE OF=228F;SQUARE ORIGINAL OF=2290;SQUARE IMAGE OF OR EQUAL TO=2291;SQUARE ORIGINAL OF OR EQUAL TO=2292;SQUARE CAP=2293;SQUARE CUP=2294;CIRCLED PLUS=2295;CIRCLED MINUS=2296;CIRCLED TIMES=2297;CIRCLED DIVISION SLASH=2298;CIRCLED DOT OPERATOR=2299;CIRCLED RING OPERATOR=229A;CIRCLED ASTERISK OPERATOR=229B;CIRCLED EQUALS=229C;CIRCLED DASH=229D;SQUARED PLUS=229E;SQUARED MINUS=229F;SQUARED TIMES=22A0;SQUARED DOT OPERATOR=22A1;RIGHT TACK=22A2;LEFT TACK=22A3;DOWN TACK=22A4;UP TACK=22A5;ASSERTION=22A6;MODELS=22A7;TRUE=22A8;FORCES=22A9;TRIPLE VERTICAL BAR RIGHT TURNSTILE=22AA;DOUBLE VERTICAL BAR DOUBLE RIGHT TURNSTILE=22AB;DOES NOT PROVE=22AC;NOT TRUE=22AD;DOES NOT FORCE=22AE;NEGATED DOUBLE VERTICAL BAR DOUBLE RIGHT TURNSTILE=22AF;PRECEDES UNDER RELATION=22B0;SUCCEEDS UNDER RELATION=22B1;NORMAL SUBGROUP OF=22B2;CONTAINS AS NORMAL SUBGROUP=22B3;NORMAL SUBGROUP OF OR EQUAL TO=22B4;CONTAINS AS NORMAL SUBGROUP OR EQUAL TO=22B5;ORIGINAL OF=22B6;IMAGE OF=22B7;MULTIMAP=22B8;HERMITIAN CONJUGATE MATRIX=22B9;INTERCALATE=22BA;XOR=22BB;NAND=22BC;NOR=22BD;RIGHT ANGLE WITH ARC=22BE;RIGHT TRIANGLE=22BF;N-ARY LOGICAL AND=22C0;N-ARY LOGICAL OR=22C1;N-ARY INTERSECTION=22C2;N-ARY UNION=22C3;DIAMOND OPERATOR=22C4;DOT OPERATOR=22C5;STAR OPERATOR=22C6;DIVISION TIMES=22C7;BOWTIE=22C8;LEFT NORMAL FACTOR SEMIDIRECT PRODUCT=22C9;RIGHT NORMAL FACTOR SEMIDIRECT PRODUCT=22CA;LEFT SEMIDIRECT PRODUCT=22CB;RIGHT SEMIDIRECT PRODUCT=22CC;REVERSED TILDE EQUALS=22CD;CURLY LOGICAL OR=22CE;CURLY LOGICAL AND=22CF;DOUBLE SUBSET=22D0;DOUBLE SUPERSET=22D1;DOUBLE INTERSECTION=22D2;DOUBLE UNION=22D3;PITCHFORK=22D4;EQUAL AND PARALLEL TO=22D5;LESS-THAN WITH DOT=22D6;GREATER-THAN WITH DOT=22D7;VERY MUCH LESS-THAN=22D8;VERY MUCH GREATER-THAN=22D9;LESS-THAN EQUAL TO OR GREATER-THAN=22DA;GREATER-THAN EQUAL TO OR LESS-THAN=22DB;EQUAL TO OR LESS-THAN=22DC;EQUAL TO OR GREATER-THAN=22DD;EQUAL TO OR PRECEDES=22DE;EQUAL TO OR SUCCEEDS=22DF;DOES NOT PRECEDE OR EQUAL=22E0;DOES NOT SUCCEED OR EQUAL=22E1;NOT SQUARE IMAGE OF OR EQUAL TO=22E2;NOT SQUARE ORIGINAL OF OR EQUAL TO=22E3;SQUARE IMAGE OF OR NOT EQUAL TO=22E4;SQUARE ORIGINAL OF OR NOT EQUAL TO=22E5;LESS-THAN BUT NOT EQUIVALENT TO=22E6;GREATER-THAN BUT NOT EQUIVALENT TO=22E7;PRECEDES BUT NOT EQUIVALENT TO=22E8;SUCCEEDS BUT NOT EQUIVALENT TO=22E9;NOT NORMAL SUBGROUP OF=22EA;DOES NOT CONTAIN AS NORMAL SUBGROUP=22EB;NOT NORMAL SUBGROUP OF OR EQUAL TO=22EC;DOES NOT CONTAIN AS NORMAL SUBGROUP OR EQUAL=22ED;VERTICAL ELLIPSIS=22EE;MIDLINE HORIZONTAL ELLIPSIS=22EF;UP RIGHT DIAGONAL ELLIPSIS=22F0;DOWN RIGHT DIAGONAL ELLIPSIS=22F1;ELEMENT OF WITH LONG HORIZONTAL STROKE=22F2;ELEMENT OF WITH VERTICAL BAR AT END OF HORIZONTAL STROKE=22F3;SMALL ELEMENT OF WITH VERTICAL BAR AT END OF HORIZONTAL STROKE=22F4;ELEMENT OF WITH DOT ABOVE=22F5;ELEMENT OF WITH OVERBAR=22F6;SMALL ELEMENT OF WITH OVERBAR=22F7;ELEMENT OF WITH UNDERBAR=22F8;ELEMENT OF WITH TWO HORIZONTAL STROKES=22F9;CONTAINS WITH LONG HORIZONTAL STROKE=22FA;CONTAINS WITH VERTICAL BAR AT END OF HORIZONTAL STROKE=22FB;SMALL CONTAINS WITH VERTICAL BAR AT END OF HORIZONTAL STROKE=22FC;CONTAINS WITH OVERBAR=22FD;SMALL CONTAINS WITH OVERBAR=22FE;Z NOTATION BAG MEMBERSHIP=22FF;BOX DRAWINGS LIGHT HORIZONTAL=2500;BOX DRAWINGS HEAVY HORIZONTAL=2501;BOX DRAWINGS LIGHT VERTICAL=2502;BOX DRAWINGS HEAVY VERTICAL=2503;BOX DRAWINGS LIGHT TRIPLE DASH HORIZONTAL=2504;BOX DRAWINGS HEAVY TRIPLE DASH HORIZONTAL=2505;BOX DRAWINGS LIGHT TRIPLE DASH VERTICAL=2506;BOX DRAWINGS HEAVY TRIPLE DASH VERTICAL=2507;BOX DRAWINGS LIGHT QUADRUPLE DASH HORIZONTAL=2508;BOX DRAWINGS HEAVY QUADRUPLE DASH HORIZONTAL=2509;BOX DRAWINGS LIGHT QUADRUPLE DASH VERTICAL=250A;BOX DRAWINGS HEAVY QUADRUPLE DASH VERTICAL=250B;BOX DRAWINGS LIGHT DOWN AND RIGHT=250C;BOX DRAWINGS DOWN LIGHT AND RIGHT HEAVY=250D;BOX DRAWINGS DOWN HEAVY AND RIGHT LIGHT=250E;BOX DRAWINGS HEAVY DOWN AND RIGHT=250F;BOX DRAWINGS LIGHT DOWN AND LEFT=2510;BOX DRAWINGS DOWN LIGHT AND LEFT HEAVY=2511;BOX DRAWINGS DOWN HEAVY AND LEFT LIGHT=2512;BOX DRAWINGS HEAVY DOWN AND LEFT=2513;BOX DRAWINGS LIGHT UP AND RIGHT=2514;BOX DRAWINGS UP LIGHT AND RIGHT HEAVY=2515;BOX DRAWINGS UP HEAVY AND RIGHT LIGHT=2516;BOX DRAWINGS HEAVY UP AND RIGHT=2517;BOX DRAWINGS LIGHT UP AND LEFT=2518;BOX DRAWINGS UP LIGHT AND LEFT HEAVY=2519;BOX DRAWINGS UP HEAVY AND LEFT LIGHT=251A;BOX DRAWINGS HEAVY UP AND LEFT=251B;BOX DRAWINGS LIGHT VERTICAL AND RIGHT=251C;BOX DRAWINGS VERTICAL LIGHT AND RIGHT HEAVY=251D;BOX DRAWINGS UP HEAVY AND RIGHT DOWN LIGHT=251E;BOX DRAWINGS DOWN HEAVY AND RIGHT UP LIGHT=251F;BOX DRAWINGS VERTICAL HEAVY AND RIGHT LIGHT=2520;BOX DRAWINGS DOWN LIGHT AND RIGHT UP HEAVY=2521;BOX DRAWINGS UP LIGHT AND RIGHT DOWN HEAVY=2522;BOX DRAWINGS HEAVY VERTICAL AND RIGHT=2523;BOX DRAWINGS LIGHT VERTICAL AND LEFT=2524;BOX DRAWINGS VERTICAL LIGHT AND LEFT HEAVY=2525;BOX DRAWINGS UP HEAVY AND LEFT DOWN LIGHT=2526;BOX DRAWINGS DOWN HEAVY AND LEFT UP LIGHT=2527;BOX DRAWINGS VERTICAL HEAVY AND LEFT LIGHT=2528;BOX DRAWINGS DOWN LIGHT AND LEFT UP HEAVY=2529;BOX DRAWINGS UP LIGHT AND LEFT DOWN HEAVY=252A;BOX DRAWINGS HEAVY VERTICAL AND LEFT=252B;BOX DRAWINGS LIGHT DOWN AND HORIZONTAL=252C;BOX DRAWINGS LEFT HEAVY AND RIGHT DOWN LIGHT=252D;BOX DRAWINGS RIGHT HEAVY AND LEFT DOWN LIGHT=252E;BOX DRAWINGS DOWN LIGHT AND HORIZONTAL HEAVY=252F;BOX DRAWINGS DOWN HEAVY AND HORIZONTAL LIGHT=2530;BOX DRAWINGS RIGHT LIGHT AND LEFT DOWN HEAVY=2531;BOX DRAWINGS LEFT LIGHT AND RIGHT DOWN HEAVY=2532;BOX DRAWINGS HEAVY DOWN AND HORIZONTAL=2533;BOX DRAWINGS LIGHT UP AND HORIZONTAL=2534;BOX DRAWINGS LEFT HEAVY AND RIGHT UP LIGHT=2535;BOX DRAWINGS RIGHT HEAVY AND LEFT UP LIGHT=2536;BOX DRAWINGS UP LIGHT AND HORIZONTAL HEAVY=2537;BOX DRAWINGS UP HEAVY AND HORIZONTAL LIGHT=2538;BOX DRAWINGS RIGHT LIGHT AND LEFT UP HEAVY=2539;BOX DRAWINGS LEFT LIGHT AND RIGHT UP HEAVY=253A;BOX DRAWINGS HEAVY UP AND HORIZONTAL=253B;BOX DRAWINGS LIGHT VERTICAL AND HORIZONTAL=253C;BOX DRAWINGS LEFT HEAVY AND RIGHT VERTICAL LIGHT=253D;BOX DRAWINGS RIGHT HEAVY AND LEFT VERTICAL LIGHT=253E;BOX DRAWINGS VERTICAL LIGHT AND HORIZONTAL HEAVY=253F;BOX DRAWINGS UP HEAVY AND DOWN HORIZONTAL LIGHT=2540;BOX DRAWINGS DOWN HEAVY AND UP HORIZONTAL LIGHT=2541;BOX DRAWINGS VERTICAL HEAVY AND HORIZONTAL LIGHT=2542;BOX DRAWINGS LEFT UP HEAVY AND RIGHT DOWN LIGHT=2543;BOX DRAWINGS RIGHT UP HEAVY AND LEFT DOWN LIGHT=2544;BOX DRAWINGS LEFT DOWN HEAVY AND RIGHT UP LIGHT=2545;BOX DRAWINGS RIGHT DOWN HEAVY AND LEFT UP LIGHT=2546;BOX DRAWINGS DOWN LIGHT AND UP HORIZONTAL HEAVY=2547;BOX DRAWINGS UP LIGHT AND DOWN HORIZONTAL HEAVY=2548;BOX DRAWINGS RIGHT LIGHT AND LEFT VERTICAL HEAVY=2549;BOX DRAWINGS LEFT LIGHT AND RIGHT VERTICAL HEAVY=254A;BOX DRAWINGS HEAVY VERTICAL AND HORIZONTAL=254B;BOX DRAWINGS LIGHT DOUBLE DASH HORIZONTAL=254C;BOX DRAWINGS HEAVY DOUBLE DASH HORIZONTAL=254D;BOX DRAWINGS LIGHT DOUBLE DASH VERTICAL=254E;BOX DRAWINGS HEAVY DOUBLE DASH VERTICAL=254F;BOX DRAWINGS DOUBLE HORIZONTAL=2550;BOX DRAWINGS DOUBLE VERTICAL=2551;BOX DRAWINGS DOWN SINGLE AND RIGHT DOUBLE=2552;BOX DRAWINGS DOWN DOUBLE AND RIGHT SINGLE=2553;BOX DRAWINGS DOUBLE DOWN AND RIGHT=2554;BOX DRAWINGS DOWN SINGLE AND LEFT DOUBLE=2555;BOX DRAWINGS DOWN DOUBLE AND LEFT SINGLE=2556;BOX DRAWINGS DOUBLE DOWN AND LEFT=2557;BOX DRAWINGS UP SINGLE AND RIGHT DOUBLE=2558;BOX DRAWINGS UP DOUBLE AND RIGHT SINGLE=2559;BOX DRAWINGS DOUBLE UP AND RIGHT=255A;BOX DRAWINGS UP SINGLE AND LEFT DOUBLE=255B;BOX DRAWINGS UP DOUBLE AND LEFT SINGLE=255C;BOX DRAWINGS DOUBLE UP AND LEFT=255D;BOX DRAWINGS VERTICAL SINGLE AND RIGHT DOUBLE=255E;BOX DRAWINGS VERTICAL DOUBLE AND RIGHT SINGLE=255F;BOX DRAWINGS DOUBLE VERTICAL AND RIGHT=2560;BOX DRAWINGS VERTICAL SINGLE AND LEFT DOUBLE=2561;BOX DRAWINGS VERTICAL DOUBLE AND LEFT SINGLE=2562;BOX DRAWINGS DOUBLE VERTICAL AND LEFT=2563;BOX DRAWINGS DOWN SINGLE AND HORIZONTAL DOUBLE=2564;BOX DRAWINGS DOWN DOUBLE AND HORIZONTAL SINGLE=2565;BOX DRAWINGS DOUBLE DOWN AND HORIZONTAL=2566;BOX DRAWINGS UP SINGLE AND HORIZONTAL DOUBLE=2567;BOX DRAWINGS UP DOUBLE AND HORIZONTAL SINGLE=2568;BOX DRAWINGS DOUBLE UP AND HORIZONTAL=2569;BOX DRAWINGS VERTICAL SINGLE AND HORIZONTAL DOUBLE=256A;BOX DRAWINGS VERTICAL DOUBLE AND HORIZONTAL SINGLE=256B;BOX DRAWINGS DOUBLE VERTICAL AND HORIZONTAL=256C;BOX DRAWINGS LIGHT ARC DOWN AND RIGHT=256D;BOX DRAWINGS LIGHT ARC DOWN AND LEFT=256E;BOX DRAWINGS LIGHT ARC UP AND LEFT=256F;BOX DRAWINGS LIGHT ARC UP AND RIGHT=2570;BOX DRAWINGS LIGHT DIAGONAL UPPER RIGHT TO LOWER LEFT=2571;BOX DRAWINGS LIGHT DIAGONAL UPPER LEFT TO LOWER RIGHT=2572;BOX DRAWINGS LIGHT DIAGONAL CROSS=2573;BOX DRAWINGS LIGHT LEFT=2574;BOX DRAWINGS LIGHT UP=2575;BOX DRAWINGS LIGHT RIGHT=2576;BOX DRAWINGS LIGHT DOWN=2577;BOX DRAWINGS HEAVY LEFT=2578;BOX DRAWINGS HEAVY UP=2579;BOX DRAWINGS HEAVY RIGHT=257A;BOX DRAWINGS HEAVY DOWN=257B;BOX DRAWINGS LIGHT LEFT AND HEAVY RIGHT=257C;BOX DRAWINGS LIGHT UP AND HEAVY DOWN=257D;BOX DRAWINGS HEAVY LEFT AND LIGHT RIGHT=257E;BOX DRAWINGS HEAVY UP AND LIGHT DOWN=257F;UPPER HALF BLOCK=2580;LOWER ONE EIGHTH BLOCK=2581;LOWER ONE QUARTER BLOCK=2582;LOWER THREE EIGHTHS BLOCK=2583;LOWER HALF BLOCK=2584;LOWER FIVE EIGHTHS BLOCK=2585;LOWER THREE QUARTERS BLOCK=2586;LOWER SEVEN EIGHTHS BLOCK=2587;FULL BLOCK=2588;LEFT SEVEN EIGHTHS BLOCK=2589;LEFT THREE QUARTERS BLOCK=258A;LEFT FIVE EIGHTHS BLOCK=258B;LEFT HALF BLOCK=258C;LEFT THREE EIGHTHS BLOCK=258D;LEFT ONE QUARTER BLOCK=258E;LEFT ONE EIGHTH BLOCK=258F;RIGHT HALF BLOCK=2590;LIGHT SHADE=2591;MEDIUM SHADE=2592;DARK SHADE=2593;UPPER ONE EIGHTH BLOCK=2594;RIGHT ONE EIGHTH BLOCK=2595;QUADRANT LOWER LEFT=2596;QUADRANT LOWER RIGHT=2597;QUADRANT UPPER LEFT=2598;QUADRANT UPPER LEFT AND LOWER LEFT AND LOWER RIGHT=2599;QUADRANT UPPER LEFT AND LOWER RIGHT=259A;QUADRANT UPPER LEFT AND UPPER RIGHT AND LOWER LEFT=259B;QUADRANT UPPER LEFT AND UPPER RIGHT AND LOWER RIGHT=259C;QUADRANT UPPER RIGHT=259D;QUADRANT UPPER RIGHT AND LOWER LEFT=259E;QUADRANT UPPER RIGHT AND LOWER LEFT AND LOWER RIGHT=259F;BLACK SQUARE=25A0;WHITE SQUARE=25A1;WHITE SQUARE WITH ROUNDED CORNERS=25A2;WHITE SQUARE CONTAINING BLACK SMALL SQUARE=25A3;SQUARE WITH HORIZONTAL FILL=25A4;SQUARE WITH VERTICAL FILL=25A5;SQUARE WITH ORTHOGONAL CROSSHATCH FILL=25A6;SQUARE WITH UPPER LEFT TO LOWER RIGHT FILL=25A7;SQUARE WITH UPPER RIGHT TO LOWER LEFT FILL=25A8;SQUARE WITH DIAGONAL CROSSHATCH FILL=25A9;BLACK SMALL SQUARE=25AA;WHITE SMALL SQUARE=25AB;BLACK RECTANGLE=25AC;WHITE RECTANGLE=25AD;BLACK VERTICAL RECTANGLE=25AE;WHITE VERTICAL RECTANGLE=25AF;BLACK PARALLELOGRAM=25B0;WHITE PARALLELOGRAM=25B1;BLACK UP-POINTING TRIANGLE=25B2;WHITE UP-POINTING TRIANGLE=25B3;BLACK UP-POINTING SMALL TRIANGLE=25B4;WHITE UP-POINTING SMALL TRIANGLE=25B5;BLACK RIGHT-POINTING TRIANGLE=25B6;WHITE RIGHT-POINTING TRIANGLE=25B7;BLACK RIGHT-POINTING SMALL TRIANGLE=25B8;WHITE RIGHT-POINTING SMALL TRIANGLE=25B9;BLACK RIGHT-POINTING POINTER=25BA;WHITE RIGHT-POINTING POINTER=25BB;BLACK DOWN-POINTING TRIANGLE=25BC;WHITE DOWN-POINTING TRIANGLE=25BD;BLACK DOWN-POINTING SMALL TRIANGLE=25BE;WHITE DOWN-POINTING SMALL TRIANGLE=25BF;BLACK LEFT-POINTING TRIANGLE=25C0;WHITE LEFT-POINTING TRIANGLE=25C1;BLACK LEFT-POINTING SMALL TRIANGLE=25C2;WHITE LEFT-POINTING SMALL TRIANGLE=25C3;BLACK LEFT-POINTING POINTER=25C4;WHITE LEFT-POINTING POINTER=25C5;BLACK DIAMOND=25C6;WHITE DIAMOND=25C7;WHITE DIAMOND CONTAINING BLACK SMALL DIAMOND=25C8;FISHEYE=25C9;LOZENGE=25CA;WHITE CIRCLE=25CB;DOTTED CIRCLE=25CC;CIRCLE WITH VERTICAL FILL=25CD;BULLSEYE=25CE;BLACK CIRCLE=25CF;CIRCLE WITH LEFT HALF BLACK=25D0;CIRCLE WITH RIGHT HALF BLACK=25D1;CIRCLE WITH LOWER HALF BLACK=25D2;CIRCLE WITH UPPER HALF BLACK=25D3;CIRCLE WITH UPPER RIGHT QUADRANT BLACK=25D4;CIRCLE WITH ALL BUT UPPER LEFT QUADRANT BLACK=25D5;LEFT HALF BLACK CIRCLE=25D6;RIGHT HALF BLACK CIRCLE=25D7;INVERSE BULLET=25D8;INVERSE WHITE CIRCLE=25D9;UPPER HALF INVERSE WHITE CIRCLE=25DA;LOWER HALF INVERSE WHITE CIRCLE=25DB;UPPER LEFT QUADRANT CIRCULAR ARC=25DC;UPPER RIGHT QUADRANT CIRCULAR ARC=25DD;LOWER RIGHT QUADRANT CIRCULAR ARC=25DE;LOWER LEFT QUADRANT CIRCULAR ARC=25DF;UPPER HALF CIRCLE=25E0;LOWER HALF CIRCLE=25E1;BLACK LOWER RIGHT TRIANGLE=25E2;BLACK LOWER LEFT TRIANGLE=25E3;BLACK UPPER LEFT TRIANGLE=25E4;BLACK UPPER RIGHT TRIANGLE=25E5;WHITE BULLET=25E6;SQUARE WITH LEFT HALF BLACK=25E7;SQUARE WITH RIGHT HALF BLACK=25E8;SQUARE WITH UPPER LEFT DIAGONAL HALF BLACK=25E9;SQUARE WITH LOWER RIGHT DIAGONAL HALF BLACK=25EA;WHITE SQUARE WITH VERTICAL BISECTING LINE=25EB;WHITE UP-POINTING TRIANGLE WITH DOT=25EC;UP-POINTING TRIANGLE WITH LEFT HALF BLACK=25ED;UP-POINTING TRIANGLE WITH RIGHT HALF BLACK=25EE;LARGE CIRCLE=25EF;WHITE SQUARE WITH UPPER LEFT QUADRANT=25F0;WHITE SQUARE WITH LOWER LEFT QUADRANT=25F1;WHITE SQUARE WITH LOWER RIGHT QUADRANT=25F2;WHITE SQUARE WITH UPPER RIGHT QUADRANT=25F3;WHITE CIRCLE WITH UPPER LEFT QUADRANT=25F4;WHITE CIRCLE WITH LOWER LEFT QUADRANT=25F5;WHITE CIRCLE WITH LOWER RIGHT QUADRANT=25F6;WHITE CIRCLE WITH UPPER RIGHT QUADRANT=25F7;UPPER LEFT TRIANGLE=25F8;UPPER RIGHT TRIANGLE=25F9;LOWER LEFT TRIANGLE=25FA;WHITE MEDIUM SQUARE=25FB;BLACK MEDIUM SQUARE=25FC;WHITE MEDIUM SMALL SQUARE=25FD;BLACK MEDIUM SMALL SQUARE=25FE;LOWER RIGHT TRIANGLE=25FF;BLACK SUN WITH RAYS=2600;CLOUD=2601;UMBRELLA=2602;SNOWMAN=2603;COMET=2604;BLACK STAR=2605;WHITE STAR=2606;LIGHTNING=2607;THUNDERSTORM=2608;SUN=2609;ASCENDING NODE=260A;DESCENDING NODE=260B;CONJUNCTION=260C;OPPOSITION=260D;BLACK TELEPHONE=260E;WHITE TELEPHONE=260F;BALLOT BOX=2610;BALLOT BOX WITH CHECK=2611;BALLOT BOX WITH X=2612;SALTIRE=2613;UMBRELLA WITH RAIN DROPS=2614;HOT BEVERAGE=2615;WHITE SHOGI PIECE=2616;BLACK SHOGI PIECE=2617;SHAMROCK=2618;REVERSED ROTATED FLORAL HEART BULLET=2619;BLACK LEFT POINTING INDEX=261A;BLACK RIGHT POINTING INDEX=261B;WHITE LEFT POINTING INDEX=261C;WHITE UP POINTING INDEX=261D;WHITE RIGHT POINTING INDEX=261E;WHITE DOWN POINTING INDEX=261F;SKULL AND CROSSBONES=2620;CAUTION SIGN=2621;RADIOACTIVE SIGN=2622;BIOHAZARD SIGN=2623;CADUCEUS=2624;ANKH=2625;ORTHODOX CROSS=2626;CHI RHO=2627;CROSS OF LORRAINE=2628;CROSS OF JERUSALEM=2629;STAR AND CRESCENT=262A;FARSI SYMBOL=262B;ADI SHAKTI=262C;HAMMER AND SICKLE=262D;PEACE SYMBOL=262E;YIN YANG=262F;TRIGRAM FOR HEAVEN=2630;TRIGRAM FOR LAKE=2631;TRIGRAM FOR FIRE=2632;TRIGRAM FOR THUNDER=2633;TRIGRAM FOR WIND=2634;TRIGRAM FOR WATER=2635;TRIGRAM FOR MOUNTAIN=2636;TRIGRAM FOR EARTH=2637;WHEEL OF DHARMA=2638;WHITE FROWNING FACE=2639;WHITE SMILING FACE=263A;BLACK SMILING FACE=263B;WHITE SUN WITH RAYS=263C;FIRST QUARTER MOON=263D;LAST QUARTER MOON=263E;MERCURY=263F;FEMALE SIGN=2640;EARTH=2641;MALE SIGN=2642;JUPITER=2643;SATURN=2644;URANUS=2645;NEPTUNE=2646;PLUTO=2647;ARIES=2648;TAURUS=2649;GEMINI=264A;CANCER=264B;LEO=264C;VIRGO=264D;LIBRA=264E;SCORPIUS=264F;SAGITTARIUS=2650;CAPRICORN=2651;AQUARIUS=2652;PISCES=2653;WHITE CHESS KING=2654;WHITE CHESS QUEEN=2655;WHITE CHESS ROOK=2656;WHITE CHESS BISHOP=2657;WHITE CHESS KNIGHT=2658;WHITE CHESS PAWN=2659;BLACK CHESS KING=265A;BLACK CHESS QUEEN=265B;BLACK CHESS ROOK=265C;BLACK CHESS BISHOP=265D;BLACK CHESS KNIGHT=265E;BLACK CHESS PAWN=265F;BLACK SPADE SUIT=2660;WHITE HEART SUIT=2661;WHITE DIAMOND SUIT=2662;BLACK CLUB SUIT=2663;WHITE SPADE SUIT=2664;BLACK HEART SUIT=2665;BLACK DIAMOND SUIT=2666;WHITE CLUB SUIT=2667;HOT SPRINGS=2668;QUARTER NOTE=2669;EIGHTH NOTE=266A;BEAMED EIGHTH NOTES=266B;BEAMED SIXTEENTH NOTES=266C;MUSIC FLAT SIGN=266D;MUSIC NATURAL SIGN=266E;MUSIC SHARP SIGN=266F;WEST SYRIAC CROSS=2670;EAST SYRIAC CROSS=2671;UNIVERSAL RECYCLING SYMBOL=2672;RECYCLING SYMBOL FOR TYPE-1 PLASTICS=2673;RECYCLING SYMBOL FOR TYPE-2 PLASTICS=2674;RECYCLING SYMBOL FOR TYPE-3 PLASTICS=2675;RECYCLING SYMBOL FOR TYPE-4 PLASTICS=2676;RECYCLING SYMBOL FOR TYPE-5 PLASTICS=2677;RECYCLING SYMBOL FOR TYPE-6 PLASTICS=2678;RECYCLING SYMBOL FOR TYPE-7 PLASTICS=2679;RECYCLING SYMBOL FOR GENERIC MATERIALS=267A;BLACK UNIVERSAL RECYCLING SYMBOL=267B;RECYCLED PAPER SYMBOL=267C;PARTIALLY-RECYCLED PAPER SYMBOL=267D;PERMANENT PAPER SIGN=267E;WHEELCHAIR SYMBOL=267F;DIE FACE-1=2680;DIE FACE-2=2681;DIE FACE-3=2682;DIE FACE-4=2683;DIE FACE-5=2684;DIE FACE-6=2685;WHITE CIRCLE WITH DOT RIGHT=2686;WHITE CIRCLE WITH TWO DOTS=2687;BLACK CIRCLE WITH WHITE DOT RIGHT=2688;BLACK CIRCLE WITH TWO WHITE DOTS=2689;MONOGRAM FOR YANG=268A;MONOGRAM FOR YIN=268B;DIGRAM FOR GREATER YANG=268C;DIGRAM FOR LESSER YIN=268D;DIGRAM FOR LESSER YANG=268E;DIGRAM FOR GREATER YIN=268F;WHITE FLAG=2690;BLACK FLAG=2691;HAMMER AND PICK=2692;ANCHOR=2693;CROSSED SWORDS=2694;STAFF OF AESCULAPIUS=2695;SCALES=2696;ALEMBIC=2697;FLOWER=2698;GEAR=2699;STAFF OF HERMES=269A;ATOM SYMBOL=269B;FLEUR-DE-LIS=269C;OUTLINED WHITE STAR=269D;THREE LINES CONVERGING RIGHT=269E;THREE LINES CONVERGING LEFT=269F;WARNING SIGN=26A0;HIGH VOLTAGE SIGN=26A1;DOUBLED FEMALE SIGN=26A2;DOUBLED MALE SIGN=26A3;INTERLOCKED FEMALE AND MALE SIGN=26A4;MALE AND FEMALE SIGN=26A5;MALE WITH STROKE SIGN=26A6;MALE WITH STROKE AND MALE AND FEMALE SIGN=26A7;VERTICAL MALE WITH STROKE SIGN=26A8;HORIZONTAL MALE WITH STROKE SIGN=26A9;MEDIUM WHITE CIRCLE=26AA;MEDIUM BLACK CIRCLE=26AB;MEDIUM SMALL WHITE CIRCLE=26AC;MARRIAGE SYMBOL=26AD;DIVORCE SYMBOL=26AE;UNMARRIED PARTNERSHIP SYMBOL=26AF;COFFIN=26B0;FUNERAL URN=26B1;NEUTER=26B2;CERES=26B3;PALLAS=26B4;JUNO=26B5;VESTA=26B6;CHIRON=26B7;BLACK MOON LILITH=26B8;SEXTILE=26B9;SEMISEXTILE=26BA;QUINCUNX=26BB;SESQUIQUADRATE=26BC;SOCCER BALL=26BD;BASEBALL=26BE;SQUARED KEY=26BF;WHITE DRAUGHTS MAN=26C0;WHITE DRAUGHTS KING=26C1;BLACK DRAUGHTS MAN=26C2;BLACK DRAUGHTS KING=26C3;SNOWMAN WITHOUT SNOW=26C4;SUN BEHIND CLOUD=26C5;RAIN=26C6;BLACK SNOWMAN=26C7;THUNDER CLOUD AND RAIN=26C8;TURNED WHITE SHOGI PIECE=26C9;TURNED BLACK SHOGI PIECE=26CA;WHITE DIAMOND IN SQUARE=26CB;CROSSING LANES=26CC;DISABLED CAR=26CD;OPHIUCHUS=26CE;PICK=26CF;CAR SLIDING=26D0;HELMET WITH WHITE CROSS=26D1;CIRCLED CROSSING LANES=26D2;CHAINS=26D3;NO ENTRY=26D4;ALTERNATE ONE-WAY LEFT WAY TRAFFIC=26D5;BLACK TWO-WAY LEFT WAY TRAFFIC=26D6;WHITE TWO-WAY LEFT WAY TRAFFIC=26D7;BLACK LEFT LANE MERGE=26D8;WHITE LEFT LANE MERGE=26D9;DRIVE SLOW SIGN=26DA;HEAVY WHITE DOWN-POINTING TRIANGLE=26DB;LEFT CLOSED ENTRY=26DC;SQUARED SALTIRE=26DD;FALLING DIAGONAL IN WHITE CIRCLE IN BLACK SQUARE=26DE;BLACK TRUCK=26DF;RESTRICTED LEFT ENTRY-1=26E0;RESTRICTED LEFT ENTRY-2=26E1;ASTRONOMICAL SYMBOL FOR URANUS=26E2;HEAVY CIRCLE WITH STROKE AND TWO DOTS ABOVE=26E3;PENTAGRAM=26E4;RIGHT-HANDED INTERLACED PENTAGRAM=26E5;LEFT-HANDED INTERLACED PENTAGRAM=26E6;INVERTED PENTAGRAM=26E7;BLACK CROSS ON SHIELD=26E8;SHINTO SHRINE=26E9;CHURCH=26EA;CASTLE=26EB;HISTORIC SITE=26EC;GEAR WITHOUT HUB=26ED;GEAR WITH HANDLES=26EE;MAP SYMBOL FOR LIGHTHOUSE=26EF;MOUNTAIN=26F0;UMBRELLA ON GROUND=26F1;FOUNTAIN=26F2;FLAG IN HOLE=26F3;FERRY=26F4;SAILBOAT=26F5;SQUARE FOUR CORNERS=26F6;SKIER=26F7;ICE SKATE=26F8;PERSON WITH BALL=26F9;TENT=26FA;JAPANESE BANK SYMBOL=26FB;HEADSTONE GRAVEYARD SYMBOL=26FC;FUEL PUMP=26FD;CUP ON BLACK SQUARE=26FE;WHITE FLAG WITH HORIZONTAL MIDDLE BLACK STRIPE=26FF;IDEOGRAPHIC SPACE=3000;IDEOGRAPHIC COMMA=3001;IDEOGRAPHIC FULL STOP=3002;DITTO MARK=3003;JAPANESE INDUSTRIAL STANDARD SYMBOL=3004;IDEOGRAPHIC ITERATION MARK=3005;IDEOGRAPHIC CLOSING MARK=3006;IDEOGRAPHIC NUMBER ZERO=3007;LEFT ANGLE BRACKET=3008;RIGHT ANGLE BRACKET=3009;LEFT DOUBLE ANGLE BRACKET=300A;RIGHT DOUBLE ANGLE BRACKET=300B;LEFT CORNER BRACKET=300C;RIGHT CORNER BRACKET=300D;LEFT WHITE CORNER BRACKET=300E;RIGHT WHITE CORNER BRACKET=300F;LEFT BLACK LENTICULAR BRACKET=3010;RIGHT BLACK LENTICULAR BRACKET=3011;POSTAL MARK=3012;GETA MARK=3013;LEFT TORTOISE SHELL BRACKET=3014;RIGHT TORTOISE SHELL BRACKET=3015;LEFT WHITE LENTICULAR BRACKET=3016;RIGHT WHITE LENTICULAR BRACKET=3017;LEFT WHITE TORTOISE SHELL BRACKET=3018;RIGHT WHITE TORTOISE SHELL BRACKET=3019;LEFT WHITE SQUARE BRACKET=301A;RIGHT WHITE SQUARE BRACKET=301B;WAVE DASH=301C;REVERSED DOUBLE PRIME QUOTATION MARK=301D;DOUBLE PRIME QUOTATION MARK=301E;LOW DOUBLE PRIME QUOTATION MARK=301F;POSTAL MARK FACE=3020;HANGZHOU NUMERAL ONE=3021;HANGZHOU NUMERAL TWO=3022;HANGZHOU NUMERAL THREE=3023;HANGZHOU NUMERAL FOUR=3024;HANGZHOU NUMERAL FIVE=3025;HANGZHOU NUMERAL SIX=3026;HANGZHOU NUMERAL SEVEN=3027;HANGZHOU NUMERAL EIGHT=3028;HANGZHOU NUMERAL NINE=3029;IDEOGRAPHIC LEVEL TONE MARK=302A;IDEOGRAPHIC RISING TONE MARK=302B;IDEOGRAPHIC DEPARTING TONE MARK=302C;IDEOGRAPHIC ENTERING TONE MARK=302D;HANGUL SINGLE DOT TONE MARK=302E;HANGUL DOUBLE DOT TONE MARK=302F;WAVY DASH=3030;VERTICAL KANA REPEAT MARK=3031;VERTICAL KANA REPEAT WITH VOICED SOUND MARK=3032;VERTICAL KANA REPEAT MARK UPPER HALF=3033;VERTICAL KANA REPEAT WITH VOICED SOUND MARK UPPER HALF=3034;VERTICAL KANA REPEAT MARK LOWER HALF=3035;CIRCLED POSTAL MARK=3036;IDEOGRAPHIC TELEGRAPH LINE FEED SEPARATOR SYMBOL=3037;HANGZHOU NUMERAL TEN=3038;HANGZHOU NUMERAL TWENTY=3039;HANGZHOU NUMERAL THIRTY=303A;VERTICAL IDEOGRAPHIC ITERATION MARK=303B;MASU MARK=303C;PART ALTERNATION MARK=303D;IDEOGRAPHIC VARIATION INDICATOR=303E;IDEOGRAPHIC HALF FILL SPACE=303F;VARIATION SELECTOR-1=FE00;VARIATION SELECTOR-2=FE01;VARIATION SELECTOR-3=FE02;VARIATION SELECTOR-4=FE03;VARIATION SELECTOR-5=FE04;VARIATION SELECTOR-6=FE05;VARIATION SELECTOR-7=FE06;VARIATION SELECTOR-8=FE07;VARIATION SELECTOR-9=FE08;VARIATION SELECTOR-10=FE09;VARIATION SELECTOR-11=FE0A;VARIATION SELECTOR-12=FE0B;VARIATION SELECTOR-13=FE0C;VARIATION SELECTOR-14=FE0D;VARIATION SELECTOR-15=FE0E;VARIATION SELECTOR-16=FE0F;INTERLINEAR ANNOTATION ANCHOR=FFF9;INTERLINEAR ANNOTATION SEPARATOR=FFFA;INTERLINEAR ANNOTATION TERMINATOR=FFFB;OBJECT REPLACEMENT CHARACTER=FFFC;REPLACEMENT CHARACTER=FFFD;";
};
PyUnicodeNames.lookup = function(name) {
  const t = PyUnicodeNames.table();
  const key = (";" + name.toUpperCase()) + "=";
  const at = t.indexOf(key);
  if ( at < 0 ) {
    return -1;
  }
  const from = at + key.length;
  const rest = t.substring(from, from + 8 );
  const hex = rest.substring(0, rest.indexOf(";") );
  let v = 0;
  const cs = Array.from(hex, (rg_c) => rg_c.codePointAt(0));
  // Loop start
  for ( const c of cs) {
    if ( c >= 48 && c <= 57 ) {
      v = v * 16 + (c - 48);
    } else {
      v = v * 16 + (c - 55);
    }
  }
  return v;
};
Object.defineProperty(PyUnicodeNames, "name", { value: function(cp) {
  const t = PyUnicodeNames.table();
  const digits = "0123456789ABCDEF";
  let hex = "";
  let v = cp;
  if ( v == 0 ) {
    hex = "0";
  }
  while (v > 0) {
    const d = v % 16;
    hex = digits.substring(d, d + 1 ) + hex;
    v = (((v - d) / 16) | 0);
  };
  const key = ("=" + hex) + ";";
  const at = t.indexOf(key);
  if ( at < 0 ) {
    return "";
  }
  let start = at;
  while (start > 0 && t.substring(start - 1, start ) != ";") {
    start = start - 1;
  };
  return t.substring(start, at );
}, writable: true, configurable: true });
export class PyLexer  {
  constructor(src) {
    this.cs = [];
    this.n = 0;
    this.pos = 0;
    this.line = 1;
    this.depth = 0;
    this.indents = [];
    this.toks = [];
    this.err = "";
    this.errLine = 0;
    this.atLineStart = true;
    this.cs = Array.from(src, (rg_c) => rg_c.codePointAt(0));
    this.n = this.cs.length;
    this.indents.push(0);
  }
  fail (msg) {
    if ( this.err.length == 0 ) {
      this.err = msg;
      this.errLine = this.line;
    }
  };
  ch (off) {
    const p = this.pos + off;
    if ( p >= this.n ) {
      return 0;
    }
    return this.cs[p];
  };
  add (k, text) {
    const t = new PyToken();
    t.kind = k;
    t.text = text;
    t.line = this.line;
    this.toks.push(t);
  };
  text (a, b) {
    return py_rt_codes(this.cs, a, b);
  };
  lineIndent () {
    let w = 0;
    let p = this.pos;
    while (p < this.n) {
      const c = this.cs[p];
      if ( c == 32 ) {
        w = w + 1;
      } else {
        if ( c == 9 ) {
          w = (((w / 8) | 0) + 1) * 8;
        } else {
          if ( c == 12 ) {
            w = 0;
          } else {
            if ( (c == 10 || c == 13) || c == 35 ) {
              this.pos = p;
              return -1;
            }
            this.pos = p;
            return w;
          }
        }
      }
      p = p + 1;
    };
    this.pos = p;
    return -1;
  };
  tokenize () {
    while (this.pos < this.n) {
      if ( this.err.length > 0 ) {
        return this.toks;
      }
      if ( this.atLineStart && this.depth == 0 ) {
        const w = this.lineIndent();
        if ( w < 0 ) {
          while (this.pos < this.n && this.ch(0) != 10) {
            this.pos = this.pos + 1;
          };
          if ( this.pos < this.n ) {
            this.pos = this.pos + 1;
            this.line = this.line + 1;
          }
          continue;
        }
        this.atLineStart = false;
        const top = this.indents[(this.indents.length - 1)];
        if ( w > top ) {
          this.indents.push(w);
          this.add(7, "");
        } else {
          while (w < this.indents[(this.indents.length - 1)]) {
            this.indents.pop();
            this.add(8, "");
          };
          if ( w != this.indents[(this.indents.length - 1)] ) {
            this.fail("unindent does not match any outer indentation level");
            return this.toks;
          }
        }
      }
      this.scanOne();
    };
    if ( this.err.length > 0 ) {
      return this.toks;
    }
    if ( this.toks.length > 0 ) {
      const lt = this.toks[(this.toks.length - 1)];
      if ( lt.kind != 6 && lt.kind != 8 ) {
        this.add(6, "");
      }
    }
    while (this.indents.length > 1) {
      this.indents.pop();
      this.add(8, "");
    };
    this.add(9, "");
    return this.toks;
  };
  scanOne () {
    const c = this.ch(0);
    if ( (c == 32 || c == 9) || c == 12 ) {
      this.pos = this.pos + 1;
      return;
    }
    if ( c == 13 ) {
      this.pos = this.pos + 1;
      return;
    }
    if ( c == 35 ) {
      while (this.pos < this.n && this.ch(0) != 10) {
        this.pos = this.pos + 1;
      };
      return;
    }
    if ( c == 92 ) {
      let p = this.pos + 1;
      if ( p < this.n && this.cs[p] == 13 ) {
        p = p + 1;
      }
      if ( p < this.n && this.cs[p] == 10 ) {
        this.pos = p + 1;
        this.line = this.line + 1;
        return;
      }
      this.fail("unexpected character after line continuation character");
      return;
    }
    if ( c == 10 ) {
      this.pos = this.pos + 1;
      if ( this.depth == 0 ) {
        this.add(6, "");
        this.atLineStart = true;
      }
      this.line = this.line + 1;
      return;
    }
    if ( PyLexer.isIdStart(c) ) {
      this.scanNameOrString();
      return;
    }
    if ( PyLexer.isDigit(c) || c == 46 && PyLexer.isDigit(this.ch(1)) ) {
      this.scanNumber();
      return;
    }
    if ( c == 34 || c == 39 ) {
      this.scanString(this.pos, false, false, false);
      return;
    }
    this.scanOp();
  };
  scanNameOrString () {
    const start = this.pos;
    while (this.pos < this.n && PyLexer.isIdChar(this.ch(0))) {
      this.pos = this.pos + 1;
    };
    const word = this.text(start, this.pos);
    const q = this.ch(0);
    if ( (q == 34 || q == 39) && this.pos - start <= 2 ) {
      const lw = word.toLowerCase();
      let raw = false;
      let fmt = false;
      let isBytes = false;
      let ok = true;
      const ws = Array.from(lw, (rg_c) => rg_c.codePointAt(0));
      // Loop start
      for ( const wc of ws) {
        if ( wc == 114 ) {
          raw = true;
        } else {
          if ( wc == 102 ) {
            fmt = true;
          } else {
            if ( wc == 98 ) {
              isBytes = true;
            } else {
              if ( wc != 117 ) {
                ok = false;
              }
            }
          }
        }
      }
      if ( ok ) {
        this.scanString(this.pos, raw, fmt, isBytes);
        return;
      }
    }
    this.add(0, word);
  };
  scanNumber () {
    const start = this.pos;
    const c = this.ch(0);
    const c1 = this.ch(1);
    if ( c == 48 && (((((c1 == 120 || c1 == 88) || c1 == 111) || c1 == 79) || c1 == 98) || c1 == 66) ) {
      this.pos = this.pos + 2;
      const ds = this.pos;
      while (this.pos < this.n && PyLexer.isIdChar(this.ch(0))) {
        this.pos = this.pos + 1;
      };
      const digits = this.text(ds, this.pos);
      let base = 16;
      if ( c1 == 111 || c1 == 79 ) {
        base = 8;
      }
      if ( c1 == 98 || c1 == 66 ) {
        base = 2;
      }
      let v = 0;
      const dcs = Array.from(digits, (rg_c) => rg_c.codePointAt(0));
      if ( dcs.length == 0 ) {
        this.fail("invalid number literal");
        return;
      }
      // Loop start
      for ( const d of dcs) {
        if ( d != 95 ) {
          const dv = PyLexer.digitValue(d);
          if ( dv < 0 || dv >= base ) {
            this.fail("invalid digit in number literal " + this.text(start, this.pos));
            return;
          }
          v = v * base + dv;
        }
      }
      this.add(1, ((v).toString()));
      return;
    }
    let isFloat = false;
    while (this.pos < this.n && (PyLexer.isDigit(this.ch(0)) || this.ch(0) == 95)) {
      this.pos = this.pos + 1;
    };
    if ( this.ch(0) == 46 ) {
      isFloat = true;
      this.pos = this.pos + 1;
      while (this.pos < this.n && (PyLexer.isDigit(this.ch(0)) || this.ch(0) == 95)) {
        this.pos = this.pos + 1;
      };
    }
    const e = this.ch(0);
    if ( e == 101 || e == 69 ) {
      let p = this.pos + 1;
      const sg = this.ch(1);
      if ( sg == 43 || sg == 45 ) {
        p = p + 1;
      }
      if ( p < this.n && PyLexer.isDigit(this.cs[p]) ) {
        isFloat = true;
        this.pos = p;
        while (this.pos < this.n && PyLexer.isDigit(this.ch(0))) {
          this.pos = this.pos + 1;
        };
      }
    }
    const jc = this.ch(0);
    let imag = false;
    if ( jc == 106 || jc == 74 ) {
      imag = true;
    }
    let raw = [];
    let i = start;
    while (i < this.pos) {
      const rc = this.cs[i];
      if ( rc != 95 ) {
        raw.push(rc);
      }
      i = i + 1;
    };
    const t = py_rt_codes(raw, 0, raw.length);
    if ( imag ) {
      this.pos = this.pos + 1;
      this.add(2, t + "j");
      return;
    }
    if ( isFloat ) {
      this.add(2, t);
    } else {
      this.add(1, t);
    }
  };
  copyNestedString (out) {
    const q = this.ch(0);
    const triple = this.ch(1) == q && this.ch(2) == q;
    let w = 1;
    if ( triple ) {
      w = 3;
    }
    let k = 0;
    while (k < w) {
      out.push(q);
      this.pos = this.pos + 1;
      k = k + 1;
    };
    while (this.pos < this.n) {
      const c = this.ch(0);
      if ( c == 92 ) {
        out.push(c);
        out.push(this.ch(1));
        this.pos = this.pos + 2;
        continue;
      }
      if ( c == 10 ) {
        this.line = this.line + 1;
      }
      if ( c == q ) {
        if ( triple == false ) {
          out.push(c);
          this.pos = this.pos + 1;
          return;
        }
        if ( this.ch(1) == q && this.ch(2) == q ) {
          out.push(q);
          out.push(q);
          out.push(q);
          this.pos = this.pos + 3;
          return;
        }
      }
      out.push(c);
      this.pos = this.pos + 1;
    };
  };
  scanString (at, raw, fmt, isBytes) {
    this.pos = at;
    const q = this.ch(0);
    const triple = this.ch(1) == q && this.ch(2) == q;
    if ( triple ) {
      this.pos = this.pos + 3;
    } else {
      this.pos = this.pos + 1;
    }
    let out = [];
    const startLine = this.line;
    let modes = [];
    while (true) {
      if ( this.pos >= this.n ) {
        this.line = startLine;
        this.fail("unterminated string literal");
        return;
      }
      const c = this.ch(0);
      if ( fmt && modes.length > 0 ) {
        const md = modes[(modes.length - 1)];
        if ( c == 123 ) {
          if ( md == 1 ) {
            modes.push(0);
          } else {
            modes.push(2);
          }
          out.push(c);
          this.pos = this.pos + 1;
          continue;
        }
        if ( c == 125 ) {
          modes.pop();
          out.push(c);
          this.pos = this.pos + 1;
          continue;
        }
        if ( c == 10 ) {
          this.line = this.line + 1;
          out.push(c);
          this.pos = this.pos + 1;
          continue;
        }
        if ( (md == 1 && c == 92) && raw == false ) {
          const e2 = this.ch(1);
          let ev = -1;
          if ( e2 == 110 ) {
            ev = 10;
          }
          if ( e2 == 116 ) {
            ev = 9;
          }
          if ( e2 == 114 ) {
            ev = 13;
          }
          if ( e2 == 102 ) {
            ev = 12;
          }
          if ( e2 == 118 ) {
            ev = 11;
          }
          if ( (e2 == 92 || e2 == 39) || e2 == 34 ) {
            ev = e2;
          }
          if ( ev >= 0 ) {
            out.push(ev);
            this.pos = this.pos + 2;
            continue;
          }
        }
        if ( md != 1 ) {
          if ( c == 40 || c == 91 ) {
            modes.push(3);
          }
          if ( (c == 41 || c == 93) && md == 3 ) {
            modes.pop();
          }
          if ( (c == 58 && md == 0) && this.ch(1) != 61 ) {
            modes.pop();
            modes.push(1);
          }
          if ( c == 39 || c == 34 ) {
            this.copyNestedString(out);
            continue;
          }
        }
        out.push(c);
        this.pos = this.pos + 1;
        continue;
      }
      if ( fmt && c == 123 ) {
        if ( this.ch(1) == 123 ) {
          out.push(c);
          out.push(c);
          this.pos = this.pos + 2;
          continue;
        }
        modes.push(0);
        out.push(c);
        this.pos = this.pos + 1;
        continue;
      }
      if ( c == q ) {
        if ( triple ) {
          if ( this.ch(1) == q && this.ch(2) == q ) {
            this.pos = this.pos + 3;
            break;
          }
          out.push(c);
          this.pos = this.pos + 1;
          continue;
        }
        this.pos = this.pos + 1;
        break;
      }
      if ( c == 10 ) {
        if ( triple == false ) {
          this.fail("unterminated string literal");
          return;
        }
        this.line = this.line + 1;
        out.push(c);
        this.pos = this.pos + 1;
        continue;
      }
      if ( c == 92 ) {
        const c2 = this.ch(1);
        if ( raw ) {
          out.push(c);
          if ( c2 == q || c2 == 92 ) {
            out.push(c2);
            this.pos = this.pos + 2;
          } else {
            this.pos = this.pos + 1;
          }
          continue;
        }
        this.pos = this.pos + 2;
        if ( c2 == 10 ) {
          this.line = this.line + 1;
          continue;
        }
        if ( c2 == 110 ) {
          out.push(10);
          continue;
        }
        if ( c2 == 116 ) {
          out.push(9);
          continue;
        }
        if ( c2 == 114 ) {
          out.push(13);
          continue;
        }
        if ( (c2 == 92 || c2 == 39) || c2 == 34 ) {
          out.push(c2);
          continue;
        }
        if ( c2 == 48 ) {
          if ( PyLexer.isDigit(this.ch(0)) == false ) {
            out.push(0);
            continue;
          }
        }
        if ( c2 >= 48 && c2 <= 55 ) {
          let v = c2 - 48;
          let k = 0;
          while ((k < 2 && this.ch(0) >= 48) && this.ch(0) <= 55) {
            v = v * 8 + (this.ch(0) - 48);
            this.pos = this.pos + 1;
            k = k + 1;
          };
          out.push(v);
          continue;
        }
        if ( c2 == 97 ) {
          out.push(7);
          continue;
        }
        if ( c2 == 98 ) {
          out.push(8);
          continue;
        }
        if ( c2 == 102 ) {
          out.push(12);
          continue;
        }
        if ( c2 == 118 ) {
          out.push(11);
          continue;
        }
        if ( (c2 == 120 || c2 == 117) || c2 == 85 ) {
          let cnt = 2;
          if ( c2 == 117 ) {
            cnt = 4;
          }
          if ( c2 == 85 ) {
            cnt = 8;
          }
          let v_1 = 0;
          let k_1 = 0;
          while (k_1 < cnt) {
            const dv = PyLexer.digitValue(this.ch(0));
            if ( dv < 0 ) {
              this.fail("truncated \\x, \\u or \\U escape");
              return;
            }
            v_1 = v_1 * 16 + dv;
            this.pos = this.pos + 1;
            k_1 = k_1 + 1;
          };
          if ( fmt && (v_1 == 123 || v_1 == 125) ) {
            out.push(v_1);
          }
          out.push(v_1);
          continue;
        }
        if ( c2 == 78 && isBytes == false ) {
          let nm = [];
          let closed = false;
          if ( this.ch(0) == 123 ) {
            this.pos = this.pos + 1;
            while (this.pos < this.n) {
              const nc = this.ch(0);
              this.pos = this.pos + 1;
              if ( nc == 125 ) {
                closed = true;
                break;
              }
              nm.push(nc);
            };
          }
          let ucp = -1;
          if ( closed ) {
            ucp = PyUnicodeNames.lookup(py_rt_codes(nm, 0, nm.length));
          }
          if ( ucp < 0 ) {
            this.fail("(unicode error) 'unicodeescape' codec can't decode bytes: unknown Unicode character name");
            return;
          }
          if ( fmt && (ucp == 123 || ucp == 125) ) {
            out.push(ucp);
          }
          out.push(ucp);
          continue;
        }
        out.push(92);
        out.push(c2);
        continue;
      }
      out.push(c);
      this.pos = this.pos + 1;
    };
    const t = new PyToken();
    if ( fmt ) {
      t.kind = 4;
    } else {
      t.kind = 3;
    }
    t.text = py_rt_codes(out, 0, out.length);
    t.line = startLine;
    t.bytes = isBytes;
    this.toks.push(t);
  };
  scanOp () {
    const c = this.ch(0);
    const three = this.text(this.pos, this.minN((this.pos + 3)));
    const two = this.text(this.pos, this.minN((this.pos + 2)));
    const one = String.fromCodePoint(c);
    if ( (((three == "**=" || three == "//=") || three == ">>=") || three == "<<=") || three == "..." ) {
      this.pos = this.pos + 3;
      this.add(5, three);
      return;
    }
    if ( (((((((((((((((((two == "**" || two == "//") || two == "==") || two == "!=") || two == "<=") || two == ">=") || two == "<<") || two == ">>") || two == "+=") || two == "-=") || two == "*=") || two == "/=") || two == "%=") || two == "&=") || two == "|=") || two == "^=") || two == "@=") || two == "->") || two == ":=" ) {
      this.pos = this.pos + 2;
      this.add(5, two);
      return;
    }
    if ( (c == 40 || c == 91) || c == 123 ) {
      this.depth = this.depth + 1;
    }
    if ( (c == 41 || c == 93) || c == 125 ) {
      if ( this.depth > 0 ) {
        this.depth = this.depth - 1;
      }
    }
    const ops = "+-*/%@&|^~<>()[]{},:.;=!";
    if ( ops.indexOf(one) >= 0 ) {
      this.pos = this.pos + 1;
      this.add(5, one);
      return;
    }
    this.fail(("invalid character '" + one) + "' in the source");
  };
  minN (p) {
    if ( p > this.n ) {
      return this.n;
    }
    return p;
  };
}
PyLexer.isIdStart = function(c) {
  if ( c >= 97 && c <= 122 ) {
    return true;
  }
  if ( c >= 65 && c <= 90 ) {
    return true;
  }
  if ( c == 95 ) {
    return true;
  }
  return c > 127;
};
PyLexer.isDigit = function(c) {
  return c >= 48 && c <= 57;
};
PyLexer.isIdChar = function(c) {
  if ( PyLexer.isIdStart(c) ) {
    return true;
  }
  return PyLexer.isDigit(c);
};
PyLexer.digitValue = function(d) {
  if ( d >= 48 && d <= 57 ) {
    return d - 48;
  }
  if ( d >= 97 && d <= 102 ) {
    return d - 87;
  }
  if ( d >= 65 && d <= 70 ) {
    return d - 55;
  }
  return -1;
};
export class PyParser  {
  constructor() {
    this.toks = [];
    this.p = 0;
    this.err = "";
    this.errLine = 0;
    this.yields = [];
    this.futureAnn = false;
    this.qualPrefix = [];
    this.mangleIn = [];
  }
  parseModule (src) {
    const m = new PyNode(65, 1);
    const lx = new PyLexer(src);
    this.toks = lx.tokenize();
    if ( lx.err.length > 0 ) {
      this.err = lx.err;
      this.errLine = lx.errLine;
      return m;
    }
    try {
      while (this.peekKind() != 9) {
        if ( this.peekKind() == 6 ) {
          this.p = this.p + 1;
          continue;
        }
        this.parseStatementInto(m.body);
      };
    } catch(e) {
      if ( this.err.length == 0 ) {
        this.err = "internal parser error: " + ( e.toString());
      }
    }
    return m;
  };
  parseExpressionText (src, line) {
    const lx = new PyLexer(("(" + src) + "\n)");
    this.toks = lx.tokenize();
    if ( lx.err.length > 0 ) {
      this.failAt("f-string: " + lx.err, line);
    }
    // Loop start
    for ( const t of this.toks) {
      t.line = line;
    }
    this.p = 0;
    const e = this.parseTestListStarExpr();
    if ( this.peekKind() == 6 ) {
      this.p = this.p + 1;
    }
    if ( this.peekKind() != 9 ) {
      this.fail("f-string: invalid expression");
    }
    return e;
  };
  failAt (msg, line) {
    if ( this.err.length == 0 ) {
      this.err = msg;
      this.errLine = line;
    }
    throw new Error("SyntaxError: " + msg);
  };
  fail (msg) {
    const t = this.peek();
    this.failAt(msg, t.line);
  };
  peek () {
    if ( this.p >= this.toks.length ) {
      return this.toks[(this.toks.length - 1)];
    }
    return this.toks[this.p];
  };
  peekAt (off) {
    const i = this.p + off;
    if ( i >= this.toks.length ) {
      return this.toks[(this.toks.length - 1)];
    }
    return this.toks[i];
  };
  peekKind () {
    const t = this.peek();
    return t.kind;
  };
  line () {
    const t = this.peek();
    return t.line;
  };
  isOp (s) {
    const t = this.peek();
    return t.kind == 5 && t.text == s;
  };
  isOpAt (off, s) {
    const t = this.peekAt(off);
    return t.kind == 5 && t.text == s;
  };
  isKw (s) {
    const t = this.peek();
    return t.kind == 0 && t.text == s;
  };
  acceptOp (s) {
    if ( this.isOp(s) ) {
      this.p = this.p + 1;
      return true;
    }
    return false;
  };
  acceptKw (s) {
    if ( this.isKw(s) ) {
      this.p = this.p + 1;
      return true;
    }
    return false;
  };
  describe (t) {
    if ( t.kind == 6 ) {
      return "end of line";
    }
    if ( t.kind == 9 ) {
      return "end of input";
    }
    if ( t.kind == 7 ) {
      return "indent";
    }
    if ( t.kind == 8 ) {
      return "dedent";
    }
    if ( t.kind == 3 || t.kind == 4 ) {
      return "string";
    }
    return ("'" + t.text) + "'";
  };
  expectOp (s) {
    if ( this.acceptOp(s) ) {
      return;
    }
    this.fail((("expected '" + s) + "', got ") + this.describe(this.peek()));
  };
  expectKw (s) {
    if ( this.acceptKw(s) ) {
      return;
    }
    this.fail((("expected '" + s) + "', got ") + this.describe(this.peek()));
  };
  expectName () {
    const t = this.peek();
    if ( t.kind == 0 && PyParser.isKeyword(t.text) == false ) {
      this.p = this.p + 1;
      return t.text;
    }
    this.fail("expected a name, got " + this.describe(t));
    return "";
  };
  expectNewline () {
    if ( this.peekKind() == 6 ) {
      this.p = this.p + 1;
      return;
    }
    if ( this.peekKind() == 9 ) {
      return;
    }
    this.fail("invalid syntax: unexpected " + this.describe(this.peek()));
  };
  parseStatementInto (out) {
    const t = this.peek();
    if ( t.kind == 7 ) {
      this.fail("unexpected indent");
    }
    if ( t.kind == 0 ) {
      const w = t.text;
      if ( w == "if" ) {
        out.push(this.parseIf());
        return;
      }
      if ( w == "match" && this.isMatchStart() ) {
        out.push(this.parseMatch());
        return;
      }
      if ( w == "while" ) {
        out.push(this.parseWhile());
        return;
      }
      if ( w == "for" ) {
        out.push(this.parseFor());
        return;
      }
      if ( w == "def" ) {
        let decos = [];
        out.push(this.parseDef(decos));
        return;
      }
      if ( w == "class" ) {
        let decos2 = [];
        out.push(this.parseClass(decos2));
        return;
      }
      if ( w == "try" ) {
        out.push(this.parseTry());
        return;
      }
      if ( w == "with" ) {
        out.push(this.parseWith());
        return;
      }
      const __REGx1 = this.peekAt(1);
      if ( w == "async" && __REGx1.text == "def" ) {
        this.p = this.p + 1;
        let decos3 = [];
        out.push(this.parseDef(decos3));
        return;
      }
      const __REGx2 = this.peekAt(1);
      const __REGx3 = this.peekAt(1);
      if ( w == "async" && (__REGx2.text == "for" || __REGx3.text == "with") ) {
        this.p = this.p + 1;
        this.parseStatementInto(out);
        return;
      }
    }
    if ( this.isOp("@") ) {
      let ds = [];
      while (this.acceptOp("@")) {
        ds.push(this.parseNamedExpr());
        this.expectNewline();
      };
      if ( this.isKw("def") ) {
        out.push(this.parseDef(ds));
        return;
      }
      const __REGx4 = this.peekAt(1);
      if ( this.isKw("async") && __REGx4.text == "def" ) {
        this.p = this.p + 1;
        out.push(this.parseDef(ds));
        return;
      }
      if ( this.isKw("class") ) {
        out.push(this.parseClass(ds));
        return;
      }
      this.fail("a decorator must be followed by def or class");
    }
    this.parseSimpleLineInto(out);
  };
  parseTypeParams () {
    let out = [];
    if ( this.acceptOp("[") == false ) {
      return out;
    }
    while (this.isOp("]") == false) {
      if ( this.acceptOp("*") == false ) {
        this.acceptOp("**");
      }
      out.push(this.expectName());
      if ( this.acceptOp(":") ) {
        this.parseTest();
      }
      if ( this.acceptOp("=") ) {
        this.parseTestOrStar();
      }
      if ( this.acceptOp(",") == false ) {
        break;
      }
    };
    this.expectOp("]");
    return out;
  };
  parseSimpleLineInto (out) {
    const tt = this.peek();
    if ( tt.kind == 0 && tt.text == "type" ) {
      const nt = this.peekAt(1);
      if ( nt.kind == 0 ) {
        const at = this.peekAt(2);
        if ( at.text == "=" || at.text == "[" ) {
          const ta = new PyNode(53, tt.line);
          this.p = this.p + 2;
          ta.s = nt.text;
          ta.tparams = this.parseTypeParams();
          this.expectOp("=");
          ta.k1 = this.parseTest();
          out.push(ta);
          this.expectNewline();
          return;
        }
      }
    }
    out.push(this.parseSmallStatement());
    while (this.acceptOp(";")) {
      if ( this.peekKind() == 6 || this.peekKind() == 9 ) {
        break;
      }
      out.push(this.parseSmallStatement());
    };
    this.expectNewline();
  };
  parseBlock () {
    this.expectOp(":");
    let out = [];
    if ( this.peekKind() == 6 ) {
      this.p = this.p + 1;
      if ( this.peekKind() != 7 ) {
        this.fail("expected an indented block");
      }
      this.p = this.p + 1;
      while (this.peekKind() != 8 && this.peekKind() != 9) {
        if ( this.peekKind() == 6 ) {
          this.p = this.p + 1;
          continue;
        }
        this.parseStatementInto(out);
      };
      if ( this.peekKind() == 8 ) {
        this.p = this.p + 1;
      }
      return out;
    }
    this.parseSimpleLineInto(out);
    return out;
  };
  isMatchStart () {
    let k = 1;
    let prev = this.peek();
    while (true) {
      const tk = this.peekAt(k);
      if ( tk.kind == 6 || tk.kind == 9 ) {
        break;
      }
      prev = tk;
      k = k + 1;
    };
    if ( k == 1 ) {
      return false;
    }
    if ( prev.kind != 5 || prev.text != ":" ) {
      return false;
    }
    const a = this.peekAt((k + 1));
    const b = this.peekAt((k + 2));
    return (a.kind == 7 && b.kind == 0) && b.text == "case";
  };
  parseMatch () {
    const node = new PyNode(66, this.line());
    this.p = this.p + 1;
    node.k1 = this.parseTestListStarExpr();
    this.expectOp(":");
    this.expectNewline();
    if ( this.peekKind() != 7 ) {
      this.fail("expected an indented block");
    }
    this.p = this.p + 1;
    while (this.peekKind() != 8 && this.peekKind() != 9) {
      if ( this.peekKind() == 6 ) {
        this.p = this.p + 1;
        continue;
      }
      const t = this.peek();
      if ( t.kind != 0 || t.text != "case" ) {
        this.fail("expected 'case'");
      }
      const cs = new PyNode(67, t.line);
      this.p = this.p + 1;
      cs.k1 = this.parsePatternTop();
      if ( this.acceptKw("if") ) {
        cs.k2 = this.parseNamedExpr();
      }
      cs.body = this.parseBlock();
      node.kids.push(cs);
    };
    if ( this.peekKind() == 8 ) {
      this.p = this.p + 1;
    }
    return node;
  };
  pat (kind) {
    const n = new PyNode(68, this.line());
    n.s = kind;
    return n;
  };
  parsePatternTop () {
    const first = this.parseMaybeStarPattern();
    if ( this.isOp(",") == false ) {
      if ( first.s == "star" ) {
        this.fail("can't use starred expression here");
      }
      return first;
    }
    const sq = this.pat("seq");
    sq.kids.push(first);
    while (this.acceptOp(",")) {
      if ( this.isOp(":") || this.isKw("if") ) {
        break;
      }
      sq.kids.push(this.parseMaybeStarPattern());
    };
    return sq;
  };
  parseMaybeStarPattern () {
    if ( this.acceptOp("*") ) {
      const st = this.pat("star");
      const nm = this.expectName();
      if ( nm == "_" ) {
        st.strs.push("");
      } else {
        st.strs.push(nm);
      }
      return st;
    }
    return this.parseAsPattern();
  };
  parseAsPattern () {
    const orp = this.parseOrPattern();
    if ( this.acceptKw("as") ) {
      const asn = this.pat("as");
      asn.k1 = orp;
      asn.strs.push(this.expectName());
      return asn;
    }
    return orp;
  };
  parseOrPattern () {
    const first = this.parseClosedPattern();
    if ( this.isOp("|") == false ) {
      return first;
    }
    const o = this.pat("or");
    o.kids.push(first);
    while (this.acceptOp("|")) {
      o.kids.push(this.parseClosedPattern());
    };
    return o;
  };
  parseClosedPattern () {
    const t = this.peek();
    if ( (t.kind == 1 || t.kind == 2) || this.isOp("-") ) {
      const v = this.pat("value");
      v.k1 = this.parseArith();
      return v;
    }
    if ( t.kind == 3 || t.kind == 4 ) {
      const sv = this.pat("value");
      sv.k1 = this.parseAtom();
      return sv;
    }
    if ( t.kind == 0 ) {
      if ( (t.text == "None" || t.text == "True") || t.text == "False" ) {
        const kv = this.pat("value");
        kv.k1 = this.parseAtom();
        kv.i = 1;
        return kv;
      }
      this.p = this.p + 1;
      let ex = new PyNode(0, t.line);
      ex.s = t.text;
      let dotted = false;
      while (this.isOp(".")) {
        this.p = this.p + 1;
        const at = new PyNode(27, t.line);
        at.k1 = ex;
        at.s = this.expectName();
        ex = at;
        dotted = true;
      };
      if ( this.isOp("(") ) {
        return this.parseClassPattern(ex);
      }
      if ( dotted ) {
        const dv = this.pat("value");
        dv.k1 = ex;
        return dv;
      }
      if ( t.text == "_" ) {
        return this.pat("wild");
      }
      const cp = this.pat("capture");
      cp.strs.push(t.text);
      return cp;
    }
    if ( this.acceptOp("(") ) {
      if ( this.acceptOp(")") ) {
        return this.pat("seq");
      }
      const inner = this.parseMaybeStarPattern();
      if ( this.isOp(",") ) {
        const sq2 = this.pat("seq");
        sq2.kids.push(inner);
        while (this.acceptOp(",")) {
          if ( this.isOp(")") ) {
            break;
          }
          sq2.kids.push(this.parseMaybeStarPattern());
        };
        this.expectOp(")");
        return sq2;
      }
      this.expectOp(")");
      return inner;
    }
    if ( this.acceptOp("[") ) {
      const sq3 = this.pat("seq");
      while (this.isOp("]") == false) {
        sq3.kids.push(this.parseMaybeStarPattern());
        if ( this.acceptOp(",") == false ) {
          break;
        }
      };
      this.expectOp("]");
      return sq3;
    }
    if ( this.acceptOp("{") ) {
      const mp = this.pat("map");
      while (this.isOp("}") == false) {
        if ( this.acceptOp("**") ) {
          mp.strs.push(this.expectName());
        } else {
          const keyn = this.parseClosedPattern();
          if ( keyn.s != "value" || (typeof(keyn.k1) === "undefined") ) {
            this.fail("mapping pattern keys may only match literals and attribute lookups");
          }
          mp.kids.push(keyn.k1);
          this.expectOp(":");
          mp.orelse.push(this.parseAsPattern());
        }
        if ( this.acceptOp(",") == false ) {
          break;
        }
      };
      this.expectOp("}");
      return mp;
    }
    this.fail("invalid syntax: unexpected " + this.describe(t));
    return this.pat("wild");
  };
  parseClassPattern (cls) {
    const cp = this.pat("class");
    cp.k1 = cls;
    this.expectOp("(");
    while (this.isOp(")") == false) {
      const t = this.peek();
      if ( t.kind == 0 && this.isOpAt(1, "=") ) {
        this.p = this.p + 2;
        cp.names.push(t.text);
        cp.orelse.push(this.parseAsPattern());
      } else {
        if ( cp.names.length > 0 ) {
          this.fail("positional patterns follow keyword patterns");
        }
        cp.kids.push(this.parseAsPattern());
      }
      if ( this.acceptOp(",") == false ) {
        break;
      }
    };
    this.expectOp(")");
    return cp;
  };
  parseIf () {
    const node = new PyNode(44, this.line());
    this.p = this.p + 1;
    node.k1 = this.parseNamedExpr();
    node.body = this.parseBlock();
    if ( this.isKw("elif") ) {
      const inner = this.parseIf();
      node.orelse.push(inner);
      return node;
    }
    if ( this.acceptKw("else") ) {
      node.orelse = this.parseBlock();
    }
    return node;
  };
  parseWhile () {
    const node = new PyNode(45, this.line());
    this.p = this.p + 1;
    node.k1 = this.parseNamedExpr();
    node.body = this.parseBlock();
    if ( this.acceptKw("else") ) {
      node.orelse = this.parseBlock();
    }
    return node;
  };
  parseFor () {
    const node = new PyNode(46, this.line());
    this.p = this.p + 1;
    node.k1 = this.parseExprList();
    this.expectKw("in");
    node.k2 = this.parseTestList();
    node.body = this.parseBlock();
    if ( this.acceptKw("else") ) {
      node.orelse = this.parseBlock();
    }
    return node;
  };
  parseTry () {
    const node = new PyNode(54, this.line());
    this.p = this.p + 1;
    node.body = this.parseBlock();
    while (this.isKw("except")) {
      const h = new PyNode(55, this.line());
      this.p = this.p + 1;
      if ( this.acceptOp("*") ) {
        node.i = 1;
      }
      if ( this.isOp(":") == false ) {
        h.k1 = this.parseTest();
        if ( this.acceptOp(",") ) {
          this.fail("multiple exception types must be parenthesized");
        }
        if ( this.acceptKw("as") ) {
          h.s = this.expectName();
        }
      }
      h.body = this.parseBlock();
      node.handlers.push(h);
    };
    if ( this.acceptKw("else") ) {
      node.orelse = this.parseBlock();
    }
    if ( this.acceptKw("finally") ) {
      node.final = this.parseBlock();
    }
    if ( node.handlers.length == 0 && node.final.length == 0 ) {
      this.fail("expected 'except' or 'finally' block");
    }
    return node;
  };
  parseWith () {
    const node = new PyNode(63, this.line());
    this.p = this.p + 1;
    let paren = false;
    if ( this.isOp("(") ) {
      const save = this.p;
      let depth = 0;
      let q = this.p;
      let found = false;
      while (q < this.toks.length) {
        const t = this.toks[q];
        if ( t.kind == 5 && ((t.text == "(" || t.text == "[") || t.text == "{") ) {
          depth = depth + 1;
        }
        if ( t.kind == 5 && ((t.text == ")" || t.text == "]") || t.text == "}") ) {
          depth = depth - 1;
          if ( depth == 0 ) {
            break;
          }
        }
        if ( (depth == 1 && t.kind == 0) && t.text == "as" ) {
          found = true;
        }
        q = q + 1;
      };
      if ( found ) {
        paren = true;
        this.p = save + 1;
      }
    }
    while (true) {
      const item = new PyNode(64, this.line());
      item.k1 = this.parseTest();
      if ( this.acceptKw("as") ) {
        item.k2 = this.parseTarget();
      }
      node.kids.push(item);
      if ( this.acceptOp(",") == false ) {
        break;
      }
      if ( paren && this.isOp(")") ) {
        break;
      }
    };
    if ( paren ) {
      this.expectOp(")");
    }
    node.body = this.parseBlock();
    return node;
  };
  parseTarget () {
    return this.parseBitOr();
  };
  curPrefix () {
    if ( this.qualPrefix.length == 0 ) {
      return "";
    }
    return this.qualPrefix[(this.qualPrefix.length - 1)];
  };
  mangle (name) {
    if ( this.mangleIn.length == 0 ) {
      return name;
    }
    const cls = this.mangleIn[(this.mangleIn.length - 1)];
    if ( ((cls.length == 0 || name.startsWith("__") == false) || name.endsWith("__")) || name.indexOf(".") >= 0 ) {
      return name;
    }
    let k = 0;
    while (k < cls.length && cls.substring(k, k + 1 ) == "_") {
      k = k + 1;
    };
    if ( k >= cls.length ) {
      return name;
    }
    return ("_" + cls.substring(k, cls.length )) + name;
  };
  parseDef (decos) {
    const node = new PyNode(51, this.line());
    this.p = this.p + 1;
    node.s = this.mangle(this.expectName());
    node.qual = this.curPrefix() + node.s;
    this.qualPrefix.push(node.qual + ".<locals>.");
    node.decos = decos;
    node.tparams = this.parseTypeParams();
    this.expectOp("(");
    node.kids = this.parseParams(")", true);
    this.expectOp(")");
    if ( this.acceptOp("->") ) {
      const ra = this.parseTest();
      if ( this.futureAnn == false ) {
        node.k3 = ra;
      }
    }
    this.yields.push(false);
    node.body = this.parseBlock();
    if ( this.yields[(this.yields.length - 1)] ) {
      node.i = 1;
    }
    this.yields.pop();
    this.qualPrefix.pop();
    return node;
  };
  parseParams (close, annotations) {
    let out = [];
    while (this.isOp(close) == false) {
      const prm = new PyNode(39, this.line());
      if ( this.acceptOp("**") ) {
        prm.i = 2;
        prm.s = this.mangle(this.expectName());
      } else {
        if ( this.acceptOp("*") ) {
          if ( this.isOp(",") || this.isOp(close) ) {
            prm.i = 3;
          } else {
            prm.i = 1;
            prm.s = this.mangle(this.expectName());
          }
        } else {
          if ( this.acceptOp("/") ) {
            prm.i = 4;
          } else {
            prm.s = this.mangle(this.expectName());
          }
        }
      }
      if ( annotations && ((prm.i == 0 || prm.i == 1) || prm.i == 2) ) {
        if ( this.acceptOp(":") ) {
          const pa = this.parseTestOrStar();
          if ( this.futureAnn == false ) {
            prm.k2 = pa;
          }
        }
      }
      if ( prm.i == 0 && this.acceptOp("=") ) {
        prm.k1 = this.parseTest();
      }
      out.push(prm);
      if ( this.acceptOp(",") == false ) {
        break;
      }
    };
    return out;
  };
  parseClass (decos) {
    const node = new PyNode(52, this.line());
    this.p = this.p + 1;
    const raw = this.expectName();
    node.s = this.mangle(raw);
    node.qual = this.curPrefix() + node.s;
    node.decos = decos;
    node.tparams = this.parseTypeParams();
    if ( this.acceptOp("(") ) {
      node.kids = this.parseCallArgs();
      this.expectOp(")");
    }
    this.yields.push(false);
    this.qualPrefix.push(node.qual + ".");
    this.mangleIn.push(raw);
    node.body = this.parseBlock();
    this.mangleIn.pop();
    this.qualPrefix.pop();
    this.yields.pop();
    return node;
  };
  parseSmallStatement () {
    const t = this.peek();
    const ln = t.line;
    if ( t.kind == 0 ) {
      const w = t.text;
      if ( w == "pass" ) {
        this.p = this.p + 1;
        return new PyNode(49, ln);
      }
      if ( w == "break" ) {
        this.p = this.p + 1;
        return new PyNode(47, ln);
      }
      if ( w == "continue" ) {
        this.p = this.p + 1;
        return new PyNode(48, ln);
      }
      if ( w == "return" ) {
        this.p = this.p + 1;
        const r = new PyNode(50, ln);
        if ( this.atStatementEnd() == false ) {
          r.k1 = this.parseTestListStarExpr();
        }
        return r;
      }
      if ( w == "raise" ) {
        this.p = this.p + 1;
        const r2 = new PyNode(56, ln);
        if ( this.atStatementEnd() == false ) {
          r2.k1 = this.parseTest();
          if ( this.acceptKw("from") ) {
            r2.k2 = this.parseTest();
          }
        }
        return r2;
      }
      if ( w == "global" || w == "nonlocal" ) {
        this.p = this.p + 1;
        const g = new PyNode(59, ln);
        if ( w == "nonlocal" ) {
          g.kind = 60;
        }
        const gn = this.expectName();
        g.strs.push(this.mangle(gn));
        while (this.acceptOp(",")) {
          const gn_1 = this.expectName();
          g.strs.push(this.mangle(gn_1));
        };
        return g;
      }
      if ( w == "import" ) {
        this.p = this.p + 1;
        const im = new PyNode(57, ln);
        while (true) {
          im.strs.push(this.parseDottedName());
          if ( this.acceptKw("as") ) {
            im.names.push(this.expectName());
          } else {
            im.names.push("");
          }
          if ( this.acceptOp(",") == false ) {
            break;
          }
        };
        return im;
      }
      if ( w == "from" ) {
        this.p = this.p + 1;
        const fi = new PyNode(58, ln);
        let dots = "";
        while (this.isOp(".") || this.isOp("...")) {
          const dt = this.peek();
          dots = dots + dt.text;
          this.p = this.p + 1;
        };
        if ( this.isKw("import") ) {
          fi.s = dots;
        } else {
          fi.s = dots + this.parseDottedName();
        }
        const __REGx1 = this.peekAt(1);
        if ( fi.s == "__future__" && __REGx1.text == "annotations" ) {
          this.futureAnn = true;
        }
        this.expectKw("import");
        if ( this.acceptOp("*") ) {
          fi.strs.push("*");
          fi.names.push("");
          return fi;
        }
        const paren = this.acceptOp("(");
        while (true) {
          if ( paren && this.isOp(")") ) {
            break;
          }
          fi.strs.push(this.expectName());
          if ( this.acceptKw("as") ) {
            fi.names.push(this.expectName());
          } else {
            fi.names.push("");
          }
          if ( this.acceptOp(",") == false ) {
            break;
          }
        };
        if ( paren ) {
          this.expectOp(")");
        }
        return fi;
      }
      if ( w == "assert" ) {
        this.p = this.p + 1;
        const a = new PyNode(61, ln);
        a.k1 = this.parseTest();
        if ( this.acceptOp(",") ) {
          a.k2 = this.parseTest();
        }
        return a;
      }
      if ( w == "del" ) {
        this.p = this.p + 1;
        const d = new PyNode(62, ln);
        const tl = this.parseExprList();
        if ( tl.kind == 10 && tl.i == 1 ) {
          d.kids = tl.kids;
        } else {
          d.kids.push(tl);
        }
        return d;
      }
    }
    const first = this.parseTestListStarExpr();
    if ( this.isOp("=") ) {
      const asg = new PyNode(41, ln);
      asg.kids.push(first);
      let value = first;
      while (this.acceptOp("=")) {
        if ( this.isKw("yield") ) {
          value = this.parseYield();
        } else {
          value = this.parseTestListStarExpr();
        }
        asg.kids.push(value);
      };
      asg.kids.pop();
      asg.k1 = value;
      // Loop start
      for ( const tg of asg.kids) {
        this.checkTarget(tg);
      }
      return asg;
    }
    const at = this.peek();
    if ( (((((at.kind == 5 && at.text.length >= 2) && PyParser.endsWithEq(at.text)) && at.text != "==") && at.text != "<=") && at.text != ">=") && at.text != "!=" ) {
      this.p = this.p + 1;
      const aug = new PyNode(42, ln);
      aug.s = at.text.substring(0, at.text.length - 1 );
      this.checkTarget(first);
      aug.k1 = first;
      if ( this.isKw("yield") ) {
        aug.k2 = this.parseYield();
      } else {
        aug.k2 = this.parseTestList();
      }
      return aug;
    }
    if ( this.acceptOp(":") ) {
      const ann = new PyNode(43, ln);
      ann.k1 = first;
      const an = this.parseTest();
      if ( this.futureAnn == false ) {
        ann.k3 = an;
      }
      if ( this.acceptOp("=") ) {
        ann.k2 = this.parseTestListStarExpr();
      }
      return ann;
    }
    const es = new PyNode(40, ln);
    es.k1 = first;
    return es;
  };
  checkTarget (t) {
    if ( (t.kind == 0 || t.kind == 27) || t.kind == 28 ) {
      return;
    }
    if ( t.kind == 10 || t.kind == 11 ) {
      // Loop start
      for ( const k of t.kids) {
        this.checkTarget(k);
      }
      return;
    }
    if ( t.kind == 25 ) {
      const inner = t.k1;
      this.checkTarget(inner);
      return;
    }
    this.failAt("cannot assign to expression", t.line);
  };
  atStatementEnd () {
    const k = this.peekKind();
    if ( k == 6 || k == 9 ) {
      return true;
    }
    return this.isOp(";");
  };
  parseDottedName () {
    let s = this.expectName();
    while (this.acceptOp(".")) {
      s = (s + ".") + this.expectName();
    };
    return s;
  };
  parseTestListStarExpr () {
    const ln = this.line();
    const first = this.parseTestOrStar();
    if ( this.isOp(",") == false ) {
      return first;
    }
    const tup = new PyNode(10, ln);
    tup.i = 1;
    tup.kids.push(first);
    while (this.acceptOp(",")) {
      if ( this.atExprEnd() ) {
        break;
      }
      tup.kids.push(this.parseTestOrStar());
    };
    return tup;
  };
  parseTestList () {
    return this.parseTestListStarExpr();
  };
  parseExprList () {
    const ln = this.line();
    const first = this.parseStarOrBitOr();
    if ( this.isOp(",") == false ) {
      return first;
    }
    const tup = new PyNode(10, ln);
    tup.i = 1;
    tup.kids.push(first);
    while (this.acceptOp(",")) {
      if ( this.isKw("in") || this.atExprEnd() ) {
        break;
      }
      tup.kids.push(this.parseStarOrBitOr());
    };
    return tup;
  };
  parseStarOrBitOr () {
    if ( this.isOp("*") ) {
      const st = new PyNode(25, this.line());
      this.p = this.p + 1;
      st.k1 = this.parseBitOr();
      return st;
    }
    return this.parseBitOr();
  };
  atExprEnd () {
    const t = this.peek();
    if ( ((t.kind == 6 || t.kind == 9) || t.kind == 7) || t.kind == 8 ) {
      return true;
    }
    if ( t.kind == 5 ) {
      const s = t.text;
      if ( ((((s == ")" || s == "]") || s == "}") || s == "=") || s == ";") || s == ":" ) {
        return true;
      }
      if ( s.length >= 2 ) {
        if ( (((s.endsWith("=") && s != "==") && s != "<=") && s != ">=") && s != "!=" ) {
          return true;
        }
      }
    }
    return false;
  };
  parseTestOrStar () {
    if ( this.isOp("*") ) {
      const st = new PyNode(25, this.line());
      this.p = this.p + 1;
      st.k1 = this.parseBitOr();
      return st;
    }
    if ( this.isKw("yield") ) {
      return this.parseYield();
    }
    return this.parseNamedExpr();
  };
  parseYield () {
    const ln = this.line();
    this.p = this.p + 1;
    if ( this.yields.length > 0 ) {
      this.yields.pop();
      this.yields.push(true);
    }
    if ( this.acceptKw("from") ) {
      const yf = new PyNode(38, ln);
      yf.k1 = this.parseTest();
      return yf;
    }
    const y = new PyNode(37, ln);
    if ( this.atExprEnd() == false ) {
      y.k1 = this.parseTestListStarExpr();
    }
    return y;
  };
  parseNamedExpr () {
    const t = this.peek();
    if ( t.kind == 0 && this.isOpAt(1, ":=") ) {
      const w = new PyNode(36, t.line);
      w.s = t.text;
      this.p = this.p + 2;
      w.k1 = this.parseTest();
      return w;
    }
    return this.parseTest();
  };
  parseTest () {
    if ( this.isKw("lambda") ) {
      return this.parseLambda();
    }
    const ln = this.line();
    const body = this.parseOr();
    if ( this.isKw("if") ) {
      const save = this.p;
      this.p = this.p + 1;
      const test = this.parseOr();
      if ( this.acceptKw("else") ) {
        const ie = new PyNode(21, ln);
        ie.k1 = body;
        ie.k2 = test;
        ie.k3 = this.parseTest();
        return ie;
      }
      this.p = save;
    }
    return body;
  };
  parseTestNoCond () {
    if ( this.isKw("lambda") ) {
      return this.parseLambda();
    }
    return this.parseOr();
  };
  parseLambda () {
    const node = new PyNode(22, this.line());
    this.p = this.p + 1;
    node.kids = this.parseParams(":", false);
    this.expectOp(":");
    this.yields.push(false);
    node.k1 = this.parseTest();
    this.yields.pop();
    return node;
  };
  parseOr () {
    let left = this.parseAnd();
    while (this.isKw("or")) {
      const n = new PyNode(19, this.line());
      this.p = this.p + 1;
      n.k1 = left;
      n.k2 = this.parseAnd();
      left = n;
    };
    return left;
  };
  parseAnd () {
    let left = this.parseNot();
    while (this.isKw("and")) {
      const n = new PyNode(18, this.line());
      this.p = this.p + 1;
      n.k1 = left;
      n.k2 = this.parseNot();
      left = n;
    };
    return left;
  };
  parseNot () {
    if ( this.isKw("not") ) {
      const n = new PyNode(17, this.line());
      this.p = this.p + 1;
      n.k1 = this.parseNot();
      return n;
    }
    return this.parseComparison();
  };
  compOp () {
    const t = this.peek();
    if ( t.kind == 5 ) {
      const s = t.text;
      if ( ((((s == "<" || s == ">") || s == "==") || s == "!=") || s == "<=") || s == ">=" ) {
        this.p = this.p + 1;
        return s;
      }
      return "";
    }
    if ( t.kind == 0 ) {
      if ( t.text == "in" ) {
        this.p = this.p + 1;
        return "in";
      }
      if ( t.text == "is" ) {
        this.p = this.p + 1;
        if ( this.acceptKw("not") ) {
          return "is not";
        }
        return "is";
      }
      const __REGx1 = this.peekAt(1);
      if ( t.text == "not" && __REGx1.text == "in" ) {
        this.p = this.p + 2;
        return "not in";
      }
    }
    return "";
  };
  parseComparison () {
    const ln = this.line();
    const first = this.parseBitOr();
    let op = this.compOp();
    if ( op.length == 0 ) {
      return first;
    }
    const c = new PyNode(20, ln);
    c.kids.push(first);
    while (op.length > 0) {
      c.strs.push(op);
      c.kids.push(this.parseBitOr());
      op = this.compOp();
    };
    return c;
  };
  bin (op, a, b, ln) {
    const n = new PyNode(15, ln);
    n.s = op;
    n.k1 = a;
    n.k2 = b;
    return n;
  };
  parseBitOr () {
    let left = this.parseBitXor();
    while (this.isOp("|")) {
      const ln = this.line();
      this.p = this.p + 1;
      left = this.bin("|", left, this.parseBitXor(), ln);
    };
    return left;
  };
  parseBitXor () {
    let left = this.parseBitAnd();
    while (this.isOp("^")) {
      const ln = this.line();
      this.p = this.p + 1;
      left = this.bin("^", left, this.parseBitAnd(), ln);
    };
    return left;
  };
  parseBitAnd () {
    let left = this.parseShift();
    while (this.isOp("&")) {
      const ln = this.line();
      this.p = this.p + 1;
      left = this.bin("&", left, this.parseShift(), ln);
    };
    return left;
  };
  parseShift () {
    let left = this.parseArith();
    while (this.isOp("<<") || this.isOp(">>")) {
      const t = this.peek();
      this.p = this.p + 1;
      left = this.bin(t.text, left, this.parseArith(), t.line);
    };
    return left;
  };
  parseArith () {
    let left = this.parseTerm();
    while (this.isOp("+") || this.isOp("-")) {
      const t = this.peek();
      this.p = this.p + 1;
      left = this.bin(t.text, left, this.parseTerm(), t.line);
    };
    return left;
  };
  parseTerm () {
    let left = this.parseFactor();
    while ((((this.isOp("*") || this.isOp("/")) || this.isOp("//")) || this.isOp("%")) || this.isOp("@")) {
      const t = this.peek();
      this.p = this.p + 1;
      left = this.bin(t.text, left, this.parseFactor(), t.line);
    };
    return left;
  };
  parseFactor () {
    if ( (this.isOp("-") || this.isOp("+")) || this.isOp("~") ) {
      const t = this.peek();
      this.p = this.p + 1;
      const u = new PyNode(16, t.line);
      u.s = t.text;
      u.k1 = this.parseFactor();
      return u;
    }
    return this.parsePower();
  };
  parsePower () {
    if ( this.isKw("await") ) {
      this.p = this.p + 1;
    }
    const base = this.parseAtomTrailers();
    if ( this.isOp("**") ) {
      const ln = this.line();
      this.p = this.p + 1;
      return this.bin("**", base, this.parseFactor(), ln);
    }
    return base;
  };
  parseAtomTrailers () {
    let e = this.parseAtom();
    while (true) {
      if ( this.isOp("(") ) {
        const c = new PyNode(23, this.line());
        this.p = this.p + 1;
        c.k1 = e;
        c.kids = this.parseCallArgs();
        this.expectOp(")");
        e = c;
        continue;
      }
      if ( this.isOp("[") ) {
        const s = new PyNode(28, this.line());
        this.p = this.p + 1;
        s.k1 = e;
        s.k2 = this.parseSubscriptList();
        this.expectOp("]");
        e = s;
        continue;
      }
      if ( this.isOp(".") ) {
        const a = new PyNode(27, this.line());
        this.p = this.p + 1;
        a.k1 = e;
        const nt = this.peek();
        if ( nt.kind != 0 ) {
          this.fail("expected an attribute name after '.'");
        }
        this.p = this.p + 1;
        a.s = this.mangle(nt.text);
        e = a;
        continue;
      }
      break;
    };
    return e;
  };
  parseCallArgs () {
    let out = [];
    while (this.isOp(")") == false) {
      const t = this.peek();
      if ( this.acceptOp("**") ) {
        const ss = new PyNode(26, t.line);
        ss.k1 = this.parseTest();
        out.push(ss);
      } else {
        if ( this.acceptOp("*") ) {
          const st = new PyNode(25, t.line);
          st.k1 = this.parseTest();
          out.push(st);
        } else {
          if ( t.kind == 0 && this.isOpAt(1, "=") ) {
            this.p = this.p + 2;
            const kw = new PyNode(24, t.line);
            kw.s = this.mangle(t.text);
            kw.k1 = this.parseTest();
            out.push(kw);
          } else {
            const a = this.parseNamedExpr();
            if ( this.isKw("for") ) {
              const g = new PyNode(32, t.line);
              g.k1 = a;
              g.kids = this.parseCompClauses();
              out.push(g);
            } else {
              out.push(a);
            }
          }
        }
      }
      if ( this.acceptOp(",") == false ) {
        break;
      }
    };
    return out;
  };
  parseSubscriptList () {
    const ln = this.line();
    const first = this.parseSubscript();
    if ( this.isOp(",") == false ) {
      return first;
    }
    const tup = new PyNode(10, ln);
    tup.kids.push(first);
    while (this.acceptOp(",")) {
      if ( this.isOp("]") ) {
        break;
      }
      tup.kids.push(this.parseSubscript());
    };
    return tup;
  };
  parseSubscript () {
    const ln = this.line();
    let lo;
    if ( this.isOp(":") == false ) {
      lo = this.parseNamedExpr();
      if ( this.isOp(":") == false ) {
        return lo;
      }
    }
    const sl = new PyNode(29, ln);
    sl.k1 = lo;
    this.expectOp(":");
    if ( (this.isOp(":") == false && this.isOp("]") == false) && this.isOp(",") == false ) {
      sl.k2 = this.parseTest();
    }
    if ( this.acceptOp(":") ) {
      if ( this.isOp("]") == false && this.isOp(",") == false ) {
        sl.k3 = this.parseTest();
      }
    }
    return sl;
  };
  parseCompClauses () {
    let out = [];
    while (true) {
      if ( this.isKw("async") ) {
        this.p = this.p + 1;
      }
      if ( this.isKw("for") ) {
        const cf = new PyNode(34, this.line());
        this.p = this.p + 1;
        cf.k1 = this.parseExprList();
        this.expectKw("in");
        cf.k2 = this.parseOr();
        out.push(cf);
        continue;
      }
      if ( this.isKw("if") ) {
        const ci = new PyNode(35, this.line());
        this.p = this.p + 1;
        ci.k1 = this.parseTestNoCond();
        out.push(ci);
        continue;
      }
      break;
    };
    return out;
  };
  parseAtom () {
    const t = this.peek();
    const ln = t.line;
    if ( t.kind == 0 ) {
      const w = t.text;
      if ( w == "None" ) {
        this.p = this.p + 1;
        return new PyNode(8, ln);
      }
      if ( w == "True" ) {
        this.p = this.p + 1;
        return new PyNode(6, ln);
      }
      if ( w == "False" ) {
        this.p = this.p + 1;
        return new PyNode(7, ln);
      }
      if ( PyParser.isKeyword(w) ) {
        this.fail(("invalid syntax near '" + w) + "'");
      }
      this.p = this.p + 1;
      const nm = new PyNode(0, ln);
      nm.s = this.mangle(w);
      return nm;
    }
    if ( t.kind == 1 ) {
      this.p = this.p + 1;
      const il = new PyNode(1, ln);
      il.s = t.text;
      const iv = isNaN( parseInt(t.text) ) ? undefined : parseInt(t.text);
      if ( typeof(iv) === "undefined" ) {
        il.kind = 2;
        il.d = Number(t.text);
      } else {
        il.i = iv;
        if ( t.text.length > 15 ) {
          const dv = Number(t.text);
          if ( dv > 9007199254740991.0 || dv < -9007199254740991.0 ) {
            il.kind = 2;
            il.d = dv;
            il.i = 1;
          }
        }
      }
      return il;
    }
    if ( t.kind == 2 ) {
      this.p = this.p + 1;
      const fl = new PyNode(2, ln);
      if ( t.text.endsWith("j") ) {
        fl.d = Number(t.text.substring(0, t.text.length - 1 ));
        fl.i = 2;
        return fl;
      }
      fl.d = Number(t.text);
      return fl;
    }
    if ( t.kind == 3 || t.kind == 4 ) {
      return this.parseStrings();
    }
    if ( t.kind == 5 ) {
      if ( t.text == "(" ) {
        this.p = this.p + 1;
        if ( this.acceptOp(")") ) {
          return new PyNode(10, ln);
        }
        if ( this.isKw("yield") ) {
          const y = this.parseYield();
          this.expectOp(")");
          return y;
        }
        const first = this.parseTestOrStar();
        if ( this.isKw("for") ) {
          const g = new PyNode(32, ln);
          g.k1 = first;
          g.kids = this.parseCompClauses();
          this.expectOp(")");
          return g;
        }
        if ( this.acceptOp(")") ) {
          if ( first.kind == 25 ) {
            this.failAt("cannot use starred expression here", ln);
          }
          return first;
        }
        const tup = new PyNode(10, ln);
        tup.kids.push(first);
        while (this.acceptOp(",")) {
          if ( this.isOp(")") ) {
            break;
          }
          tup.kids.push(this.parseTestOrStar());
        };
        this.expectOp(")");
        return tup;
      }
      if ( t.text == "[" ) {
        this.p = this.p + 1;
        const lst = new PyNode(11, ln);
        if ( this.acceptOp("]") ) {
          return lst;
        }
        const first2 = this.parseTestOrStar();
        if ( this.isKw("for") ) {
          const lc = new PyNode(30, ln);
          lc.k1 = first2;
          lc.kids = this.parseCompClauses();
          this.expectOp("]");
          return lc;
        }
        lst.kids.push(first2);
        while (this.acceptOp(",")) {
          if ( this.isOp("]") ) {
            break;
          }
          lst.kids.push(this.parseTestOrStar());
        };
        this.expectOp("]");
        return lst;
      }
      if ( t.text == "{" ) {
        return this.parseBrace();
      }
      if ( t.text == "..." ) {
        this.p = this.p + 1;
        return new PyNode(9, ln);
      }
    }
    this.fail("invalid syntax: unexpected " + this.describe(t));
    return new PyNode(8, ln);
  };
  parseBrace () {
    const ln = this.line();
    this.p = this.p + 1;
    if ( this.acceptOp("}") ) {
      return new PyNode(13, ln);
    }
    if ( this.acceptOp("**") ) {
      const d = new PyNode(13, ln);
      const sp = new PyNode(14, ln);
      sp.k1 = this.parseBitOr();
      d.kids.push(sp);
      d.kids.push(new PyNode(8, ln));
      this.parseDictRest(d);
      return d;
    }
    const first = this.parseTestOrStar();
    if ( this.acceptOp(":") ) {
      const value = this.parseTest();
      if ( this.isKw("for") ) {
        const dc = new PyNode(33, ln);
        dc.k1 = first;
        dc.k2 = value;
        dc.kids = this.parseCompClauses();
        this.expectOp("}");
        return dc;
      }
      const d2 = new PyNode(13, ln);
      d2.kids.push(first);
      d2.kids.push(value);
      this.parseDictRest(d2);
      return d2;
    }
    if ( this.isKw("for") ) {
      const sc = new PyNode(31, ln);
      sc.k1 = first;
      sc.kids = this.parseCompClauses();
      this.expectOp("}");
      return sc;
    }
    const st = new PyNode(12, ln);
    st.kids.push(first);
    while (this.acceptOp(",")) {
      if ( this.isOp("}") ) {
        break;
      }
      st.kids.push(this.parseTestOrStar());
    };
    this.expectOp("}");
    return st;
  };
  parseDictRest (d) {
    while (this.acceptOp(",")) {
      if ( this.isOp("}") ) {
        break;
      }
      if ( this.acceptOp("**") ) {
        const sp = new PyNode(14, this.line());
        sp.k1 = this.parseBitOr();
        d.kids.push(sp);
        d.kids.push(new PyNode(8, sp.line));
        continue;
      }
      d.kids.push(this.parseTest());
      this.expectOp(":");
      d.kids.push(this.parseTest());
    };
    this.expectOp("}");
  };
  parseStrings () {
    const ln = this.line();
    let parts = [];
    let anyF = false;
    while (this.peekKind() == 3 || this.peekKind() == 4) {
      const t = this.peek();
      this.p = this.p + 1;
      if ( t.kind == 4 ) {
        anyF = true;
        const fs = this.parseFString(t.text, t.line);
        // Loop start
        for ( const k of fs.kids) {
          parts.push(k);
        }
      } else {
        const sl = new PyNode(3, t.line);
        sl.s = t.text;
        parts.push(sl);
      }
    };
    if ( anyF == false ) {
      let all = "";
      // Loop start
      for ( const pt of parts) {
        all = all + pt.s;
      }
      const one = new PyNode(3, ln);
      one.s = all;
      return one;
    }
    const f = new PyNode(4, ln);
    f.kids = parts;
    return f;
  };
  parseFString (body, line) {
    const f = new PyNode(4, line);
    const cs = Array.from(body, (rg_c) => rg_c.codePointAt(0));
    const n = cs.length;
    let lit = [];
    let i = 0;
    while (i < n) {
      const c = cs[i];
      if ( c == 123 ) {
        if ( i + 1 < n && cs[(i + 1)] == 123 ) {
          lit.push(123);
          i = i + 2;
          continue;
        }
        if ( lit.length > 0 ) {
          const sl = new PyNode(3, line);
          sl.s = py_rt_codes(lit, 0, lit.length);
          f.kids.push(sl);
          lit.length = 0;
        }
        let j = i + 1;
        let depth = 0;
        let quote = 0;
        while (j < n) {
          const d = cs[j];
          if ( quote != 0 ) {
            if ( d == quote ) {
              quote = 0;
            }
            j = j + 1;
            continue;
          }
          if ( d == 39 || d == 34 ) {
            quote = d;
            j = j + 1;
            continue;
          }
          if ( (d == 40 || d == 91) || d == 123 ) {
            depth = depth + 1;
          }
          if ( d == 41 || d == 93 ) {
            depth = depth - 1;
          }
          if ( d == 125 ) {
            if ( depth == 0 ) {
              break;
            }
            depth = depth - 1;
          }
          if ( depth == 0 ) {
            let nx = 0;
            if ( j + 1 < n ) {
              nx = cs[(j + 1)];
            }
            if ( d == 33 && nx != 61 ) {
              break;
            }
            if ( d == 58 ) {
              break;
            }
            if ( d == 61 && ((nx == 125 || nx == 33) || nx == 58) ) {
              const pv = cs[(j - 1)];
              if ( ((pv != 61 && pv != 33) && pv != 60) && pv != 62 ) {
                break;
              }
            }
          }
          j = j + 1;
        };
        if ( j >= n ) {
          this.failAt("f-string: expecting '}'", line);
        }
        const exprText = py_rt_codes(cs, i + 1, j);
        if ( exprText.trim().length == 0 ) {
          this.failAt("f-string: empty expression not allowed", line);
        }
        const part = new PyNode(5, line);
        const sub = new PyParser();
        part.k1 = sub.parseExpressionText(exprText, line);
        if ( sub.err.length > 0 ) {
          this.failAt(sub.err, line);
        }
        if ( cs[j] == 61 ) {
          part.i = 1;
          const dbg = new PyNode(3, line);
          dbg.s = py_rt_codes(cs, i + 1, j + 1);
          f.kids.push(dbg);
          j = j + 1;
        }
        if ( j < n && cs[j] == 33 ) {
          if ( j + 1 >= n ) {
            this.failAt("f-string: invalid conversion character", line);
          }
          part.s = String.fromCodePoint(cs[(j + 1)]);
          if ( (part.s != "r" && part.s != "s") && part.s != "a" ) {
            this.failAt("f-string: invalid conversion character", line);
          }
          j = j + 2;
        }
        if ( j < n && cs[j] == 58 ) {
          let k = j + 1;
          let sd = 0;
          while (k < n) {
            const e = cs[k];
            if ( e == 123 ) {
              sd = sd + 1;
            }
            if ( e == 125 ) {
              if ( sd == 0 ) {
                break;
              }
              sd = sd - 1;
            }
            k = k + 1;
          };
          if ( k >= n ) {
            this.failAt("f-string: expecting '}'", line);
          }
          part.k2 = this.parseFString(py_rt_codes(cs, j + 1, k), line);
          j = k;
        }
        if ( j >= n || cs[j] != 125 ) {
          this.failAt("f-string: expecting '}'", line);
        }
        if ( (part.i == 1 && part.s.length == 0) && (typeof(part.k2) === "undefined") ) {
          part.s = "r";
        }
        f.kids.push(part);
        i = j + 1;
        continue;
      }
      if ( c == 125 ) {
        if ( i + 1 < n && cs[(i + 1)] == 125 ) {
          lit.push(125);
          i = i + 2;
          continue;
        }
        this.failAt("f-string: single '}' is not allowed", line);
      }
      lit.push(c);
      i = i + 1;
    };
    if ( lit.length > 0 ) {
      const sl2 = new PyNode(3, line);
      sl2.s = py_rt_codes(lit, 0, lit.length);
      f.kids.push(sl2);
    }
    return f;
  };
}
PyParser.isKeyword = function(s) {
  const kws = ["False", "None", "True", "and", "as", "assert", "async", "await", "break", "class", "continue", "def", "del", "elif", "else", "except", "finally", "for", "from", "global", "if", "import", "in", "is", "lambda", "nonlocal", "not", "or", "pass", "raise", "return", "try", "while", "with", "yield"];
  // Loop start
  for ( const k of kws) {
    if ( k == s ) {
      return true;
    }
  }
  return false;
};
PyParser.endsWithEq = function(s) {
  return s.endsWith("=");
};
export class PyBuiltins  {
  constructor(interp) {
    this.ip = undefined;
    this.MISSING = new PyVal(0);
    this.ip = interp;
  }
  put (name) {
    const b = new PyVal(10);
    b.s = name;
    this.ip.builtins[name] = b;
  };
  excClassMake (name, base) {
    const c = new PyVal(11);
    c.s = name;
    c.isExc = true;
    c.mode = "builtin";
    if ( base.t == 11 ) {
      c.bases.push(base);
    }
    this.ip.computeMro(c);
    this.ip.builtins[name] = c;
    return c;
  };
  install () {
    const names = ["print", "len", "range", "enumerate", "zip", "map", "filter", "sorted", "reversed", "sum", "min", "max", "abs", "round", "int", "float", "complex", "str", "bool", "list", "tuple", "dict", "set", "frozenset", "isinstance", "issubclass", "type", "repr", "ascii", "chr", "ord", "hex", "oct", "bin", "divmod", "pow", "any", "all", "open", "format", "hash", "id", "iter", "next", "getattr", "setattr", "hasattr", "delattr", "callable", "input", "vars", "globals", "locals", "dir", "property", "staticmethod", "classmethod", "super", "object", "slice", "exit", "quit", "__object_init__", "__build_class__", "__import__", "eval", "exec", "compile"];
    // Loop start
    for ( const n of names) {
      this.put(n);
    }
    this.put("NotImplemented");
    const ni = this.ip.NOTIMPL;
    this.ip.builtins["NotImplemented"] = ni;
    this.put("Ellipsis");
    this.ip.builtins["__debug__"] = this.ip.TRUE;
    const base = this.excClassMake("BaseException", this.ip.NONE);
    const exc = this.excClassMake("Exception", base);
    this.excClassMake("SystemExit", base);
    this.excClassMake("KeyboardInterrupt", base);
    this.excClassMake("GeneratorExit", base);
    const arith = this.excClassMake("ArithmeticError", exc);
    this.excClassMake("ZeroDivisionError", arith);
    this.excClassMake("OverflowError", arith);
    this.excClassMake("FloatingPointError", arith);
    const look = this.excClassMake("LookupError", exc);
    this.excClassMake("IndexError", look);
    this.excClassMake("KeyError", look);
    const val = this.excClassMake("ValueError", exc);
    const uni = this.excClassMake("UnicodeError", val);
    this.excClassMake("UnicodeDecodeError", uni);
    this.excClassMake("UnicodeEncodeError", uni);
    this.excClassMake("TypeError", exc);
    const nm = this.excClassMake("NameError", exc);
    this.excClassMake("UnboundLocalError", nm);
    this.excClassMake("AttributeError", exc);
    const rt = this.excClassMake("RuntimeError", exc);
    this.excClassMake("RecursionError", rt);
    this.excClassMake("NotImplementedError", rt);
    this.excClassMake("StopIteration", exc);
    this.excClassMake("StopAsyncIteration", exc);
    this.excClassMake("AssertionError", exc);
    const imp = this.excClassMake("ImportError", exc);
    this.excClassMake("ModuleNotFoundError", imp);
    const os = this.excClassMake("OSError", exc);
    this.ip.builtins["IOError"] = os;
    this.ip.builtins["EnvironmentError"] = os;
    this.excClassMake("FileNotFoundError", os);
    this.excClassMake("FileExistsError", os);
    this.excClassMake("IsADirectoryError", os);
    this.excClassMake("NotADirectoryError", os);
    this.excClassMake("PermissionError", os);
    this.excClassMake("TimeoutError", os);
    this.excClassMake("SyntaxError", exc);
    this.excClassMake("EOFError", exc);
    this.excClassMake("MemoryError", exc);
    this.excClassMake("BufferError", exc);
    const warn = this.excClassMake("Warning", exc);
    this.excClassMake("UserWarning", warn);
    this.excClassMake("DeprecationWarning", warn);
    this.excClassMake("RuntimeWarning", warn);
    this.excClassMake("FutureWarning", warn);
  };
  kw (kn, kv, name) {
    let none;
    // Loop start
    for ( let i = 0; i < kn.length; i++) {
      var k = kn[i];
      if ( k == name ) {
        const optr103 = kv[i];
        return optr103;
      }
    }
    return none;
  };
  arg (args, i, kn, kv, name) {
    if ( i < args.length ) {
      return args[i];
    }
    const k = this.kw(kn, kv, name);
    if ( (typeof(k) !== "undefined" && k != null )  ) {
      return k;
    }
    return this.ip.NONE;
  };
  hasArg (args, i, kn, kv, name) {
    if ( i < args.length ) {
      return true;
    }
    const k = this.kw(kn, kv, name);
    return (typeof(k) !== "undefined" && k != null ) ;
  };
  need (fname, args, lo, hi) {
    const n = args.length;
    if ( n < lo || hi >= 0 && n > hi ) {
      if ( lo == hi ) {
        this.ip.raise("TypeError", ((((((fname + "() takes exactly ") + ((lo).toString())) + " argument") + this.ip.plural(lo)) + " (") + ((n).toString())) + " given)");
      }
      if ( n < lo ) {
        this.ip.raise("TypeError", (((((fname + "() expected at least ") + ((lo).toString())) + " argument") + this.ip.plural(lo)) + ", got ") + ((n).toString()));
      }
      this.ip.raise("TypeError", (((((fname + "() expected at most ") + ((hi).toString())) + " argument") + this.ip.plural(hi)) + ", got ") + ((n).toString()));
    }
  };
  noKw (fname, kn) {
    if ( kn.length > 0 ) {
      this.ip.raise("TypeError", fname + "() takes no keyword arguments");
    }
  };
  toInt (v, what) {
    if ( this.ip.isIntLike(v) ) {
      return v.i;
    }
    const ix = this.ip.findMethod(v, "__index__");
    if ( (typeof(ix) !== "undefined" && ix != null )  ) {
      const r = this.ip.callMethodVal(ix, v, []);
      return r.i;
    }
    this.ip.raise("TypeError", ("'" + this.ip.typeName(v)) + "' object cannot be interpreted as an integer");
    return 0;
  };
  toNum (v) {
    if ( this.ip.isNum(v) ) {
      return this.ip.toDouble(v);
    }
    const fl = this.ip.findMethod(v, "__float__");
    if ( (typeof(fl) !== "undefined" && fl != null )  ) {
      const r = this.ip.callMethodVal(fl, v, []);
      return this.ip.toDouble(r);
    }
    this.ip.raise("TypeError", "must be real number, not " + this.ip.typeName(v));
    return 0.0;
  };
  makeComplex (args, kn, kv) {
    const rv = this.arg(args, 0, kn, kv, "real");
    const iv = this.arg(args, 1, kn, kv, "imag");
    const hasIm = args.length > 1 || this.kwIndex(kn, "imag") >= 0;
    if ( rv.t == 4 ) {
      if ( hasIm ) {
        this.ip.raise("TypeError", "complex() can't take second arg if first is a string");
      }
      return this.parseComplexText(rv.s);
    }
    if ( hasIm && iv.t == 4 ) {
      this.ip.raise("TypeError", "complex() second arg can't be a string");
    }
    const a = this.complexOf(rv, "first");
    if ( hasIm == false ) {
      return a;
    }
    const b = this.complexOf(iv, "second");
    let re = a.d;
    let im = b.d;
    if ( iv.t == 24 ) {
      re = re - b.d2;
    }
    if ( rv.t == 24 ) {
      im = im + a.d2;
    }
    return this.ip.mkComplex(re, im);
  };
  kwIndex (kn, name) {
    // Loop start
    for ( let i = 0; i < kn.length; i++) {
      var k = kn[i];
      if ( k == name ) {
        return i;
      }
    }
    return -1;
  };
  complexOf (v, which) {
    if ( v.t == 24 ) {
      return v;
    }
    if ( v.t == 0 ) {
      return this.ip.mkComplex(0.0, 0.0);
    }
    if ( this.ip.isNum(v) ) {
      return this.ip.mkComplex(this.ip.toDouble(v), 0.0);
    }
    const cm = this.ip.findMethod(v, "__complex__");
    if ( (typeof(cm) !== "undefined" && cm != null )  ) {
      const r = this.ip.callMethodVal(cm, v, []);
      if ( r.t != 24 ) {
        this.ip.raise("TypeError", ("__complex__ returned non-complex (type " + this.ip.typeName(r)) + ")");
      }
      return r;
    }
    const fm = this.ip.findMethod(v, "__float__");
    if ( (typeof(fm) !== "undefined" && fm != null )  ) {
      return this.ip.mkComplex(this.toNum(v), 0.0);
    }
    const im = this.ip.findMethod(v, "__index__");
    if ( (typeof(im) !== "undefined" && im != null )  ) {
      const r2 = this.ip.callMethodVal(im, v, []);
      return this.ip.mkComplex(this.ip.toDouble(r2), 0.0);
    }
    this.ip.raise("TypeError", ((("complex() " + which) + " argument must be a string or a number, not '") + this.ip.typeName(v)) + "'");
    return v;
  };
  parseComplexText (text) {
    let s = text.trim();
    const bad = "complex() arg is a malformed string";
    if ( s.startsWith("(") && s.endsWith(")") ) {
      s = s.substring(1, s.length - 1 ).trim();
    }
    const n = s.length;
    if ( n == 0 ) {
      this.ip.raise("ValueError", bad);
    }
    const last = s.substring(n - 1, n ).toLowerCase();
    if ( last != "j" ) {
      if ( s.indexOf(" ") >= 0 ) {
        this.ip.raise("ValueError", bad);
      }
      return this.ip.mkComplex(this.complexPart(s, bad), 0.0);
    }
    const body = s.substring(0, n - 1 );
    const cs = Array.from(body, (rg_c) => rg_c.codePointAt(0));
    let split = -1;
    let k = cs.length - 1;
    while (k > 0) {
      const c = cs[k];
      if ( c == 43 || c == 45 ) {
        const pc = cs[(k - 1)];
        if ( pc != 101 && pc != 69 ) {
          split = k;
          break;
        }
      }
      k = k - 1;
    };
    let reText = "";
    let imText = body;
    if ( split > 0 ) {
      reText = body.substring(0, split );
      imText = body.substring(split, body.length );
    }
    if ( reText.indexOf(" ") >= 0 || imText.indexOf(" ") >= 0 ) {
      this.ip.raise("ValueError", bad);
    }
    let im = 1.0;
    if ( imText == "+" || imText == "" ) {
      im = 1.0;
    } else {
      if ( imText == "-" ) {
        im = -1.0;
      } else {
        im = this.complexPart(imText, bad);
      }
    }
    let re = 0.0;
    if ( reText.length > 0 ) {
      re = this.complexPart(reText, bad);
    }
    return this.ip.mkComplex(re, im);
  };
  complexPart (t, bad) {
    let r = 0.0;
    let failed = false;
    try {
      r = this.parseFloatText(t);
    } catch(e) {
      failed = true;
      this.ip.takeException(( e.toString()));
    }
    if ( failed ) {
      this.ip.raise("ValueError", bad);
    }
    return r;
  };
  call0 (f, args) {
    let kn = [];
    let kv = [];
    return this.ip.callValue(f, args, kn, kv);
  };
  sortVals (vals, keyf, reverse) {
    const n = vals.length;
    let keys = [];
    if ( keyf.t == 0 ) {
      keys = vals;
    } else {
      // Loop start
      for ( const v of vals) {
        keys.push(this.call0(keyf, [v]));
      }
    }
    let idx = [];
    let i_1 = 0;
    while (i_1 < n) {
      idx.push(i_1);
      i_1 = i_1 + 1;
    };
    const sorted = this.mergeSort(idx, keys, reverse);
    let out = [];
    // Loop start
    for ( const k of sorted) {
      out.push(vals[k]);
    }
    return out;
  };
  mergeSort (idx, keys, reverse) {
    const n = idx.length;
    if ( n <= 1 ) {
      return idx;
    }
    const mid = ((n / 2) | 0);
    let left = [];
    let right = [];
    let i = 0;
    while (i < n) {
      if ( i < mid ) {
        left.push(idx[i]);
      } else {
        right.push(idx[i]);
      }
      i = i + 1;
    };
    const l = this.mergeSort(left, keys, reverse);
    const r = this.mergeSort(right, keys, reverse);
    let out = [];
    let a = 0;
    let b = 0;
    const nl = l.length;
    const nr = r.length;
    while (a < nl && b < nr) {
      const ka = keys[l[a]];
      const kb = keys[r[b]];
      let takeRight = false;
      if ( reverse ) {
        takeRight = this.ip.less(ka, kb);
      } else {
        takeRight = this.ip.less(kb, ka);
      }
      if ( takeRight ) {
        out.push(r[b]);
        b = b + 1;
      } else {
        out.push(l[a]);
        a = a + 1;
      }
    };
    while (a < nl) {
      out.push(l[a]);
      a = a + 1;
    };
    while (b < nr) {
      out.push(r[b]);
      b = b + 1;
    };
    return out;
  };
  parseIntText (text, base0) {
    const s = text.trim();
    const cs = Array.from(s, (rg_c) => rg_c.codePointAt(0));
    const n = cs.length;
    let i = 0;
    let neg = false;
    if ( i < n && (cs[i] == 43 || cs[i] == 45) ) {
      neg = cs[i] == 45;
      i = i + 1;
    }
    let base = base0;
    if ( i + 1 < n ) {
      const c0 = cs[i];
      const c1 = cs[(i + 1)];
      if ( c0 == 48 ) {
        if ( (c1 == 120 || c1 == 88) && (base == 16 || base == 0) ) {
          base = 16;
          i = i + 2;
        } else {
          if ( (c1 == 111 || c1 == 79) && (base == 8 || base == 0) ) {
            base = 8;
            i = i + 2;
          } else {
            if ( (c1 == 98 || c1 == 66) && (base == 2 || base == 0) ) {
              base = 2;
              i = i + 2;
            }
          }
        }
      }
    }
    if ( base == 0 ) {
      base = 10;
    }
    let v = 0;
    let any = false;
    let lastUnder = true;
    while (i < n) {
      const c = cs[i];
      if ( c == 95 ) {
        if ( lastUnder ) {
          any = false;
          break;
        }
        lastUnder = true;
        i = i + 1;
        continue;
      }
      let d = PyLexer.digitValue(c);
      if ( c >= 103 && c <= 122 ) {
        d = c - 87;
      }
      if ( c >= 71 && c <= 90 ) {
        d = c - 55;
      }
      if ( d < 0 || d >= base ) {
        any = false;
        break;
      }
      v = v * base + d;
      any = true;
      lastUnder = false;
      i = i + 1;
    };
    if ( any == false || lastUnder ) {
      this.ip.raise("ValueError", (("invalid literal for int() with base " + ((base0).toString())) + ": ") + PyFormat.strRepr(text));
    }
    if ( neg ) {
      v = 0 - v;
    }
    return this.ip.mkInt(v);
  };
  parseFloatText (text) {
    const s = text.trim().toLowerCase();
    const t = s.split("_").join("");
    if ( ((t == "inf" || t == "+inf") || t == "infinity") || t == "+infinity" ) {
      return Infinity;
    }
    if ( t == "-inf" || t == "-infinity" ) {
      return 0.0 - Infinity;
    }
    if ( (t == "nan" || t == "+nan") || t == "-nan" ) {
      return NaN;
    }
    const cs = Array.from(t, (rg_c) => rg_c.codePointAt(0));
    const n = cs.length;
    let i = 0;
    if ( i < n && (cs[i] == 43 || cs[i] == 45) ) {
      i = i + 1;
    }
    let digits = 0;
    while (i < n && PyLexer.isDigit(cs[i])) {
      i = i + 1;
      digits = digits + 1;
    };
    if ( i < n && cs[i] == 46 ) {
      i = i + 1;
      while (i < n && PyLexer.isDigit(cs[i])) {
        i = i + 1;
        digits = digits + 1;
      };
    }
    let ok = digits > 0;
    if ( (ok && i < n) && cs[i] == 101 ) {
      i = i + 1;
      if ( i < n && (cs[i] == 43 || cs[i] == 45) ) {
        i = i + 1;
      }
      let ed = 0;
      while (i < n && PyLexer.isDigit(cs[i])) {
        i = i + 1;
        ed = ed + 1;
      };
      if ( ed == 0 ) {
        ok = false;
      }
    }
    if ( (ok == false || i != n) || s.split("__").join("").length != s.length ) {
      this.ip.raise("ValueError", "could not convert string to float: " + PyFormat.strRepr(text));
    }
    return Number(t);
  };
  toIntValue (v) {
    if ( this.ip.isIntLike(v) ) {
      return this.ip.mkInt(v.i);
    }
    if ( v.t == 3 ) {
      if ( Number.isNaN(v.d) ) {
        this.ip.raise("ValueError", "cannot convert float NaN to integer");
      }
      if ( (Math.abs(v.d) === Infinity) ) {
        this.ip.raise("OverflowError", "cannot convert float infinity to integer");
      }
      return this.ip.mkInt(Math.trunc(v.d));
    }
    if ( v.t == 4 ) {
      return this.parseIntText(v.s, 10);
    }
    const m = this.ip.findMethod(v, "__int__");
    if ( (typeof(m) !== "undefined" && m != null )  ) {
      return this.ip.callMethodVal(m, v, []);
    }
    const m2 = this.ip.findMethod(v, "__index__");
    if ( (typeof(m2) !== "undefined" && m2 != null )  ) {
      return this.ip.callMethodVal(m2, v, []);
    }
    this.ip.raise("TypeError", ("int() argument must be a string, a bytes-like object or a real number, not '" + this.ip.typeName(v)) + "'");
    return this.ip.NONE;
  };
  roundValue (x, nd) {
    if ( this.ip.isIntLike(x) ) {
      if ( nd.t == 0 ) {
        return this.ip.mkInt(x.i);
      }
      const k = this.toInt(nd, "round");
      if ( k >= 0 ) {
        return this.ip.mkInt(x.i);
      }
      const q = this.ip.intPow(10, (0 - k)).i;
      const a = x.i;
      let fl = PyFormat.floorDivInt(a, q);
      const rem = a - fl * q;
      const twice = rem * 2;
      if ( twice > q || twice == q && PyFormat.floorModInt(fl, 2) == 1 ) {
        fl = fl + 1;
      }
      return this.ip.mkInt((fl * q));
    }
    if ( x.t == 12 ) {
      const m = this.ip.findMethod(x, "__round__");
      if ( (typeof(m) !== "undefined" && m != null )  ) {
        if ( nd.t == 0 ) {
          return this.ip.callMethodVal(m, x, []);
        }
        return this.ip.callMethodVal(m, x, [nd]);
      }
    }
    const d = this.toNum(x);
    if ( nd.t == 0 ) {
      if ( Number.isNaN(d) ) {
        this.ip.raise("ValueError", "cannot convert float NaN to integer");
      }
      if ( (Math.abs(d) === Infinity) ) {
        this.ip.raise("OverflowError", "cannot convert float infinity to integer");
      }
      const neg = d < 0.0;
      let a2 = d;
      if ( neg ) {
        a2 = 0.0 - d;
      }
      let r = PyFormat.scaledRound(a2, 0);
      if ( r < 0.0 ) {
        r = a2;
      }
      if ( neg ) {
        r = r * -1.0;
      }
      return this.ip.mkInt(Math.trunc(r));
    }
    const k2 = this.toInt(nd, "round");
    if ( Number.isNaN(d) || (Math.abs(d) === Infinity) ) {
      return this.ip.mkFloat(d);
    }
    if ( k2 > 300 ) {
      return this.ip.mkFloat(d);
    }
    const neg2 = d < 0.0 || PyFormat.isNegZero(d);
    let a3 = d;
    if ( d < 0.0 ) {
      a3 = 0.0 - d;
    }
    let r2 = 0.0;
    if ( k2 >= 0 ) {
      const txt = PyFormat.fixedAbs(a3, k2);
      r2 = Number(txt);
    } else {
      const sc = PyFormat.scaledRound(a3, k2);
      if ( sc < 0.0 ) {
        r2 = a3;
      } else {
        r2 = sc * PyFormat.pow10((0 - k2));
      }
    }
    if ( neg2 ) {
      r2 = r2 * -1.0;
    }
    return this.ip.mkFloat(r2);
  };
  sumValues (vals, start) {
    let allFloat = true;
    let anyFloat = start.t == 3;
    if ( this.ip.isNum(start) == false ) {
      allFloat = false;
    }
    // Loop start
    for ( const v of vals) {
      if ( this.ip.isNum(v) == false ) {
        allFloat = false;
      }
      if ( v.t == 3 ) {
        anyFloat = true;
      }
    }
    if ( allFloat && anyFloat ) {
      let s = this.ip.toDouble(start);
      let comp = 0.0;
      // Loop start
      for ( const v2 of vals) {
        const x = this.ip.toDouble(v2);
        const t = s + x;
        let as = s;
        if ( as < 0.0 ) {
          as = 0.0 - as;
        }
        let ax = x;
        if ( ax < 0.0 ) {
          ax = 0.0 - ax;
        }
        if ( as >= ax ) {
          comp = comp + ((s - t) + x);
        } else {
          comp = comp + ((x - t) + s);
        }
        s = t;
      }
      return this.ip.mkFloat((s + comp));
    }
    if ( start.t == 4 ) {
      this.ip.raise("TypeError", "sum() can't sum strings [use ''.join(seq) instead]");
    }
    let acc = start;
    // Loop start
    for ( const v3 of vals) {
      acc = this.ip.binop("+", acc, v3);
    }
    return acc;
  };
  minMax (fname, args, kn, kv, wantMax) {
    let vals = [];
    if ( args.length == 1 ) {
      vals = this.ip.iterValues(args[0]);
    } else {
      vals = args;
    }
    let keyf = this.ip.NONE;
    const kf = this.kw(kn, kv, "key");
    if ( (typeof(kf) !== "undefined" && kf != null )  ) {
      keyf = kf;
    }
    if ( vals.length == 0 ) {
      const dflt = this.kw(kn, kv, "default");
      if ( (typeof(dflt) !== "undefined" && dflt != null )  ) {
        return dflt;
      }
      this.ip.raise("ValueError", fname + "() arg is an empty sequence");
    }
    let best = vals[0];
    let bestKey = best;
    if ( keyf.t != 0 ) {
      bestKey = this.call0(keyf, [best]);
    }
    const n = vals.length;
    let i = 1;
    while (i < n) {
      const v = vals[i];
      let k = v;
      if ( keyf.t != 0 ) {
        k = this.call0(keyf, [v]);
      }
      let better = false;
      if ( wantMax ) {
        better = this.ip.less(bestKey, k);
      } else {
        better = this.ip.less(k, bestKey);
      }
      if ( better ) {
        best = v;
        bestKey = k;
      }
      i = i + 1;
    };
    return best;
  };
  isInstance (obj, cls) {
    if ( cls.t == 6 ) {
      // Loop start
      for ( const c of cls.items) {
        if ( this.isInstance(obj, c) ) {
          return true;
        }
      }
      return false;
    }
    if ( cls.t == 11 && ((typeof(cls.cls) !== "undefined" && cls.cls != null ) ) ) {
      const mick = this.ip.classLookup(cls.cls, "__instancecheck__");
      const __REGx1 = this.ip.orNone(mick);
      if ( ((typeof(mick) !== "undefined" && mick != null ) ) && __REGx1.t == 9 ) {
        return this.ip.truthy(this.ip.callMethodVal(mick, cls, [obj]));
      }
    }
    if ( (cls.t == 11 && obj.t == 11) && cls.bt == "type" ) {
      if ( typeof(obj.cls) === "undefined" ) {
        return false;
      }
      return this.ip.isSubclass(obj.cls, cls);
    }
    if ( cls.t == 11 ) {
      if ( ( typeof(cls.attrs["__instancecheck__"] ) != "undefined" && Object.prototype.hasOwnProperty.call(cls.attrs, "__instancecheck__") ) ) {
        const ick = this.ip.getAttr(cls, "__instancecheck__");
        let ckn = [];
        let ckv = [];
        return this.ip.truthy(this.ip.callValue(ick, [obj], ckn, ckv));
      }
      if ( obj.t == 12 && ((typeof(obj.cls) !== "undefined" && obj.cls != null ) ) ) {
        return this.ip.isSubclass(obj.cls, cls);
      }
      if ( this.ip.isDataSub(obj) ) {
        return this.ip.isSubclass(obj.cls, cls);
      }
      if ( ((obj.t == 7 || obj.t == 5) || obj.t == 6) && ( typeof(obj.attrs["__class__"] ) != "undefined" && Object.prototype.hasOwnProperty.call(obj.attrs, "__class__") ) ) {
        const dc = ( Object.prototype.hasOwnProperty.call(obj.attrs, "__class__") ? obj.attrs["__class__"] : undefined );
        return this.ip.isSubclass(dc, cls);
      }
      return false;
    }
    if ( cls.t == 10 ) {
      const n = cls.s;
      if ( n == "object" ) {
        return true;
      }
      const tn = this.ip.typeName(obj);
      if ( n == "int" ) {
        return obj.t == 2 || obj.t == 1;
      }
      if ( n == "dict" ) {
        return obj.t == 7;
      }
      if ( n == "list" ) {
        return obj.t == 5;
      }
      if ( n == "tuple" ) {
        return obj.t == 6;
      }
      if ( n == "str" ) {
        return obj.t == 4;
      }
      if ( n == "float" ) {
        return obj.t == 3;
      }
      if ( n == "complex" ) {
        return obj.t == 24;
      }
      if ( n == "type" ) {
        return obj.t == 11 || obj.t == 10 && this.ip.isTypeBuiltin(obj.s);
      }
      if ( n == "set" ) {
        return obj.t == 8 && obj.mode != "frozen";
      }
      return tn == n;
    }
    this.ip.raise("TypeError", "isinstance() arg 2 must be a type, a tuple of types, or a union");
    return false;
  };
  typeOf (v) {
    if ( v.t == 12 ) {
      return v.cls;
    }
    if ( v.t == 11 && ((typeof(v.cls) !== "undefined" && v.cls != null ) ) ) {
      return v.cls;
    }
    if ( this.ip.isDataSub(v) ) {
      return v.cls;
    }
    if ( ((v.t == 7 || v.t == 5) || v.t == 6) && ( typeof(v.attrs["__class__"] ) != "undefined" && Object.prototype.hasOwnProperty.call(v.attrs, "__class__") ) ) {
      return ( Object.prototype.hasOwnProperty.call(v.attrs, "__class__") ? v.attrs["__class__"] : undefined );
    }
    const tn = this.ip.typeName(v);
    const b = ( Object.prototype.hasOwnProperty.call(this.ip.builtins, tn) ? this.ip.builtins[tn] : undefined );
    if ( (typeof(b) !== "undefined" && b != null )  ) {
      return b;
    }
    const made = new PyVal(10);
    made.s = tn;
    return made;
  };
  openFile (args, kn, kv) {
    const pv = this.arg(args, 0, kn, kv, "file");
    if ( pv.t != 4 ) {
      this.ip.raise("TypeError", "expected str, bytes or os.PathLike object, not " + this.ip.typeName(pv));
    }
    let mode = "r";
    const mv = this.arg(args, 1, kn, kv, "mode");
    if ( mv.t == 4 ) {
      mode = mv.s;
    }
    const vfs = this.ip.fs;
    const path = vfs.normalize(pv.s);
    const f = new PyVal(16);
    f.s = pv.s;
    f.mode = mode;
    const plain = mode.split("b")
      .join("")
      .split("t")
      .join("")
      .split("+")
      .join("");
    if ( this.ip.fs.isDir(path) ) {
      if ( ( typeof(this.ip.fs.files[path] ) != "undefined" && Object.prototype.hasOwnProperty.call(this.ip.fs.files, path) ) == false ) {
        this.ip.raise("IsADirectoryError", "[Errno 21] Is a directory: " + PyFormat.strRepr(pv.s));
      }
    }
    if ( plain == "r" ) {
      if ( this.ip.fs.isFile(path) == false ) {
        this.ip.raise("FileNotFoundError", "[Errno 2] No such file or directory: " + PyFormat.strRepr(pv.s));
      }
      return f;
    }
    if ( plain == "w" ) {
      vfs.write(path, "");
      return f;
    }
    if ( plain == "a" ) {
      if ( this.ip.fs.isFile(path) == false ) {
        vfs.write(path, "");
      }
      return f;
    }
    if ( plain == "x" ) {
      if ( this.ip.fs.isFile(path) ) {
        this.ip.raise("FileExistsError", "[Errno 17] File exists: " + PyFormat.strRepr(pv.s));
      }
      vfs.write(path, "");
      return f;
    }
    this.ip.raise("ValueError", ("invalid mode: '" + mode) + "'");
    return f;
  };
  asciiOf (s) {
    const cs = Array.from(s, (rg_c) => rg_c.codePointAt(0));
    let out = "";
    // Loop start
    for ( const c of cs) {
      if ( c < 128 ) {
        out = out + String.fromCodePoint(c);
      } else {
        if ( c < 256 ) {
          out = (out + "\\x") + this.hexPad(c, 2);
        } else {
          if ( c < 65536 ) {
            out = (out + "\\u") + this.hexPad(c, 4);
          } else {
            out = (out + "\\U") + this.hexPad(c, 8);
          }
        }
      }
    }
    return out;
  };
  hexPad (v, w) {
    let h = PyFormat.intBase(v, 16, false);
    while (h.length < w) {
      h = "0" + h;
    };
    return h;
  };
  signedBase (v, base, prefix) {
    if ( v < 0 ) {
      return ("-" + prefix) + PyFormat.intBase((0 - v), base, false);
    }
    return prefix + PyFormat.intBase(v, base, false);
  };
  strHash (s) {
    const cs = Array.from(s, (rg_c) => rg_c.codePointAt(0));
    let h = 5381;
    // Loop start
    for ( const c of cs) {
      h = PyFormat.floorModInt((h * 33 + c), 2147483647);
    }
    return h;
  };
  printTo (args, kn, kv) {
    let sep = " ";
    let end = "\n";
    const sv = this.kw(kn, kv, "sep");
    const __REGx1 = this.ip.orNone(sv);
    if ( ((typeof(sv) !== "undefined" && sv != null ) ) && __REGx1.t == 4 ) {
      const tv1 = sv;
      sep = tv1.s;
    }
    const ev = this.kw(kn, kv, "end");
    const __REGx2 = this.ip.orNone(ev);
    if ( ((typeof(ev) !== "undefined" && ev != null ) ) && __REGx2.t == 4 ) {
      const tv2 = ev;
      end = tv2.s;
    }
    let parts = [];
    // Loop start
    for ( const a of args) {
      parts.push(this.ip.toStr(a));
    }
    const text = parts.join(sep) + end;
    const fv = this.kw(kn, kv, "file");
    const __REGx3 = this.ip.orNone(fv);
    if ( ((typeof(fv) !== "undefined" && fv != null ) ) && __REGx3.t != 0 ) {
      const f = fv;
      this.writeTo(f, text);
      return;
    }
    const sm = ( Object.prototype.hasOwnProperty.call(this.ip.modules, "sys") ? this.ip.modules["sys"] : undefined );
    if ( (typeof(sm) !== "undefined" && sm != null )  ) {
      const smv = sm;
      const so = ( Object.prototype.hasOwnProperty.call(smv.attrs, "stdout") ? smv.attrs["stdout"] : undefined );
      if ( (typeof(so) !== "undefined" && so != null )  ) {
        const sov = so;
        if ( sov.t != 16 || sov.s != "<stdout>" ) {
          this.writeTo(sov, text);
          return;
        }
      }
    }
    this.ip.writeOut(text);
  };
  writeTo (f, text) {
    if ( f.t == 16 ) {
      this.ip.meth.fileWrite(f, text);
      return;
    }
    let kn = [];
    let kv = [];
    this.ip.callValue(
      this.ip.getAttr(f, "write"),
      [this.ip.mkStr(text)],
      kn,
      kv
    );
  };
  call (name, args, kn, kv) {
    const n = args.length;
    if ( name == "print" ) {
      this.printTo(args, kn, kv);
      return this.ip.NONE;
    }
    if ( name == "len" ) {
      this.need("len", args, 1, 1);
      return this.ip.mkInt(this.ip.length(args[0]));
    }
    if ( name == "range" ) {
      this.need("range", args, 1, 3);
      const r = new PyVal(15);
      r.i3 = 1;
      if ( n == 1 ) {
        r.i2 = this.toInt(args[0], "range");
      } else {
        r.i = this.toInt(args[0], "range");
        r.i2 = this.toInt(args[1], "range");
        if ( n == 3 ) {
          r.i3 = this.toInt(args[2], "range");
          if ( r.i3 == 0 ) {
            this.ip.raise("ValueError", "range() arg 3 must not be zero");
          }
        }
      }
      return r;
    }
    if ( name == "enumerate" ) {
      const src = this.arg(args, 0, kn, kv, "iterable");
      let start = 0;
      const sv = this.arg(args, 1, kn, kv, "start");
      if ( sv.t != 0 ) {
        start = this.toInt(sv, "enumerate");
      }
      if ( this.ip.isLazy(src) ) {
        const le = this.ip.mkLazy("enumerate", "enumerate");
        le.wrapped = this.ip.iterOf(src);
        le.i = start;
        return le;
      }
      const vals = this.ip.iterValues(src);
      let out = [];
      // Loop start
      for ( let i = 0; i < vals.length; i++) {
        var v = vals[i];
        out.push(this.ip.pair(this.ip.mkInt(start + i), v));
      }
      const it = this.ip.mkIter(out);
      it.s = "enumerate";
      return it;
    }
    if ( name == "zip" ) {
      let anyLazy = false;
      // Loop start
      for ( const za of args) {
        if ( this.ip.isLazy(za) ) {
          anyLazy = true;
        }
      }
      if ( anyLazy ) {
        const lz = this.ip.mkLazy("zip", "zip");
        // Loop start
        for ( const zb of args) {
          lz.items.push(this.ip.iterOf(zb));
        }
        const zs = this.kw(kn, kv, "strict");
        if ( ((typeof(zs) !== "undefined" && zs != null ) ) && this.ip.truthy(zs) ) {
          lz.i = 1;
        }
        return lz;
      }
      let lists = [];
      let shortest = -1;
      let longest = 0;
      // Loop start
      for ( const a of args) {
        const vs = this.ip.iterValues(a);
        lists.push(this.ip.mkList(vs));
        if ( shortest < 0 || vs.length < shortest ) {
          shortest = vs.length;
        }
        if ( vs.length > longest ) {
          longest = vs.length;
        }
      }
      const strict = this.kw(kn, kv, "strict");
      if ( (((typeof(strict) !== "undefined" && strict != null ) ) && this.ip.truthy(strict)) && shortest != longest ) {
        this.ip.raise("ValueError", "zip() arguments have different lengths");
      }
      if ( shortest < 0 ) {
        shortest = 0;
      }
      let out2 = [];
      let k = 0;
      while (k < shortest) {
        let tup = [];
        // Loop start
        for ( const l of lists) {
          tup.push(l.items[k]);
        }
        out2.push(this.ip.mkTuple(tup));
        k = k + 1;
      };
      const it2 = this.ip.mkIter(out2);
      it2.s = "zip";
      return it2;
    }
    if ( name == "map" ) {
      if ( n < 2 ) {
        this.ip.raise("TypeError", "map() must have at least two arguments.");
      }
      const f = args[0];
      let mLazy = false;
      let mi = 1;
      while (mi < n) {
        if ( this.ip.isLazy(args[mi]) ) {
          mLazy = true;
        }
        mi = mi + 1;
      };
      if ( mLazy ) {
        const lm = this.ip.mkLazy("map", "map");
        lm.cls = f;
        let mj = 1;
        while (mj < n) {
          lm.items.push(this.ip.iterOf(args[mj]));
          mj = mj + 1;
        };
        return lm;
      }
      let lists2 = [];
      let shortest2 = -1;
      let ai = 1;
      while (ai < n) {
        const vs2 = this.ip.iterValues(args[ai]);
        lists2.push(this.ip.mkList(vs2));
        if ( shortest2 < 0 || vs2.length < shortest2 ) {
          shortest2 = vs2.length;
        }
        ai = ai + 1;
      };
      let out3 = [];
      let k2 = 0;
      while (k2 < shortest2) {
        let cargs = [];
        // Loop start
        for ( const l2 of lists2) {
          cargs.push(l2.items[k2]);
        }
        out3.push(this.call0(f, cargs));
        k2 = k2 + 1;
      };
      const it3 = this.ip.mkIter(out3);
      it3.s = "map";
      return it3;
    }
    if ( name == "filter" ) {
      this.need("filter", args, 2, 2);
      const f2 = args[0];
      if ( this.ip.isLazy(args[1]) ) {
        const lf = this.ip.mkLazy("filter", "filter");
        lf.cls = f2;
        lf.wrapped = this.ip.iterOf(args[1]);
        return lf;
      }
      const vals2 = this.ip.iterValues(args[1]);
      let out4 = [];
      // Loop start
      for ( const v2 of vals2) {
        let keep = false;
        if ( f2.t == 0 ) {
          keep = this.ip.truthy(v2);
        } else {
          keep = this.ip.truthy(this.call0(f2, [v2]));
        }
        if ( keep ) {
          out4.push(v2);
        }
      }
      const it4 = this.ip.mkIter(out4);
      it4.s = "filter";
      return it4;
    }
    if ( name == "sorted" ) {
      this.need("sorted", args, 1, 1);
      const vals3 = this.ip.iterValues(args[0]);
      let keyf = this.ip.NONE;
      const kf = this.kw(kn, kv, "key");
      if ( (typeof(kf) !== "undefined" && kf != null )  ) {
        keyf = kf;
      }
      let rev = false;
      const rv = this.kw(kn, kv, "reverse");
      if ( (typeof(rv) !== "undefined" && rv != null )  ) {
        rev = this.ip.truthy(rv);
      }
      return this.ip.mkList(this.sortVals(vals3, keyf, rev));
    }
    if ( name == "reversed" ) {
      this.need("reversed", args, 1, 1);
      const src2 = args[0];
      if ( src2.t == 12 ) {
        const rm = this.ip.findMethod(src2, "__reversed__");
        if ( (typeof(rm) !== "undefined" && rm != null )  ) {
          return this.ip.callMethodVal(rm, src2, []);
        }
      }
      if ( src2.t == 7 || src2.t == 8 ) {
        if ( src2.t == 8 ) {
          this.ip.raise("TypeError", "'set' object is not reversible");
        }
      }
      const vals4 = this.ip.iterValues(src2);
      let out5 = [];
      let ri = vals4.length - 1;
      while (ri >= 0) {
        out5.push(vals4[ri]);
        ri = ri - 1;
      };
      const it5 = this.ip.mkIter(out5);
      it5.s = "reversed";
      return it5;
    }
    if ( name == "sum" ) {
      this.need("sum", args, 1, 2);
      let start2 = this.ip.mkInt(0);
      if ( n == 2 ) {
        start2 = args[1];
      }
      const sk = this.kw(kn, kv, "start");
      if ( (typeof(sk) !== "undefined" && sk != null )  ) {
        start2 = sk;
      }
      return this.sumValues(this.ip.iterValues(args[0]), start2);
    }
    if ( name == "min" ) {
      this.need("min", args, 1, -1);
      return this.minMax("min", args, kn, kv, false);
    }
    if ( name == "max" ) {
      this.need("max", args, 1, -1);
      return this.minMax("max", args, kn, kv, true);
    }
    if ( name == "abs" ) {
      this.need("abs", args, 1, 1);
      const x = args[0];
      if ( this.ip.isIntLike(x) ) {
        if ( x.i < 0 ) {
          return this.ip.mkInt((0 - x.i));
        }
        return this.ip.mkInt(x.i);
      }
      if ( x.t == 3 ) {
        if ( x.d < 0.0 || PyFormat.isNegZero(x.d) ) {
          return this.ip.mkFloat((0.0 - x.d));
        }
        return x;
      }
      if ( x.t == 24 ) {
        const ha = PyInterp.hypot(x.d, x.d2);
        if ( ((Math.abs(ha) === Infinity) && (Math.abs(x.d) === Infinity) == false) && (Math.abs(x.d2) === Infinity) == false ) {
          this.ip.raise("OverflowError", "absolute value too large");
        }
        if ( Number.isNaN(x.d) || Number.isNaN(x.d2) ) {
          if ( (Math.abs(x.d) === Infinity) || (Math.abs(x.d2) === Infinity) ) {
            return this.ip.mkFloat(Infinity);
          }
          return this.ip.mkFloat(NaN);
        }
        return this.ip.mkFloat(ha);
      }
      const am = this.ip.findMethod(x, "__abs__");
      if ( (typeof(am) !== "undefined" && am != null )  ) {
        return this.ip.callMethodVal(am, x, []);
      }
      this.ip.raise("TypeError", ("bad operand type for abs(): '" + this.ip.typeName(x)) + "'");
    }
    if ( name == "round" ) {
      this.need("round", args, 1, 2);
      return this.roundValue(args[0], this.arg(args, 1, kn, kv, "ndigits"));
    }
    if ( name == "int" ) {
      if ( n == 0 ) {
        return this.ip.mkInt(0);
      }
      const base = this.arg(args, 1, kn, kv, "base");
      if ( base.t != 0 ) {
        const sv2 = args[0];
        if ( sv2.t != 4 ) {
          this.ip.raise("TypeError", "int() can't convert non-string with explicit base");
        }
        return this.parseIntText(sv2.s, this.toInt(base, "int"));
      }
      return this.toIntValue(args[0]);
    }
    if ( name == "complex" ) {
      return this.makeComplex(args, kn, kv);
    }
    if ( name == "float" ) {
      if ( n == 0 ) {
        return this.ip.mkFloat(0.0);
      }
      const fx = args[0];
      if ( fx.t == 4 ) {
        return this.ip.mkFloat(this.parseFloatText(fx.s));
      }
      return this.ip.mkFloat(this.toNum(fx));
    }
    if ( name == "str" ) {
      if ( n == 0 ) {
        return this.ip.mkStr("");
      }
      return this.ip.mkStr(this.ip.toStr(args[0]));
    }
    if ( name == "repr" ) {
      this.need("repr", args, 1, 1);
      return this.ip.mkStr(this.ip.toRepr(args[0]));
    }
    if ( name == "ascii" ) {
      this.need("ascii", args, 1, 1);
      return this.ip.mkStr(this.asciiOf(this.ip.toRepr(args[0])));
    }
    if ( name == "bool" ) {
      if ( n == 0 ) {
        return this.ip.FALSE;
      }
      return this.ip.mkBool(this.ip.truthy(args[0]));
    }
    if ( name == "list" ) {
      if ( n == 0 ) {
        return this.ip.mkList([]);
      }
      return this.ip.mkList(this.ip.iterValues(args[0]));
    }
    if ( name == "tuple" ) {
      if ( n == 0 ) {
        return this.ip.mkTuple([]);
      }
      const tv = args[0];
      if ( tv.t == 6 ) {
        return tv;
      }
      let tl = [];
      const tvals = this.ip.iterValues(tv);
      // Loop start
      for ( const x2 of tvals) {
        tl.push(x2);
      }
      return this.ip.mkTuple(tl);
    }
    if ( name == "dict" ) {
      const d = this.ip.mkDict();
      if ( n > 0 ) {
        this.fillDict(d, args[0]);
      }
      // Loop start
      for ( let ki = 0; ki < kn.length; ki++) {
        var key = kn[ki];
        this.ip.dictSet(d, this.ip.mkStr(key), kv[ki]);
      }
      return d;
    }
    if ( name == "set" || name == "frozenset" ) {
      const st = this.ip.mkSet();
      if ( name == "frozenset" ) {
        st.mode = "frozen";
      }
      if ( n > 0 ) {
        const svals = this.ip.iterValues(args[0]);
        // Loop start
        for ( const x3 of svals) {
          this.ip.setAdd(st, x3);
        }
      }
      return st;
    }
    if ( name == "isinstance" ) {
      this.need("isinstance", args, 2, 2);
      return this.ip.mkBool(this.isInstance(args[0], args[1]));
    }
    if ( name == "issubclass" ) {
      this.need("issubclass", args, 2, 2);
      const c = args[0];
      const b = args[1];
      if ( b.t == 6 ) {
        // Loop start
        for ( const bb of b.items) {
          const one = this.call("issubclass", [c, bb], kn, kv);
          if ( this.ip.truthy(one) ) {
            return this.ip.TRUE;
          }
        }
        return this.ip.FALSE;
      }
      if ( b.t == 11 && ((typeof(b.cls) !== "undefined" && b.cls != null ) ) ) {
        const msck = this.ip.classLookup(b.cls, "__subclasscheck__");
        const __REGx1 = this.ip.orNone(msck);
        if ( ((typeof(msck) !== "undefined" && msck != null ) ) && __REGx1.t == 9 ) {
          return this.ip.mkBool(this.ip.truthy(this.ip.callMethodVal(
            msck,
            b,
            [c]
          )));
        }
      }
      if ( b.t == 11 && ( typeof(b.attrs["__subclasscheck__"] ) != "undefined" && Object.prototype.hasOwnProperty.call(b.attrs, "__subclasscheck__") ) ) {
        const sck = this.ip.getAttr(b, "__subclasscheck__");
        return this.ip.mkBool(this.ip.truthy(this.ip.callValue(
          sck,
          [c],
          kn,
          kv
        )));
      }
      if ( c.t == 11 && b.t == 11 ) {
        return this.ip.mkBool(this.ip.isSubclass(c, b));
      }
      if ( b.t == 10 && b.s == "object" ) {
        return this.ip.TRUE;
      }
      if ( c.t == 11 && b.t == 10 ) {
        return this.ip.mkBool((c.bt == b.s || b.s == "int" && c.bt == "bool"));
      }
      if ( c.t == 10 && b.t == 10 ) {
        return this.ip.mkBool((c.s == b.s || c.s == "bool" && b.s == "int"));
      }
      return this.ip.FALSE;
    }
    if ( name == "type" ) {
      if ( n == 1 ) {
        return this.typeOf(args[0]);
      }
      if ( n != 3 ) {
        this.ip.raise("TypeError", "type() takes 1 or 3 arguments");
      }
      let tmeta = ( Object.prototype.hasOwnProperty.call(this.ip.builtins, "type") ? this.ip.builtins["type"] : undefined );
      const tb3 = args[1];
      if ( tb3.t == 6 ) {
        // Loop start
        for ( const tbb of tb3.items) {
          if ( tbb.t == 11 && ((typeof(tbb.cls) !== "undefined" && tbb.cls != null ) ) ) {
            tmeta = tbb.cls;
          }
        }
      }
      if ( tmeta.t == 11 ) {
        return this.ip.metaInstantiate(tmeta, args, kn, kv);
      }
      return this.ip.typeNew(tmeta, args, kn, kv);
    }
    if ( name == "chr" ) {
      this.need("chr", args, 1, 1);
      const cv = this.toInt(args[0], "chr");
      if ( cv < 0 || cv > 1114111 ) {
        this.ip.raise("ValueError", "chr() arg not in range(0x110000)");
      }
      return this.ip.mkStr(String.fromCodePoint(cv));
    }
    if ( name == "ord" ) {
      this.need("ord", args, 1, 1);
      const ov = args[0];
      if ( ov.t != 4 || ov.strLen() != 1 ) {
        this.ip.raise("TypeError", ("ord() expected a character, but string of length " + ((this.ip.length(ov)).toString())) + " found");
      }
      const ocs = ov.codes();
      return this.ip.mkInt(ocs[0]);
    }
    if ( name == "hex" ) {
      this.need("hex", args, 1, 1);
      return this.ip.mkStr(this.signedBase(
        this.toInt(args[0], "hex"),
        16,
        "0x"
      ));
    }
    if ( name == "oct" ) {
      this.need("oct", args, 1, 1);
      return this.ip.mkStr(this.signedBase(
        this.toInt(args[0], "oct"),
        8,
        "0o"
      ));
    }
    if ( name == "bin" ) {
      this.need("bin", args, 1, 1);
      return this.ip.mkStr(this.signedBase(
        this.toInt(args[0], "bin"),
        2,
        "0b"
      ));
    }
    if ( name == "divmod" ) {
      this.need("divmod", args, 2, 2);
      const q = this.ip.binop("//", args[0], args[1]);
      const m = this.ip.binop("%", args[0], args[1]);
      return this.ip.pair(q, m);
    }
    if ( name == "pow" ) {
      this.need("pow", args, 2, 3);
      const __REGx2 = this.ip.argAt(args, 2);
      if ( n == 3 && __REGx2.t != 0 ) {
        const base2 = this.toInt(args[0], "pow");
        const ex = this.toInt(args[1], "pow");
        const md = this.toInt(args[2], "pow");
        if ( md == 0 ) {
          this.ip.raise("ValueError", "pow() 3rd argument cannot be 0");
        }
        let res = PyFormat.floorModInt(1, md);
        let bm = PyFormat.floorModInt(base2, md);
        let e2 = ex;
        while (e2 > 0) {
          if ( PyFormat.floorModInt(e2, 2) == 1 ) {
            res = PyFormat.floorModInt((res * bm), md);
          }
          bm = PyFormat.floorModInt((bm * bm), md);
          e2 = PyFormat.floorDivInt(e2, 2);
        };
        return this.ip.mkInt(res);
      }
      return this.ip.binop("**", args[0], args[1]);
    }
    if ( name == "any" ) {
      this.need("any", args, 1, 1);
      const ait = this.ip.iterOf(args[0]);
      while (true) {
        const av = this.ip.nextOpt(ait);
        if ( typeof(av) === "undefined" ) {
          break;
        }
        if ( this.ip.truthy(av) ) {
          return this.ip.TRUE;
        }
      };
      return this.ip.FALSE;
    }
    if ( name == "all" ) {
      this.need("all", args, 1, 1);
      const alit = this.ip.iterOf(args[0]);
      while (true) {
        const alv = this.ip.nextOpt(alit);
        if ( typeof(alv) === "undefined" ) {
          break;
        }
        if ( this.ip.truthy(alv) == false ) {
          return this.ip.FALSE;
        }
      };
      return this.ip.TRUE;
    }
    if ( name == "open" ) {
      return this.openFile(args, kn, kv);
    }
    if ( name == "format" ) {
      this.need("format", args, 1, 2);
      let spec = "";
      if ( n == 2 ) {
        spec = this.ip.toStr(args[1]);
      }
      return this.ip.mkStr(this.ip.formatValue(args[0], spec));
    }
    if ( name == "hash" ) {
      this.need("hash", args, 1, 1);
      const hv = args[0];
      if ( this.ip.isIntLike(hv) ) {
        return this.ip.mkInt(hv.i);
      }
      if ( (((hv.t == 3 || hv.t == 24 && hv.d2 == 0.0) && Math.floor(hv.d) == hv.d) && hv.d < 9000000000000000.0) && hv.d > -9000000000000000.0 ) {
        return this.ip.mkInt(Math.trunc(hv.d));
      }
      if ( hv.t == 12 ) {
        const hm = this.ip.findMethod(hv, "__hash__");
        if ( (typeof(hm) !== "undefined" && hm != null )  ) {
          const hr = this.ip.callMethodVal(hm, hv, []);
          if ( this.ip.isIntLike(hr) == false ) {
            this.ip.raise("TypeError", "__hash__ method should return an integer");
          }
          return this.ip.mkInt(hr.i);
        }
      }
      const hk = this.ip.hashKey(hv);
      return this.ip.mkInt(this.strHash(hk));
    }
    if ( name == "id" ) {
      this.need("id", args, 1, 1);
      const iv0 = args[0];
      if ( ((iv0.t == 2 && iv0.i >= -5) && iv0.i <= 256) && Object.keys(iv0.attrs).length == 0 ) {
        return this.ip.mkInt((140000000000 + iv0.i * 32));
      }
      return this.ip.mkInt(this.ip.idOf(iv0));
    }
    if ( name == "iter" ) {
      this.need("iter", args, 1, 1);
      const iv = args[0];
      if ( iv.t == 17 || iv.t == 23 ) {
        return iv;
      }
      if ( iv.t == 12 ) {
        const im = this.ip.findMethod(iv, "__iter__");
        if ( (typeof(im) !== "undefined" && im != null )  ) {
          const got = this.ip.callMethodVal(im, iv, []);
          if ( (got.t == 12 || got.t == 17) || got.t == 23 ) {
            return got;
          }
          return this.ip.mkIter(this.ip.iterValues(got));
        }
      }
      const it6 = this.ip.mkIter(this.ip.iterValues(iv));
      it6.s = this.ip.typeName(iv) + "_iterator";
      return it6;
    }
    if ( name == "next" ) {
      this.need("next", args, 1, 2);
      const nv = args[0];
      if ( nv.t == 23 ) {
        if ( n == 2 ) {
          if ( this.ip.genRun(nv, this.ip.NONE, 0) ) {
            const ng = nv.gen;
            return ng.out;
          }
          return args[1];
        }
        return this.ip.genStep(nv, this.ip.NONE, 0);
      }
      if ( nv.t == 17 && nv.mode.length > 0 ) {
        const lnv = this.ip.nextOpt(nv);
        if ( (typeof(lnv) !== "undefined" && lnv != null )  ) {
          return lnv;
        }
        if ( n == 2 ) {
          return args[1];
        }
        this.ip.raise("StopIteration", "");
      }
      if ( nv.t == 17 ) {
        if ( nv.pos < nv.items.length ) {
          const r2 = nv.items[nv.pos];
          nv.pos = nv.pos + 1;
          return r2;
        }
        if ( n == 2 ) {
          return args[1];
        }
        this.ip.raise("StopIteration", "");
      }
      if ( nv.t == 12 ) {
        const nm = this.ip.findMethod(nv, "__next__");
        if ( (typeof(nm) !== "undefined" && nm != null )  ) {
          if ( n == 2 ) {
            let got2 = this.ip.NONE;
            let stop = false;
            try {
              got2 = this.ip.callMethodVal(nm, nv, []);
            } catch(e) {
              const ex_1 = this.ip.takeException(( e.toString()));
              if ( this.ip.excMatches(ex_1, this.ip.excClass("StopIteration")) ) {
                stop = true;
              } else {
                this.ip.reraise(ex_1);
              }
            }
            if ( stop ) {
              return args[1];
            }
            return got2;
          }
          return this.ip.callMethodVal(nm, nv, []);
        }
      }
      this.ip.raise("TypeError", ("'" + this.ip.typeName(nv)) + "' object is not an iterator");
    }
    if ( name == "getattr" ) {
      this.need("getattr", args, 2, 3);
      const gname = args[1];
      if ( gname.t != 4 ) {
        this.ip.raise("TypeError", "attribute name must be string");
      }
      if ( n == 3 ) {
        const gv = this.attrOrNone(args[0], gname.s);
        if ( typeof(gv) === "undefined" ) {
          return args[2];
        }
        return gv;
      }
      return this.ip.getAttr(args[0], gname.s);
    }
    if ( name == "hasattr" ) {
      this.need("hasattr", args, 2, 2);
      const hname = args[1];
      const hv2 = this.attrOrNone(args[0], hname.s);
      return this.ip.mkBool(((typeof(hv2) !== "undefined" && hv2 != null ) ));
    }
    if ( name == "setattr" ) {
      this.need("setattr", args, 3, 3);
      this.ip.setAttr(args[0], args[1].s, args[2]);
      return this.ip.NONE;
    }
    if ( name == "delattr" ) {
      this.need("delattr", args, 2, 2);
      this.ip.delAttr(args[0], args[1].s);
      return this.ip.NONE;
    }
    if ( name == "callable" ) {
      this.need("callable", args, 1, 1);
      const cv2 = args[0];
      if ( ((cv2.t == 9 || cv2.t == 10) || cv2.t == 11) || cv2.t == 14 ) {
        return this.ip.TRUE;
      }
      if ( cv2.t == 12 ) {
        const cm = this.ip.findMethod(cv2, "__call__");
        return this.ip.mkBool(((typeof(cm) !== "undefined" && cm != null ) ));
      }
      return this.ip.FALSE;
    }
    if ( name == "input" ) {
      this.ip.raise("EOFError", "EOF when reading a line");
    }
    if ( (name == "vars" || name == "globals") || name == "locals" ) {
      if ( n == 1 ) {
        return this.ip.getAttr(args[0], "__dict__");
      }
      let from = this.ip.callScope;
      if ( name == "globals" ) {
        from = this.ip.moduleScope(this.ip.callScope);
      }
      const gd = this.ip.mkDict();
      const gks = Object.keys(from.vars);
      // Loop start
      for ( const gk of gks) {
        this.ip.dictSet(
          gd,
          this.ip.mkStr(gk),
          ( Object.prototype.hasOwnProperty.call(from.vars, gk) ? from.vars[gk] : undefined )
        );
      }
      return gd;
    }
    if ( name == "eval" || name == "exec" ) {
      const g = this.arg(args, 1, kn, kv, "globals");
      const l_1 = this.arg(args, 2, kn, kv, "locals");
      return this.ip.runCode(name, this.arg(args, 0, kn, kv, "source"), g, l_1);
    }
    if ( name == "compile" ) {
      let mode = this.ip.toStr(this.arg(args, 2, kn, kv, "mode"));
      if ( mode == "single" ) {
        mode = "exec";
      }
      return this.ip.codeOf(this.arg(args, 0, kn, kv, "source"), mode);
    }
    if ( name == "dir" ) {
      let names = [];
      if ( n == 0 ) {
        names = Object.keys(this.ip.globals.vars);
      } else {
        const dv = args[0];
        if ( (dv.t == 12 || dv.t == 13) || dv.t == 11 ) {
          names = dv.attrKeys();
          if ( dv.t == 12 && ((typeof(dv.cls) !== "undefined" && dv.cls != null ) ) ) {
            const dc = dv.cls;
            const cks = dc.attrKeys();
            // Loop start
            for ( const ck of cks) {
              names.push(ck);
            }
          }
        }
      }
      let outn = [];
      const sortedN = PyFileSystem.sortStrings(names);
      let seen = {};
      // Loop start
      for ( const sn of sortedN) {
        if ( ( typeof(seen[sn] ) != "undefined" && Object.prototype.hasOwnProperty.call(seen, sn) ) == false ) {
          seen[sn] = true;
          outn.push(this.ip.mkStr(sn));
        }
      }
      return this.ip.mkList(outn);
    }
    if ( name == "property" ) {
      const p = new PyVal(19);
      const fg = this.arg(args, 0, kn, kv, "fget");
      if ( fg.t != 0 ) {
        p.wrapped = fg;
      }
      const fset = this.arg(args, 1, kn, kv, "fset");
      if ( fset.t != 0 ) {
        p.setter = fset;
      }
      const fdel = this.arg(args, 2, kn, kv, "fdel");
      if ( fdel.t != 0 ) {
        p.putAttr("__deleter__", fdel);
      }
      let pdoc = this.arg(args, 3, kn, kv, "doc");
      if ( pdoc.t == 0 && fg.t == 9 ) {
        pdoc = this.ip.getAttr(fg, "__doc__");
      }
      p.putAttr("__doc__", pdoc);
      return p;
    }
    if ( name == "staticmethod" ) {
      this.need("staticmethod", args, 1, 1);
      const sm = new PyVal(20);
      sm.wrapped = args[0];
      return sm;
    }
    if ( name == "classmethod" ) {
      this.need("classmethod", args, 1, 1);
      const cm2 = new PyVal(21);
      cm2.wrapped = args[0];
      return cm2;
    }
    if ( name == "super" ) {
      this.need("super", args, 2, 2);
      const sp = new PyVal(22);
      sp.cls = args[0];
      sp.__self = args[1];
      return sp;
    }
    if ( name == "object" ) {
      const o = new PyVal(12);
      o.cls = this.objectClass();
      return o;
    }
    if ( name == "slice" ) {
      this.need("slice", args, 1, 3);
      const sl = new PyVal(18);
      if ( n == 1 ) {
        sl.items.push(this.ip.NONE);
        sl.items.push(args[0]);
        sl.items.push(this.ip.NONE);
      } else {
        sl.items.push(args[0]);
        sl.items.push(args[1]);
        if ( n == 3 ) {
          sl.items.push(args[2]);
        } else {
          sl.items.push(this.ip.NONE);
        }
      }
      return sl;
    }
    if ( name == "exit" || name == "quit" ) {
      let code = this.ip.NONE;
      if ( n > 0 ) {
        code = args[0];
      }
      this.ip.raiseVal(this.ip.makeExcArg("SystemExit", code));
    }
    if ( name == "__import__" ) {
      const mn = this.ip.toStr(args[0]);
      const mod = this.ip.importModule(mn);
      const fl = this.arg(args, 3, kn, kv, "fromlist");
      const dot = mn.indexOf(".");
      if ( dot > 0 && this.ip.truthy(fl) == false ) {
        return this.ip.importModule(mn.substring(0, dot ));
      }
      return mod;
    }
    if ( name == "__object_init__" ) {
      if ( n > 0 ) {
        const __self = args[0];
        if ( this.ip.isExcInst(__self) ) {
          let rest = [];
          let ri2 = 1;
          while (ri2 < n) {
            rest.push(args[ri2]);
            ri2 = ri2 + 1;
          };
          __self.putAttr("args", this.ip.mkTuple(rest));
        }
      }
      return this.ip.NONE;
    }
    return this.ip.mods.callBuiltin(name, args, kn, kv);
  };
  objectClass () {
    const oc = ( Object.prototype.hasOwnProperty.call(this.ip.builtins, "__object_class__") ? this.ip.builtins["__object_class__"] : undefined );
    if ( (typeof(oc) !== "undefined" && oc != null )  ) {
      return oc;
    }
    const c = new PyVal(11);
    c.s = "object";
    c.mode = "builtin";
    this.ip.builtins["__object_class__"] = c;
    return c;
  };
  attrOrNone (obj, name) {
    let none;
    const r = this.attrOrMissing(obj, name);
    if ( this.ip.same(r, this.MISSING) ) {
      return none;
    }
    const found = r;
    return found;
  };
  attrOrMissing (obj, name) {
    let r = this.MISSING;
    try {
      r = this.ip.getAttr(obj, name);
    } catch(e) {
      const ex = this.ip.takeException(( e.toString()));
      if ( this.ip.excMatches(ex, this.ip.excClass("AttributeError")) == false ) {
        this.ip.reraise(ex);
      }
    }
    return r;
  };
  fillDict (d, src) {
    if ( src.t == 7 ) {
      // Loop start
      for ( let i = 0; i < src.items.length; i++) {
        var k = src.items[i];
        this.ip.dictSet(d, k, src.vals[i]);
      }
      return;
    }
    if ( src.t == 12 ) {
      const ks = this.attrOrNone(src, "keys");
      if ( (typeof(ks) !== "undefined" && ks != null )  ) {
        this.ip.mergeMapping(d, src);
        return;
      }
    }
    const vals = this.ip.iterValues(src);
    // Loop start
    for ( let j = 0; j < vals.length; j++) {
      var pair = vals[j];
      const kv = this.ip.iterValues(pair);
      if ( kv.length != 2 ) {
        this.ip.raise("ValueError", ((("dictionary update sequence element #" + ((j).toString())) + " has length ") + ((kv.length).toString())) + "; 2 is required");
      }
      this.ip.dictSet(d, kv[0], kv[1]);
    }
  };
  percentFormat (fmt, argv) {
    const cs = Array.from(fmt, (rg_c) => rg_c.codePointAt(0));
    const n = cs.length;
    let args = [];
    let mapping = false;
    if ( argv.t == 6 ) {
      args = argv.items;
    } else {
      args.push(argv);
      if ( argv.t == 7 || argv.t == 12 && ((typeof(this.ip.findMethod(argv, "__getitem__")) !== "undefined" && this.ip.findMethod(argv, "__getitem__") != null ) ) ) {
        mapping = true;
      }
    }
    let ai = 0;
    let out = [];
    let lit = [];
    let i = 0;
    let usedMapping = false;
    while (i < n) {
      const c = cs[i];
      if ( c != 37 ) {
        lit.push(c);
        i = i + 1;
        continue;
      }
      if ( lit.length > 0 ) {
        out.push(py_rt_codes(lit, 0, lit.length));
        lit.length = 0;
      }
      i = i + 1;
      if ( i >= n ) {
        this.ip.raise("ValueError", "incomplete format");
      }
      if ( cs[i] == 37 ) {
        out.push("%");
        i = i + 1;
        continue;
      }
      let val;
      if ( cs[i] == 40 ) {
        let j = i + 1;
        let depth = 1;
        while (j < n && depth > 0) {
          if ( cs[j] == 40 ) {
            depth = depth + 1;
          }
          if ( cs[j] == 41 ) {
            depth = depth - 1;
          }
          j = j + 1;
        };
        const key = py_rt_codes(cs, i + 1, j - 1);
        if ( mapping == false ) {
          this.ip.raise("TypeError", "format requires a mapping");
        }
        val = this.ip.getItem(argv, this.ip.mkStr(key));
        usedMapping = true;
        i = j;
      }
      const sp = new PySpec();
      sp.align = ">";
      let leftAlign = false;
      while (i < n) {
        const f = cs[i];
        if ( f == 45 ) {
          leftAlign = true;
        } else {
          if ( f == 43 ) {
            sp.sign = "+";
          } else {
            if ( f == 32 ) {
              if ( sp.sign != "+" ) {
                sp.sign = " ";
              }
            } else {
              if ( f == 35 ) {
                sp.alt = true;
              } else {
                if ( f == 48 ) {
                  sp.zero = true;
                } else {
                  break;
                }
              }
            }
          }
        }
        i = i + 1;
      };
      if ( i < n && cs[i] == 42 ) {
        if ( ai >= args.length ) {
          this.ip.raise("TypeError", "not enough arguments for format string");
        }
        const wv = args[ai];
        ai = ai + 1;
        sp.width = this.toInt(wv, "width");
        if ( sp.width < 0 ) {
          leftAlign = true;
          sp.width = 0 - sp.width;
        }
        i = i + 1;
      } else {
        while (i < n && PyLexer.isDigit(cs[i])) {
          sp.width = sp.width * 10 + (cs[i] - 48);
          i = i + 1;
        };
      }
      if ( i < n && cs[i] == 46 ) {
        i = i + 1;
        sp.prec = 0;
        if ( i < n && cs[i] == 42 ) {
          const pv = args[ai];
          ai = ai + 1;
          sp.prec = this.toInt(pv, "precision");
          i = i + 1;
        } else {
          while (i < n && PyLexer.isDigit(cs[i])) {
            sp.prec = sp.prec * 10 + (cs[i] - 48);
            i = i + 1;
          };
        }
      }
      while (i < n && ((cs[i] == 104 || cs[i] == 108) || cs[i] == 76)) {
        i = i + 1;
      };
      if ( i >= n ) {
        this.ip.raise("ValueError", "incomplete format");
      }
      const tc = cs[i];
      const t = String.fromCodePoint(tc);
      i = i + 1;
      if ( typeof(val) === "undefined" ) {
        if ( ai >= args.length ) {
          this.ip.raise("TypeError", "not enough arguments for format string");
        }
        val = args[ai];
        ai = ai + 1;
      }
      const v = val;
      if ( leftAlign ) {
        sp.align = "<";
        sp.zero = false;
      } else {
        if ( sp.zero ) {
          sp.align = "=";
          sp.fill = "0";
        }
      }
      if ( (t == "s" || t == "r") || t == "a" ) {
        let text = "";
        if ( t == "s" ) {
          text = this.ip.toStr(v);
        } else {
          text = this.ip.toRepr(v);
          if ( t == "a" ) {
            text = this.asciiOf(text);
          }
        }
        if ( sp.prec >= 0 ) {
          const tcs = Array.from(text, (rg_c) => rg_c.codePointAt(0));
          if ( tcs.length > sp.prec ) {
            text = py_rt_codes(tcs, 0, sp.prec);
          }
        }
        const ssp = new PySpec();
        ssp.width = sp.width;
        ssp.align = sp.align;
        if ( ssp.align == "=" ) {
          ssp.align = ">";
        }
        out.push(PyFormat.pad("", text, ssp, false));
        continue;
      }
      if ( t == "c" ) {
        let ch = "";
        if ( v.t == 4 ) {
          ch = v.s;
        } else {
          ch = String.fromCodePoint(this.toInt(v, "%c"));
        }
        const csp = new PySpec();
        csp.width = sp.width;
        csp.align = sp.align;
        if ( csp.align == "=" ) {
          csp.align = ">";
        }
        out.push(PyFormat.pad("", ch, csp, false));
        continue;
      }
      if ( ((((t == "d" || t == "i") || t == "u") || t == "x") || t == "X") || t == "o" ) {
        if ( this.ip.isNum(v) == false ) {
          const iv = this.ip.findMethod(v, "__int__");
          if ( typeof(iv) === "undefined" ) {
            this.ip.raise("TypeError", (("%" + t) + " format: a real number is required, not ") + this.ip.typeName(v));
          }
        }
        if ( v.t == 3 && ((t == "x" || t == "X") || t == "o") ) {
          this.ip.raise("TypeError", ("%" + t) + " format: an integer is required, not float");
        }
        const iv2 = this.toIntValue(v);
        const isp = new PySpec();
        isp.width = sp.width;
        isp.align = sp.align;
        isp.fill = sp.fill;
        isp.sign = sp.sign;
        isp.zero = sp.zero;
        isp.alt = sp.alt;
        if ( (t == "x" || t == "X") || t == "o" ) {
          isp.type = t;
        }
        let digitsOut = PyFormat.formatInt(iv2.i, isp);
        if ( sp.prec > 0 ) {
          let body = PyFormat.intBase(iv2.i, 10, false);
          if ( t == "x" || t == "X" ) {
            body = PyFormat.intBase(iv2.i, 16, (t == "X"));
          }
          if ( t == "o" ) {
            body = PyFormat.intBase(iv2.i, 8, false);
          }
          while (body.length < sp.prec) {
            body = "0" + body;
          };
          const psp = new PySpec();
          psp.width = sp.width;
          psp.align = sp.align;
          if ( psp.align == "=" ) {
            psp.align = ">";
          }
          digitsOut = PyFormat.pad(
            PyFormat.signText((iv2.i < 0), sp),
            body,
            psp,
            true
          );
        }
        out.push(digitsOut);
        continue;
      }
      if ( ((((t == "f" || t == "F") || t == "e") || t == "E") || t == "g") || t == "G" ) {
        const fsp = new PySpec();
        fsp.width = sp.width;
        fsp.align = sp.align;
        fsp.fill = sp.fill;
        fsp.sign = sp.sign;
        fsp.zero = sp.zero;
        fsp.alt = sp.alt;
        fsp.prec = sp.prec;
        fsp.type = t;
        const fo = PyFormat.formatFloat(this.toNum(v), fsp);
        out.push(fo);
        continue;
      }
      this.ip.raise("ValueError", ((("unsupported format character '" + t) + "' (0x") + PyFormat.intBase(
        tc,
        16,
        false
      )) + ")");
    };
    if ( lit.length > 0 ) {
      out.push(py_rt_codes(lit, 0, lit.length));
    }
    if ( (usedMapping == false && ai < args.length) && mapping == false ) {
      this.ip.raise("TypeError", "not all arguments converted during string formatting");
    }
    return out.join("");
  };
}
export class PyMethods  {
  constructor(interp) {
    this.ip = undefined;
    this.table = {};
    this.ip = interp;
  }
  init () {
    this.addNames("str", "join split rsplit splitlines strip lstrip rstrip replace upper lower title capitalize swapcase casefold startswith endswith find rfind index rindex count format format_map zfill ljust rjust center isdigit isnumeric isdecimal isalpha isalnum isspace isupper islower istitle isidentifier isascii isprintable partition rpartition encode expandtabs removeprefix removesuffix translate __len__ __contains__ __getitem__ __add__ __mul__ __eq__ __lt__ __hash__ __iter__ __str__ __repr__");
    this.addNames("list", "__init__ append extend insert pop remove index count sort reverse copy clear __len__ __getitem__ __setitem__ __contains__ __iter__ __add__ __eq__ __repr__");
    this.addNames("deque", "append appendleft extend extendleft pop popleft remove index count reverse rotate copy clear __len__");
    this.addNames("tuple", "index count __len__ __getitem__ __contains__ __iter__ __add__ __eq__ __hash__ __repr__");
    this.addNames("dict", "__init__ move_to_end get keys values items pop popitem setdefault update copy clear __len__ __getitem__ __setitem__ __contains__ __iter__ __eq__ __repr__ most_common elements total subtract");
    this.addNames("set", "__init__ add remove discard pop clear copy update union intersection difference symmetric_difference issubset issuperset isdisjoint intersection_update difference_update symmetric_difference_update __len__ __contains__ __iter__");
    this.addNames("frozenset", "copy union intersection difference symmetric_difference issubset issuperset isdisjoint __len__ __contains__ __iter__ __hash__");
    this.addNames("file", "write writelines read readline readlines close flush seek tell __enter__ __exit__ __iter__ __next__ readable writable");
    this.addNames("int", "bit_length bit_count as_integer_ratio conjugate is_integer __index__ __int__ __float__ __abs__ __add__ __sub__ __mul__ __eq__ __lt__ __hash__ to_bytes __repr__ __str__ __format__ __bool__ __neg__");
    this.addNames("float", "is_integer conjugate hex __int__ __float__ __abs__ __add__ __sub__ __mul__ __eq__ __lt__ __hash__ as_integer_ratio __repr__ __str__ __format__");
    this.addNames("iter", "__next__ __iter__");
    this.addNames("complex", "conjugate __abs__ __add__ __sub__ __mul__ __truediv__ __pow__ __eq__ __hash__ __neg__ __pos__ __bool__ __complex__ __radd__ __rsub__ __rmul__ __rtruediv__ __rpow__ __repr__ __str__ __getnewargs__ __format__");
    this.addNames("generator", "__next__ __iter__ send throw close");
    this.addNames("range", "index count __len__ __iter__ __contains__");
  };
  addNames (kind, names) {
    const ns = names.split(" ");
    // Loop start
    for ( const n of ns) {
      this.table[(kind + ".") + n] = true;
    }
  };
  kindOf (v) {
    switch (v.t ) { 
      case 4 : 
        return "str";
      case 5 : 
        if ( v.mode == "deque" ) {
          return "deque";
        }
        return "list";
      case 6 : 
        return "tuple";
      case 7 : 
        return "dict";
      case 8 : 
        if ( v.mode == "frozen" ) {
          return "frozenset";
        }
        return "set";
      case 16 : 
        return "file";
      case 2 : 
        return "int";
      case 1 : 
        return "int";
      case 3 : 
        return "float";
      case 24 : 
        return "complex";
      case 17 : 
        return "iter";
      case 23 : 
        return "generator";
      case 15 : 
        return "range";
    };
    return "";
  };
  attrOf (obj, name) {
    let none;
    const kind = this.kindOf(obj);
    if ( kind.length == 0 ) {
      return none;
    }
    if ( ( typeof(this.table[((kind + ".") + name)] ) != "undefined" && Object.prototype.hasOwnProperty.call(this.table, (kind + ".") + name) ) ) {
      const optr95 = this.ip.mkBuiltinMethod(obj, name);
      return optr95;
    }
    if ( obj.t == 2 || obj.t == 1 ) {
      if ( name == "real" || name == "numerator" ) {
        const optr100 = this.ip.mkInt(obj.i);
        return optr100;
      }
      if ( name == "imag" ) {
        const optr103 = this.ip.mkInt(0);
        return optr103;
      }
      if ( name == "denominator" ) {
        const optr106 = this.ip.mkInt(1);
        return optr106;
      }
    }
    if ( obj.t == 3 ) {
      if ( name == "real" ) {
        const optr111 = obj;
        return optr111;
      }
      if ( name == "imag" ) {
        const optr114 = this.ip.mkFloat(0.0);
        return optr114;
      }
    }
    if ( obj.t == 16 ) {
      if ( name == "name" ) {
        const optr119 = this.ip.mkStr(obj.s);
        return optr119;
      }
      if ( name == "mode" ) {
        const optr122 = this.ip.mkStr(obj.mode);
        return optr122;
      }
      if ( name == "closed" ) {
        const optr125 = this.ip.mkBool(obj.closed);
        return optr125;
      }
    }
    if ( obj.t == 24 ) {
      if ( name == "real" ) {
        const optrc1 = this.ip.mkFloat(obj.d);
        return optrc1;
      }
      if ( name == "imag" ) {
        const optrc2 = this.ip.mkFloat(obj.d2);
        return optrc2;
      }
    }
    if ( obj.t == 23 ) {
      const g = obj.gen;
      if ( name == "gi_running" ) {
        const optrg1 = this.ip.mkBool(g.running);
        return optrg1;
      }
      if ( name == "__name__" ) {
        const optrg2 = this.ip.mkStr(g.name);
        return optrg2;
      }
      if ( name == "__qualname__" ) {
        const optrg3 = this.ip.mkStr(g.name);
        return optrg3;
      }
      if ( name == "gi_frame" ) {
        if ( g.done ) {
          const optrg4 = this.ip.NONE;
          return optrg4;
        }
        const fo = new PyVal(12);
        fo.putAttr("f_lineno", this.ip.mkInt(g.line));
        fo.putAttr("f_locals", this.ip.NONE);
        const optrg5 = fo;
        return optrg5;
      }
      if ( name == "gi_yieldfrom" ) {
        const optrg6 = this.ip.NONE;
        return optrg6;
      }
    }
    if ( obj.t == 15 ) {
      if ( name == "start" ) {
        const optr130 = this.ip.mkInt(obj.i);
        return optr130;
      }
      if ( name == "stop" ) {
        const optr133 = this.ip.mkInt(obj.i2);
        return optr133;
      }
      if ( name == "step" ) {
        const optr136 = this.ip.mkInt(obj.i3);
        return optr136;
      }
    }
    if ( obj.t == 18 ) {
      if ( name == "start" ) {
        const optr141 = obj.items[0];
        return optr141;
      }
      if ( name == "stop" ) {
        const optr144 = obj.items[1];
        return optr144;
      }
      if ( name == "step" ) {
        const optr147 = obj.items[2];
        return optr147;
      }
    }
    if ( obj.t == 7 && name == "default_factory" ) {
      return ( Object.prototype.hasOwnProperty.call(obj.attrs, "__missing__") ? obj.attrs["__missing__"] : undefined );
    }
    if ( (obj.t == 5 && obj.mode == "deque") && name == "maxlen" ) {
      if ( obj.i3 > 0 ) {
        const optr155 = this.ip.mkInt((obj.i3 - 1));
        return optr155;
      }
      const optr157 = this.ip.NONE;
      return optr157;
    }
    return none;
  };
  typeAttr (tv, name) {
    let none;
    const kind = tv.s;
    if ( name == "__new__" && ((((((((kind == "int" || kind == "float") || kind == "str") || kind == "tuple") || kind == "frozenset") || kind == "complex") || kind == "list") || kind == "dict") || kind == "set") ) {
      const optrnw = this.ip.mkBuiltinMethod(tv, "type.__new__");
      return optrnw;
    }
    if ( name == "__new__" && kind == "type" ) {
      const optrtn = this.ip.mkBuiltinMethod(tv, "metatype.__new__");
      return optrtn;
    }
    if ( kind == "type" && ((name == "__setattr__" || name == "__delattr__") || name == "__getattribute__") ) {
      const optrts = this.ip.mkBuiltinMethod(( Object.prototype.hasOwnProperty.call(this.ip.builtins, "object") ? this.ip.builtins["object"] : undefined ), ("object." + name));
      return optrts;
    }
    if ( kind == "object" && (((((((((((((((((name == "__setattr__" || name == "__delattr__") || name == "__getattribute__") || name == "__repr__") || name == "__str__") || name == "__eq__") || name == "__ne__") || name == "__hash__") || name == "__format__") || name == "__dir__") || name == "__reduce_ex__") || name == "__reduce__") || name == "__sizeof__") || name == "__lt__") || name == "__le__") || name == "__gt__") || name == "__ge__") || name == "__getstate__") ) {
      const optrob = this.ip.mkBuiltinMethod(tv, ("object." + name));
      return optrob;
    }
    if ( name == "__new__" && kind == "object" ) {
      const optron = this.ip.mkBuiltinMethod(tv, "object.__new__");
      return optron;
    }
    if ( name == "__init__" && kind == "object" ) {
      return ( Object.prototype.hasOwnProperty.call(this.ip.builtins, "__object_init__") ? this.ip.builtins["__object_init__"] : undefined );
    }
    if ( name == "__init__" && ((kind == "list" || kind == "dict") || kind == "set") ) {
      const optrin = this.ip.mkBuiltinMethod(tv, "unbound:__init__");
      return optrin;
    }
    if ( kind == "dict" && name == "fromkeys" ) {
      const optr167 = this.ip.mkBuiltinMethod(tv, "dict.fromkeys");
      return optr167;
    }
    if ( kind == "str" && name == "maketrans" ) {
      const optr170 = this.ip.mkBuiltinMethod(tv, "str.maketrans");
      return optr170;
    }
    if ( kind == "int" && name == "from_bytes" ) {
      return none;
    }
    if ( ( typeof(this.table[((kind + ".") + name)] ) != "undefined" && Object.prototype.hasOwnProperty.call(this.table, (kind + ".") + name) ) ) {
      const optr176 = this.ip.mkBuiltinMethod(tv, ("unbound:" + name));
      return optr176;
    }
    return none;
  };
  arg (args, i) {
    if ( i < args.length ) {
      return args[i];
    }
    return this.ip.NONE;
  };
  need (name, args, lo, hi) {
    this.ip.bi.need(name, args, lo, hi);
  };
  strArg (v, what) {
    if ( v.t != 4 ) {
      this.ip.raise("TypeError", (what + " must be str, not ") + this.ip.typeName(v));
    }
    return v.s;
  };
  callMethod (__self, name, args, kn, kv) {
    if ( this.ip.isDataSub(__self) ) {
      const saved = this.ip.rawSelf;
      this.ip.rawSelf = __self;
      let r = this.ip.NONE;
      let failed = false;
      let err = this.ip.NONE;
      try {
        r = this.callMethodPlain(__self, name, args, kn, kv);
      } catch(e) {
        failed = true;
        err = this.ip.takeException(( e.toString()));
      }
      this.ip.rawSelf = saved;
      if ( failed ) {
        this.ip.reraise(err);
      }
      return r;
    }
    return this.callMethodPlain(__self, name, args, kn, kv);
  };
  callMethodPlain (__self, name, args, kn, kv) {
    if ( name == "__init__" ) {
      return this.initMethod(__self, args, kn, kv);
    }
    if ( name == "__noop__" ) {
      if ( kn.length > 0 ) {
        this.ip.raise("TypeError", "object.__init_subclass__() takes no keyword arguments");
      }
      return this.ip.NONE;
    }
    if ( name.startsWith("object.__") && name != "object.__new__" ) {
      let target = __self;
      let rest = args;
      if ( __self.t == 10 && __self.s == "object" ) {
        if ( args.length == 0 ) {
          this.ip.raise("TypeError", ("descriptor '" + name.substring(7, name.length )) + "' of 'object' object needs an argument");
        }
        target = args[0];
        let r2 = [];
        let ri = 1;
        while (ri < args.length) {
          r2.push(args[ri]);
          ri = ri + 1;
        };
        rest = r2;
      }
      const on = name.substring(7, name.length );
      if ( on == "__setattr__" ) {
        this.ip.objectSetAttr(
          target,
          this.ip.toStr(this.arg(rest, 0)),
          this.arg(rest, 1)
        );
        return this.ip.NONE;
      }
      if ( on == "__delattr__" ) {
        this.ip.objectDelAttr(target, this.ip.toStr(this.arg(rest, 0)));
        return this.ip.NONE;
      }
      if ( on == "__getattribute__" ) {
        const gn = this.ip.toStr(this.arg(rest, 0));
        const gv = this.ip.getAttrNoHook(target, gn);
        if ( typeof(gv) === "undefined" ) {
          this.ip.raise("AttributeError", ((("'" + this.ip.typeName(target)) + "' object has no attribute '") + gn) + "'");
        }
        return gv;
      }
      if ( on == "__repr__" || on == "__str__" ) {
        if ( on == "__str__" && target.t == 12 ) {
          return this.ip.mkStr(this.ip.toRepr(target));
        }
        return this.ip.mkStr(this.ip.defaultRepr(target));
      }
      if ( on == "__eq__" ) {
        if ( this.ip.same(target, this.arg(rest, 0)) ) {
          return this.ip.TRUE;
        }
        return this.ip.NOTIMPL;
      }
      if ( on == "__ne__" ) {
        const eqr = this.ip.binop("==", target, this.arg(rest, 0));
        return this.ip.mkBool((this.ip.truthy(eqr) == false));
      }
      if ( on == "__hash__" ) {
        return this.ip.mkInt(PyFormat.floorDivInt(this.ip.idOf(target), 16));
      }
      if ( on == "__format__" ) {
        const fs = this.ip.toStr(this.arg(rest, 0));
        if ( fs.length > 0 ) {
          this.ip.raise("TypeError", ("unsupported format string passed to " + this.ip.typeName(target)) + ".__format__");
        }
        return this.ip.mkStr(this.ip.toStr(target));
      }
      if ( on == "__dir__" ) {
        let dkn = [];
        let dkv = [];
        return this.ip.bi.call("dir", [target], dkn, dkv);
      }
      if ( on == "__sizeof__" ) {
        return this.ip.mkInt(16);
      }
      if ( on == "__getstate__" ) {
        if ( target.t == 12 && target.attrKeys().length > 0 ) {
          return this.ip.getAttr(target, "__dict__");
        }
        return this.ip.NONE;
      }
      if ( ((on == "__lt__" || on == "__le__") || on == "__gt__") || on == "__ge__" ) {
        return this.ip.NOTIMPL;
      }
      this.ip.raise("TypeError", ("cannot pickle '" + this.ip.typeName(target)) + "' object");
    }
    if ( name == "metatype.__new__" ) {
      if ( args.length == 0 ) {
        this.ip.raise("TypeError", "type.__new__(): not enough arguments");
      }
      let mrest = [];
      let mi = 1;
      while (mi < args.length) {
        mrest.push(args[mi]);
        mi = mi + 1;
      };
      return this.ip.typeNew(args[0], mrest, kn, kv);
    }
    if ( name == "type.__call__" ) {
      return this.ip.instantiate(__self, args, kn, kv);
    }
    if ( name == "object.__new__" ) {
      if ( args.length == 0 ) {
        this.ip.raise("TypeError", "object.__new__(): not enough arguments");
      }
      const ocls = args[0];
      if ( ocls.t != 11 ) {
        this.ip.raise("TypeError", ("object.__new__(X): X is not a type object (" + this.ip.typeName(ocls)) + ")");
      }
      if ( ocls.bt.length > 0 ) {
        let okn = [];
        let okv = [];
        let onone = [];
        return this.ip.makeDataSub(ocls, onone, okn, okv);
      }
      const oobj = new PyVal(12);
      oobj.cls = ocls;
      if ( ocls.isExc ) {
        let orest = [];
        let oi = 1;
        while (oi < args.length) {
          orest.push(args[oi]);
          oi = oi + 1;
        };
        oobj.putAttr("args", this.ip.mkTuple(orest));
      }
      return oobj;
    }
    if ( name == "type.__new__" ) {
      if ( args.length == 0 ) {
        this.ip.raise("TypeError", __self.s + ".__new__(): not enough arguments");
      }
      const ncls = args[0];
      let rest0 = [];
      let ni = 1;
      while (ni < args.length) {
        rest0.push(args[ni]);
        ni = ni + 1;
      };
      const made = this.ip.bi.call(__self.s, rest0, kn, kv);
      if ( ncls.t == 10 && ncls.s == __self.s ) {
        return made;
      }
      if ( ncls.t != 11 || ncls.bt != __self.s ) {
        this.ip.raise("TypeError", (((((__self.s + ".__new__(") + this.ip.typeName(ncls)) + "): ") + this.ip.typeName(ncls)) + " is not a subtype of ") + __self.s);
      }
      const nv = this.ip.cloneVal(made);
      nv.cls = ncls;
      return nv;
    }
    if ( name == "__counter_key__" ) {
      const ck = args[0];
      return ck.items[1];
    }
    if ( name.startsWith("nt.") ) {
      return this.ip.mods.ntMethod(__self, name, args, kn, kv);
    }
    if ( name == "lru.cache_clear" ) {
      __self.putAttr("__cache__", this.ip.mkDict());
      return this.ip.NONE;
    }
    if ( name.startsWith("unbound:") ) {
      const m = name.substring(8, name.length );
      if ( args.length == 0 ) {
        this.ip.raise("TypeError", ((("unbound method " + __self.s) + ".") + m) + "() needs an argument");
      }
      let rest_1 = [];
      let i = 1;
      while (i < args.length) {
        rest_1.push(args[i]);
        i = i + 1;
      };
      const ut = args[0];
      if ( m.startsWith("__") && this.ip.isDataSub(ut) ) {
        const saved = this.ip.rawSelf;
        this.ip.rawSelf = ut;
        const ur = this.callMethod(ut, m, rest_1, kn, kv);
        this.ip.rawSelf = saved;
        return ur;
      }
      return this.callMethod(ut, m, rest_1, kn, kv);
    }
    if ( name == "dict.fromkeys" ) {
      const d = this.ip.mkDict();
      const v = this.arg(args, 1);
      const ks = this.ip.iterValues(this.arg(args, 0));
      // Loop start
      for ( const k of ks) {
        this.ip.dictSet(d, k, v);
      }
      return d;
    }
    if ( name == "str.maketrans" ) {
      return this.makeTrans(args);
    }
    if ( name == "prop.__get__" ) {
      let nk = [];
      let nv_1 = [];
      const pin = this.arg(args, 0);
      if ( pin.t == 0 || (typeof(__self.wrapped) === "undefined") ) {
        if ( pin.t == 0 ) {
          return __self;
        }
        this.ip.raise("AttributeError", "property has no getter");
      }
      return this.ip.callValue(__self.wrapped, [pin], nk, nv_1);
    }
    if ( name == "prop.__set__" ) {
      let nk_1 = [];
      let nv_2 = [];
      if ( typeof(__self.setter) === "undefined" ) {
        this.ip.raise("AttributeError", "property has no setter");
      }
      return this.ip.callValue(
        __self.setter,
        [this.arg(args, 0), this.arg(args, 1)],
        nk_1,
        nv_2
      );
    }
    if ( name == "prop.__delete__" ) {
      let nk_2 = [];
      let nv_3 = [];
      const pdel = ( Object.prototype.hasOwnProperty.call(__self.attrs, "__deleter__") ? __self.attrs["__deleter__"] : undefined );
      if ( typeof(pdel) === "undefined" ) {
        this.ip.raise("AttributeError", "property has no deleter");
      }
      return this.ip.callValue(pdel, [this.arg(args, 0)], nk_2, nv_3);
    }
    if ( name.startsWith("property.") ) {
      const p = new PyVal(19);
      p.wrapped = __self.wrapped;
      p.setter = __self.setter;
      // Loop start
      for ( let pki = 0; pki < __self.attrKeys().length; pki++) {
        var pk = __self.attrKeys()[pki];
        p.putAttr(pk, ( Object.prototype.hasOwnProperty.call(__self.attrs, pk) ? __self.attrs[pk] : undefined ));
      }
      if ( name == "property.deleter" ) {
        p.putAttr("__deleter__", this.arg(args, 0));
      }
      if ( name == "property.setter" ) {
        p.setter = this.arg(args, 0);
      }
      if ( name == "property.getter" ) {
        p.wrapped = this.arg(args, 0);
      }
      return p;
    }
    if ( name == "__str__" ) {
      return this.ip.mkStr(this.ip.toStr(__self));
    }
    if ( name == "__repr__" ) {
      return this.ip.mkStr(this.ip.toRepr(__self));
    }
    if ( name == "__len__" ) {
      return this.ip.mkInt(this.ip.length(__self));
    }
    if ( name == "__contains__" ) {
      return this.ip.mkBool(this.ip.contains(__self, this.arg(args, 0)));
    }
    if ( name == "__getitem__" ) {
      return this.ip.getItem(__self, this.arg(args, 0));
    }
    if ( name == "__setitem__" ) {
      this.ip.setItem(__self, this.arg(args, 0), this.arg(args, 1));
      return this.ip.NONE;
    }
    if ( name == "__iter__" ) {
      if ( (__self.t == 17 || __self.t == 16) || __self.t == 23 ) {
        return __self;
      }
      return this.ip.mkIter(this.ip.iterValues(__self));
    }
    if ( name == "__hash__" ) {
      return this.ip.bi.call("hash", [__self], kn, kv);
    }
    if ( name == "__eq__" ) {
      return this.ip.mkBool(this.ip.eq(__self, this.arg(args, 0)));
    }
    if ( name == "__lt__" ) {
      return this.ip.mkBool(this.ip.less(__self, this.arg(args, 0)));
    }
    if ( name == "__add__" ) {
      return this.ip.binop("+", __self, this.arg(args, 0));
    }
    if ( name == "__sub__" ) {
      return this.ip.binop("-", __self, this.arg(args, 0));
    }
    if ( name == "__mul__" ) {
      return this.ip.binop("*", __self, this.arg(args, 0));
    }
    switch (__self.t ) { 
      case 4 : 
        return this.strMethod(__self, name, args, kn, kv);
      case 5 : 
        return this.listMethod(__self, name, args, kn, kv);
      case 6 : 
        return this.seqCommon(__self, name, args);
      case 7 : 
        return this.dictMethod(__self, name, args, kn, kv);
      case 8 : 
        return this.setMethod(__self, name, args);
      case 16 : 
        return this.fileMethod(__self, name, args, kn, kv);
      case 23 : 
        return this.genMethod(__self, name, args);
      case 24 : 
        return this.complexMethod(__self, name, args);
      case 17 : 
        if ( name == "__next__" && __self.mode.length > 0 ) {
          const lnx = this.ip.nextOpt(__self);
          if ( typeof(lnx) === "undefined" ) {
            this.ip.raise("StopIteration", "");
          }
          return lnx;
        }
        if ( name == "__next__" ) {
          if ( __self.pos < __self.items.length ) {
            const r = __self.items[__self.pos];
            __self.pos = __self.pos + 1;
            return r;
          }
          this.ip.raise("StopIteration", "");
        }
        break;
      case 15 : 
        return this.seqCommon(__self, name, args);
    };
    if ( (__self.t == 2 || __self.t == 1) || __self.t == 3 ) {
      return this.numMethod(__self, name, args);
    }
    this.ip.raise("AttributeError", ((("'" + this.ip.typeName(__self)) + "' object has no attribute '") + name) + "'");
    return this.ip.NONE;
  };
  numMethod (__self, name, args) {
    if ( name == "bit_length" ) {
      let v = __self.i;
      if ( v < 0 ) {
        v = 0 - v;
      }
      let n = 0;
      while (v > 0) {
        v = PyFormat.floorDivInt(v, 2);
        n = n + 1;
      };
      return this.ip.mkInt(n);
    }
    if ( name == "conjugate" ) {
      return __self;
    }
    if ( name == "is_integer" ) {
      if ( __self.t == 3 ) {
        return this.ip.mkBool((Math.floor(__self.d) == __self.d));
      }
      return this.ip.TRUE;
    }
    if ( name == "__index__" || name == "__int__" ) {
      return this.ip.bi.toIntValue(__self);
    }
    if ( name == "__float__" ) {
      return this.ip.mkFloat(this.ip.toDouble(__self));
    }
    if ( name == "__abs__" ) {
      let kn = [];
      let kv = [];
      return this.ip.bi.call("abs", [__self], kn, kv);
    }
    if ( name == "as_integer_ratio" && __self.t != 3 ) {
      return this.ip.pair(this.ip.bi.toIntValue(__self), this.ip.mkInt(1));
    }
    if ( name == "bit_count" ) {
      let bv = __self.i;
      if ( bv < 0 ) {
        bv = 0 - bv;
      }
      let bc = 0;
      while (bv > 0) {
        bc = bc + (bv - PyFormat.floorDivInt(bv, 2) * 2);
        bv = PyFormat.floorDivInt(bv, 2);
      };
      return this.ip.mkInt(bc);
    }
    if ( name == "numerator" || name == "real" ) {
      return __self;
    }
    if ( name == "as_integer_ratio" ) {
      let x = this.ip.toDouble(__self);
      let den = 1.0;
      while (Math.floor(x) != x && den < 4000000000000000000.0) {
        x = x * 2.0;
        den = den * 2.0;
      };
      return this.ip.pair(this.ip.mkInt(Math.trunc(x)), this.ip.mkInt(Math.trunc(den)));
    }
    this.ip.raise("AttributeError", ((("'" + this.ip.typeName(__self)) + "' object has no attribute '") + name) + "'");
    return this.ip.NONE;
  };
  complexMethod (__self, name, args) {
    if ( name == "conjugate" ) {
      return this.ip.mkComplex(__self.d, (0.0 - __self.d2));
    }
    if ( name == "__complex__" ) {
      return __self;
    }
    if ( name == "__getnewargs__" ) {
      return this.ip.mkTuple([this.ip.mkFloat(__self.d), this.ip.mkFloat(__self.d2)]);
    }
    if ( name == "__bool__" ) {
      return this.ip.mkBool(this.ip.truthy(__self));
    }
    if ( name == "__repr__" || name == "__str__" ) {
      return this.ip.mkStr(this.ip.complexRepr(__self));
    }
    if ( name == "__format__" ) {
      return this.ip.mkStr(this.ip.formatValue(__self, this.ip.toStr(this.arg(args, 0))));
    }
    if ( name == "__neg__" || name == "__pos__" ) {
      let op = "-";
      if ( name == "__pos__" ) {
        op = "+";
      }
      return this.ip.unary(op, __self);
    }
    let kn = [];
    let kv = [];
    if ( name == "__abs__" ) {
      return this.ip.bi.call("abs", [__self], kn, kv);
    }
    if ( name == "__hash__" ) {
      return this.ip.bi.call("hash", [__self], kn, kv);
    }
    const o = this.arg(args, 0);
    if ( this.ip.isComplexNum(o) == false ) {
      return this.ip.NOTIMPL;
    }
    if ( name == "__eq__" ) {
      return this.ip.mkBool(this.ip.eq(__self, o));
    }
    const ops = ["add", "sub", "mul", "truediv", "pow"];
    const sym = ["+", "-", "*", "/", "**"];
    // Loop start
    for ( let i = 0; i < ops.length; i++) {
      var nm = ops[i];
      if ( name == ("__" + nm) + "__" ) {
        return this.ip.binop(sym[i], __self, o);
      }
      if ( name == ("__r" + nm) + "__" ) {
        return this.ip.binop(sym[i], o, __self);
      }
    }
    this.ip.raise("AttributeError", ("'complex' object has no attribute '" + name) + "'");
    return this.ip.NONE;
  };
  initMethod (__self, args, kn, kv) {
    let none = [];
    if ( __self.t == 5 ) {
      __self.items = none;
      if ( args.length > 0 ) {
        // Loop start
        for ( let i = 0; i < this.ip.iterValues(args[0]).length; i++) {
          var v = this.ip.iterValues(args[0])[i];
          __self.items.push(v);
        }
      }
      return this.ip.NONE;
    }
    if ( __self.t == 7 ) {
      let noK = [];
      let noV = [];
      __self.items = noK;
      __self.vals = noV;
      let ni = {};
      __self.index = ni;
      return this.dictMethod(__self, "update", args, kn, kv);
    }
    if ( __self.t == 8 && __self.mode != "frozen" ) {
      __self.items = none;
      let ni2 = {};
      __self.index = ni2;
      if ( args.length > 0 ) {
        // Loop start
        for ( let j = 0; j < this.ip.iterValues(args[0]).length; j++) {
          var v2 = this.ip.iterValues(args[0])[j];
          this.ip.setAdd(__self, v2);
        }
      }
      return this.ip.NONE;
    }
    return this.ip.NONE;
  };
  genMethod (__self, name, args) {
    if ( name == "__next__" ) {
      return this.ip.genStep(__self, this.ip.NONE, 0);
    }
    if ( name == "send" ) {
      return this.ip.genStep(__self, this.arg(args, 0), 0);
    }
    if ( name == "throw" ) {
      let e = this.arg(args, 0);
      if ( e.t == 11 ) {
        let kn = [];
        let kv = [];
        let eargs = [];
        if ( args.length > 1 ) {
          const a1 = args[1];
          if ( a1.t != 0 ) {
            eargs.push(a1);
          }
        }
        e = this.ip.instantiate(e, eargs, kn, kv);
      }
      if ( this.ip.isExcInst(e) == false ) {
        this.ip.raise("TypeError", "exceptions must be classes or instances deriving from BaseException, not " + this.ip.typeName(e));
      }
      return this.ip.genStep(__self, e, 1);
    }
    if ( name == "close" ) {
      this.ip.genClose(__self);
      return this.ip.NONE;
    }
    this.ip.raise("AttributeError", ("'generator' object has no attribute '" + name) + "'");
    return this.ip.NONE;
  };
  seqCommon (__self, name, args) {
    if ( name == "count" ) {
      this.need("count", args, 1, 1);
      let c = 0;
      const vals = this.ip.iterValues(__self);
      // Loop start
      for ( const v of vals) {
        if ( this.ip.eq(v, args[0]) ) {
          c = c + 1;
        }
      }
      return this.ip.mkInt(c);
    }
    if ( name == "index" ) {
      this.need("index", args, 1, 3);
      const vals2 = this.ip.iterValues(__self);
      const n = vals2.length;
      let start = 0;
      let stop = n;
      if ( args.length > 1 ) {
        const tv1 = args[1];
        start = tv1.i;
        if ( start < 0 ) {
          start = start + n;
        }
        if ( start < 0 ) {
          start = 0;
        }
      }
      if ( args.length > 2 ) {
        const tv2 = args[2];
        stop = tv2.i;
        if ( stop < 0 ) {
          stop = stop + n;
        }
        if ( stop > n ) {
          stop = n;
        }
      }
      let k = start;
      while (k < stop) {
        if ( this.ip.eq(vals2[k], args[0]) ) {
          return this.ip.mkInt(k);
        }
        k = k + 1;
      };
      this.ip.raise("ValueError", (this.ip.typeName(__self) + ".index(x): x not in ") + this.ip.typeName(__self));
    }
    this.ip.raise("AttributeError", ((("'" + this.ip.typeName(__self)) + "' object has no attribute '") + name) + "'");
    return this.ip.NONE;
  };
  isWs (c) {
    return (((((((((((((c == 32 || c >= 9 && c <= 13) || c == 28) || c == 29) || c == 30) || c == 31) || c == 133) || c == 160) || c == 5760) || c >= 8192 && c <= 8202) || c == 8232) || c == 8233) || c == 8239) || c == 8287) || c == 12288;
  };
  stripCodes (s, chars, left, right) {
    const cs = s.codes();
    const n = cs.length;
    let cset = {};
    let useSet = false;
    if ( chars.t == 4 ) {
      useSet = true;
      const cc = chars.codes();
      // Loop start
      for ( const c of cc) {
        cset[((c).toString())] = true;
      }
    }
    let a = 0;
    let b = n;
    if ( left ) {
      while (a < b) {
        const c1 = cs[a];
        let strip = false;
        if ( useSet ) {
          strip = ( typeof(cset[((c1).toString())] ) != "undefined" && Object.prototype.hasOwnProperty.call(cset, ((c1).toString())) );
        } else {
          strip = this.isWs(c1);
        }
        if ( strip == false ) {
          break;
        }
        a = a + 1;
      };
    }
    if ( right ) {
      while (b > a) {
        const c2 = cs[(b - 1)];
        let strip2 = false;
        if ( useSet ) {
          strip2 = ( typeof(cset[((c2).toString())] ) != "undefined" && Object.prototype.hasOwnProperty.call(cset, ((c2).toString())) );
        } else {
          strip2 = this.isWs(c2);
        }
        if ( strip2 == false ) {
          break;
        }
        b = b - 1;
      };
    }
    return py_rt_codes(cs, a, b);
  };
  cpIndex (s, unitIdx) {
    if ( unitIdx <= 0 ) {
      return unitIdx;
    }
    return r_char_length(s.substring(0, unitIdx ));
  };
  findIn (__self, sub, args, fromRight) {
    const cs = __self.codes();
    const n = cs.length;
    let start = 0;
    let end = n;
    const __REGx1 = this.ip.argAt(args, 1);
    if ( args.length > 1 && __REGx1.t != 0 ) {
      const tv3 = args[1];
      start = tv3.i;
      if ( start < 0 ) {
        start = start + n;
      }
      if ( start < 0 ) {
        start = 0;
      }
    }
    const __REGx2 = this.ip.argAt(args, 2);
    if ( args.length > 2 && __REGx2.t != 0 ) {
      const tv4 = args[2];
      end = tv4.i;
      if ( end < 0 ) {
        end = end + n;
      }
      if ( end > n ) {
        end = n;
      }
    }
    if ( start > n ) {
      return -1;
    }
    if ( end < start ) {
      return -1;
    }
    const hay = py_rt_codes(cs, start, end);
    let u = -1;
    if ( fromRight ) {
      u = hay.lastIndexOf(sub);
      if ( sub.length == 0 ) {
        return end;
      }
    } else {
      u = hay.indexOf(sub);
    }
    if ( u < 0 ) {
      return -1;
    }
    return start + this.cpIndex(hay, u);
  };
  countIn (s, sub) {
    if ( sub.length == 0 ) {
      return r_char_length(s) + 1;
    }
    let c = 0;
    let from = 0;
    while (true) {
      const k = s.indexOf(sub, from);
      if ( k < 0 ) {
        break;
      }
      c = c + 1;
      from = k + sub.length;
    };
    return c;
  };
  splitWs (s, maxsplit) {
    const cs = s.codes();
    const n = cs.length;
    let out = [];
    let i = 0;
    while (i < n) {
      while (i < n && this.isWs(cs[i])) {
        i = i + 1;
      };
      if ( i >= n ) {
        break;
      }
      if ( maxsplit >= 0 && out.length >= maxsplit ) {
        let j0 = n;
        while (j0 > i && this.isWs(cs[(j0 - 1)])) {
          j0 = j0 - 1;
        };
        out.push(this.ip.mkStr(py_rt_codes(cs, i, j0)));
        return out;
      }
      let j = i;
      while (j < n && this.isWs(cs[j]) == false) {
        j = j + 1;
      };
      out.push(this.ip.mkStr(py_rt_codes(cs, i, j)));
      i = j;
    };
    return out;
  };
  splitSep (s, sep, maxsplit) {
    if ( sep.length == 0 ) {
      this.ip.raise("ValueError", "empty separator");
    }
    let out = [];
    let from = 0;
    while (true) {
      if ( maxsplit >= 0 && out.length >= maxsplit ) {
        break;
      }
      const k = s.indexOf(sep, from);
      if ( k < 0 ) {
        break;
      }
      out.push(this.ip.mkStr(s.substring(from, k )));
      from = k + sep.length;
    };
    out.push(this.ip.mkStr(s.substring(from, s.length )));
    return out;
  };
  charClass (__self, what) {
    const cs = __self.codes();
    if ( cs.length == 0 ) {
      return false;
    }
    let anyCased = false;
    // Loop start
    for ( const c of cs) {
      let isLower = c >= 97 && c <= 122;
      let isUpper = c >= 65 && c <= 90;
      const isDigit = c >= 48 && c <= 57;
      if ( c > 127 ) {
        const ch = String.fromCodePoint(c);
        const lo = ch.toLowerCase();
        const up = ch.toUpperCase();
        if ( lo == ch && up != ch ) {
          isLower = true;
        }
        if ( up == ch && lo != ch ) {
          isUpper = true;
        }
      }
      const isAlpha = (isLower || isUpper) || c > 127 && String.fromCodePoint(c)
        .toLowerCase() != String.fromCodePoint(c)
        .toUpperCase();
      if ( what == "digit" ) {
        if ( isDigit == false ) {
          return false;
        }
      }
      if ( what == "alpha" ) {
        if ( isAlpha == false ) {
          return false;
        }
      }
      if ( what == "alnum" ) {
        if ( isAlpha == false && isDigit == false ) {
          return false;
        }
      }
      if ( what == "space" ) {
        if ( this.isWs(c) == false ) {
          return false;
        }
      }
      if ( what == "ascii" ) {
        if ( c > 127 ) {
          return false;
        }
      }
      if ( what == "printable" ) {
        if ( c < 32 || c == 127 ) {
          return false;
        }
      }
      if ( what == "upper" ) {
        if ( isLower ) {
          return false;
        }
        if ( isUpper ) {
          anyCased = true;
        }
      }
      if ( what == "lower" ) {
        if ( isUpper ) {
          return false;
        }
        if ( isLower ) {
          anyCased = true;
        }
      }
    }
    if ( what == "upper" || what == "lower" ) {
      return anyCased;
    }
    return true;
  };
  titleCase (__self) {
    const cs = __self.codes();
    let out = "";
    let prevCased = false;
    // Loop start
    for ( const c of cs) {
      const ch = String.fromCodePoint(c);
      const cased = ch.toLowerCase() != ch.toUpperCase();
      if ( cased ) {
        if ( prevCased ) {
          out = out + ch.toLowerCase();
        } else {
          out = out + ch.toUpperCase();
        }
      } else {
        out = out + ch;
      }
      prevCased = cased;
    }
    return out;
  };
  padTo (s, width, fill, mode) {
    const n = s.strLen();
    if ( n >= width ) {
      return s.s;
    }
    const gap = width - n;
    if ( mode == "l" ) {
      return s.s + PyFormat.rep(fill, gap);
    }
    if ( mode == "r" ) {
      return PyFormat.rep(fill, gap) + s.s;
    }
    let left = ((gap / 2) | 0);
    if ( PyFormat.floorModInt(gap, 2) == 1 && PyFormat.floorModInt(width, 2) == 1 ) {
      left = left + 1;
    }
    return (PyFormat.rep(fill, left) + s.s) + PyFormat.rep(fill, (gap - left));
  };
  fillArg (args, i) {
    if ( i < args.length ) {
      const f = args[i];
      if ( f.t != 4 || f.strLen() != 1 ) {
        this.ip.raise("TypeError", "The fill character must be exactly one character long");
      }
      return f.s;
    }
    return " ";
  };
  makeTrans (args) {
    const d = this.ip.mkDict();
    const n = args.length;
    if ( n == 1 ) {
      const src = args[0];
      // Loop start
      for ( let i = 0; i < src.items.length; i++) {
        var k = src.items[i];
        let key = k;
        if ( k.t == 4 ) {
          const kc = k.codes();
          key = this.ip.mkInt(kc[0]);
        }
        this.ip.dictSet(d, key, src.vals[i]);
      }
      return d;
    }
    const a = args[0].codes();
    const b = args[1].codes();
    if ( a.length != b.length ) {
      this.ip.raise("ValueError", "the first two maketrans arguments must have equal length");
    }
    // Loop start
    for ( let j = 0; j < a.length; j++) {
      var c = a[j];
      this.ip.dictSet(
        d,
        this.ip.mkInt(c),
        this.ip.mkStr(String.fromCodePoint(b[j]))
      );
    }
    if ( n == 3 ) {
      const z = args[2].codes();
      // Loop start
      for ( const zc of z) {
        this.ip.dictSet(d, this.ip.mkInt(zc), this.ip.NONE);
      }
    }
    return d;
  };
  strMethod (__self, name, args, kn, kv) {
    const s = __self.s;
    const n = args.length;
    if ( name == "join" ) {
      this.need("join", args, 1, 1);
      const vals = this.ip.iterValues(args[0]);
      let parts = [];
      // Loop start
      for ( let i = 0; i < vals.length; i++) {
        var v = vals[i];
        if ( v.t != 4 ) {
          this.ip.raise("TypeError", ((("sequence item " + ((i).toString())) + ": expected str instance, ") + this.ip.typeName(v)) + " found");
        }
        parts.push(v.s);
      }
      return this.ip.mkStr(parts.join(s));
    }
    if ( name == "split" || name == "rsplit" ) {
      const sep = this.ip.bi.arg(args, 0, kn, kv, "sep");
      const ms = this.ip.bi.arg(args, 1, kn, kv, "maxsplit");
      let maxsplit = -1;
      if ( ms.t != 0 ) {
        maxsplit = ms.i;
      }
      if ( sep.t == 0 ) {
        if ( name == "rsplit" && maxsplit >= 0 ) {
          const all = this.splitWs(__self, -1);
          const na = all.length;
          if ( na <= maxsplit + 1 ) {
            return this.ip.mkList(all);
          }
          let tail = [];
          let k = na - maxsplit;
          while (k < na) {
            tail.push(all[k]);
            k = k + 1;
          };
          const cs = __self.codes();
          let endc = cs.length;
          let found = 0;
          while (endc > 0 && found < maxsplit) {
            while (endc > 0 && this.isWs(cs[(endc - 1)])) {
              endc = endc - 1;
            };
            while (endc > 0 && this.isWs(cs[(endc - 1)]) == false) {
              endc = endc - 1;
            };
            found = found + 1;
          };
          const headStr = this.ip.mkStr(py_rt_codes(cs, 0, endc));
          let out0 = [];
          out0.push(this.ip.mkStr(this.stripCodes(
            headStr,
            this.ip.NONE,
            true,
            true
          )));
          // Loop start
          for ( const tv of tail) {
            out0.push(tv);
          }
          return this.ip.mkList(out0);
        }
        return this.ip.mkList(this.splitWs(__self, maxsplit));
      }
      const sp = this.strArg(sep, "sep");
      if ( name == "rsplit" && maxsplit >= 0 ) {
        const all2 = this.splitSep(s, sp, -1);
        const na2 = all2.length;
        if ( na2 <= maxsplit + 1 ) {
          return this.ip.mkList(all2);
        }
        let headParts = [];
        let h = 0;
        while (h < na2 - maxsplit) {
          const hv = all2[h];
          headParts.push(hv.s);
          h = h + 1;
        };
        let out1 = [];
        out1.push(this.ip.mkStr(headParts.join(sp)));
        while (h < na2) {
          out1.push(all2[h]);
          h = h + 1;
        };
        return this.ip.mkList(out1);
      }
      return this.ip.mkList(this.splitSep(s, sp, maxsplit));
    }
    if ( name == "splitlines" ) {
      let keep = false;
      const kp = this.ip.bi.arg(args, 0, kn, kv, "keepends");
      if ( kp.t != 0 ) {
        keep = this.ip.truthy(kp);
      }
      const cs2 = __self.codes();
      const nc = cs2.length;
      let out2 = [];
      let st = 0;
      let i2 = 0;
      while (i2 < nc) {
        const c = cs2[i2];
        if ( ((((((((c == 10 || c == 13) || c == 11) || c == 12) || c == 28) || c == 29) || c == 30) || c == 133) || c == 8232) || c == 8233 ) {
          let endl = i2 + 1;
          if ( (c == 13 && i2 + 1 < nc) && cs2[(i2 + 1)] == 10 ) {
            endl = i2 + 2;
          }
          if ( keep ) {
            out2.push(this.ip.mkStr(py_rt_codes(cs2, st, endl)));
          } else {
            out2.push(this.ip.mkStr(py_rt_codes(cs2, st, i2)));
          }
          st = endl;
          i2 = endl;
          continue;
        }
        i2 = i2 + 1;
      };
      if ( st < nc ) {
        out2.push(this.ip.mkStr(py_rt_codes(cs2, st, nc)));
      }
      return this.ip.mkList(out2);
    }
    if ( name == "strip" ) {
      return this.ip.mkStr(this.stripCodes(
        __self,
        this.arg(args, 0),
        true,
        true
      ));
    }
    if ( name == "lstrip" ) {
      return this.ip.mkStr(this.stripCodes(
        __self,
        this.arg(args, 0),
        true,
        false
      ));
    }
    if ( name == "rstrip" ) {
      return this.ip.mkStr(this.stripCodes(
        __self,
        this.arg(args, 0),
        false,
        true
      ));
    }
    if ( name == "replace" ) {
      this.need("replace", args, 2, 3);
      const old = this.strArg(args[0], "replace() argument 1");
      const nw = this.strArg(args[1], "replace() argument 2");
      let count = -1;
      if ( n == 3 ) {
        const tv5 = args[2];
        count = tv5.i;
      }
      if ( old.length == 0 ) {
        const cs3 = __self.codes();
        let parts3 = [];
        parts3.push("");
        // Loop start
        for ( const c3 of cs3) {
          parts3.push(String.fromCodePoint(c3));
        }
        parts3.push("");
        return this.ip.mkStr(parts3.join(nw));
      }
      if ( count < 0 ) {
        return this.ip.mkStr(this.replaceAll(s, old, nw));
      }
      let out3 = "";
      let from = 0;
      let done = 0;
      while (done < count) {
        const k3 = s.indexOf(old, from);
        if ( k3 < 0 ) {
          break;
        }
        out3 = (out3 + s.substring(from, k3 )) + nw;
        from = k3 + old.length;
        done = done + 1;
      };
      return this.ip.mkStr((out3 + s.substring(from, s.length )));
    }
    if ( name == "upper" ) {
      return this.ip.mkStr(s.toUpperCase());
    }
    if ( name == "lower" || name == "casefold" ) {
      return this.ip.mkStr(s.toLowerCase());
    }
    if ( name == "title" ) {
      return this.ip.mkStr(this.titleCase(__self));
    }
    if ( name == "capitalize" ) {
      const cs4 = __self.codes();
      if ( cs4.length == 0 ) {
        return __self;
      }
      return this.ip.mkStr((String.fromCodePoint(cs4[0]).toUpperCase() + py_rt_codes(
        cs4,
        1,
        cs4.length
      ).toLowerCase()));
    }
    if ( name == "swapcase" ) {
      const cs5 = __self.codes();
      let out5 = "";
      // Loop start
      for ( const c5 of cs5) {
        const ch = String.fromCodePoint(c5);
        const up = ch.toUpperCase();
        if ( up == ch ) {
          out5 = out5 + ch.toLowerCase();
        } else {
          out5 = out5 + up;
        }
      }
      return this.ip.mkStr(out5);
    }
    if ( name == "startswith" || name == "endswith" ) {
      this.need(name, args, 1, 3);
      let target = __self;
      if ( n > 1 ) {
        const sl = new PyVal(18);
        sl.items.push(args[1]);
        sl.items.push(this.arg(args, 2));
        sl.items.push(this.ip.NONE);
        target = this.ip.getItem(__self, sl);
      }
      const pre = args[0];
      let opts = [];
      if ( pre.t == 6 ) {
        opts = pre.items;
      } else {
        opts.push(pre);
      }
      // Loop start
      for ( const o of opts) {
        const os = this.strArg(o, (name + " first arg"));
        if ( name == "startswith" ) {
          if ( target.s.startsWith(os) ) {
            return this.ip.TRUE;
          }
        } else {
          if ( target.s.endsWith(os) ) {
            return this.ip.TRUE;
          }
        }
      }
      return this.ip.FALSE;
    }
    if ( name == "find" || name == "index" ) {
      this.need(name, args, 1, 3);
      const r = this.findIn(
        __self,
        this.strArg(args[0], "must be str"),
        args,
        false
      );
      if ( r < 0 && name == "index" ) {
        this.ip.raise("ValueError", "substring not found");
      }
      return this.ip.mkInt(r);
    }
    if ( name == "rfind" || name == "rindex" ) {
      this.need(name, args, 1, 3);
      const r2 = this.findIn(
        __self,
        this.strArg(args[0], "must be str"),
        args,
        true
      );
      if ( r2 < 0 && name == "rindex" ) {
        this.ip.raise("ValueError", "substring not found");
      }
      return this.ip.mkInt(r2);
    }
    if ( name == "count" ) {
      this.need("count", args, 1, 3);
      let tgt = __self;
      if ( n > 1 ) {
        const sl2 = new PyVal(18);
        sl2.items.push(args[1]);
        sl2.items.push(this.arg(args, 2));
        sl2.items.push(this.ip.NONE);
        tgt = this.ip.getItem(__self, sl2);
      }
      return this.ip.mkInt(this.countIn(tgt.s, this.strArg(args[0], "must be str")));
    }
    if ( name == "format" ) {
      return this.ip.mkStr(this.strFormat(s, args, kn, kv));
    }
    if ( name == "format_map" ) {
      let mk = [];
      let mv = [];
      const m = args[0];
      // Loop start
      for ( let mii = 0; mii < m.items.length; mii++) {
        var mki = m.items[mii];
        mk.push(this.ip.toStr(mki));
        mv.push(m.vals[mii]);
      }
      return this.ip.mkStr(this.strFormat(s, [], mk, mv));
    }
    if ( name == "zfill" ) {
      this.need("zfill", args, 1, 1);
      const w = args[0].i;
      const cs6 = __self.codes();
      const __len = cs6.length;
      if ( __len >= w ) {
        return __self;
      }
      let sign = "";
      let body = s;
      if ( __len > 0 && (cs6[0] == 45 || cs6[0] == 43) ) {
        sign = String.fromCodePoint(cs6[0]);
        body = py_rt_codes(cs6, 1, __len);
      }
      return this.ip.mkStr(((sign + PyFormat.zeros((w - __len))) + body));
    }
    if ( name == "ljust" ) {
      this.need("ljust", args, 1, 2);
      return this.ip.mkStr(this.padTo(
        __self,
        args[0].i,
        this.fillArg(args, 1),
        "l"
      ));
    }
    if ( name == "rjust" ) {
      this.need("rjust", args, 1, 2);
      return this.ip.mkStr(this.padTo(
        __self,
        args[0].i,
        this.fillArg(args, 1),
        "r"
      ));
    }
    if ( name == "center" ) {
      this.need("center", args, 1, 2);
      return this.ip.mkStr(this.padTo(
        __self,
        args[0].i,
        this.fillArg(args, 1),
        "c"
      ));
    }
    if ( (name == "isdigit" || name == "isnumeric") || name == "isdecimal" ) {
      return this.ip.mkBool(this.charClass(__self, "digit"));
    }
    if ( name == "isalpha" ) {
      return this.ip.mkBool(this.charClass(__self, "alpha"));
    }
    if ( name == "isalnum" ) {
      return this.ip.mkBool(this.charClass(__self, "alnum"));
    }
    if ( name == "isspace" ) {
      return this.ip.mkBool(this.charClass(__self, "space"));
    }
    if ( name == "isupper" ) {
      return this.ip.mkBool(this.charClass(__self, "upper"));
    }
    if ( name == "islower" ) {
      return this.ip.mkBool(this.charClass(__self, "lower"));
    }
    if ( name == "isascii" ) {
      if ( s.length == 0 ) {
        return this.ip.TRUE;
      }
      return this.ip.mkBool(this.charClass(__self, "ascii"));
    }
    if ( name == "isprintable" ) {
      if ( s.length == 0 ) {
        return this.ip.TRUE;
      }
      return this.ip.mkBool(this.charClass(__self, "printable"));
    }
    if ( name == "istitle" ) {
      return this.ip.mkBool((this.titleCase(__self) == s && s.toLowerCase() != s.toUpperCase()));
    }
    if ( name == "isidentifier" ) {
      const cs7 = __self.codes();
      if ( cs7.length == 0 ) {
        return this.ip.FALSE;
      }
      // Loop start
      for ( let i7 = 0; i7 < cs7.length; i7++) {
        var c7 = cs7[i7];
        if ( i7 == 0 ) {
          if ( PyLexer.isIdStart(c7) == false ) {
            return this.ip.FALSE;
          }
        } else {
          if ( PyLexer.isIdChar(c7) == false ) {
            return this.ip.FALSE;
          }
        }
      }
      return this.ip.TRUE;
    }
    if ( name == "partition" || name == "rpartition" ) {
      this.need(name, args, 1, 1);
      const sep2 = this.strArg(args[0], "sep");
      if ( sep2.length == 0 ) {
        this.ip.raise("ValueError", "empty separator");
      }
      let at = -1;
      if ( name == "partition" ) {
        at = s.indexOf(sep2);
      } else {
        at = s.lastIndexOf(sep2);
      }
      let items = [];
      if ( at < 0 ) {
        if ( name == "partition" ) {
          items.push(__self);
          items.push(this.ip.mkStr(""));
          items.push(this.ip.mkStr(""));
        } else {
          items.push(this.ip.mkStr(""));
          items.push(this.ip.mkStr(""));
          items.push(__self);
        }
        return this.ip.mkTuple(items);
      }
      items.push(this.ip.mkStr(s.substring(0, at )));
      items.push(this.ip.mkStr(sep2));
      items.push(this.ip.mkStr(s.substring(at + sep2.length, s.length )));
      return this.ip.mkTuple(items);
    }
    if ( name == "encode" ) {
      return __self;
    }
    if ( name == "expandtabs" ) {
      let tab = 8;
      if ( n > 0 ) {
        const tv6 = args[0];
        tab = tv6.i;
      }
      const cs8 = __self.codes();
      let out8 = "";
      let col = 0;
      // Loop start
      for ( const c8 of cs8) {
        if ( c8 == 9 ) {
          let sp8 = tab - PyFormat.floorModInt(col, tab);
          if ( tab == 0 ) {
            sp8 = 0;
          }
          out8 = out8 + PyFormat.rep(" ", sp8);
          col = col + sp8;
        } else {
          out8 = out8 + String.fromCodePoint(c8);
          col = col + 1;
          if ( c8 == 10 || c8 == 13 ) {
            col = 0;
          }
        }
      }
      return this.ip.mkStr(out8);
    }
    if ( name == "removeprefix" ) {
      const pfx = this.strArg(args[0], "prefix");
      if ( pfx.length > 0 && s.startsWith(pfx) ) {
        return this.ip.mkStr(s.substring(pfx.length, s.length ));
      }
      return __self;
    }
    if ( name == "removesuffix" ) {
      const sfx = this.strArg(args[0], "suffix");
      if ( sfx.length > 0 && s.endsWith(sfx) ) {
        return this.ip.mkStr(s.substring(0, s.length - sfx.length ));
      }
      return __self;
    }
    if ( name == "translate" ) {
      const table2 = args[0];
      const cs9 = __self.codes();
      let out9 = "";
      // Loop start
      for ( const c9 of cs9) {
        const got = this.ip.dictGet(table2, this.ip.mkInt(c9));
        if ( typeof(got) === "undefined" ) {
          out9 = out9 + String.fromCodePoint(c9);
        } else {
          const gv = got;
          if ( gv.t == 4 ) {
            out9 = out9 + gv.s;
          } else {
            if ( this.ip.isIntLike(gv) ) {
              out9 = out9 + String.fromCodePoint(gv.i);
            }
          }
        }
      }
      return this.ip.mkStr(out9);
    }
    if ( name == "__mul__" ) {
      return this.ip.binop("*", __self, args[0]);
    }
    this.ip.raise("AttributeError", ("'str' object has no attribute '" + name) + "'");
    return this.ip.NONE;
  };
  replaceAll (s, old, nw) {
    let parts = [];
    let from = 0;
    while (true) {
      const k = s.indexOf(old, from);
      if ( k < 0 ) {
        break;
      }
      parts.push(s.substring(from, k ));
      from = k + old.length;
    };
    parts.push(s.substring(from, s.length ));
    return parts.join(nw);
  };
  strFormat (fmt, args, kn, kv) {
    const cs = Array.from(fmt, (rg_c) => rg_c.codePointAt(0));
    const n = cs.length;
    let out = [];
    let lit = [];
    let auto = 0;
    let i = 0;
    while (i < n) {
      const c = cs[i];
      if ( c == 123 ) {
        if ( i + 1 < n && cs[(i + 1)] == 123 ) {
          lit.push(123);
          i = i + 2;
          continue;
        }
        if ( lit.length > 0 ) {
          out.push(py_rt_codes(lit, 0, lit.length));
          lit.length = 0;
        }
        let j = i + 1;
        let depth = 1;
        while (j < n) {
          if ( cs[j] == 123 ) {
            depth = depth + 1;
          }
          if ( cs[j] == 125 ) {
            depth = depth - 1;
            if ( depth == 0 ) {
              break;
            }
          }
          j = j + 1;
        };
        if ( j >= n ) {
          this.ip.raise("ValueError", "Single '{' encountered in format string");
        }
        const field = py_rt_codes(cs, i + 1, j);
        i = j + 1;
        const fcs = Array.from(field, (rg_c) => rg_c.codePointAt(0));
        const fn2 = fcs.length;
        let k = 0;
        let bdepth = 0;
        while (k < fn2) {
          const fc = fcs[k];
          if ( fc == 91 ) {
            bdepth = bdepth + 1;
          }
          if ( fc == 93 ) {
            bdepth = bdepth - 1;
          }
          if ( bdepth == 0 && (fc == 33 || fc == 58) ) {
            break;
          }
          k = k + 1;
        };
        const fname = py_rt_codes(fcs, 0, k);
        let conv = "";
        let spec = "";
        if ( k < fn2 && fcs[k] == 33 ) {
          conv = String.fromCodePoint(fcs[(k + 1)]);
          k = k + 2;
        }
        if ( k < fn2 && fcs[k] == 58 ) {
          spec = py_rt_codes(fcs, k + 1, fn2);
          if ( spec.indexOf("{") >= 0 ) {
            spec = this.strFormat(spec, args, kn, kv);
          }
        }
        const nc = Array.from(fname, (rg_c) => rg_c.codePointAt(0));
        const nn = nc.length;
        let e = 0;
        while ((e < nn && nc[e] != 46) && nc[e] != 91) {
          e = e + 1;
        };
        const head = py_rt_codes(nc, 0, e);
        let v = this.ip.NONE;
        if ( head.length == 0 ) {
          if ( auto >= args.length ) {
            this.ip.raise("IndexError", ("Replacement index " + ((auto).toString())) + " out of range for positional args tuple");
          }
          v = args[auto];
          auto = auto + 1;
        } else {
          const num = isNaN( parseInt(head) ) ? undefined : parseInt(head);
          if ( ((typeof(num) !== "undefined" && num != null ) ) && PyLexer.isDigit(nc[0]) ) {
            const ni = num;
            if ( ni >= args.length ) {
              this.ip.raise("IndexError", ("Replacement index " + ((ni).toString())) + " out of range for positional args tuple");
            }
            v = args[ni];
          } else {
            let found = false;
            // Loop start
            for ( let kni = 0; kni < kn.length; kni++) {
              var kname = kn[kni];
              if ( kname == head ) {
                v = kv[kni];
                found = true;
              }
            }
            if ( found == false ) {
              this.ip.raiseVal(this.ip.makeExcArg("KeyError", this.ip.mkStr(head)));
            }
          }
        }
        while (e < nn) {
          if ( nc[e] == 46 ) {
            const a = e + 1;
            let b = a;
            while ((b < nn && nc[b] != 46) && nc[b] != 91) {
              b = b + 1;
            };
            v = this.ip.getAttr(v, py_rt_codes(nc, a, b));
            e = b;
            continue;
          }
          if ( nc[e] == 91 ) {
            const a2 = e + 1;
            let b2 = a2;
            while (b2 < nn && nc[b2] != 93) {
              b2 = b2 + 1;
            };
            const key = py_rt_codes(nc, a2, b2);
            const kiv = isNaN( parseInt(key) ) ? undefined : parseInt(key);
            if ( (((typeof(kiv) !== "undefined" && kiv != null ) ) && key.length > 0) && PyLexer.isDigit(nc[a2]) ) {
              v = this.ip.getItem(v, this.ip.mkInt(kiv));
            } else {
              v = this.ip.getItem(v, this.ip.mkStr(key));
            }
            e = b2 + 1;
            continue;
          }
          e = e + 1;
        };
        if ( conv == "r" ) {
          v = this.ip.mkStr(this.ip.toRepr(v));
        }
        if ( conv == "s" ) {
          v = this.ip.mkStr(this.ip.toStr(v));
        }
        if ( conv == "a" ) {
          v = this.ip.mkStr(this.ip.bi.asciiOf(this.ip.toRepr(v)));
        }
        out.push(this.ip.formatValue(v, spec));
        continue;
      }
      if ( c == 125 ) {
        if ( i + 1 < n && cs[(i + 1)] == 125 ) {
          lit.push(125);
          i = i + 2;
          continue;
        }
        this.ip.raise("ValueError", "Single '}' encountered in format string");
      }
      lit.push(c);
      i = i + 1;
    };
    if ( lit.length > 0 ) {
      out.push(py_rt_codes(lit, 0, lit.length));
    }
    return out.join("");
  };
  listMethod (__self, name, args, kn, kv) {
    const n = args.length;
    if ( name == "append" ) {
      this.need("append", args, 1, 1);
      __self.items.push(args[0]);
      this.ip.mods.dequeTrim(__self, false);
      return this.ip.NONE;
    }
    if ( name == "appendleft" ) {
      this.need("appendleft", args, 1, 1);
      let out = [];
      out.push(args[0]);
      // Loop start
      for ( const it of __self.items) {
        out.push(it);
      }
      __self.items = out;
      this.ip.mods.dequeTrim(__self, true);
      return this.ip.NONE;
    }
    if ( name == "extend" ) {
      this.need("extend", args, 1, 1);
      const vals = this.ip.iterValues(args[0]);
      // Loop start
      for ( const v of vals) {
        __self.items.push(v);
      }
      this.ip.mods.dequeTrim(__self, false);
      return this.ip.NONE;
    }
    if ( name == "extendleft" ) {
      const vals0 = this.ip.iterValues(args[0]);
      // Loop start
      for ( const v0 of vals0) {
        this.listMethod(__self, "appendleft", [v0], kn, kv);
      }
      return this.ip.NONE;
    }
    if ( name == "insert" ) {
      this.need("insert", args, 2, 2);
      const __len = __self.items.length;
      let at = args[0].i;
      if ( at < 0 ) {
        at = at + __len;
        if ( at < 0 ) {
          at = 0;
        }
      }
      if ( at > __len ) {
        at = __len;
      }
      let out2 = [];
      let k = 0;
      while (k < __len) {
        if ( k == at ) {
          out2.push(args[1]);
        }
        out2.push(__self.items[k]);
        k = k + 1;
      };
      if ( at == __len ) {
        out2.push(args[1]);
      }
      __self.items = out2;
      return this.ip.NONE;
    }
    if ( name == "pop" || name == "popleft" ) {
      const len2 = __self.items.length;
      if ( len2 == 0 ) {
        if ( __self.mode == "deque" ) {
          this.ip.raise("IndexError", "pop from an empty deque");
        }
        this.ip.raise("IndexError", "pop from empty list");
      }
      let at2 = len2 - 1;
      if ( name == "popleft" ) {
        at2 = 0;
      }
      if ( n > 0 ) {
        at2 = this.ip.normIndex(args[0].i, len2, "pop");
      }
      const v2 = __self.items[at2];
      __self.items.splice(at2, 1).pop();
      return v2;
    }
    if ( name == "remove" ) {
      this.need("remove", args, 1, 1);
      let k2 = 0;
      const len3 = __self.items.length;
      while (k2 < len3) {
        if ( this.ip.eq(__self.items[k2], args[0]) ) {
          __self.items.splice(k2, 1).pop();
          return this.ip.NONE;
        }
        k2 = k2 + 1;
      };
      this.ip.raise("ValueError", "list.remove(x): x not in list");
    }
    if ( name == "sort" ) {
      let keyf = this.ip.NONE;
      const kf = this.ip.bi.kw(kn, kv, "key");
      if ( (typeof(kf) !== "undefined" && kf != null )  ) {
        keyf = kf;
      }
      let rev = false;
      const rv = this.ip.bi.kw(kn, kv, "reverse");
      if ( (typeof(rv) !== "undefined" && rv != null )  ) {
        rev = this.ip.truthy(rv);
      }
      __self.items = this.ip.bi.sortVals(__self.items, keyf, rev);
      return this.ip.NONE;
    }
    if ( name == "reverse" ) {
      let out3 = [];
      let k3 = __self.items.length - 1;
      while (k3 >= 0) {
        out3.push(__self.items[k3]);
        k3 = k3 - 1;
      };
      __self.items = out3;
      return this.ip.NONE;
    }
    if ( name == "rotate" ) {
      let r = 1;
      if ( n > 0 ) {
        const tv7 = args[0];
        r = tv7.i;
      }
      const len4 = __self.items.length;
      if ( len4 == 0 ) {
        return this.ip.NONE;
      }
      r = PyFormat.floorModInt(r, len4);
      let out4 = [];
      let k4 = 0;
      while (k4 < len4) {
        out4.push(__self.items[PyFormat.floorModInt((k4 - r), len4)]);
        k4 = k4 + 1;
      };
      __self.items = out4;
      return this.ip.NONE;
    }
    if ( name == "copy" ) {
      let out5 = [];
      // Loop start
      for ( const it5 of __self.items) {
        out5.push(it5);
      }
      const c = this.ip.mkList(out5);
      c.mode = __self.mode;
      return c;
    }
    if ( name == "clear" ) {
      let e = [];
      __self.items = e;
      return this.ip.NONE;
    }
    return this.seqCommon(__self, name, args);
  };
  dictMethod (__self, name, args, kn, kv) {
    const r = this.dictMethodPlain(__self, name, args, kn, kv);
    if ( (typeof(__self.owner) !== "undefined" && __self.owner != null )  ) {
      __self.ownerSync();
    }
    return r;
  };
  dictMethodPlain (__self, name, args, kn, kv) {
    const n = args.length;
    if ( name == "move_to_end" ) {
      const mk = this.arg(args, 0);
      const mv = this.ip.dictGet(__self, mk);
      if ( typeof(mv) === "undefined" ) {
        this.ip.raiseVal(this.ip.makeExcArg("KeyError", mk));
      }
      let atEnd = true;
      const lv = this.ip.bi.arg(args, 1, kn, kv, "last");
      if ( lv.t != 0 ) {
        atEnd = this.ip.truthy(lv);
      }
      this.ip.dictDel(__self, mk);
      if ( atEnd ) {
        this.ip.dictSet(__self, mk, mv);
        return this.ip.NONE;
      }
      const ks = __self.items;
      const vs = __self.vals;
      let none = [];
      let none2 = [];
      let ni = {};
      __self.items = none;
      __self.vals = none2;
      __self.index = ni;
      this.ip.dictSet(__self, mk, mv);
      // Loop start
      for ( let ki = 0; ki < ks.length; ki++) {
        var k = ks[ki];
        this.ip.dictSet(__self, k, vs[ki]);
      }
      return this.ip.NONE;
    }
    if ( name == "get" ) {
      this.need("get", args, 1, 2);
      const r = this.ip.dictGet(__self, args[0]);
      if ( typeof(r) === "undefined" ) {
        return this.arg(args, 1);
      }
      return r;
    }
    if ( name == "keys" ) {
      let ks_1 = [];
      // Loop start
      for ( const k_1 of __self.items) {
        ks_1.push(k_1);
      }
      const l = this.ip.mkList(ks_1);
      l.mode = "dict_keys";
      return l;
    }
    if ( name == "values" ) {
      let vs_1 = [];
      // Loop start
      for ( const v of __self.vals) {
        vs_1.push(v);
      }
      const l2 = this.ip.mkList(vs_1);
      l2.mode = "dict_values";
      return l2;
    }
    if ( name == "items" ) {
      let its = [];
      // Loop start
      for ( let i3 = 0; i3 < __self.items.length; i3++) {
        var k3 = __self.items[i3];
        its.push(this.ip.pair(k3, __self.vals[i3]));
      }
      const l3 = this.ip.mkList(its);
      l3.mode = "dict_items";
      return l3;
    }
    if ( name == "pop" ) {
      this.need("pop", args, 1, 2);
      const r2 = this.ip.dictGet(__self, args[0]);
      if ( typeof(r2) === "undefined" ) {
        if ( n == 2 ) {
          return args[1];
        }
        this.ip.raiseVal(this.ip.makeExcArg("KeyError", args[0]));
      }
      this.ip.dictDel(__self, args[0]);
      return r2;
    }
    if ( name == "popitem" ) {
      const __len = __self.items.length;
      if ( __len == 0 ) {
        this.ip.raise("KeyError", "popitem(): dictionary is empty");
      }
      const k4 = __self.items[(__len - 1)];
      const v4 = __self.vals[(__len - 1)];
      this.ip.dictDel(__self, k4);
      return this.ip.pair(k4, v4);
    }
    if ( name == "setdefault" ) {
      this.need("setdefault", args, 1, 2);
      const r3 = this.ip.dictGet(__self, args[0]);
      if ( typeof(r3) === "undefined" ) {
        const dv = this.arg(args, 1);
        this.ip.dictSet(__self, args[0], dv);
        return dv;
      }
      return r3;
    }
    if ( name == "update" ) {
      if ( n > 0 ) {
        if ( __self.mode == "counter" ) {
          this.counterAdd(__self, args[0], 1);
        } else {
          this.ip.bi.fillDict(__self, args[0]);
        }
      }
      // Loop start
      for ( let ki_1 = 0; ki_1 < kn.length; ki_1++) {
        var key = kn[ki_1];
        this.ip.dictSet(__self, this.ip.mkStr(key), kv[ki_1]);
      }
      return this.ip.NONE;
    }
    if ( name == "subtract" ) {
      if ( n > 0 ) {
        this.counterAdd(__self, args[0], -1);
      }
      return this.ip.NONE;
    }
    if ( name == "copy" ) {
      const c = this.ip.copyDict(__self);
      c.mode = __self.mode;
      const ak = __self.attrKeys();
      // Loop start
      for ( const a of ak) {
        c.putAttr(a, ( Object.prototype.hasOwnProperty.call(__self.attrs, a) ? __self.attrs[a] : undefined ));
      }
      return c;
    }
    if ( name == "clear" ) {
      let e1 = [];
      let e2 = [];
      let e3 = {};
      __self.items = e1;
      __self.vals = e2;
      __self.index = e3;
      return this.ip.NONE;
    }
    if ( name == "most_common" ) {
      let pairs = [];
      // Loop start
      for ( let i5 = 0; i5 < __self.items.length; i5++) {
        var k5 = __self.items[i5];
        pairs.push(this.ip.pair(k5, __self.vals[i5]));
      }
      const keyf = this.ip.mkBuiltinMethod(this.ip.NONE, "__counter_key__");
      const sorted = this.ip.bi.sortVals(pairs, keyf, true);
      const __REGx1 = this.ip.argAt(args, 0);
      if ( n > 0 && __REGx1.t != 0 ) {
        const lim = args[0].i;
        let cut = [];
        // Loop start
        for ( let si = 0; si < sorted.length; si++) {
          var sv = sorted[si];
          if ( si < lim ) {
            cut.push(sv);
          }
        }
        return this.ip.mkList(cut);
      }
      return this.ip.mkList(sorted);
    }
    if ( name == "elements" ) {
      let els = [];
      // Loop start
      for ( let i6 = 0; i6 < __self.items.length; i6++) {
        var k6 = __self.items[i6];
        const cnt = __self.vals[i6].i;
        let j = 0;
        while (j < cnt) {
          els.push(k6);
          j = j + 1;
        };
      }
      return this.ip.mkIter(els);
    }
    if ( name == "total" ) {
      let kn2 = [];
      let kv2 = [];
      return this.ip.bi.call("sum", [this.ip.mkList(__self.vals)], kn2, kv2);
    }
    if ( name == "__iter__" ) {
      return this.ip.mkIter(this.ip.iterValues(__self));
    }
    this.ip.raise("AttributeError", ("'dict' object has no attribute '" + name) + "'");
    return this.ip.NONE;
  };
  counterAdd (__self, src, sign) {
    if ( src.t == 7 ) {
      // Loop start
      for ( let i = 0; i < src.items.length; i++) {
        var k = src.items[i];
        const cur = this.ip.dictGet(__self, k);
        let base = 0;
        if ( (typeof(cur) !== "undefined" && cur != null )  ) {
          const tv8 = cur;
          base = tv8.i;
        }
        this.ip.dictSet(
          __self,
          k,
          this.ip.mkInt((base + sign * src.vals[i].i))
        );
      }
      return;
    }
    const vals = this.ip.iterValues(src);
    // Loop start
    for ( const v of vals) {
      const cur2 = this.ip.dictGet(__self, v);
      let base2 = 0;
      if ( (typeof(cur2) !== "undefined" && cur2 != null )  ) {
        const tv9 = cur2;
        base2 = tv9.i;
      }
      this.ip.dictSet(__self, v, this.ip.mkInt((base2 + sign)));
    }
  };
  setOp (op, a, b) {
    const r = this.ip.mkSet();
    r.mode = a.mode;
    if ( op == "|" ) {
      // Loop start
      for ( const x of a.items) {
        this.ip.setAdd(r, x);
      }
      // Loop start
      for ( const y of b.items) {
        this.ip.setAdd(r, y);
      }
      return r;
    }
    if ( op == "&" ) {
      // Loop start
      for ( const x2 of a.items) {
        if ( ( typeof(b.index[this.ip.hashKey(x2)] ) != "undefined" && Object.prototype.hasOwnProperty.call(b.index, this.ip.hashKey(x2)) ) ) {
          this.ip.setAdd(r, x2);
        }
      }
      return r;
    }
    if ( op == "-" ) {
      // Loop start
      for ( const x3 of a.items) {
        if ( ( typeof(b.index[this.ip.hashKey(x3)] ) != "undefined" && Object.prototype.hasOwnProperty.call(b.index, this.ip.hashKey(x3)) ) == false ) {
          this.ip.setAdd(r, x3);
        }
      }
      return r;
    }
    // Loop start
    for ( const x4 of a.items) {
      if ( ( typeof(b.index[this.ip.hashKey(x4)] ) != "undefined" && Object.prototype.hasOwnProperty.call(b.index, this.ip.hashKey(x4)) ) == false ) {
        this.ip.setAdd(r, x4);
      }
    }
    // Loop start
    for ( const y4 of b.items) {
      if ( ( typeof(a.index[this.ip.hashKey(y4)] ) != "undefined" && Object.prototype.hasOwnProperty.call(a.index, this.ip.hashKey(y4)) ) == false ) {
        this.ip.setAdd(r, y4);
      }
    }
    return r;
  };
  asSet (v) {
    if ( v.t == 8 ) {
      return v;
    }
    const s = this.ip.mkSet();
    const vals = this.ip.iterValues(v);
    // Loop start
    for ( const x of vals) {
      this.ip.setAdd(s, x);
    }
    return s;
  };
  setMethod (__self, name, args) {
    const n = args.length;
    const frozen = __self.mode == "frozen";
    if ( name == "add" && frozen == false ) {
      this.need("add", args, 1, 1);
      this.ip.setAdd(__self, args[0]);
      return this.ip.NONE;
    }
    if ( (name == "remove" || name == "discard") && frozen == false ) {
      this.need(name, args, 1, 1);
      const gone = this.ip.dictDel(__self, args[0]);
      if ( gone == false && name == "remove" ) {
        this.ip.raiseVal(this.ip.makeExcArg("KeyError", args[0]));
      }
      return this.ip.NONE;
    }
    if ( name == "pop" && frozen == false ) {
      if ( __self.items.length == 0 ) {
        this.ip.raise("KeyError", "pop from an empty set");
      }
      const v = __self.items[0];
      this.ip.dictDel(__self, v);
      return v;
    }
    if ( name == "clear" && frozen == false ) {
      let e1 = [];
      let e2 = {};
      __self.items = e1;
      __self.index = e2;
      return this.ip.NONE;
    }
    if ( name == "copy" ) {
      const c = this.ip.copySet(__self);
      c.mode = __self.mode;
      return c;
    }
    if ( name == "update" && frozen == false ) {
      // Loop start
      for ( const a of args) {
        const vals = this.ip.iterValues(a);
        // Loop start
        for ( const x of vals) {
          this.ip.setAdd(__self, x);
        }
      }
      return this.ip.NONE;
    }
    if ( ((name == "union" || name == "intersection") || name == "difference") || name == "symmetric_difference" ) {
      let op = "|";
      if ( name == "intersection" ) {
        op = "&";
      }
      if ( name == "difference" ) {
        op = "-";
      }
      if ( name == "symmetric_difference" ) {
        op = "^";
      }
      let r = __self;
      if ( n == 0 ) {
        r = this.ip.copySet(__self);
        r.mode = __self.mode;
      }
      // Loop start
      for ( const a2 of args) {
        r = this.setOp(op, r, this.asSet(a2));
      }
      return r;
    }
    if ( ((name == "intersection_update" || name == "difference_update") || name == "symmetric_difference_update") && frozen == false ) {
      let op2 = "&";
      if ( name == "difference_update" ) {
        op2 = "-";
      }
      if ( name == "symmetric_difference_update" ) {
        op2 = "^";
      }
      let r2 = __self;
      // Loop start
      for ( const a3 of args) {
        r2 = this.setOp(op2, r2, this.asSet(a3));
      }
      __self.items = r2.items;
      __self.index = r2.index;
      return this.ip.NONE;
    }
    if ( name == "issubset" ) {
      return this.ip.mkBool(this.ip.isSubset(__self, this.asSet(args[0])));
    }
    if ( name == "issuperset" ) {
      return this.ip.mkBool(this.ip.isSubset(this.asSet(args[0]), __self));
    }
    if ( name == "isdisjoint" ) {
      const o = this.asSet(args[0]);
      // Loop start
      for ( const x5 of __self.items) {
        if ( ( typeof(o.index[this.ip.hashKey(x5)] ) != "undefined" && Object.prototype.hasOwnProperty.call(o.index, this.ip.hashKey(x5)) ) ) {
          return this.ip.FALSE;
        }
      }
      return this.ip.TRUE;
    }
    this.ip.raise("AttributeError", ((("'" + this.ip.typeName(__self)) + "' object has no attribute '") + name) + "'");
    return this.ip.NONE;
  };
  isStd (f) {
    return f.s == "<stdout>" || f.s == "<stderr>";
  };
  fileText (f) {
    const t = this.ip.fs.read(f.s);
    if ( typeof(t) === "undefined" ) {
      this.ip.raise("FileNotFoundError", "[Errno 2] No such file or directory: " + PyFormat.strRepr(f.s));
    }
    return t;
  };
  fileWrite (f, text) {
    if ( this.isStd(f) ) {
      this.ip.writeOut(text);
      return;
    }
    if ( f.closed ) {
      this.ip.raise("ValueError", "I/O operation on closed file.");
    }
    const plain = f.mode.split("b").join("").split("t").join("");
    if ( plain == "r" ) {
      this.ip.raise("OSError", "not writable");
    }
    this.ip.fs.append(f.s, text);
  };
  fileLines (f) {
    let out = [];
    if ( this.isStd(f) ) {
      return out;
    }
    const text = this.fileText(f);
    const cs = Array.from(text, (rg_c) => rg_c.codePointAt(0));
    const n = cs.length;
    let start = f.pos;
    let i = f.pos;
    while (i < n) {
      if ( cs[i] == 10 ) {
        out.push(this.ip.mkStr(py_rt_codes(cs, start, i + 1)));
        start = i + 1;
      }
      i = i + 1;
    };
    if ( start < n ) {
      out.push(this.ip.mkStr(py_rt_codes(cs, start, n)));
    }
    f.pos = n;
    return out;
  };
  fileMethod (__self, name, args, kn, kv) {
    if ( name == "write" ) {
      this.need("write", args, 1, 1);
      const t = args[0];
      if ( t.t != 4 ) {
        this.ip.raise("TypeError", "write() argument must be str, not " + this.ip.typeName(t));
      }
      this.fileWrite(__self, t.s);
      return this.ip.mkInt(t.strLen());
    }
    if ( name == "writelines" ) {
      const vals = this.ip.iterValues(args[0]);
      // Loop start
      for ( const v of vals) {
        this.fileWrite(__self, this.ip.toStr(v));
      }
      return this.ip.NONE;
    }
    if ( name == "close" || name == "__exit__" ) {
      __self.closed = true;
      if ( name == "__exit__" ) {
        return this.ip.FALSE;
      }
      return this.ip.NONE;
    }
    if ( name == "__enter__" ) {
      return __self;
    }
    if ( (name == "flush" || name == "readable") || name == "writable" ) {
      if ( name == "readable" ) {
        return this.ip.mkBool((__self.mode.indexOf("r") >= 0));
      }
      if ( name == "writable" ) {
        return this.ip.mkBool((__self.mode.indexOf("r") < 0));
      }
      return this.ip.NONE;
    }
    if ( __self.closed ) {
      this.ip.raise("ValueError", "I/O operation on closed file.");
    }
    if ( name == "read" ) {
      const text = this.fileText(__self);
      const cs = Array.from(text, (rg_c) => rg_c.codePointAt(0));
      const n = cs.length;
      let cnt = n - __self.pos;
      const __REGx1 = this.ip.argAt(args, 0);
      const __REGx2 = this.ip.argAt(args, 0);
      if ( (args.length > 0 && __REGx1.t != 0) && __REGx2.i >= 0 ) {
        const __REGx3 = args[0];
        if ( __REGx3.i < cnt ) {
          const tv10 = args[0];
          cnt = tv10.i;
        }
      }
      if ( cnt < 0 ) {
        cnt = 0;
      }
      const r = py_rt_codes(cs, __self.pos, __self.pos + cnt);
      __self.pos = __self.pos + cnt;
      return this.ip.mkStr(r);
    }
    if ( name == "readline" ) {
      const text2 = this.fileText(__self);
      const cs2 = Array.from(text2, (rg_c) => rg_c.codePointAt(0));
      const n2 = cs2.length;
      let i2 = __self.pos;
      while (i2 < n2 && cs2[i2] != 10) {
        i2 = i2 + 1;
      };
      if ( i2 < n2 ) {
        i2 = i2 + 1;
      }
      const r2 = py_rt_codes(cs2, __self.pos, i2);
      __self.pos = i2;
      return this.ip.mkStr(r2);
    }
    if ( name == "readlines" ) {
      return this.ip.mkList(this.fileLines(__self));
    }
    if ( name == "seek" ) {
      const tv11 = args[0];
      __self.pos = tv11.i;
      return this.ip.mkInt(__self.pos);
    }
    if ( name == "tell" ) {
      return this.ip.mkInt(__self.pos);
    }
    if ( name == "__iter__" ) {
      return __self;
    }
    if ( name == "__next__" ) {
      const line = this.fileMethod(__self, "readline", [], kn, kv);
      if ( line.s.length == 0 ) {
        this.ip.raise("StopIteration", "");
      }
      return line;
    }
    this.ip.raise("AttributeError", ("'TextIOWrapper' object has no attribute '" + name) + "'");
    return this.ip.NONE;
  };
}
export class PyModules  {
  constructor(interp) {
    this.ip = undefined;
    this.mt = [];
    this.mti = 625;
    this.seeded = false;
    this.gaussNext = 0.0;
    this.hasGauss = false;
    this.jcs = [];
    this.jpos = 0;
    this.jdoc = "";     /* note: unused */
    this.ip = interp;
  }
  fns (m, names) {
    const ns = names.split(" ");
    // Loop start
    for ( const n of ns) {
      const b = new PyVal(10);
      b.s = (m.s + ".") + n;
      m.putAttr(n, b);
    }
  };
  floatInfo () {
    const m = this.newModule("sys.float_info");
    m.putAttr("max", this.ip.mkFloat(Number("1.7976931348623157e308")));
    m.putAttr("min", this.ip.mkFloat(Number("2.2250738585072014e-308")));
    m.putAttr("epsilon", this.ip.mkFloat(Number("2.220446049250313e-16")));
    m.putAttr("dig", this.ip.mkInt(15));
    m.putAttr("mant_dig", this.ip.mkInt(53));
    m.putAttr("max_exp", this.ip.mkInt(1024));
    m.putAttr("min_exp", this.ip.mkInt(-1021));
    m.putAttr("max_10_exp", this.ip.mkInt(308));
    m.putAttr("min_10_exp", this.ip.mkInt(-307));
    m.putAttr("radix", this.ip.mkInt(2));
    return m;
  };
  newModule (name) {
    const m = new PyVal(13);
    m.s = name;
    return m;
  };
  nativeClass (name) {
    const c = new PyVal(11);
    c.s = name;
    c.mode = "native";
    this.ip.computeMro(c);
    return c;
  };
  strList (names) {
    let out = [];
    const ns = names.split(" ");
    // Loop start
    for ( const n of ns) {
      out.push(this.ip.mkStr(n));
    }
    return this.ip.mkList(out);
  };
  make (name) {
    let none;
    if ( name == "math" ) {
      const m = this.newModule("math");
      this.fns(m, "sqrt sin cos tan asin acos atan atan2 sinh cosh tanh exp expm1 log log10 log2 log1p pow floor ceil trunc fabs hypot degrees radians isnan isinf isfinite isclose gcd lcm factorial comb perm prod fsum copysign fmod modf dist isqrt");
      m.putAttr("pi", this.ip.mkFloat(3.141592653589793));
      m.putAttr("e", this.ip.mkFloat(2.718281828459045));
      m.putAttr("tau", this.ip.mkFloat(6.283185307179586));
      m.putAttr("inf", this.ip.mkFloat(Infinity));
      m.putAttr("nan", this.ip.mkFloat(NaN));
      const optr79 = m;
      return optr79;
    }
    if ( name == "random" ) {
      const m2 = this.newModule("random");
      this.fns(m2, "seed random randint randrange choice choices shuffle uniform sample gauss normalvariate triangular expovariate getrandbits");
      const optr84 = m2;
      return optr84;
    }
    if ( name == "json" ) {
      const m3 = this.newModule("json");
      this.fns(m3, "dumps loads dump load");
      const ve = ( Object.prototype.hasOwnProperty.call(this.ip.builtins, "ValueError") ? this.ip.builtins["ValueError"] : undefined );
      const jc = this.nativeClass("JSONDecodeError");
      jc.mode = "builtin";
      jc.isExc = true;
      jc.bases.push(ve);
      this.ip.computeMro(jc);
      m3.putAttr("JSONDecodeError", jc);
      const optr96 = m3;
      return optr96;
    }
    if ( name == "os" ) {
      const m4 = this.newModule("os");
      this.fns(m4, "getcwd chdir listdir mkdir makedirs remove unlink rmdir rename replace walk getenv");
      m4.putAttr("sep", this.ip.mkStr("/"));
      m4.putAttr("linesep", this.ip.mkStr("\n"));
      m4.putAttr("name", this.ip.mkStr("posix"));
      m4.putAttr("curdir", this.ip.mkStr("."));
      m4.putAttr("environ", this.ip.mkDict());
      m4.putAttr("path", this.make("os.path"));
      const optr107 = m4;
      return optr107;
    }
    if ( name == "os.path" ) {
      const m5 = this.newModule("os.path");
      this.fns(m5, "join exists isfile isdir basename dirname split splitext abspath normpath getsize isabs expanduser");
      m5.putAttr("sep", this.ip.mkStr("/"));
      const optr113 = m5;
      return optr113;
    }
    if ( name == "sys" ) {
      const m6 = this.newModule("sys");
      this.fns(m6, "exit setrecursionlimit getrecursionlimit getsizeof intern _ranger_step_budget exc_info exception");
      let args = [];
      // Loop start
      for ( let i = 0; i < this.ip.argv.length; i++) {
        var a = this.ip.argv[i];
        args.push(this.ip.mkStr(a));
      }
      if ( args.length == 0 ) {
        args.push(this.ip.mkStr("main.py"));
      }
      m6.putAttr("argv", this.ip.mkList(args));
      const so = new PyVal(16);
      so.s = "<stdout>";
      so.mode = "w";
      m6.putAttr("stdout", so);
      const se = new PyVal(16);
      se.s = "<stderr>";
      se.mode = "w";
      m6.putAttr("stderr", se);
      m6.putAttr("maxsize", this.ip.mkInt(9007199254740991));
      m6.putAttr("version", this.ip.mkStr("3.13.0 (Ranger Python runtime)"));
      m6.putAttr("platform", this.ip.mkStr("ranger"));
      m6.putAttr("path", this.ip.mkList([]));
      m6.putAttr("modules", this.ip.sysModules);
      m6.putAttr("byteorder", this.ip.mkStr("little"));
      m6.putAttr("implementation", this.newModule("ranger"));
      m6.putAttr("float_info", this.floatInfo());
      const ii = this.newModule("sys.int_info");
      ii.putAttr("bits_per_digit", this.ip.mkInt(30));
      ii.putAttr("sizeof_digit", this.ip.mkInt(4));
      ii.putAttr("default_max_str_digits", this.ip.mkInt(4300));
      ii.putAttr("str_digits_check_threshold", this.ip.mkInt(640));
      m6.putAttr("int_info", ii);
      m6.putAttr("maxunicode", this.ip.mkInt(1114111));
      m6.putAttr("executable", this.ip.mkStr(""));
      m6.putAttr("prefix", this.ip.mkStr(""));
      m6.putAttr("warnoptions", this.ip.mkList([]));
      const fl = this.newModule("sys.flags");
      const fnames = ["debug", "inspect", "interactive", "optimize", "dont_write_bytecode", "no_user_site", "no_site", "ignore_environment", "verbose", "bytes_warning", "quiet", "hash_randomization", "isolated", "dev_mode", "utf8_mode", "warn_default_encoding", "safe_path", "int_max_str_digits"];
      // Loop start
      for ( const fln of fnames) {
        fl.putAttr(fln, this.ip.mkInt(0));
      }
      m6.putAttr("flags", fl);
      m6.putAttr("hexversion", this.ip.mkInt(51183856));
      m6.putAttr("version_info", this.ip.mkTuple([this.ip.mkInt(3), this.ip.mkInt(13), this.ip.mkInt(0), this.ip.mkStr("final"), this.ip.mkInt(0)]));
      const optr139 = m6;
      return optr139;
    }
    if ( name == "_string" ) {
      const m7 = this.newModule("string");
      m7.putAttr("ascii_lowercase", this.ip.mkStr("abcdefghijklmnopqrstuvwxyz"));
      m7.putAttr("ascii_uppercase", this.ip.mkStr("ABCDEFGHIJKLMNOPQRSTUVWXYZ"));
      m7.putAttr("ascii_letters", this.ip.mkStr("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"));
      m7.putAttr("digits", this.ip.mkStr("0123456789"));
      m7.putAttr("hexdigits", this.ip.mkStr("0123456789abcdefABCDEF"));
      m7.putAttr("octdigits", this.ip.mkStr("01234567"));
      const punct = "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~";
      m7.putAttr("punctuation", this.ip.mkStr(punct));
      const ws = (" \t\n\r" + String.fromCodePoint(11)) + String.fromCodePoint(12);
      m7.putAttr("whitespace", this.ip.mkStr(ws));
      m7.putAttr("printable", this.ip.mkStr((("0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ" + punct) + ws)));
      const optr154 = m7;
      return optr154;
    }
    if ( name == "time" ) {
      const m8 = this.newModule("time");
      this.fns(m8, "time perf_counter monotonic sleep time_ns process_time");
      const optr159 = m8;
      return optr159;
    }
    if ( name == "builtins" ) {
      const mb = this.newModule("builtins");
      mb.attrs = this.ip.builtins;
      const optrmb = mb;
      return optrmb;
    }
    if ( name == "_collections" ) {
      const m9 = this.newModule("collections");
      m9.putAttr("defaultdict", this.nativeClass("defaultdict"));
      m9.putAttr("Counter", this.nativeClass("Counter"));
      m9.putAttr("OrderedDict", this.nativeClass("OrderedDict"));
      m9.putAttr("deque", this.nativeClass("deque"));
      this.fns(m9, "namedtuple");
      const optr168 = m9;
      return optr168;
    }
    if ( name == "itertools" ) {
      const m10 = this.newModule("itertools");
      this.fns(m10, "chain product permutations combinations combinations_with_replacement accumulate groupby islice zip_longest starmap takewhile dropwhile filterfalse pairwise compress repeat tee count cycle batched");
      const optr173 = m10;
      return optr173;
    }
    if ( name == "_functools" ) {
      const m11 = this.newModule("functools");
      this.fns(m11, "reduce partial lru_cache cache wraps total_ordering cmp_to_key");
      const optr178 = m11;
      return optr178;
    }
    if ( name == "statistics" ) {
      const m12 = this.newModule("statistics");
      this.fns(m12, "mean fmean median median_low median_high mode stdev pstdev variance pvariance");
      const se2 = this.nativeClass("StatisticsError");
      se2.mode = "builtin";
      se2.isExc = true;
      se2.bases.push(( Object.prototype.hasOwnProperty.call(this.ip.builtins, "ValueError") ? this.ip.builtins["ValueError"] : undefined ));
      this.ip.computeMro(se2);
      m12.putAttr("StatisticsError", se2);
      const optr189 = m12;
      return optr189;
    }
    if ( name == "copy" ) {
      const m13 = this.newModule("copy");
      this.fns(m13, "copy deepcopy");
      const optr194 = m13;
      return optr194;
    }
    return none;
  };
  num (args, i, fname) {
    if ( i >= args.length ) {
      this.ip.raise("TypeError", fname + "() missing required argument");
    }
    return this.ip.bi.toNum(args[i]);
  };
  domain () {
    this.ip.raise("ValueError", "math domain error");
  };
  checkF (x) {
    if ( (Math.abs(x) === Infinity) || Number.isNaN(x) ) {
      if ( (Math.abs(x) === Infinity) ) {
        this.ip.raise("OverflowError", "math range error");
      }
    }
    return this.ip.mkFloat(x);
  };
  intOf (x) {
    return this.ip.bi.toIntValue(this.ip.mkFloat(x));
  };
  bitLength (n0) {
    let n = n0;
    if ( n < 0 ) {
      n = 0 - n;
    }
    let k = 0;
    while (n > 0) {
      n = ((n / 2) | 0);
      k = k + 1;
    };
    return k;
  };
  codesOf (s) {
    const v = this.ip.mkStr(s);
    return v.codes();
  };
  listOf (v) {
    return this.ip.iterValues(v);
  };
  call1 (f, a) {
    let kn = [];
    let kv = [];
    return this.ip.callValue(f, [a], kn, kv);
  };
  call2 (f, a, b) {
    let kn = [];
    let kv = [];
    return this.ip.callValue(f, [a, b], kn, kv);
  };
  callBuiltin (name, args, kn, kv) {
    const dot = name.lastIndexOf(".");
    if ( dot < 0 ) {
      this.ip.raise("NameError", ("name '" + name) + "' is not defined");
    }
    const mod = name.substring(0, dot );
    const f = name.substring(dot + 1, name.length );
    if ( mod == "math" ) {
      return this.mathCall(f, args, kn, kv);
    }
    if ( mod == "random" ) {
      return this.randomCall(f, args, kn, kv);
    }
    if ( mod == "json" ) {
      return this.jsonCall(f, args, kn, kv);
    }
    if ( mod == "os" || mod == "os.path" ) {
      return this.osCall(mod, f, args, kn, kv);
    }
    if ( mod == "sys" ) {
      if ( f == "exit" ) {
        let code = this.ip.NONE;
        if ( args.length > 0 ) {
          code = args[0];
        }
        this.ip.raiseVal(this.ip.makeExcArg("SystemExit", code));
      }
      if ( f == "setrecursionlimit" ) {
        let lim = this.ip.bi.toInt(args[0], "limit");
        if ( lim > 900 ) {
          lim = 900;
        }
        this.ip.maxDepth = lim;
        return this.ip.NONE;
      }
      if ( f == "exc_info" || f == "exception" ) {
        if ( this.ip.handling.length == 0 ) {
          if ( f == "exception" ) {
            return this.ip.NONE;
          }
          return this.ip.mkTuple([this.ip.NONE, this.ip.NONE, this.ip.NONE]);
        }
        const ex = this.ip.handling[(this.ip.handling.length - 1)];
        if ( f == "exception" ) {
          return ex;
        }
        return this.ip.mkTuple([this.ip.bi.typeOf(ex), ex, this.ip.NONE]);
      }
      if ( f == "getrecursionlimit" ) {
        return this.ip.mkInt(1000);
      }
      if ( f == "intern" ) {
        return args[0];
      }
      if ( f == "getsizeof" ) {
        return this.ip.mkInt(64);
      }
      if ( f == "_ranger_step_budget" ) {
        this.ip.maxSteps = this.ip.steps + this.ip.bi.toInt(args[0], "budget");
        return this.ip.NONE;
      }
    }
    if ( mod == "time" ) {
      if ( f == "sleep" ) {
        return this.ip.NONE;
      }
      if ( f == "time_ns" ) {
        return this.ip.mkInt(Math.trunc(Date.now() * 1000000.0));
      }
      return this.ip.mkFloat((Date.now() / 1000.0));
    }
    if ( mod == "collections" ) {
      return this.namedTuple(args, kn, kv);
    }
    if ( mod == "itertools" ) {
      return this.iterCall(f, args, kn, kv);
    }
    if ( mod == "functools" ) {
      return this.funcCall(f, args, kn, kv);
    }
    if ( mod == "statistics" ) {
      return this.statCall(f, args, kn, kv);
    }
    if ( mod == "copy" ) {
      if ( f == "copy" ) {
        return this.shallowCopy(args[0]);
      }
      return this.deepCopy(args[0]);
    }
    if ( mod == "typing" ) {
      if ( f == "cast" ) {
        return args[1];
      }
      if ( f == "TypeVar" ) {
        return this.ip.mkStr(this.ip.toStr(args[0]));
      }
      return this.ip.NONE;
    }
    this.ip.raise("AttributeError", ((("module '" + mod) + "' has no attribute '") + f) + "'");
    return this.ip.NONE;
  };
  mathCall (f, args, kn, kv) {
    const n = args.length;
    if ( f == "sqrt" ) {
      const x = this.num(args, 0, f);
      if ( x < 0.0 ) {
        this.domain();
      }
      return this.ip.mkFloat(Math.sqrt(x));
    }
    if ( f == "sin" ) {
      return this.ip.mkFloat(Math.sin(this.num(args, 0, f)));
    }
    if ( f == "cos" ) {
      return this.ip.mkFloat(Math.cos(this.num(args, 0, f)));
    }
    if ( f == "tan" ) {
      return this.ip.mkFloat(Math.tan(this.num(args, 0, f)));
    }
    if ( f == "asin" || f == "acos" ) {
      const x2 = this.num(args, 0, f);
      if ( x2 > 1.0 || x2 < -1.0 ) {
        this.domain();
      }
      if ( f == "asin" ) {
        return this.ip.mkFloat(Math.asin(x2));
      }
      return this.ip.mkFloat(Math.acos(x2));
    }
    if ( f == "atan" ) {
      return this.ip.mkFloat(Math.atan(this.num(args, 0, f)));
    }
    if ( f == "atan2" ) {
      return this.ip.mkFloat(Math.atan2(this.num(
        args,
        0,
        f
      ), this.num(args, 1, f)));
    }
    if ( (f == "sinh" || f == "cosh") || f == "tanh" ) {
      const x3 = this.num(args, 0, f);
      const ep = Math.exp(x3);
      const em = Math.exp(0.0 - x3);
      if ( f == "sinh" ) {
        return this.checkF(((ep - em) / 2.0));
      }
      if ( f == "cosh" ) {
        return this.checkF(((ep + em) / 2.0));
      }
      if ( x3 > 20.0 ) {
        return this.ip.mkFloat(1.0);
      }
      if ( x3 < -20.0 ) {
        return this.ip.mkFloat(-1.0);
      }
      return this.ip.mkFloat(((ep - em) / (ep + em)));
    }
    if ( f == "exp" ) {
      return this.checkF(Math.exp(this.num(args, 0, f)));
    }
    if ( f == "expm1" ) {
      const x4 = this.num(args, 0, f);
      if ( x4 < 0.00001 && x4 > -0.00001 ) {
        return this.ip.mkFloat(((x4 + (x4 * x4) / 2.0) + ((x4 * x4) * x4) / 6.0));
      }
      return this.checkF((Math.exp(x4) - 1.0));
    }
    if ( f == "log" ) {
      const x5 = this.num(args, 0, f);
      if ( x5 <= 0.0 ) {
        this.domain();
      }
      if ( n > 1 ) {
        const b = this.num(args, 1, f);
        if ( b <= 0.0 ) {
          this.domain();
        }
        const lb = Math.log(b);
        if ( lb == 0.0 ) {
          this.ip.raise("ZeroDivisionError", "division by zero");
        }
        return this.ip.mkFloat((Math.log(x5) / lb));
      }
      return this.ip.mkFloat(Math.log(x5));
    }
    if ( f == "log10" || f == "log2" ) {
      const x6 = this.num(args, 0, f);
      if ( x6 <= 0.0 ) {
        this.domain();
      }
      if ( f == "log10" ) {
        return this.ip.mkFloat(Math.log10(x6));
      }
      return this.ip.mkFloat(Math.log2(x6));
    }
    if ( f == "log1p" ) {
      const x7 = this.num(args, 0, f);
      if ( x7 <= -1.0 ) {
        this.domain();
      }
      const u = 1.0 + x7;
      if ( u == 1.0 ) {
        return this.ip.mkFloat(x7);
      }
      return this.ip.mkFloat((Math.log(u) * (x7 / (u - 1.0))));
    }
    if ( f == "pow" ) {
      const a = this.num(args, 0, f);
      const b2 = this.num(args, 1, f);
      if ( a == 0.0 && b2 < 0.0 ) {
        this.domain();
      }
      if ( (a < 0.0 && Math.floor(b2) != b2) && (Math.abs(b2) === Infinity) == false ) {
        this.domain();
      }
      const r = Math.pow(a, b2);
      if ( ((Math.abs(r) === Infinity) && (Math.abs(a) === Infinity) == false) && (Math.abs(b2) === Infinity) == false ) {
        this.ip.raise("OverflowError", "math range error");
      }
      return this.ip.mkFloat(r);
    }
    if ( (f == "floor" || f == "ceil") || f == "trunc" ) {
      const v = args[0];
      if ( this.ip.isIntLike(v) ) {
        return this.ip.mkInt(v.i);
      }
      const mname = ("__" + f) + "__";
      const dm = this.ip.findMethod(v, mname);
      if ( (typeof(dm) !== "undefined" && dm != null )  ) {
        return this.ip.callMethodVal(dm, v, []);
      }
      const x8 = this.num(args, 0, f);
      if ( f == "floor" ) {
        return this.intOf(Math.floor(x8));
      }
      if ( f == "ceil" ) {
        return this.intOf(Math.ceil(x8));
      }
      return this.intOf(x8);
    }
    if ( f == "fabs" ) {
      const x9 = this.num(args, 0, f);
      if ( x9 < 0.0 ) {
        return this.ip.mkFloat((0.0 - x9));
      }
      return this.ip.mkFloat((x9 + 0.0));
    }
    if ( f == "hypot" || f == "dist" ) {
      let xs = [];
      if ( f == "hypot" ) {
        // Loop start
        for ( const av of args) {
          xs.push(this.ip.bi.toNum(av));
        }
      } else {
        const p = this.listOf(args[0]);
        const q = this.listOf(args[1]);
        if ( p.length != q.length ) {
          this.ip.raise("ValueError", "both points must have the same number of dimensions");
        }
        // Loop start
        for ( let pi = 0; pi < p.length; pi++) {
          var pv = p[pi];
          xs.push(this.ip.bi.toNum(pv) - this.ip.bi.toNum(q[pi]));
        }
      }
      let big = 0.0;
      // Loop start
      for ( const xv of xs) {
        let ax = xv;
        if ( ax < 0.0 ) {
          ax = 0.0 - ax;
        }
        if ( (Math.abs(ax) === Infinity) ) {
          return this.ip.mkFloat(Infinity);
        }
        if ( ax > big ) {
          big = ax;
        }
      }
      if ( big == 0.0 ) {
        return this.ip.mkFloat(0.0);
      }
      if ( xs.length == 2 ) {
        const h0 = xs[0];
        const h1 = xs[1];
        const hs = h0 * h0 + h1 * h1;
        if ( (Math.abs(hs) === Infinity) == false ) {
          if ( hs > 0.0 ) {
            return this.ip.mkFloat(Math.sqrt(hs));
          }
        }
      }
      let sq = [];
      // Loop start
      for ( const xv2 of xs) {
        const t = xv2 / big;
        sq.push(t * t);
      }
      return this.ip.mkFloat((big * Math.sqrt(this.fsumD(sq))));
    }
    if ( f == "degrees" ) {
      return this.ip.mkFloat((this.num(
        args,
        0,
        f
      ) * (180.0 / 3.141592653589793)));
    }
    if ( f == "radians" ) {
      return this.ip.mkFloat((this.num(
        args,
        0,
        f
      ) * (3.141592653589793 / 180.0)));
    }
    if ( f == "isnan" ) {
      return this.ip.mkBool(Number.isNaN(this.num(args, 0, f)));
    }
    if ( f == "isinf" ) {
      return this.ip.mkBool((Math.abs(this.num(args, 0, f)) === Infinity));
    }
    if ( f == "isfinite" ) {
      const x10 = this.num(args, 0, f);
      return this.ip.mkBool(((Math.abs(x10) === Infinity) == false && Number.isNaN(x10) == false));
    }
    if ( f == "isclose" ) {
      const a2 = this.num(args, 0, f);
      const b3 = this.num(args, 1, f);
      let rel = 1e-9;
      let abt = 0.0;
      const rk = this.ip.bi.kw(kn, kv, "rel_tol");
      if ( (typeof(rk) !== "undefined" && rk != null )  ) {
        rel = this.ip.bi.toNum(rk);
      }
      const ak = this.ip.bi.kw(kn, kv, "abs_tol");
      if ( (typeof(ak) !== "undefined" && ak != null )  ) {
        abt = this.ip.bi.toNum(ak);
      }
      if ( a2 == b3 ) {
        return this.ip.TRUE;
      }
      if ( (Math.abs(a2) === Infinity) || (Math.abs(b3) === Infinity) ) {
        return this.ip.FALSE;
      }
      let d = a2 - b3;
      if ( d < 0.0 ) {
        d = 0.0 - d;
      }
      let aa = a2;
      if ( aa < 0.0 ) {
        aa = 0.0 - aa;
      }
      let bb = b3;
      if ( bb < 0.0 ) {
        bb = 0.0 - bb;
      }
      let m = aa;
      if ( bb > m ) {
        m = bb;
      }
      return this.ip.mkBool((d <= rel * m || d <= abt));
    }
    if ( f == "gcd" || f == "lcm" ) {
      let acc = 0;
      if ( f == "lcm" ) {
        acc = 1;
      }
      // Loop start
      for ( const gv of args) {
        let x11 = this.ip.bi.toInt(gv, f);
        if ( x11 < 0 ) {
          x11 = 0 - x11;
        }
        if ( f == "gcd" ) {
          acc = this.gcd(acc, x11);
        } else {
          if ( acc == 0 || x11 == 0 ) {
            acc = 0;
          } else {
            acc = ((acc / this.gcd(acc, x11)) | 0) * x11;
          }
        }
      }
      return this.ip.mkInt(acc);
    }
    if ( f == "isqrt" ) {
      const x12 = this.ip.bi.toInt(args[0], f);
      if ( x12 < 0 ) {
        this.ip.raise("ValueError", "isqrt() argument must be nonnegative");
      }
      let r2 = Math.trunc(Math.sqrt((x12)));
      while (r2 * r2 > x12) {
        r2 = r2 - 1;
      };
      while ((r2 + 1) * (r2 + 1) <= x12) {
        r2 = r2 + 1;
      };
      return this.ip.mkInt(r2);
    }
    if ( f == "factorial" ) {
      const x13 = this.ip.bi.toInt(args[0], f);
      if ( x13 < 0 ) {
        this.ip.raise("ValueError", "factorial() not defined for negative values");
      }
      let acc2 = this.ip.mkInt(1);
      let k = 2;
      while (k <= x13) {
        acc2 = this.ip.binop("*", acc2, this.ip.mkInt(k));
        k = k + 1;
      };
      return acc2;
    }
    if ( f == "comb" || f == "perm" ) {
      const nn = this.ip.bi.toInt(args[0], f);
      let kk = nn;
      const __REGx1 = this.ip.argAt(args, 1);
      if ( n > 1 && __REGx1.t != 0 ) {
        kk = this.ip.bi.toInt(args[1], f);
      }
      if ( nn < 0 || kk < 0 ) {
        this.ip.raise("ValueError", "n must be a non-negative integer");
      }
      if ( kk > nn ) {
        return this.ip.mkInt(0);
      }
      let acc3 = this.ip.mkInt(1);
      if ( f == "perm" ) {
        let j = 0;
        while (j < kk) {
          acc3 = this.ip.binop("*", acc3, this.ip.mkInt((nn - j)));
          j = j + 1;
        };
        return acc3;
      }
      if ( kk > nn - kk ) {
        kk = nn - kk;
      }
      let j2 = 1;
      while (j2 <= kk) {
        acc3 = this.ip.binop(
          "//",
          this.ip.binop("*", acc3, this.ip.mkInt(((nn - kk) + j2))),
          this.ip.mkInt(j2)
        );
        j2 = j2 + 1;
      };
      return acc3;
    }
    if ( f == "prod" ) {
      let acc4 = this.ip.mkInt(1);
      const st = this.ip.bi.kw(kn, kv, "start");
      if ( (typeof(st) !== "undefined" && st != null )  ) {
        acc4 = st;
      }
      // Loop start
      for ( let pj = 0; pj < this.listOf(args[0]).length; pj++) {
        var pv2 = this.listOf(args[0])[pj];
        acc4 = this.ip.binop("*", acc4, pv2);
      }
      return acc4;
    }
    if ( f == "fsum" ) {
      let ds = [];
      // Loop start
      for ( let fj = 0; fj < this.listOf(args[0]).length; fj++) {
        var fv = this.listOf(args[0])[fj];
        ds.push(this.ip.bi.toNum(fv));
      }
      return this.ip.mkFloat(this.fsumD(ds));
    }
    if ( f == "copysign" ) {
      let x14 = this.num(args, 0, f);
      const y = this.num(args, 1, f);
      if ( x14 < 0.0 ) {
        x14 = 0.0 - x14;
      }
      if ( y < 0.0 || y == 0.0 && PyFormat.isNegZero(y) ) {
        return this.ip.mkFloat((0.0 - x14));
      }
      return this.ip.mkFloat(x14);
    }
    if ( f == "fmod" ) {
      const x15 = this.num(args, 0, f);
      const y2 = this.num(args, 1, f);
      if ( y2 == 0.0 ) {
        this.domain();
      }
      const q2 = (Math.trunc(x15 / y2));
      return this.ip.mkFloat((x15 - q2 * y2));
    }
    if ( f == "modf" ) {
      const x16 = this.num(args, 0, f);
      const ip2 = (Math.trunc(x16));
      return this.ip.pair(this.ip.mkFloat((x16 - ip2)), this.ip.mkFloat(ip2));
    }
    this.ip.raise("AttributeError", ("module 'math' has no attribute '" + f) + "'");
    return this.ip.NONE;
  };
  gcd (a0, b0) {
    let a = a0;
    let b = b0;
    while (b != 0) {
      const t = a % b;
      a = b;
      b = t;
    };
    return a;
  };
  fsumD (xs) {
    let parts = [];
    // Loop start
    for ( const x0 of xs) {
      let x = x0;
      let keep = [];
      // Loop start
      for ( const y0 of parts) {
        let y = y0;
        let ax = x;
        if ( ax < 0.0 ) {
          ax = 0.0 - ax;
        }
        let ay = y;
        if ( ay < 0.0 ) {
          ay = 0.0 - ay;
        }
        if ( ax < ay ) {
          const t = x;
          x = y;
          y = t;
        }
        const hi = x + y;
        const lo = y - (hi - x);
        if ( lo != 0.0 ) {
          keep.push(lo);
        }
        x = hi;
      }
      keep.push(x);
      parts = keep;
    }
    let n = parts.length;
    if ( n == 0 ) {
      return 0.0;
    }
    n = n - 1;
    let hi2 = parts[n];
    let lo2 = 0.0;
    while (n > 0) {
      const x2 = hi2;
      n = n - 1;
      const y2 = parts[n];
      hi2 = x2 + y2;
      const yr = hi2 - x2;
      lo2 = y2 - yr;
      if ( lo2 != 0.0 ) {
        break;
      }
    };
    if ( n > 0 ) {
      const pn = parts[(n - 1)];
      if ( lo2 < 0.0 && pn < 0.0 || lo2 > 0.0 && pn > 0.0 ) {
        const y3 = lo2 * 2.0;
        const x3 = hi2 + y3;
        const yr2 = x3 - hi2;
        if ( y3 == yr2 ) {
          hi2 = x3;
        }
      }
    }
    return hi2;
  };
  mul32 (a, b) {
    const al = a % 65536;
    const ah = ((a / 65536) | 0);
    const bl = b % 65536;
    const bh = ((b / 65536) | 0);
    const lo = al * bl;
    const cross = (ah * bl + al * bh) % 65536;
    return (lo + cross * 65536) % 4294967296;
  };
  initGenrand (s) {
    let arr = [];
    arr.push(s % 4294967296);
    let i = 1;
    while (i < 624) {
      const prev = arr[(i - 1)];
      const x = ((prev ^ (prev >>> 30)) >>> 0);
      arr.push((this.mul32(1812433253, x) + i) % 4294967296);
      i = i + 1;
    };
    this.mt = arr;
    this.mti = 624;
  };
  initByArray (key) {
    this.initGenrand(19650218);
    let i = 1;
    let j = 0;
    const klen = key.length;
    let k = 624;
    if ( klen > k ) {
      k = klen;
    }
    while (k > 0) {
      const prev = this.mt[(i - 1)];
      const x = ((this.mt[i] ^ this.mul32(((prev ^ (prev >>> 30)) >>> 0), 1664525)) >>> 0);
      this.mt[i] = ((x + key[j]) + j) % 4294967296;
      i = i + 1;
      j = j + 1;
      if ( i >= 624 ) {
        this.mt[0] = this.mt[623];
        i = 1;
      }
      if ( j >= klen ) {
        j = 0;
      }
      k = k - 1;
    };
    k = 623;
    while (k > 0) {
      const prev2 = this.mt[(i - 1)];
      const x2 = ((this.mt[i] ^ this.mul32(((prev2 ^ (prev2 >>> 30)) >>> 0), 1566083941)) >>> 0);
      this.mt[i] = ((x2 - i) + 4294967296) % 4294967296;
      i = i + 1;
      if ( i >= 624 ) {
        this.mt[0] = this.mt[623];
        i = 1;
      }
      k = k - 1;
    };
    this.mt[0] = 2147483648;
    this.mti = 624;
    this.seeded = true;
    this.hasGauss = false;
  };
  seedInt (v0) {
    let v = v0;
    if ( v < 0 ) {
      v = 0 - v;
    }
    let key = [];
    while (v > 0) {
      key.push(v % 4294967296);
      v = ((v / 4294967296) | 0);
    };
    if ( key.length == 0 ) {
      key.push(0);
    }
    this.initByArray(key);
  };
  genrand () {
    if ( this.seeded == false ) {
      this.seedInt(Math.trunc(Date.now()));
    }
    if ( this.mti >= 624 ) {
      let kk = 0;
      while (kk < 624) {
        const y = ((((this.mt[kk] & 2147483648) >>> 0) | ((this.mt[((kk + 1) % 624)] & 2147483647) >>> 0)) >>> 0);
        let v = ((this.mt[((kk + 397) % 624)] ^ (y >>> 1)) >>> 0);
        if ( y % 2 == 1 ) {
          v = ((v ^ 2567483615) >>> 0);
        }
        this.mt[kk] = v;
        kk = kk + 1;
      };
      this.mti = 0;
    }
    let y2 = this.mt[this.mti];
    this.mti = this.mti + 1;
    y2 = ((y2 ^ (y2 >>> 11)) >>> 0);
    y2 = ((y2 ^ ((((y2 << 7) >>> 0) & 2636928640) >>> 0)) >>> 0);
    y2 = ((y2 ^ ((((y2 << 15) >>> 0) & 4022730752) >>> 0)) >>> 0);
    y2 = ((y2 ^ (y2 >>> 18)) >>> 0);
    return y2;
  };
  rnd () {
    const a = (this.genrand() >>> 5);
    const b = (this.genrand() >>> 6);
    return ((a) * 67108864.0 + (b)) * (1.0 / 9007199254740992.0);
  };
  getrandbits (k0) {
    if ( k0 <= 32 ) {
      return (this.genrand() >>> (32 - k0));
    }
    let k = k0;
    let result = 0;
    let mul = 1;
    while (k > 0) {
      let r = this.genrand();
      if ( k < 32 ) {
        r = (r >>> (32 - k));
      }
      result = result + r * mul;
      mul = mul * 4294967296;
      k = k - 32;
    };
    return result;
  };
  randbelow (n) {
    if ( n <= 0 ) {
      return 0;
    }
    const k = this.bitLength(n);
    let r = this.getrandbits(k);
    while (r >= n) {
      r = this.getrandbits(k);
    };
    return r;
  };
  randrange (start, stop, hasStop, step) {
    if ( hasStop == false ) {
      if ( start > 0 ) {
        return this.randbelow(start);
      }
      this.ip.raise("ValueError", "empty range for randrange()");
    }
    const width = stop - start;
    if ( step == 1 ) {
      if ( width > 0 ) {
        return start + this.randbelow(width);
      }
      this.ip.raise("ValueError", ((("empty range in randrange(" + ((start).toString())) + ", ") + ((stop).toString())) + ")");
    }
    let n = 0;
    if ( step > 0 ) {
      n = PyFormat.floorDivInt(((width + step) - 1), step);
    } else {
      if ( step < 0 ) {
        n = PyFormat.floorDivInt(((width + step) + 1), step);
      } else {
        this.ip.raise("ValueError", "zero step for randrange()");
      }
    }
    if ( n <= 0 ) {
      this.ip.raise("ValueError", ((((("empty range in randrange(" + ((start).toString())) + ", ") + ((stop).toString())) + ", ") + ((step).toString())) + ")");
    }
    return start + step * this.randbelow(n);
  };
  randomCall (f, args, kn, kv) {
    const n = args.length;
    if ( f == "seed" ) {
      const a = this.ip.bi.arg(args, 0, kn, kv, "a");
      if ( a.t == 0 ) {
        this.seedInt(Math.trunc(Date.now()));
        return this.ip.NONE;
      }
      if ( this.ip.isIntLike(a) ) {
        this.seedInt(a.i);
        return this.ip.NONE;
      }
      if ( a.t == 4 ) {
        this.seedInt(this.ip.bi.strHash(a.s));
        return this.ip.NONE;
      }
      this.seedInt(this.ip.bi.toInt(this.ip.bi.toIntValue(a), "seed"));
      return this.ip.NONE;
    }
    if ( f == "random" ) {
      return this.ip.mkFloat(this.rnd());
    }
    if ( f == "getrandbits" ) {
      return this.ip.mkInt(this.getrandbits(this.ip.bi.toInt(args[0], f)));
    }
    if ( f == "randint" ) {
      this.ip.bi.need("randint", args, 2, 2);
      const lo = this.ip.bi.toInt(args[0], f);
      const hi = this.ip.bi.toInt(args[1], f);
      return this.ip.mkInt(this.randrange(lo, (hi + 1), true, 1));
    }
    if ( f == "randrange" ) {
      const st = this.ip.bi.toInt(args[0], f);
      if ( n == 1 ) {
        return this.ip.mkInt(this.randrange(st, 0, false, 1));
      }
      let stp = 1;
      if ( n > 2 ) {
        stp = this.ip.bi.toInt(args[2], f);
      }
      return this.ip.mkInt(this.randrange(
        st,
        this.ip.bi.toInt(args[1], f),
        true,
        stp
      ));
    }
    if ( f == "choice" ) {
      const seq = args[0];
      const __len = this.ip.length(seq);
      if ( __len == 0 ) {
        this.ip.raise("IndexError", "Cannot choose from an empty sequence");
      }
      const pick = this.randbelow(__len);
      return this.ip.getItem(seq, this.ip.mkInt(pick));
    }
    if ( f == "shuffle" ) {
      const x = args[0];
      let i = x.items.length - 1;
      while (i >= 1) {
        const j = this.randbelow((i + 1));
        const t = x.items[i];
        x.items[i] = x.items[j];
        x.items[j] = t;
        i = i - 1;
      };
      return this.ip.NONE;
    }
    if ( f == "uniform" ) {
      const a2 = this.num(args, 0, f);
      const b2 = this.num(args, 1, f);
      return this.ip.mkFloat((a2 + (b2 - a2) * this.rnd()));
    }
    if ( f == "triangular" ) {
      let low = 0.0;
      let high = 1.0;
      if ( n > 0 ) {
        low = this.num(args, 0, f);
      }
      if ( n > 1 ) {
        high = this.num(args, 1, f);
      }
      let u = this.rnd();
      let c = 0.5;
      const __REGx1 = this.ip.argAt(args, 2);
      if ( n > 2 && __REGx1.t != 0 ) {
        if ( high == low ) {
          return this.ip.mkFloat(low);
        }
        c = (this.num(args, 2, f) - low) / (high - low);
      }
      if ( u > c ) {
        u = 1.0 - u;
        c = 1.0 - c;
        const t2 = low;
        low = high;
        high = t2;
      }
      return this.ip.mkFloat((low + (high - low) * Math.sqrt(u * c)));
    }
    if ( f == "expovariate" ) {
      let lam = 1.0;
      if ( n > 0 ) {
        lam = this.num(args, 0, f);
      }
      return this.ip.mkFloat(((0.0 - Math.log(1.0 - this.rnd())) / lam));
    }
    if ( f == "gauss" || f == "normalvariate" ) {
      let mu = 0.0;
      let sigma = 1.0;
      const mv = this.ip.bi.arg(args, 0, kn, kv, "mu");
      if ( mv.t != 0 ) {
        mu = this.ip.bi.toNum(mv);
      }
      const sv = this.ip.bi.arg(args, 1, kn, kv, "sigma");
      if ( sv.t != 0 ) {
        sigma = this.ip.bi.toNum(sv);
      }
      if ( f == "gauss" ) {
        let z = 0.0;
        if ( this.hasGauss ) {
          z = this.gaussNext;
          this.hasGauss = false;
        } else {
          const x2pi = this.rnd() * 6.283185307179586;
          const g2rad = Math.sqrt(-2.0 * Math.log(1.0 - this.rnd()));
          z = Math.cos(x2pi) * g2rad;
          this.gaussNext = Math.sin(x2pi) * g2rad;
          this.hasGauss = true;
        }
        return this.ip.mkFloat((mu + z * sigma));
      }
      const magic = (4.0 * Math.exp(-0.5)) / Math.sqrt(2.0);
      let z2 = 0.0;
      while (true) {
        const u1 = this.rnd();
        const u2 = 1.0 - this.rnd();
        z2 = (magic * (u1 - 0.5)) / u2;
        const zz = (z2 * z2) / 4.0;
        if ( zz <= 0.0 - Math.log(u2) ) {
          break;
        }
      };
      return this.ip.mkFloat((mu + z2 * sigma));
    }
    if ( f == "choices" ) {
      const pop = this.listOf(args[0]);
      const np = pop.length;
      let k = 1;
      const kk = this.ip.bi.kw(kn, kv, "k");
      if ( (typeof(kk) !== "undefined" && kk != null )  ) {
        k = this.ip.bi.toInt(kk, "k");
      }
      const wv = this.ip.bi.arg(args, 1, kn, kv, "weights");
      const cw = this.ip.bi.kw(kn, kv, "cum_weights");
      let out = [];
      if ( wv.t == 0 && (typeof(cw) === "undefined") ) {
        let i2 = 0;
        while (i2 < k) {
          out.push(pop[Math.trunc(Math.floor(this.rnd() * (np)))]);
          i2 = i2 + 1;
        };
        return this.ip.mkList(out);
      }
      let cum = [];
      if ( (typeof(cw) !== "undefined" && cw != null )  ) {
        // Loop start
        for ( let ci = 0; ci < this.listOf(cw).length; ci++) {
          var cv = this.listOf(cw)[ci];
          cum.push(this.ip.bi.toNum(cv));
        }
      } else {
        let run = 0.0;
        // Loop start
        for ( let wi = 0; wi < this.listOf(wv).length; wi++) {
          var wv2 = this.listOf(wv)[wi];
          run = run + this.ip.bi.toNum(wv2);
          cum.push(run);
        }
      }
      if ( cum.length != np ) {
        this.ip.raise("ValueError", "The number of weights does not match the population");
      }
      const total = cum[(np - 1)];
      if ( total <= 0.0 ) {
        this.ip.raise("ValueError", "Total of weights must be greater than zero");
      }
      let i3 = 0;
      while (i3 < k) {
        const r = this.rnd() * total;
        let lo2 = 0;
        let hi2 = np - 1;
        while (lo2 < hi2) {
          const mid = (((lo2 + hi2) / 2) | 0);
          if ( r < cum[mid] ) {
            hi2 = mid;
          } else {
            lo2 = mid + 1;
          }
        };
        out.push(pop[lo2]);
        i3 = i3 + 1;
      };
      return this.ip.mkList(out);
    }
    if ( f == "sample" ) {
      const pop2 = this.listOf(args[0]);
      const np2 = pop2.length;
      const k2 = this.ip.bi.toInt(this.ip.bi.arg(args, 1, kn, kv, "k"), "k");
      if ( k2 < 0 || k2 > np2 ) {
        this.ip.raise("ValueError", "Sample larger than population or is negative");
      }
      let res = [];
      let setsize = 21;
      if ( k2 > 5 ) {
        const e = Math.trunc(Math.ceil(Math.log((k2 * 3)) / Math.log(4.0)));
        let p4 = 1;
        let ei = 0;
        while (ei < e) {
          p4 = p4 * 4;
          ei = ei + 1;
        };
        setsize = setsize + p4;
      }
      if ( np2 <= setsize ) {
        let pool = [];
        // Loop start
        for ( const pv of pop2) {
          pool.push(pv);
        }
        let i4 = 0;
        while (i4 < k2) {
          const j2 = this.randbelow((np2 - i4));
          res.push(pool[j2]);
          pool[j2] = pool[((np2 - i4) - 1)];
          i4 = i4 + 1;
        };
      } else {
        let sel = {};
        let i5 = 0;
        while (i5 < k2) {
          let j3 = this.randbelow(np2);
          while (( typeof(sel[j3] ) != "undefined" && Object.prototype.hasOwnProperty.call(sel, j3) )) {
            j3 = this.randbelow(np2);
          };
          sel[j3] = true;
          res.push(pop2[j3]);
          i5 = i5 + 1;
        };
      }
      return this.ip.mkList(res);
    }
    this.ip.raise("AttributeError", ("module 'random' has no attribute '" + f) + "'");
    return this.ip.NONE;
  };
  jsonStr (s, ascii) {
    const cs = this.codesOf(s);
    let out = [];
    out.push(34);
    // Loop start
    for ( const c of cs) {
      if ( c == 34 ) {
        out.push(92);
        out.push(34);
        continue;
      }
      if ( c == 92 ) {
        out.push(92);
        out.push(92);
        continue;
      }
      if ( c == 10 ) {
        out.push(92);
        out.push(110);
        continue;
      }
      if ( c == 13 ) {
        out.push(92);
        out.push(114);
        continue;
      }
      if ( c == 9 ) {
        out.push(92);
        out.push(116);
        continue;
      }
      if ( c == 8 ) {
        out.push(92);
        out.push(98);
        continue;
      }
      if ( c == 12 ) {
        out.push(92);
        out.push(102);
        continue;
      }
      if ( c < 32 || ascii && c > 126 ) {
        if ( c > 65535 ) {
          const v = c - 65536;
          this.pushU(out, 55296 + ((v / 1024) | 0));
          this.pushU(out, 56320 + v % 1024);
        } else {
          this.pushU(out, c);
        }
        continue;
      }
      out.push(c);
    }
    out.push(34);
    return py_rt_codes(out, 0, out.length);
  };
  pushU (out, c) {
    out.push(92);
    out.push(117);
    const hx = this.ip.bi.hexPad(c, 4);
    const hc = this.codesOf(hx);
    // Loop start
    for ( const h of hc) {
      out.push(h);
    }
  };
  jsonKey (k, ascii) {
    if ( k.t == 4 ) {
      return this.jsonStr(k.s, ascii);
    }
    if ( k.t == 1 ) {
      if ( k.i == 1 ) {
        return "\"true\"";
      }
      return "\"false\"";
    }
    if ( k.t == 0 ) {
      return "\"null\"";
    }
    if ( k.t == 2 ) {
      return ("\"" + this.ip.toStr(k)) + "\"";
    }
    if ( k.t == 3 ) {
      return ("\"" + this.jsonFloat(k.d)) + "\"";
    }
    this.ip.raise("TypeError", "keys must be str, int, float, bool or None, not " + this.ip.typeName(k));
    return "";
  };
  jsonFloat (x) {
    if ( Number.isNaN(x) ) {
      return "NaN";
    }
    if ( (Math.abs(x) === Infinity) ) {
      if ( x > 0.0 ) {
        return "Infinity";
      }
      return "-Infinity";
    }
    return PyFormat.floatRepr(x);
  };
  jsonDump (v, ind, hasInd, level, itemSep, keySep, sortKeys, ascii, dflt) {
    switch (v.t ) { 
      case 0 : 
        return "null";
      case 1 : 
        if ( v.i == 1 ) {
          return "true";
        }
        return "false";
      case 2 : 
        return this.ip.toStr(v);
      case 3 : 
        return this.jsonFloat(v.d);
      case 4 : 
        return this.jsonStr(v.s, ascii);
    };
    let nl = "";
    let nl2 = "";
    if ( hasInd ) {
      nl = "\n" + PyFormat.rep(ind, (level + 1));
      nl2 = "\n" + PyFormat.rep(ind, level);
    }
    if ( v.t == 5 || v.t == 6 ) {
      if ( v.items.length == 0 ) {
        return "[]";
      }
      let parts = [];
      // Loop start
      for ( const it of v.items) {
        parts.push(this.jsonDump(
          it,
          ind,
          hasInd,
          level + 1,
          itemSep,
          keySep,
          sortKeys,
          ascii,
          dflt
        ));
      }
      return ((("[" + nl) + parts.join(itemSep + nl)) + nl2) + "]";
    }
    if ( v.t == 7 ) {
      if ( v.items.length == 0 ) {
        return "{}";
      }
      let order = [];
      // Loop start
      for ( let ki = 0; ki < v.items.length; ki++) {
        var k = v.items[ki];
        order.push(ki);
      }
      if ( sortKeys ) {
        const ks = this.ip.bi.sortVals(v.items, this.ip.NONE, false);
        let order2 = [];
        // Loop start
        for ( const sk of ks) {
          // Loop start
          for ( let ki2 = 0; ki2 < v.items.length; ki2++) {
            var k2 = v.items[ki2];
            if ( this.ip.same(sk, k2) ) {
              order2.push(ki2);
              break;
            }
          }
        }
        order = order2;
      }
      let parts2 = [];
      // Loop start
      for ( const oi of order) {
        const kstr = this.jsonKey(v.items[oi], ascii);
        parts2.push((kstr + keySep) + this.jsonDump(
          v.vals[oi],
          ind,
          hasInd,
          (level + 1),
          itemSep,
          keySep,
          sortKeys,
          ascii,
          dflt
        ));
      }
      return ((("{" + nl) + parts2.join(itemSep + nl)) + nl2) + "}";
    }
    if ( dflt.t != 0 ) {
      return this.jsonDump(
        this.call1(dflt, v),
        ind,
        hasInd,
        level,
        itemSep,
        keySep,
        sortKeys,
        ascii,
        dflt
      );
    }
    this.ip.raise("TypeError", ("Object of type " + this.ip.typeName(v)) + " is not JSON serializable");
    return "";
  };
  jsonFail (msg, pos) {
    let line = 1;
    let last = -1;
    let i = 0;
    while (i < pos && i < this.jcs.length) {
      if ( this.jcs[i] == 10 ) {
        line = line + 1;
        last = i;
      }
      i = i + 1;
    };
    const col = pos - last;
    const full = ((((((msg + ": line ") + ((line).toString())) + " column ") + ((col).toString())) + " (char ") + ((pos).toString())) + ")";
    const jm = this.make("json");
    const cls = ( Object.prototype.hasOwnProperty.call(jm.attrs, "JSONDecodeError") ? jm.attrs["JSONDecodeError"] : undefined );
    this.ip.raiseVal(this.ip.makeExc(cls, full));
  };
  jsonWs () {
    while (this.jpos < this.jcs.length) {
      const c = this.jcs[this.jpos];
      if ( ((c == 32 || c == 9) || c == 10) || c == 13 ) {
        this.jpos = this.jpos + 1;
      } else {
        return;
      }
    };
  };
  jsonLit (word) {
    const wc = this.codesOf(word);
    const n = wc.length;
    if ( this.jpos + n > this.jcs.length ) {
      return false;
    }
    // Loop start
    for ( let i = 0; i < wc.length; i++) {
      var c = wc[i];
      if ( this.jcs[(this.jpos + i)] != c ) {
        return false;
      }
    }
    this.jpos = this.jpos + n;
    return true;
  };
  jsonValue () {
    this.jsonWs();
    if ( this.jpos >= this.jcs.length ) {
      this.jsonFail("Expecting value", this.jpos);
    }
    const c = this.jcs[this.jpos];
    if ( c == 123 ) {
      const d = this.ip.mkDict();
      const start = this.jpos;
      this.jpos = this.jpos + 1;
      this.jsonWs();
      if ( this.jpos < this.jcs.length && this.jcs[this.jpos] == 125 ) {
        this.jpos = this.jpos + 1;
        return d;
      }
      while (true) {
        this.jsonWs();
        if ( this.jpos >= this.jcs.length || this.jcs[this.jpos] != 34 ) {
          this.jsonFail("Expecting property name enclosed in double quotes", this.jpos);
        }
        const key = this.jsonString();
        this.jsonWs();
        if ( this.jpos >= this.jcs.length || this.jcs[this.jpos] != 58 ) {
          this.jsonFail("Expecting ':' delimiter", this.jpos);
        }
        this.jpos = this.jpos + 1;
        const val = this.jsonValue();
        this.ip.dictSet(d, key, val);
        this.jsonWs();
        if ( this.jpos < this.jcs.length && this.jcs[this.jpos] == 44 ) {
          this.jpos = this.jpos + 1;
          continue;
        }
        if ( this.jpos < this.jcs.length && this.jcs[this.jpos] == 125 ) {
          this.jpos = this.jpos + 1;
          return d;
        }
        this.jsonFail("Expecting ',' delimiter", this.jpos);
      };
      return d;
    }
    if ( c == 91 ) {
      let items = [];
      this.jpos = this.jpos + 1;
      this.jsonWs();
      if ( this.jpos < this.jcs.length && this.jcs[this.jpos] == 93 ) {
        this.jpos = this.jpos + 1;
        return this.ip.mkList(items);
      }
      while (true) {
        items.push(this.jsonValue());
        this.jsonWs();
        if ( this.jpos < this.jcs.length && this.jcs[this.jpos] == 44 ) {
          this.jpos = this.jpos + 1;
          continue;
        }
        if ( this.jpos < this.jcs.length && this.jcs[this.jpos] == 93 ) {
          this.jpos = this.jpos + 1;
          return this.ip.mkList(items);
        }
        this.jsonFail("Expecting ',' delimiter", this.jpos);
      };
      return this.ip.mkList(items);
    }
    if ( c == 34 ) {
      return this.jsonString();
    }
    if ( this.jsonLit("true") ) {
      return this.ip.TRUE;
    }
    if ( this.jsonLit("false") ) {
      return this.ip.FALSE;
    }
    if ( this.jsonLit("null") ) {
      return this.ip.NONE;
    }
    if ( this.jsonLit("NaN") ) {
      return this.ip.mkFloat(NaN);
    }
    if ( this.jsonLit("Infinity") ) {
      return this.ip.mkFloat(Infinity);
    }
    if ( this.jsonLit("-Infinity") ) {
      return this.ip.mkFloat((0.0 - Infinity));
    }
    const st = this.jpos;
    let isFloat = false;
    if ( c == 45 ) {
      this.jpos = this.jpos + 1;
    }
    let digits = 0;
    while (this.jpos < this.jcs.length && PyLexer.isDigit(this.jcs[this.jpos])) {
      this.jpos = this.jpos + 1;
      digits = digits + 1;
    };
    if ( digits == 0 ) {
      this.jpos = st;
      this.jsonFail("Expecting value", st);
    }
    if ( this.jpos < this.jcs.length && this.jcs[this.jpos] == 46 ) {
      const save = this.jpos;
      this.jpos = this.jpos + 1;
      let fd = 0;
      while (this.jpos < this.jcs.length && PyLexer.isDigit(this.jcs[this.jpos])) {
        this.jpos = this.jpos + 1;
        fd = fd + 1;
      };
      if ( fd == 0 ) {
        this.jpos = save;
      } else {
        isFloat = true;
      }
    }
    if ( this.jpos < this.jcs.length && (this.jcs[this.jpos] == 101 || this.jcs[this.jpos] == 69) ) {
      const save2 = this.jpos;
      this.jpos = this.jpos + 1;
      if ( this.jpos < this.jcs.length && (this.jcs[this.jpos] == 43 || this.jcs[this.jpos] == 45) ) {
        this.jpos = this.jpos + 1;
      }
      let ed = 0;
      while (this.jpos < this.jcs.length && PyLexer.isDigit(this.jcs[this.jpos])) {
        this.jpos = this.jpos + 1;
        ed = ed + 1;
      };
      if ( ed == 0 ) {
        this.jpos = save2;
      } else {
        isFloat = true;
      }
    }
    const text = py_rt_codes(this.jcs, st, this.jpos);
    if ( isFloat ) {
      return this.ip.mkFloat(Number(text));
    }
    return this.ip.bi.parseIntText(text, 10);
  };
  jsonString () {
    const start = this.jpos;
    this.jpos = this.jpos + 1;
    let out = [];
    while (true) {
      if ( this.jpos >= this.jcs.length ) {
        this.jsonFail("Unterminated string starting at", start);
      }
      const c = this.jcs[this.jpos];
      if ( c == 34 ) {
        this.jpos = this.jpos + 1;
        return this.ip.mkStr(py_rt_codes(out, 0, out.length));
      }
      if ( c < 32 ) {
        this.jsonFail("Invalid control character at", this.jpos);
      }
      if ( c == 92 ) {
        this.jpos = this.jpos + 1;
        if ( this.jpos >= this.jcs.length ) {
          this.jsonFail("Unterminated string starting at", start);
        }
        const e = this.jcs[this.jpos];
        this.jpos = this.jpos + 1;
        if ( e == 110 ) {
          out.push(10);
          continue;
        }
        if ( e == 116 ) {
          out.push(9);
          continue;
        }
        if ( e == 114 ) {
          out.push(13);
          continue;
        }
        if ( e == 98 ) {
          out.push(8);
          continue;
        }
        if ( e == 102 ) {
          out.push(12);
          continue;
        }
        if ( (e == 34 || e == 92) || e == 47 ) {
          out.push(e);
          continue;
        }
        if ( e == 117 ) {
          const u = this.jsonHex4((this.jpos - 2));
          if ( (((u >= 55296 && u < 56320) && this.jpos + 1 < this.jcs.length) && this.jcs[this.jpos] == 92) && this.jcs[(this.jpos + 1)] == 117 ) {
            const save = this.jpos;
            this.jpos = this.jpos + 2;
            const u2 = this.jsonHex4((this.jpos - 2));
            if ( u2 >= 56320 && u2 < 57344 ) {
              out.push((65536 + (u - 55296) * 1024) + (u2 - 56320));
              continue;
            }
            this.jpos = save;
          }
          out.push(u);
          continue;
        }
        this.jsonFail("Invalid \\escape", this.jpos - 2);
      }
      out.push(c);
      this.jpos = this.jpos + 1;
    };
    return this.ip.NONE;
  };
  jsonHex4 (at) {
    let v = 0;
    let i = 0;
    while (i < 4) {
      if ( this.jpos >= this.jcs.length ) {
        this.jsonFail("Invalid \\uXXXX escape", at);
      }
      const d = PyLexer.digitValue(this.jcs[this.jpos]);
      if ( d < 0 || d > 15 ) {
        this.jsonFail("Invalid \\uXXXX escape", at);
      }
      v = v * 16 + d;
      this.jpos = this.jpos + 1;
      i = i + 1;
    };
    return v;
  };
  jsonLoads (text) {
    this.jcs = this.codesOf(text);
    this.jpos = 0;
    const v = this.jsonValue();
    this.jsonWs();
    if ( this.jpos < this.jcs.length ) {
      this.jsonFail("Extra data", this.jpos);
    }
    return v;
  };
  jsonCall (f, args, kn, kv) {
    if ( f == "loads" || f == "load" ) {
      let src = args[0];
      if ( f == "load" ) {
        src = this.ip.meth.callMethod(src, "read", [], [], []);
      }
      if ( src.t != 4 ) {
        this.ip.raise("TypeError", "the JSON object must be str, bytes or bytearray, not " + this.ip.typeName(src));
      }
      return this.jsonLoads(src.s);
    }
    const obj = args[0];
    let ind = "";
    let hasInd = false;
    const iv = this.ip.bi.kw(kn, kv, "indent");
    if ( (typeof(iv) !== "undefined" && iv != null )  ) {
      const ivv = iv;
      if ( ivv.t != 0 ) {
        hasInd = true;
        if ( ivv.t == 4 ) {
          ind = ivv.s;
        } else {
          ind = PyFormat.rep(" ", ivv.i);
        }
      }
    }
    let itemSep = ", ";
    let keySep = ": ";
    if ( hasInd ) {
      itemSep = ",";
    }
    const sv = this.ip.bi.kw(kn, kv, "separators");
    if ( (typeof(sv) !== "undefined" && sv != null )  ) {
      const svv = sv;
      if ( svv.t != 0 ) {
        const tv1 = svv.items[0];
        itemSep = tv1.s;
        const tv2 = svv.items[1];
        keySep = tv2.s;
      }
    }
    let sortKeys = false;
    const sk = this.ip.bi.kw(kn, kv, "sort_keys");
    if ( (typeof(sk) !== "undefined" && sk != null )  ) {
      sortKeys = this.ip.truthy(sk);
    }
    let ascii = true;
    const ea = this.ip.bi.kw(kn, kv, "ensure_ascii");
    if ( (typeof(ea) !== "undefined" && ea != null )  ) {
      ascii = this.ip.truthy(ea);
    }
    let dflt = this.ip.NONE;
    const dv = this.ip.bi.kw(kn, kv, "default");
    if ( (typeof(dv) !== "undefined" && dv != null )  ) {
      dflt = dv;
    }
    const text = this.jsonDump(
      obj,
      ind,
      hasInd,
      0,
      itemSep,
      keySep,
      sortKeys,
      ascii,
      dflt
    );
    if ( f == "dump" ) {
      this.ip.meth.fileWrite(args[1], text);
      return this.ip.NONE;
    }
    return this.ip.mkStr(text);
  };
  pathArg (args, i) {
    if ( i >= args.length ) {
      return ".";
    }
    const v = args[i];
    if ( v.t != 4 ) {
      this.ip.raise("TypeError", "expected str, bytes or os.PathLike object, not " + this.ip.typeName(v));
    }
    return v.s;
  };
  osErr (cls, errno, text, path) {
    this.ip.raise(cls, (((("[Errno " + ((errno).toString())) + "] ") + text) + ": ") + PyFormat.strRepr(path));
  };
  pathJoin (a, b) {
    if ( b.startsWith("/") ) {
      return b;
    }
    if ( a.length == 0 || a.endsWith("/") ) {
      return a + b;
    }
    return (a + "/") + b;
  };
  normPath (p) {
    if ( p.length == 0 ) {
      return ".";
    }
    const abs = p.startsWith("/");
    const parts = p.split("/");
    let out = [];
    // Loop start
    for ( const seg of parts) {
      if ( seg.length == 0 || seg == "." ) {
        continue;
      }
      if ( seg == ".." ) {
        const n = out.length;
        if ( n > 0 && out[(n - 1)] != ".." ) {
          out.pop();
          continue;
        }
        if ( abs ) {
          continue;
        }
      }
      out.push(seg);
    }
    const body = out.join("/");
    if ( abs ) {
      return "/" + body;
    }
    if ( body.length == 0 ) {
      return ".";
    }
    return body;
  };
  splitPath (p) {
    const i = p.lastIndexOf("/");
    let head = "";
    let tail = p;
    if ( i >= 0 ) {
      head = p.substring(0, i + 1 );
      tail = p.substring(i + 1, p.length );
      let h = head;
      while (h.length > 0) {
        if ( h.endsWith("/") ) {
          h = h.substring(0, h.length - 1 );
        } else {
          break;
        }
      };
      if ( h.length > 0 ) {
        head = h;
      }
    }
    return [head, tail];
  };
  osCall (mod, f, args, kn, kv) {
    const fs = this.ip.fs;
    if ( mod == "os.path" ) {
      if ( f == "join" ) {
        let acc = this.pathArg(args, 0);
        let i = 1;
        while (i < args.length) {
          acc = this.pathJoin(acc, this.pathArg(args, i));
          i = i + 1;
        };
        return this.ip.mkStr(acc);
      }
      if ( f == "exists" ) {
        return this.ip.mkBool(fs.exists(this.pathArg(args, 0)));
      }
      if ( f == "isfile" ) {
        return this.ip.mkBool(fs.isFile(this.pathArg(args, 0)));
      }
      if ( f == "isdir" ) {
        return this.ip.mkBool(fs.isDir(this.pathArg(args, 0)));
      }
      if ( f == "isabs" ) {
        return this.ip.mkBool(this.pathArg(args, 0).startsWith("/"));
      }
      if ( f == "expanduser" ) {
        return this.ip.mkStr(this.pathArg(args, 0));
      }
      if ( f == "abspath" ) {
        return this.ip.mkStr(fs.normalize(this.pathArg(args, 0)));
      }
      if ( f == "normpath" ) {
        return this.ip.mkStr(this.normPath(this.pathArg(args, 0)));
      }
      const sp = this.splitPath(this.pathArg(args, 0));
      if ( f == "basename" ) {
        return this.ip.mkStr(sp[1]);
      }
      if ( f == "dirname" ) {
        return this.ip.mkStr(sp[0]);
      }
      if ( f == "split" ) {
        return this.ip.pair(this.ip.mkStr(sp[0]), this.ip.mkStr(sp[1]));
      }
      if ( f == "splitext" ) {
        const p = this.pathArg(args, 0);
        const base = sp[1];
        const dot = base.lastIndexOf(".");
        let lead = 0;
        while (lead < base.length && base.substring(lead, lead + 1 ) == ".") {
          lead = lead + 1;
        };
        if ( dot >= lead && lead < base.length ) {
          const cut = p.length - (base.length - dot);
          return this.ip.pair(this.ip.mkStr(p.substring(0, cut )), this.ip.mkStr(p.substring(cut, p.length )));
        }
        return this.ip.pair(this.ip.mkStr(p), this.ip.mkStr(""));
      }
      if ( f == "getsize" ) {
        const path = this.pathArg(args, 0);
        const txt = fs.read(path);
        if ( typeof(txt) === "undefined" ) {
          if ( fs.isDir(path) ) {
            return this.ip.mkInt(4096);
          }
          this.osErr("FileNotFoundError", 2, "No such file or directory", path);
        }
        let bytes = 0;
        const cs = this.codesOf(txt);
        // Loop start
        for ( const c of cs) {
          if ( c < 128 ) {
            bytes = bytes + 1;
          } else {
            if ( c < 2048 ) {
              bytes = bytes + 2;
            } else {
              if ( c < 65536 ) {
                bytes = bytes + 3;
              } else {
                bytes = bytes + 4;
              }
            }
          }
        }
        return this.ip.mkInt(bytes);
      }
    }
    if ( f == "getcwd" ) {
      return this.ip.mkStr(fs.cwd);
    }
    if ( f == "getenv" ) {
      if ( args.length > 1 ) {
        return args[1];
      }
      return this.ip.NONE;
    }
    if ( f == "chdir" ) {
      const p2 = this.pathArg(args, 0);
      if ( fs.isDir(p2) == false ) {
        this.osErr("FileNotFoundError", 2, "No such file or directory", p2);
      }
      fs.cwd = fs.normalize(p2);
      return this.ip.NONE;
    }
    if ( f == "listdir" ) {
      const p3 = this.pathArg(args, 0);
      if ( fs.isDir(p3) == false ) {
        if ( fs.isFile(p3) ) {
          this.osErr("NotADirectoryError", 20, "Not a directory", p3);
        }
        this.osErr("FileNotFoundError", 2, "No such file or directory", p3);
      }
      let out = [];
      // Loop start
      for ( let ni = 0; ni < fs.listDir(p3).length; ni++) {
        var nm = fs.listDir(p3)[ni];
        out.push(this.ip.mkStr(nm));
      }
      return this.ip.mkList(out);
    }
    if ( f == "mkdir" || f == "makedirs" ) {
      const p4 = this.pathArg(args, 0);
      let exOk = false;
      const ek = this.ip.bi.kw(kn, kv, "exist_ok");
      if ( (typeof(ek) !== "undefined" && ek != null )  ) {
        exOk = this.ip.truthy(ek);
      }
      if ( fs.exists(p4) ) {
        if ( exOk && fs.isDir(p4) ) {
          return this.ip.NONE;
        }
        this.osErr("FileExistsError", 17, "File exists", p4);
      }
      if ( f == "mkdir" ) {
        const par = fs.parentOf(fs.normalize(p4));
        if ( fs.isDir(par) == false ) {
          this.osErr("FileNotFoundError", 2, "No such file or directory", p4);
        }
      }
      fs.mkdirs(p4);
      return this.ip.NONE;
    }
    if ( f == "remove" || f == "unlink" ) {
      const p5 = this.pathArg(args, 0);
      if ( fs.isDir(p5) ) {
        this.osErr("IsADirectoryError", 21, "Is a directory", p5);
      }
      if ( fs.remove(p5) == false ) {
        this.osErr("FileNotFoundError", 2, "No such file or directory", p5);
      }
      return this.ip.NONE;
    }
    if ( f == "rmdir" ) {
      const p6 = this.pathArg(args, 0);
      if ( fs.isDir(p6) == false ) {
        this.osErr("FileNotFoundError", 2, "No such file or directory", p6);
      }
      if ( fs.listDir(p6).length > 0 ) {
        this.osErr("OSError", 39, "Directory not empty", p6);
      }
      fs.removeDir(p6);
      return this.ip.NONE;
    }
    if ( f == "rename" || f == "replace" ) {
      const a = this.pathArg(args, 0);
      const b = this.pathArg(args, 1);
      const txt2 = fs.read(a);
      if ( typeof(txt2) === "undefined" ) {
        this.osErr("FileNotFoundError", 2, "No such file or directory", a);
      }
      fs.write(b, txt2);
      fs.remove(a);
      return this.ip.NONE;
    }
    if ( f == "walk" ) {
      const top = this.pathArg(args, 0);
      let out2 = [];
      this.walkInto(top, out2);
      return this.ip.mkIter(out2);
    }
    this.ip.raise("AttributeError", ((("module '" + mod) + "' has no attribute '") + f) + "'");
    return this.ip.NONE;
  };
  walkInto (top, out) {
    const fs = this.ip.fs;
    if ( fs.isDir(top) == false ) {
      return;
    }
    let dirs = [];
    let files = [];
    let dnames = [];
    // Loop start
    for ( let i = 0; i < fs.listDir(top).length; i++) {
      var nm = fs.listDir(top)[i];
      const full = this.pathJoin(top, nm);
      if ( fs.isDir(full) ) {
        dirs.push(this.ip.mkStr(nm));
        dnames.push(full);
      } else {
        files.push(this.ip.mkStr(nm));
      }
    }
    out.push(this.ip.mkTuple([this.ip.mkStr(top), this.ip.mkList(dirs), this.ip.mkList(files)]));
    // Loop start
    for ( const d of dnames) {
      this.walkInto(d, out);
    }
  };
  construct (cls, args, kn, kv) {
    const n = args.length;
    const kind = cls.s;
    if ( ( typeof(cls.attrs["_fields"] ) != "undefined" && Object.prototype.hasOwnProperty.call(cls.attrs, "_fields") ) ) {
      return this.ntMake(cls, args, kn, kv);
    }
    if ( kind == "deque" ) {
      const d = new PyVal(5);
      d.mode = "deque";
      d.putAttr("__class__", cls);
      if ( n > 0 ) {
        d.items = this.listOf(args[0]);
      }
      const ml = this.ip.bi.arg(args, 1, kn, kv, "maxlen");
      if ( ml.t != 0 ) {
        d.i3 = ml.i + 1;
        this.dequeTrim(d, false);
      }
      return d;
    }
    const dd = this.ip.mkDict();
    dd.putAttr("__class__", cls);
    let start = 0;
    if ( kind == "defaultdict" ) {
      const __REGx1 = this.ip.argAt(args, 0);
      if ( n > 0 && __REGx1.t != 0 ) {
        dd.putAttr("__missing__", args[0]);
      }
      start = 1;
    }
    if ( kind == "Counter" ) {
      dd.mode = "counter";
      if ( n > 0 ) {
        this.counterFeed(dd, args[0]);
      }
      // Loop start
      for ( let ki = 0; ki < kn.length; ki++) {
        var k = kn[ki];
        this.ip.dictSet(dd, this.ip.mkStr(k), kv[ki]);
      }
      return dd;
    }
    if ( n > start ) {
      this.ip.bi.fillDict(dd, args[start]);
    }
    // Loop start
    for ( let ki2 = 0; ki2 < kn.length; ki2++) {
      var k2 = kn[ki2];
      this.ip.dictSet(dd, this.ip.mkStr(k2), kv[ki2]);
    }
    return dd;
  };
  counterFeed (d, src) {
    if ( src.t == 7 ) {
      // Loop start
      for ( let i = 0; i < src.items.length; i++) {
        var k = src.items[i];
        const cur = this.ip.dictGet(d, k);
        if ( typeof(cur) === "undefined" ) {
          this.ip.dictSet(d, k, src.vals[i]);
        } else {
          this.ip.dictSet(d, k, this.ip.binop("+", cur, src.vals[i]));
        }
      }
      return;
    }
    // Loop start
    for ( let j = 0; j < this.listOf(src).length; j++) {
      var v = this.listOf(src)[j];
      const cur2 = this.ip.dictGet(d, v);
      if ( typeof(cur2) === "undefined" ) {
        this.ip.dictSet(d, v, this.ip.mkInt(1));
      } else {
        const __REGx1 = cur2;
        this.ip.dictSet(d, v, this.ip.mkInt((__REGx1.i + 1)));
      }
    }
  };
  dequeTrim (d, fromLeft) {
    if ( d.i3 <= 0 ) {
      return;
    }
    const lim = d.i3 - 1;
    while (d.items.length > lim) {
      if ( fromLeft ) {
        d.items.pop();
      } else {
        let rest = [];
        let i = 1;
        while (i < d.items.length) {
          rest.push(d.items[i]);
          i = i + 1;
        };
        d.items = rest;
      }
    };
  };
  namedTuple (args, kn, kv) {
    const name = this.ip.toStr(this.ip.bi.arg(args, 0, kn, kv, "typename"));
    const fv = this.ip.bi.arg(args, 1, kn, kv, "field_names");
    let fields = [];
    if ( fv.t == 4 ) {
      const txt = fv.s.split(",").join(" ");
      // Loop start
      for ( let i = 0; i < txt.split(" ").length; i++) {
        var p = txt.split(" ")[i];
        if ( p.length > 0 ) {
          fields.push(this.ip.mkStr(p));
        }
      }
    } else {
      // Loop start
      for ( let j = 0; j < this.listOf(fv).length; j++) {
        var p2 = this.listOf(fv)[j];
        fields.push(p2);
      }
    }
    const cls = this.nativeClass(name);
    cls.putAttr("_fields", this.ip.mkTuple(fields));
    const dv = this.ip.bi.kw(kn, kv, "defaults");
    if ( (typeof(dv) !== "undefined" && dv != null )  ) {
      cls.putAttr("__defaults__", this.ip.mkTuple(this.listOf(dv)));
    }
    const mk = new PyVal(10);
    mk.s = "collections._make";
    mk.wrapped = cls;
    cls.putAttr("_make", mk);
    return cls;
  };
  ntMake (cls, args, kn, kv) {
    const fields = ( Object.prototype.hasOwnProperty.call(cls.attrs, "_fields") ? cls.attrs["_fields"] : undefined );
    const nf = fields.items.length;
    let vals = [];
    let filled = [];
    // Loop start
    for ( let i = 0; i < fields.items.length; i++) {
      var f = fields.items[i];
      if ( i < args.length ) {
        vals.push(args[i]);
        filled.push(true);
      } else {
        vals.push(this.ip.NONE);
        filled.push(false);
      }
    }
    if ( args.length > nf ) {
      this.ip.raise("TypeError", ((((cls.s + "() takes ") + ((nf + 1).toString())) + " positional arguments but ") + ((args.length + 1).toString())) + " were given");
    }
    // Loop start
    for ( let ki = 0; ki < kn.length; ki++) {
      var k = kn[ki];
      let found = false;
      // Loop start
      for ( let fi = 0; fi < fields.items.length; fi++) {
        var f2 = fields.items[fi];
        if ( f2.s == k ) {
          vals[fi] = kv[ki];
          filled[fi] = true;
          found = true;
        }
      }
      if ( found == false ) {
        this.ip.raise("TypeError", ((cls.s + "() got an unexpected keyword argument '") + k) + "'");
      }
    }
    const dv = ( Object.prototype.hasOwnProperty.call(cls.attrs, "__defaults__") ? cls.attrs["__defaults__"] : undefined );
    if ( (typeof(dv) !== "undefined" && dv != null )  ) {
      const ds = dv;
      const off = nf - ds.items.length;
      // Loop start
      for ( let di = 0; di < ds.items.length; di++) {
        var d = ds.items[di];
        if ( filled[(off + di)] == false ) {
          vals[off + di] = d;
          filled[off + di] = true;
        }
      }
    }
    // Loop start
    for ( let fi2 = 0; fi2 < filled.length; fi2++) {
      var ok = filled[fi2];
      if ( ok == false ) {
        this.ip.raise("TypeError", ((cls.s + "() missing required argument: '") + fields.items[fi2].s) + "'");
      }
    }
    const t = this.ip.mkTuple(vals);
    t.putAttr("__class__", cls);
    return t;
  };
  ntAttr (t, name) {
    let none;
    const cls = ( Object.prototype.hasOwnProperty.call(t.attrs, "__class__") ? t.attrs["__class__"] : undefined );
    const fv = ( Object.prototype.hasOwnProperty.call(cls.attrs, "_fields") ? cls.attrs["_fields"] : undefined );
    if ( typeof(fv) === "undefined" ) {
      return none;
    }
    const fields = fv;
    // Loop start
    for ( let i = 0; i < fields.items.length; i++) {
      var f = fields.items[i];
      if ( f.s == name ) {
        const optr2205 = t.items[i];
        return optr2205;
      }
    }
    if ( name == "_fields" ) {
      const optr2209 = fields;
      return optr2209;
    }
    if ( name == "_asdict" || name == "_replace" ) {
      const optr2212 = this.ip.mkBuiltinMethod(t, ("nt." + name));
      return optr2212;
    }
    return none;
  };
  ntMethod (t, name, args, kn, kv) {
    const cls = ( Object.prototype.hasOwnProperty.call(t.attrs, "__class__") ? t.attrs["__class__"] : undefined );
    const fields = ( Object.prototype.hasOwnProperty.call(cls.attrs, "_fields") ? cls.attrs["_fields"] : undefined );
    if ( name == "nt._asdict" ) {
      const d = this.ip.mkDict();
      // Loop start
      for ( let i = 0; i < fields.items.length; i++) {
        var f = fields.items[i];
        this.ip.dictSet(d, f, t.items[i]);
      }
      return d;
    }
    let vals = [];
    // Loop start
    for ( const v of t.items) {
      vals.push(v);
    }
    // Loop start
    for ( let ki = 0; ki < kn.length; ki++) {
      var k = kn[ki];
      let found = false;
      // Loop start
      for ( let fi = 0; fi < fields.items.length; fi++) {
        var f2 = fields.items[fi];
        if ( f2.s == k ) {
          vals[fi] = kv[ki];
          found = true;
        }
      }
      if ( found == false ) {
        this.ip.raise("ValueError", ("Got unexpected field names: ['" + k) + "']");
      }
    }
    const t2 = this.ip.mkTuple(vals);
    t2.putAttr("__class__", cls);
    return t2;
  };
  reprOf (v) {
    const cv = ( Object.prototype.hasOwnProperty.call(v.attrs, "__class__") ? v.attrs["__class__"] : undefined );
    if ( typeof(cv) === "undefined" ) {
      return "";
    }
    const cls = cv;
    if ( v.t == 6 && ( typeof(cls.attrs["_fields"] ) != "undefined" && Object.prototype.hasOwnProperty.call(cls.attrs, "_fields") ) ) {
      const fields = ( Object.prototype.hasOwnProperty.call(cls.attrs, "_fields") ? cls.attrs["_fields"] : undefined );
      let parts = [];
      // Loop start
      for ( let i = 0; i < fields.items.length; i++) {
        var f = fields.items[i];
        parts.push((f.s + "=") + this.ip.toRepr(v.items[i]));
      }
      return ((cls.s + "(") + parts.join(", ")) + ")";
    }
    if ( v.t == 5 ) {
      const body = ("[" + this.ip.joinRepr(v.items)) + "]";
      if ( v.i3 > 0 ) {
        return ((("deque(" + body) + ", maxlen=") + ((v.i3 - 1).toString())) + ")";
      }
      return ("deque(" + body) + ")";
    }
    if ( v.t == 7 ) {
      const plain = new PyVal(7);
      plain.items = v.items;
      plain.vals = v.vals;
      plain.index = v.index;
      if ( cls.s == "defaultdict" ) {
        let fac = "None";
        const mv = ( Object.prototype.hasOwnProperty.call(v.attrs, "__missing__") ? v.attrs["__missing__"] : undefined );
        if ( (typeof(mv) !== "undefined" && mv != null )  ) {
          fac = this.ip.toRepr(mv);
        }
        return ((("defaultdict(" + fac) + ", ") + this.ip.toRepr(plain)) + ")";
      }
      if ( v.items.length == 0 ) {
        return cls.s + "()";
      }
      if ( cls.s == "Counter" ) {
        let pairs = [];
        // Loop start
        for ( let ki = 0; ki < v.items.length; ki++) {
          var k = v.items[ki];
          pairs.push(this.ip.pair(k, v.vals[ki]));
        }
        const keyf = this.ip.mkBuiltinMethod(this.ip.NONE, "__counter_key__");
        const sorted = this.ip.bi.sortVals(pairs, keyf, true);
        let ps = [];
        // Loop start
        for ( const p of sorted) {
          ps.push((this.ip.toRepr(p.items[0]) + ": ") + this.ip.toRepr(p.items[1]));
        }
        return ("Counter({" + ps.join(", ")) + "})";
      }
      return ((cls.s + "(") + this.ip.toRepr(plain)) + ")";
    }
    return "";
  };
  iterCall (f, args, kn, kv) {
    const n = args.length;
    let out = [];
    if ( f == "chain" || f == "chain_from_iterable" ) {
      const lc = this.ip.mkLazy("chain", "chain");
      if ( f == "chain_from_iterable" ) {
        lc.wrapped = this.ip.iterOf(args[0]);
      } else {
        lc.wrapped = this.ip.mkIter(args);
      }
      return lc;
    }
    if ( f == "count" ) {
      const lcount = this.ip.mkLazy("count", "count");
      let cst = this.ip.bi.arg(args, 0, kn, kv, "start");
      let cstep = this.ip.bi.arg(args, 1, kn, kv, "step");
      if ( cst.t == 0 ) {
        cst = this.ip.mkInt(0);
      }
      if ( cstep.t == 0 ) {
        cstep = this.ip.mkInt(1);
      }
      lcount.wrapped = cst;
      lcount.cls = cstep;
      return lcount;
    }
    if ( f == "cycle" ) {
      const lcy = this.ip.mkLazy("cycle", "cycle");
      lcy.wrapped = this.ip.iterOf(this.ip.argAt(args, 0));
      return lcy;
    }
    if ( f == "islice" ) {
      const ls = this.ip.mkLazy("islice", "islice");
      ls.i = 0;
      ls.i2 = -1;
      ls.i3 = 1;
      if ( n == 2 ) {
        const __REGx1 = args[1];
        if ( __REGx1.t != 0 ) {
          const sv1 = args[1];
          ls.i2 = this.ip.bi.toInt(sv1, "islice");
        }
      } else {
        const __REGx2 = this.ip.argAt(args, 1);
        if ( __REGx2.t != 0 ) {
          const sv2 = args[1];
          ls.i = this.ip.bi.toInt(sv2, "islice");
        }
        const __REGx3 = this.ip.argAt(args, 2);
        if ( __REGx3.t != 0 ) {
          const sv3 = args[2];
          ls.i2 = this.ip.bi.toInt(sv3, "islice");
        }
        const __REGx4 = this.ip.argAt(args, 3);
        if ( n > 3 && __REGx4.t != 0 ) {
          const sv4 = args[3];
          ls.i3 = this.ip.bi.toInt(sv4, "islice");
        }
      }
      if ( (ls.i < 0 || ls.i2 < -1) || ls.i3 < 1 ) {
        this.ip.raise("ValueError", "Indices for islice() must be None or an integer: 0 <= x <= sys.maxsize.");
      }
      ls.wrapped = this.ip.iterOf(args[0]);
      return ls;
    }
    if ( f == "takewhile" || f == "dropwhile" ) {
      const ltw = this.ip.mkLazy(f, f);
      ltw.cls = args[0];
      ltw.wrapped = this.ip.iterOf(this.ip.argAt(args, 1));
      return ltw;
    }
    if ( f == "filterfalse" ) {
      const lff = this.ip.mkLazy("filterfalse", "filterfalse");
      lff.cls = args[0];
      lff.wrapped = this.ip.iterOf(this.ip.argAt(args, 1));
      return lff;
    }
    if ( f == "__unused_chain" ) {
      const srcs = args;
      // Loop start
      for ( const s of srcs) {
        // Loop start
        for ( let j = 0; j < this.listOf(s).length; j++) {
          var v = this.listOf(s)[j];
          out.push(v);
        }
      }
      return this.ip.mkIter(out);
    }
    if ( f == "product" ) {
      let rep = 1;
      const rv = this.ip.bi.kw(kn, kv, "repeat");
      if ( (typeof(rv) !== "undefined" && rv != null )  ) {
        const tv3 = rv;
        rep = tv3.i;
      }
      let pools = [];
      let r = 0;
      while (r < rep) {
        // Loop start
        for ( const a of args) {
          pools.push(this.ip.mkList(this.listOf(a)));
        }
        r = r + 1;
      };
      let acc = [];
      acc.push(this.ip.mkTuple([]));
      // Loop start
      for ( const pool of pools) {
        let next = [];
        // Loop start
        for ( const t of acc) {
          // Loop start
          for ( let yi = 0; yi < pool.items.length; yi++) {
            var y = pool.items[yi];
            let items = [];
            // Loop start
            for ( let xi = 0; xi < t.items.length; xi++) {
              var x = t.items[xi];
              items.push(x);
            }
            items.push(y);
            next.push(this.ip.mkTuple(items));
          }
        }
        acc = next;
      }
      return this.ip.mkIter(acc);
    }
    if ( (f == "permutations" || f == "combinations") || f == "combinations_with_replacement" ) {
      const pool2 = this.listOf(args[0]);
      const np = pool2.length;
      let rr = np;
      const rv2 = this.ip.bi.arg(args, 1, kn, kv, "r");
      if ( rv2.t != 0 ) {
        rr = rv2.i;
      }
      let cur = [];
      let used = [];
      // Loop start
      for ( const pv of pool2) {
        used.push(false);
      }
      this.combos(f, pool2, rr, 0, cur, used, out);
      return this.ip.mkIter(out);
    }
    if ( f == "accumulate" ) {
      const fn1 = this.ip.bi.arg(args, 1, kn, kv, "func");
      let has = false;
      let total = this.ip.NONE;
      const iv = this.ip.bi.kw(kn, kv, "initial");
      const __REGx5 = this.ip.orNone(iv);
      if ( ((typeof(iv) !== "undefined" && iv != null ) ) && __REGx5.t != 0 ) {
        total = iv;
        has = true;
        out.push(total);
      }
      // Loop start
      for ( let j2 = 0; j2 < this.listOf(args[0]).length; j2++) {
        var v2 = this.listOf(args[0])[j2];
        if ( has ) {
          if ( fn1.t == 0 ) {
            total = this.ip.binop("+", total, v2);
          } else {
            total = this.call2(fn1, total, v2);
          }
        } else {
          total = v2;
          has = true;
        }
        out.push(total);
      }
      return this.ip.mkIter(out);
    }
    if ( f == "groupby" ) {
      const keyf = this.ip.bi.arg(args, 1, kn, kv, "key");
      let curKey = this.ip.NONE;
      let group = [];
      let started = false;
      // Loop start
      for ( let j3 = 0; j3 < this.listOf(args[0]).length; j3++) {
        var v3 = this.listOf(args[0])[j3];
        let k = v3;
        if ( keyf.t != 0 ) {
          k = this.call1(keyf, v3);
        }
        if ( started && this.ip.eq(k, curKey) ) {
          group.push(v3);
        } else {
          if ( started ) {
            out.push(this.ip.pair(curKey, this.ip.mkIter(group)));
          }
          let fresh = [];
          group = fresh;
          group.push(v3);
          curKey = k;
          started = true;
        }
      }
      if ( started ) {
        out.push(this.ip.pair(curKey, this.ip.mkIter(group)));
      }
      return this.ip.mkIter(out);
    }
    if ( f == "islice" ) {
      let start = 0;
      let stop = -1;
      let step = 1;
      if ( n == 2 ) {
        const __REGx6 = args[1];
        if ( __REGx6.t != 0 ) {
          const tv4 = args[1];
          stop = tv4.i;
        }
      } else {
        const __REGx7 = args[1];
        if ( __REGx7.t != 0 ) {
          const tv5 = args[1];
          start = tv5.i;
        }
        const __REGx8 = args[2];
        if ( __REGx8.t != 0 ) {
          const tv6 = args[2];
          stop = tv6.i;
        }
        const __REGx9 = this.ip.argAt(args, 3);
        if ( n > 3 && __REGx9.t != 0 ) {
          const tv7 = args[3];
          step = tv7.i;
        }
      }
      const src = args[0];
      const vals = this.takeN(src, stop);
      let i4 = start;
      while (i4 < vals.length) {
        if ( stop >= 0 && i4 >= stop ) {
          break;
        }
        out.push(vals[i4]);
        i4 = i4 + step;
      };
      return this.ip.mkIter(out);
    }
    if ( f == "zip_longest" ) {
      let fill = this.ip.NONE;
      const fv = this.ip.bi.kw(kn, kv, "fillvalue");
      if ( (typeof(fv) !== "undefined" && fv != null )  ) {
        fill = fv;
      }
      let lists = [];
      let longest = 0;
      // Loop start
      for ( const a2 of args) {
        const l = this.ip.mkList(this.listOf(a2));
        lists.push(l);
        if ( l.items.length > longest ) {
          longest = l.items.length;
        }
      }
      let i5 = 0;
      while (i5 < longest) {
        let row = [];
        // Loop start
        for ( const l2 of lists) {
          if ( i5 < l2.items.length ) {
            row.push(l2.items[i5]);
          } else {
            row.push(fill);
          }
        }
        out.push(this.ip.mkTuple(row));
        i5 = i5 + 1;
      };
      return this.ip.mkIter(out);
    }
    if ( f == "starmap" ) {
      const fn2 = args[0];
      // Loop start
      for ( let j4 = 0; j4 < this.listOf(args[1]).length; j4++) {
        var t2 = this.listOf(args[1])[j4];
        let kn2 = [];
        let kv2 = [];
        out.push(this.ip.callValue(fn2, this.listOf(t2), kn2, kv2));
      }
      return this.ip.mkIter(out);
    }
    if ( (f == "takewhile" || f == "dropwhile") || f == "filterfalse" ) {
      const pred = args[0];
      let dropping = true;
      // Loop start
      for ( let j5 = 0; j5 < this.listOf(args[1]).length; j5++) {
        var v4 = this.listOf(args[1])[j5];
        let ok = false;
        if ( pred.t == 0 ) {
          ok = this.ip.truthy(v4);
        } else {
          ok = this.ip.truthy(this.call1(pred, v4));
        }
        if ( f == "takewhile" ) {
          if ( ok == false ) {
            break;
          }
          out.push(v4);
        }
        if ( f == "dropwhile" ) {
          if ( dropping && ok ) {
            continue;
          }
          dropping = false;
          out.push(v4);
        }
        if ( f == "filterfalse" && ok == false ) {
          out.push(v4);
        }
      }
      return this.ip.mkIter(out);
    }
    if ( f == "pairwise" ) {
      const vs = this.listOf(args[0]);
      let i6 = 1;
      while (i6 < vs.length) {
        out.push(this.ip.pair(vs[(i6 - 1)], vs[i6]));
        i6 = i6 + 1;
      };
      return this.ip.mkIter(out);
    }
    if ( f == "batched" ) {
      const size = this.ip.bi.toInt(args[1], "n");
      if ( size < 1 ) {
        this.ip.raise("ValueError", "n must be at least one");
      }
      let batch = [];
      // Loop start
      for ( let j7 = 0; j7 < this.listOf(args[0]).length; j7++) {
        var v6 = this.listOf(args[0])[j7];
        batch.push(v6);
        if ( batch.length == size ) {
          out.push(this.ip.mkTuple(batch));
          let fresh2 = [];
          batch = fresh2;
        }
      }
      if ( batch.length > 0 ) {
        out.push(this.ip.mkTuple(batch));
      }
      return this.ip.mkIter(out);
    }
    if ( f == "compress" ) {
      const sel = this.listOf(args[1]);
      // Loop start
      for ( let j6 = 0; j6 < this.listOf(args[0]).length; j6++) {
        var v5 = this.listOf(args[0])[j6];
        if ( j6 < sel.length && this.ip.truthy(sel[j6]) ) {
          out.push(v5);
        }
      }
      return this.ip.mkIter(out);
    }
    if ( f == "repeat" ) {
      const times = this.ip.bi.arg(args, 1, kn, kv, "times");
      if ( times.t == 0 ) {
        const lr = this.ip.mkLazy("repeat", "repeat");
        lr.wrapped = args[0];
        lr.i = -1;
        return lr;
      }
      let i7 = 0;
      while (i7 < times.i) {
        out.push(args[0]);
        i7 = i7 + 1;
      };
      return this.ip.mkIter(out);
    }
    if ( f == "tee" ) {
      const vs2 = this.listOf(args[0]);
      let cnt = 2;
      if ( n > 1 ) {
        const tv8 = args[1];
        cnt = tv8.i;
      }
      let i8 = 0;
      while (i8 < cnt) {
        let copy = [];
        // Loop start
        for ( const v7 of vs2) {
          copy.push(v7);
        }
        out.push(this.ip.mkIter(copy));
        i8 = i8 + 1;
      };
      return this.ip.mkTuple(out);
    }
    this.ip.raise("NotImplementedError", ("itertools." + f) + "() is infinite; this runtime evaluates iterators eagerly");
    return this.ip.NONE;
  };
  takeN (src, stop) {
    if ( stop < 0 || src.t != 17 ) {
      return this.listOf(src);
    }
    let out = [];
    while (out.length < stop) {
      if ( src.pos >= src.items.length ) {
        break;
      }
      out.push(src.items[src.pos]);
      src.pos = src.pos + 1;
    };
    return out;
  };
  combos (kind, pool, r, start, cur, used, out) {
    if ( cur.length == r ) {
      let items = [];
      // Loop start
      for ( const ix of cur) {
        items.push(pool[ix]);
      }
      out.push(this.ip.mkTuple(items));
      return;
    }
    const n = pool.length;
    let i2 = 0;
    if ( kind != "permutations" ) {
      i2 = start;
    }
    while (i2 < n) {
      if ( kind == "permutations" && used[i2] ) {
        i2 = i2 + 1;
        continue;
      }
      cur.push(i2);
      used[i2] = true;
      if ( kind == "combinations" ) {
        this.combos(kind, pool, r, i2 + 1, cur, used, out);
      } else {
        this.combos(kind, pool, r, i2, cur, used, out);
      }
      used[i2] = false;
      cur.pop();
      i2 = i2 + 1;
    };
  };
  funcCall (f, args, kn, kv) {
    const n = args.length;
    if ( f == "reduce" ) {
      const fn1 = args[0];
      const vals = this.listOf(args[1]);
      let acc = this.ip.NONE;
      let i = 0;
      if ( n > 2 ) {
        acc = args[2];
      } else {
        if ( vals.length == 0 ) {
          this.ip.raise("TypeError", "reduce() of empty iterable with no initial value");
        }
        acc = vals[0];
        i = 1;
      }
      while (i < vals.length) {
        acc = this.call2(fn1, acc, vals[i]);
        i = i + 1;
      };
      return acc;
    }
    if ( f == "partial" ) {
      const p = new PyVal(10);
      p.s = "functools.partial";
      p.wrapped = args[0];
      let i2 = 1;
      while (i2 < n) {
        p.items.push(args[i2]);
        i2 = i2 + 1;
      };
      // Loop start
      for ( let ki = 0; ki < kn.length; ki++) {
        var k = kn[ki];
        p.putAttr(k, kv[ki]);
      }
      return p;
    }
    if ( f == "lru_cache" || f == "cache" ) {
      const __REGx1 = this.ip.argAt(args, 0);
      const __REGx2 = this.ip.argAt(args, 0);
      if ( (n > 0 && __REGx1.t != 0) && __REGx2.t != 2 ) {
        const w = new PyVal(10);
        w.s = "functools._lru_wrapper";
        w.wrapped = args[0];
        w.putAttr("__cache__", this.ip.mkDict());
        return w;
      }
      const deco = new PyVal(10);
      deco.s = "functools.lru_cache";
      return deco;
    }
    if ( f == "wraps" || f == "total_ordering" ) {
      if ( f == "total_ordering" ) {
        return args[0];
      }
      const idf = new PyVal(10);
      idf.s = "functools._identity";
      return idf;
    }
    if ( f == "_identity" ) {
      return args[0];
    }
    if ( f == "cmp_to_key" ) {
      const kf = new PyVal(10);
      kf.s = "functools._cmp_key";
      kf.wrapped = args[0];
      return kf;
    }
    if ( (f == "_cmp_lt" || f == "_cmp_gt") || f == "_cmp_eq" ) {
      const a = args[0];
      const b = args[1];
      const cmp = ( Object.prototype.hasOwnProperty.call(a.attrs, "cmp") ? a.attrs["cmp"] : undefined );
      const r = this.call2(
        cmp,
        ( Object.prototype.hasOwnProperty.call(a.attrs, "obj") ? a.attrs["obj"] : undefined ),
        ( Object.prototype.hasOwnProperty.call(b.attrs, "obj") ? b.attrs["obj"] : undefined )
      );
      const rv = this.ip.toDouble(r);
      if ( f == "_cmp_lt" ) {
        return this.ip.mkBool((rv < 0.0));
      }
      if ( f == "_cmp_gt" ) {
        return this.ip.mkBool((rv > 0.0));
      }
      return this.ip.mkBool((rv == 0.0));
    }
    this.ip.raise("AttributeError", ("module 'functools' has no attribute '" + f) + "'");
    return this.ip.NONE;
  };
  callWrapped (b, args, kn, kv) {
    const target = b.wrapped;
    if ( b.s == "functools.partial" ) {
      let all = [];
      // Loop start
      for ( const a of b.items) {
        all.push(a);
      }
      // Loop start
      for ( const a2 of args) {
        all.push(a2);
      }
      let kn2 = [];
      let kv2 = [];
      const ks = b.attrKeys();
      // Loop start
      for ( const k of ks) {
        if ( kn.indexOf(k) < 0 ) {
          kn2.push(k);
          kv2.push(( Object.prototype.hasOwnProperty.call(b.attrs, k) ? b.attrs[k] : undefined ));
        }
      }
      // Loop start
      for ( let ki2 = 0; ki2 < kn.length; ki2++) {
        var k2 = kn[ki2];
        kn2.push(k2);
        kv2.push(kv[ki2]);
      }
      return this.ip.callValue(target, all, kn2, kv2);
    }
    if ( b.s == "functools._lru_wrapper" ) {
      const cache = ( Object.prototype.hasOwnProperty.call(b.attrs, "__cache__") ? b.attrs["__cache__"] : undefined );
      let keyItems = [];
      // Loop start
      for ( const a3 of args) {
        keyItems.push(a3);
      }
      // Loop start
      for ( let ki3 = 0; ki3 < kn.length; ki3++) {
        var k3 = kn[ki3];
        keyItems.push(this.ip.mkStr(k3));
        keyItems.push(kv[ki3]);
      }
      const key = this.ip.mkTuple(keyItems);
      const hit = this.ip.dictGet(cache, key);
      if ( (typeof(hit) !== "undefined" && hit != null )  ) {
        return hit;
      }
      const r = this.ip.callValue(target, args, kn, kv);
      this.ip.dictSet(cache, key, r);
      return r;
    }
    if ( b.s == "functools._cmp_key" ) {
      const kc = this.cmpKeyClass();
      const o = new PyVal(12);
      o.cls = kc;
      o.putAttr("cmp", target);
      o.putAttr("obj", args[0]);
      return o;
    }
    if ( b.s == "collections._make" ) {
      return this.ntMake(target, this.listOf(args[0]), kn, kv);
    }
    this.ip.raise("TypeError", ("'" + b.s) + "' object is not callable");
    return this.ip.NONE;
  };
  cmpKeyClass () {
    const kc = ( Object.prototype.hasOwnProperty.call(this.ip.builtins, "__cmp_key_class__") ? this.ip.builtins["__cmp_key_class__"] : undefined );
    if ( (typeof(kc) !== "undefined" && kc != null )  ) {
      return kc;
    }
    const c = this.nativeClass("functools.KeyWrapper");
    c.mode = "builtin";
    const lt = new PyVal(10);
    lt.s = "functools._cmp_lt";
    c.putAttr("__lt__", lt);
    const gt = new PyVal(10);
    gt.s = "functools._cmp_gt";
    c.putAttr("__gt__", gt);
    const eq = new PyVal(10);
    eq.s = "functools._cmp_eq";
    c.putAttr("__eq__", eq);
    this.ip.builtins["__cmp_key_class__"] = c;
    return c;
  };
  statErr (msg) {
    const sm = this.make("statistics");
    this.ip.raiseVal(this.ip.makeExc(( Object.prototype.hasOwnProperty.call(sm.attrs, "StatisticsError") ? sm.attrs["StatisticsError"] : undefined ), msg));
  };
  statCall (f, args, kn, kv) {
    const data = this.listOf(args[0]);
    const n = data.length;
    let allInt = true;
    // Loop start
    for ( const v of data) {
      if ( this.ip.isIntLike(v) == false ) {
        allInt = false;
      }
    }
    if ( f == "mode" ) {
      if ( n == 0 ) {
        this.statErr("no mode for empty data");
      }
      const counts = this.ip.mkDict();
      this.counterFeed(counts, this.ip.mkList(data));
      let best = 0;
      let bestK = this.ip.NONE;
      // Loop start
      for ( let ki = 0; ki < counts.items.length; ki++) {
        var k = counts.items[ki];
        const c = counts.vals[ki].i;
        if ( c > best ) {
          best = c;
          bestK = k;
        }
      }
      return bestK;
    }
    if ( (f == "median" || f == "median_low") || f == "median_high" ) {
      if ( n == 0 ) {
        this.statErr("no median for empty data");
      }
      const s = this.ip.bi.sortVals(data, this.ip.NONE, false);
      const mid = ((n / 2) | 0);
      if ( n % 2 == 1 ) {
        return s[mid];
      }
      if ( f == "median_low" ) {
        return s[(mid - 1)];
      }
      if ( f == "median_high" ) {
        return s[mid];
      }
      return this.ip.binop(
        "/",
        this.ip.binop("+", s[(mid - 1)], s[mid]),
        this.ip.mkInt(2)
      );
    }
    if ( f == "mean" || f == "fmean" ) {
      if ( n == 0 ) {
        if ( f == "fmean" ) {
          this.statErr("fmean requires at least one data point");
        }
        this.statErr("mean requires at least one data point");
      }
      if ( allInt && f == "mean" ) {
        let tot = 0;
        // Loop start
        for ( const v2 of data) {
          tot = tot + v2.i;
        }
        if ( PyFormat.floorModInt(tot, n) == 0 ) {
          return this.ip.mkInt(PyFormat.floorDivInt(tot, n));
        }
        return this.ip.mkFloat(((tot) / (n)));
      }
      let ds = [];
      // Loop start
      for ( const v3 of data) {
        ds.push(this.ip.bi.toNum(v3));
      }
      return this.ip.mkFloat((this.fsumD(ds) / (n)));
    }
    const sample = f == "stdev" || f == "variance";
    let need = 1;
    if ( sample ) {
      need = 2;
    }
    if ( n < need ) {
      if ( sample ) {
        this.statErr(f + " requires at least two data points");
      }
      this.statErr(f + " requires at least one data point");
    }
    let den = n;
    if ( sample ) {
      den = n - 1;
    }
    if ( allInt ) {
      let sx = 0;
      let sxx = 0;
      // Loop start
      for ( const v4 of data) {
        sx = sx + v4.i;
        sxx = sxx + v4.i * v4.i;
      }
      const num = n * sxx - sx * sx;
      const dd = n * den;
      if ( f == "variance" || f == "pvariance" ) {
        if ( num % dd == 0 ) {
          return this.ip.mkInt(((num / dd) | 0));
        }
        return this.ip.mkFloat(((num) / (dd)));
      }
      return this.ip.mkFloat(Math.sqrt((num) / (dd)));
    }
    let xs = [];
    // Loop start
    for ( const v5 of data) {
      xs.push(this.ip.bi.toNum(v5));
    }
    const m = this.fsumD(xs) / (n);
    let sq = [];
    // Loop start
    for ( const x of xs) {
      sq.push((x - m) * (x - m));
    }
    const _var = this.fsumD(sq) / (den);
    if ( f == "variance" || f == "pvariance" ) {
      return this.ip.mkFloat(_var);
    }
    return this.ip.mkFloat(Math.sqrt(_var));
  };
  shallowCopy (v) {
    switch (v.t ) { 
      case 5 : 
        const l = this.ip.mkList(this.listOf(v));
        l.mode = v.mode;
        l.i3 = v.i3;
        this.copyAttrs(v, l);
        return l;
      case 7 : 
        const d = this.ip.copyDict(v);
        d.mode = v.mode;
        this.copyAttrs(v, d);
        return d;
      case 8 : 
        if ( v.mode == "frozen" ) {
          return v;
        }
        return this.ip.copySet(v);
      case 12 : 
        const o = new PyVal(12);
        o.cls = v.cls;
        this.copyAttrs(v, o);
        return o;
    };
    return v;
  };
  copyAttrs (src, dst) {
    const ks = src.attrKeys();
    // Loop start
    for ( const k of ks) {
      dst.putAttr(k, ( Object.prototype.hasOwnProperty.call(src.attrs, k) ? src.attrs[k] : undefined ));
    }
  };
  deepCopy (v) {
    switch (v.t ) { 
      case 5 : 
        let items = [];
        // Loop start
        for ( let i = 0; i < v.items.length; i++) {
          var x = v.items[i];
          items.push(this.deepCopy(x));
        }
        const l = this.ip.mkList(items);
        l.mode = v.mode;
        l.i3 = v.i3;
        this.copyAttrs(v, l);
        return l;
      case 6 : 
        let items2 = [];
        // Loop start
        for ( const x2 of v.items) {
          items2.push(this.deepCopy(x2));
        }
        const t = this.ip.mkTuple(items2);
        this.copyAttrs(v, t);
        return t;
      case 7 : 
        const d = this.ip.mkDict();
        d.mode = v.mode;
        this.copyAttrs(v, d);
        // Loop start
        for ( let ki = 0; ki < v.items.length; ki++) {
          var k = v.items[ki];
          this.ip.dictSet(d, k, this.deepCopy(v.vals[ki]));
        }
        return d;
      case 8 : 
        if ( v.mode == "frozen" ) {
          return v;
        }
        return this.ip.copySet(v);
      case 12 : 
        if ( this.ip.isExcInst(v) ) {
          return v;
        }
        const o = new PyVal(12);
        o.cls = v.cls;
        const ks = v.attrKeys();
        // Loop start
        for ( const k2 of ks) {
          o.putAttr(k2, this.deepCopy(( Object.prototype.hasOwnProperty.call(v.attrs, k2) ? v.attrs[k2] : undefined )));
        }
        return o;
    };
    return v;
  };
}
export class PyStdlib  {
}
PyStdlib.source = function(name) {
  let none;
  if ( name == "_datetime" ) {
    const s0 = "# SPDX-License-Identifier: MIT\n# _datetime: there is no C datetime here\ndatetime_CAPI = None\n";
    return s0;
  }
  if ( name == "_exceptiongroup" ) {
    const s1 = "# SPDX-License-Identifier: MIT\n# BaseExceptionGroup and ExceptionGroup for the Ranger Python runtime, and\n# the helpers the interpreter's except* calls. The builtins load this module\n# the first time either name is looked up.\n\n\ndef _matcher(cond):\n    if isinstance(cond, type) and issubclass(cond, BaseException):\n        return lambda e: isinstance(e, cond)\n    if isinstance(cond, tuple):\n        return lambda e: isinstance(e, cond)\n    if callable(cond):\n        return cond\n    raise TypeError(\"expected an exception type, a tuple of exception types, or a callable (other than a class)\")\n\n\nclass BaseExceptionGroup(BaseException):\n    def __new__(cls, message, exceptions):\n        if not isinstance(message, str):\n            raise TypeError(\"argument 1 must be str, not \" + type(message).__name__)\n        given = exceptions\n        exceptions = tuple(exceptions) if not isinstance(exceptions, str) else exceptions\n        if not isinstance(exceptions, tuple):\n            raise TypeError(\"second argument (exceptions) must be a sequence\")\n        if not exceptions:\n            raise ValueError(\"second argument (exceptions) must be a non-empty sequence\")\n        for e in exceptions:\n            if not isinstance(e, BaseException):\n                raise ValueError(\"Item %d of second argument (exceptions) is not an exception\" % exceptions.index(e))\n        if cls is BaseExceptionGroup and all(isinstance(e, Exception) for e in exceptions):\n            cls = ExceptionGroup\n        if cls is ExceptionGroup:\n            for e in exceptions:\n                if not isinstance(e, Exception):\n                    raise TypeError(\"Cannot nest BaseExceptions in an ExceptionGroup\")\n        self = object.__new__(cls)\n        self.args = (message, exceptions)\n        self._message = message\n        self._exceptions = exceptions\n        self._given = given\n        return self\n\n    def __init__(self, message, exceptions):\n        BaseException.__init__(self, message, tuple(exceptions))\n\n    @property\n    def message(self):\n        return self._message\n\n    @property\n    def exceptions(self):\n        return self._exceptions\n\n    def __str__(self):\n        n = len(self._exceptions)\n        return \"%s (%d sub-exception%s)\" % (self._message, n, \"\" if n == 1 else \"s\")\n\n    def __repr__(self):\n        return \"%s(%r, %r)\" % (type(self).__name__, self._message, self._given)\n\n    def derive(self, excs):\n        return BaseExceptionGroup(self._message, excs)\n\n    def _copy_meta(self, eg):\n        eg.__cause__ = getattr(self, \"__cause__\", None)\n        eg.__context__ = getattr(self, \"__context__\", None)\n        eg.__traceback__ = getattr(self, \"__traceback__\", None)\n        notes = getattr(self, \"__notes__\", None)\n        if notes is not None:\n            eg.__notes__ = list(notes)\n        return eg\n\n    def split(self, condition):\n        match = _matcher(condition)\n        if match(self):\n            return self, None\n        hit, miss = [], []\n        for e in self._exceptions:\n            if isinstance(e, BaseExceptionGroup):\n                a, b = e.split(match)\n                if a is not None:\n                    hit.append(a)\n                if b is not None:\n                    miss.append(b)\n            elif match(e):\n                hit.append(e)\n            else:\n                miss.append(e)\n        m = self._copy_meta(self.derive(tuple(hit))) if hit else None\n        r = self._copy_meta(self.derive(tuple(miss))) if miss else None\n        return m, r\n\n    def subgroup(self, condition):\n        return self.split(condition)[0]\n\n\nclass ExceptionGroup(BaseExceptionGroup, Exception):\n    pass\n\n\ndef _as_group(exc):\n    # a plain exception caught by except* is handled as a group of one\n    if isinstance(exc, BaseExceptionGroup):\n        return exc\n    return BaseExceptionGroup(\"\", (exc,))\n\n\ndef _finish(original, rest, raised):\n    # what propagates once the except* clauses have run: None when all of it\n    # was handled, else the original exception, the unhandled part, and what\n    # the handlers raised\n    if rest is None and not raised:\n        return None\n    if not isinstance(original, BaseExceptionGroup):\n        if not raised:\n            return original\n        if rest is not None:\n            raised = [original] + raised\n    elif rest is not None:\n        if not raised:\n            return rest\n        raised = [rest] + raised\n    if len(raised) == 1:\n        return raised[0]\n    return BaseExceptionGroup(\"\", raised)\n";
    return s1;
  }
  if ( name == "abc" ) {
    const s2 = "# SPDX-License-Identifier: MIT\n# abc for the Ranger Python runtime: ABCMeta keeps the abstract methods a\n# class leaves unimplemented (the runtime refuses to instantiate it) and\n# answers isinstance()/issubclass() for registered classes.\n\n\ndef abstractmethod(funcobj):\n    funcobj.__isabstractmethod__ = True\n    return funcobj\n\n\ndef abstractclassmethod(callable):\n    callable.__isabstractmethod__ = True\n    return classmethod(callable)\n\n\ndef abstractstaticmethod(callable):\n    callable.__isabstractmethod__ = True\n    return staticmethod(callable)\n\n\ndef abstractproperty(fget=None, fset=None, fdel=None, doc=None):\n    if fget is not None:\n        fget.__isabstractmethod__ = True\n    return property(fget, fset, fdel, doc)\n\n\ndef _is_abstract(value):\n    return bool(getattr(value, \"__isabstractmethod__\", False))\n\n\nclass ABCMeta(type):\n    def __new__(mcls, name, bases, namespace, /, **kwargs):\n        cls = super().__new__(mcls, name, bases, namespace, **kwargs)\n        abstracts = {key for key, value in namespace.items() if _is_abstract(value)}\n        for base in bases:\n            for key in getattr(base, \"__abstractmethods__\", ()):\n                if _is_abstract(getattr(cls, key, None)):\n                    abstracts.add(key)\n        cls.__abstractmethods__ = frozenset(abstracts)\n        cls._abc_registry = []\n        return cls\n\n    def register(cls, subclass):\n        if not isinstance(subclass, type):\n            raise TypeError(\"Can only register classes\")\n        if issubclass(subclass, cls):\n            return subclass\n        if issubclass(cls, subclass):\n            raise RuntimeError(\"Refusing to create an inheritance cycle\")\n        cls._abc_registry.append(subclass)\n        return subclass\n\n    def __instancecheck__(cls, instance):\n        sub = instance.__class__ if hasattr(instance, \"__class__\") else type(instance)\n        if cls.__subclasscheck__(sub):\n            return True\n        t = type(instance)\n        return t is not sub and cls.__subclasscheck__(t)\n\n    def __subclasscheck__(cls, subclass):\n        if not isinstance(subclass, type):\n            raise TypeError(\"issubclass() arg 1 must be a class\")\n        mro = getattr(subclass, \"__mro__\", (subclass,))\n        if cls in mro:\n            return True\n        hook = cls.__dict__.get(\"__subclasshook__\")\n        if hook is None:\n            for base in cls.__mro__:\n                if \"__subclasshook__\" in base.__dict__:\n                    hook = base.__dict__[\"__subclasshook__\"]\n                    break\n        if hook is not None:\n            fn = getattr(cls, \"__subclasshook__\")\n            ok = fn(subclass)\n            if ok is not NotImplemented:\n                return bool(ok)\n        for rcls in cls.__dict__.get(\"_abc_registry\", ()):\n            if rcls is subclass or issubclass(subclass, rcls):\n                return True\n        for scls in _subclasses(cls):\n            if scls is not cls and issubclass(subclass, scls):\n                return True\n        return False\n\n    def _dump_registry(cls, file=None):\n        print(\"Class: %s.%s\" % (cls.__module__, cls.__qualname__), file=file)\n\n    def _abc_registry_clear(cls):\n        cls._abc_registry = []\n\n    def _abc_caches_clear(cls):\n        pass\n\n\n_all_abcs = []\n\n\ndef _subclasses(cls):\n    return [c for c in _all_abcs if c is not cls and cls in c.__mro__]\n\n\n_orig_new = ABCMeta.__new__\n\n\ndef _tracking_new(mcls, name, bases, namespace, /, **kwargs):\n    cls = _orig_new(mcls, name, bases, namespace, **kwargs)\n    _all_abcs.append(cls)\n    return cls\n\n\nABCMeta.__new__ = _tracking_new\n\n\nclass ABC(metaclass=ABCMeta):\n    __slots__ = ()\n\n\ndef get_cache_token():\n    return 0\n\n\ndef update_abstractmethods(cls):\n    if not hasattr(cls, \"__abstractmethods__\"):\n        return cls\n    abstracts = set()\n    for scls in cls.__bases__:\n        for name in getattr(scls, \"__abstractmethods__\", ()):\n            if _is_abstract(getattr(cls, name, None)):\n                abstracts.add(name)\n    for name, value in cls.__dict__.items():\n        if _is_abstract(value):\n            abstracts.add(name)\n    cls.__abstractmethods__ = frozenset(abstracts)\n    return cls\n";
    return s2;
  }
  if ( name == "array" ) {
    const s3 = "# SPDX-License-Identifier: MIT\n# array for the Ranger Python runtime: typed arrays kept as lists\n\ntypecodes = \"bBuwhHiIlLqQfd\"\n_SIZES = {\"b\": 1, \"B\": 1, \"u\": 4, \"w\": 4, \"h\": 2, \"H\": 2, \"i\": 4, \"I\": 4, \"l\": 8, \"L\": 8,\n          \"q\": 8, \"Q\": 8, \"f\": 4, \"d\": 8}\n_SIGNED = \"bhilq\"\n\n\ndef _check(tc, v):\n    if tc in \"fd\":\n        if not isinstance(v, (int, float)):\n            raise TypeError(\"must be real number, not %s\" % type(v).__name__)\n        return float(v)\n    if tc in \"uw\":\n        if not isinstance(v, str) or len(v) != 1:\n            raise TypeError(\"array item must be a unicode character, not %s\" % type(v).__name__)\n        return v\n    if not isinstance(v, int):\n        raise TypeError(\"'%s' object cannot be interpreted as an integer\" % type(v).__name__)\n    bits = 8 * _SIZES[tc]\n    if tc in _SIGNED:\n        lo, hi = -(1 << (bits - 1)), (1 << (bits - 1)) - 1\n    else:\n        lo, hi = 0, (1 << bits) - 1\n    if v < lo or v > hi:\n        if tc in _SIGNED:\n            raise OverflowError(\"signed %s is %s than %s\" % (\n                {\"b\": \"char\", \"h\": \"short integer\", \"i\": \"integer\", \"l\": \"integer\", \"q\": \"integer\"}[tc],\n                \"less\" if v < lo else \"greater\", \"minimum\" if v < lo else \"maximum\"))\n        raise OverflowError(\"unsigned %s is %s than %s\" % (\n            {\"B\": \"byte integer\", \"H\": \"short\", \"I\": \"int\", \"L\": \"int\", \"Q\": \"int\"}[tc],\n            \"less\" if v < lo else \"greater\", \"minimum\" if v < lo else \"maximum\"))\n    return v\n\n\nclass array:\n    def __init__(self, typecode, initializer=None):\n        if not isinstance(typecode, str) or len(typecode) != 1 or typecode not in typecodes:\n            raise ValueError(\"bad typecode (must be b, B, u, w, h, H, i, I, l, L, q, Q, f or d)\")\n        self.typecode = typecode\n        self.itemsize = _SIZES[typecode]\n        self._a = []\n        if initializer is not None:\n            if isinstance(initializer, str) and typecode not in \"uw\":\n                raise TypeError(\"cannot use a str to initialize an array with typecode '%s'\" % typecode)\n            self.extend(initializer)\n\n    def __class_getitem__(cls, item):\n        import types\n        return types.GenericAlias(cls, item)\n\n    def append(self, v):\n        self._a.append(_check(self.typecode, v))\n\n    def extend(self, it):\n        if isinstance(it, array) and it.typecode != self.typecode:\n            raise TypeError(\"can only extend with array of same kind\")\n        for v in it:\n            self.append(v)\n\n    def fromlist(self, lst):\n        if not isinstance(lst, list):\n            raise TypeError(\"arg must be list\")\n        vals = [_check(self.typecode, v) for v in lst]\n        self._a.extend(vals)\n\n    def fromunicode(self, s):\n        if self.typecode not in \"uw\":\n            raise ValueError(\"fromunicode() may only be called on unicode type arrays\")\n        self._a.extend(s)\n\n    def tolist(self):\n        return list(self._a)\n\n    def tounicode(self):\n        if self.typecode not in \"uw\":\n            raise ValueError(\"tounicode() may only be called on unicode type arrays\")\n        return \"\".join(self._a)\n\n    def buffer_info(self):\n        return (id(self), len(self._a))\n\n    def count(self, v):\n        return self._a.count(v)\n\n    def index(self, v, start=0, stop=None):\n        try:\n            return self._a.index(v, start, len(self._a) if stop is None else stop)\n        except ValueError:\n            raise ValueError(\"array.index(x): x not in array\")\n\n    def insert(self, i, v):\n        self._a.insert(i, _check(self.typecode, v))\n\n    def pop(self, i=-1):\n        if not self._a:\n            raise IndexError(\"pop from empty array\")\n        return self._a.pop(i)\n\n    def remove(self, v):\n        try:\n            self._a.remove(v)\n        except ValueError:\n            raise ValueError(\"array.remove(x): x not in array\")\n\n    def reverse(self):\n        self._a.reverse()\n\n    def byteswap(self):\n        pass\n\n    def __len__(self):\n        return len(self._a)\n\n    def __iter__(self):\n        return iter(self._a)\n\n    def __reversed__(self):\n        return reversed(self._a)\n\n    def __contains__(self, v):\n        return v in self._a\n\n    def __getitem__(self, i):\n        if isinstance(i, slice):\n            r = array(self.typecode)\n            r._a = self._a[i]\n            return r\n        try:\n            return self._a[i]\n        except IndexError:\n            raise IndexError(\"array index out of range\")\n\n    def __setitem__(self, i, v):\n        if isinstance(i, slice):\n            if not isinstance(v, array):\n                raise TypeError(\"can only assign array (not \\\"%s\\\") to array slice\" % type(v).__name__)\n            self._a[i] = v._a\n            return\n        try:\n            self._a[i] = _check(self.typecode, v)\n        except IndexError:\n            raise IndexError(\"array assignment index out of range\")\n\n    def __delitem__(self, i):\n        del self._a[i]\n\n    def __add__(self, other):\n        if not isinstance(other, array):\n            return NotImplemented\n        if other.typecode != self.typecode:\n            raise TypeError(\"bad argument type for built-in operation\")\n        r = array(self.typecode)\n        r._a = self._a + other._a\n        return r\n\n    def __iadd__(self, other):\n        if not isinstance(other, array):\n            return NotImplemented\n        self.extend(other)\n        return self\n\n    def __mul__(self, n):\n        r = array(self.typecode)\n        r._a = self._a * n\n        return r\n\n    __rmul__ = __mul__\n\n    def __imul__(self, n):\n        self._a *= n\n        return self\n\n    def __eq__(self, other):\n        if not isinstance(other, array):\n            return NotImplemented\n        return self._a == other._a\n\n    def __ne__(self, other):\n        if not isinstance(other, array):\n            return NotImplemented\n        return self._a != other._a\n\n    def __lt__(self, other):\n        return self._a < other._a\n\n    def __le__(self, other):\n        return self._a <= other._a\n\n    def __gt__(self, other):\n        return self._a > other._a\n\n    def __ge__(self, other):\n        return self._a >= other._a\n\n    __hash__ = None\n\n    def __copy__(self):\n        return array(self.typecode, self._a)\n\n    def __deepcopy__(self, memo):\n        return array(self.typecode, self._a)\n\n    def __reduce_ex__(self, proto):\n        return (array, (self.typecode, list(self._a)))\n\n    def __repr__(self):\n        if not self._a:\n            return \"array('%s')\" % self.typecode\n        if self.typecode in \"uw\":\n            return \"array('%s', %r)\" % (self.typecode, \"\".join(self._a))\n        return \"array('%s', %r)\" % (self.typecode, self._a)\n\n\nArrayType = array\n";
    return s3;
  }
  if ( name == "ast" ) {
    const s4 = "# SPDX-License-Identifier: MIT\n# ast for the Ranger Python runtime. There is no Python-level syntax tree\n# here: literal_eval works, parse() raises NotImplementedError.\n\nPyCF_ONLY_AST = 1024\nPyCF_TYPE_COMMENTS = 4096\nPyCF_ALLOW_TOP_LEVEL_AWAIT = 8192\nPyCF_OPTIMIZED_AST = 33792\n\n\nclass AST:\n    _fields = ()\n    _attributes = ()\n\n    def __init__(self, *args, **kwargs):\n        for name, value in zip(self._fields, args):\n            setattr(self, name, value)\n        for name, value in kwargs.items():\n            setattr(self, name, value)\n\n\ndef _node(name, fields=()):\n    return type(name, (AST,), {\"_fields\": fields})\n\n\nfor _name, _fields in [(\"Module\", (\"body\", \"type_ignores\")), (\"Expression\", (\"body\",)),\n                       (\"Interactive\", (\"body\",)), (\"expr\", ()), (\"stmt\", ()), (\"mod\", ()),\n                       (\"Constant\", (\"value\", \"kind\")), (\"Name\", (\"id\", \"ctx\")), (\"Load\", ()),\n                       (\"Store\", ()), (\"Del\", ()), (\"Expr\", (\"value\",)), (\"BinOp\", (\"left\", \"op\", \"right\")),\n                       (\"Add\", ()), (\"Sub\", ()), (\"Mult\", ()), (\"Div\", ()), (\"UnaryOp\", (\"op\", \"operand\")),\n                       (\"USub\", ()), (\"UAdd\", ()), (\"Not\", ()), (\"Invert\", ()), (\"Call\", (\"func\", \"args\", \"keywords\")),\n                       (\"Attribute\", (\"value\", \"attr\", \"ctx\")), (\"Assign\", (\"targets\", \"value\", \"type_comment\")),\n                       (\"FunctionDef\", (\"name\", \"args\", \"body\", \"decorator_list\", \"returns\", \"type_comment\")),\n                       (\"ClassDef\", (\"name\", \"bases\", \"keywords\", \"body\", \"decorator_list\")),\n                       (\"Return\", (\"value\",)), (\"If\", (\"test\", \"body\", \"orelse\")), (\"Pass\", ()),\n                       (\"List\", (\"elts\", \"ctx\")), (\"Tuple\", (\"elts\", \"ctx\")), (\"Dict\", (\"keys\", \"values\")),\n                       (\"Set\", (\"elts\",)), (\"arguments\", ()), (\"arg\", (\"arg\", \"annotation\")), (\"keyword\", (\"arg\", \"value\"))]:\n    globals()[_name] = _node(_name, _fields)\n\n\ndef parse(source, filename=\"<unknown>\", mode=\"exec\", *, type_comments=False, feature_version=None, optimize=-1):\n    raise NotImplementedError(\"ast.parse is not available in this runtime\")\n\n\ndef dump(node, annotate_fields=True, include_attributes=False, *, indent=None, show_empty=False):\n    return repr(node)\n\n\ndef _check_literal(src):\n    allowed = set(\"0123456789abcdefABCDEFxXoOjJ.eE+-_ \\t\\n()[]{},:'\\\"\\\\\")\n    i = 0\n    n = len(src)\n    while i < n:\n        c = src[i]\n        if c in \"'\\\"\":\n            q = c\n            triple = src.startswith(q * 3, i)\n            end = q * 3 if triple else q\n            i += len(end)\n            while i < n and not src.startswith(end, i):\n                i += 2 if src[i] == \"\\\\\" else 1\n            i += len(end)\n            continue\n        if c.isalpha():\n            j = i\n            while j < n and (src[j].isalnum() or src[j] == \"_\"):\n                j += 1\n            word = src[i:j]\n            prev = src[i - 1] if i else \"\"\n            if word in (\"True\", \"False\", \"None\") or (prev.isdigit() or prev == \".\"):\n                i = j\n                continue\n            if word.lower() in (\"b\", \"r\", \"u\", \"br\", \"rb\") and j < n and src[j] in \"'\\\"\":\n                i = j\n                continue\n            if word in (\"set\",) and src.startswith(\"()\", j):\n                i = j + 2\n                continue\n            raise ValueError(\"malformed node or string: %r\" % src)\n        if c not in allowed:\n            raise ValueError(\"malformed node or string: %r\" % src)\n        i += 1\n\n\ndef literal_eval(node_or_string):\n    if not isinstance(node_or_string, str):\n        raise ValueError(\"malformed node or string: %r\" % (node_or_string,))\n    src = node_or_string.strip(\" \\t\")\n    _check_literal(src)\n    return eval(src, {\"__builtins__\": {\"set\": set}}, {})\n\n\ndef unparse(node):\n    raise NotImplementedError(\"ast.unparse is not available in this runtime\")\n\n\nclass NodeVisitor:\n    def visit(self, node):\n        return getattr(self, \"visit_\" + type(node).__name__, self.generic_visit)(node)\n\n    def generic_visit(self, node):\n        pass\n\n\nclass NodeTransformer(NodeVisitor):\n    pass\n\n\ndef walk(node):\n    return iter([node])\n\n\ndef get_docstring(node, clean=True):\n    return None\n";
    return s4;
  }
  if ( name == "bisect" ) {
    const s5 = "# SPDX-License-Identifier: MIT\n# bisect for the Ranger Python runtime\n\n\ndef bisect_right(a, x, lo=0, hi=None, *, key=None):\n    if lo < 0:\n        raise ValueError(\"lo must be non-negative\")\n    if hi is None:\n        hi = len(a)\n    while lo < hi:\n        mid = (lo + hi) // 2\n        v = a[mid] if key is None else key(a[mid])\n        if x < v:\n            hi = mid\n        else:\n            lo = mid + 1\n    return lo\n\n\ndef bisect_left(a, x, lo=0, hi=None, *, key=None):\n    if lo < 0:\n        raise ValueError(\"lo must be non-negative\")\n    if hi is None:\n        hi = len(a)\n    while lo < hi:\n        mid = (lo + hi) // 2\n        v = a[mid] if key is None else key(a[mid])\n        if v < x:\n            lo = mid + 1\n        else:\n            hi = mid\n    return lo\n\n\ndef insort_right(a, x, lo=0, hi=None, *, key=None):\n    lo = bisect_right(a, x if key is None else key(x), lo, hi, key=key)\n    a.insert(lo, x)\n\n\ndef insort_left(a, x, lo=0, hi=None, *, key=None):\n    lo = bisect_left(a, x if key is None else key(x), lo, hi, key=key)\n    a.insert(lo, x)\n\n\nbisect = bisect_right\ninsort = insort_right\n";
    return s5;
  }
  if ( name == "codecs" ) {
    const s6 = "# SPDX-License-Identifier: MIT\n# codecs for the Ranger Python runtime. Strings are the only text type, so\n# encoding and decoding pass text through after checking the name.\nBOM_UTF8 = \"\\xef\\xbb\\xbf\"\nBOM_LE = BOM_UTF16_LE = \"\\xff\\xfe\"\nBOM_BE = BOM_UTF16_BE = \"\\xfe\\xff\"\nBOM_UTF32_LE = \"\\xff\\xfe\\x00\\x00\"\nBOM_UTF32_BE = \"\\x00\\x00\\xfe\\xff\"\nBOM = BOM_UTF16 = BOM_LE\nBOM_UTF32 = BOM_UTF32_LE\n\n_NAMES = {\"utf8\": \"utf-8\", \"utf-8\": \"utf-8\", \"u8\": \"utf-8\", \"utf\": \"utf-8\", \"latin1\": \"iso8859-1\",\n          \"latin-1\": \"iso8859-1\", \"iso-8859-1\": \"iso8859-1\", \"iso8859-1\": \"iso8859-1\", \"l1\": \"iso8859-1\",\n          \"ascii\": \"ascii\", \"us-ascii\": \"ascii\", \"utf-16\": \"utf-16\", \"utf16\": \"utf-16\", \"utf-32\": \"utf-32\",\n          \"cp1252\": \"cp1252\", \"unicode_escape\": \"unicode-escape\", \"unicode-escape\": \"unicode-escape\",\n          \"raw_unicode_escape\": \"raw-unicode-escape\", \"utf-8-sig\": \"utf-8-sig\", \"utf_8\": \"utf-8\"}\n_errors = {}\n\n\nclass CodecInfo(tuple):\n    def __new__(cls, encode, decode, streamreader=None, streamwriter=None, incrementalencoder=None,\n                incrementaldecoder=None, name=None, *, _is_text_encoding=None):\n        self = tuple.__new__(cls, (encode, decode, streamreader, streamwriter))\n        self.name = name\n        self.encode = encode\n        self.decode = decode\n        self.incrementalencoder = incrementalencoder\n        self.incrementaldecoder = incrementaldecoder\n        self.streamreader = streamreader\n        self.streamwriter = streamwriter\n        return self\n\n    def __repr__(self):\n        return \"<codecs.CodecInfo object for encoding %s at %#x>\" % (self.name, id(self))\n\n\ndef _norm(encoding):\n    key = encoding.lower().replace(\"_\", \"-\") if isinstance(encoding, str) else encoding\n    name = _NAMES.get(key) or _NAMES.get(encoding.lower())\n    if name is None:\n        raise LookupError(\"unknown encoding: %s\" % encoding)\n    return name\n\n\ndef lookup(encoding):\n    name = _norm(encoding)\n    return CodecInfo(lambda s, errors=\"strict\": (encode(s, name, errors), len(s)),\n                     lambda s, errors=\"strict\": (decode(s, name, errors), len(s)), name=name)\n\n\ndef encode(obj, encoding=\"utf-8\", errors=\"strict\"):\n    _norm(encoding)\n    return obj.encode(encoding, errors) if hasattr(obj, \"encode\") else obj\n\n\ndef decode(obj, encoding=\"utf-8\", errors=\"strict\"):\n    _norm(encoding)\n    return obj.decode(encoding, errors) if hasattr(obj, \"decode\") else obj\n\n\ndef getencoder(encoding):\n    return lookup(encoding).encode\n\n\ndef getdecoder(encoding):\n    return lookup(encoding).decode\n\n\ndef register(search_function):\n    pass\n\n\ndef unregister(search_function):\n    pass\n\n\ndef register_error(name, handler):\n    _errors[name] = handler\n\n\ndef lookup_error(name):\n    if name in _errors:\n        return _errors[name]\n    if name in (\"strict\", \"ignore\", \"replace\", \"backslashreplace\", \"xmlcharrefreplace\", \"namereplace\", \"surrogateescape\", \"surrogatepass\"):\n        return lambda exc: (\"\", exc.end)\n    raise LookupError(\"unknown error handler name %r\" % name)\n\n\ndef open(filename, mode=\"r\", encoding=None, errors=\"strict\", buffering=-1):\n    import builtins\n    return builtins.open(filename, mode)\n\n\ndef strict_errors(exc):\n    raise exc\n\n\ndef ignore_errors(exc):\n    return (\"\", exc.end)\n\n\ndef replace_errors(exc):\n    return (\"?\", exc.end)\n\n\ndef backslashreplace_errors(exc):\n    return (\"\\\\x%02x\" % ord(exc.object[exc.start]), exc.end)\n\n\nclass Codec:\n    def encode(self, input, errors=\"strict\"):\n        raise NotImplementedError\n\n    def decode(self, input, errors=\"strict\"):\n        raise NotImplementedError\n\n\nclass IncrementalEncoder:\n    def __init__(self, errors=\"strict\"):\n        self.errors = errors\n\n    def encode(self, input, final=False):\n        return input\n\n    def reset(self):\n        pass\n\n\nclass IncrementalDecoder:\n    def __init__(self, errors=\"strict\"):\n        self.errors = errors\n\n    def decode(self, input, final=False):\n        return input\n\n    def reset(self):\n        pass\n\n\ndef getincrementaldecoder(encoding):\n    _norm(encoding)\n    return IncrementalDecoder\n\n\ndef getincrementalencoder(encoding):\n    _norm(encoding)\n    return IncrementalEncoder\n";
    return s6;
  }
  if ( name == "collections" ) {
    const s7 = "# SPDX-License-Identifier: MIT\n# collections for the Ranger Python runtime: deque, defaultdict, Counter,\n# OrderedDict and namedtuple are native (_collections); the rest is here.\nfrom _collections import deque, defaultdict, Counter, OrderedDict, namedtuple\n\n\nclass UserDict:\n    def __init__(self, dict=None, /, **kwargs):\n        self.data = {}\n        if dict is not None:\n            self.update(dict)\n        if kwargs:\n            self.update(kwargs)\n\n    def __len__(self):\n        return len(self.data)\n\n    def __getitem__(self, key):\n        if key in self.data:\n            return self.data[key]\n        if hasattr(self.__class__, \"__missing__\"):\n            return self.__class__.__missing__(self, key)\n        raise KeyError(key)\n\n    def __setitem__(self, key, item):\n        self.data[key] = item\n\n    def __delitem__(self, key):\n        del self.data[key]\n\n    def __iter__(self):\n        return iter(self.data)\n\n    def __contains__(self, key):\n        return key in self.data\n\n    def __repr__(self):\n        return repr(self.data)\n\n    def __eq__(self, other):\n        if isinstance(other, UserDict):\n            return self.data == other.data\n        return self.data == other\n\n    def __or__(self, other):\n        if isinstance(other, UserDict):\n            return self.__class__(self.data | other.data)\n        if isinstance(other, dict):\n            return self.__class__(self.data | other)\n        return NotImplemented\n\n    def __ror__(self, other):\n        if isinstance(other, UserDict):\n            return self.__class__(other.data | self.data)\n        if isinstance(other, dict):\n            return self.__class__(other | self.data)\n        return NotImplemented\n\n    def __ior__(self, other):\n        if isinstance(other, UserDict):\n            self.data |= other.data\n        else:\n            self.data |= other\n        return self\n\n    def __copy__(self):\n        inst = self.__class__.__new__(self.__class__)\n        inst.__dict__.update(self.__dict__)\n        inst.__dict__[\"data\"] = self.__dict__[\"data\"].copy()\n        return inst\n\n    def copy(self):\n        if self.__class__ is UserDict:\n            return UserDict(self.data.copy())\n        import copy\n        data = self.data\n        try:\n            self.data = {}\n            c = copy.copy(self)\n        finally:\n            self.data = data\n        c.update(self)\n        return c\n\n    def keys(self):\n        return self.data.keys()\n\n    def values(self):\n        return self.data.values()\n\n    def items(self):\n        return self.data.items()\n\n    def get(self, key, default=None):\n        if key in self:\n            return self[key]\n        return default\n\n    def pop(self, key, *default):\n        if key in self.data:\n            v = self.data[key]\n            del self.data[key]\n            return v\n        if default:\n            return default[0]\n        raise KeyError(key)\n\n    def popitem(self):\n        if not self.data:\n            raise KeyError(\"popitem(): dictionary is empty\")\n        k = list(self.data)[-1]\n        return (k, self.pop(k))\n\n    def clear(self):\n        self.data.clear()\n\n    def setdefault(self, key, default=None):\n        if key not in self:\n            self[key] = default\n        return self[key]\n\n    def update(self, other=(), /, **kwds):\n        if hasattr(other, \"keys\"):\n            for key in other.keys():\n                self[key] = other[key]\n        else:\n            for key, value in other:\n                self[key] = value\n        for key, value in kwds.items():\n            self[key] = value\n\n    @classmethod\n    def fromkeys(cls, iterable, value=None):\n        d = cls()\n        for key in iterable:\n            d[key] = value\n        return d\n\n\nclass UserList:\n    def __init__(self, initlist=None):\n        self.data = []\n        if initlist is not None:\n            if isinstance(initlist, UserList):\n                self.data[:] = initlist.data[:]\n            else:\n                self.data = list(initlist)\n\n    def __repr__(self):\n        return repr(self.data)\n\n    def __lt__(self, other):\n        return self.data < self.__cast(other)\n\n    def __le__(self, other):\n        return self.data <= self.__cast(other)\n\n    def __eq__(self, other):\n        return self.data == self.__cast(other)\n\n    def __gt__(self, other):\n        return self.data > self.__cast(other)\n\n    def __ge__(self, other):\n        return self.data >= self.__cast(other)\n\n    def __cast(self, other):\n        return other.data if isinstance(other, UserList) else other\n\n    def __contains__(self, item):\n        return item in self.data\n\n    def __len__(self):\n        return len(self.data)\n\n    def __getitem__(self, i):\n        if isinstance(i, slice):\n            return self.__class__(self.data[i])\n        return self.data[i]\n\n    def __setitem__(self, i, item):\n        self.data[i] = item\n\n    def __delitem__(self, i):\n        del self.data[i]\n\n    def __iter__(self):\n        return iter(self.data)\n\n    def __add__(self, other):\n        if isinstance(other, UserList):\n            return self.__class__(self.data + other.data)\n        elif isinstance(other, type(self.data)):\n            return self.__class__(self.data + other)\n        return self.__class__(self.data + list(other))\n\n    def __radd__(self, other):\n        if isinstance(other, UserList):\n            return self.__class__(other.data + self.data)\n        elif isinstance(other, type(self.data)):\n            return self.__class__(other + self.data)\n        return self.__class__(list(other) + self.data)\n\n    def __iadd__(self, other):\n        if isinstance(other, UserList):\n            self.data += other.data\n        elif isinstance(other, type(self.data)):\n            self.data += other\n        else:\n            self.data += list(other)\n        return self\n\n    def __mul__(self, n):\n        return self.__class__(self.data * n)\n\n    __rmul__ = __mul__\n\n    def __imul__(self, n):\n        self.data *= n\n        return self\n\n    def __copy__(self):\n        inst = self.__class__.__new__(self.__class__)\n        inst.__dict__.update(self.__dict__)\n        inst.__dict__[\"data\"] = self.__dict__[\"data\"][:]\n        return inst\n\n    def append(self, item):\n        self.data.append(item)\n\n    def insert(self, i, item):\n        self.data.insert(i, item)\n\n    def pop(self, i=-1):\n        return self.data.pop(i)\n\n    def remove(self, item):\n        self.data.remove(item)\n\n    def clear(self):\n        self.data.clear()\n\n    def copy(self):\n        return self.__class__(self)\n\n    def count(self, item):\n        return self.data.count(item)\n\n    def index(self, item, *args):\n        return self.data.index(item, *args)\n\n    def reverse(self):\n        self.data.reverse()\n\n    def sort(self, /, *args, **kwds):\n        self.data.sort(*args, **kwds)\n\n    def extend(self, other):\n        if isinstance(other, UserList):\n            self.data.extend(other.data)\n        else:\n            self.data.extend(other)\n\n\nclass UserString:\n    def __init__(self, seq):\n        if isinstance(seq, str):\n            self.data = seq\n        elif isinstance(seq, UserString):\n            self.data = seq.data[:]\n        else:\n            self.data = str(seq)\n\n    def __str__(self):\n        return str(self.data)\n\n    def __repr__(self):\n        return repr(self.data)\n\n    def __int__(self):\n        return int(self.data)\n\n    def __float__(self):\n        return float(self.data)\n\n    def __hash__(self):\n        return hash(self.data)\n\n    def __eq__(self, string):\n        if isinstance(string, UserString):\n            return self.data == string.data\n        return self.data == string\n\n    def __lt__(self, string):\n        if isinstance(string, UserString):\n            return self.data < string.data\n        return self.data < string\n\n    def __le__(self, string):\n        if isinstance(string, UserString):\n            return self.data <= string.data\n        return self.data <= string\n\n    def __gt__(self, string):\n        if isinstance(string, UserString):\n            return self.data > string.data\n        return self.data > string\n\n    def __ge__(self, string):\n        if isinstance(string, UserString):\n            return self.data >= string.data\n        return self.data >= string\n\n    def __contains__(self, char):\n        if isinstance(char, UserString):\n            char = char.data\n        return char in self.data\n\n    def __len__(self):\n        return len(self.data)\n\n    def __getitem__(self, index):\n        return self.__class__(self.data[index])\n\n    def __add__(self, other):\n        if isinstance(other, UserString):\n            return self.__class__(self.data + other.data)\n        elif isinstance(other, str):\n            return self.__class__(self.data + other)\n        return self.__class__(self.data + str(other))\n\n    def __radd__(self, other):\n        if isinstance(other, str):\n            return self.__class__(other + self.data)\n        return self.__class__(str(other) + self.data)\n\n    def __mul__(self, n):\n        return self.__class__(self.data * n)\n\n    __rmul__ = __mul__\n\n    def __mod__(self, args):\n        return self.__class__(self.data % args)\n\n    def __getattr__(self, name):\n        if name.startswith(\"__\"):\n            raise AttributeError(name)\n        method = getattr(self.data, name)\n\n        def call(*args, **kwargs):\n            r = method(*[a.data if isinstance(a, UserString) else a for a in args], **kwargs)\n            if isinstance(r, str):\n                return self.__class__(r)\n            return r\n        return call\n\n\nclass ChainMap:\n    def __init__(self, *maps):\n        self.maps = list(maps) or [{}]\n\n    def __missing__(self, key):\n        raise KeyError(key)\n\n    def __getitem__(self, key):\n        for mapping in self.maps:\n            if key in mapping:\n                return mapping[key]\n        return self.__missing__(key)\n\n    def get(self, key, default=None):\n        return self[key] if key in self else default\n\n    def __len__(self):\n        return len(set().union(*self.maps))\n\n    def __iter__(self):\n        d = {}\n        for mapping in reversed(self.maps):\n            for k in mapping:\n                d[k] = None\n        return iter(d)\n\n    def __contains__(self, key):\n        return any(key in m for m in self.maps)\n\n    def __bool__(self):\n        return any(self.maps)\n\n    def __repr__(self):\n        return \"%s(%s)\" % (self.__class__.__name__, \", \".join(map(repr, self.maps)))\n\n    def keys(self):\n        return list(self)\n\n    def items(self):\n        return [(k, self[k]) for k in self]\n\n    def values(self):\n        return [self[k] for k in self]\n\n    def copy(self):\n        return self.__class__(self.maps[0].copy(), *self.maps[1:])\n\n    __copy__ = copy\n\n    def new_child(self, m=None, **kwargs):\n        if m is None:\n            m = kwargs\n        elif kwargs:\n            m.update(kwargs)\n        return self.__class__(m, *self.maps)\n\n    @property\n    def parents(self):\n        return self.__class__(*self.maps[1:])\n\n    def __setitem__(self, key, value):\n        self.maps[0][key] = value\n\n    def __delitem__(self, key):\n        try:\n            del self.maps[0][key]\n        except KeyError:\n            raise KeyError(\"Key not found in the first mapping: {!r}\".format(key))\n\n    def pop(self, key, *args):\n        try:\n            return self.maps[0].pop(key, *args)\n        except KeyError:\n            raise KeyError(\"Key not found in the first mapping: {!r}\".format(key))\n\n    def clear(self):\n        self.maps[0].clear()\n\n    def __eq__(self, other):\n        return dict(self.items()) == dict(other.items())\n\n\ndef _count_elements(mapping, iterable):\n    get = mapping.get\n    for elem in iterable:\n        mapping[elem] = get(elem, 0) + 1\n";
    return s7;
  }
  if ( name == "collections.abc" ) {
    const s8 = "# SPDX-License-Identifier: MIT\n# collections.abc for the Ranger Python runtime. There are no metaclasses\n# here: each ABC answers isinstance()/issubclass() through its own\n# __instancecheck__/__subclasscheck__, by real inheritance, by register(),\n# or (for the one-method ABCs) by the methods a class has.\n\n_abcs = []\n_ITERATORS = (\"generator\", \"map\", \"filter\", \"zip\", \"enumerate\", \"reversed\",\n              \"islice\", \"chain\", \"count\", \"cycle\", \"repeat\", \"takewhile\",\n              \"dropwhile\", \"filterfalse\", \"accumulate\", \"starmap\", \"zip_longest\",\n              \"product\", \"permutations\", \"combinations\", \"groupby\", \"pairwise\",\n              \"batched\", \"compress\", \"tee\", \"<genexpr>\")\n\n\ndef _mro(sub):\n    return getattr(sub, \"__mro__\", None) or (sub,)\n\n\ndef _is_iter_type(sub):\n    name = getattr(sub, \"__name__\", \"\")\n    return isinstance(name, str) and (name.endswith(\"iterator\") or name in _ITERATORS)\n\n\ndef _has(sub, names):\n    for name in names:\n        if name in (\"__iter__\", \"__next__\") and _is_iter_type(sub):\n            continue\n        found = False\n        for base in _mro(sub):\n            d = getattr(base, \"__dict__\", None)\n            if isinstance(d, dict) and name in d:\n                if d[name] is None:\n                    return False\n                found = True\n                break\n        if not found:\n            if getattr(sub, name, None) is None:\n                return False\n    return True\n\n\ndef _hashable(sub):\n    if not isinstance(getattr(sub, \"__dict__\", None), dict):\n        return getattr(sub, \"__hash__\", None) is not None\n    for base in _mro(sub):\n        d = getattr(base, \"__dict__\", None)\n        if not isinstance(d, dict):\n            continue\n        if \"__hash__\" in d:\n            return d[\"__hash__\"] is not None\n        if \"__eq__\" in d:\n            return False\n    return True\n\n\ndef _check(cls, sub):\n    if sub is cls:\n        return True\n    for base in _mro(sub):\n        if base is cls:\n            return True\n    for abc in _abcs:\n        if cls in _mro(abc):\n            for r in abc.__dict__.get(\"_registry\", ()):\n                if sub is r or r in _mro(sub):\n                    return True\n    hook = cls.__dict__.get(\"_hook\")\n    if hook is not None:\n        return hook(sub)\n    return False\n\n\ndef _abc(cls, hook=None):\n    cls._registry = []\n    cls._hook = hook\n\n    def instancecheck(c, obj):\n        return _check(c, type(obj))\n\n    def subclasscheck(c, sub):\n        return _check(c, sub)\n\n    def register(c, sub):\n        c._registry.append(sub)\n        return sub\n    cls.__instancecheck__ = classmethod(instancecheck)\n    cls.__subclasscheck__ = classmethod(subclasscheck)\n    cls.register = classmethod(register)\n    _abcs.append(cls)\n    return cls\n\n\nclass Hashable:\n    __slots__ = ()\n\n    def __hash__(self):\n        return 0\n\n\nclass Awaitable:\n    __slots__ = ()\n\n    def __await__(self):\n        yield\n\n\nclass Coroutine(Awaitable):\n    __slots__ = ()\n\n\nclass AsyncIterable:\n    __slots__ = ()\n\n\nclass AsyncIterator(AsyncIterable):\n    __slots__ = ()\n\n\nclass AsyncGenerator(AsyncIterator):\n    __slots__ = ()\n\n\nclass Iterable:\n    __slots__ = ()\n\n    def __iter__(self):\n        while False:\n            yield None\n\n    def __class_getitem__(cls, item):\n        return cls\n\n\nclass Iterator(Iterable):\n    __slots__ = ()\n\n    def __next__(self):\n        raise StopIteration\n\n    def __iter__(self):\n        return self\n\n\nclass Reversible(Iterable):\n    __slots__ = ()\n\n\nclass Generator(Iterator):\n    __slots__ = ()\n\n    def __next__(self):\n        return self.send(None)\n\n    def close(self):\n        try:\n            self.throw(GeneratorExit)\n        except (GeneratorExit, StopIteration):\n            pass\n        else:\n            raise RuntimeError(\"generator ignored GeneratorExit\")\n\n\nclass Sized:\n    __slots__ = ()\n\n    def __len__(self):\n        return 0\n\n\nclass Container:\n    __slots__ = ()\n\n    def __contains__(self, x):\n        return False\n\n\nclass Collection(Sized, Iterable, Container):\n    __slots__ = ()\n\n\nclass Buffer:\n    __slots__ = ()\n\n\nclass Callable:\n    __slots__ = ()\n\n    def __call__(self, *args, **kwds):\n        return False\n\n\nclass Set(Collection):\n    __slots__ = ()\n\n    def __le__(self, other):\n        if not isinstance(other, Set):\n            return NotImplemented\n        if len(self) > len(other):\n            return False\n        for elem in self:\n            if elem not in other:\n                return False\n        return True\n\n    def __lt__(self, other):\n        if not isinstance(other, Set):\n            return NotImplemented\n        return len(self) < len(other) and self.__le__(other)\n\n    def __gt__(self, other):\n        if not isinstance(other, Set):\n            return NotImplemented\n        return len(self) > len(other) and self.__ge__(other)\n\n    def __ge__(self, other):\n        if not isinstance(other, Set):\n            return NotImplemented\n        if len(self) < len(other):\n            return False\n        for elem in other:\n            if elem not in self:\n                return False\n        return True\n\n    def __eq__(self, other):\n        if not isinstance(other, Set):\n            return NotImplemented\n        return len(self) == len(other) and self.__le__(other)\n\n    @classmethod\n    def _from_iterable(cls, it):\n        return cls(it)\n\n    def __and__(self, other):\n        if not isinstance(other, Iterable):\n            return NotImplemented\n        return self._from_iterable(value for value in other if value in self)\n\n    __rand__ = __and__\n\n    def isdisjoint(self, other):\n        for value in other:\n            if value in self:\n                return False\n        return True\n\n    def __or__(self, other):\n        if not isinstance(other, Iterable):\n            return NotImplemented\n        chain = [e for s in (self, other) for e in s]\n        return self._from_iterable(chain)\n\n    __ror__ = __or__\n\n    def __sub__(self, other):\n        if not isinstance(other, Set):\n            if not isinstance(other, Iterable):\n                return NotImplemented\n            other = self._from_iterable(other)\n        return self._from_iterable(value for value in self if value not in other)\n\n    def __rsub__(self, other):\n        if not isinstance(other, Set):\n            if not isinstance(other, Iterable):\n                return NotImplemented\n            other = self._from_iterable(other)\n        return self._from_iterable(value for value in other if value not in self)\n\n    def __xor__(self, other):\n        if not isinstance(other, Set):\n            if not isinstance(other, Iterable):\n                return NotImplemented\n            other = self._from_iterable(other)\n        return (self - other) | (other - self)\n\n    __rxor__ = __xor__\n\n    def _hash(self):\n        h = 0\n        for x in self:\n            h ^= hash(x)\n        return h\n\n\nclass MutableSet(Set):\n    __slots__ = ()\n\n    def add(self, value):\n        raise NotImplementedError\n\n    def discard(self, value):\n        raise NotImplementedError\n\n    def remove(self, value):\n        if value not in self:\n            raise KeyError(value)\n        self.discard(value)\n\n    def pop(self):\n        it = iter(self)\n        try:\n            value = next(it)\n        except StopIteration:\n            raise KeyError from None\n        self.discard(value)\n        return value\n\n    def clear(self):\n        try:\n            while True:\n                self.pop()\n        except KeyError:\n            pass\n\n    def __ior__(self, it):\n        for value in it:\n            self.add(value)\n        return self\n\n    def __iand__(self, it):\n        for value in (self - it):\n            self.discard(value)\n        return self\n\n    def __ixor__(self, it):\n        if it is self:\n            self.clear()\n        else:\n            if not isinstance(it, Set):\n                it = self._from_iterable(it)\n            for value in it:\n                if value in self:\n                    self.discard(value)\n                else:\n                    self.add(value)\n        return self\n\n    def __isub__(self, it):\n        if it is self:\n            self.clear()\n        else:\n            for value in it:\n                self.discard(value)\n        return self\n\n\nclass Mapping(Collection):\n    __slots__ = ()\n\n    def get(self, key, default=None):\n        try:\n            return self[key]\n        except KeyError:\n            return default\n\n    def __contains__(self, key):\n        try:\n            self[key]\n        except KeyError:\n            return False\n        else:\n            return True\n\n    def keys(self):\n        return KeysView(self)\n\n    def items(self):\n        return ItemsView(self)\n\n    def values(self):\n        return ValuesView(self)\n\n    def __eq__(self, other):\n        if not isinstance(other, Mapping):\n            return NotImplemented\n        return dict(self.items()) == dict(other.items())\n\n    __reversed__ = None\n\n\nclass MappingView(Sized):\n    __slots__ = (\"_mapping\",)\n\n    def __init__(self, mapping):\n        self._mapping = mapping\n\n    def __len__(self):\n        return len(self._mapping)\n\n    def __repr__(self):\n        return \"{0.__class__.__name__}({0._mapping!r})\".format(self)\n\n\nclass KeysView(MappingView, Set):\n    __slots__ = ()\n\n    @classmethod\n    def _from_iterable(cls, it):\n        return set(it)\n\n    def __contains__(self, key):\n        return key in self._mapping\n\n    def __iter__(self):\n        yield from self._mapping\n\n\nclass ItemsView(MappingView, Set):\n    __slots__ = ()\n\n    @classmethod\n    def _from_iterable(cls, it):\n        return set(it)\n\n    def __contains__(self, item):\n        key, value = item\n        try:\n            v = self._mapping[key]\n        except KeyError:\n            return False\n        else:\n            return v is value or v == value\n\n    def __iter__(self):\n        for key in self._mapping:\n            yield (key, self._mapping[key])\n\n\nclass ValuesView(MappingView, Collection):\n    __slots__ = ()\n\n    def __contains__(self, value):\n        for key in self._mapping:\n            v = self._mapping[key]\n            if v is value or v == value:\n                return True\n        return False\n\n    def __iter__(self):\n        for key in self._mapping:\n            yield self._mapping[key]\n\n\nclass MutableMapping(Mapping):\n    __slots__ = ()\n\n    __marker = object()\n\n    def pop(self, key, default=__marker):\n        try:\n            value = self[key]\n        except KeyError:\n            if default is self.__marker:\n                raise\n            return default\n        else:\n            del self[key]\n            return value\n\n    def popitem(self):\n        try:\n            key = next(iter(self))\n        except StopIteration:\n            raise KeyError from None\n        value = self[key]\n        del self[key]\n        return key, value\n\n    def clear(self):\n        try:\n            while True:\n                self.popitem()\n        except KeyError:\n            pass\n\n    def update(self, other=(), /, **kwds):\n        if isinstance(other, Mapping):\n            for key in other:\n                self[key] = other[key]\n        elif hasattr(other, \"keys\"):\n            for key in other.keys():\n                self[key] = other[key]\n        else:\n            for key, value in other:\n                self[key] = value\n        for key, value in kwds.items():\n            self[key] = value\n\n    def setdefault(self, key, default=None):\n        try:\n            return self[key]\n        except KeyError:\n            self[key] = default\n        return default\n\n\nclass Sequence(Reversible, Collection):\n    __slots__ = ()\n\n    def __iter__(self):\n        i = 0\n        try:\n            while True:\n                v = self[i]\n                yield v\n                i += 1\n        except IndexError:\n            return\n\n    def __contains__(self, value):\n        for v in self:\n            if v is value or v == value:\n                return True\n        return False\n\n    def __reversed__(self):\n        for i in reversed(range(len(self))):\n            yield self[i]\n\n    def index(self, value, start=0, stop=None):\n        if start is not None and start < 0:\n            start = max(len(self) + start, 0)\n        if stop is not None and stop < 0:\n            stop += len(self)\n        i = start\n        while stop is None or i < stop:\n            try:\n                v = self[i]\n            except IndexError:\n                break\n            if v is value or v == value:\n                return i\n            i += 1\n        raise ValueError\n\n    def count(self, value):\n        return sum(1 for v in self if v is value or v == value)\n\n\nclass ByteString(Sequence):\n    __slots__ = ()\n\n\nclass MutableSequence(Sequence):\n    __slots__ = ()\n\n    def insert(self, index, value):\n        raise IndexError\n\n    def append(self, value):\n        self.insert(len(self), value)\n\n    def clear(self):\n        try:\n            while True:\n                self.pop()\n        except IndexError:\n            pass\n\n    def reverse(self):\n        n = len(self)\n        for i in range(n // 2):\n            self[i], self[n - i - 1] = self[n - i - 1], self[i]\n\n    def extend(self, values):\n        if values is self:\n            values = list(values)\n        for v in values:\n            self.append(v)\n\n    def pop(self, index=-1):\n        v = self[index]\n        del self[index]\n        return v\n\n    def remove(self, value):\n        del self[self.index(value)]\n\n    def __iadd__(self, values):\n        self.extend(values)\n        return self\n\n\n_abc(Hashable, _hashable)\n_abc(Awaitable, lambda s: _has(s, (\"__await__\",)))\n_abc(Coroutine, lambda s: _has(s, (\"send\", \"throw\", \"close\", \"__await__\")))\n_abc(AsyncIterable, lambda s: _has(s, (\"__aiter__\",)))\n_abc(AsyncIterator, lambda s: _has(s, (\"__aiter__\", \"__anext__\")))\n_abc(AsyncGenerator, lambda s: _has(s, (\"__aiter__\", \"__anext__\", \"asend\", \"athrow\", \"aclose\")))\n_abc(Iterable, lambda s: _has(s, (\"__iter__\",)) or _has(s, (\"__getitem__\",)) and False)\n_abc(Iterator, lambda s: _has(s, (\"__iter__\", \"__next__\")))\n_abc(Reversible, lambda s: _has(s, (\"__reversed__\", \"__iter__\")))\n_abc(Generator, lambda s: _has(s, (\"__iter__\", \"__next__\", \"send\", \"throw\", \"close\")))\n_abc(Sized, lambda s: _has(s, (\"__len__\",)))\n_abc(Container, lambda s: _has(s, (\"__contains__\",)))\n_abc(Collection, lambda s: _has(s, (\"__len__\", \"__iter__\", \"__contains__\")))\n_abc(Buffer, lambda s: _has(s, (\"__buffer__\",)))\n_abc(Callable, lambda s: _has(s, (\"__call__\",)) or callable(s) and getattr(s, \"__name__\", \"\") in (\"function\", \"builtin_function_or_method\", \"method\"))\n_abc(Set)\n_abc(MutableSet)\n_abc(Mapping)\n_abc(MappingView)\n_abc(KeysView)\n_abc(ItemsView)\n_abc(ValuesView)\n_abc(MutableMapping)\n_abc(Sequence)\n_abc(ByteString)\n_abc(MutableSequence)\n\nSequence.register(tuple)\nSequence.register(str)\nSequence.register(range)\nMutableSequence.register(list)\nSet.register(frozenset)\nMutableSet.register(set)\nMutableMapping.register(dict)\nReversible.register(dict)\n";
    return s8;
  }
  if ( name == "contextlib" ) {
    const s9 = "# SPDX-License-Identifier: MIT\n# contextlib for the Ranger Python runtime.\nimport sys\n\n\nclass AbstractContextManager:\n    def __enter__(self):\n        return self\n\n    def __exit__(self, exc_type, exc_value, tb):\n        return None\n\n\nclass ContextDecorator:\n    def __call__(self, func):\n        def inner(*args, **kwds):\n            with self:\n                return func(*args, **kwds)\n        return inner\n\n\nclass suppress(AbstractContextManager):\n    def __init__(self, *exceptions):\n        self._exceptions = exceptions\n\n    def __exit__(self, exc_type, exc_value, tb):\n        return exc_type is not None and issubclass(exc_type, self._exceptions)\n\n\nclass nullcontext(AbstractContextManager):\n    def __init__(self, enter_result=None):\n        self.enter_result = enter_result\n\n    def __enter__(self):\n        return self.enter_result\n\n    def __exit__(self, *exc):\n        return None\n\n\nclass closing(AbstractContextManager):\n    def __init__(self, thing):\n        self.thing = thing\n\n    def __enter__(self):\n        return self.thing\n\n    def __exit__(self, *exc):\n        self.thing.close()\n\n\nclass _Redirect(AbstractContextManager):\n    _stream = \"stdout\"\n\n    def __init__(self, new_target):\n        self._new_target = new_target\n        self._old = []\n\n    def __enter__(self):\n        self._old.append(getattr(sys, self._stream))\n        setattr(sys, self._stream, self._new_target)\n        return self._new_target\n\n    def __exit__(self, *exc):\n        setattr(sys, self._stream, self._old.pop())\n\n\nclass redirect_stdout(_Redirect):\n    _stream = \"stdout\"\n\n\nclass redirect_stderr(_Redirect):\n    _stream = \"stderr\"\n\n\nclass ExitStack(AbstractContextManager):\n    def __init__(self):\n        self._callbacks = []\n\n    def enter_context(self, cm):\n        result = cm.__enter__()\n        self._callbacks.append(cm.__exit__)\n        return result\n\n    def callback(self, fn, *args, **kwds):\n        self._callbacks.append(lambda *exc: fn(*args, **kwds))\n        return fn\n\n    def push(self, exit_fn):\n        self._callbacks.append(exit_fn)\n        return exit_fn\n\n    def close(self):\n        self.__exit__(None, None, None)\n\n    def __exit__(self, *exc):\n        suppressed = False\n        while self._callbacks:\n            cb = self._callbacks.pop()\n            if cb(*exc):\n                suppressed = True\n                exc = (None, None, None)\n        return suppressed\n\n\nclass _GeneratorContextManager(ContextDecorator, AbstractContextManager):\n    def __init__(self, func, args, kwds):\n        self.gen = func(*args, **kwds)\n        self.func, self.args, self.kwds = func, args, kwds\n        self.__doc__ = getattr(func, \"__doc__\", None)\n\n    def _recreate_cm(self):\n        return self.__class__(self.func, self.args, self.kwds)\n\n    def __call__(self, func):\n        def inner(*args, **kwds):\n            with self._recreate_cm():\n                return func(*args, **kwds)\n        return inner\n\n    def __enter__(self):\n        del self.args, self.kwds, self.func\n        try:\n            return next(self.gen)\n        except StopIteration:\n            raise RuntimeError(\"generator didn't yield\") from None\n\n    def __exit__(self, typ, value, traceback):\n        if typ is None:\n            try:\n                next(self.gen)\n            except StopIteration:\n                return False\n            else:\n                try:\n                    raise RuntimeError(\"generator didn't stop\")\n                finally:\n                    self.gen.close()\n        else:\n            if value is None:\n                value = typ()\n            try:\n                self.gen.throw(value)\n            except StopIteration as exc:\n                return exc is not value\n            except RuntimeError as exc:\n                if exc is value:\n                    return False\n                if isinstance(value, StopIteration) and exc.__cause__ is value:\n                    return False\n                raise\n            except BaseException as exc:\n                if exc is not value:\n                    raise\n                return False\n            try:\n                raise RuntimeError(\"generator didn't stop after throw()\")\n            finally:\n                self.gen.close()\n\n\ndef contextmanager(func):\n    def helper(*args, **kwds):\n        return _GeneratorContextManager(func, args, kwds)\n    helper.__name__ = getattr(func, \"__name__\", \"helper\")\n    helper.__doc__ = getattr(func, \"__doc__\", None)\n    helper.__wrapped__ = func\n    return helper\n";
    return s9;
  }
  if ( name == "copyreg" ) {
    const s10 = "# SPDX-License-Identifier: MIT\n# copyreg for the Ranger Python runtime\ndispatch_table = {}\n_extension_registry = {}\n_inverted_registry = {}\n_extension_cache = {}\n\n\ndef pickle(ob_type, pickle_function, constructor_ob=None):\n    if not callable(pickle_function):\n        raise TypeError(\"reduction functions must be callable\")\n    dispatch_table[ob_type] = pickle_function\n\n\ndef constructor(object):\n    if not callable(object):\n        raise TypeError(\"constructors must be callable\")\n\n\ndef __newobj__(cls, *args):\n    return cls.__new__(cls, *args)\n\n\ndef __newobj_ex__(cls, args, kwargs):\n    return cls.__new__(cls, *args, **kwargs)\n\n\ndef _reconstructor(cls, base, state):\n    obj = object.__new__(cls)\n    return obj\n\n\ndef add_extension(module, name, code):\n    _extension_registry[(module, name)] = code\n    _inverted_registry[code] = (module, name)\n\n\ndef remove_extension(module, name, code):\n    del _extension_registry[(module, name)]\n    del _inverted_registry[code]\n\n\ndef clear_extension_cache():\n    _extension_cache.clear()\n";
    return s10;
  }
  if ( name == "csv" ) {
    const s11 = "# SPDX-License-Identifier: MIT\n# csv for the Ranger Python runtime: reader, writer, DictReader, DictWriter\n# and the dialects, in Python.\n\n__all__ = [\"QUOTE_MINIMAL\", \"QUOTE_ALL\", \"QUOTE_NONNUMERIC\", \"QUOTE_NONE\", \"QUOTE_STRINGS\", \"QUOTE_NOTNULL\",\n           \"Error\", \"Dialect\", \"excel\", \"excel_tab\", \"unix_dialect\", \"reader\", \"writer\", \"DictReader\",\n           \"DictWriter\", \"register_dialect\", \"get_dialect\", \"list_dialects\", \"unregister_dialect\",\n           \"field_size_limit\", \"Sniffer\", \"__version__\"]\n\n__version__ = \"1.0\"\n\nQUOTE_MINIMAL, QUOTE_ALL, QUOTE_NONNUMERIC, QUOTE_NONE, QUOTE_STRINGS, QUOTE_NOTNULL = range(6)\n\n\nclass Error(Exception):\n    pass\n\n\nclass Dialect:\n    delimiter = \",\"\n    quotechar = '\"'\n    escapechar = None\n    doublequote = True\n    skipinitialspace = False\n    lineterminator = \"\\r\\n\"\n    quoting = QUOTE_MINIMAL\n    strict = False\n\n\nclass excel(Dialect):\n    pass\n\n\nclass excel_tab(excel):\n    delimiter = \"\\t\"\n\n\nclass unix_dialect(Dialect):\n    lineterminator = \"\\n\"\n    quoting = QUOTE_ALL\n\n\n_dialects = {\"excel\": excel, \"excel-tab\": excel_tab, \"unix\": unix_dialect}\n_limit = 131072\n\n\ndef register_dialect(name, dialect=None, **fmtparams):\n    _dialects[name] = _Params(dialect or excel, fmtparams)\n\n\ndef unregister_dialect(name):\n    if name not in _dialects:\n        raise Error(\"unknown dialect\")\n    del _dialects[name]\n\n\ndef get_dialect(name):\n    if name not in _dialects:\n        raise Error(\"unknown dialect\")\n    return _dialects[name]\n\n\ndef list_dialects():\n    return list(_dialects)\n\n\ndef field_size_limit(new_limit=None):\n    global _limit\n    old = _limit\n    if new_limit is not None:\n        _limit = new_limit\n    return old\n\n\nclass _Params:\n    def __init__(self, dialect, fmtparams):\n        if isinstance(dialect, str):\n            dialect = get_dialect(dialect)\n        for name in (\"delimiter\", \"quotechar\", \"escapechar\", \"doublequote\", \"skipinitialspace\",\n                     \"lineterminator\", \"quoting\", \"strict\"):\n            setattr(self, name, fmtparams.get(name, getattr(dialect, name, getattr(Dialect, name))))\n        if not isinstance(self.delimiter, str) or len(self.delimiter) != 1:\n            raise TypeError('\"delimiter\" must be a 1-character string')\n        if self.quotechar is None and self.quoting != QUOTE_NONE:\n            raise TypeError(\"quotechar must be set if quoting enabled\")\n\n\nclass _Reader:\n    def __init__(self, f, dialect, fmtparams):\n        self._it = iter(f)\n        self.dialect = _Params(dialect, fmtparams)\n        self.line_num = 0\n\n    def __iter__(self):\n        return self\n\n    def __next__(self):\n        d = self.dialect\n        row = []\n        field = []\n        state = 0  # 0 start of field, 1 in field, 2 in quotes, 3 quote in quotes\n        quoted = False\n        while True:\n            line = next(self._it)\n            self.line_num += 1\n            i = 0\n            n = len(line)\n            while i < n:\n                c = line[i]\n                if state == 2:\n                    if d.escapechar is not None and c == d.escapechar and i + 1 < n:\n                        field.append(line[i + 1])\n                        i += 2\n                        continue\n                    if c == d.quotechar:\n                        state = 3 if d.doublequote else 1\n                    else:\n                        field.append(c)\n                elif state == 3:\n                    if c == d.quotechar:\n                        field.append(c)\n                        state = 2\n                    elif c == d.delimiter:\n                        row.append(self._value(\"\".join(field), True))\n                        field = []\n                        quoted = False\n                        state = 0\n                    elif c in \"\\r\\n\":\n                        state = 1\n                    else:\n                        if d.strict:\n                            raise Error(\"'%s' expected after '%s'\" % (d.delimiter, d.quotechar))\n                        field.append(c)\n                        state = 1\n                else:\n                    if c == d.delimiter:\n                        row.append(self._value(\"\".join(field), quoted))\n                        field = []\n                        quoted = False\n                        state = 0\n                    elif state == 0 and c == \" \" and d.skipinitialspace:\n                        pass\n                    elif state == 0 and c == d.quotechar and d.quoting != QUOTE_NONE:\n                        state = 2\n                        quoted = True\n                    elif d.escapechar is not None and c == d.escapechar and i + 1 < n:\n                        field.append(line[i + 1])\n                        i += 1\n                        state = 1\n                    elif c in \"\\r\\n\":\n                        pass\n                    else:\n                        field.append(c)\n                        state = 1\n                i += 1\n            if state == 2:\n                # a newline inside quotes belongs to the field (kept above)\n                continue\n            break\n        if row or field or quoted or state != 0:\n            row.append(self._value(\"\".join(field), quoted))\n        elif not row and self.line_num and line.strip(\"\\r\\n\") == \"\":\n            return []\n        return row\n\n    def _value(self, s, quoted):\n        if not quoted and self.dialect.quoting == QUOTE_NONNUMERIC:\n            return float(s)\n        return s\n\n\ndef reader(csvfile, dialect=\"excel\", **fmtparams):\n    return _Reader(csvfile, dialect, fmtparams)\n\n\nclass _Writer:\n    def __init__(self, f, dialect, fmtparams):\n        self._f = f\n        self.dialect = _Params(dialect, fmtparams)\n\n    def _field(self, v):\n        d = self.dialect\n        if v is None:\n            s = \"\"\n        elif isinstance(v, float):\n            s = repr(v)\n        else:\n            s = str(v)\n        if d.quoting == QUOTE_NONE:\n            if d.escapechar:\n                for ch in (d.escapechar, d.delimiter, d.quotechar):\n                    if ch:\n                        s = s.replace(ch, d.escapechar + ch)\n            elif any(ch in s for ch in (d.delimiter, \"\\n\", \"\\r\")):\n                raise Error(\"need to escape, but no escapechar set\")\n            return s\n        need = (d.quoting == QUOTE_ALL\n                or (d.quoting == QUOTE_NONNUMERIC and not isinstance(v, (int, float)))\n                or (d.quoting == QUOTE_STRINGS and isinstance(v, str))\n                or (d.quoting == QUOTE_NOTNULL and v is not None)\n                or any(ch in s for ch in (d.delimiter, d.quotechar, \"\\n\", \"\\r\"))\n                or (s and (s[0] == \" \" and d.skipinitialspace)))\n        if d.quotechar and d.quotechar in s:\n            if d.doublequote:\n                s = s.replace(d.quotechar, d.quotechar * 2)\n            elif d.escapechar:\n                s = s.replace(d.quotechar, d.escapechar + d.quotechar)\n            else:\n                raise Error(\"need to escape, but no escapechar set\")\n        return d.quotechar + s + d.quotechar if need else s\n\n    def writerow(self, row):\n        fields = list(row)\n        line = self.dialect.delimiter.join(self._field(v) for v in fields)\n        if len(fields) == 1 and line == \"\":\n            line = self.dialect.quotechar * 2\n        return self._f.write(line + self.dialect.lineterminator)\n\n    def writerows(self, rows):\n        for r in rows:\n            self.writerow(r)\n\n\ndef writer(csvfile, dialect=\"excel\", **fmtparams):\n    return _Writer(csvfile, dialect, fmtparams)\n\n\nclass DictReader:\n    def __init__(self, f, fieldnames=None, restkey=None, restval=None, dialect=\"excel\", *args, **kwds):\n        self._fieldnames = fieldnames\n        self.restkey = restkey\n        self.restval = restval\n        self.reader = reader(f, dialect, *args, **kwds)\n        self.dialect = dialect\n        self.line_num = 0\n\n    def __iter__(self):\n        return self\n\n    @property\n    def fieldnames(self):\n        if self._fieldnames is None:\n            try:\n                self._fieldnames = next(self.reader)\n            except StopIteration:\n                pass\n        self.line_num = self.reader.line_num\n        return self._fieldnames\n\n    @fieldnames.setter\n    def fieldnames(self, value):\n        self._fieldnames = value\n\n    def __next__(self):\n        if self.line_num == 0:\n            self.fieldnames\n        row = next(self.reader)\n        self.line_num = self.reader.line_num\n        while row == []:\n            row = next(self.reader)\n        names = self.fieldnames\n        d = dict(zip(names, row))\n        lf, lr = len(names), len(row)\n        if lf < lr:\n            d[self.restkey] = row[lf:]\n        elif lf > lr:\n            for key in names[lr:]:\n                d[key] = self.restval\n        return d\n\n\nclass DictWriter:\n    def __init__(self, f, fieldnames, restval=\"\", extrasaction=\"raise\", dialect=\"excel\", *args, **kwds):\n        self.fieldnames = list(fieldnames)\n        self.restval = restval\n        if extrasaction.lower() not in (\"raise\", \"ignore\"):\n            raise ValueError(\"extrasaction (%s) must be 'raise' or 'ignore'\" % extrasaction)\n        self.extrasaction = extrasaction\n        self.writer = writer(f, dialect, *args, **kwds)\n\n    def writeheader(self):\n        return self.writer.writerow(self.fieldnames)\n\n    def _row(self, rowdict):\n        if self.extrasaction == \"raise\":\n            wrong = [k for k in rowdict if k not in self.fieldnames]\n            if wrong:\n                raise ValueError(\"dict contains fields not in fieldnames: \" + \", \".join(repr(x) for x in wrong))\n        return (rowdict.get(key, self.restval) for key in self.fieldnames)\n\n    def writerow(self, rowdict):\n        return self.writer.writerow(self._row(rowdict))\n\n    def writerows(self, rowdicts):\n        for r in rowdicts:\n            self.writerow(r)\n\n\nclass Sniffer:\n    def sniff(self, sample, delimiters=None):\n        cands = delimiters or \",;\\t|:\"\n        lines = [l for l in sample.splitlines() if l][:10]\n        best = None\n        for c in cands:\n            counts = [l.count(c) for l in lines]\n            if counts and counts[0] > 0 and all(x == counts[0] for x in counts):\n                best = c\n                break\n        if best is None:\n            raise Error(\"Could not determine delimiter\")\n\n        class sniffed(excel):\n            pass\n        sniffed.delimiter = best\n        sniffed.lineterminator = \"\\r\\n\" if \"\\r\\n\" in sample else \"\\n\"\n        return sniffed\n\n    def has_header(self, sample):\n        rows = list(reader(sample.splitlines(), self.sniff(sample)))\n        if len(rows) < 2:\n            return False\n\n        def num(s):\n            try:\n                float(s)\n                return True\n            except ValueError:\n                return False\n        return any(not num(h) and num(v) for h, v in zip(rows[0], rows[1]))\n";
    return s11;
  }
  if ( name == "dataclasses" ) {
    const s12 = "# SPDX-License-Identifier: MIT\n# dataclasses for the Ranger Python runtime. Like CPython's, the methods\n# are written as source text and run with exec.\nimport copy as _copy\nimport keyword as _keyword\n\n__all__ = [\"dataclass\", \"field\", \"Field\", \"FrozenInstanceError\", \"InitVar\", \"KW_ONLY\", \"MISSING\",\n           \"fields\", \"asdict\", \"astuple\", \"make_dataclass\", \"replace\", \"is_dataclass\"]\n\n\nclass FrozenInstanceError(AttributeError):\n    pass\n\n\nclass _MISSING_TYPE:\n    def __repr__(self):\n        return \"MISSING\"\n\n\nMISSING = _MISSING_TYPE()\n\n\nclass _KW_ONLY_TYPE:\n    def __repr__(self):\n        return \"KW_ONLY\"\n\n\nKW_ONLY = _KW_ONLY_TYPE()\n\n\nclass InitVar:\n    __slots__ = (\"type\",)\n\n    def __init__(self, type):\n        self.type = type\n\n    def __repr__(self):\n        name = getattr(self.type, \"__name__\", repr(self.type))\n        return f\"dataclasses.InitVar[{name}]\"\n\n    def __class_getitem__(cls, type):\n        return InitVar(type)\n\n\n_FIELD = \"_FIELD\"\n_FIELD_CLASSVAR = \"_FIELD_CLASSVAR\"\n_FIELD_INITVAR = \"_FIELD_INITVAR\"\n_FIELDS = \"__dataclass_fields__\"\n_PARAMS = \"__dataclass_params__\"\n\n\nclass Field:\n    __slots__ = (\"name\", \"type\", \"default\", \"default_factory\", \"repr\", \"hash\", \"init\", \"compare\",\n                 \"metadata\", \"kw_only\", \"_field_type\")\n\n    def __init__(self, default, default_factory, init, repr, hash, compare, metadata, kw_only):\n        self.name = None\n        self.type = None\n        self.default = default\n        self.default_factory = default_factory\n        self.init = init\n        self.repr = repr\n        self.hash = hash\n        self.compare = compare\n        self.metadata = {} if metadata is None else dict(metadata)\n        self.kw_only = kw_only\n        self._field_type = None\n\n    def __repr__(self):\n        return (\"Field(name=%r,type=%r,default=%r,default_factory=%r,init=%r,repr=%r,hash=%r,\"\n                \"compare=%r,metadata=%r,kw_only=%r,_field_type=%s)\" % (\n                    self.name, self.type, self.default, self.default_factory, self.init, self.repr,\n                    self.hash, self.compare, self.metadata, self.kw_only, self._field_type))\n\n    def __set_name__(self, owner, name):\n        func = getattr(type(self.default), \"__set_name__\", None)\n        if func:\n            func(self.default, owner, name)\n\n\nclass _DataclassParams:\n    __slots__ = (\"init\", \"repr\", \"eq\", \"order\", \"unsafe_hash\", \"frozen\", \"match_args\", \"kw_only\", \"slots\", \"weakref_slot\")\n\n    def __init__(self, init, repr, eq, order, unsafe_hash, frozen, match_args=True, kw_only=False, slots=False, weakref_slot=False):\n        self.init = init\n        self.repr = repr\n        self.eq = eq\n        self.order = order\n        self.unsafe_hash = unsafe_hash\n        self.frozen = frozen\n        self.match_args = match_args\n        self.kw_only = kw_only\n        self.slots = slots\n        self.weakref_slot = weakref_slot\n\n    def __repr__(self):\n        return (\"_DataclassParams(init=%r,repr=%r,eq=%r,order=%r,unsafe_hash=%r,frozen=%r)\" % (\n            self.init, self.repr, self.eq, self.order, self.unsafe_hash, self.frozen))\n\n\ndef field(*, default=MISSING, default_factory=MISSING, init=True, repr=True, hash=None, compare=True, metadata=None, kw_only=MISSING):\n    if default is not MISSING and default_factory is not MISSING:\n        raise ValueError(\"cannot specify both default and default_factory\")\n    return Field(default, default_factory, init, repr, hash, compare, metadata, kw_only)\n\n\ndef _is_classvar(a_type):\n    try:\n        import typing\n        cv = typing.ClassVar\n    except Exception:\n        cv = None\n    if cv is not None and (a_type is cv or getattr(a_type, \"__origin__\", None) is cv):\n        return True\n    if isinstance(a_type, str):\n        s = a_type.strip()\n        return s.startswith(\"ClassVar\") or s.startswith(\"typing.ClassVar\")\n    r = repr(a_type)\n    return r.startswith(\"typing.ClassVar\")\n\n\ndef _is_initvar(a_type):\n    if isinstance(a_type, InitVar) or a_type is InitVar:\n        return True\n    if isinstance(a_type, str):\n        s = a_type.strip()\n        return s.startswith(\"InitVar\") or s.startswith(\"dataclasses.InitVar\")\n    return False\n\n\ndef _is_kw_only(a_type):\n    if a_type is KW_ONLY:\n        return True\n    return isinstance(a_type, str) and a_type.strip() in (\"KW_ONLY\", \"dataclasses.KW_ONLY\")\n\n\ndef _get_field(cls, a_name, a_type, default_kw_only):\n    default = getattr(cls, a_name, MISSING)\n    if isinstance(default, Field):\n        f = default\n    else:\n        f = field(default=default)\n    f.name = a_name\n    f.type = a_type\n    f._field_type = _FIELD\n    if _is_classvar(a_type):\n        f._field_type = _FIELD_CLASSVAR\n    elif _is_initvar(a_type):\n        f._field_type = _FIELD_INITVAR\n    if f._field_type in (_FIELD_CLASSVAR, _FIELD_INITVAR):\n        if f.default_factory is not MISSING:\n            raise TypeError(f\"field {f.name} cannot have a default factory\")\n    if f._field_type in (_FIELD, _FIELD_INITVAR):\n        if f.kw_only is MISSING:\n            f.kw_only = default_kw_only\n    else:\n        f.kw_only = False\n    if f._field_type is _FIELD and f.default_factory is MISSING and isinstance(f.default, (list, dict, set)):\n        raise ValueError(f\"mutable default {type(f.default)} for field {f.name} is not allowed: use default_factory\")\n    return f\n\n\ndef _fields_in_init_order(fields):\n    return (tuple(f for f in fields if f.init and not f.kw_only), tuple(f for f in fields if f.init and f.kw_only))\n\n\ndef _tuple_str(obj_name, fields):\n    if not fields:\n        return \"()\"\n    return \"(\" + \",\".join([f\"{obj_name}.{f.name}\" for f in fields]) + \",)\"\n\n\ndef _exec(name, src, glob):\n    ns = {}\n    exec(src, glob, ns)\n    return ns[name]\n\n\ndef _set_new_attribute(cls, name, value):\n    if name in cls.__dict__:\n        return True\n    if hasattr(value, \"__qualname__\"):\n        try:\n            value.__qualname__ = f\"{cls.__qualname__}.{value.__name__}\"\n        except Exception:\n            pass\n    setattr(cls, name, value)\n    return False\n\n\ndef _init_fn(fields, std_fields, kw_only_fields, frozen, has_post_init, self_name, globals_):\n    seen_default = False\n    for f in std_fields:\n        if f.init:\n            if not (f.default is MISSING and f.default_factory is MISSING):\n                seen_default = True\n            elif seen_default:\n                raise TypeError(f\"non-default argument {f.name!r} follows default argument\")\n    glob = dict(globals_)\n    glob.update({\"__dataclass_MISSING__\": MISSING, \"__dataclass_object_setattr__\": object.__setattr__})\n    args = [self_name]\n    for f in std_fields:\n        args.append(_init_param(f, glob))\n    if kw_only_fields:\n        args.append(\"*\")\n        for f in kw_only_fields:\n            args.append(_init_param(f, glob))\n    body = []\n    for f in fields:\n        line = _field_init(f, frozen, glob, self_name)\n        if line:\n            body.append(line)\n    if has_post_init:\n        params = \",\".join(f.name for f in fields if f._field_type is _FIELD_INITVAR)\n        body.append(f\"{self_name}.__post_init__({params})\")\n    if not body:\n        body = [\"pass\"]\n    src = \"def __init__(\" + \",\".join(args) + \"):\\n\" + \"\\n\".join(\"  \" + b for b in body) + \"\\n\"\n    return _exec(\"__init__\", src, glob)\n\n\ndef _init_param(f, glob):\n    if f.default is MISSING and f.default_factory is MISSING:\n        return f.name\n    if f.default is not MISSING:\n        glob[f\"__dataclass_dflt_{f.name}__\"] = f.default\n        return f\"{f.name}=__dataclass_dflt_{f.name}__\"\n    return f\"{f.name}=__dataclass_HAS_DEFAULT_FACTORY__\"\n\n\ndef _field_init(f, frozen, glob, self_name):\n    default_name = f\"__dataclass_dflt_{f.name}__\"\n    if f.default_factory is not MISSING:\n        glob[f\"__dataclass_factory_{f.name}__\"] = f.default_factory\n        glob[\"__dataclass_HAS_DEFAULT_FACTORY__\"] = _HAS_DEFAULT_FACTORY\n        if f.init:\n            value = (f\"__dataclass_factory_{f.name}__() if {f.name} is __dataclass_HAS_DEFAULT_FACTORY__ else {f.name}\")\n        else:\n            value = f\"__dataclass_factory_{f.name}__()\"\n    else:\n        if f.init:\n            if f.default is MISSING:\n                value = f.name\n            else:\n                glob[default_name] = f.default\n                value = f.name\n        else:\n            if f.default is not MISSING:\n                glob[default_name] = f.default\n                value = default_name\n            else:\n                return None\n    if f._field_type is _FIELD_INITVAR:\n        return None\n    if frozen:\n        return f\"__dataclass_object_setattr__({self_name},{f.name!r},{value})\"\n    return f\"{self_name}.{f.name}={value}\"\n\n\nclass _HAS_DEFAULT_FACTORY_CLASS:\n    def __repr__(self):\n        return \"<factory>\"\n\n\n_HAS_DEFAULT_FACTORY = _HAS_DEFAULT_FACTORY_CLASS()\n\n\ndef _repr_fn(fields):\n    def __repr__(self):\n        key = (id(self), \"repr\")\n        if key in _repr_running:\n            return \"...\"\n        _repr_running.add(key)\n        try:\n            parts = \", \".join(f\"{f.name}={getattr(self, f.name)!r}\" for f in fields)\n        finally:\n            _repr_running.discard(key)\n        return f\"{self.__class__.__qualname__}({parts})\"\n    return __repr__\n\n\n_repr_running = set()\n\n\ndef _cmp_fn(name, op, fields):\n    def cmp(self, other):\n        if other.__class__ is self.__class__:\n            a = tuple(getattr(self, f.name) for f in fields)\n            b = tuple(getattr(other, f.name) for f in fields)\n            if op == \"==\":\n                return a == b\n            if op == \"<\":\n                return a < b\n            if op == \"<=\":\n                return a <= b\n            if op == \">\":\n                return a > b\n            return a >= b\n        return NotImplemented\n    cmp.__name__ = name\n    return cmp\n\n\ndef _hash_fn(fields):\n    def __hash__(self):\n        return hash(tuple(getattr(self, f.name) for f in fields))\n    return __hash__\n\n\ndef _frozen_setattr(cls, fields):\n    names = {f.name for f in fields}\n\n    def __setattr__(self, name, value):\n        if type(self) is cls or name in names:\n            raise FrozenInstanceError(f\"cannot assign to field {name!r}\")\n        super(cls, self).__setattr__(name, value)\n\n    def __delattr__(self, name):\n        if type(self) is cls or name in names:\n            raise FrozenInstanceError(f\"cannot delete field {name!r}\")\n        super(cls, self).__delattr__(name)\n    return __setattr__, __delattr__\n\n\ndef _process_class(cls, init, repr, eq, order, unsafe_hash, frozen, match_args, kw_only, slots, weakref_slot):\n    fields = {}\n    globals_ = {}\n    mod = getattr(cls, \"__module__\", None)\n    import sys\n    if mod in sys.modules:\n        globals_ = sys.modules[mod].__dict__\n    setattr(cls, _PARAMS, _DataclassParams(init, repr, eq, order, unsafe_hash, frozen, match_args, kw_only, slots, weakref_slot))\n    any_frozen_base = False\n    has_dataclass_bases = False\n    for b in cls.__mro__[-1:0:-1]:\n        base_fields = getattr(b, _FIELDS, None)\n        if base_fields is not None:\n            has_dataclass_bases = True\n            for f in base_fields.values():\n                fields[f.name] = f\n            if getattr(b, _PARAMS).frozen:\n                any_frozen_base = True\n    cls_annotations = cls.__dict__.get(\"__annotations__\", {})\n    cls_fields = []\n    KW_ONLY_seen = False\n    for name, type_ in cls_annotations.items():\n        if _is_kw_only(type_):\n            if KW_ONLY_seen:\n                raise TypeError(f\"{name!r} is KW_ONLY, but KW_ONLY has already been specified\")\n            KW_ONLY_seen = True\n            kw_only = True\n        else:\n            cls_fields.append(_get_field(cls, name, type_, kw_only))\n    for f in cls_fields:\n        fields[f.name] = f\n        if isinstance(getattr(cls, f.name, None), Field):\n            if f.default is MISSING:\n                delattr(cls, f.name)\n            else:\n                setattr(cls, f.name, f.default)\n    for name, value in cls.__dict__.items():\n        if isinstance(value, Field) and name not in cls_annotations:\n            raise TypeError(f\"{name!r} is a field but has no type annotation\")\n    if has_dataclass_bases:\n        if any_frozen_base and not frozen:\n            raise TypeError(\"cannot inherit non-frozen dataclass from a frozen one\")\n        if not any_frozen_base and frozen:\n            raise TypeError(\"cannot inherit frozen dataclass from a non-frozen one\")\n    setattr(cls, _FIELDS, fields)\n    class_hash = cls.__dict__.get(\"__hash__\", MISSING)\n    has_explicit_hash = not (class_hash is MISSING or (class_hash is None and \"__eq__\" in cls.__dict__))\n    if order and not eq:\n        raise ValueError(\"eq must be true if order is true\")\n    all_init_fields = [f for f in fields.values() if f._field_type in (_FIELD, _FIELD_INITVAR)]\n    std_init_fields, kw_only_init_fields = _fields_in_init_order(all_init_fields)\n    field_list = [f for f in fields.values() if f._field_type is _FIELD]\n    if init:\n        has_post_init = hasattr(cls, \"__post_init__\")\n        _set_new_attribute(cls, \"__init__\", _init_fn(all_init_fields, std_init_fields, kw_only_init_fields,\n                                                     frozen, has_post_init, \"self\" if \"self\" not in fields else \"__dataclass_self__\", globals_))\n    if repr:\n        flds = [f for f in field_list if f.repr]\n        _set_new_attribute(cls, \"__repr__\", _repr_fn(flds))\n    if eq:\n        cmp_fields = [f for f in field_list if f.compare]\n        _set_new_attribute(cls, \"__eq__\", _cmp_fn(\"__eq__\", \"==\", cmp_fields))\n    if order:\n        flds = [f for f in field_list if f.compare]\n        for name, op in [(\"__lt__\", \"<\"), (\"__le__\", \"<=\"), (\"__gt__\", \">\"), (\"__ge__\", \">=\")]:\n            if _set_new_attribute(cls, name, _cmp_fn(name, op, flds)):\n                raise TypeError(f\"Cannot overwrite attribute {name} in class {cls.__name__}. Consider using functools.total_ordering\")\n    if frozen:\n        sa, da = _frozen_setattr(cls, field_list)\n        if _set_new_attribute(cls, \"__setattr__\", sa):\n            raise TypeError(f\"Cannot overwrite attribute __setattr__ in class {cls.__name__}\")\n        _set_new_attribute(cls, \"__delattr__\", da)\n    hash_fields = [f for f in field_list if (f.compare if f.hash is None else f.hash)]\n    if unsafe_hash:\n        if has_explicit_hash:\n            raise TypeError(f\"Cannot overwrite attribute __hash__ in class {cls.__name__}\")\n        cls.__hash__ = _hash_fn(hash_fields)\n    elif eq and frozen:\n        if not has_explicit_hash:\n            cls.__hash__ = _hash_fn(hash_fields)\n    elif eq and not frozen:\n        if not has_explicit_hash:\n            cls.__hash__ = None\n    if not getattr(cls, \"__doc__\"):\n        try:\n            import inspect\n            text_sig = str(inspect.signature(cls)).replace(\" -> None\", \"\")\n        except Exception:\n            text_sig = \"\"\n        cls.__doc__ = cls.__name__ + text_sig\n    if match_args:\n        _set_new_attribute(cls, \"__match_args__\", tuple(f.name for f in std_init_fields))\n    if slots:\n        names = tuple(f.name for f in field_list)\n        cls.__slots__ = names\n    import abc\n    abc.update_abstractmethods(cls)\n    return cls\n\n\ndef dataclass(cls=None, /, *, init=True, repr=True, eq=True, order=False, unsafe_hash=False, frozen=False,\n              match_args=True, kw_only=False, slots=False, weakref_slot=False):\n    def wrap(cls):\n        return _process_class(cls, init, repr, eq, order, unsafe_hash, frozen, match_args, kw_only, slots, weakref_slot)\n    if cls is None:\n        return wrap\n    return wrap(cls)\n\n\ndef fields(class_or_instance):\n    try:\n        fields = getattr(class_or_instance, _FIELDS)\n    except AttributeError:\n        raise TypeError(\"must be called with a dataclass type or instance\") from None\n    return tuple(f for f in fields.values() if f._field_type is _FIELD)\n\n\ndef _is_dataclass_instance(obj):\n    return hasattr(type(obj), _FIELDS)\n\n\ndef is_dataclass(obj):\n    cls = obj if isinstance(obj, type) else type(obj)\n    return hasattr(cls, _FIELDS)\n\n\ndef asdict(obj, *, dict_factory=dict):\n    if not _is_dataclass_instance(obj):\n        raise TypeError(\"asdict() should be called on dataclass instances\")\n    return _asdict_inner(obj, dict_factory)\n\n\ndef _asdict_inner(obj, dict_factory):\n    if _is_dataclass_instance(obj):\n        return dict_factory([(f.name, _asdict_inner(getattr(obj, f.name), dict_factory)) for f in fields(obj)])\n    if isinstance(obj, tuple) and hasattr(obj, \"_fields\"):\n        return type(obj)(*[_asdict_inner(v, dict_factory) for v in obj])\n    if isinstance(obj, (list, tuple)):\n        return type(obj)(_asdict_inner(v, dict_factory) for v in obj)\n    if isinstance(obj, dict):\n        return type(obj)((_asdict_inner(k, dict_factory), _asdict_inner(v, dict_factory)) for k, v in obj.items())\n    return _copy.deepcopy(obj)\n\n\ndef astuple(obj, *, tuple_factory=tuple):\n    if not _is_dataclass_instance(obj):\n        raise TypeError(\"astuple() should be called on dataclass instances\")\n    return _astuple_inner(obj, tuple_factory)\n\n\ndef _astuple_inner(obj, tuple_factory):\n    if _is_dataclass_instance(obj):\n        return tuple_factory([_astuple_inner(getattr(obj, f.name), tuple_factory) for f in fields(obj)])\n    if isinstance(obj, tuple) and hasattr(obj, \"_fields\"):\n        return type(obj)(*[_astuple_inner(v, tuple_factory) for v in obj])\n    if isinstance(obj, (list, tuple)):\n        return type(obj)(_astuple_inner(v, tuple_factory) for v in obj)\n    if isinstance(obj, dict):\n        return type(obj)((_astuple_inner(k, tuple_factory), _astuple_inner(v, tuple_factory)) for k, v in obj.items())\n    return _copy.deepcopy(obj)\n\n\ndef replace(obj, /, **changes):\n    if not _is_dataclass_instance(obj):\n        raise TypeError(\"replace() should be called on dataclass instances\")\n    for f in getattr(obj, _FIELDS).values():\n        if f._field_type is _FIELD_CLASSVAR:\n            continue\n        if not f.init:\n            if f.name in changes:\n                raise ValueError(f\"field {f.name} is declared with init=False, it cannot be specified with replace()\")\n            continue\n        if f.name not in changes:\n            if f._field_type is _FIELD_INITVAR and f.default is MISSING:\n                raise TypeError(f\"InitVar {f.name!r} must be specified with replace()\")\n            changes[f.name] = getattr(obj, f.name)\n    return obj.__class__(**changes)\n\n\ndef make_dataclass(cls_name, fields, *, bases=(), namespace=None, init=True, repr=True, eq=True, order=False,\n                   unsafe_hash=False, frozen=False, match_args=True, kw_only=False, slots=False, weakref_slot=False, module=None):\n    namespace = dict(namespace or {})\n    annotations = {}\n    for item in fields:\n        if isinstance(item, str):\n            name, tp = item, \"typing.Any\"\n        elif len(item) == 2:\n            name, tp = item\n        else:\n            name, tp, spec = item\n            namespace[name] = spec\n        annotations[name] = tp\n    namespace[\"__annotations__\"] = annotations\n    cls = type(cls_name, bases, namespace)\n    if module is not None:\n        cls.__module__ = module\n    return dataclass(cls, init=init, repr=repr, eq=eq, order=order, unsafe_hash=unsafe_hash, frozen=frozen,\n                     match_args=match_args, kw_only=kw_only, slots=slots, weakref_slot=weakref_slot)\n";
    return s12;
  }
  if ( name == "decimal" ) {
    const s13 = "# SPDX-License-Identifier: MIT\n# decimal for the Ranger Python runtime: Decimal as sign, coefficient and\n# exponent, exact for +, -, * and comparisons. Division rounds through\n# float, and integers here are not unbounded, so precision is limited.\nimport math\n\n__all__ = [\"Decimal\", \"Context\", \"getcontext\", \"setcontext\", \"localcontext\", \"DecimalException\",\n           \"InvalidOperation\", \"DivisionByZero\", \"Inexact\", \"Rounded\", \"Overflow\", \"Underflow\",\n           \"ROUND_HALF_EVEN\", \"ROUND_HALF_UP\", \"ROUND_DOWN\", \"ROUND_UP\", \"ROUND_FLOOR\",\n           \"ROUND_CEILING\", \"ROUND_HALF_DOWN\", \"ROUND_05UP\", \"MAX_PREC\", \"MAX_EMAX\", \"MIN_EMIN\"]\n\nROUND_DOWN = \"ROUND_DOWN\"\nROUND_HALF_UP = \"ROUND_HALF_UP\"\nROUND_HALF_EVEN = \"ROUND_HALF_EVEN\"\nROUND_CEILING = \"ROUND_CEILING\"\nROUND_FLOOR = \"ROUND_FLOOR\"\nROUND_UP = \"ROUND_UP\"\nROUND_HALF_DOWN = \"ROUND_HALF_DOWN\"\nROUND_05UP = \"ROUND_05UP\"\nMAX_PREC = 999999999999999999\nMAX_EMAX = 999999999999999999\nMIN_EMIN = -999999999999999999\n\n\nclass DecimalException(ArithmeticError):\n    pass\n\n\nclass InvalidOperation(DecimalException):\n    pass\n\n\nclass DivisionByZero(DecimalException, ZeroDivisionError):\n    pass\n\n\nclass Inexact(DecimalException):\n    pass\n\n\nclass Rounded(DecimalException):\n    pass\n\n\nclass Overflow(Inexact, Rounded):\n    pass\n\n\nclass Underflow(Inexact, Rounded):\n    pass\n\n\nclass Context:\n    def __init__(self, prec=28, rounding=ROUND_HALF_EVEN, Emin=-999999, Emax=999999, capitals=1,\n                 clamp=0, flags=None, traps=None):\n        self.prec = prec\n        self.rounding = rounding\n        self.Emin = Emin\n        self.Emax = Emax\n        self.capitals = capitals\n        self.clamp = clamp\n        self.flags = {}\n        self.traps = {InvalidOperation: True, DivisionByZero: True, Overflow: True}\n\n    def copy(self):\n        c = Context(self.prec, self.rounding, self.Emin, self.Emax, self.capitals, self.clamp)\n        return c\n\n    def __repr__(self):\n        return \"Context(prec=%d, rounding=%s, Emin=%d, Emax=%d, capitals=%d, clamp=%d, flags=[], traps=[InvalidOperation, DivisionByZero, Overflow])\" % (\n            self.prec, self.rounding, self.Emin, self.Emax, self.capitals, self.clamp)\n\n\nDefaultContext = Context()\nBasicContext = Context(prec=9, rounding=ROUND_HALF_UP)\nExtendedContext = Context(prec=9)\n_current = [Context()]\n\n\ndef getcontext():\n    return _current[0]\n\n\ndef setcontext(ctx):\n    _current[0] = ctx\n\n\nclass _LocalContext:\n    def __init__(self, ctx, kw):\n        self.new = (ctx or getcontext()).copy()\n        for k, v in kw.items():\n            setattr(self.new, k, v)\n\n    def __enter__(self):\n        self.saved = getcontext()\n        setcontext(self.new)\n        return self.new\n\n    def __exit__(self, *exc):\n        setcontext(self.saved)\n\n\ndef localcontext(ctx=None, **kwargs):\n    return _LocalContext(ctx, kwargs)\n\n\nclass Decimal:\n    __slots__ = (\"_sign\", \"_int\", \"_exp\", \"_special\")\n\n    def __new__(cls, value=\"0\", context=None):\n        self = object.__new__(cls)\n        self._special = None\n        if isinstance(value, Decimal):\n            self._sign, self._int, self._exp, self._special = value._sign, value._int, value._exp, value._special\n            return self\n        if isinstance(value, bool) or isinstance(value, int):\n            self._sign = 1 if value < 0 else 0\n            self._int = abs(value)\n            self._exp = 0\n            return self\n        if isinstance(value, float):\n            if value != value:\n                self._sign, self._int, self._exp, self._special = 0, 0, 0, \"n\"\n                return self\n            if value in (float(\"inf\"), float(\"-inf\")):\n                self._sign, self._int, self._exp, self._special = (1 if value < 0 else 0), 0, 0, \"F\"\n                return self\n            return Decimal.__new__(cls, repr(value))\n        if isinstance(value, tuple):\n            sign, digits, exp = value\n            self._sign = sign\n            self._int = int(\"\".join(str(d) for d in digits) or \"0\")\n            if exp in (\"F\", \"n\", \"N\"):\n                self._special = exp\n                self._exp = 0\n            else:\n                self._exp = exp\n            return self\n        if not isinstance(value, str):\n            raise TypeError(\"conversion from %s to Decimal is not supported\" % type(value).__name__)\n        s = value.strip().replace(\"_\", \"\")\n        sign = 0\n        if s[:1] in \"+-\" and s:\n            sign = 1 if s[0] == \"-\" else 0\n            s = s[1:]\n        low = s.lower()\n        if low in (\"inf\", \"infinity\"):\n            self._sign, self._int, self._exp, self._special = sign, 0, 0, \"F\"\n            return self\n        if low in (\"nan\", \"snan\") or (low.startswith(\"nan\") and low[3:].isdigit()):\n            self._sign, self._int, self._exp, self._special = sign, 0, 0, \"n\"\n            return self\n        exp = 0\n        if \"e\" in low:\n            low, e = low.split(\"e\", 1)\n            try:\n                exp = int(e)\n            except ValueError:\n                raise InvalidOperation(\"[<class 'decimal.ConversionSyntax'>]\")\n        if \".\" in low:\n            a, b = low.split(\".\", 1)\n        else:\n            a, b = low, \"\"\n        digits = a + b\n        if not digits or not digits.isdigit():\n            raise InvalidOperation(\"[<class 'decimal.ConversionSyntax'>]\")\n        self._sign = sign\n        self._int = int(digits)\n        self._exp = exp - len(b)\n        return self\n\n    @classmethod\n    def from_float(cls, f):\n        return cls(f)\n\n    def _isnan(self):\n        return self._special == \"n\"\n\n    def is_nan(self):\n        return self._special in (\"n\", \"N\")\n\n    def is_infinite(self):\n        return self._special == \"F\"\n\n    def is_finite(self):\n        return self._special is None\n\n    def is_zero(self):\n        return self._special is None and self._int == 0\n\n    def is_signed(self):\n        return self._sign == 1\n\n    def _frac(self):\n        from fractions import Fraction\n        v = Fraction(self._int) * (Fraction(10) ** self._exp)\n        return -v if self._sign else v\n\n    def _digits(self):\n        return str(self._int)\n\n    def as_tuple(self):\n        return (self._sign, tuple(int(d) for d in self._digits()), self._exp if self._special is None else self._special)\n\n    def __repr__(self):\n        return \"Decimal('%s')\" % self\n\n    def __str__(self, eng=False, context=None):\n        sign = \"-\" if self._sign else \"\"\n        if self._special == \"F\":\n            return sign + \"Infinity\"\n        if self._special:\n            return sign + \"NaN\"\n        digits = self._digits()\n        exp = self._exp\n        leftdigits = exp + len(digits)\n        if exp <= 0 and leftdigits > -6:\n            dotplace = leftdigits\n        else:\n            dotplace = 1\n        if dotplace <= 0:\n            intpart = \"0\"\n            fracpart = \".\" + \"0\" * (-dotplace) + digits\n        elif dotplace >= len(digits):\n            intpart = digits + \"0\" * (dotplace - len(digits))\n            fracpart = \"\"\n        else:\n            intpart = digits[:dotplace]\n            fracpart = \".\" + digits[dotplace:]\n        if leftdigits == dotplace:\n            e = \"\"\n        else:\n            e = \"E%+d\" % (leftdigits - dotplace)\n        return sign + intpart + fracpart + e\n\n    def to_eng_string(self, context=None):\n        return str(self)\n\n    def _coerce(self, other):\n        if isinstance(other, Decimal):\n            return other\n        if isinstance(other, int):\n            return Decimal(other)\n        return None\n\n    def _from_frac(self, f, exact_exp=None):\n        if f.denominator == 1:\n            d = Decimal(int(f.numerator))\n            if exact_exp is not None and exact_exp < 0:\n                d._int = d._int * 10 ** (-exact_exp)\n                d._exp = exact_exp\n            return d\n        v = f.numerator / f.denominator\n        return Decimal(repr(v))\n\n    def __add__(self, other):\n        o = self._coerce(other)\n        if o is None:\n            return NotImplemented\n        if self._special or o._special:\n            return Decimal(float(self) + float(o))\n        e = min(self._exp, o._exp)\n        a = self._int * 10 ** (self._exp - e) * (-1 if self._sign else 1)\n        b = o._int * 10 ** (o._exp - e) * (-1 if o._sign else 1)\n        r = Decimal((1 if a + b < 0 else 0, tuple(int(c) for c in str(abs(a + b))), e))\n        if a + b == 0 and self._sign and o._sign:\n            r._sign = 1\n        return r\n\n    __radd__ = __add__\n\n    def __sub__(self, other):\n        o = self._coerce(other)\n        if o is None:\n            return NotImplemented\n        return self + (-o)\n\n    def __rsub__(self, other):\n        o = self._coerce(other)\n        if o is None:\n            return NotImplemented\n        return o - self\n\n    def __mul__(self, other):\n        o = self._coerce(other)\n        if o is None:\n            return NotImplemented\n        if self._special or o._special:\n            return Decimal(float(self) * float(o))\n        r = Decimal((self._sign ^ o._sign, tuple(int(c) for c in str(self._int * o._int)), self._exp + o._exp))\n        return r\n\n    __rmul__ = __mul__\n\n    def __truediv__(self, other):\n        o = self._coerce(other)\n        if o is None:\n            return NotImplemented\n        if o.is_zero():\n            if self.is_zero():\n                raise InvalidOperation(\"[<class 'decimal.DivisionUndefined'>]\")\n            raise DivisionByZero(\"[<class 'decimal.DivisionByZero'>]\")\n        f = self._frac() / o._frac()\n        return self._from_frac(f)\n\n    def __rtruediv__(self, other):\n        o = self._coerce(other)\n        if o is None:\n            return NotImplemented\n        return o / self\n\n    def __floordiv__(self, other):\n        o = self._coerce(other)\n        if o is None:\n            return NotImplemented\n        q = self._frac() / o._frac()\n        n = int(q)\n        return Decimal(n)\n\n    def __mod__(self, other):\n        o = self._coerce(other)\n        if o is None:\n            return NotImplemented\n        return self - o * (self // o)\n\n    def __pow__(self, other):\n        o = self._coerce(other)\n        if o is None:\n            return NotImplemented\n        if o._exp >= 0 and not o._special:\n            n = int(o)\n            if n >= 0:\n                r = Decimal(1)\n                for _ in range(n):\n                    r = r * self\n                return r\n            return Decimal(1) / (self ** -n)\n        return Decimal(float(self) ** float(o))\n\n    def __neg__(self):\n        r = Decimal(self)\n        r._sign = 1 - self._sign\n        return r\n\n    def __pos__(self):\n        return Decimal(self)\n\n    def __abs__(self):\n        r = Decimal(self)\n        r._sign = 0\n        return r\n\n    def _cmp(self, other):\n        o = self._coerce(other)\n        if o is None:\n            if isinstance(other, float):\n                o = Decimal(other)\n            else:\n                return None\n        if self._special or o._special:\n            a, b = float(self), float(o)\n            return (a > b) - (a < b)\n        a = self._frac()\n        b = o._frac()\n        return (a > b) - (a < b)\n\n    def __eq__(self, other):\n        if self._isnan():\n            return False\n        c = self._cmp(other)\n        return NotImplemented if c is None else c == 0\n\n    def __lt__(self, other):\n        c = self._cmp(other)\n        return NotImplemented if c is None else c < 0\n\n    def __le__(self, other):\n        c = self._cmp(other)\n        return NotImplemented if c is None else c <= 0\n\n    def __gt__(self, other):\n        c = self._cmp(other)\n        return NotImplemented if c is None else c > 0\n\n    def __ge__(self, other):\n        c = self._cmp(other)\n        return NotImplemented if c is None else c >= 0\n\n    def __hash__(self):\n        if self._special:\n            return hash(float(self))\n        f = self._frac()\n        if f.denominator == 1:\n            return hash(f.numerator)\n        return hash(float(self))\n\n    def __bool__(self):\n        return self._special is not None or self._int != 0\n\n    def __float__(self):\n        if self._special == \"F\":\n            return float(\"-inf\") if self._sign else float(\"inf\")\n        if self._special:\n            return float(\"nan\")\n        return float(str(self))\n\n    def __int__(self):\n        if self._special:\n            raise ValueError(\"cannot convert NaN to integer\") if self._special == \"n\" else OverflowError(\"cannot convert Infinity to integer\")\n        if self._exp >= 0:\n            v = self._int * 10 ** self._exp\n        else:\n            v = self._int // 10 ** (-self._exp)\n        return -v if self._sign else v\n\n    __trunc__ = __int__\n\n    def __round__(self, n=None):\n        if n is None:\n            return round(self._frac())\n        return self.quantize(Decimal((0, (1,), -n)))\n\n    def __floor__(self):\n        return math.floor(self._frac())\n\n    def __ceil__(self):\n        return math.ceil(self._frac())\n\n    def quantize(self, exp, rounding=None, context=None):\n        e = exp._exp\n        f = self._frac() * (10 ** -e) if e <= 0 else self._frac() / (10 ** e)\n        n = round(f) if (rounding or getcontext().rounding) == ROUND_HALF_EVEN else int(f + (f > 0) * 0.5 - (f < 0) * 0.5)\n        r = Decimal((1 if n < 0 or (n == 0 and self._sign) else 0, tuple(int(c) for c in str(abs(n))), e))\n        return r\n\n    def normalize(self, context=None):\n        if self._special or self._int == 0:\n            return Decimal((self._sign, (0,), 0)) if not self._special else Decimal(self)\n        i, e = self._int, self._exp\n        while i % 10 == 0:\n            i //= 10\n            e += 1\n        return Decimal((self._sign, tuple(int(c) for c in str(i)), e))\n\n    def sqrt(self, context=None):\n        return Decimal(repr(math.sqrt(float(self))))\n\n    def exp(self, context=None):\n        return Decimal(repr(math.exp(float(self))))\n\n    def ln(self, context=None):\n        return Decimal(repr(math.log(float(self))))\n\n    def log10(self, context=None):\n        return Decimal(repr(math.log10(float(self))))\n\n    def adjusted(self):\n        return self._exp + len(self._digits()) - 1\n\n    def copy_sign(self, other):\n        r = Decimal(self)\n        r._sign = other._sign\n        return r\n\n    def copy_abs(self):\n        return abs(self)\n\n    def copy_negate(self):\n        return -self\n\n    def as_integer_ratio(self):\n        f = self._frac()\n        return (f.numerator, f.denominator)\n\n    def __format__(self, spec):\n        if not spec:\n            return str(self)\n        return format(float(self), spec)\n\n    def __reduce__(self):\n        return (Decimal, (str(self),))\n\n    def __copy__(self):\n        return self\n\n    def __deepcopy__(self, memo):\n        return self\n";
    return s13;
  }
  if ( name == "doctest" ) {
    const s14 = "# SPDX-License-Identifier: MIT\n# doctest stand-in for the Ranger Python runtime: the names tests import,\n# with no examples run.\nimport unittest\n\nOPTIONFLAGS_BY_NAME = {}\nELLIPSIS = 8\nNORMALIZE_WHITESPACE = 4\nIGNORE_EXCEPTION_DETAIL = 32\nDONT_ACCEPT_TRUE_FOR_1 = 1\n\n\ndef DocTestSuite(*args, **kwargs):\n    return unittest.TestSuite()\n\n\ndef DocFileSuite(*args, **kwargs):\n    return unittest.TestSuite()\n\n\ndef testmod(*args, **kwargs):\n    return (0, 0)\n";
    return s14;
  }
  if ( name == "enum" ) {
    const s15 = "# SPDX-License-Identifier: MIT\n# enum for the Ranger Python runtime: Enum, IntEnum, StrEnum, Flag,\n# IntFlag, auto and unique, on a metaclass as in CPython.\n\n__all__ = [\"EnumType\", \"EnumMeta\", \"Enum\", \"IntEnum\", \"StrEnum\", \"Flag\", \"IntFlag\", \"ReprEnum\",\n           \"auto\", \"unique\", \"property\", \"verify\", \"member\", \"nonmember\", \"global_enum\",\n           \"FlagBoundary\", \"STRICT\", \"CONFORM\", \"EJECT\", \"KEEP\", \"EnumCheck\", \"CONTINUOUS\",\n           \"NAMED_FLAGS\", \"UNIQUE\", \"pickle_by_enum_name\", \"pickle_by_global_name\"]\n\n_builtin_property = property\nEnum = Flag = ReprEnum = IntEnum = IntFlag = StrEnum = None\n\n\nclass auto:\n    _counter = 0\n\n    def __init__(self, value=None):\n        self.value = value if value is not None else _auto_null\n        auto._counter += 1\n        self._order = auto._counter\n\n    def __repr__(self):\n        return \"auto(%r)\" % (self.value,)\n\n\nclass _AutoNull:\n    def __repr__(self):\n        return \"_auto_null\"\n\n\n_auto_null = _AutoNull()\n\n\nclass member:\n    def __init__(self, value):\n        self.value = value\n\n\nclass nonmember:\n    def __init__(self, value):\n        self.value = value\n\n\nproperty = _builtin_property\n\n\ndef _is_descriptor(obj):\n    return hasattr(obj, \"__get__\") or hasattr(obj, \"__set__\") or hasattr(obj, \"__delete__\")\n\n\ndef _is_dunder(name):\n    return len(name) > 4 and name[:2] == name[-2:] == \"__\" and name[2] != \"_\" and name[-3] != \"_\"\n\n\ndef _is_sunder(name):\n    return len(name) > 2 and name[0] == name[-1] == \"_\" and name[1] != \"_\" and name[-2] != \"_\"\n\n\ndef _is_private(cls_name, name):\n    pattern = \"_%s__\" % (cls_name,)\n    return len(name) > len(pattern) and name.startswith(pattern) and not name.endswith(\"__\")\n\n\nclass FlagBoundary:\n    pass\n\n\nSTRICT = \"strict\"\nCONFORM = \"conform\"\nEJECT = \"eject\"\nKEEP = \"keep\"\n\n\nclass EnumCheck:\n    pass\n\n\nCONTINUOUS = \"no skipped integer values\"\nNAMED_FLAGS = \"multi-flag aliases may not contain unnamed flags\"\nUNIQUE = \"one name per value\"\n\n\ndef _member_names_of(ns):\n    return [k for k in ns if not (_is_dunder(k) or _is_sunder(k)) and not _is_descriptor(ns[k]) and not callable(ns[k]) or isinstance(ns.get(k), (auto, member))]\n\n\nclass EnumDict(dict):\n    def __init__(self):\n        super().__init__()\n        self._member_names = []\n        self._last_values = []\n        self._cls_name = None\n        self._generate = None\n\n    def __setitem__(self, key, value):\n        if _is_private(self._cls_name or \"\", key):\n            pass\n        elif _is_sunder(key):\n            if key == \"_generate_next_value_\":\n                self._generate = value\n            elif key == \"_ignore_\":\n                pass\n        elif _is_dunder(key):\n            if key == \"__order__\":\n                key = \"_order_\"\n        elif key in self._member_names:\n            raise TypeError(\"%r already defined as %r\" % (key, self[key]))\n        elif isinstance(value, nonmember):\n            value = value.value\n        elif isinstance(value, member) or not (_is_descriptor(value) or callable(value) and not isinstance(value, auto)) or isinstance(value, auto):\n            if isinstance(value, member):\n                value = value.value\n            if key in self:\n                raise TypeError(\"%r already defined as %r\" % (key, self[key]))\n            if isinstance(value, auto):\n                if value.value is _auto_null:\n                    gen = self._generate or _default_generate\n                    value.value = gen(key, 1, len(self._member_names), list(self._last_values))\n                value = value.value\n            elif isinstance(value, tuple) and any(isinstance(v, auto) for v in value):\n                vals = []\n                for v in value:\n                    if isinstance(v, auto):\n                        if v.value is _auto_null:\n                            gen = self._generate or _default_generate\n                            v.value = gen(key, 1, len(self._member_names), list(self._last_values))\n                        v = v.value\n                    vals.append(v)\n                value = tuple(vals)\n            self._member_names.append(key)\n            self._last_values.append(value)\n        super().__setitem__(key, value)\n\n\n_EnumDict = EnumDict\n\n\ndef _default_generate(name, start, count, last_values):\n    for v in reversed(last_values):\n        try:\n            return v + 1\n        except TypeError:\n            pass\n    return start\n\n\nclass EnumType(type):\n    @classmethod\n    def __prepare__(metacls, cls, bases, **kwds):\n        d = EnumDict()\n        d._cls_name = cls\n        for b in bases:\n            g = getattr(b, \"_generate_next_value_\", None)\n            if g is not None and not isinstance(b, EnumType) or (g is not None and b.__name__ not in (\"Enum\",)):\n                gf = getattr(g, \"__func__\", g)\n                d._generate = gf\n                break\n        return d\n\n    def __new__(metacls, cls, bases, classdict, *, boundary=None, _simple=False, **kwds):\n        if not isinstance(classdict, EnumDict):\n            ed = EnumDict()\n            ed._cls_name = cls\n            for k, v in classdict.items():\n                ed[k] = v\n            classdict = ed\n        member_names = list(classdict._member_names)\n        members = {k: classdict[k] for k in member_names}\n        ns = {k: v for k, v in classdict.items() if k not in member_names}\n        member_type, first_enum = metacls._get_mixins_(cls, bases)\n        ns[\"_member_names_\"] = []\n        ns[\"_member_map_\"] = {}\n        ns[\"_value2member_map_\"] = {}\n        ns[\"_member_type_\"] = member_type\n        ns[\"_unhashable_values_\"] = []\n        if \"_generate_next_value_\" in classdict:\n            ns[\"_generate_next_value_\"] = staticmethod(classdict._generate) if not isinstance(classdict[\"_generate_next_value_\"], staticmethod) else classdict[\"_generate_next_value_\"]\n        if members and first_enum is not None and first_enum._member_names_:\n            raise TypeError(\"%s: cannot extend %r\" % (cls, first_enum))\n        enum_class = super().__new__(metacls, cls, bases, ns, **kwds)\n        enum_class._new_member_ = ns.get(\"__new__\") if \"__new__\" in ns else None\n        for name in member_names:\n            value = members[name]\n            enum_class._add_member_(name, value)\n        if Flag is not None and issubclass(enum_class, Flag):\n            enum_class._flag_mask_ = 0\n            for m in enum_class._member_map_.values():\n                if isinstance(m._value_, int):\n                    enum_class._flag_mask_ |= m._value_\n            enum_class._all_bits_ = 2 ** (enum_class._flag_mask_.bit_length()) - 1 if enum_class._flag_mask_ else 0\n            enum_class._boundary_ = boundary or getattr(first_enum, \"_boundary_\", None)\n        return enum_class\n\n    def _add_member_(cls, name, value):\n        args = value if isinstance(value, tuple) else (value,)\n        mtype = cls._member_type_\n        new = cls.__dict__.get(\"_new_member_\")\n        custom_new = None\n        for base in cls.__mro__:\n            if \"__new__\" in base.__dict__ and base not in (Enum, ReprEnum, Flag, IntEnum, IntFlag, StrEnum) and base is not object and base is not mtype and isinstance(base, EnumType):\n                custom_new = base.__dict__[\"__new__\"]\n                break\n        if custom_new is not None:\n            fn = getattr(custom_new, \"__func__\", custom_new)\n            m = fn(cls, *args)\n            if not hasattr(m, \"_value_\"):\n                if mtype is object:\n                    m._value_ = value\n                else:\n                    try:\n                        m._value_ = mtype(*args)\n                    except Exception:\n                        m._value_ = value\n        elif mtype is object:\n            m = object.__new__(cls)\n            m._value_ = value\n        else:\n            if mtype is str and not isinstance(value, tuple):\n                args = (value,)\n            m = mtype.__new__(cls, *args)\n            m._value_ = mtype(*args) if not hasattr(m, \"_value_\") else m._value_\n        value = m._value_\n        m._name_ = name\n        m.__objclass__ = cls\n        m._sort_order_ = len(cls._member_names_)\n        init = None\n        for base in cls.__mro__:\n            if \"__init__\" in base.__dict__ and isinstance(base, EnumType) and base not in (Enum, Flag):\n                init = base.__dict__[\"__init__\"]\n                break\n        if init is not None:\n            init(m, *args)\n        # an alias names an existing member with the same value\n        for other in cls._member_map_.values():\n            if other._value_ == value and type(other._value_) is type(value) or (other._value_ == value and not (isinstance(value, bool) != isinstance(other._value_, bool))):\n                cls._member_map_[name] = other\n                type.__setattr__(cls, name, other)\n                return\n        is_flag_alias = False\n        if Flag is not None and issubclass(cls, Flag) and isinstance(value, int):\n            if value != 0 and value & (value - 1) != 0:\n                is_flag_alias = True\n        if not is_flag_alias or True:\n            cls._member_names_.append(name)\n        cls._member_map_[name] = m\n        type.__setattr__(cls, name, m)\n        try:\n            cls._value2member_map_[value] = m\n        except TypeError:\n            cls._unhashable_values_.append(value)\n\n    @classmethod\n    def _get_mixins_(metacls, class_name, bases):\n        if not bases:\n            return object, Enum\n        first_enum = None\n        member_type = object\n        for chain in bases:\n            for base in chain.__mro__:\n                if base is object:\n                    continue\n                if isinstance(base, EnumType):\n                    if first_enum is None:\n                        first_enum = base\n                    if base._member_type_ is not object:\n                        member_type = base._member_type_\n                        break\n                elif member_type is object:\n                    member_type = base\n                    break\n        return member_type, first_enum\n\n    def __call__(cls, value=None, names=None, *values, module=None, qualname=None, type=None, start=1, boundary=None):\n        if names is None:\n            if isinstance(value, cls):\n                return value\n            return cls.__new__member_lookup(value)\n        return cls._create_(value, names, module=module, qualname=qualname, type=type, start=start, boundary=boundary)\n\n    def _EnumType__new__member_lookup(cls, value):\n        return cls._lookup_(value)\n\n    def __new__member_lookup(cls, value):\n        return cls._lookup_(value)\n\n    def _lookup_(cls, value):\n        try:\n            if value in cls._value2member_map_:\n                return cls._value2member_map_[value]\n        except TypeError:\n            pass\n        for m in cls._member_map_.values():\n            if m._value_ == value:\n                return m\n        if Flag is not None and issubclass(cls, Flag) and isinstance(value, int):\n            return cls._missing_flag_(value)\n        result = cls._missing_(value)\n        if isinstance(result, cls):\n            return result\n        if result is None:\n            raise ValueError(\"%r is not a valid %s\" % (value, cls.__qualname__))\n        raise TypeError(\"error in %s._missing_: returned %r instead of None or a valid member\" % (cls.__name__, result))\n\n    def _create_(cls, class_name, names, *, module=None, qualname=None, type=None, start=1, boundary=None):\n        metacls = cls.__class__\n        bases = (cls,) if type is None else (type, cls)\n        classdict = metacls.__prepare__(class_name, bases)\n        if isinstance(names, str):\n            names = names.replace(\",\", \" \").split()\n        if isinstance(names, (tuple, list)) and names and isinstance(names[0], str):\n            original_names, names = names, []\n            last_values = []\n            for count, name in enumerate(original_names):\n                value = cls._generate_next_value_(name, start, count, last_values[:])\n                last_values.append(value)\n                names.append((name, value))\n        if isinstance(names, dict):\n            names = names.items()\n        for item in names:\n            if isinstance(item, str):\n                member_name, member_value = item, names[item]\n            else:\n                member_name, member_value = item\n            classdict[member_name] = member_value\n        classdict[\"__module__\"] = module or \"__main__\"\n        if qualname is not None:\n            classdict[\"__qualname__\"] = qualname\n        return metacls.__new__(metacls, class_name, bases, classdict, boundary=boundary)\n\n    def __contains__(cls, value):\n        if isinstance(value, cls):\n            return True\n        try:\n            return value in cls._value2member_map_ or value in cls._unhashable_values_\n        except TypeError:\n            return False\n\n    def __delattr__(cls, attr):\n        if attr in cls._member_map_:\n            raise AttributeError(\"%r cannot delete member %r.\" % (cls.__name__, attr))\n        super().__delattr__(attr)\n\n    def __getitem__(cls, name):\n        return cls._member_map_[name]\n\n    def __iter__(cls):\n        return (cls._member_map_[name] for name in cls._member_names_ if not cls._is_multi_flag(name))\n\n    def _is_multi_flag(cls, name):\n        if Flag is None or not issubclass(cls, Flag):\n            return False\n        v = cls._member_map_[name]._value_\n        return isinstance(v, int) and v != 0 and v & (v - 1) != 0 and False\n\n    def __reversed__(cls):\n        return reversed(list(cls))\n\n    def __len__(cls):\n        return len(cls._member_names_)\n\n    def __bool__(cls):\n        return True\n\n    def __repr__(cls):\n        if Flag is not None and issubclass(cls, Flag):\n            return \"<flag %r>\" % cls.__name__\n        return \"<enum %r>\" % cls.__name__\n\n    @_builtin_property\n    def __members__(cls):\n        import types\n        return types.MappingProxyType(cls._member_map_) if hasattr(types, \"MappingProxyType\") else dict(cls._member_map_)\n\n    def __setattr__(cls, name, value):\n        member_map = cls.__dict__.get(\"_member_map_\", {})\n        if name in member_map:\n            raise AttributeError(\"cannot reassign member %r\" % (name,))\n        super().__setattr__(name, value)\n\n    def __getattr__(cls, name):\n        if _is_dunder(name):\n            raise AttributeError(name)\n        try:\n            return cls.__dict__[\"_member_map_\"][name]\n        except KeyError:\n            raise AttributeError(name) from None\n\n\nEnumMeta = EnumType\nFlag = None\n\n\nclass Enum(metaclass=EnumType):\n    @classmethod\n    def _generate_next_value_(cls_or_name, *args):\n        # called as name, start, count, last_values\n        if len(args) == 3:\n            name, (start, count, last_values) = cls_or_name, args\n        else:\n            name, start, count, last_values = args\n        return _default_generate(name, start, count, last_values)\n\n    def __new__(cls, value):\n        return cls._lookup_(value)\n\n    def __init__(self, *args, **kwds):\n        pass\n\n    @classmethod\n    def _missing_(cls, value):\n        return None\n\n    def __repr__(self):\n        v_repr = self.__class__._value_repr_ if hasattr(self.__class__, \"_value_repr_\") else repr\n        return \"<%s.%s: %s>\" % (self.__class__.__name__, self._name_, v_repr(self._value_))\n\n    def __str__(self):\n        return \"%s.%s\" % (self.__class__.__name__, self._name_)\n\n    def __dir__(self):\n        return [\"__class__\", \"__doc__\", \"__eq__\", \"__hash__\", \"__module__\", \"name\", \"value\"]\n\n    def __format__(self, format_spec):\n        return str.__format__(str(self), format_spec)\n\n    def __hash__(self):\n        return hash(self._name_)\n\n    def __reduce_ex__(self, proto):\n        return self.__class__, (self._value_,)\n\n    def __deepcopy__(self, memo):\n        return self\n\n    def __copy__(self):\n        return self\n\n    def __eq__(self, other):\n        return self is other\n\n    def __ne__(self, other):\n        return self is not other\n\n    @_builtin_property\n    def name(self):\n        return self._name_\n\n    @_builtin_property\n    def value(self):\n        return self._value_\n\n\nclass ReprEnum(Enum):\n    pass\n\n\nclass IntEnum(int, ReprEnum):\n    def __str__(self):\n        return int.__repr__(self._value_)\n\n    def __format__(self, spec):\n        return format(self._value_, spec)\n\n    def __eq__(self, other):\n        return int(self._value_) == other\n\n    def __ne__(self, other):\n        return int(self._value_) != other\n\n    def __hash__(self):\n        return hash(self._value_)\n\n    def __lt__(self, other):\n        return int(self._value_) < other\n\n    def __le__(self, other):\n        return int(self._value_) <= other\n\n    def __gt__(self, other):\n        return int(self._value_) > other\n\n    def __ge__(self, other):\n        return int(self._value_) >= other\n\n\nclass StrEnum(str, ReprEnum):\n    def __new__(cls, *values):\n        if len(values) > 3:\n            raise TypeError(\"too many arguments for str(): %r\" % (values,))\n        if len(values) == 1 and not isinstance(values[0], str):\n            raise TypeError(\"%r is not a string\" % (values[0],))\n        value = str(*values)\n        member = str.__new__(cls, value)\n        member._value_ = value\n        return member\n\n    def __str__(self):\n        return self._value_\n\n    def __format__(self, spec):\n        return format(self._value_, spec)\n\n    def __eq__(self, other):\n        return str(self._value_) == other\n\n    def __ne__(self, other):\n        return str(self._value_) != other\n\n    def __hash__(self):\n        return hash(self._value_)\n\n    @staticmethod\n    def _generate_next_value_(name, start, count, last_values):\n        return name.lower()\n\n\ndef _high_bit(value):\n    return value.bit_length() - 1\n\n\nclass Flag(Enum):\n    _numeric_repr_ = repr\n\n    @staticmethod\n    def _generate_next_value_(name, start, count, last_values):\n        if not count:\n            return start if start is not None else 1\n        last_value = max(last_values)\n        try:\n            high_bit = _high_bit(last_value)\n        except Exception:\n            raise TypeError(\"invalid flag value %r\" % last_value) from None\n        return 2 ** (high_bit + 1)\n\n    @classmethod\n    def _missing_flag_(cls, value):\n        if not isinstance(value, int):\n            raise ValueError(\"%r is not a valid %s\" % (value, cls.__qualname__))\n        if value < 0:\n            value = cls._all_bits_ + 1 + value if cls._all_bits_ else value\n        pseudo = object.__new__(cls) if cls._member_type_ is object else cls._member_type_.__new__(cls, value)\n        pseudo._value_ = value\n        names = []\n        for m in cls._member_map_.values():\n            v = m._value_\n            if v and v & (v - 1) == 0 and value & v == v:\n                names.append(m._name_)\n        pseudo._name_ = \"|\".join(names) if names else None\n        cls._value2member_map_[value] = pseudo\n        return pseudo\n\n    def __repr__(self):\n        name = self._name_\n        if name is None:\n            return \"<%s: %r>\" % (self.__class__.__name__, self._value_)\n        return \"<%s.%s: %r>\" % (self.__class__.__name__, name, self._value_)\n\n    def __str__(self):\n        if self._name_ is None:\n            return \"%s(%r)\" % (self.__class__.__name__, self._value_)\n        return \"%s.%s\" % (self.__class__.__name__, self._name_)\n\n    def __bool__(self):\n        return bool(self._value_)\n\n    def __contains__(self, other):\n        if not isinstance(other, self.__class__):\n            raise TypeError(\"unsupported operand type(s) for 'in': %r and %r\" % (type(other).__qualname__, self.__class__.__qualname__))\n        return other._value_ & self._value_ == other._value_\n\n    def __iter__(self):\n        for m in self.__class__._member_map_.values():\n            v = m._value_\n            if v and v & (v - 1) == 0 and self._value_ & v == v and m._name_ in self.__class__._member_names_:\n                yield m\n\n    def __len__(self):\n        return bin(self._value_).count(\"1\")\n\n    def __hash__(self):\n        return hash(self._value_)\n\n    def __eq__(self, other):\n        if isinstance(other, Flag):\n            return self._value_ == other._value_ and type(self) is type(other)\n        return NotImplemented\n\n    def __or__(self, other):\n        if isinstance(other, self.__class__):\n            other = other._value_\n        elif self._member_type_ is not object and isinstance(other, self._member_type_):\n            pass\n        else:\n            return NotImplemented\n        return self.__class__(self._value_ | other)\n\n    def __and__(self, other):\n        if isinstance(other, self.__class__):\n            other = other._value_\n        elif self._member_type_ is not object and isinstance(other, self._member_type_):\n            pass\n        else:\n            return NotImplemented\n        return self.__class__(self._value_ & other)\n\n    def __xor__(self, other):\n        if isinstance(other, self.__class__):\n            other = other._value_\n        elif self._member_type_ is not object and isinstance(other, self._member_type_):\n            pass\n        else:\n            return NotImplemented\n        return self.__class__(self._value_ ^ other)\n\n    def __invert__(self):\n        return self.__class__(self.__class__._flag_mask_ ^ self._value_)\n\n    __ror__ = __or__\n    __rand__ = __and__\n    __rxor__ = __xor__\n\n\nclass IntFlag(int, ReprEnum, Flag):\n    def __str__(self):\n        return int.__repr__(self._value_)\n\n    def __format__(self, spec):\n        return format(self._value_, spec)\n\n    def __eq__(self, other):\n        return int(self._value_) == other\n\n    def __hash__(self):\n        return hash(self._value_)\n\n    def __or__(self, other):\n        if isinstance(other, int):\n            other = int(other)\n            return self.__class__(self._value_ | other)\n        return NotImplemented\n\n    def __and__(self, other):\n        if isinstance(other, int):\n            return self.__class__(self._value_ & int(other))\n        return NotImplemented\n\n    def __xor__(self, other):\n        if isinstance(other, int):\n            return self.__class__(self._value_ ^ int(other))\n        return NotImplemented\n\n    __ror__ = __or__\n    __rand__ = __and__\n    __rxor__ = __xor__\n\n\ndef unique(enumeration):\n    duplicates = []\n    for name, member in enumeration.__members__.items():\n        if name != member.name:\n            duplicates.append((name, member.name))\n    if duplicates:\n        alias_details = \", \".join([\"%s -> %s\" % (alias, name) for (alias, name) in duplicates])\n        raise ValueError(\"duplicate values found in %r: %s\" % (enumeration, alias_details))\n    return enumeration\n\n\ndef verify(*checks):\n    def decorator(enumeration):\n        return enumeration\n    return decorator\n\n\ndef global_enum(cls, update_str=False):\n    import sys\n    g = sys.modules[cls.__module__].__dict__ if cls.__module__ in sys.modules else {}\n    g.update(cls.__members__)\n    return cls\n\n\ndef pickle_by_enum_name(self, proto):\n    return getattr, (self.__class__, self._name_)\n\n\ndef pickle_by_global_name(self, proto):\n    return self.name\n\n\ndef _simple_enum(etype=Enum, *, boundary=None, use_args=None):\n    def convert(cls):\n        return cls\n    return convert\n";
    return s15;
  }
  if ( name == "errno" ) {
    const s16 = "# SPDX-License-Identifier: MIT\n# errno for the Ranger Python runtime: the Linux numbers\n\nEPERM = 1\nENOENT = 2\nESRCH = 3\nEINTR = 4\nEIO = 5\nENXIO = 6\nE2BIG = 7\nENOEXEC = 8\nEBADF = 9\nECHILD = 10\nEAGAIN = 11\nENOMEM = 12\nEACCES = 13\nEFAULT = 14\nENOTBLK = 15\nEBUSY = 16\nEEXIST = 17\nEXDEV = 18\nENODEV = 19\nENOTDIR = 20\nEISDIR = 21\nEINVAL = 22\nENFILE = 23\nEMFILE = 24\nENOTTY = 25\nETXTBSY = 26\nEFBIG = 27\nENOSPC = 28\nESPIPE = 29\nEROFS = 30\nEMLINK = 31\nEPIPE = 32\nEDOM = 33\nERANGE = 34\nEDEADLOCK = 35\nENAMETOOLONG = 36\nENOLCK = 37\nENOSYS = 38\nENOTEMPTY = 39\nELOOP = 40\nENOMSG = 42\nEIDRM = 43\nECHRNG = 44\nEL2NSYNC = 45\nEL3HLT = 46\nEL3RST = 47\nELNRNG = 48\nEUNATCH = 49\nENOCSI = 50\nEL2HLT = 51\nEBADE = 52\nEBADR = 53\nEXFULL = 54\nENOANO = 55\nEBADRQC = 56\nEBADSLT = 57\nEBFONT = 59\nENOSTR = 60\nENODATA = 61\nETIME = 62\nENOSR = 63\nENONET = 64\nENOPKG = 65\nEREMOTE = 66\nENOLINK = 67\nEADV = 68\nESRMNT = 69\nECOMM = 70\nEPROTO = 71\nEMULTIHOP = 72\nEDOTDOT = 73\nEBADMSG = 74\nEOVERFLOW = 75\nENOTUNIQ = 76\nEBADFD = 77\nEREMCHG = 78\nELIBACC = 79\nELIBBAD = 80\nELIBSCN = 81\nELIBMAX = 82\nELIBEXEC = 83\nEILSEQ = 84\nERESTART = 85\nESTRPIPE = 86\nEUSERS = 87\nENOTSOCK = 88\nEDESTADDRREQ = 89\nEMSGSIZE = 90\nEPROTOTYPE = 91\nENOPROTOOPT = 92\nEPROTONOSUPPORT = 93\nESOCKTNOSUPPORT = 94\nENOTSUP = 95\nEPFNOSUPPORT = 96\nEAFNOSUPPORT = 97\nEADDRINUSE = 98\nEADDRNOTAVAIL = 99\nENETDOWN = 100\nENETUNREACH = 101\nENETRESET = 102\nECONNABORTED = 103\nECONNRESET = 104\nENOBUFS = 105\nEISCONN = 106\nENOTCONN = 107\nESHUTDOWN = 108\nETOOMANYREFS = 109\nETIMEDOUT = 110\nECONNREFUSED = 111\nEHOSTDOWN = 112\nEHOSTUNREACH = 113\nEALREADY = 114\nEINPROGRESS = 115\nESTALE = 116\nEUCLEAN = 117\nENOTNAM = 118\nENAVAIL = 119\nEISNAM = 120\nEREMOTEIO = 121\nEDQUOT = 122\nENOMEDIUM = 123\nEMEDIUMTYPE = 124\nECANCELED = 125\nENOKEY = 126\nEKEYEXPIRED = 127\nEKEYREVOKED = 128\nEKEYREJECTED = 129\nEOWNERDEAD = 130\nENOTRECOVERABLE = 131\nERFKILL = 132\n\nerrorcode = {1: 'EPERM', 2: 'ENOENT', 3: 'ESRCH', 4: 'EINTR', 5: 'EIO', 6: 'ENXIO', 7: 'E2BIG', 8: 'ENOEXEC', 9: 'EBADF', 10: 'ECHILD', 11: 'EAGAIN', 12: 'ENOMEM', 13: 'EACCES', 14: 'EFAULT', 15: 'ENOTBLK', 16: 'EBUSY', 17: 'EEXIST', 18: 'EXDEV', 19: 'ENODEV', 20: 'ENOTDIR', 21: 'EISDIR', 22: 'EINVAL', 23: 'ENFILE', 24: 'EMFILE', 25: 'ENOTTY', 26: 'ETXTBSY', 27: 'EFBIG', 28: 'ENOSPC', 29: 'ESPIPE', 30: 'EROFS', 31: 'EMLINK', 32: 'EPIPE', 33: 'EDOM', 34: 'ERANGE', 35: 'EDEADLOCK', 36: 'ENAMETOOLONG', 37: 'ENOLCK', 38: 'ENOSYS', 39: 'ENOTEMPTY', 40: 'ELOOP', 42: 'ENOMSG', 43: 'EIDRM', 44: 'ECHRNG', 45: 'EL2NSYNC', 46: 'EL3HLT', 47: 'EL3RST', 48: 'ELNRNG', 49: 'EUNATCH', 50: 'ENOCSI', 51: 'EL2HLT', 52: 'EBADE', 53: 'EBADR', 54: 'EXFULL', 55: 'ENOANO', 56: 'EBADRQC', 57: 'EBADSLT', 59: 'EBFONT', 60: 'ENOSTR', 61: 'ENODATA', 62: 'ETIME', 63: 'ENOSR', 64: 'ENONET', 65: 'ENOPKG', 66: 'EREMOTE', 67: 'ENOLINK', 68: 'EADV', 69: 'ESRMNT', 70: 'ECOMM', 71: 'EPROTO', 72: 'EMULTIHOP', 73: 'EDOTDOT', 74: 'EBADMSG', 75: 'EOVERFLOW', 76: 'ENOTUNIQ', 77: 'EBADFD', 78: 'EREMCHG', 79: 'ELIBACC', 80: 'ELIBBAD', 81: 'ELIBSCN', 82: 'ELIBMAX', 83: 'ELIBEXEC', 84: 'EILSEQ', 85: 'ERESTART', 86: 'ESTRPIPE', 87: 'EUSERS', 88: 'ENOTSOCK', 89: 'EDESTADDRREQ', 90: 'EMSGSIZE', 91: 'EPROTOTYPE', 92: 'ENOPROTOOPT', 93: 'EPROTONOSUPPORT', 94: 'ESOCKTNOSUPPORT', 95: 'ENOTSUP', 96: 'EPFNOSUPPORT', 97: 'EAFNOSUPPORT', 98: 'EADDRINUSE', 99: 'EADDRNOTAVAIL', 100: 'ENETDOWN', 101: 'ENETUNREACH', 102: 'ENETRESET', 103: 'ECONNABORTED', 104: 'ECONNRESET', 105: 'ENOBUFS', 106: 'EISCONN', 107: 'ENOTCONN', 108: 'ESHUTDOWN', 109: 'ETOOMANYREFS', 110: 'ETIMEDOUT', 111: 'ECONNREFUSED', 112: 'EHOSTDOWN', 113: 'EHOSTUNREACH', 114: 'EALREADY', 115: 'EINPROGRESS', 116: 'ESTALE', 117: 'EUCLEAN', 118: 'ENOTNAM', 119: 'ENAVAIL', 120: 'EISNAM', 121: 'EREMOTEIO', 122: 'EDQUOT', 123: 'ENOMEDIUM', 124: 'EMEDIUMTYPE', 125: 'ECANCELED', 126: 'ENOKEY', 127: 'EKEYEXPIRED', 128: 'EKEYREVOKED', 129: 'EKEYREJECTED', 130: 'EOWNERDEAD', 131: 'ENOTRECOVERABLE', 132: 'ERFKILL'}\n";
    return s16;
  }
  if ( name == "fractions" ) {
    const s17 = "# SPDX-License-Identifier: MIT\n# fractions for the Ranger Python runtime\nimport math\nimport numbers\nimport re\n\n__all__ = [\"Fraction\"]\n\n_RATIONAL_FORMAT = re.compile(r\"\"\"\n    \\A\\s*\n    (?P<sign>[-+]?)\n    (?=\\d|\\.\\d)\n    (?P<num>\\d*|\\d+(_\\d+)*)\n    (?:\n        (?:\\s*/\\s*(?P<denom>\\d+(_\\d+)*))?\n        |\n        (?:\\.(?P<decimal>\\d*|\\d+(_\\d+)*))?\n        (?:E(?P<exp>[-+]?\\d+(_\\d+)*))?\n    )\n    \\s*\\Z\n\"\"\", re.VERBOSE | re.IGNORECASE)\n\n\nclass Fraction:\n    __slots__ = (\"_numerator\", \"_denominator\")\n\n    def __new__(cls, numerator=0, denominator=None):\n        self = object.__new__(cls)\n        if denominator is None:\n            if type(numerator) is int:\n                self._numerator = numerator\n                self._denominator = 1\n                return self\n            if isinstance(numerator, Fraction):\n                self._numerator = numerator._numerator\n                self._denominator = numerator._denominator\n                return self\n            if isinstance(numerator, (float, int)) or hasattr(numerator, \"as_integer_ratio\"):\n                if isinstance(numerator, float) and (numerator != numerator or numerator in (float(\"inf\"), float(\"-inf\"))):\n                    raise (ValueError if numerator != numerator else OverflowError)(\n                        \"cannot convert %s to integer ratio\" % (\"NaN\" if numerator != numerator else \"Infinity\"))\n                n, d = numerator.as_integer_ratio()\n                self._numerator = int(n)\n                self._denominator = int(d)\n                return self\n            if isinstance(numerator, str):\n                m = _RATIONAL_FORMAT.match(numerator)\n                if m is None:\n                    raise ValueError(\"Invalid literal for Fraction: %r\" % numerator)\n                numerator = int(m.group(\"num\") or \"0\")\n                denom = m.group(\"denom\")\n                if denom:\n                    denominator = int(denom)\n                else:\n                    denominator = 1\n                    decimal = m.group(\"decimal\")\n                    if decimal:\n                        decimal = decimal.replace(\"_\", \"\")\n                        scale = 10 ** len(decimal)\n                        numerator = numerator * scale + int(decimal)\n                        denominator *= scale\n                    exp = m.group(\"exp\")\n                    if exp:\n                        exp = int(exp)\n                        if exp >= 0:\n                            numerator *= 10 ** exp\n                        else:\n                            denominator *= 10 ** -exp\n                if m.group(\"sign\") == \"-\":\n                    numerator = -numerator\n            else:\n                raise TypeError(\"argument should be a string or a Rational instance\")\n        elif type(numerator) is int and type(denominator) is int:\n            pass\n        elif isinstance(numerator, (int, Fraction)) and isinstance(denominator, (int, Fraction)):\n            a = Fraction(numerator)\n            b = Fraction(denominator)\n            numerator = a._numerator * b._denominator\n            denominator = a._denominator * b._numerator\n        else:\n            raise TypeError(\"both arguments should be Rational instances\")\n        if denominator == 0:\n            raise ZeroDivisionError(\"Fraction(%s, 0)\" % numerator)\n        g = math.gcd(numerator, denominator)\n        if denominator < 0:\n            g = -g\n        self._numerator = numerator // g\n        self._denominator = denominator // g\n        return self\n\n    @classmethod\n    def from_float(cls, f):\n        if isinstance(f, int):\n            return cls(f)\n        if not isinstance(f, float):\n            raise TypeError(\"%s.from_float() only takes floats, not %r (%s)\" % (cls.__name__, f, type(f).__name__))\n        return cls(*f.as_integer_ratio())\n\n    @classmethod\n    def from_decimal(cls, dec):\n        return cls(str(dec))\n\n    @classmethod\n    def _from_coprime_ints(cls, numerator, denominator):\n        obj = object.__new__(cls)\n        obj._numerator = numerator\n        obj._denominator = denominator\n        return obj\n\n    def is_integer(self):\n        return self._denominator == 1\n\n    def as_integer_ratio(self):\n        return (self._numerator, self._denominator)\n\n    def limit_denominator(self, max_denominator=1000000):\n        if max_denominator < 1:\n            raise ValueError(\"max_denominator should be at least 1\")\n        if self._denominator <= max_denominator:\n            return Fraction(self)\n        p0, q0, p1, q1 = 0, 1, 1, 0\n        n, d = self._numerator, self._denominator\n        while True:\n            a = n // d\n            q2 = q0 + a * q1\n            if q2 > max_denominator:\n                break\n            p0, q0, p1, q1 = p1, q1, p0 + a * p1, q2\n            n, d = d, n - a * d\n        k = (max_denominator - q0) // q1\n        if 2 * d * (q0 + k * q1) <= self._denominator:\n            return Fraction._from_coprime_ints(p1, q1)\n        return Fraction._from_coprime_ints(p0 + k * p1, q0 + k * q1)\n\n    @property\n    def numerator(self):\n        return self._numerator\n\n    @property\n    def denominator(self):\n        return self._denominator\n\n    @property\n    def real(self):\n        return self\n\n    @property\n    def imag(self):\n        return 0\n\n    def conjugate(self):\n        return self\n\n    def __repr__(self):\n        return \"%s(%s, %s)\" % (self.__class__.__name__, self._numerator, self._denominator)\n\n    def __str__(self):\n        if self._denominator == 1:\n            return str(self._numerator)\n        return \"%s/%s\" % (self._numerator, self._denominator)\n\n    def __format__(self, spec):\n        if not spec:\n            return str(self)\n        return format(float(self), spec)\n\n    def _op(self, other, fn, rev=False):\n        if isinstance(other, (int, Fraction)):\n            o = Fraction(other)\n            return fn(o, self) if rev else fn(self, o)\n        if isinstance(other, float):\n            return fn(float(other), float(self)) if rev else fn(float(self), other)\n        if isinstance(other, complex):\n            return fn(other, complex(float(self))) if rev else fn(complex(float(self)), other)\n        return NotImplemented\n\n    def __add__(a, b):\n        return a._op(b, lambda x, y: x + y if not isinstance(x, Fraction) else\n                     Fraction(x._numerator * y._denominator + y._numerator * x._denominator, x._denominator * y._denominator))\n\n    def __radd__(a, b):\n        return a._op(b, lambda x, y: x + y if not isinstance(x, Fraction) else Fraction.__add__(x, y), True)\n\n    def __sub__(a, b):\n        return a._op(b, lambda x, y: x - y if not isinstance(x, Fraction) else\n                     Fraction(x._numerator * y._denominator - y._numerator * x._denominator, x._denominator * y._denominator))\n\n    def __rsub__(a, b):\n        return a._op(b, lambda x, y: x - y if not isinstance(x, Fraction) else Fraction.__sub__(x, y), True)\n\n    def __mul__(a, b):\n        return a._op(b, lambda x, y: x * y if not isinstance(x, Fraction) else\n                     Fraction(x._numerator * y._numerator, x._denominator * y._denominator))\n\n    def __rmul__(a, b):\n        return a._op(b, lambda x, y: x * y if not isinstance(x, Fraction) else Fraction.__mul__(x, y), True)\n\n    def __truediv__(a, b):\n        return a._op(b, lambda x, y: x / y if not isinstance(x, Fraction) else\n                     Fraction(x._numerator * y._denominator, x._denominator * y._numerator))\n\n    def __rtruediv__(a, b):\n        return a._op(b, lambda x, y: x / y if not isinstance(x, Fraction) else Fraction.__truediv__(x, y), True)\n\n    def __floordiv__(a, b):\n        return a._op(b, lambda x, y: x // y if not isinstance(x, Fraction) else\n                     (x._numerator * y._denominator) // (y._numerator * x._denominator))\n\n    def __rfloordiv__(a, b):\n        return a._op(b, lambda x, y: x // y if not isinstance(x, Fraction) else Fraction.__floordiv__(x, y), True)\n\n    def __mod__(a, b):\n        return a._op(b, lambda x, y: x % y if not isinstance(x, Fraction) else\n                     Fraction((x._numerator * y._denominator) % (y._numerator * x._denominator), x._denominator * y._denominator))\n\n    def __rmod__(a, b):\n        return a._op(b, lambda x, y: x % y if not isinstance(x, Fraction) else Fraction.__mod__(x, y), True)\n\n    def __divmod__(a, b):\n        r = a._op(b, lambda x, y: divmod(x, y) if not isinstance(x, Fraction) else (x // y, x % y))\n        return r\n\n    def __rdivmod__(a, b):\n        return a._op(b, lambda x, y: divmod(x, y) if not isinstance(x, Fraction) else (x // y, x % y), True)\n\n    def __pow__(a, b):\n        if isinstance(b, int) or (isinstance(b, Fraction) and b._denominator == 1):\n            p = int(b)\n            if p >= 0:\n                return Fraction._from_coprime_ints(a._numerator ** p, a._denominator ** p)\n            if a._numerator == 0:\n                raise ZeroDivisionError(\"Fraction(%s, 0)\" % (a._denominator ** -p))\n            if a._numerator > 0:\n                return Fraction._from_coprime_ints(a._denominator ** -p, a._numerator ** -p)\n            return Fraction._from_coprime_ints((-a._denominator) ** -p, (-a._numerator) ** -p)\n        if isinstance(b, (float, complex)):\n            return float(a) ** b\n        if isinstance(b, Fraction):\n            return float(a) ** float(b)\n        return NotImplemented\n\n    def __rpow__(b, a):\n        if b._denominator == 1 and b._numerator >= 0:\n            return a ** b._numerator\n        if isinstance(a, (int, Fraction)):\n            return Fraction(a) ** b\n        return a ** float(b)\n\n    def __pos__(a):\n        return Fraction._from_coprime_ints(a._numerator, a._denominator)\n\n    def __neg__(a):\n        return Fraction._from_coprime_ints(-a._numerator, a._denominator)\n\n    def __abs__(a):\n        return Fraction._from_coprime_ints(abs(a._numerator), a._denominator)\n\n    def __int__(a):\n        if a._numerator < 0:\n            return -(-a._numerator // a._denominator)\n        return a._numerator // a._denominator\n\n    def __trunc__(a):\n        return a.__int__()\n\n    def __floor__(a):\n        return a._numerator // a._denominator\n\n    def __ceil__(a):\n        return -(-a._numerator // a._denominator)\n\n    def __round__(self, ndigits=None):\n        if ndigits is None:\n            d = self._denominator\n            floor, remainder = divmod(self._numerator, d)\n            if remainder * 2 < d:\n                return floor\n            if remainder * 2 > d:\n                return floor + 1\n            return floor if floor % 2 == 0 else floor + 1\n        shift = 10 ** abs(ndigits)\n        if ndigits > 0:\n            return Fraction(round(self * shift), shift)\n        return Fraction(round(self / shift) * shift)\n\n    def __float__(a):\n        return a._numerator / a._denominator\n\n    def __hash__(self):\n        if self._denominator == 1:\n            return hash(self._numerator)\n        return hash(self._numerator / self._denominator)\n\n    def __eq__(a, b):\n        if isinstance(b, int):\n            return a._numerator == b and a._denominator == 1\n        if isinstance(b, Fraction):\n            return a._numerator == b._numerator and a._denominator == b._denominator\n        if isinstance(b, complex) and b.imag == 0:\n            b = b.real\n        if isinstance(b, float):\n            if b != b or b in (float(\"inf\"), float(\"-inf\")):\n                return False\n            return a == Fraction(b)\n        return NotImplemented\n\n    def _cmp(a, b, op):\n        if isinstance(b, (int, Fraction)):\n            b = Fraction(b)\n            return op(a._numerator * b._denominator, b._numerator * a._denominator)\n        if isinstance(b, float):\n            if b != b:\n                return False\n            if b in (float(\"inf\"), float(\"-inf\")):\n                return op(0.0, b)\n            return a._cmp(Fraction(b), op)\n        return NotImplemented\n\n    def __lt__(a, b):\n        return a._cmp(b, lambda x, y: x < y)\n\n    def __gt__(a, b):\n        return a._cmp(b, lambda x, y: x > y)\n\n    def __le__(a, b):\n        return a._cmp(b, lambda x, y: x <= y)\n\n    def __ge__(a, b):\n        return a._cmp(b, lambda x, y: x >= y)\n\n    def __bool__(a):\n        return a._numerator != 0\n\n    def __reduce__(self):\n        return (self.__class__, (self._numerator, self._denominator))\n\n    def __copy__(self):\n        return self\n\n    def __deepcopy__(self, memo):\n        return self\n\n\nnumbers.Rational.register(Fraction)\n";
    return s17;
  }
  if ( name == "functools" ) {
    const s18 = "# SPDX-License-Identifier: MIT\n# functools for the Ranger Python runtime: reduce, lru_cache, cache, wraps,\n# total_ordering and cmp_to_key are native (_functools); the rest is here.\nfrom _functools import reduce, lru_cache, cache, total_ordering, cmp_to_key\nfrom _functools import partial as _native_partial\n\n__all__ = [\"update_wrapper\", \"wraps\", \"WRAPPER_ASSIGNMENTS\", \"WRAPPER_UPDATES\", \"total_ordering\",\n           \"cache\", \"cmp_to_key\", \"lru_cache\", \"reduce\", \"partial\", \"partialmethod\",\n           \"singledispatch\", \"singledispatchmethod\", \"cached_property\"]\n\nWRAPPER_ASSIGNMENTS = (\"__module__\", \"__name__\", \"__qualname__\", \"__doc__\", \"__annotations__\", \"__type_params__\")\nWRAPPER_UPDATES = (\"__dict__\",)\n\n\ndef wraps(wrapped, assigned=WRAPPER_ASSIGNMENTS, updated=WRAPPER_UPDATES):\n    def deco(wrapper):\n        return update_wrapper(wrapper, wrapped, assigned, updated)\n    return deco\n\n\ndef update_wrapper(wrapper, wrapped, assigned=WRAPPER_ASSIGNMENTS, updated=WRAPPER_UPDATES):\n    for attr in assigned:\n        try:\n            value = getattr(wrapped, attr)\n        except AttributeError:\n            pass\n        else:\n            try:\n                setattr(wrapper, attr, value)\n            except (AttributeError, TypeError):\n                pass\n    for attr in updated:\n        d = getattr(wrapped, attr, None)\n        if isinstance(d, dict):\n            for k, v in d.items():\n                try:\n                    setattr(wrapper, k, v)\n                except (AttributeError, TypeError):\n                    pass\n    wrapper.__wrapped__ = wrapped\n    return wrapper\n\n\nclass partial:\n    __slots__ = (\"func\", \"args\", \"keywords\", \"__dict__\", \"__weakref__\")\n\n    def __new__(cls, func, /, *args, **keywords):\n        if not callable(func):\n            raise TypeError(\"the first argument must be callable\")\n        if isinstance(func, partial):\n            args = func.args + args\n            keywords = {**func.keywords, **keywords}\n            func = func.func\n        self = object.__new__(cls)\n        self.func = func\n        self.args = args\n        self.keywords = keywords\n        return self\n\n    def __call__(self, /, *args, **keywords):\n        keywords = {**self.keywords, **keywords}\n        return self.func(*self.args, *args, **keywords)\n\n    def __repr__(self):\n        qualname = type(self).__qualname__\n        module = type(self).__module__\n        args = [repr(self.func)]\n        args.extend(repr(x) for x in self.args)\n        args.extend(f\"{k}={v!r}\" for (k, v) in self.keywords.items())\n        if module == \"functools\" or module == \"__main__\" and qualname == \"partial\":\n            return f\"functools.{qualname}({', '.join(args)})\"\n        return f\"{module}.{qualname}({', '.join(args)})\"\n\n    def __get__(self, obj, objtype=None):\n        return self\n\n    def __reduce__(self):\n        return type(self), (self.func,), (self.func, self.args, self.keywords or None, self.__dict__ or None)\n\n    def __setstate__(self, state):\n        func, args, kwds, namespace = state\n        self.func = func\n        self.args = tuple(args)\n        self.keywords = dict(kwds) if kwds else {}\n        if namespace:\n            self.__dict__.update(namespace)\n\n    def __class_getitem__(cls, item):\n        import types\n        return types.GenericAlias(cls, item)\n\n\npartial.__module__ = \"functools\"\n\n\nclass partialmethod:\n    def __init__(self, func, /, *args, **keywords):\n        if isinstance(func, partialmethod):\n            self.func = func.func\n            self.args = func.args + args\n            self.keywords = {**func.keywords, **keywords}\n        else:\n            self.func = func\n            self.args = args\n            self.keywords = keywords\n\n    def __get__(self, obj, cls=None):\n        if obj is None:\n            return self\n        func = self.func\n\n        def bound(*args, **keywords):\n            return func(obj, *self.args, *args, **{**self.keywords, **keywords})\n        return bound\n\n    @property\n    def __isabstractmethod__(self):\n        return getattr(self.func, \"__isabstractmethod__\", False)\n\n\nclass _lru_list_wrapper:\n    pass\n\n\ndef _find_impl(cls, registry):\n    for base in getattr(cls, \"__mro__\", (cls,)):\n        if base in registry:\n            return registry[base]\n    if cls is bool and int in registry:\n        return registry[int]\n    for typ, impl in registry.items():\n        try:\n            if typ is not object and isinstance(typ, type) and issubclass(cls, typ):\n                return impl\n        except TypeError:\n            pass\n    return registry.get(object)\n\n\ndef singledispatch(func):\n    registry = {}\n    registry[object] = func\n\n    def dispatch(cls):\n        return _find_impl(cls, registry)\n\n    def register(cls, func=None):\n        if func is None:\n            if isinstance(cls, type):\n                return lambda f: register(cls, f)\n            ann = getattr(cls, \"__annotations__\", {})\n            if not ann:\n                raise TypeError(\"Invalid first argument to `register()`: %r. Use either `@register(some_class)` or plain `@register` on an annotated function.\" % (cls,))\n            func = cls\n            argname, cls = next(iter(ann.items()))\n            if isinstance(cls, str):\n                cls = func.__globals__.get(cls, cls) if hasattr(func, \"__globals__\") else cls\n        registry[cls] = func\n        return func\n\n    def wrapper(*args, **kw):\n        if not args:\n            raise TypeError(f\"{funcname} requires at least 1 positional argument\")\n        return dispatch(args[0].__class__ if hasattr(args[0], \"__class__\") else type(args[0]))(*args, **kw)\n\n    funcname = getattr(func, \"__name__\", \"singledispatch function\")\n    wrapper.register = register\n    wrapper.dispatch = dispatch\n    wrapper.registry = registry\n    wrapper._clear_cache = lambda: None\n    update_wrapper(wrapper, func)\n    return wrapper\n\n\nclass singledispatchmethod:\n    def __init__(self, func):\n        self.dispatcher = singledispatch(func)\n        self.func = func\n\n    def register(self, cls, method=None):\n        return self.dispatcher.register(cls, func=method)\n\n    def __get__(self, obj, cls=None):\n        def _method(*args, **kwargs):\n            method = self.dispatcher.dispatch(args[0].__class__)\n            return method.__get__(obj, cls)(*args, **kwargs) if hasattr(method, \"__get__\") and obj is not None else method(obj, *args, **kwargs)\n        _method.register = self.register\n        update_wrapper(_method, self.func)\n        return _method\n\n\n_NOT_FOUND = object()\n\n\nclass cached_property:\n    def __init__(self, func):\n        self.func = func\n        self.attrname = None\n        self.__doc__ = func.__doc__\n        self.__module__ = getattr(func, \"__module__\", None)\n\n    def __set_name__(self, owner, name):\n        if self.attrname is None:\n            self.attrname = name\n        elif name != self.attrname:\n            raise TypeError(\"Cannot assign the same cached_property to two different names (%r and %r).\" % (self.attrname, name))\n\n    def __get__(self, instance, owner=None):\n        if instance is None:\n            return self\n        if self.attrname is None:\n            raise TypeError(\"Cannot use cached_property instance without calling __set_name__ on it.\")\n        cache = instance.__dict__\n        val = cache.get(self.attrname, _NOT_FOUND)\n        if val is _NOT_FOUND:\n            val = self.func(instance)\n            cache[self.attrname] = val\n        return val\n";
    return s18;
  }
  if ( name == "gc" ) {
    const s19 = "# SPDX-License-Identifier: MIT\n# gc for the Ranger Python runtime: memory is the host's, so these do nothing.\n_enabled = True\ndef collect(generation=2): return 0\ndef enable():\n    global _enabled\n    _enabled = True\ndef disable():\n    global _enabled\n    _enabled = False\ndef isenabled(): return _enabled\ndef get_referrers(*objs): return []\ndef get_referents(*objs): return []\ndef is_tracked(obj): return False\ndef freeze(): pass\ngarbage = []\n";
    return s19;
  }
  if ( name == "heapq" ) {
    const s20 = "# SPDX-License-Identifier: MIT\n# heapq for the Ranger Python runtime: a binary min-heap on a list\n\n__all__ = [\"heappush\", \"heappop\", \"heapify\", \"heapreplace\", \"merge\", \"nlargest\", \"nsmallest\", \"heappushpop\"]\n\n\ndef heappush(heap, item):\n    heap.append(item)\n    _siftdown(heap, 0, len(heap) - 1)\n\n\ndef heappop(heap):\n    last = heap.pop()\n    if heap:\n        item = heap[0]\n        heap[0] = last\n        _siftup(heap, 0)\n        return item\n    return last\n\n\ndef heapreplace(heap, item):\n    returnitem = heap[0]\n    heap[0] = item\n    _siftup(heap, 0)\n    return returnitem\n\n\ndef heappushpop(heap, item):\n    if heap and heap[0] < item:\n        item, heap[0] = heap[0], item\n        _siftup(heap, 0)\n    return item\n\n\ndef heapify(x):\n    n = len(x)\n    for i in reversed(range(n // 2)):\n        _siftup(x, i)\n\n\ndef _siftdown(heap, startpos, pos):\n    newitem = heap[pos]\n    while pos > startpos:\n        parentpos = (pos - 1) >> 1\n        parent = heap[parentpos]\n        if newitem < parent:\n            heap[pos] = parent\n            pos = parentpos\n            continue\n        break\n    heap[pos] = newitem\n\n\ndef _siftup(heap, pos):\n    endpos = len(heap)\n    startpos = pos\n    newitem = heap[pos]\n    childpos = 2 * pos + 1\n    while childpos < endpos:\n        rightpos = childpos + 1\n        if rightpos < endpos and not heap[childpos] < heap[rightpos]:\n            childpos = rightpos\n        heap[pos] = heap[childpos]\n        pos = childpos\n        childpos = 2 * pos + 1\n    heap[pos] = newitem\n    _siftdown(heap, startpos, pos)\n\n\ndef _heappop_max(heap):\n    last = heap.pop()\n    if heap:\n        item = heap[0]\n        heap[0] = last\n        _siftup_max(heap, 0)\n        return item\n    return last\n\n\ndef _heapify_max(x):\n    for i in reversed(range(len(x) // 2)):\n        _siftup_max(x, i)\n\n\ndef _siftdown_max(heap, startpos, pos):\n    newitem = heap[pos]\n    while pos > startpos:\n        parentpos = (pos - 1) >> 1\n        parent = heap[parentpos]\n        if parent < newitem:\n            heap[pos] = parent\n            pos = parentpos\n            continue\n        break\n    heap[pos] = newitem\n\n\ndef _siftup_max(heap, pos):\n    endpos = len(heap)\n    startpos = pos\n    newitem = heap[pos]\n    childpos = 2 * pos + 1\n    while childpos < endpos:\n        rightpos = childpos + 1\n        if rightpos < endpos and not heap[rightpos] < heap[childpos]:\n            childpos = rightpos\n        heap[pos] = heap[childpos]\n        pos = childpos\n        childpos = 2 * pos + 1\n    heap[pos] = newitem\n    _siftdown_max(heap, startpos, pos)\n\n\ndef merge(*iterables, key=None, reverse=False):\n    items = []\n    for it in iterables:\n        items.extend(it)\n    return iter(sorted(items, key=key, reverse=reverse))\n\n\ndef nsmallest(n, iterable, key=None):\n    return sorted(iterable, key=key)[:n]\n\n\ndef nlargest(n, iterable, key=None):\n    return sorted(iterable, key=key, reverse=True)[:n]\n";
    return s20;
  }
  if ( name == "inspect" ) {
    const s21 = "# SPDX-License-Identifier: MIT\n# inspect for the Ranger Python runtime: type tests, members, and\n# signatures read from __code__, __defaults__ and __kwdefaults__\nimport types\nimport sys\n\nCO_OPTIMIZED = 1\nCO_NEWLOCALS = 2\nCO_VARARGS = 4\nCO_VARKEYWORDS = 8\nCO_NESTED = 16\nCO_GENERATOR = 32\nCO_NOFREE = 64\nCO_COROUTINE = 128\nCO_ITERABLE_COROUTINE = 256\nCO_ASYNC_GENERATOR = 512\n\n\ndef _flags(obj):\n    obj = unwrap(obj) if callable(obj) else obj\n    code = getattr(obj, \"__code__\", None)\n    return getattr(code, \"co_flags\", 0) if code is not None else 0\n\n\ndef ismodule(obj):\n    return isinstance(obj, types.ModuleType)\n\n\ndef isclass(obj):\n    return isinstance(obj, type)\n\n\ndef ismethod(obj):\n    return isinstance(obj, types.MethodType)\n\n\ndef isfunction(obj):\n    return isinstance(obj, types.FunctionType)\n\n\ndef isgeneratorfunction(obj):\n    return isfunction(obj) and bool(_flags(obj) & CO_GENERATOR)\n\n\ndef iscoroutinefunction(obj):\n    return (isfunction(obj) or ismethod(obj)) and bool(_flags(obj) & CO_COROUTINE)\n\n\ndef isasyncgenfunction(obj):\n    return isfunction(obj) and bool(_flags(obj) & CO_ASYNC_GENERATOR)\n\n\ndef isgenerator(obj):\n    return type(obj).__name__ == \"generator\"\n\n\ndef iscoroutine(obj):\n    return type(obj).__name__ == \"coroutine\"\n\n\ndef isasyncgen(obj):\n    return type(obj).__name__ == \"async_generator\"\n\n\ndef isawaitable(obj):\n    return iscoroutine(obj) or hasattr(type(obj), \"__await__\")\n\n\ndef istraceback(obj):\n    return type(obj).__name__ == \"traceback\"\n\n\ndef isframe(obj):\n    return type(obj).__name__ == \"frame\"\n\n\ndef iscode(obj):\n    return type(obj).__name__ == \"code\"\n\n\ndef isbuiltin(obj):\n    return type(obj).__name__ == \"builtin_function_or_method\"\n\n\ndef isroutine(obj):\n    return isbuiltin(obj) or isfunction(obj) or ismethod(obj) or ismethoddescriptor(obj)\n\n\ndef ismethoddescriptor(obj):\n    if isclass(obj) or ismethod(obj) or isfunction(obj):\n        return False\n    tp = type(obj)\n    return hasattr(tp, \"__get__\") and not hasattr(tp, \"__set__\")\n\n\ndef isdatadescriptor(obj):\n    if isclass(obj) or ismethod(obj) or isfunction(obj):\n        return False\n    tp = type(obj)\n    return hasattr(tp, \"__set__\") or hasattr(tp, \"__delete__\") or isinstance(obj, property)\n\n\ndef isabstract(obj):\n    return bool(getattr(obj, \"__abstractmethods__\", False))\n\n\ndef unwrap(func, *, stop=None):\n    seen = set()\n    while hasattr(func, \"__wrapped__\"):\n        if stop is not None and stop(func):\n            break\n        if id(func) in seen:\n            raise ValueError(\"wrapper loop when unwrapping %r\" % (func,))\n        seen.add(id(func))\n        func = func.__wrapped__\n    return func\n\n\ndef getmembers(obj, predicate=None):\n    out = []\n    for key in dir(obj):\n        try:\n            value = getattr(obj, key)\n        except AttributeError:\n            continue\n        if predicate is None or predicate(value):\n            out.append((key, value))\n    out.sort(key=lambda kv: kv[0])\n    return out\n\n\ndef getmembers_static(obj, predicate=None):\n    return getmembers(obj, predicate)\n\n\ndef getattr_static(obj, attr, default=None):\n    for base in ([obj] if not isclass(obj) else []) + list(getattr(obj if isclass(obj) else type(obj), \"__mro__\", ())):\n        d = getattr(base, \"__dict__\", None)\n        if isinstance(d, dict) and attr in d:\n            return d[attr]\n    if default is not None:\n        return default\n    raise AttributeError(attr)\n\n\ndef getmro(cls):\n    return cls.__mro__\n\n\ndef getdoc(obj):\n    doc = getattr(obj, \"__doc__\", None)\n    if not isinstance(doc, str):\n        return None\n    return cleandoc(doc)\n\n\ndef cleandoc(doc):\n    lines = doc.expandtabs().split(\"\\n\")\n    margin = None\n    for line in lines[1:]:\n        content = len(line.lstrip())\n        if content:\n            indent = len(line) - content\n            margin = indent if margin is None else min(margin, indent)\n    if lines:\n        lines[0] = lines[0].lstrip()\n    if margin is not None:\n        for i in range(1, len(lines)):\n            lines[i] = lines[i][margin:]\n    while lines and not lines[-1]:\n        lines.pop()\n    while lines and not lines[0]:\n        lines.pop(0)\n    return \"\\n\".join(lines)\n\n\ndef getmodule(obj, _filename=None):\n    if ismodule(obj):\n        return obj\n    name = getattr(obj, \"__module__\", None)\n    return sys.modules.get(name) if name else None\n\n\ndef getfile(obj):\n    mod = getmodule(obj)\n    f = getattr(mod, \"__file__\", None)\n    if f is None:\n        raise TypeError(\"source code not available\")\n    return f\n\n\ngetsourcefile = getfile\n\n\ndef getsource(obj):\n    raise OSError(\"could not get source code\")\n\n\ndef getsourcelines(obj):\n    raise OSError(\"could not get source code\")\n\n\ndef currentframe():\n    return None\n\n\ndef stack(context=1):\n    return []\n\n\nclass _empty:\n    pass\n\n\nclass Parameter:\n    POSITIONAL_ONLY = 0\n    POSITIONAL_OR_KEYWORD = 1\n    VAR_POSITIONAL = 2\n    KEYWORD_ONLY = 3\n    VAR_KEYWORD = 4\n    empty = _empty\n    _names = (\"positional-only\", \"positional or keyword\", \"variadic positional\", \"keyword-only\", \"variadic keyword\")\n\n    def __init__(self, name, kind, *, default=_empty, annotation=_empty):\n        self.name = name\n        self.kind = kind\n        self.default = default\n        self.annotation = annotation\n\n    def replace(self, **kw):\n        return Parameter(kw.get(\"name\", self.name), kw.get(\"kind\", self.kind),\n                         default=kw.get(\"default\", self.default), annotation=kw.get(\"annotation\", self.annotation))\n\n    def __str__(self):\n        s = self.name\n        if self.annotation is not _empty:\n            s += \": \" + _annot(self.annotation)\n        if self.default is not _empty:\n            s += (\" = \" if self.annotation is not _empty else \"=\") + repr(self.default)\n        if self.kind == 2:\n            s = \"*\" + s\n        elif self.kind == 4:\n            s = \"**\" + s\n        return s\n\n    def __repr__(self):\n        return '<Parameter \"%s\">' % self\n\n    def __eq__(self, other):\n        if not isinstance(other, Parameter):\n            return NotImplemented\n        return (self.name, self.kind, self.default, self.annotation) == (other.name, other.kind, other.default, other.annotation)\n\n    def __hash__(self):\n        return hash((self.name, self.kind))\n\n\ndef _annot(a):\n    if isinstance(a, type):\n        if a.__module__ == \"builtins\":\n            return a.__qualname__\n        return a.__module__ + \".\" + a.__qualname__\n    return repr(a)\n\n\nclass BoundArguments:\n    def __init__(self, signature, arguments):\n        self.signature = signature\n        self.arguments = arguments\n\n    @property\n    def args(self):\n        out = []\n        for p in self.signature.parameters.values():\n            if p.kind in (3, 4) or p.name not in self.arguments:\n                if p.kind in (3, 4):\n                    break\n                continue\n            if p.kind == 2:\n                out.extend(self.arguments[p.name])\n            else:\n                out.append(self.arguments[p.name])\n        return tuple(out)\n\n    @property\n    def kwargs(self):\n        out = {}\n        for p in self.signature.parameters.values():\n            if p.name not in self.arguments:\n                continue\n            if p.kind == 3:\n                out[p.name] = self.arguments[p.name]\n            elif p.kind == 4:\n                out.update(self.arguments[p.name])\n        return out\n\n    def apply_defaults(self):\n        new = {}\n        for p in self.signature.parameters.values():\n            if p.name in self.arguments:\n                new[p.name] = self.arguments[p.name]\n            elif p.default is not _empty:\n                new[p.name] = p.default\n            elif p.kind == 2:\n                new[p.name] = ()\n            elif p.kind == 4:\n                new[p.name] = {}\n        self.arguments = new\n\n\nclass Signature:\n    empty = _empty\n\n    def __init__(self, parameters=None, *, return_annotation=_empty):\n        self.parameters = {}\n        for p in parameters or ():\n            self.parameters[p.name] = p\n        self.return_annotation = return_annotation\n\n    @classmethod\n    def from_callable(cls, obj, *, follow_wrapped=True):\n        return signature(obj, follow_wrapped=follow_wrapped)\n\n    def replace(self, *, parameters=None, return_annotation=None):\n        return Signature(list(self.parameters.values()) if parameters is None else parameters,\n                         return_annotation=self.return_annotation if return_annotation is None else return_annotation)\n\n    def __str__(self):\n        parts = []\n        kinds = [p.kind for p in self.parameters.values()]\n        star_done = False\n        for i, p in enumerate(self.parameters.values()):\n            if p.kind == 3 and not star_done and 2 not in kinds:\n                parts.append(\"*\")\n                star_done = True\n            if p.kind == 2:\n                star_done = True\n            parts.append(str(p))\n            if p.kind == 0 and (i + 1 == len(kinds) or kinds[i + 1] != 0):\n                parts.append(\"/\")\n        s = \"(\" + \", \".join(parts) + \")\"\n        if self.return_annotation is not _empty:\n            s += \" -> \" + _annot(self.return_annotation)\n        return s\n\n    def __repr__(self):\n        return \"<Signature %s>\" % self\n\n    def __eq__(self, other):\n        if not isinstance(other, Signature):\n            return NotImplemented\n        return list(self.parameters.values()) == list(other.parameters.values()) and self.return_annotation == other.return_annotation\n\n    def _bind(self, args, kwargs, partial):\n        arguments = {}\n        params = list(self.parameters.values())\n        args = list(args)\n        kwargs = dict(kwargs)\n        i = 0\n        for p in params:\n            if p.kind in (0, 1):\n                if args:\n                    arguments[p.name] = args.pop(0)\n                elif p.kind == 1 and p.name in kwargs:\n                    arguments[p.name] = kwargs.pop(p.name)\n                elif p.default is _empty and not partial:\n                    raise TypeError(\"missing a required argument: %r\" % p.name)\n            elif p.kind == 2:\n                if args:\n                    arguments[p.name] = tuple(args)\n                    args = []\n            elif p.kind == 3:\n                if p.name in kwargs:\n                    arguments[p.name] = kwargs.pop(p.name)\n                elif p.default is _empty and not partial:\n                    raise TypeError(\"missing a required argument: %r\" % p.name)\n            elif p.kind == 4:\n                if kwargs:\n                    arguments[p.name] = kwargs\n                    kwargs = {}\n        if args:\n            raise TypeError(\"too many positional arguments\")\n        if kwargs:\n            raise TypeError(\"got an unexpected keyword argument %r\" % next(iter(kwargs)))\n        return BoundArguments(self, arguments)\n\n    def bind(self, *args, **kwargs):\n        return self._bind(args, kwargs, False)\n\n    def bind_partial(self, *args, **kwargs):\n        return self._bind(args, kwargs, True)\n\n\ndef signature(obj, *, follow_wrapped=True, globals=None, locals=None, eval_str=False):\n    if not callable(obj):\n        raise TypeError(\"%r is not a callable object\" % (obj,))\n    if follow_wrapped:\n        obj = unwrap(obj, stop=lambda f: hasattr(f, \"__signature__\"))\n    sig = getattr(obj, \"__signature__\", None)\n    if sig is not None:\n        return sig\n    skip = 0\n    if ismethod(obj):\n        obj = obj.__func__\n        skip = 1\n    elif isclass(obj):\n        init = getattr(obj, \"__init__\", None)\n        if isfunction(init):\n            obj = init\n            skip = 1\n        else:\n            return Signature()\n    elif not isfunction(obj):\n        call = getattr(type(obj), \"__call__\", None)\n        if isfunction(call):\n            obj = call\n            skip = 1\n        else:\n            raise ValueError(\"no signature found for %r\" % (obj,))\n    code = obj.__code__\n    names = code.co_varnames\n    npos = code.co_argcount\n    nposonly = getattr(code, \"co_posonlyargcount\", 0)\n    nkw = code.co_kwonlyargcount\n    defaults = obj.__defaults__ or ()\n    kwdefaults = obj.__kwdefaults__ or {}\n    ann = getattr(obj, \"__annotations__\", {}) or {}\n    params = []\n    first_default = npos - len(defaults)\n    for i in range(npos):\n        name = names[i]\n        kind = 0 if i < nposonly else 1\n        d = defaults[i - first_default] if i >= first_default else _empty\n        params.append(Parameter(name, kind, default=d, annotation=ann.get(name, _empty)))\n    k = npos\n    kwonly = names[k:k + nkw]\n    k += nkw\n    if code.co_flags & CO_VARARGS:\n        params.append(Parameter(names[k], 2, annotation=ann.get(names[k], _empty)))\n        k += 1\n    for name in kwonly:\n        params.append(Parameter(name, 3, default=kwdefaults.get(name, _empty), annotation=ann.get(name, _empty)))\n    if code.co_flags & CO_VARKEYWORDS:\n        params.append(Parameter(names[k], 4, annotation=ann.get(names[k], _empty)))\n    return Signature(params[skip:], return_annotation=ann.get(\"return\", _empty))\n\n\ndef get_annotations(obj, *, globals=None, locals=None, eval_str=False):\n    ann = getattr(obj, \"__annotations__\", None)\n    return dict(ann) if ann else {}\n\n\ndef getfullargspec(func):\n    sig = signature(func, follow_wrapped=False)\n    args = [p.name for p in sig.parameters.values() if p.kind in (0, 1)]\n    varargs = next((p.name for p in sig.parameters.values() if p.kind == 2), None)\n    varkw = next((p.name for p in sig.parameters.values() if p.kind == 4), None)\n    kwonly = [p.name for p in sig.parameters.values() if p.kind == 3]\n    return (args, varargs, varkw, func.__defaults__, kwonly, func.__kwdefaults__, getattr(func, \"__annotations__\", {}))\n\n\ndef getgeneratorstate(gen):\n    if gen.gi_running:\n        return \"GEN_RUNNING\"\n    if gen.gi_frame is None:\n        return \"GEN_CLOSED\"\n    return \"GEN_CREATED\"\n\n\ndef getattr_static_safe(obj, attr):\n    return getattr_static(obj, attr)\n";
    return s21;
  }
  if ( name == "io" ) {
    const s22 = "# SPDX-License-Identifier: MIT\n# io for the Ranger Python runtime: StringIO, and open as in builtins.\n\nopen = open\nSEEK_SET = 0\nSEEK_CUR = 1\nSEEK_END = 2\n\n\nclass UnsupportedOperation(OSError, ValueError):\n    pass\n\n\nclass StringIO:\n    def __init__(self, initial_value=\"\", newline=\"\\n\"):\n        self._parts = [initial_value] if initial_value else []\n        self._value = None\n        self._pos = 0\n        self.closed = False\n\n    def _text(self):\n        if self._value is None:\n            self._value = \"\".join(self._parts)\n            self._parts = [self._value]\n        return self._value\n\n    def write(self, s):\n        if self.closed:\n            raise ValueError(\"I/O operation on closed file.\")\n        if not isinstance(s, str):\n            raise TypeError(\"string argument expected, got '%s'\" % type(s).__name__)\n        text = self._text()\n        if self._pos == len(text):\n            self._parts.append(s)\n            self._value = None\n        else:\n            self._value = text[:self._pos] + s + text[self._pos + len(s):]\n            self._parts = [self._value]\n        self._pos += len(s)\n        return len(s)\n\n    def writelines(self, lines):\n        for line in lines:\n            self.write(line)\n\n    def getvalue(self):\n        return self._text()\n\n    def read(self, size=-1):\n        text = self._text()\n        if size is None or size < 0:\n            out = text[self._pos:]\n        else:\n            out = text[self._pos:self._pos + size]\n        self._pos += len(out)\n        return out\n\n    def readline(self, size=-1):\n        text = self._text()\n        end = text.find(\"\\n\", self._pos)\n        end = len(text) if end < 0 else end + 1\n        out = text[self._pos:end]\n        self._pos = end\n        return out\n\n    def readlines(self):\n        out = []\n        while True:\n            line = self.readline()\n            if not line:\n                return out\n            out.append(line)\n\n    def __iter__(self):\n        return iter(self.readlines())\n\n    def seek(self, pos, whence=0):\n        if whence == 0:\n            self._pos = pos\n        elif whence == 2:\n            self._pos = len(self._text()) + pos\n        else:\n            self._pos += pos\n        return self._pos\n\n    def tell(self):\n        return self._pos\n\n    def truncate(self, size=None):\n        text = self._text()\n        if size is None:\n            size = self._pos\n        self._value = text[:size]\n        self._parts = [self._value]\n        return size\n\n    def flush(self):\n        pass\n\n    def close(self):\n        self.closed = True\n\n    def readable(self):\n        return True\n\n    def writable(self):\n        return True\n\n    def seekable(self):\n        return True\n\n    def __enter__(self):\n        return self\n\n    def __exit__(self, *args):\n        self.close()\n        return False\n\n\nTextIOWrapper = StringIO\n";
    return s22;
  }
  if ( name == "keyword" ) {
    const s23 = "# SPDX-License-Identifier: MIT\nkwlist = [\"False\", \"None\", \"True\", \"and\", \"as\", \"assert\", \"async\", \"await\", \"break\", \"class\", \"continue\", \"def\", \"del\", \"elif\", \"else\", \"except\", \"finally\", \"for\", \"from\", \"global\", \"if\", \"import\", \"in\", \"is\", \"lambda\", \"nonlocal\", \"not\", \"or\", \"pass\", \"raise\", \"return\", \"try\", \"while\", \"with\", \"yield\"]\nsoftkwlist = [\"_\", \"case\", \"match\", \"type\"]\ndef iskeyword(s): return s in kwlist\ndef issoftkeyword(s): return s in softkwlist\n";
    return s23;
  }
  if ( name == "locale" ) {
    const s24 = "# SPDX-License-Identifier: MIT\n# locale for the Ranger Python runtime: the C locale, always\nLC_CTYPE = 0\nLC_NUMERIC = 1\nLC_TIME = 2\nLC_COLLATE = 3\nLC_MONETARY = 4\nLC_MESSAGES = 5\nLC_ALL = 6\nCHAR_MAX = 127\n\n\nclass Error(Exception):\n    pass\n\n\ndef setlocale(category, locale=None):\n    if locale not in (None, \"\", \"C\", \"POSIX\"):\n        raise Error(\"unsupported locale setting\")\n    return \"C\"\n\n\ndef getlocale(category=LC_CTYPE):\n    return (None, None)\n\n\ndef getpreferredencoding(do_setlocale=True):\n    return \"UTF-8\"\n\n\ndef getencoding():\n    return \"UTF-8\"\n\n\ndef localeconv():\n    return {\"decimal_point\": \".\", \"thousands_sep\": \"\", \"grouping\": [], \"int_curr_symbol\": \"\",\n            \"currency_symbol\": \"\", \"mon_decimal_point\": \"\", \"mon_thousands_sep\": \"\",\n            \"mon_grouping\": [], \"positive_sign\": \"\", \"negative_sign\": \"\", \"int_frac_digits\": 127,\n            \"frac_digits\": 127, \"p_cs_precedes\": 127, \"p_sep_by_space\": 127, \"n_cs_precedes\": 127,\n            \"n_sep_by_space\": 127, \"p_sign_posn\": 127, \"n_sign_posn\": 127}\n\n\ndef strcoll(a, b):\n    return (a > b) - (a < b)\n\n\ndef strxfrm(s):\n    return s\n\n\ndef format_string(f, val, grouping=False, monetary=False):\n    return f % val\n\n\ndef str(val):\n    return \"%.12g\" % val\n\n\ndef atof(s, func=float):\n    return func(s)\n\n\ndef atoi(s):\n    return int(s)\n\n\ndef normalize(localename):\n    return localename\n";
    return s24;
  }
  if ( name == "numbers" ) {
    const s25 = "# SPDX-License-Identifier: MIT\n# numbers for the Ranger Python runtime: the numeric tower as classes that\n# answer isinstance() for the built-in numbers and for registered classes\n\n\ndef _tower(cls, builtins):\n    cls._registry = []\n    cls._builtins = builtins\n\n    def instancecheck(c, obj):\n        return subclasscheck(c, type(obj))\n\n    def subclasscheck(c, sub):\n        if sub is bool:\n            sub = int\n        if sub in c._builtins:\n            return True\n        for base in getattr(sub, \"__mro__\", (sub,)):\n            if base is c:\n                return True\n            for t in _all:\n                if c in t.__mro__ and (base in t._registry):\n                    return True\n        return False\n\n    def register(c, sub):\n        c._registry.append(sub)\n        return sub\n    cls.__instancecheck__ = classmethod(instancecheck)\n    cls.__subclasscheck__ = classmethod(subclasscheck)\n    cls.register = classmethod(register)\n    _all.append(cls)\n    return cls\n\n\n_all = []\n\n\nclass Number:\n    __slots__ = ()\n    __hash__ = None\n\n\nclass Complex(Number):\n    __slots__ = ()\n\n    def __bool__(self):\n        return self != 0\n\n    @property\n    def real(self):\n        raise NotImplementedError\n\n    @property\n    def imag(self):\n        raise NotImplementedError\n\n\nclass Real(Complex):\n    __slots__ = ()\n\n    def __complex__(self):\n        return complex(float(self))\n\n    @property\n    def real(self):\n        return +self\n\n    @property\n    def imag(self):\n        return 0\n\n    def conjugate(self):\n        return +self\n\n    def __divmod__(self, other):\n        return (self // other, self % other)\n\n    def __rdivmod__(self, other):\n        return (other // self, other % self)\n\n\nclass Rational(Real):\n    __slots__ = ()\n\n    def __float__(self):\n        return int(self.numerator) / int(self.denominator)\n\n\nclass Integral(Rational):\n    __slots__ = ()\n\n    def __float__(self):\n        return float(int(self))\n\n    def __index__(self):\n        return int(self)\n\n    @property\n    def numerator(self):\n        return +self\n\n    @property\n    def denominator(self):\n        return 1\n\n\n_tower(Number, (int, float, complex))\n_tower(Complex, (int, float, complex))\n_tower(Real, (int, float))\n_tower(Rational, (int,))\n_tower(Integral, (int,))\n";
    return s25;
  }
  if ( name == "operator" ) {
    const s26 = "# SPDX-License-Identifier: MIT\n# operator for the Ranger Python runtime.\n\ndef lt(a, b): return a < b\ndef le(a, b): return a <= b\ndef eq(a, b): return a == b\ndef ne(a, b): return a != b\ndef ge(a, b): return a >= b\ndef gt(a, b): return a > b\ndef not_(a): return not a\ndef truth(a): return bool(a)\ndef is_(a, b): return a is b\ndef is_not(a, b): return a is not b\ndef is_none(a): return a is None\ndef is_not_none(a): return a is not None\ndef abs(a): return __builtins_abs(a)\ndef add(a, b): return a + b\ndef and_(a, b): return a & b\ndef floordiv(a, b): return a // b\ndef index(a): return a.__index__() if hasattr(a, \"__index__\") else int(a)\ndef inv(a): return ~a\ninvert = inv\ndef lshift(a, b): return a << b\ndef mod(a, b): return a % b\ndef mul(a, b): return a * b\ndef matmul(a, b): return a @ b\ndef neg(a): return -a\ndef or_(a, b): return a | b\ndef pos(a): return +a\ndef pow(a, b): return a ** b\ndef rshift(a, b): return a >> b\ndef sub(a, b): return a - b\ndef truediv(a, b): return a / b\ndef xor(a, b): return a ^ b\ndef concat(a, b): return a + b\ndef contains(a, b): return b in a\ndef countOf(a, b):\n    n = 0\n    for x in a:\n        if x is b or x == b:\n            n += 1\n    return n\ndef delitem(a, b): del a[b]\ndef getitem(a, b): return a[b]\ndef indexOf(a, b):\n    for i, x in enumerate(a):\n        if x is b or x == b:\n            return i\n    raise ValueError(\"sequence.index(x): x not in sequence\")\ndef setitem(a, b, c): a[b] = c\ndef length_hint(obj, default=0):\n    try:\n        return len(obj)\n    except TypeError:\n        return default\ndef iadd(a, b):\n    a += b\n    return a\ndef isub(a, b):\n    a -= b\n    return a\ndef imul(a, b):\n    a *= b\n    return a\ndef iconcat(a, b):\n    a += b\n    return a\n\ndef __builtins_abs(a):\n    return a if a >= 0 else -a\n\n\nclass attrgetter:\n    def __init__(self, attr, *attrs):\n        self._attrs = (attr,) + attrs\n\n    def __call__(self, obj):\n        out = []\n        for name in self._attrs:\n            v = obj\n            for part in name.split(\".\"):\n                v = getattr(v, part)\n            out.append(v)\n        if len(out) == 1:\n            return out[0]\n        return tuple(out)\n\n\nclass itemgetter:\n    def __init__(self, item, *items):\n        self._items = (item,) + items\n\n    def __call__(self, obj):\n        if len(self._items) == 1:\n            return obj[self._items[0]]\n        return tuple(obj[i] for i in self._items)\n\n\nclass methodcaller:\n    def __init__(self, name, *args, **kwargs):\n        self._name = name\n        self._args = args\n        self._kwargs = kwargs\n\n    def __call__(self, obj):\n        return getattr(obj, self._name)(*self._args, **self._kwargs)\n";
    return s26;
  }
  if ( name == "pickle" ) {
    const s27 = "# SPDX-License-Identifier: MIT\n# pickle for the Ranger Python runtime: dumps() keeps a deep copy of the\n# value behind a small bytes key, and loads() copies it back out.\nimport copy as _copy\n\nHIGHEST_PROTOCOL = 5\nDEFAULT_PROTOCOL = 5\n_store = {}\n\n\nclass PickleError(Exception):\n    pass\n\n\nclass PicklingError(PickleError):\n    pass\n\n\nclass UnpicklingError(PickleError):\n    pass\n\n\ndef dumps(obj, protocol=None, *, fix_imports=True, buffer_callback=None):\n    key = len(_store)\n    _store[key] = _copy.deepcopy(obj)\n    return (\"\\x80pickle:%d.\" % key).encode(\"latin-1\")\n\n\ndef loads(data, /, *, fix_imports=True, encoding=\"ASCII\", errors=\"strict\", buffers=()):\n    text = data if isinstance(data, str) else data.decode(\"latin-1\")\n    if not text.startswith(\"\\x80pickle:\"):\n        raise UnpicklingError(\"invalid load key\")\n    return _copy.deepcopy(_store[int(text[8:-1])])\n\n\ndef dump(obj, file, protocol=None, **kw):\n    file.write(dumps(obj, protocol))\n\n\ndef load(file, **kw):\n    return loads(file.read())\n";
    return s27;
  }
  if ( name == "platform" ) {
    const s28 = "# SPDX-License-Identifier: MIT\n# platform for the Ranger Python runtime\nimport sys\n\n\ndef python_implementation():\n    return \"RangerPython\"\n\n\ndef python_version():\n    return \"%d.%d.%d\" % sys.version_info[:3]\n\n\ndef python_version_tuple():\n    return tuple(str(x) for x in sys.version_info[:3])\n\n\ndef system():\n    return \"\"\n\n\ndef machine():\n    return \"\"\n\n\ndef node():\n    return \"\"\n\n\ndef release():\n    return \"\"\n\n\ndef version():\n    return \"\"\n\n\ndef platform(aliased=False, terse=False):\n    return \"ranger\"\n\n\ndef architecture(executable=None, bits=\"\", linkage=\"\"):\n    return (\"64bit\", \"\")\n\n\ndef uname():\n    return (\"\", \"\", \"\", \"\", \"\", \"\")\n\n\ndef libc_ver(*args, **kw):\n    return (\"\", \"\")\n\n\ndef mac_ver(*args, **kw):\n    return (\"\", (\"\", \"\", \"\"), \"\")\n\n\ndef win32_ver(*args, **kw):\n    return (\"\", \"\", \"\", \"\")\n\n\ndef python_compiler():\n    return \"Ranger\"\n";
    return s28;
  }
  if ( name == "re" ) {
    const s29 = "# SPDX-License-Identifier: MIT\n# re for the Ranger Python runtime: a backtracking regular expression\n# engine written in Python, so it runs the same on every target. A pattern\n# is parsed to a tree, compiled to a small instruction list, and run by a\n# loop with an explicit backtrack stack (no recursion per character).\n\n__all__ = [\"match\", \"fullmatch\", \"search\", \"sub\", \"subn\", \"split\", \"findall\",\n           \"finditer\", \"compile\", \"purge\", \"escape\", \"error\", \"Pattern\", \"Match\",\n           \"A\", \"I\", \"L\", \"M\", \"S\", \"X\", \"U\", \"ASCII\", \"IGNORECASE\", \"LOCALE\",\n           \"MULTILINE\", \"DOTALL\", \"VERBOSE\", \"UNICODE\", \"NOFLAG\", \"RegexFlag\"]\n\n\nclass RegexFlag(int):\n    def __or__(self, other):\n        if isinstance(other, int):\n            return RegexFlag(int(self) | int(other))\n        return NotImplemented\n\n    __ror__ = __or__\n\n    def __and__(self, other):\n        if isinstance(other, int):\n            return RegexFlag(int(self) & int(other))\n        return NotImplemented\n\n    __rand__ = __and__\n\n    def __xor__(self, other):\n        if isinstance(other, int):\n            return RegexFlag(int(self) ^ int(other))\n        return NotImplemented\n\n    def __invert__(self):\n        return RegexFlag(~int(self) & 511)\n\n    def __repr__(self):\n        v = int(self)\n        if v == 0:\n            return \"re.NOFLAG\"\n        parts = []\n        for name, bit in _FLAG_NAMES:\n            if v & bit:\n                parts.append(\"re.\" + name)\n                v &= ~bit\n        if v:\n            parts.append(hex(v))\n        return \"|\".join(parts)\n\n    __str__ = __repr__\n\n    @property\n    def name(self):\n        for n, bit in _FLAG_NAMES:\n            if int(self) == bit:\n                return n\n        return None\n\n    @property\n    def value(self):\n        return int(self)\n\n\n_FLAG_NAMES = [(\"TEMPLATE\", 1), (\"IGNORECASE\", 2), (\"LOCALE\", 4), (\"MULTILINE\", 8),\n               (\"DOTALL\", 16), (\"UNICODE\", 32), (\"VERBOSE\", 64), (\"DEBUG\", 128),\n               (\"ASCII\", 256)]\nNOFLAG = RegexFlag(0)\nIGNORECASE = I = RegexFlag(2)\nLOCALE = L = RegexFlag(4)\nMULTILINE = M = RegexFlag(8)\nDOTALL = S = RegexFlag(16)\nUNICODE = U = RegexFlag(32)\nVERBOSE = X = RegexFlag(64)\nDEBUG = RegexFlag(128)\nASCII = A = RegexFlag(256)\n_INLINE = {\"a\": 256, \"i\": 2, \"L\": 4, \"m\": 8, \"s\": 16, \"u\": 32, \"x\": 64}\n\n\nclass error(Exception):\n    def __init__(self, msg, pattern=None, pos=None):\n        self.msg = msg\n        self.pattern = pattern\n        self.pos = pos\n        if pattern is not None and pos is not None:\n            self.lineno = pattern.count(\"\\n\", 0, pos) + 1\n            self.colno = pos - (pattern.rfind(\"\\n\", 0, pos) + 1) + 1\n            if \"\\n\" in pattern:\n                msg = \"%s at position %d (line %d, column %d)\" % (msg, pos, self.lineno, self.colno)\n            else:\n                msg = \"%s at position %d\" % (msg, pos)\n        else:\n            self.lineno = self.colno = None\n        Exception.__init__(self, msg)\n\n\nPatternError = error\n\n# ---- parsing: pattern text to a tree of tuples\n#   (\"lit\", ch, icase)  (\"any\", dotall)  (\"set\", neg, items, icase, ascii)\n#   (\"cat\", [nodes])  (\"alt\", [nodes])  (\"group\", n, node)\n#   (\"rep\", node, lo, hi, mode)  (\"at\", kind, multiline, ascii)\n#   (\"look\", ahead, neg, node)  (\"atomic\", node)  (\"bref\", n, icase)\n#   (\"cond\", n, yes, no)\n\n_DIGITS = \"0123456789\"\n_HEX = \"0123456789abcdefABCDEF\"\n_WS = \" \\t\\n\\r\\v\\f\"\n_REPEAT = \"*+?{\"\n_SPECIAL = \".^$*+?{}[]\\\\|()\"\n\n\nclass _Parser:\n    def __init__(self, pattern, flags):\n        self.p = pattern\n        self.i = 0\n        self.flags = flags\n        self.groups = 1\n        self.names = {}\n        self.open = []\n        self.lookbehind = 0\n        self.scan_global_flags()\n\n    def err(self, msg, pos=None):\n        raise error(msg, self.p, self.i if pos is None else pos)\n\n    def scan_global_flags(self):\n        # (?flags) is only allowed at the start, and applies to all of it\n        p = self.p\n        i = 0\n        while p.startswith(\"(?\", i):\n            j = i + 2\n            while j < len(p) and p[j] in _INLINE:\n                j += 1\n            if j < len(p) and p[j] == \")\" and j > i + 2:\n                for c in p[i + 2:j]:\n                    self.flags |= _INLINE[c]\n                i = j + 1\n            else:\n                break\n\n    def peek(self):\n        return self.p[self.i] if self.i < len(self.p) else \"\"\n\n    def skip_verbose(self, flags):\n        if not flags & 64:\n            return\n        p = self.p\n        while self.i < len(p):\n            c = p[self.i]\n            if c in _WS:\n                self.i += 1\n            elif c == \"#\":\n                while self.i < len(p) and p[self.i] != \"\\n\":\n                    self.i += 1\n            else:\n                break\n\n    def parse(self):\n        node = self.alt(self.flags)\n        if self.i < len(self.p):\n            if self.p[self.i] == \")\":\n                self.err(\"unbalanced parenthesis\")\n            self.err(\"unexpected end\")\n        return node\n\n    def alt(self, flags):\n        items = [self.seq(flags)]\n        while self.peek() == \"|\":\n            self.i += 1\n            items.append(self.seq(flags))\n        if len(items) == 1:\n            return items[0]\n        return (\"alt\", items)\n\n    def seq(self, flags):\n        items = []\n        while True:\n            self.skip_verbose(flags)\n            c = self.peek()\n            if c == \"\" or c == \"|\" or c == \")\":\n                break\n            start = self.i\n            atom = self.atom(flags)\n            if atom is None:\n                continue\n            atom = self.repeats(atom, flags, start)\n            items.append(atom)\n        if len(items) == 1:\n            return items[0]\n        return (\"cat\", items)\n\n    def repeats(self, atom, flags, start):\n        self.skip_verbose(flags)\n        c = self.peek()\n        if c == \"\" or c not in _REPEAT:\n            return atom\n        here = self.i\n        lo, hi = self.quant()\n        if lo is None:\n            return atom\n        if atom[0] == \"at\" or (atom[0] == \"cat\" and not atom[1]):\n            self.err(\"nothing to repeat\", here)\n        if atom[0] == \"look\":\n            pass\n        mode = \"greedy\"\n        if self.peek() == \"?\":\n            self.i += 1\n            mode = \"lazy\"\n        elif self.peek() == \"+\":\n            self.i += 1\n            mode = \"poss\"\n        node = (\"rep\", atom, lo, hi, mode)\n        self.skip_verbose(flags)\n        c = self.peek()\n        if c and c in \"*+?\" or (c == \"{\" and self.is_quant(self.i)):\n            self.err(\"multiple repeat\", self.i)\n        return node\n\n    def is_quant(self, i):\n        p = self.p\n        j = i + 1\n        while j < len(p) and p[j] in _DIGITS:\n            j += 1\n        if j < len(p) and p[j] == \",\":\n            j += 1\n            while j < len(p) and p[j] in _DIGITS:\n                j += 1\n        return j < len(p) and p[j] == \"}\"\n\n    def quant(self):\n        c = self.p[self.i]\n        if c == \"*\":\n            self.i += 1\n            return 0, None\n        if c == \"+\":\n            self.i += 1\n            return 1, None\n        if c == \"?\":\n            self.i += 1\n            return 0, 1\n        # {m,n}; anything else is a literal \"{\"\n        if not self.is_quant(self.i):\n            return None, None\n        here = self.i\n        j = self.p.index(\"}\", self.i)\n        body = self.p[self.i + 1:j]\n        self.i = j + 1\n        if \",\" in body:\n            a, b = body.split(\",\", 1)\n            lo = int(a) if a else 0\n            hi = int(b) if b else None\n        else:\n            lo = hi = int(body)\n        if hi is not None and hi < lo:\n            self.err(\"min repeat greater than max repeat\", here + 1)\n        if lo >= 4294967295 or (hi is not None and hi >= 4294967295):\n            raise OverflowError(\"the repetition number is too large\")\n        return lo, hi\n\n    def atom(self, flags):\n        p = self.p\n        c = p[self.i]\n        if c == \"(\":\n            return self.group(flags)\n        if c == \"[\":\n            return self.charset(flags)\n        if c == \".\":\n            self.i += 1\n            return (\"any\", bool(flags & 16))\n        if c == \"^\":\n            self.i += 1\n            return (\"at\", \"^\", bool(flags & 8), False)\n        if c == \"$\":\n            self.i += 1\n            return (\"at\", \"$\", bool(flags & 8), False)\n        if c == \"\\\\\":\n            return self.escape(flags, False)\n        if c in \"*+?\":\n            self.err(\"nothing to repeat\")\n        if c == \"{\" and self.is_quant(self.i):\n            self.err(\"nothing to repeat\")\n        self.i += 1\n        return (\"lit\", c, bool(flags & 2))\n\n    def group_name(self, end):\n        j = self.p.find(end, self.i)\n        if j < 0:\n            self.err(\"missing %s, unterminated name\" % end)\n        name = self.p[self.i:j]\n        if not name:\n            self.err(\"missing group name\")\n        if not name.isidentifier():\n            self.err(\"bad character in group name %r\" % name)\n        self.i = j + 1\n        return name\n\n    def group(self, flags):\n        p = self.p\n        start = self.i\n        self.i += 1\n        if p.startswith(\"?\", self.i):\n            self.i += 1\n            c = self.peek()\n            if c == \"P\":\n                self.i += 1\n                c2 = self.peek()\n                if c2 == \"<\":\n                    self.i += 1\n                    name = self.group_name(\">\")\n                    if name in self.names:\n                        self.err(\"redefinition of group name %r as group %d; was group %d\" % (name, self.groups, self.names[name]), start + 4)\n                    n = self.new_group()\n                    self.names[name] = n\n                    return self.group_body((\"group\", n), flags, start)\n                if c2 == \"=\":\n                    self.i += 1\n                    here = self.i\n                    name = self.group_name(\")\")\n                    if name not in self.names:\n                        self.err(\"unknown group name %r\" % name, here)\n                    n = self.names[name]\n                    if n in self.open:\n                        self.err(\"cannot refer to an open group\", start)\n                    return (\"bref\", n, bool(flags & 2))\n                if c2 == \"\":\n                    self.err(\"unexpected end of pattern\")\n                self.err(\"unknown extension ?P\" + c2, start + 1)\n            if c == \":\":\n                self.i += 1\n                return self.group_body(None, flags, start)\n            if c == \"#\":\n                j = p.find(\")\", self.i)\n                if j < 0:\n                    self.err(\"missing ), unterminated comment\", start)\n                self.i = j + 1\n                return None\n            if c == \"=\" or c == \"!\":\n                self.i += 1\n                return self.group_body((\"look\", True, c == \"!\"), flags, start)\n            if c == \"<\":\n                self.i += 1\n                c2 = self.peek()\n                if c2 == \"=\" or c2 == \"!\":\n                    self.i += 1\n                    self.lookbehind += 1\n                    node = self.group_body((\"look\", False, c2 == \"!\"), flags, start)\n                    self.lookbehind -= 1\n                    lo, hi = _width(node[3])\n                    if lo != hi:\n                        self.err(\"look-behind requires fixed-width pattern\", start)\n                    return node\n                if c2 and (c2.isalpha() or c2 == \"_\"):\n                    name = self.group_name(\">\")\n                    if name in self.names:\n                        self.err(\"redefinition of group name %r as group %d; was group %d\" % (name, self.groups, self.names[name]), start + 3)\n                    n = self.new_group()\n                    self.names[name] = n\n                    return self.group_body((\"group\", n), flags, start)\n                if c2 == \"\":\n                    self.err(\"unexpected end of pattern\")\n                self.err(\"unknown extension ?<\" + c2, start + 1)\n            if c == \">\":\n                self.i += 1\n                return self.group_body((\"atomic\",), flags, start)\n            if c == \"(\":\n                self.i += 1\n                here = self.i\n                j = p.find(\")\", self.i)\n                if j < 0:\n                    self.err(\"missing ), unterminated name\")\n                ref = p[self.i:j]\n                self.i = j + 1\n                if ref.isdigit():\n                    n = int(ref)\n                    if n == 0:\n                        self.err(\"bad group number\", here)\n                elif ref.isidentifier():\n                    if ref not in self.names:\n                        self.err(\"unknown group name %r\" % ref, here)\n                    n = self.names[ref]\n                else:\n                    self.err(\"bad character in group name %r\" % ref, here)\n                yes = self.seq(flags)\n                no = (\"cat\", [])\n                if self.peek() == \"|\":\n                    self.i += 1\n                    no = self.seq(flags)\n                    if self.peek() == \"|\":\n                        self.err(\"conditional backref with more than two branches\")\n                if self.peek() != \")\":\n                    self.err(\"missing ), unterminated subpattern\", start)\n                self.i += 1\n                if n >= 100:\n                    self.err(\"invalid group reference %d\" % n, here)\n                return (\"cond\", n, yes, no)\n            # inline flags, scoped (?i:...) or global (?i)\n            add = 0\n            rem = 0\n            neg = False\n            while True:\n                c = self.peek()\n                if c in _INLINE and c != \"\":\n                    if neg:\n                        if c not in \"imsx\":\n                            self.err(\"bad inline flags: cannot turn off flags 'a', 'u' and 'L'\")\n                        rem |= _INLINE[c]\n                    else:\n                        add |= _INLINE[c]\n                    self.i += 1\n                elif c == \"-\" and not neg:\n                    neg = True\n                    self.i += 1\n                    if self.peek() not in _INLINE or self.peek() == \"\":\n                        self.err(\"missing flag\" if self.peek() in \":)\" else \"unknown flag\")\n                else:\n                    break\n            if (add & 256) and (add & 32):\n                self.err(\"bad inline flags: flags 'a', 'u' and 'L' are incompatible\")\n            c = self.peek()\n            if c == \")\":\n                if neg:\n                    self.err(\"missing :\")\n                if start != 0 and not self.at_global_prefix(start):\n                    self.err(\"global flags not at the start of the expression\", start)\n                self.i += 1\n                return None\n            if c == \":\":\n                self.i += 1\n                nf = (flags | add) & ~rem\n                return self.group_body(None, nf, start)\n            if c == \"\":\n                self.err(\"missing -, : or )\")\n            if add == 0 and rem == 0 and not neg:\n                self.err(\"unknown extension ?\" + c, start + 1)\n            self.err(\"unknown flag\" if c.isalpha() else \"missing -, : or )\")\n        n = self.new_group()\n        return self.group_body((\"group\", n), flags, start)\n\n    def at_global_prefix(self, start):\n        # every (?flags) before this one at the very start is also global\n        i = 0\n        p = self.p\n        while i < start:\n            if not p.startswith(\"(?\", i):\n                return False\n            j = p.find(\")\", i)\n            i = j + 1\n        return i == start\n\n    def new_group(self):\n        n = self.groups\n        self.groups += 1\n        if self.groups > 101:\n            pass\n        return n\n\n    def group_body(self, kind, flags, start):\n        if kind is not None and kind[0] == \"group\":\n            self.open.append(kind[1])\n        node = self.alt(flags)\n        if self.peek() != \")\":\n            self.err(\"missing ), unterminated subpattern\", start)\n        self.i += 1\n        if kind is None:\n            return node\n        if kind[0] == \"group\":\n            self.open.pop()\n            return (\"group\", kind[1], node)\n        if kind[0] == \"look\":\n            return (\"look\", kind[1], kind[2], node)\n        return (\"atomic\", node)\n\n    def escape_char(self, inset):\n        # an escape that stands for one character, or None\n        p = self.p\n        start = self.i\n        c = p[self.i + 1] if self.i + 1 < len(p) else \"\"\n        if c == \"\":\n            self.err(\"bad escape (end of pattern)\")\n        simple = {\"n\": \"\\n\", \"t\": \"\\t\", \"r\": \"\\r\", \"f\": \"\\f\", \"v\": \"\\v\", \"a\": \"\\a\", \"\\\\\": \"\\\\\"}\n        if c in simple:\n            self.i += 2\n            return simple[c]\n        if inset and c == \"b\":\n            self.i += 2\n            return \"\\b\"\n        if c == \"x\":\n            h = p[self.i + 2:self.i + 4]\n            if len(h) < 2 or not all(ch in _HEX for ch in h):\n                self.err(\"incomplete escape \\\\x\" + h)\n            self.i += 4\n            return chr(int(h, 16))\n        if c == \"u\" or c == \"U\":\n            k = 4 if c == \"u\" else 8\n            h = p[self.i + 2:self.i + 2 + k]\n            if len(h) < k or not all(ch in _HEX for ch in h):\n                self.err(\"incomplete escape \\\\%s%s\" % (c, h))\n            v = int(h, 16)\n            if v > 0x10FFFF:\n                self.err(\"bad escape \\\\%s%s\" % (c, h))\n            self.i += 2 + k\n            return chr(v)\n        if c == \"N\":\n            if not p.startswith(\"{\", self.i + 2):\n                self.err(\"missing {\", self.i + 2)\n            j = p.find(\"}\", self.i + 3)\n            if j < 0:\n                self.err(\"missing }, unterminated name\", self.i + 3)\n            name = p[self.i + 3:j]\n            if not name:\n                self.err(\"missing character name\", self.i + 3)\n            try:\n                import unicodedata\n                ch = unicodedata.lookup(name)\n            except Exception:\n                try:\n                    ch = eval('\"\\\\N{' + name + '}\"')\n                except Exception:\n                    self.err(\"undefined character name %r\" % name)\n            self.i = j + 1\n            return ch\n        if c == \"0\":\n            j = self.i + 2\n            while j < len(p) and j < self.i + 4 and p[j] in \"01234567\":\n                j += 1\n            v = int(p[self.i + 1:j], 8)\n            self.i = j\n            return chr(v)\n        if c in \"1234567\" and inset:\n            j = self.i + 1\n            while j < len(p) and j < self.i + 4 and p[j] in \"01234567\":\n                j += 1\n            v = int(p[self.i + 1:j], 8)\n            if v > 0o377:\n                self.err(\"octal escape value \\\\%s outside of range 0-0o377\" % p[self.i + 1:j])\n            self.i = j\n            return chr(v)\n        if c.isalnum() and c.isascii():\n            return None\n        self.i += 2\n        return c\n\n    def escape(self, flags, inset):\n        p = self.p\n        start = self.i\n        c = p[self.i + 1] if self.i + 1 < len(p) else \"\"\n        icase = bool(flags & 2)\n        asc = bool(flags & 256)\n        if c in \"dDwWsS\" and c:\n            self.i += 2\n            return (\"set\", False, [(\"cls\", c)], icase, asc)\n        if c in \"AZbB\" and c:\n            self.i += 2\n            return (\"at\", c, False, asc)\n        if c and c in \"123456789\":\n            # a back reference, or an octal escape of three digits\n            j = self.i + 1\n            if (j + 2 < len(p) + 0 and p[j] in \"01234567\" and p[j + 1:j + 2] and p[j + 1] in \"01234567\"\n                    and p[j + 2:j + 3] and p[j + 2] in \"01234567\"):\n                v = int(p[j:j + 3], 8)\n                if v > 0o377:\n                    self.err(\"octal escape value \\\\%s outside of range 0-0o377\" % p[j:j + 3])\n                self.i = j + 3\n                return (\"lit\", chr(v), icase)\n            k = j\n            while k < len(p) and k < j + 2 and p[k] in _DIGITS:\n                k += 1\n            n = int(p[j:k])\n            self.i = k\n            if n >= self.groups:\n                self.err(\"invalid group reference %d\" % n, j)\n            if n in self.open:\n                self.err(\"cannot refer to an open group\", start)\n            return (\"bref\", n, icase)\n        ch = self.escape_char(False)\n        if ch is None:\n            self.err(\"bad escape \\\\\" + c)\n        return (\"lit\", ch, icase)\n\n    def charset(self, flags):\n        p = self.p\n        start = self.i\n        self.i += 1\n        neg = False\n        if self.peek() == \"^\":\n            neg = True\n            self.i += 1\n        items = []\n        first = True\n        while True:\n            if self.i >= len(p):\n                self.err(\"unterminated character set\", start)\n            c = p[self.i]\n            if c == \"]\" and not first:\n                self.i += 1\n                break\n            first = False\n            here = self.i\n            lo = self.set_item(flags)\n            if lo[0] == \"ch\" and p.startswith(\"-\", self.i) and self.i + 1 < len(p) and p[self.i + 1] != \"]\":\n                self.i += 1\n                hi = self.set_item(flags)\n                if hi[0] != \"ch\":\n                    self.err(\"bad character range %s\" % p[here:self.i], here)\n                if ord(hi[1]) < ord(lo[1]):\n                    self.err(\"bad character range %s\" % p[here:self.i], here)\n                items.append((\"range\", lo[1], hi[1]))\n            else:\n                items.append(lo)\n        return (\"set\", neg, items, bool(flags & 2), bool(flags & 256))\n\n    def set_item(self, flags):\n        p = self.p\n        c = p[self.i]\n        if c == \"\\\\\":\n            d = p[self.i + 1] if self.i + 1 < len(p) else \"\"\n            if d and d in \"dDwWsS\":\n                self.i += 2\n                return (\"cls\", d)\n            ch = self.escape_char(True)\n            if ch is None:\n                self.err(\"bad escape \\\\\" + d)\n            return (\"ch\", ch)\n        self.i += 1\n        return (\"ch\", c)\n\n\ndef _width(node):\n    k = node[0]\n    if k == \"lit\" or k == \"any\" or k == \"set\":\n        return 1, 1\n    if k == \"cat\":\n        lo = hi = 0\n        for n in node[1]:\n            a, b = _width(n)\n            lo += a\n            hi = None if hi is None or b is None else hi + b\n        return lo, hi\n    if k == \"alt\":\n        ws = [_width(n) for n in node[1]]\n        lo = min(w[0] for w in ws)\n        his = [w[1] for w in ws]\n        return lo, None if None in his else max(his)\n    if k == \"group\" or k == \"atomic\":\n        return _width(node[-1])\n    if k == \"rep\":\n        a, b = _width(node[1])\n        hi = None if node[3] is None or b is None else b * node[3]\n        return a * node[2], hi\n    if k == \"at\" or k == \"look\":\n        return 0, 0\n    if k == \"bref\":\n        return 0, None\n    if k == \"cond\":\n        a = _width(node[2])\n        b = _width(node[3])\n        return min(a[0], b[0]), (None if a[1] is None or b[1] is None else max(a[1], b[1]))\n    return 0, 0\n\n\n# ---- compiling the tree to instructions\n_CHAR, _ANY, _SET, _SPLIT, _JMP, _SAVE, _AT, _BREF, _LOOK, _ATOMIC, _COND, _MARK, _PROG, _MATCH = range(14)\n\n\nclass _Compiler:\n    def __init__(self, nslots):\n        self.code = []\n        self.nslots = nslots\n        self.marks = 0\n\n    def emit(self, *ins):\n        self.code.append(list(ins))\n        return len(self.code) - 1\n\n    def sub(self, node):\n        c = _Compiler(self.nslots)\n        c.marks = self.marks\n        c.node(node)\n        c.emit(_MATCH)\n        self.marks = c.marks\n        return c.code\n\n    def node(self, n):\n        k = n[0]\n        if k == \"lit\":\n            if n[2] and n[1].lower() != n[1].upper():\n                self.emit(_SET, False, [(\"ch\", n[1])], True, False)\n            else:\n                self.emit(_CHAR, n[1])\n        elif k == \"any\":\n            self.emit(_ANY, n[1])\n        elif k == \"set\":\n            self.emit(_SET, n[1], n[2], n[3], n[4])\n        elif k == \"cat\":\n            for x in n[1]:\n                self.node(x)\n        elif k == \"alt\":\n            jumps = []\n            items = n[1]\n            for idx in range(len(items)):\n                if idx < len(items) - 1:\n                    sp = self.emit(_SPLIT, 0, 0)\n                    self.code[sp][1] = len(self.code)\n                    self.node(items[idx])\n                    jumps.append(self.emit(_JMP, 0))\n                    self.code[sp][2] = len(self.code)\n                else:\n                    self.node(items[idx])\n            for j in jumps:\n                self.code[j][1] = len(self.code)\n        elif k == \"group\":\n            self.emit(_SAVE, 2 * n[1])\n            self.node(n[2])\n            self.emit(_SAVE, 2 * n[1] + 1)\n        elif k == \"at\":\n            self.emit(_AT, n[1], n[2], n[3])\n        elif k == \"bref\":\n            self.emit(_BREF, n[1], n[2])\n        elif k == \"look\":\n            w = _width(n[3])[0] if not n[1] else 0\n            self.emit(_LOOK, self.sub(n[3]), n[1], n[2], w)\n        elif k == \"atomic\":\n            self.emit(_ATOMIC, self.sub(n[1]))\n        elif k == \"cond\":\n            c = self.emit(_COND, n[1], 0)\n            self.node(n[2])\n            j = self.emit(_JMP, 0)\n            self.code[c][2] = len(self.code)\n            self.node(n[3])\n            self.code[j][1] = len(self.code)\n        elif k == \"rep\":\n            self.rep(n)\n\n    def rep(self, n):\n        body, lo, hi, mode = n[1], n[2], n[3], n[4]\n        if mode == \"poss\":\n            self.emit(_ATOMIC, self.sub((\"rep\", body, lo, hi, \"greedy\")))\n            return\n        lazy = mode == \"lazy\"\n        nullable = _width(body)[0] == 0\n        for _ in range(lo):\n            self.node(body)\n        if hi is None:\n            # L: split body, out; body; jmp L\n            top = len(self.code)\n            sp = self.emit(_SPLIT, 0, 0)\n            mark = None\n            first = len(self.code)\n            if nullable:\n                mark = self.nslots + self.marks\n                self.marks += 1\n                self.emit(_MARK, mark)\n            self.node(body)\n            prog = None\n            if nullable:\n                prog = self.emit(_PROG, mark, 0)\n            self.emit(_JMP, top)\n            out = len(self.code)\n            if prog is not None:\n                self.code[prog][2] = out\n            self.code[sp][1], self.code[sp][2] = (out, first) if lazy else (first, out)\n        else:\n            splits = []\n            for _ in range(hi - lo):\n                sp = self.emit(_SPLIT, 0, 0)\n                splits.append((sp, len(self.code)))\n                self.node(body)\n            out = len(self.code)\n            for sp, first in splits:\n                self.code[sp][1], self.code[sp][2] = (out, first) if lazy else (first, out)\n\n\ndef _is_word(ch, asc):\n    if asc:\n        return ch.isascii() and (ch.isalnum() or ch == \"_\")\n    return ch.isalnum() or ch == \"_\"\n\n\ndef _in_cls(ch, cls, asc):\n    if cls == \"d\":\n        return (\"0\" <= ch <= \"9\") if asc else ch.isdigit() and ch.isdecimal()\n    if cls == \"D\":\n        return not _in_cls(ch, \"d\", asc)\n    if cls == \"w\":\n        return _is_word(ch, asc)\n    if cls == \"W\":\n        return not _is_word(ch, asc)\n    if cls == \"s\":\n        if asc:\n            return ch in _WS\n        return ch.isspace() or ch in \"\\x1c\\x1d\\x1e\\x1f\\x85\"\n    return not _in_cls(ch, \"s\", asc)\n\n\ndef _in_set(ch, items, icase, asc):\n    for it in items:\n        k = it[0]\n        if k == \"ch\":\n            if ch == it[1]:\n                return True\n            if icase and (ch.lower() == it[1].lower() or ch.upper() == it[1].upper()):\n                return True\n        elif k == \"range\":\n            if it[1] <= ch <= it[2]:\n                return True\n            if icase:\n                lo = ch.lower()\n                up = ch.upper()\n                if it[1] <= lo <= it[2] or it[1] <= up <= it[2]:\n                    return True\n        elif _in_cls(ch, it[1], asc):\n            return True\n    return False\n\n\ndef _run(code, s, start, end, sv, avoid):\n    # one match attempt at start; returns (end, slots) or None. avoid is a\n    # position where an empty match is not accepted (after an empty match)\n    stack = [(0, start, sv)]\n    while stack:\n        pc, i, sv = stack.pop()\n        while True:\n            ins = code[pc]\n            op = ins[0]\n            if op == _CHAR:\n                if i < end and s[i] == ins[1]:\n                    i += 1\n                    pc += 1\n                    continue\n                break\n            if op == _SET:\n                if i < end and _in_set(s[i], ins[2], ins[3], ins[4]) != ins[1]:\n                    i += 1\n                    pc += 1\n                    continue\n                break\n            if op == _SPLIT:\n                stack.append((ins[2], i, sv))\n                pc = ins[1]\n                continue\n            if op == _JMP:\n                pc = ins[1]\n                continue\n            if op == _ANY:\n                if i < end and (ins[1] or s[i] != \"\\n\"):\n                    i += 1\n                    pc += 1\n                    continue\n                break\n            if op == _SAVE:\n                sv = sv[:]\n                sv[ins[1]] = i\n                if ins[1] & 1:\n                    sv[-1] = ins[1] >> 1\n                pc += 1\n                continue\n            if op == _MARK:\n                sv = sv[:]\n                sv[ins[1]] = i\n                pc += 1\n                continue\n            if op == _PROG:\n                # an iteration that matched nothing ends the loop\n                if sv[ins[1]] == i:\n                    pc = ins[2]\n                else:\n                    pc += 1\n                continue\n            if op == _MATCH:\n                if avoid is not None and i == start == avoid:\n                    break\n                return i, sv\n            if op == _AT:\n                if _at(ins[1], ins[2], ins[3], s, i, end):\n                    pc += 1\n                    continue\n                break\n            if op == _BREF:\n                a = sv[2 * ins[1]]\n                b = sv[2 * ins[1] + 1]\n                if a < 0 or b < 0:\n                    break\n                g = s[a:b]\n                if ins[2]:\n                    if s[i:i + len(g)].lower() == g.lower() and i + len(g) <= end:\n                        i += len(g)\n                        pc += 1\n                        continue\n                    break\n                if i + len(g) <= end and s.startswith(g, i):\n                    i += len(g)\n                    pc += 1\n                    continue\n                break\n            if op == _LOOK:\n                if ins[2]:\n                    at = i\n                else:\n                    at = i - ins[4]\n                if at < 0:\n                    r = None\n                else:\n                    r = _run(ins[1], s, at, end if ins[2] else i, sv, None)\n                    if r is not None and not ins[2] and r[0] != i:\n                        r = None\n                if ins[3]:\n                    if r is not None:\n                        break\n                else:\n                    if r is None:\n                        break\n                    sv = r[1]\n                pc += 1\n                continue\n            if op == _ATOMIC:\n                r = _run(ins[1], s, i, end, sv, None)\n                if r is None:\n                    break\n                i, sv = r\n                pc += 1\n                continue\n            if op == _COND:\n                if sv[2 * ins[1] + 1] >= 0:\n                    pc += 1\n                else:\n                    pc = ins[2]\n                continue\n            break\n    return None\n\n\ndef _at(kind, ml, asc, s, i, end):\n    if kind == \"^\":\n        return i == 0 or (ml and s[i - 1] == \"\\n\")\n    if kind == \"$\":\n        if ml:\n            return i == end or s[i] == \"\\n\"\n        return i == end or (i == end - 1 and s[i] == \"\\n\")\n    if kind == \"A\":\n        return i == 0\n    if kind == \"Z\":\n        return i == end\n    a = i > 0 and _is_word(s[i - 1], asc)\n    b = i < end and _is_word(s[i], asc)\n    if kind == \"b\":\n        return a != b\n    return a == b\n\n\nclass Pattern:\n    def __init__(self, pattern, flags):\n        if isinstance(flags, RegexFlag) is False:\n            flags = RegexFlag(flags)\n        if not isinstance(pattern, str):\n            raise TypeError(\"first argument must be string or compiled pattern\")\n        if flags & 256 and flags & 32:\n            raise ValueError(\"ASCII and UNICODE flags are incompatible\")\n        if flags & 4:\n            raise ValueError(\"cannot use LOCALE flag with a str pattern\")\n        ps = _Parser(pattern, int(flags))\n        tree = ps.parse()\n        self.pattern = pattern\n        self.flags = RegexFlag(ps.flags | (0 if ps.flags & 256 else 32))\n        self.groups = ps.groups - 1\n        self.groupindex = dict(ps.names)\n        nslots = 2 * ps.groups\n        c = _Compiler(nslots)\n        c.node(tree)\n        c.emit(_MATCH)\n        self._code = c.code\n        self._slots = [-1] * (nslots + c.marks + 1)\n        # a pattern that starts with a plain character can skip ahead to it\n        self._first = c.code[0][1] if c.code[0][0] == _CHAR else None\n\n    def __repr__(self):\n        fl = int(self.flags) & ~32\n        r = repr(self.pattern)\n        if len(r) > 200:\n            r = r[:200]\n        if fl:\n            return \"re.compile(%s, %r)\" % (r, RegexFlag(fl))\n        return \"re.compile(%s)\" % r\n\n    def __eq__(self, other):\n        if not isinstance(other, Pattern):\n            return NotImplemented\n        return self.pattern == other.pattern and self.flags == other.flags\n\n    def __hash__(self):\n        return hash((self.pattern, int(self.flags)))\n\n    def __copy__(self):\n        return self\n\n    def __deepcopy__(self, memo):\n        return self\n\n    def __class_getitem__(cls, item):\n        import types\n        return types.GenericAlias(cls, item)\n\n    def _bounds(self, string, pos, endpos):\n        if not isinstance(string, str):\n            raise TypeError(\"expected string or bytes-like object, got '%s'\" % type(string).__name__)\n        n = len(string)\n        if pos < 0:\n            pos = 0\n        if endpos is None or endpos > n:\n            endpos = n\n        if pos > endpos:\n            pos = endpos\n        return pos, endpos\n\n    def _at(self, string, i, endpos, avoid=None):\n        r = _run(self._code, string, i, endpos, self._slots, avoid)\n        if r is None:\n            return None\n        return r\n\n    def _search(self, string, pos, endpos, avoid=None):\n        i = pos\n        first = self._first\n        while i <= endpos:\n            if first is not None:\n                i = string.find(first, i, endpos)\n                if i < 0:\n                    return None\n            r = _run(self._code, string, i, endpos, self._slots, avoid)\n            if r is not None:\n                return Match(self, string, pos, endpos, i, r[0], r[1])\n            i += 1\n        return None\n\n    def match(self, string, pos=0, endpos=None):\n        pos, endpos = self._bounds(string, pos, endpos)\n        r = _run(self._code, string, pos, endpos, self._slots, None)\n        if r is None:\n            return None\n        return Match(self, string, pos, endpos, pos, r[0], r[1])\n\n    def fullmatch(self, string, pos=0, endpos=None):\n        pos, endpos = self._bounds(string, pos, endpos)\n        code = self._full_code()\n        r = _run(code, string, pos, endpos, self._slots, None)\n        if r is None:\n            return None\n        return Match(self, string, pos, endpos, pos, r[0], r[1])\n\n    def _full_code(self):\n        fc = getattr(self, \"_fcode\", None)\n        if fc is None:\n            # the pattern, atomic so far as it must reach the end: append \\Z\n            fc = [list(x) for x in self._code]\n            fc[-1] = [_AT, \"Z\", False, False]\n            fc.append([_MATCH])\n            self._fcode = fc\n        return fc\n\n    def search(self, string, pos=0, endpos=None):\n        pos, endpos = self._bounds(string, pos, endpos)\n        return self._search(string, pos, endpos)\n\n    def finditer(self, string, pos=0, endpos=None):\n        pos, endpos = self._bounds(string, pos, endpos)\n        return iter(self._all(string, pos, endpos, 0))\n\n    def _all(self, string, pos, endpos, count):\n        out = []\n        avoid = None\n        i = pos\n        while i <= endpos:\n            m = self._search(string, i, endpos, avoid)\n            if m is None:\n                break\n            m.pos = pos\n            out.append(m)\n            if count and len(out) >= count:\n                break\n            a, b = m.span()\n            if a == b:\n                avoid = b\n            else:\n                avoid = None\n            i = b\n        return out\n\n    def findall(self, string, pos=0, endpos=None):\n        pos, endpos = self._bounds(string, pos, endpos)\n        out = []\n        for m in self._all(string, pos, endpos, 0):\n            if self.groups == 0:\n                out.append(m.group())\n            elif self.groups == 1:\n                out.append(m.group(1) or \"\")\n            else:\n                out.append(tuple(g or \"\" for g in m.groups()))\n        return out\n\n    def split(self, string, maxsplit=0):\n        pos, endpos = self._bounds(string, 0, None)\n        out = []\n        last = 0\n        for m in self._all(string, 0, endpos, maxsplit if maxsplit > 0 else 0):\n            if maxsplit < 0:\n                break\n            a, b = m.span()\n            out.append(string[last:a])\n            out.extend(m.groups())\n            last = b\n        out.append(string[last:])\n        return out\n\n    def subn(self, repl, string, count=0):\n        pos, endpos = self._bounds(string, 0, None)\n        if callable(repl):\n            fn = repl\n        else:\n            if not isinstance(repl, str):\n                raise TypeError(\"expected str instance, %s found\" % type(repl).__name__)\n            tpl = _template(self, repl)\n            if len(tpl) == 1 and isinstance(tpl[0], str):\n                lit = tpl[0]\n                fn = lambda m: lit\n            else:\n                fn = lambda m: _expand(m, tpl)\n        parts = []\n        last = 0\n        n = 0\n        for m in self._all(string, 0, endpos, count if count > 0 else 0):\n            if count < 0:\n                break\n            a, b = m.span()\n            parts.append(string[last:a])\n            r = fn(m)\n            if r is None:\n                r = \"\"\n            if not isinstance(r, str):\n                raise TypeError(\"expected str instance, %s found\" % type(r).__name__)\n            parts.append(r)\n            last = b\n            n += 1\n        parts.append(string[last:])\n        return \"\".join(parts), n\n\n    def sub(self, repl, string, count=0):\n        return self.subn(repl, string, count)[0]\n\n    def scanner(self, string, pos=0, endpos=None):\n        return _Scanner(self, string, pos, endpos)\n\n\nclass _Scanner:\n    def __init__(self, pattern, string, pos, endpos):\n        self.pattern = pattern\n        self._s = string\n        self._pos, self._end = pattern._bounds(string, pos, endpos)\n        self._avoid = None\n\n    def match(self):\n        if self._pos > self._end:\n            return None\n        r = _run(self.pattern._code, self._s, self._pos, self._end, self.pattern._slots, self._avoid)\n        if r is None:\n            return None\n        m = Match(self.pattern, self._s, self._pos, self._end, self._pos, r[0], r[1])\n        self._avoid = r[0] if r[0] == self._pos else None\n        self._pos = r[0]\n        return m\n\n    def search(self):\n        if self._pos > self._end:\n            return None\n        m = self.pattern._search(self._s, self._pos, self._end, self._avoid)\n        if m is None:\n            self._pos = self._end + 1\n            return None\n        a, b = m.span()\n        self._avoid = b if a == b else None\n        self._pos = b\n        return m\n\n\ndef _template(pattern, repl):\n    # a replacement string as literal parts and group numbers\n    out = []\n    lit = []\n    i = 0\n    n = len(repl)\n    simple = {\"n\": \"\\n\", \"t\": \"\\t\", \"r\": \"\\r\", \"f\": \"\\f\", \"v\": \"\\v\", \"a\": \"\\a\", \"b\": \"\\b\", \"\\\\\": \"\\\\\"}\n    while i < n:\n        c = repl[i]\n        if c != \"\\\\\":\n            lit.append(c)\n            i += 1\n            continue\n        if i + 1 >= n:\n            raise error(\"bad escape (end of pattern)\", repl, i)\n        d = repl[i + 1]\n        if d == \"g\":\n            if not repl.startswith(\"<\", i + 2):\n                raise error(\"missing <\", repl, i + 2)\n            j = repl.find(\">\", i + 3)\n            if j < 0:\n                raise error(\"missing >, unterminated name\", repl, i + 3)\n            name = repl[i + 3:j]\n            if not name:\n                raise error(\"missing group name\", repl, i + 3)\n            if name.isdigit() and name.isascii():\n                g = int(name)\n                if g > pattern.groups:\n                    raise error(\"invalid group reference %d\" % g, repl, i + 3)\n            elif name.isidentifier():\n                if name not in pattern.groupindex:\n                    raise IndexError(\"unknown group name %r\" % name)\n                g = pattern.groupindex[name]\n            else:\n                raise error(\"bad character in group name %r\" % name, repl, i + 3)\n            out.append(\"\".join(lit))\n            lit = []\n            out.append(g)\n            i = j + 1\n        elif d == \"0\":\n            j = i + 2\n            while j < n and j < i + 4 and repl[j] in \"01234567\":\n                j += 1\n            lit.append(chr(int(repl[i + 1:j], 8)))\n            i = j\n        elif d in \"123456789\":\n            j = i + 1\n            if (j + 2 < n + 0 and repl[j] in \"01234567\" and repl[j + 1] in \"01234567\"\n                    and repl[j + 2] in \"01234567\"):\n                v = int(repl[j:j + 3], 8)\n                if v > 0o377:\n                    raise error(\"octal escape value \\\\%s outside of range 0-0o377\" % repl[j:j + 3], repl, i)\n                lit.append(chr(v))\n                i = j + 3\n                continue\n            k = j\n            while k < n and k < j + 2 and repl[k] in _DIGITS:\n                k += 1\n            g = int(repl[j:k])\n            if g > pattern.groups:\n                raise error(\"invalid group reference %d\" % g, repl, j)\n            out.append(\"\".join(lit))\n            lit = []\n            out.append(g)\n            i = k\n        elif d in simple:\n            lit.append(simple[d])\n            i += 2\n        elif d.isascii() and d.isalpha():\n            raise error(\"bad escape \\\\\" + d, repl, i)\n        else:\n            lit.append(\"\\\\\" + d)\n            i += 2\n    out.append(\"\".join(lit))\n    return [x for x in out if x != \"\"] or [\"\"]\n\n\ndef _expand(m, tpl):\n    parts = []\n    for x in tpl:\n        if isinstance(x, str):\n            parts.append(x)\n        else:\n            parts.append(m.group(x) or \"\")\n    return \"\".join(parts)\n\n\nclass Match:\n    def __init__(self, pattern, string, pos, endpos, start, end, slots):\n        self.re = pattern\n        self.string = string\n        self.pos = pos\n        self.endpos = endpos\n        n = pattern.groups + 1\n        sp = [(start, end)]\n        for g in range(1, n):\n            a = slots[2 * g]\n            b = slots[2 * g + 1]\n            sp.append((a, b) if a >= 0 and b >= 0 else (-1, -1))\n        self._spans = sp\n        last = slots[-1] if slots[-1] > 0 else None\n        self.lastindex = last\n        self.lastgroup = None\n        if last is not None:\n            for k, v in pattern.groupindex.items():\n                if v == last:\n                    self.lastgroup = k\n\n    def _index(self, g):\n        if isinstance(g, str):\n            if g in self.re.groupindex:\n                return self.re.groupindex[g]\n            raise IndexError(\"no such group\")\n        if isinstance(g, int) and not isinstance(g, bool) or hasattr(g, \"__index__\"):\n            try:\n                g = g.__index__()\n            except AttributeError:\n                pass\n            if 0 <= g <= self.re.groups:\n                return g\n        raise IndexError(\"no such group\")\n\n    def _get(self, g, default=None):\n        a, b = self._spans[self._index(g)]\n        if a < 0:\n            return default\n        return self.string[a:b]\n\n    def group(self, *args):\n        if not args:\n            return self._get(0)\n        if len(args) == 1:\n            return self._get(args[0])\n        return tuple(self._get(g) for g in args)\n\n    def __getitem__(self, g):\n        return self._get(g)\n\n    def groups(self, default=None):\n        return tuple(self._get(g, default) for g in range(1, self.re.groups + 1))\n\n    def groupdict(self, default=None):\n        return {k: self._get(v, default) for k, v in self.re.groupindex.items()}\n\n    def start(self, group=0):\n        return self._spans[self._index(group)][0]\n\n    def end(self, group=0):\n        return self._spans[self._index(group)][1]\n\n    def span(self, group=0):\n        return self._spans[self._index(group)]\n\n    @property\n    def regs(self):\n        return tuple(self._spans)\n\n    def expand(self, template):\n        return _expand(self, _template(self.re, template))\n\n    def __repr__(self):\n        return \"<re.Match object; span=%r, match=%r>\" % (self.span(), self.group())\n\n    def __bool__(self):\n        return True\n\n    def __copy__(self):\n        return self\n\n    def __deepcopy__(self, memo):\n        return self\n\n    def __class_getitem__(cls, item):\n        import types\n        return types.GenericAlias(cls, item)\n\n\n_cache = {}\n\n\ndef _compile(pattern, flags):\n    if isinstance(pattern, Pattern):\n        if flags:\n            raise ValueError(\"cannot process flags argument with a compiled pattern\")\n        return pattern\n    key = (type(pattern).__name__, pattern, int(flags))\n    p = _cache.get(key)\n    if p is None:\n        p = Pattern(pattern, flags)\n        if len(_cache) >= 512:\n            _cache.clear()\n        _cache[key] = p\n    return p\n\n\ndef compile(pattern, flags=0):\n    return _compile(pattern, flags)\n\n\ndef match(pattern, string, flags=0):\n    return _compile(pattern, flags).match(string)\n\n\ndef fullmatch(pattern, string, flags=0):\n    return _compile(pattern, flags).fullmatch(string)\n\n\ndef search(pattern, string, flags=0):\n    return _compile(pattern, flags).search(string)\n\n\ndef findall(pattern, string, flags=0):\n    return _compile(pattern, flags).findall(string)\n\n\ndef finditer(pattern, string, flags=0):\n    return _compile(pattern, flags).finditer(string)\n\n\ndef sub(pattern, repl, string, count=0, flags=0):\n    return _compile(pattern, flags).sub(repl, string, count)\n\n\ndef subn(pattern, repl, string, count=0, flags=0):\n    return _compile(pattern, flags).subn(repl, string, count)\n\n\ndef split(pattern, string, maxsplit=0, flags=0):\n    return _compile(pattern, flags).split(string, maxsplit)\n\n\ndef purge():\n    _cache.clear()\n\n\n_ALNUM = frozenset(\"_abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789\")\n\n\ndef escape(pattern):\n    out = []\n    for c in pattern:\n        if c in _ALNUM or ord(c) >= 128:\n            out.append(c)\n        elif c == \"\\x00\":\n            out.append(\"\\\\000\" if False else \"\\\\\\x00\")\n        else:\n            out.append(\"\\\\\" + c)\n    return \"\".join(out)\n\n\nclass Scanner:\n    def __init__(self, lexicon, flags=0):\n        self.lexicon = lexicon\n        self._pats = [(compile(p, flags), action) for p, action in lexicon]\n\n    def scan(self, string):\n        result = []\n        i = 0\n        while True:\n            best = None\n            for p, action in self._pats:\n                m = p.match(string, i)\n                if m is not None:\n                    best = (m, action)\n                    break\n            if best is None:\n                break\n            m, action = best\n            j = m.end()\n            if j == i:\n                break\n            if callable(action):\n                self.match = m\n                action = action(self, m.group())\n            if action is not None:\n                result.append(action)\n            i = j\n        return result, string[i:]\n";
    return s29;
  }
  if ( name == "shlex" ) {
    const s30 = "# SPDX-License-Identifier: MIT\n# shlex for the Ranger Python runtime: split, quote and join\nimport re\n\n_find_unsafe = re.compile(r\"[^\\w@%+=:,./-]\", re.ASCII).search\n\n\ndef quote(s):\n    if not s:\n        return \"''\"\n    if _find_unsafe(s) is None:\n        return s\n    return \"'\" + s.replace(\"'\", \"'\\\"'\\\"'\") + \"'\"\n\n\ndef join(split_command):\n    return \" \".join(quote(arg) for arg in split_command)\n\n\ndef split(s, comments=False, posix=True):\n    if s is None:\n        raise ValueError(\"s argument must not be None\")\n    out = []\n    cur = []\n    have = False\n    i = 0\n    n = len(s)\n    while i < n:\n        c = s[i]\n        if c in \" \\t\\r\\n\":\n            if have:\n                out.append(\"\".join(cur))\n                cur = []\n                have = False\n            i += 1\n        elif comments and c == \"#\":\n            while i < n and s[i] != \"\\n\":\n                i += 1\n        elif c == \"'\":\n            j = s.find(\"'\", i + 1)\n            if j < 0:\n                raise ValueError(\"No closing quotation\")\n            cur.append(s[i + 1:j])\n            have = True\n            i = j + 1\n        elif c == '\"':\n            i += 1\n            while True:\n                if i >= n:\n                    raise ValueError(\"No closing quotation\")\n                if s[i] == '\"':\n                    break\n                if s[i] == \"\\\\\" and i + 1 < n and s[i + 1] in '\"\\\\$`\\n':\n                    cur.append(s[i + 1])\n                    i += 2\n                    continue\n                cur.append(s[i])\n                i += 1\n            have = True\n            i += 1\n        elif c == \"\\\\\" and posix:\n            if i + 1 < n:\n                cur.append(s[i + 1])\n            have = True\n            i += 2\n        else:\n            cur.append(c)\n            have = True\n            i += 1\n    if have:\n        out.append(\"\".join(cur))\n    return out\n";
    return s30;
  }
  if ( name == "shutil" ) {
    const s31 = "# SPDX-License-Identifier: MIT\n# shutil for the Ranger Python runtime, on its file system\nimport os\n\n\nclass Error(OSError):\n    pass\n\n\ndef copyfile(src, dst, *, follow_symlinks=True):\n    with open(src) as f:\n        data = f.read()\n    with open(dst, \"w\") as f:\n        f.write(data)\n    return dst\n\n\ndef copy(src, dst, *, follow_symlinks=True):\n    if os.path.isdir(dst):\n        dst = os.path.join(dst, os.path.basename(src))\n    return copyfile(src, dst)\n\n\ncopy2 = copy\n\n\ndef copymode(src, dst, *, follow_symlinks=True):\n    pass\n\n\ncopystat = copymode\n\n\ndef rmtree(path, ignore_errors=False, onerror=None, *, onexc=None, dir_fd=None):\n    try:\n        for name in os.listdir(path):\n            full = os.path.join(path, name)\n            if os.path.isdir(full):\n                rmtree(full, ignore_errors)\n            else:\n                os.remove(full)\n        if hasattr(os, \"rmdir\"):\n            os.rmdir(path)\n    except OSError:\n        if not ignore_errors:\n            raise\n\n\ndef move(src, dst):\n    copyfile(src, dst)\n    os.remove(src)\n    return dst\n\n\ndef which(cmd, mode=None, path=None):\n    return None\n\n\ndef get_terminal_size(fallback=(80, 24)):\n    return os.terminal_size(fallback) if hasattr(os, \"terminal_size\") else fallback\n\n\ndef disk_usage(path):\n    return (0, 0, 0)\n";
    return s31;
  }
  if ( name == "string" ) {
    const s32 = "# SPDX-License-Identifier: MIT\n# string for the Ranger Python runtime: the constants come from the\n# native _string, Template, Formatter and capwords are written here.\nfrom _string import *\nimport re as _re\n\n__all__ = [\"ascii_letters\", \"ascii_lowercase\", \"ascii_uppercase\", \"capwords\",\n           \"digits\", \"hexdigits\", \"octdigits\", \"printable\", \"punctuation\",\n           \"whitespace\", \"Formatter\", \"Template\"]\n\n\ndef capwords(s, sep=None):\n    return (sep or \" \").join(map(str.capitalize, s.split(sep)))\n\n\nclass Template:\n    delimiter = \"$\"\n    idpattern = r\"(?a:[_a-z][_a-z0-9]*)\"\n    braceidpattern = None\n    flags = _re.IGNORECASE\n\n    def __init_subclass__(cls, **kw):\n        cls.pattern = cls._compile()\n\n    @classmethod\n    def _compile(cls):\n        if \"pattern\" in cls.__dict__ and not isinstance(cls.__dict__[\"pattern\"], str):\n            return cls.__dict__[\"pattern\"]\n        d = _re.escape(cls.delimiter)\n        ident = cls.idpattern.replace(\"(?a:\", \"(?:\")\n        bid = cls.braceidpattern or ident\n        src = cls.__dict__.get(\"pattern\")\n        if not isinstance(src, str):\n            src = (d + r\"(?:(?P<escaped>\" + d + r\")|(?P<named>\" + ident + r\")|\\{(?P<braced>\" + bid +\n                   r\")\\}|(?P<invalid>))\")\n        return _re.compile(src, cls.flags | _re.VERBOSE if src is cls.__dict__.get(\"pattern\") else cls.flags)\n\n    def __init__(self, template):\n        self.template = template\n\n    def _pat(self):\n        p = type(self).__dict__.get(\"pattern\")\n        if p is None or isinstance(p, str):\n            p = type(self)._compile()\n        return p\n\n    def _invalid(self, mo):\n        i = mo.start(\"invalid\")\n        lines = self.template[:i].splitlines(keepends=True)\n        if not lines:\n            colno = 1\n            lineno = 1\n        else:\n            colno = i - len(\"\".join(lines[:-1]))\n            lineno = len(lines)\n        raise ValueError(\"Invalid placeholder in string: line %d, col %d\" % (lineno, colno))\n\n    def substitute(self, mapping=None, /, **kws):\n        if mapping is None:\n            mapping = kws\n        elif kws:\n            m = dict(mapping)\n            m.update(kws)\n            mapping = m\n\n        def convert(mo):\n            named = mo.group(\"named\") or mo.group(\"braced\")\n            if named is not None:\n                return str(mapping[named])\n            if mo.group(\"escaped\") is not None:\n                return self.delimiter\n            if mo.group(\"invalid\") is not None:\n                self._invalid(mo)\n            raise ValueError(\"Unrecognized named group in pattern\", self.pattern)\n        return self._pat().sub(convert, self.template)\n\n    def safe_substitute(self, mapping=None, /, **kws):\n        if mapping is None:\n            mapping = kws\n        elif kws:\n            m = dict(mapping)\n            m.update(kws)\n            mapping = m\n\n        def convert(mo):\n            named = mo.group(\"named\") or mo.group(\"braced\")\n            if named is not None:\n                try:\n                    return str(mapping[named])\n                except KeyError:\n                    return mo.group()\n            if mo.group(\"escaped\") is not None:\n                return self.delimiter\n            if mo.group(\"invalid\") is not None:\n                return mo.group()\n            raise ValueError(\"Unrecognized named group in pattern\", self.pattern)\n        return self._pat().sub(convert, self.template)\n\n    def is_valid(self):\n        for mo in self._pat().finditer(self.template):\n            if mo.group(\"invalid\") is not None:\n                return False\n            if mo.group(\"named\") is None and mo.group(\"braced\") is None and mo.group(\"escaped\") is None:\n                raise ValueError(\"Unrecognized named group in pattern\", self.pattern)\n        return True\n\n    def get_identifiers(self):\n        ids = []\n        for mo in self._pat().finditer(self.template):\n            named = mo.group(\"named\") or mo.group(\"braced\")\n            if named is not None and named not in ids:\n                ids.append(named)\n            elif named is None and mo.group(\"invalid\") is None and mo.group(\"escaped\") is None:\n                raise ValueError(\"Unrecognized named group in pattern\", self.pattern)\n        return ids\n\n\nclass Formatter:\n    def format(self, format_string, /, *args, **kwargs):\n        return self.vformat(format_string, args, kwargs)\n\n    def vformat(self, format_string, args, kwargs):\n        used = set()\n        result, _ = self._vformat(format_string, args, kwargs, used, 2)\n        self.check_unused_args(used, args, kwargs)\n        return result\n\n    def _vformat(self, format_string, args, kwargs, used_args, recursion_depth, auto_arg_index=0):\n        if recursion_depth < 0:\n            raise ValueError(\"Max string recursion exceeded\")\n        result = []\n        for literal_text, field_name, format_spec, conversion in self.parse(format_string):\n            if literal_text:\n                result.append(literal_text)\n            if field_name is not None:\n                if field_name == \"\":\n                    if auto_arg_index is False:\n                        raise ValueError(\"cannot switch from manual field specification to automatic field numbering\")\n                    field_name = str(auto_arg_index)\n                    auto_arg_index += 1\n                elif field_name.isdigit():\n                    if auto_arg_index:\n                        raise ValueError(\"cannot switch from manual field specification to automatic field numbering\")\n                    auto_arg_index = False\n                obj, arg_used = self.get_field(field_name, args, kwargs)\n                used_args.add(arg_used)\n                obj = self.convert_field(obj, conversion)\n                format_spec, auto_arg_index = self._vformat(format_spec, args, kwargs, used_args,\n                                                            recursion_depth - 1, auto_arg_index=auto_arg_index)\n                result.append(self.format_field(obj, format_spec))\n        return \"\".join(result), auto_arg_index\n\n    def get_value(self, key, args, kwargs):\n        if isinstance(key, int):\n            return args[key]\n        return kwargs[key]\n\n    def check_unused_args(self, used_args, args, kwargs):\n        pass\n\n    def format_field(self, value, format_spec):\n        return format(value, format_spec)\n\n    def convert_field(self, value, conversion):\n        if conversion is None:\n            return value\n        if conversion == \"s\":\n            return str(value)\n        if conversion == \"r\":\n            return repr(value)\n        if conversion == \"a\":\n            return ascii(value)\n        raise ValueError(\"Unknown conversion specifier \" + str(conversion))\n\n    def parse(self, format_string):\n        out = []\n        i = 0\n        n = len(format_string)\n        lit = []\n        while i < n:\n            c = format_string[i]\n            if c == \"{\":\n                if i + 1 < n and format_string[i + 1] == \"{\":\n                    lit.append(\"{\")\n                    i += 2\n                    continue\n                depth = 1\n                j = i + 1\n                while j < n and depth:\n                    if format_string[j] == \"{\":\n                        depth += 1\n                    elif format_string[j] == \"}\":\n                        depth -= 1\n                    j += 1\n                if depth:\n                    raise ValueError(\"expected '}' before end of string\")\n                field = format_string[i + 1:j - 1]\n                conv = None\n                spec = \"\"\n                k = 0\n                bracket = 0\n                name_end = len(field)\n                while k < len(field):\n                    ch = field[k]\n                    if ch == \"[\":\n                        bracket += 1\n                    elif ch == \"]\":\n                        bracket -= 1\n                    elif bracket == 0 and ch in \"!:\":\n                        name_end = k\n                        break\n                    k += 1\n                name = field[:name_end]\n                rest = field[name_end:]\n                if rest.startswith(\"!\"):\n                    if len(rest) < 2:\n                        raise ValueError(\"end of string while looking for conversion specifier\")\n                    conv = rest[1]\n                    rest = rest[2:]\n                    if rest and not rest.startswith(\":\"):\n                        raise ValueError(\"expected ':' after conversion specifier\")\n                if rest.startswith(\":\"):\n                    spec = rest[1:]\n                out.append((\"\".join(lit), name, spec, conv))\n                lit = []\n                i = j\n            elif c == \"}\":\n                if i + 1 < n and format_string[i + 1] == \"}\":\n                    lit.append(\"}\")\n                    i += 2\n                    continue\n                raise ValueError(\"Single '}' encountered in format string\")\n            else:\n                lit.append(c)\n                i += 1\n        if lit:\n            out.append((\"\".join(lit), None, None, None))\n        return out\n\n    def get_field(self, field_name, args, kwargs):\n        first = field_name\n        rest = \"\"\n        for k, ch in enumerate(field_name):\n            if ch in \".[\":\n                first = field_name[:k]\n                rest = field_name[k:]\n                break\n        key = int(first) if first.isdigit() else first\n        obj = self.get_value(key, args, kwargs)\n        while rest:\n            if rest[0] == \".\":\n                k = 1\n                while k < len(rest) and rest[k] not in \".[\":\n                    k += 1\n                obj = getattr(obj, rest[1:k])\n                rest = rest[k:]\n            else:\n                k = rest.index(\"]\")\n                idx = rest[1:k]\n                obj = obj[int(idx) if idx.isdigit() else idx]\n                rest = rest[k + 1:]\n        return obj, key\nTemplate.pattern = Template._compile()\n";
    return s32;
  }
  if ( name == "tempfile" ) {
    const s33 = "# SPDX-License-Identifier: MIT\n# tempfile for the Ranger Python runtime, on its file system\nimport os\n\ntempdir = \"/tmp\"\ntemplate = \"tmp\"\n_count = [0]\n\n\ndef gettempdir():\n    return tempdir\n\n\ndef _name(prefix, suffix, dir):\n    _count[0] += 1\n    return os.path.join(dir or tempdir, \"%s%d%s\" % (prefix or template, _count[0], suffix or \"\"))\n\n\ndef mkdtemp(suffix=None, prefix=None, dir=None):\n    path = _name(prefix, suffix, dir)\n    os.makedirs(path, exist_ok=True)\n    return path\n\n\ndef mkstemp(suffix=None, prefix=None, dir=None, text=False):\n    path = _name(prefix, suffix, dir)\n    with open(path, \"w\") as f:\n        f.write(\"\")\n    return (3, path)\n\n\ndef mktemp(suffix=\"\", prefix=template, dir=None):\n    return _name(prefix, suffix, dir)\n\n\nclass TemporaryDirectory:\n    def __init__(self, suffix=None, prefix=None, dir=None, ignore_cleanup_errors=False, *, delete=True):\n        self.name = mkdtemp(suffix, prefix, dir)\n\n    def __enter__(self):\n        return self.name\n\n    def __exit__(self, *exc):\n        self.cleanup()\n\n    def cleanup(self):\n        import shutil\n        shutil.rmtree(self.name, ignore_errors=True)\n\n\ndef NamedTemporaryFile(mode=\"w+b\", buffering=-1, encoding=None, newline=None, suffix=None,\n                       prefix=None, dir=None, delete=True, *, errors=None, delete_on_close=True):\n    path = _name(prefix, suffix, dir)\n    f = open(path, \"w\")\n    return f\n\n\nTemporaryFile = NamedTemporaryFile\n";
    return s33;
  }
  if ( name == "textwrap" ) {
    const s34 = "# SPDX-License-Identifier: MIT\n# textwrap for the Ranger Python runtime: dedent, indent, wrap, fill, shorten.\n\ndef dedent(text):\n    lines = text.split(\"\\n\")\n    margin = None\n    for line in lines:\n        stripped = line.lstrip(\" \\t\")\n        if not stripped:\n            continue\n        ws = line[:len(line) - len(stripped)]\n        if margin is None:\n            margin = ws\n        else:\n            n = 0\n            while n < len(margin) and n < len(ws) and margin[n] == ws[n]:\n                n += 1\n            margin = margin[:n]\n    out = []\n    for line in lines:\n        if not line.strip(\" \\t\"):\n            out.append(line.lstrip(\" \\t\") if line.strip() == \"\" else line)\n        elif margin:\n            out.append(line[len(margin):])\n        else:\n            out.append(line)\n    return \"\\n\".join(\"\" if l.strip() == \"\" else l for l in out)\n\n\ndef indent(text, prefix, predicate=None):\n    if predicate is None:\n        predicate = lambda line: line.strip()\n    out = []\n    for line in text.splitlines(True):\n        out.append(prefix + line if predicate(line) else line)\n    return \"\".join(out)\n\n\ndef wrap(text, width=70, initial_indent=\"\", subsequent_indent=\"\", break_long_words=True, **kwargs):\n    words = text.split()\n    lines = []\n    cur = initial_indent\n    empty = True\n    for w in words:\n        prefix = initial_indent if not lines else subsequent_indent\n        if empty:\n            cand = prefix + w\n        else:\n            cand = cur + \" \" + w\n        if len(cand) <= width or empty and not break_long_words:\n            cur = cand\n            empty = False\n        elif empty:\n            while len(prefix + w) > width:\n                lines.append(prefix + w[:width - len(prefix)])\n                w = w[width - len(prefix):]\n                prefix = subsequent_indent\n            cur = prefix + w\n            empty = False\n        else:\n            lines.append(cur)\n            cur = subsequent_indent + w\n    if not empty:\n        lines.append(cur)\n    return lines\n\n\ndef fill(text, width=70, **kwargs):\n    return \"\\n\".join(wrap(text, width, **kwargs))\n\n\ndef shorten(text, width, placeholder=\" [...]\", **kwargs):\n    text = \" \".join(text.split())\n    if len(text) <= width:\n        return text\n    words = text.split()\n    out = \"\"\n    for w in words:\n        cand = w if not out else out + \" \" + w\n        if len(cand) + len(placeholder) > width:\n            break\n        out = cand\n    return out + placeholder\n";
    return s34;
  }
  if ( name == "threading" ) {
    const s35 = "# SPDX-License-Identifier: MIT\n# threading for the Ranger Python runtime. There is one thread: start()\n# runs the target to the end, and locks never have to wait.\n\n_ident = [1]\n\n\nclass Lock:\n    def __init__(self):\n        self._locked = False\n\n    def acquire(self, blocking=True, timeout=-1):\n        if self._locked:\n            if not blocking:\n                return False\n            raise RuntimeError(\"deadlock: the lock is held and there is only one thread\")\n        self._locked = True\n        return True\n\n    def release(self):\n        if not self._locked:\n            raise RuntimeError(\"release unlocked lock\")\n        self._locked = False\n\n    def locked(self):\n        return self._locked\n\n    def __enter__(self):\n        return self.acquire()\n\n    def __exit__(self, *exc):\n        self.release()\n\n\nclass RLock:\n    def __init__(self):\n        self._count = 0\n\n    def acquire(self, blocking=True, timeout=-1):\n        self._count += 1\n        return True\n\n    def release(self):\n        if not self._count:\n            raise RuntimeError(\"cannot release un-acquired lock\")\n        self._count -= 1\n\n    def __enter__(self):\n        return self.acquire()\n\n    def __exit__(self, *exc):\n        self.release()\n\n\nclass Condition:\n    def __init__(self, lock=None):\n        self._lock = lock or RLock()\n\n    def __enter__(self):\n        return self._lock.__enter__()\n\n    def __exit__(self, *exc):\n        return self._lock.__exit__(*exc)\n\n    def wait(self, timeout=None):\n        return True\n\n    def wait_for(self, predicate, timeout=None):\n        return predicate()\n\n    def notify(self, n=1):\n        pass\n\n    def notify_all(self):\n        pass\n\n\nclass Semaphore:\n    def __init__(self, value=1):\n        self._value = value\n\n    def acquire(self, blocking=True, timeout=None):\n        if self._value <= 0:\n            return False\n        self._value -= 1\n        return True\n\n    def release(self, n=1):\n        self._value += n\n\n    __enter__ = acquire\n\n    def __exit__(self, *exc):\n        self.release()\n\n\nBoundedSemaphore = Semaphore\n\n\nclass Event:\n    def __init__(self):\n        self._flag = False\n\n    def is_set(self):\n        return self._flag\n\n    def set(self):\n        self._flag = True\n\n    def clear(self):\n        self._flag = False\n\n    def wait(self, timeout=None):\n        return self._flag\n\n\nclass local:\n    pass\n\n\nclass Thread:\n    def __init__(self, group=None, target=None, name=None, args=(), kwargs=None, *, daemon=None):\n        self._target = target\n        self._args = args\n        self._kwargs = kwargs or {}\n        _ident[0] += 1\n        self.name = name or \"Thread-%d\" % (_ident[0] - 1)\n        self.ident = None\n        self.daemon = bool(daemon)\n        self._started = False\n        self._done = False\n\n    def start(self):\n        if self._started:\n            raise RuntimeError(\"threads can only be started once\")\n        self._started = True\n        self.ident = _ident[0]\n        try:\n            self.run()\n        finally:\n            self._done = True\n\n    def run(self):\n        if self._target is not None:\n            self._target(*self._args, **self._kwargs)\n\n    def join(self, timeout=None):\n        pass\n\n    def is_alive(self):\n        return self._started and not self._done\n\n\nclass _MainThread(Thread):\n    def __init__(self):\n        Thread.__init__(self, name=\"MainThread\")\n        self.ident = 1\n        self._started = True\n\n\n_main = _MainThread()\n\n\ndef current_thread():\n    return _main\n\n\ndef main_thread():\n    return _main\n\n\ndef get_ident():\n    return 1\n\n\ndef active_count():\n    return 1\n\n\ndef enumerate():\n    return [_main]\n\n\nTIMEOUT_MAX = 2147483.0\n";
    return s35;
  }
  if ( name == "traceback" ) {
    const s36 = "# SPDX-License-Identifier: MIT\n# traceback for the Ranger Python runtime: the exception's type and message.\nimport sys\n\ndef format_exception_only(exc, value=None):\n    if value is None and isinstance(exc, BaseException):\n        value = exc\n    msg = str(value)\n    name = type(value).__name__\n    return [name + (\": \" + msg if msg else \"\") + \"\\n\"]\n\ndef format_exception(exc, value=None, tb=None, limit=None, chain=True):\n    return [\"Traceback (most recent call last):\\n\"] + format_exception_only(exc, value)\n\ndef format_exc(limit=None, chain=True):\n    e = sys.exception()\n    if e is None:\n        return \"NoneType: None\\n\"\n    return \"\".join(format_exception(type(e), e))\n\ndef print_exc(limit=None, file=None, chain=True):\n    print(format_exc(), end=\"\", file=file)\n\ndef print_exception(exc, value=None, tb=None, limit=None, file=None, chain=True):\n    print(\"\".join(format_exception(exc, value)), end=\"\", file=file)\n\ndef format_stack(f=None, limit=None):\n    return []\n\ndef extract_stack(f=None, limit=None):\n    return []\n";
    return s36;
  }
  if ( name == "types" ) {
    const s37 = "# SPDX-License-Identifier: MIT\n# types for the Ranger Python runtime.\n\ndef _f():\n    pass\n\nclass _C:\n    def _m(self):\n        pass\n\nFunctionType = type(_f)\nLambdaType = type(lambda: None)\nMethodType = type(_C()._m)\nBuiltinFunctionType = type(len)\nBuiltinMethodType = type([].append)\nModuleType = type(__import__(\"sys\"))\nNoneType = type(None)\nEllipsisType = type(...)\nNotImplementedType = type(NotImplemented)\nGeneratorType = type((x for x in ()))\n\n\nclass SimpleNamespace:\n    def __init__(self, mapping_or_iterable=(), /, **kwargs):\n        if isinstance(mapping_or_iterable, dict):\n            for k in mapping_or_iterable:\n                setattr(self, k, mapping_or_iterable[k])\n        else:\n            for k, v in mapping_or_iterable:\n                setattr(self, k, v)\n        for k in kwargs:\n            setattr(self, k, kwargs[k])\n\n    def __repr__(self):\n        items = [\"%s=%r\" % (k, v) for k, v in self.__dict__.items()]\n        return \"namespace(\" + \", \".join(items) + \")\"\n\n    def __eq__(self, other):\n        return isinstance(other, SimpleNamespace) and self.__dict__ == other.__dict__\n\n\nclass MappingProxyType:\n    def __init__(self, mapping):\n        self._m = mapping\n\n    def __getitem__(self, k):\n        return self._m[k]\n\n    def __len__(self):\n        return len(self._m)\n\n    def __iter__(self):\n        return iter(self._m)\n\n    def __contains__(self, k):\n        return k in self._m\n\n    def get(self, k, default=None):\n        return self._m.get(k, default)\n\n    def keys(self):\n        return self._m.keys()\n\n    def values(self):\n        return self._m.values()\n\n    def items(self):\n        return self._m.items()\n\n    def copy(self):\n        return dict(self._m)\n\n\ndef new_class(name, bases=(), kwds=None, exec_body=None):\n    ns = {}\n    if exec_body is not None:\n        exec_body(ns)\n    return type(name, bases, ns)\n\n\nclass GenericAlias:\n    def __init__(self, origin, args):\n        if not isinstance(args, tuple):\n            args = (args,)\n        self.__origin__ = origin\n        self.__args__ = args\n        self.__parameters__ = ()\n\n    def __call__(self, *args, **kwargs):\n        return self.__origin__(*args, **kwargs)\n\n    def __getitem__(self, params):\n        return GenericAlias(self.__origin__, params)\n\n    def __or__(self, other):\n        return UnionType((self, other))\n\n    def __ror__(self, other):\n        return UnionType((other, self))\n\n    def __eq__(self, other):\n        return isinstance(other, GenericAlias) and self.__origin__ == other.__origin__ and self.__args__ == other.__args__\n\n    def __hash__(self):\n        return hash((self.__origin__, self.__args__))\n\n    def __mro_entries__(self, bases):\n        return (self.__origin__,)\n\n    def __repr__(self):\n        return _repr(self.__origin__) + \"[\" + \", \".join(_repr(a) for a in self.__args__) + \"]\"\n\n\nclass UnionType:\n    def __init__(self, args):\n        self.__args__ = args\n\n    def __or__(self, other):\n        return UnionType(self.__args__ + (other,))\n\n    def __repr__(self):\n        return \" | \".join(_repr(a) for a in self.__args__)\n\n\ndef _repr(obj):\n    if obj is ...:\n        return \"...\"\n    if obj is None or obj is type(None):\n        return \"None\"\n    if isinstance(obj, type):\n        if obj.__module__ == \"builtins\":\n            return obj.__qualname__\n        return obj.__module__ + \".\" + obj.__qualname__\n    if isinstance(obj, tuple) and not obj:\n        return \"()\"\n    return repr(obj)\n";
    return s37;
  }
  if ( name == "typing" ) {
    const s38 = "# SPDX-License-Identifier: MIT\n# typing for the Ranger Python runtime: the names exist and subscripting\n# them works, so annotated code runs; nothing is checked.\n\nTYPE_CHECKING = False\n\n\nclass _Alias:\n    def __init__(self, name, args=(), origin=None):\n        self._name = name\n        self.__args__ = args\n        self.__origin__ = origin\n        self.__name__ = name\n\n    def __getitem__(self, params):\n        if not isinstance(params, tuple):\n            params = (params,)\n        return _Alias(self._name, params, self)\n\n    def __call__(self, *args, **kwargs):\n        if self.__origin__ is not None:\n            return self.__origin__(*args, **kwargs)\n        raise TypeError(\"Cannot instantiate typing.\" + self._name)\n\n    def __or__(self, other):\n        return Union[self, other]\n\n    def __ror__(self, other):\n        return Union[other, self]\n\n    def __repr__(self):\n        if self.__args__:\n            return \"typing.%s[%s]\" % (self._name, \", \".join(_type_repr(a) for a in self.__args__))\n        return \"typing.\" + self._name\n\n    def __eq__(self, other):\n        return isinstance(other, _Alias) and self._name == other._name and self.__args__ == other.__args__\n\n    def __hash__(self):\n        return hash((self._name, self.__args__))\n\n\ndef _type_repr(obj):\n    if isinstance(obj, type):\n        if obj.__module__ == \"builtins\":\n            return obj.__qualname__\n        return obj.__module__ + \".\" + obj.__qualname__\n    if obj is ...:\n        return \"...\"\n    return repr(obj)\n\n\nclass _Builtin(_Alias):\n    def __init__(self, name, origin):\n        _Alias.__init__(self, name)\n        self._real = origin\n\n    def __call__(self, *args, **kwargs):\n        return self._real(*args, **kwargs)\n\n    def __getitem__(self, params):\n        if not isinstance(params, tuple):\n            params = (params,)\n        a = _Alias(self._name, params, self._real)\n        return a\n\n\nAny = _Alias(\"Any\")\nNoReturn = _Alias(\"NoReturn\")\nNever = _Alias(\"Never\")\nSelf = _Alias(\"Self\")\nLiteralString = _Alias(\"LiteralString\")\nUnion = _Alias(\"Union\")\nOptional = _Alias(\"Optional\")\nLiteral = _Alias(\"Literal\")\nClassVar = _Alias(\"ClassVar\")\nFinal = _Alias(\"Final\")\nAnnotated = _Alias(\"Annotated\")\nConcatenate = _Alias(\"Concatenate\")\nTypeGuard = _Alias(\"TypeGuard\")\nTypeIs = _Alias(\"TypeIs\")\nUnpack = _Alias(\"Unpack\")\nRequired = _Alias(\"Required\")\nNotRequired = _Alias(\"NotRequired\")\nReadOnly = _Alias(\"ReadOnly\")\nList = _Builtin(\"List\", list)\nDict = _Builtin(\"Dict\", dict)\nSet = _Builtin(\"Set\", set)\nFrozenSet = _Builtin(\"FrozenSet\", frozenset)\nTuple = _Builtin(\"Tuple\", tuple)\nType = _Builtin(\"Type\", type)\nCallable = _Alias(\"Callable\")\nIterable = _Alias(\"Iterable\")\nIterator = _Alias(\"Iterator\")\nGenerator = _Alias(\"Generator\")\nAsyncIterator = _Alias(\"AsyncIterator\")\nAsyncGenerator = _Alias(\"AsyncGenerator\")\nAwaitable = _Alias(\"Awaitable\")\nCoroutine = _Alias(\"Coroutine\")\nSequence = _Alias(\"Sequence\")\nMutableSequence = _Alias(\"MutableSequence\")\nMapping = _Alias(\"Mapping\")\nMutableMapping = _Alias(\"MutableMapping\")\nAbstractSet = _Alias(\"AbstractSet\")\nMutableSet = _Alias(\"MutableSet\")\nCollection = _Alias(\"Collection\")\nContainer = _Alias(\"Container\")\nHashable = _Alias(\"Hashable\")\nSized = _Alias(\"Sized\")\nReversible = _Alias(\"Reversible\")\nKeysView = _Alias(\"KeysView\")\nValuesView = _Alias(\"ValuesView\")\nItemsView = _Alias(\"ItemsView\")\nIO = _Alias(\"IO\")\nTextIO = _Alias(\"TextIO\")\nBinaryIO = _Alias(\"BinaryIO\")\nPattern = _Alias(\"Pattern\")\nMatch = _Alias(\"Match\")\nSupportsInt = _Alias(\"SupportsInt\")\nSupportsFloat = _Alias(\"SupportsFloat\")\nSupportsIndex = _Alias(\"SupportsIndex\")\nSupportsAbs = _Alias(\"SupportsAbs\")\nSupportsRound = _Alias(\"SupportsRound\")\nSupportsComplex = _Alias(\"SupportsComplex\")\nSupportsBytes = _Alias(\"SupportsBytes\")\nAnyStr = _Alias(\"AnyStr\")\nText = str\n\n\ndef _deque(*args):\n    import collections\n    return collections.deque(*args)\n\n\ndef _defaultdict(*args):\n    import collections\n    return collections.defaultdict(*args)\n\n\ndef _counter(*args):\n    import collections\n    return collections.Counter(*args)\n\n\ndef _ordereddict(*args):\n    import collections\n    return collections.OrderedDict(*args)\n\n\nDeque = _Builtin(\"Deque\", _deque)\nDefaultDict = _Builtin(\"DefaultDict\", _defaultdict)\nCounter = _Builtin(\"Counter\", _counter)\nOrderedDict = _Builtin(\"OrderedDict\", _ordereddict)\nChainMap = _Alias(\"ChainMap\")\n\n\nclass TypeVar:\n    def __init__(self, name, *constraints, bound=None, covariant=False, contravariant=False, infer_variance=False, default=None):\n        self.__name__ = name\n        self.__constraints__ = constraints\n        self.__bound__ = bound\n        self.__covariant__ = covariant\n        self.__contravariant__ = contravariant\n        self.__default__ = default\n        self.__infer_variance__ = infer_variance\n\n    def __or__(self, other):\n        return Union[self, other]\n\n    def __ror__(self, other):\n        return Union[other, self]\n\n    def __repr__(self):\n        if self.__infer_variance__:\n            return self.__name__\n        if self.__covariant__:\n            return \"+\" + self.__name__\n        if self.__contravariant__:\n            return \"-\" + self.__name__\n        return \"~\" + self.__name__\n\n\nclass ParamSpec(TypeVar):\n    def __repr__(self):\n        return \"~\" + self.__name__\n\n\nclass TypeVarTuple(TypeVar):\n    def __repr__(self):\n        return self.__name__\n\n\nclass TypeAliasType:\n    def __init__(self, name, value, *, type_params=()):\n        self.__name__ = name\n        self.__value__ = value\n        self.__type_params__ = type_params\n\n    def __getitem__(self, params):\n        return self\n\n    def __repr__(self):\n        return self.__name__\n\n\nclass Generic:\n    def __class_getitem__(cls, params):\n        return cls\n\n\nclass Protocol(Generic):\n    pass\n\n\ndef runtime_checkable(cls):\n    return cls\n\n\ndef cast(typ, val):\n    return val\n\n\ndef assert_type(val, typ):\n    return val\n\n\ndef reveal_type(obj):\n    return obj\n\n\ndef assert_never(arg):\n    raise AssertionError(\"Expected code to be unreachable\")\n\n\ndef overload(func):\n    return func\n\n\ndef final(f):\n    return f\n\n\ndef override(method):\n    return method\n\n\ndef no_type_check(arg):\n    return arg\n\n\ndef get_overloads(func):\n    return []\n\n\ndef get_type_hints(obj, globalns=None, localns=None, include_extras=False):\n    return dict(getattr(obj, \"__annotations__\", {}) or {})\n\n\ndef get_origin(tp):\n    return getattr(tp, \"__origin__\", None)\n\n\ndef get_args(tp):\n    return getattr(tp, \"__args__\", ())\n\n\ndef NewType(name, tp):\n    def new_type(x):\n        return x\n    new_type.__name__ = name\n    new_type.__supertype__ = tp\n    return new_type\n\n\n_NT_SKIP = (\"__module__\", \"__qualname__\", \"__annotations__\", \"__doc__\", \"__orig_bases__\", \"__classcell__\")\n\n\ndef NamedTuple(typename, fields=None, **kwargs):\n    import collections\n    if fields is None:\n        fields = list(kwargs.items())\n    return collections.namedtuple(typename, [f[0] for f in fields])\n\n\nclass _NamedTupleMeta(type):\n    def __new__(mcls, typename, bases, ns):\n        import collections\n        if ns.get(\"_root\"):\n            return super().__new__(mcls, typename, bases, ns)\n        ann = ns.get(\"__annotations__\", {})\n        names = list(ann)\n        defaults = [ns[n] for n in names if n in ns]\n        nt = collections.namedtuple(typename, names, defaults=defaults)\n        for k, v in ns.items():\n            if k not in names and k not in _NT_SKIP:\n                setattr(nt, k, v)\n        return nt\n\n\nNamedTuple.__mro_entries__ = lambda bases: (_NTBase,)\n\n\nclass _NTBase(metaclass=_NamedTupleMeta):\n    _root = True\n\n\nclass TypedDict(dict):\n    pass\n";
    return s38;
  }
  if ( name == "unittest" ) {
    const s39 = "# SPDX-License-Identifier: MIT\n#\n# unittest for the Ranger Python runtime: TestCase with the assert methods,\n# skips, expected failures, subTest, setUp/tearDown/cleanups, TestResult,\n# TestSuite, TestLoader and main(). Written against CPython's documented\n# behaviour, not copied from it.\n\nimport sys\n\n\nclass SkipTest(Exception):\n    pass\n\n\ndef _id(obj):\n    return obj\n\n\ndef skip(reason):\n    def decorator(obj):\n        obj.__unittest_skip__ = True\n        obj.__unittest_skip_why__ = reason\n        return obj\n    if callable(reason) and not isinstance(reason, str):\n        fn = reason\n        reason = \"\"\n        return decorator(fn)\n    return decorator\n\n\ndef skipIf(condition, reason):\n    if condition:\n        return skip(reason)\n    return _id\n\n\ndef skipUnless(condition, reason):\n    if not condition:\n        return skip(reason)\n    return _id\n\n\ndef expectedFailure(fn):\n    fn.__unittest_expecting_failure__ = True\n    return fn\n\n\ndef _safe_repr(obj):\n    try:\n        return repr(obj)\n    except Exception:\n        return object.__repr__(obj)\n\n\ndef _matches(pattern, text):\n    # a regular expression is matched as text with its escapes removed\n    # (there is no re module yet); a pattern with other syntax matches\n    if pattern is None:\n        return True\n    if not isinstance(pattern, str):\n        pattern = getattr(pattern, \"pattern\", str(pattern))\n    special = \"^$*+?{}[]|()\"\n    plain = \"\"\n    i = 0\n    while i < len(pattern):\n        ch = pattern[i]\n        if ch == \"\\\\\" and i + 1 < len(pattern):\n            plain += pattern[i + 1]\n            i += 2\n            continue\n        if ch in special:\n            return True\n        if ch == \".\":\n            return True\n        plain += ch\n        i += 1\n    return plain in text\n\n\nclass _AssertRaisesContext:\n    def __init__(self, expected, test_case, regex=None, msg=None):\n        self.expected = expected\n        self.test_case = test_case\n        self.regex = regex\n        self.msg = msg\n        self.exception = None\n\n    def __enter__(self):\n        return self\n\n    def __exit__(self, exc_type, exc_value, tb):\n        if exc_type is None:\n            try:\n                name = self.expected.__name__\n            except AttributeError:\n                name = str(self.expected)\n            raise self.test_case.failureException(self.msg or (name + \" not raised\"))\n        if not issubclass(exc_type, self.expected):\n            return False\n        self.exception = exc_value\n        if self.regex is not None and not _matches(self.regex, str(exc_value)):\n            raise self.test_case.failureException('\"%s\" does not match \"%s\"' % (self.regex, str(exc_value)))\n        return True\n\n\nclass _AssertWarnsContext:\n    def __init__(self, expected, test_case):\n        self.expected = expected\n        self.test_case = test_case\n        self.warning = None\n        self.warnings = []\n\n    def __enter__(self):\n        return self\n\n    def __exit__(self, exc_type, exc_value, tb):\n        return False\n\n\nclass _SubTest:\n    def __init__(self, case, msg, params):\n        self.case = case\n        self.msg = msg\n        self.params = params\n\n    def __enter__(self):\n        return self\n\n    def __exit__(self, exc_type, exc_value, tb):\n        if exc_type is None:\n            return False\n        if issubclass(exc_type, SkipTest):\n            return True\n        if issubclass(exc_type, (KeyboardInterrupt, SystemExit)):\n            return False\n        self.case._subtest_failures.append((exc_type, exc_value))\n        return True\n\n\nclass TestCase:\n    failureException = AssertionError\n    longMessage = True\n    maxDiff = 640\n\n    def __init__(self, methodName=\"runTest\"):\n        self._testMethodName = methodName\n        self._cleanups = []\n        self._subtest_failures = []\n\n    def __repr__(self):\n        return \"<%s testMethod=%s>\" % (type(self).__name__, self._testMethodName)\n\n    def __str__(self):\n        return \"%s (%s)\" % (self._testMethodName, type(self).__name__)\n\n    def id(self):\n        return type(self).__name__ + \".\" + self._testMethodName\n\n    def shortDescription(self):\n        return None\n\n    def setUp(self):\n        pass\n\n    def tearDown(self):\n        pass\n\n    @classmethod\n    def setUpClass(cls):\n        pass\n\n    @classmethod\n    def tearDownClass(cls):\n        pass\n\n    def addCleanup(self, function, *args, **kwargs):\n        self._cleanups.append((function, args, kwargs))\n\n    def enterContext(self, cm):\n        result = cm.__enter__()\n        self.addCleanup(cm.__exit__, None, None, None)\n        return result\n\n    def doCleanups(self):\n        while self._cleanups:\n            function, args, kwargs = self._cleanups.pop()\n            function(*args, **kwargs)\n\n    def skipTest(self, reason):\n        raise SkipTest(reason)\n\n    def subTest(self, msg=None, **params):\n        return _SubTest(self, msg, params)\n\n    def run(self, result=None):\n        if result is None:\n            result = TestResult()\n        result.startTest(self)\n        method = getattr(self, self._testMethodName)\n        cls_skip = getattr(type(self), \"__unittest_skip__\", False)\n        if cls_skip or getattr(method, \"__unittest_skip__\", False):\n            why = getattr(method, \"__unittest_skip_why__\", \"\") or getattr(type(self), \"__unittest_skip_why__\", \"\")\n            result.addSkip(self, why)\n            result.stopTest(self)\n            return result\n        expecting = getattr(method, \"__unittest_expecting_failure__\", False)\n        outcome = None\n        try:\n            self.setUp()\n        except SkipTest as e:\n            outcome = (\"skip\", str(e))\n        except Exception as e:\n            outcome = (\"error\", e)\n        if outcome is None:\n            try:\n                method()\n            except SkipTest as e:\n                outcome = (\"skip\", str(e))\n            except AssertionError as e:\n                outcome = (\"fail\", e)\n            except Exception as e:\n                outcome = (\"error\", e)\n            if outcome is None and self._subtest_failures:\n                et, ev = self._subtest_failures[0]\n                if issubclass(et, AssertionError):\n                    outcome = (\"fail\", ev)\n                else:\n                    outcome = (\"error\", ev)\n            try:\n                self.tearDown()\n            except Exception as e:\n                if outcome is None:\n                    outcome = (\"error\", e)\n        try:\n            self.doCleanups()\n        except Exception as e:\n            if outcome is None:\n                outcome = (\"error\", e)\n        if outcome is None:\n            if expecting:\n                result.addUnexpectedSuccess(self)\n            else:\n                result.addSuccess(self)\n        elif outcome[0] == \"skip\":\n            result.addSkip(self, outcome[1])\n        elif expecting:\n            result.addExpectedFailure(self, outcome[1])\n        elif outcome[0] == \"fail\":\n            result.addFailure(self, outcome[1])\n        else:\n            result.addError(self, outcome[1])\n        result.stopTest(self)\n        return result\n\n    def __call__(self, *args, **kwds):\n        return self.run(*args, **kwds)\n\n    def debug(self):\n        self.setUp()\n        getattr(self, self._testMethodName)()\n        self.tearDown()\n        self.doCleanups()\n\n    # ---- assertions\n\n    def _formatMessage(self, msg, standardMsg):\n        if msg is None:\n            return standardMsg\n        if not self.longMessage:\n            return msg\n        return \"%s : %s\" % (standardMsg, msg)\n\n    def fail(self, msg=None):\n        raise self.failureException(msg)\n\n    def assertTrue(self, expr, msg=None):\n        if not expr:\n            self.fail(self._formatMessage(msg, \"%s is not true\" % _safe_repr(expr)))\n\n    def assertFalse(self, expr, msg=None):\n        if expr:\n            self.fail(self._formatMessage(msg, \"%s is not false\" % _safe_repr(expr)))\n\n    def assertEqual(self, first, second, msg=None):\n        if not first == second:\n            self.fail(self._formatMessage(msg, \"%s != %s\" % (_safe_repr(first), _safe_repr(second))))\n\n    def assertNotEqual(self, first, second, msg=None):\n        if not first != second:\n            self.fail(self._formatMessage(msg, \"%s == %s\" % (_safe_repr(first), _safe_repr(second))))\n\n    assertEquals = assertEqual\n    assertNotEquals = assertNotEqual\n    failUnless = assertTrue\n    failIf = assertFalse\n\n    def assertListEqual(self, first, second, msg=None):\n        self.assertIsInstance(first, list, msg)\n        self.assertIsInstance(second, list, msg)\n        self.assertEqual(first, second, msg)\n\n    def assertTupleEqual(self, first, second, msg=None):\n        self.assertIsInstance(first, tuple, msg)\n        self.assertIsInstance(second, tuple, msg)\n        self.assertEqual(first, second, msg)\n\n    def assertDictEqual(self, first, second, msg=None):\n        self.assertIsInstance(first, dict, msg)\n        self.assertIsInstance(second, dict, msg)\n        self.assertEqual(first, second, msg)\n\n    def assertSetEqual(self, first, second, msg=None):\n        self.assertEqual(set(first), set(second), msg)\n\n    def assertSequenceEqual(self, first, second, msg=None, seq_type=None):\n        if seq_type is not None:\n            self.assertIsInstance(first, seq_type, msg)\n            self.assertIsInstance(second, seq_type, msg)\n        self.assertEqual(list(first), list(second), msg)\n\n    def assertMultiLineEqual(self, first, second, msg=None):\n        self.assertEqual(first, second, msg)\n\n    def assertCountEqual(self, first, second, msg=None):\n        a = list(first)\n        b = list(second)\n        if len(a) != len(b):\n            self.fail(self._formatMessage(msg, \"Element counts were not equal\"))\n        rest = list(b)\n        for x in a:\n            found = False\n            for i in range(len(rest)):\n                if rest[i] == x:\n                    del rest[i]\n                    found = True\n                    break\n            if not found:\n                self.fail(self._formatMessage(msg, \"Element counts were not equal\"))\n\n    def assertIs(self, first, second, msg=None):\n        if first is not second:\n            self.fail(self._formatMessage(msg, \"%s is not %s\" % (_safe_repr(first), _safe_repr(second))))\n\n    def assertIsNot(self, first, second, msg=None):\n        if first is second:\n            self.fail(self._formatMessage(msg, \"unexpectedly identical: %s\" % _safe_repr(first)))\n\n    def assertIsNone(self, obj, msg=None):\n        if obj is not None:\n            self.fail(self._formatMessage(msg, \"%s is not None\" % _safe_repr(obj)))\n\n    def assertIsNotNone(self, obj, msg=None):\n        if obj is None:\n            self.fail(self._formatMessage(msg, \"unexpectedly None\"))\n\n    def assertIn(self, member, container, msg=None):\n        if member not in container:\n            self.fail(self._formatMessage(msg, \"%s not found in %s\" % (_safe_repr(member), _safe_repr(container))))\n\n    def assertNotIn(self, member, container, msg=None):\n        if member in container:\n            self.fail(self._formatMessage(msg, \"%s unexpectedly found in %s\" % (_safe_repr(member), _safe_repr(container))))\n\n    def assertIsInstance(self, obj, cls, msg=None):\n        if not isinstance(obj, cls):\n            self.fail(self._formatMessage(msg, \"%s is not an instance of %r\" % (_safe_repr(obj), cls)))\n\n    def assertNotIsInstance(self, obj, cls, msg=None):\n        if isinstance(obj, cls):\n            self.fail(self._formatMessage(msg, \"%s is an instance of %r\" % (_safe_repr(obj), cls)))\n\n    def assertGreater(self, a, b, msg=None):\n        if not a > b:\n            self.fail(self._formatMessage(msg, \"%s not greater than %s\" % (_safe_repr(a), _safe_repr(b))))\n\n    def assertGreaterEqual(self, a, b, msg=None):\n        if not a >= b:\n            self.fail(self._formatMessage(msg, \"%s not greater than or equal to %s\" % (_safe_repr(a), _safe_repr(b))))\n\n    def assertLess(self, a, b, msg=None):\n        if not a < b:\n            self.fail(self._formatMessage(msg, \"%s not less than %s\" % (_safe_repr(a), _safe_repr(b))))\n\n    def assertLessEqual(self, a, b, msg=None):\n        if not a <= b:\n            self.fail(self._formatMessage(msg, \"%s not less than or equal to %s\" % (_safe_repr(a), _safe_repr(b))))\n\n    def assertAlmostEqual(self, first, second, places=None, msg=None, delta=None):\n        if first == second:\n            return\n        diff = abs(first - second)\n        if delta is not None:\n            if diff <= delta:\n                return\n            standard = \"%s != %s within %s delta (%s difference)\" % (_safe_repr(first), _safe_repr(second), _safe_repr(delta), _safe_repr(diff))\n        else:\n            if places is None:\n                places = 7\n            if round(diff, places) == 0:\n                return\n            standard = \"%s != %s within %r places (%s difference)\" % (_safe_repr(first), _safe_repr(second), places, _safe_repr(diff))\n        self.fail(self._formatMessage(msg, standard))\n\n    def assertNotAlmostEqual(self, first, second, places=None, msg=None, delta=None):\n        diff = abs(first - second)\n        if delta is not None:\n            if not (first == second) and diff > delta:\n                return\n        else:\n            if places is None:\n                places = 7\n            if not (first == second) and round(diff, places) != 0:\n                return\n        self.fail(self._formatMessage(msg, \"%s == %s\" % (_safe_repr(first), _safe_repr(second))))\n\n    def assertRaises(self, expected_exception, *args, **kwargs):\n        ctx = _AssertRaisesContext(expected_exception, self, None, kwargs.pop(\"msg\", None))\n        if not args:\n            return ctx\n        callable_obj = args[0]\n        with ctx:\n            callable_obj(*args[1:], **kwargs)\n        return ctx\n\n    def assertRaisesRegex(self, expected_exception, expected_regex, *args, **kwargs):\n        ctx = _AssertRaisesContext(expected_exception, self, expected_regex, kwargs.pop(\"msg\", None))\n        if not args:\n            return ctx\n        callable_obj = args[0]\n        with ctx:\n            callable_obj(*args[1:], **kwargs)\n        return ctx\n\n    assertRaisesRegexp = assertRaisesRegex\n\n    def assertWarns(self, expected_warning, *args, **kwargs):\n        ctx = _AssertWarnsContext(expected_warning, self)\n        if not args:\n            return ctx\n        with ctx:\n            args[0](*args[1:], **kwargs)\n        return ctx\n\n    def assertWarnsRegex(self, expected_warning, expected_regex, *args, **kwargs):\n        return self.assertWarns(expected_warning, *args, **kwargs)\n\n    def assertNoLogs(self, logger=None, level=None):\n        return _AssertWarnsContext(None, self)\n\n    def assertRegex(self, text, expected_regex, msg=None):\n        if not _matches(expected_regex, text):\n            self.fail(self._formatMessage(msg, \"Regex didn't match: %r not found in %r\" % (expected_regex, text)))\n\n    def assertNotRegex(self, text, unexpected_regex, msg=None):\n        pass\n\n    def assertHasAttr(self, obj, name, msg=None):\n        if not hasattr(obj, name):\n            self.fail(self._formatMessage(msg, \"%s has no attribute %r\" % (_safe_repr(obj), name)))\n\n    def assertNotHasAttr(self, obj, name, msg=None):\n        if hasattr(obj, name):\n            self.fail(self._formatMessage(msg, \"%s has unexpected attribute %r\" % (_safe_repr(obj), name)))\n\n\nclass FunctionTestCase(TestCase):\n    def __init__(self, testFunc, setUp=None, tearDown=None, description=None):\n        TestCase.__init__(self, \"runTest\")\n        self._testFunc = testFunc\n\n    def runTest(self):\n        self._testFunc()\n\n\nclass TestResult:\n    def __init__(self, stream=None, descriptions=None, verbosity=None):\n        self.failures = []\n        self.errors = []\n        self.skipped = []\n        self.expectedFailures = []\n        self.unexpectedSuccesses = []\n        self.successes = []\n        self.testsRun = 0\n        self.shouldStop = False\n\n    def startTest(self, test):\n        self.testsRun += 1\n\n    def stopTest(self, test):\n        pass\n\n    def addSuccess(self, test):\n        self.successes.append(test)\n\n    def addFailure(self, test, err):\n        self.failures.append((test, _describe(err)))\n\n    def addError(self, test, err):\n        self.errors.append((test, _describe(err)))\n\n    def addSkip(self, test, reason):\n        self.skipped.append((test, reason))\n\n    def addExpectedFailure(self, test, err):\n        self.expectedFailures.append((test, _describe(err)))\n\n    def addUnexpectedSuccess(self, test):\n        self.unexpectedSuccesses.append(test)\n\n    def wasSuccessful(self):\n        return not self.failures and not self.errors and not self.unexpectedSuccesses\n\n    def stop(self):\n        self.shouldStop = True\n\n\ndef _describe(err):\n    if isinstance(err, BaseException):\n        return type(err).__name__ + \": \" + str(err)\n    return str(err)\n\n\nclass TestSuite:\n    def __init__(self, tests=()):\n        self._tests = []\n        self.addTests(tests)\n\n    def __iter__(self):\n        return iter(self._tests)\n\n    def countTestCases(self):\n        n = 0\n        for t in self._tests:\n            if isinstance(t, TestSuite):\n                n += t.countTestCases()\n            else:\n                n += 1\n        return n\n\n    def addTest(self, test):\n        self._tests.append(test)\n\n    def addTests(self, tests):\n        for t in tests:\n            self.addTest(t)\n\n    def run(self, result):\n        for t in self._tests:\n            if result.shouldStop:\n                break\n            t.run(result)\n        return result\n\n    def __call__(self, result):\n        return self.run(result)\n\n\ndef _test_names(cls, prefix=\"test\"):\n    names = []\n    seen = set()\n    for klass in cls.__mro__:\n        for name in list(klass.__dict__):\n            if name.startswith(prefix) and name not in seen:\n                seen.add(name)\n                if callable(getattr(cls, name, None)):\n                    names.append(name)\n    return sorted(names)\n\n\nclass TestLoader:\n    testMethodPrefix = \"test\"\n\n    def getTestCaseNames(self, testCaseClass):\n        return _test_names(testCaseClass, self.testMethodPrefix)\n\n    def loadTestsFromTestCase(self, testCaseClass):\n        return TestSuite([testCaseClass(n) for n in self.getTestCaseNames(testCaseClass)])\n\n    def loadTestsFromModule(self, module, pattern=None):\n        tests = []\n        for name in sorted(dir(module)):\n            obj = getattr(module, name)\n            if isinstance(obj, type) and issubclass(obj, TestCase) and obj is not TestCase:\n                tests.append(self.loadTestsFromTestCase(obj))\n        return TestSuite(tests)\n\n\ndefaultTestLoader = TestLoader()\n\n\nclass TextTestRunner:\n    def __init__(self, stream=None, descriptions=True, verbosity=1, **kwargs):\n        self.verbosity = verbosity\n\n    def run(self, test):\n        result = TestResult()\n        test.run(result)\n        for t, err in result.failures:\n            print(\"FAIL:\", t, err)\n        for t, err in result.errors:\n            print(\"ERROR:\", t, err)\n        print(\"Ran %d tests: %d failures, %d errors, %d skipped\" % (result.testsRun, len(result.failures), len(result.errors), len(result.skipped)))\n        return result\n\n\ndef main(module=\"__main__\", exit=True, verbosity=1, **kwargs):\n    mod = sys.modules.get(module) if isinstance(module, str) else module\n    if mod is None:\n        names = globals()\n        raise RuntimeError(\"unittest.main(): module %r not found\" % (module,))\n    suite = defaultTestLoader.loadTestsFromModule(mod)\n    result = TextTestRunner(verbosity=verbosity).run(suite)\n    if exit:\n        sys.exit(0 if result.wasSuccessful() else 1)\n    return result\n";
    return s39;
  }
  if ( name == "unittest.mock" ) {
    const s40 = "# SPDX-License-Identifier: MIT\n# unittest.mock for the Ranger Python runtime: Mock, MagicMock, patch,\n# call, ANY, sentinel and DEFAULT, written against the documented API.\nimport functools\n\n__all__ = [\"Mock\", \"MagicMock\", \"NonCallableMock\", \"NonCallableMagicMock\", \"patch\", \"call\",\n           \"ANY\", \"DEFAULT\", \"sentinel\", \"create_autospec\", \"PropertyMock\", \"mock_open\", \"seal\"]\n\n\nclass _Sentinel:\n    def __init__(self, name):\n        self.name = name\n\n    def __repr__(self):\n        return \"sentinel.\" + self.name\n\n    def __reduce__(self):\n        return \"sentinel.\" + self.name\n\n\nclass _SentinelObject:\n    def __init__(self):\n        self._sentinels = {}\n\n    def __getattr__(self, name):\n        if name == \"__bases__\":\n            raise AttributeError\n        if name not in self._sentinels:\n            self._sentinels[name] = _Sentinel(name)\n        return self._sentinels[name]\n\n\nsentinel = _SentinelObject()\nDEFAULT = sentinel.DEFAULT\n_missing = sentinel.MISSING\n\n\nclass _ANY:\n    def __eq__(self, other):\n        return True\n\n    def __ne__(self, other):\n        return False\n\n    def __hash__(self):\n        return 0\n\n    def __repr__(self):\n        return \"<ANY>\"\n\n\nANY = _ANY()\n\n\nclass _Call(tuple):\n    # call(args, kwargs) with a name, compared loosely like CPython's\n    def __new__(cls, value=(), name=\"\", parent=None, two=False, from_kall=True):\n        if len(value) == 3:\n            name, args, kwargs = value\n        elif len(value) == 2:\n            args, kwargs = value\n        elif len(value) == 1:\n            args, kwargs = value[0], {}\n        else:\n            args, kwargs = (), {}\n        self = tuple.__new__(cls, (args, kwargs))\n        self._mock_name = name\n        self._mock_parent = parent\n        return self\n\n    def __init__(self, *a, **k):\n        pass\n\n    @property\n    def args(self):\n        return tuple.__getitem__(self, 0)\n\n    @property\n    def kwargs(self):\n        return tuple.__getitem__(self, 1)\n\n    def __eq__(self, other):\n        if other is ANY:\n            return True\n        try:\n            if isinstance(other, _Call):\n                if self._mock_name and other._mock_name and self._mock_name != other._mock_name:\n                    return False\n                return self.args == other.args and self.kwargs == other.kwargs\n            if isinstance(other, tuple):\n                if len(other) == 2:\n                    return (self.args, self.kwargs) == (tuple(other[0]), dict(other[1]))\n                if len(other) == 3:\n                    return (self.args, self.kwargs) == (tuple(other[1]), dict(other[2]))\n                if len(other) == 1:\n                    if isinstance(other[0], dict):\n                        return self.args == () and self.kwargs == other[0]\n                    return self.args == tuple(other[0]) and not self.kwargs\n                return False\n        except Exception:\n            return False\n        return NotImplemented\n\n    def __ne__(self, other):\n        r = self.__eq__(other)\n        return r if r is NotImplemented else not r\n\n    __hash__ = None\n\n    def __call__(self, *args, **kwargs):\n        return _Call((self._mock_name, args, kwargs))\n\n    def __getattr__(self, attr):\n        if attr.startswith(\"__\"):\n            raise AttributeError(attr)\n        name = attr if not self._mock_name else self._mock_name + \".\" + attr\n        return _Call((name, (), {}))\n\n    def __repr__(self):\n        name = self._mock_name or \"call\"\n        if not name.startswith(\"call\") and name != \"call\":\n            name = \"call.\" + name\n        parts = [repr(a) for a in self.args]\n        parts += [\"%s=%r\" % kv for kv in self.kwargs.items()]\n        return \"%s(%s)\" % (name, \", \".join(parts))\n\n    def call_list(self):\n        return _CallList([self])\n\n\ncall = _Call((\"\", (), {}))\n\n\nclass _CallList(list):\n    def __contains__(self, value):\n        if not isinstance(value, list):\n            return list.__contains__(self, value)\n        n = len(value)\n        for i in range(0, len(self) - n + 1):\n            if list(self[i:i + n]) == list(value):\n                return True\n        return False\n\n    def __repr__(self):\n        return \"[\" + \", \".join(repr(c) for c in self) + \"]\"\n\n\n_MAGICS = (\"__lt__ __gt__ __le__ __ge__ __int__ __contains__ __len__ __iter__ __exit__ \"\n           \"__complex__ __float__ __bool__ __index__ __hash__ __str__ __sizeof__ __enter__ \"\n           \"__getitem__ __setitem__ __delitem__ __add__ __sub__ __mul__ __truediv__ __floordiv__ \"\n           \"__mod__ __pow__ __and__ __or__ __xor__ __lshift__ __rshift__ __neg__ __pos__ \"\n           \"__abs__ __invert__ __round__ __fspath__ __aenter__ __aexit__ __aiter__ __anext__\").split()\n\n\nclass NonCallableMock:\n    def __init__(self, spec=None, wraps=None, name=None, spec_set=None, parent=None,\n                 _new_name=\"\", _new_parent=None, unsafe=False, return_value=DEFAULT,\n                 side_effect=None, **kwargs):\n        d = self.__dict__\n        d[\"_mock_children\"] = {}\n        d[\"_mock_name\"] = name\n        d[\"_mock_parent\"] = parent\n        d[\"_mock_new_name\"] = _new_name\n        d[\"_mock_wraps\"] = wraps\n        d[\"_mock_return_value\"] = return_value\n        d[\"_mock_side_effect\"] = side_effect\n        d[\"_mock_sealed\"] = False\n        d[\"_spec_class\"] = None\n        d[\"_spec_set\"] = spec_set is not None\n        d[\"_mock_methods\"] = None\n        d[\"method_calls\"] = _CallList()\n        d[\"mock_calls\"] = _CallList()\n        d[\"call_args_list\"] = _CallList()\n        d[\"call_count\"] = 0\n        d[\"call_args\"] = None\n        d[\"called\"] = False\n        s = spec if spec is not None else spec_set\n        if s is not None:\n            if isinstance(s, list) and all(isinstance(x, str) for x in s):\n                d[\"_mock_methods\"] = list(s)\n            else:\n                d[\"_spec_class\"] = s if isinstance(s, type) else type(s)\n                d[\"_mock_methods\"] = dir(s)\n        self.configure_mock(**kwargs)\n\n    def configure_mock(self, **kwargs):\n        for arg, val in sorted(kwargs.items(), key=lambda kv: kv[0].count(\".\")):\n            parts = arg.split(\".\")\n            obj = self\n            for p in parts[:-1]:\n                obj = getattr(obj, p)\n            setattr(obj, parts[-1], val)\n\n    @property\n    def __class__(self):\n        if self._spec_class is None:\n            return type(self)\n        return self._spec_class\n\n    def _get_child_mock(self, **kw):\n        klass = type(self)\n        if issubclass(klass, NonCallableMagicMock) and not issubclass(klass, MagicMock):\n            klass = MagicMock\n        elif issubclass(klass, NonCallableMock) and not issubclass(klass, Mock) and not issubclass(klass, MagicMock):\n            klass = Mock\n        return klass(**kw)\n\n    def _full_name(self):\n        names = []\n        m = self\n        while m is not None:\n            n = m.__dict__.get(\"_mock_new_name\") or m.__dict__.get(\"_mock_name\")\n            if m.__dict__.get(\"_mock_parent\") is None:\n                break\n            names.append(n or \"\")\n            m = m.__dict__.get(\"_mock_parent\")\n        return \".\".join(reversed(names))\n\n    def __getattr__(self, name):\n        if name in (\"_mock_methods\", \"_mock_children\", \"_spec_class\"):\n            raise AttributeError(name)\n        if name.startswith((\"assert\", \"assret\", \"asert\", \"aseert\", \"assrt\")):\n            raise AttributeError(name + \" is not a valid assertion. Use a spec for the mock if %r is meant to be an attribute.\" % name)\n        methods = self.__dict__.get(\"_mock_methods\")\n        if methods is not None and name not in methods:\n            raise AttributeError(\"Mock object has no attribute %r\" % name)\n        if name.startswith(\"__\") and name.endswith(\"__\"):\n            raise AttributeError(name)\n        children = self.__dict__[\"_mock_children\"]\n        if name not in children:\n            if self.__dict__.get(\"_mock_sealed\"):\n                raise AttributeError(\"mock.%s\" % name)\n            wraps = self.__dict__.get(\"_mock_wraps\")\n            kw = {}\n            if wraps is not None:\n                kw[\"wraps\"] = getattr(wraps, name)\n            children[name] = self._get_child_mock(parent=self, name=name, _new_name=name, _new_parent=self, **kw)\n        return children[name]\n\n    def __setattr__(self, name, value):\n        if name in (\"return_value\", \"side_effect\"):\n            self.__dict__[\"_mock_\" + name] = value\n            return\n        if self.__dict__.get(\"_spec_set\") and self.__dict__.get(\"_mock_methods\") is not None and \\\n                name not in self.__dict__[\"_mock_methods\"] and name not in self.__dict__:\n            raise AttributeError(\"Mock object has no attribute '%s'\" % name)\n        if isinstance(value, NonCallableMock) and value.__dict__.get(\"_mock_parent\") is None:\n            value.__dict__[\"_mock_parent\"] = self\n            value.__dict__[\"_mock_new_name\"] = name\n            self.__dict__[\"_mock_children\"][name] = value\n            return\n        if name in self.__dict__[\"_mock_children\"]:\n            del self.__dict__[\"_mock_children\"][name]\n        self.__dict__[name] = value\n\n    def __delattr__(self, name):\n        if name in self.__dict__[\"_mock_children\"]:\n            del self.__dict__[\"_mock_children\"][name]\n        elif name in self.__dict__:\n            del self.__dict__[name]\n        else:\n            raise AttributeError(name)\n\n    def _get_return_value(self):\n        rv = self.__dict__[\"_mock_return_value\"]\n        if rv is DEFAULT:\n            if self.__dict__.get(\"_mock_wraps\") is not None:\n                return DEFAULT\n            rv = self._get_child_mock(parent=self, _new_name=\"()\", _new_parent=self)\n            self.__dict__[\"_mock_return_value\"] = rv\n        return rv\n\n    return_value = property(_get_return_value, lambda self, v: self.__dict__.__setitem__(\"_mock_return_value\", v))\n\n    @property\n    def side_effect(self):\n        return self.__dict__[\"_mock_side_effect\"]\n\n    def reset_mock(self, return_value=False, side_effect=False):\n        d = self.__dict__\n        d[\"called\"] = False\n        d[\"call_args\"] = None\n        d[\"call_count\"] = 0\n        d[\"mock_calls\"] = _CallList()\n        d[\"call_args_list\"] = _CallList()\n        d[\"method_calls\"] = _CallList()\n        if return_value:\n            d[\"_mock_return_value\"] = DEFAULT\n        if side_effect:\n            d[\"_mock_side_effect\"] = None\n        for child in d[\"_mock_children\"].values():\n            if isinstance(child, NonCallableMock):\n                child.reset_mock()\n        rv = d[\"_mock_return_value\"]\n        if isinstance(rv, NonCallableMock) and rv is not self:\n            rv.reset_mock()\n\n    def attach_mock(self, mock, attribute):\n        mock.__dict__[\"_mock_parent\"] = None\n        setattr(self, attribute, mock)\n\n    def mock_add_spec(self, spec, spec_set=False):\n        self.__dict__[\"_mock_methods\"] = dir(spec)\n        self.__dict__[\"_spec_set\"] = spec_set\n\n    def _record(self, args, kwargs):\n        d = self.__dict__\n        d[\"called\"] = True\n        d[\"call_count\"] += 1\n        c = _Call((\"\", args, kwargs))\n        d[\"call_args\"] = c\n        d[\"call_args_list\"].append(c)\n        d[\"mock_calls\"].append(_Call((\"\", args, kwargs)))\n        # parents see the call as name(...), and method calls in method_calls\n        name = d.get(\"_mock_new_name\") or \"\"\n        parent = d.get(\"_mock_parent\")\n        while parent is not None:\n            pd = parent.__dict__\n            if name and \"()\" not in name:\n                pd[\"method_calls\"].append(_Call((name, args, kwargs)))\n            pd[\"mock_calls\"].append(_Call((name, args, kwargs)))\n            pn = pd.get(\"_mock_new_name\") or \"\"\n            if pd.get(\"_mock_parent\") is None:\n                break\n            if name.startswith(\"()\"):\n                name = pn + name\n            else:\n                name = pn + (\".\" + name if name else \"\")\n            parent = pd[\"_mock_parent\"]\n\n    def _call(self, args, kwargs):\n        self._record(args, kwargs)\n        effect = self.__dict__[\"_mock_side_effect\"]\n        if effect is not None:\n            if isinstance(effect, BaseException) or (isinstance(effect, type) and issubclass(effect, BaseException)):\n                raise effect\n            if not callable(effect):\n                it = self.__dict__.get(\"_mock_effect_iter\")\n                if it is None or self.__dict__.get(\"_mock_effect_src\") is not effect:\n                    it = iter(effect)\n                    self.__dict__[\"_mock_effect_iter\"] = it\n                    self.__dict__[\"_mock_effect_src\"] = effect\n                result = next(it)\n                if isinstance(result, BaseException) or (isinstance(result, type) and issubclass(result, BaseException)):\n                    raise result\n                return result\n            result = effect(*args, **kwargs)\n            if result is not DEFAULT:\n                return result\n        rv = self.__dict__[\"_mock_return_value\"]\n        if rv is not DEFAULT:\n            return rv\n        wraps = self.__dict__.get(\"_mock_wraps\")\n        if wraps is not None:\n            return wraps(*args, **kwargs)\n        return self.return_value\n\n    def _calls_repr(self):\n        return \"Calls: %r.\" % (self.mock_calls,) if self.mock_calls else \"\"\n\n    def _name(self):\n        return self.__dict__.get(\"_mock_name\") or \"mock\"\n\n    def assert_called(self):\n        if self.call_count == 0:\n            raise AssertionError(\"Expected '%s' to have been called.\" % self._name())\n\n    def assert_not_called(self):\n        if self.call_count != 0:\n            raise AssertionError(\"Expected '%s' to not have been called. Called %d times.%s\" % (\n                self._name(), self.call_count, self._calls_repr()))\n\n    def assert_called_once(self):\n        if self.call_count != 1:\n            raise AssertionError(\"Expected '%s' to have been called once. Called %d times.%s\" % (\n                self._name(), self.call_count, self._calls_repr()))\n\n    def assert_called_with(self, *args, **kwargs):\n        if self.call_args is None:\n            raise AssertionError(\"expected call not found.\\nExpected: %r\\n  Actual: not called.\" % (_Call((\"mock\", args, kwargs)),))\n        if self.call_args != _Call((\"\", args, kwargs)):\n            raise AssertionError(\"expected call not found.\\nExpected: %r\\n  Actual: %r\" % (\n                _Call((\"mock\", args, kwargs)), _Call((\"mock\", self.call_args.args, self.call_args.kwargs))))\n\n    def assert_called_once_with(self, *args, **kwargs):\n        if self.call_count != 1:\n            raise AssertionError(\"Expected '%s' to be called once. Called %d times.%s\" % (\n                self._name(), self.call_count, self._calls_repr()))\n        self.assert_called_with(*args, **kwargs)\n\n    def assert_any_call(self, *args, **kwargs):\n        if _Call((\"\", args, kwargs)) not in self.call_args_list:\n            raise AssertionError(\"%r call not found\" % (_Call((\"mock\", args, kwargs)),))\n\n    def assert_has_calls(self, calls, any_order=False):\n        calls = list(calls)\n        have = _CallList(self.mock_calls)\n        if not any_order:\n            if calls not in have:\n                raise AssertionError(\"Calls not found.\\nExpected: %r\\n  Actual: %r\" % (calls, self.mock_calls))\n            return\n        have = list(have)\n        for c in calls:\n            try:\n                have.remove(c)\n            except ValueError:\n                raise AssertionError(\"%r not all found in call list\" % (tuple(calls),))\n\n    def __repr__(self):\n        name = self._full_name()\n        n = \"\"\n        if self.__dict__.get(\"_mock_name\"):\n            n = \" name=%r\" % (self.__dict__[\"_mock_name\"] if not name else \"mock.\" + name)\n        elif name:\n            n = \" name=%r\" % (\"mock.\" + name)\n        spec = \"\"\n        if self._spec_class is not None:\n            spec = \" spec=%r\" % self._spec_class.__name__\n        return \"<%s%s%s id='%d'>\" % (type(self).__name__, n, spec, id(self))\n\n    def __dir__(self):\n        return sorted(set(dir(type(self)) + list(self.__dict__[\"_mock_children\"])))\n\n\nclass CallableMixin:\n    def __call__(self, *args, **kwargs):\n        return self._call(args, kwargs)\n\n\nclass Mock(CallableMixin, NonCallableMock):\n    pass\n\n\ndef _magic(name):\n    def default(self, *args, **kwargs):\n        m = self.__dict__[\"_mock_children\"].get(name)\n        if m is None:\n            m = MagicMock(parent=self, name=name, _new_name=name)\n            rv = _magic_default(self, name)\n            if rv is not DEFAULT:\n                m.return_value = rv\n            self.__dict__[\"_mock_children\"][name] = m\n        return m(*args, **kwargs)\n    default.__name__ = name\n    return default\n\n\ndef _magic_default(self, name):\n    defaults = {\"__lt__\": NotImplemented, \"__gt__\": NotImplemented, \"__le__\": NotImplemented,\n                \"__ge__\": NotImplemented, \"__int__\": 1, \"__contains__\": False, \"__len__\": 0,\n                \"__exit__\": False, \"__complex__\": 1j, \"__float__\": 1.0, \"__bool__\": True,\n                \"__index__\": 1, \"__hash__\": id(self), \"__str__\": object.__repr__(self) if False else \"<MagicMock id='%d'>\" % id(self),\n                \"__sizeof__\": 0, \"__fspath__\": \"mock\"}\n    if name == \"__iter__\":\n        return iter([])\n    if name == \"__enter__\":\n        return DEFAULT\n    return defaults.get(name, DEFAULT)\n\n\nclass MagicMixin:\n    pass\n\n\nclass _MagicProxy:\n    # mock.__len__ is a child mock, and len(mock) calls it\n    def __init__(self, name):\n        self.name = name\n\n    def __get__(self, obj, _type=None):\n        if obj is None:\n            return self\n        m = obj.__dict__[\"_mock_children\"].get(self.name)\n        if m is None:\n            m = MagicMock(parent=obj, name=self.name, _new_name=self.name)\n            rv = _magic_default(obj, self.name)\n            if rv is not DEFAULT:\n                m.return_value = rv\n            obj.__dict__[\"_mock_children\"][self.name] = m\n        return m\n\n\nfor _n in _MAGICS:\n    if _n != \"__hash__\":\n        setattr(MagicMixin, _n, _MagicProxy(_n))\n\n\ndef _magic_eq(self, other):\n    m = self.__dict__[\"_mock_children\"].get(\"__eq__\")\n    if m is not None:\n        return m(other)\n    return self is other\n\n\ndef _magic_ne(self, other):\n    m = self.__dict__[\"_mock_children\"].get(\"__ne__\")\n    if m is not None:\n        return m(other)\n    return self is not other\n\n\nMagicMixin.__eq__ = _magic_eq\nMagicMixin.__ne__ = _magic_ne\n\n\ndef _magic_hash(self):\n    m = self.__dict__[\"_mock_children\"].get(\"__hash__\")\n    if m is not None:\n        return m()\n    return id(self)\n\n\nMagicMixin.__hash__ = _magic_hash\n\n\nclass NonCallableMagicMock(MagicMixin, NonCallableMock):\n    def __setattr__(self, name, value):\n        if name.startswith(\"__\") and name.endswith(\"__\") and name in _MAGICS + [\"__eq__\", \"__ne__\"]:\n            if not isinstance(value, NonCallableMock):\n                v = value\n                value = MagicMock(side_effect=lambda *a, **k: v(self, *a, **k) if callable(v) else v)\n            self.__dict__[\"_mock_children\"][name] = value\n            return\n        NonCallableMock.__setattr__(self, name, value)\n\n    def __getattr__(self, name):\n        if name in _MAGICS or name in (\"__eq__\", \"__ne__\"):\n            m = self.__dict__[\"_mock_children\"].get(name)\n            if m is None:\n                _magic(name)(self) if False else None\n                m = MagicMock(parent=self, name=name, _new_name=name)\n                rv = _magic_default(self, name)\n                if rv is not DEFAULT:\n                    m.return_value = rv\n                self.__dict__[\"_mock_children\"][name] = m\n            return m\n        return NonCallableMock.__getattr__(self, name)\n\n\nclass MagicMock(MagicMixin, Mock):\n    __setattr__ = NonCallableMagicMock.__setattr__\n    __getattr__ = NonCallableMagicMock.__getattr__\n\n\nclass PropertyMock(Mock):\n    def __get__(self, obj, obj_type=None):\n        return self()\n\n    def __set__(self, obj, val):\n        self(val)\n\n\ndef create_autospec(spec, spec_set=False, instance=False, _parent=None, _name=None, **kwargs):\n    klass = MagicMock\n    if not callable(spec) or instance:\n        klass = NonCallableMagicMock if not callable(spec) else MagicMock\n    m = klass(spec=spec, **kwargs) if not spec_set else klass(spec_set=spec, **kwargs)\n    return m\n\n\ndef seal(mock):\n    mock.__dict__[\"_mock_sealed\"] = True\n    for child in mock.__dict__[\"_mock_children\"].values():\n        if isinstance(child, NonCallableMock):\n            seal(child)\n\n\ndef mock_open(mock=None, read_data=\"\"):\n    if mock is None:\n        mock = MagicMock(name=\"open\")\n    handle = MagicMock()\n    handle.__enter__.return_value = handle\n    lines = read_data.splitlines(keepends=True)\n    handle.read.return_value = read_data\n    handle.readline.side_effect = lines + [\"\"] * 100\n    handle.readlines.return_value = lines\n    handle.__iter__.return_value = iter(lines)\n    mock.return_value = handle\n    return mock\n\n\ndef _importer(target):\n    parts = target.split(\".\")\n    import_path = parts.pop(0)\n    thing = __import__(import_path)\n    for comp in parts:\n        import_path += \".\" + comp\n        try:\n            thing = getattr(thing, comp)\n        except AttributeError:\n            __import__(import_path)\n            thing = getattr(thing, comp)\n    return thing\n\n\nclass _patch:\n    def __init__(self, getter, attribute, new, spec, create, spec_set, autospec, new_callable, kwargs):\n        self.getter = getter\n        self.attribute = attribute\n        self.new = new\n        self.new_callable = new_callable\n        self.spec = spec\n        self.create = create\n        self.spec_set = spec_set\n        self.autospec = autospec\n        self.kwargs = kwargs\n        self.additional_patchers = []\n        self.attribute_name = None\n\n    def copy(self):\n        return _patch(self.getter, self.attribute, self.new, self.spec, self.create,\n                      self.spec_set, self.autospec, self.new_callable, self.kwargs)\n\n    def __call__(self, func):\n        if isinstance(func, type):\n            for attr in dir(func):\n                if attr.startswith(\"test\"):\n                    value = getattr(func, attr)\n                    if callable(value):\n                        setattr(func, attr, self.copy()(value))\n            return func\n\n        @functools.wraps(func)\n        def patched(*args, **kw):\n            extra = []\n            entered = []\n            try:\n                for p in [self] + self.additional_patchers:\n                    v = p.__enter__()\n                    entered.append(p)\n                    if p.attribute_name is not None:\n                        kw[p.attribute_name] = v\n                    elif p.new is DEFAULT:\n                        extra.append(v)\n                return func(*(args + tuple(extra)), **kw)\n            finally:\n                for p in reversed(entered):\n                    p.__exit__(None, None, None)\n        return patched\n\n    def __enter__(self):\n        self.target = self.getter()\n        original = getattr(self.target, self.attribute, _missing)\n        if original is _missing and not self.create:\n            raise AttributeError(\"%r does not have the attribute %r\" % (self.target, self.attribute))\n        d = getattr(self.target, \"__dict__\", None)\n        self.is_local = isinstance(d, dict) and self.attribute in d or not isinstance(self.target, type)\n        self.temp_original = original\n        new = self.new\n        if new is DEFAULT:\n            klass = self.new_callable or MagicMock\n            kw = dict(self.kwargs)\n            if self.spec is not None and self.spec is not True:\n                kw[\"spec\"] = self.spec\n            elif self.spec is True:\n                kw[\"spec\"] = original\n            if self.spec_set is not None and self.spec_set is not False:\n                kw[\"spec_set\"] = original if self.spec_set is True else self.spec_set\n            if self.autospec is not None and self.autospec is not False:\n                new = create_autospec(original if self.autospec is True else self.autospec, **kw)\n            else:\n                new = klass(**kw)\n        self.new_value = new\n        setattr(self.target, self.attribute, new)\n        return new\n\n    def __exit__(self, *exc):\n        if self.temp_original is _missing:\n            try:\n                delattr(self.target, self.attribute)\n            except AttributeError:\n                pass\n        else:\n            setattr(self.target, self.attribute, self.temp_original)\n        return False\n\n    def start(self):\n        r = self.__enter__()\n        _active.append(self)\n        return r\n\n    def stop(self):\n        if self in _active:\n            _active.remove(self)\n        return self.__exit__(None, None, None)\n\n\n_active = []\n\n\ndef _get_target(target):\n    try:\n        target, attribute = target.rsplit(\".\", 1)\n    except (TypeError, ValueError, AttributeError):\n        raise TypeError(\"Need a valid target to patch. You supplied: %r\" % (target,))\n    return (lambda: _importer(target)), attribute\n\n\ndef patch(target, new=DEFAULT, spec=None, create=False, spec_set=None, autospec=None, new_callable=None, **kwargs):\n    getter, attribute = _get_target(target)\n    return _patch(getter, attribute, new, spec, create, spec_set, autospec, new_callable, kwargs)\n\n\ndef _patch_object(target, attribute, new=DEFAULT, spec=None, create=False, spec_set=None,\n                  autospec=None, new_callable=None, **kwargs):\n    return _patch(lambda: target, attribute, new, spec, create, spec_set, autospec, new_callable, kwargs)\n\n\ndef _patch_multiple(target, spec=None, create=False, spec_set=None, autospec=None, new_callable=None, **kwargs):\n    if isinstance(target, str):\n        getter = lambda: _importer(target)\n    else:\n        getter = lambda: target\n    items = list(kwargs.items())\n    attribute, new = items[0]\n    p = _patch(getter, attribute, new, spec, create, spec_set, autospec, new_callable, {})\n    p.attribute_name = attribute if new is DEFAULT else None\n    for attribute, new in items[1:]:\n        q = _patch(getter, attribute, new, spec, create, spec_set, autospec, new_callable, {})\n        q.attribute_name = attribute if new is DEFAULT else None\n        p.additional_patchers.append(q)\n    return p\n\n\nclass _patch_dict:\n    def __init__(self, in_dict, values=(), clear=False, **kwargs):\n        self.in_dict = in_dict\n        self.values = dict(values)\n        self.values.update(kwargs)\n        self.clear = clear\n        self._original = None\n\n    def __call__(self, f):\n        @functools.wraps(f)\n        def inner(*args, **kw):\n            self.__enter__()\n            try:\n                return f(*args, **kw)\n            finally:\n                self.__exit__(None, None, None)\n        return inner\n\n    def __enter__(self):\n        if isinstance(self.in_dict, str):\n            self.in_dict = _importer(self.in_dict)\n        self._original = dict(self.in_dict)\n        if self.clear:\n            self.in_dict.clear()\n        self.in_dict.update(self.values)\n        return self.in_dict\n\n    def __exit__(self, *args):\n        self.in_dict.clear()\n        self.in_dict.update(self._original)\n        return False\n\n    start = __enter__\n\n    def stop(self):\n        return self.__exit__(None, None, None)\n\n\ndef _stopall():\n    for p in reversed(list(_active)):\n        p.stop()\n\n\npatch.object = _patch_object\npatch.dict = _patch_dict\npatch.multiple = _patch_multiple\npatch.stopall = _stopall\npatch.TEST_PREFIX = \"test\"\n\nFILTER_DIR = True\n\n\nclass AsyncMock(MagicMock):\n    pass\n";
    return s40;
  }
  if ( name == "warnings" ) {
    const s41 = "# SPDX-License-Identifier: MIT\n# warnings for the Ranger Python runtime: warnings are collected, not printed,\n# unless an \"error\" filter turns them into exceptions.\n\n_filters = []\n_log = None\n\n\nclass WarningMessage:\n    def __init__(self, message, category, filename=\"\", lineno=0):\n        self.message = message\n        self.category = category\n        self.filename = filename\n        self.lineno = lineno\n\n\ndef warn(message, category=None, stacklevel=1, source=None, skip_file_prefixes=()):\n    if isinstance(message, Warning):\n        category = type(message)\n    if category is None:\n        category = UserWarning\n    for action, cat in _filters:\n        if issubclass(category, cat):\n            if action == \"error\":\n                raise message if isinstance(message, Warning) else category(message)\n            if action == \"ignore\":\n                return\n            break\n    if _log is not None:\n        msg = message if isinstance(message, Warning) else category(message)\n        _log.append(WarningMessage(msg, category))\n\n\ndef warn_explicit(message, category, filename, lineno, *args, **kwargs):\n    warn(message, category)\n\n\ndef simplefilter(action, category=Warning, lineno=0, append=False):\n    entry = (action, category)\n    if append:\n        _filters.append(entry)\n    else:\n        _filters.insert(0, entry)\n\n\ndef filterwarnings(action, message=\"\", category=Warning, module=\"\", lineno=0, append=False):\n    simplefilter(action, category, lineno, append)\n\n\ndef resetwarnings():\n    del _filters[:]\n\n\nclass catch_warnings:\n    def __init__(self, *, record=False, module=None, action=None, category=Warning, lineno=0, append=False):\n        self._record = record\n        self._action = action\n        self._category = category\n\n    def __enter__(self):\n        global _log\n        self._saved = (list(_filters), _log)\n        if self._action is not None:\n            simplefilter(self._action, self._category)\n        if self._record:\n            _log = []\n            return _log\n        return None\n\n    def __exit__(self, *exc):\n        global _log\n        _filters[:] = self._saved[0]\n        _log = self._saved[1]\n        return False\n";
    return s41;
  }
  if ( name == "weakref" ) {
    const s42 = "# SPDX-License-Identifier: MIT\n# weakref for the Ranger Python runtime. Values here live as long as\n# anything holds them, so a weak reference is a strong one that never dies.\n\n\nclass ref:\n    __slots__ = (\"_obj\", \"_callback\")\n\n    def __init__(self, obj, callback=None):\n        self._obj = obj\n        self._callback = callback\n\n    def __call__(self):\n        return self._obj\n\n    def __eq__(self, other):\n        if isinstance(other, ref):\n            return self._obj is other._obj or self._obj == other._obj\n        return NotImplemented\n\n    def __hash__(self):\n        return hash(self._obj)\n\n    def __repr__(self):\n        return \"<weakref; to '%s'>\" % type(self._obj).__name__\n\n    __callback__ = property(lambda self: self._callback)\n\n\nReferenceType = ref\n\n\ndef proxy(obj, callback=None):\n    return obj\n\n\nProxyType = type(None)\nCallableProxyType = type(None)\nProxyTypes = (ProxyType, CallableProxyType)\n\n\ndef getweakrefcount(obj):\n    return 0\n\n\ndef getweakrefs(obj):\n    return []\n\n\nclass WeakMethod(ref):\n    __slots__ = ()\n\n\nclass WeakValueDictionary(dict):\n    def valuerefs(self):\n        return [ref(v) for v in self.values()]\n\n    def itervaluerefs(self):\n        return iter(self.valuerefs())\n\n\nclass WeakKeyDictionary(dict):\n    def keyrefs(self):\n        return [ref(k) for k in self.keys()]\n\n\nclass WeakSet(set):\n    pass\n\n\nclass finalize:\n    def __init__(self, obj, func, /, *args, **kwargs):\n        self._func = func\n        self._args = args\n        self._kwargs = kwargs\n        self.atexit = True\n        self._alive = True\n\n    def __call__(self, _=None):\n        if self._alive:\n            self._alive = False\n            return self._func(*self._args, **self._kwargs)\n\n    def detach(self):\n        if self._alive:\n            self._alive = False\n            return (None, self._func, self._args, self._kwargs)\n\n    def peek(self):\n        if self._alive:\n            return (None, self._func, self._args, self._kwargs)\n\n    @property\n    def alive(self):\n        return self._alive\n";
    return s42;
  }
  return none;
};
PyStdlib.isPackage = function(name) {
  if ( name == "collections" ) {
    return true;
  }
  if ( name == "unittest" ) {
    return true;
  }
  return false;
};
export class PyFrame  {
  constructor() {
    this.func = undefined;
    this.__self = undefined;
    this.name = "";
    this.line = 0;
    this.callLine = 0;
  }
}
export class PyInterp  {
  constructor(filesystem) {
    this.fs = new PyFileSystem();
    this.outParts = [];
    this.echo = false;
    this.pendingLine = "";
    this.builtins = {};
    this.modules = {};
    this.NONE = new PyVal(0);
    this.TRUE = new PyVal(1);
    this.FALSE = new PyVal(1);
    this.NOTIMPL = new PyVal(10);
    this.excValue = undefined;
    this.handling = [];
    this.retVal = new PyVal(0);
    this.nextId = 1;
    this.maxDepth = 200;
    this.steps = 0;
    this.maxSteps = 0;
    this.curLine = 0;
    this.frames = [];
    this.curGen = undefined;
    this.suspending = false;
    this.SUSPEND = new PyVal(0);
    this.ENDED = new PyVal(0);
    this.compFirst = undefined;
    this.rawSelf = undefined;
    this.reprDepth = 0;
    this.globals = new PyScope();
    this.bi = undefined;
    this.meth = undefined;
    this.mods = undefined;
    this.argv = [];
    this.searchPath = [];
    this.sysModules = new PyVal(7);
    this.callScope = new PyScope();
    this.noAttrHook = false;
    this.codeCls = undefined;
    this.fs = filesystem;
    this.TRUE.i = 1;
    this.NOTIMPL.s = "NotImplemented";
    this.retVal = this.NONE;
    this.globals = new PyScope();
  }
  makeModule (name) {
    const mdl = this.mods;
    return mdl.make(name);
  };
  setup () {
    const b = new PyBuiltins(this);
    this.bi = b;
    const m = new PyMethods(this);
    this.meth = m;
    const md = new PyModules(this);
    this.mods = md;
    m.init();
    b.install();
    this.resetGlobals();
  };
  resetGlobals () {
    this.globals = new PyScope();
    this.globals.vars["__name__"] = this.mkStr("__main__");
    this.globals.vars["__file__"] = this.mkStr("<script>");
  };
  newId () {
    this.nextId = this.nextId + 1;
    return this.nextId;
  };
  idOf (v) {
    if ( v.id == 0 ) {
      v.id = this.newId();
    }
    return v.id;
  };
  mkInt (n) {
    const v = new PyVal(2);
    v.i = n;
    return v;
  };
  mkFloat (x) {
    const v = new PyVal(3);
    v.d = x;
    return v;
  };
  mkStr (s) {
    const v = new PyVal(4);
    v.s = s;
    return v;
  };
  mkBool (b) {
    if ( b ) {
      return this.TRUE;
    }
    return this.FALSE;
  };
  mkList (items) {
    const v = new PyVal(5);
    v.items = items;
    return v;
  };
  mkTuple (items) {
    const v = new PyVal(6);
    v.items = items;
    return v;
  };
  mkDict () {
    return new PyVal(7);
  };
  mkSet () {
    return new PyVal(8);
  };
  mkIter (items) {
    const v = new PyVal(17);
    v.items = items;
    v.s = "iterator";
    return v;
  };
  mkBuiltinMethod (__self, name) {
    const b = new PyVal(14);
    b.__self = __self;
    b.s = name;
    return b;
  };
  argAt (xs, i) {
    if ( i >= 0 && i < xs.length ) {
      return xs[i];
    }
    return this.NONE;
  };
  orNone (v) {
    if ( typeof(v) === "undefined" ) {
      return this.NONE;
    }
    const got = v;
    return got;
  };
  pair (a, b) {
    let it = [];
    it.push(a);
    it.push(b);
    return this.mkTuple(it);
  };
  isNum (v) {
    return (v.t == 2 || v.t == 1) || v.t == 3;
  };
  isIntLike (v) {
    return v.t == 2 || v.t == 1;
  };
  toDouble (v) {
    if ( v.t == 3 ) {
      return v.d;
    }
    return (v.i);
  };
  excClass (name) {
    const c = ( Object.prototype.hasOwnProperty.call(this.builtins, name) ? this.builtins[name] : undefined );
    if ( typeof(c) === "undefined" ) {
      const e = ( Object.prototype.hasOwnProperty.call(this.builtins, "Exception") ? this.builtins["Exception"] : undefined );
      return e;
    }
    return c;
  };
  makeExc (cls, msg) {
    const e = new PyVal(12);
    e.cls = cls;
    let args = [];
    if ( msg.length > 0 ) {
      args.push(this.mkStr(msg));
    }
    e.putAttr("args", this.mkTuple(args));
    return e;
  };
  raise (name, msg) {
    const e = this.makeExc(this.excClass(name), msg);
    this.raiseVal(e);
  };
  raiseVal (e) {
    if ( ( typeof(e.attrs["__line__"] ) != "undefined" && Object.prototype.hasOwnProperty.call(e.attrs, "__line__") ) == false ) {
      e.putAttr("__line__", this.mkInt(this.curLine));
      e.putAttr("__trace__", this.mkStr(this.traceText()));
    }
    this.excValue = e;
    throw new Error("python exception");
  };
  markExc (e) {
    if ( ( typeof(e.attrs["__line__"] ) != "undefined" && Object.prototype.hasOwnProperty.call(e.attrs, "__line__") ) == false ) {
      e.putAttr("__line__", this.mkInt(this.curLine));
      e.putAttr("__trace__", this.mkStr(this.traceText()));
    }
  };
  traceText () {
    let where = [];
    let caller = "<module>";
    // Loop start
    for ( const f of this.frames) {
      where.push((("  File \"<script>\", line " + ((f.callLine).toString())) + ", in ") + caller);
      caller = f.name;
    }
    where.push((("  File \"<script>\", line " + ((this.curLine).toString())) + ", in ") + caller);
    let lines = [];
    lines.push("Traceback (most recent call last):");
    let prev = "";
    let run = 0;
    // Loop start
    for ( const w of where) {
      if ( w == prev ) {
        run = run + 1;
        if ( run >= 3 ) {
          continue;
        }
      } else {
        if ( run >= 3 ) {
          lines.push(((("  [Previous line repeated " + ((run - 2).toString())) + " more time") + this.plural((run - 2))) + "]");
        }
        run = 0;
      }
      lines.push(w);
      prev = w;
    }
    if ( run >= 3 ) {
      lines.push(((("  [Previous line repeated " + ((run - 2).toString())) + " more time") + this.plural((run - 2))) + "]");
    }
    return lines.join("\n");
  };
  takeException (msg) {
    if ( (typeof(this.excValue) !== "undefined" && this.excValue != null )  ) {
      const e = this.excValue;
      this.clearExc();
      return e;
    }
    const low = msg.toLowerCase();
    if ( ((low.indexOf("call stack") >= 0 || low.indexOf("stack overflow") >= 0) || low.indexOf("recursion") >= 0) || low.indexOf("goroutine stack exceeds") >= 0 ) {
      const rx = this.makeExc(this.excClass("RecursionError"), "maximum recursion depth exceeded");
      this.markExc(rx);
      return rx;
    }
    return this.makeExc(this.excClass("RuntimeError"), ("internal error: " + msg));
  };
  clearExc () {
    let none;
    this.excValue = none;
  };
  excMatches (e, cls) {
    if ( cls.t == 6 ) {
      // Loop start
      for ( const c of cls.items) {
        if ( this.excMatches(e, c) ) {
          return true;
        }
      }
      return false;
    }
    if ( cls.t != 11 ) {
      this.raise("TypeError", "catching classes that do not inherit from BaseException is not allowed");
    }
    if ( typeof(e.cls) === "undefined" ) {
      return false;
    }
    const ec = e.cls;
    return this.isSubclass(ec, cls);
  };
  excMessage (e) {
    let name = "Exception";
    if ( (typeof(e.cls) !== "undefined" && e.cls != null )  ) {
      const c = e.cls;
      name = c.s;
    }
    const text = this.toStr(e);
    if ( text.length == 0 ) {
      return name;
    }
    return (name + ": ") + text;
  };
  tick () {
    this.steps = this.steps + 1;
    if ( this.maxSteps > 0 && this.steps > this.maxSteps ) {
      this.raise("TimeoutError", ("the script ran more than " + ((this.maxSteps).toString())) + " steps");
    }
  };
  writeOut (s) {
    this.outParts.push(s);
    if ( this.echo ) {
      const cs = Array.from(s, (rg_c) => rg_c.codePointAt(0));
      let start = 0;
      const n = cs.length;
      let i = 0;
      while (i < n) {
        if ( cs[i] == 10 ) {
          console.log(this.pendingLine + py_rt_codes(cs, start, i));
          this.pendingLine = "";
          start = i + 1;
        }
        i = i + 1;
      };
      if ( start < n ) {
        this.pendingLine = this.pendingLine + py_rt_codes(cs, start, n);
      }
    }
  };
  flushEcho () {
    if ( this.echo && this.pendingLine.length > 0 ) {
      console.log(this.pendingLine);
      this.pendingLine = "";
    }
  };
  output () {
    return this.outParts.join("");
  };
  typeName (v) {
    if ( (typeof(v.cls) !== "undefined" && v.cls != null )  ) {
      if ( this.isDataSub(v) ) {
        const tcl = v.cls;
        return tcl.s;
      }
    }
    if ( ((v.t == 5 || v.t == 7) || v.t == 6) && ( typeof(v.attrs["__class__"] ) != "undefined" && Object.prototype.hasOwnProperty.call(v.attrs, "__class__") ) ) {
      const tc = ( Object.prototype.hasOwnProperty.call(v.attrs, "__class__") ? v.attrs["__class__"] : undefined );
      return tc.s;
    }
    switch (v.t ) { 
      case 0 : 
        return "NoneType";
      case 1 : 
        return "bool";
      case 2 : 
        return "int";
      case 3 : 
        return "float";
      case 24 : 
        return "complex";
      case 4 : 
        return "str";
      case 5 : 
        return "list";
      case 6 : 
        return "tuple";
      case 7 : 
        return "dict";
      case 8 : 
        if ( v.mode == "frozen" ) {
          return "frozenset";
        }
        return "set";
      case 9 : 
        return "function";
      case 10 : 
        if ( this.isTypeBuiltin(v.s) ) {
          return "type";
        }
        return "builtin_function_or_method";
      case 11 : 
        return "type";
      case 12 : 
        if ( (typeof(v.cls) !== "undefined" && v.cls != null )  ) {
          const c = v.cls;
          return c.s;
        }
        return "object";
      case 13 : 
        return "module";
      case 14 : 
        if ( typeof(v.cls) === "undefined" ) {
          return "builtin_function_or_method";
        }
        return "method";
      case 15 : 
        return "range";
      case 16 : 
        return "TextIOWrapper";
      case 17 : 
        return v.s;
      case 23 : 
        return "generator";
      case 18 : 
        return "slice";
      case 19 : 
        return "property";
      case 20 : 
        return "staticmethod";
      case 21 : 
        return "classmethod";
      case 22 : 
        return "super";
    };
    return "object";
  };
  isTypeBuiltin (name) {
    const ts = ["int", "float", "complex", "str", "bool", "list", "tuple", "dict", "set", "frozenset", "object", "type", "range", "slice", "enumerate", "zip", "map", "filter", "reversed", "property", "staticmethod", "classmethod", "super"];
    // Loop start
    for ( const t of ts) {
      if ( t == name ) {
        return true;
      }
    }
    return false;
  };
  truthy (v) {
    if ( (typeof(v.cls) !== "undefined" && v.cls != null )  ) {
      const sb = this.subDunder(v, "__bool__", []);
      if ( (typeof(sb) !== "undefined" && sb != null )  ) {
        return this.truthy(sb);
      }
      const sbl = this.subDunder(v, "__len__", []);
      if ( (typeof(sbl) !== "undefined" && sbl != null )  ) {
        return this.truthy(sbl);
      }
    }
    switch (v.t ) { 
      case 0 : 
        return false;
      case 1 : 
        return v.i != 0;
      case 2 : 
        return v.i != 0;
      case 3 : 
        return v.d != 0.0;
      case 24 : 
        return v.d != 0.0 || v.d2 != 0.0;
      case 4 : 
        return v.s.length > 0;
      case 5 : 
        return v.items.length > 0;
      case 6 : 
        return v.items.length > 0;
      case 7 : 
        return v.items.length > 0;
      case 8 : 
        return v.items.length > 0;
      case 15 : 
        return this.rangeLen(v) > 0;
      case 12 : 
        const b = this.findMethod(v, "__bool__");
        if ( (typeof(b) !== "undefined" && b != null )  ) {
          const r = this.callMethodVal(b, v, []);
          return this.truthy(r);
        }
        const l = this.findMethod(v, "__len__");
        if ( (typeof(l) !== "undefined" && l != null )  ) {
          const r2 = this.callMethodVal(l, v, []);
          return r2.i != 0;
        }
        return true;
    };
    return true;
  };
  rangeLen (r) {
    if ( r.i3 > 0 ) {
      if ( r.i2 <= r.i ) {
        return 0;
      }
      return PyFormat.floorDivInt(((r.i2 - r.i) + (r.i3 - 1)), r.i3);
    }
    if ( r.i2 >= r.i ) {
      return 0;
    }
    const st = 0 - r.i3;
    return PyFormat.floorDivInt(((r.i - r.i2) + (st - 1)), st);
  };
  length (v) {
    if ( (typeof(v.cls) !== "undefined" && v.cls != null )  ) {
      const sl = this.subDunder(v, "__len__", []);
      if ( (typeof(sl) !== "undefined" && sl != null )  ) {
        const slv = sl;
        return slv.i;
      }
    }
    switch (v.t ) { 
      case 4 : 
        return v.strLen();
      case 5 : 
        return v.items.length;
      case 6 : 
        return v.items.length;
      case 7 : 
        return v.items.length;
      case 8 : 
        return v.items.length;
      case 15 : 
        return this.rangeLen(v);
      case 12 : 
        const l = this.findMethod(v, "__len__");
        if ( (typeof(l) !== "undefined" && l != null )  ) {
          const r = this.callMethodVal(l, v, []);
          if ( this.isIntLike(r) == false ) {
            this.raise("TypeError", "__len__ must return an integer");
          }
          return r.i;
        }
        break;
    };
    this.raise("TypeError", ("object of type '" + this.typeName(v)) + "' has no len()");
    return 0;
  };
  toStr (v) {
    if ( (typeof(v.cls) !== "undefined" && v.cls != null )  ) {
      const ss = this.subDunder(v, "__str__", []);
      if ( (typeof(ss) !== "undefined" && ss != null )  ) {
        return this.toStr(ss);
      }
      if ( v.t != 4 ) {
        const ss2 = this.subDunder(v, "__repr__", []);
        if ( (typeof(ss2) !== "undefined" && ss2 != null )  ) {
          return this.toStr(ss2);
        }
      }
    }
    switch (v.t ) { 
      case 4 : 
        return v.s;
      case 12 : 
        const m = this.findMethod(v, "__str__");
        if ( (typeof(m) !== "undefined" && m != null )  ) {
          const r = this.callMethodVal(m, v, []);
          if ( r.t != 4 ) {
            this.raise("TypeError", "__str__ returned non-string");
          }
          return r.s;
        }
        if ( this.isExcInst(v) ) {
          const args = this.excArgs(v);
          const n = args.items.length;
          if ( n == 0 ) {
            return "";
          }
          if ( n == 1 ) {
            if ( this.excMatches(v, this.excClass("KeyError")) ) {
              return this.toRepr(args.items[0]);
            }
            return this.toStr(args.items[0]);
          }
          return this.toRepr(args);
        }
        return this.toRepr(v);
    };
    return this.toRepr(v);
  };
  isExcInst (v) {
    if ( v.t != 12 || (typeof(v.cls) === "undefined") ) {
      return false;
    }
    const c = v.cls;
    return c.isExc;
  };
  excArgs (v) {
    const a = ( Object.prototype.hasOwnProperty.call(v.attrs, "args") ? v.attrs["args"] : undefined );
    if ( typeof(a) === "undefined" ) {
      return this.mkTuple([]);
    }
    return a;
  };
  hexId (v) {
    const n = this.idOf(v) * 16 + 140000000000;
    return "0x" + PyFormat.intBase(n, 16, false);
  };
  joinRepr (items) {
    let parts = [];
    // Loop start
    for ( const it of items) {
      parts.push(this.toRepr(it));
    }
    return parts.join(", ");
  };
  defaultRepr (v) {
    if ( v.t == 12 && ((typeof(v.cls) !== "undefined" && v.cls != null ) ) ) {
      const c = v.cls;
      let mod = "__main__";
      const mv = this.orNone(( Object.prototype.hasOwnProperty.call(c.attrs, "__module__") ? c.attrs["__module__"] : undefined ));
      if ( mv.t == 4 ) {
        mod = mv.s;
      }
      let qn = c.s;
      const qv = this.orNone(( Object.prototype.hasOwnProperty.call(c.attrs, "__qualname__") ? c.attrs["__qualname__"] : undefined ));
      if ( qv.t == 4 ) {
        qn = qv.s;
      }
      return ((((("<" + mod) + ".") + qn) + " object at ") + this.hexId(v)) + ">";
    }
    return this.toRepr(v);
  };
  getAttrNoHook (obj, name) {
    const saved = this.noAttrHook;
    this.noAttrHook = true;
    const r = this.getAttrOpt(obj, name);
    this.noAttrHook = saved;
    return r;
  };
  toRepr (v) {
    if ( (typeof(v.cls) !== "undefined" && v.cls != null )  ) {
      const sr = this.subDunder(v, "__repr__", []);
      if ( (typeof(sr) !== "undefined" && sr != null )  ) {
        return this.toStr(sr);
      }
    }
    switch (v.t ) { 
      case 0 : 
        return "None";
      case 1 : 
        if ( v.i != 0 ) {
          return "True";
        }
        return "False";
      case 2 : 
        return ((v.i).toString());
      case 3 : 
        return PyFormat.floatRepr(v.d);
      case 24 : 
        return this.complexRepr(v);
      case 4 : 
        return PyFormat.strRepr(v.s);
    };
    this.reprDepth = this.reprDepth + 1;
    if ( this.reprDepth > 60 ) {
      this.reprDepth = this.reprDepth - 1;
      return "...";
    }
    const r = this.toReprObj(v);
    this.reprDepth = this.reprDepth - 1;
    return r;
  };
  toReprObj (v) {
    if ( ((v.t == 5 || v.t == 7) || v.t == 6) && ( typeof(v.attrs["__class__"] ) != "undefined" && Object.prototype.hasOwnProperty.call(v.attrs, "__class__") ) ) {
      const cr = this.mods.reprOf(v);
      if ( cr.length > 0 ) {
        return cr;
      }
    }
    if ( v.t == 5 && v.mode.startsWith("dict_") ) {
      return ((v.mode + "([") + this.joinRepr(v.items)) + "])";
    }
    switch (v.t ) { 
      case 5 : 
        return ("[" + this.joinRepr(v.items)) + "]";
      case 6 : 
        if ( v.items.length == 1 ) {
          return ("(" + this.toRepr(v.items[0])) + ",)";
        }
        return ("(" + this.joinRepr(v.items)) + ")";
      case 7 : 
        let parts = [];
        // Loop start
        for ( let i = 0; i < v.items.length; i++) {
          var k = v.items[i];
          parts.push((this.toRepr(k) + ": ") + this.toRepr(v.vals[i]));
        }
        return ("{" + parts.join(", ")) + "}";
      case 8 : 
        if ( v.mode == "frozen" ) {
          if ( v.items.length == 0 ) {
            return "frozenset()";
          }
          return ("frozenset({" + this.joinRepr(v.items)) + "})";
        }
        if ( v.items.length == 0 ) {
          return "set()";
        }
        return ("{" + this.joinRepr(v.items)) + "}";
      case 9 : 
        return ((("<function " + v.s) + " at ") + this.hexId(v)) + ">";
      case 10 : 
        if ( this.isTypeBuiltin(v.s) ) {
          return ("<class '" + v.s) + "'>";
        }
        if ( v.s == "NotImplemented" ) {
          return "NotImplemented";
        }
        if ( v.s == "Ellipsis" ) {
          return "Ellipsis";
        }
        const bd2 = v.s.lastIndexOf(".");
        return ("<built-in function " + v.s.substring(bd2 + 1, v.s.length )) + ">";
      case 11 : 
        if ( v.mode == "builtin" ) {
          return ("<class '" + v.s) + "'>";
        }
        if ( v.mode == "native" ) {
          if ( ( typeof(v.attrs["_fields"] ) != "undefined" && Object.prototype.hasOwnProperty.call(v.attrs, "_fields") ) ) {
            return ("<class '__main__." + v.s) + "'>";
          }
          return ("<class 'collections." + v.s) + "'>";
        }
        return ("<class '__main__." + v.s) + "'>";
      case 12 : 
        const m = this.findMethod(v, "__repr__");
        if ( (typeof(m) !== "undefined" && m != null )  ) {
          const r = this.callMethodVal(m, v, []);
          if ( r.t != 4 ) {
            this.raise("TypeError", "__repr__ returned non-string");
          }
          return r.s;
        }
        if ( this.isExcInst(v) ) {
          const args = this.excArgs(v);
          const c = v.cls;
          if ( args.items.length == 1 ) {
            return ((c.s + "(") + this.toRepr(args.items[0])) + ")";
          }
          return c.s + this.toRepr(args);
        }
        return this.defaultRepr(v);
      case 13 : 
        return ("<module '" + v.s) + "'>";
      case 14 : 
        if ( typeof(v.cls) === "undefined" ) {
          return ("<built-in method " + v.s) + ">";
        }
        const f = v.cls;
        return ("<bound method " + f.s) + ">";
      case 15 : 
        if ( v.i3 == 1 ) {
          return ((("range(" + ((v.i).toString())) + ", ") + ((v.i2).toString())) + ")";
        }
        return ((((("range(" + ((v.i).toString())) + ", ") + ((v.i2).toString())) + ", ") + ((v.i3).toString())) + ")";
      case 16 : 
        return ((("<_io.TextIOWrapper name=" + PyFormat.strRepr(v.s)) + " mode=") + PyFormat.strRepr(v.mode)) + " encoding='UTF-8'>";
      case 17 : 
        return ((("<" + v.s) + " object at ") + this.hexId(v)) + ">";
      case 23 : 
        return ((("<generator object " + v.s) + " at ") + this.hexId(v)) + ">";
      case 18 : 
        return ("slice(" + this.joinRepr(v.items)) + ")";
    };
    return ("<" + this.typeName(v)) + " object>";
  };
  hashKey (v) {
    if ( (typeof(v.cls) !== "undefined" && v.cls != null )  ) {
      const sh = this.subDunder(v, "__hash__", []);
      if ( (typeof(sh) !== "undefined" && sh != null )  ) {
        return "h" + this.toStr(sh);
      }
    }
    switch (v.t ) { 
      case 0 : 
        return "N";
      case 1 : 
        return "i" + ((v.i).toString());
      case 2 : 
        return "i" + ((v.i).toString());
      case 3 : 
        if ( (Math.floor(v.d) == v.d && v.d < 9000000000000000.0) && v.d > -9000000000000000.0 ) {
          return "i" + ((Math.trunc(v.d)).toString());
        }
        return "f" + PyFormat.floatRepr(v.d);
      case 4 : 
        return "s" + v.s;
      case 24 : 
        if ( v.d2 == 0.0 ) {
          return this.hashKey(this.mkFloat(v.d));
        }
        return "c" + this.complexRepr(v);
      case 6 : 
        let parts = [];
        // Loop start
        for ( const it of v.items) {
          parts.push(this.hashKey(it));
        }
        return ("t(" + parts.join(String.fromCodePoint(1))) + ")";
      case 8 : 
        if ( v.mode == "frozen" ) {
          let ks = [];
          // Loop start
          for ( const it2 of v.items) {
            ks.push(this.hashKey(it2));
          }
          return ("z(" + PyFileSystem.sortStrings(ks).join(String.fromCodePoint(1))) + ")";
        }
        break;
      case 12 : 
        const h = this.findMethod(v, "__hash__");
        if ( (typeof(h) !== "undefined" && h != null )  ) {
          const r = this.callMethodVal(h, v, []);
          return "h" + ((r.i).toString());
        }
        return "o" + ((this.idOf(v)).toString());
    };
    if ( (v.t == 5 || v.t == 7) || v.t == 8 ) {
      this.raise("TypeError", ("unhashable type: '" + this.typeName(v)) + "'");
    }
    return "o" + ((this.idOf(v)).toString());
  };
  same (a, b) {
    if ( a.t != b.t ) {
      return false;
    }
    if ( a.t == 0 || a.t == 1 ) {
      return a.i == b.i;
    }
    if ( ((a.t == 2 && a.i == b.i) && a.i >= -5) && a.i <= 256 ) {
      return true;
    }
    return this.idOf(a) == this.idOf(b);
  };
  eq (a, b) {
    if ( ((typeof(a.cls) !== "undefined" && a.cls != null ) ) || ((typeof(b.cls) !== "undefined" && b.cls != null ) ) ) {
      if ( this.isDataSub(a) || this.isDataSub(b) ) {
        const se = this.dunderBin("__eq__", "__eq__", a, b);
        if ( (typeof(se) !== "undefined" && se != null )  ) {
          return this.truthy(se);
        }
      }
    }
    if ( ((a.t == 24 || b.t == 24) && this.isComplexNum(a)) && this.isComplexNum(b) ) {
      return this.reOf(a) == this.reOf(b) && this.imOf(a) == this.imOf(b);
    }
    if ( this.isNum(a) && this.isNum(b) ) {
      if ( this.isIntLike(a) && this.isIntLike(b) ) {
        return a.i == b.i;
      }
      return this.toDouble(a) == this.toDouble(b);
    }
    if ( a.t == 4 && b.t == 4 ) {
      return a.s == b.s;
    }
    if ( a.t == 12 || b.t == 12 ) {
      const r = this.dunderBin("__eq__", "__eq__", a, b);
      if ( (typeof(r) !== "undefined" && r != null )  ) {
        return this.truthy(r);
      }
      return this.same(a, b);
    }
    if ( a.t != b.t ) {
      if ( a.t == 8 && b.t == 8 ) {
        return this.setEq(a, b);
      }
      return false;
    }
    switch (a.t ) { 
      case 0 : 
        return true;
      case 5 : 
        return this.seqEq(a.items, b.items);
      case 6 : 
        return this.seqEq(a.items, b.items);
      case 7 : 
        if ( a.items.length != b.items.length ) {
          return false;
        }
        // Loop start
        for ( let i = 0; i < a.items.length; i++) {
          var k = a.items[i];
          const bi2 = ( Object.prototype.hasOwnProperty.call(b.index, this.hashKey(k)) ? b.index[this.hashKey(k)] : undefined );
          if ( typeof(bi2) === "undefined" ) {
            return false;
          }
          if ( this.eq(a.vals[i], b.vals[bi2]) == false ) {
            return false;
          }
        }
        return true;
      case 8 : 
        return this.setEq(a, b);
      case 15 : 
        return (a.i == b.i && a.i2 == b.i2) && a.i3 == b.i3;
    };
    return this.same(a, b);
  };
  setEq (a, b) {
    if ( a.items.length != b.items.length ) {
      return false;
    }
    // Loop start
    for ( const k of a.items) {
      if ( ( typeof(b.index[this.hashKey(k)] ) != "undefined" && Object.prototype.hasOwnProperty.call(b.index, this.hashKey(k)) ) == false ) {
        return false;
      }
    }
    return true;
  };
  seqEq (a, b) {
    if ( a.length != b.length ) {
      return false;
    }
    let i = 0;
    const n = a.length;
    while (i < n) {
      if ( this.eq(a[i], b[i]) == false ) {
        return false;
      }
      i = i + 1;
    };
    return true;
  };
  less (a, b) {
    if ( ((typeof(a.cls) !== "undefined" && a.cls != null ) ) || ((typeof(b.cls) !== "undefined" && b.cls != null ) ) ) {
      if ( this.isDataSub(a) || this.isDataSub(b) ) {
        const sl = this.dunderBin("__lt__", "__gt__", a, b);
        if ( (typeof(sl) !== "undefined" && sl != null )  ) {
          return this.truthy(sl);
        }
      }
    }
    if ( this.isNum(a) && this.isNum(b) ) {
      if ( this.isIntLike(a) && this.isIntLike(b) ) {
        return a.i < b.i;
      }
      return this.toDouble(a) < this.toDouble(b);
    }
    if ( a.t == 4 && b.t == 4 ) {
      return PyInterp.strCmp(a, b) < 0;
    }
    if ( a.t == b.t && (a.t == 5 || a.t == 6) ) {
      const na = a.items.length;
      const nb = b.items.length;
      let i = 0;
      while (i < na && i < nb) {
        const x = a.items[i];
        const y = b.items[i];
        if ( this.eq(x, y) == false ) {
          return this.less(x, y);
        }
        i = i + 1;
      };
      return na < nb;
    }
    if ( a.t == 8 && b.t == 8 ) {
      return a.items.length < b.items.length && this.isSubset(a, b);
    }
    if ( this.isObjLike(a) || this.isObjLike(b) ) {
      const r = this.dunderBin("__lt__", "__gt__", a, b);
      if ( (typeof(r) !== "undefined" && r != null )  ) {
        return this.truthy(r);
      }
    }
    this.raise("TypeError", ((("'<' not supported between instances of '" + this.typeName(a)) + "' and '") + this.typeName(b)) + "'");
    return false;
  };
  isSubset (a, b) {
    // Loop start
    for ( const k of a.items) {
      if ( ( typeof(b.index[this.hashKey(k)] ) != "undefined" && Object.prototype.hasOwnProperty.call(b.index, this.hashKey(k)) ) == false ) {
        return false;
      }
    }
    return true;
  };
  compare (op, a, b) {
    if ( op == "==" ) {
      return this.eq(a, b);
    }
    if ( op == "!=" ) {
      if ( this.isObjLike(a) || this.isObjLike(b) ) {
        const r = this.dunderBin("__ne__", "__ne__", a, b);
        if ( (typeof(r) !== "undefined" && r != null )  ) {
          return this.truthy(r);
        }
      }
      return this.eq(a, b) == false;
    }
    if ( op == "<" ) {
      return this.less(a, b);
    }
    if ( op == ">" ) {
      if ( this.isObjLike(a) || this.isObjLike(b) ) {
        const r2 = this.dunderBin("__gt__", "__lt__", a, b);
        if ( (typeof(r2) !== "undefined" && r2 != null )  ) {
          return this.truthy(r2);
        }
      }
      return this.less(b, a);
    }
    if ( op == "<=" ) {
      if ( this.isObjLike(a) || this.isObjLike(b) ) {
        const r3 = this.dunderBin("__le__", "__ge__", a, b);
        if ( (typeof(r3) !== "undefined" && r3 != null )  ) {
          return this.truthy(r3);
        }
      }
      if ( a.t == 8 && b.t == 8 ) {
        return this.isSubset(a, b);
      }
      if ( this.isNum(a) && this.isNum(b) ) {
        if ( this.isIntLike(a) && this.isIntLike(b) ) {
          return a.i <= b.i;
        }
        return this.toDouble(a) <= this.toDouble(b);
      }
      return this.less(b, a) == false;
    }
    if ( op == ">=" ) {
      if ( this.isObjLike(a) || this.isObjLike(b) ) {
        const r4 = this.dunderBin("__ge__", "__le__", a, b);
        if ( (typeof(r4) !== "undefined" && r4 != null )  ) {
          return this.truthy(r4);
        }
      }
      if ( a.t == 8 && b.t == 8 ) {
        return this.isSubset(b, a);
      }
      if ( this.isNum(a) && this.isNum(b) ) {
        if ( this.isIntLike(a) && this.isIntLike(b) ) {
          return a.i >= b.i;
        }
        return this.toDouble(a) >= this.toDouble(b);
      }
      return this.less(a, b) == false;
    }
    if ( op == "in" ) {
      return this.contains(b, a);
    }
    if ( op == "not in" ) {
      return this.contains(b, a) == false;
    }
    if ( op == "is" ) {
      return this.same(a, b);
    }
    if ( op == "is not" ) {
      return this.same(a, b) == false;
    }
    this.raise("SyntaxError", "unknown comparison " + op);
    return false;
  };
  contains (container, item) {
    if ( (typeof(container.cls) !== "undefined" && container.cls != null )  ) {
      const sc = this.subDunder(container, "__contains__", [item]);
      if ( (typeof(sc) !== "undefined" && sc != null )  ) {
        return this.truthy(sc);
      }
    }
    switch (container.t ) { 
      case 4 : 
        if ( item.t != 4 ) {
          this.raise("TypeError", "'in <string>' requires string as left operand, not " + this.typeName(item));
        }
        return container.s.indexOf(item.s) >= 0;
      case 5 : 
        // Loop start
        for ( const it of container.items) {
          if ( this.eq(it, item) ) {
            return true;
          }
        }
        return false;
      case 6 : 
        // Loop start
        for ( const it2 of container.items) {
          if ( this.eq(it2, item) ) {
            return true;
          }
        }
        return false;
      case 7 : 
        return ( typeof(container.index[this.hashKey(item)] ) != "undefined" && Object.prototype.hasOwnProperty.call(container.index, this.hashKey(item)) );
      case 8 : 
        return ( typeof(container.index[this.hashKey(item)] ) != "undefined" && Object.prototype.hasOwnProperty.call(container.index, this.hashKey(item)) );
      case 15 : 
        if ( this.isIntLike(item) == false ) {
          return false;
        }
        const v = item.i;
        if ( container.i3 > 0 ) {
          if ( v < container.i || v >= container.i2 ) {
            return false;
          }
        } else {
          if ( v > container.i || v <= container.i2 ) {
            return false;
          }
        }
        return PyFormat.floorModInt(v - container.i, container.i3) == 0;
      case 12 : 
        const m = this.findMethod(container, "__contains__");
        if ( (typeof(m) !== "undefined" && m != null )  ) {
          const r = this.callMethodVal(m, container, [item]);
          return this.truthy(r);
        }
        break;
    };
    if ( this.isLazy(container) ) {
      const cit = this.iterOf(container);
      while (true) {
        const cv = this.nextOpt(cit);
        if ( typeof(cv) === "undefined" ) {
          return false;
        }
        if ( this.eq(cv, item) ) {
          return true;
        }
      };
    }
    const all = this.iterValues(container);
    // Loop start
    for ( const a of all) {
      if ( this.eq(a, item) ) {
        return true;
      }
    }
    return false;
  };
  dictGet (d, key) {
    const i = ( Object.prototype.hasOwnProperty.call(d.index, this.hashKey(key)) ? d.index[this.hashKey(key)] : undefined );
    if ( typeof(i) === "undefined" ) {
      let none;
      return none;
    }
    const optr1110 = d.vals[i];
    return optr1110;
  };
  dictSet (d, key, val) {
    const hk = this.hashKey(key);
    const i = ( Object.prototype.hasOwnProperty.call(d.index, hk) ? d.index[hk] : undefined );
    if ( (typeof(d.owner) !== "undefined" && d.owner != null )  ) {
      this.ownerPut(d, key, val);
    }
    if ( typeof(i) === "undefined" ) {
      d.index[hk] = d.items.length;
      d.items.push(key);
      d.vals.push(val);
      return;
    }
    d.vals[i] = val;
  };
  ownerPut (d, key, val) {
    if ( key.t != 4 ) {
      return;
    }
    const o = d.owner;
    if ( ( typeof(o.attrs[key.s] ) != "undefined" && Object.prototype.hasOwnProperty.call(o.attrs, key.s) ) == false ) {
      o.akeys.push(key.s);
    }
    o.attrs[key.s] = val;
  };
  dictDel (d, key) {
    const hk = this.hashKey(key);
    const i = ( Object.prototype.hasOwnProperty.call(d.index, hk) ? d.index[hk] : undefined );
    if ( typeof(i) === "undefined" ) {
      return false;
    }
    const at = i;
    d.items.splice(at, 1).pop();
    if ( d.t == 7 ) {
      d.vals.splice(at, 1).pop();
    }
    delete d.index[hk];
    if ( ((typeof(d.owner) !== "undefined" && d.owner != null ) ) && key.t == 4 ) {
      const dow = d.owner;
      delete dow.attrs[key.s];
    }
    const n = d.items.length;
    let k = at;
    while (k < n) {
      d.index[this.hashKey(d.items[k])] = k;
      k = k + 1;
    };
    return true;
  };
  dictStr (d, key) {
    const r = this.dictGet(d, this.mkStr(key));
    if ( typeof(r) === "undefined" ) {
      return this.NONE;
    }
    return r;
  };
  setAdd (s, item) {
    const hk = this.hashKey(item);
    if ( ( typeof(s.index[hk] ) != "undefined" && Object.prototype.hasOwnProperty.call(s.index, hk) ) ) {
      return;
    }
    s.index[hk] = s.items.length;
    s.items.push(item);
  };
  copyDict (d) {
    const r = this.mkDict();
    // Loop start
    for ( let i = 0; i < d.items.length; i++) {
      var k = d.items[i];
      this.dictSet(r, k, d.vals[i]);
    }
    return r;
  };
  copySet (d) {
    const r = this.mkSet();
    // Loop start
    for ( const k of d.items) {
      this.setAdd(r, k);
    }
    return r;
  };
  iterValues (v) {
    if ( (typeof(v.cls) !== "undefined" && v.cls != null )  ) {
      const si = this.subDunder(v, "__iter__", []);
      if ( (typeof(si) !== "undefined" && si != null )  ) {
        return this.iterValues(si);
      }
    }
    switch (v.t ) { 
      case 5 : 
        let out = [];
        // Loop start
        for ( const it of v.items) {
          out.push(it);
        }
        return out;
      case 6 : 
        return v.items;
      case 4 : 
        let out2 = [];
        const cs = v.codes();
        // Loop start
        for ( const c of cs) {
          out2.push(this.mkStr(String.fromCodePoint(c)));
        }
        return out2;
      case 7 : 
        let out3 = [];
        // Loop start
        for ( const k of v.items) {
          out3.push(k);
        }
        return out3;
      case 8 : 
        let out4 = [];
        // Loop start
        for ( const k2 of v.items) {
          out4.push(k2);
        }
        return out4;
      case 15 : 
        let out5 = [];
        const n = this.rangeLen(v);
        let j = 0;
        let x = v.i;
        while (j < n) {
          out5.push(this.mkInt(x));
          x = x + v.i3;
          j = j + 1;
        };
        return out5;
      case 17 : 
        let out6 = [];
        if ( v.mode.length > 0 ) {
          while (true) {
            const lz = this.lazyNext(v);
            if ( typeof(lz) === "undefined" ) {
              break;
            }
            out6.push(lz);
            this.tick();
          };
          return out6;
        }
        let k3 = v.pos;
        const n2 = v.items.length;
        while (k3 < n2) {
          out6.push(v.items[k3]);
          k3 = k3 + 1;
        };
        v.pos = n2;
        return out6;
      case 16 : 
        return this.meth.fileLines(v);
      case 23 : 
        let out8 = [];
        while (this.genRun(v, this.NONE, 0)) {
          const g = v.gen;
          out8.push(g.out);
          this.tick();
        };
        return out8;
      case 12 : 
        const it3 = this.findMethod(v, "__iter__");
        if ( (typeof(it3) !== "undefined" && it3 != null )  ) {
          const r = this.callMethodVal(it3, v, []);
          if ( r.t == 12 && this.same(r, v) ) {
            return this.drainNext(r);
          }
          if ( r.t == 12 ) {
            return this.drainNext(r);
          }
          return this.iterValues(r);
        }
        const gi = this.findMethod(v, "__getitem__");
        if ( (typeof(gi) !== "undefined" && gi != null )  ) {
          let out7 = [];
          let idx = 0;
          while (true) {
            let stop = false;
            let got = this.NONE;
            try {
              got = this.callMethodVal(gi, v, [this.mkInt(idx)]);
            } catch(e) {
              const ex = this.takeException(( e.toString()));
              if ( this.excMatches(ex, this.excClass("IndexError")) ) {
                stop = true;
              } else {
                this.raiseVal(ex);
              }
            }
            if ( stop ) {
              break;
            }
            out7.push(got);
            idx = idx + 1;
          };
          return out7;
        }
        break;
    };
    this.raise("TypeError", ("'" + this.typeName(v)) + "' object is not iterable");
    let none = [];
    return none;
  };
  drainNext (obj) {
    let out = [];
    const nx = this.findMethod(obj, "__next__");
    if ( typeof(nx) === "undefined" ) {
      this.raise("TypeError", ("iter() returned non-iterator of type '" + this.typeName(obj)) + "'");
    }
    const f = nx;
    while (true) {
      let stop = false;
      let got = this.NONE;
      try {
        got = this.callMethodVal(f, obj, []);
      } catch(e) {
        const ex = this.takeException(( e.toString()));
        if ( this.excMatches(ex, this.excClass("StopIteration")) ) {
          stop = true;
        } else {
          this.raiseVal(ex);
        }
      }
      if ( stop ) {
        break;
      }
      out.push(got);
      this.tick();
    };
    return out;
  };
  isSubclass (c, base) {
    if ( this.same(c, base) ) {
      return true;
    }
    // Loop start
    for ( const m of c.vals) {
      if ( this.same(m, base) ) {
        return true;
      }
    }
    return false;
  };
  computeMro (cls) {
    let seqs = [];
    // Loop start
    for ( const b of cls.bases) {
      let l = [];
      l.push(b);
      // Loop start
      for ( const m of b.vals) {
        l.push(m);
      }
      seqs.push(this.mkList(l));
    }
    let bl = [];
    // Loop start
    for ( const b2 of cls.bases) {
      bl.push(b2);
    }
    seqs.push(this.mkList(bl));
    let result = [];
    while (true) {
      let anyLeft = false;
      // Loop start
      for ( const sq of seqs) {
        if ( sq.items.length > 0 ) {
          anyLeft = true;
        }
      }
      if ( anyLeft == false ) {
        break;
      }
      let picked;
      // Loop start
      for ( let si2 = 0; si2 < seqs.length; si2++) {
        var sq2 = seqs[si2];
        if ( sq2.items.length == 0 ) {
          continue;
        }
        const cand = sq2.items[0];
        let inTail = false;
        // Loop start
        for ( const sq3 of seqs) {
          const n = sq3.items.length;
          let t = 1;
          while (t < n) {
            if ( this.same(sq3.items[t], cand) ) {
              inTail = true;
            }
            t = t + 1;
          };
        }
        if ( inTail == false ) {
          picked = cand;
          break;
        }
      }
      if ( typeof(picked) === "undefined" ) {
        this.raise("TypeError", "Cannot create a consistent method resolution order (MRO)");
      }
      const pk = picked;
      result.push(pk);
      // Loop start
      for ( const sq4 of seqs) {
        if ( sq4.items.length > 0 && this.same(sq4.items[0], pk) ) {
          sq4.items.splice(0, 1).pop();
        }
      }
    };
    cls.vals = result;
  };
  classLookup (cls, name) {
    const r = ( Object.prototype.hasOwnProperty.call(cls.attrs, name) ? cls.attrs[name] : undefined );
    if ( (typeof(r) !== "undefined" && r != null )  ) {
      return r;
    }
    // Loop start
    for ( const m of cls.vals) {
      const r2 = ( Object.prototype.hasOwnProperty.call(m.attrs, name) ? m.attrs[name] : undefined );
      if ( (typeof(r2) !== "undefined" && r2 != null )  ) {
        return r2;
      }
    }
    let none;
    return none;
  };
  findMethod (v, name) {
    let none;
    if ( typeof(v.cls) === "undefined" ) {
      return none;
    }
    if ( v.t != 12 ) {
      if ( this.isDataSub(v) == false ) {
        return none;
      }
    }
    const c = v.cls;
    const f = this.classLookup(c, name);
    if ( typeof(f) === "undefined" ) {
      return none;
    }
    const fv = f;
    if ( fv.t == 9 || fv.t == 10 ) {
      return f;
    }
    if ( fv.t == 12 && ((typeof(fv.cls) !== "undefined" && fv.cls != null ) ) ) {
      const dg = this.classLookup(fv.cls, "__get__");
      if ( (typeof(dg) !== "undefined" && dg != null )  ) {
        const got = this.callValue(dg, [fv, v, c], [], []);
        if ( got.t == 12 || got.t == 14 ) {
          const optrdg = got;
          return optrdg;
        }
      }
    }
    return none;
  };
  callMethodVal (f, __self, args) {
    if ( f.t == 12 || f.t == 14 ) {
      let bkn = [];
      let bkv = [];
      return this.callValue(f, args, bkn, bkv);
    }
    let all = [];
    all.push(__self);
    // Loop start
    for ( const a of args) {
      all.push(a);
    }
    let kn = [];
    let kv = [];
    return this.callValue(f, all, kn, kv);
  };
  dunderBin (name, rname, a, b) {
    let none;
    const m = this.findMethod(a, name);
    if ( (typeof(m) !== "undefined" && m != null )  ) {
      const r = this.callMethodVal(m, a, [b]);
      if ( this.same(r, this.NOTIMPL) == false ) {
        const optr1448 = r;
        return optr1448;
      }
    }
    const m2 = this.findMethod(b, rname);
    if ( (typeof(m2) !== "undefined" && m2 != null )  ) {
      const r2 = this.callMethodVal(m2, b, [a]);
      if ( this.same(r2, this.NOTIMPL) == false ) {
        const optr1455 = r2;
        return optr1455;
      }
    }
    return none;
  };
  descMeth (v, name) {
    let none;
    if ( v.t != 12 || (typeof(v.cls) === "undefined") ) {
      return none;
    }
    return this.classLookup(v.cls, name);
  };
  bindValue (v, obj, cls) {
    if ( v.t == 9 ) {
      const b = new PyVal(14);
      b.__self = obj;
      b.cls = v;
      b.s = v.s;
      return b;
    }
    if ( v.t == 19 ) {
      if ( typeof(v.wrapped) === "undefined" ) {
        this.raise("AttributeError", "unreadable attribute");
      }
      return this.callMethodVal(v.wrapped, obj, []);
    }
    if ( v.t == 20 ) {
      return v.wrapped;
    }
    if ( v.t == 21 ) {
      const b2 = new PyVal(14);
      b2.__self = cls;
      b2.cls = v.wrapped;
      b2.s = b2.cls.s;
      return b2;
    }
    return v;
  };
  getAttr (obj, name) {
    const r = this.getAttrOpt(obj, name);
    if ( typeof(r) === "undefined" ) {
      if ( obj.t == 13 ) {
        this.raise("AttributeError", ((("module '" + obj.s) + "' has no attribute '") + name) + "'");
      }
      if ( obj.t == 11 && ((typeof(obj.cls) !== "undefined" && obj.cls != null ) ) ) {
        const mga = this.classLookup(obj.cls, "__getattr__");
        const __REGx1 = this.orNone(mga);
        if ( ((typeof(mga) !== "undefined" && mga != null ) ) && __REGx1.t == 9 ) {
          return this.callMethodVal(mga, obj, [this.mkStr(name)]);
        }
      }
      if ( obj.t == 11 ) {
        this.raise("AttributeError", ((("type object '" + obj.s) + "' has no attribute '") + name) + "'");
      }
      this.raise("AttributeError", ((("'" + this.typeName(obj)) + "' object has no attribute '") + name) + "'");
    }
    return r;
  };
  getAttrOpt (obj, name) {
    let none;
    if ( (obj.t == 21 || obj.t == 20) && ((typeof(obj.wrapped) !== "undefined" && obj.wrapped != null ) ) ) {
      if ( name == "__func__" || name == "__wrapped__" ) {
        return obj.wrapped;
      }
      if ( ( typeof(obj.attrs[name] ) != "undefined" && Object.prototype.hasOwnProperty.call(obj.attrs, name) ) ) {
        return ( Object.prototype.hasOwnProperty.call(obj.attrs, name) ? obj.attrs[name] : undefined );
      }
      if ( ((name == "__isabstractmethod__" || name == "__doc__") || name == "__name__") || name == "__qualname__" ) {
        const wa = this.getAttrOpt(obj.wrapped, name);
        if ( (typeof(wa) === "undefined") && name == "__isabstractmethod__" ) {
          const optrfa = this.FALSE;
          return optrfa;
        }
        return wa;
      }
    }
    if ( (typeof(obj.cls) !== "undefined" && obj.cls != null )  ) {
      if ( this.isDataSub(obj) ) {
        const dsa = this.dataSubAttr(obj, name);
        if ( (typeof(dsa) !== "undefined" && dsa != null )  ) {
          return dsa;
        }
      }
    }
    switch (obj.t ) { 
      case 12 : 
        const c = obj.cls;
        if ( name == "__class__" ) {
          const optr1508 = c;
          return optr1508;
        }
        if ( name == "__dict__" ) {
          if ( (typeof(obj.dictv) !== "undefined" && obj.dictv != null )  ) {
            return obj.dictv;
          }
          const d = this.mkDict();
          const ks = obj.attrKeys();
          // Loop start
          for ( const k of ks) {
            if ( (k.startsWith("__") && k.endsWith("__")) && c.isExc ) {
              continue;
            }
            this.dictSet(
              d,
              this.mkStr(k),
              ( Object.prototype.hasOwnProperty.call(obj.attrs, k) ? obj.attrs[k] : undefined )
            );
          }
          d.owner = obj;
          obj.dictv = d;
          const optr1519 = d;
          return optr1519;
        }
        const cv = this.classLookup(c, name);
        let cget;
        if ( (typeof(cv) !== "undefined" && cv != null )  ) {
          const cvv = cv;
          if ( cvv.t == 19 ) {
            const optr1526 = this.bindValue(cvv, obj, c);
            return optr1526;
          }
          cget = this.descMeth(cvv, "__get__");
          if ( ((typeof(cget) !== "undefined" && cget != null ) ) && (((typeof(this.descMeth(cvv, "__set__")) !== "undefined" && this.descMeth(cvv, "__set__") != null ) ) || ((typeof(this.descMeth(cvv, "__delete__")) !== "undefined" && this.descMeth(cvv, "__delete__") != null ) )) ) {
            const optrdd = this.callValue(cget, [cvv, obj, c], [], []);
            return optrdd;
          }
        }
        const own = ( Object.prototype.hasOwnProperty.call(obj.attrs, name) ? obj.attrs[name] : undefined );
        if ( (typeof(own) !== "undefined" && own != null )  ) {
          return own;
        }
        if ( (typeof(cget) !== "undefined" && cget != null )  ) {
          const optrnd = this.callValue(cget, [cv, obj, c], [], []);
          return optrnd;
        }
        if ( (typeof(cv) !== "undefined" && cv != null )  ) {
          const optr1534 = this.bindValue(cv, obj, c);
          return optr1534;
        }
        if ( name == "__name__" ) {
          return none;
        }
        const ga = this.classLookup(c, "__getattr__");
        if ( ((typeof(ga) !== "undefined" && ga != null ) ) && this.noAttrHook == false ) {
          const optr1541 = this.callMethodVal(ga, obj, [this.mkStr(name)]);
          return optr1541;
        }
        return none;
      case 11 : 
        if ( name == "__name__" ) {
          const optr1547 = this.mkStr(obj.s);
          return optr1547;
        }
        if ( name == "__qualname__" ) {
          if ( ( typeof(obj.attrs["__qualname__"] ) != "undefined" && Object.prototype.hasOwnProperty.call(obj.attrs, "__qualname__") ) ) {
            return ( Object.prototype.hasOwnProperty.call(obj.attrs, "__qualname__") ? obj.attrs["__qualname__"] : undefined );
          }
          const optr1550 = this.mkStr(obj.s);
          return optr1550;
        }
        if ( name == "__mro__" ) {
          let m = [];
          m.push(obj);
          // Loop start
          for ( const mv of obj.vals) {
            m.push(mv);
          }
          if ( obj.bt.length > 0 && obj.bt != "native" ) {
            m.push(( Object.prototype.hasOwnProperty.call(this.builtins, obj.bt) ? this.builtins[obj.bt] : undefined ));
          }
          m.push(( Object.prototype.hasOwnProperty.call(this.builtins, "object") ? this.builtins["object"] : undefined ));
          const optr1558 = this.mkTuple(m);
          return optr1558;
        }
        if ( name == "__bases__" ) {
          if ( (obj.bases.length == 0 && obj.bt.length > 0) && obj.bt != "native" ) {
            const optrbb = this.mkTuple([( Object.prototype.hasOwnProperty.call(this.builtins, obj.bt) ? this.builtins[obj.bt] : undefined )]);
            return optrbb;
          }
          const optr1561 = this.mkTuple(obj.bases);
          return optr1561;
        }
        if ( name == "__dict__" ) {
          const d2 = this.mkDict();
          const ks2 = obj.attrKeys();
          // Loop start
          for ( const k2 of ks2) {
            this.dictSet(
              d2,
              this.mkStr(k2),
              ( Object.prototype.hasOwnProperty.call(obj.attrs, k2) ? obj.attrs[k2] : undefined )
            );
          }
          const optr1569 = d2;
          return optr1569;
        }
        const v = this.classLookup(obj, name);
        if ( (typeof(v) === "undefined") && ((typeof(obj.cls) !== "undefined" && obj.cls != null ) ) ) {
          const mv_1 = this.classLookup(obj.cls, name);
          if ( (typeof(mv_1) !== "undefined" && mv_1 != null )  ) {
            const mvv = mv_1;
            if ( mvv.t == 21 ) {
              const optrmc = this.bindValue(mvv, obj, obj.cls);
              return optrmc;
            }
            const optrmv = this.bindValue(mvv, obj, obj.cls);
            return optrmv;
          }
        }
        if ( (typeof(v) === "undefined") && name == "__class__" ) {
          if ( (typeof(obj.cls) !== "undefined" && obj.cls != null )  ) {
            return obj.cls;
          }
          return ( Object.prototype.hasOwnProperty.call(this.builtins, "type") ? this.builtins["type"] : undefined );
        }
        if ( typeof(v) === "undefined" ) {
          if ( name == "__annotations__" ) {
            const ca = this.mkDict();
            obj.putAttr("__annotations__", ca);
            const optrca = ca;
            return optrca;
          }
          if ( name == "__doc__" ) {
            const optrcd = this.NONE;
            return optrcd;
          }
          if ( name == "__type_params__" ) {
            const optrct = this.mkTuple([]);
            return optrct;
          }
          if ( name == "__init__" && obj.bt.length == 0 ) {
            return ( Object.prototype.hasOwnProperty.call(this.builtins, "__object_init__") ? this.builtins["__object_init__"] : undefined );
          }
          return none;
        }
        const vv = v;
        if ( vv.t == 21 ) {
          const optr1577 = this.bindValue(vv, obj, obj);
          return optr1577;
        }
        const vget = this.descMeth(vv, "__get__");
        if ( (typeof(vget) !== "undefined" && vget != null )  ) {
          const optrcg = this.callValue(vget, [vv, this.NONE, obj], [], []);
          return optrcg;
        }
        if ( vv.t == 20 ) {
          const optr1580 = vv.wrapped;
          return optr1580;
        }
        const optr1582 = vv;
        return optr1582;
      case 13 : 
        const mv_2 = ( Object.prototype.hasOwnProperty.call(obj.attrs, name) ? obj.attrs[name] : undefined );
        if ( typeof(mv_2) === "undefined" ) {
          const mga = ( Object.prototype.hasOwnProperty.call(obj.attrs, "__getattr__") ? obj.attrs["__getattr__"] : undefined );
          if ( ((typeof(mga) !== "undefined" && mga != null ) ) && name != "__getattr__" ) {
            let kn0 = [];
            let kv0 = [];
            const got0 = this.callValue(mga, [this.mkStr(name)], kn0, kv0);
            const r0 = got0;
            return r0;
          }
        }
        return mv_2;
      case 22 : 
        const __self = obj.__self;
        const after = obj.cls;
        let start = __self;
        if ( __self.t == 12 || this.isDataSub(__self) ) {
          start = __self.cls;
        }
        let mro = [];
        mro.push(start);
        // Loop start
        for ( const sv of start.vals) {
          mro.push(sv);
        }
        let found = false;
        // Loop start
        for ( const mc of mro) {
          if ( found ) {
            const got = ( Object.prototype.hasOwnProperty.call(mc.attrs, name) ? mc.attrs[name] : undefined );
            if ( (typeof(got) !== "undefined" && got != null )  ) {
              const gv = got;
              if ( __self.t == 11 ) {
                if ( gv.t == 21 ) {
                  const optr1607 = this.bindValue(gv, __self, __self);
                  return optr1607;
                }
                if ( gv.t == 20 ) {
                  const optr1610 = gv.wrapped;
                  return optr1610;
                }
                const optr1612 = gv;
                return optr1612;
              }
              const optr1614 = this.bindValue(gv, __self, start);
              return optr1614;
            }
          }
          if ( this.same(mc, after) ) {
            found = true;
          }
        }
        if ( this.isDataSub(__self) && name != "__class__" ) {
          const saved = this.rawSelf;
          this.rawSelf = __self;
          const bm = this.meth.attrOf(__self, name);
          this.rawSelf = saved;
          if ( (typeof(bm) !== "undefined" && bm != null )  ) {
            return bm;
          }
        }
        if ( (__self.t == 11 && __self.bt.length > 0) && __self.bt != "native" ) {
          const bty = ( Object.prototype.hasOwnProperty.call(this.builtins, __self.bt) ? this.builtins[__self.bt] : undefined );
          if ( (typeof(bty) !== "undefined" && bty != null )  ) {
            const tam = this.meth.typeAttr(bty, name);
            if ( (typeof(tam) !== "undefined" && tam != null )  ) {
              return tam;
            }
          }
        }
        if ( name == "__init_subclass__" ) {
          const optris = this.mkBuiltinMethod(__self, "__noop__");
          return optris;
        }
        if ( ((((((((((((((((name == "__setattr__" || name == "__delattr__") || name == "__getattribute__") || name == "__repr__") || name == "__str__") || name == "__eq__") || name == "__ne__") || name == "__hash__") || name == "__format__") || name == "__dir__") || name == "__reduce_ex__") || name == "__reduce__") || name == "__sizeof__") || name == "__lt__") || name == "__le__") || name == "__gt__") || name == "__ge__") || name == "__getstate__" ) {
          const optrod = this.mkBuiltinMethod(__self, ("object." + name));
          return optrod;
        }
        if ( name == "__call__" && __self.t == 11 ) {
          const optrtc = this.mkBuiltinMethod(__self, "type.__call__");
          return optrtc;
        }
        if ( name == "__new__" && __self.bt.length == 0 ) {
          const optrsn = this.mkBuiltinMethod(( Object.prototype.hasOwnProperty.call(this.builtins, "object") ? this.builtins["object"] : undefined ), "object.__new__");
          return optrsn;
        }
        if ( name == "__init__" ) {
          const oi = new PyVal(14);
          oi.__self = __self;
          oi.cls = ( Object.prototype.hasOwnProperty.call(this.builtins, "__object_init__") ? this.builtins["__object_init__"] : undefined );
          oi.s = "__init__";
          const optr1623 = oi;
          return optr1623;
        }
        return none;
      case 9 : 
        if ( ( typeof(obj.attrs[name] ) != "undefined" && Object.prototype.hasOwnProperty.call(obj.attrs, name) ) ) {
          return ( Object.prototype.hasOwnProperty.call(obj.attrs, name) ? obj.attrs[name] : undefined );
        }
        return this.funcAttr(obj, name);
      case 14 : 
        if ( name == "__name__" ) {
          const optr1641 = this.mkStr(obj.s);
          return optr1641;
        }
        if ( name == "__self__" ) {
          const optr1644 = obj.__self;
          return optr1644;
        }
        if ( name == "__func__" && ((typeof(obj.cls) !== "undefined" && obj.cls != null ) ) ) {
          return obj.cls;
        }
        if ( (typeof(obj.cls) !== "undefined" && obj.cls != null )  ) {
          const bf = obj.cls;
          if ( bf.t == 9 ) {
            return ( Object.prototype.hasOwnProperty.call(bf.attrs, name) ? bf.attrs[name] : undefined );
          }
        }
        return none;
      case 10 : 
        if ( name == "__name__" || name == "__qualname__" ) {
          const bd = obj.s.lastIndexOf(".");
          const optr1654 = this.mkStr(obj.s.substring(bd + 1, obj.s.length ));
          return optr1654;
        }
        if ( name == "__module__" ) {
          const bmd = obj.s.lastIndexOf(".");
          if ( bmd > 0 ) {
            const optrbm = this.mkStr(obj.s.substring(0, bmd ));
            return optrbm;
          }
          const optrbm2 = this.mkStr("builtins");
          return optrbm2;
        }
        if ( name == "__dict__" && this.isBuiltinTypeName(obj.s) ) {
          const bdd = this.mkDict();
          // Loop start
          for ( let bdi = 0; bdi < this.builtinTypeNames(obj).length; bdi++) {
            var bdn = this.builtinTypeNames(obj)[bdi];
            const bdv = this.meth.typeAttr(obj, bdn);
            if ( (typeof(bdv) !== "undefined" && bdv != null )  ) {
              this.dictSet(bdd, this.mkStr(bdn), bdv);
            }
          }
          const optrbdd = bdd;
          return optrbdd;
        }
        if ( (name == "__mro__" || name == "__bases__") && this.isBuiltinTypeName(obj.s) ) {
          let bm_1 = [];
          if ( name == "__mro__" ) {
            bm_1.push(obj);
          }
          if ( obj.s == "bool" ) {
            bm_1.push(( Object.prototype.hasOwnProperty.call(this.builtins, "int") ? this.builtins["int"] : undefined ));
          }
          if ( obj.s != "object" && (name == "__mro__" || obj.s != "bool") ) {
            bm_1.push(( Object.prototype.hasOwnProperty.call(this.builtins, "object") ? this.builtins["object"] : undefined ));
          }
          const optrbmr = this.mkTuple(bm_1);
          return optrbmr;
        }
        if ( obj.s == "itertools.chain" && name == "from_iterable" ) {
          const cf = new PyVal(10);
          cf.s = "itertools.chain_from_iterable";
          const optr1659 = cf;
          return optr1659;
        }
        if ( name == "__wrapped__" && ((typeof(obj.wrapped) !== "undefined" && obj.wrapped != null ) ) ) {
          return obj.wrapped;
        }
        if ( name == "cache_clear" && ( typeof(obj.attrs["__cache__"] ) != "undefined" && Object.prototype.hasOwnProperty.call(obj.attrs, "__cache__") ) ) {
          const optr1665 = this.mkBuiltinMethod(obj, "lru.cache_clear");
          return optr1665;
        }
        return this.meth.typeAttr(obj, name);
      case 19 : 
        if ( (name == "setter" || name == "getter") || name == "deleter" ) {
          const optr1671 = this.mkBuiltinMethod(obj, ("property." + name));
          return optr1671;
        }
        if ( (name == "__get__" || name == "__set__") || name == "__delete__" ) {
          const optrpg = this.mkBuiltinMethod(obj, ("prop." + name));
          return optrpg;
        }
        if ( name == "fget" ) {
          const optr1674 = this.orNone(obj.wrapped);
          return optr1674;
        }
        if ( name == "fset" ) {
          const optrfs = this.orNone(obj.setter);
          return optrfs;
        }
        if ( name == "fdel" ) {
          const optrfd = this.orNone(( Object.prototype.hasOwnProperty.call(obj.attrs, "__deleter__") ? obj.attrs["__deleter__"] : undefined ));
          return optrfd;
        }
        if ( ( typeof(obj.attrs[name] ) != "undefined" && Object.prototype.hasOwnProperty.call(obj.attrs, name) ) ) {
          return ( Object.prototype.hasOwnProperty.call(obj.attrs, name) ? obj.attrs[name] : undefined );
        }
        if ( name == "__isabstractmethod__" ) {
          if ( (typeof(obj.wrapped) !== "undefined" && obj.wrapped != null )  ) {
            const pia = this.getAttrOpt(obj.wrapped, name);
            if ( (typeof(pia) !== "undefined" && pia != null )  ) {
              return pia;
            }
          }
          const optria = this.FALSE;
          return optria;
        }
        return none;
    };
    if ( obj.t == 6 && ( typeof(obj.attrs["__class__"] ) != "undefined" && Object.prototype.hasOwnProperty.call(obj.attrs, "__class__") ) ) {
      const nt = this.mods.ntAttr(obj, name);
      if ( (typeof(nt) !== "undefined" && nt != null )  ) {
        return nt;
      }
    }
    return this.meth.attrOf(obj, name);
  };
  builtinTypeNames (t) {
    const cands = ["__new__", "__init__", "__repr__", "__str__", "__hash__", "__eq__", "__ne__", "__lt__", "__le__", "__gt__", "__ge__", "__getattribute__", "__setattr__", "__delattr__", "__format__", "__reduce__", "__reduce_ex__", "__sizeof__", "__dir__", "__init_subclass__", "__subclasshook__", "__add__", "__sub__", "__mul__", "__truediv__", "__floordiv__", "__mod__", "__pow__", "__neg__", "__pos__", "__abs__", "__bool__", "__int__", "__float__", "__index__", "__len__", "__getitem__", "__setitem__", "__delitem__", "__contains__", "__iter__", "__reversed__", "__or__", "__and__", "__xor__", "__lshift__", "__rshift__", "__invert__", "__round__", "__trunc__", "__floor__", "__ceil__", "__getnewargs__", "append", "extend", "insert", "remove", "pop", "clear", "index", "count", "sort", "reverse", "copy", "keys", "values", "items", "get", "setdefault", "update", "popitem", "fromkeys", "add", "discard", "union", "intersection", "difference", "symmetric_difference", "issubset", "issuperset", "isdisjoint", "join", "split", "rsplit", "strip", "lstrip", "rstrip", "replace", "find", "rfind", "rindex", "startswith", "endswith", "upper", "lower", "title", "capitalize", "casefold", "swapcase", "center", "ljust", "rjust", "zfill", "format", "format_map", "partition", "rpartition", "splitlines", "expandtabs", "encode", "isdigit", "isalpha", "isalnum", "isspace", "isupper", "islower", "istitle", "isidentifier", "isprintable", "isdecimal", "isnumeric", "isascii", "maketrans", "translate", "removeprefix", "removesuffix", "bit_length", "bit_count", "to_bytes", "from_bytes", "as_integer_ratio", "conjugate", "is_integer", "hex", "fromhex", "real", "imag", "numerator", "denominator", "mro"];
    let out = [];
    let owned = {};
    if ( t.s != "object" ) {
      const o = ( Object.prototype.hasOwnProperty.call(this.builtins, "object") ? this.builtins["object"] : undefined );
      // Loop start
      for ( const c0 of cands) {
        if ( ((typeof(o) !== "undefined" && o != null ) ) && ((typeof(this.meth.typeAttr(o, c0)) !== "undefined" && this.meth.typeAttr(o, c0) != null ) ) ) {
          owned[c0] = true;
        }
      }
    }
    // Loop start
    for ( const c of cands) {
      if ( (((((( typeof(owned[c] ) != "undefined" && Object.prototype.hasOwnProperty.call(owned, c) ) && c != "__new__") && c != "__init__") && c != "__repr__") && c != "__hash__") && c != "__eq__") && c != "__str__" ) {
        continue;
      }
      out.push(c);
    }
    return out;
  };
  isBuiltinTypeName (n) {
    return ((((((((((((n == "int" || n == "str") || n == "float") || n == "bool") || n == "list") || n == "dict") || n == "set") || n == "frozenset") || n == "tuple") || n == "complex") || n == "object") || n == "type") || n == "bytes") || n == "bytearray";
  };
  setAttr (obj, name, val) {
    if ( obj.t == 12 ) {
      const c0 = obj.cls;
      const sa = this.classLookup(c0, "__setattr__");
      const __REGx1 = this.orNone(sa);
      if ( ((typeof(sa) !== "undefined" && sa != null ) ) && __REGx1.t == 9 ) {
        this.callMethodVal(sa, obj, [this.mkStr(name), val]);
        return;
      }
    }
    this.objectSetAttr(obj, name, val);
  };
  slotsOf (c) {
    if ( c.slotsKnown == 0 ) {
      let limited = true;
      let names = {};
      let chain = [];
      chain.push(c);
      // Loop start
      for ( const mc of c.vals) {
        chain.push(mc);
      }
      // Loop start
      for ( const k of chain) {
        if ( k.mode == "builtin" || k.mode == "native" ) {
          continue;
        }
        const sl = ( Object.prototype.hasOwnProperty.call(k.attrs, "__slots__") ? k.attrs["__slots__"] : undefined );
        if ( typeof(sl) === "undefined" ) {
          limited = false;
        } else {
          const slv = sl;
          let sitems = [];
          if ( slv.t == 4 ) {
            sitems.push(slv);
          } else {
            sitems = this.iterValues(slv);
          }
          // Loop start
          for ( const si of sitems) {
            const sn = this.toStr(si);
            if ( sn == "__dict__" ) {
              limited = false;
            }
            names[sn] = true;
          }
        }
      }
      if ( c.bt.length > 0 ) {
        limited = false;
      }
      c.slotNames = names;
      c.slotsKnown = 1;
      if ( limited ) {
        c.slotsKnown = 2;
      }
    }
    return c;
  };
  objectSetAttr (obj, name, val) {
    if ( obj.t == 12 ) {
      const c = obj.cls;
      const cv = this.classLookup(c, name);
      if ( (typeof(cv) !== "undefined" && cv != null )  ) {
        const cvv = cv;
        if ( cvv.t == 19 ) {
          if ( typeof(cvv.setter) === "undefined" ) {
            this.raise("AttributeError", ((("property '" + name) + "' of '") + c.s) + "' object has no setter");
          }
          this.callMethodVal(cvv.setter, obj, [val]);
          return;
        }
        const dset = this.descMeth(cvv, "__set__");
        if ( (typeof(dset) !== "undefined" && dset != null )  ) {
          this.callValue(dset, [cvv, obj, val], [], []);
          return;
        }
      }
      const sc = this.slotsOf(c);
      if ( (sc.slotsKnown == 2 && ( typeof(sc.slotNames[name] ) != "undefined" && Object.prototype.hasOwnProperty.call(sc.slotNames, name) ) == false) && c.isExc == false ) {
        if ( (typeof(cv) !== "undefined" && cv != null )  ) {
          this.raise("AttributeError", ((("'" + c.s) + "' object attribute '") + name) + "' is read-only");
        }
        this.raise("AttributeError", ((("'" + c.s) + "' object has no attribute '") + name) + "' and no __dict__ for setting new attributes");
      }
      obj.putAttr(name, val);
      return;
    }
    if ( (obj.t == 11 || obj.t == 13) || obj.t == 9 ) {
      obj.putAttr(name, val);
      return;
    }
    if ( this.isDataSub(obj) ) {
      const dc = obj.cls;
      const dcv = this.classLookup(dc, name);
      if ( (typeof(dcv) !== "undefined" && dcv != null )  ) {
        const dcvv = dcv;
        if ( dcvv.t == 19 && ((typeof(dcvv.setter) !== "undefined" && dcvv.setter != null ) ) ) {
          this.callMethodVal(dcvv.setter, obj, [val]);
          return;
        }
      }
      obj.putAttr(name, val);
      return;
    }
    this.raise("AttributeError", ((("'" + this.typeName(obj)) + "' object has no attribute '") + name) + "'");
  };
  setAttrRaw (obj, name, val) {
    obj.putAttr(name, val);
  };
  delAttr (obj, name) {
    if ( obj.t == 12 ) {
      const da0 = this.classLookup(obj.cls, "__delattr__");
      const __REGx1 = this.orNone(da0);
      if ( ((typeof(da0) !== "undefined" && da0 != null ) ) && __REGx1.t == 9 ) {
        this.callMethodVal(da0, obj, [this.mkStr(name)]);
        return;
      }
    }
    this.objectDelAttr(obj, name);
  };
  objectDelAttr (obj, name) {
    if ( obj.t == 12 ) {
      const dc = obj.cls;
      const dcv = this.classLookup(dc, name);
      if ( (typeof(dcv) !== "undefined" && dcv != null )  ) {
        const dcvv = dcv;
        if ( dcvv.t == 19 ) {
          const pdel = ( Object.prototype.hasOwnProperty.call(dcvv.attrs, "__deleter__") ? dcvv.attrs["__deleter__"] : undefined );
          if ( typeof(pdel) === "undefined" ) {
            this.raise("AttributeError", ((("property '" + name) + "' of '") + dc.s) + "' object has no deleter");
          }
          this.callMethodVal(pdel, obj, []);
          return;
        }
        const ddel = this.descMeth(dcvv, "__delete__");
        if ( (typeof(ddel) !== "undefined" && ddel != null )  ) {
          this.callValue(ddel, [dcvv, obj], [], []);
          return;
        }
      }
      if ( ( typeof(obj.attrs[name] ) != "undefined" && Object.prototype.hasOwnProperty.call(obj.attrs, name) ) ) {
        obj.delAttrKey(name);
        return;
      }
      this.raise("AttributeError", ((("'" + dc.s) + "' object has no attribute '") + name) + "'");
    }
    if ( ((obj.t == 12 || obj.t == 11) || obj.t == 13) && ( typeof(obj.attrs[name] ) != "undefined" && Object.prototype.hasOwnProperty.call(obj.attrs, name) ) ) {
      obj.delAttrKey(name);
      return;
    }
    this.raise("AttributeError", name);
  };
  normIndex (i, n, what) {
    let k = i;
    if ( k < 0 ) {
      k = k + n;
    }
    if ( k < 0 || k >= n ) {
      this.raise("IndexError", what + " index out of range");
    }
    return k;
  };
  sliceInt (v) {
    if ( this.isIntLike(v) ) {
      return v.i;
    }
    const ix = this.findMethod(v, "__index__");
    if ( (typeof(ix) !== "undefined" && ix != null )  ) {
      const r = this.callMethodVal(ix, v, []);
      return r.i;
    }
    this.raise("TypeError", "slice indices must be integers or None or have an __index__ method");
    return 0;
  };
  sliceBounds (sl, n) {
    const lo = sl.items[0];
    const hi = sl.items[1];
    const st = sl.items[2];
    let step = 1;
    if ( st.t != 0 ) {
      step = this.sliceInt(st);
      if ( step == 0 ) {
        this.raise("ValueError", "slice step cannot be zero");
      }
    }
    let start = 0;
    let stop = 0;
    if ( step > 0 ) {
      start = 0;
      stop = n;
    } else {
      start = n - 1;
      stop = -1;
    }
    if ( lo.t != 0 ) {
      start = this.sliceInt(lo);
      if ( start < 0 ) {
        start = start + n;
        if ( start < 0 ) {
          if ( step < 0 ) {
            start = -1;
          } else {
            start = 0;
          }
        }
      } else {
        if ( start >= n ) {
          if ( step < 0 ) {
            start = n - 1;
          } else {
            start = n;
          }
        }
      }
    }
    if ( hi.t != 0 ) {
      stop = this.sliceInt(hi);
      if ( stop < 0 ) {
        stop = stop + n;
        if ( stop < 0 ) {
          if ( step < 0 ) {
            stop = -1;
          } else {
            stop = 0;
          }
        }
      } else {
        if ( stop >= n ) {
          if ( step < 0 ) {
            stop = n - 1;
          } else {
            stop = n;
          }
        }
      }
    }
    let r = [];
    r.push(start);
    r.push(stop);
    r.push(step);
    return r;
  };
  slicePositions (sl, n) {
    const b = this.sliceBounds(sl, n);
    const start = b[0];
    const stop = b[1];
    const step = b[2];
    let out = [];
    let i = start;
    if ( step > 0 ) {
      while (i < stop) {
        out.push(i);
        i = i + step;
      };
    } else {
      while (i > stop) {
        out.push(i);
        i = i + step;
      };
    }
    return out;
  };
  getItem (obj, idx) {
    if ( (typeof(obj.cls) !== "undefined" && obj.cls != null )  ) {
      const sg = this.subDunder(obj, "__getitem__", [idx]);
      if ( (typeof(sg) !== "undefined" && sg != null )  ) {
        return sg;
      }
      if ( (obj.t == 7 && this.isDataSub(obj)) && (typeof(this.dictGet(obj, idx)) === "undefined") ) {
        const smi = this.subDunder(obj, "__missing__", [idx]);
        if ( (typeof(smi) !== "undefined" && smi != null )  ) {
          return smi;
        }
      }
    }
    switch (obj.t ) { 
      case 5 : 
        if ( idx.t == 18 ) {
          const ps = this.slicePositions(idx, obj.items.length);
          let out = [];
          // Loop start
          for ( const p of ps) {
            out.push(obj.items[p]);
          }
          return this.mkList(out);
        }
        const k = this.normIndex(
          this.indexInt(idx, "list"),
          obj.items.length,
          "list"
        );
        return obj.items[k];
      case 6 : 
        if ( idx.t == 18 ) {
          const ps2 = this.slicePositions(idx, obj.items.length);
          let out2 = [];
          // Loop start
          for ( const p2 of ps2) {
            out2.push(obj.items[p2]);
          }
          return this.mkTuple(out2);
        }
        const k2 = this.normIndex(
          this.indexInt(idx, "tuple"),
          obj.items.length,
          "tuple"
        );
        return obj.items[k2];
      case 4 : 
        const cs = obj.codes();
        const n = cs.length;
        if ( idx.t == 18 ) {
          const b = this.sliceBounds(idx, n);
          if ( b[2] == 1 ) {
            if ( b[0] >= b[1] ) {
              return this.mkStr("");
            }
            return this.mkStr(py_rt_codes(cs, b[0], b[1]));
          }
          const ps3 = this.slicePositions(idx, n);
          let outc = [];
          // Loop start
          for ( const p3 of ps3) {
            outc.push(cs[p3]);
          }
          return this.mkStr(py_rt_codes(outc, 0, outc.length));
        }
        const k3 = this.normIndex(this.indexInt(idx, "string"), n, "string");
        return this.mkStr(String.fromCodePoint(cs[k3]));
      case 7 : 
        const r = this.dictGet(obj, idx);
        if ( typeof(r) === "undefined" ) {
          const dm = ( Object.prototype.hasOwnProperty.call(obj.attrs, "__missing__") ? obj.attrs["__missing__"] : undefined );
          if ( (typeof(dm) !== "undefined" && dm != null )  ) {
            let kn = [];
            let kv = [];
            const made = this.callValue(dm, [], kn, kv);
            this.dictSet(obj, idx, made);
            return made;
          }
          if ( obj.mode == "counter" ) {
            return this.mkInt(0);
          }
          this.raiseVal(this.makeExcArg("KeyError", idx));
        }
        return r;
      case 15 : 
        if ( idx.t == 18 ) {
          const b2 = this.sliceBounds(idx, this.rangeLen(obj));
          const rr = new PyVal(15);
          rr.i = obj.i + b2[0] * obj.i3;
          rr.i2 = obj.i + b2[1] * obj.i3;
          rr.i3 = obj.i3 * b2[2];
          return rr;
        }
        const k4 = this.normIndex(
          this.indexInt(idx, "range"),
          this.rangeLen(obj),
          "range object"
        );
        return this.mkInt((obj.i + k4 * obj.i3));
      case 12 : 
        const m = this.findMethod(obj, "__getitem__");
        if ( (typeof(m) !== "undefined" && m != null )  ) {
          return this.callMethodVal(m, obj, [idx]);
        }
        break;
      case 11 : 
        const cg = this.classLookup(obj, "__class_getitem__");
        if ( (typeof(cg) !== "undefined" && cg != null )  ) {
          let cgf = cg;
          if ( cgf.t == 21 || cgf.t == 20 ) {
            cgf = cgf.wrapped;
          }
          let kn_1 = [];
          let kv_1 = [];
          return this.callValue(cgf, [obj, idx], kn_1, kv_1);
        }
        if ( obj.mode == "native" ) {
          return this.genericAlias(obj, idx);
        }
        break;
      case 10 : 
        if ( this.isTypeBuiltin(obj.s) ) {
          return this.genericAlias(obj, idx);
        }
        break;
    };
    this.raise("TypeError", ("'" + this.typeName(obj)) + "' object is not subscriptable");
    return this.NONE;
  };
  genericAlias (origin, args) {
    const ga = this.getAttr(this.importModule("types"), "GenericAlias");
    let kn = [];
    let kv = [];
    return this.callValue(ga, [origin, args], kn, kv);
  };
  makeExcArg (name, arg) {
    const e = this.makeExc(this.excClass(name), "");
    e.putAttr("args", this.mkTuple([arg]));
    return e;
  };
  indexInt (idx, what) {
    if ( this.isIntLike(idx) ) {
      return idx.i;
    }
    const ix = this.findMethod(idx, "__index__");
    if ( (typeof(ix) !== "undefined" && ix != null )  ) {
      const r = this.callMethodVal(ix, idx, []);
      return r.i;
    }
    this.raise("TypeError", (what + " indices must be integers or slices, not ") + this.typeName(idx));
    return 0;
  };
  setItem (obj, idx, val) {
    if ( (typeof(obj.cls) !== "undefined" && obj.cls != null )  ) {
      const ssi = this.subDunder(obj, "__setitem__", [idx, val]);
      if ( (typeof(ssi) !== "undefined" && ssi != null )  ) {
        return;
      }
    }
    switch (obj.t ) { 
      case 5 : 
        if ( idx.t == 18 ) {
          const vals = this.iterValues(val);
          const n = obj.items.length;
          const b = this.sliceBounds(idx, n);
          if ( b[2] == 1 ) {
            const start = b[0];
            let stop = b[1];
            if ( stop < start ) {
              stop = start;
            }
            let out = [];
            let i = 0;
            while (i < start) {
              out.push(obj.items[i]);
              i = i + 1;
            };
            // Loop start
            for ( const v of vals) {
              out.push(v);
            }
            i = stop;
            while (i < n) {
              out.push(obj.items[i]);
              i = i + 1;
            };
            obj.items = out;
            return;
          }
          const ps = this.slicePositions(idx, n);
          if ( ps.length != vals.length ) {
            this.raise("ValueError", (("attempt to assign sequence of size " + ((vals.length).toString())) + " to extended slice of size ") + ((ps.length).toString()));
          }
          // Loop start
          for ( let k = 0; k < ps.length; k++) {
            var p = ps[k];
            obj.items[p] = vals[k];
          }
          return;
        }
        const k2 = this.normIndex(
          this.indexInt(idx, "list"),
          obj.items.length,
          "list assignment"
        );
        obj.items[k2] = val;
        return;
      case 7 : 
        this.dictSet(obj, idx, val);
        return;
      case 12 : 
        const m = this.findMethod(obj, "__setitem__");
        if ( (typeof(m) !== "undefined" && m != null )  ) {
          this.callMethodVal(m, obj, [idx, val]);
          return;
        }
        break;
    };
    this.raise("TypeError", ("'" + this.typeName(obj)) + "' object does not support item assignment");
  };
  delItem (obj, idx) {
    if ( (typeof(obj.cls) !== "undefined" && obj.cls != null )  ) {
      const sdi = this.subDunder(obj, "__delitem__", [idx]);
      if ( (typeof(sdi) !== "undefined" && sdi != null )  ) {
        return;
      }
    }
    if ( obj.t == 5 ) {
      if ( idx.t == 18 ) {
        const ps = this.slicePositions(idx, obj.items.length);
        let drop = {};
        // Loop start
        for ( const p of ps) {
          drop[((p).toString())] = true;
        }
        let out = [];
        // Loop start
        for ( let j = 0; j < obj.items.length; j++) {
          var it = obj.items[j];
          if ( ( typeof(drop[((j).toString())] ) != "undefined" && Object.prototype.hasOwnProperty.call(drop, ((j).toString())) ) == false ) {
            out.push(it);
          }
        }
        obj.items = out;
        return;
      }
      const k = this.normIndex(
        this.indexInt(idx, "list"),
        obj.items.length,
        "list assignment"
      );
      obj.items.splice(k, 1).pop();
      return;
    }
    if ( obj.t == 7 ) {
      if ( this.dictDel(obj, idx) == false ) {
        this.raiseVal(this.makeExcArg("KeyError", idx));
      }
      return;
    }
    if ( obj.t == 12 ) {
      const m = this.findMethod(obj, "__delitem__");
      if ( (typeof(m) !== "undefined" && m != null )  ) {
        this.callMethodVal(m, obj, [idx]);
        return;
      }
    }
    this.raise("TypeError", ("'" + this.typeName(obj)) + "' object does not support item deletion");
  };
  intPow (base, ex) {
    if ( ex < 0 ) {
      return this.mkFloat(Math.pow((base), (ex)));
    }
    let r = 1;
    let b = base;
    let e = ex;
    const big = false;
    const approx = 1.0;
    while (e > 0) {
      if ( PyFormat.floorModInt(e, 2) == 1 ) {
        r = r * b;
      }
      e = PyFormat.floorDivInt(e, 2);
      if ( e > 0 ) {
        b = b * b;
      }
    };
    const check = Math.pow((base), (ex));
    if ( check > 9007199254740991.0 || check < -9007199254740991.0 ) {
      return this.mkFloat(check);
    }
    return this.mkInt(r);
  };
  mkComplex (re, im) {
    const v = new PyVal(24);
    v.d = re;
    v.d2 = im;
    return v;
  };
  isComplexNum (v) {
    return this.isNum(v) || v.t == 24;
  };
  reOf (v) {
    if ( v.t == 24 ) {
      return v.d;
    }
    return this.toDouble(v);
  };
  imOf (v) {
    if ( v.t == 24 ) {
      return v.d2;
    }
    return 0.0;
  };
  cQuot (ar, ai, br, bi) {
    const abr = PyInterp.fabs(br);
    const abi = PyInterp.fabs(bi);
    if ( abr >= abi ) {
      if ( abr == 0.0 ) {
        this.raise("ZeroDivisionError", "complex division by zero");
      }
      const ratio = bi / br;
      const denom = br + bi * ratio;
      return this.mkComplex(((ar + ai * ratio) / denom), ((ai - ar * ratio) / denom));
    }
    if ( abi >= abr ) {
      const ratio2 = br / bi;
      const denom2 = br * ratio2 + bi;
      return this.mkComplex(((ar * ratio2 + ai) / denom2), ((ai * ratio2 - ar) / denom2));
    }
    return this.mkComplex(NaN, NaN);
  };
  cPow (ar, ai, br, bi) {
    if ( (bi == 0.0 && Math.floor(br) == br) && PyInterp.fabs(br) <= 100.0 ) {
      let n = Math.trunc(br);
      const neg = n < 0;
      if ( neg ) {
        n = 0 - n;
      }
      let rr = 1.0;
      let ri = 0.0;
      let pr = ar;
      let pi = ai;
      let mask = 1;
      while (mask > 0 && n >= mask) {
        if ( ((n & mask) >>> 0) != 0 ) {
          const tr = rr * pr - ri * pi;
          ri = rr * pi + ri * pr;
          rr = tr;
        }
        mask = mask * 2;
        const qr = pr * pr - pi * pi;
        pi = pr * pi + pi * pr;
        pr = qr;
      };
      if ( neg ) {
        return this.cQuot(1.0, 0.0, rr, ri);
      }
      return this.mkComplex(rr, ri);
    }
    if ( br == 0.0 && bi == 0.0 ) {
      return this.mkComplex(1.0, 0.0);
    }
    if ( ar == 0.0 && ai == 0.0 ) {
      if ( bi != 0.0 || br < 0.0 ) {
        this.raise("ZeroDivisionError", "zero to a negative or complex power");
      }
      return this.mkComplex(0.0, 0.0);
    }
    const vabs = PyInterp.hypot(ar, ai);
    let __len = Math.pow(vabs, br);
    const at = Math.atan2(ai, ar);
    let phase = at * br;
    if ( bi != 0.0 ) {
      __len = __len / Math.exp(at * bi);
      phase = phase + bi * Math.log(vabs);
    }
    return this.mkComplex((__len * Math.cos(phase)), (__len * Math.sin(phase)));
  };
  complexBin (op, a, b) {
    const ar = this.reOf(a);
    const ai = this.imOf(a);
    const br = this.reOf(b);
    const bi_1 = this.imOf(b);
    if ( op == "+" ) {
      return this.mkComplex((ar + br), (ai + bi_1));
    }
    if ( op == "-" ) {
      return this.mkComplex((ar - br), (ai - bi_1));
    }
    if ( op == "*" ) {
      return this.mkComplex((ar * br - ai * bi_1), (ar * bi_1 + ai * br));
    }
    if ( op == "/" ) {
      return this.cQuot(ar, ai, br, bi_1);
    }
    if ( op == "**" ) {
      return this.cPow(ar, ai, br, bi_1);
    }
    this.raise("TypeError", ((((("unsupported operand type(s) for " + op) + ": '") + this.typeName(a)) + "' and '") + this.typeName(b)) + "'");
    return this.NONE;
  };
  complexRepr (v) {
    let im = PyInterp.cPart(v.d2);
    if ( v.d == 0.0 && PyFormat.isNegZero(v.d) == false ) {
      return im + "j";
    }
    const re = PyInterp.cPart(v.d);
    let sign = "+";
    if ( (v.d2 < 0.0 || v.d2 == 0.0 && PyFormat.isNegZero(v.d2)) || Number.isNaN(v.d2) && false ) {
      sign = "-";
      im = PyInterp.cPart((0.0 - v.d2));
      if ( v.d2 == 0.0 ) {
        im = "0";
      }
    }
    return ((("(" + re) + sign) + im) + "j)";
  };
  binop (op, a, b) {
    if ( ((a.t == 3 && b.t == 3) && (typeof(a.cls) === "undefined")) && (typeof(b.cls) === "undefined") ) {
      if ( op == "*" ) {
        return this.mkFloat((a.d * b.d));
      }
      if ( op == "+" ) {
        return this.mkFloat((a.d + b.d));
      }
      if ( op == "-" ) {
        return this.mkFloat((a.d - b.d));
      }
    }
    if ( ((a.t == 2 && b.t == 2) && (typeof(a.cls) === "undefined")) && (typeof(b.cls) === "undefined") ) {
      if ( op == "+" ) {
        return this.mkInt((a.i + b.i));
      }
      if ( op == "-" ) {
        return this.mkInt((a.i - b.i));
      }
    }
    if ( ((typeof(a.cls) !== "undefined" && a.cls != null ) ) || ((typeof(b.cls) !== "undefined" && b.cls != null ) ) ) {
      if ( this.isDataSub(a) || this.isDataSub(b) ) {
        const bnm = this.dunderName(op);
        const sbo = this.dunderBin(
          (("__" + bnm) + "__"),
          (("__r" + bnm) + "__"),
          a,
          b
        );
        if ( (typeof(sbo) !== "undefined" && sbo != null )  ) {
          return sbo;
        }
      }
    }
    if ( this.isIntLike(a) && this.isIntLike(b) ) {
      const x = a.i;
      const y = b.i;
      if ( op == "+" ) {
        return this.mkInt((x + y));
      }
      if ( op == "-" ) {
        return this.mkInt((x - y));
      }
      if ( op == "*" ) {
        return this.mkInt((x * y));
      }
      if ( op == "/" ) {
        if ( y == 0 ) {
          this.raise("ZeroDivisionError", "division by zero");
        }
        return this.mkFloat(((x) / (y)));
      }
      if ( op == "//" ) {
        if ( y == 0 ) {
          this.raise("ZeroDivisionError", "integer division or modulo by zero");
        }
        return this.mkInt(PyFormat.floorDivInt(x, y));
      }
      if ( op == "%" ) {
        if ( y == 0 ) {
          this.raise("ZeroDivisionError", "integer modulo by zero");
        }
        return this.mkInt(PyFormat.floorModInt(x, y));
      }
      if ( op == "**" ) {
        return this.intPow(x, y);
      }
      if ( op == "<<" ) {
        if ( y < 0 ) {
          this.raise("ValueError", "negative shift count");
        }
        return this.intPow2(x, y);
      }
      if ( op == ">>" ) {
        if ( y < 0 ) {
          this.raise("ValueError", "negative shift count");
        }
        const dd = Math.pow(2.0, (y));
        return this.mkInt(Math.trunc(Math.floor((x) / dd)));
      }
      if ( (op == "&" || op == "|") || op == "^" ) {
        const r = PyInterp.bitOp(op, x, y);
        if ( a.t == 1 && b.t == 1 ) {
          return this.mkBool((r != 0));
        }
        return this.mkInt(r);
      }
    }
    if ( ((a.t == 24 || b.t == 24) && this.isComplexNum(a)) && this.isComplexNum(b) ) {
      return this.complexBin(op, a, b);
    }
    if ( this.isNum(a) && this.isNum(b) ) {
      const fx = this.toDouble(a);
      const fy = this.toDouble(b);
      if ( op == "+" ) {
        return this.mkFloat((fx + fy));
      }
      if ( op == "-" ) {
        return this.mkFloat((fx - fy));
      }
      if ( op == "*" ) {
        return this.mkFloat((fx * fy));
      }
      if ( op == "/" ) {
        if ( fy == 0.0 ) {
          this.raise("ZeroDivisionError", "float division by zero");
        }
        return this.mkFloat((fx / fy));
      }
      if ( op == "//" ) {
        if ( fy == 0.0 ) {
          this.raise("ZeroDivisionError", "float floor division by zero");
        }
        return this.mkFloat(Math.floor(fx / fy));
      }
      if ( op == "%" ) {
        if ( fy == 0.0 ) {
          this.raise("ZeroDivisionError", "float modulo");
        }
        const q = Math.floor(fx / fy);
        let m = fx - fy * q;
        if ( m != 0.0 && (m < 0.0) != (fy < 0.0) ) {
          m = m + fy;
        }
        return this.mkFloat(m);
      }
      if ( op == "**" ) {
        if ( fx == 0.0 && fy < 0.0 ) {
          this.raise("ZeroDivisionError", "0.0 cannot be raised to a negative power");
        }
        if ( (fx < 0.0 && Math.floor(fy) != fy) && (Math.abs(fy) === Infinity) == false ) {
          return this.cPow(fx, 0.0, fy, 0.0);
        }
        return this.mkFloat(Math.pow(fx, fy));
      }
    }
    if ( op == "+" ) {
      if ( a.t == 4 && b.t == 4 ) {
        return this.mkStr((a.s + b.s));
      }
      if ( a.t == 5 && b.t == 5 ) {
        let out = [];
        // Loop start
        for ( const it of a.items) {
          out.push(it);
        }
        // Loop start
        for ( const it2 of b.items) {
          out.push(it2);
        }
        return this.mkList(out);
      }
      if ( a.t == 6 && b.t == 6 ) {
        let out2 = [];
        // Loop start
        for ( const it3 of a.items) {
          out2.push(it3);
        }
        // Loop start
        for ( const it4 of b.items) {
          out2.push(it4);
        }
        return this.mkTuple(out2);
      }
    }
    if ( op == "*" ) {
      if ( ((a.t == 4 || a.t == 5) || a.t == 6) && this.isIntLike(b) ) {
        return this.repeatSeq(a, b.i);
      }
      if ( ((b.t == 4 || b.t == 5) || b.t == 6) && this.isIntLike(a) ) {
        return this.repeatSeq(b, a.i);
      }
    }
    if ( op == "%" && a.t == 4 ) {
      return this.mkStr(this.bi.percentFormat(a.s, b));
    }
    if ( a.t == 8 && b.t == 8 ) {
      if ( ((op == "|" || op == "&") || op == "-") || op == "^" ) {
        return this.meth.setOp(op, a, b);
      }
    }
    if ( (op == "|" && ((a.t == 10 && this.isTypeBuiltin(a.s) || a.t == 11) || a.t == 0)) && ((b.t == 10 && this.isTypeBuiltin(b.s) || b.t == 11) || b.t == 0) ) {
      const ut = this.getAttr(this.importModule("types"), "UnionType");
      let ukn = [];
      let ukv = [];
      return this.callValue(ut, [this.mkTuple([a, b])], ukn, ukv);
    }
    if ( (a.t == 7 && b.t == 7) && op == "|" ) {
      const d = this.copyDict(a);
      // Loop start
      for ( let ki = 0; ki < b.items.length; ki++) {
        var k = b.items[ki];
        this.dictSet(d, k, b.vals[ki]);
      }
      return d;
    }
    if ( a.t == 12 || b.t == 12 ) {
      const nm = this.dunderName(op);
      const r2 = this.dunderBin(
        (("__" + nm) + "__"),
        (("__r" + nm) + "__"),
        a,
        b
      );
      if ( (typeof(r2) !== "undefined" && r2 != null )  ) {
        return r2;
      }
    }
    if ( op == "+" && a.t == 4 ) {
      this.raise("TypeError", ("can only concatenate str (not \"" + this.typeName(b)) + "\") to str");
    }
    if ( (op == "+" && a.t == 5) && this.typeName(a) == "list" ) {
      this.raise("TypeError", ("can only concatenate list (not \"" + this.typeName(b)) + "\") to list");
    }
    this.raise("TypeError", ((((("unsupported operand type(s) for " + op) + ": '") + this.typeName(a)) + "' and '") + this.typeName(b)) + "'");
    return this.NONE;
  };
  intPow2 (x, y) {
    const p = Math.pow(2.0, (y));
    const r = (x) * p;
    if ( r > 9007199254740991.0 || r < -9007199254740991.0 ) {
      return this.mkFloat(r);
    }
    return this.mkInt(Math.trunc(r));
  };
  dunderName (op) {
    if ( op == "+" ) {
      return "add";
    }
    if ( op == "-" ) {
      return "sub";
    }
    if ( op == "*" ) {
      return "mul";
    }
    if ( op == "/" ) {
      return "truediv";
    }
    if ( op == "//" ) {
      return "floordiv";
    }
    if ( op == "%" ) {
      return "mod";
    }
    if ( op == "**" ) {
      return "pow";
    }
    if ( op == "@" ) {
      return "matmul";
    }
    if ( op == "&" ) {
      return "and";
    }
    if ( op == "|" ) {
      return "or";
    }
    if ( op == "^" ) {
      return "xor";
    }
    if ( op == "<<" ) {
      return "lshift";
    }
    if ( op == ">>" ) {
      return "rshift";
    }
    return "op";
  };
  repeatSeq (seq, n) {
    if ( seq.t == 4 ) {
      let parts = [];
      let i = 0;
      while (i < n) {
        parts.push(seq.s);
        i = i + 1;
      };
      return this.mkStr(parts.join(""));
    }
    let out = [];
    let k = 0;
    while (k < n) {
      // Loop start
      for ( const it of seq.items) {
        out.push(it);
      }
      k = k + 1;
    };
    if ( seq.t == 6 ) {
      return this.mkTuple(out);
    }
    return this.mkList(out);
  };
  inplace (op, a, b) {
    if ( a.t == 5 && op == "+" ) {
      const vals = this.iterValues(b);
      // Loop start
      for ( const v of vals) {
        a.items.push(v);
      }
      return a;
    }
    if ( (a.t == 5 && op == "*") && this.isIntLike(b) ) {
      const r = this.repeatSeq(a, b.i);
      a.items = r.items;
      return a;
    }
    if ( (a.t == 8 && b.t == 8) && (((op == "|" || op == "&") || op == "-") || op == "^") ) {
      const r2 = this.meth.setOp(op, a, b);
      a.items = r2.items;
      a.index = r2.index;
      return a;
    }
    if ( (a.t == 7 && b.t == 7) && op == "|" ) {
      // Loop start
      for ( let ki = 0; ki < b.items.length; ki++) {
        var k = b.items[ki];
        this.dictSet(a, k, b.vals[ki]);
      }
      return a;
    }
    if ( a.t == 12 ) {
      const m = this.findMethod(a, (("__i" + this.dunderName(op)) + "__"));
      if ( (typeof(m) !== "undefined" && m != null )  ) {
        return this.callMethodVal(m, a, [b]);
      }
    }
    return this.binop(op, a, b);
  };
  unary (op, a) {
    if ( (typeof(a.cls) !== "undefined" && a.cls != null )  ) {
      let un = "__invert__";
      if ( op == "-" ) {
        un = "__neg__";
      }
      if ( op == "+" ) {
        un = "__pos__";
      }
      const su = this.subDunder(a, un, []);
      if ( (typeof(su) !== "undefined" && su != null )  ) {
        return su;
      }
    }
    if ( op == "-" ) {
      if ( this.isIntLike(a) ) {
        return this.mkInt((0 - a.i));
      }
      if ( a.t == 3 ) {
        return this.mkFloat((a.d * -1.0));
      }
      if ( a.t == 24 ) {
        return this.mkComplex((a.d * -1.0), (a.d2 * -1.0));
      }
      const m = this.findMethod(a, "__neg__");
      if ( (typeof(m) !== "undefined" && m != null )  ) {
        return this.callMethodVal(m, a, []);
      }
    }
    if ( op == "+" ) {
      if ( this.isIntLike(a) ) {
        return this.mkInt(a.i);
      }
      if ( a.t == 3 || a.t == 24 ) {
        return a;
      }
      const m2 = this.findMethod(a, "__pos__");
      if ( (typeof(m2) !== "undefined" && m2 != null )  ) {
        return this.callMethodVal(m2, a, []);
      }
    }
    if ( op == "~" ) {
      if ( this.isIntLike(a) ) {
        return this.mkInt(((0 - a.i) - 1));
      }
      const m3 = this.findMethod(a, "__invert__");
      if ( (typeof(m3) !== "undefined" && m3 != null )  ) {
        return this.callMethodVal(m3, a, []);
      }
    }
    this.raise("TypeError", ((("bad operand type for unary " + op) + ": '") + this.typeName(a)) + "'");
    return this.NONE;
  };
  moduleScope (sc) {
    if ( (typeof(sc.globals) !== "undefined" && sc.globals != null )  ) {
      return sc.globals;
    }
    return sc;
  };
  lookup (name, sc) {
    const r = this.lookupOpt(name, sc);
    if ( typeof(r) === "undefined" ) {
      if ( sc.isClass == false ) {
        if ( ( typeof(sc.vars[name] ) != "undefined" && Object.prototype.hasOwnProperty.call(sc.vars, name) ) ) {
          const v = ( Object.prototype.hasOwnProperty.call(sc.vars, name) ? sc.vars[name] : undefined );
          return v;
        }
      }
      if ( name == "__class__" ) {
        let fi = this.frames.length - 1;
        while (fi >= 0) {
          const cfr = this.frames[fi];
          if ( (typeof(cfr.func) !== "undefined" && cfr.func != null )  ) {
            const cf = cfr.func;
            if ( (typeof(cf.cls) !== "undefined" && cf.cls != null )  ) {
              return cf.cls;
            }
          }
          fi = fi - 1;
        };
      }
      if ( name == "ExceptionGroup" || name == "BaseExceptionGroup" ) {
        return this.lazyBuiltin(name);
      }
      this.raise("NameError", ("name '" + name) + "' is not defined");
    }
    return r;
  };
  lazyBuiltin (name) {
    const m = this.importModule("_exceptiongroup");
    const v = this.getAttr(m, name);
    this.builtins[name] = v;
    return v;
  };
  lookupOpt (name, sc) {
    const d = ( Object.prototype.hasOwnProperty.call(sc.decl, name) ? sc.decl[name] : undefined );
    if ( ((typeof(d) !== "undefined" && d != null ) ) && d == 1 ) {
      const g = this.moduleScope(sc);
      const gv = ( Object.prototype.hasOwnProperty.call(g.vars, name) ? g.vars[name] : undefined );
      if ( (typeof(gv) !== "undefined" && gv != null )  ) {
        return gv;
      }
      return ( Object.prototype.hasOwnProperty.call(this.builtins, name) ? this.builtins[name] : undefined );
    }
    let s = sc;
    while ((typeof(s) !== "undefined" && s != null ) ) {
      const cur = s;
      const v = ( Object.prototype.hasOwnProperty.call(cur.vars, name) ? cur.vars[name] : undefined );
      if ( (typeof(v) !== "undefined" && v != null )  ) {
        return v;
      }
      s = cur.parent;
    };
    const g2 = this.moduleScope(sc);
    const gv2 = ( Object.prototype.hasOwnProperty.call(g2.vars, name) ? g2.vars[name] : undefined );
    if ( (typeof(gv2) !== "undefined" && gv2 != null )  ) {
      return gv2;
    }
    return ( Object.prototype.hasOwnProperty.call(this.builtins, name) ? this.builtins[name] : undefined );
  };
  store (name, val, sc) {
    const d = ( Object.prototype.hasOwnProperty.call(sc.decl, name) ? sc.decl[name] : undefined );
    if ( (typeof(d) !== "undefined" && d != null )  ) {
      if ( d == 1 ) {
        const g = this.moduleScope(sc);
        g.vars[name] = val;
        return;
      }
      let s = sc.parent;
      while ((typeof(s) !== "undefined" && s != null ) ) {
        const cur = s;
        if ( ( typeof(cur.vars[name] ) != "undefined" && Object.prototype.hasOwnProperty.call(cur.vars, name) ) ) {
          cur.vars[name] = val;
          return;
        }
        s = cur.parent;
      };
      this.raise("SyntaxError", ("no binding for nonlocal '" + name) + "' found");
    }
    if ( sc.isClass ) {
      sc.put(name, val);
      return;
    }
    sc.vars[name] = val;
  };
  deleteName (name, sc) {
    const d = ( Object.prototype.hasOwnProperty.call(sc.decl, name) ? sc.decl[name] : undefined );
    if ( ((typeof(d) !== "undefined" && d != null ) ) && d == 1 ) {
      const g = this.moduleScope(sc);
      delete g.vars[name];
      return;
    }
    if ( ( typeof(sc.vars[name] ) != "undefined" && Object.prototype.hasOwnProperty.call(sc.vars, name) ) == false ) {
      this.raise("NameError", ("name '" + name) + "' is not defined");
    }
    delete sc.vars[name];
  };
  assign (target, val, sc) {
    switch (target.kind ) { 
      case 0 : 
        this.store(target.s, val, sc);
        return;
      case 27 : 
        const obj = this.eval(target.k1, sc);
        this.setAttr(obj, target.s, val);
        return;
      case 28 : 
        const obj2 = this.eval(target.k1, sc);
        const idx = this.eval(target.k2, sc);
        this.setItem(obj2, idx, val);
        return;
      case 10 : 
        this.unpack(target.kids, val, sc);
        return;
      case 11 : 
        this.unpack(target.kids, val, sc);
        return;
      case 25 : 
        this.raise("SyntaxError", "starred assignment target must be in a list or tuple");
        break;
    };
    this.raise("SyntaxError", "cannot assign to expression");
  };
  unpack (targets, val, sc) {
    const vals = this.iterValues(val);
    const nt = targets.length;
    const nv = vals.length;
    let star = -1;
    // Loop start
    for ( let i = 0; i < targets.length; i++) {
      var t = targets[i];
      if ( t.kind == 25 ) {
        star = i;
      }
    }
    if ( star < 0 ) {
      if ( nv < nt ) {
        this.raise("ValueError", ((("not enough values to unpack (expected " + ((nt).toString())) + ", got ") + ((nv).toString())) + ")");
      }
      if ( nv > nt ) {
        this.raise("ValueError", ("too many values to unpack (expected " + ((nt).toString())) + ")");
      }
      let i2 = 0;
      while (i2 < nt) {
        this.assign(targets[i2], vals[i2], sc);
        i2 = i2 + 1;
      };
      return;
    }
    if ( nv < nt - 1 ) {
      this.raise("ValueError", ((("not enough values to unpack (expected at least " + ((nt - 1).toString())) + ", got ") + ((nv).toString())) + ")");
    }
    let k = 0;
    while (k < star) {
      this.assign(targets[k], vals[k], sc);
      k = k + 1;
    };
    const after = (nt - star) - 1;
    let mid = [];
    let m = star;
    while (m < nv - after) {
      mid.push(vals[m]);
      m = m + 1;
    };
    const st = targets[star];
    this.assign(st.k1, this.mkList(mid), sc);
    let j = 0;
    while (j < after) {
      this.assign(targets[((star + 1) + j)], vals[((nv - after) + j)], sc);
      j = j + 1;
    };
  };
  eval (n, sc) {
    switch (n.kind ) { 
      case 0 : 
        return this.lookup(n.s, sc);
      case 1 : 
        return this.mkInt(n.i);
      case 2 : 
        if ( n.i == 2 ) {
          return this.mkComplex(0.0, n.d);
        }
        return this.mkFloat(n.d);
      case 3 : 
        return this.mkStr(n.s);
      case 4 : 
        return this.mkStr(this.evalFStr(n, sc));
      case 6 : 
        return this.TRUE;
      case 7 : 
        return this.FALSE;
      case 8 : 
        return this.NONE;
      case 9 : 
        return ( Object.prototype.hasOwnProperty.call(this.builtins, "Ellipsis") ? this.builtins["Ellipsis"] : undefined );
      case 15 : 
        const a = this.eval(n.k1, sc);
        const b = this.eval(n.k2, sc);
        return this.binop(n.s, a, b);
      case 16 : 
        const u = this.eval(n.k1, sc);
        return this.unary(n.s, u);
      case 17 : 
        const nv = this.eval(n.k1, sc);
        return this.mkBool((this.truthy(nv) == false));
      case 18 : 
        const l = this.eval(n.k1, sc);
        if ( this.truthy(l) == false ) {
          return l;
        }
        return this.eval(n.k2, sc);
      case 19 : 
        const l2 = this.eval(n.k1, sc);
        if ( this.truthy(l2) ) {
          return l2;
        }
        return this.eval(n.k2, sc);
      case 20 : 
        let left = this.eval(n.kids[0], sc);
        const cnt = n.strs.length;
        let ci = 0;
        while (ci < cnt) {
          const right = this.eval(n.kids[(ci + 1)], sc);
          if ( this.compare(n.strs[ci], left, right) == false ) {
            return this.FALSE;
          }
          left = right;
          ci = ci + 1;
        };
        return this.TRUE;
      case 21 : 
        const tv = this.eval(n.k2, sc);
        if ( this.truthy(tv) ) {
          return this.eval(n.k1, sc);
        }
        return this.eval(n.k3, sc);
      case 23 : 
        return this.evalCall(n, sc);
      case 27 : 
        const o = this.eval(n.k1, sc);
        return this.getAttr(o, n.s);
      case 28 : 
        const obj = this.eval(n.k1, sc);
        const idx = this.eval(n.k2, sc);
        return this.getItem(obj, idx);
      case 29 : 
        const sl = new PyVal(18);
        sl.items.push(this.evalOpt(n.k1, sc));
        sl.items.push(this.evalOpt(n.k2, sc));
        sl.items.push(this.evalOpt(n.k3, sc));
        return sl;
      case 10 : 
        return this.mkTuple(this.evalItems(n.kids, sc));
      case 11 : 
        return this.mkList(this.evalItems(n.kids, sc));
      case 12 : 
        const st = this.mkSet();
        const its = this.evalItems(n.kids, sc);
        // Loop start
        for ( const it of its) {
          this.setAdd(st, it);
        }
        return st;
      case 13 : 
        const d = this.mkDict();
        const nk = n.kids.length;
        let k = 0;
        while (k < nk) {
          const kn = n.kids[k];
          if ( kn.kind == 14 ) {
            const src = this.eval(kn.k1, sc);
            this.mergeMapping(d, src);
          } else {
            const kv = this.eval(kn, sc);
            const vv = this.eval(n.kids[(k + 1)], sc);
            this.dictSet(d, kv, vv);
          }
          k = k + 2;
        };
        return d;
      case 30 : 
        const out = this.mkList([]);
        this.runComp(n, out, sc);
        return out;
      case 32 : 
        const gcl = n.kids[0];
        const gsrc = this.eval(gcl.k2, sc);
        if ( this.isLazy(gsrc) ) {
          return this.lazyGenexp(n, gsrc, sc);
        }
        const out2 = this.mkList([]);
        this.compFirst = gsrc;
        this.runComp(n, out2, sc);
        const g = this.mkIter(out2.items);
        g.s = "generator";
        return g;
      case 31 : 
        const out3 = this.mkSet();
        this.runComp(n, out3, sc);
        return out3;
      case 33 : 
        const out4 = this.mkDict();
        this.runComp(n, out4, sc);
        return out4;
      case 22 : 
        return this.makeFunction(n, "<lambda>", sc);
      case 36 : 
        const wv = this.eval(n.k1, sc);
        let target = sc;
        while (target.isClass == false) {
          if ( ( typeof(target.decl["__comp__"] ) != "undefined" && Object.prototype.hasOwnProperty.call(target.decl, "__comp__") ) && ((typeof(target.parent) !== "undefined" && target.parent != null ) ) ) {
            target = target.parent;
          } else {
            break;
          }
        };
        this.store(n.s, wv, target);
        return wv;
      case 37 : 
        if ( typeof(this.curGen) === "undefined" ) {
          this.raise("SyntaxError", "'yield' outside function");
        }
        const g_1 = this.curGen;
        const ys = g_1.live[(g_1.live.length - 1)];
        if ( ys.used < ys.res.length ) {
          const r = ys.res[ys.used];
          ys.used = ys.used + 1;
          if ( r.kind == 1 ) {
            this.raiseVal(r.v);
          }
          return r.v;
        }
        let yv = this.NONE;
        if ( (typeof(n.k1) !== "undefined" && n.k1 != null )  ) {
          yv = this.eval(n.k1, sc);
        }
        g_1.out = yv;
        this.tick();
        this.genSuspend();
        return this.NONE;
      case 38 : 
        if ( typeof(this.curGen) === "undefined" ) {
          this.raise("SyntaxError", "'yield' outside function");
        }
        return this.yieldFrom(n, sc);
      case 25 : 
        this.raise("SyntaxError", "can't use starred expression here");
        break;
    };
    this.raise("SyntaxError", "unsupported expression");
    return this.NONE;
  };
  evalOpt (n, sc) {
    if ( typeof(n) === "undefined" ) {
      return this.NONE;
    }
    return this.eval(n, sc);
  };
  mergeMapping (d, src) {
    if ( src.t == 7 ) {
      // Loop start
      for ( let i = 0; i < src.items.length; i++) {
        var k = src.items[i];
        this.dictSet(d, k, src.vals[i]);
      }
      return;
    }
    const ks = this.getAttrOpt(src, "keys");
    if ( typeof(ks) === "undefined" ) {
      this.raise("TypeError", ("'" + this.typeName(src)) + "' object is not a mapping");
    }
    let kn = [];
    let kv = [];
    const keysv = this.callValue(ks, [], kn, kv);
    const all = this.iterValues(keysv);
    // Loop start
    for ( const k2 of all) {
      this.dictSet(d, k2, this.getItem(src, k2));
    }
  };
  evalItems (kids, sc) {
    let out = [];
    // Loop start
    for ( const k of kids) {
      if ( k.kind == 25 ) {
        const v = this.eval(k.k1, sc);
        const vals = this.iterValues(v);
        // Loop start
        for ( const x of vals) {
          out.push(x);
        }
      } else {
        out.push(this.eval(k, sc));
      }
    }
    return out;
  };
  runComp (n, out, sc) {
    const cs = new PyScope();
    cs.parent = sc;
    cs.globals = this.moduleScope(sc);
    cs.decl["__comp__"] = 3;
    this.compStep(n, 0, out, cs);
  };
  compStep (n, ci, out, sc) {
    if ( ci >= n.kids.length ) {
      if ( n.kind == 33 ) {
        const k = this.eval(n.k1, sc);
        const v = this.eval(n.k2, sc);
        this.dictSet(out, k, v);
        return;
      }
      const e = this.eval(n.k1, sc);
      if ( out.t == 8 ) {
        this.setAdd(out, e);
      } else {
        out.items.push(e);
      }
      return;
    }
    const cl = n.kids[ci];
    if ( cl.kind == 35 ) {
      const c = this.eval(cl.k1, sc);
      if ( this.truthy(c) ) {
        this.compStep(n, ci + 1, out, sc);
      }
      return;
    }
    let src = this.NONE;
    if ( ci == 0 && ((typeof(this.compFirst) !== "undefined" && this.compFirst != null ) ) ) {
      src = this.compFirst;
      let nf;
      this.compFirst = nf;
    } else {
      if ( ci == 0 && ((typeof(sc.parent) !== "undefined" && sc.parent != null ) ) ) {
        src = this.eval(cl.k2, sc.parent);
      } else {
        src = this.eval(cl.k2, sc);
      }
    }
    const tg = cl.k1;
    if ( src.t == 15 ) {
      const cnt = this.rangeLen(src);
      let j = 0;
      let x = src.i;
      while (j < cnt) {
        this.tick();
        this.assign(tg, this.mkInt(x), sc);
        this.compStep(n, ci + 1, out, sc);
        x = x + src.i3;
        j = j + 1;
      };
      return;
    }
    const vals = this.iterValues(src);
    // Loop start
    for ( const v2 of vals) {
      this.tick();
      this.assign(tg, v2, sc);
      this.compStep(n, ci + 1, out, sc);
    }
  };
  evalFStr (n, sc) {
    let parts = [];
    // Loop start
    for ( const k of n.kids) {
      if ( k.kind == 3 ) {
        parts.push(k.s);
        continue;
      }
      let v = this.eval(k.k1, sc);
      let spec = "";
      if ( (typeof(k.k2) !== "undefined" && k.k2 != null )  ) {
        spec = this.evalFStr(k.k2, sc);
      }
      if ( k.s == "r" ) {
        v = this.mkStr(this.toRepr(v));
      }
      if ( k.s == "s" ) {
        v = this.mkStr(this.toStr(v));
      }
      if ( k.s == "a" ) {
        v = this.mkStr(this.bi.asciiOf(this.toRepr(v)));
      }
      parts.push(this.formatValue(v, spec));
    }
    return parts.join("");
  };
  formatValue (v, spec) {
    if ( v.t == 12 ) {
      const m = this.findMethod(v, "__format__");
      if ( (typeof(m) !== "undefined" && m != null )  ) {
        const r = this.callMethodVal(m, v, [this.mkStr(spec)]);
        return this.toStr(r);
      }
    }
    if ( spec.length == 0 ) {
      return this.toStr(v);
    }
    const sp = PyFormat.parseSpec(spec);
    if ( sp.err.length > 0 ) {
      this.raise("ValueError", sp.err);
    }
    let out = "";
    if ( this.isIntLike(v) ) {
      out = PyFormat.formatInt(v.i, sp);
    } else {
      if ( v.t == 3 ) {
        out = PyFormat.formatFloat(v.d, sp);
      } else {
        if ( v.t == 4 ) {
          out = PyFormat.formatStr(v.s, sp);
        } else {
          if ( (((v.t == 0 || v.t == 5) || v.t == 6) || v.t == 7) || v.t == 12 ) {
            this.raise("TypeError", ("unsupported format string passed to " + this.typeName(v)) + ".__format__");
          }
          out = PyFormat.formatStr(this.toStr(v), sp);
        }
      }
    }
    if ( out.startsWith(String.fromCodePoint(0) + "!") ) {
      this.raise("ValueError", out.substring(2, out.length ));
    }
    return out;
  };
  evalCall (n, sc) {
    const fnode = n.k1;
    let f = this.NONE;
    if ( (fnode.kind == 0 && fnode.s == "super") && n.kids.length == 0 ) {
      return this.zeroArgSuper(sc);
    }
    f = this.eval(fnode, sc);
    let args = [];
    let kn = [];
    let kv = [];
    // Loop start
    for ( const a of n.kids) {
      if ( a.kind == 24 ) {
        kn.push(a.s);
        kv.push(this.eval(a.k1, sc));
        continue;
      }
      if ( a.kind == 25 ) {
        const sv = this.eval(a.k1, sc);
        const vals = this.iterValues(sv);
        // Loop start
        for ( const x of vals) {
          args.push(x);
        }
        continue;
      }
      if ( a.kind == 26 ) {
        const dv = this.eval(a.k1, sc);
        const tmp = this.mkDict();
        this.mergeMapping(tmp, dv);
        // Loop start
        for ( let di = 0; di < tmp.items.length; di++) {
          var dk = tmp.items[di];
          if ( dk.t != 4 ) {
            this.raise("TypeError", "keywords must be strings");
          }
          kn.push(dk.s);
          kv.push(tmp.vals[di]);
        }
        continue;
      }
      args.push(this.eval(a, sc));
    }
    const saved = this.curLine;
    if ( f.t == 10 ) {
      this.callScope = sc;
    }
    const r = this.callValue(f, args, kn, kv);
    this.curLine = saved;
    return r;
  };
  parseSource (src, mode) {
    const ps = new PyParser();
    let tree;
    try {
      if ( mode == "eval" ) {
        tree = ps.parseExpressionText(src.trim(), 1);
      } else {
        tree = ps.parseModule(src);
      }
    } catch(e) {
      if ( ps.err.length == 0 ) {
        ps.err = ( e.toString());
      }
    }
    if ( ps.err.length > 0 ) {
      const e = this.makeExc(this.excClass("SyntaxError"), ps.err);
      e.putAttr("msg", this.mkStr(ps.err));
      e.putAttr("lineno", this.mkInt(ps.errLine));
      e.putAttr("offset", this.NONE);
      e.putAttr("text", this.NONE);
      e.putAttr("filename", this.mkStr("<string>"));
      this.raiseVal(e);
    }
    const t = tree;
    return t;
  };
  codeOf (src, mode) {
    if ( src.t == 10 && src.mode == "code" ) {
      return src;
    }
    if ( src.t != 4 ) {
      this.raise("TypeError", mode + "() arg 1 must be a string, bytes or code object");
    }
    const c = new PyVal(10);
    c.s = "<code>";
    c.mode = "code";
    c.putAttr("co_source", src);
    c.putAttr("co_mode", this.mkStr(mode));
    const tree = this.parseSource(src.s, mode);
    c.node = tree;
    return c;
  };
  scopeFor (g, l) {
    let ns = g;
    if ( ns.t == 0 && l.t == 7 ) {
      ns = l;
    }
    if ( ns.t == 0 ) {
      return this.callScope;
    }
    if ( ns.t != 7 ) {
      this.raise("TypeError", "globals must be a dict, not " + this.typeName(ns));
    }
    const sc = new PyScope();
    // Loop start
    for ( let i = 0; i < ns.items.length; i++) {
      var k = ns.items[i];
      if ( k.t == 4 ) {
        sc.vars[k.s] = ns.vals[i];
      }
    }
    if ( l.t == 7 && this.same(l, ns) == false ) {
      // Loop start
      for ( let i2 = 0; i2 < l.items.length; i2++) {
        var k2 = l.items[i2];
        if ( k2.t == 4 ) {
          sc.vars[k2.s] = l.vals[i2];
        }
      }
    }
    return sc;
  };
  writeBack (sc, g, l) {
    let ns = g;
    if ( l.t == 7 && (ns.t == 0 || this.same(l, ns) == false) ) {
      ns = l;
    }
    if ( ns.t != 7 ) {
      return;
    }
    const ks = sc.orderedKeys();
    // Loop start
    for ( const k of ks) {
      this.dictSet(
        ns,
        this.mkStr(k),
        ( Object.prototype.hasOwnProperty.call(sc.vars, k) ? sc.vars[k] : undefined )
      );
    }
  };
  runCode (fname, src, g, l) {
    let mode = fname;
    const code = this.codeOf(src, mode);
    const cm = ( Object.prototype.hasOwnProperty.call(code.attrs, "co_mode") ? code.attrs["co_mode"] : undefined );
    if ( (typeof(cm) !== "undefined" && cm != null )  ) {
      const cmv = cm;
      mode = cmv.s;
    }
    const tree = code.node;
    const sc = this.scopeFor(g, l);
    const saved = this.curLine;
    let result = this.NONE;
    if ( mode == "eval" ) {
      result = this.eval(tree, sc);
    } else {
      this.hoistDecls(tree.body, sc);
      this.setupAnn(tree.body, sc);
      this.execBlock(tree.body, sc);
    }
    this.curLine = saved;
    this.writeBack(sc, g, l);
    return result;
  };
  zeroArgSuper (sc) {
    if ( this.frames.length == 0 ) {
      this.raise("RuntimeError", "super(): no arguments");
    }
    const fr = this.frames[(this.frames.length - 1)];
    if ( (typeof(fr.func) === "undefined") || (typeof(fr.__self) === "undefined") ) {
      this.raise("RuntimeError", "super(): no arguments");
    }
    const f = fr.func;
    if ( typeof(f.cls) === "undefined" ) {
      this.raise("RuntimeError", "super(): __class__ cell not found");
    }
    const s = new PyVal(22);
    s.cls = f.cls;
    s.__self = fr.__self;
    return s;
  };
  makeFunction (n, name, sc) {
    const f = new PyVal(9);
    f.s = name;
    f.node = n;
    let closure = sc;
    while (closure.isClass && ((typeof(closure.parent) !== "undefined" && closure.parent != null ) )) {
      closure = closure.parent;
    };
    if ( closure.isClass ) {
      closure = this.moduleScope(closure);
    }
    f.scope = closure;
    // Loop start
    for ( const p of n.kids) {
      if ( (typeof(p.k1) !== "undefined" && p.k1 != null )  ) {
        f.defs[p.s] = this.eval(p.k1, sc);
      }
    }
    if ( n.kind == 51 ) {
      let asc = sc;
      if ( n.tparams.length > 0 ) {
        const ts = new PyScope();
        ts.parent = closure;
        ts.globals = this.moduleScope(sc);
        f.putAttr("__type_params__", this.bindTypeParams(n.tparams, ts));
        f.scope = ts;
        asc = ts;
      }
      const ann = this.mkDict();
      // Loop start
      for ( const p2 of n.kids) {
        if ( (typeof(p2.k2) !== "undefined" && p2.k2 != null )  ) {
          this.dictSet(ann, this.mkStr(p2.s), this.eval(p2.k2, asc));
        }
      }
      if ( (typeof(n.k3) !== "undefined" && n.k3 != null )  ) {
        this.dictSet(ann, this.mkStr("return"), this.eval(n.k3, asc));
      }
      if ( ann.items.length > 0 ) {
        f.putAttr("__annotations__", ann);
      }
    }
    return f;
  };
  funcAttr (f, name) {
    let none;
    const node = f.node;
    let r;
    if ( name == "__name__" ) {
      r = this.mkStr(f.s);
      return r;
    }
    if ( name == "__qualname__" ) {
      if ( node.qual.length > 0 ) {
        r = this.mkStr(node.qual);
      } else {
        r = this.mkStr(f.s);
      }
      return r;
    }
    if ( name == "__doc__" ) {
      if ( node.kind == 51 ) {
        r = this.docOf(node.body);
        return r;
      }
      r = this.NONE;
      return r;
    }
    if ( name == "__module__" ) {
      r = this.moduleName(f.scope);
      return r;
    }
    if ( name == "__annotations__" ) {
      const ad = this.mkDict();
      f.putAttr("__annotations__", ad);
      r = ad;
      return r;
    }
    if ( name == "__type_params__" ) {
      r = this.mkTuple([]);
      return r;
    }
    if ( name == "__defaults__" || name == "__kwdefaults__" ) {
      let pos = [];
      const kwd = this.mkDict();
      let kwOnly = false;
      // Loop start
      for ( const p of node.kids) {
        if ( p.i == 1 || p.i == 3 ) {
          kwOnly = true;
        }
        if ( ( typeof(f.defs[p.s] ) != "undefined" && Object.prototype.hasOwnProperty.call(f.defs, p.s) ) ) {
          if ( kwOnly ) {
            this.dictSet(
              kwd,
              this.mkStr(p.s),
              ( Object.prototype.hasOwnProperty.call(f.defs, p.s) ? f.defs[p.s] : undefined )
            );
          } else {
            pos.push(( Object.prototype.hasOwnProperty.call(f.defs, p.s) ? f.defs[p.s] : undefined ));
          }
        }
      }
      if ( name == "__defaults__" ) {
        if ( pos.length == 0 ) {
          r = this.NONE;
          return r;
        }
        r = this.mkTuple(pos);
        return r;
      }
      if ( kwd.items.length == 0 ) {
        r = this.NONE;
        return r;
      }
      r = kwd;
      return r;
    }
    if ( name == "__dict__" ) {
      const dd = this.mkDict();
      // Loop start
      for ( let ki = 0; ki < f.attrKeys().length; ki++) {
        var k = f.attrKeys()[ki];
        this.dictSet(
          dd,
          this.mkStr(k),
          ( Object.prototype.hasOwnProperty.call(f.attrs, k) ? f.attrs[k] : undefined )
        );
      }
      r = dd;
      return r;
    }
    if ( name == "__globals__" ) {
      const gs = this.moduleScope(f.scope);
      const gd = this.mkDict();
      const gsk = Object.keys(gs.vars);
      // Loop start
      for ( const gk of gsk) {
        this.dictSet(
          gd,
          this.mkStr(gk),
          ( Object.prototype.hasOwnProperty.call(gs.vars, gk) ? gs.vars[gk] : undefined )
        );
      }
      r = gd;
      return r;
    }
    if ( name == "__closure__" ) {
      r = this.NONE;
      return r;
    }
    if ( name == "__code__" ) {
      const co = new PyVal(12);
      co.cls = this.codeClass();
      co.putAttr("co_name", this.mkStr(f.s));
      co.putAttr("co_qualname", this.mkStr(node.qual));
      co.putAttr("co_firstlineno", this.mkInt(node.line));
      co.putAttr("co_filename", this.mkStr("<script>"));
      let argc = 0;
      let kwc = 0;
      let posonly = 0;
      let flags = 0;
      let names = [];
      let kwOnly2 = false;
      // Loop start
      for ( const p3 of node.kids) {
        if ( p3.i == 4 ) {
          posonly = argc;
        }
        if ( p3.i == 1 ) {
          flags = flags + 4;
          kwOnly2 = true;
        }
        if ( p3.i == 2 ) {
          flags = flags + 8;
        }
        if ( p3.i == 3 ) {
          kwOnly2 = true;
        }
        if ( p3.i == 0 ) {
          if ( kwOnly2 ) {
            kwc = kwc + 1;
          } else {
            argc = argc + 1;
          }
        }
      }
      // Loop start
      for ( const p4 of node.kids) {
        if ( p4.i == 0 ) {
          names.push(this.mkStr(p4.s));
        }
      }
      // Loop start
      for ( const p5 of node.kids) {
        if ( p5.i == 1 || p5.i == 2 ) {
          names.push(this.mkStr(p5.s));
        }
      }
      if ( node.i == 1 ) {
        flags = flags + 32;
      }
      co.putAttr("co_argcount", this.mkInt(argc));
      co.putAttr("co_kwonlyargcount", this.mkInt(kwc));
      co.putAttr("co_posonlyargcount", this.mkInt(posonly));
      co.putAttr("co_flags", this.mkInt(flags));
      co.putAttr("co_varnames", this.mkTuple(names));
      co.putAttr("co_nlocals", this.mkInt(names.length));
      r = co;
      return r;
    }
    return none;
  };
  codeClass () {
    if ( typeof(this.codeCls) === "undefined" ) {
      const c = new PyVal(11);
      c.s = "code";
      this.codeCls = c;
    }
    return this.codeCls;
  };
  execTypeAlias (n, sc) {
    let ts = sc;
    let tps = this.mkTuple([]);
    if ( n.tparams.length > 0 ) {
      const tsc = new PyScope();
      tsc.parent = sc;
      tsc.globals = this.moduleScope(sc);
      tps = this.bindTypeParams(n.tparams, tsc);
      ts = tsc;
    }
    let val = this.NONE;
    try {
      val = this.eval(n.k1, ts);
    } catch(e) {
      this.takeException(( e.toString()));
    }
    const tat = this.getAttr(this.importModule("typing"), "TypeAliasType");
    let kn = [];
    let kv = [];
    kn.push("type_params");
    kv.push(tps);
    const alias = this.callValue(tat, [this.mkStr(n.s), val], kn, kv);
    this.store(n.s, alias, sc);
  };
  bindTypeParams (names, sc) {
    const tv = this.getAttr(this.importModule("typing"), "TypeVar");
    let out = [];
    // Loop start
    for ( const nm of names) {
      let kn = [];
      let kv = [];
      kn.push("infer_variance");
      kv.push(this.TRUE);
      const v = this.callValue(tv, [this.mkStr(nm)], kn, kv);
      sc.vars[nm] = v;
      out.push(v);
    }
    return this.mkTuple(out);
  };
  docOf (body) {
    if ( body.length > 0 ) {
      const st = body[0];
      if ( st.kind == 40 && ((typeof(st.k1) !== "undefined" && st.k1 != null ) ) ) {
        const e = st.k1;
        if ( e.kind == 3 ) {
          return this.mkStr(e.s);
        }
      }
    }
    return this.NONE;
  };
  moduleName (sc) {
    const ms = this.moduleScope(sc);
    const nm = ( Object.prototype.hasOwnProperty.call(ms.vars, "__name__") ? ms.vars["__name__"] : undefined );
    if ( typeof(nm) === "undefined" ) {
      return this.mkStr("__main__");
    }
    return nm;
  };
  callValue (f, args, kn, kv) {
    switch (f.t ) { 
      case 9 : 
        return this.callFunc(f, args, kn, kv);
      case 14 : 
        const __self = f.__self;
        if ( typeof(f.cls) === "undefined" ) {
          return this.meth.callMethod(__self, f.s, args, kn, kv);
        }
        let all = [];
        all.push(__self);
        // Loop start
        for ( const a of args) {
          all.push(a);
        }
        const fn2 = f.cls;
        if ( fn2.t == 9 ) {
          return this.callFunc(fn2, all, kn, kv);
        }
        return this.callValue(fn2, all, kn, kv);
      case 10 : 
        if ( (typeof(f.wrapped) !== "undefined" && f.wrapped != null )  ) {
          return this.mods.callWrapped(f, args, kn, kv);
        }
        return this.bi.call(f.s, args, kn, kv);
      case 11 : 
        if ( (typeof(f.cls) !== "undefined" && f.cls != null )  ) {
          const mcall = this.classLookup(f.cls, "__call__");
          const __REGx1 = this.orNone(mcall);
          if ( ((typeof(mcall) !== "undefined" && mcall != null ) ) && __REGx1.t == 9 ) {
            let mall = [];
            mall.push(f);
            // Loop start
            for ( const ma of args) {
              mall.push(ma);
            }
            return this.callValue(mcall, mall, kn, kv);
          }
        }
        return this.instantiate(f, args, kn, kv);
      case 12 : 
        const m = this.findMethod(f, "__call__");
        if ( (typeof(m) !== "undefined" && m != null )  ) {
          const mv = m;
          if ( mv.t == 12 || mv.t == 14 ) {
            return this.callValue(mv, args, kn, kv);
          }
          let all2 = [];
          all2.push(f);
          // Loop start
          for ( const a2 of args) {
            all2.push(a2);
          }
          return this.callValue(m, all2, kn, kv);
        }
        break;
      case 20 : 
        return this.callValue(f.wrapped, args, kn, kv);
    };
    if ( this.isDataSub(f) ) {
      const dm = this.findMethod(f, "__call__");
      if ( (typeof(dm) !== "undefined" && dm != null )  ) {
        let all3 = [];
        all3.push(f);
        // Loop start
        for ( const a3 of args) {
          all3.push(a3);
        }
        return this.callValue(dm, all3, kn, kv);
      }
    }
    this.raise("TypeError", ("'" + this.typeName(f)) + "' object is not callable");
    return this.NONE;
  };
  callFunc (f, args, kn, kv) {
    const node = f.node;
    const sc = new PyScope();
    sc.parent = f.scope;
    const fsc = f.scope;
    sc.globals = this.moduleScope(fsc);
    this.bindArgs(f, node, sc, args, kn, kv);
    if ( this.frames.length >= this.maxDepth ) {
      this.raise("RecursionError", "maximum recursion depth exceeded");
    }
    const fr = new PyFrame();
    fr.func = f;
    fr.name = f.s;
    fr.line = this.curLine;
    fr.callLine = this.curLine;
    if ( args.length > 0 ) {
      fr.__self = args[0];
    }
    this.frames.push(fr);
    let result = this.NONE;
    let failed = false;
    let err = this.NONE;
    const outerGen = this.curGen;
    let noGen;
    this.curGen = noGen;
    if ( node.kind == 22 ) {
      try {
        result = this.eval(node.k1, sc);
      } catch(e) {
        failed = true;
        err = this.takeException(( e.toString()));
      }
    } else {
      this.hoistDecls(node.body, sc);
      if ( node.i == 1 ) {
        const gen = new PyGen();
        gen.func = f;
        gen.node = node;
        gen.scope = sc;
        gen.name = f.s;
        gen.line = node.line;
        if ( args.length > 0 ) {
          gen.selfv = args[0];
        }
        const g = new PyVal(23);
        g.gen = gen;
        g.s = f.s;
        result = g;
      } else {
        try {
          const sig = this.execBlock(node.body, sc);
          if ( sig == 3 ) {
            result = this.retVal;
          }
        } catch(e) {
          failed = true;
          err = this.takeException(( e.toString()));
        }
      }
    }
    this.frames.pop();
    this.curGen = outerGen;
    if ( failed ) {
      this.reraise(err);
    }
    this.retVal = this.NONE;
    return result;
  };
  reraise (e) {
    this.excValue = e;
    throw new Error("python exception");
  };
  hoistDecls (body, sc) {
    // Loop start
    for ( const st of body) {
      if ( st.kind == 59 ) {
        // Loop start
        for ( const nm of st.strs) {
          sc.decl[nm] = 1;
        }
      }
      if ( st.kind == 60 ) {
        // Loop start
        for ( const nm2 of st.strs) {
          sc.decl[nm2] = 2;
        }
      }
      if ( (((st.kind == 44 || st.kind == 45) || st.kind == 46) || st.kind == 63) || st.kind == 54 ) {
        this.hoistDecls(st.body, sc);
        this.hoistDecls(st.orelse, sc);
        this.hoistDecls(st.final, sc);
        // Loop start
        for ( const h of st.handlers) {
          this.hoistDecls(h.body, sc);
        }
      }
    }
  };
  bindArgs (f, node, sc, args, kn, kv) {
    const params = node.kids;
    const nargs = args.length;
    let ai = 0;
    let afterStar = false;
    let kwParam = "";
    let positional = 0;
    let bound = {};
    // Loop start
    for ( const p of params) {
      if ( p.i == 4 ) {
        continue;
      }
      if ( p.i == 3 ) {
        afterStar = true;
        continue;
      }
      if ( p.i == 1 ) {
        let rest = [];
        while (ai < nargs) {
          rest.push(args[ai]);
          ai = ai + 1;
        };
        sc.vars[p.s] = this.mkTuple(rest);
        bound[p.s] = true;
        afterStar = true;
        continue;
      }
      if ( p.i == 2 ) {
        kwParam = p.s;
        continue;
      }
      if ( afterStar == false ) {
        positional = positional + 1;
        if ( ai < nargs ) {
          sc.vars[p.s] = args[ai];
          bound[p.s] = true;
          ai = ai + 1;
        }
      }
    }
    if ( ai < nargs ) {
      this.raise("TypeError", ((((((((f.s + "() takes ") + ((positional).toString())) + " positional argument") + this.plural(positional)) + " but ") + ((nargs).toString())) + " ") + this.wasWere(nargs)) + " given");
    }
    let kwDict = this.NONE;
    if ( kwParam.length > 0 ) {
      kwDict = this.mkDict();
    }
    let slashAt = -1;
    // Loop start
    for ( let j4 = 0; j4 < params.length; j4++) {
      var p4 = params[j4];
      if ( p4.i == 4 ) {
        slashAt = j4;
      }
    }
    // Loop start
    for ( let k = 0; k < kn.length; k++) {
      var name = kn[k];
      let found = false;
      // Loop start
      for ( let j = 0; j < params.length; j++) {
        var p2 = params[j];
        if ( (p2.i == 0 && p2.s == name) && j > slashAt ) {
          found = true;
        }
      }
      if ( found ) {
        if ( ( typeof(bound[name] ) != "undefined" && Object.prototype.hasOwnProperty.call(bound, name) ) ) {
          this.raise("TypeError", ((f.s + "() got multiple values for argument '") + name) + "'");
        }
        sc.vars[name] = kv[k];
        bound[name] = true;
      } else {
        if ( kwParam.length == 0 ) {
          this.raise("TypeError", ((f.s + "() got an unexpected keyword argument '") + name) + "'");
        }
        this.dictSet(kwDict, this.mkStr(name), kv[k]);
      }
    }
    if ( kwParam.length > 0 ) {
      sc.vars[kwParam] = kwDict;
    }
    let missing = [];
    // Loop start
    for ( const p3 of params) {
      if ( p3.i == 0 && ( typeof(bound[p3.s] ) != "undefined" && Object.prototype.hasOwnProperty.call(bound, p3.s) ) == false ) {
        const dv = ( Object.prototype.hasOwnProperty.call(f.defs, p3.s) ? f.defs[p3.s] : undefined );
        if ( (typeof(dv) !== "undefined" && dv != null )  ) {
          sc.vars[p3.s] = dv;
        } else {
          missing.push(("'" + p3.s) + "'");
        }
      }
    }
    if ( missing.length > 0 ) {
      this.raise("TypeError", (((((f.s + "() missing ") + ((missing.length).toString())) + " required argument") + this.plural(missing.length)) + ": ") + missing.join(", "));
    }
  };
  plural (n) {
    if ( n == 1 ) {
      return "";
    }
    return "s";
  };
  wasWere (n) {
    if ( n == 1 ) {
      return "was";
    }
    return "were";
  };
  isObjLike (v) {
    if ( v.t == 12 ) {
      return true;
    }
    return this.isDataSub(v);
  };
  isDataT (t) {
    return ((((((t == 5 || t == 7) || t == 4) || t == 2) || t == 3) || t == 6) || t == 8) || t == 24;
  };
  isDataSub (v) {
    if ( typeof(v.cls) === "undefined" ) {
      return false;
    }
    if ( this.isDataT(v.t) == false ) {
      return false;
    }
    if ( (typeof(this.rawSelf) !== "undefined" && this.rawSelf != null )  ) {
      if ( this.rawSelf == v ) {
        return false;
      }
    }
    return true;
  };
  subDunder (v, name, args) {
    let none;
    if ( v.t == 11 && ((typeof(v.cls) !== "undefined" && v.cls != null ) ) ) {
      const mm = this.classLookup(v.cls, name);
      const __REGx1 = this.orNone(mm);
      if ( (typeof(mm) === "undefined") || __REGx1.t != 9 ) {
        return none;
      }
      const mr = this.callMethodVal(mm, v, args);
      return mr;
    }
    if ( this.isDataSub(v) == false ) {
      return none;
    }
    const m = this.findMethod(v, name);
    if ( typeof(m) === "undefined" ) {
      return none;
    }
    const r = this.callMethodVal(m, v, args);
    return r;
  };
  dataSubAttr (obj, name) {
    let none;
    const c = obj.cls;
    let r;
    if ( name == "__class__" ) {
      r = c;
      return r;
    }
    if ( ( typeof(obj.attrs[name] ) != "undefined" && Object.prototype.hasOwnProperty.call(obj.attrs, name) ) ) {
      if ( obj.t != 7 || name != "__missing__" ) {
        return ( Object.prototype.hasOwnProperty.call(obj.attrs, name) ? obj.attrs[name] : undefined );
      }
    }
    if ( name == "__dict__" ) {
      const d = this.mkDict();
      // Loop start
      for ( let i = 0; i < obj.attrKeys().length; i++) {
        var k = obj.attrKeys()[i];
        this.dictSet(
          d,
          this.mkStr(k),
          ( Object.prototype.hasOwnProperty.call(obj.attrs, k) ? obj.attrs[k] : undefined )
        );
      }
      r = d;
      return r;
    }
    const cv = this.classLookup(c, name);
    if ( typeof(cv) === "undefined" ) {
      return none;
    }
    r = this.bindValue(cv, obj, c);
    return r;
  };
  cloneVal (v) {
    const c = new PyVal(v.t);
    c.i = v.i;
    c.i2 = v.i2;
    c.i3 = v.i3;
    c.d = v.d;
    c.d2 = v.d2;
    c.s = v.s;
    c.mode = v.mode;
    // Loop start
    for ( const it of v.items) {
      c.items.push(it);
    }
    // Loop start
    for ( const vv of v.vals) {
      c.vals.push(vv);
    }
    const vik = Object.keys(v.index);
    // Loop start
    for ( const k of vik) {
      c.index[k] = ( Object.prototype.hasOwnProperty.call(v.index, k) ? v.index[k] : undefined );
    }
    // Loop start
    for ( let ai = 0; ai < v.attrKeys().length; ai++) {
      var ak = v.attrKeys()[ai];
      c.putAttr(ak, ( Object.prototype.hasOwnProperty.call(v.attrs, ak) ? v.attrs[ak] : undefined ));
    }
    return c;
  };
  makeDataSub (cls, args, kn, kv) {
    const bt = cls.bt;
    const hasInit = (typeof(this.classLookup(cls, "__init__")) !== "undefined" && this.classLookup(cls, "__init__") != null ) ;
    let base = this.NONE;
    let none = [];
    let nkn = [];
    let nkv = [];
    if ( bt == "native" ) {
      if ( hasInit ) {
        base = this.mods.construct(cls.wrapped, none, nkn, nkv);
      } else {
        base = this.mods.construct(cls.wrapped, args, kn, kv);
      }
    } else {
      if ( hasInit && ((bt == "list" || bt == "dict") || bt == "set") ) {
        base = this.bi.call(bt, none, nkn, nkv);
      } else {
        base = this.bi.call(bt, args, kn, kv);
      }
    }
    const v = this.cloneVal(base);
    v.cls = cls;
    return v;
  };
  newDataSub (cls, args, kn, kv) {
    const nw = this.classLookup(cls, "__new__");
    let obj = this.NONE;
    if ( (typeof(nw) !== "undefined" && nw != null )  ) {
      let nf = nw;
      if ( nf.t == 20 ) {
        nf = nf.wrapped;
      }
      let all0 = [];
      all0.push(cls);
      // Loop start
      for ( const a0 of args) {
        all0.push(a0);
      }
      obj = this.callValue(nf, all0, kn, kv);
      if ( ((typeof(obj.cls) === "undefined") || this.isDataT(obj.t) == false) || this.same(obj.cls, cls) == false ) {
        return obj;
      }
    } else {
      obj = this.makeDataSub(cls, args, kn, kv);
    }
    const init = this.classLookup(cls, "__init__");
    if ( (typeof(init) !== "undefined" && init != null )  ) {
      let all = [];
      all.push(obj);
      // Loop start
      for ( const a of args) {
        all.push(a);
      }
      const r = this.callValue(init, all, kn, kv);
      if ( r.t != 0 ) {
        this.raise("TypeError", "__init__() should return None");
      }
    }
    return obj;
  };
  instantiate (cls, args, kn, kv) {
    if ( cls.mode == "native" ) {
      return this.mods.construct(cls, args, kn, kv);
    }
    if ( cls.bt == "type" ) {
      return this.metaInstantiate(cls, args, kn, kv);
    }
    const abm = this.orNone(this.classLookup(cls, "__abstractmethods__"));
    if ( (abm.t == 8 || abm.t == 6) && abm.items.length > 0 ) {
      let anames = [];
      // Loop start
      for ( const an of abm.items) {
        anames.push(this.toStr(an));
      }
      anames = PyFileSystem.sortStrings(anames);
      let alist = "";
      // Loop start
      for ( let ani2 = 0; ani2 < anames.length; ani2++) {
        var an2 = anames[ani2];
        if ( ani2 > 0 ) {
          alist = alist + ", ";
        }
        alist = ((alist + "'") + an2) + "'";
      }
      let plural = "";
      if ( anames.length > 1 ) {
        plural = "s";
      }
      this.raise("TypeError", (((("Can't instantiate abstract class " + cls.s) + " without an implementation for abstract method") + plural) + " ") + alist);
    }
    if ( cls.bt.length > 0 ) {
      return this.newDataSub(cls, args, kn, kv);
    }
    const nw = this.classLookup(cls, "__new__");
    let obj = new PyVal(12);
    obj.cls = cls;
    if ( cls.isExc ) {
      obj.putAttr("args", this.mkTuple(args));
    }
    const __REGx1 = this.orNone(nw);
    const __REGx2 = this.orNone(nw);
    if ( ((typeof(nw) !== "undefined" && nw != null ) ) && __REGx1.t == 20 || ((typeof(nw) !== "undefined" && nw != null ) ) && __REGx2.t == 9 ) {
      let nf = nw;
      if ( nf.t == 20 ) {
        nf = nf.wrapped;
      }
      let all0 = [];
      all0.push(cls);
      // Loop start
      for ( const a0 of args) {
        all0.push(a0);
      }
      const made = this.callValue(nf, all0, kn, kv);
      if ( made.t != 12 || this.same(made.cls, cls) == false ) {
        return made;
      }
      obj = made;
    }
    const init = this.classLookup(cls, "__init__");
    if ( (typeof(init) === "undefined") && ((typeof(nw) !== "undefined" && nw != null ) ) ) {
      return obj;
    }
    if ( (typeof(init) !== "undefined" && init != null )  ) {
      const fv = init;
      let all = [];
      all.push(obj);
      // Loop start
      for ( const a of args) {
        all.push(a);
      }
      const r = this.callValue(fv, all, kn, kv);
      if ( r.t != 0 ) {
        this.raise("TypeError", "__init__() should return None");
      }
      return obj;
    }
    if ( cls.isExc == false && (args.length > 0 || kn.length > 0) ) {
      this.raise("TypeError", cls.s + "() takes no arguments");
    }
    return obj;
  };
  execBlock (body, sc) {
    if ( (typeof(this.curGen) !== "undefined" && this.curGen != null )  ) {
      return this.execBlockGen(body, sc);
    }
    // Loop start
    for ( const st of body) {
      const sig = this.exec(st, sc);
      if ( sig != 0 ) {
        return sig;
      }
    }
    return 0;
  };
  exec (n, sc) {
    this.curLine = n.line;
    if ( this.frames.length > 0 ) {
      const fr = this.frames[(this.frames.length - 1)];
      fr.line = n.line;
    }
    this.tick();
    switch (n.kind ) { 
      case 40 : 
        this.eval(n.k1, sc);
        return 0;
      case 41 : 
        const v = this.eval(n.k1, sc);
        // Loop start
        for ( const t of n.kids) {
          this.assign(t, v, sc);
        }
        return 0;
      case 42 : 
        this.execAug(n, sc);
        return 0;
      case 43 : 
        if ( (typeof(n.k2) !== "undefined" && n.k2 != null )  ) {
          const av = this.eval(n.k2, sc);
          this.assign(n.k1, av, sc);
        }
        const at = n.k1;
        if ( (((typeof(n.k3) !== "undefined" && n.k3 != null ) ) && at.kind == 0) && (sc.isClass || (typeof(sc.parent) === "undefined")) ) {
          const anv = this.eval(n.k3, sc);
          let ad = ( Object.prototype.hasOwnProperty.call(sc.vars, "__annotations__") ? sc.vars["__annotations__"] : undefined );
          if ( typeof(ad) === "undefined" ) {
            ad = this.mkDict();
            sc.vars["__annotations__"] = ad;
          }
          this.dictSet(ad, this.mkStr(at.s), anv);
        }
        return 0;
      case 53 : 
        this.execTypeAlias(n, sc);
        return 0;
      case 44 : 
        if ( (typeof(this.curGen) !== "undefined" && this.curGen != null )  ) {
          return this.execIfGen(n, sc);
        }
        const c = this.eval(n.k1, sc);
        if ( this.truthy(c) ) {
          return this.execBlock(n.body, sc);
        }
        return this.execBlock(n.orelse, sc);
      case 45 : 
        if ( (typeof(this.curGen) !== "undefined" && this.curGen != null )  ) {
          return this.execWhileGen(n, sc);
        }
        while (true) {
          const wc = this.eval(n.k1, sc);
          if ( this.truthy(wc) == false ) {
            return this.execBlock(n.orelse, sc);
          }
          const ws = this.execBlock(n.body, sc);
          if ( ws == 1 ) {
            return 0;
          }
          if ( ws == 3 ) {
            return 3;
          }
          this.tick();
        };
        break;
      case 46 : 
        if ( (typeof(this.curGen) !== "undefined" && this.curGen != null )  ) {
          return this.execForGen(n, sc);
        }
        return this.execFor(n, sc);
      case 47 : 
        return 1;
      case 48 : 
        return 2;
      case 49 : 
        return 0;
      case 50 : 
        if ( typeof(n.k1) === "undefined" ) {
          this.retVal = this.NONE;
        } else {
          this.retVal = this.eval(n.k1, sc);
        }
        return 3;
      case 51 : 
        const f = this.makeFunction(n, n.s, sc);
        const fv = this.applyDecorators(n.decos, f, sc);
        this.store(n.s, fv, sc);
        return 0;
      case 52 : 
        this.execClass(n, sc);
        return 0;
      case 54 : 
        if ( (typeof(this.curGen) !== "undefined" && this.curGen != null )  ) {
          return this.execTryGen(n, sc);
        }
        return this.execTry(n, sc);
      case 56 : 
        this.execRaise(n, sc);
        return 0;
      case 57 : 
        this.execImport(n, sc);
        return 0;
      case 58 : 
        this.execFromImport(n, sc);
        return 0;
      case 59 : 
        // Loop start
        for ( const nm of n.strs) {
          sc.decl[nm] = 1;
        }
        return 0;
      case 60 : 
        // Loop start
        for ( const nm2 of n.strs) {
          sc.decl[nm2] = 2;
        }
        return 0;
      case 61 : 
        const ac = this.eval(n.k1, sc);
        if ( this.truthy(ac) == false ) {
          if ( (typeof(n.k2) !== "undefined" && n.k2 != null )  ) {
            const am = this.eval(n.k2, sc);
            this.raiseVal(this.makeExcArg("AssertionError", am));
          }
          this.raise("AssertionError", "");
        }
        return 0;
      case 62 : 
        // Loop start
        for ( const dt of n.kids) {
          this.execDel(dt, sc);
        }
        return 0;
      case 63 : 
        if ( (typeof(this.curGen) !== "undefined" && this.curGen != null )  ) {
          return this.execWithGen(n, 0, sc);
        }
        return this.execWith(n, 0, sc);
      case 66 : 
        if ( (typeof(this.curGen) !== "undefined" && this.curGen != null )  ) {
          return this.execMatchGen(n, sc);
        }
        const mc = this.matchCase(n, sc);
        if ( mc < 0 ) {
          return 0;
        }
        const mcs = n.kids[mc];
        return this.execBlock(mcs.body, sc);
    };
    this.raise("SyntaxError", "unsupported statement");
    return 0;
  };
  genSuspend () {
    this.suspending = true;
    this.excValue = this.SUSPEND;
    throw new Error("generator suspend");
  };
  resuspend () {
    this.excValue = this.SUSPEND;
    throw new Error("generator suspend");
  };
  genResuming (g) {
    return g.depth < g.live.length;
  };
  genEnter (g) {
    if ( g.depth < g.live.length ) {
      const s = g.live[g.depth];
      g.depth = g.depth + 1;
      return s;
    }
    const ns = new PyGenSlot();
    g.live.push(ns);
    g.depth = g.depth + 1;
    return ns;
  };
  genLeave (g) {
    g.live.pop();
    g.depth = g.live.length;
  };
  genTruncate (g, d) {
    while (g.live.length > d) {
      g.live.pop();
    };
    g.depth = d;
  };
  clearRes (s) {
    let none = [];
    s.res = none;
    s.used = 0;
  };
  stopIter (v) {
    const e = this.makeExc(this.excClass("StopIteration"), "");
    if ( v.t != 0 ) {
      e.putAttr("args", this.mkTuple([v]));
    }
    e.putAttr("value", v);
    return e;
  };
  genRun (gv, sendv, mode) {
    const g = gv.gen;
    if ( g.running ) {
      this.raise("ValueError", "generator already executing");
    }
    if ( g.done ) {
      if ( mode == 1 ) {
        this.raiseVal(sendv);
      }
      g.out = this.NONE;
      return false;
    }
    if ( g.started == false ) {
      if ( mode == 0 && sendv.t != 0 ) {
        this.raise("TypeError", "can't send non-None value to a just-started generator");
      }
      g.started = true;
      if ( mode == 1 ) {
        g.done = true;
        this.raiseVal(sendv);
      }
    } else {
      const top = g.live[(g.live.length - 1)];
      const nres = top.res.length;
      let handed = false;
      if ( nres > 0 ) {
        const lr = top.res[(nres - 1)];
        if ( lr.kind == 2 ) {
          lr.send = sendv;
          lr.sendKind = mode;
          handed = true;
        }
      }
      if ( handed == false ) {
        const r = new PyGenRes();
        r.kind = mode;
        r.v = sendv;
        top.res.push(r);
      }
      // Loop start
      for ( const sl of g.live) {
        sl.used = 0;
        sl.resumed = true;
      }
    }
    g.depth = 0;
    if ( this.frames.length >= this.maxDepth ) {
      this.raise("RecursionError", "maximum recursion depth exceeded");
    }
    const fr = new PyFrame();
    fr.func = g.func;
    fr.__self = g.selfv;
    fr.name = g.name;
    fr.line = g.line;
    fr.callLine = this.curLine;
    this.frames.push(fr);
    const keepLine = this.curLine;
    const outer = this.curGen;
    this.curGen = g;
    g.running = true;
    let sig = 0;
    let failed = false;
    let err = this.NONE;
    const node = g.node;
    try {
      sig = this.execBlock(node.body, g.scope);
    } catch(e) {
      failed = true;
      err = this.takeException(( e.toString()));
    }
    g.running = false;
    this.curGen = outer;
    g.line = fr.line;
    this.frames.pop();
    this.curLine = keepLine;
    if ( failed ) {
      if ( this.suspending ) {
        this.suspending = false;
        return true;
      }
      g.done = true;
      let none = [];
      g.live = none;
      if ( this.isExcInst(err) && this.excMatches(err, this.excClass("StopIteration")) ) {
        const rx = this.makeExc(this.excClass("RuntimeError"), "generator raised StopIteration");
        rx.putAttr("__cause__", err);
        rx.putAttr("__context__", err);
        this.raiseVal(rx);
      }
      this.reraise(err);
    }
    g.done = true;
    let none2 = [];
    g.live = none2;
    g.out = this.NONE;
    if ( sig == 3 ) {
      g.out = this.retVal;
    }
    this.retVal = this.NONE;
    return false;
  };
  genStep (gv, sendv, mode) {
    const g = gv.gen;
    if ( this.genRun(gv, sendv, mode) ) {
      return g.out;
    }
    this.raiseVal(this.stopIter(g.out));
    return this.NONE;
  };
  genClose (gv) {
    const g = gv.gen;
    if ( g.started == false || g.done ) {
      g.done = true;
      return;
    }
    const exit = this.makeExc(this.excClass("GeneratorExit"), "");
    let yielded = false;
    let failed = false;
    let err = this.NONE;
    try {
      yielded = this.genRun(gv, exit, 1);
    } catch(e) {
      failed = true;
      err = this.takeException(( e.toString()));
    }
    if ( failed ) {
      if ( this.excMatches(err, this.excClass("GeneratorExit")) || this.excMatches(err, this.excClass("StopIteration")) ) {
        return;
      }
      this.reraise(err);
    }
    if ( yielded ) {
      this.raise("RuntimeError", "generator ignored GeneratorExit");
    }
  };
  iterOf (v) {
    if ( v.t == 23 || v.t == 17 ) {
      return v;
    }
    if ( v.t == 12 ) {
      const im = this.findMethod(v, "__iter__");
      if ( (typeof(im) !== "undefined" && im != null )  ) {
        const r = this.callMethodVal(im, v, []);
        if ( r.t == 23 || r.t == 17 ) {
          return r;
        }
        if ( r.t == 12 ) {
          if ( typeof(this.findMethod(r, "__next__")) === "undefined" ) {
            this.raise("TypeError", ("iter() returned non-iterator of type '" + this.typeName(r)) + "'");
          }
          return r;
        }
        return this.mkIter(this.iterValues(r));
      }
    }
    return this.mkIter(this.iterValues(v));
  };
  nextOpt (it) {
    let none;
    if ( it.t == 23 ) {
      if ( this.genRun(it, this.NONE, 0) ) {
        const g = it.gen;
        const out = g.out;
        return out;
      }
      return none;
    }
    if ( it.t == 17 ) {
      if ( it.mode.length > 0 ) {
        return this.lazyNext(it);
      }
      if ( it.pos < it.items.length ) {
        const v = it.items[it.pos];
        it.pos = it.pos + 1;
        return v;
      }
      return none;
    }
    const nx = this.findMethod(it, "__next__");
    if ( typeof(nx) === "undefined" ) {
      this.raise("TypeError", ("'" + this.typeName(it)) + "' object is not an iterator");
    }
    const got = this.callNext(nx, it);
    if ( got == this.ENDED ) {
      return none;
    }
    const r = got;
    return r;
  };
  callNext (nx, it) {
    let got = this.ENDED;
    try {
      got = this.callMethodVal(nx, it, []);
    } catch(e) {
      const ex = this.takeException(( e.toString()));
      if ( this.excMatches(ex, this.excClass("StopIteration")) == false ) {
        this.raiseVal(ex);
      }
    }
    return got;
  };
  yieldFrom (n, sc) {
    const g = this.curGen;
    const s = g.live[(g.live.length - 1)];
    let gr = new PyGenRes();
    if ( s.used < s.res.length ) {
      gr = s.res[s.used];
      if ( gr.kind == 0 ) {
        s.used = s.used + 1;
        return gr.v;
      }
      if ( gr.kind == 1 ) {
        s.used = s.used + 1;
        this.raiseVal(gr.v);
      }
    } else {
      const src = this.eval(n.k1, sc);
      gr.kind = 2;
      gr.v = this.iterOf(src);
      gr.send = this.NONE;
      gr.sendKind = 0;
      s.res.push(gr);
    }
    const sendv = gr.send;
    const mode = gr.sendKind;
    gr.send = this.NONE;
    gr.sendKind = 0;
    const sub = gr.v;
    if ( sub.t == 23 ) {
      const sg = sub.gen;
      if ( this.genRun(sub, sendv, mode) ) {
        g.out = sg.out;
        this.genSuspend();
      }
      gr.kind = 0;
      gr.v = sg.out;
      s.used = s.used + 1;
      return gr.v;
    }
    if ( mode == 1 ) {
      gr.kind = 0;
      s.used = s.used + 1;
      const th = this.getAttrOpt(sub, "throw");
      if ( typeof(th) === "undefined" ) {
        this.raiseVal(sendv);
      }
      let kn = [];
      let kv = [];
      g.out = this.callValue(th, [sendv], kn, kv);
      gr.kind = 2;
      s.used = s.used - 1;
      this.genSuspend();
    }
    if ( sendv.t != 0 && sub.t == 12 ) {
      const sd = this.getAttrOpt(sub, "send");
      if ( (typeof(sd) !== "undefined" && sd != null )  ) {
        let kn2 = [];
        let kv2 = [];
        let sent = this.NONE;
        let stopped = false;
        try {
          sent = this.callValue(sd, [sendv], kn2, kv2);
        } catch(e) {
          const ex = this.takeException(( e.toString()));
          if ( this.isExcInst(ex) && this.excMatches(ex, this.excClass("StopIteration")) ) {
            stopped = true;
            gr.v = this.orNone(this.getAttrOpt(ex, "value"));
          } else {
            this.raiseVal(ex);
          }
        }
        if ( stopped ) {
          gr.kind = 0;
          s.used = s.used + 1;
          return gr.v;
        }
        g.out = sent;
        this.genSuspend();
      }
    }
    const got = this.nextOpt(sub);
    if ( (typeof(got) !== "undefined" && got != null )  ) {
      g.out = got;
      this.genSuspend();
    }
    gr.kind = 0;
    gr.v = this.NONE;
    s.used = s.used + 1;
    return this.NONE;
  };
  isLazy (v) {
    if ( v.t == 23 || v.t == 12 ) {
      return true;
    }
    return v.t == 17 && v.mode.length > 0;
  };
  mkLazy (mode, name) {
    const v = new PyVal(17);
    v.id = this.newId();
    v.mode = mode;
    v.s = name;
    return v;
  };
  call1v (f, a) {
    let kn = [];
    let kv = [];
    return this.callValue(f, [a], kn, kv);
  };
  lazyNext (it) {
    let none;
    const m = it.mode;
    if ( m == "map" ) {
      let margs = [];
      // Loop start
      for ( const src of it.items) {
        const v = this.nextOpt(src);
        if ( typeof(v) === "undefined" ) {
          return none;
        }
        margs.push(v);
      }
      let kn = [];
      let kv = [];
      const mr = this.callValue(it.cls, margs, kn, kv);
      return mr;
    }
    if ( m == "filter" || m == "filterfalse" ) {
      while (true) {
        const fv = this.nextOpt(it.wrapped);
        if ( typeof(fv) === "undefined" ) {
          return none;
        }
        const x = fv;
        const f = it.cls;
        let ok = false;
        if ( f.t == 0 ) {
          ok = this.truthy(x);
        } else {
          ok = this.truthy(this.call1v(f, x));
        }
        if ( ok == (m == "filter") ) {
          return fv;
        }
      };
    }
    if ( m == "zip" ) {
      let tup = [];
      let k = 0;
      // Loop start
      for ( let i2 = 0; i2 < it.items.length; i2++) {
        var src2 = it.items[i2];
        const zv = this.nextOpt(src2);
        if ( typeof(zv) === "undefined" ) {
          if ( it.i == 1 ) {
            if ( k > 0 ) {
              this.raise("ValueError", (("zip() argument " + ((k + 1).toString())) + " is shorter than argument") + this.zipArgs(k));
            }
            let j = 1;
            while (j < it.items.length) {
              const more = this.nextOpt(it.items[j]);
              if ( (typeof(more) !== "undefined" && more != null )  ) {
                this.raise("ValueError", (("zip() argument " + ((j + 1).toString())) + " is longer than argument") + this.zipArgs(j));
              }
              j = j + 1;
            };
          }
          return none;
        }
        tup.push(zv);
        k = k + 1;
      }
      const zt = this.mkTuple(tup);
      return zt;
    }
    if ( m == "enumerate" ) {
      const ev = this.nextOpt(it.wrapped);
      if ( typeof(ev) === "undefined" ) {
        return none;
      }
      const ep = this.pair(this.mkInt(it.i), ev);
      it.i = it.i + 1;
      return ep;
    }
    if ( m == "count" ) {
      const cur = it.wrapped;
      it.wrapped = this.binop("+", it.wrapped, it.cls);
      return cur;
    }
    if ( m == "cycle" ) {
      if ( it.i == 0 ) {
        const cv = this.nextOpt(it.wrapped);
        if ( (typeof(cv) !== "undefined" && cv != null )  ) {
          it.vals.push(cv);
          return cv;
        }
        it.i = 1;
        it.pos = 0;
      }
      const nv = it.vals.length;
      if ( nv == 0 ) {
        return none;
      }
      const cy = it.vals[(it.pos % nv)];
      it.pos = it.pos + 1;
      return cy;
    }
    if ( m == "repeat" ) {
      if ( it.i == 0 ) {
        return none;
      }
      if ( it.i > 0 ) {
        it.i = it.i - 1;
      }
      const rv = it.wrapped;
      return rv;
    }
    if ( m == "islice" ) {
      if ( it.i2 >= 0 && it.i >= it.i2 ) {
        return none;
      }
      while (it.pos < it.i) {
        const skip = this.nextOpt(it.wrapped);
        if ( typeof(skip) === "undefined" ) {
          return none;
        }
        it.pos = it.pos + 1;
      };
      const iv = this.nextOpt(it.wrapped);
      if ( typeof(iv) === "undefined" ) {
        return none;
      }
      it.pos = it.pos + 1;
      it.i = it.i + it.i3;
      return iv;
    }
    if ( m == "takewhile" ) {
      if ( it.i == 1 ) {
        return none;
      }
      const tv = this.nextOpt(it.wrapped);
      if ( typeof(tv) === "undefined" ) {
        return none;
      }
      if ( this.truthy(this.call1v(it.cls, tv)) ) {
        return tv;
      }
      it.i = 1;
      return none;
    }
    if ( m == "dropwhile" ) {
      while (true) {
        const dv = this.nextOpt(it.wrapped);
        if ( typeof(dv) === "undefined" ) {
          return none;
        }
        if ( it.i == 0 && this.truthy(this.call1v(it.cls, dv)) ) {
          continue;
        }
        it.i = 1;
        return dv;
      };
    }
    if ( m == "chain" ) {
      while (true) {
        if ( (typeof(it.cur) !== "undefined" && it.cur != null )  ) {
          const chv = this.nextOpt(it.cur);
          if ( (typeof(chv) !== "undefined" && chv != null )  ) {
            return chv;
          }
          it.cur = none;
        }
        const nxt = this.nextOpt(it.wrapped);
        if ( typeof(nxt) === "undefined" ) {
          return none;
        }
        it.cur = this.iterOf(nxt);
      };
    }
    this.raise("RuntimeError", "internal error: iterator mode " + m);
    return none;
  };
  zipArgs (k) {
    if ( k == 1 ) {
      return " 1";
    }
    return "s 1-" + ((k).toString());
  };
  genexpBody (n) {
    if ( n.body.length > 0 ) {
      return n.body;
    }
    const y = new PyNode(37, n.line);
    y.k1 = n.k1;
    const st = new PyNode(40, n.line);
    st.k1 = y;
    let inner = [];
    inner.push(st);
    let i = n.kids.length - 1;
    while (i >= 0) {
      const cl = n.kids[i];
      let wrap = [];
      if ( cl.kind == 35 ) {
        const ifn = new PyNode(44, cl.line);
        ifn.k1 = cl.k1;
        ifn.body = inner;
        wrap.push(ifn);
      } else {
        const fn1 = new PyNode(46, cl.line);
        fn1.k1 = cl.k1;
        if ( i == 0 ) {
          const src = new PyNode(0, cl.line);
          src.s = ".0";
          fn1.k2 = src;
        } else {
          fn1.k2 = cl.k2;
        }
        fn1.body = inner;
        wrap.push(fn1);
      }
      inner = wrap;
      i = i - 1;
    };
    n.body = inner;
    return inner;
  };
  lazyGenexp (n, src, sc) {
    const cs = new PyScope();
    cs.parent = sc;
    cs.globals = this.moduleScope(sc);
    cs.decl["__comp__"] = 3;
    cs.vars[".0"] = this.iterOf(src);
    this.genexpBody(n);
    const gen = new PyGen();
    gen.node = n;
    gen.scope = cs;
    gen.name = "<genexpr>";
    gen.line = n.line;
    const g = new PyVal(23);
    g.gen = gen;
    g.s = "<genexpr>";
    return g;
  };
  execBlockGen (body, sc) {
    const g = this.curGen;
    let i = 0;
    const resumed = this.genResuming(g);
    const s = this.genEnter(g);
    if ( resumed ) {
      s.resumed = false;
      i = s.i;
    } else {
      s.i = 0;
    }
    const nb = body.length;
    while (i < nb) {
      const sig = this.exec(body[i], sc);
      if ( sig != 0 ) {
        this.genLeave(g);
        return sig;
      }
      i = i + 1;
      s.i = i;
      this.clearRes(s);
    };
    this.genLeave(g);
    return 0;
  };
  matchCase (n, sc) {
    const subj = this.eval(n.k1, sc);
    // Loop start
    for ( let i = 0; i < n.kids.length; i++) {
      var cs = n.kids[i];
      if ( this.matchPat(cs.k1, subj, sc) ) {
        if ( (typeof(cs.k2) !== "undefined" && cs.k2 != null )  ) {
          if ( this.truthy(this.eval(cs.k2, sc)) == false ) {
            continue;
          }
        }
        return i;
      }
    }
    return -1;
  };
  execMatchGen (n, sc) {
    const g = this.curGen;
    let ci = -1;
    if ( this.genResuming(g) ) {
      const rs = this.genEnter(g);
      rs.resumed = false;
      ci = rs.i;
    } else {
      ci = this.matchCase(n, sc);
      const ns = this.genEnter(g);
      ns.i = ci;
    }
    let sig = 0;
    if ( ci >= 0 ) {
      const cs = n.kids[ci];
      sig = this.execBlock(cs.body, sc);
    }
    this.genLeave(g);
    return sig;
  };
  isSeqSubject (v) {
    if ( (v.t == 5 || v.t == 6) || v.t == 15 ) {
      return true;
    }
    if ( v.t == 12 && ((typeof(v.cls) !== "undefined" && v.cls != null ) ) ) {
      const c = v.cls;
      return (((typeof(this.classLookup(c, "__getitem__")) !== "undefined" && this.classLookup(c, "__getitem__") != null ) ) && ((typeof(this.classLookup(c, "__len__")) !== "undefined" && this.classLookup(c, "__len__") != null ) )) && (typeof(this.classLookup(c, "keys")) === "undefined");
    }
    return false;
  };
  matchPat (p, v, sc) {
    const k = p.s;
    if ( k == "wild" ) {
      return true;
    }
    if ( k == "capture" ) {
      this.store(p.strs[0], v, sc);
      return true;
    }
    if ( k == "value" ) {
      const x = this.eval(p.k1, sc);
      if ( p.i == 1 ) {
        return this.same(v, x);
      }
      return this.eq(v, x);
    }
    if ( k == "or" ) {
      // Loop start
      for ( const alt of p.kids) {
        if ( this.matchPat(alt, v, sc) ) {
          return true;
        }
      }
      return false;
    }
    if ( k == "as" ) {
      if ( this.matchPat(p.k1, v, sc) ) {
        this.store(p.strs[0], v, sc);
        return true;
      }
      return false;
    }
    if ( k == "seq" ) {
      if ( this.isSeqSubject(v) == false ) {
        return false;
      }
      const items = this.iterValues(v);
      const n = items.length;
      const np = p.kids.length;
      let star = -1;
      // Loop start
      for ( let si = 0; si < p.kids.length; si++) {
        var sp = p.kids[si];
        if ( sp.s == "star" ) {
          star = si;
        }
      }
      if ( star < 0 ) {
        if ( n != np ) {
          return false;
        }
        // Loop start
        for ( let si2 = 0; si2 < p.kids.length; si2++) {
          var sp2 = p.kids[si2];
          if ( this.matchPat(sp2, items[si2], sc) == false ) {
            return false;
          }
        }
        return true;
      }
      if ( n < np - 1 ) {
        return false;
      }
      const after = (np - star) - 1;
      let j = 0;
      while (j < star) {
        if ( this.matchPat(p.kids[j], items[j], sc) == false ) {
          return false;
        }
        j = j + 1;
      };
      let j2 = 0;
      while (j2 < after) {
        if ( this.matchPat(p.kids[((star + 1) + j2)], items[((n - after) + j2)], sc) == false ) {
          return false;
        }
        j2 = j2 + 1;
      };
      const stp = p.kids[star];
      const sname = stp.strs[0];
      if ( sname.length > 0 ) {
        let mid = [];
        let m = star;
        while (m < n - after) {
          mid.push(items[m]);
          m = m + 1;
        };
        this.store(sname, this.mkList(mid), sc);
      }
      return true;
    }
    if ( k == "map" ) {
      let isMap = v.t == 7;
      if ( v.t == 12 && ((typeof(v.cls) !== "undefined" && v.cls != null ) ) ) {
        isMap = ((typeof(this.classLookup(v.cls, "keys")) !== "undefined" && this.classLookup(v.cls, "keys") != null ) ) && ((typeof(this.classLookup(v.cls, "__getitem__")) !== "undefined" && this.classLookup(v.cls, "__getitem__") != null ) );
      }
      if ( isMap == false ) {
        return false;
      }
      let used = [];
      // Loop start
      for ( let ki = 0; ki < p.kids.length; ki++) {
        var kn = p.kids[ki];
        const key = this.eval(kn, sc);
        let got;
        if ( v.t == 7 ) {
          got = this.dictGet(v, key);
        } else {
          if ( this.contains(v, key) ) {
            got = this.getItem(v, key);
          }
        }
        if ( typeof(got) === "undefined" ) {
          return false;
        }
        if ( this.matchPat(p.orelse[ki], got, sc) == false ) {
          return false;
        }
        used.push(key);
      }
      if ( p.strs.length > 0 ) {
        const rest = this.mkDict();
        // Loop start
        for ( let rki = 0; rki < this.iterValues(v).length; rki++) {
          var rk = this.iterValues(v)[rki];
          let taken = false;
          // Loop start
          for ( const uk of used) {
            if ( this.eq(uk, rk) ) {
              taken = true;
            }
          }
          if ( taken == false ) {
            this.dictSet(rest, rk, this.getItem(v, rk));
          }
        }
        this.store(p.strs[0], rest, sc);
      }
      return true;
    }
    if ( k == "class" ) {
      const cls = this.eval(p.k1, sc);
      if ( cls.t != 11 && cls.t != 10 ) {
        this.raise("TypeError", "called match pattern must be a class");
      }
      if ( this.bi.isInstance(v, cls) == false ) {
        return false;
      }
      const npos = p.kids.length;
      if ( npos > 0 ) {
        let self1 = false;
        if ( cls.t == 10 ) {
          self1 = true;
        }
        if ( (cls.t == 11 && cls.bt.length > 0) && (typeof(this.classLookup(cls, "__match_args__")) === "undefined") ) {
          self1 = true;
        }
        if ( self1 ) {
          if ( npos > 1 ) {
            this.raise("TypeError", ((cls.s + "() accepts 1 positional sub-pattern (") + ((npos).toString())) + " given)");
          }
          if ( this.matchPat(p.kids[0], v, sc) == false ) {
            return false;
          }
        } else {
          let ma = this.getAttrOpt(cls, "__match_args__");
          if ( typeof(ma) === "undefined" ) {
            ma = this.getAttrOpt(cls, "_fields");
          }
          let margs = [];
          if ( (typeof(ma) !== "undefined" && ma != null )  ) {
            margs = this.iterValues(ma);
          }
          if ( npos > margs.length ) {
            this.raise("TypeError", ((((((cls.s + "() accepts ") + ((margs.length).toString())) + " positional sub-pattern") + this.plural(margs.length)) + " (") + ((npos).toString())) + " given)");
          }
          // Loop start
          for ( let pi = 0; pi < p.kids.length; pi++) {
            var pp = p.kids[pi];
            const an = this.toStr(margs[pi]);
            const av = this.getAttrOpt(v, an);
            if ( typeof(av) === "undefined" ) {
              return false;
            }
            if ( this.matchPat(pp, av, sc) == false ) {
              return false;
            }
          }
        }
      }
      // Loop start
      for ( let kwi = 0; kwi < p.names.length; kwi++) {
        var kw = p.names[kwi];
        const kv = this.getAttrOpt(v, kw);
        if ( typeof(kv) === "undefined" ) {
          return false;
        }
        if ( this.matchPat(p.orelse[kwi], kv, sc) == false ) {
          return false;
        }
      }
      return true;
    }
    return false;
  };
  execIfGen (n, sc) {
    const g = this.curGen;
    let br = 0;
    if ( this.genResuming(g) ) {
      const rs = this.genEnter(g);
      rs.resumed = false;
      br = rs.i;
    } else {
      const c = this.eval(n.k1, sc);
      if ( this.truthy(c) == false ) {
        br = 1;
      }
      const ns = this.genEnter(g);
      ns.i = br;
    }
    let sig = 0;
    if ( br == 0 ) {
      sig = this.execBlock(n.body, sc);
    } else {
      sig = this.execBlock(n.orelse, sc);
    }
    this.genLeave(g);
    return sig;
  };
  execWhileGen (n, sc) {
    const g = this.curGen;
    const resumed = this.genResuming(g);
    const s = this.genEnter(g);
    if ( resumed ) {
      s.resumed = false;
    } else {
      s.i = 0;
    }
    while (true) {
      if ( s.i == 0 ) {
        const wc = this.eval(n.k1, sc);
        this.clearRes(s);
        if ( this.truthy(wc) ) {
          s.i = 1;
        } else {
          s.i = 2;
        }
      }
      if ( s.i == 2 ) {
        const os = this.execBlock(n.orelse, sc);
        this.genLeave(g);
        return os;
      }
      const ws = this.execBlock(n.body, sc);
      if ( ws == 1 ) {
        this.genLeave(g);
        return 0;
      }
      if ( ws == 3 ) {
        this.genLeave(g);
        return 3;
      }
      s.i = 0;
      this.tick();
    };
    return 0;
  };
  loopNext (s) {
    let none;
    const it = s.v;
    if ( it.t == 15 ) {
      if ( s.j < this.rangeLen(it) ) {
        const x = it.i + s.j * it.i3;
        s.j = s.j + 1;
        const rv = this.mkInt(x);
        return rv;
      }
      return none;
    }
    if ( it.t == 5 ) {
      if ( s.j < it.items.length ) {
        const lv = it.items[s.j];
        s.j = s.j + 1;
        return lv;
      }
      return none;
    }
    return this.nextOpt(it);
  };
  execForGen (n, sc) {
    const g = this.curGen;
    let resumed = this.genResuming(g);
    let s = new PyGenSlot();
    if ( resumed ) {
      s = this.genEnter(g);
      s.resumed = false;
    } else {
      const src = this.eval(n.k2, sc);
      let it = src;
      if ( src.t != 15 && src.t != 5 ) {
        it = this.iterOf(src);
      }
      s = this.genEnter(g);
      s.v = it;
      s.j = 0;
      s.i = 1;
    }
    const tg = n.k1;
    while (true) {
      if ( s.i == 2 ) {
        const os = this.execBlock(n.orelse, sc);
        this.genLeave(g);
        return os;
      }
      if ( resumed ) {
        resumed = false;
      } else {
        const item = this.loopNext(s);
        if ( typeof(item) === "undefined" ) {
          s.i = 2;
          continue;
        }
        this.assign(tg, item, sc);
      }
      const sig = this.execBlock(n.body, sc);
      if ( sig == 1 ) {
        this.genLeave(g);
        return 0;
      }
      if ( sig == 3 ) {
        this.genLeave(g);
        return 3;
      }
      this.tick();
    };
    return 0;
  };
  execTryGen (n, sc) {
    const g = this.curGen;
    const resumed = this.genResuming(g);
    const s = this.genEnter(g);
    if ( resumed ) {
      s.resumed = false;
    } else {
      s.i = 0;
    }
    const myDepth = g.depth;
    if ( s.i == 0 ) {
      let failed = false;
      let caught = this.NONE;
      let sig = 0;
      try {
        sig = this.execBlock(n.body, sc);
      } catch(e) {
        failed = true;
        caught = this.takeException(( e.toString()));
      }
      if ( failed ) {
        if ( this.suspending ) {
          this.resuspend();
        }
        this.genTruncate(g, myDepth);
        s.i = 3;
        s.w = caught;
        s.sig = 0;
        // Loop start
        for ( let hi = 0; hi < n.handlers.length; hi++) {
          var h = n.handlers[hi];
          let matches = false;
          if ( typeof(h.k1) === "undefined" ) {
            matches = true;
          } else {
            let hfail = false;
            let herr = this.NONE;
            try {
              const hv = this.eval(h.k1, sc);
              matches = this.excMatches(caught, hv);
            } catch(e) {
              hfail = true;
              herr = this.takeException(( e.toString()));
            }
            if ( hfail ) {
              s.w = herr;
              break;
            }
          }
          if ( matches ) {
            let none;
            s.w = none;
            s.i = 1;
            s.j = hi;
            s.v = caught;
            if ( h.s.length > 0 ) {
              this.store(h.s, caught, sc);
            }
            break;
          }
        }
      } else {
        s.sig = sig;
        if ( sig == 0 && n.orelse.length > 0 ) {
          s.i = 2;
        } else {
          s.i = 3;
        }
      }
      s.keep = this.retVal;
    }
    if ( s.i == 1 ) {
      const h2 = n.handlers[s.j];
      this.handling.push(s.v);
      let bfail = false;
      let berr = this.NONE;
      let bsig = 0;
      try {
        bsig = this.execBlock(h2.body, sc);
      } catch(e) {
        bfail = true;
        berr = this.takeException(( e.toString()));
      }
      this.handling.pop();
      if ( bfail && this.suspending ) {
        this.resuspend();
      }
      if ( h2.s.length > 0 ) {
        delete sc.vars[h2.s];
      }
      s.sig = bsig;
      if ( bfail ) {
        this.genTruncate(g, myDepth);
        if ( (this.isExcInst(berr) && ( typeof(berr.attrs["__context__"] ) != "undefined" && Object.prototype.hasOwnProperty.call(berr.attrs, "__context__") ) == false) && this.same(berr, s.v) == false ) {
          berr.putAttr("__context__", s.v);
        }
        s.w = berr;
        s.sig = 0;
      }
      s.i = 3;
      s.keep = this.retVal;
    }
    if ( s.i == 2 ) {
      let ofail = false;
      let oerr = this.NONE;
      let osig = 0;
      try {
        osig = this.execBlock(n.orelse, sc);
      } catch(e) {
        ofail = true;
        oerr = this.takeException(( e.toString()));
      }
      if ( ofail && this.suspending ) {
        this.resuspend();
      }
      s.sig = osig;
      if ( ofail ) {
        this.genTruncate(g, myDepth);
        s.w = oerr;
        s.sig = 0;
      }
      s.i = 3;
      s.keep = this.retVal;
    }
    if ( n.final.length > 0 ) {
      const fsig = this.execBlock(n.final, sc);
      if ( fsig != 0 ) {
        this.genLeave(g);
        return fsig;
      }
      this.retVal = s.keep;
    }
    this.genLeave(g);
    if ( (typeof(s.w) !== "undefined" && s.w != null )  ) {
      this.reraise(s.w);
    }
    return s.sig;
  };
  execWithGen (n, idx, sc) {
    if ( idx >= n.kids.length ) {
      return this.execBlock(n.body, sc);
    }
    const g = this.curGen;
    let ctx = this.NONE;
    if ( this.genResuming(g) ) {
      const rs = this.genEnter(g);
      rs.resumed = false;
      ctx = rs.v;
    } else {
      const item = n.kids[idx];
      ctx = this.eval(item.k1, sc);
      const ns = this.genEnter(g);
      ns.v = ctx;
      const entered = this.callAttr(ctx, "__enter__", []);
      if ( (typeof(item.k2) !== "undefined" && item.k2 != null )  ) {
        this.assign(item.k2, entered, sc);
      }
    }
    const myDepth = g.depth;
    let sig = 0;
    let failed = false;
    let err = this.NONE;
    try {
      sig = this.execWithGen(n, (idx + 1), sc);
    } catch(e) {
      failed = true;
      err = this.takeException(( e.toString()));
    }
    if ( failed ) {
      if ( this.suspending ) {
        this.resuspend();
      }
      this.genTruncate(g, myDepth);
      this.genLeave(g);
      let cls = this.NONE;
      if ( (typeof(err.cls) !== "undefined" && err.cls != null )  ) {
        cls = err.cls;
      }
      const keep = this.callAttr(ctx, "__exit__", [cls, err, this.NONE]);
      if ( this.truthy(keep) == false ) {
        this.reraise(err);
      }
      return 0;
    }
    this.genLeave(g);
    const keepRet = this.retVal;
    this.callAttr(ctx, "__exit__", [this.NONE, this.NONE, this.NONE]);
    this.retVal = keepRet;
    return sig;
  };
  execAug (n, sc) {
    const t = n.k1;
    if ( t.kind == 0 ) {
      const cur = this.lookup(t.s, sc);
      const rhs = this.eval(n.k2, sc);
      this.store(t.s, this.inplace(n.s, cur, rhs), sc);
      return;
    }
    if ( t.kind == 27 ) {
      const obj = this.eval(t.k1, sc);
      const cur2 = this.getAttr(obj, t.s);
      const rhs2 = this.eval(n.k2, sc);
      this.setAttr(obj, t.s, this.inplace(n.s, cur2, rhs2));
      return;
    }
    if ( t.kind == 28 ) {
      const obj2 = this.eval(t.k1, sc);
      const idx = this.eval(t.k2, sc);
      const cur3 = this.getItem(obj2, idx);
      const rhs3 = this.eval(n.k2, sc);
      this.setItem(obj2, idx, this.inplace(n.s, cur3, rhs3));
      return;
    }
    this.raise("SyntaxError", "illegal expression for augmented assignment");
  };
  execDel (t, sc) {
    if ( t.kind == 0 ) {
      this.deleteName(t.s, sc);
      return;
    }
    if ( t.kind == 28 ) {
      const obj = this.eval(t.k1, sc);
      const idx = this.eval(t.k2, sc);
      this.delItem(obj, idx);
      return;
    }
    if ( t.kind == 27 ) {
      const obj2 = this.eval(t.k1, sc);
      this.delAttr(obj2, t.s);
      return;
    }
    if ( t.kind == 10 || t.kind == 11 ) {
      // Loop start
      for ( const k of t.kids) {
        this.execDel(k, sc);
      }
      return;
    }
    this.raise("SyntaxError", "cannot delete expression");
  };
  execFor (n, sc) {
    const src = this.eval(n.k2, sc);
    const tg = n.k1;
    if ( src.t == 15 ) {
      const cnt = this.rangeLen(src);
      let j = 0;
      let x = src.i;
      while (j < cnt) {
        this.assign(tg, this.mkInt(x), sc);
        const sig = this.execBlock(n.body, sc);
        if ( sig == 1 ) {
          return 0;
        }
        if ( sig == 3 ) {
          return 3;
        }
        x = x + src.i3;
        j = j + 1;
      };
      return this.execBlock(n.orelse, sc);
    }
    if ( src.t == 5 ) {
      let k = 0;
      while (k < src.items.length) {
        this.assign(tg, src.items[k], sc);
        const sig2 = this.execBlock(n.body, sc);
        if ( sig2 == 1 ) {
          return 0;
        }
        if ( sig2 == 3 ) {
          return 3;
        }
        k = k + 1;
      };
      return this.execBlock(n.orelse, sc);
    }
    if ( src.t == 17 && src.mode.length == 0 ) {
      while (src.pos < src.items.length) {
        const item = src.items[src.pos];
        src.pos = src.pos + 1;
        this.assign(tg, item, sc);
        const sig3 = this.execBlock(n.body, sc);
        if ( sig3 == 1 ) {
          return 0;
        }
        if ( sig3 == 3 ) {
          return 3;
        }
      };
      return this.execBlock(n.orelse, sc);
    }
    if ( this.isLazy(src) ) {
      const it = this.iterOf(src);
      while (true) {
        const got = this.nextOpt(it);
        if ( typeof(got) === "undefined" ) {
          break;
        }
        this.assign(tg, got, sc);
        const sig5 = this.execBlock(n.body, sc);
        if ( sig5 == 1 ) {
          return 0;
        }
        if ( sig5 == 3 ) {
          return 3;
        }
      };
      return this.execBlock(n.orelse, sc);
    }
    const vals = this.iterValues(src);
    // Loop start
    for ( const v of vals) {
      this.assign(tg, v, sc);
      const sig4 = this.execBlock(n.body, sc);
      if ( sig4 == 1 ) {
        return 0;
      }
      if ( sig4 == 3 ) {
        return 3;
      }
    }
    return this.execBlock(n.orelse, sc);
  };
  applyDecorators (decos, f, sc) {
    let v = f;
    let i = decos.length - 1;
    while (i >= 0) {
      const d = this.eval(decos[i], sc);
      let kn = [];
      let kv = [];
      v = this.callValue(d, [v], kn, kv);
      i = i - 1;
    };
    return v;
  };
  execClass (n, sc) {
    let bvals = [];
    let ikn = [];
    let ikv = [];
    let meta;
    // Loop start
    for ( const b of n.kids) {
      if ( b.kind == 24 ) {
        const kval = this.eval(b.k1, sc);
        if ( b.s == "metaclass" ) {
          meta = kval;
        } else {
          ikn.push(b.s);
          ikv.push(kval);
        }
        continue;
      }
      bvals.push(this.eval(b, sc));
    }
    let needEntries = false;
    // Loop start
    for ( const eb of bvals) {
      if ( eb.t != 11 && eb.t != 10 ) {
        needEntries = true;
      }
    }
    if ( needEntries ) {
      const orig = this.mkTuple(bvals);
      let resolved = [];
      // Loop start
      for ( const eb2 of bvals) {
        if ( eb2.t != 11 && eb2.t != 10 ) {
          const me = this.getAttrOpt(eb2, "__mro_entries__");
          if ( (typeof(me) !== "undefined" && me != null )  ) {
            const ents = this.callValue(me, [orig], [], []);
            // Loop start
            for ( let evi = 0; evi < this.iterValues(ents).length; evi++) {
              var ev = this.iterValues(ents)[evi];
              resolved.push(ev);
            }
            continue;
          }
        }
        resolved.push(eb2);
      }
      bvals = resolved;
    }
    if ( typeof(meta) === "undefined" ) {
      // Loop start
      for ( const mb0 of bvals) {
        if ( mb0.t == 11 && ((typeof(mb0.cls) !== "undefined" && mb0.cls != null ) ) ) {
          const bm = mb0.cls;
          if ( (typeof(meta) === "undefined") || this.isSubclass(bm, meta) ) {
            meta = bm;
          }
        }
      }
    }
    if ( (typeof(meta) !== "undefined" && meta != null )  ) {
      const mt = meta;
      if ( mt.t == 11 && mt.bt == "type" ) {
        this.execClassMeta(n, sc, mt, bvals, ikn, ikv);
        return;
      }
      if ( mt.t != 10 || mt.s != "type" ) {
        const mres = this.callValue(
          mt,
          [this.mkStr(n.s), this.mkTuple(bvals), this.classNamespace(n, sc)],
          ikn,
          ikv
        );
        this.store(n.s, this.applyDecorators(n.decos, mres, sc), sc);
        return;
      }
    }
    const cls = new PyVal(11);
    cls.s = n.s;
    // Loop start
    for ( const bv0 of bvals) {
      this.addBase(cls, bv0);
    }
    this.computeMro(cls);
    const body = new PyScope();
    body.parent = sc;
    body.globals = this.moduleScope(sc);
    body.isClass = true;
    let qn = n.qual;
    if ( qn.length == 0 ) {
      qn = n.s;
    }
    cls.putAttr("__module__", this.moduleName(sc));
    cls.putAttr("__qualname__", this.mkStr(qn));
    cls.putAttr("__doc__", this.docOf(n.body));
    body.put("__qualname__", this.mkStr(qn));
    body.put("__module__", this.moduleName(sc));
    if ( n.tparams.length > 0 ) {
      cls.putAttr("__type_params__", this.bindTypeParams(n.tparams, body));
    }
    this.setupAnn(n.body, body);
    this.execBlock(n.body, body);
    // Loop start
    for ( const tp of n.tparams) {
      delete body.vars[tp];
    }
    const ks = body.orderedKeys();
    // Loop start
    for ( const k of ks) {
      const v = ( Object.prototype.hasOwnProperty.call(body.vars, k) ? body.vars[k] : undefined );
      cls.putAttr(k, v);
      this.setOwner(v, cls);
    }
    this.finishClass(cls, ikn, ikv);
    const cv = this.applyDecorators(n.decos, cls, sc);
    this.store(n.s, cv, sc);
  };
  classNamespace (n, sc) {
    const body = new PyScope();
    body.parent = sc;
    body.globals = this.moduleScope(sc);
    body.isClass = true;
    let qn = n.qual;
    if ( qn.length == 0 ) {
      qn = n.s;
    }
    body.put("__module__", this.moduleName(sc));
    body.put("__qualname__", this.mkStr(qn));
    const doc = this.docOf(n.body);
    if ( doc.t != 0 ) {
      body.put("__doc__", doc);
    }
    if ( n.tparams.length > 0 ) {
      body.put("__type_params__", this.bindTypeParams(n.tparams, body));
    }
    this.setupAnn(n.body, body);
    this.execBlock(n.body, body);
    // Loop start
    for ( const tp of n.tparams) {
      delete body.vars[tp];
    }
    const d = this.mkDict();
    // Loop start
    for ( let j = 0; j < body.orderedKeys().length; j++) {
      var k = body.orderedKeys()[j];
      this.dictSet(
        d,
        this.mkStr(k),
        ( Object.prototype.hasOwnProperty.call(body.vars, k) ? body.vars[k] : undefined )
      );
    }
    return d;
  };
  execClassMeta (n, sc, meta, bases, kn, kv) {
    let ns = this.classNamespace(n, sc);
    const bt = this.mkTuple(bases);
    const pre = this.classLookup(meta, "__prepare__");
    if ( (typeof(pre) !== "undefined" && pre != null )  ) {
      let pf = pre;
      if ( pf.t == 21 || pf.t == 20 ) {
        pf = pf.wrapped;
      }
      let pargs = [];
      const __REGx1 = pre;
      if ( __REGx1.t == 21 ) {
        pargs.push(meta);
      }
      pargs.push(this.mkStr(n.s));
      pargs.push(bt);
      const pm = this.callValue(pf, pargs, kn, kv);
      if ( pm.t != 7 ) {
        // Loop start
        for ( let nki = 0; nki < ns.items.length; nki++) {
          var nk = ns.items[nki];
          this.setItem(pm, nk, ns.vals[nki]);
        }
        ns = pm;
      } else {
        // Loop start
        for ( let nki2 = 0; nki2 < ns.items.length; nki2++) {
          var nk2 = ns.items[nki2];
          this.setItem(pm, nk2, ns.vals[nki2]);
        }
        ns = pm;
      }
    }
    const made = this.metaInstantiate(meta, [this.mkStr(n.s), bt, ns], kn, kv);
    this.store(n.s, this.applyDecorators(n.decos, made, sc), sc);
  };
  metaInstantiate (meta, args, kn, kv) {
    if ( args.length == 1 && kn.length == 0 ) {
      return this.bi.typeOf(args[0]);
    }
    const nw = this.classLookup(meta, "__new__");
    let made = this.NONE;
    const __REGx1 = this.orNone(nw);
    const __REGx2 = this.orNone(nw);
    if ( ((typeof(nw) !== "undefined" && nw != null ) ) && (__REGx1.t == 9 || __REGx2.t == 20) ) {
      let nf = nw;
      if ( nf.t == 20 ) {
        nf = nf.wrapped;
      }
      let all0 = [];
      all0.push(meta);
      // Loop start
      for ( const a0 of args) {
        all0.push(a0);
      }
      made = this.callValue(nf, all0, kn, kv);
    } else {
      made = this.typeNew(meta, args, kn, kv);
    }
    if ( (made.t == 11 && ((typeof(made.cls) !== "undefined" && made.cls != null ) )) && this.isSubclass(made.cls, meta) ) {
      const init = this.classLookup(meta, "__init__");
      const __REGx3 = this.orNone(init);
      if ( ((typeof(init) !== "undefined" && init != null ) ) && __REGx3.t == 9 ) {
        let all = [];
        all.push(made);
        // Loop start
        for ( const a of args) {
          all.push(a);
        }
        this.callValue(init, all, kn, kv);
      }
    }
    return made;
  };
  typeNew (meta, args, kn, kv) {
    if ( args.length != 3 ) {
      this.raise("TypeError", "type.__new__() takes exactly 3 arguments");
    }
    const tname = args[0];
    const tbases = args[1];
    const tdict = args[2];
    if ( tname.t != 4 ) {
      this.raise("TypeError", "type.__new__() argument 1 must be str");
    }
    if ( tbases.t != 6 ) {
      this.raise("TypeError", "type.__new__() argument 2 must be tuple");
    }
    const ncls = new PyVal(11);
    ncls.s = tname.s;
    // Loop start
    for ( const tb of tbases.items) {
      this.addBase(ncls, tb);
    }
    this.computeMro(ncls);
    if ( meta.t == 11 && meta.bt == "type" ) {
      ncls.cls = meta;
    }
    ncls.putAttr("__module__", this.mkStr("__main__"));
    ncls.putAttr("__qualname__", tname);
    ncls.putAttr("__doc__", this.NONE);
    // Loop start
    for ( let tki = 0; tki < this.iterValues(tdict).length; tki++) {
      var tk = this.iterValues(tdict)[tki];
      if ( tk.t != 4 ) {
        continue;
      }
      const tv = this.getItem(tdict, tk);
      ncls.putAttr(tk.s, tv);
      this.setOwner(tv, ncls);
    }
    this.finishClass(ncls, kn, kv);
    return ncls;
  };
  addBase (cls, bv) {
    if ( bv.t == 10 && bv.s == "object" ) {
      return;
    }
    if ( bv.t == 10 ) {
      const okb = ["int", "float", "complex", "str", "list", "tuple", "dict", "set", "frozenset", "type"];
      let isOk = false;
      // Loop start
      for ( const ob of okb) {
        if ( ob == bv.s ) {
          isOk = true;
        }
      }
      if ( isOk == false ) {
        this.raise("TypeError", ("type '" + bv.s) + "' is not an acceptable base type");
      }
      if ( cls.bt.length > 0 && cls.bt != bv.s ) {
        this.raise("TypeError", "multiple bases have instance lay-out conflict");
      }
      cls.bt = bv.s;
      return;
    }
    if ( bv.t != 11 ) {
      this.raise("TypeError", "bases must be types");
    }
    if ( bv.mode == "native" ) {
      cls.bt = "native";
      cls.wrapped = bv;
    }
    if ( bv.bt.length > 0 && cls.bt.length == 0 ) {
      cls.bt = bv.bt;
      cls.wrapped = bv.wrapped;
    }
    cls.bases.push(bv);
    if ( bv.isExc ) {
      cls.isExc = true;
    }
  };
  finishClass (cls, ikn, ikv) {
    // Loop start
    for ( let j2 = 0; j2 < cls.attrKeys().length; j2++) {
      var k2 = cls.attrKeys()[j2];
      const v2 = ( Object.prototype.hasOwnProperty.call(cls.attrs, k2) ? cls.attrs[k2] : undefined );
      if ( v2.t == 12 ) {
        const sn = this.findMethod(v2, "__set_name__");
        if ( (typeof(sn) !== "undefined" && sn != null )  ) {
          this.callMethodVal(sn, v2, [cls, this.mkStr(k2)]);
        }
      }
    }
    let isub;
    // Loop start
    for ( let mbi = 0; mbi < cls.vals.length; mbi++) {
      var mb = cls.vals[mbi];
      if ( ((typeof(isub) === "undefined") && this.same(mb, cls) == false) && ( typeof(mb.attrs["__init_subclass__"] ) != "undefined" && Object.prototype.hasOwnProperty.call(mb.attrs, "__init_subclass__") ) ) {
        isub = ( Object.prototype.hasOwnProperty.call(mb.attrs, "__init_subclass__") ? mb.attrs["__init_subclass__"] : undefined );
      }
    }
    if ( (typeof(isub) !== "undefined" && isub != null )  ) {
      let isf = isub;
      if ( isf.t == 21 || isf.t == 20 ) {
        isf = isf.wrapped;
      }
      this.callValue(isf, [cls], ikn, ikv);
    } else {
      if ( ikn.length > 0 ) {
        this.raise("TypeError", cls.s + ".__init_subclass__() takes no keyword arguments");
      }
    }
  };
  setOwner (v, cls) {
    if ( v.t == 9 && (typeof(v.cls) === "undefined") ) {
      v.cls = cls;
    }
    if ( (v.t == 20 || v.t == 21) && ((typeof(v.wrapped) !== "undefined" && v.wrapped != null ) ) ) {
      this.setOwner(v.wrapped, cls);
    }
    if ( v.t == 19 ) {
      if ( (typeof(v.wrapped) !== "undefined" && v.wrapped != null )  ) {
        this.setOwner(v.wrapped, cls);
      }
      if ( (typeof(v.setter) !== "undefined" && v.setter != null )  ) {
        this.setOwner(v.setter, cls);
      }
    }
  };
  execExceptStar (n, sc, caught) {
    const m = this.importModule("_exceptiongroup");
    let nk = [];
    let nv = [];
    let rest = this.callValue(this.getAttr(m, "_as_group"), [caught], nk, nv);
    let raised = [];
    // Loop start
    for ( const h of n.handlers) {
      if ( rest.t == 0 ) {
        break;
      }
      const hv = this.eval(h.k1, sc);
      const pair = this.callValue(this.getAttr(rest, "split"), [hv], nk, nv);
      const match = pair.items[0];
      rest = pair.items[1];
      if ( match.t == 0 ) {
        continue;
      }
      if ( h.s.length > 0 ) {
        this.store(h.s, match, sc);
      }
      this.handling.push(match);
      let bfail = false;
      let berr = this.NONE;
      try {
        this.execBlock(h.body, sc);
      } catch(e) {
        bfail = true;
        berr = this.takeException(( e.toString()));
      }
      this.handling.pop();
      if ( h.s.length > 0 ) {
        delete sc.vars[h.s];
      }
      if ( bfail ) {
        if ( this.same(berr, match) == false ) {
          raised.push(berr);
        }
        if ( this.same(berr, match) ) {
          raised.push(match);
        }
      }
    }
    return this.callValue(
      this.getAttr(m, "_finish"),
      [caught, rest, this.mkList(raised)],
      nk,
      nv
    );
  };
  execTry (n, sc) {
    let sig = 0;
    let pending;
    let failed = false;
    let caught = this.NONE;
    try {
      sig = this.execBlock(n.body, sc);
    } catch(e) {
      failed = true;
      caught = this.takeException(( e.toString()));
    }
    if ( failed && n.i == 1 ) {
      const fin = this.execExceptStar(n, sc, caught);
      if ( fin.t != 0 ) {
        pending = fin;
      }
    }
    if ( failed && n.i != 1 ) {
      pending = caught;
      let handled = false;
      // Loop start
      for ( const h of n.handlers) {
        if ( handled ) {
          break;
        }
        let matches = false;
        if ( typeof(h.k1) === "undefined" ) {
          matches = true;
        } else {
          let hv = this.NONE;
          let hfail = false;
          let herr = this.NONE;
          try {
            hv = this.eval(h.k1, sc);
            matches = this.excMatches(caught, hv);
          } catch(e) {
            hfail = true;
            herr = this.takeException(( e.toString()));
          }
          if ( hfail ) {
            pending = herr;
            break;
          }
        }
        if ( matches ) {
          handled = true;
          let none;
          pending = none;
          if ( h.s.length > 0 ) {
            this.store(h.s, caught, sc);
          }
          this.handling.push(caught);
          let bfail = false;
          let berr = this.NONE;
          try {
            sig = this.execBlock(h.body, sc);
          } catch(e) {
            bfail = true;
            berr = this.takeException(( e.toString()));
          }
          this.handling.pop();
          if ( h.s.length > 0 ) {
            delete sc.vars[h.s];
          }
          if ( bfail ) {
            if ( (this.isExcInst(berr) && ( typeof(berr.attrs["__context__"] ) != "undefined" && Object.prototype.hasOwnProperty.call(berr.attrs, "__context__") ) == false) && this.same(berr, caught) == false ) {
              berr.putAttr("__context__", caught);
            }
            pending = berr;
          }
        }
      }
    } else {
      if ( sig == 0 && n.orelse.length > 0 ) {
        let ofail = false;
        let oerr = this.NONE;
        try {
          sig = this.execBlock(n.orelse, sc);
        } catch(e) {
          ofail = true;
          oerr = this.takeException(( e.toString()));
        }
        if ( ofail ) {
          pending = oerr;
        }
      }
    }
    if ( n.final.length > 0 ) {
      const keepRet = this.retVal;
      const fsig = this.execBlock(n.final, sc);
      if ( fsig != 0 ) {
        return fsig;
      }
      this.retVal = keepRet;
    }
    if ( (typeof(pending) !== "undefined" && pending != null )  ) {
      this.reraise(pending);
    }
    return sig;
  };
  execRaise (n, sc) {
    if ( typeof(n.k1) === "undefined" ) {
      if ( this.handling.length == 0 ) {
        this.raise("RuntimeError", "No active exception to reraise");
      }
      this.reraise(this.handling[(this.handling.length - 1)]);
    }
    const v = this.eval(n.k1, sc);
    let e = v;
    if ( v.t == 11 ) {
      let kn = [];
      let kv = [];
      e = this.instantiate(v, [], kn, kv);
    }
    if ( this.isExcInst(e) == false ) {
      this.raise("TypeError", "exceptions must derive from BaseException");
    }
    if ( (typeof(n.k2) !== "undefined" && n.k2 != null )  ) {
      const cause = this.eval(n.k2, sc);
      e.putAttr("__cause__", cause);
    }
    this.raiseVal(e);
  };
  execWith (n, idx, sc) {
    if ( idx >= n.kids.length ) {
      return this.execBlock(n.body, sc);
    }
    const item = n.kids[idx];
    const ctx = this.eval(item.k1, sc);
    const entered = this.callAttr(ctx, "__enter__", []);
    if ( (typeof(item.k2) !== "undefined" && item.k2 != null )  ) {
      this.assign(item.k2, entered, sc);
    }
    let sig = 0;
    let failed = false;
    let err = this.NONE;
    try {
      sig = this.execWith(n, (idx + 1), sc);
    } catch(e) {
      failed = true;
      err = this.takeException(( e.toString()));
    }
    if ( failed ) {
      let cls = this.NONE;
      if ( (typeof(err.cls) !== "undefined" && err.cls != null )  ) {
        cls = err.cls;
      }
      const keep = this.callAttr(ctx, "__exit__", [cls, err, this.NONE]);
      if ( this.truthy(keep) == false ) {
        this.reraise(err);
      }
      return 0;
    }
    const keepRet = this.retVal;
    this.callAttr(ctx, "__exit__", [this.NONE, this.NONE, this.NONE]);
    this.retVal = keepRet;
    return sig;
  };
  callAttr (obj, name, args) {
    const m = this.getAttr(obj, name);
    let kn = [];
    let kv = [];
    return this.callValue(m, args, kn, kv);
  };
  importModule (name) {
    const m = ( Object.prototype.hasOwnProperty.call(this.modules, name) ? this.modules[name] : undefined );
    if ( (typeof(m) !== "undefined" && m != null )  ) {
      return m;
    }
    const dot = name.lastIndexOf(".");
    let parent;
    if ( dot > 0 ) {
      parent = this.importModule(name.substring(0, dot ));
      const again = ( Object.prototype.hasOwnProperty.call(this.modules, name) ? this.modules[name] : undefined );
      if ( (typeof(again) !== "undefined" && again != null )  ) {
        return again;
      }
    }
    let made = this.makeModule(name);
    if ( typeof(made) === "undefined" ) {
      made = this.importFromFiles(name);
    }
    if ( typeof(made) === "undefined" ) {
      this.raise("ModuleNotFoundError", ("No module named '" + name) + "'");
    }
    const mv = made;
    this.modules[name] = mv;
    this.dictSet(this.sysModules, this.mkStr(name), mv);
    if ( (typeof(parent) !== "undefined" && parent != null )  ) {
      const pv = parent;
      pv.putAttr(name.substring(dot + 1, name.length ), mv);
    }
    return mv;
  };
  importFromFiles (name) {
    let none;
    const rel = name.split(".").join("/");
    let roots = [];
    roots.push("");
    // Loop start
    for ( const sp of this.searchPath) {
      roots.push(sp);
    }
    // Loop start
    for ( const root of roots) {
      let base = rel;
      if ( root.length > 0 ) {
        base = (root + "/") + rel;
      }
      let cands = [];
      cands.push(base + ".py");
      cands.push(base + "/__init__.py");
      // Loop start
      for ( const c of cands) {
        const src = this.fs.read(c);
        if ( (typeof(src) !== "undefined" && src != null )  ) {
          const mod = this.runModuleSource(name, src);
          mod.putAttr("__file__", this.mkStr(this.fs.normalize(c)));
          if ( c.endsWith("/__init__.py") ) {
            mod.putAttr("__path__", this.mkList([this.mkStr(this.fs.normalize(base))]));
          }
          const got = mod;
          return got;
        }
      }
    }
    const lsrc = PyStdlib.source(name);
    if ( (typeof(lsrc) !== "undefined" && lsrc != null )  ) {
      const lmod = this.runModuleSource(name, lsrc);
      lmod.putAttr("__file__", this.mkStr((("<stdlib>/" + rel) + ".py")));
      if ( PyStdlib.isPackage(name) ) {
        lmod.putAttr("__path__", this.mkList([]));
      }
      const lgot = lmod;
      return lgot;
    }
    return none;
  };
  hasAnn (body) {
    // Loop start
    for ( const st of body) {
      if ( st.kind == 43 ) {
        return true;
      }
      if ( (((st.kind == 44 || st.kind == 45) || st.kind == 46) || st.kind == 54) || st.kind == 63 ) {
        if ( (this.hasAnn(st.body) || this.hasAnn(st.orelse)) || this.hasAnn(st.final) ) {
          return true;
        }
        // Loop start
        for ( const h of st.handlers) {
          if ( this.hasAnn(h.body) ) {
            return true;
          }
        }
      }
    }
    return false;
  };
  setupAnn (body, sc) {
    if ( ( typeof(sc.vars["__annotations__"] ) != "undefined" && Object.prototype.hasOwnProperty.call(sc.vars, "__annotations__") ) == false ) {
      if ( this.hasAnn(body) ) {
        sc.vars["__annotations__"] = this.mkDict();
      }
    }
  };
  runModuleSource (name, src) {
    const ps = new PyParser();
    let tree;
    try {
      tree = ps.parseModule(src);
    } catch(e) {
      if ( ps.err.length == 0 ) {
        ps.err = ( e.toString());
      }
    }
    if ( ps.err.length > 0 ) {
      this.raise("SyntaxError", ((((ps.err + " (") + name) + ".py, line ") + ((ps.errLine).toString())) + ")");
    }
    const mod = new PyVal(13);
    mod.s = name;
    this.modules[name] = mod;
    const msc = new PyScope();
    msc.vars = mod.attrs;
    msc.vars["__name__"] = this.mkStr(name);
    let failed = false;
    let caught = this.NONE;
    const tr = tree;
    try {
      this.setupAnn(tr.body, msc);
      this.execBlock(tr.body, msc);
    } catch(e) {
      caught = this.takeException(( e.toString()));
      failed = true;
    }
    mod.attrs = msc.vars;
    if ( failed ) {
      delete this.modules[name];
      this.reraise(caught);
    }
    return mod;
  };
  execImport (n, sc) {
    let i = 0;
    while (i < n.strs.length) {
      const name = n.strs[i];
      const alias = n.names[i];
      const mod = this.importModule(name);
      if ( alias.length > 0 ) {
        this.store(alias, mod, sc);
      } else {
        const dot = name.indexOf(".");
        if ( dot > 0 ) {
          const top = name.substring(0, dot );
          this.store(top, this.importModule(top), sc);
        } else {
          this.store(name, mod, sc);
        }
      }
      i = i + 1;
    };
  };
  execFromImport (n, sc) {
    if ( n.s == "__future__" ) {
      return;
    }
    const mod = this.importModule(n.s);
    let i = 0;
    while (i < n.strs.length) {
      const name = n.strs[i];
      if ( name == "*" ) {
        const ks = mod.attrKeys();
        // Loop start
        for ( const k of ks) {
          if ( k.startsWith("_") == false ) {
            this.store(
              k,
              ( Object.prototype.hasOwnProperty.call(mod.attrs, k) ? mod.attrs[k] : undefined ),
              sc
            );
          }
        }
      } else {
        const alias = n.names[i];
        let v = ( Object.prototype.hasOwnProperty.call(mod.attrs, name) ? mod.attrs[name] : undefined );
        if ( (((typeof(v) === "undefined") && ( typeof(mod.attrs["__getattr__"] ) != "undefined" && Object.prototype.hasOwnProperty.call(mod.attrs, "__getattr__") )) && ( typeof(this.modules[((n.s + ".") + name)] ) != "undefined" && Object.prototype.hasOwnProperty.call(this.modules, (n.s + ".") + name) ) == false) && (typeof(this.importFromFiles((n.s + ".") + name)) === "undefined") ) {
          v = this.getAttrOpt(mod, name);
        }
        if ( typeof(v) === "undefined" ) {
          let sub = ( Object.prototype.hasOwnProperty.call(this.modules, (n.s + ".") + name) ? this.modules[((n.s + ".") + name)] : undefined );
          if ( typeof(sub) === "undefined" ) {
            sub = this.makeModule(((n.s + ".") + name));
          }
          if ( typeof(sub) === "undefined" ) {
            sub = this.importFromFiles(((n.s + ".") + name));
          }
          if ( typeof(sub) === "undefined" ) {
            this.raise("ImportError", ((("cannot import name '" + name) + "' from '") + n.s) + "'");
          }
          this.modules[(n.s + ".") + name] = sub;
          v = sub;
        }
        if ( alias.length > 0 ) {
          this.store(alias, v, sc);
        } else {
          this.store(name, v, sc);
        }
      }
      i = i + 1;
    };
  };
  runTree (tree) {
    this.setupAnn(tree.body, this.globals);
    this.execBlock(tree.body, this.globals);
  };
}
PyInterp.strCmp = function(a, b) {
  const ca = a.codes();
  const cb = b.codes();
  const na = ca.length;
  const nb = cb.length;
  let i = 0;
  while (i < na && i < nb) {
    const x = ca[i];
    const y = cb[i];
    if ( x < y ) {
      return -1;
    }
    if ( x > y ) {
      return 1;
    }
    i = i + 1;
  };
  if ( na < nb ) {
    return -1;
  }
  if ( na > nb ) {
    return 1;
  }
  return 0;
};
PyInterp.bitOp = function(op, a0, b0) {
  let a = a0;
  let b = b0;
  let result = 0;
  let bit = 1;
  let k = 0;
  while (k < 62) {
    if ( (a == 0 || a == -1) && (b == 0 || b == -1) ) {
      break;
    }
    const x = PyFormat.floorModInt(a, 2);
    const y = PyFormat.floorModInt(b, 2);
    let z = 0;
    if ( op == "&" ) {
      if ( x == 1 && y == 1 ) {
        z = 1;
      }
    }
    if ( op == "|" ) {
      if ( x == 1 || y == 1 ) {
        z = 1;
      }
    }
    if ( op == "^" ) {
      if ( x != y ) {
        z = 1;
      }
    }
    if ( z == 1 ) {
      result = result + bit;
    }
    a = PyFormat.floorDivInt(a, 2);
    b = PyFormat.floorDivInt(b, 2);
    bit = bit * 2;
    k = k + 1;
  };
  let sa = 0;
  if ( a == -1 ) {
    sa = 1;
  }
  let sb = 0;
  if ( b == -1 ) {
    sb = 1;
  }
  let sz = 0;
  if ( op == "&" ) {
    if ( sa == 1 && sb == 1 ) {
      sz = 1;
    }
  }
  if ( op == "|" ) {
    if ( sa == 1 || sb == 1 ) {
      sz = 1;
    }
  }
  if ( op == "^" ) {
    if ( sa != sb ) {
      sz = 1;
    }
  }
  if ( sz == 1 ) {
    result = result - bit;
  }
  return result;
};
PyInterp.fabs = function(x) {
  if ( x < 0.0 ) {
    return 0.0 - x;
  }
  return x;
};
PyInterp.hypot = function(a, b) {
  let x = PyInterp.fabs(a);
  let y = PyInterp.fabs(b);
  if ( (Math.abs(x) === Infinity) || (Math.abs(y) === Infinity) ) {
    return Infinity;
  }
  if ( x < y ) {
    const t = x;
    x = y;
    y = t;
  }
  if ( x == 0.0 ) {
    return 0.0;
  }
  const r = y / x;
  return x * Math.sqrt(1.0 + r * r);
};
PyInterp.cPart = function(x) {
  const s = PyFormat.floatRepr(x);
  if ( s.length > 2 ) {
    if ( s.endsWith(".0") ) {
      return s.substring(0, s.length - 2 );
    }
  }
  return s;
};
export class PyRuntime  {
  constructor() {
    this.fs = new PyFileSystem();
    this.stdout = "";
    this.error = "";
    this.errorType = "";
    this.errorLine = 0;
    this.traceback = "";
    this.maxSteps = 0;
    this.echo = false;
    this.argv = [];
    this.searchPath = [];
    this.fs = new PyFileSystem();
  }
  writeFile (path, text) {
    this.fs.put(path, text);
  };
  readFile (path) {
    const t = this.fs.get(path);
    if ( typeof(t) === "undefined" ) {
      return "";
    }
    return t;
  };
  hasFile (path) {
    return this.fs.isFile(path);
  };
  listFiles () {
    return this.fs.allFiles();
  };
  changedFiles () {
    return this.fs.changedFiles();
  };
  removedFiles () {
    return this.fs.removedFiles();
  };
  clearChanges () {
    this.fs.clearChanges();
  };
  setCwd (path) {
    this.fs.mkdirs(path);
    this.fs.cwd = this.fs.normalize(path);
  };
  addPath (dir) {
    this.searchPath.push(dir);
  };
  addArg (a) {
    this.argv.push(a);
  };
  run (code) {
    this.stdout = "";
    this.error = "";
    this.errorType = "";
    this.errorLine = 0;
    this.traceback = "";
    const ps = new PyParser();
    let tree;
    try {
      tree = ps.parseModule(code);
    } catch(e) {
      if ( ps.err.length == 0 ) {
        ps.err = ( e.toString());
      }
    }
    if ( ps.err.length > 0 ) {
      this.errorType = "SyntaxError";
      this.error = "SyntaxError: " + ps.err;
      this.errorLine = ps.errLine;
      this.traceback = (("  File \"<script>\", line " + ((this.errorLine).toString())) + "\n") + this.error;
      return false;
    }
    const ip = new PyInterp(this.fs);
    ip.setup();
    ip.maxSteps = this.maxSteps;
    ip.echo = this.echo;
    ip.argv = this.argv;
    ip.searchPath = this.searchPath;
    let ok = true;
    try {
      ip.runTree(tree);
    } catch(e) {
      const ex = ip.takeException(( e.toString()));
      let cname = "Exception";
      if ( (typeof(ex.cls) !== "undefined" && ex.cls != null )  ) {
        const c = ex.cls;
        cname = c.s;
      }
      ok = false;
      if ( cname == "SystemExit" ) {
        const args = ip.excArgs(ex);
        if ( args.items.length == 0 ) {
          ok = true;
        } else {
          const code0 = args.items[0];
          if ( code0.t == 0 || code0.t == 2 && code0.i == 0 ) {
            ok = true;
          } else {
            if ( code0.t != 2 ) {
              ip.writeOut(ip.toStr(code0) + "\n");
            }
          }
        }
      }
      if ( ok == false ) {
        this.errorType = cname;
        this.error = ip.excMessage(ex);
        const lv = ( Object.prototype.hasOwnProperty.call(ex.attrs, "__line__") ? ex.attrs["__line__"] : undefined );
        if ( (typeof(lv) !== "undefined" && lv != null )  ) {
          const tv1 = lv;
          this.errorLine = tv1.i;
        }
        const tv = ( Object.prototype.hasOwnProperty.call(ex.attrs, "__trace__") ? ex.attrs["__trace__"] : undefined );
        if ( (typeof(tv) !== "undefined" && tv != null )  ) {
          const tv2 = tv;
          this.traceback = tv2.s;
        }
        if ( this.traceback.length == 0 ) {
          this.traceback = "Traceback (most recent call last):";
        }
        this.traceback = (this.traceback + "\n") + this.error;
      }
    }
    ip.flushEcho();
    this.stdout = ip.output();
    return ok;
  };
}

function py_rt_codes(cs, s, e) {
  let out = "";
  for (let i = s; i < e; i += 4096) {
    out += String.fromCodePoint.apply(null, cs.slice(i, Math.min(e, i + 4096)));
  }
  return out;
}


function r_char_length(s) {
    let n = 0;
    for (let i = 0; i < s.length; i++) {
        const c = s.charCodeAt(i);
        if (c >= 0xDC00 && c <= 0xDFFF) { continue; }
        n++;
    }
    return n;
}

