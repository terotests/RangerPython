# SPDX-License-Identifier: MIT
# csv for the Ranger Python runtime: reader, writer, DictReader, DictWriter
# and the dialects, in Python.

__all__ = ["QUOTE_MINIMAL", "QUOTE_ALL", "QUOTE_NONNUMERIC", "QUOTE_NONE", "QUOTE_STRINGS", "QUOTE_NOTNULL",
           "Error", "Dialect", "excel", "excel_tab", "unix_dialect", "reader", "writer", "DictReader",
           "DictWriter", "register_dialect", "get_dialect", "list_dialects", "unregister_dialect",
           "field_size_limit", "Sniffer", "__version__"]

__version__ = "1.0"

QUOTE_MINIMAL, QUOTE_ALL, QUOTE_NONNUMERIC, QUOTE_NONE, QUOTE_STRINGS, QUOTE_NOTNULL = range(6)


class Error(Exception):
    pass


class Dialect:
    delimiter = ","
    quotechar = '"'
    escapechar = None
    doublequote = True
    skipinitialspace = False
    lineterminator = "\r\n"
    quoting = QUOTE_MINIMAL
    strict = False


class excel(Dialect):
    pass


class excel_tab(excel):
    delimiter = "\t"


class unix_dialect(Dialect):
    lineterminator = "\n"
    quoting = QUOTE_ALL


_dialects = {"excel": excel, "excel-tab": excel_tab, "unix": unix_dialect}
_limit = 131072


def register_dialect(name, dialect=None, **fmtparams):
    _dialects[name] = _Params(dialect or excel, fmtparams)


def unregister_dialect(name):
    if name not in _dialects:
        raise Error("unknown dialect")
    del _dialects[name]


def get_dialect(name):
    if name not in _dialects:
        raise Error("unknown dialect")
    return _dialects[name]


def list_dialects():
    return list(_dialects)


def field_size_limit(new_limit=None):
    global _limit
    old = _limit
    if new_limit is not None:
        _limit = new_limit
    return old


class _Params:
    def __init__(self, dialect, fmtparams):
        if isinstance(dialect, str):
            dialect = get_dialect(dialect)
        for name in ("delimiter", "quotechar", "escapechar", "doublequote", "skipinitialspace",
                     "lineterminator", "quoting", "strict"):
            setattr(self, name, fmtparams.get(name, getattr(dialect, name, getattr(Dialect, name))))
        if not isinstance(self.delimiter, str) or len(self.delimiter) != 1:
            raise TypeError('"delimiter" must be a 1-character string')
        if self.quotechar is None and self.quoting != QUOTE_NONE:
            raise TypeError("quotechar must be set if quoting enabled")


class _Reader:
    def __init__(self, f, dialect, fmtparams):
        self._it = iter(f)
        self.dialect = _Params(dialect, fmtparams)
        self.line_num = 0

    def __iter__(self):
        return self

    def __next__(self):
        d = self.dialect
        row = []
        field = []
        state = 0  # 0 start of field, 1 in field, 2 in quotes, 3 quote in quotes
        quoted = False
        while True:
            line = next(self._it)
            self.line_num += 1
            i = 0
            n = len(line)
            while i < n:
                c = line[i]
                if state == 2:
                    if d.escapechar is not None and c == d.escapechar and i + 1 < n:
                        field.append(line[i + 1])
                        i += 2
                        continue
                    if c == d.quotechar:
                        state = 3 if d.doublequote else 1
                    else:
                        field.append(c)
                elif state == 3:
                    if c == d.quotechar:
                        field.append(c)
                        state = 2
                    elif c == d.delimiter:
                        row.append(self._value("".join(field), True))
                        field = []
                        quoted = False
                        state = 0
                    elif c in "\r\n":
                        state = 1
                    else:
                        if d.strict:
                            raise Error("'%s' expected after '%s'" % (d.delimiter, d.quotechar))
                        field.append(c)
                        state = 1
                else:
                    if c == d.delimiter:
                        row.append(self._value("".join(field), quoted))
                        field = []
                        quoted = False
                        state = 0
                    elif state == 0 and c == " " and d.skipinitialspace:
                        pass
                    elif state == 0 and c == d.quotechar and d.quoting != QUOTE_NONE:
                        state = 2
                        quoted = True
                    elif d.escapechar is not None and c == d.escapechar and i + 1 < n:
                        field.append(line[i + 1])
                        i += 1
                        state = 1
                    elif c in "\r\n":
                        pass
                    else:
                        field.append(c)
                        state = 1
                i += 1
            if state == 2:
                # a newline inside quotes belongs to the field (kept above)
                continue
            break
        if row or field or quoted or state != 0:
            row.append(self._value("".join(field), quoted))
        elif not row and self.line_num and line.strip("\r\n") == "":
            return []
        return row

    def _value(self, s, quoted):
        if not quoted and self.dialect.quoting == QUOTE_NONNUMERIC:
            return float(s)
        return s


def reader(csvfile, dialect="excel", **fmtparams):
    return _Reader(csvfile, dialect, fmtparams)


class _Writer:
    def __init__(self, f, dialect, fmtparams):
        self._f = f
        self.dialect = _Params(dialect, fmtparams)

    def _field(self, v):
        d = self.dialect
        if v is None:
            s = ""
        elif isinstance(v, float):
            s = repr(v)
        else:
            s = str(v)
        if d.quoting == QUOTE_NONE:
            if d.escapechar:
                for ch in (d.escapechar, d.delimiter, d.quotechar):
                    if ch:
                        s = s.replace(ch, d.escapechar + ch)
            elif any(ch in s for ch in (d.delimiter, "\n", "\r")):
                raise Error("need to escape, but no escapechar set")
            return s
        need = (d.quoting == QUOTE_ALL
                or (d.quoting == QUOTE_NONNUMERIC and not isinstance(v, (int, float)))
                or (d.quoting == QUOTE_STRINGS and isinstance(v, str))
                or (d.quoting == QUOTE_NOTNULL and v is not None)
                or any(ch in s for ch in (d.delimiter, d.quotechar, "\n", "\r"))
                or (s and (s[0] == " " and d.skipinitialspace)))
        if d.quotechar and d.quotechar in s:
            if d.doublequote:
                s = s.replace(d.quotechar, d.quotechar * 2)
            elif d.escapechar:
                s = s.replace(d.quotechar, d.escapechar + d.quotechar)
            else:
                raise Error("need to escape, but no escapechar set")
        return d.quotechar + s + d.quotechar if need else s

    def writerow(self, row):
        fields = list(row)
        line = self.dialect.delimiter.join(self._field(v) for v in fields)
        if len(fields) == 1 and line == "":
            line = self.dialect.quotechar * 2
        return self._f.write(line + self.dialect.lineterminator)

    def writerows(self, rows):
        for r in rows:
            self.writerow(r)


def writer(csvfile, dialect="excel", **fmtparams):
    return _Writer(csvfile, dialect, fmtparams)


class DictReader:
    def __init__(self, f, fieldnames=None, restkey=None, restval=None, dialect="excel", *args, **kwds):
        self._fieldnames = fieldnames
        self.restkey = restkey
        self.restval = restval
        self.reader = reader(f, dialect, *args, **kwds)
        self.dialect = dialect
        self.line_num = 0

    def __iter__(self):
        return self

    @property
    def fieldnames(self):
        if self._fieldnames is None:
            try:
                self._fieldnames = next(self.reader)
            except StopIteration:
                pass
        self.line_num = self.reader.line_num
        return self._fieldnames

    @fieldnames.setter
    def fieldnames(self, value):
        self._fieldnames = value

    def __next__(self):
        if self.line_num == 0:
            self.fieldnames
        row = next(self.reader)
        self.line_num = self.reader.line_num
        while row == []:
            row = next(self.reader)
        names = self.fieldnames
        d = dict(zip(names, row))
        lf, lr = len(names), len(row)
        if lf < lr:
            d[self.restkey] = row[lf:]
        elif lf > lr:
            for key in names[lr:]:
                d[key] = self.restval
        return d


class DictWriter:
    def __init__(self, f, fieldnames, restval="", extrasaction="raise", dialect="excel", *args, **kwds):
        self.fieldnames = list(fieldnames)
        self.restval = restval
        if extrasaction.lower() not in ("raise", "ignore"):
            raise ValueError("extrasaction (%s) must be 'raise' or 'ignore'" % extrasaction)
        self.extrasaction = extrasaction
        self.writer = writer(f, dialect, *args, **kwds)

    def writeheader(self):
        return self.writer.writerow(self.fieldnames)

    def _row(self, rowdict):
        if self.extrasaction == "raise":
            wrong = [k for k in rowdict if k not in self.fieldnames]
            if wrong:
                raise ValueError("dict contains fields not in fieldnames: " + ", ".join(repr(x) for x in wrong))
        return (rowdict.get(key, self.restval) for key in self.fieldnames)

    def writerow(self, rowdict):
        return self.writer.writerow(self._row(rowdict))

    def writerows(self, rowdicts):
        for r in rowdicts:
            self.writerow(r)


class Sniffer:
    def sniff(self, sample, delimiters=None):
        cands = delimiters or ",;\t|:"
        lines = [l for l in sample.splitlines() if l][:10]
        best = None
        for c in cands:
            counts = [l.count(c) for l in lines]
            if counts and counts[0] > 0 and all(x == counts[0] for x in counts):
                best = c
                break
        if best is None:
            raise Error("Could not determine delimiter")

        class sniffed(excel):
            pass
        sniffed.delimiter = best
        sniffed.lineterminator = "\r\n" if "\r\n" in sample else "\n"
        return sniffed

    def has_header(self, sample):
        rows = list(reader(sample.splitlines(), self.sniff(sample)))
        if len(rows) < 2:
            return False

        def num(s):
            try:
                float(s)
                return True
            except ValueError:
                return False
        return any(not num(h) and num(v) for h, v in zip(rows[0], rows[1]))
