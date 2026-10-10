# SPDX-License-Identifier: MIT
# time for the Rust runtime. The clock is the host's; the time zone is UTC.
import _time
from _time import time, time_ns, perf_counter, perf_counter_ns, monotonic, monotonic_ns, sleep

timezone = 0
altzone = 0
daylight = 0
tzname = ("UTC", "UTC")

CLOCK_REALTIME = 0
CLOCK_MONOTONIC = 1
CLOCK_PROCESS_CPUTIME_ID = 2
CLOCK_THREAD_CPUTIME_ID = 3

_FIELDS = ("tm_year", "tm_mon", "tm_mday", "tm_hour", "tm_min", "tm_sec", "tm_wday", "tm_yday", "tm_isdst")


class struct_time(tuple):
    n_sequence_fields = 9
    n_fields = 11
    n_unnamed_fields = 0
    __match_args__ = _FIELDS

    def __new__(cls, seq, dict=None):
        seq = tuple(seq)
        if len(seq) < 9:
            raise TypeError(f"time.struct_time() takes an at least 9-sequence ({len(seq)}-sequence given)")
        if len(seq) > 11:
            raise TypeError(f"time.struct_time() takes an at most 11-sequence ({len(seq)}-sequence given)")
        self = tuple.__new__(cls, seq[:9])
        self._extra = (seq[9] if len(seq) > 9 else (dict or {}).get("tm_zone"),
                       seq[10] if len(seq) > 10 else (dict or {}).get("tm_gmtoff"))
        return self

    tm_year = property(lambda self: self[0])
    tm_mon = property(lambda self: self[1])
    tm_mday = property(lambda self: self[2])
    tm_hour = property(lambda self: self[3])
    tm_min = property(lambda self: self[4])
    tm_sec = property(lambda self: self[5])
    tm_wday = property(lambda self: self[6])
    tm_yday = property(lambda self: self[7])
    tm_isdst = property(lambda self: self[8])
    tm_zone = property(lambda self: self._extra[0])
    tm_gmtoff = property(lambda self: self._extra[1])

    def __repr__(self):
        return "time.struct_time(" + ", ".join(f"{n}={v!r}" for n, v in zip(_FIELDS, self)) + ")"

    def __reduce__(self):
        return (struct_time, (tuple(self), {"tm_zone": self.tm_zone, "tm_gmtoff": self.tm_gmtoff}))


struct_time.__module__ = "time"


def _secs(secs):
    if secs is None:
        return time()
    if isinstance(secs, float) and secs != secs:
        raise ValueError("Invalid value NaN (not a number)")
    return secs


def gmtime(secs=None):
    t = _time.gmtime(_secs(secs))
    return struct_time(tuple(t[:8]) + (0, "UTC", 0))


def localtime(secs=None):
    return gmtime(secs)


def _tuple(t, name):
    if t is None:
        return tuple(gmtime())
    if not isinstance(t, tuple):
        raise TypeError(f"Tuple or struct_time argument required")
    if len(t) != 9:
        raise TypeError(f"{name}(): illegal time tuple argument")
    return tuple(t)


def mktime(t):
    t = _tuple(t, "mktime")
    y, m, d, H, M, S = t[:6]
    # days from the civil date, after Howard Hinnant's algorithm
    y -= m <= 2
    era = (y if y >= 0 else y - 399) // 400
    yoe = y - era * 400
    doy = (153 * (m + (-3 if m > 2 else 9)) + 2) // 5 + d - 1
    doe = yoe * 365 + yoe // 4 - yoe // 100 + doy
    days = era * 146097 + doe - 719468
    return float(days * 86400 + H * 3600 + M * 60 + S)


def strftime(format, t=None):
    if not isinstance(format, str):
        raise TypeError(f"strftime() argument 1 must be str, not {type(format).__name__}")
    t = _tuple(t, "strftime")
    if not 1 <= t[1] <= 12 and t[1] != 0:
        raise ValueError("month out of range")
    return _time.strftime(format, tuple(max(x, 1) if i in (1, 2) else x for i, x in enumerate(t)))


_DAYS = ("Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun")
_MONTHS = ("Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec")


def asctime(t=None):
    t = _tuple(t, "asctime")
    return "%s %s %2d %02d:%02d:%02d %d" % (_DAYS[t[6] % 7], _MONTHS[(t[1] - 1) % 12], t[2], t[3], t[4], t[5], t[0])


def ctime(secs=None):
    return asctime(localtime(secs))


def process_time():
    return perf_counter()


def process_time_ns():
    return perf_counter_ns()


def thread_time():
    return perf_counter()


def thread_time_ns():
    return perf_counter_ns()


def clock_gettime(clk_id):
    return time() if clk_id == CLOCK_REALTIME else monotonic()


def clock_gettime_ns(clk_id):
    return time_ns() if clk_id == CLOCK_REALTIME else monotonic_ns()


def get_clock_info(name):
    import types
    if name not in ("time", "monotonic", "perf_counter", "process_time", "thread_time"):
        raise ValueError("unknown clock")
    return types.SimpleNamespace(implementation=name, monotonic=name != "time", adjustable=name == "time", resolution=1e-09)


def tzset():
    pass


def strptime(string, format="%a %b %d %H:%M:%S %Y"):
    import _strptime
    return _strptime._strptime_time(string, format)
