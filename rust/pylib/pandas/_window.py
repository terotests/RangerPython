# SPDX-License-Identifier: MIT
# rolling, expanding, ewm and resample for rpy's pandas.

import math
from datetime import datetime, timedelta

from ._util import NaN, NaT, isna_scalar, is_numeric, month_end, add_months, DT_FLOAT, DT_DATE
from ._index import Index, _hkey
from ._series import Series
from . import _agg


class _Window:
    def __init__(self, obj, min_periods):
        self._obj = obj
        self._min = min_periods

    def _windows(self, n):
        raise NotImplementedError

    def _apply_series(self, s, how, **kw):
        vals = s._values
        out = []
        for lo, hi in self._windows(len(vals)):
            w = [vals[i] for i in range(lo, hi)]
            clean = [v for v in w if not isna_scalar(v)]
            if len(clean) < (self._min if self._min is not None else (hi - lo)) or not clean and how != "count":
                out.append(NaN)
                continue
            if callable(how):
                out.append(how(Series(w)) if not kw.get("raw") else how(w))
            else:
                out.append(float(_agg.reduce(w, how, s._dtype, **kw)))
        return Series._new(out, s.index, s.name, DT_FLOAT)

    def _apply(self, how, **kw):
        from ._frame import DataFrame
        obj = self._obj
        if isinstance(obj, Series):
            return self._apply_series(obj, how, **kw)
        data = {}
        for c in obj._data:
            if is_numeric(obj._dtypes[c]):
                data[c] = self._apply_series(obj._col(c), how, **kw)._values
        return DataFrame._new(data, {c: DT_FLOAT for c in data}, obj.index)

    def sum(self): return self._apply("sum")
    def mean(self): return self._apply("mean")
    def median(self): return self._apply("median")
    def min(self): return self._apply("min")
    def max(self): return self._apply("max")
    def std(self, ddof=1): return self._apply("std", ddof=ddof)
    def var(self, ddof=1): return self._apply("var", ddof=ddof)
    def count(self): return self._apply("count")

    def quantile(self, q):
        return self._apply("quantile", q=q)

    def apply(self, func, raw=False):
        return self._apply(func, raw=raw)

    def agg(self, func):
        if isinstance(func, str):
            return getattr(self, func)()
        return self.apply(func)


class Rolling(_Window):
    def __init__(self, obj, window, min_periods=None, center=False):
        if isinstance(window, str):
            raise NotImplementedError("time-based rolling windows are not supported; use an integer window")
        _Window.__init__(self, obj, window if min_periods is None else min_periods)
        self._w = window
        self._center = center

    def _windows(self, n):
        w = self._w
        for i in range(n):
            if self._center:
                lo = i - (w - 1) // 2 - ((w - 1) % 2)
                lo = i - w // 2
                hi = lo + w
            else:
                lo, hi = i - w + 1, i + 1
            yield max(lo, 0), min(hi, n)


class Expanding(_Window):
    def __init__(self, obj, min_periods=1):
        _Window.__init__(self, obj, min_periods)

    def _windows(self, n):
        for i in range(n):
            yield 0, i + 1


class EWM:
    def __init__(self, obj, span=None, alpha=None, com=None, adjust=True):
        if alpha is None:
            if span is not None:
                alpha = 2.0 / (span + 1)
            elif com is not None:
                alpha = 1.0 / (com + 1)
            else:
                raise ValueError("Must pass one of com, span, halflife, or alpha")
        self._obj = obj
        self._a = alpha
        self._adjust = adjust

    def _series(self, s):
        a = self._a
        out = []
        num = den = 0.0
        prev = None
        for v in s._values:
            if isna_scalar(v):
                out.append(out[-1] if out else NaN)
                continue
            if self._adjust:
                num = v + (1 - a) * num
                den = 1 + (1 - a) * den
                out.append(num / den)
            else:
                prev = v if prev is None else (1 - a) * prev + a * v
                out.append(prev)
        return Series._new(out, s.index, s.name, DT_FLOAT)

    def mean(self):
        from ._frame import DataFrame
        if isinstance(self._obj, Series):
            return self._series(self._obj)
        o = self._obj
        return DataFrame._new({c: self._series(o._col(c))._values for c in o._data if is_numeric(o._dtypes[c])}, {}, o.index)


class GroupRolling:
    def __init__(self, gb, window, min_periods):
        self._gb = gb
        self._w = window
        self._min = min_periods

    def __getattr__(self, how):
        def call(*a, **k):
            from ._groupby import SeriesGroupBy
            gb = self._gb
            s = gb._obj
            if not isinstance(s, Series):
                raise NotImplementedError("rolling over a grouped frame: select one column first")
            labels, vals = [], []
            for key, p in zip(gb._g.order, gb._g.positions()):
                sub = s._take(p)
                r = getattr(Rolling(sub, self._w, self._min), how)(*a, **k)
                kv = gb._g.keys[key]
                for lab, v in zip(r.index._values, r._values):
                    labels.append((kv if isinstance(kv, tuple) else (kv,)) + (lab,))
                    vals.append(v)
            return Series._new(vals, Index._new(labels, gb._g.names + list(s.index._names)), s.name, DT_FLOAT)
        return call


# ------------------------------------------------------------------ resample

def _rule(rule):
    r = rule.upper()
    n = ""
    while r and r[0].isdigit():
        n += r[0]
        r = r[1:]
    return int(n or 1), r


def period_start(d, rule):
    n, r = _rule(rule)
    if r in ("D",):
        return datetime(d.year, d.month, d.day)
    if r in ("H", "h"):
        return datetime(d.year, d.month, d.day, d.hour)
    if r in ("T", "MIN"):
        return datetime(d.year, d.month, d.day, d.hour, d.minute)
    if r.startswith("W"):
        end_day = 6
        base = datetime(d.year, d.month, d.day)
        return base - timedelta(days=(base.weekday() - end_day - 1) % 7)
    if r in ("M", "ME", "MS"):
        return datetime(d.year, d.month, 1)
    if r in ("Q", "QE", "QS"):
        return datetime(d.year, 3 * ((d.month - 1) // 3) + 1, 1)
    if r in ("Y", "A", "YE", "YS", "AS"):
        return datetime(d.year, 1, 1)
    raise ValueError("Invalid frequency: %s" % rule)


def _bin_label(start, rule):
    n, r = _rule(rule)
    if r.startswith("W"):
        return start + timedelta(days=6)
    if r in ("M", "ME"):
        return month_end(start)
    if r in ("Q", "QE"):
        y, m = add_months(start, 2)
        return month_end(datetime(y, m, 1))
    if r in ("Y", "A", "YE"):
        return datetime(start.year, 12, 31)
    return start


def _next_start(start, rule):
    n, r = _rule(rule)
    if r == "D":
        return start + timedelta(days=n)
    if r in ("H",):
        return start + timedelta(hours=n)
    if r in ("T", "MIN"):
        return start + timedelta(minutes=n)
    if r.startswith("W"):
        return start + timedelta(weeks=n)
    if r in ("M", "ME", "MS"):
        y, m = add_months(start, n)
        return datetime(y, m, 1)
    if r in ("Q", "QE", "QS"):
        y, m = add_months(start, 3 * n)
        return datetime(y, m, 1)
    y = start.year + n
    return datetime(y, 1, 1)


class _Resampler:
    def __init__(self, obj, rule, dates):
        self._obj = obj
        self._rule = rule
        starts = [period_start(d, rule) if not isna_scalar(d) else None for d in dates]
        valid = [s for s in starts if s is not None]
        bins = []
        if valid:
            cur = min(valid)
            last = max(valid)
            while cur <= last:
                bins.append(cur)
                cur = _next_start(cur, rule)
        self._bins = bins
        pos = {b: [] for b in bins}
        for i, s in enumerate(starts):
            if s is not None:
                # with multi-period rules (2D, 3M) map onto the bin that holds s
                if s in pos:
                    pos[s].append(i)
                else:
                    for b in reversed(bins):
                        if b <= s:
                            pos[b].append(i)
                            break
        self._pos = pos
        self._labels = [_bin_label(b, rule) for b in bins]

    def _index(self, name):
        idx = Index._new(self._labels, [name], DT_DATE)
        n, r = _rule(self._rule)
        r = {"M": "ME", "Q": "QE-DEC", "QE": "QE-DEC", "Y": "YE-DEC", "YE": "YE-DEC", "A": "YE-DEC", "W": "W-SUN"}.get(r, r)
        idx.freq = (str(n) if n > 1 else "") + r
        return idx

    def _agg(self, how, **kw):
        from ._frame import DataFrame
        obj = self._obj
        empty_val = 0 if how in ("sum", "count", "size") else NaN
        if isinstance(obj, Series):
            vals = []
            for b in self._bins:
                p = self._pos[b]
                vals.append(_agg.reduce([obj._values[i] for i in p], how, obj._dtype, **kw) if p else empty_val)
            return Series._new(vals, self._index(self._name), obj.name)
        data = {}
        for c in obj._data:
            dt = obj._dtypes[c]
            if not is_numeric(dt) and how not in ("count", "first", "last", "size"):
                continue
            col = obj._data[c]
            data[c] = [_agg.reduce([col[i] for i in self._pos[b]], how, dt, **kw) if self._pos[b] else empty_val for b in self._bins]
        return DataFrame._new(data, {}, self._index(self._name))

    def sum(self, **kw): return self._agg("sum")
    def mean(self, **kw): return self._agg("mean")
    def median(self, **kw): return self._agg("median")
    def min(self, **kw): return self._agg("min")
    def max(self, **kw): return self._agg("max")
    def count(self): return self._agg("count")
    def std(self, ddof=1): return self._agg("std", ddof=ddof)
    def first(self): return self._agg("first")
    def last(self): return self._agg("last")
    def nunique(self): return self._agg("nunique")

    def size(self):
        return Series._new([len(self._pos[b]) for b in self._bins], self._index(self._name), None)

    def ohlc(self):
        from ._frame import DataFrame
        s = self._obj
        data = {k: [] for k in ("open", "high", "low", "close")}
        for b in self._bins:
            v = [s._values[i] for i in self._pos[b]]
            data["open"].append(v[0] if v else NaN)
            data["high"].append(max(v) if v else NaN)
            data["low"].append(min(v) if v else NaN)
            data["close"].append(v[-1] if v else NaN)
        return DataFrame._new(data, {}, self._index(self._name))

    def agg(self, func):
        from ._frame import DataFrame
        if isinstance(func, str):
            return getattr(self, func)()
        if isinstance(func, dict):
            data = {}
            for c, f in func.items():
                r = _Resampler.__new__(_Resampler)
                r.__dict__.update(self.__dict__)
                r._obj = self._obj._col(c)
                data[c] = r._agg(f if isinstance(f, str) else _agg.resolve(f))._values
            return DataFrame._new(data, {}, self._index(self._name))
        raise NotImplementedError("resample().agg with %r" % (func,))

    aggregate = agg

    def apply(self, func):
        return self._agg(func)

    def __getitem__(self, key):
        r = _Resampler.__new__(_Resampler)
        r.__dict__.update(self.__dict__)
        r._obj = self._obj[key]
        return r


def resample_series(s, rule, **kw):
    r = _Resampler(s, rule, s.index._values)
    r._name = s.index.name
    return r


def resample_frame(df, rule, on=None, **kw):
    if on is not None:
        dates = df._data[on]
        obj = df.drop(columns=[on])
        name = on
    else:
        dates = df.index._values
        obj = df
        name = df.index.name
    r = _Resampler(obj, rule, dates)
    r._name = name
    return r
