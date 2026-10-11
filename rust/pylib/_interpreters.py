# SPDX-License-Identifier: MIT
# _interpreters (PEP 734's low-level module) for the Rust runtime, over the
# subinterpreters of _rpyinterp: each one a Vm of its own, whose objects
# never cross; what crosses is the data of shareable objects.
import _rpyinterp
from types import CodeType as _CodeType, SimpleNamespace as _NS

__all__ = []

WHENCE_UNKNOWN = 0
WHENCE_RUNTIME = 1
WHENCE_LEGACY_CAPI = 2
WHENCE_CAPI = 3
WHENCE_XI = 4
WHENCE_STDLIB = 5


class InterpreterError(Exception):
    """A cross-interpreter operation failed"""


class InterpreterNotFoundError(InterpreterError):
    """An interpreter was not found"""


class NotShareableError(ValueError):
    pass


for _cls in (InterpreterError, InterpreterNotFoundError, NotShareableError):
    _cls.__module__ = "interpreters"
del _cls


# the types other modules taught to cross: type -> obj -> (module, function, args)
_reducers = {}


def _register_xi(cls, reduce):
    _reducers[cls] = reduce


def _reduce(obj):
    reduce = _reducers.get(type(obj))
    if reduce is None:
        raise NotShareableError(f"{obj!r} does not support cross-interpreter data")
    return reduce(obj)


def is_shareable(obj):
    """Return True if the object's data may be shared between interpreters."""
    cls = type(obj)
    if cls is tuple:
        return all(is_shareable(item) for item in obj)
    return cls in (type(None), bool, int, float, str, bytes) or cls in _reducers


# reference counts of the interpreters made with reqrefs, by id
_refs = {}
_whence = {}


def _id(id):
    if isinstance(id, bool) or not hasattr(type(id), "__index__"):
        raise TypeError(f"interpreter ID must be an int, got {type(id).__name__}")
    id = id.__index__()
    if id < 0:
        raise ValueError(f"interpreter ID must be a non-negative int, got {id}")
    return id


def _check(id, restrict=False):
    id = _id(id)
    status = _rpyinterp.status(id)
    if status is None:
        raise InterpreterNotFoundError(f"unrecognized interpreter ID {id}")
    if restrict and id == 0:
        raise InterpreterError("cannot modify the main interpreter")
    return id


def new_config(name="isolated", /, **overrides):
    if name not in ("isolated", "legacy", "empty", ""):
        raise ValueError(f"unsupported config name '{name}'")
    config = _NS(use_main_obmalloc=name == "legacy", allow_fork=name == "legacy",
                 allow_exec=name == "legacy", allow_threads=True,
                 allow_daemon_threads=name == "legacy", check_multi_interp_extensions=name != "legacy",
                 gil="shared" if name == "legacy" else "own")
    for k, v in overrides.items():
        if not hasattr(config, k):
            raise ValueError(f"unsupported field {k!r}")
        setattr(config, k, v)
    return config


def create(config="isolated", *, reqrefs=False):
    """Create a new interpreter and return a unique generated ID."""
    if isinstance(config, str):
        new_config(config)
    id = _rpyinterp.create()
    _whence[id] = WHENCE_STDLIB
    if reqrefs:
        _refs[id] = 0
    return id


def destroy(id, *, restrict=False):
    """Destroy the identified interpreter."""
    id = _check(id)
    if id == 0:
        raise InterpreterError("cannot destroy the main interpreter")
    if id == _rpyinterp.current():
        raise InterpreterError("cannot destroy the current interpreter")
    if _rpyinterp.status(id):
        raise InterpreterError("cannot destroy a running interpreter")
    _rpyinterp.destroy(id)
    _refs.pop(id, None)
    _whence.pop(id, None)


def list_all(*, require_ready=False):
    """Return a list containing the ID of every existing interpreter."""
    return [(id, whence(id)) for id in _rpyinterp.list_all()]


def get_current():
    """Return (ID, whence) of the current interpreter."""
    id = _rpyinterp.current()
    return id, whence(id)


def get_main():
    """Return (ID, whence) of the main interpreter."""
    return 0, WHENCE_RUNTIME


def whence(id):
    """Return an identifier for where the interpreter was created."""
    id = _check(id)
    if id == 0:
        return WHENCE_RUNTIME
    # one made by another interpreter's _interpreters is still the module's
    return _whence.get(id, WHENCE_STDLIB)


def get_config(id, *, restrict=False):
    _check(id, restrict)
    return new_config("isolated")


def incref(id, *, implieslink=False, restrict=False):
    id = _check(id, restrict)
    if id in _refs:
        _refs[id] += 1


def decref(id, *, restrict=False):
    id = _check(id, restrict)
    if id in _refs:
        _refs[id] -= 1
        if _refs[id] <= 0 and not _rpyinterp.status(id):
            destroy(id)


def is_running(id, *, restrict=False):
    """Return whether or not the identified interpreter is running."""
    return bool(_rpyinterp.status(_check(id, restrict)))


def set___main___attrs(id, updates, *, restrict=False):
    """Bind the given attributes in the interpreter's __main__ module."""
    id = _check(id, restrict)
    if not isinstance(updates, dict):
        raise TypeError(f"updates must be a dict, got {updates!r}")
    for value in updates.values():
        if not is_shareable(value):
            raise NotShareableError(f"{value!r} does not support cross-interpreter data")
    if _rpyinterp.status(id):
        raise InterpreterError("interpreter already running")
    _rpyinterp.set_main_attrs(id, updates, _reduce)


def _excinfo(info):
    name, qualname, module, msg, formatted, errdisplay = info
    return _NS(type=_NS(__name__=name, __qualname__=qualname, __module__=module),
               msg=msg, formatted=formatted, errdisplay=errdisplay)


def _code_of(code):
    if isinstance(code, str):
        return code
    func = code
    code = getattr(func, "__code__", code)
    if not isinstance(code, _CodeType):
        raise TypeError(f"expected a str, code object or function, got {type(func).__name__}")
    if func is not code:
        if func.__closure__:
            raise ValueError("closures not supported")
        if code.co_argcount or code.co_kwonlyargcount or code.co_flags & 0xc:
            raise ValueError("arguments not supported")
    return code


def _run(id, code, restrict):
    id = _check(id, restrict)
    code = _code_of(code)
    if _rpyinterp.status(id):
        raise InterpreterError("interpreter already running")
    info = _rpyinterp.exec(id, code)
    return None if info is None else _excinfo(info)


def exec(id, code, shared=None, *, restrict=False):
    """Execute the provided code in the identified interpreter.

    Return None, or the description of the exception it did not catch.
    """
    if shared is not None:
        set___main___attrs(id, shared, restrict=restrict)
    return _run(id, code, restrict)


def call(id, callable, args=None, kwargs=None, *, restrict=False):
    """Call the function (with no arguments and no closure) in the interpreter."""
    if args or kwargs:
        raise ValueError("arguments not supported")
    return _run(id, callable, restrict)


def run_string(id, script, shared=None, *, restrict=False):
    if not isinstance(script, str):
        raise TypeError(f"expected str, got {type(script).__name__}")
    return exec(id, script, shared, restrict=restrict)


def run_func(id, func, shared=None, *, restrict=False):
    return exec(id, func, shared, restrict=restrict)


def capture_exception(exc=None):
    """Return a snapshot of an exception, by default the one being handled."""
    import sys
    import traceback
    if exc is None:
        exc = sys.exception()
        if exc is None:
            return None
    elif not isinstance(exc, BaseException):
        raise TypeError(f"expected exception, got {exc!r}")
    cls = type(exc)
    msg = str(exc)
    formatted = f"{cls.__name__}: {msg}" if msg else cls.__name__
    errdisplay = "".join(traceback.format_exception(exc)).rstrip("\n")
    return _excinfo((cls.__name__, cls.__qualname__, cls.__module__, msg, formatted, errdisplay))
