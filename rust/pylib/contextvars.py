# SPDX-License-Identifier: MIT
# contextvars for the Rust runtime. There is one thread and no asyncio task
# switching between contexts, so a context is a plain mapping that run()
# swaps in.

__all__ = ("Context", "ContextVar", "Token", "copy_context")

_MISSING = object()


class Context:
    def __init__(self):
        self._data = {}

    def run(self, callable, /, *args, **kwargs):
        global _current
        prev = _current
        _current = self
        try:
            return callable(*args, **kwargs)
        finally:
            _current = prev

    def copy(self):
        c = Context()
        c._data = dict(self._data)
        return c

    def __getitem__(self, var):
        return self._data[var]

    def get(self, var, default=None):
        return self._data.get(var, default)

    def __contains__(self, var):
        return var in self._data

    def __iter__(self):
        return iter(self._data)

    def __len__(self):
        return len(self._data)

    def keys(self):
        return self._data.keys()

    def values(self):
        return self._data.values()

    def items(self):
        return self._data.items()


_current = Context()


def copy_context():
    return _current.copy()


class Token:
    MISSING = _MISSING

    def __init__(self, context, var, old_value):
        self._context = context
        self._var = var
        self._old_value = old_value
        self._used = False

    @property
    def var(self):
        return self._var

    @property
    def old_value(self):
        return self._old_value

    def __class_getitem__(cls, item):
        return cls


class ContextVar:
    def __init__(self, name, *, default=_MISSING):
        if not isinstance(name, str):
            raise TypeError("context variable name must be a str")
        self._name = name
        self._default = default

    @property
    def name(self):
        return self._name

    def get(self, *args):
        try:
            return _current._data[self]
        except KeyError:
            pass
        if args:
            return args[0]
        if self._default is not _MISSING:
            return self._default
        raise LookupError(self)

    def set(self, value):
        old = _current._data.get(self, _MISSING)
        _current._data[self] = value
        return Token(_current, self, old)

    def reset(self, token):
        if token._used:
            raise RuntimeError("Token has already been used once")
        if token._var is not self:
            raise ValueError("Token was created by a different ContextVar")
        if token._context is not _current:
            raise ValueError("Token was created in a different Context")
        if token._old_value is _MISSING:
            _current._data.pop(self, None)
        else:
            _current._data[self] = token._old_value
        token._used = True

    def __repr__(self):
        return f"<ContextVar name={self._name!r} at {id(self):#x}>"

    def __hash__(self):
        return id(self)

    def __class_getitem__(cls, item):
        return cls
