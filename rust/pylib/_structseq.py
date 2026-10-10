# SPDX-License-Identifier: MIT
# Named tuples like CPython's struct sequences (sys.version_info,
# sys.float_info...). The Rust runtime builds sys's with make().


def make(name, fields, values, module="sys", n_sequence=None):
    n = len(fields) if n_sequence is None else n_sequence

    def __new__(cls, seq=(), dict=None):
        seq = tuple(seq)
        if not (n <= len(seq) <= len(fields)):
            raise TypeError(f"{module}.{name}() takes a {n}-sequence ({len(seq)}-sequence given)")
        self = tuple.__new__(cls, seq[:n])
        self._extra = seq[n:]
        return self

    def __repr__(self):
        return f"{module}.{name}(" + ", ".join(f"{f}={getattr(self, f)!r}" for f in fields) + ")"

    def __reduce__(self):
        return (type(self), (tuple(self),))

    ns = {"__new__": __new__, "__repr__": __repr__, "__reduce__": __reduce__, "__slots__": ("_extra",),
          "n_sequence_fields": n, "n_fields": len(fields), "n_unnamed_fields": 0, "_fields": tuple(fields),
          "__match_args__": tuple(fields[:n]), "__module__": module}
    for i, f in enumerate(fields):
        if i < n:
            ns[f] = property(lambda self, i=i: tuple.__getitem__(self, i))
        else:
            ns[f] = property(lambda self, i=i: self._extra[i - n] if i - n < len(self._extra) else None)
    cls = type(name, (tuple,), ns)
    cls.__qualname__ = name
    return cls(values)
