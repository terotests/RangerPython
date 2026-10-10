# SPDX-License-Identifier: MIT
# Asynchronous generators for the Rust runtime. Calling an `async def` that
# yields runs its body as a plain generator; the VM hands that generator to
# async_generator() here. A `yield` in the body and an `await` both suspend
# the generator: an await suspends it inside its yield-from (gi_yieldfrom is
# set) and passes on to whoever drives the awaitable, while a yield finishes
# the pending __anext__()/asend()/athrow() with the value.

_INIT, _ITER, _CLOSED = range(3)


class async_generator:
    __slots__ = ("_gen", "_running", "_finalizer", "__weakref__")

    def __init__(self, gen):
        self._gen = gen
        self._running = False
        self._finalizer = None

    @property
    def __name__(self):
        return self._gen.__name__

    @property
    def __qualname__(self):
        return self._gen.__qualname__

    @property
    def ag_code(self):
        return self._gen.gi_code

    @property
    def ag_frame(self):
        return self._gen.gi_frame

    @property
    def ag_await(self):
        return self._gen.gi_yieldfrom

    @property
    def ag_running(self):
        return self._running

    @property
    def ag_suspended(self):
        return self._gen.gi_suspended

    def __repr__(self):
        return f"<async_generator object {self.__qualname__} at {id(self):#x}>"

    def __aiter__(self):
        return self

    def __anext__(self):
        return _ASend(self, None)

    def asend(self, value):
        return _ASend(self, value)

    def athrow(self, typ, val=None, tb=None):
        return _AThrow(self, (typ, val, tb))

    def aclose(self):
        return _AThrow(self, None)

    def __class_getitem__(cls, item):
        import types
        return types.GenericAlias(cls, item)


class _Awaitable:
    __slots__ = ("_agen", "_state")

    def __await__(self):
        return self

    def __iter__(self):
        return self

    def __next__(self):
        return self.send(None)

    def close(self):
        self._state = _CLOSED

    def _step(self, resume):
        # runs the generator to its next suspension; StopIteration carries a
        # value it yielded, a yield of an await goes up to the caller
        agen = self._agen
        agen._running = True
        try:
            r = resume()
        except StopIteration:
            self._state = _CLOSED
            agen._running = False
            raise StopAsyncIteration from None
        except BaseException:
            self._state = _CLOSED
            agen._running = False
            raise
        if agen._gen.gi_yieldfrom is not None:
            return r
        self._state = _CLOSED
        agen._running = False
        raise StopIteration(r)


class _ASend(_Awaitable):
    """What __anext__() and asend() return."""
    __slots__ = ("_value",)

    def __init__(self, agen, value):
        self._agen = agen
        self._value = value
        self._state = _INIT

    def send(self, value):
        if self._state == _CLOSED:
            raise RuntimeError("cannot reuse already awaited __anext__()/asend()")
        if self._state == _INIT:
            if self._agen._running:
                raise RuntimeError("anext(): asynchronous generator is already running")
            if value is None:
                value = self._value
            self._state = _ITER
        return self._step(lambda: self._agen._gen.send(value))

    def throw(self, typ, val=None, tb=None):
        if self._state == _CLOSED:
            raise RuntimeError("cannot reuse already awaited __anext__()/asend()")
        self._state = _ITER
        return self._step(lambda: _throw(self._agen._gen, typ, val, tb))


class _AThrow(_Awaitable):
    """What athrow() and aclose() (args None) return."""
    __slots__ = ("_args",)

    def __init__(self, agen, args):
        self._agen = agen
        self._args = args
        self._state = _INIT

    def send(self, value):
        gen = self._agen._gen
        if self._state == _CLOSED:
            raise RuntimeError("cannot reuse already awaited aclose()/athrow()")
        if self._state == _INIT:
            if self._agen._running:
                raise RuntimeError("athrow(): asynchronous generator is already running")
            self._state = _ITER
            if self._args is None:
                return self._closing(lambda: gen.throw(GeneratorExit))
            typ, val, tb = self._args
            return self._step(lambda: _throw(gen, typ, val, tb))
        if self._args is None:
            return self._closing(lambda: gen.send(value))
        return self._step(lambda: gen.send(value))

    def throw(self, typ, val=None, tb=None):
        gen = self._agen._gen
        if self._state == _CLOSED:
            raise RuntimeError("cannot reuse already awaited aclose()/athrow()")
        self._state = _ITER
        resume = lambda: _throw(gen, typ, val, tb)
        if self._args is None:
            return self._closing(resume)
        return self._step(resume)

    def _closing(self, resume):
        # aclose(): the generator must finish, not yield a value
        try:
            r = resume()
        except (StopIteration, StopAsyncIteration, GeneratorExit):
            self._state = _CLOSED
            raise StopIteration from None
        except BaseException:
            self._state = _CLOSED
            raise
        if self._agen._gen.gi_yieldfrom is not None:
            return r
        self._state = _CLOSED
        raise RuntimeError("async generator ignored GeneratorExit")



def _throw(gen, typ, val, tb):
    if val is None and tb is None:
        return gen.throw(typ)
    return gen.throw(typ, val, tb)


class _ANextDefault:
    """anext(iterator, default): the awaitable of __anext__(), with
    StopAsyncIteration turned into the default."""
    __slots__ = ("_it", "_default")

    def __init__(self, awaitable, default):
        self._it = type(awaitable).__await__(awaitable)
        self._default = default

    def __await__(self):
        return self

    def __iter__(self):
        return self

    def __next__(self):
        return self.send(None)

    def send(self, value):
        try:
            return self._it.send(value)
        except StopAsyncIteration:
            raise StopIteration(self._default) from None

    def throw(self, typ, val=None, tb=None):
        try:
            return _throw(self._it, typ, val, tb)
        except StopAsyncIteration:
            raise StopIteration(self._default) from None

    def close(self):
        close = getattr(self._it, "close", None)
        if close is not None:
            close()
