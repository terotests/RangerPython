# SPDX-License-Identifier: MIT
# weakref for the Ranger Python runtime. A ref to an instance, function,
# class, generator, set or native object really is weak (it reads None once
# the object is gone; callbacks never run); to anything else it is a strong
# one that never dies.
from _rpyweak import new as _new, get as _get


class ref:
    __slots__ = ("_obj", "_callback", "_hash")

    def __init__(self, obj, callback=None):
        self._obj = _new(obj)
        self._callback = callback

    def __call__(self):
        return _get(self._obj)

    def __eq__(self, other):
        if isinstance(other, ref):
            a, b = self(), other()
            if a is None or b is None:
                return self is other
            return a is b or a == b
        return NotImplemented

    def __ne__(self, other):
        r = self.__eq__(other)
        return r if r is NotImplemented else not r

    def __hash__(self):
        try:
            return self._hash
        except AttributeError:
            pass
        o = self()
        if o is None:
            raise TypeError("weak object has gone away")
        self._hash = hash(o)
        return self._hash

    def __repr__(self):
        o = self()
        if o is None:
            return "<weakref at %#x; dead>" % id(self)
        return "<weakref at %#x; to '%s' at %#x>" % (id(self), type(o).__name__, id(o))

    __callback__ = property(lambda self: self._callback)


ReferenceType = ref


def _referent(p):
    o = _get(object.__getattribute__(p, "_ref"))
    if o is None:
        raise ReferenceError("weakly-referenced object no longer exists")
    return o


class ProxyType:
    """A proxy forwards every operation to its referent while it lives."""
    __slots__ = ("_ref", "_callback")

    def __init__(self, obj, callback=None):
        object.__setattr__(self, "_ref", _new(obj))
        object.__setattr__(self, "_callback", callback)

    def __getattribute__(self, name):
        return getattr(_referent(self), name)

    def __setattr__(self, name, value):
        setattr(_referent(self), name, value)

    def __delattr__(self, name):
        delattr(_referent(self), name)

    def __repr__(self):
        o = _referent(self)
        return "<weakproxy at %#x; to '%s' at %#x>" % (id(self), type(o).__name__, id(o))

    def __str__(self):
        return str(_referent(self))

    def __bytes__(self):
        return bytes(_referent(self))

    def __bool__(self):
        return bool(_referent(self))

    def __hash__(self):
        raise TypeError("unhashable type: 'weakref.ProxyType'")

    def __len__(self):
        return len(_referent(self))

    def __iter__(self):
        return iter(_referent(self))

    def __next__(self):
        return next(_referent(self))

    def __contains__(self, item):
        return item in _referent(self)

    def __getitem__(self, key):
        return _referent(self)[key]

    def __setitem__(self, key, value):
        _referent(self)[key] = value

    def __delitem__(self, key):
        del _referent(self)[key]

    def __reversed__(self):
        return reversed(_referent(self))


def _unwrap(o):
    return _referent(o) if isinstance(o, ProxyType) else o


def _forward(name, op):
    def method(self, *args):
        return op(_referent(self), *map(_unwrap, args))
    method.__name__ = name
    setattr(ProxyType, name, method)


import operator as _op
for _name, _fn in [("__lt__", _op.lt), ("__le__", _op.le), ("__eq__", _op.eq),
                   ("__ne__", _op.ne), ("__gt__", _op.gt), ("__ge__", _op.ge),
                   ("__neg__", _op.neg), ("__pos__", _op.pos), ("__abs__", abs),
                   ("__invert__", _op.invert), ("__int__", int), ("__float__", float),
                   ("__index__", _op.index)]:
    _forward(_name, _fn)
for _name in ("add", "sub", "mul", "matmul", "truediv", "floordiv", "mod", "pow",
              "lshift", "rshift", "and", "or", "xor"):
    _fn = getattr(_op, _name + "_" if _name in ("and", "or") else _name)
    _forward("__%s__" % _name, _fn)
    _forward("__r%s__" % _name, lambda a, b, _fn=_fn: _fn(b, a))
    _forward("__i%s__" % _name, getattr(_op, "i" + _name, _fn))
_forward("__divmod__", divmod)
_forward("__rdivmod__", lambda a, b: divmod(b, a))
del _name, _fn


class CallableProxyType(ProxyType):
    __slots__ = ()

    def __call__(self, /, *args, **kwargs):
        return _referent(self)(*args, **kwargs)


ProxyTypes = (ProxyType, CallableProxyType)


def proxy(obj, callback=None):
    if isinstance(obj, ProxyType):
        obj = _referent(obj)
    return (CallableProxyType if callable(obj) else ProxyType)(obj, callback)


def getweakrefcount(obj):
    return 0


def getweakrefs(obj):
    return []


class WeakMethod(ref):
    __slots__ = ()


class _WeakMapping:
    # entries whose referent is gone are dropped lazily, on the next access,
    # since callbacks never run here
    def __init__(self, other=(), /, **kw):
        self.data = {}
        self.update(other, **kw)

    def __len__(self):
        self._purge()
        return len(self.data)

    def __iter__(self):
        return iter(self.keys())

    def __contains__(self, key):
        try:
            self[key]
        except (KeyError, TypeError):
            return False
        return True

    def get(self, key, default=None):
        try:
            return self[key]
        except KeyError:
            return default

    def keys(self):
        return [k for k, v in self.items()]

    def values(self):
        return [v for k, v in self.items()]

    def __eq__(self, other):
        if not isinstance(other, (_WeakMapping, dict)):
            return NotImplemented
        return dict(self.items()) == dict(other.items())

    def __ne__(self, other):
        r = self.__eq__(other)
        return r if r is NotImplemented else not r

    __hash__ = None

    def update(self, other=(), /, **kw):
        if hasattr(other, "keys"):
            for k in other.keys():
                self[k] = other[k]
        else:
            for k, v in other:
                self[k] = v
        for k, v in kw.items():
            self[k] = v

    def setdefault(self, key, default=None):
        try:
            return self[key]
        except KeyError:
            self[key] = default
            return default

    def pop(self, key, *args):
        try:
            v = self[key]
        except KeyError:
            if args:
                return args[0]
            raise
        del self[key]
        return v

    def popitem(self):
        for k, v in self.items():
            del self[k]
            return k, v
        raise KeyError("popitem(): dictionary is empty")

    def clear(self):
        self.data.clear()

    def __copy__(self):
        return self.copy()

    def __or__(self, other):
        c = self.copy()
        c.update(other)
        return c

    def __ior__(self, other):
        self.update(other)
        return self

    def __repr__(self):
        return "<%s at %#x>" % (type(self).__name__, id(self))


class WeakValueDictionary(_WeakMapping):
    def _purge(self):
        dead = [k for k, r in self.data.items() if r() is None]
        for k in dead:
            del self.data[k]

    def __getitem__(self, key):
        o = self.data[key]()
        if o is None:
            raise KeyError(key)
        return o

    def __setitem__(self, key, value):
        self.data[key] = ref(value)

    def __delitem__(self, key):
        del self.data[key]

    def items(self):
        out = []
        for k, r in list(self.data.items()):
            v = r()
            if v is not None:
                out.append((k, v))
        return out

    def copy(self):
        new = WeakValueDictionary()
        for k, v in self.items():
            new[k] = v
        return new

    def __deepcopy__(self, memo):
        from copy import deepcopy
        new = self.__class__()
        for k, v in self.items():
            new[deepcopy(k, memo)] = v
        return new

    def valuerefs(self):
        self._purge()
        return list(self.data.values())

    def itervaluerefs(self):
        return iter(self.valuerefs())


class WeakKeyDictionary(_WeakMapping):
    def _purge(self):
        dead = [r for r in self.data if r() is None]
        for r in dead:
            del self.data[r]

    def __getitem__(self, key):
        return self.data[ref(key)]

    def __setitem__(self, key, value):
        self.data[ref(key)] = value

    def __delitem__(self, key):
        del self.data[ref(key)]

    def items(self):
        out = []
        for r, v in list(self.data.items()):
            k = r()
            if k is not None:
                out.append((k, v))
        return out

    def copy(self):
        new = WeakKeyDictionary()
        for k, v in self.items():
            new[k] = v
        return new

    def __deepcopy__(self, memo):
        from copy import deepcopy
        new = self.__class__()
        for k, v in self.items():
            new[k] = deepcopy(v, memo)
        return new

    def keyrefs(self):
        self._purge()
        return list(self.data)


class WeakSet(set):
    pass


class finalize:
    def __init__(self, obj, func, /, *args, **kwargs):
        self._func = func
        self._args = args
        self._kwargs = kwargs
        self.atexit = True
        self._alive = True

    def __call__(self, _=None):
        if self._alive:
            self._alive = False
            return self._func(*self._args, **self._kwargs)

    def detach(self):
        if self._alive:
            self._alive = False
            return (None, self._func, self._args, self._kwargs)

    def peek(self):
        if self._alive:
            return (None, self._func, self._args, self._kwargs)

    @property
    def alive(self):
        return self._alive
