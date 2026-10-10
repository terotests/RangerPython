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
    this.s = "";
    this.items = [];
    this.vals = [];
    this.index = {};
    this.attrs = {};
    this.defs = {};
    this.bases = [];
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
    this.cps = [];
    this.cpsKnown = false;
    this.t = tag;
  }
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
export class PyScope  {
  constructor() {
    this.vars = {};
    this.parent = undefined;
    this.globals = undefined;
    this.decl = {};
    this.isClass = false;
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
    if ( jc == 106 || jc == 74 ) {
      this.fail("complex numbers are not supported");
      return;
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
    if ( isFloat ) {
      this.add(2, t);
    } else {
      this.add(1, t);
    }
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
    while (true) {
      if ( this.pos >= this.n ) {
        this.line = startLine;
        this.fail("unterminated string literal");
        return;
      }
      const c = this.ch(0);
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
          out.push(v_1);
          continue;
        }
        if ( c2 == 78 ) {
          this.fail("\\N{...} escapes are not supported");
          return;
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
  }
  parseModule (src) {
    const m = new PyNode(64, 1);
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
      if ( this.isKw("class") ) {
        out.push(this.parseClass(ds));
        return;
      }
      this.fail("a decorator must be followed by def or class");
    }
    this.parseSimpleLineInto(out);
  };
  parseSimpleLineInto (out) {
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
    const node = new PyNode(53, this.line());
    this.p = this.p + 1;
    node.body = this.parseBlock();
    while (this.isKw("except")) {
      const h = new PyNode(54, this.line());
      this.p = this.p + 1;
      this.acceptOp("*");
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
    const node = new PyNode(62, this.line());
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
      const item = new PyNode(63, this.line());
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
  parseDef (decos) {
    const node = new PyNode(51, this.line());
    this.p = this.p + 1;
    node.s = this.expectName();
    node.decos = decos;
    this.expectOp("(");
    node.kids = this.parseParams(")", true);
    this.expectOp(")");
    if ( this.acceptOp("->") ) {
      this.parseTest();
    }
    this.yields.push(false);
    node.body = this.parseBlock();
    if ( this.yields[(this.yields.length - 1)] ) {
      node.i = 1;
    }
    this.yields.pop();
    return node;
  };
  parseParams (close, annotations) {
    let out = [];
    while (this.isOp(close) == false) {
      const prm = new PyNode(39, this.line());
      if ( this.acceptOp("**") ) {
        prm.i = 2;
        prm.s = this.expectName();
      } else {
        if ( this.acceptOp("*") ) {
          if ( this.isOp(",") || this.isOp(close) ) {
            prm.i = 3;
          } else {
            prm.i = 1;
            prm.s = this.expectName();
          }
        } else {
          if ( this.acceptOp("/") ) {
            prm.i = 4;
          } else {
            prm.s = this.expectName();
          }
        }
      }
      if ( annotations && ((prm.i == 0 || prm.i == 1) || prm.i == 2) ) {
        if ( this.acceptOp(":") ) {
          this.parseTest();
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
    node.s = this.expectName();
    node.decos = decos;
    if ( this.acceptOp("(") ) {
      node.kids = this.parseCallArgs();
      this.expectOp(")");
    }
    this.yields.push(false);
    node.body = this.parseBlock();
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
        const r2 = new PyNode(55, ln);
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
        const g = new PyNode(58, ln);
        if ( w == "nonlocal" ) {
          g.kind = 59;
        }
        g.strs.push(this.expectName());
        while (this.acceptOp(",")) {
          g.strs.push(this.expectName());
        };
        return g;
      }
      if ( w == "import" ) {
        this.p = this.p + 1;
        const im = new PyNode(56, ln);
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
        const fi = new PyNode(57, ln);
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
        const a = new PyNode(60, ln);
        a.k1 = this.parseTest();
        if ( this.acceptOp(",") ) {
          a.k2 = this.parseTest();
        }
        return a;
      }
      if ( w == "del" ) {
        this.p = this.p + 1;
        const d = new PyNode(61, ln);
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
      this.parseTest();
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
        a.s = nt.text;
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
            kw.s = t.text;
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
      nm.s = w;
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
    const names = ["print", "len", "range", "enumerate", "zip", "map", "filter", "sorted", "reversed", "sum", "min", "max", "abs", "round", "int", "float", "str", "bool", "list", "tuple", "dict", "set", "frozenset", "isinstance", "issubclass", "type", "repr", "ascii", "chr", "ord", "hex", "oct", "bin", "divmod", "pow", "any", "all", "open", "format", "hash", "id", "iter", "next", "getattr", "setattr", "hasattr", "delattr", "callable", "input", "vars", "globals", "locals", "dir", "property", "staticmethod", "classmethod", "super", "object", "slice", "exit", "quit", "__object_init__", "__build_class__"];
    // Loop start
    for ( const n of names) {
      this.put(n);
    }
    this.put("NotImplemented");
    const ni = this.ip.NOTIMPL;
    this.ip.builtins["NotImplemented"] = ni;
    this.put("Ellipsis");
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
    if ( cls.t == 11 ) {
      if ( obj.t == 12 && ((typeof(obj.cls) !== "undefined" && obj.cls != null ) ) ) {
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
    const path = this.ip.fs.normalize(pv.s);
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
      this.ip.fs.write(path, "");
      return f;
    }
    if ( plain == "a" ) {
      if ( this.ip.fs.isFile(path) == false ) {
        this.ip.fs.write(path, "");
      }
      return f;
    }
    if ( plain == "x" ) {
      if ( this.ip.fs.isFile(path) ) {
        this.ip.raise("FileExistsError", "[Errno 17] File exists: " + PyFormat.strRepr(pv.s));
      }
      this.ip.fs.write(path, "");
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
      this.ip.meth.fileWrite(f, text);
      return;
    }
    this.ip.writeOut(text);
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
          if ( (c.t == 11 && bb.t == 11) && this.ip.isSubclass(c, bb) ) {
            return this.ip.TRUE;
          }
        }
        return this.ip.FALSE;
      }
      if ( c.t == 11 && b.t == 11 ) {
        return this.ip.mkBool(this.ip.isSubclass(c, b));
      }
      if ( b.t == 10 && b.s == "object" ) {
        return this.ip.TRUE;
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
      this.ip.raise("TypeError", "type() with three arguments is not supported");
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
      const __REGx1 = this.ip.argAt(args, 2);
      if ( n == 3 && __REGx1.t != 0 ) {
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
      const avals = this.ip.iterValues(args[0]);
      // Loop start
      for ( const av of avals) {
        if ( this.ip.truthy(av) ) {
          return this.ip.TRUE;
        }
      }
      return this.ip.FALSE;
    }
    if ( name == "all" ) {
      this.need("all", args, 1, 1);
      const alvals = this.ip.iterValues(args[0]);
      // Loop start
      for ( const alv of alvals) {
        if ( this.ip.truthy(alv) == false ) {
          return this.ip.FALSE;
        }
      }
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
      const hk = this.ip.hashKey(hv);
      return this.ip.mkInt(this.strHash(hk));
    }
    if ( name == "id" ) {
      this.need("id", args, 1, 1);
      return this.ip.mkInt(this.ip.idOf(args[0]));
    }
    if ( name == "iter" ) {
      this.need("iter", args, 1, 1);
      const iv = args[0];
      if ( iv.t == 17 ) {
        return iv;
      }
      if ( iv.t == 12 ) {
        const im = this.ip.findMethod(iv, "__iter__");
        if ( (typeof(im) !== "undefined" && im != null )  ) {
          const got = this.ip.callMethodVal(im, iv, []);
          if ( got.t == 12 || got.t == 17 ) {
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
      const gd = this.ip.mkDict();
      const gks = Object.keys(this.ip.globals.vars);
      // Loop start
      for ( const gk of gks) {
        this.ip.dictSet(
          gd,
          this.ip.mkStr(gk),
          ( Object.prototype.hasOwnProperty.call(this.ip.globals.vars, gk) ? this.ip.globals.vars[gk] : undefined )
        );
      }
      return gd;
    }
    if ( name == "dir" ) {
      let names = [];
      if ( n == 0 ) {
        names = Object.keys(this.ip.globals.vars);
      } else {
        const dv = args[0];
        if ( (dv.t == 12 || dv.t == 13) || dv.t == 11 ) {
          names = Object.keys(dv.attrs);
          if ( dv.t == 12 && ((typeof(dv.cls) !== "undefined" && dv.cls != null ) ) ) {
            const dc = dv.cls;
            const cks = Object.keys(dc.attrs);
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
          __self.attrs["args"] = this.ip.mkTuple(rest);
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
    this.addNames("str", "join split rsplit splitlines strip lstrip rstrip replace upper lower title capitalize swapcase casefold startswith endswith find rfind index rindex count format format_map zfill ljust rjust center isdigit isnumeric isdecimal isalpha isalnum isspace isupper islower istitle isidentifier isascii isprintable partition rpartition encode expandtabs removeprefix removesuffix translate __len__ __contains__ __getitem__ __add__ __mul__ __eq__ __lt__ __hash__ __iter__ __str__ __repr__");
    this.addNames("list", "append extend insert pop remove index count sort reverse copy clear __len__ __getitem__ __setitem__ __contains__ __iter__ __add__ __eq__ __repr__");
    this.addNames("deque", "append appendleft extend extendleft pop popleft remove index count reverse rotate copy clear __len__");
    this.addNames("tuple", "index count __len__ __getitem__ __contains__ __iter__ __add__ __eq__ __hash__ __repr__");
    this.addNames("dict", "get keys values items pop popitem setdefault update copy clear __len__ __getitem__ __setitem__ __contains__ __iter__ __eq__ __repr__ most_common elements total subtract");
    this.addNames("set", "add remove discard pop clear copy update union intersection difference symmetric_difference issubset issuperset isdisjoint intersection_update difference_update symmetric_difference_update __len__ __contains__ __iter__");
    this.addNames("frozenset", "copy union intersection difference symmetric_difference issubset issuperset isdisjoint __len__ __contains__ __iter__ __hash__");
    this.addNames("file", "write writelines read readline readlines close flush seek tell __enter__ __exit__ __iter__ __next__ readable writable");
    this.addNames("int", "bit_length conjugate is_integer __index__ __int__ __float__ __abs__ __add__ __sub__ __mul__ __eq__ __lt__ __hash__ to_bytes");
    this.addNames("float", "is_integer conjugate hex __int__ __float__ __abs__ __add__ __sub__ __mul__ __eq__ __lt__ __hash__ as_integer_ratio");
    this.addNames("iter", "__next__ __iter__");
    this.addNames("range", "index count __len__ __iter__ __contains__");
  }
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
      case 17 : 
        return "iter";
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
    if ( name == "__counter_key__" ) {
      const ck = args[0];
      return ck.items[1];
    }
    if ( name.startsWith("nt.") ) {
      return this.ip.mods.ntMethod(__self, name, args, kn, kv);
    }
    if ( name == "lru.cache_clear" ) {
      __self.attrs["__cache__"] = this.ip.mkDict();
      return this.ip.NONE;
    }
    if ( name.startsWith("unbound:") ) {
      const m = name.substring(8, name.length );
      if ( args.length == 0 ) {
        this.ip.raise("TypeError", ((("unbound method " + __self.s) + ".") + m) + "() needs an argument");
      }
      let rest = [];
      let i = 1;
      while (i < args.length) {
        rest.push(args[i]);
        i = i + 1;
      };
      return this.callMethod(args[0], m, rest, kn, kv);
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
    if ( name.startsWith("property.") ) {
      const p = new PyVal(19);
      p.wrapped = __self.wrapped;
      p.setter = __self.setter;
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
      if ( __self.t == 17 || __self.t == 16 ) {
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
      case 17 : 
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
          headParts.push(all2[h].s);
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
    const n = args.length;
    if ( name == "get" ) {
      this.need("get", args, 1, 2);
      const r = this.ip.dictGet(__self, args[0]);
      if ( typeof(r) === "undefined" ) {
        return this.arg(args, 1);
      }
      return r;
    }
    if ( name == "keys" ) {
      let ks = [];
      // Loop start
      for ( const k of __self.items) {
        ks.push(k);
      }
      const l = this.ip.mkList(ks);
      l.mode = "dict_keys";
      return l;
    }
    if ( name == "values" ) {
      let vs = [];
      // Loop start
      for ( const v of __self.vals) {
        vs.push(v);
      }
      const l2 = this.ip.mkList(vs);
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
      for ( let ki = 0; ki < kn.length; ki++) {
        var key = kn[ki];
        this.ip.dictSet(__self, this.ip.mkStr(key), kv[ki]);
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
      const ak = Object.keys(__self.attrs);
      // Loop start
      for ( const a of ak) {
        c.attrs[a] = ( Object.prototype.hasOwnProperty.call(__self.attrs, a) ? __self.attrs[a] : undefined );
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
      m.attrs[n] = b;
    }
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
      m.attrs["pi"] = this.ip.mkFloat(3.141592653589793);
      m.attrs["e"] = this.ip.mkFloat(2.718281828459045);
      m.attrs["tau"] = this.ip.mkFloat(6.283185307179586);
      m.attrs["inf"] = this.ip.mkFloat(Infinity);
      m.attrs["nan"] = this.ip.mkFloat(NaN);
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
      m3.attrs["JSONDecodeError"] = jc;
      const optr96 = m3;
      return optr96;
    }
    if ( name == "os" ) {
      const m4 = this.newModule("os");
      this.fns(m4, "getcwd chdir listdir mkdir makedirs remove unlink rmdir rename replace walk getenv");
      m4.attrs["sep"] = this.ip.mkStr("/");
      m4.attrs["linesep"] = this.ip.mkStr("\n");
      m4.attrs["name"] = this.ip.mkStr("posix");
      m4.attrs["curdir"] = this.ip.mkStr(".");
      m4.attrs["environ"] = this.ip.mkDict();
      m4.attrs["path"] = this.make("os.path");
      const optr107 = m4;
      return optr107;
    }
    if ( name == "os.path" ) {
      const m5 = this.newModule("os.path");
      this.fns(m5, "join exists isfile isdir basename dirname split splitext abspath normpath getsize isabs expanduser");
      m5.attrs["sep"] = this.ip.mkStr("/");
      const optr113 = m5;
      return optr113;
    }
    if ( name == "sys" ) {
      const m6 = this.newModule("sys");
      this.fns(m6, "exit setrecursionlimit getrecursionlimit");
      let args = [];
      // Loop start
      for ( let i = 0; i < this.ip.argv.length; i++) {
        var a = this.ip.argv[i];
        args.push(this.ip.mkStr(a));
      }
      if ( args.length == 0 ) {
        args.push(this.ip.mkStr("main.py"));
      }
      m6.attrs["argv"] = this.ip.mkList(args);
      const so = new PyVal(16);
      so.s = "<stdout>";
      so.mode = "w";
      m6.attrs["stdout"] = so;
      const se = new PyVal(16);
      se.s = "<stderr>";
      se.mode = "w";
      m6.attrs["stderr"] = se;
      m6.attrs["maxsize"] = this.ip.mkInt(9007199254740991);
      m6.attrs["version"] = this.ip.mkStr("3.13.0 (Ranger Python runtime)");
      m6.attrs["platform"] = this.ip.mkStr("ranger");
      m6.attrs["path"] = this.ip.mkList([]);
      m6.attrs["modules"] = this.ip.mkDict();
      const optr139 = m6;
      return optr139;
    }
    if ( name == "string" ) {
      const m7 = this.newModule("string");
      m7.attrs["ascii_lowercase"] = this.ip.mkStr("abcdefghijklmnopqrstuvwxyz");
      m7.attrs["ascii_uppercase"] = this.ip.mkStr("ABCDEFGHIJKLMNOPQRSTUVWXYZ");
      m7.attrs["ascii_letters"] = this.ip.mkStr("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ");
      m7.attrs["digits"] = this.ip.mkStr("0123456789");
      m7.attrs["hexdigits"] = this.ip.mkStr("0123456789abcdefABCDEF");
      m7.attrs["octdigits"] = this.ip.mkStr("01234567");
      const punct = "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~";
      m7.attrs["punctuation"] = this.ip.mkStr(punct);
      const ws = (" \t\n\r" + String.fromCodePoint(11)) + String.fromCodePoint(12);
      m7.attrs["whitespace"] = this.ip.mkStr(ws);
      m7.attrs["printable"] = this.ip.mkStr((("0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ" + punct) + ws));
      const optr154 = m7;
      return optr154;
    }
    if ( name == "time" ) {
      const m8 = this.newModule("time");
      this.fns(m8, "time perf_counter monotonic sleep time_ns process_time");
      const optr159 = m8;
      return optr159;
    }
    if ( name == "collections" ) {
      const m9 = this.newModule("collections");
      m9.attrs["defaultdict"] = this.nativeClass("defaultdict");
      m9.attrs["Counter"] = this.nativeClass("Counter");
      m9.attrs["OrderedDict"] = this.nativeClass("OrderedDict");
      m9.attrs["deque"] = this.nativeClass("deque");
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
    if ( name == "functools" ) {
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
      m12.attrs["StatisticsError"] = se2;
      const optr189 = m12;
      return optr189;
    }
    if ( name == "copy" ) {
      const m13 = this.newModule("copy");
      this.fns(m13, "copy deepcopy");
      const optr194 = m13;
      return optr194;
    }
    if ( name == "typing" ) {
      const m14 = this.newModule("typing");
      this.fns(m14, "List Dict Set Tuple Optional Union Any Callable Iterable Iterator Sequence Mapping Generator Type FrozenSet DefaultDict NamedTuple TypeVar cast");
      const optr199 = m14;
      return optr199;
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
      if ( f == "getrecursionlimit" ) {
        return this.ip.mkInt(1000);
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
      return this.ip.getItem(seq, this.ip.mkInt(this.randbelow(__len)));
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
      d.attrs["__class__"] = cls;
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
    dd.attrs["__class__"] = cls;
    let start = 0;
    if ( kind == "defaultdict" ) {
      const __REGx1 = this.ip.argAt(args, 0);
      if ( n > 0 && __REGx1.t != 0 ) {
        dd.attrs["__missing__"] = args[0];
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
    cls.attrs["_fields"] = this.ip.mkTuple(fields);
    const dv = this.ip.bi.kw(kn, kv, "defaults");
    if ( (typeof(dv) !== "undefined" && dv != null )  ) {
      cls.attrs["__defaults__"] = this.ip.mkTuple(this.listOf(dv));
    }
    const mk = new PyVal(10);
    mk.s = "collections._make";
    mk.wrapped = cls;
    cls.attrs["_make"] = mk;
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
    t.attrs["__class__"] = cls;
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
    t2.attrs["__class__"] = cls;
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
      let srcs = args;
      if ( f == "chain_from_iterable" ) {
        srcs = this.listOf(args[0]);
      }
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
      const __REGx1 = this.ip.orNone(iv);
      if ( ((typeof(iv) !== "undefined" && iv != null ) ) && __REGx1.t != 0 ) {
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
        const __REGx2 = args[1];
        if ( __REGx2.t != 0 ) {
          const tv4 = args[1];
          stop = tv4.i;
        }
      } else {
        const __REGx3 = args[1];
        if ( __REGx3.t != 0 ) {
          const tv5 = args[1];
          start = tv5.i;
        }
        const __REGx4 = args[2];
        if ( __REGx4.t != 0 ) {
          const tv6 = args[2];
          stop = tv6.i;
        }
        const __REGx5 = this.ip.argAt(args, 3);
        if ( n > 3 && __REGx5.t != 0 ) {
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
        this.ip.raise("NotImplementedError", "itertools.repeat() without times is infinite; this runtime evaluates iterators eagerly");
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
        p.attrs[k] = kv[ki];
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
        w.attrs["__cache__"] = this.ip.mkDict();
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
      const ks = Object.keys(b.attrs);
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
      o.attrs["cmp"] = target;
      o.attrs["obj"] = args[0];
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
    c.attrs["__lt__"] = lt;
    const gt = new PyVal(10);
    gt.s = "functools._cmp_gt";
    c.attrs["__gt__"] = gt;
    const eq = new PyVal(10);
    eq.s = "functools._cmp_eq";
    c.attrs["__eq__"] = eq;
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
    const ks = Object.keys(src.attrs);
    // Loop start
    for ( const k of ks) {
      dst.attrs[k] = ( Object.prototype.hasOwnProperty.call(src.attrs, k) ? src.attrs[k] : undefined );
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
        const ks = Object.keys(v.attrs);
        // Loop start
        for ( const k2 of ks) {
          o.attrs[k2] = this.deepCopy(( Object.prototype.hasOwnProperty.call(v.attrs, k2) ? v.attrs[k2] : undefined ));
        }
        return o;
    };
    return v;
  };
}
export class PyGenFrame  {
  constructor() {
    this.items = [];
  }
}
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
    this.maxYield = 1000000;
    this.curLine = 0;
    this.frames = [];
    this.genStack = [];
    this.reprDepth = 0;
    this.globals = new PyScope();
    this.bi = undefined;
    this.meth = undefined;
    this.mods = undefined;
    this.argv = [];
    this.fs = filesystem;
    this.TRUE.i = 1;
    this.NOTIMPL.s = "NotImplemented";
    this.retVal = this.NONE;
    this.globals = new PyScope();
    this.bi = new PyBuiltins(this);
    this.meth = new PyMethods(this);
    this.mods = new PyModules(this);
    this.bi.install();
    this.resetGlobals();
  }
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
    e.attrs["args"] = this.mkTuple(args);
    return e;
  };
  raise (name, msg) {
    const e = this.makeExc(this.excClass(name), msg);
    this.raiseVal(e);
  };
  raiseVal (e) {
    if ( ( typeof(e.attrs["__line__"] ) != "undefined" && Object.prototype.hasOwnProperty.call(e.attrs, "__line__") ) == false ) {
      e.attrs["__line__"] = this.mkInt(this.curLine);
      e.attrs["__trace__"] = this.mkStr(this.traceText());
    }
    this.excValue = e;
    throw new Error("python exception");
  };
  markExc (e) {
    if ( ( typeof(e.attrs["__line__"] ) != "undefined" && Object.prototype.hasOwnProperty.call(e.attrs, "__line__") ) == false ) {
      e.attrs["__line__"] = this.mkInt(this.curLine);
      e.attrs["__trace__"] = this.mkStr(this.traceText());
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
    const ts = ["int", "float", "str", "bool", "list", "tuple", "dict", "set", "frozenset", "object", "type", "range", "slice", "enumerate", "zip", "map", "filter", "reversed", "property", "staticmethod", "classmethod", "super"];
    // Loop start
    for ( const t of ts) {
      if ( t == name ) {
        return true;
      }
    }
    return false;
  };
  truthy (v) {
    switch (v.t ) { 
      case 0 : 
        return false;
      case 1 : 
        return v.i != 0;
      case 2 : 
        return v.i != 0;
      case 3 : 
        return v.d != 0.0;
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
  toRepr (v) {
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
        return ((("<__main__." + this.typeName(v)) + " object at ") + this.hexId(v)) + ">";
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
      case 18 : 
        return ("slice(" + this.joinRepr(v.items)) + ")";
    };
    return ("<" + this.typeName(v)) + " object>";
  };
  hashKey (v) {
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
    if ( a.t == 12 || b.t == 12 ) {
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
      if ( a.t == 12 ) {
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
      if ( a.t == 12 || b.t == 12 ) {
        const r2 = this.dunderBin("__gt__", "__lt__", a, b);
        if ( (typeof(r2) !== "undefined" && r2 != null )  ) {
          return this.truthy(r2);
        }
      }
      return this.less(b, a);
    }
    if ( op == "<=" ) {
      if ( a.t == 12 || b.t == 12 ) {
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
      if ( a.t == 12 || b.t == 12 ) {
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
    if ( typeof(i) === "undefined" ) {
      d.index[hk] = d.items.length;
      d.items.push(key);
      d.vals.push(val);
      return;
    }
    d.vals[i] = val;
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
    if ( v.t != 12 || (typeof(v.cls) === "undefined") ) {
      return none;
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
    return none;
  };
  callMethodVal (f, __self, args) {
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
      if ( obj.t == 11 ) {
        this.raise("AttributeError", ((("type object '" + obj.s) + "' has no attribute '") + name) + "'");
      }
      this.raise("AttributeError", ((("'" + this.typeName(obj)) + "' object has no attribute '") + name) + "'");
    }
    return r;
  };
  getAttrOpt (obj, name) {
    let none;
    switch (obj.t ) { 
      case 12 : 
        const c = obj.cls;
        if ( name == "__class__" ) {
          const optr1508 = c;
          return optr1508;
        }
        if ( name == "__dict__" ) {
          const d = this.mkDict();
          const ks = Object.keys(obj.attrs);
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
          const optr1519 = d;
          return optr1519;
        }
        const cv = this.classLookup(c, name);
        if ( (typeof(cv) !== "undefined" && cv != null )  ) {
          const cvv = cv;
          if ( cvv.t == 19 ) {
            const optr1526 = this.bindValue(cvv, obj, c);
            return optr1526;
          }
        }
        const own = ( Object.prototype.hasOwnProperty.call(obj.attrs, name) ? obj.attrs[name] : undefined );
        if ( (typeof(own) !== "undefined" && own != null )  ) {
          return own;
        }
        if ( (typeof(cv) !== "undefined" && cv != null )  ) {
          const optr1534 = this.bindValue(cv, obj, c);
          return optr1534;
        }
        if ( name == "__name__" ) {
          return none;
        }
        const ga = this.classLookup(c, "__getattr__");
        if ( (typeof(ga) !== "undefined" && ga != null )  ) {
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
          m.push(this.bi.objectClass());
          const optr1558 = this.mkTuple(m);
          return optr1558;
        }
        if ( name == "__bases__" ) {
          const optr1561 = this.mkTuple(obj.bases);
          return optr1561;
        }
        if ( name == "__dict__" ) {
          const d2 = this.mkDict();
          const ks2 = Object.keys(obj.attrs);
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
        if ( typeof(v) === "undefined" ) {
          return none;
        }
        const vv = v;
        if ( vv.t == 21 ) {
          const optr1577 = this.bindValue(vv, obj, obj);
          return optr1577;
        }
        if ( vv.t == 20 ) {
          const optr1580 = vv.wrapped;
          return optr1580;
        }
        const optr1582 = vv;
        return optr1582;
      case 13 : 
        return ( Object.prototype.hasOwnProperty.call(obj.attrs, name) ? obj.attrs[name] : undefined );
      case 22 : 
        const __self = obj.__self;
        const after = obj.cls;
        let start = __self;
        if ( __self.t == 12 ) {
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
        if ( name == "__name__" ) {
          const optr1629 = this.mkStr(obj.s);
          return optr1629;
        }
        if ( name == "__qualname__" ) {
          const optr1632 = this.mkStr(obj.s);
          return optr1632;
        }
        if ( name == "__doc__" ) {
          const optr1635 = this.NONE;
          return optr1635;
        }
        return ( Object.prototype.hasOwnProperty.call(obj.attrs, name) ? obj.attrs[name] : undefined );
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
        return none;
      case 10 : 
        if ( name == "__name__" ) {
          const bd = obj.s.lastIndexOf(".");
          const optr1654 = this.mkStr(obj.s.substring(bd + 1, obj.s.length ));
          return optr1654;
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
        if ( name == "fget" ) {
          const optr1674 = obj.wrapped;
          return optr1674;
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
  setAttr (obj, name, val) {
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
      }
      const sa = this.classLookup(c, "__setattr__");
      const __REGx1 = this.orNone(sa);
      if ( ((typeof(sa) !== "undefined" && sa != null ) ) && __REGx1.t == 9 ) {
        this.callMethodVal(sa, obj, [this.mkStr(name), val]);
        return;
      }
      obj.attrs[name] = val;
      return;
    }
    if ( (obj.t == 11 || obj.t == 13) || obj.t == 9 ) {
      obj.attrs[name] = val;
      return;
    }
    this.raise("AttributeError", ((("'" + this.typeName(obj)) + "' object has no attribute '") + name) + "'");
  };
  setAttrRaw (obj, name, val) {
    obj.attrs[name] = val;
  };
  delAttr (obj, name) {
    if ( ((obj.t == 12 || obj.t == 11) || obj.t == 13) && ( typeof(obj.attrs[name] ) != "undefined" && Object.prototype.hasOwnProperty.call(obj.attrs, name) ) ) {
      delete obj.attrs[name];
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
        return obj;
      case 10 : 
        return obj;
    };
    this.raise("TypeError", ("'" + this.typeName(obj)) + "' object is not subscriptable");
    return this.NONE;
  };
  makeExcArg (name, arg) {
    const e = this.makeExc(this.excClass(name), "");
    e.attrs["args"] = this.mkTuple([arg]);
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
  binop (op, a, b) {
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
        if ( fx < 0.0 && Math.floor(fy) != fy ) {
          this.raise("ValueError", "negative number cannot be raised to a fractional power (complex numbers are not supported)");
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
    if ( op == "-" ) {
      if ( this.isIntLike(a) ) {
        return this.mkInt((0 - a.i));
      }
      if ( a.t == 3 ) {
        return this.mkFloat((a.d * -1.0));
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
      if ( a.t == 3 ) {
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
      this.raise("NameError", ("name '" + name) + "' is not defined");
    }
    return r;
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
        const out2 = this.mkList([]);
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
        let yv = this.NONE;
        if ( (typeof(n.k1) !== "undefined" && n.k1 != null )  ) {
          yv = this.eval(n.k1, sc);
        }
        if ( this.genStack.length == 0 ) {
          this.raise("SyntaxError", "'yield' outside function");
        }
        const fr = this.genStack[(this.genStack.length - 1)];
        fr.items.push(yv);
        if ( fr.items.length > this.maxYield ) {
          this.raise("RuntimeError", ("generator produced more than " + ((this.maxYield).toString())) + " values: generators run to completion in this runtime, so an endless generator cannot be consumed lazily");
        }
        this.tick();
        return this.NONE;
      case 38 : 
        const src2 = this.eval(n.k1, sc);
        const vals = this.iterValues(src2);
        if ( this.genStack.length == 0 ) {
          this.raise("SyntaxError", "'yield' outside function");
        }
        const fr2 = this.genStack[(this.genStack.length - 1)];
        // Loop start
        for ( const yv2 of vals) {
          fr2.items.push(yv2);
        }
        return this.NONE;
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
    if ( ci == 0 && ((typeof(sc.parent) !== "undefined" && sc.parent != null ) ) ) {
      src = this.eval(cl.k2, sc.parent);
    } else {
      src = this.eval(cl.k2, sc);
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
    const r = this.callValue(f, args, kn, kv);
    this.curLine = saved;
    return r;
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
    return f;
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
        return this.instantiate(f, args, kn, kv);
      case 12 : 
        const m = this.findMethod(f, "__call__");
        if ( (typeof(m) !== "undefined" && m != null )  ) {
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
        const gf = new PyGenFrame();
        this.genStack.push(gf);
        try {
          this.execBlock(node.body, sc);
        } catch(e) {
          failed = true;
          err = this.takeException(( e.toString()));
        }
        this.genStack.pop();
        if ( failed == false ) {
          const g = this.mkIter(gf.items);
          g.s = "generator";
          result = g;
        }
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
    if ( failed ) {
      if ( (this.isExcInst(err) && this.excMatches(err, this.excClass("StopIteration"))) && node.i == 1 ) {
        this.raise("RuntimeError", "generator raised StopIteration");
      }
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
      if ( st.kind == 58 ) {
        // Loop start
        for ( const nm of st.strs) {
          sc.decl[nm] = 1;
        }
      }
      if ( st.kind == 59 ) {
        // Loop start
        for ( const nm2 of st.strs) {
          sc.decl[nm2] = 2;
        }
      }
      if ( (((st.kind == 44 || st.kind == 45) || st.kind == 46) || st.kind == 62) || st.kind == 53 ) {
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
    const kwDict = this.mkDict();
    // Loop start
    for ( let k = 0; k < kn.length; k++) {
      var name = kn[k];
      let found = false;
      // Loop start
      for ( const p2 of params) {
        if ( p2.i == 0 && p2.s == name ) {
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
  instantiate (cls, args, kn, kv) {
    if ( cls.mode == "native" ) {
      return this.mods.construct(cls, args, kn, kv);
    }
    const nw = this.classLookup(cls, "__new__");
    let obj = new PyVal(12);
    obj.cls = cls;
    if ( cls.isExc ) {
      obj.attrs["args"] = this.mkTuple(args);
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
        return 0;
      case 44 : 
        const c = this.eval(n.k1, sc);
        if ( this.truthy(c) ) {
          return this.execBlock(n.body, sc);
        }
        return this.execBlock(n.orelse, sc);
      case 45 : 
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
      case 53 : 
        return this.execTry(n, sc);
      case 55 : 
        this.execRaise(n, sc);
        return 0;
      case 56 : 
        this.execImport(n, sc);
        return 0;
      case 57 : 
        this.execFromImport(n, sc);
        return 0;
      case 58 : 
        // Loop start
        for ( const nm of n.strs) {
          sc.decl[nm] = 1;
        }
        return 0;
      case 59 : 
        // Loop start
        for ( const nm2 of n.strs) {
          sc.decl[nm2] = 2;
        }
        return 0;
      case 60 : 
        const ac = this.eval(n.k1, sc);
        if ( this.truthy(ac) == false ) {
          if ( (typeof(n.k2) !== "undefined" && n.k2 != null )  ) {
            const am = this.eval(n.k2, sc);
            this.raiseVal(this.makeExcArg("AssertionError", am));
          }
          this.raise("AssertionError", "");
        }
        return 0;
      case 61 : 
        // Loop start
        for ( const dt of n.kids) {
          this.execDel(dt, sc);
        }
        return 0;
      case 62 : 
        return this.execWith(n, 0, sc);
    };
    this.raise("SyntaxError", "unsupported statement");
    return 0;
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
    if ( src.t == 17 ) {
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
    const cls = new PyVal(11);
    cls.s = n.s;
    // Loop start
    for ( const b of n.kids) {
      if ( b.kind == 24 ) {
        continue;
      }
      const bv = this.eval(b, sc);
      if ( bv.t == 10 && bv.s == "object" ) {
        continue;
      }
      if ( bv.t == 10 ) {
        this.raise("TypeError", ("subclassing the built-in type '" + bv.s) + "' is not supported");
      }
      if ( bv.t != 11 ) {
        this.raise("TypeError", "bases must be types");
      }
      cls.bases.push(bv);
      if ( bv.isExc ) {
        cls.isExc = true;
      }
    }
    this.computeMro(cls);
    const body = new PyScope();
    body.parent = sc;
    body.globals = this.moduleScope(sc);
    body.isClass = true;
    body.vars["__qualname__"] = this.mkStr(n.s);
    body.vars["__module__"] = this.mkStr("__main__");
    this.execBlock(n.body, body);
    const ks = Object.keys(body.vars);
    // Loop start
    for ( const k of ks) {
      const v = ( Object.prototype.hasOwnProperty.call(body.vars, k) ? body.vars[k] : undefined );
      cls.attrs[k] = v;
      this.setOwner(v, cls);
    }
    const cv = this.applyDecorators(n.decos, cls, sc);
    this.store(n.s, cv, sc);
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
    if ( failed ) {
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
              berr.attrs["__context__"] = caught;
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
      e.attrs["__cause__"] = cause;
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
    const made = this.mods.make(name);
    if ( typeof(made) === "undefined" ) {
      const path = name.split(".").join("/") + ".py";
      const src = this.fs.read(path);
      if ( typeof(src) === "undefined" ) {
        this.raise("ModuleNotFoundError", ("No module named '" + name) + "'");
      }
      const mod = this.runModuleSource(name, src);
      this.modules[name] = mod;
      return mod;
    }
    const mv = made;
    this.modules[name] = mv;
    return mv;
  };
  runModuleSource (name, src) {
    const ps = new PyParser();
    const tree = ps.parseModule(src);
    if ( ps.err.length > 0 ) {
      this.raise("SyntaxError", ((((ps.err + " (") + name) + ".py, line ") + ((ps.errLine).toString())) + ")");
    }
    const mod = new PyVal(13);
    mod.s = name;
    this.modules[name] = mod;
    const msc = new PyScope();
    msc.vars["__name__"] = this.mkStr(name);
    this.execBlock(tree.body, msc);
    const ks = Object.keys(msc.vars);
    // Loop start
    for ( const k of ks) {
      mod.attrs[k] = ( Object.prototype.hasOwnProperty.call(msc.vars, k) ? msc.vars[k] : undefined );
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
        const ks = Object.keys(mod.attrs);
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
        if ( typeof(v) === "undefined" ) {
          const sub = this.mods.make(((n.s + ".") + name));
          if ( typeof(sub) === "undefined" ) {
            this.raise("ImportError", ((("cannot import name '" + name) + "' from '") + n.s) + "'");
          }
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
    ip.maxSteps = this.maxSteps;
    ip.echo = this.echo;
    ip.argv = this.argv;
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

