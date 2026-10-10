# SPDX-License-Identifier: MIT
# dataclasses for the Ranger Python runtime. Like CPython's, the methods
# are written as source text and run with exec.
import copy as _copy
import keyword as _keyword

__all__ = ["dataclass", "field", "Field", "FrozenInstanceError", "InitVar", "KW_ONLY", "MISSING",
           "fields", "asdict", "astuple", "make_dataclass", "replace", "is_dataclass"]


class FrozenInstanceError(AttributeError):
    pass


class _MISSING_TYPE:
    def __repr__(self):
        return "MISSING"


MISSING = _MISSING_TYPE()


class _KW_ONLY_TYPE:
    def __repr__(self):
        return "KW_ONLY"


KW_ONLY = _KW_ONLY_TYPE()


class InitVar:
    __slots__ = ("type",)

    def __init__(self, type):
        self.type = type

    def __repr__(self):
        name = getattr(self.type, "__name__", repr(self.type))
        return f"dataclasses.InitVar[{name}]"

    def __class_getitem__(cls, type):
        return InitVar(type)


_FIELD = "_FIELD"
_FIELD_CLASSVAR = "_FIELD_CLASSVAR"
_FIELD_INITVAR = "_FIELD_INITVAR"
_FIELDS = "__dataclass_fields__"
_PARAMS = "__dataclass_params__"


class Field:
    __slots__ = ("name", "type", "default", "default_factory", "repr", "hash", "init", "compare",
                 "metadata", "kw_only", "_field_type")

    def __init__(self, default, default_factory, init, repr, hash, compare, metadata, kw_only):
        self.name = None
        self.type = None
        self.default = default
        self.default_factory = default_factory
        self.init = init
        self.repr = repr
        self.hash = hash
        self.compare = compare
        self.metadata = {} if metadata is None else dict(metadata)
        self.kw_only = kw_only
        self._field_type = None

    def __repr__(self):
        return ("Field(name=%r,type=%r,default=%r,default_factory=%r,init=%r,repr=%r,hash=%r,"
                "compare=%r,metadata=%r,kw_only=%r,_field_type=%s)" % (
                    self.name, self.type, self.default, self.default_factory, self.init, self.repr,
                    self.hash, self.compare, self.metadata, self.kw_only, self._field_type))

    def __set_name__(self, owner, name):
        func = getattr(type(self.default), "__set_name__", None)
        if func:
            func(self.default, owner, name)


class _DataclassParams:
    __slots__ = ("init", "repr", "eq", "order", "unsafe_hash", "frozen", "match_args", "kw_only", "slots", "weakref_slot")

    def __init__(self, init, repr, eq, order, unsafe_hash, frozen, match_args=True, kw_only=False, slots=False, weakref_slot=False):
        self.init = init
        self.repr = repr
        self.eq = eq
        self.order = order
        self.unsafe_hash = unsafe_hash
        self.frozen = frozen
        self.match_args = match_args
        self.kw_only = kw_only
        self.slots = slots
        self.weakref_slot = weakref_slot

    def __repr__(self):
        return ("_DataclassParams(init=%r,repr=%r,eq=%r,order=%r,unsafe_hash=%r,frozen=%r)" % (
            self.init, self.repr, self.eq, self.order, self.unsafe_hash, self.frozen))


def field(*, default=MISSING, default_factory=MISSING, init=True, repr=True, hash=None, compare=True, metadata=None, kw_only=MISSING):
    if default is not MISSING and default_factory is not MISSING:
        raise ValueError("cannot specify both default and default_factory")
    return Field(default, default_factory, init, repr, hash, compare, metadata, kw_only)


def _is_classvar(a_type):
    try:
        import typing
        cv = typing.ClassVar
    except Exception:
        cv = None
    if cv is not None and (a_type is cv or getattr(a_type, "__origin__", None) is cv):
        return True
    if isinstance(a_type, str):
        s = a_type.strip()
        return s.startswith("ClassVar") or s.startswith("typing.ClassVar")
    r = repr(a_type)
    return r.startswith("typing.ClassVar")


def _is_initvar(a_type):
    if isinstance(a_type, InitVar) or a_type is InitVar:
        return True
    if isinstance(a_type, str):
        s = a_type.strip()
        return s.startswith("InitVar") or s.startswith("dataclasses.InitVar")
    return False


def _is_kw_only(a_type):
    if a_type is KW_ONLY:
        return True
    return isinstance(a_type, str) and a_type.strip() in ("KW_ONLY", "dataclasses.KW_ONLY")


def _get_field(cls, a_name, a_type, default_kw_only):
    default = getattr(cls, a_name, MISSING)
    if isinstance(default, Field):
        f = default
    else:
        f = field(default=default)
    f.name = a_name
    f.type = a_type
    f._field_type = _FIELD
    if _is_classvar(a_type):
        f._field_type = _FIELD_CLASSVAR
    elif _is_initvar(a_type):
        f._field_type = _FIELD_INITVAR
    if f._field_type in (_FIELD_CLASSVAR, _FIELD_INITVAR):
        if f.default_factory is not MISSING:
            raise TypeError(f"field {f.name} cannot have a default factory")
    if f._field_type in (_FIELD, _FIELD_INITVAR):
        if f.kw_only is MISSING:
            f.kw_only = default_kw_only
    else:
        f.kw_only = False
    if f._field_type is _FIELD and f.default_factory is MISSING and isinstance(f.default, (list, dict, set)):
        raise ValueError(f"mutable default {type(f.default)} for field {f.name} is not allowed: use default_factory")
    return f


def _fields_in_init_order(fields):
    return (tuple(f for f in fields if f.init and not f.kw_only), tuple(f for f in fields if f.init and f.kw_only))


def _tuple_str(obj_name, fields):
    if not fields:
        return "()"
    return "(" + ",".join([f"{obj_name}.{f.name}" for f in fields]) + ",)"


def _exec(name, src, glob):
    ns = {}
    exec(src, glob, ns)
    return ns[name]


def _set_new_attribute(cls, name, value):
    if name in cls.__dict__:
        return True
    if hasattr(value, "__qualname__"):
        try:
            value.__qualname__ = f"{cls.__qualname__}.{value.__name__}"
        except Exception:
            pass
    setattr(cls, name, value)
    return False


def _init_fn(fields, std_fields, kw_only_fields, frozen, has_post_init, self_name, globals_):
    seen_default = False
    for f in std_fields:
        if f.init:
            if not (f.default is MISSING and f.default_factory is MISSING):
                seen_default = True
            elif seen_default:
                raise TypeError(f"non-default argument {f.name!r} follows default argument")
    glob = dict(globals_)
    glob.update({"__dataclass_MISSING__": MISSING, "__dataclass_object_setattr__": object.__setattr__})
    args = [self_name]
    for f in std_fields:
        args.append(_init_param(f, glob))
    if kw_only_fields:
        args.append("*")
        for f in kw_only_fields:
            args.append(_init_param(f, glob))
    body = []
    for f in fields:
        line = _field_init(f, frozen, glob, self_name)
        if line:
            body.append(line)
    if has_post_init:
        params = ",".join(f.name for f in fields if f._field_type is _FIELD_INITVAR)
        body.append(f"{self_name}.__post_init__({params})")
    if not body:
        body = ["pass"]
    src = "def __init__(" + ",".join(args) + "):\n" + "\n".join("  " + b for b in body) + "\n"
    return _exec("__init__", src, glob)


def _init_param(f, glob):
    if f.default is MISSING and f.default_factory is MISSING:
        return f.name
    if f.default is not MISSING:
        glob[f"__dataclass_dflt_{f.name}__"] = f.default
        return f"{f.name}=__dataclass_dflt_{f.name}__"
    return f"{f.name}=__dataclass_HAS_DEFAULT_FACTORY__"


def _field_init(f, frozen, glob, self_name):
    default_name = f"__dataclass_dflt_{f.name}__"
    if f.default_factory is not MISSING:
        glob[f"__dataclass_factory_{f.name}__"] = f.default_factory
        glob["__dataclass_HAS_DEFAULT_FACTORY__"] = _HAS_DEFAULT_FACTORY
        if f.init:
            value = (f"__dataclass_factory_{f.name}__() if {f.name} is __dataclass_HAS_DEFAULT_FACTORY__ else {f.name}")
        else:
            value = f"__dataclass_factory_{f.name}__()"
    else:
        if f.init:
            if f.default is MISSING:
                value = f.name
            else:
                glob[default_name] = f.default
                value = f.name
        else:
            if f.default is not MISSING:
                glob[default_name] = f.default
                value = default_name
            else:
                return None
    if f._field_type is _FIELD_INITVAR:
        return None
    if frozen:
        return f"__dataclass_object_setattr__({self_name},{f.name!r},{value})"
    return f"{self_name}.{f.name}={value}"


class _HAS_DEFAULT_FACTORY_CLASS:
    def __repr__(self):
        return "<factory>"


_HAS_DEFAULT_FACTORY = _HAS_DEFAULT_FACTORY_CLASS()


def _repr_fn(fields):
    def __repr__(self):
        key = (id(self), "repr")
        if key in _repr_running:
            return "..."
        _repr_running.add(key)
        try:
            parts = ", ".join(f"{f.name}={getattr(self, f.name)!r}" for f in fields)
        finally:
            _repr_running.discard(key)
        return f"{self.__class__.__qualname__}({parts})"
    return __repr__


_repr_running = set()


def _cmp_fn(name, op, fields):
    def cmp(self, other):
        if other.__class__ is self.__class__:
            a = tuple(getattr(self, f.name) for f in fields)
            b = tuple(getattr(other, f.name) for f in fields)
            if op == "==":
                return a == b
            if op == "<":
                return a < b
            if op == "<=":
                return a <= b
            if op == ">":
                return a > b
            return a >= b
        return NotImplemented
    cmp.__name__ = name
    return cmp


def _hash_fn(fields):
    def __hash__(self):
        return hash(tuple(getattr(self, f.name) for f in fields))
    return __hash__


def _frozen_setattr(cls, fields):
    names = {f.name for f in fields}

    def __setattr__(self, name, value):
        if type(self) is cls or name in names:
            raise FrozenInstanceError(f"cannot assign to field {name!r}")
        super(cls, self).__setattr__(name, value)

    def __delattr__(self, name):
        if type(self) is cls or name in names:
            raise FrozenInstanceError(f"cannot delete field {name!r}")
        super(cls, self).__delattr__(name)
    return __setattr__, __delattr__


def _process_class(cls, init, repr, eq, order, unsafe_hash, frozen, match_args, kw_only, slots, weakref_slot):
    fields = {}
    globals_ = {}
    mod = getattr(cls, "__module__", None)
    import sys
    if mod in sys.modules:
        globals_ = sys.modules[mod].__dict__
    setattr(cls, _PARAMS, _DataclassParams(init, repr, eq, order, unsafe_hash, frozen, match_args, kw_only, slots, weakref_slot))
    any_frozen_base = False
    has_dataclass_bases = False
    for b in cls.__mro__[-1:0:-1]:
        base_fields = getattr(b, _FIELDS, None)
        if base_fields is not None:
            has_dataclass_bases = True
            for f in base_fields.values():
                fields[f.name] = f
            if getattr(b, _PARAMS).frozen:
                any_frozen_base = True
    cls_annotations = cls.__dict__.get("__annotations__", {})
    cls_fields = []
    KW_ONLY_seen = False
    for name, type_ in cls_annotations.items():
        if _is_kw_only(type_):
            if KW_ONLY_seen:
                raise TypeError(f"{name!r} is KW_ONLY, but KW_ONLY has already been specified")
            KW_ONLY_seen = True
            kw_only = True
        else:
            cls_fields.append(_get_field(cls, name, type_, kw_only))
    for f in cls_fields:
        fields[f.name] = f
        if isinstance(getattr(cls, f.name, None), Field):
            if f.default is MISSING:
                delattr(cls, f.name)
            else:
                setattr(cls, f.name, f.default)
    for name, value in cls.__dict__.items():
        if isinstance(value, Field) and name not in cls_annotations:
            raise TypeError(f"{name!r} is a field but has no type annotation")
    if has_dataclass_bases:
        if any_frozen_base and not frozen:
            raise TypeError("cannot inherit non-frozen dataclass from a frozen one")
        if not any_frozen_base and frozen:
            raise TypeError("cannot inherit frozen dataclass from a non-frozen one")
    setattr(cls, _FIELDS, fields)
    class_hash = cls.__dict__.get("__hash__", MISSING)
    has_explicit_hash = not (class_hash is MISSING or (class_hash is None and "__eq__" in cls.__dict__))
    if order and not eq:
        raise ValueError("eq must be true if order is true")
    all_init_fields = [f for f in fields.values() if f._field_type in (_FIELD, _FIELD_INITVAR)]
    std_init_fields, kw_only_init_fields = _fields_in_init_order(all_init_fields)
    field_list = [f for f in fields.values() if f._field_type is _FIELD]
    if init:
        has_post_init = hasattr(cls, "__post_init__")
        _set_new_attribute(cls, "__init__", _init_fn(all_init_fields, std_init_fields, kw_only_init_fields,
                                                     frozen, has_post_init, "self" if "self" not in fields else "__dataclass_self__", globals_))
    if repr:
        flds = [f for f in field_list if f.repr]
        _set_new_attribute(cls, "__repr__", _repr_fn(flds))
    if eq:
        cmp_fields = [f for f in field_list if f.compare]
        _set_new_attribute(cls, "__eq__", _cmp_fn("__eq__", "==", cmp_fields))
    if order:
        flds = [f for f in field_list if f.compare]
        for name, op in [("__lt__", "<"), ("__le__", "<="), ("__gt__", ">"), ("__ge__", ">=")]:
            if _set_new_attribute(cls, name, _cmp_fn(name, op, flds)):
                raise TypeError(f"Cannot overwrite attribute {name} in class {cls.__name__}. Consider using functools.total_ordering")
    if frozen:
        sa, da = _frozen_setattr(cls, field_list)
        if _set_new_attribute(cls, "__setattr__", sa):
            raise TypeError(f"Cannot overwrite attribute __setattr__ in class {cls.__name__}")
        _set_new_attribute(cls, "__delattr__", da)
    hash_fields = [f for f in field_list if (f.compare if f.hash is None else f.hash)]
    if unsafe_hash:
        if has_explicit_hash:
            raise TypeError(f"Cannot overwrite attribute __hash__ in class {cls.__name__}")
        cls.__hash__ = _hash_fn(hash_fields)
    elif eq and frozen:
        if not has_explicit_hash:
            cls.__hash__ = _hash_fn(hash_fields)
    elif eq and not frozen:
        if not has_explicit_hash:
            cls.__hash__ = None
    if not getattr(cls, "__doc__"):
        try:
            import inspect
            text_sig = str(inspect.signature(cls)).replace(" -> None", "")
        except Exception:
            text_sig = ""
        cls.__doc__ = cls.__name__ + text_sig
    if match_args:
        _set_new_attribute(cls, "__match_args__", tuple(f.name for f in std_init_fields))
    cls.__replace__ = _replace
    if slots:
        names = tuple(f.name for f in field_list)
        cls.__slots__ = names
    import abc
    abc.update_abstractmethods(cls)
    return cls


def dataclass(cls=None, /, *, init=True, repr=True, eq=True, order=False, unsafe_hash=False, frozen=False,
              match_args=True, kw_only=False, slots=False, weakref_slot=False):
    def wrap(cls):
        return _process_class(cls, init, repr, eq, order, unsafe_hash, frozen, match_args, kw_only, slots, weakref_slot)
    if cls is None:
        return wrap
    return wrap(cls)


def fields(class_or_instance):
    try:
        fields = getattr(class_or_instance, _FIELDS)
    except AttributeError:
        raise TypeError("must be called with a dataclass type or instance") from None
    return tuple(f for f in fields.values() if f._field_type is _FIELD)


def _is_dataclass_instance(obj):
    return hasattr(type(obj), _FIELDS)


def is_dataclass(obj):
    cls = obj if isinstance(obj, type) else type(obj)
    return hasattr(cls, _FIELDS)


def asdict(obj, *, dict_factory=dict):
    if not _is_dataclass_instance(obj):
        raise TypeError("asdict() should be called on dataclass instances")
    return _asdict_inner(obj, dict_factory)


def _asdict_inner(obj, dict_factory):
    if _is_dataclass_instance(obj):
        return dict_factory([(f.name, _asdict_inner(getattr(obj, f.name), dict_factory)) for f in fields(obj)])
    if isinstance(obj, tuple) and hasattr(obj, "_fields"):
        return type(obj)(*[_asdict_inner(v, dict_factory) for v in obj])
    if isinstance(obj, (list, tuple)):
        return type(obj)(_asdict_inner(v, dict_factory) for v in obj)
    if isinstance(obj, dict):
        return type(obj)((_asdict_inner(k, dict_factory), _asdict_inner(v, dict_factory)) for k, v in obj.items())
    return _copy.deepcopy(obj)


def astuple(obj, *, tuple_factory=tuple):
    if not _is_dataclass_instance(obj):
        raise TypeError("astuple() should be called on dataclass instances")
    return _astuple_inner(obj, tuple_factory)


def _astuple_inner(obj, tuple_factory):
    if _is_dataclass_instance(obj):
        return tuple_factory([_astuple_inner(getattr(obj, f.name), tuple_factory) for f in fields(obj)])
    if isinstance(obj, tuple) and hasattr(obj, "_fields"):
        return type(obj)(*[_astuple_inner(v, tuple_factory) for v in obj])
    if isinstance(obj, (list, tuple)):
        return type(obj)(_astuple_inner(v, tuple_factory) for v in obj)
    if isinstance(obj, dict):
        return type(obj)((_astuple_inner(k, tuple_factory), _astuple_inner(v, tuple_factory)) for k, v in obj.items())
    return _copy.deepcopy(obj)


def replace(obj, /, **changes):
    if not _is_dataclass_instance(obj):
        raise TypeError("replace() should be called on dataclass instances")
    return _replace(obj, **changes)


def _replace(obj, /, **changes):
    for f in getattr(obj, _FIELDS).values():
        if f._field_type is _FIELD_CLASSVAR:
            continue
        if not f.init:
            if f.name in changes:
                raise ValueError(f"field {f.name} is declared with init=False, it cannot be specified with replace()")
            continue
        if f.name not in changes:
            if f._field_type is _FIELD_INITVAR and f.default is MISSING:
                raise TypeError(f"InitVar {f.name!r} must be specified with replace()")
            changes[f.name] = getattr(obj, f.name)
    return obj.__class__(**changes)


def make_dataclass(cls_name, fields, *, bases=(), namespace=None, init=True, repr=True, eq=True, order=False,
                   unsafe_hash=False, frozen=False, match_args=True, kw_only=False, slots=False, weakref_slot=False, module=None):
    namespace = dict(namespace or {})
    annotations = {}
    for item in fields:
        if isinstance(item, str):
            name, tp = item, "typing.Any"
        elif len(item) == 2:
            name, tp = item
        else:
            name, tp, spec = item
            namespace[name] = spec
        annotations[name] = tp
    namespace["__annotations__"] = annotations
    cls = type(cls_name, bases, namespace)
    if module is not None:
        cls.__module__ = module
    return dataclass(cls, init=init, repr=repr, eq=eq, order=order, unsafe_hash=unsafe_hash, frozen=frozen,
                     match_args=match_args, kw_only=kw_only, slots=slots, weakref_slot=weakref_slot)
