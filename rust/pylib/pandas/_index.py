# SPDX-License-Identifier: MIT
# Index, RangeIndex and MultiIndex for rpy's pandas.

from ._util import (NaN, NaT, isna_scalar, infer_dtype, normalize, dtype,
                    DT_INT, DT_OBJ)


def _sort_key(v):
    if isna_scalar(v):
        return (2, 0)
    if isinstance(v, tuple):
        return (0, tuple(_sort_key(x) for x in v))
    if isinstance(v, (int, float)):
        return (0, v)
    return (1, v) if isinstance(v, str) else (0, v)


class Index:
    _is_range = False
    freq = None

    def __init__(self, data=None, dtype=None, name=None, names=None, copy=False, tupleize_cols=True):
        if isinstance(data, Index):
            vals = list(data._values)
            if name is None and names is None:
                names = list(data.names)
        elif data is None:
            vals = []
        else:
            vals = list(data)
        if names is not None:
            names = list(names)
        elif tupleize_cols and vals and all(isinstance(v, tuple) for v in vals) and len(set(len(v) for v in vals)) == 1 and len(vals[0]) > 1:
            names = [None] * len(vals[0])
        else:
            names = [name]
        self._values = vals
        self._names = names
        self._dtype = None if dtype is None else _dt(dtype)
        self._lookup = None

    @classmethod
    def _new(cls, vals, names, dt=None):
        self = object.__new__(Index)
        self._values = vals
        self._names = list(names)
        self._dtype = dt
        self._lookup = None
        return self

    # --------------------------------------------------- basics
    @property
    def dtype(self):
        if self._dtype is None:
            self._dtype = DT_OBJ if self.nlevels > 1 else infer_dtype(self._values)
        return self._dtype

    @property
    def nlevels(self):
        return len(self._names)

    @property
    def name(self):
        return self._names[0] if len(self._names) == 1 else None

    @name.setter
    def name(self, v):
        self._names = [v] if self.nlevels == 1 else self._names

    @property
    def names(self):
        return _FrozenList(self._names)

    @names.setter
    def names(self, v):
        v = list(v)
        if len(v) != self.nlevels:
            raise ValueError("Length of names must match number of levels in MultiIndex.")
        self._names = v

    @property
    def values(self):
        return list(self._values)

    @property
    def array(self):
        return list(self._values)

    def to_numpy(self):
        return list(self._values)

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
    def empty(self):
        return not self._values

    def __len__(self):
        return len(self._values)

    def __iter__(self):
        return iter(self._values)

    def __contains__(self, key):
        return key in self._get_lookup()

    def __hash__(self):
        raise TypeError("unhashable type: 'Index'")

    def __bool__(self):
        raise ValueError("The truth value of a Index is ambiguous. Use a.empty, a.bool(), a.item(), a.any() or a.all().")

    def _get_lookup(self):
        if self._lookup is None:
            d = {}
            for i, v in enumerate(self._values):
                k = _hkey(v)
                if k in d:
                    p = d[k]
                    if isinstance(p, list):
                        p.append(i)
                    else:
                        d[k] = [p, i]
                else:
                    d[k] = i
            self._lookup = d
        return self._lookup

    def get_loc(self, key):
        d = self._get_lookup()
        k = _hkey(key)
        if k not in d:
            if self.nlevels > 1 and not isinstance(key, tuple):
                hits = [i for i, v in enumerate(self._values) if v[0] == key]
                if hits:
                    return hits
            raise KeyError(key)
        return d[k]

    def get_indexer(self, keys):
        d = self._get_lookup()
        out = []
        for k in keys:
            p = d.get(_hkey(k), -1)
            out.append(p[0] if isinstance(p, list) else p)
        return out

    def _positions(self, key):
        p = self.get_loc(key)
        return p if isinstance(p, list) else [p]

    @property
    def is_unique(self):
        return len(self._get_lookup()) == len(self._values)

    @property
    def has_duplicates(self):
        return not self.is_unique

    @property
    def is_monotonic_increasing(self):
        v = self._values
        try:
            return all(v[i] <= v[i + 1] for i in range(len(v) - 1))
        except TypeError:
            return False

    def __getitem__(self, key):
        if isinstance(key, int):
            return self._values[key]
        if isinstance(key, slice):
            return self._take_slice(key)
        from ._core import Series
        if isinstance(key, Series):
            key = key._values
        key = list(key)
        if key and isinstance(key[0], bool):
            return self.take([i for i, b in enumerate(key) if b])
        return self.take(key)

    def _take_slice(self, sl):
        return Index._new(self._values[sl], self._names, self._dtype)

    def take(self, positions):
        vals = self._values
        return Index._new([vals[i] for i in positions], self._names, self._dtype)

    def equals(self, other):
        if self is other:
            return True
        if not isinstance(other, Index):
            return False
        if len(self) != len(other):
            return False
        if self._is_range and other._is_range:
            return self._values == other._values
        a, b = self._values, other._values
        for x, y in zip(a, b):
            if x != y and not (isna_scalar(x) and isna_scalar(y)):
                return False
        return True

    def _cmp(self, other, op):
        vals = self._values
        if isinstance(other, (list, tuple, Index)):
            return [op(a, b) for a, b in zip(vals, other)]
        return [op(a, other) for a in vals]

    def __eq__(self, other):
        return self._cmp(other, lambda a, b: a == b)

    def __ne__(self, other):
        return self._cmp(other, lambda a, b: a != b)

    def copy(self, name=None):
        out = Index._new(list(self._values), self._names, self._dtype)
        if name is not None:
            out._names = [name]
        return out

    def rename(self, name, inplace=False):
        out = self if inplace else self.copy()
        if isinstance(name, (list, tuple)) and self.nlevels > 1:
            out._names = list(name)
        else:
            out._names = [name]
        return None if inplace else out

    set_names = rename

    def astype(self, dt):
        from ._util import convert
        vals, d = convert(self._values, dt)
        return Index._new(vals, self._names, d)

    def map(self, f):
        if isinstance(f, dict):
            return Index([f.get(v, NaN) for v in self._values], name=self.name)
        return Index([f(v) for v in self._values], name=self.name)

    def unique(self):
        seen = set()
        out = []
        for v in self._values:
            k = _hkey(v)
            if k not in seen:
                seen.add(k)
                out.append(v)
        return Index._new(out, self._names, self._dtype)

    def nunique(self):
        return len(self.unique())

    def isin(self, values):
        s = set(_hkey(v) for v in values)
        return [_hkey(v) in s for v in self._values]

    def isna(self):
        return [isna_scalar(v) for v in self._values]

    def sort_values(self, ascending=True):
        vals = sorted(self._values, key=_sort_key, reverse=not ascending)
        return Index._new(vals, self._names, self._dtype)

    def append(self, other):
        if isinstance(other, (list, tuple)):
            vals = list(self._values)
            for o in other:
                vals.extend(o._values)
            return Index(vals, name=self.name)
        return Index(self._values + other._values, name=self.name if self.name == other.name else None)

    def union(self, other, sort=None):
        seen = set(_hkey(v) for v in self._values)
        vals = list(self._values) + [v for v in other if _hkey(v) not in seen]
        if sort is not False:
            try:
                vals = sorted(vals, key=_sort_key)
            except TypeError:
                pass
        return Index(vals, name=self.name)

    def intersection(self, other):
        s = set(_hkey(v) for v in other)
        return Index([v for v in self._values if _hkey(v) in s], name=self.name)

    def difference(self, other, sort=None):
        s = set(_hkey(v) for v in other)
        vals = [v for v in self._values if _hkey(v) not in s]
        if sort is not False:
            try:
                vals = sorted(vals, key=_sort_key)
            except TypeError:
                pass
        return Index(vals, name=self.name)

    def drop(self, labels):
        if not isinstance(labels, (list, tuple, Index)):
            labels = [labels]
        s = set(_hkey(v) for v in labels)
        return Index._new([v for v in self._values if _hkey(v) not in s], self._names)

    def min(self):
        return min(v for v in self._values if not isna_scalar(v))

    def max(self):
        return max(v for v in self._values if not isna_scalar(v))

    def argmax(self):
        v = self._values
        return max(range(len(v)), key=lambda i: v[i])

    def to_series(self):
        from ._core import Series
        return Series._new(list(self._values), self, self.name)

    def to_frame(self, index=True, name=None):
        from ._core import DataFrame
        if self.nlevels > 1:
            data = {nm if nm is not None else "level_%d" % i: [t[i] for t in self._values] for i, nm in enumerate(self._names)}
        else:
            data = {name if name is not None else (self.name if self.name is not None else 0): list(self._values)}
        return DataFrame(data, index=self if index else None)

    def get_level_values(self, level):
        if self.nlevels == 1:
            return self
        if not isinstance(level, int):
            level = self._names.index(level)
        return Index([t[level] for t in self._values], name=self._names[level])

    @property
    def levels(self):
        return [Index(sorted(set(t[i] for t in self._values), key=_sort_key), name=self._names[i]) for i in range(self.nlevels)]

    def droplevel(self, level=0):
        if not isinstance(level, (list, tuple)):
            level = [level]
        lv = [l if isinstance(l, int) else self._names.index(l) for l in level]
        keep = [i for i in range(self.nlevels) if i not in lv and i - self.nlevels not in lv]
        if len(keep) == 1:
            k = keep[0]
            return Index([t[k] for t in self._values], name=self._names[k])
        return Index._new([tuple(t[k] for k in keep) for t in self._values], [self._names[k] for k in keep])

    def to_flat_index(self):
        return Index._new(list(self._values), [None])

    def strftime(self, fmt):
        from ._util import strftime
        return Index([strftime(v, fmt) for v in self._values], name=self.name)

    def _dt_part(self, f):
        return Index([f(v) for v in self._values], name=self.name)

    year = property(lambda self: self._dt_part(lambda d: d.year))
    month = property(lambda self: self._dt_part(lambda d: d.month))
    day = property(lambda self: self._dt_part(lambda d: d.day))
    dayofweek = property(lambda self: self._dt_part(lambda d: d.weekday()))
    quarter = property(lambda self: self._dt_part(lambda d: (d.month - 1) // 3 + 1))

    def month_name(self):
        from ._series import _MONTH_NAMES
        return self._dt_part(lambda d: _MONTH_NAMES[d.month - 1])

    def day_name(self):
        from ._series import _DAY_NAMES
        return self._dt_part(lambda d: _DAY_NAMES[d.weekday()])

    @property
    def date(self):
        return [d.date() for d in self._values]

    @property
    def str(self):
        from ._core import _StrAccessor
        return _StrAccessor(self.to_series(), as_index=True)

    def __repr__(self):
        from ._fmt import index_repr
        return index_repr(self)

    __str__ = __repr__

    def __add__(self, other):
        return Index([a + b for a, b in zip(self._values, other)] if isinstance(other, (list, Index)) else [a + other for a in self._values], name=self.name)


class RangeIndex(Index):
    _is_range = True

    def __init__(self, start=0, stop=None, step=1, name=None):
        if stop is None:
            start, stop = 0, start
        self._start, self._stop, self._step = start, stop, step
        self._values = list(range(start, stop, step))
        self._names = [name]
        self._dtype = DT_INT
        self._lookup = None

    def get_loc(self, key):
        if isinstance(key, int) and not isinstance(key, bool) and self._step == 1 and self._start <= key < self._stop:
            return key - self._start
        return Index.get_loc(self, key)

    def _take_slice(self, sl):
        start, stop, step = sl.indices(len(self._values))
        if step == 1:
            r = RangeIndex(self._start + start * self._step, self._start + max(stop, start) * self._step, self._step, self.name)
            return r
        return Index._new(self._values[sl], self._names, DT_INT)

    def copy(self, name=None):
        return RangeIndex(self._start, self._stop, self._step, name if name is not None else self.name)


def default_index(n):
    return RangeIndex(0, n)


class MultiIndex(Index):
    def __new__(cls, *a, **k):
        return object.__new__(Index)

    @staticmethod
    def from_tuples(tuples, names=None):
        tuples = [tuple(t) for t in tuples]
        n = len(tuples[0]) if tuples else 0
        return Index._new(tuples, names if names is not None else [None] * n)

    @staticmethod
    def from_arrays(arrays, names=None):
        arrays = [list(a) for a in arrays]
        tuples = list(zip(*arrays))
        return Index._new(tuples, names if names is not None else [getattr(a, "name", None) for a in arrays])

    @staticmethod
    def from_product(iterables, names=None):
        out = [()]
        for it in iterables:
            out = [t + (v,) for t in out for v in it]
        return Index._new(out, names if names is not None else [None] * len(iterables))

    @staticmethod
    def from_frame(df, names=None):
        cols = list(df.columns)
        return Index._new(list(zip(*[df._data[c] for c in cols])), names or cols)


class _FrozenList(list):
    def __repr__(self):
        return "FrozenList(%s)" % list.__repr__(self)


def _hkey(v):
    if type(v) is float and v != v:
        return ("__nan__",)
    if v is NaT or v is None:
        return ("__nan__",)
    try:
        hash(v)
    except TypeError:
        return repr(v)
    return v


def _dt(d):
    return dtype(d)


def ensure_index(v, n=None, name=None):
    if v is None:
        return default_index(n)
    if isinstance(v, Index):
        return v
    from ._core import Series
    if isinstance(v, Series):
        return Index(v._values, name=v.name)
    if isinstance(v, range) and v.step == 1:
        return RangeIndex(v.start, v.stop, name=name)
    return Index(list(v), name=name)
