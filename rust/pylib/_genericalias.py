# SPDX-License-Identifier: MIT
# types.GenericAlias and types.UnionType for the Ranger Python runtime,
# following CPython 3.13's Objects/genericaliasobject.c and unionobject.c.
# The interpreter builds list[int] and int | str through this module.

# attributes a GenericAlias answers itself; every other one is the origin's
_EXCEPTIONS = frozenset({
    "__class__", "__bases__", "__origin__", "__args__", "__unpacked__",
    "__parameters__", "__typing_unpacked_tuple_args__", "__mro_entries__",
    "__reduce_ex__", "__reduce__", "__copy__", "__deepcopy__", "_ga_params",
})


def _repr_item(p):
    if p is ...:
        return "..."
    try:
        if hasattr(p, "__origin__") and hasattr(p, "__args__"):
            return repr(p)
        qualname = getattr(p, "__qualname__", None)
        if qualname is None:
            return repr(p)
        module = getattr(p, "__module__", None)
    except AttributeError:
        return repr(p)
    if module is None:
        return repr(p)
    if module == "builtins":
        return str(qualname)
    return "%s.%s" % (module, qualname)


def _is_typevar_like(t):
    return not isinstance(t, type) and hasattr(t, "__typing_subst__")


def _make_parameters(args):
    """the type variables in args, in order of first appearance"""
    params = []
    for t in args:
        if isinstance(t, type):
            continue
        if hasattr(t, "__typing_subst__"):
            if t not in params:
                params.append(t)
            continue
        sub = getattr(t, "__parameters__", None)
        if sub is None and isinstance(t, (tuple, list)):
            sub = _make_parameters(t)
        if isinstance(sub, tuple):
            for t2 in sub:
                if t2 not in params:
                    params.append(t2)
    return tuple(params)


def _unpacked_tuple_args(arg):
    if isinstance(arg, GenericAlias) and arg.__unpacked__ and arg.__origin__ is tuple:
        return arg.__args__
    return getattr(arg, "__typing_unpacked_tuple_args__", None)


def _unpack_args(item):
    items = item if isinstance(item, tuple) else (item,)
    out = []
    for x in items:
        if not isinstance(x, type):
            sub = _unpacked_tuple_args(x)
            if isinstance(sub, tuple) and not (sub and sub[-1] is ...):
                out.extend(sub)
                continue
        out.append(x)
    return tuple(out)


def _is_unpacked_typevartuple(arg):
    return not isinstance(arg, type) and bool(getattr(arg, "__typing_is_unpacked_typevartuple__", False))


def _subs_tvars(obj, params, argitems):
    subparams = getattr(obj, "__parameters__", None)
    if not (isinstance(subparams, tuple) and subparams):
        return obj
    subargs = []
    for arg in subparams:
        if arg in params:
            i = params.index(arg)
            param = params[i]
            arg = argitems[i]
            if hasattr(type(param), "__iter__") and isinstance(arg, tuple):
                subargs.extend(arg)
                continue
        subargs.append(arg)
    return obj[tuple(subargs)]


def _subs_args(args, params, argitems):
    newargs = []
    for arg in args:
        if isinstance(arg, type):
            newargs.append(arg)
            continue
        if isinstance(arg, (tuple, list)):
            sub = _subs_args(arg, params, argitems)
            newargs.append(sub if isinstance(arg, tuple) else list(sub))
            continue
        unpack = _is_unpacked_typevartuple(arg)
        subst = getattr(arg, "__typing_subst__", None)
        if subst is not None:
            arg = subst(argitems[params.index(arg)])
        else:
            arg = _subs_tvars(arg, params, argitems)
        if unpack:
            newargs.extend(arg)
        else:
            newargs.append(arg)
    return tuple(newargs)


def _subs_parameters(self, args, params, item):
    """args with the type variables params replaced by the values in item"""
    if not params:
        raise TypeError("%r is not a generic class" % (self,))
    item = _unpack_args(item)
    for param in params:
        prepare = getattr(param, "__typing_prepare_subst__", None)
        if prepare is not None:
            item = prepare(self, item if isinstance(item, tuple) else (item,))
    argitems = item if isinstance(item, tuple) else (item,)
    if len(argitems) != len(params):
        raise TypeError("Too %s arguments for %r; actual %d, expected %d"
                        % ("many" if len(argitems) > len(params) else "few", self, len(argitems), len(params)))
    return _subs_args(args, params, argitems)


def _new_alias(cls, origin, args, starred=False):
    self = object.__new__(cls)
    if not isinstance(args, tuple):
        args = (args,)
    object.__setattr__(self, "__origin__", origin)
    object.__setattr__(self, "__args__", args)
    object.__setattr__(self, "__unpacked__", starred)
    object.__setattr__(self, "_ga_params", None)
    return self


class GenericAlias:
    """Represent a PEP 585 generic type

E.g. for t = list[int], t.__origin__ is list and t.__args__ is (int,)."""

    def __new__(cls, origin, args, /, **kwargs):
        if kwargs and cls is GenericAlias:
            raise TypeError("GenericAlias() takes no keyword arguments")
        return _new_alias(cls, origin, args)

    def __getattribute__(self, name):
        if name in _EXCEPTIONS:
            return object.__getattribute__(self, name)
        return getattr(object.__getattribute__(self, "__origin__"), name)

    def __setattr__(self, name, value):
        if name in ("__origin__", "__args__", "__unpacked__", "__parameters__"):
            raise AttributeError("readonly attribute")
        raise AttributeError("'types.GenericAlias' object has no attribute '%s' and no __dict__ for setting new attributes" % name)

    def __repr__(self):
        args = self.__args__
        parts = []
        for p in args:
            if type(p) is list:
                parts.append("[%s]" % ", ".join(_repr_item(x) for x in p))
            else:
                parts.append(_repr_item(p))
        star = "*" if self.__unpacked__ else ""
        return "%s%s[%s]" % (star, _repr_item(self.__origin__), ", ".join(parts) if args else "()")

    def __hash__(self):
        return hash(self.__origin__) ^ hash(self.__args__)

    def __eq__(self, other):
        if not isinstance(other, GenericAlias):
            return NotImplemented
        if self.__unpacked__ != other.__unpacked__:
            return False
        return self.__origin__ == other.__origin__ and self.__args__ == other.__args__

    def __ne__(self, other):
        r = self.__eq__(other)
        return r if r is NotImplemented else not r

    def __call__(self, *args, **kwargs):
        obj = self.__origin__(*args, **kwargs)
        try:
            obj.__orig_class__ = self
        except (AttributeError, TypeError):
            pass
        return obj

    def __mro_entries__(self, bases):
        return (self.__origin__,)

    def __instancecheck__(self, obj):
        raise TypeError("isinstance() argument 2 cannot be a parameterized generic")

    def __subclasscheck__(self, cls):
        raise TypeError("issubclass() argument 2 cannot be a parameterized generic")

    def __reduce__(self):
        if self.__unpacked__:
            return next, (iter(GenericAlias(self.__origin__, self.__args__)),)
        return type(self), (self.__origin__, self.__args__)

    def __dir__(self):
        names = dir(self.__origin__)
        for n in sorted(_EXCEPTIONS - {"_ga_params"}):
            if n not in names:
                names.append(n)
        return names

    def __iter__(self):
        return _GenericAliasIterator(self)

    @property
    def __parameters__(self):
        p = object.__getattribute__(self, "_ga_params")
        if p is None:
            p = _make_parameters(self.__args__)
            object.__setattr__(self, "_ga_params", p)
        return p

    @property
    def __typing_unpacked_tuple_args__(self):
        if self.__unpacked__ and self.__origin__ is tuple:
            return self.__args__
        return None

    def __getitem__(self, item):
        newargs = _subs_parameters(self, self.__args__, self.__parameters__, item)
        return _new_alias(GenericAlias, self.__origin__, newargs, self.__unpacked__)

    def __or__(self, other):
        return _union_or(self, other)

    def __ror__(self, other):
        return _union_or(other, self)


GenericAlias.__module__ = "types"


class _GenericAliasIterator:
    """iter(list[int]): yields *list[int] once"""

    def __init__(self, alias):
        self._obj = alias

    def __iter__(self):
        return self

    def __next__(self):
        obj = self._obj
        if obj is None:
            raise StopIteration
        self._obj = None
        return _new_alias(GenericAlias, obj.__origin__, obj.__args__, True)

    def __reduce__(self):
        # look iter up first: that may run code that exhausts this iterator
        it = iter
        if self._obj is not None:
            return it, (self._obj,)
        return it, ((),)


_GenericAliasIterator.__name__ = _GenericAliasIterator.__qualname__ = "generic_alias_iterator"


def _union_repr_item(p):
    if p is _NoneType:
        return "None"
    return _repr_item(p)


class UnionType:
    """Represent a PEP 604 union type

E.g. for int | str"""

    def __new__(cls, *args, **kwargs):
        raise TypeError("cannot create 'types.UnionType' instances")

    def __init_subclass__(cls, **kwargs):
        raise TypeError("type 'types.UnionType' is not an acceptable base type")

    def __setattr__(self, name, value):
        if name in ("__args__", "__parameters__"):
            raise AttributeError("readonly attribute")
        raise AttributeError("'types.UnionType' object has no attribute '%s' and no __dict__ for setting new attributes" % name)

    def __repr__(self):
        return " | ".join(_union_repr_item(p) for p in self.__args__)

    def __hash__(self):
        return hash(frozenset(self.__args__))

    def __eq__(self, other):
        if not isinstance(other, UnionType):
            return NotImplemented
        return set(self.__args__) == set(other.__args__)

    def __ne__(self, other):
        if not isinstance(other, UnionType):
            return NotImplemented
        return set(self.__args__) != set(other.__args__)

    @property
    def __parameters__(self):
        p = object.__getattribute__(self, "_ga_params")
        if p is None:
            p = _make_parameters(self.__args__)
            object.__setattr__(self, "_ga_params", p)
        return p

    def __getitem__(self, item):
        newargs = _subs_parameters(self, self.__args__, self.__parameters__, item)
        if not newargs:
            return _make_union(newargs)
        res = newargs[0]
        for arg in newargs[1:]:
            res = res | arg
        return res

    def __or__(self, other):
        return _union_or(self, other)

    def __ror__(self, other):
        return _union_or(other, self)


UnionType.__module__ = "types"
_NoneType = type(None)
_TypeAliasType = None


def _is_unionable(obj):
    global _TypeAliasType
    if obj is None or isinstance(obj, (type, GenericAlias, UnionType)):
        return True
    if _TypeAliasType is None:
        import _typing
        _TypeAliasType = _typing.TypeAliasType
    return isinstance(obj, _TypeAliasType)


def _make_union(args):
    flat = []
    for a in args:
        if isinstance(a, UnionType):
            flat.extend(a.__args__)
        elif a is None:
            flat.append(_NoneType)
        else:
            flat.append(a)
    new = []
    for a in flat:
        for b in new:
            if isinstance(a, GenericAlias) and isinstance(b, GenericAlias):
                if a == b:
                    break
            elif a is b:
                break
        else:
            new.append(a)
    if len(new) == 1:
        return new[0]
    self = object.__new__(UnionType)
    object.__setattr__(self, "__args__", tuple(new))
    object.__setattr__(self, "_ga_params", None)
    return self


def _union_or(a, b):
    """a | b on types, or NotImplemented"""
    if not _is_unionable(a) or not _is_unionable(b):
        return NotImplemented
    return _make_union((a, b))


def _generic_alias(origin, args):
    return _new_alias(GenericAlias, origin, args)
