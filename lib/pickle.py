# SPDX-License-Identifier: MIT
# pickle for the Ranger Python runtime: dumps() keeps a deep copy of the
# value behind a small bytes key, and loads() copies it back out.
import copy as _copy

HIGHEST_PROTOCOL = 5
DEFAULT_PROTOCOL = 5
_store = {}


class PickleError(Exception):
    pass


class PicklingError(PickleError):
    pass


class UnpicklingError(PickleError):
    pass


def dumps(obj, protocol=None, *, fix_imports=True, buffer_callback=None):
    key = len(_store)
    _store[key] = _copy.deepcopy(obj)
    return ("\x80pickle:%d." % key).encode("latin-1")


def loads(data, /, *, fix_imports=True, encoding="ASCII", errors="strict", buffers=()):
    text = data if isinstance(data, str) else data.decode("latin-1")
    if not text.startswith("\x80pickle:"):
        raise UnpicklingError("invalid load key")
    return _copy.deepcopy(_store[int(text[8:-1])])


def dump(obj, file, protocol=None, **kw):
    file.write(dumps(obj, protocol))


def load(file, **kw):
    return loads(file.read())
