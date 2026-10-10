# SPDX-License-Identifier: MIT
# copy and deepcopy for the Rust runtime, after CPython's copy module.

__all__ = ["Error", "copy", "deepcopy", "replace"]


class Error(Exception):
    pass


error = Error

_atomic = (type(None), int, float, bool, complex, str, bytes, type, range,
           type(Ellipsis), type(NotImplemented), type(len), type(lambda: 0), property)


def copy(x):
    cls = type(x)
    if cls in _atomic or cls is frozenset or cls is tuple:
        return x
    if cls is list:
        return x[:]
    if cls is dict:
        return x.copy()
    if cls is set:
        return x.copy()
    if cls is bytearray:
        return bytearray(x)
    if issubclass(cls, type):
        return x
    cp = getattr(cls, "__copy__", None)
    if cp is not None:
        return cp(x)
    rv = _reduce(x)
    if isinstance(rv, str):
        return x
    return _reconstruct(x, None, *rv)


def _reduce(x):
    cls = type(x)
    reductor = getattr(cls, "__reduce_ex__", None)
    if reductor is not None:
        try:
            return reductor(x, 4)
        except TypeError:
            pass
    reductor = getattr(x, "__reduce__", None)
    if reductor is not None:
        return reductor()
    raise Error(f"un(deep)copyable object of type {cls}")


def deepcopy(x, memo=None, _nil=[]):
    d = id(x)
    if memo is None:
        memo = {}
    else:
        y = memo.get(d, _nil)
        if y is not _nil:
            return y
    cls = type(x)
    if cls in _atomic:
        return x
    if cls is list:
        y = []
        memo[d] = y
        for a in x:
            y.append(deepcopy(a, memo))
    elif cls is tuple:
        y = [deepcopy(a, memo) for a in x]
        try:
            return memo[d]
        except KeyError:
            pass
        for k, j in zip(x, y):
            if k is not j:
                y = tuple(y)
                break
        else:
            y = x
    elif cls is dict:
        y = {}
        memo[d] = y
        for key, value in x.items():
            y[deepcopy(key, memo)] = deepcopy(value, memo)
    elif cls is set:
        y = {deepcopy(a, memo) for a in x}
    elif cls is frozenset:
        y = frozenset(deepcopy(a, memo) for a in x)
    elif cls is bytearray:
        y = bytearray(x)
    elif issubclass(cls, type):
        y = x
    else:
        copier = getattr(x, "__deepcopy__", None)
        if copier is not None:
            y = copier(memo)
        else:
            rv = _reduce(x)
            if isinstance(rv, str):
                y = x
            else:
                y = _reconstruct(x, memo, *rv)
    if y is not x:
        memo[d] = y
        _keep_alive(x, memo)
    return y


def _keep_alive(x, memo):
    try:
        memo[id(memo)].append(x)
    except KeyError:
        memo[id(memo)] = [x]


def _reconstruct(x, memo, func, args, state=None, listiter=None, dictiter=None, *, deepcopy=deepcopy):
    deep = memo is not None
    if deep and args:
        args = tuple(deepcopy(arg, memo) for arg in args)
    y = func(*args)
    if deep:
        memo[id(x)] = y
    if state is not None:
        if deep:
            state = deepcopy(state, memo)
        if hasattr(y, "__setstate__"):
            y.__setstate__(state)
        else:
            if isinstance(state, tuple) and len(state) == 2:
                state, slotstate = state
            else:
                slotstate = None
            if state is not None:
                y.__dict__.update(state)
            if slotstate is not None:
                for key, value in slotstate.items():
                    setattr(y, key, value)
    if listiter is not None:
        if deep:
            for item in listiter:
                y.append(deepcopy(item, memo))
        else:
            for item in listiter:
                y.append(item)
    if dictiter is not None:
        if deep:
            for key, value in dictiter:
                y[deepcopy(key, memo)] = deepcopy(value, memo)
        else:
            for key, value in dictiter:
                y[key] = value
    return y


def replace(obj, /, **changes):
    cls = obj.__class__
    func = getattr(cls, "__replace__", None)
    if func is None:
        raise TypeError(f"replace() does not support {cls.__name__} objects")
    return func(obj, **changes)
