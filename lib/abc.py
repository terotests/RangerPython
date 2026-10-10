# SPDX-License-Identifier: MIT
# abc for the Ranger Python runtime: ABCMeta keeps the abstract methods a
# class leaves unimplemented (the runtime refuses to instantiate it) and
# answers isinstance()/issubclass() for registered classes.


def abstractmethod(funcobj):
    funcobj.__isabstractmethod__ = True
    return funcobj


def abstractclassmethod(callable):
    callable.__isabstractmethod__ = True
    return classmethod(callable)


def abstractstaticmethod(callable):
    callable.__isabstractmethod__ = True
    return staticmethod(callable)


def abstractproperty(fget=None, fset=None, fdel=None, doc=None):
    if fget is not None:
        fget.__isabstractmethod__ = True
    return property(fget, fset, fdel, doc)


def _is_abstract(value):
    return bool(getattr(value, "__isabstractmethod__", False))


class ABCMeta(type):
    def __new__(mcls, name, bases, namespace, /, **kwargs):
        cls = super().__new__(mcls, name, bases, namespace, **kwargs)
        abstracts = {key for key, value in namespace.items() if _is_abstract(value)}
        for base in bases:
            for key in getattr(base, "__abstractmethods__", ()):
                if _is_abstract(getattr(cls, key, None)):
                    abstracts.add(key)
        cls.__abstractmethods__ = frozenset(abstracts)
        cls._abc_registry = []
        return cls

    def register(cls, subclass):
        if not isinstance(subclass, type):
            raise TypeError("Can only register classes")
        if issubclass(subclass, cls):
            return subclass
        if issubclass(cls, subclass):
            raise RuntimeError("Refusing to create an inheritance cycle")
        cls._abc_registry.append(subclass)
        return subclass

    def __instancecheck__(cls, instance):
        sub = instance.__class__ if hasattr(instance, "__class__") else type(instance)
        if cls.__subclasscheck__(sub):
            return True
        t = type(instance)
        return t is not sub and cls.__subclasscheck__(t)

    def __subclasscheck__(cls, subclass):
        if not isinstance(subclass, type):
            raise TypeError("issubclass() arg 1 must be a class")
        mro = getattr(subclass, "__mro__", (subclass,))
        if cls in mro:
            return True
        hook = cls.__dict__.get("__subclasshook__")
        if hook is None:
            for base in cls.__mro__:
                if "__subclasshook__" in base.__dict__:
                    hook = base.__dict__["__subclasshook__"]
                    break
        if hook is not None:
            fn = getattr(cls, "__subclasshook__")
            ok = fn(subclass)
            if ok is not NotImplemented:
                return bool(ok)
        for rcls in cls.__dict__.get("_abc_registry", ()):
            if rcls is subclass or issubclass(subclass, rcls):
                return True
        for scls in _subclasses(cls):
            if scls is not cls and issubclass(subclass, scls):
                return True
        return False

    def _dump_registry(cls, file=None):
        print("Class: %s.%s" % (cls.__module__, cls.__qualname__), file=file)

    def _abc_registry_clear(cls):
        cls._abc_registry = []

    def _abc_caches_clear(cls):
        pass


_all_abcs = []


def _subclasses(cls):
    return [c for c in _all_abcs if c is not cls and cls in c.__mro__]


_orig_new = ABCMeta.__new__


def _tracking_new(mcls, name, bases, namespace, /, **kwargs):
    cls = _orig_new(mcls, name, bases, namespace, **kwargs)
    _all_abcs.append(cls)
    return cls


ABCMeta.__new__ = _tracking_new


class ABC(metaclass=ABCMeta):
    __slots__ = ()


def get_cache_token():
    return 0


def update_abstractmethods(cls):
    if not hasattr(cls, "__abstractmethods__"):
        return cls
    abstracts = set()
    for scls in cls.__bases__:
        for name in getattr(scls, "__abstractmethods__", ()):
            if _is_abstract(getattr(cls, name, None)):
                abstracts.add(name)
    for name, value in cls.__dict__.items():
        if _is_abstract(value):
            abstracts.add(name)
    cls.__abstractmethods__ = frozenset(abstracts)
    return cls
