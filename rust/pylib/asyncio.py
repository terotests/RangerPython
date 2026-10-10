# SPDX-License-Identifier: MIT
# A small asyncio for the Rust runtime: one event loop that runs coroutines
# to completion on virtual time (sleep advances a clock, nothing waits).
import heapq as _heapq

__all__ = ["run", "sleep", "gather", "create_task", "get_event_loop", "get_running_loop",
           "new_event_loop", "set_event_loop", "wait_for", "wait", "as_completed", "Future", "Task",
           "Lock", "Event", "Semaphore", "Queue", "QueueEmpty", "QueueFull", "CancelledError",
           "TimeoutError", "InvalidStateError", "iscoroutine", "iscoroutinefunction", "ensure_future",
           "shield", "timeout", "FIRST_COMPLETED", "FIRST_EXCEPTION", "ALL_COMPLETED"]

FIRST_COMPLETED = "FIRST_COMPLETED"
FIRST_EXCEPTION = "FIRST_EXCEPTION"
ALL_COMPLETED = "ALL_COMPLETED"


class CancelledError(BaseException):
    pass


TimeoutError = TimeoutError


class InvalidStateError(Exception):
    pass


class QueueEmpty(Exception):
    pass


class QueueFull(Exception):
    pass


_running = None
_current = None


class _Wake:
    """yielded by sleep(): resume the task at `when`"""
    __slots__ = ("when",)

    def __init__(self, when):
        self.when = when


class AbstractEventLoop:
    pass


class _Loop(AbstractEventLoop):
    def __init__(self):
        self._ready = []
        self._timers = []
        self._seq = 0
        self._time = 0.0
        self._closed = False
        self._running = False

    def time(self):
        return self._time

    def call_soon(self, cb, *args, context=None):
        h = _Handle(cb, args)
        self._ready.append(h)
        return h

    def call_later(self, delay, cb, *args, context=None):
        return self.call_at(self._time + max(delay, 0), cb, *args)

    def call_at(self, when, cb, *args, context=None):
        h = _Handle(cb, args)
        self._seq += 1
        _heapq.heappush(self._timers, (when, self._seq, h))
        return h

    def create_future(self):
        return Future(loop=self)

    def create_task(self, coro, *, name=None, context=None):
        return Task(coro, loop=self, name=name)

    def is_running(self):
        return self._running

    def is_closed(self):
        return self._closed

    def close(self):
        self._closed = True

    def _run_once(self):
        if not self._ready and self._timers:
            when, _, h = _heapq.heappop(self._timers)
            if when > self._time:
                self._time = when
            self._ready.append(h)
            while self._timers and self._timers[0][0] <= self._time:
                self._ready.append(_heapq.heappop(self._timers)[2])
        ready, self._ready = self._ready, []
        for h in ready:
            if not h._cancelled:
                h._cb(*h._args)

    def run_until_complete(self, fut):
        global _running
        fut = ensure_future(fut, loop=self)
        prev = _running
        _running = self
        self._running = True
        try:
            while not fut.done():
                if not self._ready and not self._timers:
                    raise RuntimeError("Event loop stopped before Future completed.")
                self._run_once()
        finally:
            self._running = False
            _running = prev
        return fut.result()

    def run_forever(self):
        while self._ready or self._timers:
            self._run_once()

    def stop(self):
        pass

    async def shutdown_asyncgens(self):
        pass

    async def shutdown_default_executor(self, timeout=None):
        pass


class _Handle:
    __slots__ = ("_cb", "_args", "_cancelled")

    def __init__(self, cb, args):
        self._cb = cb
        self._args = args
        self._cancelled = False

    def cancel(self):
        self._cancelled = True

    def cancelled(self):
        return self._cancelled


class Future:
    _asyncio_future_blocking = False

    def __init__(self, *, loop=None):
        self._loop = loop if loop is not None else get_event_loop()
        self._state = "PENDING"
        self._result = None
        self._exception = None
        self._callbacks = []

    def get_loop(self):
        return self._loop

    def done(self):
        return self._state != "PENDING"

    def cancelled(self):
        return self._state == "CANCELLED"

    def cancel(self, msg=None):
        if self._state != "PENDING":
            return False
        self._state = "CANCELLED"
        self._exception = CancelledError() if msg is None else CancelledError(msg)
        self._schedule()
        return True

    def result(self):
        if self._state == "CANCELLED":
            raise self._exception
        if self._state != "FINISHED":
            raise InvalidStateError("Result is not ready.")
        if self._exception is not None:
            raise self._exception
        return self._result

    def exception(self):
        if self._state == "CANCELLED":
            raise self._exception
        if self._state != "FINISHED":
            raise InvalidStateError("Exception is not set.")
        return self._exception

    def set_result(self, result):
        if self._state != "PENDING":
            raise InvalidStateError("invalid state")
        self._result = result
        self._state = "FINISHED"
        self._schedule()

    def set_exception(self, exception):
        if self._state != "PENDING":
            raise InvalidStateError("invalid state")
        if isinstance(exception, type):
            exception = exception()
        self._exception = exception
        self._state = "FINISHED"
        self._schedule()

    def add_done_callback(self, fn, *, context=None):
        if self.done():
            self._loop.call_soon(fn, self)
        else:
            self._callbacks.append(fn)

    def remove_done_callback(self, fn):
        n = len(self._callbacks)
        self._callbacks = [f for f in self._callbacks if f != fn]
        return n - len(self._callbacks)

    def _schedule(self):
        cbs, self._callbacks = self._callbacks, []
        for fn in cbs:
            self._loop.call_soon(fn, self)

    def __await__(self):
        if not self.done():
            yield self
        return self.result()

    __iter__ = __await__

    def __repr__(self):
        return f"<{type(self).__name__} {self._state.lower()}>"


class Task(Future):
    _counter = 0

    def __init__(self, coro, *, loop=None, name=None, context=None):
        super().__init__(loop=loop)
        if not iscoroutine(coro):
            raise TypeError(f"a coroutine was expected, got {coro!r}")
        self._coro = coro
        Task._counter += 1
        self._name = name if name is not None else f"Task-{Task._counter}"
        self._must_cancel = False
        self._waiting = None
        self._loop.call_soon(self._step)

    def get_coro(self):
        return self._coro

    def get_name(self):
        return self._name

    def set_name(self, value):
        self._name = str(value)

    def cancel(self, msg=None):
        if self.done():
            return False
        if self._waiting is not None and self._waiting.cancel(msg):
            return True
        self._must_cancel = True
        return True

    def _step(self, exc=None):
        global _current
        if self.done():
            return
        if self._must_cancel:
            exc = CancelledError()
            self._must_cancel = False
        prev = _current
        _current = self
        self._waiting = None
        try:
            if exc is None:
                y = self._coro.send(None)
            else:
                y = self._coro.throw(exc)
        except StopIteration as e:
            Future.set_result(self, e.value)
        except CancelledError as e:
            Future.cancel(self)
        except BaseException as e:
            Future.set_exception(self, e)
        else:
            if isinstance(y, Future):
                self._waiting = y
                y.add_done_callback(self._wakeup)
            elif isinstance(y, _Wake):
                self._loop.call_at(y.when, self._step)
            else:
                self._loop.call_soon(self._step)
        finally:
            _current = prev

    def _wakeup(self, fut):
        self._step()

    def __repr__(self):
        return f"<Task {self._state.lower()} name={self._name!r}>"


class _SleepAwaitable:
    __slots__ = ("delay",)

    def __init__(self, delay):
        self.delay = delay

    def __await__(self):
        if self.delay <= 0:
            yield None
        else:
            yield _Wake(get_running_loop().time() + self.delay)


async def sleep(delay, result=None):
    await _SleepAwaitable(delay)
    return result


def iscoroutine(obj):
    return type(obj).__name__ == "coroutine" or (hasattr(obj, "send") and hasattr(obj, "throw"))


def iscoroutinefunction(func):
    code = getattr(func, "__code__", None)
    return code is not None and bool(getattr(code, "co_flags", 0) & 0x80)


def ensure_future(obj, *, loop=None):
    if isinstance(obj, Future):
        return obj
    if iscoroutine(obj):
        return (loop or get_event_loop()).create_task(obj)
    if hasattr(obj, "__await__"):
        async def _wrap():
            return await obj
        return (loop or get_event_loop()).create_task(_wrap())
    raise TypeError("An asyncio.Future, a coroutine or an awaitable is required")


def get_running_loop():
    if _running is None:
        raise RuntimeError("no running event loop")
    return _running


_default = None


def get_event_loop():
    global _default
    if _running is not None:
        return _running
    if _default is None or _default.is_closed():
        _default = _Loop()
    return _default


def new_event_loop():
    return _Loop()


def set_event_loop(loop):
    global _default
    _default = loop


class _Policy:
    def get_event_loop(self):
        return get_event_loop()

    def new_event_loop(self):
        return new_event_loop()

    def set_event_loop(self, loop):
        set_event_loop(loop)


_policy = _Policy()


def get_event_loop_policy():
    return _policy


def set_event_loop_policy(policy):
    global _policy
    _policy = policy if policy is not None else _Policy()


def current_task(loop=None):
    return _current


def all_tasks(loop=None):
    return set()


def create_task(coro, *, name=None, context=None):
    return get_running_loop().create_task(coro, name=name)


def run(main, *, debug=None, loop_factory=None):
    if _running is not None:
        raise RuntimeError("asyncio.run() cannot be called from a running event loop")
    if not iscoroutine(main):
        raise ValueError(f"a coroutine was expected, got {main!r}")
    loop = new_event_loop() if loop_factory is None else loop_factory()
    try:
        return loop.run_until_complete(main)
    finally:
        loop.close()


def gather(*aws, return_exceptions=False):
    loop = get_event_loop()
    outer = loop.create_future()
    futs = [ensure_future(a, loop=loop) for a in aws]
    if not futs:
        outer.set_result([])
        return outer
    left = [len(futs)]

    def done(f):
        if outer.done():
            return
        if not return_exceptions:
            if f.cancelled():
                outer.set_exception(CancelledError())
                return
            if f.exception() is not None:
                outer.set_exception(f.exception())
                return
        left[0] -= 1
        if left[0] == 0:
            res = []
            for g in futs:
                if g.cancelled():
                    res.append(CancelledError())
                elif g.exception() is not None:
                    res.append(g.exception())
                else:
                    res.append(g.result())
            outer.set_result(res)
    for f in futs:
        f.add_done_callback(done)
    return outer


def shield(arg):
    return ensure_future(arg)


async def wait_for(fut, timeout):
    fut = ensure_future(fut)
    if timeout is None:
        return await fut
    loop = get_running_loop()
    waiter = loop.create_future()
    h = loop.call_later(timeout, lambda: waiter.done() or waiter.set_result(None))
    fut.add_done_callback(lambda f: waiter.done() or waiter.set_result(None))
    await waiter
    h.cancel()
    if not fut.done():
        fut.cancel()
        raise TimeoutError()
    return fut.result()


async def wait(fs, *, timeout=None, return_when=ALL_COMPLETED):
    fs = {ensure_future(f) for f in fs}
    if not fs:
        raise ValueError("Set of Tasks/Futures is empty.")
    loop = get_running_loop()
    waiter = loop.create_future()

    def check(_=None):
        if waiter.done():
            return
        done = [f for f in fs if f.done()]
        if return_when == FIRST_COMPLETED and done:
            waiter.set_result(None)
        elif return_when == FIRST_EXCEPTION and any(not f.cancelled() and f.exception() is not None for f in done):
            waiter.set_result(None)
        elif len(done) == len(fs):
            waiter.set_result(None)
    for f in fs:
        f.add_done_callback(check)
    if timeout is not None:
        loop.call_later(timeout, lambda: waiter.done() or waiter.set_result(None))
    check()
    await waiter
    done = {f for f in fs if f.done()}
    return done, fs - done


def as_completed(fs, *, timeout=None):
    fs = [ensure_future(f) for f in fs]
    pending = list(fs)

    async def _next():
        done, _ = await wait(pending, return_when=FIRST_COMPLETED)
        f = next(f for f in pending if f in done)
        pending.remove(f)
        return f.result()
    for _ in range(len(fs)):
        yield _next()


class timeout:
    def __init__(self, delay):
        self._delay = delay

    async def __aenter__(self):
        return self

    async def __aexit__(self, et, e, tb):
        return False


class Lock:
    def __init__(self):
        self._locked = False
        self._waiters = []

    def locked(self):
        return self._locked

    async def acquire(self):
        while self._locked:
            f = get_running_loop().create_future()
            self._waiters.append(f)
            await f
        self._locked = True
        return True

    def release(self):
        if not self._locked:
            raise RuntimeError("Lock is not acquired.")
        self._locked = False
        if self._waiters:
            f = self._waiters.pop(0)
            if not f.done():
                f.set_result(None)

    async def __aenter__(self):
        await self.acquire()
        return None

    async def __aexit__(self, et, e, tb):
        self.release()


class Event:
    def __init__(self):
        self._set = False
        self._waiters = []

    def is_set(self):
        return self._set

    def set(self):
        self._set = True
        for f in self._waiters:
            if not f.done():
                f.set_result(True)
        self._waiters = []

    def clear(self):
        self._set = False

    async def wait(self):
        if self._set:
            return True
        f = get_running_loop().create_future()
        self._waiters.append(f)
        await f
        return True


class Semaphore:
    def __init__(self, value=1):
        if value < 0:
            raise ValueError("Semaphore initial value must be >= 0")
        self._value = value
        self._waiters = []

    def locked(self):
        return self._value == 0

    async def acquire(self):
        while self._value <= 0:
            f = get_running_loop().create_future()
            self._waiters.append(f)
            await f
        self._value -= 1
        return True

    def release(self):
        self._value += 1
        if self._waiters:
            f = self._waiters.pop(0)
            if not f.done():
                f.set_result(None)

    async def __aenter__(self):
        await self.acquire()

    async def __aexit__(self, et, e, tb):
        self.release()


BoundedSemaphore = Semaphore


class Queue:
    def __init__(self, maxsize=0):
        self._maxsize = maxsize
        self._items = []
        self._getters = []
        self._putters = []
        self._unfinished = 0
        self._finished = Event()
        self._finished.set()

    def qsize(self):
        return len(self._items)

    @property
    def maxsize(self):
        return self._maxsize

    def empty(self):
        return not self._items

    def full(self):
        return 0 < self._maxsize <= len(self._items)

    def _wake(self, waiters):
        while waiters:
            f = waiters.pop(0)
            if not f.done():
                f.set_result(None)
                break

    async def put(self, item):
        while self.full():
            f = get_running_loop().create_future()
            self._putters.append(f)
            await f
        self.put_nowait(item)

    def put_nowait(self, item):
        if self.full():
            raise QueueFull
        self._items.append(item)
        self._unfinished += 1
        self._finished.clear()
        self._wake(self._getters)

    async def get(self):
        while self.empty():
            f = get_running_loop().create_future()
            self._getters.append(f)
            await f
        return self.get_nowait()

    def get_nowait(self):
        if self.empty():
            raise QueueEmpty
        item = self._items.pop(0)
        self._wake(self._putters)
        return item

    def task_done(self):
        if self._unfinished <= 0:
            raise ValueError("task_done() called too many times")
        self._unfinished -= 1
        if self._unfinished == 0:
            self._finished.set()

    async def join(self):
        if self._unfinished > 0:
            await self._finished.wait()
