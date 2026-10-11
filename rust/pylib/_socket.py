# SPDX-License-Identifier: MIT
# _socket for the Rust runtime: the socket type over the host's sockets,
# which _rpysocket works on by their descriptors. A timeout makes the
# descriptor non-blocking and each operation wait for it, as CPython's do.
import _rpysocket as _s
from _rpysocket import *
import errno as _errno
import os as _os

error = OSError
timeout = TimeoutError

_default_timeout = None


def getdefaulttimeout():
    return _default_timeout


def _timeout_of(value):
    if value is None:
        return None
    try:
        t = float(value)
    except TypeError:
        raise TypeError("Timeout value must be int, float or None") from None
    if t != t:
        raise ValueError("Invalid value NaN (not a number)")
    if t < 0.0:
        raise ValueError("Timeout value out of range")
    return t


def setdefaulttimeout(timeout):
    global _default_timeout
    _default_timeout = _timeout_of(timeout)


def _int(value, what):
    if not isinstance(value, int):
        raise TypeError("'%s' object cannot be interpreted as an integer"
                        % type(value).__name__)
    return int(value)


class socket:
    """socket(family=AF_INET, type=SOCK_STREAM, proto=0) -> socket object
socket(family=-1, type=-1, proto=-1, fileno=None) -> socket object

Open a socket of the given type."""

    # a C type's instances, without a __dict__ or weak references of their own
    __slots__ = ("_fd", "_family", "_type", "_proto", "_timeout")

    def __init__(self, family=-1, type=-1, proto=-1, fileno=None):
        family, type, proto = int(family), int(type), int(proto)
        if fileno is None:
            if family == -1:
                family = AF_INET
            if type == -1:
                type = SOCK_STREAM
            if proto == -1:
                proto = 0
            fd = _s._socket(family, type, proto)
        else:
            if isinstance(fileno, float):
                raise TypeError("'float' object cannot be interpreted as an integer")
            fd = _int(fileno, "fileno")
            if fd < 0:
                raise ValueError("negative file descriptor")
            f, t, p = _s._info(fd)
            if family == -1:
                family = f
            if type == -1:
                type = t
            if proto == -1:
                proto = p
        self._fd = fd
        self._family = family
        self._type = type
        self._proto = proto
        self._timeout = None
        self.settimeout(_default_timeout)

    @property
    def family(self):
        return self._family

    @property
    def type(self):
        return self._type

    @property
    def proto(self):
        return self._proto

    @property
    def timeout(self):
        return self._timeout

    def __repr__(self):
        return "<socket object, fd=%d, family=%d, type=%d, proto=%d>" % (
            self._fd, self._family, self._type, self._proto)

    def __del__(self):
        if getattr(self, "_fd", -1) >= 0:
            import warnings
            warnings.warn("unclosed %r" % (self,), ResourceWarning, source=self)
            self.close()

    def fileno(self):
        return self._fd

    def detach(self):
        fd, self._fd = self._fd, -1
        return fd

    def close(self):
        fd, self._fd = self._fd, -1
        if fd >= 0:
            try:
                _os.close(fd)
            except OSError as e:
                # a connection the peer reset is closed all the same
                if e.errno != _errno.ECONNRESET:
                    raise

    def _check(self):
        if self._fd < 0:
            raise OSError(9, "Bad file descriptor")

    def settimeout(self, value):
        self._timeout = _timeout_of(value)
        if self._fd >= 0:
            _os.set_blocking(self._fd, self._timeout is None)

    def gettimeout(self):
        return self._timeout

    def setblocking(self, flag):
        self.settimeout(None if flag else 0.0)

    def getblocking(self):
        return self._timeout != 0.0

    def bind(self, address):
        self._check()
        _s._bind(self._fd, self._family, address)

    def listen(self, backlog=128):
        self._check()
        _s._listen(self._fd, backlog)

    def connect(self, address):
        self._check()
        err = _s._connect(self._fd, self._family, address, self._timeout)
        if err:
            raise OSError(err, _os.strerror(err))

    def connect_ex(self, address):
        self._check()
        try:
            return _s._connect(self._fd, self._family, address, self._timeout)
        except TimeoutError:
            return _errno.EAGAIN

    def _accept(self):
        self._check()
        return _s._accept(self._fd, self._timeout)

    def getsockname(self):
        self._check()
        return _s._getsockname(self._fd)

    def getpeername(self):
        self._check()
        return _s._getpeername(self._fd)

    def send(self, data, flags=0):
        self._check()
        return _s._send(self._fd, data, flags, self._timeout)

    def sendall(self, data, flags=0):
        self._check()
        _s._sendall(self._fd, data, flags, self._timeout)

    def sendto(self, data, flags_or_address, address=None):
        self._check()
        if address is None:
            flags, address = 0, flags_or_address
        else:
            flags = flags_or_address
        return _s._sendto(self._fd, data, flags, self._family, address, self._timeout)

    def recv(self, bufsize, flags=0):
        self._check()
        return _s._recv(self._fd, bufsize, flags, self._timeout)

    def recvfrom(self, bufsize, flags=0):
        self._check()
        return _s._recvfrom(self._fd, bufsize, flags, self._timeout)

    def recv_into(self, buffer, nbytes=0, flags=0):
        if nbytes < 0:
            raise ValueError("negative buffersize in recv_into")
        view = memoryview(buffer).cast("B")
        data = self.recv(nbytes or len(view), flags)
        view[:len(data)] = data
        return len(data)

    def recvfrom_into(self, buffer, nbytes=0, flags=0):
        if nbytes < 0:
            raise ValueError("negative buffersize in recvfrom_into")
        view = memoryview(buffer).cast("B")
        data, address = self.recvfrom(nbytes or len(view), flags)
        view[:len(data)] = data
        return len(data), address

    def shutdown(self, how):
        self._check()
        _s._shutdown(self._fd, how)

    def setsockopt(self, level, optname, value, optlen=None):
        self._check()
        if value is None:
            value = b"\0" * optlen
        _s._setsockopt(self._fd, level, optname, value)

    def getsockopt(self, level, optname, buflen=None):
        self._check()
        return _s._getsockopt(self._fd, level, optname, buflen)


SocketType = socket


def socketpair(family=AF_UNIX, type=SOCK_STREAM, proto=0):
    a, b = _s._socketpair(int(family), int(type), int(proto))
    return socket(family, type, proto, a), socket(family, type, proto, b)


def dup(fd):
    return _os.dup(fd)


def close(fd):
    _os.close(fd)


def _uint(x, bits, name):
    if not isinstance(x, int):
        raise TypeError("'%s' object cannot be interpreted as an integer" % type(x).__name__)
    if x < 0:
        raise OverflowError("%s: can't convert negative Python int to C %d-bit unsigned integer"
                            % (name, bits))
    if x >= 1 << bits:
        raise OverflowError("%s: Python int too large to convert to C %d-bit unsigned integer"
                            % (name, bits))
    return x


def htons(x):
    x = _uint(x, 16, "htons")
    return ((x & 0xff) << 8) | (x >> 8)


def ntohs(x):
    x = _uint(x, 16, "ntohs")
    return ((x & 0xff) << 8) | (x >> 8)


def htonl(x):
    x = _uint(x, 32, "htonl")
    return int.from_bytes(x.to_bytes(4, "big"), "little")


def ntohl(x):
    x = _uint(x, 32, "ntohl")
    return int.from_bytes(x.to_bytes(4, "big"), "little")

