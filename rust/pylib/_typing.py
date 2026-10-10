# SPDX-License-Identifier: MIT
# _typing for the Ranger Python runtime: the classes CPython 3.13 implements
# in C (Objects/typevarobject.c), written in Python.  Most of the work is
# delegated to helpers in typing.py, exactly as the C code does.

import sys


def _typing(name):
    import typing
    return getattr(typing, name)


def _caller(depth=2):
    try:
        return sys._getframe(depth).f_globals.get("__name__")
    except (AttributeError, ValueError):
        return None


def _type_check(arg, msg):
    if arg is None:
        return type(None)
    return _typing("_type_check")(arg, msg)


def _idfunc(_, x):
    return x


class _NoDefaultType:
    """The type of the NoDefault singleton."""

    __slots__ = ()

    def __new__(cls, *args, **kwargs):
        if args or kwargs:
            raise TypeError("NoDefaultType takes no arguments")
        return NoDefault

    def __repr__(self):
        return "typing.NoDefault"

    def __reduce__(self):
        return "NoDefault"


_NoDefaultType.__name__ = _NoDefaultType.__qualname__ = "NoDefaultType"
NoDefault = object.__new__(_NoDefaultType)


def _no_subclass(name):
    def __init_subclass__(cls, *args, **kwargs):
        raise TypeError("type 'typing.%s' is not an acceptable base type" % name)
    return classmethod(__init_subclass__)


def _set_module(obj, module):
    if module is not None:
        object.__setattr__(obj, "__module__", module)


class TypeVar:
    """Type variable."""

    def __new__(cls, name, *constraints, bound=None, covariant=False,
                contravariant=False, infer_variance=False, default=NoDefault):
        if not isinstance(name, str):
            raise TypeError("TypeVar() argument 'name' must be str, not %s" % type(name).__name__)
        if covariant and contravariant:
            raise ValueError("Bivariant type variables are not supported.")
        if infer_variance and (covariant or contravariant):
            raise ValueError("Variance cannot be specified with infer_variance.")
        if bound is not None:
            bound = _type_check(bound, "Bound must be a type.")
        if len(constraints) == 1:
            raise TypeError("A single constraint is not allowed")
        if constraints and bound is not None:
            raise TypeError("Constraints cannot be combined with bound=...")
        if default is None:
            default = type(None)
        self = object.__new__(cls)
        d = self.__dict__
        d["__name__"] = name
        d["__bound__"] = bound
        d["__constraints__"] = tuple(constraints)
        d["__covariant__"] = bool(covariant)
        d["__contravariant__"] = bool(contravariant)
        d["__infer_variance__"] = bool(infer_variance)
        d["__default__"] = default
        _set_module(self, _caller())
        return self

    __init_subclass__ = _no_subclass("TypeVar")

    def __setattr__(self, name, value):
        if name in ("__name__", "__bound__", "__constraints__", "__covariant__",
                    "__contravariant__", "__infer_variance__", "__default__"):
            raise AttributeError("readonly attribute")
        object.__setattr__(self, name, value)

    def __repr__(self):
        if self.__infer_variance__:
            prefix = ""
        elif self.__covariant__:
            prefix = "+"
        elif self.__contravariant__:
            prefix = "-"
        else:
            prefix = "~"
        return prefix + self.__name__

    def __or__(self, right):
        return _typing("_make_union")(self, right)

    def __ror__(self, left):
        return _typing("_make_union")(left, self)

    def __typing_subst__(self, arg):
        return _typing("_typevar_subst")(self, arg)

    def __typing_prepare_subst__(self, alias, args):
        params = alias.__parameters__
        i = params.index(self)
        if i < len(args):
            return args
        if i == len(args) and self.has_default():
            return (*args, self.__default__)
        raise TypeError("Too few arguments for %s; actual %d, expected at least %d"
                        % (alias, len(args), i + 1))

    def has_default(self):
        return self.__default__ is not NoDefault

    def __reduce__(self):
        return self.__name__

    def __mro_entries__(self, bases):
        raise TypeError("Cannot subclass an instance of TypeVar")


class ParamSpecArgs:
    """The args for a ParamSpec object."""

    def __init__(self, origin):
        self.__origin__ = origin

    __init_subclass__ = _no_subclass("ParamSpecArgs")

    def __repr__(self):
        origin = self.__origin__
        if isinstance(origin, ParamSpec):
            return "%s.args" % origin.__name__
        return "%r.args" % (origin,)

    def __eq__(self, other):
        if not isinstance(other, ParamSpecArgs):
            return NotImplemented
        return self.__origin__ == other.__origin__

    __hash__ = None

    def __mro_entries__(self, bases):
        raise TypeError("Cannot subclass an instance of ParamSpecArgs")


class ParamSpecKwargs:
    """The kwargs for a ParamSpec object."""

    def __init__(self, origin):
        self.__origin__ = origin

    __init_subclass__ = _no_subclass("ParamSpecKwargs")

    def __repr__(self):
        origin = self.__origin__
        if isinstance(origin, ParamSpec):
            return "%s.kwargs" % origin.__name__
        return "%r.kwargs" % (origin,)

    def __eq__(self, other):
        if not isinstance(other, ParamSpecKwargs):
            return NotImplemented
        return self.__origin__ == other.__origin__

    __hash__ = None

    def __mro_entries__(self, bases):
        raise TypeError("Cannot subclass an instance of ParamSpecKwargs")


class ParamSpec:
    """Parameter specification variable."""

    def __new__(cls, name, *, bound=None, covariant=False, contravariant=False,
                infer_variance=False, default=NoDefault):
        if not isinstance(name, str):
            raise TypeError("ParamSpec() argument 'name' must be str, not %s" % type(name).__name__)
        if covariant and contravariant:
            raise ValueError("Bivariant type variables are not supported.")
        if infer_variance and (covariant or contravariant):
            raise ValueError("Variance cannot be specified with infer_variance.")
        if bound is not None:
            bound = _type_check(bound, "Bound must be a type.")
        if default is None:
            default = type(None)
        self = object.__new__(cls)
        d = self.__dict__
        d["__name__"] = name
        d["__bound__"] = bound
        d["__covariant__"] = bool(covariant)
        d["__contravariant__"] = bool(contravariant)
        d["__infer_variance__"] = bool(infer_variance)
        d["__default__"] = default
        _set_module(self, _caller())
        return self

    __init_subclass__ = _no_subclass("ParamSpec")
    __setattr__ = TypeVar.__setattr__
    __repr__ = TypeVar.__repr__
    __or__ = TypeVar.__or__
    __ror__ = TypeVar.__ror__
    has_default = TypeVar.has_default
    __reduce__ = TypeVar.__reduce__

    @property
    def args(self):
        return ParamSpecArgs(self)

    @property
    def kwargs(self):
        return ParamSpecKwargs(self)

    def __typing_subst__(self, arg):
        return _typing("_paramspec_subst")(self, arg)

    def __typing_prepare_subst__(self, alias, args):
        return _typing("_paramspec_prepare_subst")(self, alias, args)

    def __mro_entries__(self, bases):
        raise TypeError("Cannot subclass an instance of ParamSpec")


class TypeVarTuple:
    """Type variable tuple."""

    def __new__(cls, name, *, default=NoDefault):
        if not isinstance(name, str):
            raise TypeError("TypeVarTuple() argument 'name' must be str, not %s" % type(name).__name__)
        self = object.__new__(cls)
        self.__dict__["__name__"] = name
        self.__dict__["__default__"] = default
        _set_module(self, _caller())
        return self

    __init_subclass__ = _no_subclass("TypeVarTuple")

    def __setattr__(self, name, value):
        if name in ("__name__", "__default__"):
            raise AttributeError("readonly attribute")
        object.__setattr__(self, name, value)

    def __repr__(self):
        return self.__name__

    def __iter__(self):
        yield _typing("Unpack")[self]

    def __typing_subst__(self, arg):
        raise TypeError("Substitution of bare TypeVarTuple is not supported")

    def __typing_prepare_subst__(self, alias, args):
        return _typing("_typevartuple_prepare_subst")(self, alias, args)

    has_default = TypeVar.has_default
    __reduce__ = TypeVar.__reduce__

    def __mro_entries__(self, bases):
        raise TypeError("Cannot subclass an instance of TypeVarTuple")


class TypeAliasType:
    """Type alias."""

    def __new__(cls, name, value, *, type_params=()):
        if not isinstance(name, str):
            raise TypeError("TypeAliasType.__new__() argument 'name' must be str, not %s" % type(name).__name__)
        if not isinstance(type_params, tuple):
            raise TypeError("type_params must be a tuple")
        seen_default = False
        for p in type_params:
            if not isinstance(p, (TypeVar, ParamSpec, TypeVarTuple)):
                raise TypeError("Expected a type param, got %r" % (p,))
            if p.has_default():
                seen_default = True
            elif seen_default:
                raise TypeError("non-default type parameter '%r' follows default type parameter" % (p,))
        self = object.__new__(cls)
        d = self.__dict__
        d["__name__"] = name
        d["__value__"] = value
        d["__type_params__"] = type_params
        d["__module__"] = _caller()
        return self

    __init_subclass__ = _no_subclass("TypeAliasType")

    def __setattr__(self, name, value):
        if name in ("__name__", "__value__", "__type_params__", "__parameters__", "__module__"):
            raise AttributeError("readonly attribute")
        raise AttributeError("'typing.TypeAliasType' object has no attribute '%s'" % name)

    def __delattr__(self, name):
        raise AttributeError("readonly attribute")

    @property
    def __parameters__(self):
        unpack = None
        out = []
        for p in self.__type_params__:
            if isinstance(p, TypeVarTuple):
                if unpack is None:
                    unpack = _typing("Unpack")
                out.append(unpack[p])
            else:
                out.append(p)
        return tuple(out)

    def __getitem__(self, parameters):
        if not self.__type_params__:
            raise TypeError("Only generic type aliases are subscriptable")
        import types
        return types.GenericAlias(self, parameters)

    def __repr__(self):
        return self.__name__

    def __reduce__(self):
        return self.__name__

    def __or__(self, right):
        return _typing("_make_union")(self, right)

    def __ror__(self, left):
        return _typing("_make_union")(left, self)

    def __mro_entries__(self, bases):
        raise TypeError("Cannot subclass an instance of TypeAliasType")


class Generic:
    """Abstract base class for generic types."""

    __slots__ = ()

    def __class_getitem__(cls, params):
        return _typing("_generic_class_getitem")(cls, params)

    def __init_subclass__(cls, *args, **kwargs):
        return _typing("_generic_init_subclass")(cls, *args, **kwargs)


for _c in (TypeVar, ParamSpecArgs, ParamSpecKwargs, ParamSpec, TypeVarTuple, TypeAliasType, Generic):
    _c.__module__ = "typing"
_NoDefaultType.__module__ = "typing"
del _c
