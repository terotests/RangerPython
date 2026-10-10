# SPDX-License-Identifier: MIT
# collections.abc for the Ranger Python runtime. There are no metaclasses
# here: each ABC answers isinstance()/issubclass() through its own
# __instancecheck__/__subclasscheck__, by real inheritance, by register(),
# or (for the one-method ABCs) by the methods a class has.

_abcs = []
_ITERATORS = ("generator", "map", "filter", "zip", "enumerate", "reversed",
              "islice", "chain", "count", "cycle", "repeat", "takewhile",
              "dropwhile", "filterfalse", "accumulate", "starmap", "zip_longest",
              "product", "permutations", "combinations", "groupby", "pairwise",
              "batched", "compress", "tee", "<genexpr>")


def _mro(sub):
    return getattr(sub, "__mro__", None) or (sub,)


def _is_iter_type(sub):
    name = getattr(sub, "__name__", "")
    return isinstance(name, str) and (name.endswith("iterator") or name in _ITERATORS)


def _has(sub, names):
    for name in names:
        if name in ("__iter__", "__next__") and _is_iter_type(sub):
            continue
        found = False
        for base in _mro(sub):
            d = getattr(base, "__dict__", None)
            if isinstance(d, dict) and name in d:
                if d[name] is None:
                    return False
                found = True
                break
        if not found:
            if getattr(sub, name, None) is None:
                return False
    return True


def _hashable(sub):
    if not isinstance(getattr(sub, "__dict__", None), dict):
        return getattr(sub, "__hash__", None) is not None
    for base in _mro(sub):
        d = getattr(base, "__dict__", None)
        if not isinstance(d, dict):
            continue
        if "__hash__" in d:
            return d["__hash__"] is not None
        if "__eq__" in d:
            return False
    return True


def _check(cls, sub):
    if sub is cls:
        return True
    for base in _mro(sub):
        if base is cls:
            return True
    for abc in _abcs:
        if cls in _mro(abc):
            for r in abc.__dict__.get("_registry", ()):
                if sub is r or r in _mro(sub):
                    return True
    hook = cls.__dict__.get("_hook")
    if hook is not None:
        return hook(sub)
    return False


def _abc(cls, hook=None):
    cls._registry = []
    cls._hook = hook

    def instancecheck(c, obj):
        return _check(c, type(obj))

    def subclasscheck(c, sub):
        return _check(c, sub)

    def register(c, sub):
        c._registry.append(sub)
        return sub
    cls.__instancecheck__ = classmethod(instancecheck)
    cls.__subclasscheck__ = classmethod(subclasscheck)
    cls.register = classmethod(register)
    _abcs.append(cls)
    return cls


class Hashable:
    __slots__ = ()

    def __hash__(self):
        return 0


class Awaitable:
    __slots__ = ()

    def __await__(self):
        yield


class Coroutine(Awaitable):
    __slots__ = ()


class AsyncIterable:
    __slots__ = ()


class AsyncIterator(AsyncIterable):
    __slots__ = ()


class AsyncGenerator(AsyncIterator):
    __slots__ = ()


class Iterable:
    __slots__ = ()

    def __iter__(self):
        while False:
            yield None

    def __class_getitem__(cls, item):
        return cls


class Iterator(Iterable):
    __slots__ = ()

    def __next__(self):
        raise StopIteration

    def __iter__(self):
        return self


class Reversible(Iterable):
    __slots__ = ()


class Generator(Iterator):
    __slots__ = ()

    def __next__(self):
        return self.send(None)

    def close(self):
        try:
            self.throw(GeneratorExit)
        except (GeneratorExit, StopIteration):
            pass
        else:
            raise RuntimeError("generator ignored GeneratorExit")


class Sized:
    __slots__ = ()

    def __len__(self):
        return 0


class Container:
    __slots__ = ()

    def __contains__(self, x):
        return False


class Collection(Sized, Iterable, Container):
    __slots__ = ()


class Buffer:
    __slots__ = ()


class Callable:
    __slots__ = ()

    def __call__(self, *args, **kwds):
        return False


class Set(Collection):
    __slots__ = ()

    def __le__(self, other):
        if not isinstance(other, Set):
            return NotImplemented
        if len(self) > len(other):
            return False
        for elem in self:
            if elem not in other:
                return False
        return True

    def __lt__(self, other):
        if not isinstance(other, Set):
            return NotImplemented
        return len(self) < len(other) and self.__le__(other)

    def __gt__(self, other):
        if not isinstance(other, Set):
            return NotImplemented
        return len(self) > len(other) and self.__ge__(other)

    def __ge__(self, other):
        if not isinstance(other, Set):
            return NotImplemented
        if len(self) < len(other):
            return False
        for elem in other:
            if elem not in self:
                return False
        return True

    def __eq__(self, other):
        if not isinstance(other, Set):
            return NotImplemented
        return len(self) == len(other) and self.__le__(other)

    @classmethod
    def _from_iterable(cls, it):
        return cls(it)

    def __and__(self, other):
        if not isinstance(other, Iterable):
            return NotImplemented
        return self._from_iterable(value for value in other if value in self)

    __rand__ = __and__

    def isdisjoint(self, other):
        for value in other:
            if value in self:
                return False
        return True

    def __or__(self, other):
        if not isinstance(other, Iterable):
            return NotImplemented
        chain = [e for s in (self, other) for e in s]
        return self._from_iterable(chain)

    __ror__ = __or__

    def __sub__(self, other):
        if not isinstance(other, Set):
            if not isinstance(other, Iterable):
                return NotImplemented
            other = self._from_iterable(other)
        return self._from_iterable(value for value in self if value not in other)

    def __rsub__(self, other):
        if not isinstance(other, Set):
            if not isinstance(other, Iterable):
                return NotImplemented
            other = self._from_iterable(other)
        return self._from_iterable(value for value in other if value not in self)

    def __xor__(self, other):
        if not isinstance(other, Set):
            if not isinstance(other, Iterable):
                return NotImplemented
            other = self._from_iterable(other)
        return (self - other) | (other - self)

    __rxor__ = __xor__

    def _hash(self):
        h = 0
        for x in self:
            h ^= hash(x)
        return h


class MutableSet(Set):
    __slots__ = ()

    def add(self, value):
        raise NotImplementedError

    def discard(self, value):
        raise NotImplementedError

    def remove(self, value):
        if value not in self:
            raise KeyError(value)
        self.discard(value)

    def pop(self):
        it = iter(self)
        try:
            value = next(it)
        except StopIteration:
            raise KeyError from None
        self.discard(value)
        return value

    def clear(self):
        try:
            while True:
                self.pop()
        except KeyError:
            pass

    def __ior__(self, it):
        for value in it:
            self.add(value)
        return self

    def __iand__(self, it):
        for value in (self - it):
            self.discard(value)
        return self

    def __ixor__(self, it):
        if it is self:
            self.clear()
        else:
            if not isinstance(it, Set):
                it = self._from_iterable(it)
            for value in it:
                if value in self:
                    self.discard(value)
                else:
                    self.add(value)
        return self

    def __isub__(self, it):
        if it is self:
            self.clear()
        else:
            for value in it:
                self.discard(value)
        return self


class Mapping(Collection):
    __slots__ = ()

    def get(self, key, default=None):
        try:
            return self[key]
        except KeyError:
            return default

    def __contains__(self, key):
        try:
            self[key]
        except KeyError:
            return False
        else:
            return True

    def keys(self):
        return KeysView(self)

    def items(self):
        return ItemsView(self)

    def values(self):
        return ValuesView(self)

    def __eq__(self, other):
        if not isinstance(other, Mapping):
            return NotImplemented
        return dict(self.items()) == dict(other.items())

    __reversed__ = None


class MappingView(Sized):
    __slots__ = ("_mapping",)

    def __init__(self, mapping):
        self._mapping = mapping

    def __len__(self):
        return len(self._mapping)

    def __repr__(self):
        return "{0.__class__.__name__}({0._mapping!r})".format(self)


class KeysView(MappingView, Set):
    __slots__ = ()

    @classmethod
    def _from_iterable(cls, it):
        return set(it)

    def __contains__(self, key):
        return key in self._mapping

    def __iter__(self):
        yield from self._mapping


class ItemsView(MappingView, Set):
    __slots__ = ()

    @classmethod
    def _from_iterable(cls, it):
        return set(it)

    def __contains__(self, item):
        key, value = item
        try:
            v = self._mapping[key]
        except KeyError:
            return False
        else:
            return v is value or v == value

    def __iter__(self):
        for key in self._mapping:
            yield (key, self._mapping[key])


class ValuesView(MappingView, Collection):
    __slots__ = ()

    def __contains__(self, value):
        for key in self._mapping:
            v = self._mapping[key]
            if v is value or v == value:
                return True
        return False

    def __iter__(self):
        for key in self._mapping:
            yield self._mapping[key]


class MutableMapping(Mapping):
    __slots__ = ()

    __marker = object()

    def pop(self, key, default=__marker):
        try:
            value = self[key]
        except KeyError:
            if default is self.__marker:
                raise
            return default
        else:
            del self[key]
            return value

    def popitem(self):
        try:
            key = next(iter(self))
        except StopIteration:
            raise KeyError from None
        value = self[key]
        del self[key]
        return key, value

    def clear(self):
        try:
            while True:
                self.popitem()
        except KeyError:
            pass

    def update(self, other=(), /, **kwds):
        if isinstance(other, Mapping):
            for key in other:
                self[key] = other[key]
        elif hasattr(other, "keys"):
            for key in other.keys():
                self[key] = other[key]
        else:
            for key, value in other:
                self[key] = value
        for key, value in kwds.items():
            self[key] = value

    def setdefault(self, key, default=None):
        try:
            return self[key]
        except KeyError:
            self[key] = default
        return default


class Sequence(Reversible, Collection):
    __slots__ = ()

    def __iter__(self):
        i = 0
        try:
            while True:
                v = self[i]
                yield v
                i += 1
        except IndexError:
            return

    def __contains__(self, value):
        for v in self:
            if v is value or v == value:
                return True
        return False

    def __reversed__(self):
        for i in reversed(range(len(self))):
            yield self[i]

    def index(self, value, start=0, stop=None):
        if start is not None and start < 0:
            start = max(len(self) + start, 0)
        if stop is not None and stop < 0:
            stop += len(self)
        i = start
        while stop is None or i < stop:
            try:
                v = self[i]
            except IndexError:
                break
            if v is value or v == value:
                return i
            i += 1
        raise ValueError

    def count(self, value):
        return sum(1 for v in self if v is value or v == value)


class ByteString(Sequence):
    __slots__ = ()


class MutableSequence(Sequence):
    __slots__ = ()

    def insert(self, index, value):
        raise IndexError

    def append(self, value):
        self.insert(len(self), value)

    def clear(self):
        try:
            while True:
                self.pop()
        except IndexError:
            pass

    def reverse(self):
        n = len(self)
        for i in range(n // 2):
            self[i], self[n - i - 1] = self[n - i - 1], self[i]

    def extend(self, values):
        if values is self:
            values = list(values)
        for v in values:
            self.append(v)

    def pop(self, index=-1):
        v = self[index]
        del self[index]
        return v

    def remove(self, value):
        del self[self.index(value)]

    def __iadd__(self, values):
        self.extend(values)
        return self


_abc(Hashable, _hashable)
_abc(Awaitable, lambda s: _has(s, ("__await__",)))
_abc(Coroutine, lambda s: _has(s, ("send", "throw", "close", "__await__")))
_abc(AsyncIterable, lambda s: _has(s, ("__aiter__",)))
_abc(AsyncIterator, lambda s: _has(s, ("__aiter__", "__anext__")))
_abc(AsyncGenerator, lambda s: _has(s, ("__aiter__", "__anext__", "asend", "athrow", "aclose")))
_abc(Iterable, lambda s: _has(s, ("__iter__",)) or _has(s, ("__getitem__",)) and False)
_abc(Iterator, lambda s: _has(s, ("__iter__", "__next__")))
_abc(Reversible, lambda s: _has(s, ("__reversed__", "__iter__")))
_abc(Generator, lambda s: _has(s, ("__iter__", "__next__", "send", "throw", "close")))
_abc(Sized, lambda s: _has(s, ("__len__",)))
_abc(Container, lambda s: _has(s, ("__contains__",)))
_abc(Collection, lambda s: _has(s, ("__len__", "__iter__", "__contains__")))
_abc(Buffer, lambda s: _has(s, ("__buffer__",)))
_abc(Callable, lambda s: _has(s, ("__call__",)) or callable(s) and getattr(s, "__name__", "") in ("function", "builtin_function_or_method", "method"))
_abc(Set)
_abc(MutableSet)
_abc(Mapping)
_abc(MappingView)
_abc(KeysView)
_abc(ItemsView)
_abc(ValuesView)
_abc(MutableMapping)
_abc(Sequence)
_abc(ByteString)
_abc(MutableSequence)

Sequence.register(tuple)
Sequence.register(str)
Sequence.register(range)
MutableSequence.register(list)
Set.register(frozenset)
MutableSet.register(set)
MutableMapping.register(dict)
Reversible.register(dict)
