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
