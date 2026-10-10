# SPDX-License-Identifier: MIT
# fractions for the Ranger Python runtime
import math
import numbers
import re

__all__ = ["Fraction"]

_RATIONAL_FORMAT = re.compile(r"""
    \A\s*
    (?P<sign>[-+]?)
    (?=\d|\.\d)
    (?P<num>\d*|\d+(_\d+)*)
    (?:
        (?:\s*/\s*(?P<denom>\d+(_\d+)*))?
        |
        (?:\.(?P<decimal>\d*|\d+(_\d+)*))?
        (?:E(?P<exp>[-+]?\d+(_\d+)*))?
    )
    \s*\Z
""", re.VERBOSE | re.IGNORECASE)


class Fraction:
    __slots__ = ("_numerator", "_denominator")

    def __new__(cls, numerator=0, denominator=None):
        self = object.__new__(cls)
        if denominator is None:
            if type(numerator) is int:
                self._numerator = numerator
                self._denominator = 1
                return self
            if isinstance(numerator, Fraction):
                self._numerator = numerator._numerator
                self._denominator = numerator._denominator
                return self
            if isinstance(numerator, (float, int)) or hasattr(numerator, "as_integer_ratio"):
                if isinstance(numerator, float) and (numerator != numerator or numerator in (float("inf"), float("-inf"))):
                    raise (ValueError if numerator != numerator else OverflowError)(
                        "cannot convert %s to integer ratio" % ("NaN" if numerator != numerator else "Infinity"))
                n, d = numerator.as_integer_ratio()
                self._numerator = int(n)
                self._denominator = int(d)
                return self
            if isinstance(numerator, str):
                m = _RATIONAL_FORMAT.match(numerator)
                if m is None:
                    raise ValueError("Invalid literal for Fraction: %r" % numerator)
                numerator = int(m.group("num") or "0")
                denom = m.group("denom")
                if denom:
                    denominator = int(denom)
                else:
                    denominator = 1
                    decimal = m.group("decimal")
                    if decimal:
                        decimal = decimal.replace("_", "")
                        scale = 10 ** len(decimal)
                        numerator = numerator * scale + int(decimal)
                        denominator *= scale
                    exp = m.group("exp")
                    if exp:
                        exp = int(exp)
                        if exp >= 0:
                            numerator *= 10 ** exp
                        else:
                            denominator *= 10 ** -exp
                if m.group("sign") == "-":
                    numerator = -numerator
            else:
                raise TypeError("argument should be a string or a Rational instance")
        elif type(numerator) is int and type(denominator) is int:
            pass
        elif isinstance(numerator, (int, Fraction)) and isinstance(denominator, (int, Fraction)):
            a = Fraction(numerator)
            b = Fraction(denominator)
            numerator = a._numerator * b._denominator
            denominator = a._denominator * b._numerator
        else:
            raise TypeError("both arguments should be Rational instances")
        if denominator == 0:
            raise ZeroDivisionError("Fraction(%s, 0)" % numerator)
        g = math.gcd(numerator, denominator)
        if denominator < 0:
            g = -g
        self._numerator = numerator // g
        self._denominator = denominator // g
        return self

    @classmethod
    def from_float(cls, f):
        if isinstance(f, int):
            return cls(f)
        if not isinstance(f, float):
            raise TypeError("%s.from_float() only takes floats, not %r (%s)" % (cls.__name__, f, type(f).__name__))
        return cls(*f.as_integer_ratio())

    @classmethod
    def from_decimal(cls, dec):
        return cls(str(dec))

    @classmethod
    def _from_coprime_ints(cls, numerator, denominator):
        obj = object.__new__(cls)
        obj._numerator = numerator
        obj._denominator = denominator
        return obj

    def is_integer(self):
        return self._denominator == 1

    def as_integer_ratio(self):
        return (self._numerator, self._denominator)

    def limit_denominator(self, max_denominator=1000000):
        if max_denominator < 1:
            raise ValueError("max_denominator should be at least 1")
        if self._denominator <= max_denominator:
            return Fraction(self)
        p0, q0, p1, q1 = 0, 1, 1, 0
        n, d = self._numerator, self._denominator
        while True:
            a = n // d
            q2 = q0 + a * q1
            if q2 > max_denominator:
                break
            p0, q0, p1, q1 = p1, q1, p0 + a * p1, q2
            n, d = d, n - a * d
        k = (max_denominator - q0) // q1
        if 2 * d * (q0 + k * q1) <= self._denominator:
            return Fraction._from_coprime_ints(p1, q1)
        return Fraction._from_coprime_ints(p0 + k * p1, q0 + k * q1)

    @property
    def numerator(self):
        return self._numerator

    @property
    def denominator(self):
        return self._denominator

    @property
    def real(self):
        return self

    @property
    def imag(self):
        return 0

    def conjugate(self):
        return self

    def __repr__(self):
        return "%s(%s, %s)" % (self.__class__.__name__, self._numerator, self._denominator)

    def __str__(self):
        if self._denominator == 1:
            return str(self._numerator)
        return "%s/%s" % (self._numerator, self._denominator)

    def __format__(self, spec):
        if not spec:
            return str(self)
        return format(float(self), spec)

    def _op(self, other, fn, rev=False):
        if isinstance(other, (int, Fraction)):
            o = Fraction(other)
            return fn(o, self) if rev else fn(self, o)
        if isinstance(other, float):
            return fn(float(other), float(self)) if rev else fn(float(self), other)
        if isinstance(other, complex):
            return fn(other, complex(float(self))) if rev else fn(complex(float(self)), other)
        return NotImplemented

    def __add__(a, b):
        return a._op(b, lambda x, y: x + y if not isinstance(x, Fraction) else
                     Fraction(x._numerator * y._denominator + y._numerator * x._denominator, x._denominator * y._denominator))

    def __radd__(a, b):
        return a._op(b, lambda x, y: x + y if not isinstance(x, Fraction) else Fraction.__add__(x, y), True)

    def __sub__(a, b):
        return a._op(b, lambda x, y: x - y if not isinstance(x, Fraction) else
                     Fraction(x._numerator * y._denominator - y._numerator * x._denominator, x._denominator * y._denominator))

    def __rsub__(a, b):
        return a._op(b, lambda x, y: x - y if not isinstance(x, Fraction) else Fraction.__sub__(x, y), True)

    def __mul__(a, b):
        return a._op(b, lambda x, y: x * y if not isinstance(x, Fraction) else
                     Fraction(x._numerator * y._numerator, x._denominator * y._denominator))

    def __rmul__(a, b):
        return a._op(b, lambda x, y: x * y if not isinstance(x, Fraction) else Fraction.__mul__(x, y), True)

    def __truediv__(a, b):
        return a._op(b, lambda x, y: x / y if not isinstance(x, Fraction) else
                     Fraction(x._numerator * y._denominator, x._denominator * y._numerator))

    def __rtruediv__(a, b):
        return a._op(b, lambda x, y: x / y if not isinstance(x, Fraction) else Fraction.__truediv__(x, y), True)

    def __floordiv__(a, b):
        return a._op(b, lambda x, y: x // y if not isinstance(x, Fraction) else
                     (x._numerator * y._denominator) // (y._numerator * x._denominator))

    def __rfloordiv__(a, b):
        return a._op(b, lambda x, y: x // y if not isinstance(x, Fraction) else Fraction.__floordiv__(x, y), True)

    def __mod__(a, b):
        return a._op(b, lambda x, y: x % y if not isinstance(x, Fraction) else
                     Fraction((x._numerator * y._denominator) % (y._numerator * x._denominator), x._denominator * y._denominator))

    def __rmod__(a, b):
        return a._op(b, lambda x, y: x % y if not isinstance(x, Fraction) else Fraction.__mod__(x, y), True)

    def __divmod__(a, b):
        r = a._op(b, lambda x, y: divmod(x, y) if not isinstance(x, Fraction) else (x // y, x % y))
        return r

    def __rdivmod__(a, b):
        return a._op(b, lambda x, y: divmod(x, y) if not isinstance(x, Fraction) else (x // y, x % y), True)

    def __pow__(a, b):
        if isinstance(b, int) or (isinstance(b, Fraction) and b._denominator == 1):
            p = int(b)
            if p >= 0:
                return Fraction._from_coprime_ints(a._numerator ** p, a._denominator ** p)
            if a._numerator == 0:
                raise ZeroDivisionError("Fraction(%s, 0)" % (a._denominator ** -p))
            if a._numerator > 0:
                return Fraction._from_coprime_ints(a._denominator ** -p, a._numerator ** -p)
            return Fraction._from_coprime_ints((-a._denominator) ** -p, (-a._numerator) ** -p)
        if isinstance(b, (float, complex)):
            return float(a) ** b
        if isinstance(b, Fraction):
            return float(a) ** float(b)
        return NotImplemented

    def __rpow__(b, a):
        if b._denominator == 1 and b._numerator >= 0:
            return a ** b._numerator
        if isinstance(a, (int, Fraction)):
            return Fraction(a) ** b
        return a ** float(b)

    def __pos__(a):
        return Fraction._from_coprime_ints(a._numerator, a._denominator)

    def __neg__(a):
        return Fraction._from_coprime_ints(-a._numerator, a._denominator)

    def __abs__(a):
        return Fraction._from_coprime_ints(abs(a._numerator), a._denominator)

    def __int__(a):
        if a._numerator < 0:
            return -(-a._numerator // a._denominator)
        return a._numerator // a._denominator

    def __trunc__(a):
        return a.__int__()

    def __floor__(a):
        return a._numerator // a._denominator

    def __ceil__(a):
        return -(-a._numerator // a._denominator)

    def __round__(self, ndigits=None):
        if ndigits is None:
            d = self._denominator
            floor, remainder = divmod(self._numerator, d)
            if remainder * 2 < d:
                return floor
            if remainder * 2 > d:
                return floor + 1
            return floor if floor % 2 == 0 else floor + 1
        shift = 10 ** abs(ndigits)
        if ndigits > 0:
            return Fraction(round(self * shift), shift)
        return Fraction(round(self / shift) * shift)

    def __float__(a):
        return a._numerator / a._denominator

    def __hash__(self):
        if self._denominator == 1:
            return hash(self._numerator)
        return hash(self._numerator / self._denominator)

    def __eq__(a, b):
        if isinstance(b, int):
            return a._numerator == b and a._denominator == 1
        if isinstance(b, Fraction):
            return a._numerator == b._numerator and a._denominator == b._denominator
        if isinstance(b, complex) and b.imag == 0:
            b = b.real
        if isinstance(b, float):
            if b != b or b in (float("inf"), float("-inf")):
                return False
            return a == Fraction(b)
        return NotImplemented

    def _cmp(a, b, op):
        if isinstance(b, (int, Fraction)):
            b = Fraction(b)
            return op(a._numerator * b._denominator, b._numerator * a._denominator)
        if isinstance(b, float):
            if b != b:
                return False
            if b in (float("inf"), float("-inf")):
                return op(0.0, b)
            return a._cmp(Fraction(b), op)
        return NotImplemented

    def __lt__(a, b):
        return a._cmp(b, lambda x, y: x < y)

    def __gt__(a, b):
        return a._cmp(b, lambda x, y: x > y)

    def __le__(a, b):
        return a._cmp(b, lambda x, y: x <= y)

    def __ge__(a, b):
        return a._cmp(b, lambda x, y: x >= y)

    def __bool__(a):
        return a._numerator != 0

    def __reduce__(self):
        return (self.__class__, (self._numerator, self._denominator))

    def __copy__(self):
        return self

    def __deepcopy__(self, memo):
        return self


numbers.Rational.register(Fraction)
