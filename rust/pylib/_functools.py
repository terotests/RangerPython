# SPDX-License-Identifier: MIT
# _functools for the Rust runtime: cmp_to_key and reduce behave like the C
# builtins, so they do not bind as methods when stored on a class.


class _BuiltinFunction:
    def __init__(self, f, text_signature=None):
        self._f = f
        self.__text_signature__ = text_signature
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


class partial:
    """Create a new function with partial application of the given arguments
and keywords."""

    __slots__ = "func", "args", "keywords", "__dict__", "__weakref__"
    __module__ = "functools"

    def __new__(cls, func, /, *args, **keywords):
        if not callable(func):
            raise TypeError("the first argument must be callable")
        if isinstance(func, partial) and not func.__dict__:
            args = func.args + args
            keywords = {**func.keywords, **keywords}
            func = func.func
        self = object.__new__(cls)
        object.__setattr__(self, "func", func)
        object.__setattr__(self, "args", args)
        object.__setattr__(self, "keywords", keywords)
        return self

    # func, args and keywords are read-only members, as in C
    def __setattr__(self, name, value):
        if name in ("func", "args", "keywords"):
            raise AttributeError("readonly attribute")
        if name == "__dict__" and not isinstance(value, dict):
            raise TypeError("setting partial object's dictionary to a non-dict")
        object.__setattr__(self, name, value)

    def __delattr__(self, name):
        if name in ("func", "args", "keywords"):
            raise AttributeError("readonly attribute")
        if name == "__dict__":
            raise TypeError("a partial object's dictionary may not be deleted")
        object.__delattr__(self, name)

    def __call__(self, /, *args, **keywords):
        keywords = {**self.keywords, **keywords}
        return self.func(*self.args, *args, **keywords)

    def __repr__(self):
        if self in _repr_running:
            return "..."
        _repr_running.append(self)
        try:
            cls = type(self)
            name = cls.__qualname__ if cls is partial else cls.__name__
            args = [repr(self.func)]
            args.extend(repr(x) for x in self.args)
            args.extend(f"{k}={v!r}" for (k, v) in list(self.keywords.items()))
            return f"{cls.__module__}.{name}({', '.join(args)})"
        finally:
            _repr_running.remove(self)

    def __get__(self, obj, objtype=None):
        if obj is None:
            return self
        import warnings
        warnings.warn('functools.partial will be a method descriptor in '
                      'future Python versions; wrap it in staticmethod() '
                      'if you want to preserve the old behavior',
                      FutureWarning, 2)
        return self

    def __reduce__(self):
        return type(self), (self.func,), (self.func, self.args,
               self.keywords or None, self.__dict__ or None)

    def __setstate__(self, state):
        if not isinstance(state, tuple):
            raise TypeError("argument to __setstate__ must be a tuple")
        if len(state) != 4:
            raise TypeError(f"expected 4 items in state, got {len(state)}")
        func, args, kwds, namespace = state
        if (not callable(func) or not isinstance(args, tuple) or
           (kwds is not None and not isinstance(kwds, dict)) or
           (namespace is not None and not isinstance(namespace, dict))):
            raise TypeError("invalid partial state")
        args = tuple(args)
        if kwds is None:
            kwds = {}
        elif type(kwds) is not dict:
            kwds = dict(kwds)
        if namespace is None:
            namespace = {}
        object.__setattr__(self, "__dict__", namespace)
        object.__setattr__(self, "func", func)
        object.__setattr__(self, "args", args)
        object.__setattr__(self, "keywords", kwds)

    def __class_getitem__(cls, item):
        from types import GenericAlias
        return GenericAlias(cls, item)


_repr_running = []


cmp_to_key = _BuiltinFunction(cmp_to_key, "($module, /, mycmp)")
reduce = _BuiltinFunction(reduce)
