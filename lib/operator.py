# SPDX-License-Identifier: MIT
# operator for the Ranger Python runtime.

def lt(a, b): return a < b
def le(a, b): return a <= b
def eq(a, b): return a == b
def ne(a, b): return a != b
def ge(a, b): return a >= b
def gt(a, b): return a > b
def not_(a): return not a
def truth(a): return bool(a)
def is_(a, b): return a is b
def is_not(a, b): return a is not b
def is_none(a): return a is None
def is_not_none(a): return a is not None
def abs(a): return __builtins_abs(a)
def add(a, b): return a + b
def and_(a, b): return a & b
def floordiv(a, b): return a // b
def index(a): return a.__index__() if hasattr(a, "__index__") else int(a)
def inv(a): return ~a
invert = inv
def lshift(a, b): return a << b
def mod(a, b): return a % b
def mul(a, b): return a * b
def matmul(a, b): return a @ b
def neg(a): return -a
def or_(a, b): return a | b
def pos(a): return +a
def pow(a, b): return a ** b
def rshift(a, b): return a >> b
def sub(a, b): return a - b
def truediv(a, b): return a / b
def xor(a, b): return a ^ b
def concat(a, b): return a + b
def contains(a, b): return b in a
def countOf(a, b):
    n = 0
    for x in a:
        if x is b or x == b:
            n += 1
    return n
def delitem(a, b): del a[b]
def getitem(a, b): return a[b]
def indexOf(a, b):
    for i, x in enumerate(a):
        if x is b or x == b:
            return i
    raise ValueError("sequence.index(x): x not in sequence")
def setitem(a, b, c): a[b] = c
def length_hint(obj, default=0):
    try:
        return len(obj)
    except TypeError:
        return default
def iadd(a, b):
    a += b
    return a
def isub(a, b):
    a -= b
    return a
def imul(a, b):
    a *= b
    return a
def iconcat(a, b):
    a += b
    return a

def __builtins_abs(a):
    return a if a >= 0 else -a


class attrgetter:
    def __init__(self, attr, *attrs):
        self._attrs = (attr,) + attrs

    def __call__(self, obj):
        out = []
        for name in self._attrs:
            v = obj
            for part in name.split("."):
                v = getattr(v, part)
            out.append(v)
        if len(out) == 1:
            return out[0]
        return tuple(out)


class itemgetter:
    def __init__(self, item, *items):
        self._items = (item,) + items

    def __call__(self, obj):
        if len(self._items) == 1:
            return obj[self._items[0]]
        return tuple(obj[i] for i in self._items)


class methodcaller:
    def __init__(self, name, *args, **kwargs):
        self._name = name
        self._args = args
        self._kwargs = kwargs

    def __call__(self, obj):
        return getattr(obj, self._name)(*self._args, **self._kwargs)
