# SPDX-License-Identifier: MIT
# io for the Rust runtime: in-memory text and byte streams, and open() as in
# builtins (files live on the runtime's own filesystem).

open = open
SEEK_SET = 0
SEEK_CUR = 1
SEEK_END = 2
DEFAULT_BUFFER_SIZE = 8192


class UnsupportedOperation(OSError, ValueError):
    pass


def text_encoding(encoding, stacklevel=2):
    return "utf-8" if encoding is None else encoding


class IOBase:
    closed = False

    def _check_closed(self):
        if self.closed:
            raise ValueError("I/O operation on closed file.")

    def close(self):
        self.closed = True

    def flush(self):
        self._check_closed()

    def readable(self):
        return False

    def writable(self):
        return False

    def seekable(self):
        return False

    def isatty(self):
        self._check_closed()
        return False

    def fileno(self):
        raise UnsupportedOperation("fileno")

    def __enter__(self):
        self._check_closed()
        return self

    def __exit__(self, *args):
        self.close()
        return False

    def __iter__(self):
        self._check_closed()
        return self

    def __next__(self):
        line = self.readline()
        if not line:
            raise StopIteration
        return line

    def readlines(self, hint=-1):
        out = []
        n = 0
        for line in self:
            out.append(line)
            n += len(line)
            if 0 < hint <= n:
                break
        return out

    def writelines(self, lines):
        for line in lines:
            self.write(line)


class RawIOBase(IOBase):
    pass


class BufferedIOBase(IOBase):
    pass


class TextIOBase(IOBase):
    encoding = None
    errors = None
    newlines = None


class StringIO(TextIOBase):
    def __init__(self, initial_value="", newline="\n"):
        if initial_value is None:
            initial_value = ""
        if not isinstance(initial_value, str):
            raise TypeError("initial_value must be str or None, not %s" % type(initial_value).__name__)
        if newline not in (None, "", "\n", "\r", "\r\n"):
            raise ValueError("illegal newline value: %r" % (newline,))
        self._parts = [initial_value] if initial_value else []
        self._value = None
        self._len = len(initial_value)
        self._pos = 0
        self.closed = False

    def _text(self):
        if self._value is None:
            self._value = "".join(self._parts)
            self._parts = [self._value]
        return self._value

    def write(self, s):
        self._check_closed()
        if not isinstance(s, str):
            raise TypeError("string argument expected, got '%s'" % type(s).__name__)
        if self._pos == self._len:
            self._parts.append(s)
            self._value = None
            self._len += len(s)
            self._pos += len(s)
            return len(s)
        text = self._text()
        if self._pos > len(text):
            text = text + "\0" * (self._pos - len(text))
        self._value = text[:self._pos] + s + text[self._pos + len(s):]
        self._parts = [self._value]
        self._len = len(self._value)
        self._pos += len(s)
        return len(s)

    def getvalue(self):
        self._check_closed()
        return self._text()

    def read(self, size=-1):
        self._check_closed()
        text = self._text()
        if size is None or size < 0:
            out = text[self._pos:]
        else:
            out = text[self._pos:self._pos + size]
        self._pos += len(out)
        return out

    def readline(self, size=-1):
        self._check_closed()
        text = self._text()
        end = text.find("\n", self._pos)
        end = len(text) if end < 0 else end + 1
        if size is not None and size >= 0:
            end = min(end, self._pos + size)
        out = text[self._pos:end]
        self._pos = max(self._pos, end)
        return out

    def seek(self, pos, whence=0):
        self._check_closed()
        if whence == 0:
            if pos < 0:
                raise ValueError("Negative seek position %d" % pos)
            self._pos = pos
        elif whence == 1:
            if pos != 0:
                raise OSError("Can't do nonzero cur-relative seeks")
        elif whence == 2:
            if pos != 0:
                raise OSError("Can't do nonzero end-relative seeks")
            self._pos = len(self._text())
        else:
            raise ValueError("Invalid whence (%d, should be 0, 1 or 2)" % whence)
        return self._pos

    def tell(self):
        self._check_closed()
        return self._pos

    def truncate(self, size=None):
        self._check_closed()
        text = self._text()
        if size is None:
            size = self._pos
        if size < 0:
            raise ValueError("Negative size value %d" % size)
        self._value = text[:size]
        self._parts = [self._value]
        self._len = len(self._value)
        return size

    def readable(self):
        self._check_closed()
        return True

    def writable(self):
        self._check_closed()
        return True

    def seekable(self):
        self._check_closed()
        return True

    @property
    def line_buffering(self):
        return False


class BytesIO(BufferedIOBase):
    def __init__(self, initial_bytes=b""):
        if initial_bytes is None:
            initial_bytes = b""
        self._buf = bytearray(initial_bytes)
        self._pos = 0
        self.closed = False

    def getvalue(self):
        self._check_closed()
        return bytes(self._buf)

    def getbuffer(self):
        self._check_closed()
        return self._buf

    def write(self, b):
        self._check_closed()
        if isinstance(b, str):
            raise TypeError("a bytes-like object is required, not 'str'")
        b = bytes(b)
        n = len(b)
        if n == 0:
            return 0
        pos = self._pos
        if pos > len(self._buf):
            self._buf.extend(b"\0" * (pos - len(self._buf)))
        self._buf[pos:pos + n] = b
        self._pos = pos + n
        return n

    def read(self, size=-1):
        self._check_closed()
        if size is None or size < 0:
            size = len(self._buf)
        if self._pos >= len(self._buf):
            return b""
        out = bytes(self._buf[self._pos:self._pos + size])
        self._pos += len(out)
        return out

    read1 = read

    def readinto(self, b):
        data = self.read(len(b))
        n = len(data)
        b[:n] = data
        return n

    def readline(self, size=-1):
        self._check_closed()
        end = self._buf.find(b"\n", self._pos)
        end = len(self._buf) if end < 0 else end + 1
        if size is not None and size >= 0:
            end = min(end, self._pos + size)
        out = bytes(self._buf[self._pos:end])
        self._pos = max(self._pos, end)
        return out

    def seek(self, pos, whence=0):
        self._check_closed()
        if whence == 0:
            if pos < 0:
                raise ValueError("negative seek value %d" % pos)
            self._pos = pos
        elif whence == 1:
            self._pos = max(0, self._pos + pos)
        elif whence == 2:
            self._pos = max(0, len(self._buf) + pos)
        else:
            raise ValueError("invalid whence (%d, should be 0, 1 or 2)" % whence)
        return self._pos

    def tell(self):
        self._check_closed()
        return self._pos

    def truncate(self, size=None):
        self._check_closed()
        if size is None:
            size = self._pos
        if size < 0:
            raise ValueError("negative size value %d" % size)
        del self._buf[size:]
        return size

    def readable(self):
        self._check_closed()
        return True

    def writable(self):
        self._check_closed()
        return True

    def seekable(self):
        self._check_closed()
        return True


TextIOWrapper = StringIO
BufferedReader = BufferedWriter = BufferedRandom = BufferedRWPair = BytesIO
FileIO = BytesIO
