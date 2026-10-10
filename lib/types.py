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
