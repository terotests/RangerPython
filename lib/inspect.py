# SPDX-License-Identifier: MIT
# inspect for the Ranger Python runtime: type tests, members, and
# signatures read from __code__, __defaults__ and __kwdefaults__
import types
import sys

CO_OPTIMIZED = 1
CO_NEWLOCALS = 2
CO_VARARGS = 4
CO_VARKEYWORDS = 8
CO_NESTED = 16
CO_GENERATOR = 32
CO_NOFREE = 64
CO_COROUTINE = 128
CO_ITERABLE_COROUTINE = 256
CO_ASYNC_GENERATOR = 512


def _flags(obj):
    obj = unwrap(obj) if callable(obj) else obj
    code = getattr(obj, "__code__", None)
    return getattr(code, "co_flags", 0) if code is not None else 0


def ismodule(obj):
    return isinstance(obj, types.ModuleType)


def isclass(obj):
    return isinstance(obj, type)


def ismethod(obj):
    return isinstance(obj, types.MethodType)


def isfunction(obj):
    return isinstance(obj, types.FunctionType)


def isgeneratorfunction(obj):
    return isfunction(obj) and bool(_flags(obj) & CO_GENERATOR)


def iscoroutinefunction(obj):
    return (isfunction(obj) or ismethod(obj)) and bool(_flags(obj) & CO_COROUTINE)


def isasyncgenfunction(obj):
    return isfunction(obj) and bool(_flags(obj) & CO_ASYNC_GENERATOR)


def isgenerator(obj):
    return type(obj).__name__ == "generator"


def iscoroutine(obj):
    return type(obj).__name__ == "coroutine"


def isasyncgen(obj):
    return type(obj).__name__ == "async_generator"


def isawaitable(obj):
    return iscoroutine(obj) or hasattr(type(obj), "__await__")


def istraceback(obj):
    return type(obj).__name__ == "traceback"


def isframe(obj):
    return type(obj).__name__ == "frame"


def iscode(obj):
    return type(obj).__name__ == "code"


def isbuiltin(obj):
    return type(obj).__name__ == "builtin_function_or_method"


def isroutine(obj):
    return isbuiltin(obj) or isfunction(obj) or ismethod(obj) or ismethoddescriptor(obj)


def ismethoddescriptor(obj):
    if isclass(obj) or ismethod(obj) or isfunction(obj):
        return False
    tp = type(obj)
    return hasattr(tp, "__get__") and not hasattr(tp, "__set__")


def isdatadescriptor(obj):
    if isclass(obj) or ismethod(obj) or isfunction(obj):
        return False
    tp = type(obj)
    return hasattr(tp, "__set__") or hasattr(tp, "__delete__") or isinstance(obj, property)


def isabstract(obj):
    return bool(getattr(obj, "__abstractmethods__", False))


def unwrap(func, *, stop=None):
    seen = set()
    while hasattr(func, "__wrapped__"):
        if stop is not None and stop(func):
            break
        if id(func) in seen:
            raise ValueError("wrapper loop when unwrapping %r" % (func,))
        seen.add(id(func))
        func = func.__wrapped__
    return func


def getmembers(obj, predicate=None):
    out = []
    for key in dir(obj):
        try:
            value = getattr(obj, key)
        except AttributeError:
            continue
        if predicate is None or predicate(value):
            out.append((key, value))
    out.sort(key=lambda kv: kv[0])
    return out


def getmembers_static(obj, predicate=None):
    return getmembers(obj, predicate)


def getattr_static(obj, attr, default=None):
    for base in ([obj] if not isclass(obj) else []) + list(getattr(obj if isclass(obj) else type(obj), "__mro__", ())):
        d = getattr(base, "__dict__", None)
        if isinstance(d, dict) and attr in d:
            return d[attr]
    if default is not None:
        return default
    raise AttributeError(attr)


def getmro(cls):
    return cls.__mro__


def getdoc(obj):
    doc = getattr(obj, "__doc__", None)
    if not isinstance(doc, str):
        return None
    return cleandoc(doc)


def cleandoc(doc):
    lines = doc.expandtabs().split("\n")
    margin = None
    for line in lines[1:]:
        content = len(line.lstrip())
        if content:
            indent = len(line) - content
            margin = indent if margin is None else min(margin, indent)
    if lines:
        lines[0] = lines[0].lstrip()
    if margin is not None:
        for i in range(1, len(lines)):
            lines[i] = lines[i][margin:]
    while lines and not lines[-1]:
        lines.pop()
    while lines and not lines[0]:
        lines.pop(0)
    return "\n".join(lines)


def getmodule(obj, _filename=None):
    if ismodule(obj):
        return obj
    name = getattr(obj, "__module__", None)
    return sys.modules.get(name) if name else None


def getfile(obj):
    mod = getmodule(obj)
    f = getattr(mod, "__file__", None)
    if f is None:
        raise TypeError("source code not available")
    return f


getsourcefile = getfile


def getsource(obj):
    raise OSError("could not get source code")


def getsourcelines(obj):
    raise OSError("could not get source code")


def currentframe():
    return None


def stack(context=1):
    return []


class _empty:
    pass


class Parameter:
    POSITIONAL_ONLY = 0
    POSITIONAL_OR_KEYWORD = 1
    VAR_POSITIONAL = 2
    KEYWORD_ONLY = 3
    VAR_KEYWORD = 4
    empty = _empty
    _names = ("positional-only", "positional or keyword", "variadic positional", "keyword-only", "variadic keyword")

    def __init__(self, name, kind, *, default=_empty, annotation=_empty):
        self.name = name
        self.kind = kind
        self.default = default
        self.annotation = annotation

    def replace(self, **kw):
        return Parameter(kw.get("name", self.name), kw.get("kind", self.kind),
                         default=kw.get("default", self.default), annotation=kw.get("annotation", self.annotation))

    def __str__(self):
        s = self.name
        if self.annotation is not _empty:
            s += ": " + _annot(self.annotation)
        if self.default is not _empty:
            s += (" = " if self.annotation is not _empty else "=") + repr(self.default)
        if self.kind == 2:
            s = "*" + s
        elif self.kind == 4:
            s = "**" + s
        return s

    def __repr__(self):
        return '<Parameter "%s">' % self

    def __eq__(self, other):
        if not isinstance(other, Parameter):
            return NotImplemented
        return (self.name, self.kind, self.default, self.annotation) == (other.name, other.kind, other.default, other.annotation)

    def __hash__(self):
        return hash((self.name, self.kind))


def _annot(a):
    if isinstance(a, type):
        if a.__module__ == "builtins":
            return a.__qualname__
        return a.__module__ + "." + a.__qualname__
    return repr(a)


class BoundArguments:
    def __init__(self, signature, arguments):
        self.signature = signature
        self.arguments = arguments

    @property
    def args(self):
        out = []
        for p in self.signature.parameters.values():
            if p.kind in (3, 4) or p.name not in self.arguments:
                if p.kind in (3, 4):
                    break
                continue
            if p.kind == 2:
                out.extend(self.arguments[p.name])
            else:
                out.append(self.arguments[p.name])
        return tuple(out)

    @property
    def kwargs(self):
        out = {}
        for p in self.signature.parameters.values():
            if p.name not in self.arguments:
                continue
            if p.kind == 3:
                out[p.name] = self.arguments[p.name]
            elif p.kind == 4:
                out.update(self.arguments[p.name])
        return out

    def apply_defaults(self):
        new = {}
        for p in self.signature.parameters.values():
            if p.name in self.arguments:
                new[p.name] = self.arguments[p.name]
            elif p.default is not _empty:
                new[p.name] = p.default
            elif p.kind == 2:
                new[p.name] = ()
            elif p.kind == 4:
                new[p.name] = {}
        self.arguments = new


class Signature:
    empty = _empty

    def __init__(self, parameters=None, *, return_annotation=_empty):
        self.parameters = {}
        for p in parameters or ():
            self.parameters[p.name] = p
        self.return_annotation = return_annotation

    @classmethod
    def from_callable(cls, obj, *, follow_wrapped=True):
        return signature(obj, follow_wrapped=follow_wrapped)

    def replace(self, *, parameters=None, return_annotation=None):
        return Signature(list(self.parameters.values()) if parameters is None else parameters,
                         return_annotation=self.return_annotation if return_annotation is None else return_annotation)

    def __str__(self):
        parts = []
        kinds = [p.kind for p in self.parameters.values()]
        star_done = False
        for i, p in enumerate(self.parameters.values()):
            if p.kind == 3 and not star_done and 2 not in kinds:
                parts.append("*")
                star_done = True
            if p.kind == 2:
                star_done = True
            parts.append(str(p))
            if p.kind == 0 and (i + 1 == len(kinds) or kinds[i + 1] != 0):
                parts.append("/")
        s = "(" + ", ".join(parts) + ")"
        if self.return_annotation is not _empty:
            s += " -> " + _annot(self.return_annotation)
        return s

    def __repr__(self):
        return "<Signature %s>" % self

    def __eq__(self, other):
        if not isinstance(other, Signature):
            return NotImplemented
        return list(self.parameters.values()) == list(other.parameters.values()) and self.return_annotation == other.return_annotation

    def _bind(self, args, kwargs, partial):
        arguments = {}
        params = list(self.parameters.values())
        args = list(args)
        kwargs = dict(kwargs)
        i = 0
        for p in params:
            if p.kind in (0, 1):
                if args:
                    arguments[p.name] = args.pop(0)
                elif p.kind == 1 and p.name in kwargs:
                    arguments[p.name] = kwargs.pop(p.name)
                elif p.default is _empty and not partial:
                    raise TypeError("missing a required argument: %r" % p.name)
            elif p.kind == 2:
                if args:
                    arguments[p.name] = tuple(args)
                    args = []
            elif p.kind == 3:
                if p.name in kwargs:
                    arguments[p.name] = kwargs.pop(p.name)
                elif p.default is _empty and not partial:
                    raise TypeError("missing a required argument: %r" % p.name)
            elif p.kind == 4:
                if kwargs:
                    arguments[p.name] = kwargs
                    kwargs = {}
        if args:
            raise TypeError("too many positional arguments")
        if kwargs:
            raise TypeError("got an unexpected keyword argument %r" % next(iter(kwargs)))
        return BoundArguments(self, arguments)

    def bind(self, *args, **kwargs):
        return self._bind(args, kwargs, False)

    def bind_partial(self, *args, **kwargs):
        return self._bind(args, kwargs, True)


def signature(obj, *, follow_wrapped=True, globals=None, locals=None, eval_str=False):
    if not callable(obj):
        raise TypeError("%r is not a callable object" % (obj,))
    if follow_wrapped:
        obj = unwrap(obj, stop=lambda f: hasattr(f, "__signature__"))
    sig = getattr(obj, "__signature__", None)
    if sig is not None:
        return sig
    skip = 0
    if ismethod(obj):
        obj = obj.__func__
        skip = 1
    elif isclass(obj):
        init = getattr(obj, "__init__", None)
        if isfunction(init):
            obj = init
            skip = 1
        else:
            return Signature()
    elif not isfunction(obj):
        call = getattr(type(obj), "__call__", None)
        if isfunction(call):
            obj = call
            skip = 1
        else:
            raise ValueError("no signature found for %r" % (obj,))
    code = obj.__code__
    names = code.co_varnames
    npos = code.co_argcount
    nposonly = getattr(code, "co_posonlyargcount", 0)
    nkw = code.co_kwonlyargcount
    defaults = obj.__defaults__ or ()
    kwdefaults = obj.__kwdefaults__ or {}
    ann = getattr(obj, "__annotations__", {}) or {}
    params = []
    first_default = npos - len(defaults)
    for i in range(npos):
        name = names[i]
        kind = 0 if i < nposonly else 1
        d = defaults[i - first_default] if i >= first_default else _empty
        params.append(Parameter(name, kind, default=d, annotation=ann.get(name, _empty)))
    k = npos
    kwonly = names[k:k + nkw]
    k += nkw
    if code.co_flags & CO_VARARGS:
        params.append(Parameter(names[k], 2, annotation=ann.get(names[k], _empty)))
        k += 1
    for name in kwonly:
        params.append(Parameter(name, 3, default=kwdefaults.get(name, _empty), annotation=ann.get(name, _empty)))
    if code.co_flags & CO_VARKEYWORDS:
        params.append(Parameter(names[k], 4, annotation=ann.get(names[k], _empty)))
    return Signature(params[skip:], return_annotation=ann.get("return", _empty))


def get_annotations(obj, *, globals=None, locals=None, eval_str=False):
    ann = getattr(obj, "__annotations__", None)
    return dict(ann) if ann else {}


def getfullargspec(func):
    sig = signature(func, follow_wrapped=False)
    args = [p.name for p in sig.parameters.values() if p.kind in (0, 1)]
    varargs = next((p.name for p in sig.parameters.values() if p.kind == 2), None)
    varkw = next((p.name for p in sig.parameters.values() if p.kind == 4), None)
    kwonly = [p.name for p in sig.parameters.values() if p.kind == 3]
    return (args, varargs, varkw, func.__defaults__, kwonly, func.__kwdefaults__, getattr(func, "__annotations__", {}))


GEN_CREATED = 'GEN_CREATED'
GEN_RUNNING = 'GEN_RUNNING'
GEN_SUSPENDED = 'GEN_SUSPENDED'
GEN_CLOSED = 'GEN_CLOSED'


def getgeneratorstate(generator):
    if generator.gi_running:
        return GEN_RUNNING
    if generator.gi_suspended:
        return GEN_SUSPENDED
    if generator.gi_frame is None:
        return GEN_CLOSED
    return GEN_CREATED


CORO_CREATED = 'CORO_CREATED'
CORO_RUNNING = 'CORO_RUNNING'
CORO_SUSPENDED = 'CORO_SUSPENDED'
CORO_CLOSED = 'CORO_CLOSED'


def getcoroutinestate(coroutine):
    if coroutine.cr_running:
        return CORO_RUNNING
    if coroutine.cr_suspended:
        return CORO_SUSPENDED
    if coroutine.cr_frame is None:
        return CORO_CLOSED
    return CORO_CREATED


def getattr_static_safe(obj, attr):
    return getattr_static(obj, attr)
