# SPDX-License-Identifier: MIT
# Top-level functions of rpy's pandas.

from datetime import datetime, timedelta

from ._util import (NA, NaN, nan, NaT, isna_scalar, parse_datetime, get_option,
                    set_option, option_context, OPTIONS as options, DT_DATE,
                    DT_FLOAT, DT_OBJ, month_end, add_months, dtype)
from ._index import Index, RangeIndex, MultiIndex, default_index, _hkey
from ._series import Series
from ._frame import DataFrame
from ._reshape import concat, merge, cut, qcut, crosstab, get_dummies, pivot_table


def isna(obj):
    if isinstance(obj, (Series, DataFrame)):
        return obj.isna()
    if isinstance(obj, Index):
        return obj.isna()
    if isinstance(obj, (list, tuple)):
        return [isna_scalar(v) for v in obj]
    return isna_scalar(obj)


isnull = isna


def notna(obj):
    r = isna(obj)
    if isinstance(r, (Series, DataFrame)):
        return ~r
    if isinstance(r, list):
        return [not v for v in r]
    return not r


notnull = notna


def _to_num(v, errors, downcast=None):
    if isna_scalar(v):
        return NaN
    if isinstance(v, (int, float)):
        return v
    s = str(v).strip()
    try:
        return int(s)
    except ValueError:
        pass
    try:
        return float(s)
    except ValueError:
        if errors == "coerce":
            return NaN
        if errors == "ignore":
            return v
        raise ValueError('Unable to parse string "%s"' % s)


def to_numeric(arg, errors="raise", downcast=None):
    if isinstance(arg, Series):
        return arg._derive([_to_num(v, errors) for v in arg._values], None)
    if isinstance(arg, (list, tuple, Index)):
        return [_to_num(v, errors) for v in arg]
    return _to_num(arg, errors)


def to_datetime(arg, errors="raise", dayfirst=False, format=None, unit=None, utc=False, **kw):
    def conv(v):
        if unit is not None and isinstance(v, (int, float)) and not isna_scalar(v):
            mult = {"s": 1, "ms": 1e-3, "us": 1e-6, "ns": 1e-9, "D": 86400}[unit]
            return datetime(1970, 1, 1) + timedelta(seconds=v * mult)
        if format in ("mixed", "ISO8601"):
            return parse_datetime(v, None, dayfirst, errors)
        return parse_datetime(v, format, dayfirst, errors)
    if isinstance(arg, Series):
        return arg._derive([conv(v) for v in arg._values], DT_DATE if errors != "ignore" else None)
    if isinstance(arg, Index):
        return Index._new([conv(v) for v in arg._values], arg._names, DT_DATE)
    if isinstance(arg, DataFrame):
        cols = {c.lower(): c for c in arg._data}
        out = []
        for i in range(len(arg)):
            out.append(datetime(int(arg._data[cols["year"]][i]), int(arg._data[cols["month"]][i]), int(arg._data[cols.get("day", cols.get("days"))][i])))
        return Series._new(out, arg.index, None, DT_DATE)
    if isinstance(arg, (list, tuple)):
        return Index._new([conv(v) for v in arg], [None], DT_DATE)
    return conv(arg)


def Timestamp(ts_input=None, *a, **kw):
    if ts_input is None and kw:
        return datetime(kw.get("year"), kw.get("month"), kw.get("day"), kw.get("hour", 0), kw.get("minute", 0), kw.get("second", 0))
    if a:
        return datetime(ts_input, *a)
    if isinstance(ts_input, str) and ts_input.lower() == "now":
        return datetime.now()
    if isinstance(ts_input, str) and ts_input.lower() == "today":
        n = datetime.now()
        return datetime(n.year, n.month, n.day)
    return parse_datetime(ts_input)


Timestamp.now = lambda tz=None: datetime.now()
Timestamp.today = lambda: datetime.now()


def Timedelta(value=None, unit=None, **kw):
    if kw:
        return timedelta(**kw)
    if isinstance(value, (int, float)):
        u = {"D": "days", "d": "days", "days": "days", "h": "hours", "H": "hours", "m": "minutes", "min": "minutes",
             "s": "seconds", "ms": "milliseconds", "W": "weeks", "w": "weeks"}.get(unit or "ns", None)
        if u is None:
            return timedelta(microseconds=value / 1000)
        return timedelta(**{u: value})
    import re
    s = str(value).strip()
    m = re.match(r"^(-?\d+(?:\.\d+)?)\s*(days?|d|hours?|h|minutes?|min|m|seconds?|s|weeks?|w)$", s, re.IGNORECASE)
    if m:
        n = float(m.group(1))
        u = m.group(2).lower()
        key = "days" if u.startswith("d") else "hours" if u.startswith("h") else "minutes" if u.startswith("min") or u == "m" else "weeks" if u.startswith("w") else "seconds"
        return timedelta(**{key: n})
    raise ValueError("cannot parse Timedelta %r" % (value,))


def _freq_step(freq):
    import re
    m = re.match(r"^(\d*)([A-Za-z]+)$", freq or "D")
    n = int(m.group(1) or 1)
    unit = m.group(2)
    return n, unit


def date_range(start=None, end=None, periods=None, freq="D", name=None, normalize=False, inclusive="both"):
    n, unit = _freq_step(freq)
    u = unit.upper()
    start = parse_datetime(start) if start is not None else None
    end = parse_datetime(end) if end is not None else None

    def step(d, k=1):
        if u == "D":
            return d + timedelta(days=n * k)
        if u in ("H",):
            return d + timedelta(hours=n * k)
        if u in ("T", "MIN"):
            return d + timedelta(minutes=n * k)
        if u == "S":
            return d + timedelta(seconds=n * k)
        if u.startswith("W"):
            return d + timedelta(weeks=n * k)
        if u == "B":
            out = d
            for _ in range(abs(n * k)):
                out = out + timedelta(days=1 if k > 0 else -1)
                while out.weekday() >= 5:
                    out = out + timedelta(days=1 if k > 0 else -1)
            return out
        if u in ("MS",):
            y, mth = add_months(d, n * k)
            return datetime(y, mth, 1, d.hour, d.minute)
        if u in ("M", "ME"):
            y, mth = add_months(datetime(d.year, d.month, 1), n * k)
            return month_end(datetime(y, mth, 1))
        if u in ("QS",):
            y, mth = add_months(d, 3 * n * k)
            return datetime(y, mth, 1)
        if u in ("Q", "QE"):
            y, mth = add_months(datetime(d.year, d.month, 1), 3 * n * k)
            return month_end(datetime(y, mth, 1))
        if u in ("YS", "AS"):
            return datetime(d.year + n * k, 1, 1)
        if u in ("Y", "A", "YE"):
            return datetime(d.year + n * k, 12, 31)
        raise ValueError("Invalid frequency: %s" % freq)

    def anchor(d):
        if u in ("M", "ME"):
            return month_end(d)
        if u in ("Q", "QE"):
            q_end_month = 3 * ((d.month - 1) // 3) + 3
            return month_end(datetime(d.year, q_end_month, 1))
        if u in ("Y", "A", "YE"):
            return datetime(d.year, 12, 31)
        if u == "MS" and d.day != 1:
            y, m = add_months(d, 1)
            return datetime(y, m, 1)
        if u == "YS" and (d.month, d.day) != (1, 1):
            return datetime(d.year + 1, 1, 1)
        if u.startswith("W"):
            target = 6
            if "-" in u:
                days = ["MON", "TUE", "WED", "THU", "FRI", "SAT", "SUN"]
                target = days.index(u.split("-")[1])
            return d + timedelta(days=(target - d.weekday()) % 7)
        if u == "B":
            while d.weekday() >= 5:
                d = d + timedelta(days=1)
        return d

    out = []
    if start is not None:
        cur = anchor(start)
        while True:
            if end is not None and cur > end:
                break
            if periods is not None and len(out) >= periods:
                break
            out.append(cur)
            cur = step(cur)
    elif end is not None and periods is not None:
        cur = end
        while len(out) < periods:
            out.append(cur)
            cur = step(cur, -1)
        out.reverse()
    if normalize:
        out = [datetime(d.year, d.month, d.day) for d in out]
    idx = Index._new(out, [name], DT_DATE)
    idx.freq = {"M": "ME", "Q": "QE", "Y": "YE", "A": "YE"}.get(freq, freq)
    return idx


def period_range(start=None, end=None, periods=None, freq="M", name=None):
    idx = date_range(start, end, periods, "MS" if freq.upper() in ("M", "ME") else freq, name)
    if freq.upper() in ("M", "ME"):
        return Index(["%04d-%02d" % (d.year, d.month) for d in idx._values], name=name)
    return idx


def unique(values):
    if isinstance(values, Series):
        return values.unique()
    seen = set()
    out = []
    for v in values:
        k = _hkey(v)
        if k not in seen:
            seen.add(k)
            out.append(v)
    return out


def factorize(values, sort=False):
    vals = list(values)
    uniq = unique(vals)
    if sort:
        uniq = sorted(uniq)
    pos = {_hkey(u): i for i, u in enumerate(uniq)}
    return [pos[_hkey(v)] if not isna_scalar(v) else -1 for v in vals], Index(uniq)


def melt(frame, id_vars=None, value_vars=None, var_name=None, value_name="value"):
    return frame.melt(id_vars, value_vars, var_name, value_name)


def pivot(data, index=None, columns=None, values=None):
    return data.pivot(index=index, columns=columns, values=values)


class NamedAgg(tuple):
    def __new__(cls, column, aggfunc):
        self = tuple.__new__(cls, (column, aggfunc))
        return self

    column = property(lambda self: self[0])
    aggfunc = property(lambda self: self[1])


class Grouper:
    def __init__(self, key=None, freq=None, level=None):
        self.key = key
        self.freq = freq
        self.level = level


def read_excel(*a, **k):
    raise NotImplementedError("read_excel is not available in this runtime; export the sheet as CSV and use read_csv")


def read_parquet(*a, **k):
    raise NotImplementedError("read_parquet is not available in this runtime; use CSV or JSON")


class _Types:
    @staticmethod
    def _dt(x):
        if isinstance(x, (Series, Index)):
            return x.dtype
        return dtype(x)

    @staticmethod
    def is_numeric_dtype(x):
        return str(_Types._dt(x)) in ("int64", "float64", "bool")

    @staticmethod
    def is_integer_dtype(x):
        return str(_Types._dt(x)) == "int64"

    @staticmethod
    def is_float_dtype(x):
        return str(_Types._dt(x)) == "float64"

    @staticmethod
    def is_bool_dtype(x):
        return str(_Types._dt(x)) == "bool"

    @staticmethod
    def is_string_dtype(x):
        return str(_Types._dt(x)) == "object"

    is_object_dtype = is_string_dtype

    @staticmethod
    def is_datetime64_any_dtype(x):
        return str(_Types._dt(x)) == "datetime64[ns]"

    is_datetime64_dtype = is_datetime64_any_dtype


class _Api:
    types = _Types()


api = _Api()
