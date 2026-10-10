# SPDX-License-Identifier: MIT
# threading for the Ranger Python runtime. There is one thread: start()
# runs the target to the end, and locks never have to wait.

_ident = [1]


class Lock:
    def __init__(self):
        self._locked = False

    def acquire(self, blocking=True, timeout=-1):
        if self._locked:
            if not blocking:
                return False
            raise RuntimeError("deadlock: the lock is held and there is only one thread")
        self._locked = True
        return True

    def release(self):
        if not self._locked:
            raise RuntimeError("release unlocked lock")
        self._locked = False

    def locked(self):
        return self._locked

    def __enter__(self):
        return self.acquire()

    def __exit__(self, *exc):
        self.release()


class RLock:
    def __init__(self):
        self._count = 0

    def acquire(self, blocking=True, timeout=-1):
        self._count += 1
        return True

    def release(self):
        if not self._count:
            raise RuntimeError("cannot release un-acquired lock")
        self._count -= 1

    def __enter__(self):
        return self.acquire()

    def __exit__(self, *exc):
        self.release()


class Condition:
    def __init__(self, lock=None):
        self._lock = lock or RLock()

    def __enter__(self):
        return self._lock.__enter__()

    def __exit__(self, *exc):
        return self._lock.__exit__(*exc)

    def wait(self, timeout=None):
        return True

    def wait_for(self, predicate, timeout=None):
        return predicate()

    def notify(self, n=1):
        pass

    def notify_all(self):
        pass


class Semaphore:
    def __init__(self, value=1):
        self._value = value

    def acquire(self, blocking=True, timeout=None):
        if self._value <= 0:
            return False
        self._value -= 1
        return True

    def release(self, n=1):
        self._value += n

    __enter__ = acquire

    def __exit__(self, *exc):
        self.release()


BoundedSemaphore = Semaphore


class Event:
    def __init__(self):
        self._flag = False

    def is_set(self):
        return self._flag

    def set(self):
        self._flag = True

    def clear(self):
        self._flag = False

    def wait(self, timeout=None):
        return self._flag


class local:
    pass


class Thread:
    def __init__(self, group=None, target=None, name=None, args=(), kwargs=None, *, daemon=None):
        self._target = target
        self._args = args
        self._kwargs = kwargs or {}
        _ident[0] += 1
        self.name = name or "Thread-%d" % (_ident[0] - 1)
        self.ident = None
        self.daemon = bool(daemon)
        self._started = False
        self._done = False

    def start(self):
        if self._started:
            raise RuntimeError("threads can only be started once")
        self._started = True
        self.ident = _ident[0]
        try:
            self.run()
        finally:
            self._done = True

    def run(self):
        if self._target is not None:
            self._target(*self._args, **self._kwargs)

    def join(self, timeout=None):
        pass

    def is_alive(self):
        return self._started and not self._done


class _MainThread(Thread):
    def __init__(self):
        Thread.__init__(self, name="MainThread")
        self.ident = 1
        self._started = True


_main = _MainThread()


def current_thread():
    return _main


def main_thread():
    return _main


def get_ident():
    return 1


def active_count():
    return 1


def enumerate():
    return [_main]


TIMEOUT_MAX = 2147483.0
