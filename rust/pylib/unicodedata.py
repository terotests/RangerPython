# SPDX-License-Identifier: MIT
# unicodedata for the Rust runtime. Names come from the runtime's table; the
# categories are worked out from the str predicates, so the rarer ones
# (Lt, Lm, Mn, Mc, Pc...) are approximations.
from _unicodedata import name, lookup

unidata_version = "15.1.0"

_ASCII_PUNCT = {
    "!": "Po", '"': "Po", "#": "Po", "%": "Po", "&": "Po", "'": "Po", "*": "Po", ",": "Po",
    ".": "Po", "/": "Po", ":": "Po", ";": "Po", "?": "Po", "@": "Po", "\\": "Po",
    "(": "Ps", "[": "Ps", "{": "Ps", ")": "Pe", "]": "Pe", "}": "Pe",
    "-": "Pd", "_": "Pc", "$": "Sc", "+": "Sm", "<": "Sm", "=": "Sm", ">": "Sm",
    "|": "Sm", "~": "Sm", "^": "Sk", "`": "Sk",
}


def _check(chr, fn):
    if not isinstance(chr, str) or len(chr) != 1:
        raise TypeError(f"{fn}() argument must be a unicode character, not {type(chr).__name__}")


def category(chr):
    _check(chr, "category")
    o = ord(chr)
    if o < 0x20 or 0x7f <= o < 0xa0:
        return "Cc"
    if chr in _ASCII_PUNCT:
        return _ASCII_PUNCT[chr]
    if chr == " " or chr == "\xa0" or chr == "　" or 0x2000 <= o <= 0x200a or chr in "   ":
        return "Zs"
    if chr == " ":
        return "Zl"
    if chr == " ":
        return "Zp"
    if 0xd800 <= o < 0xe000:
        return "Cs"
    if 0xe000 <= o < 0xf900 or o >= 0xf0000:
        return "Co"
    if chr.isupper():
        return "Lu"
    if chr.islower():
        return "Ll"
    if chr.istitle():
        return "Lt"
    if chr.isalpha():
        return "Lo"
    if chr.isdecimal():
        return "Nd"
    if chr.isdigit():
        return "No"
    if chr.isnumeric():
        return "Nl" if 0x2160 <= o <= 0x2188 else "No"
    if 0x300 <= o < 0x370 or 0x1ab0 <= o < 0x1b00 or 0x1dc0 <= o < 0x1e00 or 0x20d0 <= o < 0x2100 or 0xfe20 <= o < 0xfe30:
        return "Mn"
    if o in (0x200b, 0x200c, 0x200d, 0x200e, 0x200f, 0xfeff) or 0x202a <= o <= 0x202e or 0x2060 <= o <= 0x206f:
        return "Cf"
    if chr.isprintable():
        if 0x2010 <= o <= 0x2015:
            return "Pd"
        if 0x2016 <= o <= 0x2027 or 0x2030 <= o <= 0x205e:
            return "Po"
        if 0x20a0 <= o <= 0x20cf:
            return "Sc"
        if 0x2190 <= o <= 0x22ff:
            return "Sm"
        return "So"
    return "Cn"


def bidirectional(chr):
    _check(chr, "bidirectional")
    o = ord(chr)
    if chr in "\n\r\x1c\x1d\x1e\x85 ":
        return "B"
    if chr in "\t\x0b\x1f":
        return "S"
    if chr in " \x0c" or category(chr) == "Zs" or chr == " ":
        return "WS"
    if "0" <= chr <= "9":
        return "EN"
    if chr in "+-":
        return "ES"
    if chr in "#$%\xa2\xa3\xa4\xa5\xb0\xb1":
        return "ET"
    if chr in ",.:/\xa0":
        return "CS"
    if 0x590 <= o < 0x600 or 0x7c0 <= o < 0x860 or 0xfb1d <= o < 0xfb50:
        return "R"
    if 0x600 <= o < 0x7c0 or 0x860 <= o < 0x8ff or 0xfb50 <= o < 0xfe00 or 0xfe70 <= o < 0xff00:
        return "AL"
    cat = category(chr)
    if cat in ("Mn", "Me"):
        return "NSM"
    if cat == "Cc" or cat == "Cf":
        return "BN"
    if cat[0] in "LN" or cat == "Co":
        return "L"
    if cat == "Cn":
        return ""
    return "ON"


def combining(chr):
    _check(chr, "combining")
    return 230 if category(chr) == "Mn" else 0


def mirrored(chr):
    _check(chr, "mirrored")
    return 1 if chr in "()<>[]{}\xab\xbb" else 0


def east_asian_width(chr):
    _check(chr, "east_asian_width")
    o = ord(chr)
    if 0x1100 <= o <= 0x115f or 0x2e80 <= o <= 0xa4cf or 0xac00 <= o <= 0xd7a3 or 0xf900 <= o <= 0xfaff or 0xfe30 <= o <= 0xfe4f or 0xff00 <= o <= 0xff60 or 0xffe0 <= o <= 0xffe6 or 0x1f300 <= o <= 0x1faff or 0x20000 <= o <= 0x3fffd:
        return "F" if 0xff00 <= o <= 0xff60 or 0xffe0 <= o <= 0xffe6 else "W"
    if 0xff61 <= o <= 0xffdc:
        return "H"
    if o < 0x7f and chr.isprintable():
        return "Na"
    return "N"


def decimal(chr, default=None):
    _check(chr, "decimal")
    if chr.isdecimal():
        return int(chr)
    if default is None:
        raise ValueError("not a decimal")
    return default


def digit(chr, default=None):
    _check(chr, "digit")
    if chr.isdigit():
        try:
            return int(chr)
        except ValueError:
            pass
        n = name(chr, "")
        for i, w in enumerate(("ZERO", "ONE", "TWO", "THREE", "FOUR", "FIVE", "SIX", "SEVEN", "EIGHT", "NINE")):
            if n.endswith(" " + w):
                return i
    if default is None:
        raise ValueError("not a digit")
    return default


_FRACTIONS = {"\xbc": 0.25, "\xbd": 0.5, "\xbe": 0.75, "⅓": 1 / 3, "⅔": 2 / 3, "⅛": 0.125}


def numeric(chr, default=None):
    _check(chr, "numeric")
    if chr in _FRACTIONS:
        return _FRACTIONS[chr]
    if chr.isnumeric():
        try:
            return float(digit(chr))
        except ValueError:
            pass
        if 0x2160 <= ord(chr) <= 0x216b:
            return float(ord(chr) - 0x215f)
    if default is None:
        raise ValueError("not a numeric character")
    return default


def decomposition(chr):
    _check(chr, "decomposition")
    return ""


def normalize(form, unistr):
    if form not in ("NFC", "NFD", "NFKC", "NFKD"):
        raise ValueError("invalid normalization form")
    if not isinstance(unistr, str):
        raise TypeError(f"normalize() argument 2 must be str, not {type(unistr).__name__}")
    return unistr


def is_normalized(form, unistr):
    if form not in ("NFC", "NFD", "NFKC", "NFKD"):
        raise ValueError("invalid normalization form")
    return True


class UCD:
    unidata_version = "3.2.0"

    def __getattr__(self, name):
        return globals()[name]


ucd_3_2_0 = UCD()
