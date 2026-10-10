# SPDX-License-Identifier: MIT
# typing for the Ranger Python runtime: the names exist and subscripting
# them works, so annotated code runs; nothing is checked.

TYPE_CHECKING = False


class _Alias:
    def __init__(self, name, args=(), origin=None):
        self._name = name
        self.__args__ = args
        self.__origin__ = origin
        self.__name__ = name

    def __getitem__(self, params):
        if not isinstance(params, tuple):
            params = (params,)
        return _Alias(self._name, params, self)

    def __call__(self, *args, **kwargs):
        if self.__origin__ is not None:
            return self.__origin__(*args, **kwargs)
        raise TypeError("Cannot instantiate typing." + self._name)

    def __or__(self, other):
        return Union[self, other]

    def __ror__(self, other):
        return Union[other, self]

    def __repr__(self):
        if self.__args__:
            return "typing.%s[%s]" % (self._name, ", ".join(_type_repr(a) for a in self.__args__))
        return "typing." + self._name

    def __eq__(self, other):
        return isinstance(other, _Alias) and self._name == other._name and self.__args__ == other.__args__

    def __hash__(self):
        return hash((self._name, self.__args__))


def _type_repr(obj):
    if isinstance(obj, type):
        if obj.__module__ == "builtins":
            return obj.__qualname__
        return obj.__module__ + "." + obj.__qualname__
    if obj is ...:
        return "..."
    return repr(obj)


class _Builtin(_Alias):
    def __init__(self, name, origin):
        _Alias.__init__(self, name)
        self._real = origin

    def __call__(self, *args, **kwargs):
        return self._real(*args, **kwargs)

    def __getitem__(self, params):
        if not isinstance(params, tuple):
            params = (params,)
        a = _Alias(self._name, params, self._real)
        return a


Any = _Alias("Any")
NoReturn = _Alias("NoReturn")
Never = _Alias("Never")
Self = _Alias("Self")
LiteralString = _Alias("LiteralString")
class _UnionForm(_Alias):
    # Union[...] flattens nested unions, drops duplicates and spells None
    # as NoneType; a single remaining argument is the type itself
    def __getitem__(self, params):
        if not isinstance(params, tuple):
            params = (params,)
        if not params:
            raise TypeError("Cannot take a Union of no types.")
        args = []
        for p in params:
            if p is None:
                p = type(None)
            if isinstance(p, _Alias) and p.__origin__ is Union or _is_union_type(p):
                sub = p.__args__
            else:
                sub = (p,)
            for a in sub:
                if a not in args:
                    args.append(a)
        if len(args) == 1:
            return args[0]
        return _Alias(self._name, tuple(args), self)


class _OptionalForm(_Alias):
    def __getitem__(self, arg):
        return Union[arg, None]


def _is_union_type(tp):
    return type(tp) is type(int | str)


Union = _UnionForm("Union")
Optional = _OptionalForm("Optional")
Literal = _Alias("Literal")
ClassVar = _Alias("ClassVar")
Final = _Alias("Final")
Annotated = _Alias("Annotated")
Concatenate = _Alias("Concatenate")
TypeGuard = _Alias("TypeGuard")
TypeIs = _Alias("TypeIs")
Unpack = _Alias("Unpack")
Required = _Alias("Required")
NotRequired = _Alias("NotRequired")
ReadOnly = _Alias("ReadOnly")
List = _Builtin("List", list)
Dict = _Builtin("Dict", dict)
Set = _Builtin("Set", set)
FrozenSet = _Builtin("FrozenSet", frozenset)
Tuple = _Builtin("Tuple", tuple)
Type = _Builtin("Type", type)
Callable = _Alias("Callable")
Iterable = _Alias("Iterable")
Iterator = _Alias("Iterator")
Generator = _Alias("Generator")
AsyncIterator = _Alias("AsyncIterator")
AsyncGenerator = _Alias("AsyncGenerator")
Awaitable = _Alias("Awaitable")
Coroutine = _Alias("Coroutine")
Sequence = _Alias("Sequence")
MutableSequence = _Alias("MutableSequence")
Mapping = _Alias("Mapping")
MutableMapping = _Alias("MutableMapping")
AbstractSet = _Alias("AbstractSet")
MutableSet = _Alias("MutableSet")
Collection = _Alias("Collection")
Container = _Alias("Container")
Hashable = _Alias("Hashable")
Sized = _Alias("Sized")
Reversible = _Alias("Reversible")
KeysView = _Alias("KeysView")
ValuesView = _Alias("ValuesView")
ItemsView = _Alias("ItemsView")
IO = _Alias("IO")
TextIO = _Alias("TextIO")
BinaryIO = _Alias("BinaryIO")
Pattern = _Alias("Pattern")
Match = _Alias("Match")
SupportsInt = _Alias("SupportsInt")
SupportsFloat = _Alias("SupportsFloat")
SupportsIndex = _Alias("SupportsIndex")
SupportsAbs = _Alias("SupportsAbs")
SupportsRound = _Alias("SupportsRound")
SupportsComplex = _Alias("SupportsComplex")
SupportsBytes = _Alias("SupportsBytes")
AnyStr = _Alias("AnyStr")
Text = str


def _deque(*args):
    import collections
    return collections.deque(*args)


def _defaultdict(*args):
    import collections
    return collections.defaultdict(*args)


def _counter(*args):
    import collections
    return collections.Counter(*args)


def _ordereddict(*args):
    import collections
    return collections.OrderedDict(*args)


Deque = _Builtin("Deque", _deque)
DefaultDict = _Builtin("DefaultDict", _defaultdict)
Counter = _Builtin("Counter", _counter)
OrderedDict = _Builtin("OrderedDict", _ordereddict)
ChainMap = _Alias("ChainMap")


class TypeVar:
    def __init__(self, name, *constraints, bound=None, covariant=False, contravariant=False, infer_variance=False, default=None):
        self.__name__ = name
        self.__constraints__ = constraints
        self.__bound__ = bound
        self.__covariant__ = covariant
        self.__contravariant__ = contravariant
        self.__default__ = default
        self.__infer_variance__ = infer_variance

    def __or__(self, other):
        return Union[self, other]

    def __ror__(self, other):
        return Union[other, self]

    def __repr__(self):
        if self.__infer_variance__:
            return self.__name__
        if self.__covariant__:
            return "+" + self.__name__
        if self.__contravariant__:
            return "-" + self.__name__
        return "~" + self.__name__


class ParamSpec(TypeVar):
    def __repr__(self):
        return "~" + self.__name__


class TypeVarTuple(TypeVar):
    def __repr__(self):
        return self.__name__


class TypeAliasType:
    def __init__(self, name, value, *, type_params=()):
        self.__name__ = name
        self.__value__ = value
        self.__type_params__ = type_params

    def __getitem__(self, params):
        return self

    def __repr__(self):
        return self.__name__


class Generic:
    def __class_getitem__(cls, params):
        return cls


class Protocol(Generic):
    pass


def runtime_checkable(cls):
    return cls


def cast(typ, val):
    return val


def assert_type(val, typ):
    return val


def reveal_type(obj):
    return obj


def assert_never(arg):
    raise AssertionError("Expected code to be unreachable")


def overload(func):
    return func


def final(f):
    return f


def override(method):
    return method


def no_type_check(arg):
    return arg


def get_overloads(func):
    return []


def _eval_hint(value, globalns, localns):
    if value is None:
        return type(None)
    if isinstance(value, str):
        return _eval_hint(eval(value, globalns, localns), globalns, localns)
    return value


def get_type_hints(obj, globalns=None, localns=None, include_extras=False):
    import sys
    if isinstance(obj, type):
        hints = {}
        for base in reversed(obj.__mro__):
            ann = base.__dict__.get("__annotations__") or {}
            if globalns is None:
                base_globals = getattr(sys.modules.get(base.__module__), "__dict__", {})
            else:
                base_globals = globalns
            base_locals = dict(vars(base)) if localns is None else localns
            if localns is None and globalns is None:
                base_globals, base_locals = base_locals, base_globals
            for name, value in ann.items():
                hints[name] = _eval_hint(value, base_globals, base_locals)
        return hints
    if globalns is None:
        nsobj = obj
        while hasattr(nsobj, "__wrapped__"):
            nsobj = nsobj.__wrapped__
        globalns = getattr(nsobj, "__globals__", None)
        if globalns is None:
            globalns = getattr(sys.modules.get(getattr(obj, "__module__", None)), "__dict__", {})
    if localns is None:
        localns = globalns
    ann = getattr(obj, "__annotations__", None) or {}
    return {name: _eval_hint(value, globalns, localns) for name, value in ann.items()}


def get_origin(tp):
    if _is_union_type(tp):
        return type(tp)
    return getattr(tp, "__origin__", None)


def get_args(tp):
    return getattr(tp, "__args__", ())


def NewType(name, tp):
    def new_type(x):
        return x
    new_type.__name__ = name
    new_type.__supertype__ = tp
    return new_type


_NT_SKIP = ("__module__", "__qualname__", "__annotations__", "__doc__", "__orig_bases__", "__classcell__")


def NamedTuple(typename, fields=None, **kwargs):
    import collections
    if fields is None:
        fields = list(kwargs.items())
    return collections.namedtuple(typename, [f[0] for f in fields])


class _NamedTupleMeta(type):
    def __new__(mcls, typename, bases, ns):
        import collections
        if ns.get("_root"):
            return super().__new__(mcls, typename, bases, ns)
        ann = ns.get("__annotations__", {})
        names = list(ann)
        defaults = [ns[n] for n in names if n in ns]
        nt = collections.namedtuple(typename, names, defaults=defaults)
        for k, v in ns.items():
            if k not in names and k not in _NT_SKIP:
                setattr(nt, k, v)
        return nt


NamedTuple.__mro_entries__ = lambda bases: (_NTBase,)


class _NTBase(metaclass=_NamedTupleMeta):
    _root = True


class TypedDict(dict):
    pass
