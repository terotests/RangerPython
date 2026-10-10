# SPDX-License-Identifier: MIT
# reduce, partial, lru_cache, cache, total_ordering and cmp_to_key for the Rust runtime.

_initial_missing = object()


def reduce(function, iterable, initial=_initial_missing, /):
    it = iter(iterable)
    if initial is _initial_missing:
        try:
            value = next(it)
        except StopIteration:
            raise TypeError("reduce() of empty iterable with no initial value") from None
    else:
        value = initial
    for element in it:
        value = function(value, element)
    return value


class partial:
    __slots__ = ("func", "args", "keywords", "__dict__")

    def __new__(cls, func, /, *args, **keywords):
        if not callable(func):
            raise TypeError("the first argument must be callable")
        if isinstance(func, partial):
            args = func.args + args
            keywords = {**func.keywords, **keywords}
            func = func.func
        self = object.__new__(cls)
        self.func = func
        self.args = args
        self.keywords = keywords
        return self

    def __call__(self, /, *args, **keywords):
        keywords = {**self.keywords, **keywords}
        return self.func(*self.args, *args, **keywords)

    def __repr__(self):
        a = [repr(self.func)]
        a.extend(repr(x) for x in self.args)
        a.extend(f"{k}={v!r}" for k, v in self.keywords.items())
        return f"functools.partial({', '.join(a)})"


class _CacheInfo(tuple):
    __slots__ = ()

    def __new__(cls, hits, misses, maxsize, currsize):
        return tuple.__new__(cls, (hits, misses, maxsize, currsize))

    hits = property(lambda self: self[0])
    misses = property(lambda self: self[1])
    maxsize = property(lambda self: self[2])
    currsize = property(lambda self: self[3])

    def __repr__(self):
        return "CacheInfo(hits=%r, misses=%r, maxsize=%r, currsize=%r)" % tuple(self)


_kwd_mark = object()


def _make_key(args, kwds, typed):
    key = args
    if kwds:
        key += (_kwd_mark,)
        for item in kwds.items():
            key += item
    if typed:
        key += tuple(type(v) for v in args)
        if kwds:
            key += tuple(type(v) for v in kwds.values())
    elif len(key) == 1 and type(key[0]) in (int, str):
        return key[0]
    return key


class _lru_cache_wrapper:
    def __init__(self, func, maxsize, typed):
        self.__wrapped__ = func
        self._maxsize = maxsize
        self._typed = typed
        self._cache = {}
        self._hits = 0
        self._misses = 0
        for a in ("__module__", "__name__", "__qualname__", "__doc__"):
            try:
                setattr(self, a, getattr(func, a))
            except AttributeError:
                pass

    def __call__(self, *args, **kwds):
        key = _make_key(args, kwds, self._typed)
        cache = self._cache
        if key in cache:
            self._hits += 1
            if self._maxsize is not None:
                v = cache.pop(key)
                cache[key] = v
                return v
            return cache[key]
        self._misses += 1
        v = self.__wrapped__(*args, **kwds)
        if self._maxsize is not None:
            if self._maxsize <= 0:
                return v
            if key not in cache and len(cache) >= self._maxsize:
                del cache[next(iter(cache))]
        cache[key] = v
        return v

    def cache_info(self):
        return _CacheInfo(self._hits, self._misses, self._maxsize, len(self._cache))

    def cache_clear(self):
        self._cache.clear()
        self._hits = 0
        self._misses = 0

    def cache_parameters(self):
        return {"maxsize": self._maxsize, "typed": self._typed}

    def __get__(self, obj, objtype=None):
        if obj is None:
            return self
        return partial(self, obj)

    def __repr__(self):
        return f"<functools._lru_cache_wrapper object at 0x{id(self):x}>"


def lru_cache(maxsize=128, typed=False):
    if isinstance(maxsize, int):
        if maxsize < 0:
            maxsize = 0
    elif callable(maxsize) and isinstance(typed, bool):
        return _lru_cache_wrapper(maxsize, 128, typed)
    elif maxsize is not None:
        raise TypeError("Expected first argument to be an integer, a callable, or None")

    def decorating_function(user_function):
        return _lru_cache_wrapper(user_function, maxsize, typed)
    return decorating_function


def cache(user_function, /):
    return _lru_cache_wrapper(user_function, None, False)


def _gt_from_lt(self, other):
    op_result = type(self).__lt__(self, other)
    if op_result is NotImplemented:
        return op_result
    return not op_result and self != other


def _le_from_lt(self, other):
    op_result = type(self).__lt__(self, other)
    if op_result is NotImplemented:
        return op_result
    return op_result or self == other


def _ge_from_lt(self, other):
    op_result = type(self).__lt__(self, other)
    if op_result is NotImplemented:
        return op_result
    return not op_result


def _ge_from_le(self, other):
    op_result = type(self).__le__(self, other)
    if op_result is NotImplemented:
        return op_result
    return not op_result or self == other


def _lt_from_le(self, other):
    op_result = type(self).__le__(self, other)
    if op_result is NotImplemented:
        return op_result
    return op_result and self != other


def _gt_from_le(self, other):
    op_result = type(self).__le__(self, other)
    if op_result is NotImplemented:
        return op_result
    return not op_result


def _lt_from_gt(self, other):
    op_result = type(self).__gt__(self, other)
    if op_result is NotImplemented:
        return op_result
    return not op_result and self != other


def _ge_from_gt(self, other):
    op_result = type(self).__gt__(self, other)
    if op_result is NotImplemented:
        return op_result
    return op_result or self == other


def _le_from_gt(self, other):
    op_result = type(self).__gt__(self, other)
    if op_result is NotImplemented:
        return op_result
    return not op_result


def _le_from_ge(self, other):
    op_result = type(self).__ge__(self, other)
    if op_result is NotImplemented:
        return op_result
    return not op_result or self == other


def _gt_from_ge(self, other):
    op_result = type(self).__ge__(self, other)
    if op_result is NotImplemented:
        return op_result
    return op_result and self != other


def _lt_from_ge(self, other):
    op_result = type(self).__ge__(self, other)
    if op_result is NotImplemented:
        return op_result
    return not op_result


_convert = {
    "__lt__": [("__gt__", _gt_from_lt), ("__le__", _le_from_lt), ("__ge__", _ge_from_lt)],
    "__le__": [("__ge__", _ge_from_le), ("__lt__", _lt_from_le), ("__gt__", _gt_from_le)],
    "__gt__": [("__lt__", _lt_from_gt), ("__ge__", _ge_from_gt), ("__le__", _le_from_gt)],
    "__ge__": [("__le__", _le_from_ge), ("__gt__", _gt_from_ge), ("__lt__", _lt_from_ge)],
}


def total_ordering(cls):
    roots = {op for op in _convert if getattr(cls, op, None) is not getattr(object, op, None)}
    if not roots:
        raise ValueError("must define at least one ordering operation: < > <= >=")
    root = max(roots)
    for opname, opfunc in _convert[root]:
        if opname not in roots:
            opfunc.__name__ = opname
            setattr(cls, opname, opfunc)
    return cls


def cmp_to_key(mycmp):
    class K(object):
        __slots__ = ["obj"]

        def __init__(self, obj):
            self.obj = obj

        def __lt__(self, other):
            return mycmp(self.obj, other.obj) < 0

        def __gt__(self, other):
            return mycmp(self.obj, other.obj) > 0

        def __eq__(self, other):
            return mycmp(self.obj, other.obj) == 0

        def __le__(self, other):
            return mycmp(self.obj, other.obj) <= 0

        def __ge__(self, other):
            return mycmp(self.obj, other.obj) >= 0

        __hash__ = None
    return K
