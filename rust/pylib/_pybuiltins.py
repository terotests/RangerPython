# SPDX-License-Identifier: MIT
# Built-ins of the Rust runtime that are written in Python. The runtime
# copies every name in __all__ into builtins at start-up.

__all__ = ["memoryview"]


class memoryview:
    __slots__ = ("_obj", "_start", "_stop", "_released", "_format", "_itemsize", "_shape")

    def __new__(cls, object):
        if isinstance(object, memoryview):
            object._check()
            m = _new(object._obj, object._start, object._stop)
            m._format = object._format
            m._itemsize = object._itemsize
            return m
        if not isinstance(object, (bytes, bytearray)):
            raise TypeError(f"memoryview: a bytes-like object is required, not '{type(object).__name__}'")
        return _new(object, 0, len(object))

    def _check(self):
        if self._released:
            raise ValueError("operation forbidden on released memoryview object")

    def _bytes(self):
        self._check()
        return bytes(self._obj[self._start:self._stop])

    @property
    def obj(self):
        self._check()
        return self._obj

    @property
    def readonly(self):
        self._check()
        return isinstance(self._obj, bytes)

    @property
    def nbytes(self):
        self._check()
        return self._stop - self._start

    @property
    def format(self):
        return self._format

    @property
    def itemsize(self):
        return self._itemsize

    @property
    def ndim(self):
        return 1

    @property
    def shape(self):
        return (len(self),)

    @property
    def strides(self):
        return (self._itemsize,)

    @property
    def c_contiguous(self):
        return True

    f_contiguous = contiguous = c_contiguous

    def __len__(self):
        self._check()
        return (self._stop - self._start) // self._itemsize

    def __getitem__(self, index):
        self._check()
        if isinstance(index, slice):
            start, stop, step = index.indices(len(self))
            if step != 1:
                return memoryview(bytes(self._bytes()[start:stop:step]))
            m = _new(self._obj, self._start + start, self._start + max(start, stop))
            return m
        n = len(self)
        i = index.__index__()
        if i < 0:
            i += n
        if not 0 <= i < n:
            raise IndexError("index out of bounds on dimension 1")
        if self._itemsize == 1:
            v = self._obj[self._start + i]
            return bytes([v]) if self._format == "c" else v
        import struct
        at = self._start + i * self._itemsize
        return struct.unpack("@" + self._format, bytes(self._obj[at:at + self._itemsize]))[0]

    def __setitem__(self, index, value):
        self._check()
        if isinstance(self._obj, bytes):
            raise TypeError("cannot modify read-only memory")
        if isinstance(index, slice):
            start, stop, step = index.indices(len(self))
            value = bytes(value)
            if len(value) != len(range(start, stop, step)):
                raise ValueError("memoryview assignment: lvalue and rvalue have different structures")
            for k, j in enumerate(range(start, stop, step)):
                self._obj[self._start + j] = value[k]
            return
        i = index.__index__()
        if i < 0:
            i += len(self)
        if not 0 <= i < len(self):
            raise IndexError("index out of bounds on dimension 1")
        self._obj[self._start + i] = value

    def __iter__(self):
        for i in range(len(self)):
            yield self[i]

    def __eq__(self, other):
        if self._released:
            return self is other
        if isinstance(other, memoryview):
            if other._released:
                return False
            return self.tolist() == other.tolist()
        if isinstance(other, (bytes, bytearray)):
            return self._bytes() == bytes(other)
        return NotImplemented

    def __hash__(self):
        if not self.readonly:
            raise ValueError("cannot hash writable memoryview object")
        return hash(self._bytes())

    def __bytes__(self):
        return self._bytes()

    def __buffer__(self, flags):
        return self

    def __release_buffer__(self, view):
        pass

    def __repr__(self):
        if self._released:
            return f"<released memory at {id(self):#x}>"
        return f"<memory at {id(self):#x}>"

    def __enter__(self):
        self._check()
        return self

    def __exit__(self, *exc):
        self.release()

    def release(self):
        self._released = True

    def __buffer__(self, flags, /):
        self._check()
        return memoryview(self)

    def tobytes(self, order="C"):
        return self._bytes()

    def tolist(self):
        self._check()
        return [self[i] for i in range(len(self))]

    def hex(self, *args):
        return self._bytes().hex(*args)

    def toreadonly(self):
        return memoryview(self._bytes())

    def cast(self, format, shape=None):
        self._check()
        import struct
        size = struct.calcsize(format)
        if self.nbytes % size:
            raise TypeError("memoryview: length is not a multiple of itemsize")
        m = _new(self._obj, self._start, self._stop)
        m._format = format
        m._itemsize = size
        return m


def _new(obj, start, stop):
    m = object.__new__(memoryview)
    m._obj = obj
    m._start = start
    m._stop = stop
    m._released = False
    m._format = "B"
    m._itemsize = 1
    return m


memoryview.__module__ = "builtins"
