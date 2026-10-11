# SPDX-License-Identifier: MIT
# Series for rpy's pandas.

import math
import operator
import re
from datetime import datetime, date, timedelta

from ._util import (ArrayList, NaN, NaT, NA, isna_scalar, infer_dtype, normalize, convert,
                    dtype, is_numeric, DT_OBJ, DT_INT, DT_FLOAT, DT_BOOL,
                    DT_DATE, parse_datetime, strftime, month_end)
from ._index import Index, RangeIndex, default_index, ensure_index, _hkey, _sort_key
from . import _agg


def _is_listlike(v):
    return isinstance(v, (list, tuple, range, Index, Series)) or (hasattr(v, "__iter__") and not isinstance(v, (str, bytes, dict)) and hasattr(v, "__len__"))


def _na_for(dt):
    return NaT if dt == "datetime64[ns]" else NaN


def _safe(op):
    def f(a, b):
        if isna_scalar(a) or isna_scalar(b):
            if a is NaT or b is NaT:
                return NaT
            return NaN
        try:
            return op(a, b)
        except ZeroDivisionError:
            if op is operator.mod:
                return NaN
            if a == 0 or a != a:
                return NaN
            return math.inf if (a > 0) == (not _neg0(b)) else -math.inf
    return f


def _neg0(b):
    return math.copysign(1.0, b) < 0 if isinstance(b, float) else False


def _cmp(op, na_result=False):
    def f(a, b):
        if isna_scalar(a) or isna_scalar(b):
            return na_result
        return op(a, b)
    return f


def _bool(op):
    def f(a, b):
        if isna_scalar(a):
            a = False
        if isna_scalar(b):
            b = False
        r = op(a, b)
        return bool(r) if isinstance(a, bool) and isinstance(b, bool) else r
    return f


_ARITH = {
    "add": _safe(operator.add), "sub": _safe(operator.sub), "mul": _safe(operator.mul),
    "truediv": _safe(operator.truediv), "floordiv": _safe(operator.floordiv),
    "mod": _safe(operator.mod), "pow": _safe(operator.pow),
}
_CMP = {
    "eq": _cmp(operator.eq), "ne": _cmp(operator.ne, True), "lt": _cmp(operator.lt),
    "le": _cmp(operator.le), "gt": _cmp(operator.gt), "ge": _cmp(operator.ge),
}
_LOGIC = {"and": _bool(operator.and_), "or": _bool(operator.or_), "xor": _bool(operator.xor)}


def _flip(f):
    return lambda a, b: f(b, a)


def align_series(a, b, how="outer"):
    """Values of two Series on one shared index."""
    if a.index.equals(b.index):
        return a.index, a._values, b._values
    la, lb = a.index._get_lookup(), b.index._get_lookup()
    labels = list(a.index._values)
    seen = set(_hkey(v) for v in labels)
    extra = [v for v in b.index._values if _hkey(v) not in seen]
    labels = labels + extra
    try:
        labels = sorted(labels, key=_sort_key)
    except TypeError:
        pass
    av, bv = [], []
    for lab in labels:
        k = _hkey(lab)
        p = la.get(k)
        av.append(a._values[p if not isinstance(p, list) else p[0]] if p is not None else NaN)
        p = lb.get(k)
        bv.append(b._values[p if not isinstance(p, list) else p[0]] if p is not None else NaN)
    names = a.index._names if a.index._names == b.index._names else [None] * a.index.nlevels
    return Index._new(labels, names), av, bv


class Series:
    __array_priority__ = 1000

    def __init__(self, data=None, index=None, dtype=None, name=None, copy=False):
        if isinstance(data, Series):
            vals = list(data._values)
            idx = data.index
            if name is None:
                name = data.name
            if index is not None:
                idx2 = ensure_index(index)
                pos = idx.get_indexer(idx2._values)
                vals = [vals[p] if p >= 0 else NaN for p in pos]
                idx = idx2
        elif isinstance(data, dict):
            if index is not None:
                idx = ensure_index(index)
                vals = [data.get(k, NaN) for k in idx._values]
            else:
                keys = list(data.keys())
                vals = list(data.values())
                idx = Index(keys)
        elif data is None:
            vals = []
            idx = ensure_index(index, 0) if index is not None else default_index(0)
            if index is not None:
                vals = [NaN] * len(idx)
        elif isinstance(data, (str, int, float, bool)) or isna_scalar(data) or isinstance(data, (datetime, date)):
            idx = ensure_index(index, 1) if index is not None else default_index(1)
            vals = [data] * len(idx)
        else:
            if isinstance(data, Index):
                if name is None:
                    name = data.name
            vals = list(data)
            idx = ensure_index(index, len(vals))
            if len(idx) != len(vals):
                raise ValueError("Length of values (%d) does not match length of index (%d)" % (len(vals), len(idx)))
        if dtype is not None:
            vals, dt = convert(vals, dtype)
        else:
            dt = infer_dtype(vals)
            vals = normalize(vals, dt)
        self._values = vals
        self._index = idx
        self.name = name
        self._dtype = dt

    @classmethod
    def _new(cls, vals, index, name=None, dt=None):
        self = object.__new__(Series)
        if dt is None:
            dt = infer_dtype(vals)
            vals = normalize(vals, dt)
        self._values = vals
        self._index = index
        self.name = name
        self._dtype = dtype(dt) if not isinstance(dt, dtype) else dt
        return self

    def _derive(self, vals, dt=None, name=None, index=None):
        return Series._new(vals, self._index if index is None else index, self.name if name is None else name, dt)

    # ------------------------------------------------------------ basics
    @property
    def index(self):
        return self._index

    @index.setter
    def index(self, v):
        v = ensure_index(v, len(self._values))
        if len(v) != len(self._values):
            raise ValueError("Length mismatch")
        self._index = v

    @property
    def dtype(self):
        return self._dtype

    dtypes = dtype

    @property
    def values(self):
        return ArrayList(self._values)

    @property
    def array(self):
        return list(self._values)

    def to_numpy(self, dtype=None, na_value=None):
        if na_value is not None:
            return ArrayList(na_value if isna_scalar(v) else v for v in self._values)
        return ArrayList(self._values)

    def tolist(self):
        return list(self._values)

    to_list = tolist

    @property
    def shape(self):
        return (len(self._values),)

    @property
    def size(self):
        return len(self._values)

    @property
    def ndim(self):
        return 1

    @property
    def empty(self):
        return not self._values

    @property
    def hasnans(self):
        return any(isna_scalar(v) for v in self._values)

    @property
    def axes(self):
        return [self._index]

    def __len__(self):
        return len(self._values)

    def __iter__(self):
        return iter(self._values)

    def __contains__(self, key):
        return key in self._index

    def __bool__(self):
        raise ValueError("The truth value of a Series is ambiguous. Use a.empty, a.bool(), a.item(), a.any() or a.all().")

    def __hash__(self):
        raise TypeError("unhashable type: 'Series'")

    def keys(self):
        return self._index

    def items(self):
        return zip(self._index._values, self._values)

    iteritems = items

    def item(self):
        if len(self._values) != 1:
            raise ValueError("can only convert an array of size 1 to a Python scalar")
        return self._values[0]

    def copy(self, deep=True):
        return Series._new(list(self._values), self._index, self.name, self._dtype)

    def __repr__(self):
        from ._fmt import series_to_string
        return series_to_string(self)

    __str__ = __repr__

    def to_string(self, index=True, header=True, name=False, dtype=False, length=False, max_rows=None):
        from ._fmt import format_values, _index_texts
        vals = [v.strip() if False else v for v in format_values(self._values, self._dtype)]
        vw = max((len(v) for v in vals), default=0)
        if not index:
            return "\n".join(v.rjust(vw) for v in vals)
        it = _index_texts(self._index)[0] if self._index.nlevels == 1 else [str(v) for v in self._index._values]
        iw = max((len(x) for x in it), default=0)
        return "\n".join(i.ljust(iw) + "   " + v.rjust(vw) for i, v in zip(it, vals))

    def to_markdown(self, index=True, **kw):
        from ._fmt import to_markdown
        return to_markdown(self, index=index)

    def to_frame(self, name=None):
        from ._frame import DataFrame
        n = name if name is not None else (self.name if self.name is not None else 0)
        return DataFrame._new({n: list(self._values)}, {n: self._dtype}, self._index)

    def to_dict(self, into=dict):
        return dict(zip(self._index._values, self._values))

    def to_csv(self, path_or_buf=None, **kw):
        return self.to_frame().to_csv(path_or_buf, **kw)

    def to_json(self, path_or_buf=None, orient="index", **kw):
        from ._io import series_to_json
        return series_to_json(self, path_or_buf, orient, **kw)

    def rename(self, index=None, inplace=False, **kw):
        out = self if inplace else self.copy()
        if callable(index) and not isinstance(index, (dict, Series)):
            out._index = Index._new([index(v) for v in self._index._values], self._index._names)
        elif isinstance(index, dict):
            out._index = Index._new([index.get(v, v) for v in self._index._values], self._index._names)
        else:
            out.name = index
        return None if inplace else out

    def rename_axis(self, mapper=None, index=None, **kw):
        out = self.copy()
        nm = index if index is not None else mapper
        out._index = self._index.copy()
        out._index._names = list(nm) if isinstance(nm, (list, tuple)) else [nm]
        return out

    def set_axis(self, labels, axis=0, **kw):
        out = self.copy()
        out.index = labels
        return out

    def astype(self, dt, errors="raise", copy=True):
        if dt is str or dt == "str" or dt == "string":
            vals = [v if isna_scalar(v) else str(v) for v in self._values] if dt != str else [str(v) if not isna_scalar(v) else NaN for v in self._values]
            return self._derive(vals, DT_OBJ)
        if dt == "category":
            return self._derive(list(self._values), DT_OBJ)
        vals, d = convert(self._values, dt)
        return self._derive(vals, d)

    def infer_objects(self):
        return self._derive(list(self._values), None)

    def head(self, n=5):
        if n < 0:
            n = max(len(self._values) + n, 0)
        return self.iloc[:n]

    def tail(self, n=5):
        if n == 0:
            return self.iloc[0:0]
        return self.iloc[-n:]

    def _take(self, positions):
        vals = self._values
        return Series._new([vals[i] for i in positions], self._index.take(positions), self.name, self._dtype)

    # ------------------------------------------------------------ selection
    def __getitem__(self, key):
        if isinstance(key, slice):
            if (key.start is None or isinstance(key.start, int)) and (key.stop is None or isinstance(key.stop, int)):
                return self.iloc[key]
            return self.loc[key]
        if isinstance(key, Series) and key._dtype == "bool":
            return self._take(_mask_positions(key, self._index))
        if isinstance(key, (list, Index)) or (isinstance(key, Series)):
            keys = list(key)
            if keys and isinstance(keys[0], bool):
                return self._take([i for i, b in enumerate(keys) if b])
            return self.loc[keys]
        if isinstance(key, tuple) and self._index.nlevels == 1:
            return self.loc[list(key)]
        try:
            p = self._index.get_loc(key)
        except KeyError:
            if isinstance(key, int) and not isinstance(key, bool) and self._index.dtype != "int64":
                return self._values[key]
            raise
        if isinstance(p, list):
            if self._index.nlevels > 1 and not isinstance(key, tuple):
                sub = self._take(p)
                sub._index = self._index.take(p).droplevel(0)
                return sub
            return self._take(p)
        return self._values[p]

    def __setitem__(self, key, value):
        self._values = list(self._values)
        if isinstance(key, Series) and key._dtype == "bool":
            positions = _mask_positions(key, self._index)
        elif isinstance(key, list) and key and isinstance(key[0], bool):
            positions = [i for i, b in enumerate(key) if b]
        elif isinstance(key, slice):
            positions = list(range(len(self._values)))[key]
        elif isinstance(key, (list, Index)):
            positions = [self._index._positions(k)[0] for k in key]
        else:
            try:
                positions = self._index._positions(key)
            except KeyError:
                if isinstance(key, int) and self._index.dtype != "int64":
                    positions = [key]
                else:
                    self._values.append(value)
                    self._index = Index._new(self._index._values + [key], self._index._names)
                    self._retype()
                    return
        if isinstance(value, Series):
            vals = value.reindex(self._index.take(positions))._values if not value.index.equals(self._index.take(positions)) else value._values
        elif _is_listlike(value) and not isinstance(value, str):
            vals = list(value)
        else:
            vals = [value] * len(positions)
        for p, v in zip(positions, vals):
            self._values[p] = v
        self._retype()

    def _retype(self):
        dt = infer_dtype(self._values)
        self._values = normalize(self._values, dt)
        self._dtype = dt

    @property
    def loc(self):
        return _SeriesLoc(self)

    @property
    def iloc(self):
        return _SeriesILoc(self)

    @property
    def at(self):
        return _SeriesLoc(self)

    @property
    def iat(self):
        return _SeriesILoc(self)

    def get(self, key, default=None):
        try:
            return self[key]
        except (KeyError, IndexError):
            return default

    # ------------------------------------------------------------ operators
    def _binop(self, other, f, name_keep=True):
        from ._frame import DataFrame
        if isinstance(other, DataFrame):
            return NotImplemented
        if isinstance(other, Series):
            idx, a, b = align_series(self, other)
            vals = [f(x, y) for x, y in zip(a, b)]
            name = self.name if self.name == other.name else None
            return Series._new(vals, idx, name)
        if isinstance(other, Index):
            other = other._values
        if isinstance(other, (list, tuple, range)):
            other = list(other)
            if len(other) != len(self._values):
                raise ValueError("Lengths must match")
            return Series._new([f(x, y) for x, y in zip(self._values, other)], self._index, self.name)
        if isinstance(other, str) and self._dtype == "datetime64[ns]":
            other = parse_datetime(other)
        return Series._new([f(x, other) for x in self._values], self._index, self.name)

    def __add__(self, o): return self._binop(o, _ARITH["add"])
    def __radd__(self, o): return self._binop(o, _flip(_ARITH["add"]))
    def __sub__(self, o): return self._binop(o, _ARITH["sub"])
    def __rsub__(self, o): return self._binop(o, _flip(_ARITH["sub"]))
    def __mul__(self, o): return self._binop(o, _ARITH["mul"])
    def __rmul__(self, o): return self._binop(o, _flip(_ARITH["mul"]))
    def __truediv__(self, o): return self._binop(o, _truediv)
    def __rtruediv__(self, o): return self._binop(o, _flip(_truediv))
    def __floordiv__(self, o): return self._binop(o, _ARITH["floordiv"])
    def __rfloordiv__(self, o): return self._binop(o, _flip(_ARITH["floordiv"]))
    def __mod__(self, o): return self._binop(o, _ARITH["mod"])
    def __rmod__(self, o): return self._binop(o, _flip(_ARITH["mod"]))
    def __pow__(self, o): return self._binop(o, _ARITH["pow"])
    def __rpow__(self, o): return self._binop(o, _flip(_ARITH["pow"]))
    def __eq__(self, o): return self._binop(o, _CMP["eq"])
    def __ne__(self, o): return self._binop(o, _CMP["ne"])
    def __lt__(self, o): return self._binop(o, _CMP["lt"])
    def __le__(self, o): return self._binop(o, _CMP["le"])
    def __gt__(self, o): return self._binop(o, _CMP["gt"])
    def __ge__(self, o): return self._binop(o, _CMP["ge"])
    def __and__(self, o): return self._binop(o, _LOGIC["and"])
    def __rand__(self, o): return self._binop(o, _LOGIC["and"])
    def __or__(self, o): return self._binop(o, _LOGIC["or"])
    def __ror__(self, o): return self._binop(o, _LOGIC["or"])
    def __xor__(self, o): return self._binop(o, _LOGIC["xor"])

    def add(self, o, fill_value=None): return self._fill_op(o, _ARITH["add"], fill_value)
    def sub(self, o, fill_value=None): return self._fill_op(o, _ARITH["sub"], fill_value)
    def mul(self, o, fill_value=None): return self._fill_op(o, _ARITH["mul"], fill_value)
    def div(self, o, fill_value=None): return self._fill_op(o, _truediv, fill_value)
    truediv = divide = div
    multiply = mul
    subtract = sub
    def floordiv(self, o, fill_value=None): return self._fill_op(o, _ARITH["floordiv"], fill_value)
    def mod(self, o, fill_value=None): return self._fill_op(o, _ARITH["mod"], fill_value)
    def pow(self, o, fill_value=None): return self._fill_op(o, _ARITH["pow"], fill_value)
    def radd(self, o): return self.__radd__(o)
    def rsub(self, o): return self.__rsub__(o)
    def rmul(self, o): return self.__rmul__(o)
    def rdiv(self, o): return self.__rtruediv__(o)
    rtruediv = rdiv
    def eq(self, o): return self.__eq__(o)
    def ne(self, o): return self.__ne__(o)
    def lt(self, o): return self.__lt__(o)
    def le(self, o): return self.__le__(o)
    def gt(self, o): return self.__gt__(o)
    def ge(self, o): return self.__ge__(o)

    def _fill_op(self, o, f, fill_value):
        if fill_value is None:
            return self._binop(o, f)
        if isinstance(o, Series):
            idx, a, b = align_series(self, o)
            vals = []
            for x, y in zip(a, b):
                if isna_scalar(x) and not isna_scalar(y):
                    x = fill_value
                elif isna_scalar(y) and not isna_scalar(x):
                    y = fill_value
                vals.append(f(x, y))
            return Series._new(vals, idx, self.name if self.name == o.name else None)
        return self.fillna(fill_value)._binop(o, f)

    def __neg__(self):
        return self._derive([v if isna_scalar(v) else -v for v in self._values])

    def __pos__(self):
        return self.copy()

    def __abs__(self):
        return self.abs()

    def __invert__(self):
        if self._dtype == "bool":
            return self._derive([not v for v in self._values], DT_BOOL)
        return self._derive([~v for v in self._values])

    def __round__(self, n=0):
        return self.round(n)

    def __float__(self):
        return float(self.item())

    def __int__(self):
        return int(self.item())

    def abs(self):
        return self._derive([v if isna_scalar(v) else abs(v) for v in self._values], self._dtype)

    def round(self, decimals=0):
        if self._dtype == "float64":
            return self._derive([v if v != v else _round(v, decimals) for v in self._values], DT_FLOAT)
        if self._dtype == "object":
            return self._derive([v if not isinstance(v, float) or v != v else _round(v, decimals) for v in self._values])
        return self.copy()

    def clip(self, lower=None, upper=None):
        out = []
        for v in self._values:
            if not isna_scalar(v):
                if lower is not None and v < lower:
                    v = lower
                if upper is not None and v > upper:
                    v = upper
            out.append(v)
        return self._derive(out)

    # ------------------------------------------------------------ missing
    def isna(self):
        return self._derive([isna_scalar(v) for v in self._values], DT_BOOL)

    isnull = isna

    def notna(self):
        return self._derive([not isna_scalar(v) for v in self._values], DT_BOOL)

    notnull = notna

    def fillna(self, value=None, method=None, inplace=False, limit=None):
        if method in ("ffill", "pad"):
            out = self.ffill()
        elif method in ("bfill", "backfill"):
            out = self.bfill()
        elif isinstance(value, (dict, Series)):
            m = value if isinstance(value, dict) else value.to_dict()
            out = self._derive([m.get(k, v) if isna_scalar(v) else v for k, v in zip(self._index._values, self._values)], None)
        else:
            out = self._derive([value if isna_scalar(v) else v for v in self._values], None)
        if inplace:
            self._values, self._dtype = out._values, out._dtype
            return None
        return out

    def ffill(self, limit=None):
        out = []
        last = NaN
        for v in self._values:
            if isna_scalar(v):
                out.append(last)
            else:
                last = v
                out.append(v)
        return self._derive(out, None)

    pad = ffill

    def bfill(self, limit=None):
        out = []
        nxt = NaN
        for v in reversed(self._values):
            if isna_scalar(v):
                out.append(nxt)
            else:
                nxt = v
                out.append(v)
        return self._derive(out[::-1], None)

    backfill = bfill

    def interpolate(self, method="linear"):
        vals = list(self._values)
        n = len(vals)
        i = 0
        while i < n:
            if isna_scalar(vals[i]):
                j = i
                while j < n and isna_scalar(vals[j]):
                    j += 1
                if i > 0 and j < n:
                    a, b = vals[i - 1], vals[j]
                    for k in range(i, j):
                        vals[k] = a + (b - a) * (k - i + 1) / (j - i + 1)
                elif i > 0:
                    for k in range(i, j):
                        vals[k] = vals[i - 1]
                i = j
            else:
                i += 1
        return self._derive(vals, DT_FLOAT)

    def dropna(self, inplace=False, **kw):
        keep = [i for i, v in enumerate(self._values) if not isna_scalar(v)]
        out = self._take(keep)
        if inplace:
            self._values, self._index = out._values, out._index
            return None
        return out

    def replace(self, to_replace=None, value=None, regex=False, inplace=False):
        if isinstance(to_replace, dict):
            m = {_hkey(k): v for k, v in to_replace.items()}
        elif isinstance(to_replace, (list, tuple)):
            if isinstance(value, (list, tuple)):
                m = {_hkey(k): v for k, v in zip(to_replace, value)}
            else:
                m = {_hkey(k): value for k in to_replace}
        else:
            m = {_hkey(to_replace): value}
        if regex:
            vals = []
            for v in self._values:
                if isinstance(v, str):
                    for k, r in m.items():
                        v = re.sub(k, r, v)
                vals.append(v)
        else:
            vals = [m.get(_hkey(v), v) if _hashable(v) else v for v in self._values]
        out = self._derive(vals, None)
        if inplace:
            self._values, self._dtype = out._values, out._dtype
            return None
        return out

    def where(self, cond, other=NaN):
        c = _cond_list(cond, self)
        if isinstance(other, Series):
            o = other.reindex(self._index)._values
        else:
            o = [other] * len(self._values)
        return self._derive([v if k else ov for v, k, ov in zip(self._values, c, o)], None)

    def mask(self, cond, other=NaN):
        c = _cond_list(cond, self)
        return self.where([not k for k in c], other)

    # ------------------------------------------------------------ reductions
    def _reduce(self, how, skipna=True, **kw):
        return _agg.reduce(self._values, how, self._dtype, skipna=skipna, **kw)

    def sum(self, axis=None, skipna=True, numeric_only=False, min_count=0, **kw):
        return self._reduce("sum", skipna, min_count=min_count)

    def mean(self, axis=None, skipna=True, numeric_only=False, **kw):
        _check_numeric(self, "mean")
        return self._reduce("mean", skipna)

    def median(self, axis=None, skipna=True, numeric_only=False, **kw):
        _check_numeric(self, "median")
        return self._reduce("median", skipna)

    def min(self, axis=None, skipna=True, **kw):
        return self._reduce("min", skipna)

    def max(self, axis=None, skipna=True, **kw):
        return self._reduce("max", skipna)

    def std(self, axis=None, skipna=True, ddof=1, **kw):
        _check_numeric(self, "std")
        return self._reduce("std", skipna, ddof=ddof)

    def var(self, axis=None, skipna=True, ddof=1, **kw):
        _check_numeric(self, "var")
        return self._reduce("var", skipna, ddof=ddof)

    def sem(self, axis=None, skipna=True, ddof=1, **kw):
        return self._reduce("sem", skipna, ddof=ddof)

    def prod(self, axis=None, skipna=True, **kw):
        return self._reduce("prod", skipna)

    product = prod

    def skew(self, axis=None, skipna=True, **kw):
        return self._reduce("skew", skipna)

    def count(self):
        return self._reduce("count")

    def nunique(self, dropna=True):
        n = self._reduce("nunique")
        if not dropna and self.hasnans:
            n += 1
        return n

    def any(self, axis=0, skipna=True, **kw):
        return any(bool(v) for v in self._values if not isna_scalar(v))

    def all(self, axis=0, skipna=True, **kw):
        return all(bool(v) for v in self._values if not isna_scalar(v))

    def quantile(self, q=0.5, interpolation="linear"):
        xs = _agg._clean(self._values)
        if isinstance(q, (list, tuple)):
            return Series._new([_agg.quantile(xs, x, interpolation) for x in q], Index(list(q)), self.name, DT_FLOAT)
        r = _agg.quantile(xs, q, interpolation)
        return float(r) if self._dtype in ("int64", "float64") and r == r else r

    def mode(self, dropna=True):
        m = _agg._mode(_agg._clean(self._values))
        return Series._new(m, default_index(len(m)), self.name, self._dtype if m else None)

    def idxmax(self, skipna=True):
        best = None
        for i, v in enumerate(self._values):
            if isna_scalar(v):
                continue
            if best is None or v > self._values[best]:
                best = i
        if best is None:
            return NaN
        return self._index._values[best]

    def idxmin(self, skipna=True):
        best = None
        for i, v in enumerate(self._values):
            if isna_scalar(v):
                continue
            if best is None or v < self._values[best]:
                best = i
        if best is None:
            return NaN
        return self._index._values[best]

    def argmax(self):
        return self._index.get_loc(self.idxmax())

    def argmin(self):
        return self._index.get_loc(self.idxmin())

    def agg(self, func=None, *args, **kw):
        if isinstance(func, (list, tuple)):
            names = [_agg.func_name(f) for f in func]
            vals = [self.agg(f) for f in func]
            return Series._new(vals, Index(names), self.name)
        if isinstance(func, dict):
            return Series._new([self.agg(f) for f in func.values()], Index(list(func.keys())), self.name)
        f = _agg.resolve(func)
        if isinstance(f, str):
            return getattr(self, f)(*args, **kw)
        return f(self, *args, **kw)

    aggregate = agg

    def describe(self, percentiles=None, include=None):
        from ._frame import _describe_column
        labels, vals = _describe_column(self._values, self._dtype, percentiles)
        dt = DT_FLOAT if is_numeric(self._dtype) else None
        return Series._new(vals, Index(labels), self.name, dt)

    def value_counts(self, normalize=False, sort=True, ascending=False, bins=None, dropna=True):
        counts = {}
        first = {}
        for v in self._values:
            if isna_scalar(v):
                if dropna:
                    continue
            k = _hkey(v)
            if k in counts:
                counts[k] += 1
            else:
                counts[k] = 1
                first[k] = v
        keys = list(counts.keys())
        if sort:
            keys.sort(key=lambda k: counts[k], reverse=not ascending)
        vals = [counts[k] for k in keys]
        name = "count"
        if normalize:
            tot = sum(vals)
            vals = [v / tot for v in vals]
            name = "proportion"
        idx = Index._new([first[k] for k in keys], [self.name])
        return Series._new(vals, idx, name, DT_FLOAT if normalize else DT_INT)

    def unique(self):
        seen = set()
        out = ArrayList()
        for v in self._values:
            k = _hkey(v)
            if k not in seen:
                seen.add(k)
                out.append(v)
        return out

    def drop_duplicates(self, keep="first", inplace=False):
        dup = self.duplicated(keep)._values
        return self._take([i for i, d in enumerate(dup) if not d])

    def duplicated(self, keep="first"):
        seen = {}
        out = [False] * len(self._values)
        order = range(len(self._values)) if keep != "last" else range(len(self._values) - 1, -1, -1)
        for i in order:
            k = _hkey(self._values[i])
            if k in seen:
                out[i] = True
                if keep is False:
                    out[seen[k]] = True
            else:
                seen[k] = i
        return self._derive(out, DT_BOOL)

    @property
    def is_unique(self):
        return len(self.unique()) == len(self._values)

    @property
    def is_monotonic_increasing(self):
        v = self._values
        return all(v[i] <= v[i + 1] for i in range(len(v) - 1))

    @property
    def is_monotonic_decreasing(self):
        v = self._values
        return all(v[i] >= v[i + 1] for i in range(len(v) - 1))

    # ------------------------------------------------------------ transforms
    def map(self, arg, na_action=None):
        if isinstance(arg, dict):
            m = {_hkey(k): v for k, v in arg.items()}
            vals = [m.get(_hkey(v), NaN) for v in self._values]
        elif isinstance(arg, Series):
            d = arg.to_dict()
            vals = [d.get(v, NaN) for v in self._values]
        else:
            vals = [v if (na_action == "ignore" and isna_scalar(v)) else arg(v) for v in self._values]
        return self._derive(vals, None)

    def apply(self, func, convert_dtype=True, args=(), **kw):
        if isinstance(func, str):
            return getattr(self, func)(*args, **kw)
        vals = [func(v, *args, **kw) for v in self._values]
        if vals and all(isinstance(v, Series) for v in vals):
            from ._frame import DataFrame
            return DataFrame([v.to_dict() for v in vals], index=self._index)
        return self._derive(vals, None)

    def transform(self, func):
        if isinstance(func, str):
            r = getattr(self, func)()
            return r if isinstance(r, Series) else self._derive([r] * len(self._values), None)
        return self.apply(func)

    def pipe(self, func, *a, **kw):
        return func(self, *a, **kw)

    def isin(self, values):
        if isinstance(values, Series):
            values = values._values
        s = set(_hkey(v) for v in values)
        return self._derive([_hkey(v) in s for v in self._values], DT_BOOL)

    def between(self, left, right, inclusive="both"):
        out = []
        for v in self._values:
            if isna_scalar(v):
                out.append(False)
                continue
            lo = v >= left if inclusive in ("both", "left") else v > left
            hi = v <= right if inclusive in ("both", "right") else v < right
            out.append(lo and hi)
        return self._derive(out, DT_BOOL)

    def cumsum(self, skipna=True):
        return self._cum(lambda a, b: a + b)

    def cumprod(self, skipna=True):
        return self._cum(lambda a, b: a * b)

    def cummax(self, skipna=True):
        return self._cum(lambda a, b: a if a >= b else b)

    def cummin(self, skipna=True):
        return self._cum(lambda a, b: a if a <= b else b)

    def _cum(self, f):
        out = []
        acc = None
        for v in self._values:
            if isna_scalar(v):
                out.append(NaN)
                continue
            acc = v if acc is None else f(acc, v)
            out.append(acc)
        return self._derive(out, None)

    def diff(self, periods=1):
        v = self._values
        n = len(v)
        out = []
        for i in range(n):
            j = i - periods
            if 0 <= j < n and not isna_scalar(v[i]) and not isna_scalar(v[j]):
                out.append(v[i] - v[j])
            else:
                out.append(NaN)
        if self._dtype == "datetime64[ns]":
            return self._derive(out, None)
        return self._derive([float(x) if not isinstance(x, timedelta) and x == x else x for x in out], None)

    def shift(self, periods=1, fill_value=None):
        v = self._values
        n = len(v)
        fill = _na_for(self._dtype) if fill_value is None else fill_value
        out = []
        for i in range(n):
            j = i - periods
            out.append(v[j] if 0 <= j < n else fill)
        return self._derive(out, None)

    def pct_change(self, periods=1, fill_method=None):
        v = self._values
        out = []
        for i in range(len(v)):
            j = i - periods
            if 0 <= j < len(v) and not isna_scalar(v[i]) and not isna_scalar(v[j]):
                out.append(_truediv(v[i], v[j]) - 1 if v[j] != 0 else (math.inf if v[i] > v[j] else NaN))
            else:
                out.append(NaN)
        return self._derive(out, DT_FLOAT)

    def rank(self, method="average", ascending=True, pct=False, na_option="keep"):
        v = self._values
        order = [i for i in range(len(v)) if not isna_scalar(v[i])]
        order.sort(key=lambda i: v[i], reverse=not ascending)
        ranks = [NaN] * len(v)
        i = 0
        dense = 0
        while i < len(order):
            j = i
            while j + 1 < len(order) and v[order[j + 1]] == v[order[i]]:
                j += 1
            dense += 1
            for k in range(i, j + 1):
                if method == "average":
                    r = (i + j) / 2 + 1
                elif method == "min":
                    r = i + 1
                elif method == "max":
                    r = j + 1
                elif method == "first":
                    r = k + 1
                elif method == "dense":
                    r = dense
                else:
                    r = (i + j) / 2 + 1
                ranks[order[k]] = float(r)
            i = j + 1
        if pct:
            n = dense if method == "dense" else len(order)
            ranks = [r / n if r == r else r for r in ranks]
        return self._derive(ranks, DT_FLOAT)

    def sort_values(self, ascending=True, inplace=False, na_position="last", ignore_index=False, key=None, kind=None):
        vals = self._values if key is None else key(self)._values
        pos = _sorted_positions([vals], [ascending], na_position)
        out = self._take(pos)
        if ignore_index:
            out._index = default_index(len(pos))
        if inplace:
            self._values, self._index = out._values, out._index
            return None
        return out

    def sort_index(self, ascending=True, inplace=False, level=None, key=None, na_position="last"):
        labels = self._index._values
        if key is not None:
            labels = key(self._index)._values if hasattr(key(self._index), "_values") else list(key(self._index))
        pos = _sorted_positions([labels], [ascending], na_position)
        out = self._take(pos)
        if inplace:
            self._values, self._index = out._values, out._index
            return None
        return out

    def argsort(self):
        return Series._new(_sorted_positions([self._values], [True], "last"), self._index, self.name, DT_INT)

    def nlargest(self, n=5, keep="first"):
        return self.sort_values(ascending=False).head(n)

    def nsmallest(self, n=5, keep="first"):
        return self.sort_values(ascending=True).head(n)

    def reset_index(self, level=None, drop=False, name=None, inplace=False):
        from ._frame import DataFrame
        if drop:
            out = Series._new(list(self._values), default_index(len(self._values)), self.name, self._dtype)
            if inplace:
                self._index = out._index
                return None
            return out
        df = self.to_frame(name if name is not None else (self.name if self.name is not None else 0))
        return df.reset_index()

    def reindex(self, index=None, fill_value=None):
        idx = ensure_index(index)
        pos = self._index.get_indexer(idx._values)
        fill = _na_for(self._dtype) if fill_value is None else fill_value
        vals = [self._values[p] if p >= 0 else fill for p in pos]
        return Series._new(vals, idx, self.name)

    def drop(self, labels=None, index=None, errors="raise", inplace=False):
        labels = labels if labels is not None else index
        if not isinstance(labels, (list, tuple, Index)):
            labels = [labels]
        s = set(_hkey(l) for l in labels)
        keep = [i for i, v in enumerate(self._index._values) if _hkey(v) not in s]
        return self._take(keep)

    def set_index(self, *a, **k):
        raise AttributeError("'Series' object has no attribute 'set_index'")

    def explode(self):
        vals, labels = [], []
        for lab, v in zip(self._index._values, self._values):
            if isinstance(v, (list, tuple)):
                for x in v:
                    vals.append(x)
                    labels.append(lab)
                if not v:
                    vals.append(NaN)
                    labels.append(lab)
            else:
                vals.append(v)
                labels.append(lab)
        return Series._new(vals, Index._new(labels, self._index._names), self.name)

    def sample(self, n=None, frac=None, replace=False, random_state=None):
        from ._frame import _sample_positions
        return self._take(_sample_positions(len(self._values), n, frac, replace, random_state))

    def equals(self, other):
        return isinstance(other, Series) and self._index.equals(other._index) and all(
            (a == b) or (isna_scalar(a) and isna_scalar(b)) for a, b in zip(self._values, other._values)) and len(self) == len(other)

    def corr(self, other, method="pearson"):
        from ._frame import _pearson, _spearman
        idx, a, b = align_series(self, other)
        pairs = [(x, y) for x, y in zip(a, b) if not isna_scalar(x) and not isna_scalar(y)]
        if method == "spearman":
            return _spearman([p[0] for p in pairs], [p[1] for p in pairs])
        return _pearson([p[0] for p in pairs], [p[1] for p in pairs])

    def cov(self, other, ddof=1):
        idx, a, b = align_series(self, other)
        pairs = [(x, y) for x, y in zip(a, b) if not isna_scalar(x) and not isna_scalar(y)]
        n = len(pairs)
        if n - ddof <= 0:
            return NaN
        mx = sum(p[0] for p in pairs) / n
        my = sum(p[1] for p in pairs) / n
        return sum((p[0] - mx) * (p[1] - my) for p in pairs) / (n - ddof)

    def autocorr(self, lag=1):
        return self.corr(self.shift(lag))

    def groupby(self, by=None, level=None, sort=True, dropna=True, as_index=True, **kw):
        from ._groupby import SeriesGroupBy
        return SeriesGroupBy(self, by, level=level, sort=sort, dropna=dropna)

    def rolling(self, window, min_periods=None, center=False):
        from ._window import Rolling
        return Rolling(self, window, min_periods, center)

    def expanding(self, min_periods=1):
        from ._window import Expanding
        return Expanding(self, min_periods)

    def ewm(self, span=None, alpha=None, com=None, adjust=True):
        from ._window import EWM
        return EWM(self, span=span, alpha=alpha, com=com, adjust=adjust)

    def resample(self, rule, **kw):
        from ._window import resample_series
        return resample_series(self, rule, **kw)

    def unstack(self, level=-1, fill_value=None):
        from ._frame import DataFrame
        idx = self._index
        if idx.nlevels < 2:
            raise ValueError("index must be a MultiIndex to unstack")
        rows, cols, cells = [], [], {}
        rseen, cseen = {}, {}
        for t, v in zip(idx._values, self._values):
            r = t[:-1] if len(t) > 2 else t[0]
            c = t[-1]
            if _hkey(r) not in rseen:
                rseen[_hkey(r)] = True
                rows.append(r)
            if _hkey(c) not in cseen:
                cseen[_hkey(c)] = True
                cols.append(c)
            cells[(_hkey(r), _hkey(c))] = v
        try:
            rows.sort(key=_sort_key)
            cols.sort(key=_sort_key)
        except TypeError:
            pass
        fill = NaN if fill_value is None else fill_value
        data = {c: [cells.get((_hkey(r), _hkey(c)), fill) for r in rows] for c in cols}
        rnames = idx._names[:-1]
        ridx = Index._new(rows, rnames)
        out = DataFrame(data, index=ridx)
        out._columns_names = [idx._names[-1]]
        return out

    def to_period(self, freq="M"):
        return self.dt.to_period(freq)

    # ------------------------------------------------------------ accessors
    @property
    def str(self):
        return _StrAccessor(self)

    @property
    def dt(self):
        return _DtAccessor(self)

    @property
    def cat(self):
        return _CatAccessor(self)

    @property
    def plot(self):
        from ._plot import PlotAccessor
        return PlotAccessor(self)

    def __getattr__(self, name):
        if name.startswith("_"):
            raise AttributeError(name)
        idx = self.__dict__.get("_index")
        if idx is not None and idx.dtype == "object" and name in idx:
            return self[name]
        raise AttributeError("'Series' object has no attribute '%s'" % name)

    def __array__(self, *a):
        return list(self._values)


def _round(v, d):
    """numpy's round: scale, round half to even, scale back."""
    if v != v or v in (math.inf, -math.inf):
        return v
    if d >= 0:
        f = 10.0 ** d
        y = v * f
        r = round(y)
        return float(r) / f if abs(y) < 1e300 else v
    f = 10.0 ** -d
    return float(round(v / f)) * f


def _truediv(a, b):
    if isna_scalar(a) or isna_scalar(b):
        return NaN
    if isinstance(a, timedelta) and isinstance(b, timedelta):
        return a / b
    if b == 0:
        if a == 0 or a != a:
            return NaN
        return math.inf if (a > 0) != _neg0(b) else -math.inf
    return a / b


def _hashable(v):
    try:
        hash(v)
        return True
    except TypeError:
        return False


def _check_numeric(s, how):
    if s._dtype == "object" and any(isinstance(v, str) for v in s._values):
        raise TypeError("Could not convert string to numeric for '%s'" % how)


def _mask_positions(mask, index):
    if mask.index.equals(index):
        return [i for i, b in enumerate(mask._values) if b is True or (b and not isna_scalar(b))]
    m = mask.reindex(index)._values
    return [i for i, b in enumerate(m) if b is True]


def _cond_list(cond, s):
    if callable(cond) and not isinstance(cond, Series):
        cond = cond(s)
    if isinstance(cond, Series):
        return [bool(b) and not isna_scalar(b) for b in cond.reindex(s._index)._values] if not cond.index.equals(s._index) else [b is True or (bool(b) and not isna_scalar(b)) for b in cond._values]
    return list(cond)


def _sorted_positions(keylists, ascending, na_position="last"):
    n = len(keylists[0]) if keylists else 0
    order = list(range(n))
    if isinstance(ascending, bool):
        ascending = [ascending] * len(keylists)
    for vals, asc in reversed(list(zip(keylists, ascending))):
        good = [i for i in order if not isna_scalar(vals[i])]
        bad = [i for i in order if isna_scalar(vals[i])]
        try:
            good.sort(key=lambda i: vals[i], reverse=not asc)
        except TypeError:
            good.sort(key=lambda i: _sort_key(vals[i]), reverse=not asc)
        order = good + bad if na_position == "last" else bad + good
    return order


# -------------------------------------------------------------------- loc

class _SeriesLoc:
    def __init__(self, s):
        self._s = s

    def __getitem__(self, key):
        s = self._s
        if isinstance(key, Series) and key._dtype == "bool":
            return s._take(_mask_positions(key, s._index))
        if isinstance(key, slice):
            return s._take(_label_slice(s._index, key))
        if isinstance(key, (list, Index)) or (isinstance(key, Series)):
            keys = list(key)
            if keys and isinstance(keys[0], bool):
                return s._take([i for i, b in enumerate(keys) if b])
            pos = []
            for k in keys:
                pos.extend(s._index._positions(k))
            return s._take(pos)
        if callable(key):
            return self[key(s)]
        p = s._index.get_loc(key)
        if isinstance(p, list):
            return s._take(p)
        return s._values[p]

    def __setitem__(self, key, value):
        s = self._s
        if isinstance(key, slice):
            s.__setitem__(_label_slice(s._index, key) and [s._index._values[i] for i in _label_slice(s._index, key)], value)
            return
        s.__setitem__(key, value)


class _SeriesILoc:
    def __init__(self, s):
        self._s = s

    def __getitem__(self, key):
        s = self._s
        n = len(s._values)
        if isinstance(key, slice):
            return s._take(list(range(n))[key])
        if isinstance(key, (list, tuple, Series)):
            keys = list(key)
            if keys and isinstance(keys[0], bool):
                return s._take([i for i, b in enumerate(keys) if b])
            return s._take([k if k >= 0 else n + k for k in keys])
        if key < 0:
            key += n
        if not 0 <= key < n:
            raise IndexError("single positional indexer is out-of-bounds")
        return s._values[key]

    def __setitem__(self, key, value):
        s = self._s
        s._values = list(s._values)
        n = len(s._values)
        positions = list(range(n))[key] if isinstance(key, slice) else (list(key) if isinstance(key, (list, tuple)) else [key])
        vals = list(value) if _is_listlike(value) and not isinstance(value, str) else [value] * len(positions)
        for p, v in zip(positions, vals):
            s._values[p] = v
        s._retype()


def _label_slice(index, sl):
    n = len(index)
    start = 0
    stop = n
    if sl.start is not None:
        p = index.get_loc(sl.start)
        start = p[0] if isinstance(p, list) else p
    if sl.stop is not None:
        p = index.get_loc(sl.stop)
        stop = (p[-1] if isinstance(p, list) else p) + 1
    return list(range(start, stop))[::sl.step or 1]


# -------------------------------------------------------------------- str

class _StrAccessor:
    def __init__(self, s, as_index=False):
        self._s = s
        self._as_index = as_index

    def _map(self, f, dt=None):
        vals = [v if not isinstance(v, str) and isna_scalar(v) else f(v) for v in self._s._values]
        out = self._s._derive(vals, dt)
        if self._as_index:
            return Index(out._values, name=self._s.name)
        return out

    def _bool(self, f, na=None):
        vals = [(na if na is not None else v) if isna_scalar(v) else f(v) for v in self._s._values]
        out = self._s._derive(vals, DT_BOOL if all(isinstance(v, bool) for v in vals) else None)
        if self._as_index:
            return out._values
        return out

    def lower(self): return self._map(lambda v: str(v).lower())
    def upper(self): return self._map(lambda v: str(v).upper())
    def title(self): return self._map(lambda v: str(v).title())
    def capitalize(self): return self._map(lambda v: str(v).capitalize())
    def swapcase(self): return self._map(lambda v: str(v).swapcase())
    def casefold(self): return self._map(lambda v: str(v).casefold())
    def strip(self, to_strip=None): return self._map(lambda v: str(v).strip(to_strip))
    def lstrip(self, to_strip=None): return self._map(lambda v: str(v).lstrip(to_strip))
    def rstrip(self, to_strip=None): return self._map(lambda v: str(v).rstrip(to_strip))
    def len(self): return self._map(lambda v: len(v))
    def zfill(self, w): return self._map(lambda v: str(v).zfill(w))
    def center(self, w, fillchar=" "): return self._map(lambda v: str(v).center(w, fillchar))
    def ljust(self, w, fillchar=" "): return self._map(lambda v: str(v).ljust(w, fillchar))
    def rjust(self, w, fillchar=" "): return self._map(lambda v: str(v).rjust(w, fillchar))

    def pad(self, width, side="left", fillchar=" "):
        if side == "left":
            return self.rjust(width, fillchar)
        if side == "right":
            return self.ljust(width, fillchar)
        return self.center(width, fillchar)

    def slice(self, start=None, stop=None, step=None):
        return self._map(lambda v: v[start:stop:step])

    def __getitem__(self, key):
        if isinstance(key, slice):
            return self._map(lambda v: v[key])
        return self._map(lambda v: v[key] if -len(v) <= key < len(v) else NaN)

    def get(self, i):
        return self._map(lambda v: v[i] if (isinstance(v, (list, tuple)) and -len(v) <= i < len(v)) or (isinstance(v, str) and -len(v) <= i < len(v)) else (v.get(i, NaN) if isinstance(v, dict) else NaN))

    def contains(self, pat, case=True, flags=0, na=None, regex=True):
        if regex:
            rx = re.compile(pat, flags | (0 if case else re.IGNORECASE))
            return self._bool(lambda v: rx.search(v) is not None, na)
        if not case:
            p = pat.lower()
            return self._bool(lambda v: p in v.lower(), na)
        return self._bool(lambda v: pat in v, na)

    def startswith(self, pat, na=None):
        return self._bool(lambda v: v.startswith(pat), na)

    def endswith(self, pat, na=None):
        return self._bool(lambda v: v.endswith(pat), na)

    def match(self, pat, case=True, flags=0, na=None):
        rx = re.compile(pat, flags | (0 if case else re.IGNORECASE))
        return self._bool(lambda v: rx.match(v) is not None, na)

    def fullmatch(self, pat, case=True, flags=0, na=None):
        rx = re.compile(pat, flags | (0 if case else re.IGNORECASE))
        return self._bool(lambda v: rx.fullmatch(v) is not None, na)

    def isdigit(self): return self._bool(lambda v: v.isdigit())
    def isnumeric(self): return self._bool(lambda v: v.isnumeric())
    def isdecimal(self): return self._bool(lambda v: v.isdecimal())
    def isalpha(self): return self._bool(lambda v: v.isalpha())
    def isalnum(self): return self._bool(lambda v: v.isalnum())
    def isspace(self): return self._bool(lambda v: v.isspace())
    def islower(self): return self._bool(lambda v: v.islower())
    def isupper(self): return self._bool(lambda v: v.isupper())
    def istitle(self): return self._bool(lambda v: v.istitle())

    def replace(self, pat, repl, n=-1, case=None, flags=0, regex=False):
        if regex or (not isinstance(pat, str)):
            rx = re.compile(pat, flags | (re.IGNORECASE if case is False else 0)) if isinstance(pat, str) else pat
            if callable(repl):
                return self._map(lambda v: rx.sub(repl, v, count=max(n, 0)))
            return self._map(lambda v: rx.sub(repl, v, count=max(n, 0)))
        return self._map(lambda v: v.replace(pat, repl, n))

    def find(self, sub, start=0, end=None):
        return self._map(lambda v: v.find(sub, start, end if end is not None else len(v)))

    def rfind(self, sub, start=0, end=None):
        return self._map(lambda v: v.rfind(sub, start, end if end is not None else len(v)))

    def count(self, pat, flags=0):
        rx = re.compile(pat, flags)
        return self._map(lambda v: len(rx.findall(v)))

    def findall(self, pat, flags=0):
        rx = re.compile(pat, flags)
        return self._map(lambda v: rx.findall(v))

    def repeat(self, n):
        return self._map(lambda v: v * n)

    def join(self, sep):
        return self._map(lambda v: sep.join(v))

    def cat(self, others=None, sep=None, na_rep=None):
        sep = sep or ""
        if others is None:
            return sep.join(v for v in self._s._values if isinstance(v, str))
        if isinstance(others, Series):
            others = [others]
        out = []
        for i, v in enumerate(self._s._values):
            parts = [v] + [o._values[i] if isinstance(o, Series) else o[i] for o in others]
            if any(isna_scalar(p) for p in parts):
                out.append(NaN if na_rep is None else sep.join(na_rep if isna_scalar(p) else str(p) for p in parts))
            else:
                out.append(sep.join(str(p) for p in parts))
        return self._s._derive(out, None)

    def split(self, pat=None, n=-1, expand=False, regex=None):
        if pat is not None and (regex or (regex is None and len(pat) > 1 and any(c in pat for c in ".*+?[](){}|\\^$"))):
            rx = re.compile(pat)
            f = lambda v: rx.split(v, maxsplit=max(n, 0))
        else:
            f = lambda v: v.split(pat, n)
        if not expand:
            return self._map(f)
        return self._expand([f(v) if isinstance(v, str) else None for v in self._s._values])

    def rsplit(self, pat=None, n=-1, expand=False):
        f = lambda v: v.rsplit(pat, n)
        if not expand:
            return self._map(f)
        return self._expand([f(v) if isinstance(v, str) else None for v in self._s._values])

    def partition(self, sep=" ", expand=True):
        return self._expand([list(v.partition(sep)) if isinstance(v, str) else None for v in self._s._values])

    def _expand(self, parts):
        from ._frame import DataFrame
        width = max((len(p) for p in parts if p is not None), default=0)
        data = {}
        for j in range(width):
            data[j] = [(p[j] if j < len(p) else None) if p is not None else NaN for p in parts]
        return DataFrame(data, index=self._s._index)

    def extract(self, pat, flags=0, expand=True):
        from ._frame import DataFrame
        rx = re.compile(pat, flags)
        names = [None] * rx.groups
        for nm, i in rx.groupindex.items():
            names[i - 1] = nm
        rows = []
        for v in self._s._values:
            m = rx.search(v) if isinstance(v, str) else None
            rows.append([NaN if m is None or m.group(i + 1) is None else m.group(i + 1) for i in range(rx.groups)])
        if rx.groups == 1 and not expand:
            return self._s._derive([r[0] for r in rows], None, name=names[0])
        data = {(names[j] if names[j] is not None else j): [r[j] for r in rows] for j in range(rx.groups)}
        return DataFrame(data, index=self._s._index)

    def extractall(self, pat, flags=0):
        raise NotImplementedError("str.extractall")

    def get_dummies(self, sep="|"):
        from ._frame import DataFrame
        cats = set()
        rows = []
        for v in self._s._values:
            items = v.split(sep) if isinstance(v, str) else []
            rows.append(set(items))
            cats.update(items)
        cats = sorted(cats)
        return DataFrame({c: [1 if c in r else 0 for r in rows] for c in cats}, index=self._s._index)

    def normalize(self, form):
        import unicodedata
        return self._map(lambda v: unicodedata.normalize(form, v))

    def encode(self, encoding="utf-8"):
        return self._map(lambda v: v.encode(encoding))

    def decode(self, encoding="utf-8"):
        return self._map(lambda v: v.decode(encoding))

    def wrap(self, width):
        import textwrap
        return self._map(lambda v: "\n".join(textwrap.wrap(v, width)))

    def removeprefix(self, p):
        return self._map(lambda v: v[len(p):] if v.startswith(p) else v)

    def removesuffix(self, p):
        return self._map(lambda v: v[:-len(p)] if p and v.endswith(p) else v)


# -------------------------------------------------------------------- dt

_DAY_NAMES = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"]
_MONTH_NAMES = ["January", "February", "March", "April", "May", "June", "July",
                "August", "September", "October", "November", "December"]


class _DtAccessor:
    def __init__(self, s):
        if s._dtype not in ("datetime64[ns]", "timedelta64[ns]"):
            if s._dtype == "object" and all(isinstance(v, (datetime, date)) or isna_scalar(v) for v in s._values):
                pass
            else:
                raise AttributeError("Can only use .dt accessor with datetimelike values")
        self._s = s

    def _map(self, f, dt=None):
        vals = [NaN if isna_scalar(v) else f(v) for v in self._s._values]
        return self._s._derive(vals, dt)

    def _int(self, f):
        out = self._map(f)
        return out

    year = property(lambda self: self._int(lambda d: d.year))
    month = property(lambda self: self._int(lambda d: d.month))
    day = property(lambda self: self._int(lambda d: d.day))
    hour = property(lambda self: self._int(lambda d: d.hour))
    minute = property(lambda self: self._int(lambda d: d.minute))
    second = property(lambda self: self._int(lambda d: d.second))
    microsecond = property(lambda self: self._int(lambda d: d.microsecond))
    dayofweek = property(lambda self: self._int(lambda d: d.weekday()))
    weekday = dayofweek
    day_of_week = dayofweek
    dayofyear = property(lambda self: self._int(lambda d: d.timetuple().tm_yday))
    day_of_year = dayofyear
    quarter = property(lambda self: self._int(lambda d: (d.month - 1) // 3 + 1))
    days_in_month = property(lambda self: self._int(lambda d: month_end(d).day))
    is_month_end = property(lambda self: self._map(lambda d: d.day == month_end(d).day, DT_BOOL))
    is_month_start = property(lambda self: self._map(lambda d: d.day == 1, DT_BOOL))
    date = property(lambda self: self._map(lambda d: d.date() if isinstance(d, datetime) else d, DT_OBJ))
    time = property(lambda self: self._map(lambda d: d.time(), DT_OBJ))

    @property
    def days(self):
        return self._int(lambda t: t.days)

    @property
    def seconds(self):
        return self._int(lambda t: t.seconds)

    def total_seconds(self):
        return self._map(lambda t: t.total_seconds(), DT_FLOAT)

    def isocalendar(self):
        from ._frame import DataFrame
        cal = [d.isocalendar() if not isna_scalar(d) else (NaN, NaN, NaN) for d in self._s._values]
        return DataFrame({"year": [c[0] for c in cal], "week": [c[1] for c in cal], "day": [c[2] for c in cal]}, index=self._s._index)

    @property
    def week(self):
        return self._int(lambda d: d.isocalendar()[1])

    weekofyear = week

    def day_name(self):
        return self._map(lambda d: _DAY_NAMES[d.weekday()], DT_OBJ)

    def month_name(self):
        return self._map(lambda d: _MONTH_NAMES[d.month - 1], DT_OBJ)

    def strftime(self, fmt):
        return self._map(lambda d: strftime(d, fmt), DT_OBJ)

    def normalize(self):
        return self._map(lambda d: datetime(d.year, d.month, d.day), DT_DATE)

    def floor(self, freq):
        from ._window import period_start
        return self._map(lambda d: period_start(d, freq), DT_DATE)

    def to_period(self, freq="M"):
        f = freq.upper()
        if f in ("M", "ME", "MS"):
            g = lambda d: "%04d-%02d" % (d.year, d.month)
        elif f in ("Q", "QE", "QS"):
            g = lambda d: "%04dQ%d" % (d.year, (d.month - 1) // 3 + 1)
        elif f in ("Y", "A", "YE", "YS", "AS"):
            g = lambda d: "%04d" % d.year
        elif f.startswith("W"):
            def g(d):
                start = datetime(d.year, d.month, d.day) - timedelta(days=d.weekday())
                end = start + timedelta(days=6)
                return "%s/%s" % (start.strftime("%Y-%m-%d") if hasattr(start, "strftime") else start, end.strftime("%Y-%m-%d"))
        else:
            g = lambda d: "%04d-%02d-%02d" % (d.year, d.month, d.day)
        return self._map(g, DT_OBJ)

    def tz_localize(self, tz):
        return self._s.copy()

    def tz_convert(self, tz):
        return self._s.copy()


class _CatAccessor:
    def __init__(self, s):
        self._s = s

    @property
    def categories(self):
        return Index(sorted(set(v for v in self._s._values if not isna_scalar(v)), key=_sort_key))

    @property
    def codes(self):
        cats = self.categories._values
        m = {c: i for i, c in enumerate(cats)}
        return self._s._derive([m.get(v, -1) for v in self._s._values], DT_INT)
