# SPDX-License-Identifier: MIT
# CPython's test.support.threading_helper, where threads can start (the
# native _thread); elsewhere (wasm32) the tests that need them are skipped.
import contextlib
import functools
import sys
import time
import unittest

from test import support

try:
    from _thread import start_new_thread as _start_new_thread
    import _thread
    import threading
    can_start_thread = True
except ImportError:
    can_start_thread = False


def requires_working_threading(*, module=False):
    msg = "no threads"
    if module:
        if not can_start_thread:
            raise unittest.SkipTest(msg)
    else:
        return unittest.skipUnless(can_start_thread, msg)


def threading_setup():
    if not can_start_thread:
        return 0, 0
    return _thread._count(), len(threading._dangling)


def threading_cleanup(*original_values):
    if not can_start_thread:
        return
    orig_count, orig_ndangling = original_values
    timeout = 1.0
    for _ in support.sleeping_retry(timeout, error=False):
        dangling_threads = list(threading._dangling)
        count = _thread._count()
        if count <= orig_count:
            return
    support.environment_altered = True
    support.print_warning(
        f"threading_cleanup() failed to clean up threads "
        f"in {timeout:.1f} seconds\n"
        f"  before: thread count={orig_count}, dangling={orig_ndangling}\n"
        f"  after: thread count={count}, dangling={len(dangling_threads)}")
    for thread in dangling_threads:
        support.print_warning(f"Dangling thread: {thread!r}")


def reap_threads(func):
    @functools.wraps(func)
    def decorator(*args):
        key = threading_setup()
        try:
            return func(*args)
        finally:
            threading_cleanup(*key)
    return decorator


@contextlib.contextmanager
def wait_threads_exit(timeout=None):
    if not can_start_thread:
        raise unittest.SkipTest("no threads")
    if timeout is None:
        timeout = support.SHORT_TIMEOUT
    old_count = _thread._count()
    try:
        yield
    finally:
        start_time = time.monotonic()
        for _ in support.sleeping_retry(timeout, error=False):
            support.gc_collect()
            count = _thread._count()
            if count <= old_count:
                break
        else:
            dt = time.monotonic() - start_time
            msg = (f"wait_threads() failed to cleanup {count - old_count} "
                   f"threads after {dt:.1f} seconds "
                   f"(count: {count}, old count: {old_count})")
            raise AssertionError(msg)


def join_thread(thread, timeout=None):
    if timeout is None:
        timeout = support.SHORT_TIMEOUT
    thread.join(timeout)
    if thread.is_alive():
        msg = f"failed to join the thread in {timeout:.1f} seconds"
        raise AssertionError(msg)


@contextlib.contextmanager
def start_threads(threads, unlock=None):
    if not can_start_thread:
        raise unittest.SkipTest("no threads")
    threads = list(threads)
    started = []
    try:
        try:
            for t in threads:
                t.start()
                started.append(t)
        except:
            if support.verbose:
                print("Can't start %d threads, only %d threads started" %
                      (len(threads), len(started)))
            raise
        yield
    finally:
        try:
            if unlock:
                unlock()
            endtime = time.monotonic()
            for timeout in range(1, 16):
                endtime += 60
                for t in started:
                    t.join(max(endtime - time.monotonic(), 0.01))
                started = [t for t in started if t.is_alive()]
                if not started:
                    break
                if support.verbose:
                    print('Unable to join %d threads during a period of '
                          '%d minutes' % (len(started), timeout))
        finally:
            started = [t for t in started if t.is_alive()]
            if started:
                raise AssertionError('Unable to join %d threads' % len(started))


class catch_threading_exception:
    def __init__(self):
        self.exc_type = None
        self.exc_value = None
        self.exc_traceback = None
        self.thread = None
        self._old_hook = None

    def _hook(self, args):
        self.exc_type = args.exc_type
        self.exc_value = args.exc_value
        self.exc_traceback = args.exc_traceback
        self.thread = args.thread

    def __enter__(self):
        self._old_hook = threading.excepthook
        threading.excepthook = self._hook
        return self

    def __exit__(self, *exc_info):
        threading.excepthook = self._old_hook
        del self.exc_type
        del self.exc_value
        del self.exc_traceback
        del self.thread
