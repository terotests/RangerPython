# SPDX-License-Identifier: MIT
# gc for the Ranger Python runtime: memory is the host's, so these do nothing.
_enabled = True
def collect(generation=2): return 0
def enable():
    global _enabled
    _enabled = True
def disable():
    global _enabled
    _enabled = False
def isenabled(): return _enabled
def get_referrers(*objs): return []
def get_referents(*objs): return []
def is_tracked(obj): return False
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
def get_count(): return (0, 0, 0)
def get_debug(): return _debug
def set_debug(flags):
    global _debug
    _debug = flags
def get_objects(generation=None): return []
def get_stats(): return [{"collections": 0, "collected": 0, "uncollectable": 0} for _ in range(3)]
def get_freeze_count(): return 0
def unfreeze(): pass
def is_finalized(obj): return False
