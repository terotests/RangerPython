# SPDX-License-Identifier: MIT
# object.__reduce_ex__ and object.__getstate__ for the Rust runtime, after
# CPython's Objects/typeobject.c.
import copyreg


def _slot_values(obj):
    slots = {}
    for name in copyreg._slotnames(type(obj)):
        try:
            slots[name] = getattr(obj, name)
        except AttributeError:
            pass
    return slots


def getstate(obj):
    try:
        d = object.__getattribute__(obj, "__dict__")
    except AttributeError:
        d = None
    state = dict(d) if d else None
    slots = _slot_values(obj)
    if slots:
        return (state, slots)
    return state


def reduce_ex(obj, proto):
    cls = type(obj)
    reduce = getattr(cls, "__reduce__", None)
    if reduce is not None and reduce is not object.__reduce__:
        return obj.__reduce__()
    return common_reduce(obj, proto)


def common_reduce(obj, proto):
    if proto < 2:
        return copyreg._reduce_ex(obj, proto)
    cls = type(obj)
    getnewargs_ex = getattr(obj, "__getnewargs_ex__", None)
    kwargs = None
    if getnewargs_ex is not None:
        r = getnewargs_ex()
        if not isinstance(r, tuple):
            raise TypeError(f"__getnewargs_ex__ should return a tuple, not '{type(r).__name__}'")
        if len(r) != 2:
            raise ValueError(f"__getnewargs_ex__ should return a tuple of length 2, not {len(r)}")
        args, kwargs = r
        if not isinstance(args, tuple):
            raise TypeError(f"first item of the tuple returned by __getnewargs_ex__ must be a tuple, not '{type(args).__name__}'")
        if not isinstance(kwargs, dict):
            raise TypeError(f"second item of the tuple returned by __getnewargs_ex__ must be a dict, not '{type(kwargs).__name__}'")
    else:
        getnewargs = getattr(obj, "__getnewargs__", None)
        if getnewargs is not None:
            args = getnewargs()
            if not isinstance(args, tuple):
                raise TypeError(f"__getnewargs__ should return a tuple, not '{type(args).__name__}'")
        else:
            args = ()
    state = obj.__getstate__()
    listitems = iter(obj) if isinstance(obj, list) else None
    dictitems = iter(obj.items()) if isinstance(obj, dict) else None
    if kwargs:
        return (copyreg.__newobj_ex__, (cls, args, kwargs), state, listitems, dictitems)
    return (copyreg.__newobj__, (cls,) + args, state, listitems, dictitems)
