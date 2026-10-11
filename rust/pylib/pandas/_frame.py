# SPDX-License-Identifier: MIT
# DataFrame for rpy's pandas.

import math
import re

from ._util import (NaN, NaT, isna_scalar, infer_dtype, normalize, convert,
                    dtype, is_numeric, DT_OBJ, DT_INT, DT_FLOAT, DT_BOOL, DT_DATE)
from ._index import Index, RangeIndex, default_index, ensure_index, _hkey, _sort_key
from ._series import (Series, _sorted_positions, _mask_positions, _is_listlike,
                      _label_slice, _cond_list, _na_for, align_series, _truediv)
from . import _agg


def _col_values(v, index, n):
    """A column's values from what was assigned (scalar, list or Series)."""
    if isinstance(v, Series):
        if index is None or v.index.equals(index):
            return list(v._values), v._dtype
        pos = v.index.get_indexer(index._values)
        vals = [v._values[p] if p >= 0 else _na_for(v._dtype) for p in pos]
        return vals, None
    if isinstance(v, Index):
        return list(v._values), None
    if isinstance(v, (list, tuple, range)) or (hasattr(v, "__iter__") and not isinstance(v, (str, bytes, dict))):
        vals = list(v)
        if n is not None and len(vals) != n:
            raise ValueError("Length of values (%d) does not match length of index (%d)" % (len(vals), n))
        return vals, None
    return [v] * (n or 0), None


class DataFrame:
    def __init__(self, data=None, index=None, columns=None, dtype=None, copy=None):
        d = {}
        dts = {}
        idx = None
        if isinstance(data, DataFrame):
            d = {c: list(v) for c, v in data._data.items()}
            dts = dict(data._dtypes)
            idx = data._index
            cnames = data._columns_names
        else:
            cnames = [None]
        if isinstance(data, dict):
            series_idx = None
            for v in data.values():
                if isinstance(v, Series):
                    series_idx = v.index if series_idx is None else (series_idx if series_idx.equals(v.index) else series_idx.union(v.index))
            if index is not None:
                idx = ensure_index(index)
            elif series_idx is not None:
                idx = series_idx
            n = len(idx) if idx is not None else None
            if n is None:
                for v in data.values():
                    if not isinstance(v, (str, bytes)) and _is_listlike(v) or isinstance(v, dict):
                        n = len(v)
                        break
                if n is None:
                    if index is None and data:
                        raise ValueError("If using all scalar values, you must pass an index")
                    n = 0
            for k, v in data.items():
                if isinstance(v, dict):
                    if idx is None:
                        idx = Index(list(v.keys()))
                        n = len(idx)
                    vals = [v.get(lab, NaN) for lab in idx._values]
                    dt0 = None
                else:
                    vals, dt0 = _col_values(v, idx, n)
                d[k] = vals
                dts[k] = dt0
            if idx is None:
                idx = default_index(n)
        elif isinstance(data, (list, tuple)) or (data is not None and hasattr(data, "__iter__") and not isinstance(data, (DataFrame, str))):
            rows = list(data)
            if rows and isinstance(rows[0], dict):
                keys = []
                seen = set()
                for r in rows:
                    for k in r:
                        if k not in seen:
                            seen.add(k)
                            keys.append(k)
                for k in keys:
                    d[k] = [r.get(k, NaN) for r in rows]
                    dts[k] = None
            elif rows and isinstance(rows[0], Series):
                keys = []
                seen = set()
                for r in rows:
                    for k in r.index._values:
                        if k not in seen:
                            seen.add(k)
                            keys.append(k)
                for k in keys:
                    d[k] = [r.get(k, NaN) for r in rows]
                    dts[k] = None
                if index is None:
                    index = [r.name for r in rows] if all(r.name is not None for r in rows) else None
            elif rows:
                width = max(len(r) if isinstance(r, (list, tuple)) else 1 for r in rows)
                cols = list(columns) if columns is not None else list(range(width))
                for j, c in enumerate(cols):
                    d[c] = [(r[j] if j < len(r) else NaN) if isinstance(r, (list, tuple)) else (r if j == 0 else NaN) for r in rows]
                    dts[c] = None
                columns = None
            else:
                if columns is not None:
                    for c in columns:
                        d[c] = []
                        dts[c] = DT_OBJ
                    columns = None
            n = len(rows)
            idx = ensure_index(index, n) if index is not None else default_index(n)
        elif data is None:
            n = len(index) if index is not None else 0
            idx = ensure_index(index, n) if index is not None else default_index(0)
            if columns is not None:
                for c in columns:
                    d[c] = [NaN] * n
                    dts[c] = DT_OBJ if n == 0 else DT_FLOAT
                columns = None
        elif isinstance(data, Series):
            nm = data.name if data.name is not None else 0
            d[nm] = list(data._values)
            dts[nm] = data._dtype
            idx = data.index
        elif idx is None:
            raise ValueError("DataFrame constructor not properly called!")
        if isinstance(data, DataFrame) and index is not None:
            idx = ensure_index(index)
        if columns is not None:
            columns = list(columns)
            nd, ndt = {}, {}
            for c in columns:
                if c in d:
                    nd[c], ndt[c] = d[c], dts[c]
                else:
                    nd[c], ndt[c] = [NaN] * len(idx), None
            d, dts = nd, ndt
        for k in list(d.keys()):
            if dtype is not None:
                d[k], dts[k] = convert(d[k], dtype)
            elif dts.get(k) is None:
                dt0 = infer_dtype(d[k])
                d[k] = normalize(d[k], dt0)
                dts[k] = dt0
        object.__setattr__(self, "_data", d)
        object.__setattr__(self, "_dtypes", dts)
        object.__setattr__(self, "_index", idx)
        object.__setattr__(self, "_columns_names", cnames)

    @classmethod
    def _new(cls, data, dtypes, index, columns_names=None):
        self = object.__new__(DataFrame)
        for k in data:
            if dtypes.get(k) is None:
                dt0 = infer_dtype(data[k])
                data[k] = normalize(data[k], dt0)
                dtypes[k] = dt0
        object.__setattr__(self, "_data", data)
        object.__setattr__(self, "_dtypes", dtypes)
        object.__setattr__(self, "_index", index)
        object.__setattr__(self, "_columns_names", columns_names or [None])
        return self

    def _with(self, data, dtypes=None, index=None):
        if dtypes is None:
            dtypes = {k: self._dtypes.get(k) for k in data}
        return DataFrame._new(data, dtypes, self._index if index is None else index, list(self._columns_names))

    # ------------------------------------------------------------ basics
    @property
    def index(self):
        return self._index

    @index.setter
    def index(self, v):
        v = ensure_index(v, len(self._index))
        if len(v) != len(self._index):
            raise ValueError("Length mismatch: Expected axis has %d elements, new values have %d elements" % (len(self._index), len(v)))
        object.__setattr__(self, "_index", v)

    @property
    def columns(self):
        cols = list(self._data.keys())
        if cols and all(isinstance(c, tuple) for c in cols) and len(set(len(c) for c in cols)) == 1 and len(cols[0]) > 1:
            names = self._columns_names if len(self._columns_names) == len(cols[0]) else [None] * len(cols[0])
            return Index._new(cols, names)
        return Index._new(cols, [self._columns_names[0] if len(self._columns_names) == 1 else None])

    @columns.setter
    def columns(self, new):
        new = list(new)
        if len(new) != len(self._data):
            raise ValueError("Length mismatch: Expected axis has %d elements, new values have %d elements" % (len(self._data), len(new)))
        old = list(self._data.keys())
        d = {}
        dts = {}
        for o, n in zip(old, new):
            d[n] = self._data[o]
            dts[n] = self._dtypes[o]
        object.__setattr__(self, "_data", d)
        object.__setattr__(self, "_dtypes", dts)

    @property
    def dtypes(self):
        cols = list(self._data.keys())
        return Series._new([self._dtypes[c] for c in cols], self.columns, None, DT_OBJ)

    @property
    def shape(self):
        return (len(self._index), len(self._data))

    @property
    def size(self):
        return len(self._index) * len(self._data)

    @property
    def ndim(self):
        return 2

    @property
    def empty(self):
        return len(self._index) == 0 or not self._data

    @property
    def axes(self):
        return [self._index, self.columns]

    @property
    def values(self):
        cols = list(self._data.values())
        return [list(r) for r in zip(*cols)] if cols else [[] for _ in range(len(self._index))]

    def to_numpy(self, dtype=None, na_value=None):
        return self.values

    def __len__(self):
        return len(self._index)

    def __iter__(self):
        return iter(list(self._data.keys()))

    def __contains__(self, key):
        return key in self._data

    def __bool__(self):
        raise ValueError("The truth value of a DataFrame is ambiguous. Use a.empty, a.bool(), a.item(), a.any() or a.all().")

    def keys(self):
        return self.columns

    def items(self):
        for c in list(self._data.keys()):
            yield c, self._col(c)

    iteritems = items

    def iterrows(self):
        cols = list(self._data.keys())
        cidx = self.columns
        dt = _common_dtype([self._dtypes[c] for c in cols])
        for i, lab in enumerate(self._index._values):
            vals = [self._data[c][i] for c in cols]
            yield lab, Series._new(normalize(vals, dt) if dt == "float64" else vals, cidx, lab, dt)

    def itertuples(self, index=True, name="Pandas"):
        from collections import namedtuple
        cols = list(self._data.keys())
        fields = (["Index"] if index else []) + [c if isinstance(c, str) and c.isidentifier() and not c.startswith("_") else "_%d" % (j + (1 if index else 0)) for j, c in enumerate(cols)]
        T = namedtuple(name, fields, rename=True) if name else None
        for i, lab in enumerate(self._index._values):
            vals = ([lab] if index else []) + [self._data[c][i] for c in cols]
            yield T(*vals) if T else tuple(vals)

    def copy(self, deep=True):
        return self._with({c: list(v) for c, v in self._data.items()}, dict(self._dtypes))

    def __repr__(self):
        from ._fmt import frame_to_string
        return frame_to_string(self)

    __str__ = __repr__

    def to_string(self, index=True, max_rows=None, **kw):
        from ._fmt import frame_to_string
        if not index:
            df = self.reset_index(drop=True)
            s = frame_to_string(df, max_rows=max_rows if max_rows is not None else len(df) + 1)
            lines = s.split("\n")
            w = max(len(str(len(df) - 1)), 1) + 1
            return "\n".join(l[w:] for l in lines)
        return frame_to_string(self, max_rows=max_rows if max_rows is not None else len(self) + 1)

    def to_markdown(self, buf=None, index=True, **kw):
        from ._fmt import to_markdown
        s = to_markdown(self, index=index)
        if buf is not None:
            _write_text(buf, s)
            return None
        return s

    def to_html(self, buf=None, index=True, **kw):
        from ._io import to_html
        s = to_html(self, index=index, **kw)
        if buf is not None:
            _write_text(buf, s)
            return None
        return s

    def _repr_html_(self):
        return self.to_html()

    def info(self, verbose=True, buf=None, **kw):
        lines = ["<class 'pandas.DataFrame'>"]
        idx = self._index
        n = len(idx)
        if isinstance(idx, RangeIndex):
            lines.append("RangeIndex: %d entries, %d to %d" % (n, idx._start, idx._stop - 1) if n else "RangeIndex: 0 entries")
        else:
            lines.append("Index: %d entries, %s to %s" % (n, idx._values[0], idx._values[-1]) if n else "Index: 0 entries")
        cols = list(self._data.keys())
        lines.append("Data columns (total %d columns):" % len(cols))
        nums = [" %d " % j for j in range(len(cols))]
        names = [str(c) for c in cols]
        counts = ["%d non-null" % sum(1 for v in self._data[c] if not isna_scalar(v)) for c in cols]
        types = [str(self._dtypes[c]) for c in cols]
        heads = [" # ", "Column", "Non-Null Count", "Dtype"]
        table = [nums, names, counts, types]
        widths = [max([len(h)] + [len(x) for x in col]) for h, col in zip(heads, table)]
        lines.append("  ".join(h.ljust(w) for h, w in zip(heads, widths)))
        lines.append("  ".join(("-" * len(h)).ljust(w) for h, w in zip(heads, widths)))
        for j in range(len(cols)):
            lines.append("  ".join(col[j].ljust(w) for col, w in zip(table, widths)))
        tc = {}
        for t in types:
            tc[t] = tc.get(t, 0) + 1
        lines.append("dtypes: " + ", ".join("%s(%d)" % (k, v) for k, v in sorted(tc.items())))
        mem = sum(n if self._dtypes[c] == "bool" else 8 * n for c in cols) + (132 if isinstance(idx, RangeIndex) else 8 * n)
        lines.append("memory usage: %s" % _mem(mem))
        text = "\n".join(lines)
        if buf is not None:
            buf.write(text + "\n")
        else:
            print(text)

    # ------------------------------------------------------------ columns
    def _col(self, c):
        return Series._new(self._data[c], self._index, c, self._dtypes[c])

    def __getattr__(self, name):
        if name.startswith("_"):
            raise AttributeError(name)
        d = self.__dict__.get("_data")
        if d is not None and name in d:
            return self._col(name)
        raise AttributeError("'DataFrame' object has no attribute '%s'" % name)

    def __setattr__(self, name, value):
        if name in ("index", "columns"):
            object.__setattr__(self, name, value)
        elif not name.startswith("_") and name in self._data:
            self[name] = value
        else:
            object.__setattr__(self, name, value)

    def __getitem__(self, key):
        if isinstance(key, Series) and key._dtype == "bool":
            return self._take(_mask_positions(key, self._index))
        if isinstance(key, slice):
            if (key.start is None or isinstance(key.start, int)) and (key.stop is None or isinstance(key.stop, int)):
                return self._take(list(range(len(self._index)))[key])
            return self._take(_label_slice(self._index, key))
        if isinstance(key, DataFrame):
            return self.where(key)
        if isinstance(key, (list, Index, Series)) or (isinstance(key, tuple) and key not in self._data):
            keys = list(key)
            if keys and isinstance(keys[0], bool):
                return self._take([i for i, b in enumerate(keys) if b])
            missing = [k for k in keys if k not in self._data]
            if missing:
                raise KeyError("%s not in index" % (missing,))
            return self._with({k: self._data[k] for k in keys})
        if callable(key):
            return self[key(self)]
        if key not in self._data:
            subs = [c for c in self._data if isinstance(c, tuple) and c[0] == key]
            if subs:
                out = self._with({c[1:] if len(c) > 2 else c[1]: self._data[c] for c in subs}, {c[1:] if len(c) > 2 else c[1]: self._dtypes[c] for c in subs})
                out._columns_names = self._columns_names[1:] or [None]
                return out
            raise KeyError(key)
        return self._col(key)

    def __setitem__(self, key, value):
        if isinstance(key, (list, Index)) and not (key and isinstance(key[0], bool)):
            if isinstance(value, DataFrame):
                for k, c in zip(key, list(value._data.keys())):
                    self[k] = value._col(c)
            else:
                for k in key:
                    self[k] = value
            return
        if isinstance(key, Series) and key._dtype == "bool":
            pos = _mask_positions(key, self._index)
            for c in self._data:
                vals = list(self._data[c])
                for p in pos:
                    vals[p] = value
                self._set_col(c, vals, None)
            return
        if isinstance(value, DataFrame):
            if len(value._data) == 1:
                value = value._col(list(value._data.keys())[0])
            else:
                raise ValueError("Cannot set a DataFrame with multiple columns to the single column %s" % (key,))
        n = len(self._index)
        if not self._data and len(self._index) == 0 and _is_listlike(value):
            if isinstance(value, Series):
                object.__setattr__(self, "_index", value.index)
            else:
                object.__setattr__(self, "_index", default_index(len(value)))
            n = len(self._index)
        vals, dt = _col_values(value, self._index, n)
        self._set_col(key, vals, dt)

    def _set_col(self, key, vals, dt):
        if dt is None:
            dt = infer_dtype(vals)
            vals = normalize(vals, dt)
        self._data[key] = vals
        self._dtypes[key] = dt

    def __delitem__(self, key):
        del self._data[key]
        del self._dtypes[key]

    def pop(self, key):
        s = self._col(key)
        del self[key]
        return s

    def get(self, key, default=None):
        try:
            return self[key]
        except KeyError:
            return default

    def insert(self, loc, column, value, allow_duplicates=False):
        vals, dt = _col_values(value, self._index, len(self._index))
        items = list(self._data.items())
        items.insert(loc, (column, vals))
        dts = dict(self._dtypes)
        dts[column] = dt
        object.__setattr__(self, "_data", dict(items))
        object.__setattr__(self, "_dtypes", {k: dts.get(k) for k in self._data})
        if dt is None:
            self._set_col(column, vals, None)

    def assign(self, **kw):
        out = self.copy()
        for k, v in kw.items():
            if callable(v) and not isinstance(v, Series):
                v = v(out)
            out[k] = v
        return out

    def _take(self, positions):
        data = {c: [v[i] for i in positions] for c, v in self._data.items()}
        return self._with(data, dict(self._dtypes), self._index.take(positions))

    @property
    def loc(self):
        return _FrameLoc(self)

    @property
    def iloc(self):
        return _FrameILoc(self)

    @property
    def at(self):
        return _FrameLoc(self)

    @property
    def iat(self):
        return _FrameILoc(self)

    def head(self, n=5):
        if n < 0:
            n = max(len(self) + n, 0)
        return self._take(list(range(min(n, len(self)))))

    def tail(self, n=5):
        N = len(self)
        if n < 0:
            n = max(N + n, 0)
        return self._take(list(range(max(N - n, 0), N)))

    def sample(self, n=None, frac=None, replace=False, random_state=None, **kw):
        return self._take(_sample_positions(len(self), n, frac, replace, random_state))

    def select_dtypes(self, include=None, exclude=None):
        def match(dt, spec):
            if spec is None:
                return False
            specs = spec if isinstance(spec, (list, tuple)) else [spec]
            for s in specs:
                if s in ("number", "numeric") or s is float or s is int:
                    if s is float and dt == "float64":
                        return True
                    if s is int and dt == "int64":
                        return True
                    if s in ("number", "numeric") and dt in ("int64", "float64"):
                        return True
                elif s in ("object", object, str, "string", "category", "O"):
                    if dt == "object":
                        return True
                elif s in ("datetime", "datetime64", "datetime64[ns]"):
                    if dt == "datetime64[ns]":
                        return True
                elif s in ("bool", bool):
                    if dt == "bool":
                        return True
                else:
                    try:
                        if dt == s:
                            return True
                    except Exception:
                        pass
            return False
        keep = []
        for c, dt in self._dtypes.items():
            if include is not None and not match(dt, include):
                continue
            if exclude is not None and match(dt, exclude):
                continue
            keep.append(c)
        return self[keep]

    def filter(self, items=None, like=None, regex=None, axis=1):
        if axis in (0, "index"):
            labels = self._index._values
            if items is not None:
                pos = [i for i, l in enumerate(labels) if l in items]
            elif like is not None:
                pos = [i for i, l in enumerate(labels) if like in str(l)]
            else:
                rx = re.compile(regex)
                pos = [i for i, l in enumerate(labels) if rx.search(str(l))]
            return self._take(pos)
        cols = list(self._data.keys())
        if items is not None:
            keep = [c for c in items if c in self._data]
        elif like is not None:
            keep = [c for c in cols if like in str(c)]
        else:
            rx = re.compile(regex)
            keep = [c for c in cols if rx.search(str(c))]
        return self[keep]

    def drop(self, labels=None, axis=0, index=None, columns=None, errors="raise", inplace=False):
        if labels is not None:
            if axis in (1, "columns"):
                columns = labels
            else:
                index = labels
        out = self
        if columns is not None:
            if not isinstance(columns, (list, tuple, Index)):
                columns = [columns]
            for c in columns:
                if c not in self._data and errors == "raise":
                    raise KeyError("%s not found in axis" % ([c],))
            drop = set(columns)
            out = self._with({c: v for c, v in self._data.items() if c not in drop})
        if index is not None:
            if not isinstance(index, (list, tuple, Index)):
                index = [index]
            s = set(_hkey(l) for l in index)
            if errors == "raise":
                for l in index:
                    if l not in out._index:
                        raise KeyError("%s not found in axis" % ([l],))
            keep = [i for i, v in enumerate(out._index._values) if _hkey(v) not in s and not (out._index.nlevels > 1 and _hkey(v[0]) in s)]
            out = out._take(keep)
        if inplace:
            self._replace_with(out)
            return None
        return out

    def _replace_with(self, other):
        object.__setattr__(self, "_data", other._data)
        object.__setattr__(self, "_dtypes", other._dtypes)
        object.__setattr__(self, "_index", other._index)
        object.__setattr__(self, "_columns_names", other._columns_names)

    def rename(self, mapper=None, index=None, columns=None, axis=None, inplace=False, errors="ignore"):
        if mapper is not None:
            if axis in (1, "columns"):
                columns = mapper
            else:
                index = mapper
        out = self.copy() if not inplace else self

        def f(m):
            if callable(m) and not isinstance(m, dict):
                return m
            return lambda x: m.get(x, x)
        if columns is not None:
            g = f(columns)
            d, dts = {}, {}
            for c, v in out._data.items():
                n = g(c)
                d[n] = v
                dts[n] = out._dtypes[c]
            object.__setattr__(out, "_data", d)
            object.__setattr__(out, "_dtypes", dts)
        if index is not None:
            g = f(index)
            object.__setattr__(out, "_index", Index._new([g(v) for v in out._index._values], out._index._names))
        return None if inplace else out

    def rename_axis(self, mapper=None, index=None, columns=None, axis=0):
        out = self.copy()
        if mapper is not None:
            if axis in (1, "columns"):
                columns = mapper
            else:
                index = mapper
        if index is not None:
            idx = out._index.copy()
            idx._names = list(index) if isinstance(index, (list, tuple)) else [index]
            object.__setattr__(out, "_index", idx)
        if columns is not None:
            out._columns_names = [columns]
        return out

    def add_prefix(self, p):
        return self.rename(columns=lambda c: p + str(c))

    def add_suffix(self, s):
        return self.rename(columns=lambda c: str(c) + s)

    def set_axis(self, labels, axis=0, **kw):
        out = self.copy()
        if axis in (1, "columns"):
            out.columns = labels
        else:
            out.index = labels
        return out

    def reindex(self, index=None, columns=None, fill_value=None, labels=None, axis=None):
        if labels is not None:
            if axis in (1, "columns"):
                columns = labels
            else:
                index = labels
        out = self
        if index is not None:
            idx = ensure_index(index)
            pos = out._index.get_indexer(idx._values)
            data = {}
            for c, v in out._data.items():
                fill = _na_for(out._dtypes[c]) if fill_value is None else fill_value
                data[c] = [v[p] if p >= 0 else fill for p in pos]
            out = DataFrame._new(data, {}, idx, list(out._columns_names))
        if columns is not None:
            data = {}
            for c in columns:
                data[c] = list(out._data[c]) if c in out._data else [NaN if fill_value is None else fill_value] * len(out)
            out = DataFrame._new(data, {}, out._index, list(out._columns_names))
        return out

    def astype(self, dt, errors="raise", copy=True):
        out = self.copy()
        if isinstance(dt, dict):
            for c, t in dt.items():
                s = out._col(c).astype(t)
                out._data[c], out._dtypes[c] = s._values, s._dtype
        else:
            for c in list(out._data):
                s = out._col(c).astype(dt)
                out._data[c], out._dtypes[c] = s._values, s._dtype
        return out

    def infer_objects(self):
        return DataFrame._new({c: list(v) for c, v in self._data.items()}, {}, self._index, list(self._columns_names))

    def convert_dtypes(self):
        return self.infer_objects()

    # ------------------------------------------------------------ missing
    def isna(self):
        return self._with({c: [isna_scalar(x) for x in v] for c, v in self._data.items()}, {c: DT_BOOL for c in self._data})

    isnull = isna

    def notna(self):
        return self._with({c: [not isna_scalar(x) for x in v] for c, v in self._data.items()}, {c: DT_BOOL for c in self._data})

    notnull = notna

    def fillna(self, value=None, method=None, axis=None, inplace=False, limit=None):
        out = {}
        dts = {}
        for c in self._data:
            s = self._col(c)
            if isinstance(value, dict):
                if c in value:
                    s = s.fillna(value[c])
            elif isinstance(value, Series):
                if c in value.index:
                    s = s.fillna(value[c])
            else:
                s = s.fillna(value, method=method)
            out[c], dts[c] = s._values, s._dtype
        res = self._with(out, dts)
        if inplace:
            self._replace_with(res)
            return None
        return res

    def ffill(self, **kw):
        return self.fillna(method="ffill", **kw)

    def bfill(self, **kw):
        return self.fillna(method="bfill", **kw)

    def interpolate(self, method="linear", **kw):
        out = self.copy()
        for c in out._data:
            if is_numeric(out._dtypes[c]):
                s = out._col(c).interpolate(method)
                out._data[c], out._dtypes[c] = s._values, s._dtype
        return out

    def dropna(self, axis=0, how="any", thresh=None, subset=None, inplace=False, ignore_index=False):
        if axis in (1, "columns"):
            keep = []
            for c, v in self._data.items():
                nn = sum(1 for x in v if not isna_scalar(x))
                if thresh is not None:
                    ok = nn >= thresh
                elif how == "all":
                    ok = nn > 0
                else:
                    ok = nn == len(v)
                if ok:
                    keep.append(c)
            out = self[keep]
        else:
            cols = list(subset) if subset is not None else list(self._data.keys())
            if isinstance(subset, str):
                cols = [subset]
            vals = [self._data[c] for c in cols]
            keep = []
            for i in range(len(self._index)):
                nn = sum(1 for v in vals if not isna_scalar(v[i]))
                if thresh is not None:
                    ok = nn >= thresh
                elif how == "all":
                    ok = nn > 0
                else:
                    ok = nn == len(cols)
                if ok:
                    keep.append(i)
            out = self._take(keep)
            if ignore_index:
                object.__setattr__(out, "_index", default_index(len(keep)))
        if inplace:
            self._replace_with(out)
            return None
        return out

    def replace(self, to_replace=None, value=None, regex=False, inplace=False):
        out = {}
        dts = {}
        for c in self._data:
            if isinstance(to_replace, dict) and to_replace and all(k in self._data for k in to_replace) and isinstance(next(iter(to_replace.values())), dict):
                s = self._col(c).replace(to_replace[c], regex=regex) if c in to_replace else self._col(c)
            else:
                s = self._col(c).replace(to_replace, value, regex=regex)
            out[c], dts[c] = s._values, s._dtype
        res = self._with(out, dts)
        if inplace:
            self._replace_with(res)
            return None
        return res

    def where(self, cond, other=NaN):
        out = {}
        for c in self._data:
            cc = cond[c] if isinstance(cond, DataFrame) else cond
            o = other[c] if isinstance(other, DataFrame) else other
            out[c] = self._col(c).where(cc, o)._values
        return self._with(out, {c: None for c in out})

    def mask(self, cond, other=NaN):
        if isinstance(cond, DataFrame):
            cond = ~cond
        else:
            cond = [not x for x in _cond_list(cond, self._col(list(self._data)[0]))]
        return self.where(cond, other)

    def clip(self, lower=None, upper=None):
        return self._map_numeric(lambda s: s.clip(lower, upper))

    def _map_numeric(self, f):
        out, dts = {}, {}
        for c in self._data:
            s = self._col(c)
            if is_numeric(s._dtype):
                s = f(s)
            out[c], dts[c] = s._values, s._dtype
        return self._with(out, dts)

    def round(self, decimals=0):
        out, dts = {}, {}
        for c in self._data:
            s = self._col(c)
            d = decimals.get(c) if isinstance(decimals, dict) else decimals
            if d is not None:
                s = s.round(d)
            out[c], dts[c] = s._values, s._dtype
        return self._with(out, dts)

    def abs(self):
        return self._map_numeric(lambda s: s.abs())

    def cumsum(self, axis=0):
        return self._map_numeric(lambda s: s.cumsum())

    def cumprod(self, axis=0):
        return self._map_numeric(lambda s: s.cumprod())

    def cummax(self, axis=0):
        return self._map_numeric(lambda s: s.cummax())

    def cummin(self, axis=0):
        return self._map_numeric(lambda s: s.cummin())

    def diff(self, periods=1, axis=0):
        return self._map_numeric(lambda s: s.diff(periods))

    def pct_change(self, periods=1, **kw):
        return self._map_numeric(lambda s: s.pct_change(periods))

    def shift(self, periods=1, fill_value=None, axis=0):
        out = {c: self._col(c).shift(periods, fill_value)._values for c in self._data}
        return self._with(out, {c: None for c in out})

    def rank(self, method="average", ascending=True, pct=False, axis=0, numeric_only=False):
        out = {}
        for c in self._data:
            if numeric_only and not is_numeric(self._dtypes[c]):
                continue
            out[c] = self._col(c).rank(method=method, ascending=ascending, pct=pct)._values
        return self._with(out, {c: DT_FLOAT for c in out})

    # ------------------------------------------------------------ operators
    def _binop(self, other, op):
        out = {}
        if isinstance(other, DataFrame):
            cols = list(self._data.keys()) + [c for c in other._data if c not in self._data]
            for c in cols:
                a = self._col(c) if c in self._data else Series._new([NaN] * len(self), self._index)
                b = other._col(c) if c in other._data else Series._new([NaN] * len(other), other._index)
                r = op(a, b)
                out[c] = r
            idx = out[cols[0]].index if cols else self._index
            return DataFrame._new({c: s._values for c, s in out.items()}, {c: s._dtype for c, s in out.items()}, idx, list(self._columns_names))
        if isinstance(other, Series):
            for c in self._data:
                v = other[c] if c in other.index else NaN
                out[c] = op(self._col(c), v)
        else:
            for c in self._data:
                out[c] = op(self._col(c), other)
        return self._with({c: s._values for c, s in out.items()}, {c: s._dtype for c, s in out.items()})

    def __add__(self, o): return self._binop(o, lambda a, b: a + b)
    def __radd__(self, o): return self._binop(o, lambda a, b: b + a)
    def __sub__(self, o): return self._binop(o, lambda a, b: a - b)
    def __rsub__(self, o): return self._binop(o, lambda a, b: b - a)
    def __mul__(self, o): return self._binop(o, lambda a, b: a * b)
    def __rmul__(self, o): return self._binop(o, lambda a, b: b * a)
    def __truediv__(self, o): return self._binop(o, lambda a, b: a / b)
    def __rtruediv__(self, o): return self._binop(o, lambda a, b: b / a)
    def __floordiv__(self, o): return self._binop(o, lambda a, b: a // b)
    def __mod__(self, o): return self._binop(o, lambda a, b: a % b)
    def __pow__(self, o): return self._binop(o, lambda a, b: a ** b)
    def __eq__(self, o): return self._binop(o, lambda a, b: a == b)
    def __ne__(self, o): return self._binop(o, lambda a, b: a != b)
    def __lt__(self, o): return self._binop(o, lambda a, b: a < b)
    def __le__(self, o): return self._binop(o, lambda a, b: a <= b)
    def __gt__(self, o): return self._binop(o, lambda a, b: a > b)
    def __ge__(self, o): return self._binop(o, lambda a, b: a >= b)
    def __and__(self, o): return self._binop(o, lambda a, b: a & b)
    def __or__(self, o): return self._binop(o, lambda a, b: a | b)

    def __invert__(self):
        return self._with({c: [not x for x in v] for c, v in self._data.items()}, {c: DT_BOOL for c in self._data})

    def __neg__(self):
        return self._map_numeric(lambda s: -s)

    def __abs__(self):
        return self.abs()

    def __hash__(self):
        raise TypeError("unhashable type: 'DataFrame'")

    def _axis_op(self, other, op, axis):
        if isinstance(other, Series) and axis in (0, "index"):
            out = {c: op(self._col(c), other) for c in self._data}
            return self._with({c: s._values for c, s in out.items()}, {c: s._dtype for c, s in out.items()})
        return self._binop(other, op)

    def add(self, o, axis="columns", fill_value=None): return self._axis_op(o, lambda a, b: a.add(b, fill_value=fill_value) if isinstance(a, Series) else a + b, axis)
    def sub(self, o, axis="columns", fill_value=None): return self._axis_op(o, lambda a, b: a - b, axis)
    def mul(self, o, axis="columns", fill_value=None): return self._axis_op(o, lambda a, b: a * b, axis)
    def div(self, o, axis="columns", fill_value=None): return self._axis_op(o, lambda a, b: a / b, axis)
    truediv = divide = div
    multiply = mul
    subtract = sub

    def eq(self, o): return self.__eq__(o)
    def ne(self, o): return self.__ne__(o)
    def gt(self, o): return self.__gt__(o)
    def lt(self, o): return self.__lt__(o)
    def ge(self, o): return self.__ge__(o)
    def le(self, o): return self.__le__(o)

    # ------------------------------------------------------------ reductions
    def _reduce(self, how, axis=0, numeric_only=False, skipna=True, **kw):
        cols = list(self._data.keys())
        if numeric_only or how in _agg.NUMERIC_ONLY:
            if numeric_only:
                cols = [c for c in cols if is_numeric(self._dtypes[c])]
            else:
                for c in cols:
                    if self._dtypes[c] == "object" and any(isinstance(v, str) for v in self._data[c]):
                        raise TypeError("Could not convert %r to numeric for '%s' (use numeric_only=True)" % (c, how))
        if axis in (1, "columns"):
            vals = []
            dts = [self._dtypes[c] for c in cols]
            dt = _common_dtype(dts)
            for i in range(len(self._index)):
                row = [self._data[c][i] for c in cols]
                vals.append(_agg.reduce(row, how, dt, skipna=skipna, **kw))
            return Series._new(vals, self._index, None)
        vals = [_agg.reduce(self._data[c], how, self._dtypes[c], skipna=skipna, **kw) for c in cols]
        return Series._new(vals, self.columns.take([list(self._data.keys()).index(c) for c in cols]) if cols else Index([]), None)

    def sum(self, axis=0, skipna=True, numeric_only=False, min_count=0, **kw):
        return self._reduce("sum", axis, numeric_only, skipna, min_count=min_count)

    def mean(self, axis=0, skipna=True, numeric_only=False, **kw):
        return self._reduce("mean", axis, numeric_only, skipna)

    def median(self, axis=0, skipna=True, numeric_only=False, **kw):
        return self._reduce("median", axis, numeric_only, skipna)

    def min(self, axis=0, skipna=True, numeric_only=False, **kw):
        return self._reduce("min", axis, numeric_only, skipna)

    def max(self, axis=0, skipna=True, numeric_only=False, **kw):
        return self._reduce("max", axis, numeric_only, skipna)

    def std(self, axis=0, skipna=True, ddof=1, numeric_only=False, **kw):
        return self._reduce("std", axis, numeric_only, skipna, ddof=ddof)

    def var(self, axis=0, skipna=True, ddof=1, numeric_only=False, **kw):
        return self._reduce("var", axis, numeric_only, skipna, ddof=ddof)

    def sem(self, axis=0, skipna=True, ddof=1, numeric_only=False, **kw):
        return self._reduce("sem", axis, numeric_only, skipna, ddof=ddof)

    def prod(self, axis=0, skipna=True, numeric_only=False, **kw):
        return self._reduce("prod", axis, numeric_only, skipna)

    product = prod

    def count(self, axis=0, numeric_only=False):
        return self._reduce("count", axis, numeric_only)

    def nunique(self, axis=0, dropna=True):
        return self._reduce("nunique", axis)

    def any(self, axis=0, **kw):
        return self._reduce("any", axis)

    def all(self, axis=0, **kw):
        return self._reduce("all", axis)

    def quantile(self, q=0.5, axis=0, numeric_only=False, interpolation="linear"):
        cols = [c for c in self._data if is_numeric(self._dtypes[c])]
        if isinstance(q, (list, tuple)):
            data = {c: [_agg.quantile(_agg._clean(self._data[c]), x, interpolation) for x in q] for c in cols}
            return DataFrame._new(data, {c: DT_FLOAT for c in cols}, Index(list(q)))
        vals = [float(_agg.quantile(_agg._clean(self._data[c]), q, interpolation)) for c in cols]
        return Series._new(vals, Index(cols), q, DT_FLOAT)

    def idxmax(self, axis=0, skipna=True, numeric_only=False):
        cols = [c for c in self._data if is_numeric(self._dtypes[c])] if numeric_only else list(self._data)
        return Series._new([self._col(c).idxmax() for c in cols], Index(cols), None)

    def idxmin(self, axis=0, skipna=True, numeric_only=False):
        cols = [c for c in self._data if is_numeric(self._dtypes[c])] if numeric_only else list(self._data)
        return Series._new([self._col(c).idxmin() for c in cols], Index(cols), None)

    def mode(self, axis=0, numeric_only=False, dropna=True):
        res = {c: self._col(c).mode()._values for c in self._data}
        n = max((len(v) for v in res.values()), default=0)
        return DataFrame({c: v + [NaN] * (n - len(v)) for c, v in res.items()})

    def agg(self, func=None, axis=0, *args, **kw):
        if func is None and kw:
            data = {}
            for name, (col, f) in kw.items():
                data[name] = self._col(col).agg(f)
            return Series(data)
        if isinstance(func, dict):
            rows = []
            seen = set()
            per = {}
            for c, fs in func.items():
                fl = fs if isinstance(fs, (list, tuple)) else [fs]
                per[c] = {}
                for f in fl:
                    nm = _agg.func_name(f)
                    per[c][nm] = self._col(c).agg(f)
                    if nm not in seen:
                        seen.add(nm)
                        rows.append(nm)
            if all(not isinstance(fs, (list, tuple)) for fs in func.values()):
                return Series._new([list(per[c].values())[0] for c in func], Index(list(func.keys())), None)
            data = {c: [per[c].get(r, NaN) for r in rows] for c in func}
            return DataFrame(data, index=Index(rows))
        if isinstance(func, (list, tuple)):
            names = [_agg.func_name(f) for f in func]
            data = {}
            for c in self._data:
                s = self._col(c)
                vals = []
                ok = True
                for f in func:
                    try:
                        vals.append(s.agg(f))
                    except TypeError:
                        vals.append(NaN)
                data[c] = vals
            return DataFrame(data, index=Index(names))
        f = _agg.resolve(func)
        if isinstance(f, str):
            return getattr(self, f)(axis=axis)
        return self.apply(f, axis=axis)

    aggregate = agg

    def describe(self, percentiles=None, include=None, exclude=None):
        cols = list(self._data.keys())
        if include == "all":
            use = cols
        elif include is not None:
            use = list(self.select_dtypes(include=include)._data.keys())
        else:
            use = [c for c in cols if self._dtypes[c] in ("int64", "float64", "datetime64[ns]")]  # bool describes as categorical
            if not any(self._dtypes[c] != "datetime64[ns]" for c in use) and any(self._dtypes[c] in ("int64", "float64") for c in cols):
                use = [c for c in cols if self._dtypes[c] in ("int64", "float64")]
            if not use:
                use = [c for c in cols if self._dtypes[c] == "datetime64[ns]"] or [c for c in cols if self._dtypes[c] != "bool"] or cols
        results = {}
        order = []
        for c in use:
            labels, vals = _describe_column(self._data[c], self._dtypes[c], percentiles)
            results[c] = dict(zip(labels, vals))
            for l in labels:
                if l not in order:
                    order.append(l)
        kinds = set(str(self._dtypes[c]) for c in use)
        if "datetime64[ns]" in kinds and kinds - {"datetime64[ns]"} and include != "all" and not (kinds - {"datetime64[ns]", "int64", "float64"}):
            pct = [l for l in order if l.endswith("%")]
            order = ["count", "mean", "min"] + pct + ["max", "std"]
        elif include == "all" or any(self._dtypes[c] not in ("int64", "float64") for c in use):
            canonical = ["count", "unique", "top", "freq", "mean", "std", "min"]
            pct = [l for l in order if l.endswith("%")]
            canonical += pct + ["max"]
            order = [l for l in canonical if l in order]
        data = {c: [results[c].get(l, NaN) for l in order] for c in use}
        out = DataFrame(data, index=Index(order))
        for c in use:
            if self._dtypes[c] in ("int64", "float64") and all(not isinstance(v, str) for v in data[c]):
                out._data[c] = [float(v) if not isna_scalar(v) else NaN for v in data[c]]
                out._dtypes[c] = DT_FLOAT
        return out

    def value_counts(self, subset=None, normalize=False, sort=True, ascending=False, dropna=True):
        cols = list(subset) if subset is not None else list(self._data.keys())
        counts = {}
        for i in range(len(self._index)):
            k = tuple(self._data[c][i] for c in cols)
            if dropna and any(isna_scalar(x) for x in k):
                continue
            counts[k] = counts.get(k, 0) + 1
        keys = list(counts.keys())
        if sort:
            keys.sort(key=lambda k: counts[k], reverse=not ascending)
        vals = [counts[k] for k in keys]
        name = "count"
        if normalize:
            t = sum(vals)
            vals = [v / t for v in vals]
            name = "proportion"
        idx = Index._new(keys if len(cols) > 1 else [k[0] for k in keys], cols)
        return Series._new(vals, idx, name)

    def corr(self, method="pearson", numeric_only=False, min_periods=1):
        cols = [c for c in self._data if is_numeric(self._dtypes[c])]
        data = {}
        for c in cols:
            vals = []
            for d in cols:
                a, b = self._data[c], self._data[d]
                pairs = [(x, y) for x, y in zip(a, b) if not isna_scalar(x) and not isna_scalar(y)]
                xs = [float(p[0]) for p in pairs]
                ys = [float(p[1]) for p in pairs]
                vals.append(_spearman(xs, ys) if method == "spearman" else _pearson(xs, ys))
            data[c] = vals
        return DataFrame(data, index=Index(cols))

    def cov(self, ddof=1, numeric_only=False):
        cols = [c for c in self._data if is_numeric(self._dtypes[c])]
        return DataFrame({c: [self._col(c).cov(self._col(d), ddof) for d in cols] for c in cols}, index=Index(cols))

    # ------------------------------------------------------------ apply
    def apply(self, func, axis=0, raw=False, result_type=None, args=(), **kw):
        if isinstance(func, str):
            return getattr(self, func)(axis=axis, *args, **kw)
        if axis in (1, "columns"):
            cols = list(self._data.keys())
            cidx = self.columns
            dt = _common_dtype([self._dtypes[c] for c in cols])
            results = []
            for i, lab in enumerate(self._index._values):
                vals = [self._data[c][i] for c in cols]
                row = Series._new(normalize(vals, dt) if dt == "float64" else vals, cidx, lab, dt)
                results.append(func(row, *args, **kw))
            if results and all(isinstance(r, Series) for r in results):
                keys = []
                seen = set()
                for r in results:
                    for k in r.index._values:
                        if k not in seen:
                            seen.add(k)
                            keys.append(k)
                return DataFrame({k: [r.get(k, NaN) for r in results] for k in keys}, index=self._index)
            if result_type == "expand" and results and isinstance(results[0], (list, tuple)):
                width = len(results[0])
                return DataFrame({j: [r[j] for r in results] for j in range(width)}, index=self._index)
            return Series._new(results, self._index, None)
        res = {}
        for c in self._data:
            res[c] = func(self._col(c), *args, **kw)
        if res and all(isinstance(r, Series) for r in res.values()):
            first = next(iter(res.values()))
            return DataFrame({c: r for c, r in res.items()}, index=first.index)
        return Series._new(list(res.values()), self.columns, None)

    def map(self, func, na_action=None):
        out = {}
        for c, v in self._data.items():
            out[c] = [x if (na_action == "ignore" and isna_scalar(x)) else func(x) for x in v]
        return self._with(out, {c: None for c in out})

    applymap = map

    def pipe(self, func, *a, **kw):
        return func(self, *a, **kw)

    def transform(self, func, axis=0):
        out = {c: self._col(c).transform(func)._values for c in self._data}
        return self._with(out, {c: None for c in out})

    def isin(self, values):
        out = {}
        for c in self._data:
            vs = values[c] if isinstance(values, dict) else values
            out[c] = self._col(c).isin(vs if vs is not None else [])._values
        return self._with(out, {c: DT_BOOL for c in out})

    def eval(self, expr, inplace=False):
        from ._query import evaluate
        if "=" in expr and not re.search(r"[=!<>]=", expr.split("=")[0] + "=") and re.match(r"^\s*\w+\s*=[^=]", expr):
            name, rhs = expr.split("=", 1)
            out = self if inplace else self.copy()
            out[name.strip()] = evaluate(rhs, self)
            return None if inplace else out
        return evaluate(expr, self)

    def query(self, expr, inplace=False, **kw):
        from ._query import evaluate
        import sys
        frame = sys._getframe(1)
        mask = evaluate(expr, self, frame.f_locals, frame.f_globals)
        out = self[mask]
        if inplace:
            self._replace_with(out)
            return None
        return out

    # ------------------------------------------------------------ order
    def sort_values(self, by, axis=0, ascending=True, inplace=False, na_position="last", ignore_index=False, key=None, kind=None):
        if not isinstance(by, (list, tuple)):
            by = [by]
        keys = []
        for b in by:
            if b in self._data:
                vals = self._data[b]
            elif b in self._index.names:
                vals = self._index.get_level_values(b)._values
            else:
                raise KeyError(b)
            if key is not None:
                vals = key(Series._new(vals, self._index, b))._values
            keys.append(vals)
        pos = _sorted_positions(keys, ascending, na_position)
        out = self._take(pos)
        if ignore_index:
            object.__setattr__(out, "_index", default_index(len(pos)))
        if inplace:
            self._replace_with(out)
            return None
        return out

    def sort_index(self, axis=0, ascending=True, inplace=False, level=None, na_position="last", ignore_index=False):
        if axis in (1, "columns"):
            cols = sorted(self._data.keys(), key=_sort_key, reverse=not ascending)
            return self[cols]
        pos = _sorted_positions([self._index._values], ascending, na_position)
        out = self._take(pos)
        if inplace:
            self._replace_with(out)
            return None
        return out

    def nlargest(self, n, columns, keep="first"):
        return self.sort_values(columns, ascending=False).head(n)

    def nsmallest(self, n, columns, keep="first"):
        return self.sort_values(columns, ascending=True).head(n)

    def drop_duplicates(self, subset=None, keep="first", inplace=False, ignore_index=False):
        dup = self.duplicated(subset, keep)._values
        out = self._take([i for i, d in enumerate(dup) if not d])
        if ignore_index:
            object.__setattr__(out, "_index", default_index(len(out)))
        if inplace:
            self._replace_with(out)
            return None
        return out

    def duplicated(self, subset=None, keep="first"):
        cols = [subset] if isinstance(subset, str) else (list(subset) if subset is not None else list(self._data))
        seen = {}
        n = len(self._index)
        out = [False] * n
        order = range(n) if keep != "last" else range(n - 1, -1, -1)
        for i in order:
            k = tuple(_hkey(self._data[c][i]) for c in cols)
            if k in seen:
                out[i] = True
                if keep is False:
                    out[seen[k]] = True
            else:
                seen[k] = i
        return Series._new(out, self._index, None, DT_BOOL)

    # ------------------------------------------------------------ index
    def set_index(self, keys, drop=True, append=False, inplace=False, verify_integrity=False):
        if not isinstance(keys, (list, tuple)):
            keys = [keys]
        arrays = []
        names = []
        for k in keys:
            if isinstance(k, Series):
                arrays.append(list(k._values))
                names.append(k.name)
            elif isinstance(k, Index):
                arrays.append(list(k._values))
                names.append(k.name)
            else:
                arrays.append(list(self._data[k]))
                names.append(k)
        if append:
            if self._index.nlevels == 1:
                arrays.insert(0, list(self._index._values))
            names = list(self._index.names) + names
        if len(arrays) == 1:
            idx = Index._new(arrays[0], names, self._dtypes.get(keys[0]) if not isinstance(keys[0], (Series, Index)) else None)
        else:
            idx = Index._new(list(zip(*arrays)), names)
        data = {c: v for c, v in self._data.items() if not (drop and any(c is k or (not isinstance(k, (Series, Index)) and c == k) for k in keys))}
        out = self._with(data, None, idx)
        if inplace:
            self._replace_with(out)
            return None
        return out

    def reset_index(self, level=None, drop=False, inplace=False, names=None, allow_duplicates=False):
        idx = self._index
        data = {}
        dts = {}
        if not drop:
            if idx.nlevels > 1:
                for i, nm in enumerate(idx._names):
                    key = nm if nm is not None else "level_%d" % i
                    data[key] = [t[i] for t in idx._values]
                    dts[key] = None
            else:
                nm = idx.name
                key = nm if nm is not None else ("index" if "index" not in self._data else "level_0")
                if names is not None:
                    key = names if not isinstance(names, list) else names[0]
                data[key] = list(idx._values)
                dts[key] = idx._dtype
            if self.columns.nlevels > 1:
                n = self.columns.nlevels
                data = {(k,) + ("",) * (n - 1): v for k, v in data.items()}
                dts = {(k,) + ("",) * (n - 1): v for k, v in dts.items()}
        for c, v in self._data.items():
            data[c] = v
            dts[c] = self._dtypes[c]
        out = DataFrame._new(data, dts, default_index(len(idx)), list(self._columns_names))
        if inplace:
            self._replace_with(out)
            return None
        return out

    def droplevel(self, level, axis=0):
        out = self.copy()
        if axis in (1, "columns"):
            cols = self.columns.droplevel(level)
            out.columns = cols._values
            out._columns_names = cols._names
        else:
            object.__setattr__(out, "_index", self._index.droplevel(level))
        return out

    @property
    def T(self):
        return self.transpose()

    def transpose(self):
        cols = list(self._data.keys())
        data = {}
        for i, lab in enumerate(self._index._values):
            data[lab] = [self._data[c][i] for c in cols]
        return DataFrame._new(data, {}, self.columns, list(self._index._names))

    def stack(self, level=-1, dropna=True):
        cols = list(self._data.keys())
        labels, vals = [], []
        for i, lab in enumerate(self._index._values):
            for c in cols:
                v = self._data[c][i]
                if dropna and isna_scalar(v):
                    continue
                labels.append((lab if not isinstance(lab, tuple) else lab) + (c,) if isinstance(lab, tuple) else (lab, c))
                vals.append(v)
        names = list(self._index._names) + [self._columns_names[0]]
        return Series._new(vals, Index._new(labels, names), None)

    def unstack(self, level=-1, fill_value=None):
        cols = list(self._data.keys())
        if len(cols) == 1:
            out = self._col(cols[0]).unstack(level, fill_value)
            return out
        parts = {}
        for c in cols:
            u = self._col(c).unstack(level, fill_value)
            for k in u._data:
                parts[(c, k)] = u._data[k]
            idx = u._index
        return DataFrame._new(parts, {}, idx, [None, self._index._names[-1]])

    def melt(self, id_vars=None, value_vars=None, var_name=None, value_name="value", ignore_index=True):
        id_vars = [id_vars] if isinstance(id_vars, str) else list(id_vars or [])
        if value_vars is None:
            value_vars = [c for c in self._data if c not in id_vars]
        elif isinstance(value_vars, str):
            value_vars = [value_vars]
        var_name = var_name or (self._columns_names[0] if self._columns_names[0] is not None else "variable")
        data = {c: [] for c in id_vars}
        data[var_name] = []
        data[value_name] = []
        for vv in value_vars:
            for c in id_vars:
                data[c].extend(self._data[c])
            data[var_name].extend([vv] * len(self))
            data[value_name].extend(self._data[vv])
        return DataFrame(data)

    def pivot(self, index=None, columns=None, values=None):
        from ._reshape import pivot
        return pivot(self, index, columns, values)

    def pivot_table(self, values=None, index=None, columns=None, aggfunc="mean", fill_value=None, margins=False, dropna=True, margins_name="All", observed=True, sort=True):
        from ._reshape import pivot_table
        return pivot_table(self, values, index, columns, aggfunc, fill_value, margins, dropna, margins_name, sort)

    def explode(self, column, ignore_index=False):
        rows = []
        labels = []
        cols = list(self._data.keys())
        for i, lab in enumerate(self._index._values):
            v = self._data[column][i]
            items = v if isinstance(v, (list, tuple)) and v else [NaN if isinstance(v, (list, tuple)) else v]
            for x in items:
                rows.append([x if c == column else self._data[c][i] for c in cols])
                labels.append(lab)
        out = DataFrame({c: [r[j] for r in rows] for j, c in enumerate(cols)}, index=Index._new(labels, self._index._names))
        if ignore_index:
            object.__setattr__(out, "_index", default_index(len(rows)))
        return out

    # ------------------------------------------------------------ combine
    def groupby(self, by=None, axis=0, level=None, as_index=True, sort=True, group_keys=True, observed=True, dropna=True):
        from ._groupby import DataFrameGroupBy
        return DataFrameGroupBy(self, by, level=level, as_index=as_index, sort=sort, dropna=dropna)

    def merge(self, right, how="inner", on=None, left_on=None, right_on=None, left_index=False, right_index=False, sort=False, suffixes=("_x", "_y"), indicator=False, validate=None):
        from ._reshape import merge
        return merge(self, right, how, on, left_on, right_on, left_index, right_index, sort, suffixes, indicator)

    def join(self, other, on=None, how="left", lsuffix="", rsuffix="", sort=False):
        from ._reshape import merge
        if isinstance(other, Series):
            other = other.to_frame()
        overlap = [c for c in other._data if c in self._data]
        if overlap and not (lsuffix or rsuffix):
            raise ValueError("columns overlap but no suffix specified: %s" % overlap)
        if on is not None:
            return merge(self, other, how, None, on, None, False, True, sort, (lsuffix, rsuffix), False)
        return merge(self, other, how, None, None, None, True, True, sort, (lsuffix, rsuffix), False)

    def combine_first(self, other):
        out = self.copy()
        for c in other._data:
            if c in out._data:
                out[c] = out._col(c).fillna(other._col(c))
            else:
                out[c] = other._col(c)
        return out

    def update(self, other):
        for c in other._data:
            if c in self._data:
                o = other._col(c).reindex(self._index)
                vals = [ov if not isna_scalar(ov) else v for v, ov in zip(self._data[c], o._values)]
                self._set_col(c, vals, None)

    def equals(self, other):
        if not isinstance(other, DataFrame) or list(self._data) != list(other._data):
            return False
        return all(self._col(c).equals(other._col(c)) for c in self._data)

    def rolling(self, window, min_periods=None, center=False, on=None):
        from ._window import Rolling
        return Rolling(self, window, min_periods, center)

    def expanding(self, min_periods=1):
        from ._window import Expanding
        return Expanding(self, min_periods)

    def ewm(self, span=None, alpha=None, com=None, adjust=True):
        from ._window import EWM
        return EWM(self, span=span, alpha=alpha, com=com, adjust=adjust)

    def resample(self, rule, on=None, **kw):
        from ._window import resample_frame
        return resample_frame(self, rule, on, **kw)

    # ------------------------------------------------------------ output
    def to_dict(self, orient="dict", into=dict, index=True):
        cols = list(self._data.keys())
        if orient == "dict":
            return {c: dict(zip(self._index._values, self._data[c])) for c in cols}
        if orient == "list":
            return {c: list(self._data[c]) for c in cols}
        if orient == "records":
            return [{c: self._data[c][i] for c in cols} for i in range(len(self._index))]
        if orient == "index":
            return {lab: {c: self._data[c][i] for c in cols} for i, lab in enumerate(self._index._values)}
        if orient == "series":
            return {c: self._col(c) for c in cols}
        if orient == "split":
            return {"index": list(self._index._values), "columns": cols, "data": self.values}
        if orient == "tight":
            return {"index": list(self._index._values), "columns": cols, "data": self.values,
                    "index_names": list(self._index.names), "column_names": list(self._columns_names)}
        raise ValueError("orient '%s' not understood" % orient)

    def to_csv(self, path_or_buf=None, sep=",", na_rep="", float_format=None, columns=None, header=True, index=True, index_label=None, mode="w", encoding=None, lineterminator="\n", quoting=None, quotechar='"', date_format=None, decimal="."):
        from ._io import to_csv
        return to_csv(self, path_or_buf, sep, na_rep, float_format, columns, header, index, index_label, mode, lineterminator, quotechar, date_format, decimal)

    def to_json(self, path_or_buf=None, orient=None, indent=None, date_format=None, double_precision=10, force_ascii=True, lines=False, index=None):
        from ._io import frame_to_json
        return frame_to_json(self, path_or_buf, orient, indent, date_format, double_precision, force_ascii, lines, index)

    def to_records(self, index=True):
        cols = list(self._data.keys())
        return [((self._index._values[i],) if index else ()) + tuple(self._data[c][i] for c in cols) for i in range(len(self._index))]

    @property
    def plot(self):
        from ._plot import PlotAccessor
        return PlotAccessor(self)

    def __array__(self, *a):
        return self.values


# -------------------------------------------------------------------- helpers

def _mem(b):
    for unit in ("bytes", "KB", "MB", "GB"):
        if b < 1024 or unit == "GB":
            return ("%.1f+ %s" % (b, unit)) if unit != "bytes" else "%.1f+ bytes" % b
        b /= 1024.0


def _write_text(buf, s):
    if hasattr(buf, "write"):
        buf.write(s)
    else:
        with open(str(buf), "w") as f:
            f.write(s)


def _common_dtype(dts):
    s = set(str(d) for d in dts)
    if not s:
        return DT_OBJ
    if s <= {"int64"}:
        return DT_INT
    if s <= {"int64", "float64"} and s:
        return DT_FLOAT
    if s == {"bool"}:
        return DT_BOOL
    if s == {"datetime64[ns]"}:
        return DT_DATE
    return DT_OBJ


def _describe_column(vals, dt, percentiles=None):
    pcts = percentiles if percentiles is not None else [0.25, 0.5, 0.75]
    if 0.5 not in pcts:
        pcts = sorted(list(pcts) + [0.5])
    xs = _agg._clean(vals)
    if dt in ("int64", "float64"):
        fx = [float(x) for x in xs]
        labels = ["count", "mean", "std", "min"] + [_pct_label(p) for p in pcts] + ["max"]
        vals = [float(len(xs)), _agg._mean(fx), _std(fx), min(fx) if fx else NaN]
        vals += [_agg.quantile(fx, p) for p in pcts]
        vals += [max(fx) if fx else NaN]
        return labels, vals
    if dt == "datetime64[ns]":
        labels = ["count", "mean", "min"] + [_pct_label(p) for p in pcts] + ["max"]
        s = sorted(xs)
        from datetime import timedelta
        def q(p):
            if not s:
                return NaT
            pos = (len(s) - 1) * p
            lo = int(pos)
            hi = min(lo + 1, len(s) - 1)
            return s[lo] + (s[hi] - s[lo]) * (pos - lo)
        vals = [len(xs), _agg._dt_mean(xs), s[0] if s else NaT] + [q(p) for p in pcts] + [s[-1] if s else NaT]
        return labels, vals
    counts = {}
    for x in xs:
        k = _hkey(x)
        counts[k] = counts.get(k, 0) + 1
    if counts:
        top = max(counts.values())
        topv = next(x for x in xs if counts[_hkey(x)] == top)
    else:
        top, topv = NaN, NaN
    return ["count", "unique", "top", "freq"], [len(xs), len(counts), topv, top]


def _pct_label(p):
    v = p * 100
    if v == int(v):
        return "%d%%" % int(v)
    s = ("%.10f" % v).rstrip("0").rstrip(".")
    return s + "%"


def _std(fx):
    v = _agg._var(fx, 1)
    return math.sqrt(v) if v == v else NaN


def _pearson(xs, ys):
    n = len(xs)
    if n < 2:
        return NaN
    mx = sum(xs) / n
    my = sum(ys) / n
    sxy = sum((x - mx) * (y - my) for x, y in zip(xs, ys))
    sxx = sum((x - mx) ** 2 for x in xs)
    syy = sum((y - my) ** 2 for y in ys)
    if sxx == 0 or syy == 0:
        return NaN
    r = sxy / math.sqrt(sxx * syy)
    return max(-1.0, min(1.0, r))


def _spearman(xs, ys):
    rx = Series(xs).rank()._values
    ry = Series(ys).rank()._values
    return _pearson(rx, ry)


def _sample_positions(n, k, frac, replace, random_state):
    import random
    rng = random.Random(random_state) if not hasattr(random_state, "random") else random_state
    if k is None:
        k = int(round(n * frac)) if frac is not None else 1
    if replace:
        return [rng.randrange(n) for _ in range(k)]
    return rng.sample(range(n), k)


# -------------------------------------------------------------------- loc

def _row_positions(df, key, by_label):
    idx = df._index
    n = len(idx)
    if isinstance(key, slice):
        if by_label:
            return _label_slice(idx, key), False
        return list(range(n))[key], False
    if isinstance(key, Series) and key._dtype == "bool":
        return _mask_positions(key, idx), False
    if isinstance(key, (list, Index, Series)):
        keys = list(key)
        if keys and isinstance(keys[0], bool):
            return [i for i, b in enumerate(keys) if b], False
        if by_label:
            pos = []
            for k in keys:
                pos.extend(idx._positions(k))
            return pos, False
        return [k if k >= 0 else n + k for k in keys], False
    if callable(key):
        return _row_positions(df, key(df), by_label)
    if by_label:
        p = idx.get_loc(key)
        if isinstance(p, list):
            return p, False
        return [p], True
    if key < 0:
        key += n
    if not 0 <= key < n:
        raise IndexError("single positional indexer is out-of-bounds")
    return [key], True


def _col_keys(df, key, by_label):
    cols = list(df._data.keys())
    if isinstance(key, slice):
        if by_label:
            start = cols.index(key.start) if key.start is not None else 0
            stop = cols.index(key.stop) + 1 if key.stop is not None else len(cols)
            return cols[start:stop:key.step], False
        return cols[key], False
    if isinstance(key, (list, Index, Series)):
        keys = list(key)
        if keys and isinstance(keys[0], bool):
            return [c for c, b in zip(cols, keys) if b], False
        if by_label:
            return keys, False
        return [cols[k] for k in keys], False
    if callable(key):
        return _col_keys(df, key(df), by_label)
    if by_label:
        return [key], True
    return [cols[key]], True


class _FrameLoc:
    _by_label = True

    def __init__(self, df):
        self._df = df

    def __getitem__(self, key):
        df = self._df
        if isinstance(key, tuple) and len(key) == 2 and not (df._index.nlevels > 1 and key in df._index):
            rk, ck = key
        else:
            rk, ck = key, slice(None)
        rows, rscalar = _row_positions(df, rk, self._by_label)
        cols, cscalar = _col_keys(df, ck, self._by_label)
        for c in cols:
            if c not in df._data:
                raise KeyError(c)
        if rscalar and cscalar:
            return df._data[cols[0]][rows[0]]
        if cscalar:
            c = cols[0]
            s = Series._new([df._data[c][i] for i in rows], df._index.take(rows), c, df._dtypes[c])
            return s
        if rscalar:
            i = rows[0]
            vals = [df._data[c][i] for c in cols]
            dt = _common_dtype([df._dtypes[c] for c in cols])
            sub_cols = df.columns.take([list(df._data.keys()).index(c) for c in cols])
            return Series._new(normalize(vals, dt) if dt == "float64" else vals, sub_cols, df._index._values[i], dt)
        data = {c: [df._data[c][i] for i in rows] for c in cols}
        return DataFrame._new(data, {c: df._dtypes[c] for c in cols}, df._index.take(rows), list(df._columns_names))

    def __setitem__(self, key, value):
        df = self._df
        if isinstance(key, tuple) and len(key) == 2:
            rk, ck = key
        else:
            rk, ck = key, slice(None)
        try:
            rows, rscalar = _row_positions(df, rk, self._by_label)
        except KeyError:
            if not self._by_label:
                raise
            # enlarge with a new row
            n = len(df._index)
            object.__setattr__(df, "_index", Index._new(df._index._values + [rk], df._index._names))
            for c in df._data:
                df._data[c] = list(df._data[c]) + [NaN]
            rows, rscalar = [n], True
        if isinstance(ck, (str, int)) and self._by_label and ck not in df._data:
            df[ck] = [NaN] * len(df._index)
        cols, cscalar = _col_keys(df, ck, self._by_label)
        for j, c in enumerate(cols):
            if c not in df._data:
                df[c] = [NaN] * len(df._index)
            vals = list(df._data[c])
            if isinstance(value, DataFrame):
                src = value._col(value.columns[j] if c not in value._data else c)
                v = src.reindex(df._index.take(rows))._values
            elif isinstance(value, Series):
                if rscalar and not cscalar:
                    v = [value[c] if c in value.index else NaN]
                else:
                    v = value.reindex(df._index.take(rows))._values if not value.index.equals(df._index.take(rows)) else value._values
            elif _is_listlike(value) and not isinstance(value, str):
                lv = list(value)
                if rscalar and not cscalar:
                    v = [lv[j]]
                elif lv and isinstance(lv[0], (list, tuple)):
                    v = [r[j] for r in lv]
                else:
                    v = lv
            else:
                v = [value] * len(rows)
            for p, x in zip(rows, v):
                vals[p] = x
            df._set_col(c, vals, None)


class _FrameILoc(_FrameLoc):
    _by_label = False
