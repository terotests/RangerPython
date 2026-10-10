# SPDX-License-Identifier: MIT
# io for the Ranger Python runtime: StringIO, and open as in builtins.

open = open
SEEK_SET = 0
SEEK_CUR = 1
SEEK_END = 2


class UnsupportedOperation(OSError, ValueError):
    pass


class StringIO:
    def __init__(self, initial_value="", newline="\n"):
        self._parts = [initial_value] if initial_value else []
        self._value = None
        self._pos = 0
        self.closed = False

    def _text(self):
        if self._value is None:
            self._value = "".join(self._parts)
            self._parts = [self._value]
        return self._value

    def write(self, s):
        if self.closed:
            raise ValueError("I/O operation on closed file.")
        if not isinstance(s, str):
            raise TypeError("string argument expected, got '%s'" % type(s).__name__)
        text = self._text()
        if self._pos == len(text):
            self._parts.append(s)
            self._value = None
        else:
            self._value = text[:self._pos] + s + text[self._pos + len(s):]
            self._parts = [self._value]
        self._pos += len(s)
        return len(s)

    def writelines(self, lines):
        for line in lines:
            self.write(line)

    def getvalue(self):
        return self._text()

    def read(self, size=-1):
        text = self._text()
        if size is None or size < 0:
            out = text[self._pos:]
        else:
            out = text[self._pos:self._pos + size]
        self._pos += len(out)
        return out

    def readline(self, size=-1):
        text = self._text()
        end = text.find("\n", self._pos)
        end = len(text) if end < 0 else end + 1
        out = text[self._pos:end]
        self._pos = end
        return out

    def readlines(self):
        out = []
        while True:
            line = self.readline()
            if not line:
                return out
            out.append(line)

    def __iter__(self):
        return iter(self.readlines())

    def seek(self, pos, whence=0):
        if whence == 0:
            self._pos = pos
        elif whence == 2:
            self._pos = len(self._text()) + pos
        else:
            self._pos += pos
        return self._pos

    def tell(self):
        return self._pos

    def truncate(self, size=None):
        text = self._text()
        if size is None:
            size = self._pos
        self._value = text[:size]
        self._parts = [self._value]
        return size

    def flush(self):
        pass

    def close(self):
        self.closed = True

    def readable(self):
        return True

    def writable(self):
        return True

    def seekable(self):
        return True

    def __enter__(self):
        return self

    def __exit__(self, *args):
        self.close()
        return False


TextIOWrapper = StringIO
