# SPDX-License-Identifier: MIT
# _random.Random, CPython's C base type of random.Random; the MT19937 core
# is native in _rpyrandom.
import _rpyrandom


class Random:
    """Random() -> create a random number generator with its own internal state."""

    __slots__ = ("_mt",)

    def __new__(cls, *args, **kwargs):
        self = object.__new__(cls)
        self._mt = _rpyrandom.new()
        return self

    def __init__(self, *args, **kwargs):
        if type(self).__init__ is Random.__init__ and kwargs:
            raise TypeError("Random() takes no keyword arguments")
        if len(args) > 1:
            raise TypeError("Random() requires 0 or 1 argument")
        _rpyrandom.seed(self._mt, args[0] if args else None)

    def seed(self, n=None):
        """seed([n]) -> None.

        Defaults to use urandom and falls back to a combination
        of the current time and the process identifier."""
        _rpyrandom.seed(self._mt, n)

    def random(self):
        """random() -> x in the interval [0, 1)."""
        return _rpyrandom.random(self._mt)

    def getrandbits(self, k):
        """getrandbits(k) -> x.  Generates an int with k random bits."""
        return _rpyrandom.getrandbits(self._mt, k)

    def getstate(self):
        """getstate() -> tuple containing the current state."""
        return _rpyrandom.getstate(self._mt)

    def setstate(self, state):
        """setstate(state) -> None.  Restores generator state."""
        _rpyrandom.setstate(self._mt, state)

    def __getstate__(self):
        name = "_random.Random" if type(self) is Random else type(self).__qualname__
        raise TypeError(f"cannot pickle {name!r} object")


Random.__module__ = "_random"
