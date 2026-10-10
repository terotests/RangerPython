# SPDX-License-Identifier: MIT
# traceback for the Ranger Python runtime: frames from __traceback__,
# source lines through linecache, and CPython's chaining messages.
import sys
import linecache

__all__ = ["extract_stack", "extract_tb", "format_exception",
           "format_exception_only", "format_list", "format_stack",
           "format_tb", "print_exc", "format_exc", "print_exception",
           "print_last", "print_stack", "print_tb", "clear_frames",
           "FrameSummary", "StackSummary", "TracebackException",
           "walk_stack", "walk_tb"]

_cause_message = ("\nThe above exception was the direct cause "
                  "of the following exception:\n\n")
_context_message = ("\nDuring handling of the above exception, "
                    "another exception occurred:\n\n")


class FrameSummary:
    __slots__ = ("filename", "lineno", "end_lineno", "colno", "end_colno",
                 "name", "_line", "locals")

    def __init__(self, filename, lineno, name, *, lookup_line=True,
                 locals=None, line=None, end_lineno=None, colno=None,
                 end_colno=None):
        self.filename = filename
        self.lineno = lineno
        self.end_lineno = lineno if end_lineno is None else end_lineno
        self.colno = colno
        self.end_colno = end_colno
        self.name = name
        self._line = line
        self.locals = {k: repr(v) for k, v in locals.items()} if locals else None
        if lookup_line:
            self.line

    def __eq__(self, other):
        if isinstance(other, FrameSummary):
            return (self.filename == other.filename and self.lineno == other.lineno
                    and self.name == other.name and self.locals == other.locals)
        if isinstance(other, tuple):
            return (self.filename, self.lineno, self.name, self.line) == other
        return NotImplemented

    def __getitem__(self, pos):
        return (self.filename, self.lineno, self.name, self.line)[pos]

    def __iter__(self):
        return iter([self.filename, self.lineno, self.name, self.line])

    def __len__(self):
        return 4

    def __repr__(self):
        return "<FrameSummary file {filename}, line {lineno} in {name}>".format(
            filename=self.filename, lineno=self.lineno, name=self.name)

    @property
    def line(self):
        if self._line is None:
            if self.lineno is None:
                return None
            self._line = linecache.getline(self.filename, self.lineno)
        return self._line.strip()


def walk_tb(tb):
    while tb is not None:
        yield tb.tb_frame, tb.tb_lineno
        tb = tb.tb_next


def walk_stack(f):
    if f is None:
        f = sys._getframe().f_back.f_back
    while f is not None:
        yield f, f.f_lineno
        f = getattr(f, "f_back", None)


def _repeated(n):
    return "  [Previous line repeated %d more time%s]\n" % (n, "s" if n > 1 else "")


class StackSummary(list):
    @classmethod
    def extract(klass, frame_gen, *, limit=None, lookup_lines=True,
                capture_locals=False):
        result = klass()
        for f, lineno in frame_gen:
            if limit is not None and len(result) >= limit:
                break
            co = f.f_code
            result.append(FrameSummary(co.co_filename, lineno, co.co_name,
                                       lookup_line=lookup_lines,
                                       locals=f.f_locals if capture_locals else None))
        return result

    @classmethod
    def from_list(klass, a_list):
        result = klass()
        for frame in a_list:
            if isinstance(frame, FrameSummary):
                result.append(frame)
            else:
                filename, lineno, name, line = frame
                result.append(FrameSummary(filename, lineno, name, line=line))
        return result

    def format_frame_summary(self, frame_summary):
        row = ['  File "{}", line {}, in {}\n'.format(
            frame_summary.filename, frame_summary.lineno, frame_summary.name)]
        if frame_summary.line:
            row.append("    {}\n".format(frame_summary.line))
        if frame_summary.locals:
            for name, value in sorted(frame_summary.locals.items()):
                row.append("    {name} = {value}\n".format(name=name, value=value))
        return "".join(row)

    def format(self):
        result = []
        last = None
        count = 0
        for fs in self:
            key = (fs.filename, fs.lineno, fs.name)
            if last is None or key != last:
                if count > 3:
                    result.append(_repeated(count - 3))
                last = key
                count = 0
            count += 1
            if count > 3:
                continue
            result.append(self.format_frame_summary(fs))
        if count > 3:
            result.append(_repeated(count - 3))
        return result


def extract_tb(tb, limit=None):
    return StackSummary.extract(walk_tb(tb), limit=limit)


def extract_stack(f=None, limit=None):
    try:
        stack = StackSummary.extract(walk_stack(f), limit=limit)
    except Exception:
        return StackSummary()
    stack.reverse()
    return stack


def format_list(extracted_list):
    return StackSummary.from_list(extracted_list).format()


def format_tb(tb, limit=None):
    return extract_tb(tb, limit=limit).format()


def print_list(extracted_list, file=None):
    if file is None:
        file = sys.stderr
    for item in StackSummary.from_list(extracted_list).format():
        print(item, file=file, end="")


def print_tb(tb, limit=None, file=None):
    print_list(extract_tb(tb, limit=limit), file=file)


def format_stack(f=None, limit=None):
    return format_list(extract_stack(f, limit=limit))


def print_stack(f=None, limit=None, file=None):
    print_list(extract_stack(f, limit=limit), file=file)


def _safe_string(value, what, func=str):
    try:
        return func(value)
    except BaseException:
        return "<%s %s() failed>" % (what, func.__name__)


def _parse_value_tb(exc, value, tb):
    if value is None and tb is None:
        if exc is not None:
            if isinstance(exc, BaseException):
                return exc, exc.__traceback__
            raise TypeError(f"Exception expected for value, "
                            f"{type(exc).__name__} found")
        return None, None
    return value, tb


class TracebackException:
    def __init__(self, exc_type, exc_value, exc_traceback, *, limit=None,
                 lookup_lines=True, capture_locals=False, compact=False,
                 max_group_width=15, max_group_depth=10, _seen=None):
        if _seen is None:
            _seen = set()
        _seen.add(id(exc_value))
        self.stack = StackSummary.extract(walk_tb(exc_traceback), limit=limit,
                                          lookup_lines=lookup_lines,
                                          capture_locals=capture_locals)
        self.exc_type = exc_type
        self._str = _safe_string(exc_value, "exception")
        try:
            self.__notes__ = getattr(exc_value, "__notes__", None)
        except Exception as e:
            self.__notes__ = [f"Ignored error getting __notes__: {_safe_string(e, '__notes__', repr)}"]
        if exc_type is not None and issubclass(exc_type, SyntaxError):
            self.filename = exc_value.filename
            lno = exc_value.lineno
            self.lineno = str(lno) if lno is not None else None
            self.text = exc_value.text
            self.offset = exc_value.offset
            self.end_offset = exc_value.end_offset
            self.msg = exc_value.msg
        self.__cause__ = None
        self.__context__ = None
        self.__suppress_context__ = exc_value.__suppress_context__ if exc_value is not None else False
        if exc_value is not None:
            c = exc_value.__cause__
            if c is not None and id(c) not in _seen:
                self.__cause__ = TracebackException(type(c), c, c.__traceback__,
                                                    limit=limit, lookup_lines=lookup_lines,
                                                    capture_locals=capture_locals, _seen=_seen)
            c = exc_value.__context__
            if c is not None and id(c) not in _seen:
                self.__context__ = TracebackException(type(c), c, c.__traceback__,
                                                      limit=limit, lookup_lines=lookup_lines,
                                                      capture_locals=capture_locals, _seen=_seen)

    @classmethod
    def from_exception(cls, exc, *args, **kwargs):
        return cls(type(exc), exc, exc.__traceback__, *args, **kwargs)

    @property
    def exc_type_str(self):
        t = self.exc_type
        if t is None:
            return None
        stype = t.__qualname__
        smod = t.__module__
        if smod not in ("__main__", "builtins"):
            if not isinstance(smod, str):
                smod = "<unknown>"
            stype = smod + "." + stype
        return stype

    def __eq__(self, other):
        if isinstance(other, TracebackException):
            return self.__dict__ == other.__dict__
        return NotImplemented

    def __str__(self):
        return self._str

    def format_exception_only(self, *, show_group=False, _depth=0):
        if self.exc_type is None:
            yield "NoneType: None\n"
            return
        stype = self.exc_type_str
        if not issubclass(self.exc_type, SyntaxError):
            if self._str:
                yield "%s: %s\n" % (stype, self._str)
            else:
                yield "%s\n" % stype
        else:
            yield from self._format_syntax_error(stype)
        if isinstance(self.__notes__, (list, tuple)):
            for note in self.__notes__:
                note = _safe_string(note, "note")
                yield from [l + "\n" for l in note.split("\n")]
        elif self.__notes__ is not None:
            yield "%s\n" % _safe_string(self.__notes__, "__notes__", func=repr)

    def _format_syntax_error(self, stype):
        filename_suffix = ""
        if self.lineno is not None:
            yield '  File "{}", line {}\n'.format(self.filename or "<string>", self.lineno)
        elif self.filename is not None:
            filename_suffix = " ({})".format(self.filename)
        text = self.text
        if text is not None:
            rtext = text.rstrip("\n")
            ltext = rtext.lstrip(" \n\f")
            spaces = len(rtext) - len(ltext)
            yield "    {}\n".format(ltext)
            if self.offset is not None:
                offset = self.offset
                end_offset = self.end_offset if self.end_offset not in {None, 0} else offset
                if self.text and offset > len(self.text):
                    offset = len(self.text) + 1
                if self.text and end_offset > len(self.text):
                    end_offset = len(self.text) + 1
                if offset >= end_offset or end_offset < 0:
                    end_offset = offset + 1
                colno = offset - 1 - spaces
                end_colno = end_offset - 1 - spaces
                if colno >= 0:
                    caretspace = "".join(c if c.isspace() else " " for c in ltext[:colno])
                    yield "    {}{}\n".format(caretspace, "^" * (end_colno - colno))
        msg = self.msg or "<no detail available>"
        yield "{}: {}{}\n".format(stype, msg, filename_suffix)

    def format(self, *, chain=True, _ctx=None):
        output = []
        exc = self
        if chain:
            while exc:
                if exc.__cause__ is not None:
                    chained_msg = _cause_message
                    chained_exc = exc.__cause__
                elif exc.__context__ is not None and not exc.__suppress_context__:
                    chained_msg = _context_message
                    chained_exc = exc.__context__
                else:
                    chained_msg = None
                    chained_exc = None
                output.append((chained_msg, exc))
                exc = chained_exc
        else:
            output.append((None, exc))
        for msg, exc in reversed(output):
            if msg is not None:
                yield msg
            if exc.stack:
                yield "Traceback (most recent call last):\n"
                yield from exc.stack.format()
            yield from exc.format_exception_only()

    def print(self, *, file=None, chain=True):
        if file is None:
            file = sys.stderr
        for line in self.format(chain=chain):
            print(line, file=file, end="")


def format_exception(exc, /, value=None, tb=None, limit=None, chain=True):
    value, tb = _parse_value_tb(exc, value, tb)
    te = TracebackException(type(value), value, tb, limit=limit)
    return list(te.format(chain=chain))


def format_exception_only(exc, /, value=None, *, show_group=False):
    if value is None:
        value = exc
    te = TracebackException(type(value), value, None)
    return list(te.format_exception_only(show_group=show_group))


def print_exception(exc, /, value=None, tb=None, limit=None, file=None, chain=True):
    value, tb = _parse_value_tb(exc, value, tb)
    te = TracebackException(type(value), value, tb, limit=limit)
    te.print(file=file, chain=chain)


def format_exc(limit=None, chain=True):
    return "".join(format_exception(sys.exception(), limit=limit, chain=chain))


def print_exc(limit=None, file=None, chain=True):
    print_exception(sys.exception(), limit=limit, file=file, chain=chain)


def print_last(limit=None, file=None, chain=True):
    if not hasattr(sys, "last_exc") and not hasattr(sys, "last_type"):
        raise ValueError("no last exception")
    if hasattr(sys, "last_exc"):
        print_exception(sys.last_exc, limit=limit, file=file, chain=chain)
    else:
        print_exception(sys.last_type, sys.last_value, sys.last_traceback,
                        limit=limit, file=file, chain=chain)


def clear_frames(tb):
    pass
