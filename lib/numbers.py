# SPDX-License-Identifier: MIT
# numbers for the Ranger Python runtime: the numeric tower as classes that
# answer isinstance() for the built-in numbers and for registered classes


def _tower(cls, builtins):
    cls._registry = []
    cls._builtins = builtins

    def instancecheck(c, obj):
        return subclasscheck(c, type(obj))

    def subclasscheck(c, sub):
        if sub is bool:
            sub = int
        if sub in c._builtins:
            return True
        for base in getattr(sub, "__mro__", (sub,)):
            if base is c:
                return True
            for t in _all:
                if c in t.__mro__ and (base in t._registry):
                    return True
        return False

    def register(c, sub):
        c._registry.append(sub)
        return sub
    cls.__instancecheck__ = classmethod(instancecheck)
    cls.__subclasscheck__ = classmethod(subclasscheck)
    cls.register = classmethod(register)
    _all.append(cls)
    return cls


_all = []


class Number:
    __slots__ = ()
    __hash__ = None


class Complex(Number):
    __slots__ = ()

    def __bool__(self):
        return self != 0

    @property
    def real(self):
        raise NotImplementedError

    @property
    def imag(self):
        raise NotImplementedError


class Real(Complex):
    __slots__ = ()

    def __complex__(self):
        return complex(float(self))

    @property
    def real(self):
        return +self

    @property
    def imag(self):
        return 0

    def conjugate(self):
        return +self

    def __divmod__(self, other):
        return (self // other, self % other)

    def __rdivmod__(self, other):
        return (other // self, other % self)


class Rational(Real):
    __slots__ = ()

    def __float__(self):
        return int(self.numerator) / int(self.denominator)


class Integral(Rational):
    __slots__ = ()

    def __float__(self):
        return float(int(self))

    def __index__(self):
        return int(self)

    @property
    def numerator(self):
        return +self

    @property
    def denominator(self):
        return 1


_tower(Number, (int, float, complex))
_tower(Complex, (int, float, complex))
_tower(Real, (int, float))
_tower(Rational, (int,))
_tower(Integral, (int,))
