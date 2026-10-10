# SPDX-License-Identifier: MIT
# weakref for the Ranger Python runtime. Values here live as long as
# anything holds them, so a weak reference is a strong one that never dies.


class ref:
    __slots__ = ("_obj", "_callback")

    def __init__(self, obj, callback=None):
        self._obj = obj
        self._callback = callback

    def __call__(self):
        return self._obj

    def __eq__(self, other):
        if isinstance(other, ref):
            return self._obj is other._obj or self._obj == other._obj
        return NotImplemented

    def __hash__(self):
        return hash(self._obj)

    def __repr__(self):
        return "<weakref; to '%s'>" % type(self._obj).__name__

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
