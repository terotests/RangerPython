# SPDX-License-Identifier: MIT
# decimal for the Ranger Python runtime: Decimal as sign, coefficient and
# exponent, exact for +, -, * and comparisons. Division rounds through
# float, and integers here are not unbounded, so precision is limited.
import math

__all__ = ["Decimal", "Context", "getcontext", "setcontext", "localcontext", "DecimalException",
           "InvalidOperation", "DivisionByZero", "Inexact", "Rounded", "Overflow", "Underflow",
           "ROUND_HALF_EVEN", "ROUND_HALF_UP", "ROUND_DOWN", "ROUND_UP", "ROUND_FLOOR",
           "ROUND_CEILING", "ROUND_HALF_DOWN", "ROUND_05UP", "MAX_PREC", "MAX_EMAX", "MIN_EMIN"]

ROUND_DOWN = "ROUND_DOWN"
ROUND_HALF_UP = "ROUND_HALF_UP"
ROUND_HALF_EVEN = "ROUND_HALF_EVEN"
ROUND_CEILING = "ROUND_CEILING"
ROUND_FLOOR = "ROUND_FLOOR"
ROUND_UP = "ROUND_UP"
ROUND_HALF_DOWN = "ROUND_HALF_DOWN"
ROUND_05UP = "ROUND_05UP"
MAX_PREC = 999999999999999999
MAX_EMAX = 999999999999999999
MIN_EMIN = -999999999999999999


class DecimalException(ArithmeticError):
    pass


class InvalidOperation(DecimalException):
    pass


class DivisionByZero(DecimalException, ZeroDivisionError):
    pass


class Inexact(DecimalException):
    pass


class Rounded(DecimalException):
    pass


class Overflow(Inexact, Rounded):
    pass


class Underflow(Inexact, Rounded):
    pass


class Context:
    def __init__(self, prec=28, rounding=ROUND_HALF_EVEN, Emin=-999999, Emax=999999, capitals=1,
                 clamp=0, flags=None, traps=None):
        self.prec = prec
        self.rounding = rounding
        self.Emin = Emin
        self.Emax = Emax
        self.capitals = capitals
        self.clamp = clamp
        self.flags = {}
        self.traps = {InvalidOperation: True, DivisionByZero: True, Overflow: True}

    def copy(self):
        c = Context(self.prec, self.rounding, self.Emin, self.Emax, self.capitals, self.clamp)
        return c

    def __repr__(self):
        return "Context(prec=%d, rounding=%s, Emin=%d, Emax=%d, capitals=%d, clamp=%d, flags=[], traps=[InvalidOperation, DivisionByZero, Overflow])" % (
            self.prec, self.rounding, self.Emin, self.Emax, self.capitals, self.clamp)


DefaultContext = Context()
BasicContext = Context(prec=9, rounding=ROUND_HALF_UP)
ExtendedContext = Context(prec=9)
_current = [Context()]


def getcontext():
    return _current[0]


def setcontext(ctx):
    _current[0] = ctx


class _LocalContext:
    def __init__(self, ctx, kw):
        self.new = (ctx or getcontext()).copy()
        for k, v in kw.items():
            setattr(self.new, k, v)

    def __enter__(self):
        self.saved = getcontext()
        setcontext(self.new)
        return self.new

    def __exit__(self, *exc):
        setcontext(self.saved)


def localcontext(ctx=None, **kwargs):
    return _LocalContext(ctx, kwargs)


class Decimal:
    __slots__ = ("_sign", "_int", "_exp", "_special")

    def __new__(cls, value="0", context=None):
        self = object.__new__(cls)
        self._special = None
        if isinstance(value, Decimal):
            self._sign, self._int, self._exp, self._special = value._sign, value._int, value._exp, value._special
            return self
        if isinstance(value, bool) or isinstance(value, int):
            self._sign = 1 if value < 0 else 0
            self._int = abs(value)
            self._exp = 0
            return self
        if isinstance(value, float):
            if value != value:
                self._sign, self._int, self._exp, self._special = 0, 0, 0, "n"
                return self
            if value in (float("inf"), float("-inf")):
                self._sign, self._int, self._exp, self._special = (1 if value < 0 else 0), 0, 0, "F"
                return self
            return Decimal.__new__(cls, repr(value))
        if isinstance(value, tuple):
            sign, digits, exp = value
            self._sign = sign
            self._int = int("".join(str(d) for d in digits) or "0")
            if exp in ("F", "n", "N"):
                self._special = exp
                self._exp = 0
            else:
                self._exp = exp
            return self
        if not isinstance(value, str):
            raise TypeError("conversion from %s to Decimal is not supported" % type(value).__name__)
        s = value.strip().replace("_", "")
        sign = 0
        if s[:1] in "+-" and s:
            sign = 1 if s[0] == "-" else 0
            s = s[1:]
        low = s.lower()
        if low in ("inf", "infinity"):
            self._sign, self._int, self._exp, self._special = sign, 0, 0, "F"
            return self
        if low in ("nan", "snan") or (low.startswith("nan") and low[3:].isdigit()):
            self._sign, self._int, self._exp, self._special = sign, 0, 0, "n"
            return self
        exp = 0
        if "e" in low:
            low, e = low.split("e", 1)
            try:
                exp = int(e)
            except ValueError:
                raise InvalidOperation("[<class 'decimal.ConversionSyntax'>]")
        if "." in low:
            a, b = low.split(".", 1)
        else:
            a, b = low, ""
        digits = a + b
        if not digits or not digits.isdigit():
            raise InvalidOperation("[<class 'decimal.ConversionSyntax'>]")
        self._sign = sign
        self._int = int(digits)
        self._exp = exp - len(b)
        return self

    @classmethod
    def from_float(cls, f):
        return cls(f)

    def _isnan(self):
        return self._special == "n"

    def is_nan(self):
        return self._special in ("n", "N")

    def is_infinite(self):
        return self._special == "F"

    def is_finite(self):
        return self._special is None

    def is_zero(self):
        return self._special is None and self._int == 0

    def is_signed(self):
        return self._sign == 1

    def _frac(self):
        from fractions import Fraction
        v = Fraction(self._int) * (Fraction(10) ** self._exp)
        return -v if self._sign else v

    def _digits(self):
        return str(self._int)

    def as_tuple(self):
        return (self._sign, tuple(int(d) for d in self._digits()), self._exp if self._special is None else self._special)

    def __repr__(self):
        return "Decimal('%s')" % self

    def __str__(self, eng=False, context=None):
        sign = "-" if self._sign else ""
        if self._special == "F":
            return sign + "Infinity"
        if self._special:
            return sign + "NaN"
        digits = self._digits()
        exp = self._exp
        leftdigits = exp + len(digits)
        if exp <= 0 and leftdigits > -6:
            dotplace = leftdigits
        else:
            dotplace = 1
        if dotplace <= 0:
            intpart = "0"
            fracpart = "." + "0" * (-dotplace) + digits
        elif dotplace >= len(digits):
            intpart = digits + "0" * (dotplace - len(digits))
            fracpart = ""
        else:
            intpart = digits[:dotplace]
            fracpart = "." + digits[dotplace:]
        if leftdigits == dotplace:
            e = ""
        else:
            e = "E%+d" % (leftdigits - dotplace)
        return sign + intpart + fracpart + e

    def to_eng_string(self, context=None):
        return str(self)

    def _coerce(self, other):
        if isinstance(other, Decimal):
            return other
        if isinstance(other, int):
            return Decimal(other)
        return None

    def _from_frac(self, f, exact_exp=None):
        if f.denominator == 1:
            d = Decimal(int(f.numerator))
            if exact_exp is not None and exact_exp < 0:
                d._int = d._int * 10 ** (-exact_exp)
                d._exp = exact_exp
            return d
        v = f.numerator / f.denominator
        return Decimal(repr(v))

    def __add__(self, other):
        o = self._coerce(other)
        if o is None:
            return NotImplemented
        if self._special or o._special:
            return Decimal(float(self) + float(o))
        e = min(self._exp, o._exp)
        a = self._int * 10 ** (self._exp - e) * (-1 if self._sign else 1)
        b = o._int * 10 ** (o._exp - e) * (-1 if o._sign else 1)
        r = Decimal((1 if a + b < 0 else 0, tuple(int(c) for c in str(abs(a + b))), e))
        if a + b == 0 and self._sign and o._sign:
            r._sign = 1
        return r

    __radd__ = __add__

    def __sub__(self, other):
        o = self._coerce(other)
        if o is None:
            return NotImplemented
        return self + (-o)

    def __rsub__(self, other):
        o = self._coerce(other)
        if o is None:
            return NotImplemented
        return o - self

    def __mul__(self, other):
        o = self._coerce(other)
        if o is None:
            return NotImplemented
        if self._special or o._special:
            return Decimal(float(self) * float(o))
        r = Decimal((self._sign ^ o._sign, tuple(int(c) for c in str(self._int * o._int)), self._exp + o._exp))
        return r

    __rmul__ = __mul__

    def __truediv__(self, other):
        o = self._coerce(other)
        if o is None:
            return NotImplemented
        if o.is_zero():
            if self.is_zero():
                raise InvalidOperation("[<class 'decimal.DivisionUndefined'>]")
            raise DivisionByZero("[<class 'decimal.DivisionByZero'>]")
        f = self._frac() / o._frac()
        return self._from_frac(f)

    def __rtruediv__(self, other):
        o = self._coerce(other)
        if o is None:
            return NotImplemented
        return o / self

    def __floordiv__(self, other):
        o = self._coerce(other)
        if o is None:
            return NotImplemented
        q = self._frac() / o._frac()
        n = int(q)
        return Decimal(n)

    def __mod__(self, other):
        o = self._coerce(other)
        if o is None:
            return NotImplemented
        return self - o * (self // o)

    def __pow__(self, other):
        o = self._coerce(other)
        if o is None:
            return NotImplemented
        if o._exp >= 0 and not o._special:
            n = int(o)
            if n >= 0:
                r = Decimal(1)
                for _ in range(n):
                    r = r * self
                return r
            return Decimal(1) / (self ** -n)
        return Decimal(float(self) ** float(o))

    def __neg__(self):
        r = Decimal(self)
        r._sign = 1 - self._sign
        return r

    def __pos__(self):
        return Decimal(self)

    def __abs__(self):
        r = Decimal(self)
        r._sign = 0
        return r

    def _cmp(self, other):
        o = self._coerce(other)
        if o is None:
            if isinstance(other, float):
                o = Decimal(other)
            else:
                return None
        if self._special or o._special:
            a, b = float(self), float(o)
            return (a > b) - (a < b)
        a = self._frac()
        b = o._frac()
        return (a > b) - (a < b)

    def __eq__(self, other):
        if self._isnan():
            return False
        c = self._cmp(other)
        return NotImplemented if c is None else c == 0

    def __lt__(self, other):
        c = self._cmp(other)
        return NotImplemented if c is None else c < 0

    def __le__(self, other):
        c = self._cmp(other)
        return NotImplemented if c is None else c <= 0

    def __gt__(self, other):
        c = self._cmp(other)
        return NotImplemented if c is None else c > 0

    def __ge__(self, other):
        c = self._cmp(other)
        return NotImplemented if c is None else c >= 0

    def __hash__(self):
        if self._special:
            return hash(float(self))
        f = self._frac()
        if f.denominator == 1:
            return hash(f.numerator)
        return hash(float(self))

    def __bool__(self):
        return self._special is not None or self._int != 0

    def __float__(self):
        if self._special == "F":
            return float("-inf") if self._sign else float("inf")
        if self._special:
            return float("nan")
        return float(str(self))

    def __int__(self):
        if self._special:
            raise ValueError("cannot convert NaN to integer") if self._special == "n" else OverflowError("cannot convert Infinity to integer")
        if self._exp >= 0:
            v = self._int * 10 ** self._exp
        else:
            v = self._int // 10 ** (-self._exp)
        return -v if self._sign else v

    __trunc__ = __int__

    def __round__(self, n=None):
        if n is None:
            return round(self._frac())
        return self.quantize(Decimal((0, (1,), -n)))

    def __floor__(self):
        return math.floor(self._frac())

    def __ceil__(self):
        return math.ceil(self._frac())

    def quantize(self, exp, rounding=None, context=None):
        e = exp._exp
        f = self._frac() * (10 ** -e) if e <= 0 else self._frac() / (10 ** e)
        n = round(f) if (rounding or getcontext().rounding) == ROUND_HALF_EVEN else int(f + (f > 0) * 0.5 - (f < 0) * 0.5)
        r = Decimal((1 if n < 0 or (n == 0 and self._sign) else 0, tuple(int(c) for c in str(abs(n))), e))
        return r

    def normalize(self, context=None):
        if self._special or self._int == 0:
            return Decimal((self._sign, (0,), 0)) if not self._special else Decimal(self)
        i, e = self._int, self._exp
        while i % 10 == 0:
            i //= 10
            e += 1
        return Decimal((self._sign, tuple(int(c) for c in str(i)), e))

    def sqrt(self, context=None):
        return Decimal(repr(math.sqrt(float(self))))

    def exp(self, context=None):
        return Decimal(repr(math.exp(float(self))))

    def ln(self, context=None):
        return Decimal(repr(math.log(float(self))))

    def log10(self, context=None):
        return Decimal(repr(math.log10(float(self))))

    def adjusted(self):
        return self._exp + len(self._digits()) - 1

    def copy_sign(self, other):
        r = Decimal(self)
        r._sign = other._sign
        return r

    def copy_abs(self):
        return abs(self)

    def copy_negate(self):
        return -self

    def as_integer_ratio(self):
        f = self._frac()
        return (f.numerator, f.denominator)

    def __format__(self, spec):
        if not spec:
            return str(self)
        return format(float(self), spec)

    def __reduce__(self):
        return (Decimal, (str(self),))

    def __copy__(self):
        return self

    def __deepcopy__(self, memo):
        return self
