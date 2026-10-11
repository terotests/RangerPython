# SPDX-License-Identifier: MIT
# itertools for the Rust runtime, following CPython's Modules/itertoolsmodule.c
# (argument checking, __reduce__/__setstate__ and copy support).

from operator import index as _index
from _rpyitertools import immutable as _immutable, reuse as _reuse
from _rpyitertools import chain_next as _chain_next, tee_next as _tee_next, repeat_next as _repeat_next

_NULL = object()
_MAXSIZE = (1 << 63) - 1
_object_new = object.__new__


def _deprecated():
    import warnings
    warnings.warn("Pickle, copy, and deepcopy support will be "
                  "removed from itertools in Python 3.14.",
                  DeprecationWarning, stacklevel=3)


def _nokw(cls, base, kw, name):
    if kw and (cls is base or cls.__init__ is object.__init__):
        raise TypeError(f"{name}() takes no keyword arguments")


def _ssize(obj):
    # PyNumber_AsSsize_t with errors folded into -1, as islice() does.
    try:
        v = _index(obj)
    except TypeError:
        return -1
    if v > _MAXSIZE or v < -_MAXSIZE - 1:
        return -1
    return v


def _ssize_arg(obj):
    v = _index(obj)
    if v > _MAXSIZE or v < -_MAXSIZE - 1:
        raise OverflowError("Python int too large to convert to C ssize_t")
    return v


def _int_arg(obj):
    if isinstance(obj, float):
        raise TypeError("'float' object cannot be interpreted as an integer")
    return _index(obj)


def _isnumber(obj):
    t = type(obj)
    return (isinstance(obj, complex) or hasattr(t, "__index__")
            or hasattr(t, "__int__") or hasattr(t, "__float__"))


class groupby:
    def __new__(cls, iterable, key=None):
        self = _object_new(cls)
        self._keyfunc = key
        self._it = iter(iterable)
        self._tgtkey = _NULL
        self._currkey = _NULL
        self._currvalue = _NULL
        self._currgrouper = None
        return self

    def __iter__(self):
        return self

    def _step(self):
        newvalue = next(self._it)
        if self._keyfunc is None:
            newkey = newvalue
        else:
            newkey = self._keyfunc(newvalue)
        self._currvalue = newvalue
        self._currkey = newkey

    def __next__(self):
        self._currgrouper = None
        while True:
            if self._currkey is _NULL:
                pass
            elif self._tgtkey is _NULL:
                break
            else:
                tgt, cur = self._tgtkey, self._currkey
                if not (tgt is cur or tgt == cur):
                    break
            self._step()
        self._tgtkey = self._currkey
        grouper = _grouper._create(self, self._tgtkey)
        return (self._currkey, grouper)

    def __reduce__(self):
        _deprecated()
        if (self._tgtkey is not _NULL and self._currkey is not _NULL
                and self._currvalue is not _NULL):
            return (type(self), (self._it, self._keyfunc),
                    (self._currkey, self._currvalue, self._tgtkey))
        return (type(self), (self._it, self._keyfunc))

    def __setstate__(self, state):
        _deprecated()
        if not isinstance(state, tuple):
            raise TypeError("state is not a tuple")
        if len(state) != 3:
            raise TypeError(f"function takes exactly 3 arguments ({len(state)} given)")
        self._currkey, self._currvalue, self._tgtkey = state


class _grouper:
    def __new__(cls, parent, tgtkey, /):
        if not isinstance(parent, groupby):
            raise TypeError(f"_grouper() argument 1 must be itertools.groupby, not {type(parent).__name__}")
        return cls._create(parent, tgtkey)

    @classmethod
    def _create(cls, parent, tgtkey):
        self = _object_new(cls)
        self._parent = parent
        self._tgtkey = tgtkey
        parent._currgrouper = self
        return self

    def __iter__(self):
        return self

    def __next__(self):
        gbo = self._parent
        if gbo._currgrouper is not self:
            raise StopIteration
        if gbo._currvalue is _NULL:
            gbo._step()
        tgt, cur = self._tgtkey, gbo._currkey
        if not (tgt is cur or tgt == cur):
            raise StopIteration
        r = gbo._currvalue
        gbo._currvalue = _NULL
        gbo._currkey = _NULL
        if r is _NULL:
            raise StopIteration
        return r

    def __reduce__(self):
        _deprecated()
        if self._parent._currgrouper is not self:
            return (iter, ((),))
        return (type(self), (self._parent, self._tgtkey))


_LINKCELLS = 57


class _tee_dataobject:
    def __new__(cls, it, values, next, /):
        self = cls._create(it)
        if not isinstance(values, list):
            raise TypeError("_tee_dataobject argument 2 must be a list")
        if len(values) > _LINKCELLS:
            raise ValueError("Invalid arguments")
        self._values = list(values)
        if len(values) == _LINKCELLS:
            if next is not None:
                if type(next) is not _tee_dataobject:
                    raise ValueError("Invalid arguments")
                self._nextlink = next
        elif next is not None:
            raise ValueError("Invalid arguments")
        return self

    @classmethod
    def _create(cls, it):
        self = _object_new(cls)
        self._it = it
        self._values = []
        self._running = False
        self._nextlink = None
        return self

    def _getitem(self, i):
        values = self._values
        if i < len(values):
            return values[i]
        if self._running:
            raise RuntimeError("cannot re-enter the tee iterator")
        self._running = True
        try:
            value = next(self._it)
        finally:
            self._running = False
        values.append(value)
        return value

    def _jumplink(self):
        if self._nextlink is None:
            self._nextlink = _tee_dataobject._create(self._it)
        return self._nextlink

    def __reduce__(self):
        _deprecated()
        return (type(self), (self._it, list(self._values), self._nextlink))


class _tee:
    def __new__(cls, iterable, /):
        return cls._fromiterable(iterable)

    @classmethod
    def _fromiterable(cls, iterable):
        it = iter(iterable)
        if isinstance(it, _tee):
            return it.__copy__()
        self = _object_new(cls)
        self._dataobj = _tee_dataobject._create(it)
        self._index = 0
        return self

    def __iter__(self):
        return self

    # native, as CPython's tee_next: item _index of _dataobj, as
    # _tee_dataobject._getitem gives it
    __next__ = _tee_next

    def __copy__(self):
        new = _object_new(type(self))
        new._dataobj = self._dataobj
        new._index = self._index
        return new

    def __reduce__(self):
        _deprecated()
        return (type(self), ((),), (self._dataobj, self._index))

    def __setstate__(self, state):
        _deprecated()
        if not isinstance(state, tuple):
            raise TypeError("state is not a tuple")
        if len(state) != 2:
            raise TypeError(f"function takes exactly 2 arguments ({len(state)} given)")
        tdo, index = state
        if not isinstance(tdo, _tee_dataobject):
            raise TypeError(f"argument 1 must be itertools._tee_dataobject, not {type(tdo).__name__}")
        index = _int_arg(index)
        if index < 0 or index > _LINKCELLS:
            raise ValueError("Index out of range")
        self._dataobj = tdo
        self._index = index


def tee(iterable, n=2, /):
    n = _ssize_arg(n)
    if n < 0:
        raise ValueError("n must be >= 0")
    if n == 0:
        return ()
    it = iter(iterable)
    copyfunc = getattr(it, "__copy__", None)
    if copyfunc is not None:
        copyable = it
    else:
        copyable = _tee._fromiterable(it)
        copyfunc = copyable.__copy__
    result = [copyable]
    for i in range(1, n):
        result.append(copyfunc())
    return tuple(result)


class cycle:
    def __new__(cls, iterable, /, **kw):
        _nokw(cls, cycle, kw, "cycle")
        self = _object_new(cls)
        self._it = iter(iterable)
        self._saved = []
        self._firstpass = False
        self._index = 0
        return self

    def __iter__(self):
        return self

    def __next__(self):
        if self._it is not None:
            try:
                item = next(self._it)
            except StopIteration:
                self._it = None
            else:
                if not self._firstpass:
                    self._saved.append(item)
                return item
        saved = self._saved
        if not saved:
            raise StopIteration
        if self._index >= len(saved):
            self._index = 0
        item = saved[self._index]
        self._index += 1
        if self._index >= len(saved):
            self._index = 0
        return item

    def __reduce__(self):
        _deprecated()
        if self._it is None:
            it = iter(self._saved)
            if self._index != 0:
                it.__setstate__(self._index)
            return (type(self), (it,), (self._saved, True))
        return (type(self), (self._it,), (self._saved, self._firstpass))

    def __setstate__(self, state):
        _deprecated()
        if not isinstance(state, tuple):
            raise TypeError("state is not a tuple")
        if len(state) != 2:
            raise TypeError(f"function takes exactly 2 arguments ({len(state)} given)")
        saved, firstpass = state
        if not isinstance(saved, list):
            raise TypeError(f"argument 1 must be list, not {type(saved).__name__}")
        firstpass = _int_arg(firstpass)
        self._saved = saved
        self._firstpass = firstpass != 0
        self._index = 0


class dropwhile:
    def __new__(cls, predicate, iterable, /, **kw):
        _nokw(cls, dropwhile, kw, "dropwhile")
        self = _object_new(cls)
        self._func = predicate
        self._it = iter(iterable)
        self._start = 0
        return self

    def __iter__(self):
        return self

    def __next__(self):
        while True:
            item = next(self._it)
            if self._start == 1:
                return item
            if not self._func(item):
                self._start = 1
                return item

    def __reduce__(self):
        _deprecated()
        return (type(self), (self._func, self._it), self._start)

    def __setstate__(self, state):
        _deprecated()
        self._start = 1 if state else 0


class takewhile:
    def __new__(cls, predicate, iterable, /, **kw):
        _nokw(cls, takewhile, kw, "takewhile")
        self = _object_new(cls)
        self._func = predicate
        self._it = iter(iterable)
        self._stop = 0
        return self

    def __iter__(self):
        return self

    def __next__(self):
        if self._stop == 1:
            raise StopIteration
        item = next(self._it)
        if self._func(item):
            return item
        self._stop = 1
        raise StopIteration

    def __reduce__(self):
        _deprecated()
        return (type(self), (self._func, self._it), self._stop)

    def __setstate__(self, state):
        _deprecated()
        self._stop = 1 if state else 0


_ISLICE_STOP = "Stop argument for islice() must be None or an integer: 0 <= x <= sys.maxsize."


class islice:
    def __new__(cls, *args, **kw):
        _nokw(cls, islice, kw, "islice")
        numargs = len(args)
        if numargs < 2:
            raise TypeError(f"islice expected at least 2 arguments, got {numargs}")
        if numargs > 4:
            raise TypeError(f"islice expected at most 4 arguments, got {numargs}")
        seq = args[0]
        start, stop, step = 0, -1, 1
        if numargs == 2:
            if args[1] is not None:
                stop = _ssize(args[1])
                if stop == -1:
                    raise ValueError(_ISLICE_STOP)
        else:
            if args[1] is not None:
                start = _ssize(args[1])
            if args[2] is not None:
                stop = _ssize(args[2])
                if stop == -1:
                    raise ValueError(_ISLICE_STOP)
        if start < 0 or stop < -1:
            raise ValueError("Indices for islice() must be None or an integer: 0 <= x <= sys.maxsize.")
        if numargs == 4 and args[3] is not None:
            step = _ssize(args[3])
        if step < 1:
            raise ValueError("Step for islice() must be a positive integer or None.")
        self = _object_new(cls)
        self._it = iter(seq)
        self._next = start
        self._stop = stop
        self._step = step
        self._cnt = 0
        return self

    def __iter__(self):
        return self

    def __next__(self):
        it = self._it
        if it is None:
            raise StopIteration
        stop = self._stop
        try:
            while self._cnt < self._next:
                next(it)
                self._cnt += 1
            if stop != -1 and self._cnt >= stop:
                raise StopIteration
            item = next(it)
        except BaseException:
            self._it = None
            raise
        self._cnt += 1
        self._next += self._step
        if stop != -1 and self._next > stop:
            self._next = stop
        return item

    def __reduce__(self):
        _deprecated()
        if self._it is None:
            return (type(self), (iter([]), 0), 0)
        stop = None if self._stop == -1 else self._stop
        return (type(self), (self._it, self._next, stop, self._step), self._cnt)

    def __setstate__(self, state):
        _deprecated()
        self._cnt = _ssize_arg(state)


class starmap:
    def __new__(cls, function, iterable, /, **kw):
        _nokw(cls, starmap, kw, "starmap")
        self = _object_new(cls)
        self._func = function
        self._it = iter(iterable)
        return self

    def __iter__(self):
        return self

    def __next__(self):
        args = next(self._it)
        if type(args) is not tuple:
            args = tuple(args)
        return self._func(*args)

    def __reduce__(self):
        _deprecated()
        return (type(self), (self._func, self._it))


class chain:
    def __new__(cls, *args, **kw):
        _nokw(cls, chain, kw, "chain")
        return cls._create(iter(args))

    @classmethod
    def _create(cls, source):
        self = _object_new(cls)
        self._source = source
        self._active = None
        return self

    @classmethod
    def from_iterable(cls, iterable, /):
        return cls._create(iter(iterable))

    def __iter__(self):
        return self

    # native, as CPython's chain_next: next item of _active, moving on
    # along _source as each is exhausted
    __next__ = _chain_next

    def __reduce__(self):
        _deprecated()
        if self._source is not None:
            if self._active is not None:
                return (type(self), (), (self._source, self._active))
            return (type(self), (), (self._source,))
        return (type(self), ())

    def __setstate__(self, state):
        _deprecated()
        if not isinstance(state, tuple):
            raise TypeError("state is not a tuple")
        if not state:
            raise TypeError("function takes at least 1 argument (0 given)")
        if len(state) > 2:
            raise TypeError(f"function takes at most 2 arguments ({len(state)} given)")
        source = state[0]
        active = state[1] if len(state) > 1 else None
        if not hasattr(type(source), "__next__") or (
                active is not None and not hasattr(type(active), "__next__")):
            raise TypeError("Arguments must be iterators.")
        self._source = source
        self._active = active


def _objsize(fields, n):
    # the size CPython's object of these fields takes, with n Py_ssize_t
    # indices after it: what sys.getsizeof counts for rpy's objects too
    import struct
    return struct.calcsize('nP' + fields + '0n') + n * struct.calcsize('n')


class product:
    def __new__(cls, *args, repeat=1):
        repeat = _ssize_arg(repeat)
        if repeat < 0:
            raise ValueError("repeat argument cannot be negative")
        if repeat == 0:
            args = ()
        elif args and len(args) > _MAXSIZE // 8 // repeat:
            raise OverflowError("repeat argument too large")
        pools = tuple(tuple(a) for a in args) * repeat
        self = _object_new(cls)
        self._pools = pools
        self._indices = [0] * len(pools)
        self._result = None
        self._stopped = False
        return self

    def __iter__(self):
        return self

    def __next__(self):
        if self._stopped:
            raise StopIteration
        pools = self._pools
        if self._result is None:
            result = []
            for pool in pools:
                if not pool:
                    self._stopped = True
                    raise StopIteration
                result.append(pool[0])
        else:
            result = list(self._result)
            indices = self._indices
            i = len(pools) - 1
            while i >= 0:
                pool = pools[i]
                indices[i] += 1
                if indices[i] == len(pool):
                    indices[i] = 0
                    result[i] = pool[0]
                else:
                    result[i] = pool[indices[i]]
                    break
                i -= 1
            if i < 0:
                self._stopped = True
                raise StopIteration
        return _reuse(self, result)

    def __sizeof__(self):
        return _objsize('3Pi', len(self._pools))

    def __reduce__(self):
        _deprecated()
        if self._stopped:
            return (type(self), ((),))
        if self._result is None:
            return (type(self), self._pools)
        return (type(self), self._pools, tuple(self._indices))

    def __setstate__(self, state):
        _deprecated()
        pools = self._pools
        n = len(pools)
        if not isinstance(state, tuple) or len(state) != n:
            raise ValueError("invalid arguments")
        indices = []
        for i in range(n):
            index = _ssize_arg(state[i])
            poolsize = len(pools[i])
            if poolsize == 0:
                self._stopped = True
                return
            if index < 0:
                index = 0
            elif index > poolsize - 1:
                index = poolsize - 1
            indices.append(index)
        self._indices = indices
        self._result = tuple(pools[i][indices[i]] for i in range(n))


class combinations:
    def __new__(cls, iterable, r):
        pool = tuple(iterable)
        r = _ssize_arg(r)
        if r < 0:
            raise ValueError("r must be non-negative")
        self = _object_new(cls)
        self._pool = pool
        self._r = r
        self._indices = list(range(r))
        self._result = None
        self._stopped = r > len(pool)
        return self

    def __iter__(self):
        return self

    def __next__(self):
        if self._stopped:
            raise StopIteration
        pool = self._pool
        indices = self._indices
        r = self._r
        if self._result is not None:
            n = len(pool)
            i = r - 1
            while i >= 0 and indices[i] == i + n - r:
                i -= 1
            if i < 0:
                self._stopped = True
                raise StopIteration
            indices[i] += 1
            for j in range(i + 1, r):
                indices[j] = indices[j - 1] + 1
        return _reuse(self, [pool[i] for i in indices])

    def __sizeof__(self):
        return _objsize('3Pni', self._r)

    def __reduce__(self):
        _deprecated()
        if self._result is None:
            return (type(self), (self._pool, self._r))
        if self._stopped:
            return (type(self), ((), self._r))
        return (type(self), (self._pool, self._r), tuple(self._indices))

    def __setstate__(self, state):
        _deprecated()
        r = self._r
        n = len(self._pool)
        if not isinstance(state, tuple) or len(state) != r:
            raise ValueError("invalid arguments")
        for i in range(r):
            index = _ssize_arg(state[i])
            hi = i + n - r
            if index > hi:
                index = hi
            if index < 0:
                index = 0
            self._indices[i] = index
        self._result = tuple(self._pool[i] for i in self._indices)


class combinations_with_replacement:
    def __new__(cls, iterable, r):
        pool = tuple(iterable)
        r = _ssize_arg(r)
        if r < 0:
            raise ValueError("r must be non-negative")
        self = _object_new(cls)
        self._pool = pool
        self._r = r
        self._indices = [0] * r
        self._result = None
        self._stopped = not pool and r > 0
        return self

    def __iter__(self):
        return self

    def __next__(self):
        if self._stopped:
            raise StopIteration
        pool = self._pool
        indices = self._indices
        r = self._r
        if self._result is None:
            self._result = result = (pool[0],) * r if r else ()
            return result
        n = len(pool)
        i = r - 1
        while i >= 0 and indices[i] == n - 1:
            i -= 1
        if i < 0:
            self._stopped = True
            raise StopIteration
        index = indices[i] + 1
        for k in range(i, r):
            indices[k] = index
        return _reuse(self, [pool[k] for k in indices])

    def __sizeof__(self):
        return _objsize('3Pni', self._r)

    def __reduce__(self):
        _deprecated()
        if self._result is None:
            return (type(self), (self._pool, self._r))
        if self._stopped:
            return (type(self), ((), self._r))
        return (type(self), (self._pool, self._r), tuple(self._indices))

    def __setstate__(self, state):
        _deprecated()
        r = self._r
        n = len(self._pool)
        if not isinstance(state, tuple) or len(state) != r:
            raise ValueError("invalid arguments")
        for i in range(r):
            index = _ssize_arg(state[i])
            if index < 0:
                index = 0
            elif index > n - 1:
                index = n - 1
            self._indices[i] = index
        self._result = tuple(self._pool[i] for i in self._indices)


class permutations:
    def __new__(cls, iterable, r=None):
        pool = tuple(iterable)
        n = len(pool)
        if r is None:
            r = n
        else:
            if not isinstance(r, int):
                raise TypeError("Expected int as r")
            r = _ssize_arg(r)
        if r < 0:
            raise ValueError("r must be non-negative")
        self = _object_new(cls)
        self._pool = pool
        self._r = r
        self._indices = list(range(n))
        self._cycles = list(range(n, n - r, -1))
        self._result = None
        self._stopped = r > n
        return self

    def __iter__(self):
        return self

    def __next__(self):
        if self._stopped:
            raise StopIteration
        pool = self._pool
        indices = self._indices
        r = self._r
        if self._result is not None:
            n = len(pool)
            if n == 0:
                self._stopped = True
                raise StopIteration
            cycles = self._cycles
            i = r - 1
            while i >= 0:
                cycles[i] -= 1
                if cycles[i] == 0:
                    indices[i:] = indices[i + 1:] + indices[i:i + 1]
                    cycles[i] = n - i
                else:
                    j = cycles[i]
                    indices[i], indices[-j] = indices[-j], indices[i]
                    break
                i -= 1
            if i < 0:
                self._stopped = True
                raise StopIteration
        return _reuse(self, [pool[indices[k]] for k in range(r)])

    def __sizeof__(self):
        return _objsize('4Pni', len(self._pool) + self._r)

    def __reduce__(self):
        _deprecated()
        if self._result is None:
            return (type(self), (self._pool, self._r))
        if self._stopped:
            return (type(self), ((), self._r))
        return (type(self), (self._pool, self._r),
                (tuple(self._indices), tuple(self._cycles)))

    def __setstate__(self, state):
        _deprecated()
        if not isinstance(state, tuple):
            raise TypeError("state is not a tuple")
        if len(state) != 2:
            raise TypeError(f"function takes exactly 2 arguments ({len(state)} given)")
        indices, cycles = state
        for k, v in ((1, indices), (2, cycles)):
            if not isinstance(v, tuple):
                raise TypeError(f"argument {k} must be tuple, not {type(v).__name__}")
        n = len(self._pool)
        r = self._r
        if len(indices) != n or len(cycles) != r:
            raise ValueError("invalid arguments")
        for i in range(n):
            index = _ssize_arg(indices[i])
            if index < 0:
                index = 0
            elif index > n - 1:
                index = n - 1
            self._indices[i] = index
        for i in range(r):
            index = _ssize_arg(cycles[i])
            if index < 1:
                index = 1
            elif index > n - i:
                index = n - i
            self._cycles[i] = index
        self._result = tuple(self._pool[self._indices[k]] for k in range(r))


class accumulate:
    def __new__(cls, iterable, func=None, *, initial=None):
        self = _object_new(cls)
        self._it = iter(iterable)
        self._binop = func
        self._total = _NULL
        self._initial = initial
        return self

    def __iter__(self):
        return self

    def __next__(self):
        if self._initial is not None:
            self._total = self._initial
            self._initial = None
            return self._total
        val = next(self._it)
        if self._total is _NULL:
            self._total = val
            return val
        if self._binop is None:
            self._total = self._total + val
        else:
            self._total = self._binop(self._total, val)
        return self._total

    def __reduce__(self):
        _deprecated()
        if self._initial is not None:
            it = chain((self._initial,), self._it)
            return (type(self), (it, self._binop), None)
        if self._total is None:
            it = chain((self._total,), self._it)
            it = type(self)(it, self._binop)
            return (islice, (it, 1, None))
        total = None if self._total is _NULL else self._total
        return (type(self), (self._it, self._binop), total)

    def __setstate__(self, state):
        _deprecated()
        self._total = _NULL if state is None else state


class batched:
    def __new__(cls, iterable, n, *, strict=False):
        n = _ssize_arg(n)
        if n < 1:
            raise ValueError("n must be at least one")
        self = _object_new(cls)
        self._it = iter(iterable)
        self._n = n
        self._strict = bool(strict)
        return self

    def __iter__(self):
        return self

    def __next__(self):
        it = self._it
        if it is None:
            raise StopIteration
        result = []
        n = self._n
        try:
            while len(result) < n:
                result.append(next(it))
        except StopIteration:
            pass
        except BaseException:
            self._it = None
            raise
        if not result:
            self._it = None
            raise StopIteration
        if len(result) < n and self._strict:
            self._it = None
            raise ValueError("batched(): incomplete batch")
        return tuple(result)


class compress:
    def __new__(cls, data, selectors):
        self = _object_new(cls)
        self._data = iter(data)
        self._selectors = iter(selectors)
        return self

    def __iter__(self):
        return self

    def __next__(self):
        data = self._data
        selectors = self._selectors
        while True:
            datum = next(data)
            if next(selectors):
                return datum

    def __reduce__(self):
        _deprecated()
        return (type(self), (self._data, self._selectors))


class filterfalse:
    def __new__(cls, function, iterable, /, **kw):
        _nokw(cls, filterfalse, kw, "filterfalse")
        self = _object_new(cls)
        self._func = function
        self._it = iter(iterable)
        return self

    def __iter__(self):
        return self

    def __next__(self):
        func = self._func
        while True:
            item = next(self._it)
            if func is None or func is bool:
                ok = item
            else:
                ok = func(item)
            if not ok:
                return item

    def __reduce__(self):
        _deprecated()
        return (type(self), (self._func, self._it))


class count:
    def __new__(cls, start=0, step=1):
        if not _isnumber(start) or not _isnumber(step):
            raise TypeError("a number is required")
        self = _object_new(cls)
        self._cnt = start
        self._step = step
        return self

    def __iter__(self):
        return self

    def __next__(self):
        n = self._cnt
        self._cnt = n + self._step
        return n

    def __repr__(self):
        step = self._step
        if isinstance(step, int) and step == 1:
            return f"{type(self).__name__}({self._cnt!r})"
        return f"{type(self).__name__}({self._cnt!r}, {step!r})"

    def __reduce__(self):
        _deprecated()
        step = self._step
        if isinstance(step, int) and step == 1 and type(self._cnt) is int:
            return (type(self), (self._cnt,))
        return (type(self), (self._cnt, step))


class repeat:
    def __new__(cls, object, times=_NULL):
        self = _object_new(cls)
        self._element = object
        if times is _NULL:
            self._cnt = -1
        else:
            cnt = _ssize_arg(times)
            self._cnt = cnt if cnt >= 0 else 0
        return self

    def __iter__(self):
        return self

    # native, as CPython's repeat_next
    __next__ = _repeat_next

    def __length_hint__(self):
        if self._cnt == -1:
            raise TypeError("len() of unsized object")
        return self._cnt

    def __repr__(self):
        if self._cnt == -1:
            return f"{type(self).__name__}({self._element!r})"
        return f"{type(self).__name__}({self._element!r}, {self._cnt})"

    def __reduce__(self):
        _deprecated()
        if self._cnt >= 0:
            return (type(self), (self._element, self._cnt))
        return (type(self), (self._element,))


class pairwise:
    def __new__(cls, iterable, /, **kw):
        _nokw(cls, pairwise, kw, "pairwise")
        self = _object_new(cls)
        self._it = iter(iterable)
        self._old = _NULL
        return self

    def __iter__(self):
        return self

    def __next__(self):
        it = self._it
        if it is None:
            raise StopIteration
        old = self._old
        if old is _NULL:
            try:
                old = next(it)
            except BaseException:
                self._old = _NULL
                self._it = None
                raise
            self._old = old
            it = self._it
            if it is None:
                self._old = _NULL
                raise StopIteration
        try:
            new = next(it)
        except BaseException:
            self._it = None
            self._old = _NULL
            raise
        self._old = new
        return _reuse(self, (old, new))


class zip_longest:
    def __new__(cls, *args, **kw):
        fillvalue = None
        if kw:
            if len(kw) != 1 or "fillvalue" not in kw:
                raise TypeError("zip_longest() got an unexpected keyword argument")
            fillvalue = kw["fillvalue"]
        iterators = [iter(a) for a in args]
        self = _object_new(cls)
        self._iterators = iterators
        self._numactive = len(iterators)
        self._fillvalue = fillvalue
        return self

    def __iter__(self):
        return self

    def __next__(self):
        iterators = self._iterators
        if not iterators or self._numactive == 0:
            raise StopIteration
        result = []
        for i, it in enumerate(iterators):
            if it is None:
                item = self._fillvalue
            else:
                try:
                    item = next(it)
                except StopIteration:
                    self._numactive -= 1
                    if self._numactive == 0:
                        raise
                    item = self._fillvalue
                    iterators[i] = None
                except BaseException:
                    self._numactive = 0
                    raise
            result.append(item)
        return _reuse(self, result)

    def __reduce__(self):
        _deprecated()
        args = tuple(() if it is None else it for it in self._iterators)
        return (type(self), args, self._fillvalue)

    def __setstate__(self, state):
        _deprecated()
        self._fillvalue = state


# the C module's types are static: their attributes cannot be set
for _tp in (accumulate, batched, chain, combinations,
            combinations_with_replacement, compress, count, cycle, dropwhile,
            filterfalse, groupby, _grouper, islice, pairwise, permutations,
            product, repeat, starmap, takewhile, _tee, _tee_dataobject,
            zip_longest):
    _immutable(_tp)
del _tp
