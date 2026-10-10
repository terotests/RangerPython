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
Union = _Alias("Union")
Optional = _Alias("Optional")
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


def get_type_hints(obj, globalns=None, localns=None, include_extras=False):
    return dict(getattr(obj, "__annotations__", {}) or {})


def get_origin(tp):
    return getattr(tp, "__origin__", None)


def get_args(tp):
    return getattr(tp, "__args__", ())


def NewType(name, tp):
    def new_type(x):
        return x
    new_type.__name__ = name
    new_type.__supertype__ = tp
    return new_type


def NamedTuple(typename, fields=None, **kwargs):
    import collections
    if fields is None:
        fields = list(kwargs.items())
    return collections.namedtuple(typename, [f[0] for f in fields])


class TypedDict(dict):
    pass
