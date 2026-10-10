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

# the encodings the runtime implements natively, by normalised name
_NAMES = {"utf8": "utf-8", "utf-8": "utf-8", "u8": "utf-8", "utf": "utf-8", "latin1": "iso8859-1",
          "latin-1": "iso8859-1", "iso-8859-1": "iso8859-1", "iso8859-1": "iso8859-1", "l1": "iso8859-1",
          "ascii": "ascii", "us-ascii": "ascii", "utf-16": "utf-16", "utf16": "utf-16", "utf-32": "utf-32",
          "utf-16-le": "utf-16-le", "utf-16-be": "utf-16-be", "utf-32-le": "utf-32-le", "utf-32-be": "utf-32-be",
          "utf-7": "utf-7", "utf7": "utf-7", "u7": "utf-7",
          "unicode-escape": "unicode-escape", "raw-unicode-escape": "raw-unicode-escape",
          "utf-8-sig": "utf-8-sig"}
_errors = {}
_search_functions = []
_cache = {}


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
        if _is_text_encoding is not None:
            self._is_text_encoding = _is_text_encoding
        return self

    def __repr__(self):
        return "<%s.%s object for encoding %s at %#x>" % (
            self.__class__.__module__, self.__class__.__qualname__, self.name, id(self))


def _native(name):
    return CodecInfo(lambda s, errors="strict": (s.encode(name, errors), len(s)),
                     lambda s, errors="strict": (bytes(s).decode(name, errors), len(s)), name=name)


def _norm(encoding):
    key = encoding.lower().replace("_", "-") if isinstance(encoding, str) else encoding
    name = _NAMES.get(key) or _NAMES.get(encoding.lower())
    if name is None:
        raise LookupError("unknown encoding: %s" % encoding)
    return name


def lookup(encoding):
    # as _PyCodec_Lookup: lower case, spaces become underscores, then the
    # search functions in registration order (the encodings package first)
    if not isinstance(encoding, str):
        raise TypeError("lookup() argument must be str, not %s" % type(encoding).__name__)
    key = encoding.lower().replace(" ", "_")
    info = _cache.get(key)
    if info is not None:
        return info
    name = _NAMES.get(key.replace("_", "-"))
    if name is not None:
        info = _native(name)
    else:
        import encodings
        for search in list(_search_functions):
            info = search(key)
            if info is not None:
                break
        else:
            raise LookupError("unknown encoding: %s" % encoding)
        if not isinstance(info, tuple) or len(info) != 4:
            raise TypeError("codec search functions must return 4-tuples")
    _cache[key] = info
    return info


def encode(obj, encoding="utf-8", errors="strict"):
    r = lookup(encoding)[0](obj, errors)
    if not isinstance(r, tuple) or len(r) != 2:
        raise TypeError("encoder must return a tuple (object, integer)")
    return r[0]


def decode(obj, encoding="utf-8", errors="strict"):
    r = lookup(encoding)[1](obj, errors)
    if not isinstance(r, tuple) or len(r) != 2:
        raise TypeError("decoder must return a tuple (object, integer)")
    return r[0]


def getencoder(encoding):
    return lookup(encoding)[0]


def getdecoder(encoding):
    return lookup(encoding)[1]


def register(search_function):
    if not callable(search_function):
        raise TypeError("argument must be callable")
    _search_functions.append(search_function)


def unregister(search_function):
    if search_function in _search_functions:
        _search_functions.remove(search_function)
        _cache.clear()


def register_error(name, handler):
    _errors[name] = handler


def lookup_error(name):
    if name in _errors:
        return _errors[name]
    handler = _HANDLERS.get(name)
    if handler is not None:
        return handler
    if name in ("namereplace", "surrogateescape", "surrogatepass"):
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
    if isinstance(exc, UnicodeDecodeError):
        return ("\ufffd", exc.end)
    return ("?" * (exc.end - exc.start), exc.end)


def backslashreplace_errors(exc):
    part = exc.object[exc.start:exc.end]
    if isinstance(exc, UnicodeDecodeError):
        return ("".join("\\x%02x" % b for b in part), exc.end)
    out = []
    for c in part:
        n = ord(c)
        out.append("\\x%02x" % n if n < 0x100 else "\\u%04x" % n if n < 0x10000 else "\\U%08x" % n)
    return ("".join(out), exc.end)


def xmlcharrefreplace_errors(exc):
    return ("".join("&#%d;" % ord(c) for c in exc.object[exc.start:exc.end]), exc.end)


_HANDLERS = {"strict": strict_errors, "ignore": ignore_errors, "replace": replace_errors,
             "backslashreplace": backslashreplace_errors, "xmlcharrefreplace": xmlcharrefreplace_errors}


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


class BufferedIncrementalEncoder(IncrementalEncoder):
    # as CPython's: unencoded input is kept between calls
    def __init__(self, errors="strict"):
        IncrementalEncoder.__init__(self, errors)
        self.buffer = ""

    def _buffer_encode(self, input, errors, final):
        raise NotImplementedError

    def encode(self, input, final=False):
        data = self.buffer + input
        (result, consumed) = self._buffer_encode(data, self.errors, final)
        self.buffer = data[consumed:]
        return result

    def reset(self):
        IncrementalEncoder.reset(self)
        self.buffer = ""


class BufferedIncrementalDecoder(IncrementalDecoder):
    # as CPython's: undecoded input is kept between calls
    def __init__(self, errors="strict"):
        IncrementalDecoder.__init__(self, errors)
        self.buffer = b""

    def _buffer_decode(self, input, errors, final):
        raise NotImplementedError

    def decode(self, input, final=False):
        data = self.buffer + input
        (result, consumed) = self._buffer_decode(data, self.errors, final)
        self.buffer = data[consumed:]
        return result

    def reset(self):
        IncrementalDecoder.reset(self)
        self.buffer = b""


def getincrementaldecoder(encoding):
    return lookup(encoding).incrementaldecoder or IncrementalDecoder


def getincrementalencoder(encoding):
    return lookup(encoding).incrementalencoder or IncrementalEncoder


class StreamWriter(Codec):
    def __init__(self, stream, errors="strict"):
        self.stream = stream
        self.errors = errors

    def write(self, object):
        data, consumed = self.encode(object, self.errors)
        self.stream.write(data)

    def writelines(self, list):
        self.write("".join(list))

    def reset(self):
        pass

    def __getattr__(self, name):
        return getattr(self.stream, name)


class StreamReader(Codec):
    def __init__(self, stream, errors="strict"):
        self.stream = stream
        self.errors = errors

    def read(self, size=-1, chars=-1, firstline=False):
        data = self.stream.read() if size < 0 else self.stream.read(size)
        return self.decode(data, self.errors)[0]

    def reset(self):
        pass

    def __getattr__(self, name):
        return getattr(self.stream, name)


# charmap codecs (the encodings package's code pages)

def make_identity_dict(rng):
    return {i: i for i in rng}


def make_encoding_map(decoding_map):
    m = {}
    for k, v in decoding_map.items():
        if v not in m:
            m[v] = k
        else:
            m[v] = None
    return m


def charmap_build(decoding_table):
    return {ord(c): i for i, c in enumerate(decoding_table) if c != "\ufffe"}


def _handle(errors, exc):
    r = lookup_error(errors)(exc)
    if not isinstance(r, tuple) or len(r) != 2:
        raise TypeError("encoding error handler must return (str/bytes, int) tuple")
    return r


def charmap_decode(data, errors="strict", mapping=None):
    data = bytes(data)
    if mapping is None:
        return data.decode("latin-1"), len(data)
    out = []
    i = 0
    while i < len(data):
        b = data[i]
        if isinstance(mapping, str):
            c = mapping[b] if b < len(mapping) else "\ufffe"
        else:
            c = mapping.get(b)
            if isinstance(c, int):
                c = chr(c)
            elif c is None:
                c = "\ufffe"
        if c == "\ufffe":
            exc = UnicodeDecodeError("charmap", data, i, i + 1, "character maps to <undefined>")
            if errors == "strict":
                raise exc
            rep, i = _handle(errors, exc)
            out.append(rep)
            continue
        out.append(c)
        i += 1
    return "".join(out), len(data)


def charmap_encode(text, errors="strict", mapping=None):
    if mapping is None:
        return text.encode("latin-1", errors), len(text)
    out = bytearray()
    i = 0
    while i < len(text):
        b = mapping.get(ord(text[i]))
        if b is None:
            exc = UnicodeEncodeError("charmap", text, i, i + 1, "character maps to <undefined>")
            if errors == "strict":
                raise exc
            if errors == "replace":
                rep, i = "?", i + 1
            else:
                rep, i = _handle(errors, exc)
            for ch in rep:
                if isinstance(rep, bytes):
                    out.append(ch)
                elif mapping.get(ord(ch)) is None:
                    raise exc
                else:
                    out.append(mapping[ord(ch)])
            continue
        out.append(b)
        i += 1
    return bytes(out), len(text)
