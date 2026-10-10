# SPDX-License-Identifier: MIT
# BaseExceptionGroup and ExceptionGroup for the Ranger Python runtime, and
# the helpers the interpreter's except* calls. The builtins load this module
# the first time either name is looked up.


def _matcher(cond):
    if isinstance(cond, type) and issubclass(cond, BaseException):
        return lambda e: isinstance(e, cond)
    if isinstance(cond, tuple):
        return lambda e: isinstance(e, cond)
    if callable(cond):
        return cond
    raise TypeError("expected an exception type, a tuple of exception types, or a callable (other than a class)")


class BaseExceptionGroup(BaseException):
    def __new__(cls, message, exceptions):
        if not isinstance(message, str):
            raise TypeError("argument 1 must be str, not " + type(message).__name__)
        given = exceptions
        exceptions = tuple(exceptions) if not isinstance(exceptions, str) else exceptions
        if not isinstance(exceptions, tuple):
            raise TypeError("second argument (exceptions) must be a sequence")
        if not exceptions:
            raise ValueError("second argument (exceptions) must be a non-empty sequence")
        for e in exceptions:
            if not isinstance(e, BaseException):
                raise ValueError("Item %d of second argument (exceptions) is not an exception" % exceptions.index(e))
        if cls is BaseExceptionGroup and all(isinstance(e, Exception) for e in exceptions):
            cls = ExceptionGroup
        if cls is ExceptionGroup:
            for e in exceptions:
                if not isinstance(e, Exception):
                    raise TypeError("Cannot nest BaseExceptions in an ExceptionGroup")
        self = object.__new__(cls)
        self.args = (message, exceptions)
        self._message = message
        self._exceptions = exceptions
        self._given = given
        return self

    def __init__(self, message, exceptions):
        BaseException.__init__(self, message, tuple(exceptions))

    @property
    def message(self):
        return self._message

    @property
    def exceptions(self):
        return self._exceptions

    def __str__(self):
        n = len(self._exceptions)
        return "%s (%d sub-exception%s)" % (self._message, n, "" if n == 1 else "s")

    def __repr__(self):
        return "%s(%r, %r)" % (type(self).__name__, self._message, self._given)

    def derive(self, excs):
        return BaseExceptionGroup(self._message, excs)

    def _copy_meta(self, eg):
        eg.__cause__ = getattr(self, "__cause__", None)
        eg.__context__ = getattr(self, "__context__", None)
        eg.__traceback__ = getattr(self, "__traceback__", None)
        notes = getattr(self, "__notes__", None)
        if notes is not None:
            eg.__notes__ = list(notes)
        return eg

    def split(self, condition):
        match = _matcher(condition)
        if match(self):
            return self, None
        hit, miss = [], []
        for e in self._exceptions:
            if isinstance(e, BaseExceptionGroup):
                a, b = e.split(match)
                if a is not None:
                    hit.append(a)
                if b is not None:
                    miss.append(b)
            elif match(e):
                hit.append(e)
            else:
                miss.append(e)
        m = self._copy_meta(self.derive(tuple(hit))) if hit else None
        r = self._copy_meta(self.derive(tuple(miss))) if miss else None
        return m, r

    def subgroup(self, condition):
        return self.split(condition)[0]


class ExceptionGroup(BaseExceptionGroup, Exception):
    pass


def _as_group(exc):
    # a plain exception caught by except* is handled as a group of one
    if isinstance(exc, BaseExceptionGroup):
        return exc
    return BaseExceptionGroup("", (exc,))


def _finish(original, rest, raised):
    # what propagates once the except* clauses have run: None when all of it
    # was handled, else the original exception, the unhandled part, and what
    # the handlers raised
    if rest is None and not raised:
        return None
    if not isinstance(original, BaseExceptionGroup):
        if not raised:
            return original
        if rest is not None:
            raised = [original] + raised
    elif rest is not None:
        if not raised:
            return rest
        raised = [rest] + raised
    if len(raised) == 1:
        return raised[0]
    return BaseExceptionGroup("", raised)
