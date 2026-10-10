# SPDX-License-Identifier: MIT
# codecs for the Ranger Python runtime. Strings are the only text type, so
# encoding and decoding pass text through after checking the name.
BOM_UTF8 = "\xef\xbb\xbf"
BOM_LE = BOM_UTF16_LE = "\xff\xfe"
BOM_BE = BOM_UTF16_BE = "\xfe\xff"
BOM_UTF32_LE = "\xff\xfe\x00\x00"
BOM_UTF32_BE = "\x00\x00\xfe\xff"
BOM = BOM_UTF16 = BOM_LE
BOM_UTF32 = BOM_UTF32_LE

_NAMES = {"utf8": "utf-8", "utf-8": "utf-8", "u8": "utf-8", "utf": "utf-8", "latin1": "iso8859-1",
          "latin-1": "iso8859-1", "iso-8859-1": "iso8859-1", "iso8859-1": "iso8859-1", "l1": "iso8859-1",
          "ascii": "ascii", "us-ascii": "ascii", "utf-16": "utf-16", "utf16": "utf-16", "utf-32": "utf-32",
          "cp1252": "cp1252", "unicode_escape": "unicode-escape", "unicode-escape": "unicode-escape",
          "raw_unicode_escape": "raw-unicode-escape", "utf-8-sig": "utf-8-sig", "utf_8": "utf-8"}
_errors = {}


class CodecInfo(tuple):
    def __new__(cls, encode, decode, streamreader=None, streamwriter=None, incrementalencoder=None,
                incrementaldecoder=None, name=None, *, _is_text_encoding=None):
        self = tuple.__new__(cls, (encode, decode, streamreader, streamwriter))
        self.name = name
        self.encode = encode
        self.decode = decode
        self.incrementalencoder = incrementalencoder
        self.incrementaldecoder = incrementaldecoder
        self.streamreader = streamreader
        self.streamwriter = streamwriter
        return self

    def __repr__(self):
        return "<codecs.CodecInfo object for encoding %s at %#x>" % (self.name, id(self))


def _norm(encoding):
    key = encoding.lower().replace("_", "-") if isinstance(encoding, str) else encoding
    name = _NAMES.get(key) or _NAMES.get(encoding.lower())
    if name is None:
        raise LookupError("unknown encoding: %s" % encoding)
    return name


def lookup(encoding):
    name = _norm(encoding)
    return CodecInfo(lambda s, errors="strict": (encode(s, name, errors), len(s)),
                     lambda s, errors="strict": (decode(s, name, errors), len(s)), name=name)


def encode(obj, encoding="utf-8", errors="strict"):
    _norm(encoding)
    return obj.encode(encoding, errors) if hasattr(obj, "encode") else obj


def decode(obj, encoding="utf-8", errors="strict"):
    _norm(encoding)
    return obj.decode(encoding, errors) if hasattr(obj, "decode") else obj


def getencoder(encoding):
    return lookup(encoding).encode


def getdecoder(encoding):
    return lookup(encoding).decode


def register(search_function):
    pass


def unregister(search_function):
    pass


def register_error(name, handler):
    _errors[name] = handler


def lookup_error(name):
    if name in _errors:
        return _errors[name]
    if name in ("strict", "ignore", "replace", "backslashreplace", "xmlcharrefreplace", "namereplace", "surrogateescape", "surrogatepass"):
        return lambda exc: ("", exc.end)
    raise LookupError("unknown error handler name %r" % name)


def open(filename, mode="r", encoding=None, errors="strict", buffering=-1):
    import builtins
    return builtins.open(filename, mode)


def strict_errors(exc):
    raise exc


def ignore_errors(exc):
    return ("", exc.end)


def replace_errors(exc):
    return ("?", exc.end)


def backslashreplace_errors(exc):
    return ("\\x%02x" % ord(exc.object[exc.start]), exc.end)


class Codec:
    def encode(self, input, errors="strict"):
        raise NotImplementedError

    def decode(self, input, errors="strict"):
        raise NotImplementedError


class IncrementalEncoder:
    def __init__(self, errors="strict"):
        self.errors = errors

    def encode(self, input, final=False):
        return input

    def reset(self):
        pass


class IncrementalDecoder:
    def __init__(self, errors="strict"):
        self.errors = errors

    def decode(self, input, final=False):
        return input

    def reset(self):
        pass


def getincrementaldecoder(encoding):
    _norm(encoding)
    return IncrementalDecoder


def getincrementalencoder(encoding):
    _norm(encoding)
    return IncrementalEncoder
