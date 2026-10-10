# SPDX-License-Identifier: MIT
# struct for the Rust runtime; float bit patterns come from the native _struct.
import _struct

__all__ = ["calcsize", "pack", "pack_into", "unpack", "unpack_from", "iter_unpack", "Struct", "error"]


class error(Exception):
    pass


# code: (standard size, native size, native alignment)
_SIZES = {
    "x": (1, 1, 1), "c": (1, 1, 1), "b": (1, 1, 1), "B": (1, 1, 1), "?": (1, 1, 1),
    "h": (2, 2, 2), "H": (2, 2, 2), "i": (4, 4, 4), "I": (4, 4, 4), "l": (4, 8, 8),
    "L": (4, 8, 8), "q": (8, 8, 8), "Q": (8, 8, 8), "n": (None, 8, 8), "N": (None, 8, 8),
    "e": (2, 2, 2), "f": (4, 4, 4), "d": (8, 8, 8), "s": (1, 1, 1), "p": (1, 1, 1),
    "P": (None, 8, 8),
}
_SIGNED = set("bhilqn")


def _parse(fmt):
    if isinstance(fmt, (bytes, bytearray)):
        fmt = fmt.decode("ascii")
    if not isinstance(fmt, str):
        raise TypeError(f"Struct() argument 1 must be a str or bytes object, not {type(fmt).__name__}")
    order = "@"
    i = 0
    if fmt and fmt[0] in "@=<>!":
        order = fmt[0]
        i = 1
    native = order == "@"
    little = order in "@=<"
    items = []
    size = 0
    n = len(fmt)
    while i < n:
        c = fmt[i]
        if c.isspace():
            i += 1
            continue
        count = None
        if c.isdigit():
            j = i
            while j < n and fmt[j].isdigit():
                j += 1
            count = int(fmt[i:j])
            i = j
            if i >= n:
                raise error("repeat count given without format specifier")
            c = fmt[i]
        if c not in _SIZES or (not native and c in "nNP"):
            raise error("bad char in struct format")
        std, nsize, align = _SIZES[c]
        sz = nsize if native else std
        if native and align > 1:
            size = (size + align - 1) // align * align
        if c in "sp":
            items.append((c, size, 1 if count is None else count))
            size += 1 if count is None else count
        elif c == "x":
            size += 1 if count is None else count
        else:
            for _ in range(1 if count is None else count):
                items.append((c, size, sz))
                size += sz
        i += 1
    return items, size, little


def _int(v, c):
    if not isinstance(v, int):
        try:
            v = v.__index__()
        except AttributeError:
            raise error("required argument is not an integer") from None
    return v


class Struct:
    def __init__(self, format):
        self.format = format if isinstance(format, str) else format.decode("ascii")
        self._items, self.size, self._little = _parse(format)

    def __repr__(self):
        return f"Struct({self.format!r})"

    def pack(self, *args):
        nvals = len(self._items)
        if len(args) != nvals:
            raise error(f"pack expected {nvals} items for packing (got {len(args)})")
        out = bytearray(self.size)
        self._pack_into(out, 0, args)
        return bytes(out)

    def pack_into(self, buffer, offset, *args):
        nvals = len(self._items)
        if len(args) != nvals:
            raise error(f"pack_into expected {nvals} items for packing (got {len(args)})")
        if offset < 0:
            offset += len(buffer)
        if len(buffer) - offset < self.size:
            raise error(f"pack_into requires a buffer of at least {self.size + offset} bytes for packing {self.size} bytes at offset {offset} (actual buffer size is {len(buffer)})")
        self._pack_into(buffer, offset, args)

    def _pack_into(self, out, base, args):
        order = "little" if self._little else "big"
        for (c, off, sz), v in zip(self._items, args):
            at = base + off
            if c == "s" or c == "p":
                if not isinstance(v, (bytes, bytearray)):
                    raise error("argument for 's' must be a bytes object")
                if c == "p":
                    n = min(len(v), sz - 1, 255)
                    data = bytes([n]) + bytes(v[:n])
                else:
                    data = bytes(v[:sz])
                out[at:at + len(data)] = data
                continue
            if c == "c":
                if not isinstance(v, (bytes, bytearray)) or len(v) != 1:
                    raise error("char format requires a bytes object of length 1")
                out[at] = v[0]
                continue
            if c == "?":
                out[at] = 1 if v else 0
                continue
            if c in "efd":
                if not isinstance(v, (int, float)):
                    try:
                        v = v.__float__()
                    except AttributeError:
                        raise error("required argument is not a float") from None
                if c == "d":
                    bits = _struct.double_bits(float(v))
                elif c == "f":
                    bits = _struct.float_bits(float(v))
                else:
                    bits = _struct.half_bits(float(v))
                out[at:at + sz] = bits.to_bytes(sz, order)
                continue
            v = _int(v, c)
            signed = c in _SIGNED
            bits = sz * 8
            lo, hi = (-(1 << (bits - 1)), (1 << (bits - 1)) - 1) if signed else (0, (1 << bits) - 1)
            if not lo <= v <= hi:
                if signed:
                    raise error(f"'{c}' format requires {lo} <= number <= {hi}")
                raise error(f"'{c}' format requires 0 <= number <= {hi}")
            out[at:at + sz] = v.to_bytes(sz, order, signed=signed)

    def unpack(self, buffer):
        if len(buffer) != self.size:
            raise error(f"unpack requires a buffer of {self.size} bytes")
        return self._unpack(bytes(buffer), 0)

    def unpack_from(self, buffer, offset=0):
        if offset < 0:
            offset += len(buffer)
        if len(buffer) - offset < self.size:
            raise error(f"unpack_from requires a buffer of at least {self.size + offset} bytes for unpacking {self.size} bytes at offset {offset} (actual buffer size is {len(buffer)})")
        return self._unpack(bytes(buffer), offset)

    def iter_unpack(self, buffer):
        if self.size == 0:
            raise error("cannot iteratively unpack with a struct of length 0")
        if len(buffer) % self.size:
            raise error(f"iterative unpacking requires a buffer of a multiple of {self.size} bytes")
        b = bytes(buffer)
        for i in range(0, len(b), self.size):
            yield self._unpack(b, i)

    def _unpack(self, b, base):
        order = "little" if self._little else "big"
        res = []
        for c, off, sz in self._items:
            at = base + off
            if c == "s":
                res.append(b[at:at + sz])
            elif c == "p":
                n = min(b[at], sz - 1)
                res.append(b[at + 1:at + 1 + n])
            elif c == "c":
                res.append(b[at:at + 1])
            elif c == "?":
                res.append(b[at] != 0)
            elif c == "d":
                res.append(_struct.bits_double(int.from_bytes(b[at:at + 8], order)))
            elif c == "f":
                res.append(_struct.bits_float(int.from_bytes(b[at:at + 4], order)))
            elif c == "e":
                res.append(_struct.bits_half(int.from_bytes(b[at:at + 2], order)))
            else:
                res.append(int.from_bytes(b[at:at + sz], order, signed=c in _SIGNED))
        return tuple(res)


_cache = {}


def _get(fmt):
    s = _cache.get(fmt)
    if s is None:
        s = _cache[fmt] = Struct(fmt)
    return s


def calcsize(format):
    return _get(format).size


def pack(format, *args):
    return _get(format).pack(*args)


def pack_into(format, buffer, offset, *args):
    return _get(format).pack_into(buffer, offset, *args)


def unpack(format, buffer):
    return _get(format).unpack(buffer)


def unpack_from(format, buffer, offset=0):
    return _get(format).unpack_from(buffer, offset)


def iter_unpack(format, buffer):
    return _get(format).iter_unpack(buffer)
