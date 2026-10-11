# SPDX-License-Identifier: MIT
# Reductions over a column's values, shared by Series, DataFrame and GroupBy.

import math

from ._util import NaN, NaT, isna_scalar, is_numeric

NUMERIC_ONLY = {"mean", "median", "std", "var", "sem", "prod", "quantile", "skew"}


def pairwise_sum(a, lo=0, n=None):
    """numpy's float summation (8 accumulators, blocks of 128)."""
    if n is None:
        n = len(a)
    if n < 8:
        res = 0.0
        for i in range(lo, lo + n):
            res += a[i]
        return res
    if n <= 128:
        r0, r1, r2, r3, r4, r5, r6, r7 = a[lo], a[lo + 1], a[lo + 2], a[lo + 3], a[lo + 4], a[lo + 5], a[lo + 6], a[lo + 7]
        i = 8
        end = n - (n % 8)
        while i < end:
            j = lo + i
            r0 += a[j]
            r1 += a[j + 1]
            r2 += a[j + 2]
            r3 += a[j + 3]
            r4 += a[j + 4]
            r5 += a[j + 5]
            r6 += a[j + 6]
            r7 += a[j + 7]
            i += 8
        res = ((r0 + r1) + (r2 + r3)) + ((r4 + r5) + (r6 + r7))
        while i < n:
            res += a[lo + i]
            i += 1
        return res
    n2 = n // 2
    n2 -= n2 % 8
    return pairwise_sum(a, lo, n2) + pairwise_sum(a, lo + n2, n - n2)


def fsum(xs):
    """Sum of floats as numpy.sum does it."""
    return pairwise_sum(xs, 0, len(xs))


def kahan_sum(xs):
    """pandas' groupby summation (Kahan compensated)."""
    s = 0.0
    c = 0.0
    for v in xs:
        y = v - c
        t = s + y
        c = t - s - y
        s = t
    return s


def _clean(vals):
    return [v for v in vals if not (v is None or v is NaT or (type(v) is float and v != v))]


def _sum(xs, dt):
    if dt == "object":
        if not xs:
            return 0
        out = xs[0]
        for x in xs[1:]:
            out = out + x
        return out
    if dt == "bool":
        return sum(1 for x in xs if x)
    if dt == "float64":
        return fsum(xs)
    return sum(xs)


def _mean(xs):
    if not xs:
        return NaN
    return fsum(xs) / len(xs)


def _var(xs, ddof=1):
    n = len(xs)
    if n - ddof <= 0:
        return NaN
    m = fsum(xs) / n
    return fsum([(x - m) * (x - m) for x in xs]) / (n - ddof)


def _median(xs):
    if not xs:
        return NaN
    s = sorted(xs)
    n = len(s)
    if n % 2:
        return float(s[n // 2])
    return (s[n // 2 - 1] + s[n // 2]) / 2


def quantile(xs, q, interpolation="linear"):
    if not xs:
        return NaN
    s = sorted(xs)
    pos = (len(s) - 1) * q
    lo = int(math.floor(pos))
    hi = min(lo + 1, len(s) - 1)
    frac = pos - lo
    if interpolation == "lower":
        return s[lo]
    if interpolation == "higher":
        return s[hi] if frac > 0 else s[lo]
    if interpolation == "nearest":
        return s[hi] if frac > 0.5 or (frac == 0.5 and hi % 2 == 0) else s[lo]
    if interpolation == "midpoint":
        return (s[lo] + s[hi]) / 2 if frac > 0 else float(s[lo])
    return s[lo] + (s[hi] - s[lo]) * frac if frac else float(s[lo])


def _skew(xs):
    n = len(xs)
    if n < 3:
        return NaN
    m = _mean(xs)
    m2 = math.fsum((x - m) ** 2 for x in xs) / n
    m3 = math.fsum((x - m) ** 3 for x in xs) / n
    if m2 == 0:
        return 0.0
    g1 = m3 / m2 ** 1.5
    return g1 * math.sqrt(n * (n - 1)) / (n - 2)


def _mode(xs):
    counts = {}
    for x in xs:
        counts[x] = counts.get(x, 0) + 1
    if not counts:
        return []
    top = max(counts.values())
    try:
        return sorted(k for k, c in counts.items() if c == top)
    except TypeError:
        return [k for k, c in counts.items() if c == top]


def reduce(vals, how, dt, skipna=True, ddof=1, q=0.5, min_count=0, grouped=False):
    """One reduction (by name) of a list of values; grouped follows pandas'
    groupby kernels (Kahan sums) rather than numpy's."""
    if grouped and dt == "float64" and how in ("sum", "mean"):
        xs = _clean(vals)
        if how == "sum":
            if min_count and len(xs) < min_count:
                return NaN
            return kahan_sum(xs)
        return kahan_sum(xs) / len(xs) if xs else NaN
    if how == "size":
        return len(vals)
    xs = _clean(vals) if skipna else vals
    if not skipna and len(xs) != len(_clean(vals)) and how not in ("count", "size", "first", "last", "nunique"):
        return NaN
    if how == "sum":
        if min_count and len(xs) < min_count:
            return NaN
        return _sum(xs, dt)
    if how == "count":
        return len(_clean(vals))
    if how == "mean":
        if dt == "datetime64[ns]":
            return _dt_mean(xs)
        return _mean([float(x) for x in xs])
    if how == "median":
        return _median(xs)
    if how in ("min", "max"):
        if not xs:
            return NaT if dt == "datetime64[ns]" else NaN
        try:
            return min(xs) if how == "min" else max(xs)
        except TypeError:
            ss = [str(x) for x in xs]
            return min(ss) if how == "min" else max(ss)
    if how == "std":
        v = _var([float(x) for x in xs], ddof)
        return math.sqrt(v) if v == v else NaN
    if how == "var":
        return _var([float(x) for x in xs], ddof)
    if how == "sem":
        v = _var([float(x) for x in xs], ddof)
        return math.sqrt(v / len(xs)) if v == v else NaN
    if how == "prod":
        out = 1
        for x in xs:
            out *= x
        return out
    if how == "nunique":
        from ._index import _hkey
        return len(set(_hkey(x) for x in xs))
    if how == "first":
        return xs[0] if xs else NaN
    if how == "last":
        return xs[-1] if xs else NaN
    if how == "any":
        return any(xs)
    if how == "all":
        return all(xs)
    if how == "quantile":
        return quantile(xs, q)
    if how == "skew":
        return _skew([float(x) for x in xs])
    if how in ("idxmax", "idxmin", "argmax", "argmin"):
        raise ValueError(how)
    raise AttributeError("'%s' is not a valid function for aggregation" % how)


def _dt_mean(xs):
    if not xs:
        return NaT
    from datetime import timedelta
    base = xs[0]
    tot = sum(((x - base).total_seconds() for x in xs))
    return base + timedelta(seconds=tot / len(xs))


def func_name(f):
    if isinstance(f, str):
        return f
    n = getattr(f, "__name__", None)
    if n in ("amax", "max"):
        return "max"
    if n in ("amin", "min"):
        return "min"
    if n in ("sum", "mean", "median", "std", "var", "len", "count", "nunique", "prod"):
        return "size" if n == "len" else n
    return n or "<lambda>"


_BUILTINS = {"sum": "sum", "max": "max", "min": "min", "len": "size"}


def resolve(f):
    """A string how, or the callable itself."""
    if isinstance(f, str):
        return f
    n = getattr(f, "__name__", None)
    if f in (sum, max, min, len):
        return _BUILTINS[n]
    mod = getattr(f, "__module__", "") or ""
    if n in ("mean", "median", "std", "var", "sum", "max", "min", "prod", "count", "nunique") and ("numpy" in mod or "statistics" in mod):
        if "statistics" in mod and n == "stdev":
            return "std"
        return n
    if n == "stdev" and "statistics" in mod:
        return "std"
    return f


def result_dtype(how, dt):
    if how in ("count", "size", "nunique"):
        return "int64"
    if how in ("mean", "median", "std", "var", "sem", "quantile", "skew"):
        return "datetime64[ns]" if dt == "datetime64[ns]" and how in ("mean", "median") else "float64"
    if how == "sum" and dt == "bool":
        return "int64"
    return None
