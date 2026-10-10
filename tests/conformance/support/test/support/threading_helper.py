# SPDX-License-Identifier: MIT
import unittest
requires_working_threading = lambda *a, **k: (unittest.skip("no threads") if not (len(a) == 1 and callable(a[0])) else unittest.skip("no threads")(a[0]))
reap_threads = lambda f: f
def join_thread(thread, timeout=None): pass
def start_threads(threads, unlock=None):
    raise unittest.SkipTest("no threads")
