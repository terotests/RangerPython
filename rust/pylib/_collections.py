# SPDX-License-Identifier: MIT
# deque, defaultdict, Counter, OrderedDict and namedtuple for the Rust runtime.
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


class OrderedDict(dict):
    def __repr__(self):
        if not self:
            return f"{type(self).__name__}()"
        return f"{type(self).__name__}({dict.__repr__(self)})"

    def move_to_end(self, key, last=True):
        v = dict.pop(self, key)
        if last:
            dict.__setitem__(self, key, v)
        else:
            items = list(dict.items(self))
            dict.clear(self)
            dict.__setitem__(self, key, v)
            for k, x in items:
                dict.__setitem__(self, k, x)

    def popitem(self, last=True):
        if not self:
            raise KeyError("dictionary is empty")
        if last:
            return dict.popitem(self)
        k = next(iter(self))
        return (k, dict.pop(self, k))

    def copy(self):
        return type(self)(self)

    __copy__ = copy

    def __eq__(self, other):
        if isinstance(other, OrderedDict):
            return dict.__eq__(self, other) and list(self) == list(other)
        return dict.__eq__(self, other)

    def __ne__(self, other):
        r = self.__eq__(other)
        if r is NotImplemented:
            return r
        return not r

    __hash__ = None

    def __reversed__(self):
        return reversed(list(dict.keys(self)))

    def __or__(self, other):
        if not isinstance(other, dict):
            return NotImplemented
        r = self.copy()
        r.update(other)
        return r


class Counter(dict):
    def __init__(self, iterable=None, /, **kwds):
        dict.__init__(self)
        self.update(iterable, **kwds)

    def __missing__(self, key):
        return 0

    def total(self):
        return sum(self.values())

    def most_common(self, n=None):
        items = sorted(self.items(), key=_itemgetter(1), reverse=True)
        if n is None:
            return items
        return items[:max(n, 0)]

    def elements(self):
        for k, c in self.items():
            for _ in range(c):
                yield k

    @classmethod
    def fromkeys(cls, iterable, v=None):
        raise NotImplementedError("Counter.fromkeys() is undefined.  Use Counter(iterable) instead.")

    def update(self, iterable=None, /, **kwds):
        if iterable is not None:
            if isinstance(iterable, dict):
                if self:
                    for k, c in iterable.items():
                        self[k] = self.get(k, 0) + c
                else:
                    dict.update(self, iterable)
            else:
                get = self.get
                for k in iterable:
                    self[k] = get(k, 0) + 1
        if kwds:
            self.update(kwds)

    def subtract(self, iterable=None, /, **kwds):
        if iterable is not None:
            if isinstance(iterable, dict):
                for k, c in iterable.items():
                    self[k] = self.get(k, 0) - c
            else:
                for k in iterable:
                    self[k] = self.get(k, 0) - 1
        if kwds:
            self.subtract(kwds)

    def copy(self):
        return type(self)(self)

    __copy__ = copy

    def __delitem__(self, k):
        if k in self:
            dict.__delitem__(self, k)

    def __repr__(self):
        if not self:
            return f"{type(self).__name__}()"
        try:
            d = dict(self.most_common())
        except TypeError:
            d = dict(self)
        return f"{type(self).__name__}({d!r})"

    def __eq__(self, other):
        if not isinstance(other, Counter):
            return NotImplemented
        return all(self[e] == other[e] for c in (self, other) for e in c)

    def __ne__(self, other):
        if not isinstance(other, Counter):
            return NotImplemented
        return not self == other

    def __le__(self, other):
        if not isinstance(other, Counter):
            return NotImplemented
        return all(self[e] <= other[e] for c in (self, other) for e in c)

    def __lt__(self, other):
        if not isinstance(other, Counter):
            return NotImplemented
        return self <= other and self != other

    def __ge__(self, other):
        if not isinstance(other, Counter):
            return NotImplemented
        return all(self[e] >= other[e] for c in (self, other) for e in c)

    def __gt__(self, other):
        if not isinstance(other, Counter):
            return NotImplemented
        return self >= other and self != other

    __hash__ = None

    def __add__(self, other):
        if not isinstance(other, Counter):
            return NotImplemented
        r = Counter()
        for e, c in self.items():
            n = c + other[e]
            if n > 0:
                r[e] = n
        for e, c in other.items():
            if e not in self and c > 0:
                r[e] = c
        return r

    def __sub__(self, other):
        if not isinstance(other, Counter):
            return NotImplemented
        r = Counter()
        for e, c in self.items():
            n = c - other[e]
            if n > 0:
                r[e] = n
        for e, c in other.items():
            if e not in self and c < 0:
                r[e] = 0 - c
        return r

    def __or__(self, other):
        if not isinstance(other, Counter):
            return NotImplemented
        r = Counter()
        for e, c in self.items():
            o = other[e]
            n = o if c < o else c
            if n > 0:
                r[e] = n
        for e, c in other.items():
            if e not in self and c > 0:
                r[e] = c
        return r

    def __and__(self, other):
        if not isinstance(other, Counter):
            return NotImplemented
        r = Counter()
        for e, c in self.items():
            o = other[e]
            n = c if c < o else o
            if n > 0:
                r[e] = n
        return r

    def __pos__(self):
        r = Counter()
        for e, c in self.items():
            if c > 0:
                r[e] = c
        return r

    def __neg__(self):
        r = Counter()
        for e, c in self.items():
            if c < 0:
                r[e] = 0 - c
        return r

    def _keep_positive(self):
        for e in [e for e, c in self.items() if not c > 0]:
            dict.__delitem__(self, e)
        return self

    def __iadd__(self, other):
        for e, c in other.items():
            self[e] += c
        return self._keep_positive()

    def __isub__(self, other):
        for e, c in other.items():
            self[e] -= c
        return self._keep_positive()

    def __ior__(self, other):
        for e, c in other.items():
            if c > self[e]:
                self[e] = c
        return self._keep_positive()

    def __iand__(self, other):
        for e, c in self.items():
            o = other[e]
            if o < c:
                self[e] = o
        return self._keep_positive()


def namedtuple(typename, field_names, *, rename=False, defaults=None, module=None):
    if isinstance(field_names, str):
        field_names = field_names.replace(",", " ").split()
    field_names = list(map(str, field_names))
    typename = str(typename)
    if rename:
        seen = set()
        for i, name in enumerate(field_names):
            if (not name.isidentifier() or _iskeyword(name) or name.startswith("_") or name in seen):
                field_names[i] = f"_{i}"
            seen.add(name)
    for name in [typename] + field_names:
        if not name.isidentifier():
            raise ValueError(f"Type names and field names must be valid identifiers: {name!r}")
        if _iskeyword(name):
            raise ValueError(f"Type names and field names cannot be a keyword: {name!r}")
    seen = set()
    for name in field_names:
        if name.startswith("_") and not rename:
            raise ValueError(f"Field names cannot start with an underscore: {name!r}")
        if name in seen:
            raise ValueError(f"Encountered duplicate field name: {name!r}")
        seen.add(name)
    fields = tuple(field_names)
    nf = len(fields)
    field_defaults = {}
    if defaults is not None:
        defaults = tuple(defaults)
        if len(defaults) > nf:
            raise TypeError("Got more default values than field names")
        field_defaults = dict(zip(fields[nf - len(defaults):], defaults))

    def __new__(cls, *args, **kwargs):
        if len(args) > nf:
            raise TypeError(f"{typename}.__new__() takes {nf + 1} positional arguments but {len(args) + 1} were given")
        vals = list(args)
        for name in fields[len(args):]:
            if name in kwargs:
                vals.append(kwargs.pop(name))
            elif name in field_defaults:
                vals.append(field_defaults[name])
            else:
                missing = [n for n in fields[len(vals):] if n not in kwargs and n not in field_defaults]
                if len(missing) == 1:
                    raise TypeError(f"{typename}.__new__() missing 1 required positional argument: '{missing[0]}'")
                names = ", ".join(repr(m) for m in missing[:-1]) + f" and {missing[-1]!r}"
                raise TypeError(f"{typename}.__new__() missing {len(missing)} required positional arguments: {names}")
        if kwargs:
            k = next(iter(kwargs))
            if k in fields:
                raise TypeError(f"{typename}.__new__() got multiple values for argument '{k}'")
            raise TypeError(f"{typename}.__new__() got an unexpected keyword argument '{k}'")
        return tuple.__new__(cls, vals)

    @classmethod
    def _make(cls, iterable):
        r = tuple.__new__(cls, iterable)
        if len(r) != nf:
            raise TypeError(f"Expected {nf} arguments, got {len(r)}")
        return r

    def _replace(self, **kwds):
        r = self._make([kwds.pop(n) if n in kwds else getattr(self, n) for n in fields])
        if kwds:
            raise ValueError(f"Got unexpected field names: {list(kwds)!r}")
        return r

    def __repr__(self):
        return typename + "(" + ", ".join(f"{n}={v!r}" for n, v in zip(fields, self)) + ")"

    def _asdict(self):
        return dict(zip(fields, self))

    def __getnewargs__(self):
        return tuple(self)

    ns = {
        "__doc__": f"{typename}({', '.join(fields)})",
        "__slots__": (),
        "_fields": fields,
        "_field_defaults": field_defaults,
        "__new__": __new__,
        "_make": _make,
        "_replace": _replace,
        "__repr__": __repr__,
        "_asdict": _asdict,
        "__getnewargs__": __getnewargs__,
        "__match_args__": fields,
    }
    for i, name in enumerate(fields):
        ns[name] = property(_itemgetter(i), doc=f"Alias for field number {i}")
    result = type(typename, (tuple,), ns)
    if module is None:
        try:
            module = _sys._getframe(1).f_globals.get("__name__", "__main__")
        except (AttributeError, ValueError):
            module = "__main__"
    result.__module__ = module
    return result
