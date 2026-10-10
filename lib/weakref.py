# SPDX-License-Identifier: MIT
# weakref for the Ranger Python runtime. A ref to an instance, function or
# class really is weak (it reads None once the object is gone; callbacks never
# run); to anything else it is a strong one that never dies.
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


def proxy(obj, callback=None):
    return obj


ProxyType = type(None)
CallableProxyType = type(None)
ProxyTypes = (ProxyType, CallableProxyType)


def getweakrefcount(obj):
    return 0


def getweakrefs(obj):
    return []


class WeakMethod(ref):
    __slots__ = ()


class WeakValueDictionary(dict):
    def valuerefs(self):
        return [ref(v) for v in self.values()]

    def itervaluerefs(self):
        return iter(self.valuerefs())


class WeakKeyDictionary(dict):
    def keyrefs(self):
        return [ref(k) for k in self.keys()]


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
