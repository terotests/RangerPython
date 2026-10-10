# SPDX-License-Identifier: MIT
# gc for the Ranger Python runtime: memory is reference counted, and
# collect() finds and frees the objects caught in reference cycles (see
# rust/src/gc.rs); is_tracked answers as CPython's collector would.
import _rpygc
_enabled = True


def collect(generation=2):
    if not isinstance(generation, int):
        raise TypeError("'%s' object cannot be interpreted as an integer" % type(generation).__name__)
    if not 0 <= generation <= 2:
        raise ValueError("invalid generation")
    for cb in callbacks:
        cb("start", {"generation": generation, "collected": 0, "uncollectable": 0})
    n = _rpygc.collect()
    for cb in callbacks:
        cb("stop", {"generation": generation, "collected": n, "uncollectable": 0})
    return n


def enable():
    global _enabled
    _enabled = True
    _rpygc.auto(_enabled, _threshold[0])


def disable():
    global _enabled
    _enabled = False
    _rpygc.auto(_enabled, _threshold[0])


def isenabled(): return _enabled
def get_referrers(*objs): return []
def get_referents(*objs): return []
from _rpygc import is_tracked
def freeze(): pass
garbage = []
callbacks = []
DEBUG_STATS = 1
DEBUG_COLLECTABLE = 2
DEBUG_UNCOLLECTABLE = 4
DEBUG_SAVEALL = 32
DEBUG_LEAK = 38
_threshold = (2000, 10, 0)
_debug = 0
def get_threshold(): return _threshold
def set_threshold(threshold0, threshold1=None, threshold2=None):
    global _threshold
    t1 = _threshold[1] if threshold1 is None else threshold1
    t2 = _threshold[2] if threshold2 is None else threshold2
    _threshold = (threshold0, t1, t2)
    _rpygc.auto(_enabled, threshold0)
def get_count(): return (0, 0, 0)
def get_debug(): return _debug
def set_debug(flags):
    global _debug
    _debug = flags
def get_objects(generation=None): return []
def get_stats():
    runs, found = _rpygc.collections()
    return [{"collections": 0, "collected": 0, "uncollectable": 0},
            {"collections": 0, "collected": 0, "uncollectable": 0},
            {"collections": runs, "collected": found, "uncollectable": 0}]
def get_freeze_count(): return 0
def unfreeze(): pass
from _rpygc import is_finalized
