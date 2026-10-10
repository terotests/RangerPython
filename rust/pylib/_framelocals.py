"""frame.f_locals of a function, or of a module or class body while a
comprehension inlined there runs: a view of the frame's variables as they
are at each access, which writes go through to (PEP 667)."""

import _rpyframe


class FrameLocalsProxy:
    __slots__ = ('_frame', '_extra')

    def __init__(self, frame):
        self._frame = frame
        # names set that are no variable of the frame
        self._extra = {}

    def _now(self):
        d = dict(_rpyframe.locals(self._frame))
        d.update(self._extra)
        return d

    def __getitem__(self, key):
        return self._now()[key]

    def __setitem__(self, key, value):
        if not _rpyframe.set_local(self._frame, key, value):
            self._extra[key] = value

    def __delitem__(self, key):
        if key in self._extra:
            del self._extra[key]
        elif key in self._now():
            raise ValueError("cannot remove local variables from FrameLocalsProxy")
        else:
            raise KeyError(key)

    def __contains__(self, key):
        return key in self._now()

    def __iter__(self):
        return iter(self._now())

    def __reversed__(self):
        return reversed(self._now())

    def __len__(self):
        return len(self._now())

    def __eq__(self, other):
        if isinstance(other, FrameLocalsProxy):
            other = other._now()
        return self._now() == other

    __hash__ = None

    def __repr__(self):
        return repr(self._now())

    def __or__(self, other):
        if isinstance(other, FrameLocalsProxy):
            other = other._now()
        return self._now() | other

    def __ror__(self, other):
        return other | self._now()

    def __ior__(self, other):
        self.update(other)
        return self

    def keys(self):
        return list(self._now())

    def values(self):
        return list(self._now().values())

    def items(self):
        return list(self._now().items())

    def get(self, key, default=None):
        return self._now().get(key, default)

    def setdefault(self, key, default=None):
        d = self._now()
        if key in d:
            return d[key]
        self[key] = default
        return default

    def update(self, other=(), /, **kw):
        for k, v in dict(other, **kw).items():
            self[k] = v

    def pop(self, key, *default):
        if key in self._extra:
            return self._extra.pop(key)
        if key in self._now():
            raise ValueError("cannot remove local variables from FrameLocalsProxy")
        if default:
            return default[0]
        raise KeyError(key)

    def copy(self):
        return self._now()


FrameLocalsProxy.__module__ = 'builtins'
