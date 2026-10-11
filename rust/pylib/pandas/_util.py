# SPDX-License-Identifier: MIT
# Shared helpers for rpy's pandas: missing values, dtypes, options, dates.

import math
import re
from datetime import datetime, date, timedelta

NaN = float("nan")
nan = NaN


class _NAType:
    _inst = None

    def __new__(cls):
        if cls._inst is None:
            cls._inst = object.__new__(cls)
        return cls._inst

    def __repr__(self):
        return "<NA>"

    __str__ = __repr__

    def __bool__(self):
        raise TypeError("boolean value of NA is ambiguous")

    def __hash__(self):
        return 2 ** 61 - 7

    def __eq__(self, other):
        return self

    def __ne__(self, other):
        return self


class _NaTType:
    _inst = None

    def __new__(cls):
        if cls._inst is None:
            cls._inst = object.__new__(cls)
        return cls._inst

    def __repr__(self):
        return "NaT"

    __str__ = __repr__

    def __bool__(self):
        return True

    def __hash__(self):
        return 2 ** 61 - 11

    def __eq__(self, other):
        return False

    def __ne__(self, other):
        return True

    def __lt__(self, other):
        return False

    __le__ = __gt__ = __ge__ = __lt__

    def isoformat(self, *a):
        return "NaT"

    def strftime(self, fmt):
        return "NaT"

    year = month = day = hour = minute = second = microsecond = property(lambda self: NaN)


NA = _NAType()
NaT = _NaTType()


def isna_scalar(v):
    if v is None or v is NaT or v is NA:
        return True
    return type(v) is float and v != v


class ArrayList(list):
    """What pandas hands back as a numpy array (unique(), .values): a list
    with tolist() and numpy's print form."""

    def tolist(self):
        return list(self)

    @property
    def size(self):
        return len(self)

    @property
    def shape(self):
        return (len(self),)

    def _items(self):
        if all(type(v) is int for v in self) or all(type(v) is bool for v in self):
            return [str(v) for v in self]
        if all(isinstance(v, str) for v in self):
            return [repr(v) for v in self]
        return None

    def __str__(self):
        it = self._items()
        if it is None:
            return list.__repr__(self)
        return "[" + " ".join(it) + "]"

    def __repr__(self):
        it = self._items()
        if it is None:
            return "array(%s)" % list.__repr__(self)
        if self and isinstance(self[0], str):
            return "array([%s], dtype=object)" % ", ".join(it)
        return "array([%s])" % ", ".join(it)


# ---------------------------------------------------------------- dtypes

_ALIASES = {
    "O": "object", "object": "object", "str": "object", "string": "object",
    "category": "object", "U": "object", "S": "object",
    "int": "int64", "int64": "int64", "int32": "int64", "int16": "int64",
    "int8": "int64", "i8": "int64", "i": "int64", "Int64": "int64",
    "uint8": "int64", "uint16": "int64", "uint32": "int64", "uint64": "int64",
    "float": "float64", "float64": "float64", "float32": "float64",
    "f8": "float64", "f": "float64", "double": "float64", "Float64": "float64",
    "bool": "bool", "boolean": "bool", "?": "bool",
    "datetime64[ns]": "datetime64[ns]", "datetime64": "datetime64[ns]",
    "datetime64[ms]": "datetime64[ns]", "datetime64[s]": "datetime64[ns]",
    "M8[ns]": "datetime64[ns]",
    "timedelta64[ns]": "timedelta64[ns]", "timedelta64": "timedelta64[ns]",
}
_KINDS = {"object": "O", "int64": "i", "float64": "f", "bool": "b",
          "datetime64[ns]": "M", "timedelta64[ns]": "m"}


def dtype_name(t):
    if t is None:
        return None
    if isinstance(t, dtype):
        return str.__str__(t)
    if t is object or t is str:
        return "object"
    if t is int:
        return "int64"
    if t is float:
        return "float64"
    if t is bool:
        return "bool"
    if t is datetime or t is date:
        return "datetime64[ns]"
    if isinstance(t, str):
        n = _ALIASES.get(t)
        if n is None:
            raise TypeError("data type %r not understood" % (t,))
        return n
    n = getattr(t, "__name__", None)
    if n in _ALIASES:
        return _ALIASES[n]
    raise TypeError("data type %r not understood" % (t,))


class dtype(str):
    """A dtype name that compares equal to its aliases and Python types."""

    def __new__(cls, t):
        return str.__new__(cls, dtype_name(t))

    @property
    def name(self):
        return str.__str__(self)

    @property
    def kind(self):
        return _KINDS.get(str.__str__(self), "O")

    @property
    def type(self):
        return {"object": object, "int64": int, "float64": float, "bool": bool,
                "datetime64[ns]": datetime, "timedelta64[ns]": timedelta}.get(self.name, object)

    def __repr__(self):
        n = self.name
        return "dtype('O')" if n == "object" else "dtype('%s')" % n

    def __str__(self):
        return str.__str__(self)

    def __eq__(self, other):
        try:
            return self.name == dtype_name(other)
        except TypeError:
            return False

    def __ne__(self, other):
        return not self.__eq__(other)

    def __hash__(self):
        return str.__hash__(self)


DT_OBJ = dtype("object")
DT_INT = dtype("int64")
DT_FLOAT = dtype("float64")
DT_BOOL = dtype("bool")
DT_DATE = dtype("datetime64[ns]")
DT_TD = dtype("timedelta64[ns]")


def is_numeric(dt):
    return dt in ("int64", "float64", "bool")


def infer_dtype(vals):
    kind = None
    has_na = False
    for v in vals:
        if v is None or v is NaT or v is NA or (type(v) is float and v != v):
            has_na = True
            continue
        t = type(v)
        if t is bool:
            k = 1
        elif t is int:
            k = 2
        elif t is float:
            k = 3
        elif t is datetime:
            k = 4
        elif t is timedelta:
            k = 5
        else:
            return DT_OBJ
        if kind is None:
            kind = k
        elif kind != k:
            if (kind == 2 and k == 3) or (kind == 3 and k == 2):
                kind = 3
            else:
                return DT_OBJ
    if kind is None:
        return DT_FLOAT if has_na else DT_OBJ
    if kind == 1:
        return DT_OBJ if has_na else DT_BOOL
    if kind == 2:
        return DT_FLOAT if has_na else DT_INT
    if kind == 3:
        return DT_FLOAT
    if kind == 4:
        return DT_DATE
    return DT_TD


def normalize(vals, dt):
    """Make vals hold dt's representation (in place when possible)."""
    if dt == "float64":
        out = []
        for v in vals:
            if v is None or v is NA or v is NaT:
                out.append(NaN)
            elif type(v) is float:
                out.append(v)
            else:
                out.append(float(v))
        return out
    if dt == "datetime64[ns]":
        return [NaT if isna_scalar(v) else v for v in vals]
    return vals


def convert(vals, dt):
    """astype: convert every value to dt."""
    dt = dtype(dt)
    out = []
    if dt == "int64":
        for v in vals:
            if isna_scalar(v):
                raise ValueError("cannot convert NA to integer")
            if isinstance(v, str):
                v = v.strip()
                try:
                    v = int(v)
                except ValueError:
                    v = int(float(v))
            out.append(int(v))
    elif dt == "float64":
        for v in vals:
            if isna_scalar(v):
                out.append(NaN)
            elif isinstance(v, str):
                out.append(float(v.strip().replace(",", "")) if v.strip() else NaN)
            else:
                out.append(float(v))
    elif dt == "bool":
        out = [bool(v) for v in vals]
    elif dt == "datetime64[ns]":
        out = [parse_datetime(v) for v in vals]
    elif dt == "object":
        out = list(vals)
    else:
        out = list(vals)
    return out, dt


def to_str(v):
    if isna_scalar(v):
        return NaN
    return str(v)


# ---------------------------------------------------------------- options

class _Options:
    def __init__(self):
        object.__setattr__(self, "_d", {
            "display.max_rows": 60,
            "display.min_rows": 10,
            "display.max_columns": 0,
            "display.width": 80,
            "display.precision": 6,
            "display.float_format": None,
            "display.max_colwidth": 50,
            "mode.chained_assignment": "warn",
            "future.infer_string": False,
            "mode.copy_on_write": True,
        })

    def _key(self, k):
        if k in self._d:
            return k
        hits = [x for x in self._d if x.endswith("." + k) or x == k]
        if len(hits) == 1:
            return hits[0]
        raise KeyError("No such option: %s" % k)

    def get(self, k):
        return self._d[self._key(k)]

    def set(self, k, v):
        self._d[self._key(k)] = v

    def __getattr__(self, k):
        return _OptionGroup(self, k)


class _OptionGroup:
    def __init__(self, opts, prefix):
        object.__setattr__(self, "_o", opts)
        object.__setattr__(self, "_p", prefix)

    def __getattr__(self, k):
        key = self._p + "." + k
        if key in self._o._d:
            return self._o._d[key]
        return _OptionGroup(self._o, key)

    def __setattr__(self, k, v):
        self._o.set(self._p + "." + k, v)


OPTIONS = _Options()


def get_option(k):
    return OPTIONS.get(k)


def set_option(*args):
    for i in range(0, len(args), 2):
        OPTIONS.set(args[i], args[i + 1])


class option_context:
    def __init__(self, *args):
        self._args = args
        self._old = []

    def __enter__(self):
        for i in range(0, len(self._args), 2):
            self._old.append((self._args[i], get_option(self._args[i])))
            set_option(self._args[i], self._args[i + 1])
        return self

    def __exit__(self, *a):
        for k, v in self._old:
            set_option(k, v)
        return False


# ---------------------------------------------------------------- dates

_MONTHS = ["january", "february", "march", "april", "may", "june", "july",
           "august", "september", "october", "november", "december"]
_ISO = re.compile(r"^(\d{4})-(\d{1,2})-(\d{1,2})(?:[T ](\d{1,2}):(\d{2})(?::(\d{2})(?:\.(\d{1,9}))?)?)?(Z|[+-]\d{2}:?\d{2})?$")
_SLASH_YMD = re.compile(r"^(\d{4})[/.](\d{1,2})[/.](\d{1,2})(?:[T ](\d{1,2}):(\d{2})(?::(\d{2}))?)?$")
_DMY = re.compile(r"^(\d{1,2})([/.\-])(\d{1,2})\2(\d{4})(?:[T ](\d{1,2}):(\d{2})(?::(\d{2}))?)?$")
_YM = re.compile(r"^(\d{4})-(\d{1,2})$")
_TEXT = re.compile(r"^(\d{1,2})?\s*([A-Za-z]+)\.?\s*(\d{1,2})?,?\s*(\d{4})$")


def _frac(s):
    return int((s + "000000")[:6]) if s else 0


def parse_datetime(v, fmt=None, dayfirst=False, errors="raise"):
    if isna_scalar(v):
        return NaT
    if isinstance(v, datetime):
        return v
    if isinstance(v, date):
        return datetime(v.year, v.month, v.day)
    if isinstance(v, (int, float)) and not isinstance(v, bool):
        return datetime(1970, 1, 1) + timedelta(seconds=v / 1e9)
    s = str(v).strip()
    n = len(s)
    if fmt is None and (n == 10 or n == 19) and s[4:5] == "-" and s[7:8] == "-":
        # the common ISO shapes without the regex engine
        try:
            if n == 10:
                return datetime(int(s[:4]), int(s[5:7]), int(s[8:10]))
            if s[10] in " T" and s[13] == ":" and s[16] == ":":
                return datetime(int(s[:4]), int(s[5:7]), int(s[8:10]), int(s[11:13]), int(s[14:16]), int(s[17:19]))
        except ValueError:
            pass
    try:
        if fmt is not None:
            return strptime(s, fmt)
        m = _ISO.match(s)
        if m:
            y, mo, d, h, mi, se, fr, tz = m.groups()
            return datetime(int(y), int(mo), int(d), int(h or 0), int(mi or 0), int(se or 0), _frac(fr))
        m = _SLASH_YMD.match(s)
        if m:
            y, mo, d, h, mi, se = m.groups()
            return datetime(int(y), int(mo), int(d), int(h or 0), int(mi or 0), int(se or 0))
        m = _DMY.match(s)
        if m:
            a, sep, b, y, h, mi, se = m.groups()
            a, b = int(a), int(b)
            if sep == "." or dayfirst or a > 12:
                d, mo = a, b
            else:
                mo, d = a, b
            return datetime(int(y), mo, d, int(h or 0), int(mi or 0), int(se or 0))
        m = _YM.match(s)
        if m:
            return datetime(int(m.group(1)), int(m.group(2)), 1)
        if re.match(r"^\d{4}$", s):
            return datetime(int(s), 1, 1)
        if re.match(r"^\d{8}$", s):
            return datetime(int(s[:4]), int(s[4:6]), int(s[6:]))
        m = _TEXT.match(s)
        if m:
            d1, mon, d2, y = m.groups()
            mo = _month_from_name(mon)
            if mo:
                return datetime(int(y), mo, int(d1 or d2 or 1))
    except (ValueError, TypeError):
        pass
    if errors == "coerce":
        return NaT
    if errors == "ignore":
        return v
    raise ValueError("could not convert string to Timestamp: %r" % (s,))


def _month_from_name(name):
    n = name.lower()
    for i, m in enumerate(_MONTHS):
        if m == n or (len(n) >= 3 and m.startswith(n)):
            return i + 1
    return 0


_STRP = {
    "%Y": r"(?P<Y>\d{4})", "%m": r"(?P<m>\d{1,2})", "%d": r"(?P<d>\d{1,2})",
    "%H": r"(?P<H>\d{1,2})", "%M": r"(?P<M>\d{1,2})", "%S": r"(?P<S>\d{1,2})",
    "%y": r"(?P<y>\d{2})", "%b": r"(?P<b>[A-Za-z]+)", "%B": r"(?P<B>[A-Za-z]+)",
    "%f": r"(?P<f>\d{1,6})", "%p": r"(?P<p>AM|PM|am|pm)", "%I": r"(?P<I>\d{1,2})",
    "%j": r"(?P<j>\d{1,3})", "%a": r"[A-Za-z]+", "%A": r"[A-Za-z]+", "%%": "%",
    "%z": r"(?P<z>[+-]\d{2}:?\d{2}|Z)", "%Z": r"[A-Za-z]+",
}
_strp_cache = {}


def strptime(s, fmt):
    rx = _strp_cache.get(fmt)
    if rx is None:
        out = []
        i = 0
        while i < len(fmt):
            if fmt[i] == "%" and i + 1 < len(fmt):
                code = fmt[i:i + 2]
                if code not in _STRP:
                    raise ValueError("unsupported format code %s" % code)
                out.append(_STRP[code])
                i += 2
            else:
                out.append(re.escape(fmt[i]))
                i += 1
        rx = _strp_cache[fmt] = re.compile("".join(out) + r"\Z")
    m = rx.match(s)
    if not m:
        raise ValueError("time data %r does not match format %r" % (s, fmt))
    g = m.groupdict()
    y = int(g["Y"]) if g.get("Y") else (2000 + int(g["y"]) if int(g["y"]) < 69 else 1900 + int(g["y"])) if g.get("y") else 1900
    mo = int(g["m"]) if g.get("m") else _month_from_name(g.get("b") or g.get("B") or "") or 1
    d = int(g["d"]) if g.get("d") else 1
    h = int(g["H"]) if g.get("H") else 0
    if g.get("I"):
        h = int(g["I"]) % 12 + (12 if (g.get("p") or "").lower() == "pm" else 0)
    out = datetime(y, mo, d, h, int(g.get("M") or 0), int(g.get("S") or 0), _frac(g.get("f")))
    if g.get("j"):
        out = datetime(y, 1, 1) + timedelta(days=int(g["j"]) - 1)
    return out


_DAYS = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"]


def strftime(d, fmt):
    if d is NaT:
        return NaN
    out = []
    i = 0
    while i < len(fmt):
        c = fmt[i]
        if c != "%" or i + 1 >= len(fmt):
            out.append(c)
            i += 1
            continue
        code = fmt[i + 1]
        i += 2
        if code == "Y":
            out.append("%04d" % d.year)
        elif code == "y":
            out.append("%02d" % (d.year % 100))
        elif code == "m":
            out.append("%02d" % d.month)
        elif code == "d":
            out.append("%02d" % d.day)
        elif code == "H":
            out.append("%02d" % d.hour)
        elif code == "I":
            out.append("%02d" % ((d.hour % 12) or 12))
        elif code == "p":
            out.append("AM" if d.hour < 12 else "PM")
        elif code == "M":
            out.append("%02d" % d.minute)
        elif code == "S":
            out.append("%02d" % d.second)
        elif code == "f":
            out.append("%06d" % d.microsecond)
        elif code == "B":
            out.append(_MONTHS[d.month - 1].capitalize())
        elif code == "b":
            out.append(_MONTHS[d.month - 1][:3].capitalize())
        elif code == "A":
            out.append(_DAYS[d.weekday()])
        elif code == "a":
            out.append(_DAYS[d.weekday()][:3])
        elif code == "j":
            out.append("%03d" % (d.timetuple().tm_yday if hasattr(d, "timetuple") else 1))
        elif code == "U" or code == "W":
            yday = (d - datetime(d.year, 1, 1)).days
            wd = d.weekday() if code == "W" else (d.weekday() + 1) % 7
            out.append("%02d" % ((yday + 7 - wd) // 7))
        elif code == "V":
            out.append("%02d" % d.isocalendar()[1])
        elif code == "G":
            out.append("%04d" % d.isocalendar()[0])
        elif code == "u":
            out.append(str(d.isoweekday()))
        elif code == "w":
            out.append(str((d.weekday() + 1) % 7))
        elif code == "q":
            out.append(str((d.month - 1) // 3 + 1))
        elif code == "%":
            out.append("%")
        else:
            out.append("%" + code)
    return "".join(out)


def fmt_datetime(d, date_only):
    if d is NaT or d is None:
        return "NaT"
    if date_only:
        return "%04d-%02d-%02d" % (d.year, d.month, d.day)
    s = "%04d-%02d-%02d %02d:%02d:%02d" % (d.year, d.month, d.day, d.hour, d.minute, d.second)
    if d.microsecond:
        s += ".%06d" % d.microsecond
    return s


def fmt_timedelta(t):
    if isna_scalar(t):
        return "NaT"
    days = t.days
    secs = t.seconds
    h, rem = divmod(secs, 3600)
    m, s = divmod(rem, 60)
    out = "%d days %02d:%02d:%02d" % (days, h, m, s)
    if t.microseconds:
        out += ".%06d" % t.microseconds
    return out


def month_end(d):
    if d.month == 12:
        return datetime(d.year, 12, 31)
    return datetime(d.year, d.month + 1, 1) - timedelta(days=1)


def add_months(d, n):
    m = d.month - 1 + n
    y = d.year + m // 12
    m = m % 12 + 1
    return y, m
