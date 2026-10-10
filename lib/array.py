# SPDX-License-Identifier: MIT
# array for the Ranger Python runtime: typed arrays kept as lists

typecodes = "bBuwhHiIlLqQfd"
_SIZES = {"b": 1, "B": 1, "u": 4, "w": 4, "h": 2, "H": 2, "i": 4, "I": 4, "l": 8, "L": 8,
          "q": 8, "Q": 8, "f": 4, "d": 8}
_SIGNED = "bhilq"


def _check(tc, v):
    if tc in "fd":
        if not isinstance(v, (int, float)):
            raise TypeError("must be real number, not %s" % type(v).__name__)
        return float(v)
    if tc in "uw":
        if not isinstance(v, str) or len(v) != 1:
            raise TypeError("array item must be a unicode character, not %s" % type(v).__name__)
        return v
    if not isinstance(v, int):
        raise TypeError("'%s' object cannot be interpreted as an integer" % type(v).__name__)
    bits = 8 * _SIZES[tc]
    if tc in _SIGNED:
        lo, hi = -(1 << (bits - 1)), (1 << (bits - 1)) - 1
    else:
        lo, hi = 0, (1 << bits) - 1
    if v < lo or v > hi:
        if tc in _SIGNED:
            raise OverflowError("signed %s is %s than %s" % (
                {"b": "char", "h": "short integer", "i": "integer", "l": "integer", "q": "integer"}[tc],
                "less" if v < lo else "greater", "minimum" if v < lo else "maximum"))
        raise OverflowError("unsigned %s is %s than %s" % (
            {"B": "byte integer", "H": "short", "I": "int", "L": "int", "Q": "int"}[tc],
            "less" if v < lo else "greater", "minimum" if v < lo else "maximum"))
    return v


class array:
    def __init__(self, typecode, initializer=None):
        if not isinstance(typecode, str) or len(typecode) != 1 or typecode not in typecodes:
            raise ValueError("bad typecode (must be b, B, u, w, h, H, i, I, l, L, q, Q, f or d)")
        self.typecode = typecode
        self.itemsize = _SIZES[typecode]
        self._a = []
        if initializer is not None:
            if isinstance(initializer, str) and typecode not in "uw":
                raise TypeError("cannot use a str to initialize an array with typecode '%s'" % typecode)
            self.extend(initializer)

    def __class_getitem__(cls, item):
        import types
        return types.GenericAlias(cls, item)

    def append(self, v):
        self._a.append(_check(self.typecode, v))

    def extend(self, it):
        if isinstance(it, array) and it.typecode != self.typecode:
            raise TypeError("can only extend with array of same kind")
        for v in it:
            self.append(v)

    def fromlist(self, lst):
        if not isinstance(lst, list):
            raise TypeError("arg must be list")
        vals = [_check(self.typecode, v) for v in lst]
        self._a.extend(vals)

    def fromunicode(self, s):
        if self.typecode not in "uw":
            raise ValueError("fromunicode() may only be called on unicode type arrays")
        self._a.extend(s)

    def tolist(self):
        return list(self._a)

    def tounicode(self):
        if self.typecode not in "uw":
            raise ValueError("tounicode() may only be called on unicode type arrays")
        return "".join(self._a)

    def buffer_info(self):
        return (id(self), len(self._a))

    def count(self, v):
        return self._a.count(v)

    def index(self, v, start=0, stop=None):
        try:
            return self._a.index(v, start, len(self._a) if stop is None else stop)
        except ValueError:
            raise ValueError("array.index(x): x not in array")

    def insert(self, i, v):
        self._a.insert(i, _check(self.typecode, v))

    def pop(self, i=-1):
        if not self._a:
            raise IndexError("pop from empty array")
        return self._a.pop(i)

    def remove(self, v):
        try:
            self._a.remove(v)
        except ValueError:
            raise ValueError("array.remove(x): x not in array")

    def reverse(self):
        self._a.reverse()

    def byteswap(self):
        pass

    def __len__(self):
        return len(self._a)

    def __iter__(self):
        return iter(self._a)

    def __reversed__(self):
        return reversed(self._a)

    def __contains__(self, v):
        return v in self._a

    def __getitem__(self, i):
        if isinstance(i, slice):
            r = array(self.typecode)
            r._a = self._a[i]
            return r
        try:
            return self._a[i]
        except IndexError:
            raise IndexError("array index out of range")

    def __setitem__(self, i, v):
        if isinstance(i, slice):
            if not isinstance(v, array):
                raise TypeError("can only assign array (not \"%s\") to array slice" % type(v).__name__)
            self._a[i] = v._a
            return
        try:
            self._a[i] = _check(self.typecode, v)
        except IndexError:
            raise IndexError("array assignment index out of range")

    def __delitem__(self, i):
        del self._a[i]

    def __add__(self, other):
        if not isinstance(other, array):
            return NotImplemented
        if other.typecode != self.typecode:
            raise TypeError("bad argument type for built-in operation")
        r = array(self.typecode)
        r._a = self._a + other._a
        return r

    def __iadd__(self, other):
        if not isinstance(other, array):
            return NotImplemented
        self.extend(other)
        return self

    def __mul__(self, n):
        r = array(self.typecode)
        r._a = self._a * n
        return r

    __rmul__ = __mul__

    def __imul__(self, n):
        self._a *= n
        return self

    def __eq__(self, other):
        if not isinstance(other, array):
            return NotImplemented
        return self._a == other._a

    def __ne__(self, other):
        if not isinstance(other, array):
            return NotImplemented
        return self._a != other._a

    def __lt__(self, other):
        return self._a < other._a

    def __le__(self, other):
        return self._a <= other._a

    def __gt__(self, other):
        return self._a > other._a

    def __ge__(self, other):
        return self._a >= other._a

    __hash__ = None

    def __copy__(self):
        return array(self.typecode, self._a)

    def __deepcopy__(self, memo):
        return array(self.typecode, self._a)

    def __reduce_ex__(self, proto):
        return (array, (self.typecode, list(self._a)))

    def __repr__(self):
        if not self._a:
            return "array('%s')" % self.typecode
        if self.typecode in "uw":
            return "array('%s', %r)" % (self.typecode, "".join(self._a))
        return "array('%s', %r)" % (self.typecode, self._a)


ArrayType = array
