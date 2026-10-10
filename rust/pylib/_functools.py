# SPDX-License-Identifier: MIT
# _functools for the Rust runtime: cmp_to_key and reduce behave like the C
# builtins, so they do not bind as methods when stored on a class.


class _BuiltinFunction:
    def __init__(self, f):
        self._f = f
        self.__name__ = f.__name__
        self.__qualname__ = f.__name__
        self.__doc__ = f.__doc__
        self.__module__ = "_functools"

    def __call__(self, /, *args, **kwargs):
        return self._f(*args, **kwargs)

    def __repr__(self):
        return "<built-in function %s>" % self.__name__

    def __reduce__(self):
        return self.__name__


def cmp_to_key(mycmp):
    """Convert a cmp= function into a key= function."""
    class K(object):
        __slots__ = ['obj']
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


_initial_missing = object()


def reduce(function, iterable, initial=_initial_missing, /):
    """reduce(function, iterable[, initial], /) -> value

Apply a function of two arguments cumulatively to the items of an iterable, from left to right.

This effectively reduces the iterable to a single value.  If initial is present,
it is placed before the items of the iterable in the calculation, and serves as
a default when the iterable is empty.

For example, reduce(lambda x, y: x+y, [1, 2, 3, 4, 5])
calculates ((((1 + 2) + 3) + 4) + 5)."""
    try:
        it = iter(iterable)
    except TypeError:
        raise TypeError("reduce() arg 2 must support iteration") from None
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


cmp_to_key = _BuiltinFunction(cmp_to_key)
reduce = _BuiltinFunction(reduce)
