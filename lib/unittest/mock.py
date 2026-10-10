# SPDX-License-Identifier: MIT
# unittest.mock for the Ranger Python runtime: Mock, MagicMock, patch,
# call, ANY, sentinel and DEFAULT, written against the documented API.
import functools
import sys

__all__ = ["Mock", "MagicMock", "NonCallableMock", "NonCallableMagicMock", "patch", "call",
           "ANY", "DEFAULT", "sentinel", "create_autospec", "PropertyMock", "mock_open", "seal"]


class _Sentinel:
    def __init__(self, name):
        self.name = name

    def __repr__(self):
        return "sentinel." + self.name

    def __reduce__(self):
        return "sentinel." + self.name


class _SentinelObject:
    def __init__(self):
        self._sentinels = {}

    def __getattr__(self, name):
        if name == "__bases__":
            raise AttributeError
        if name not in self._sentinels:
            self._sentinels[name] = _Sentinel(name)
        return self._sentinels[name]


sentinel = _SentinelObject()
DEFAULT = sentinel.DEFAULT
_missing = sentinel.MISSING


class _ANY:
    def __eq__(self, other):
        return True

    def __ne__(self, other):
        return False

    def __hash__(self):
        return 0

    def __repr__(self):
        return "<ANY>"


ANY = _ANY()


class _Call(tuple):
    # call(args, kwargs) with a name, compared loosely like CPython's
    def __new__(cls, value=(), name="", parent=None, two=False, from_kall=True):
        if len(value) == 3:
            name, args, kwargs = value
        elif len(value) == 2:
            args, kwargs = value
        elif len(value) == 1:
            args, kwargs = value[0], {}
        else:
            args, kwargs = (), {}
        self = tuple.__new__(cls, (args, kwargs))
        self._mock_name = name
        self._mock_parent = parent
        return self

    def __init__(self, *a, **k):
        pass

    @property
    def args(self):
        return tuple.__getitem__(self, 0)

    @property
    def kwargs(self):
        return tuple.__getitem__(self, 1)

    def __eq__(self, other):
        if other is ANY:
            return True
        try:
            if isinstance(other, _Call):
                if self._mock_name and other._mock_name and self._mock_name != other._mock_name:
                    return False
                return self.args == other.args and self.kwargs == other.kwargs
            if isinstance(other, tuple):
                if len(other) == 2:
                    return (self.args, self.kwargs) == (tuple(other[0]), dict(other[1]))
                if len(other) == 3:
                    return (self.args, self.kwargs) == (tuple(other[1]), dict(other[2]))
                if len(other) == 1:
                    if isinstance(other[0], dict):
                        return self.args == () and self.kwargs == other[0]
                    return self.args == tuple(other[0]) and not self.kwargs
                return False
        except Exception:
            return False
        return NotImplemented

    def __ne__(self, other):
        r = self.__eq__(other)
        return r if r is NotImplemented else not r

    __hash__ = None

    def __call__(self, *args, **kwargs):
        return _Call((self._mock_name, args, kwargs))

    def __getattr__(self, attr):
        if attr.startswith("__"):
            raise AttributeError(attr)
        name = attr if not self._mock_name else self._mock_name + "." + attr
        return _Call((name, (), {}))

    def __repr__(self):
        name = self._mock_name or "call"
        if not name.startswith("call") and name != "call":
            name = "call." + name
        parts = [repr(a) for a in self.args]
        parts += ["%s=%r" % kv for kv in self.kwargs.items()]
        return "%s(%s)" % (name, ", ".join(parts))

    def call_list(self):
        return _CallList([self])


call = _Call(("", (), {}))


class _CallList(list):
    def __contains__(self, value):
        if not isinstance(value, list):
            return list.__contains__(self, value)
        n = len(value)
        for i in range(0, len(self) - n + 1):
            if list(self[i:i + n]) == list(value):
                return True
        return False

    def __repr__(self):
        return "[" + ", ".join(repr(c) for c in self) + "]"


_MAGICS = ("__lt__ __gt__ __le__ __ge__ __int__ __contains__ __len__ __iter__ __exit__ "
           "__complex__ __float__ __bool__ __index__ __hash__ __str__ __sizeof__ __enter__ "
           "__getitem__ __setitem__ __delitem__ __add__ __sub__ __mul__ __truediv__ __floordiv__ "
           "__mod__ __pow__ __and__ __or__ __xor__ __lshift__ __rshift__ __neg__ __pos__ "
           "__abs__ __invert__ __round__ __fspath__ __aenter__ __aexit__ __aiter__ __anext__").split()


class NonCallableMock:
    def __init__(self, spec=None, wraps=None, name=None, spec_set=None, parent=None,
                 _new_name="", _new_parent=None, unsafe=False, return_value=DEFAULT,
                 side_effect=None, **kwargs):
        d = self.__dict__
        d["_mock_children"] = {}
        d["_mock_name"] = name
        d["_mock_parent"] = parent
        d["_mock_new_name"] = _new_name
        d["_mock_wraps"] = wraps
        d["_mock_return_value"] = return_value
        d["_mock_side_effect"] = side_effect
        d["_mock_sealed"] = False
        d["_spec_class"] = None
        d["_spec_set"] = spec_set is not None
        d["_mock_methods"] = None
        d["method_calls"] = _CallList()
        d["mock_calls"] = _CallList()
        d["call_args_list"] = _CallList()
        d["call_count"] = 0
        d["call_args"] = None
        d["called"] = False
        s = spec if spec is not None else spec_set
        if s is not None:
            if isinstance(s, list) and all(isinstance(x, str) for x in s):
                d["_mock_methods"] = list(s)
            else:
                d["_spec_class"] = s if isinstance(s, type) else type(s)
                d["_mock_methods"] = dir(s)
        self.configure_mock(**kwargs)

    def configure_mock(self, **kwargs):
        for arg, val in sorted(kwargs.items(), key=lambda kv: kv[0].count(".")):
            parts = arg.split(".")
            obj = self
            for p in parts[:-1]:
                obj = getattr(obj, p)
            setattr(obj, parts[-1], val)

    @property
    def __class__(self):
        if self._spec_class is None:
            return type(self)
        return self._spec_class

    def _get_child_mock(self, **kw):
        klass = type(self)
        if issubclass(klass, NonCallableMagicMock) and not issubclass(klass, MagicMock):
            klass = MagicMock
        elif issubclass(klass, NonCallableMock) and not issubclass(klass, Mock) and not issubclass(klass, MagicMock):
            klass = Mock
        return klass(**kw)

    def _full_name(self):
        names = []
        m = self
        while m is not None:
            n = m.__dict__.get("_mock_new_name") or m.__dict__.get("_mock_name")
            if m.__dict__.get("_mock_parent") is None:
                break
            names.append(n or "")
            m = m.__dict__.get("_mock_parent")
        return ".".join(reversed(names))

    def __getattr__(self, name):
        if name in ("_mock_methods", "_mock_children", "_spec_class"):
            raise AttributeError(name)
        if name.startswith(("assert", "assret", "asert", "aseert", "assrt")):
            raise AttributeError(name + " is not a valid assertion. Use a spec for the mock if %r is meant to be an attribute." % name)
        methods = self.__dict__.get("_mock_methods")
        if methods is not None and name not in methods:
            raise AttributeError("Mock object has no attribute %r" % name)
        if name.startswith("__") and name.endswith("__"):
            raise AttributeError(name)
        children = self.__dict__["_mock_children"]
        if name not in children:
            if self.__dict__.get("_mock_sealed"):
                raise AttributeError("mock.%s" % name)
            wraps = self.__dict__.get("_mock_wraps")
            kw = {}
            if wraps is not None:
                kw["wraps"] = getattr(wraps, name)
            children[name] = self._get_child_mock(parent=self, name=name, _new_name=name, _new_parent=self, **kw)
        return children[name]

    def __setattr__(self, name, value):
        if name in ("return_value", "side_effect"):
            self.__dict__["_mock_" + name] = value
            return
        if self.__dict__.get("_spec_set") and self.__dict__.get("_mock_methods") is not None and \
                name not in self.__dict__["_mock_methods"] and name not in self.__dict__:
            raise AttributeError("Mock object has no attribute '%s'" % name)
        if isinstance(value, NonCallableMock) and value.__dict__.get("_mock_parent") is None:
            value.__dict__["_mock_parent"] = self
            value.__dict__["_mock_new_name"] = name
            self.__dict__["_mock_children"][name] = value
            return
        if name in self.__dict__["_mock_children"]:
            del self.__dict__["_mock_children"][name]
        self.__dict__[name] = value

    def __delattr__(self, name):
        if name in self.__dict__["_mock_children"]:
            del self.__dict__["_mock_children"][name]
        elif name in self.__dict__:
            del self.__dict__[name]
        else:
            raise AttributeError(name)

    def _get_return_value(self):
        rv = self.__dict__["_mock_return_value"]
        if rv is DEFAULT:
            if self.__dict__.get("_mock_wraps") is not None:
                return DEFAULT
            rv = self._get_child_mock(parent=self, _new_name="()", _new_parent=self)
            self.__dict__["_mock_return_value"] = rv
        return rv

    return_value = property(_get_return_value, lambda self, v: self.__dict__.__setitem__("_mock_return_value", v))

    @property
    def side_effect(self):
        return self.__dict__["_mock_side_effect"]

    def reset_mock(self, return_value=False, side_effect=False):
        d = self.__dict__
        d["called"] = False
        d["call_args"] = None
        d["call_count"] = 0
        d["mock_calls"] = _CallList()
        d["call_args_list"] = _CallList()
        d["method_calls"] = _CallList()
        if return_value:
            d["_mock_return_value"] = DEFAULT
        if side_effect:
            d["_mock_side_effect"] = None
        for child in d["_mock_children"].values():
            if isinstance(child, NonCallableMock):
                child.reset_mock()
        rv = d["_mock_return_value"]
        if isinstance(rv, NonCallableMock) and rv is not self:
            rv.reset_mock()

    def attach_mock(self, mock, attribute):
        mock.__dict__["_mock_parent"] = None
        setattr(self, attribute, mock)

    def mock_add_spec(self, spec, spec_set=False):
        self.__dict__["_mock_methods"] = dir(spec)
        self.__dict__["_spec_set"] = spec_set

    def _record(self, args, kwargs):
        d = self.__dict__
        d["called"] = True
        d["call_count"] += 1
        c = _Call(("", args, kwargs))
        d["call_args"] = c
        d["call_args_list"].append(c)
        d["mock_calls"].append(_Call(("", args, kwargs)))
        # parents see the call as name(...), and method calls in method_calls
        name = d.get("_mock_new_name") or ""
        parent = d.get("_mock_parent")
        while parent is not None:
            pd = parent.__dict__
            if name and "()" not in name:
                pd["method_calls"].append(_Call((name, args, kwargs)))
            pd["mock_calls"].append(_Call((name, args, kwargs)))
            pn = pd.get("_mock_new_name") or ""
            if pd.get("_mock_parent") is None:
                break
            if name.startswith("()"):
                name = pn + name
            else:
                name = pn + ("." + name if name else "")
            parent = pd["_mock_parent"]

    def _call(self, args, kwargs):
        self._record(args, kwargs)
        effect = self.__dict__["_mock_side_effect"]
        if effect is not None:
            if isinstance(effect, BaseException) or (isinstance(effect, type) and issubclass(effect, BaseException)):
                raise effect
            if not callable(effect):
                it = self.__dict__.get("_mock_effect_iter")
                if it is None or self.__dict__.get("_mock_effect_src") is not effect:
                    it = iter(effect)
                    self.__dict__["_mock_effect_iter"] = it
                    self.__dict__["_mock_effect_src"] = effect
                result = next(it)
                if isinstance(result, BaseException) or (isinstance(result, type) and issubclass(result, BaseException)):
                    raise result
                return result
            result = effect(*args, **kwargs)
            if result is not DEFAULT:
                return result
        rv = self.__dict__["_mock_return_value"]
        if rv is not DEFAULT:
            return rv
        wraps = self.__dict__.get("_mock_wraps")
        if wraps is not None:
            return wraps(*args, **kwargs)
        return self.return_value

    def _calls_repr(self):
        return "Calls: %r." % (self.mock_calls,) if self.mock_calls else ""

    def _name(self):
        return self.__dict__.get("_mock_name") or "mock"

    def assert_called(self):
        if self.call_count == 0:
            raise AssertionError("Expected '%s' to have been called." % self._name())

    def assert_not_called(self):
        if self.call_count != 0:
            raise AssertionError("Expected '%s' to not have been called. Called %d times.%s" % (
                self._name(), self.call_count, self._calls_repr()))

    def assert_called_once(self):
        if self.call_count != 1:
            raise AssertionError("Expected '%s' to have been called once. Called %d times.%s" % (
                self._name(), self.call_count, self._calls_repr()))

    def assert_called_with(self, *args, **kwargs):
        if self.call_args is None:
            raise AssertionError("expected call not found.\nExpected: %r\n  Actual: not called." % (_Call(("mock", args, kwargs)),))
        if self.call_args != _Call(("", args, kwargs)):
            raise AssertionError("expected call not found.\nExpected: %r\n  Actual: %r" % (
                _Call(("mock", args, kwargs)), _Call(("mock", self.call_args.args, self.call_args.kwargs))))

    def assert_called_once_with(self, *args, **kwargs):
        if self.call_count != 1:
            raise AssertionError("Expected '%s' to be called once. Called %d times.%s" % (
                self._name(), self.call_count, self._calls_repr()))
        self.assert_called_with(*args, **kwargs)

    def assert_any_call(self, *args, **kwargs):
        if _Call(("", args, kwargs)) not in self.call_args_list:
            raise AssertionError("%r call not found" % (_Call(("mock", args, kwargs)),))

    def assert_has_calls(self, calls, any_order=False):
        calls = list(calls)
        have = _CallList(self.mock_calls)
        if not any_order:
            if calls not in have:
                raise AssertionError("Calls not found.\nExpected: %r\n  Actual: %r" % (calls, self.mock_calls))
            return
        have = list(have)
        for c in calls:
            try:
                have.remove(c)
            except ValueError:
                raise AssertionError("%r not all found in call list" % (tuple(calls),))

    def __repr__(self):
        name = self._full_name()
        n = ""
        if self.__dict__.get("_mock_name"):
            n = " name=%r" % (self.__dict__["_mock_name"] if not name else "mock." + name)
        elif name:
            n = " name=%r" % ("mock." + name)
        spec = ""
        if self._spec_class is not None:
            spec = " spec=%r" % self._spec_class.__name__
        return "<%s%s%s id='%d'>" % (type(self).__name__, n, spec, id(self))

    def __dir__(self):
        return sorted(set(dir(type(self)) + list(self.__dict__["_mock_children"])))


class CallableMixin:
    def __call__(self, *args, **kwargs):
        return self._call(args, kwargs)


class Mock(CallableMixin, NonCallableMock):
    pass


def _magic(name):
    def default(self, *args, **kwargs):
        m = self.__dict__["_mock_children"].get(name)
        if m is None:
            m = MagicMock(parent=self, name=name, _new_name=name)
            rv = _magic_default(self, name)
            if rv is not DEFAULT:
                m.return_value = rv
            self.__dict__["_mock_children"][name] = m
        return m(*args, **kwargs)
    default.__name__ = name
    return default


def _magic_default(self, name):
    defaults = {"__lt__": NotImplemented, "__gt__": NotImplemented, "__le__": NotImplemented,
                "__ge__": NotImplemented, "__int__": 1, "__contains__": False, "__len__": 0,
                "__exit__": False, "__complex__": 1j, "__float__": 1.0, "__bool__": True,
                "__index__": 1, "__hash__": id(self), "__str__": object.__repr__(self) if False else "<MagicMock id='%d'>" % id(self),
                "__sizeof__": 0, "__fspath__": "mock"}
    if name == "__iter__":
        return iter([])
    if name == "__enter__":
        return DEFAULT
    return defaults.get(name, DEFAULT)


class MagicMixin:
    pass


class _MagicProxy:
    # mock.__len__ is a child mock, and len(mock) calls it
    def __init__(self, name):
        self.name = name

    def __get__(self, obj, _type=None):
        if obj is None:
            return self
        m = obj.__dict__["_mock_children"].get(self.name)
        if m is None:
            m = MagicMock(parent=obj, name=self.name, _new_name=self.name)
            rv = _magic_default(obj, self.name)
            if rv is not DEFAULT:
                m.return_value = rv
            obj.__dict__["_mock_children"][self.name] = m
        return m


for _n in _MAGICS:
    if _n != "__hash__":
        setattr(MagicMixin, _n, _MagicProxy(_n))


def _magic_eq(self, other):
    m = self.__dict__["_mock_children"].get("__eq__")
    if m is not None:
        return m(other)
    return self is other


def _magic_ne(self, other):
    m = self.__dict__["_mock_children"].get("__ne__")
    if m is not None:
        return m(other)
    return self is not other


MagicMixin.__eq__ = _magic_eq
MagicMixin.__ne__ = _magic_ne


def _magic_hash(self):
    m = self.__dict__["_mock_children"].get("__hash__")
    if m is not None:
        return m()
    return id(self)


MagicMixin.__hash__ = _magic_hash


class NonCallableMagicMock(MagicMixin, NonCallableMock):
    def __setattr__(self, name, value):
        if name.startswith("__") and name.endswith("__") and name in _MAGICS + ["__eq__", "__ne__"]:
            if not isinstance(value, NonCallableMock):
                v = value
                value = MagicMock(side_effect=lambda *a, **k: v(self, *a, **k) if callable(v) else v)
            self.__dict__["_mock_children"][name] = value
            return
        NonCallableMock.__setattr__(self, name, value)

    def __getattr__(self, name):
        if name in _MAGICS or name in ("__eq__", "__ne__"):
            m = self.__dict__["_mock_children"].get(name)
            if m is None:
                _magic(name)(self) if False else None
                m = MagicMock(parent=self, name=name, _new_name=name)
                rv = _magic_default(self, name)
                if rv is not DEFAULT:
                    m.return_value = rv
                self.__dict__["_mock_children"][name] = m
            return m
        return NonCallableMock.__getattr__(self, name)


class MagicMock(MagicMixin, Mock):
    __setattr__ = NonCallableMagicMock.__setattr__
    __getattr__ = NonCallableMagicMock.__getattr__


class PropertyMock(Mock):
    def __get__(self, obj, obj_type=None):
        return self()

    def __set__(self, obj, val):
        self(val)


def create_autospec(spec, spec_set=False, instance=False, _parent=None, _name=None, **kwargs):
    klass = MagicMock
    if not callable(spec) or instance:
        klass = NonCallableMagicMock if not callable(spec) else MagicMock
    m = klass(spec=spec, **kwargs) if not spec_set else klass(spec_set=spec, **kwargs)
    return m


def seal(mock):
    mock.__dict__["_mock_sealed"] = True
    for child in mock.__dict__["_mock_children"].values():
        if isinstance(child, NonCallableMock):
            seal(child)


def mock_open(mock=None, read_data=""):
    if mock is None:
        mock = MagicMock(name="open")
    handle = MagicMock()
    handle.__enter__.return_value = handle
    lines = read_data.splitlines(keepends=True)
    handle.read.return_value = read_data
    handle.readline.side_effect = lines + [""] * 100
    handle.readlines.return_value = lines
    handle.__iter__.return_value = iter(lines)
    mock.return_value = handle
    return mock


def _importer(target):
    parts = target.split(".")
    import_path = parts.pop(0)
    thing = __import__(import_path)
    for comp in parts:
        import_path += "." + comp
        try:
            thing = getattr(thing, comp)
        except AttributeError:
            __import__(import_path)
            thing = getattr(thing, comp)
    return thing


class _patch:
    def __init__(self, getter, attribute, new, spec, create, spec_set, autospec, new_callable, kwargs):
        self.getter = getter
        self.attribute = attribute
        self.new = new
        self.new_callable = new_callable
        self.spec = spec
        self.create = create
        self.spec_set = spec_set
        self.autospec = autospec
        self.kwargs = kwargs
        self.additional_patchers = []
        self.attribute_name = None

    def copy(self):
        return _patch(self.getter, self.attribute, self.new, self.spec, self.create,
                      self.spec_set, self.autospec, self.new_callable, self.kwargs)

    def __call__(self, func):
        if isinstance(func, type):
            for attr in dir(func):
                if attr.startswith("test"):
                    value = getattr(func, attr)
                    if callable(value):
                        setattr(func, attr, self.copy()(value))
            return func

        @functools.wraps(func)
        def patched(*args, **kw):
            extra = []
            entered = []
            try:
                for p in [self] + self.additional_patchers:
                    v = p.__enter__()
                    entered.append(p)
                    if p.attribute_name is not None:
                        kw[p.attribute_name] = v
                    elif p.new is DEFAULT:
                        extra.append(v)
                return func(*(args + tuple(extra)), **kw)
            finally:
                for p in reversed(entered):
                    p.__exit__(None, None, None)
        return patched

    def __enter__(self):
        self.target = self.getter()
        original = getattr(self.target, self.attribute, _missing)
        if original is _missing and not self.create and not (self.attribute in _builtins and isinstance(self.target, type(sys))):
            raise AttributeError("%r does not have the attribute %r" % (self.target, self.attribute))
        d = getattr(self.target, "__dict__", None)
        self.is_local = isinstance(d, dict) and self.attribute in d or not isinstance(self.target, type)
        self.temp_original = original
        new = self.new
        if new is DEFAULT:
            klass = self.new_callable or MagicMock
            kw = dict(self.kwargs)
            if self.spec is not None and self.spec is not True:
                kw["spec"] = self.spec
            elif self.spec is True:
                kw["spec"] = original
            if self.spec_set is not None and self.spec_set is not False:
                kw["spec_set"] = original if self.spec_set is True else self.spec_set
            if self.autospec is not None and self.autospec is not False:
                new = create_autospec(original if self.autospec is True else self.autospec, **kw)
            else:
                new = klass(**kw)
        self.new_value = new
        setattr(self.target, self.attribute, new)
        return new

    def __exit__(self, *exc):
        if self.temp_original is _missing:
            try:
                delattr(self.target, self.attribute)
            except AttributeError:
                pass
        else:
            setattr(self.target, self.attribute, self.temp_original)
        return False

    def start(self):
        r = self.__enter__()
        _active.append(self)
        return r

    def stop(self):
        if self in _active:
            _active.remove(self)
        return self.__exit__(None, None, None)


_active = []
_builtins = {name for name in dir(__import__("builtins")) if not name.startswith('_')}


def _get_target(target):
    try:
        target, attribute = target.rsplit(".", 1)
    except (TypeError, ValueError, AttributeError):
        raise TypeError("Need a valid target to patch. You supplied: %r" % (target,))
    return (lambda: _importer(target)), attribute


def patch(target, new=DEFAULT, spec=None, create=False, spec_set=None, autospec=None, new_callable=None, **kwargs):
    getter, attribute = _get_target(target)
    return _patch(getter, attribute, new, spec, create, spec_set, autospec, new_callable, kwargs)


def _patch_object(target, attribute, new=DEFAULT, spec=None, create=False, spec_set=None,
                  autospec=None, new_callable=None, **kwargs):
    return _patch(lambda: target, attribute, new, spec, create, spec_set, autospec, new_callable, kwargs)


def _patch_multiple(target, spec=None, create=False, spec_set=None, autospec=None, new_callable=None, **kwargs):
    if isinstance(target, str):
        getter = lambda: _importer(target)
    else:
        getter = lambda: target
    items = list(kwargs.items())
    attribute, new = items[0]
    p = _patch(getter, attribute, new, spec, create, spec_set, autospec, new_callable, {})
    p.attribute_name = attribute if new is DEFAULT else None
    for attribute, new in items[1:]:
        q = _patch(getter, attribute, new, spec, create, spec_set, autospec, new_callable, {})
        q.attribute_name = attribute if new is DEFAULT else None
        p.additional_patchers.append(q)
    return p


class _patch_dict:
    def __init__(self, in_dict, values=(), clear=False, **kwargs):
        self.in_dict = in_dict
        self.values = dict(values)
        self.values.update(kwargs)
        self.clear = clear
        self._original = None

    def __call__(self, f):
        @functools.wraps(f)
        def inner(*args, **kw):
            self.__enter__()
            try:
                return f(*args, **kw)
            finally:
                self.__exit__(None, None, None)
        return inner

    def __enter__(self):
        if isinstance(self.in_dict, str):
            self.in_dict = _importer(self.in_dict)
        self._original = dict(self.in_dict)
        if self.clear:
            self.in_dict.clear()
        self.in_dict.update(self.values)
        return self.in_dict

    def __exit__(self, *args):
        self.in_dict.clear()
        self.in_dict.update(self._original)
        return False

    start = __enter__

    def stop(self):
        return self.__exit__(None, None, None)


def _stopall():
    for p in reversed(list(_active)):
        p.stop()


patch.object = _patch_object
patch.dict = _patch_dict
patch.multiple = _patch_multiple
patch.stopall = _stopall
patch.TEST_PREFIX = "test"

FILTER_DIR = True


class AsyncMock(MagicMock):
    pass
