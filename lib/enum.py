# SPDX-License-Identifier: MIT
# enum for the Ranger Python runtime: Enum, IntEnum, StrEnum, Flag,
# IntFlag, auto and unique, on a metaclass as in CPython.

__all__ = ["EnumType", "EnumMeta", "Enum", "IntEnum", "StrEnum", "Flag", "IntFlag", "ReprEnum",
           "auto", "unique", "property", "verify", "member", "nonmember", "global_enum",
           "FlagBoundary", "STRICT", "CONFORM", "EJECT", "KEEP", "EnumCheck", "CONTINUOUS",
           "NAMED_FLAGS", "UNIQUE", "pickle_by_enum_name", "pickle_by_global_name"]

_builtin_property = property
Enum = Flag = ReprEnum = IntEnum = IntFlag = StrEnum = None


class auto:
    _counter = 0

    def __init__(self, value=None):
        self.value = value if value is not None else _auto_null
        auto._counter += 1
        self._order = auto._counter

    def __repr__(self):
        return "auto(%r)" % (self.value,)


class _AutoNull:
    def __repr__(self):
        return "_auto_null"


_auto_null = _AutoNull()


class member:
    def __init__(self, value):
        self.value = value


class nonmember:
    def __init__(self, value):
        self.value = value


property = _builtin_property


def _is_descriptor(obj):
    return hasattr(obj, "__get__") or hasattr(obj, "__set__") or hasattr(obj, "__delete__")


def _is_dunder(name):
    return len(name) > 4 and name[:2] == name[-2:] == "__" and name[2] != "_" and name[-3] != "_"


def _is_sunder(name):
    return len(name) > 2 and name[0] == name[-1] == "_" and name[1] != "_" and name[-2] != "_"


def _is_private(cls_name, name):
    pattern = "_%s__" % (cls_name,)
    return len(name) > len(pattern) and name.startswith(pattern) and not name.endswith("__")


class FlagBoundary:
    pass


STRICT = "strict"
CONFORM = "conform"
EJECT = "eject"
KEEP = "keep"


class EnumCheck:
    pass


CONTINUOUS = "no skipped integer values"
NAMED_FLAGS = "multi-flag aliases may not contain unnamed flags"
UNIQUE = "one name per value"


def _member_names_of(ns):
    return [k for k in ns if not (_is_dunder(k) or _is_sunder(k)) and not _is_descriptor(ns[k]) and not callable(ns[k]) or isinstance(ns.get(k), (auto, member))]


class EnumDict(dict):
    def __init__(self):
        super().__init__()
        self._member_names = []
        self._last_values = []
        self._cls_name = None
        self._generate = None

    def __setitem__(self, key, value):
        if _is_private(self._cls_name or "", key):
            pass
        elif _is_sunder(key):
            if key == "_generate_next_value_":
                self._generate = value
            elif key == "_ignore_":
                pass
        elif _is_dunder(key):
            if key == "__order__":
                key = "_order_"
        elif key in self._member_names:
            raise TypeError("%r already defined as %r" % (key, self[key]))
        elif isinstance(value, nonmember):
            value = value.value
        elif isinstance(value, member) or not (_is_descriptor(value) or callable(value) and not isinstance(value, auto)) or isinstance(value, auto):
            if isinstance(value, member):
                value = value.value
            if key in self:
                raise TypeError("%r already defined as %r" % (key, self[key]))
            if isinstance(value, auto):
                if value.value is _auto_null:
                    gen = self._generate or _default_generate
                    value.value = gen(key, 1, len(self._member_names), list(self._last_values))
                value = value.value
            elif isinstance(value, tuple) and any(isinstance(v, auto) for v in value):
                vals = []
                for v in value:
                    if isinstance(v, auto):
                        if v.value is _auto_null:
                            gen = self._generate or _default_generate
                            v.value = gen(key, 1, len(self._member_names), list(self._last_values))
                        v = v.value
                    vals.append(v)
                value = tuple(vals)
            self._member_names.append(key)
            self._last_values.append(value)
        super().__setitem__(key, value)


_EnumDict = EnumDict


def _default_generate(name, start, count, last_values):
    for v in reversed(last_values):
        try:
            return v + 1
        except TypeError:
            pass
    return start


class EnumType(type):
    @classmethod
    def __prepare__(metacls, cls, bases, **kwds):
        d = EnumDict()
        d._cls_name = cls
        for b in bases:
            g = getattr(b, "_generate_next_value_", None)
            if g is not None and not isinstance(b, EnumType) or (g is not None and b.__name__ not in ("Enum",)):
                gf = getattr(g, "__func__", g)
                d._generate = gf
                break
        return d

    def __new__(metacls, cls, bases, classdict, *, boundary=None, _simple=False, **kwds):
        if not isinstance(classdict, EnumDict):
            ed = EnumDict()
            ed._cls_name = cls
            for k, v in classdict.items():
                ed[k] = v
            classdict = ed
        member_names = list(classdict._member_names)
        members = {k: classdict[k] for k in member_names}
        ns = {k: v for k, v in classdict.items() if k not in member_names}
        member_type, first_enum = metacls._get_mixins_(cls, bases)
        ns["_member_names_"] = []
        ns["_member_map_"] = {}
        ns["_value2member_map_"] = {}
        ns["_member_type_"] = member_type
        ns["_unhashable_values_"] = []
        if "_generate_next_value_" in classdict:
            ns["_generate_next_value_"] = staticmethod(classdict._generate) if not isinstance(classdict["_generate_next_value_"], staticmethod) else classdict["_generate_next_value_"]
        if members and first_enum is not None and first_enum._member_names_:
            raise TypeError("%s: cannot extend %r" % (cls, first_enum))
        enum_class = super().__new__(metacls, cls, bases, ns, **kwds)
        enum_class._new_member_ = ns.get("__new__") if "__new__" in ns else None
        for name in member_names:
            value = members[name]
            enum_class._add_member_(name, value)
        if Flag is not None and issubclass(enum_class, Flag):
            enum_class._flag_mask_ = 0
            for m in enum_class._member_map_.values():
                if isinstance(m._value_, int):
                    enum_class._flag_mask_ |= m._value_
            enum_class._all_bits_ = 2 ** (enum_class._flag_mask_.bit_length()) - 1 if enum_class._flag_mask_ else 0
            enum_class._boundary_ = boundary or getattr(first_enum, "_boundary_", None)
        return enum_class

    def _add_member_(cls, name, value):
        args = value if isinstance(value, tuple) else (value,)
        mtype = cls._member_type_
        new = cls.__dict__.get("_new_member_")
        custom_new = None
        for base in cls.__mro__:
            if "__new__" in base.__dict__ and base not in (Enum, ReprEnum, Flag, IntEnum, IntFlag, StrEnum) and base is not object and base is not mtype and isinstance(base, EnumType):
                custom_new = base.__dict__["__new__"]
                break
        if custom_new is not None:
            fn = getattr(custom_new, "__func__", custom_new)
            m = fn(cls, *args)
            if not hasattr(m, "_value_"):
                if mtype is object:
                    m._value_ = value
                else:
                    try:
                        m._value_ = mtype(*args)
                    except Exception:
                        m._value_ = value
        elif mtype is object:
            m = object.__new__(cls)
            m._value_ = value
        else:
            if mtype is str and not isinstance(value, tuple):
                args = (value,)
            m = mtype.__new__(cls, *args)
            m._value_ = mtype(*args) if not hasattr(m, "_value_") else m._value_
        value = m._value_
        m._name_ = name
        m.__objclass__ = cls
        m._sort_order_ = len(cls._member_names_)
        init = None
        for base in cls.__mro__:
            if "__init__" in base.__dict__ and isinstance(base, EnumType) and base not in (Enum, Flag):
                init = base.__dict__["__init__"]
                break
        if init is not None:
            init(m, *args)
        # an alias names an existing member with the same value
        for other in cls._member_map_.values():
            if other._value_ == value and type(other._value_) is type(value) or (other._value_ == value and not (isinstance(value, bool) != isinstance(other._value_, bool))):
                cls._member_map_[name] = other
                type.__setattr__(cls, name, other)
                return
        is_flag_alias = False
        if Flag is not None and issubclass(cls, Flag) and isinstance(value, int):
            if value != 0 and value & (value - 1) != 0:
                is_flag_alias = True
        if not is_flag_alias or True:
            cls._member_names_.append(name)
        cls._member_map_[name] = m
        type.__setattr__(cls, name, m)
        try:
            cls._value2member_map_[value] = m
        except TypeError:
            cls._unhashable_values_.append(value)

    @classmethod
    def _get_mixins_(metacls, class_name, bases):
        if not bases:
            return object, Enum
        first_enum = None
        member_type = object
        for chain in bases:
            for base in chain.__mro__:
                if base is object:
                    continue
                if isinstance(base, EnumType):
                    if first_enum is None:
                        first_enum = base
                    if base._member_type_ is not object:
                        member_type = base._member_type_
                        break
                elif member_type is object:
                    member_type = base
                    break
        return member_type, first_enum

    def __call__(cls, value=None, names=None, *values, module=None, qualname=None, type=None, start=1, boundary=None):
        if names is None:
            if isinstance(value, cls):
                return value
            return cls.__new__member_lookup(value)
        return cls._create_(value, names, module=module, qualname=qualname, type=type, start=start, boundary=boundary)

    def _EnumType__new__member_lookup(cls, value):
        return cls._lookup_(value)

    def __new__member_lookup(cls, value):
        return cls._lookup_(value)

    def _lookup_(cls, value):
        try:
            if value in cls._value2member_map_:
                return cls._value2member_map_[value]
        except TypeError:
            pass
        for m in cls._member_map_.values():
            if m._value_ == value:
                return m
        if Flag is not None and issubclass(cls, Flag) and isinstance(value, int):
            return cls._missing_flag_(value)
        result = cls._missing_(value)
        if isinstance(result, cls):
            return result
        if result is None:
            raise ValueError("%r is not a valid %s" % (value, cls.__qualname__))
        raise TypeError("error in %s._missing_: returned %r instead of None or a valid member" % (cls.__name__, result))

    def _create_(cls, class_name, names, *, module=None, qualname=None, type=None, start=1, boundary=None):
        metacls = cls.__class__
        bases = (cls,) if type is None else (type, cls)
        classdict = metacls.__prepare__(class_name, bases)
        if isinstance(names, str):
            names = names.replace(",", " ").split()
        if isinstance(names, (tuple, list)) and names and isinstance(names[0], str):
            original_names, names = names, []
            last_values = []
            for count, name in enumerate(original_names):
                value = cls._generate_next_value_(name, start, count, last_values[:])
                last_values.append(value)
                names.append((name, value))
        if isinstance(names, dict):
            names = names.items()
        for item in names:
            if isinstance(item, str):
                member_name, member_value = item, names[item]
            else:
                member_name, member_value = item
            classdict[member_name] = member_value
        classdict["__module__"] = module or "__main__"
        if qualname is not None:
            classdict["__qualname__"] = qualname
        return metacls.__new__(metacls, class_name, bases, classdict, boundary=boundary)

    def __contains__(cls, value):
        if isinstance(value, cls):
            return True
        try:
            return value in cls._value2member_map_ or value in cls._unhashable_values_
        except TypeError:
            return False

    def __delattr__(cls, attr):
        if attr in cls._member_map_:
            raise AttributeError("%r cannot delete member %r." % (cls.__name__, attr))
        super().__delattr__(attr)

    def __getitem__(cls, name):
        return cls._member_map_[name]

    def __iter__(cls):
        return (cls._member_map_[name] for name in cls._member_names_ if not cls._is_multi_flag(name))

    def _is_multi_flag(cls, name):
        if Flag is None or not issubclass(cls, Flag):
            return False
        v = cls._member_map_[name]._value_
        return isinstance(v, int) and v != 0 and v & (v - 1) != 0 and False

    def __reversed__(cls):
        return reversed(list(cls))

    def __len__(cls):
        return len(cls._member_names_)

    def __bool__(cls):
        return True

    def __repr__(cls):
        if Flag is not None and issubclass(cls, Flag):
            return "<flag %r>" % cls.__name__
        return "<enum %r>" % cls.__name__

    @_builtin_property
    def __members__(cls):
        import types
        return types.MappingProxyType(cls._member_map_) if hasattr(types, "MappingProxyType") else dict(cls._member_map_)

    def __setattr__(cls, name, value):
        member_map = cls.__dict__.get("_member_map_", {})
        if name in member_map:
            raise AttributeError("cannot reassign member %r" % (name,))
        super().__setattr__(name, value)

    def __getattr__(cls, name):
        if _is_dunder(name):
            raise AttributeError(name)
        try:
            return cls.__dict__["_member_map_"][name]
        except KeyError:
            raise AttributeError(name) from None


EnumMeta = EnumType
Flag = None


class Enum(metaclass=EnumType):
    @classmethod
    def _generate_next_value_(cls_or_name, *args):
        # called as name, start, count, last_values
        if len(args) == 3:
            name, (start, count, last_values) = cls_or_name, args
        else:
            name, start, count, last_values = args
        return _default_generate(name, start, count, last_values)

    def __new__(cls, value):
        return cls._lookup_(value)

    def __init__(self, *args, **kwds):
        pass

    @classmethod
    def _missing_(cls, value):
        return None

    def __repr__(self):
        v_repr = self.__class__._value_repr_ if hasattr(self.__class__, "_value_repr_") else repr
        return "<%s.%s: %s>" % (self.__class__.__name__, self._name_, v_repr(self._value_))

    def __str__(self):
        return "%s.%s" % (self.__class__.__name__, self._name_)

    def __dir__(self):
        return ["__class__", "__doc__", "__eq__", "__hash__", "__module__", "name", "value"]

    def __format__(self, format_spec):
        return str.__format__(str(self), format_spec)

    def __hash__(self):
        return hash(self._name_)

    def __reduce_ex__(self, proto):
        return self.__class__, (self._value_,)

    def __deepcopy__(self, memo):
        return self

    def __copy__(self):
        return self

    def __eq__(self, other):
        return self is other

    def __ne__(self, other):
        return self is not other

    @_builtin_property
    def name(self):
        return self._name_

    @_builtin_property
    def value(self):
        return self._value_


class ReprEnum(Enum):
    pass


class IntEnum(int, ReprEnum):
    __repr__ = Enum.__repr__

    def __str__(self):
        return int.__repr__(self._value_)

    def __format__(self, spec):
        return format(self._value_, spec)

    def __eq__(self, other):
        return int(self._value_) == other

    def __ne__(self, other):
        return int(self._value_) != other

    def __hash__(self):
        return hash(self._value_)

    def __lt__(self, other):
        return int(self._value_) < other

    def __le__(self, other):
        return int(self._value_) <= other

    def __gt__(self, other):
        return int(self._value_) > other

    def __ge__(self, other):
        return int(self._value_) >= other


class StrEnum(str, ReprEnum):
    __repr__ = Enum.__repr__

    def __new__(cls, *values):
        if len(values) > 3:
            raise TypeError("too many arguments for str(): %r" % (values,))
        if len(values) == 1 and not isinstance(values[0], str):
            raise TypeError("%r is not a string" % (values[0],))
        value = str(*values)
        member = str.__new__(cls, value)
        member._value_ = value
        return member

    def __str__(self):
        return self._value_

    def __format__(self, spec):
        return format(self._value_, spec)

    def __eq__(self, other):
        return str(self._value_) == other

    def __ne__(self, other):
        return str(self._value_) != other

    def __hash__(self):
        return hash(self._value_)

    @staticmethod
    def _generate_next_value_(name, start, count, last_values):
        return name.lower()


def _high_bit(value):
    return value.bit_length() - 1


class Flag(Enum):
    _numeric_repr_ = repr

    @staticmethod
    def _generate_next_value_(name, start, count, last_values):
        if not count:
            return start if start is not None else 1
        last_value = max(last_values)
        try:
            high_bit = _high_bit(last_value)
        except Exception:
            raise TypeError("invalid flag value %r" % last_value) from None
        return 2 ** (high_bit + 1)

    @classmethod
    def _missing_flag_(cls, value):
        if not isinstance(value, int):
            raise ValueError("%r is not a valid %s" % (value, cls.__qualname__))
        if value < 0:
            value = cls._all_bits_ + 1 + value if cls._all_bits_ else value
        pseudo = object.__new__(cls) if cls._member_type_ is object else cls._member_type_.__new__(cls, value)
        pseudo._value_ = value
        names = []
        for m in cls._member_map_.values():
            v = m._value_
            if v and v & (v - 1) == 0 and value & v == v:
                names.append(m._name_)
        pseudo._name_ = "|".join(names) if names else None
        cls._value2member_map_[value] = pseudo
        return pseudo

    def __repr__(self):
        name = self._name_
        if name is None:
            return "<%s: %r>" % (self.__class__.__name__, self._value_)
        return "<%s.%s: %r>" % (self.__class__.__name__, name, self._value_)

    def __str__(self):
        if self._name_ is None:
            return "%s(%r)" % (self.__class__.__name__, self._value_)
        return "%s.%s" % (self.__class__.__name__, self._name_)

    def __bool__(self):
        return bool(self._value_)

    def __contains__(self, other):
        if not isinstance(other, self.__class__):
            raise TypeError("unsupported operand type(s) for 'in': %r and %r" % (type(other).__qualname__, self.__class__.__qualname__))
        return other._value_ & self._value_ == other._value_

    def __iter__(self):
        for m in self.__class__._member_map_.values():
            v = m._value_
            if v and v & (v - 1) == 0 and self._value_ & v == v and m._name_ in self.__class__._member_names_:
                yield m

    def __len__(self):
        return bin(self._value_).count("1")

    def __hash__(self):
        return hash(self._value_)

    def __eq__(self, other):
        if isinstance(other, Flag):
            return self._value_ == other._value_ and type(self) is type(other)
        return NotImplemented

    def __or__(self, other):
        if isinstance(other, self.__class__):
            other = other._value_
        elif self._member_type_ is not object and isinstance(other, self._member_type_):
            pass
        else:
            return NotImplemented
        return self.__class__(self._value_ | other)

    def __and__(self, other):
        if isinstance(other, self.__class__):
            other = other._value_
        elif self._member_type_ is not object and isinstance(other, self._member_type_):
            pass
        else:
            return NotImplemented
        return self.__class__(self._value_ & other)

    def __xor__(self, other):
        if isinstance(other, self.__class__):
            other = other._value_
        elif self._member_type_ is not object and isinstance(other, self._member_type_):
            pass
        else:
            return NotImplemented
        return self.__class__(self._value_ ^ other)

    def __invert__(self):
        return self.__class__(self.__class__._flag_mask_ ^ self._value_)

    __ror__ = __or__
    __rand__ = __and__
    __rxor__ = __xor__


class IntFlag(int, ReprEnum, Flag):
    __repr__ = Flag.__repr__

    def __str__(self):
        return int.__repr__(self._value_)

    def __format__(self, spec):
        return format(self._value_, spec)

    def __eq__(self, other):
        return int(self._value_) == other

    def __hash__(self):
        return hash(self._value_)

    def __or__(self, other):
        if isinstance(other, int):
            other = int(other)
            return self.__class__(self._value_ | other)
        return NotImplemented

    def __and__(self, other):
        if isinstance(other, int):
            return self.__class__(self._value_ & int(other))
        return NotImplemented

    def __xor__(self, other):
        if isinstance(other, int):
            return self.__class__(self._value_ ^ int(other))
        return NotImplemented

    __ror__ = __or__
    __rand__ = __and__
    __rxor__ = __xor__


def unique(enumeration):
    duplicates = []
    for name, member in enumeration.__members__.items():
        if name != member.name:
            duplicates.append((name, member.name))
    if duplicates:
        alias_details = ", ".join(["%s -> %s" % (alias, name) for (alias, name) in duplicates])
        raise ValueError("duplicate values found in %r: %s" % (enumeration, alias_details))
    return enumeration


def verify(*checks):
    def decorator(enumeration):
        return enumeration
    return decorator


def global_enum(cls, update_str=False):
    import sys
    g = sys.modules[cls.__module__].__dict__ if cls.__module__ in sys.modules else {}
    g.update(cls.__members__)
    return cls


def pickle_by_enum_name(self, proto):
    return getattr, (self.__class__, self._name_)


def pickle_by_global_name(self, proto):
    return self.name


def _simple_enum(etype=Enum, *, boundary=None, use_args=None):
    # the decorated class body, replayed as an enum of etype
    def convert(cls):
        body = {k: v for k, v in cls.__dict__.items() if k not in ("__dict__", "__weakref__")}
        kw = {} if boundary is None else {"boundary": boundary}
        return type(etype)(cls.__name__, (etype,), body, **kw)
    return convert
