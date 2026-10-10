# SPDX-License-Identifier: MIT
# types for the Ranger Python runtime.

def _f():
    pass

class _C:
    def _m(self):
        pass

FunctionType = type(_f)
LambdaType = type(lambda: None)
MethodType = type(_C()._m)
BuiltinFunctionType = type(len)
BuiltinMethodType = type([].append)
ModuleType = type(__import__("sys"))
NoneType = type(None)
EllipsisType = type(...)
NotImplementedType = type(NotImplemented)
GeneratorType = type((x for x in ()))


class SimpleNamespace:
    def __init__(self, mapping_or_iterable=(), /, **kwargs):
        if isinstance(mapping_or_iterable, dict):
            for k in mapping_or_iterable:
                setattr(self, k, mapping_or_iterable[k])
        else:
            for k, v in mapping_or_iterable:
                setattr(self, k, v)
        for k in kwargs:
            setattr(self, k, kwargs[k])

    def __repr__(self):
        items = ["%s=%r" % (k, v) for k, v in self.__dict__.items()]
        return "namespace(" + ", ".join(items) + ")"

    def __eq__(self, other):
        return isinstance(other, SimpleNamespace) and self.__dict__ == other.__dict__


class MappingProxyType:
    def __init__(self, mapping):
        self._m = mapping

    def __getitem__(self, k):
        return self._m[k]

    def __len__(self):
        return len(self._m)

    def __iter__(self):
        return iter(self._m)

    def __contains__(self, k):
        return k in self._m

    def get(self, k, default=None):
        return self._m.get(k, default)

    def keys(self):
        return self._m.keys()

    def values(self):
        return self._m.values()

    def items(self):
        return self._m.items()

    def copy(self):
        return dict(self._m)


def new_class(name, bases=(), kwds=None, exec_body=None):
    ns = {}
    if exec_body is not None:
        exec_body(ns)
    return type(name, bases, ns)


class GenericAlias:
    def __init__(self, origin, args):
        if not isinstance(args, tuple):
            args = (args,)
        self.__origin__ = origin
        self.__args__ = args
        self.__parameters__ = ()

    def __call__(self, *args, **kwargs):
        return self.__origin__(*args, **kwargs)

    def __getitem__(self, params):
        return GenericAlias(self.__origin__, params)

    def __or__(self, other):
        return UnionType((self, other))

    def __ror__(self, other):
        return UnionType((other, self))

    def __eq__(self, other):
        return isinstance(other, GenericAlias) and self.__origin__ == other.__origin__ and self.__args__ == other.__args__

    def __hash__(self):
        return hash((self.__origin__, self.__args__))

    def __mro_entries__(self, bases):
        return (self.__origin__,)

    def __repr__(self):
        return _repr(self.__origin__) + "[" + ", ".join(_repr(a) for a in self.__args__) + "]"


class UnionType:
    def __init__(self, args):
        self.__args__ = args

    def __or__(self, other):
        return UnionType(self.__args__ + (other,))

    def __repr__(self):
        return " | ".join(_repr(a) for a in self.__args__)


def _repr(obj):
    if obj is ...:
        return "..."
    if obj is None or obj is type(None):
        return "None"
    if isinstance(obj, type):
        if obj.__module__ == "builtins":
            return obj.__qualname__
        return obj.__module__ + "." + obj.__qualname__
    if isinstance(obj, tuple) and not obj:
        return "()"
    return repr(obj)
