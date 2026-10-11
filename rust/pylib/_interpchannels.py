# SPDX-License-Identifier: MIT
# _interpchannels (cross-interpreter channels) for the Rust runtime: the
# channels themselves are _rpyinterp's, holding the data of what is sent,
# so they belong to no interpreter.
import collections
import _rpyinterp
import _interpreters


class ChannelError(RuntimeError):
    pass


class ChannelNotFoundError(ChannelError):
    pass


class ChannelClosedError(ChannelError):
    pass


class ChannelEmptyError(ChannelError):
    pass


class ChannelNotEmptyError(ChannelError):
    pass


def _cid(id):
    if isinstance(id, ChannelID):
        return id._cid
    if isinstance(id, bool) or not hasattr(type(id), "__index__"):
        raise TypeError(f"channel ID must be an int, got {id!r}")
    id = id.__index__()
    if id < 0:
        raise ValueError(f"channel ID must be a non-negative int, got {id}")
    return id


def _status(cid, status):
    if status == -1:
        raise ChannelNotFoundError(f"channel {cid} not found")
    if status == -2:
        raise ChannelClosedError(f"channel {cid} is closed")
    if status == -3:
        raise ChannelNotEmptyError(f"channel {cid} may not be closed if not empty (try force=True)")


class ChannelID:
    """A channel ID identifies a channel and may be used as an int."""

    __slots__ = ("_cid", "_end")

    def __new__(cls, id, *, send=None, recv=None, force=False, _resolve=False):
        cid = _cid(id)
        if send and recv or not send and not recv:
            end = "both"
        else:
            end = "send" if send else "recv"
        if not force and _rpyinterp.channel_info(cid)[0] == -1:
            raise ChannelNotFoundError(f"channel {cid} not found")
        self = object.__new__(cls)
        self._cid = cid
        self._end = end
        return self

    @property
    def end(self):
        """'send', 'recv', or 'both'"""
        return self._end

    @property
    def send(self):
        """the 'send' end of the channel"""
        return ChannelID(self._cid, send=True, force=True)

    @property
    def recv(self):
        """the 'recv' end of the channel"""
        return ChannelID(self._cid, recv=True, force=True)

    def __index__(self):
        return self._cid

    __int__ = __index__

    def __repr__(self):
        if self._end == "send":
            return f"{type(self).__name__}({self._cid}, send=True)"
        if self._end == "recv":
            return f"{type(self).__name__}({self._cid}, recv=True)"
        return f"{type(self).__name__}({self._cid})"

    def __str__(self):
        return str(self._cid)

    def __hash__(self):
        return hash(self._cid)

    def __eq__(self, other):
        if isinstance(other, ChannelID):
            return self._cid == other._cid and self._end == other._end
        if isinstance(other, int) and not isinstance(other, bool):
            return self._cid == other
        return NotImplemented


_interpreters._register_xi(ChannelID, lambda c: ("_interpchannels", "_revive_id", (c._cid, c._end)))


def _revive_id(cid, end):
    return ChannelID(cid, send=end != "recv", recv=end != "send", force=True)


ChannelInfo = collections.namedtuple("ChannelInfo", [
    "open", "closing", "closed", "count",
    "num_interp_send", "num_interp_send_released",
    "num_interp_recv", "num_interp_recv_released",
])
ChannelInfo.__module__ = "_interpchannels"
ChannelInfo.status = property(lambda self: "closed" if self.closed else "closing" if self.closing else "open")
for _name in ("send_associated", "send_released", "recv_associated", "recv_released"):
    setattr(ChannelInfo, _name, property(lambda self: False))
del _name


def create(unboundop=-1):
    """Create a new cross-interpreter channel and return its ID."""
    if unboundop == -1:
        unboundop = 3
    elif unboundop not in (1, 2, 3):
        raise ValueError(f"unsupported unboundop {unboundop}")
    return ChannelID(_rpyinterp.channel_create(unboundop), force=True)


def destroy(cid):
    """Close and finalize the channel."""
    cid = _cid(cid)
    if not _rpyinterp.channel_destroy(cid):
        raise ChannelNotFoundError(f"channel {cid} not found")


def list_all():
    """Return (ID, default unboundop) of every open channel."""
    return [(ChannelID(cid, force=True), op) for cid, op in _rpyinterp.channel_list()]


def list_interpreters(cid, *, send):
    """The interpreters associated with one end of the channel."""
    _status(_cid(cid), _rpyinterp.channel_info(_cid(cid))[0])
    return []


def send(cid, obj, unboundop=-1, *, blocking=True, timeout=None):
    """Add the object's data to the channel's queue."""
    cid = _cid(cid)
    if unboundop == -1:
        unboundop = get_channel_defaults(cid)
    _status(cid, _rpyinterp.channel_info(cid)[0])
    if not _interpreters.is_shareable(obj):
        raise _interpreters.NotShareableError(f"{obj!r} does not support cross-interpreter data")
    count = _rpyinterp.channel_info(cid)[3]
    _status(cid, _rpyinterp.channel_send(cid, obj, unboundop, _interpreters._reduce))
    if blocking:
        _wait_received(cid, count, timeout)


def send_buffer(cid, obj, unboundop=-1, *, blocking=True, timeout=None):
    """Add the object's buffer to the channel's queue."""
    send(cid, bytes(memoryview(obj)), unboundop, blocking=blocking, timeout=timeout)


def _wait_received(cid, count, timeout):
    # the item is received once the queue is no longer than it was
    # before it, which takes another thread
    import time
    end = None if timeout is None else time.monotonic() + timeout
    while True:
        status, closing, closed, n, _ = _rpyinterp.channel_info(cid)
        if status != 0 or closed or n <= count:
            return
        if end is not None and time.monotonic() >= end:
            raise TimeoutError
        time.sleep(0.001)


_NOT_SET = object()


def recv(cid, default=_NOT_SET):
    """Return (obj, unboundop) for the channel's next item."""
    cid = _cid(cid)
    status, obj, unboundop = _rpyinterp.channel_recv(cid)
    if status == 1:
        if default is _NOT_SET:
            raise ChannelEmptyError(f"channel {cid} is empty")
        return default, None
    _status(cid, status)
    return obj, unboundop


def close(cid, *, send=None, recv=None, force=False):
    """Close the channel for all interpreters."""
    cid = _cid(cid)
    drain = bool(send) and not recv
    _status(cid, _rpyinterp.channel_close(cid, force, drain))


def release(cid, *, send=None, recv=None, force=False):
    """Close the channel for the current interpreter."""
    cid = _cid(cid)
    _status(cid, _rpyinterp.channel_info(cid)[0])


def get_count(cid):
    """The number of items in the channel."""
    cid = _cid(cid)
    status, _, _, count, _ = _rpyinterp.channel_info(cid)
    _status(cid, status)
    return count


def get_info(cid):
    """A ChannelInfo for the channel."""
    cid = _cid(cid)
    status, closing, closed, count, _ = _rpyinterp.channel_info(cid)
    _status(cid, status)
    return ChannelInfo(not closing and not closed, closing, closed, count, 0, 0, 0, 0)


def get_channel_defaults(cid):
    """The channel's default unboundop."""
    cid = _cid(cid)
    status, _, _, _, unboundop = _rpyinterp.channel_info(cid)
    _status(cid, status)
    return unboundop


def _channel_id(id, *, send=None, recv=None, force=False, _resolve=False):
    return ChannelID(id, send=send, recv=recv, force=force)


# the channel end types the high-level module registered, in this interpreter
_end_types = {}


def _register_end_types(send, recv):
    """Teach the high-level end types to cross between interpreters."""
    _end_types["send"] = send
    _end_types["recv"] = recv
    for end, cls in _end_types.items():
        _interpreters._register_xi(cls, lambda e, end=end: ("_interpchannels", "_revive_end", (int(e.id), end)))


def _revive_end(cid, end):
    # as in CPython, the high-level module is loaded to register them
    if end not in _end_types:
        try:
            import interpreters.channels
        except ImportError:
            import test.support.interpreters.channels
    return _end_types[end](ChannelID(cid, send=end == "send", recv=end == "recv", force=True))
