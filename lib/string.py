# SPDX-License-Identifier: MIT
# string for the Ranger Python runtime: the constants come from the
# native _string, Template, Formatter and capwords are written here.
from _string import *
import re as _re

__all__ = ["ascii_letters", "ascii_lowercase", "ascii_uppercase", "capwords",
           "digits", "hexdigits", "octdigits", "printable", "punctuation",
           "whitespace", "Formatter", "Template"]


def capwords(s, sep=None):
    return (sep or " ").join(map(str.capitalize, s.split(sep)))


class Template:
    delimiter = "$"
    idpattern = r"(?a:[_a-z][_a-z0-9]*)"
    braceidpattern = None
    flags = _re.IGNORECASE

    def __init_subclass__(cls, **kw):
        cls.pattern = cls._compile()

    @classmethod
    def _compile(cls):
        if "pattern" in cls.__dict__ and not isinstance(cls.__dict__["pattern"], str):
            return cls.__dict__["pattern"]
        d = _re.escape(cls.delimiter)
        ident = cls.idpattern.replace("(?a:", "(?:")
        bid = cls.braceidpattern or ident
        src = cls.__dict__.get("pattern")
        if not isinstance(src, str):
            src = (d + r"(?:(?P<escaped>" + d + r")|(?P<named>" + ident + r")|\{(?P<braced>" + bid +
                   r")\}|(?P<invalid>))")
        return _re.compile(src, cls.flags | _re.VERBOSE if src is cls.__dict__.get("pattern") else cls.flags)

    def __init__(self, template):
        self.template = template

    def _pat(self):
        p = type(self).__dict__.get("pattern")
        if p is None or isinstance(p, str):
            p = type(self)._compile()
        return p

    def _invalid(self, mo):
        i = mo.start("invalid")
        lines = self.template[:i].splitlines(keepends=True)
        if not lines:
            colno = 1
            lineno = 1
        else:
            colno = i - len("".join(lines[:-1]))
            lineno = len(lines)
        raise ValueError("Invalid placeholder in string: line %d, col %d" % (lineno, colno))

    def substitute(self, mapping=None, /, **kws):
        if mapping is None:
            mapping = kws
        elif kws:
            m = dict(mapping)
            m.update(kws)
            mapping = m

        def convert(mo):
            named = mo.group("named") or mo.group("braced")
            if named is not None:
                return str(mapping[named])
            if mo.group("escaped") is not None:
                return self.delimiter
            if mo.group("invalid") is not None:
                self._invalid(mo)
            raise ValueError("Unrecognized named group in pattern", self.pattern)
        return self._pat().sub(convert, self.template)

    def safe_substitute(self, mapping=None, /, **kws):
        if mapping is None:
            mapping = kws
        elif kws:
            m = dict(mapping)
            m.update(kws)
            mapping = m

        def convert(mo):
            named = mo.group("named") or mo.group("braced")
            if named is not None:
                try:
                    return str(mapping[named])
                except KeyError:
                    return mo.group()
            if mo.group("escaped") is not None:
                return self.delimiter
            if mo.group("invalid") is not None:
                return mo.group()
            raise ValueError("Unrecognized named group in pattern", self.pattern)
        return self._pat().sub(convert, self.template)

    def is_valid(self):
        for mo in self._pat().finditer(self.template):
            if mo.group("invalid") is not None:
                return False
            if mo.group("named") is None and mo.group("braced") is None and mo.group("escaped") is None:
                raise ValueError("Unrecognized named group in pattern", self.pattern)
        return True

    def get_identifiers(self):
        ids = []
        for mo in self._pat().finditer(self.template):
            named = mo.group("named") or mo.group("braced")
            if named is not None and named not in ids:
                ids.append(named)
            elif named is None and mo.group("invalid") is None and mo.group("escaped") is None:
                raise ValueError("Unrecognized named group in pattern", self.pattern)
        return ids


class Formatter:
    def format(self, format_string, /, *args, **kwargs):
        return self.vformat(format_string, args, kwargs)

    def vformat(self, format_string, args, kwargs):
        used = set()
        result, _ = self._vformat(format_string, args, kwargs, used, 2)
        self.check_unused_args(used, args, kwargs)
        return result

    def _vformat(self, format_string, args, kwargs, used_args, recursion_depth, auto_arg_index=0):
        if recursion_depth < 0:
            raise ValueError("Max string recursion exceeded")
        result = []
        for literal_text, field_name, format_spec, conversion in self.parse(format_string):
            if literal_text:
                result.append(literal_text)
            if field_name is not None:
                if field_name == "":
                    if auto_arg_index is False:
                        raise ValueError("cannot switch from manual field specification to automatic field numbering")
                    field_name = str(auto_arg_index)
                    auto_arg_index += 1
                elif field_name.isdigit():
                    if auto_arg_index:
                        raise ValueError("cannot switch from manual field specification to automatic field numbering")
                    auto_arg_index = False
                obj, arg_used = self.get_field(field_name, args, kwargs)
                used_args.add(arg_used)
                obj = self.convert_field(obj, conversion)
                format_spec, auto_arg_index = self._vformat(format_spec, args, kwargs, used_args,
                                                            recursion_depth - 1, auto_arg_index=auto_arg_index)
                result.append(self.format_field(obj, format_spec))
        return "".join(result), auto_arg_index

    def get_value(self, key, args, kwargs):
        if isinstance(key, int):
            return args[key]
        return kwargs[key]

    def check_unused_args(self, used_args, args, kwargs):
        pass

    def format_field(self, value, format_spec):
        return format(value, format_spec)

    def convert_field(self, value, conversion):
        if conversion is None:
            return value
        if conversion == "s":
            return str(value)
        if conversion == "r":
            return repr(value)
        if conversion == "a":
            return ascii(value)
        raise ValueError("Unknown conversion specifier " + str(conversion))

    def parse(self, format_string):
        out = []
        i = 0
        n = len(format_string)
        lit = []
        while i < n:
            c = format_string[i]
            if c == "{":
                if i + 1 < n and format_string[i + 1] == "{":
                    lit.append("{")
                    i += 2
                    continue
                depth = 1
                j = i + 1
                while j < n and depth:
                    if format_string[j] == "{":
                        depth += 1
                    elif format_string[j] == "}":
                        depth -= 1
                    j += 1
                if depth:
                    raise ValueError("expected '}' before end of string")
                field = format_string[i + 1:j - 1]
                conv = None
                spec = ""
                k = 0
                bracket = 0
                name_end = len(field)
                while k < len(field):
                    ch = field[k]
                    if ch == "[":
                        bracket += 1
                    elif ch == "]":
                        bracket -= 1
                    elif bracket == 0 and ch in "!:":
                        name_end = k
                        break
                    k += 1
                name = field[:name_end]
                rest = field[name_end:]
                if rest.startswith("!"):
                    if len(rest) < 2:
                        raise ValueError("end of string while looking for conversion specifier")
                    conv = rest[1]
                    rest = rest[2:]
                    if rest and not rest.startswith(":"):
                        raise ValueError("expected ':' after conversion specifier")
                if rest.startswith(":"):
                    spec = rest[1:]
                out.append(("".join(lit), name, spec, conv))
                lit = []
                i = j
            elif c == "}":
                if i + 1 < n and format_string[i + 1] == "}":
                    lit.append("}")
                    i += 2
                    continue
                raise ValueError("Single '}' encountered in format string")
            else:
                lit.append(c)
                i += 1
        if lit:
            out.append(("".join(lit), None, None, None))
        return out

    def get_field(self, field_name, args, kwargs):
        first = field_name
        rest = ""
        for k, ch in enumerate(field_name):
            if ch in ".[":
                first = field_name[:k]
                rest = field_name[k:]
                break
        key = int(first) if first.isdigit() else first
        obj = self.get_value(key, args, kwargs)
        while rest:
            if rest[0] == ".":
                k = 1
                while k < len(rest) and rest[k] not in ".[":
                    k += 1
                obj = getattr(obj, rest[1:k])
                rest = rest[k:]
            else:
                k = rest.index("]")
                idx = rest[1:k]
                obj = obj[int(idx) if idx.isdigit() else idx]
                rest = rest[k + 1:]
        return obj, key
Template.pattern = Template._compile()
