# SPDX-License-Identifier: MIT
# marshal for the Ranger Python runtime: CPython's format for the plain
# value types (code objects are not supported).
import struct as _struct

version = 4

_NULL = b"0"


def _w_long(out, n):
    out += _struct.pack("<i", n)


def _dump(v, out, ver):
    if v is None:
        out += b"N"
    elif v is StopIteration:
        out += b"S"
    elif v is ...:
        out += b"."
    elif v is False:
        out += b"F"
    elif v is True:
        out += b"T"
    elif type(v) is int:
        if -2**31 <= v < 2**31:
            out += b"i"
            _w_long(out, v)
        else:
            out += b"l"
            sign = -1 if v < 0 else 1
            m = abs(v)
            digits = []
            while m:
                digits.append(m & 0x7FFF)
                m >>= 15
            _w_long(out, sign * len(digits))
            for d in digits:
                out += _struct.pack("<H", d)
    elif type(v) is float:
        out += b"g" + _struct.pack("<d", v)
    elif type(v) is complex:
        out += b"y" + _struct.pack("<d", v.real) + _struct.pack("<d", v.imag)
    elif type(v) is str:
        b = v.encode("utf-8", "surrogatepass")
        if ver >= 4 and v.isascii() and len(b) < 256:
            out += b"z" + bytes([len(b)]) + b
        else:
            out += b"u"
            _w_long(out, len(b))
            out += b
    elif type(v) in (bytes, bytearray, memoryview):
        b = bytes(v)
        out += b"s"
        _w_long(out, len(b))
        out += b
    elif type(v) is tuple:
        if ver >= 4 and len(v) < 256:
            out += b")" + bytes([len(v)])
        else:
            out += b"("
            _w_long(out, len(v))
        for x in v:
            _dump(x, out, ver)
    elif type(v) is list:
        out += b"["
        _w_long(out, len(v))
        for x in v:
            _dump(x, out, ver)
    elif type(v) is dict:
        out += b"{"
        for k, x in v.items():
            _dump(k, out, ver)
            _dump(x, out, ver)
        out += _NULL
    elif type(v) in (set, frozenset):
        out += b"<" if type(v) is set else b">"
        _w_long(out, len(v))
        for x in v:
            _dump(x, out, ver)
    else:
        raise ValueError("unmarshallable object")


def dumps(value, version=version, /, *, allow_code=True):
    out = bytearray()
    _dump(value, out, version)
    return bytes(out)


def dump(value, file, version=version, /, *, allow_code=True):
    file.write(dumps(value, version))


class _Reader:
    def __init__(self, data):
        self.data = data
        self.pos = 0
        self.refs = []

    def take(self, n):
        if self.pos + n > len(self.data):
            raise EOFError("marshal data too short")
        b = self.data[self.pos:self.pos + n]
        self.pos += n
        return b

    def long(self):
        return _struct.unpack("<i", self.take(4))[0]

    def load(self):
        if self.pos >= len(self.data):
            raise EOFError("EOF read where object expected")
        code = self.data[self.pos]
        self.pos += 1
        flag = code & 0x80
        t = chr(code & 0x7F)
        idx = None
        if flag and t in "([{<>)ilgysuzZaAtfc":
            idx = len(self.refs)
            self.refs.append(None)
        v = self._value(t, idx)
        if idx is not None:
            self.refs[idx] = v
        return v

    def _value(self, t, idx):
        if t == "N":
            return None
        if t == "F":
            return False
        if t == "T":
            return True
        if t == "S":
            return StopIteration
        if t == ".":
            return ...
        if t == "i":
            return self.long()
        if t == "l":
            n = self.long()
            r = 0
            for i in range(abs(n)):
                r |= _struct.unpack("<H", self.take(2))[0] << (15 * i)
            return -r if n < 0 else r
        if t == "g":
            return _struct.unpack("<d", self.take(8))[0]
        if t == "y":
            re = _struct.unpack("<d", self.take(8))[0]
            return complex(re, _struct.unpack("<d", self.take(8))[0])
        if t == "s":
            return bytes(self.take(self.long()))
        if t in "ut":
            return bytes(self.take(self.long())).decode("utf-8", "surrogatepass")
        if t in "aA":
            return bytes(self.take(self.long())).decode("latin-1")
        if t in "zZ":
            return bytes(self.take(self.take(1)[0])).decode("latin-1")
        if t == ")":
            n = self.take(1)[0]
            return tuple(self.load() for _ in range(n))
        if t == "(":
            n = self.long()
            return tuple(self.load() for _ in range(n))
        if t == "[":
            n = self.long()
            r = []
            if idx is not None:
                self.refs[idx] = r
            for _ in range(n):
                r.append(self.load())
            return r
        if t == "{":
            r = {}
            if idx is not None:
                self.refs[idx] = r
            while True:
                if self.pos < len(self.data) and self.data[self.pos] == 0x30:
                    self.pos += 1
                    return r
                k = self.load()
                r[k] = self.load()
        if t in "<>":
            n = self.long()
            items = [self.load() for _ in range(n)]
            return set(items) if t == "<" else frozenset(items)
        if t == "r":
            n = self.long()
            if not 0 <= n < len(self.refs) or self.refs[n] is None:
                raise ValueError("bad marshal data (invalid reference)")
            return self.refs[n]
        raise ValueError("bad marshal data (unknown type code)")


def loads(data, /, *, allow_code=True):
    return _Reader(bytes(data)).load()


def load(file, /, *, allow_code=True):
    r = _Reader(file.read())
    return r.load()
