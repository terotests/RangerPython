# SPDX-License-Identifier: MIT
# string constants for the Rust runtime (lib/string.py adds the rest)
whitespace = " \t\n\r\v\f"
ascii_lowercase = "abcdefghijklmnopqrstuvwxyz"
ascii_uppercase = "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
ascii_letters = ascii_lowercase + ascii_uppercase
digits = "0123456789"
hexdigits = digits + "abcdef" + "ABCDEF"
octdigits = "01234567"
punctuation = r"""!"#$%&'()*+,-./:;<=>?@[\]^_`{|}~"""
printable = digits + ascii_letters + punctuation + whitespace


def formatter_parser(s):
    """(literal, field_name, format_spec, conversion) for each part of s"""
    if not isinstance(s, str):
        raise TypeError("expected str, got %s" % type(s).__name__)
    out = []
    lit = []
    i, n = 0, len(s)
    while i < n:
        c = s[i]
        if c == "{" and i + 1 < n and s[i + 1] == "{":
            lit.append("{")
            out.append(("".join(lit), None, None, None))
            lit = []
            i += 2
        elif c == "}" and i + 1 < n and s[i + 1] == "}":
            lit.append("}")
            out.append(("".join(lit), None, None, None))
            lit = []
            i += 2
        elif c == "{":
            depth = 1
            bracket = False
            j = i + 1
            while j < n:
                d = s[j]
                if d == "[":
                    bracket = True
                elif d == "]":
                    bracket = False
                elif d == "{" and not bracket:
                    depth += 1
                elif d == "}" and not bracket:
                    depth -= 1
                    if depth == 0:
                        break
                j += 1
            if j >= n:
                raise ValueError("expected '}' before end of string")
            body = s[i + 1:j]
            k, bracket = 0, False
            while k < len(body):
                d = body[k]
                if d == "[":
                    bracket = True
                elif d == "]":
                    bracket = False
                elif d in "!:" and not bracket:
                    break
                k += 1
            name, rest = body[:k], body[k:]
            conv = None
            if rest.startswith("!"):
                if len(rest) < 2:
                    raise ValueError("end of string while looking for conversion specifier")
                conv = rest[1]
                rest = rest[2:]
                if rest and not rest.startswith(":"):
                    raise ValueError("expected ':' after conversion specifier")
            spec = rest[1:] if rest.startswith(":") else ""
            out.append(("".join(lit), name, spec, conv))
            lit = []
            i = j + 1
        elif c == "}":
            raise ValueError("Single '}' encountered in format string")
        else:
            lit.append(c)
            i += 1
    if lit:
        out.append(("".join(lit), None, None, None))
    return iter(out)


def formatter_field_name_split(name):
    """(first, iterator of (is_attribute, key)) for a field name"""
    if not isinstance(name, str):
        raise TypeError("expected str, got %s" % type(name).__name__)
    i = 0
    while i < len(name) and name[i] not in ".[":
        i += 1
    first = name[:i]
    if first.isdigit():
        first = int(first)
    rest = []
    while i < len(name):
        if name[i] == ".":
            j = i + 1
            while j < len(name) and name[j] not in ".[":
                j += 1
            if j == i + 1:
                raise ValueError("Empty attribute in format string")
            rest.append((True, name[i + 1:j]))
            i = j
        elif name[i] == "[":
            j = name.find("]", i)
            if j < 0:
                raise ValueError("Missing ']' in format string")
            key = name[i + 1:j]
            if key == "":
                raise ValueError("Empty attribute in format string")
            rest.append((False, int(key) if key.isdigit() else key))
            i = j + 1
            if i < len(name) and name[i] not in ".[":
                raise ValueError("Only '.' or '[' may follow ']' in format field specifier")
        else:
            raise ValueError("Only '.' or '[' may follow ']' in format field specifier")
    return first, iter(rest)
