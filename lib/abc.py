# SPDX-License-Identifier: MIT
# abc for the Ranger Python runtime: abstract methods are recorded, and a
# class with any left unimplemented refuses to be instantiated.

def abstractmethod(funcobj):
    funcobj.__isabstractmethod__ = True
    return funcobj


def _abstracts(cls):
    names = set()
    for klass in reversed(cls.__mro__):
        for name, value in klass.__dict__.items():
            if getattr(value, "__isabstractmethod__", False):
                names.add(name)
            elif name in names:
                names.discard(name)
    return names


class ABC:
    def __new__(cls, *args, **kwargs):
        missing = sorted(_abstracts(cls))
        if missing:
            plural = "s" if len(missing) > 1 else ""
            raise TypeError("Can't instantiate abstract class %s without an implementation for abstract method%s %s" % (cls.__name__, plural, ", ".join("'" + m + "'" for m in missing)))
        return object.__new__(cls)


ABCMeta = type
abstractproperty = abstractmethod
abstractclassmethod = abstractmethod
abstractstaticmethod = abstractmethod
