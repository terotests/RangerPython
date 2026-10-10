# SPDX-License-Identifier: MIT
# statistics for the Rust runtime. Sums are exact rationals, as in CPython,
# so results round the same way; rationals are (numerator, denominator) pairs.
import math
from math import gcd as _gcd, isqrt as _isqrt, fsum as _fsum
from collections import Counter as _Counter

__all__ = ["StatisticsError", "fmean", "geometric_mean", "harmonic_mean", "mean", "median",
           "median_grouped", "median_high", "median_low", "mode", "multimode", "pstdev",
           "pvariance", "quantiles", "stdev", "variance", "correlation", "covariance",
           "linear_regression"]


class StatisticsError(ValueError):
    pass


def _q(n, d=1):
    if d < 0:
        n, d = -n, -d
    g = _gcd(n, d)
    if g > 1:
        n //= g
        d //= g
    return (n, d)


def _qadd(a, b):
    return _q(a[0] * b[1] + b[0] * a[1], a[1] * b[1])


def _qsub(a, b):
    return _q(a[0] * b[1] - b[0] * a[1], a[1] * b[1])


def _qmul(a, b):
    return _q(a[0] * b[0], a[1] * b[1])


def _qdiv(a, b):
    return _q(a[0] * b[1], a[1] * b[0])


def _exact_ratio(x):
    try:
        return x.as_integer_ratio()
    except AttributeError:
        pass
    except (OverflowError, ValueError):
        return (x, None)
    try:
        return (x.numerator, x.denominator)
    except AttributeError:
        raise TypeError(f"can't convert type '{type(x).__name__}' to numerator/denominator") from None


def _coerce(T, S):
    if T is S:
        return T
    if S is int or S is bool:
        return T
    if T is int:
        return S
    if issubclass(S, T):
        return S
    if issubclass(T, S):
        return T
    if issubclass(T, int):
        return S
    if issubclass(S, int):
        return T
    if issubclass(T, float) or issubclass(S, float):
        return float
    raise TypeError(f"don't know how to coerce {T.__name__} and {S.__name__}")


def _sum(data):
    count = 0
    types = set()
    partials = {}
    special = None
    for x in data:
        types.add(type(x))
        n, d = _exact_ratio(x)
        count += 1
        if d is None:
            special = n if special is None else special + n
            continue
        partials[d] = partials.get(d, 0) + n
    T = int
    for t in types:
        T = _coerce(T, t)
    if special is not None:
        return (T, special, count)
    total = (0, 1)
    for d, n in partials.items():
        total = _qadd(total, _q(n, d))
    return (T, total, count)


def _convert(value, T):
    if not isinstance(value, tuple):
        return T(value)
    n, d = value
    if T is int or T is bool:
        if d == 1:
            return n
        T = float
    if T is float or issubclass(T, float):
        return T(n / d)
    try:
        return T(n) / T(d)
    except TypeError:
        return n / d


def _ss(data, c=None):
    if c is not None:
        cr = _exact_ratio(c)
        T, ssd, count = _sum((x - c) * (x - c) for x in data)
        return (T, ssd, c, count)
    count = 0
    types = set()
    sx = {}
    sxx = {}
    special = None
    for x in data:
        types.add(type(x))
        n, d = _exact_ratio(x)
        count += 1
        if d is None:
            special = n if special is None else special + n
            continue
        sx[d] = sx.get(d, 0) + n
        sxx[d] = sxx.get(d, 0) + n * n
    T = int
    for t in types:
        T = _coerce(T, t)
    if not count:
        return (T, (0, 1), (0, 1), 0)
    if special is not None:
        return (T, special, special, count)
    s1 = (0, 1)
    for d, n in sx.items():
        s1 = _qadd(s1, _q(n, d))
    s2 = (0, 1)
    for d, n in sxx.items():
        s2 = _qadd(s2, _q(n, d * d))
    ssd = _qdiv(_qsub(_qmul((count, 1), s2), _qmul(s1, s1)), (count, 1))
    return (T, ssd, _qdiv(s1, (count, 1)), count)


def _integer_sqrt_of_frac_rto(n, m):
    a = _isqrt(n // m)
    return a | (a * a * m != n)


_sqrt_bit_width = 2 * 53 + 3


def _float_sqrt_of_frac(n, m):
    q = (n.bit_length() - m.bit_length() - _sqrt_bit_width) // 2
    if q >= 0:
        numerator = _integer_sqrt_of_frac_rto(n, m << 2 * q) << q
        denominator = 1
    else:
        numerator = _integer_sqrt_of_frac_rto(n << -2 * q, m)
        denominator = 1 << -q
    return numerator / denominator


def mean(data):
    T, total, n = _sum(data)
    if n < 1:
        raise StatisticsError("mean requires at least one data point")
    if not isinstance(total, tuple):
        return _convert(total / n, T)
    return _convert(_qdiv(total, (n, 1)), T)


def fmean(data, weights=None):
    if weights is None:
        try:
            n = len(data)
        except TypeError:
            data = list(data)
            n = len(data)
        if not n:
            raise StatisticsError("fmean requires at least one data point")
        return _fsum(data) / n
    if not isinstance(weights, (list, tuple)):
        weights = list(weights)
    data = list(data)
    if len(data) != len(weights):
        raise StatisticsError("data and weights must be the same length")
    num = _fsum(x * w for x, w in zip(data, weights))
    den = _fsum(weights)
    if not den:
        raise StatisticsError("sum of weights must be non-zero")
    return num / den


def geometric_mean(data):
    n = 0
    found_zero = False

    def count_positive(iterable):
        nonlocal n, found_zero
        for n, x in enumerate(iterable, start=1):
            if x > 0.0 or math.isnan(x):
                yield x
            elif x == 0.0:
                found_zero = True
            else:
                raise StatisticsError("No negative inputs allowed", x)
    total = _fsum(map(math.log, count_positive(data)))
    if not n:
        raise StatisticsError("Must have a non-empty dataset")
    if math.isnan(total):
        return math.nan
    if found_zero:
        return math.nan if total == math.inf else 0.0
    return math.exp(total / n)


def harmonic_mean(data, weights=None):
    if iter(data) is data:
        data = list(data)
    errmsg = "harmonic mean does not support negative values"
    n = len(data)
    if n < 1:
        raise StatisticsError("harmonic_mean requires at least one data point")
    elif n == 1 and weights is None:
        x = data[0]
        if isinstance(x, (int, float)):
            if x < 0:
                raise StatisticsError(errmsg)
            return x
        raise TypeError("unsupported type")
    if weights is None:
        weights = [1] * n
    else:
        weights = list(weights)
        if len(weights) != n:
            raise StatisticsError("Number of weights does not match data size")
    for x in data:
        if x < 0:
            raise StatisticsError(errmsg)
    for w in weights:
        if w < 0:
            raise StatisticsError("Weighted harmonic mean does not support negative weights")
    if any(x == 0 for x in data):
        return 0
    T, wsum, _ = _sum(weights)
    T2, total, _ = _sum(w / x if w else 0 for w, x in zip(weights, data))
    if total == (0, 1):
        raise StatisticsError("Weighted sum must be non-zero")
    return _convert(_qdiv(wsum, total), _coerce(T, T2))


def median(data):
    data = sorted(data)
    n = len(data)
    if n == 0:
        raise StatisticsError("no median for empty data")
    if n % 2 == 1:
        return data[n // 2]
    i = n // 2
    return (data[i - 1] + data[i]) / 2


def median_low(data):
    data = sorted(data)
    n = len(data)
    if n == 0:
        raise StatisticsError("no median for empty data")
    if n % 2 == 1:
        return data[n // 2]
    return data[n // 2 - 1]


def median_high(data):
    data = sorted(data)
    n = len(data)
    if n == 0:
        raise StatisticsError("no median for empty data")
    return data[n // 2]


def median_grouped(data, interval=1.0):
    data = sorted(data)
    n = len(data)
    if not n:
        raise StatisticsError("no median for empty data")
    x = data[n // 2]
    i = 0
    while i < n and data[i] < x:
        i += 1
    j = i
    while j < n and data[j] == x:
        j += 1
    try:
        interval = float(interval)
        x = float(x)
    except ValueError:
        raise TypeError("Value cannot be converted to a float")
    L = x - interval / 2.0
    cf = i
    f = j - i
    return L + interval * (n / 2 - cf) / f


def mode(data):
    pairs = _Counter(iter(data)).most_common(1)
    try:
        return pairs[0][0]
    except IndexError:
        raise StatisticsError("no mode for empty data") from None


def multimode(data):
    counts = _Counter(iter(data))
    if not counts:
        return []
    maxcount = max(counts.values())
    return [value for value, count in counts.items() if count == maxcount]


def quantiles(data, *, n=4, method="exclusive"):
    if n < 1:
        raise StatisticsError("n must be at least 1")
    data = sorted(data)
    ld = len(data)
    if ld < 2:
        if ld == 1:
            return data * (n - 1)
        raise StatisticsError("must have at least one data point")
    if method == "inclusive":
        m = ld - 1
        result = []
        for i in range(1, n):
            j, delta = divmod(i * m, n)
            interpolated = (data[j] * (n - delta) + data[j + 1] * delta) / n
            result.append(interpolated)
        return result
    if method == "exclusive":
        m = ld + 1
        result = []
        for i in range(1, n):
            j = i * m // n
            j = 1 if j < 1 else ld - 1 if j > ld - 1 else j
            delta = i * m - j * n
            interpolated = (data[j - 1] * (n - delta) + data[j] * delta) / n
            result.append(interpolated)
        return result
    raise ValueError(f"Unknown method: {method!r}")


def variance(data, xbar=None):
    T, ss, c, n = _ss(data, xbar)
    if n < 2:
        raise StatisticsError("variance requires at least two data points")
    if not isinstance(ss, tuple):
        return _convert(ss / (n - 1), T)
    return _convert(_qdiv(ss, (n - 1, 1)), T)


def pvariance(data, mu=None):
    T, ss, c, n = _ss(data, mu)
    if n < 1:
        raise StatisticsError("pvariance requires at least one data point")
    if not isinstance(ss, tuple):
        return _convert(ss / n, T)
    return _convert(_qdiv(ss, (n, 1)), T)


def stdev(data, xbar=None):
    T, ss, c, n = _ss(data, xbar)
    if n < 2:
        raise StatisticsError("stdev requires at least two data points")
    if not isinstance(ss, tuple):
        return math.sqrt(ss / (n - 1))
    mss = _qdiv(ss, (n - 1, 1))
    return _float_sqrt_of_frac(mss[0], mss[1])


def pstdev(data, mu=None):
    T, ss, c, n = _ss(data, mu)
    if n < 1:
        raise StatisticsError("pstdev requires at least one data point")
    if not isinstance(ss, tuple):
        return math.sqrt(ss / n)
    mss = _qdiv(ss, (n, 1))
    return _float_sqrt_of_frac(mss[0], mss[1])


def covariance(x, y, /):
    n = len(x)
    if len(y) != n:
        raise StatisticsError("covariance requires that both inputs have same number of data points")
    if n < 2:
        raise StatisticsError("covariance requires at least two data points")
    xbar = _fsum(x) / n
    ybar = _fsum(y) / n
    sxy = _fsum((xi - xbar) * (yi - ybar) for xi, yi in zip(x, y))
    return sxy / (n - 1)


def correlation(x, y, /, *, method="linear"):
    n = len(x)
    if len(y) != n:
        raise StatisticsError("correlation requires that both inputs have same number of data points")
    if n < 2:
        raise StatisticsError("correlation requires at least two data points")
    xbar = _fsum(x) / n
    ybar = _fsum(y) / n
    sxy = _fsum((xi - xbar) * (yi - ybar) for xi, yi in zip(x, y))
    sxx = _fsum((d := xi - xbar) * d for xi in x)
    syy = _fsum((d := yi - ybar) * d for yi in y)
    try:
        return sxy / math.sqrt(sxx * syy)
    except ZeroDivisionError:
        raise StatisticsError("at least one of the inputs is constant")


class LinearRegression(tuple):
    def __new__(cls, slope, intercept):
        return tuple.__new__(cls, (slope, intercept))

    slope = property(lambda self: self[0])
    intercept = property(lambda self: self[1])

    def __repr__(self):
        return f"LinearRegression(slope={self[0]!r}, intercept={self[1]!r})"


def linear_regression(x, y, /, *, proportional=False):
    n = len(x)
    if len(y) != n:
        raise StatisticsError("linear regression requires that both inputs have same number of data points")
    if n < 2:
        raise StatisticsError("linear regression requires at least two data points")
    if not proportional:
        xbar = _fsum(x) / n
        ybar = _fsum(y) / n
        x = [xi - xbar for xi in x]
        y = [yi - ybar for yi in y]
    sxy = _fsum(xi * yi for xi, yi in zip(x, y))
    sxx = _fsum(xi * xi for xi in x)
    try:
        slope = sxy / sxx
    except ZeroDivisionError:
        raise StatisticsError("x is constant")
    intercept = 0.0 if proportional else ybar - slope * xbar
    return LinearRegression(slope=slope, intercept=intercept)
