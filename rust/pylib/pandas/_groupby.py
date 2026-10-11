# SPDX-License-Identifier: MIT
# groupby for rpy's pandas.

from ._util import NaN, isna_scalar, infer_dtype, is_numeric, DT_INT, DT_FLOAT, DT_OBJ
from ._index import Index, default_index, _hkey, _sort_key
from ._series import Series, _sorted_positions
from . import _agg


class _Grouper:
    def __init__(self, obj, by, level=None, sort=True, dropna=True):
        from ._frame import DataFrame
        n = len(obj)
        arrays = []
        names = []
        self.key_columns = []
        if by is None and level is None:
            raise TypeError("You have to supply one of 'by' and 'level'")
        if level is not None:
            levels = level if isinstance(level, (list, tuple)) else [level]
            idx = obj.index
            for lv in levels:
                if idx.nlevels == 1:
                    arrays.append(idx._values)
                    names.append(idx.name)
                else:
                    li = lv if isinstance(lv, int) else idx._names.index(lv)
                    arrays.append([t[li] for t in idx._values])
                    names.append(idx._names[li])
        else:
            bys = by if isinstance(by, list) else [by]
            for b in bys:
                if isinstance(b, Series):
                    if b.index.equals(obj.index):
                        arrays.append(b._values)
                    else:
                        arrays.append(b.reindex(obj.index)._values)
                    names.append(b.name)
                elif isinstance(b, Index):
                    arrays.append(b._values)
                    names.append(b.name)
                elif callable(b) and not isinstance(b, str):
                    arrays.append([b(v) for v in obj.index._values])
                    names.append(None)
                elif isinstance(b, dict):
                    arrays.append([b.get(v, NaN) for v in obj.index._values])
                    names.append(None)
                elif isinstance(b, (list, tuple)) and len(b) == n and not isinstance(by, list):
                    arrays.append(list(b))
                    names.append(None)
                elif isinstance(obj, DataFrame) and b in obj._data:
                    arrays.append(obj._data[b])
                    names.append(b)
                    self.key_columns.append(b)
                elif b in obj.index.names:
                    li = list(obj.index.names).index(b)
                    arrays.append(obj.index._values if obj.index.nlevels == 1 else [t[li] for t in obj.index._values])
                    names.append(b)
                else:
                    raise KeyError(b)
        self.names = names
        self.multi = len(arrays) > 1
        groups = {}
        keys = {}
        order = []
        if not self.multi:
            arr = arrays[0]
            for i in range(n):
                v = arr[i]
                if dropna and isna_scalar(v):
                    continue
                k = _hkey(v)
                g = groups.get(k)
                if g is None:
                    groups[k] = [i]
                    keys[k] = v
                    order.append(k)
                else:
                    g.append(i)
        else:
            for i in range(n):
                t = tuple(a[i] for a in arrays)
                if dropna and any(isna_scalar(v) for v in t):
                    continue
                k = tuple(_hkey(v) for v in t)
                g = groups.get(k)
                if g is None:
                    groups[k] = [i]
                    keys[k] = t
                    order.append(k)
                else:
                    g.append(i)
        if sort:
            try:
                order.sort(key=lambda k: _sort_key(keys[k]))
            except TypeError:
                order.sort(key=lambda k: str(keys[k]))
        self.order = order
        self.groups = groups
        self.keys = keys
        self.arrays = arrays

    def result_index(self):
        vals = [self.keys[k] for k in self.order]
        if self.multi:
            return Index._new(vals, self.names)
        return Index._new(vals, self.names, infer_dtype(vals) if vals else None)

    def positions(self):
        return [self.groups[k] for k in self.order]


def _apply_how(vals, how, dt, kw):
    if isinstance(how, str):
        return _agg.reduce(vals, how, dt, grouped=True, **kw)
    return how(Series._new(vals, default_index(len(vals)), None, dt))


class _GroupByBase:
    def __len__(self):
        return len(self._g.order)

    @property
    def ngroups(self):
        return len(self._g.order)

    @property
    def groups(self):
        out = {}
        idx = self._obj.index
        for k in self._g.order:
            out[self._g.keys[k]] = Index._new([idx._values[i] for i in self._g.groups[k]], idx._names)
        return out

    @property
    def indices(self):
        return {self._g.keys[k]: list(self._g.groups[k]) for k in self._g.order}

    def __iter__(self):
        for k in self._g.order:
            yield self._g.keys[k], self._obj._take(self._g.groups[k])

    def get_group(self, name):
        k = tuple(_hkey(v) for v in name) if self._g.multi else _hkey(name)
        if k not in self._g.groups:
            raise KeyError(name)
        return self._obj._take(self._g.groups[k])

    def size(self):
        idx = self._g.result_index()
        s = Series._new([len(p) for p in self._g.positions()], idx, None, DT_INT)
        if not getattr(self, "_as_index", True):
            df = s.to_frame("size").reset_index()
            return df
        return s

    def ngroup(self):
        out = [0] * len(self._obj)
        for gi, p in enumerate(self._g.positions()):
            for i in p:
                out[i] = gi
        return Series._new(out, self._obj.index, None, DT_INT)

    def cumcount(self, ascending=True):
        out = [0] * len(self._obj)
        for p in self._g.positions():
            m = len(p)
            for j, i in enumerate(p):
                out[i] = j if ascending else m - 1 - j
        return Series._new(out, self._obj.index, None, DT_INT)

    def head(self, n=5):
        keep = sorted(i for p in self._g.positions() for i in p[:n])
        return self._obj._take(keep)

    def tail(self, n=5):
        keep = sorted(i for p in self._g.positions() for i in p[-n:])
        return self._obj._take(keep)

    def nth(self, n):
        keep = sorted(p[n] for p in self._g.positions() if -len(p) <= n < len(p))
        return self._obj._take(keep)

    def filter(self, func, dropna=True):
        keep = []
        for p in self._g.positions():
            if func(self._obj._take(p)):
                keep.extend(p)
        return self._obj._take(sorted(keep))


class SeriesGroupBy(_GroupByBase):
    def __init__(self, s, by=None, level=None, sort=True, dropna=True, grouper=None):
        self._obj = s
        self._g = grouper if grouper is not None else _Grouper(s, by, level, sort, dropna)
        self._as_index = True

    def _agg1(self, how, **kw):
        s = self._obj
        if how in _agg.NUMERIC_ONLY and s._dtype == "object" and any(isinstance(v, str) for v in s._values):
            raise TypeError("agg function failed [how->%s,dtype->object]" % how)
        vals = s._values
        out = [_apply_how([vals[i] for i in p], how, s._dtype, kw) for p in self._g.positions()]
        rdt = _agg.result_dtype(how, s._dtype) if isinstance(how, str) else None
        if rdt is None and isinstance(how, str) and how in ("min", "max", "first", "last", "sum", "prod") and s._dtype in ("int64", "bool", "datetime64[ns]") and not any(isna_scalar(v) for v in out):
            rdt = s._dtype if how != "sum" or s._dtype != "bool" else DT_INT
        return Series._new(out, self._g.result_index(), s.name, rdt)

    def sum(self, numeric_only=False, min_count=0): return self._agg1("sum", min_count=min_count)
    def mean(self, numeric_only=False): return self._agg1("mean")
    def median(self, numeric_only=False): return self._agg1("median")
    def min(self, numeric_only=False): return self._agg1("min")
    def max(self, numeric_only=False): return self._agg1("max")
    def std(self, ddof=1, numeric_only=False): return self._agg1("std", ddof=ddof)
    def var(self, ddof=1, numeric_only=False): return self._agg1("var", ddof=ddof)
    def sem(self, ddof=1): return self._agg1("sem", ddof=ddof)
    def prod(self, numeric_only=False): return self._agg1("prod")
    def count(self): return self._agg1("count")
    def nunique(self, dropna=True): return self._agg1("nunique")
    def first(self, numeric_only=False): return self._agg1("first")
    def last(self, numeric_only=False): return self._agg1("last")
    def any(self): return self._agg1("any")
    def all(self): return self._agg1("all")
    def skew(self): return self._agg1("skew")

    def quantile(self, q=0.5):
        return self._agg1("quantile", q=q)

    def idxmax(self):
        s = self._obj
        return Series._new([s._take(p).idxmax() for p in self._g.positions()], self._g.result_index(), s.name)

    def idxmin(self):
        s = self._obj
        return Series._new([s._take(p).idxmin() for p in self._g.positions()], self._g.result_index(), s.name)

    def agg(self, func=None, *args, **kw):
        from ._frame import DataFrame
        if func is None and kw:
            data = {}
            for name, f in kw.items():
                data[name] = self.agg(f)._values
            return DataFrame._new(data, {}, self._g.result_index())
        if isinstance(func, (list, tuple)):
            data = {}
            for f in func:
                if isinstance(f, tuple):
                    name, f = f
                else:
                    name = _agg.func_name(f)
                data[name] = self.agg(f)._values
            return DataFrame._new(data, {}, self._g.result_index())
        if isinstance(func, dict):
            data = {name: self.agg(f)._values for name, f in func.items()}
            return DataFrame._new(data, {}, self._g.result_index())
        f = _agg.resolve(func)
        if isinstance(f, str):
            if f == "size":
                return Series._new([len(p) for p in self._g.positions()], self._g.result_index(), self._obj.name, DT_INT)
            return getattr(self, f)(*args, **kw)
        s = self._obj
        out = [f(s._take(p), *args, **kw) for p in self._g.positions()]
        return Series._new(out, self._g.result_index(), s.name)

    aggregate = agg

    def apply(self, func, *args, **kw):
        from ._reshape import concat
        s = self._obj
        res = [func(s._take(p), *args, **kw) for p in self._g.positions()]
        if res and all(isinstance(r, Series) for r in res):
            keys = [self._g.keys[k] for k in self._g.order]
            labels = []
            vals = []
            for key, r in zip(keys, res):
                for lab, v in zip(r.index._values, r._values):
                    labels.append((key if not isinstance(key, tuple) else key) + (lab,) if isinstance(key, tuple) else (key, lab))
                    vals.append(v)
            return Series._new(vals, Index._new(labels, self._g.names + list(res[0].index._names)), s.name)
        return Series._new(res, self._g.result_index(), s.name)

    def transform(self, func, *args, **kw):
        s = self._obj
        out = [NaN] * len(s)
        for p in self._g.positions():
            sub = s._take(p)
            if isinstance(func, str):
                r = getattr(SeriesGroupBy(sub, [0] * len(p)), func)() if func in ("cumsum", "cumcount", "rank", "shift", "diff", "pct_change", "cummax", "cummin", "ffill", "bfill") else _agg.reduce(sub._values, func, sub._dtype)
            else:
                r = func(sub, *args, **kw)
            if isinstance(r, Series):
                for j, i in enumerate(p):
                    out[i] = r._values[j]
            else:
                for i in p:
                    out[i] = r
        return Series._new(out, s.index, s.name)

    def _per_group(self, f):
        s = self._obj
        out = [NaN] * len(s)
        for p in self._g.positions():
            r = f(s._take(p))
            for j, i in enumerate(p):
                out[i] = r._values[j]
        return Series._new(out, s.index, s.name)

    def cumsum(self): return self._per_group(lambda x: x.cumsum())
    def cumprod(self): return self._per_group(lambda x: x.cumprod())
    def cummax(self): return self._per_group(lambda x: x.cummax())
    def cummin(self): return self._per_group(lambda x: x.cummin())
    def shift(self, periods=1, fill_value=None): return self._per_group(lambda x: x.shift(periods, fill_value))
    def diff(self, periods=1): return self._per_group(lambda x: x.diff(periods))
    def pct_change(self, periods=1): return self._per_group(lambda x: x.pct_change(periods))
    def ffill(self): return self._per_group(lambda x: x.ffill())
    def bfill(self): return self._per_group(lambda x: x.bfill())
    def rank(self, method="average", ascending=True, pct=False): return self._per_group(lambda x: x.rank(method, ascending, pct))
    def fillna(self, value=None, method=None): return self._per_group(lambda x: x.fillna(value, method=method))

    def rolling(self, window, min_periods=None):
        from ._window import GroupRolling
        return GroupRolling(self, window, min_periods)

    def value_counts(self, normalize=False, sort=True, ascending=False, dropna=True):
        s = self._obj
        labels, vals = [], []
        for k, p in zip(self._g.order, self._g.positions()):
            vc = s._take(p).value_counts(normalize=normalize, sort=sort, ascending=ascending, dropna=dropna)
            key = self._g.keys[k]
            for lab, v in zip(vc.index._values, vc._values):
                labels.append((key + (lab,)) if isinstance(key, tuple) else (key, lab))
                vals.append(v)
        return Series._new(vals, Index._new(labels, self._g.names + [s.name]), "proportion" if normalize else "count")

    def unique(self):
        s = self._obj
        return Series._new([s._take(p).unique() for p in self._g.positions()], self._g.result_index(), s.name, DT_OBJ)

    def describe(self):
        from ._frame import DataFrame, _describe_column
        rows = []
        labels = None
        for p in self._g.positions():
            sub = self._obj._take(p)
            labels, vals = _describe_column(sub._values, sub._dtype)
            rows.append(vals)
        return DataFrame._new({l: [r[j] for r in rows] for j, l in enumerate(labels or [])}, {}, self._g.result_index())

    def nlargest(self, n=5):
        return self.apply(lambda x: x.nlargest(n))

    def nsmallest(self, n=5):
        return self.apply(lambda x: x.nsmallest(n))

    @property
    def plot(self):
        from ._plot import PlotAccessor
        return PlotAccessor(self._obj)


class DataFrameGroupBy(_GroupByBase):
    def __init__(self, df, by=None, level=None, as_index=True, sort=True, dropna=True, grouper=None, selection=None, series_sel=False):
        self._obj = df
        self._g = grouper if grouper is not None else _Grouper(df, by, level, sort, dropna)
        self._as_index = as_index
        self._selection = selection

    def _cols(self):
        if self._selection is not None:
            return list(self._selection)
        keys = set(self._g.key_columns)
        return [c for c in self._obj._data if c not in keys]

    def __getitem__(self, key):
        if isinstance(key, (list, tuple)):
            return DataFrameGroupBy(self._obj, as_index=self._as_index, grouper=self._g, selection=list(key))
        if key not in self._obj._data:
            raise KeyError("Column not found: %s" % (key,))
        sg = SeriesGroupBy(self._obj._col(key), grouper=self._g)
        sg._as_index = self._as_index
        if not self._as_index:
            return _AsFrame(sg, self)
        return sg

    def __getattr__(self, name):
        if name.startswith("_"):
            raise AttributeError(name)
        if name in self._obj._data:
            return self[name]
        raise AttributeError("'DataFrameGroupBy' object has no attribute '%s'" % name)

    def _finish(self, df):
        if not self._as_index:
            out = df.reset_index()
            return out
        return df

    def _agg_all(self, how, numeric_only=False, **kw):
        from ._frame import DataFrame
        df = self._obj
        data = {}
        dts = {}
        positions = self._g.positions()
        for c in self._cols():
            dt = df._dtypes[c]
            if (numeric_only or (isinstance(how, str) and how in _agg.NUMERIC_ONLY)) and not is_numeric(dt) and dt != "datetime64[ns]":
                if isinstance(how, str) and how in _agg.NUMERIC_ONLY and not numeric_only and any(isinstance(v, str) for v in df._data[c]):
                    continue
                if numeric_only:
                    continue
            vals = df._data[c]
            out = [_apply_how([vals[i] for i in p], how, dt, kw) for p in positions]
            rdt = _agg.result_dtype(how, dt) if isinstance(how, str) else None
            if rdt is None and isinstance(how, str) and how in ("min", "max", "first", "last", "sum") and dt in ("int64", "datetime64[ns]") and not any(isna_scalar(v) for v in out):
                rdt = dt
            data[c] = out
            dts[c] = rdt
        res = DataFrame._new(data, dts, self._g.result_index(), list(df._columns_names))
        return self._finish(res)

    def sum(self, numeric_only=False, min_count=0): return self._agg_all("sum", numeric_only, min_count=min_count)
    def mean(self, numeric_only=False): return self._agg_all("mean", numeric_only)
    def median(self, numeric_only=False): return self._agg_all("median", numeric_only)
    def min(self, numeric_only=False): return self._agg_all("min", numeric_only)
    def max(self, numeric_only=False): return self._agg_all("max", numeric_only)
    def std(self, ddof=1, numeric_only=False): return self._agg_all("std", numeric_only, ddof=ddof)
    def var(self, ddof=1, numeric_only=False): return self._agg_all("var", numeric_only, ddof=ddof)
    def sem(self, ddof=1, numeric_only=False): return self._agg_all("sem", numeric_only, ddof=ddof)
    def prod(self, numeric_only=False): return self._agg_all("prod", numeric_only)
    def count(self): return self._agg_all("count")
    def nunique(self, dropna=True): return self._agg_all("nunique")
    def first(self, numeric_only=False): return self._agg_all("first", numeric_only)
    def last(self, numeric_only=False): return self._agg_all("last", numeric_only)
    def any(self): return self._agg_all("any")
    def all(self): return self._agg_all("all")

    def quantile(self, q=0.5, numeric_only=False):
        return self._agg_all("quantile", numeric_only, q=q)

    def idxmax(self, numeric_only=False):
        from ._frame import DataFrame
        data = {c: [self._obj._col(c)._take(p).idxmax() for p in self._g.positions()] for c in self._cols() if is_numeric(self._obj._dtypes[c])}
        return DataFrame._new(data, {}, self._g.result_index())

    def idxmin(self, numeric_only=False):
        from ._frame import DataFrame
        data = {c: [self._obj._col(c)._take(p).idxmin() for p in self._g.positions()] for c in self._cols() if is_numeric(self._obj._dtypes[c])}
        return DataFrame._new(data, {}, self._g.result_index())

    def agg(self, func=None, *args, **kw):
        from ._frame import DataFrame
        df = self._obj
        if func is None and kw:
            data, dts = {}, {}
            for name, spec in kw.items():
                if isinstance(spec, tuple):
                    col, f = spec
                else:
                    col, f = spec.column, spec.aggfunc
                s = SeriesGroupBy(df._col(col), grouper=self._g).agg(f)
                data[name], dts[name] = s._values, s._dtype
            return self._finish(DataFrame._new(data, dts, self._g.result_index()))
        if isinstance(func, dict):
            data, dts = {}, {}
            multi = any(isinstance(f, (list, tuple)) for f in func.values())
            for col, fs in func.items():
                sg = SeriesGroupBy(df._col(col), grouper=self._g)
                fl = fs if isinstance(fs, (list, tuple)) else [fs]
                for f in fl:
                    s = sg.agg(f)
                    key = (col, _agg.func_name(f)) if multi else col
                    data[key], dts[key] = s._values, s._dtype
            out = DataFrame._new(data, dts, self._g.result_index())
            if multi:
                out._columns_names = [None, None]
            return self._finish(out)
        if isinstance(func, (list, tuple)):
            data, dts = {}, {}
            for c in self._cols():
                sg = SeriesGroupBy(df._col(c), grouper=self._g)
                for f in func:
                    try:
                        s = sg.agg(f)
                    except TypeError:
                        continue
                    key = (c, _agg.func_name(f))
                    data[key], dts[key] = s._values, s._dtype
            out = DataFrame._new(data, dts, self._g.result_index())
            out._columns_names = [None, None]
            return self._finish(out)
        f = _agg.resolve(func)
        if isinstance(f, str):
            return getattr(self, f)(*args, **kw)
        data, dts = {}, {}
        for c in self._cols():
            s = SeriesGroupBy(df._col(c), grouper=self._g).agg(f, *args, **kw)
            data[c], dts[c] = s._values, s._dtype
        return self._finish(DataFrame._new(data, dts, self._g.result_index()))

    aggregate = agg

    def apply(self, func, *args, include_groups=True, **kw):
        from ._frame import DataFrame
        from ._reshape import concat
        df = self._obj
        sub_cols = None if include_groups else self._cols()
        res = []
        for p in self._g.positions():
            sub = df._take(p)
            if sub_cols is not None:
                sub = sub[sub_cols]
            res.append(func(sub, *args, **kw))
        idx = self._g.result_index()
        if not res:
            return DataFrame()
        if all(isinstance(r, DataFrame) for r in res):
            keys = [self._g.keys[k] for k in self._g.order]
            labels = []
            for key, r in zip(keys, res):
                for lab in r.index._values:
                    labels.append((key + (lab if isinstance(lab, tuple) else (lab,))) if isinstance(key, tuple) else (key,) + (lab if isinstance(lab, tuple) else (lab,)))
            out = concat(res)
            names = self._g.names + list(res[0].index._names)
            object.__setattr__(out, "_index", Index._new(labels, names))
            return out
        if all(isinstance(r, Series) for r in res):
            keys = []
            seen = set()
            for r in res:
                for k in r.index._values:
                    if k not in seen:
                        seen.add(k)
                        keys.append(k)
            out = DataFrame._new({k: [r.get(k, NaN) for r in res] for k in keys}, {}, idx)
            return self._finish(out)
        s = Series._new(res, idx, None)
        if not self._as_index:
            return s.to_frame(None).reset_index() if False else s.reset_index()
        return s

    def transform(self, func, *args, **kw):
        from ._frame import DataFrame
        df = self._obj
        data = {}
        for c in self._cols():
            if isinstance(func, str) and func in _agg.NUMERIC_ONLY and not is_numeric(df._dtypes[c]):
                continue
            data[c] = SeriesGroupBy(df._col(c), grouper=self._g).transform(func, *args, **kw)._values
        return DataFrame._new(data, {}, df.index)

    def _per_group_all(self, name, *args):
        from ._frame import DataFrame
        df = self._obj
        data = {}
        for c in self._cols():
            if not is_numeric(df._dtypes[c]) and name not in ("shift", "ffill", "bfill", "fillna"):
                continue
            data[c] = getattr(SeriesGroupBy(df._col(c), grouper=self._g), name)(*args)._values
        return DataFrame._new(data, {}, df.index)

    def cumsum(self): return self._per_group_all("cumsum")
    def cumprod(self): return self._per_group_all("cumprod")
    def cummax(self): return self._per_group_all("cummax")
    def cummin(self): return self._per_group_all("cummin")
    def shift(self, periods=1, fill_value=None): return self._per_group_all("shift", periods, fill_value)
    def diff(self, periods=1): return self._per_group_all("diff", periods)
    def pct_change(self, periods=1): return self._per_group_all("pct_change", periods)
    def ffill(self): return self._per_group_all("ffill")
    def bfill(self): return self._per_group_all("bfill")
    def rank(self, method="average", ascending=True, pct=False): return self._per_group_all("rank", method, ascending, pct)

    def fillna(self, value=None, method=None):
        return self._per_group_all("fillna", value, method)

    def describe(self):
        from ._frame import DataFrame
        cols = [c for c in self._cols() if is_numeric(self._obj._dtypes[c])]
        data = {}
        for c in cols:
            d = SeriesGroupBy(self._obj._col(c), grouper=self._g).describe()
            for k in d._data:
                data[(c, k)] = d._data[k]
        out = DataFrame._new(data, {}, self._g.result_index())
        out._columns_names = [None, None]
        return out

    def value_counts(self, normalize=False, sort=True, ascending=False):
        df = self._obj
        cols = self._cols()
        labels, vals = [], []
        for k, p in zip(self._g.order, self._g.positions()):
            vc = df._take(p)[cols].value_counts(normalize=normalize, sort=sort, ascending=ascending)
            key = self._g.keys[k]
            for lab, v in zip(vc.index._values, vc._values):
                kk = key if isinstance(key, tuple) else (key,)
                ll = lab if isinstance(lab, tuple) else (lab,)
                labels.append(kk + ll)
                vals.append(v)
        return Series._new(vals, Index._new(labels, self._g.names + cols), "proportion" if normalize else "count")

    def rolling(self, window, min_periods=None):
        from ._window import GroupRolling
        return GroupRolling(self, window, min_periods)

    def resample(self, rule, on=None, **kw):
        from ._window import resample_frame
        return _GroupResample(self, rule, on)

    @property
    def plot(self):
        from ._plot import PlotAccessor
        return PlotAccessor(self._obj)


class _AsFrame:
    """df.groupby(..., as_index=False)["col"]: aggregations come back as a frame."""

    def __init__(self, sg, parent):
        self._sg = sg
        self._parent = parent

    def __getattr__(self, name):
        attr = getattr(self._sg, name)
        if not callable(attr):
            return attr

        def call(*a, **k):
            r = attr(*a, **k)
            if isinstance(r, Series) and r.index.equals(self._sg._g.result_index()):
                return r.reset_index()
            from ._frame import DataFrame
            if isinstance(r, DataFrame) and r.index.equals(self._sg._g.result_index()):
                return r.reset_index()
            return r
        return call


class _GroupResample:
    def __init__(self, gb, rule, on):
        self._gb = gb
        self._rule = rule
        self._on = on

    def __getattr__(self, how):
        def call(*a, **k):
            from ._reshape import concat
            from ._window import resample_frame
            parts = []
            labels = []
            for key, sub in self._gb:
                r = getattr(resample_frame(sub, self._rule, self._on), how)(*a, **k)
                for c in self._gb._g.key_columns:
                    if c in r._data:
                        del r[c]
                parts.append(r)
                for lab in r.index._values:
                    labels.append((key if isinstance(key, tuple) else (key,)) + (lab,))
            out = concat(parts)
            object.__setattr__(out, "_index", Index._new(labels, self._gb._g.names + [self._on or parts[0].index.name if parts else None]))
            return out
        return call
