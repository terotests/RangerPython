# SPDX-License-Identifier: MIT
# contextlib for the Ranger Python runtime.
import sys


class AbstractContextManager:
    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_value, tb):
        return None


class AbstractAsyncContextManager:
    async def __aenter__(self):
        return self

    async def __aexit__(self, exc_type, exc_value, tb):
        return None


class ContextDecorator:
    def __call__(self, func):
        def inner(*args, **kwds):
            with self:
                return func(*args, **kwds)
        return inner


class suppress(AbstractContextManager):
    def __init__(self, *exceptions):
        self._exceptions = exceptions

    def __exit__(self, exc_type, exc_value, tb):
        return exc_type is not None and issubclass(exc_type, self._exceptions)


class nullcontext(AbstractContextManager):
    def __init__(self, enter_result=None):
        self.enter_result = enter_result

    def __enter__(self):
        return self.enter_result

    def __exit__(self, *exc):
        return None


class closing(AbstractContextManager):
    def __init__(self, thing):
        self.thing = thing

    def __enter__(self):
        return self.thing

    def __exit__(self, *exc):
        self.thing.close()


class _Redirect(AbstractContextManager):
    _stream = "stdout"

    def __init__(self, new_target):
        self._new_target = new_target
        self._old = []

    def __enter__(self):
        self._old.append(getattr(sys, self._stream))
        setattr(sys, self._stream, self._new_target)
        return self._new_target

    def __exit__(self, *exc):
        setattr(sys, self._stream, self._old.pop())


class redirect_stdout(_Redirect):
    _stream = "stdout"


class redirect_stderr(_Redirect):
    _stream = "stderr"


class ExitStack(AbstractContextManager):
    def __init__(self):
        self._callbacks = []

    def enter_context(self, cm):
        result = cm.__enter__()
        self._callbacks.append(cm.__exit__)
        return result

    def callback(self, fn, *args, **kwds):
        self._callbacks.append(lambda *exc: fn(*args, **kwds))
        return fn

    def push(self, exit_fn):
        self._callbacks.append(exit_fn)
        return exit_fn

    def close(self):
        self.__exit__(None, None, None)

    def __exit__(self, *exc):
        suppressed = False
        while self._callbacks:
            cb = self._callbacks.pop()
            if cb(*exc):
                suppressed = True
                exc = (None, None, None)
        return suppressed


class _GeneratorContextManager(ContextDecorator, AbstractContextManager):
    def __init__(self, func, args, kwds):
        self.gen = func(*args, **kwds)
        self.func, self.args, self.kwds = func, args, kwds
        self.__doc__ = getattr(func, "__doc__", None)

    def _recreate_cm(self):
        return self.__class__(self.func, self.args, self.kwds)

    def __call__(self, func):
        def inner(*args, **kwds):
            with self._recreate_cm():
                return func(*args, **kwds)
        return inner

    def __enter__(self):
        del self.args, self.kwds, self.func
        try:
            return next(self.gen)
        except StopIteration:
            raise RuntimeError("generator didn't yield") from None

    def __exit__(self, typ, value, traceback):
        if typ is None:
            try:
                next(self.gen)
            except StopIteration:
                return False
            else:
                try:
                    raise RuntimeError("generator didn't stop")
                finally:
                    self.gen.close()
        else:
            if value is None:
                value = typ()
            try:
                self.gen.throw(value)
            except StopIteration as exc:
                return exc is not value
            except RuntimeError as exc:
                if exc is value:
                    return False
                if isinstance(value, StopIteration) and exc.__cause__ is value:
                    return False
                raise
            except BaseException as exc:
                if exc is not value:
                    raise
                return False
            try:
                raise RuntimeError("generator didn't stop after throw()")
            finally:
                self.gen.close()


def contextmanager(func):
    from functools import wraps

    @wraps(func)
    def helper(*args, **kwds):
        return _GeneratorContextManager(func, args, kwds)
    return helper
