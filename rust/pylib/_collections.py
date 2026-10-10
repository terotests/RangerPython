# SPDX-License-Identifier: MIT
# deque and defaultdict for the Rust runtime, where CPython has them in C.
import sys as _sys
from operator import itemgetter as _itemgetter
from keyword import iskeyword as _iskeyword


class deque:
    __slots__ = ("_d", "_max")

    def __init__(self, iterable=(), maxlen=None):
        if maxlen is not None:
            if not isinstance(maxlen, int):
                raise TypeError("an integer is required")
            if maxlen < 0:
                raise ValueError("maxlen must be non-negative")
        self._max = maxlen
        self._d = []
        self.extend(iterable)

    @property
    def maxlen(self):
        return self._max

    def _trim_right(self):
        if self._max is not None and len(self._d) > self._max:
            del self._d[:len(self._d) - self._max]

    def _trim_left(self):
        if self._max is not None and len(self._d) > self._max:
            del self._d[self._max:]

    def append(self, x):
        self._d.append(x)
        self._trim_right()

    def appendleft(self, x):
        self._d.insert(0, x)
        self._trim_left()

    def extend(self, it):
        if it is self:
            it = list(it)
        for x in it:
            self.append(x)

    def extendleft(self, it):
        if it is self:
            it = list(it)
        for x in it:
            self.appendleft(x)

    def pop(self):
        if not self._d:
            raise IndexError("pop from an empty deque")
        return self._d.pop()

    def popleft(self):
        if not self._d:
            raise IndexError("pop from an empty deque")
        return self._d.pop(0)

    def clear(self):
        self._d.clear()

    def copy(self):
        return deque(self._d, self._max)

    __copy__ = copy

    def count(self, x):
        return self._d.count(x)

    def index(self, x, *args):
        try:
            return self._d.index(x, *args)
        except ValueError:
            raise ValueError(f"{x!r} is not in deque") from None

    def insert(self, i, x):
        if self._max is not None and len(self._d) >= self._max:
            raise IndexError("deque already at its maximum size")
        self._d.insert(i, x)

    def remove(self, x):
        try:
            self._d.remove(x)
        except ValueError:
            raise ValueError(f"{x!r} is not in deque") from None

    def reverse(self):
        self._d.reverse()

    def rotate(self, n=1):
        d = self._d
        if not d:
            return
        n %= len(d)
        if n:
            d[:] = d[-n:] + d[:-n]

    def __len__(self):
        return len(self._d)

    def __bool__(self):
        return bool(self._d)

    def __iter__(self):
        return iter(self._d)

    def __reversed__(self):
        return reversed(self._d)

    def __contains__(self, x):
        return x in self._d

    def __getitem__(self, i):
        if isinstance(i, slice):
            raise TypeError("sequence index must be integer, not 'slice'")
        try:
            return self._d[i]
        except IndexError:
            raise IndexError("deque index out of range") from None

    def __setitem__(self, i, v):
        try:
            self._d[i] = v
        except IndexError:
            raise IndexError("deque index out of range") from None

    def __delitem__(self, i):
        try:
            del self._d[i]
        except IndexError:
            raise IndexError("deque index out of range") from None

    def __eq__(self, o):
        if not isinstance(o, deque):
            return NotImplemented
        return self._d == o._d

    def __lt__(self, o):
        if not isinstance(o, deque):
            return NotImplemented
        return self._d < o._d

    def __le__(self, o):
        if not isinstance(o, deque):
            return NotImplemented
        return self._d <= o._d

    def __gt__(self, o):
        if not isinstance(o, deque):
            return NotImplemented
        return self._d > o._d

    def __ge__(self, o):
        if not isinstance(o, deque):
            return NotImplemented
        return self._d >= o._d

    __hash__ = None

    def __add__(self, o):
        if not isinstance(o, deque):
            raise TypeError(f'can only concatenate deque (not "{type(o).__name__}") to deque')
        r = deque(self._d, self._max)
        r.extend(o)
        return r

    def __iadd__(self, o):
        self.extend(o)
        return self

    def __mul__(self, n):
        return deque(self._d * n, self._max)

    __rmul__ = __mul__

    def __imul__(self, n):
        self._d *= n
        self._trim_right()
        return self

    def __repr__(self):
        if self._max is None:
            return f"{type(self).__name__}({self._d!r})"
        return f"{type(self).__name__}({self._d!r}, maxlen={self._max})"

    def __class_getitem__(cls, item):
        return cls


class defaultdict(dict):
    def __init__(self, default_factory=None, *args, **kwargs):
        if default_factory is not None and not callable(default_factory):
            raise TypeError("first argument must be callable or None")
        dict.__init__(self, *args, **kwargs)
        self.default_factory = default_factory

    def __missing__(self, key):
        if self.default_factory is None:
            raise KeyError(key)
        v = self.default_factory()
        self[key] = v
        return v

    def copy(self):
        return type(self)(self.default_factory, self)

    __copy__ = copy

    def __repr__(self):
        f = self.default_factory
        fr = "None" if f is None else repr(f)
        return f"{type(self).__name__}({fr}, {dict.__repr__(self)})"

    def __or__(self, other):
        if not isinstance(other, dict):
            return NotImplemented
        r = self.copy()
        r.update(other)
        return r

    def __reduce__(self):
        return (type(self), (self.default_factory,), None, None, iter(self.items()))
