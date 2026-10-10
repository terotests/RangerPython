# SPDX-License-Identifier: MIT
# functools for the Ranger Python runtime: reduce, lru_cache, cache, wraps,
# total_ordering and cmp_to_key are native (_functools); the rest is here.
from _functools import reduce, lru_cache, cache, total_ordering, cmp_to_key
from _functools import partial as _native_partial

__all__ = ["update_wrapper", "wraps", "WRAPPER_ASSIGNMENTS", "WRAPPER_UPDATES", "total_ordering",
           "cache", "cmp_to_key", "lru_cache", "reduce", "partial", "partialmethod",
           "singledispatch", "singledispatchmethod", "cached_property"]

WRAPPER_ASSIGNMENTS = ("__module__", "__name__", "__qualname__", "__doc__", "__annotations__", "__type_params__")
WRAPPER_UPDATES = ("__dict__",)


def wraps(wrapped, assigned=WRAPPER_ASSIGNMENTS, updated=WRAPPER_UPDATES):
    def deco(wrapper):
        return update_wrapper(wrapper, wrapped, assigned, updated)
    return deco


def update_wrapper(wrapper, wrapped, assigned=WRAPPER_ASSIGNMENTS, updated=WRAPPER_UPDATES):
    for attr in assigned:
        try:
            value = getattr(wrapped, attr)
        except AttributeError:
            pass
        else:
            try:
                setattr(wrapper, attr, value)
            except (AttributeError, TypeError):
                pass
    for attr in updated:
        d = getattr(wrapped, attr, None)
        if isinstance(d, dict):
            for k, v in d.items():
                try:
                    setattr(wrapper, k, v)
                except (AttributeError, TypeError):
                    pass
    wrapper.__wrapped__ = wrapped
    return wrapper


class partial:
    __slots__ = ("func", "args", "keywords", "__dict__", "__weakref__")

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
        qualname = type(self).__qualname__
        module = type(self).__module__
        args = [repr(self.func)]
        args.extend(repr(x) for x in self.args)
        args.extend(f"{k}={v!r}" for (k, v) in self.keywords.items())
        if module == "functools" or module == "__main__" and qualname == "partial":
            return f"functools.{qualname}({', '.join(args)})"
        return f"{module}.{qualname}({', '.join(args)})"

    def __get__(self, obj, objtype=None):
        return self

    def __reduce__(self):
        return type(self), (self.func,), (self.func, self.args, self.keywords or None, self.__dict__ or None)

    def __setstate__(self, state):
        func, args, kwds, namespace = state
        self.func = func
        self.args = tuple(args)
        self.keywords = dict(kwds) if kwds else {}
        if namespace:
            self.__dict__.update(namespace)

    def __class_getitem__(cls, item):
        import types
        return types.GenericAlias(cls, item)


partial.__module__ = "functools"


class partialmethod:
    def __init__(self, func, /, *args, **keywords):
        if isinstance(func, partialmethod):
            self.func = func.func
            self.args = func.args + args
            self.keywords = {**func.keywords, **keywords}
        else:
            self.func = func
            self.args = args
            self.keywords = keywords

    def __get__(self, obj, cls=None):
        if obj is None:
            return self
        func = self.func

        def bound(*args, **keywords):
            return func(obj, *self.args, *args, **{**self.keywords, **keywords})
        return bound

    @property
    def __isabstractmethod__(self):
        return getattr(self.func, "__isabstractmethod__", False)


class _lru_list_wrapper:
    pass


def _find_impl(cls, registry):
    for base in getattr(cls, "__mro__", (cls,)):
        if base in registry:
            return registry[base]
    if cls is bool and int in registry:
        return registry[int]
    for typ, impl in registry.items():
        try:
            if typ is not object and isinstance(typ, type) and issubclass(cls, typ):
                return impl
        except TypeError:
            pass
    return registry.get(object)


def singledispatch(func):
    registry = {}
    registry[object] = func

    def dispatch(cls):
        return _find_impl(cls, registry)

    def register(cls, func=None):
        if func is None:
            if isinstance(cls, type):
                return lambda f: register(cls, f)
            ann = getattr(cls, "__annotations__", {})
            if not ann:
                raise TypeError("Invalid first argument to `register()`: %r. Use either `@register(some_class)` or plain `@register` on an annotated function." % (cls,))
            func = cls
            argname, cls = next(iter(ann.items()))
            if isinstance(cls, str):
                cls = func.__globals__.get(cls, cls) if hasattr(func, "__globals__") else cls
        registry[cls] = func
        return func

    def wrapper(*args, **kw):
        if not args:
            raise TypeError(f"{funcname} requires at least 1 positional argument")
        return dispatch(args[0].__class__ if hasattr(args[0], "__class__") else type(args[0]))(*args, **kw)

    funcname = getattr(func, "__name__", "singledispatch function")
    wrapper.register = register
    wrapper.dispatch = dispatch
    wrapper.registry = registry
    wrapper._clear_cache = lambda: None
    update_wrapper(wrapper, func)
    return wrapper


class singledispatchmethod:
    def __init__(self, func):
        self.dispatcher = singledispatch(func)
        self.func = func

    def register(self, cls, method=None):
        return self.dispatcher.register(cls, func=method)

    def __get__(self, obj, cls=None):
        def _method(*args, **kwargs):
            method = self.dispatcher.dispatch(args[0].__class__)
            return method.__get__(obj, cls)(*args, **kwargs) if hasattr(method, "__get__") and obj is not None else method(obj, *args, **kwargs)
        _method.register = self.register
        update_wrapper(_method, self.func)
        return _method


_NOT_FOUND = object()


class cached_property:
    def __init__(self, func):
        self.func = func
        self.attrname = None
        self.__doc__ = func.__doc__
        self.__module__ = getattr(func, "__module__", None)

    def __set_name__(self, owner, name):
        if self.attrname is None:
            self.attrname = name
        elif name != self.attrname:
            raise TypeError("Cannot assign the same cached_property to two different names (%r and %r)." % (self.attrname, name))

    def __get__(self, instance, owner=None):
        if instance is None:
            return self
        if self.attrname is None:
            raise TypeError("Cannot use cached_property instance without calling __set_name__ on it.")
        cache = instance.__dict__
        val = cache.get(self.attrname, _NOT_FOUND)
        if val is _NOT_FOUND:
            val = self.func(instance)
            cache[self.attrname] = val
        return val
