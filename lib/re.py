# SPDX-License-Identifier: MIT
# re for the Ranger Python runtime: a backtracking regular expression
# engine written in Python, so it runs the same on every target. A pattern
# is parsed to a tree, compiled to a small instruction list, and run by a
# loop with an explicit backtrack stack (no recursion per character).

__all__ = ["match", "fullmatch", "search", "sub", "subn", "split", "findall",
           "finditer", "compile", "purge", "escape", "error", "Pattern", "Match",
           "A", "I", "L", "M", "S", "X", "U", "ASCII", "IGNORECASE", "LOCALE",
           "MULTILINE", "DOTALL", "VERBOSE", "UNICODE", "NOFLAG", "RegexFlag"]


class RegexFlag(int):
    def __or__(self, other):
        if isinstance(other, int):
            return RegexFlag(int(self) | int(other))
        return NotImplemented

    __ror__ = __or__

    def __and__(self, other):
        if isinstance(other, int):
            return RegexFlag(int(self) & int(other))
        return NotImplemented

    __rand__ = __and__

    def __xor__(self, other):
        if isinstance(other, int):
            return RegexFlag(int(self) ^ int(other))
        return NotImplemented

    def __invert__(self):
        return RegexFlag(~int(self) & 511)

    def __repr__(self):
        v = int(self)
        if v == 0:
            return "re.NOFLAG"
        parts = []
        for name, bit in _FLAG_NAMES:
            if v & bit:
                parts.append("re." + name)
                v &= ~bit
        if v:
            parts.append(hex(v))
        return "|".join(parts)

    __str__ = __repr__

    @property
    def name(self):
        for n, bit in _FLAG_NAMES:
            if int(self) == bit:
                return n
        return None

    @property
    def value(self):
        return int(self)


_FLAG_NAMES = [("TEMPLATE", 1), ("IGNORECASE", 2), ("LOCALE", 4), ("MULTILINE", 8),
               ("DOTALL", 16), ("UNICODE", 32), ("VERBOSE", 64), ("DEBUG", 128),
               ("ASCII", 256)]
NOFLAG = RegexFlag(0)
IGNORECASE = I = RegexFlag(2)
LOCALE = L = RegexFlag(4)
MULTILINE = M = RegexFlag(8)
DOTALL = S = RegexFlag(16)
UNICODE = U = RegexFlag(32)
VERBOSE = X = RegexFlag(64)
DEBUG = RegexFlag(128)
ASCII = A = RegexFlag(256)
_INLINE = {"a": 256, "i": 2, "L": 4, "m": 8, "s": 16, "u": 32, "x": 64}


class error(Exception):
    def __init__(self, msg, pattern=None, pos=None):
        self.msg = msg
        self.pattern = pattern
        self.pos = pos
        if pattern is not None and pos is not None:
            self.lineno = pattern.count("\n", 0, pos) + 1
            self.colno = pos - (pattern.rfind("\n", 0, pos) + 1) + 1
            if "\n" in pattern:
                msg = "%s at position %d (line %d, column %d)" % (msg, pos, self.lineno, self.colno)
            else:
                msg = "%s at position %d" % (msg, pos)
        else:
            self.lineno = self.colno = None
        Exception.__init__(self, msg)


PatternError = error

# ---- parsing: pattern text to a tree of tuples
#   ("lit", ch, icase)  ("any", dotall)  ("set", neg, items, icase, ascii)
#   ("cat", [nodes])  ("alt", [nodes])  ("group", n, node)
#   ("rep", node, lo, hi, mode)  ("at", kind, multiline, ascii)
#   ("look", ahead, neg, node)  ("atomic", node)  ("bref", n, icase)
#   ("cond", n, yes, no)

_DIGITS = "0123456789"
_HEX = "0123456789abcdefABCDEF"
_WS = " \t\n\r\v\f"
_REPEAT = "*+?{"
_SPECIAL = ".^$*+?{}[]\\|()"


class _Parser:
    def __init__(self, pattern, flags):
        self.p = pattern
        self.i = 0
        self.flags = flags
        self.groups = 1
        self.names = {}
        self.open = []
        self.lookbehind = 0
        self.scan_global_flags()

    def err(self, msg, pos=None):
        raise error(msg, self.p, self.i if pos is None else pos)

    def scan_global_flags(self):
        # (?flags) is only allowed at the start, and applies to all of it
        p = self.p
        i = 0
        while p.startswith("(?", i):
            j = i + 2
            while j < len(p) and p[j] in _INLINE:
                j += 1
            if j < len(p) and p[j] == ")" and j > i + 2:
                for c in p[i + 2:j]:
                    self.flags |= _INLINE[c]
                i = j + 1
            else:
                break

    def peek(self):
        return self.p[self.i] if self.i < len(self.p) else ""

    def skip_verbose(self, flags):
        if not flags & 64:
            return
        p = self.p
        while self.i < len(p):
            c = p[self.i]
            if c in _WS:
                self.i += 1
            elif c == "#":
                while self.i < len(p) and p[self.i] != "\n":
                    self.i += 1
            else:
                break

    def parse(self):
        node = self.alt(self.flags)
        if self.i < len(self.p):
            if self.p[self.i] == ")":
                self.err("unbalanced parenthesis")
            self.err("unexpected end")
        return node

    def alt(self, flags):
        items = [self.seq(flags)]
        while self.peek() == "|":
            self.i += 1
            items.append(self.seq(flags))
        if len(items) == 1:
            return items[0]
        return ("alt", items)

    def seq(self, flags):
        items = []
        while True:
            self.skip_verbose(flags)
            c = self.peek()
            if c == "" or c == "|" or c == ")":
                break
            start = self.i
            atom = self.atom(flags)
            if atom is None:
                continue
            atom = self.repeats(atom, flags, start)
            items.append(atom)
        if len(items) == 1:
            return items[0]
        return ("cat", items)

    def repeats(self, atom, flags, start):
        self.skip_verbose(flags)
        c = self.peek()
        if c == "" or c not in _REPEAT:
            return atom
        here = self.i
        lo, hi = self.quant()
        if lo is None:
            return atom
        if atom[0] == "at" or (atom[0] == "cat" and not atom[1]):
            self.err("nothing to repeat", here)
        if atom[0] == "look":
            pass
        mode = "greedy"
        if self.peek() == "?":
            self.i += 1
            mode = "lazy"
        elif self.peek() == "+":
            self.i += 1
            mode = "poss"
        node = ("rep", atom, lo, hi, mode)
        self.skip_verbose(flags)
        c = self.peek()
        if c and c in "*+?" or (c == "{" and self.is_quant(self.i)):
            self.err("multiple repeat", self.i)
        return node

    def is_quant(self, i):
        p = self.p
        j = i + 1
        while j < len(p) and p[j] in _DIGITS:
            j += 1
        if j < len(p) and p[j] == ",":
            j += 1
            while j < len(p) and p[j] in _DIGITS:
                j += 1
        return j < len(p) and p[j] == "}"

    def quant(self):
        c = self.p[self.i]
        if c == "*":
            self.i += 1
            return 0, None
        if c == "+":
            self.i += 1
            return 1, None
        if c == "?":
            self.i += 1
            return 0, 1
        # {m,n}; anything else is a literal "{"
        if not self.is_quant(self.i):
            return None, None
        here = self.i
        j = self.p.index("}", self.i)
        body = self.p[self.i + 1:j]
        self.i = j + 1
        if "," in body:
            a, b = body.split(",", 1)
            lo = int(a) if a else 0
            hi = int(b) if b else None
        else:
            lo = hi = int(body)
        if hi is not None and hi < lo:
            self.err("min repeat greater than max repeat", here + 1)
        if lo >= 4294967295 or (hi is not None and hi >= 4294967295):
            raise OverflowError("the repetition number is too large")
        return lo, hi

    def atom(self, flags):
        p = self.p
        c = p[self.i]
        if c == "(":
            return self.group(flags)
        if c == "[":
            return self.charset(flags)
        if c == ".":
            self.i += 1
            return ("any", bool(flags & 16))
        if c == "^":
            self.i += 1
            return ("at", "^", bool(flags & 8), False)
        if c == "$":
            self.i += 1
            return ("at", "$", bool(flags & 8), False)
        if c == "\\":
            return self.escape(flags, False)
        if c in "*+?":
            self.err("nothing to repeat")
        if c == "{" and self.is_quant(self.i):
            self.err("nothing to repeat")
        self.i += 1
        return ("lit", c, bool(flags & 2))

    def group_name(self, end):
        j = self.p.find(end, self.i)
        if j < 0:
            self.err("missing %s, unterminated name" % end)
        name = self.p[self.i:j]
        if not name:
            self.err("missing group name")
        if not name.isidentifier():
            self.err("bad character in group name %r" % name)
        self.i = j + 1
        return name

    def group(self, flags):
        p = self.p
        start = self.i
        self.i += 1
        if p.startswith("?", self.i):
            self.i += 1
            c = self.peek()
            if c == "P":
                self.i += 1
                c2 = self.peek()
                if c2 == "<":
                    self.i += 1
                    name = self.group_name(">")
                    if name in self.names:
                        self.err("redefinition of group name %r as group %d; was group %d" % (name, self.groups, self.names[name]), start + 4)
                    n = self.new_group()
                    self.names[name] = n
                    return self.group_body(("group", n), flags, start)
                if c2 == "=":
                    self.i += 1
                    here = self.i
                    name = self.group_name(")")
                    if name not in self.names:
                        self.err("unknown group name %r" % name, here)
                    n = self.names[name]
                    if n in self.open:
                        self.err("cannot refer to an open group", start)
                    return ("bref", n, bool(flags & 2))
                if c2 == "":
                    self.err("unexpected end of pattern")
                self.err("unknown extension ?P" + c2, start + 1)
            if c == ":":
                self.i += 1
                return self.group_body(None, flags, start)
            if c == "#":
                j = p.find(")", self.i)
                if j < 0:
                    self.err("missing ), unterminated comment", start)
                self.i = j + 1
                return None
            if c == "=" or c == "!":
                self.i += 1
                return self.group_body(("look", True, c == "!"), flags, start)
            if c == "<":
                self.i += 1
                c2 = self.peek()
                if c2 == "=" or c2 == "!":
                    self.i += 1
                    self.lookbehind += 1
                    node = self.group_body(("look", False, c2 == "!"), flags, start)
                    self.lookbehind -= 1
                    lo, hi = _width(node[3])
                    if lo != hi:
                        self.err("look-behind requires fixed-width pattern", start)
                    return node
                if c2 and (c2.isalpha() or c2 == "_"):
                    name = self.group_name(">")
                    if name in self.names:
                        self.err("redefinition of group name %r as group %d; was group %d" % (name, self.groups, self.names[name]), start + 3)
                    n = self.new_group()
                    self.names[name] = n
                    return self.group_body(("group", n), flags, start)
                if c2 == "":
                    self.err("unexpected end of pattern")
                self.err("unknown extension ?<" + c2, start + 1)
            if c == ">":
                self.i += 1
                return self.group_body(("atomic",), flags, start)
            if c == "(":
                self.i += 1
                here = self.i
                j = p.find(")", self.i)
                if j < 0:
                    self.err("missing ), unterminated name")
                ref = p[self.i:j]
                self.i = j + 1
                if ref.isdigit():
                    n = int(ref)
                    if n == 0:
                        self.err("bad group number", here)
                elif ref.isidentifier():
                    if ref not in self.names:
                        self.err("unknown group name %r" % ref, here)
                    n = self.names[ref]
                else:
                    self.err("bad character in group name %r" % ref, here)
                yes = self.seq(flags)
                no = ("cat", [])
                if self.peek() == "|":
                    self.i += 1
                    no = self.seq(flags)
                    if self.peek() == "|":
                        self.err("conditional backref with more than two branches")
                if self.peek() != ")":
                    self.err("missing ), unterminated subpattern", start)
                self.i += 1
                if n >= 100:
                    self.err("invalid group reference %d" % n, here)
                return ("cond", n, yes, no)
            # inline flags, scoped (?i:...) or global (?i)
            add = 0
            rem = 0
            neg = False
            while True:
                c = self.peek()
                if c in _INLINE and c != "":
                    if neg:
                        if c not in "imsx":
                            self.err("bad inline flags: cannot turn off flags 'a', 'u' and 'L'")
                        rem |= _INLINE[c]
                    else:
                        add |= _INLINE[c]
                    self.i += 1
                elif c == "-" and not neg:
                    neg = True
                    self.i += 1
                    if self.peek() not in _INLINE or self.peek() == "":
                        self.err("missing flag" if self.peek() in ":)" else "unknown flag")
                else:
                    break
            if (add & 256) and (add & 32):
                self.err("bad inline flags: flags 'a', 'u' and 'L' are incompatible")
            c = self.peek()
            if c == ")":
                if neg:
                    self.err("missing :")
                if start != 0 and not self.at_global_prefix(start):
                    self.err("global flags not at the start of the expression", start)
                self.i += 1
                return None
            if c == ":":
                self.i += 1
                nf = (flags | add) & ~rem
                return self.group_body(None, nf, start)
            if c == "":
                self.err("missing -, : or )")
            if add == 0 and rem == 0 and not neg:
                self.err("unknown extension ?" + c, start + 1)
            self.err("unknown flag" if c.isalpha() else "missing -, : or )")
        n = self.new_group()
        return self.group_body(("group", n), flags, start)

    def at_global_prefix(self, start):
        # every (?flags) before this one at the very start is also global
        i = 0
        p = self.p
        while i < start:
            if not p.startswith("(?", i):
                return False
            j = p.find(")", i)
            i = j + 1
        return i == start

    def new_group(self):
        n = self.groups
        self.groups += 1
        if self.groups > 101:
            pass
        return n

    def group_body(self, kind, flags, start):
        if kind is not None and kind[0] == "group":
            self.open.append(kind[1])
        node = self.alt(flags)
        if self.peek() != ")":
            self.err("missing ), unterminated subpattern", start)
        self.i += 1
        if kind is None:
            return node
        if kind[0] == "group":
            self.open.pop()
            return ("group", kind[1], node)
        if kind[0] == "look":
            return ("look", kind[1], kind[2], node)
        return ("atomic", node)

    def escape_char(self, inset):
        # an escape that stands for one character, or None
        p = self.p
        start = self.i
        c = p[self.i + 1] if self.i + 1 < len(p) else ""
        if c == "":
            self.err("bad escape (end of pattern)")
        simple = {"n": "\n", "t": "\t", "r": "\r", "f": "\f", "v": "\v", "a": "\a", "\\": "\\"}
        if c in simple:
            self.i += 2
            return simple[c]
        if inset and c == "b":
            self.i += 2
            return "\b"
        if c == "x":
            h = p[self.i + 2:self.i + 4]
            if len(h) < 2 or not all(ch in _HEX for ch in h):
                self.err("incomplete escape \\x" + h)
            self.i += 4
            return chr(int(h, 16))
        if c == "u" or c == "U":
            k = 4 if c == "u" else 8
            h = p[self.i + 2:self.i + 2 + k]
            if len(h) < k or not all(ch in _HEX for ch in h):
                self.err("incomplete escape \\%s%s" % (c, h))
            v = int(h, 16)
            if v > 0x10FFFF:
                self.err("bad escape \\%s%s" % (c, h))
            self.i += 2 + k
            return chr(v)
        if c == "N":
            if not p.startswith("{", self.i + 2):
                self.err("missing {", self.i + 2)
            j = p.find("}", self.i + 3)
            if j < 0:
                self.err("missing }, unterminated name", self.i + 3)
            name = p[self.i + 3:j]
            if not name:
                self.err("missing character name", self.i + 3)
            try:
                import unicodedata
                ch = unicodedata.lookup(name)
            except Exception:
                try:
                    ch = eval('"\\N{' + name + '}"')
                except Exception:
                    self.err("undefined character name %r" % name)
            self.i = j + 1
            return ch
        if c == "0":
            j = self.i + 2
            while j < len(p) and j < self.i + 4 and p[j] in "01234567":
                j += 1
            v = int(p[self.i + 1:j], 8)
            self.i = j
            return chr(v)
        if c in "1234567" and inset:
            j = self.i + 1
            while j < len(p) and j < self.i + 4 and p[j] in "01234567":
                j += 1
            v = int(p[self.i + 1:j], 8)
            if v > 0o377:
                self.err("octal escape value \\%s outside of range 0-0o377" % p[self.i + 1:j])
            self.i = j
            return chr(v)
        if c.isalnum() and c.isascii():
            return None
        self.i += 2
        return c

    def escape(self, flags, inset):
        p = self.p
        start = self.i
        c = p[self.i + 1] if self.i + 1 < len(p) else ""
        icase = bool(flags & 2)
        asc = bool(flags & 256)
        if c in "dDwWsS" and c:
            self.i += 2
            return ("set", False, [("cls", c)], icase, asc)
        if c in "AZbB" and c:
            self.i += 2
            return ("at", c, False, asc)
        if c and c in "123456789":
            # a back reference, or an octal escape of three digits
            j = self.i + 1
            if (j + 2 < len(p) + 0 and p[j] in "01234567" and p[j + 1:j + 2] and p[j + 1] in "01234567"
                    and p[j + 2:j + 3] and p[j + 2] in "01234567"):
                v = int(p[j:j + 3], 8)
                if v > 0o377:
                    self.err("octal escape value \\%s outside of range 0-0o377" % p[j:j + 3])
                self.i = j + 3
                return ("lit", chr(v), icase)
            k = j
            while k < len(p) and k < j + 2 and p[k] in _DIGITS:
                k += 1
            n = int(p[j:k])
            self.i = k
            if n >= self.groups:
                self.err("invalid group reference %d" % n, j)
            if n in self.open:
                self.err("cannot refer to an open group", start)
            return ("bref", n, icase)
        ch = self.escape_char(False)
        if ch is None:
            self.err("bad escape \\" + c)
        return ("lit", ch, icase)

    def charset(self, flags):
        p = self.p
        start = self.i
        self.i += 1
        neg = False
        if self.peek() == "^":
            neg = True
            self.i += 1
        items = []
        first = True
        while True:
            if self.i >= len(p):
                self.err("unterminated character set", start)
            c = p[self.i]
            if c == "]" and not first:
                self.i += 1
                break
            first = False
            here = self.i
            lo = self.set_item(flags)
            if lo[0] == "ch" and p.startswith("-", self.i) and self.i + 1 < len(p) and p[self.i + 1] != "]":
                self.i += 1
                hi = self.set_item(flags)
                if hi[0] != "ch":
                    self.err("bad character range %s" % p[here:self.i], here)
                if ord(hi[1]) < ord(lo[1]):
                    self.err("bad character range %s" % p[here:self.i], here)
                items.append(("range", lo[1], hi[1]))
            else:
                items.append(lo)
        return ("set", neg, items, bool(flags & 2), bool(flags & 256))

    def set_item(self, flags):
        p = self.p
        c = p[self.i]
        if c == "\\":
            d = p[self.i + 1] if self.i + 1 < len(p) else ""
            if d and d in "dDwWsS":
                self.i += 2
                return ("cls", d)
            ch = self.escape_char(True)
            if ch is None:
                self.err("bad escape \\" + d)
            return ("ch", ch)
        self.i += 1
        return ("ch", c)


def _width(node):
    k = node[0]
    if k == "lit" or k == "any" or k == "set":
        return 1, 1
    if k == "cat":
        lo = hi = 0
        for n in node[1]:
            a, b = _width(n)
            lo += a
            hi = None if hi is None or b is None else hi + b
        return lo, hi
    if k == "alt":
        ws = [_width(n) for n in node[1]]
        lo = min(w[0] for w in ws)
        his = [w[1] for w in ws]
        return lo, None if None in his else max(his)
    if k == "group" or k == "atomic":
        return _width(node[-1])
    if k == "rep":
        a, b = _width(node[1])
        hi = None if node[3] is None or b is None else b * node[3]
        return a * node[2], hi
    if k == "at" or k == "look":
        return 0, 0
    if k == "bref":
        return 0, None
    if k == "cond":
        a = _width(node[2])
        b = _width(node[3])
        return min(a[0], b[0]), (None if a[1] is None or b[1] is None else max(a[1], b[1]))
    return 0, 0


# ---- compiling the tree to instructions
_CHAR, _ANY, _SET, _SPLIT, _JMP, _SAVE, _AT, _BREF, _LOOK, _ATOMIC, _COND, _MARK, _PROG, _MATCH = range(14)


class _Compiler:
    def __init__(self, nslots):
        self.code = []
        self.nslots = nslots
        self.marks = 0

    def emit(self, *ins):
        self.code.append(list(ins))
        return len(self.code) - 1

    def sub(self, node):
        c = _Compiler(self.nslots)
        c.marks = self.marks
        c.node(node)
        c.emit(_MATCH)
        self.marks = c.marks
        return c.code

    def node(self, n):
        k = n[0]
        if k == "lit":
            if n[2] and n[1].lower() != n[1].upper():
                self.emit(_SET, False, [("ch", n[1])], True, False)
            else:
                self.emit(_CHAR, n[1])
        elif k == "any":
            self.emit(_ANY, n[1])
        elif k == "set":
            self.emit(_SET, n[1], n[2], n[3], n[4])
        elif k == "cat":
            for x in n[1]:
                self.node(x)
        elif k == "alt":
            jumps = []
            items = n[1]
            for idx in range(len(items)):
                if idx < len(items) - 1:
                    sp = self.emit(_SPLIT, 0, 0)
                    self.code[sp][1] = len(self.code)
                    self.node(items[idx])
                    jumps.append(self.emit(_JMP, 0))
                    self.code[sp][2] = len(self.code)
                else:
                    self.node(items[idx])
            for j in jumps:
                self.code[j][1] = len(self.code)
        elif k == "group":
            self.emit(_SAVE, 2 * n[1])
            self.node(n[2])
            self.emit(_SAVE, 2 * n[1] + 1)
        elif k == "at":
            self.emit(_AT, n[1], n[2], n[3])
        elif k == "bref":
            self.emit(_BREF, n[1], n[2])
        elif k == "look":
            w = _width(n[3])[0] if not n[1] else 0
            self.emit(_LOOK, self.sub(n[3]), n[1], n[2], w)
        elif k == "atomic":
            self.emit(_ATOMIC, self.sub(n[1]))
        elif k == "cond":
            c = self.emit(_COND, n[1], 0)
            self.node(n[2])
            j = self.emit(_JMP, 0)
            self.code[c][2] = len(self.code)
            self.node(n[3])
            self.code[j][1] = len(self.code)
        elif k == "rep":
            self.rep(n)

    def rep(self, n):
        body, lo, hi, mode = n[1], n[2], n[3], n[4]
        if mode == "poss":
            self.emit(_ATOMIC, self.sub(("rep", body, lo, hi, "greedy")))
            return
        lazy = mode == "lazy"
        nullable = _width(body)[0] == 0
        for _ in range(lo):
            self.node(body)
        if hi is None:
            # L: split body, out; body; jmp L
            top = len(self.code)
            sp = self.emit(_SPLIT, 0, 0)
            mark = None
            first = len(self.code)
            if nullable:
                mark = self.nslots + self.marks
                self.marks += 1
                self.emit(_MARK, mark)
            self.node(body)
            prog = None
            if nullable:
                prog = self.emit(_PROG, mark, 0)
            self.emit(_JMP, top)
            out = len(self.code)
            if prog is not None:
                self.code[prog][2] = out
            self.code[sp][1], self.code[sp][2] = (out, first) if lazy else (first, out)
        else:
            splits = []
            for _ in range(hi - lo):
                sp = self.emit(_SPLIT, 0, 0)
                splits.append((sp, len(self.code)))
                self.node(body)
            out = len(self.code)
            for sp, first in splits:
                self.code[sp][1], self.code[sp][2] = (out, first) if lazy else (first, out)


def _is_word(ch, asc):
    if asc:
        return ch.isascii() and (ch.isalnum() or ch == "_")
    return ch.isalnum() or ch == "_"


def _in_cls(ch, cls, asc):
    if cls == "d":
        return ("0" <= ch <= "9") if asc else ch.isdigit() and ch.isdecimal()
    if cls == "D":
        return not _in_cls(ch, "d", asc)
    if cls == "w":
        return _is_word(ch, asc)
    if cls == "W":
        return not _is_word(ch, asc)
    if cls == "s":
        if asc:
            return ch in _WS
        return ch.isspace() or ch in "\x1c\x1d\x1e\x1f\x85"
    return not _in_cls(ch, "s", asc)


def _in_set(ch, items, icase, asc):
    for it in items:
        k = it[0]
        if k == "ch":
            if ch == it[1]:
                return True
            if icase and (ch.lower() == it[1].lower() or ch.upper() == it[1].upper()):
                return True
        elif k == "range":
            if it[1] <= ch <= it[2]:
                return True
            if icase:
                lo = ch.lower()
                up = ch.upper()
                if it[1] <= lo <= it[2] or it[1] <= up <= it[2]:
                    return True
        elif _in_cls(ch, it[1], asc):
            return True
    return False


def _run(code, s, start, end, sv, avoid):
    # one match attempt at start; returns (end, slots) or None. avoid is a
    # position where an empty match is not accepted (after an empty match)
    stack = [(0, start, sv)]
    while stack:
        pc, i, sv = stack.pop()
        while True:
            ins = code[pc]
            op = ins[0]
            if op == _CHAR:
                if i < end and s[i] == ins[1]:
                    i += 1
                    pc += 1
                    continue
                break
            if op == _SET:
                if i < end and _in_set(s[i], ins[2], ins[3], ins[4]) != ins[1]:
                    i += 1
                    pc += 1
                    continue
                break
            if op == _SPLIT:
                stack.append((ins[2], i, sv))
                pc = ins[1]
                continue
            if op == _JMP:
                pc = ins[1]
                continue
            if op == _ANY:
                if i < end and (ins[1] or s[i] != "\n"):
                    i += 1
                    pc += 1
                    continue
                break
            if op == _SAVE:
                sv = sv[:]
                sv[ins[1]] = i
                if ins[1] & 1:
                    sv[-1] = ins[1] >> 1
                pc += 1
                continue
            if op == _MARK:
                sv = sv[:]
                sv[ins[1]] = i
                pc += 1
                continue
            if op == _PROG:
                # an iteration that matched nothing ends the loop
                if sv[ins[1]] == i:
                    pc = ins[2]
                else:
                    pc += 1
                continue
            if op == _MATCH:
                if avoid is not None and i == start == avoid:
                    break
                return i, sv
            if op == _AT:
                if _at(ins[1], ins[2], ins[3], s, i, end):
                    pc += 1
                    continue
                break
            if op == _BREF:
                a = sv[2 * ins[1]]
                b = sv[2 * ins[1] + 1]
                if a < 0 or b < 0:
                    break
                g = s[a:b]
                if ins[2]:
                    if s[i:i + len(g)].lower() == g.lower() and i + len(g) <= end:
                        i += len(g)
                        pc += 1
                        continue
                    break
                if i + len(g) <= end and s.startswith(g, i):
                    i += len(g)
                    pc += 1
                    continue
                break
            if op == _LOOK:
                if ins[2]:
                    at = i
                else:
                    at = i - ins[4]
                if at < 0:
                    r = None
                else:
                    r = _run(ins[1], s, at, end if ins[2] else i, sv, None)
                    if r is not None and not ins[2] and r[0] != i:
                        r = None
                if ins[3]:
                    if r is not None:
                        break
                else:
                    if r is None:
                        break
                    sv = r[1]
                pc += 1
                continue
            if op == _ATOMIC:
                r = _run(ins[1], s, i, end, sv, None)
                if r is None:
                    break
                i, sv = r
                pc += 1
                continue
            if op == _COND:
                if sv[2 * ins[1] + 1] >= 0:
                    pc += 1
                else:
                    pc = ins[2]
                continue
            break
    return None


def _at(kind, ml, asc, s, i, end):
    if kind == "^":
        return i == 0 or (ml and s[i - 1] == "\n")
    if kind == "$":
        if ml:
            return i == end or s[i] == "\n"
        return i == end or (i == end - 1 and s[i] == "\n")
    if kind == "A":
        return i == 0
    if kind == "Z":
        return i == end
    a = i > 0 and _is_word(s[i - 1], asc)
    b = i < end and _is_word(s[i], asc)
    if kind == "b":
        return a != b
    return a == b


class Pattern:
    def __init__(self, pattern, flags):
        if isinstance(flags, RegexFlag) is False:
            flags = RegexFlag(flags)
        if not isinstance(pattern, str):
            raise TypeError("first argument must be string or compiled pattern")
        if flags & 256 and flags & 32:
            raise ValueError("ASCII and UNICODE flags are incompatible")
        if flags & 4:
            raise ValueError("cannot use LOCALE flag with a str pattern")
        ps = _Parser(pattern, int(flags))
        tree = ps.parse()
        self.pattern = pattern
        self.flags = RegexFlag(ps.flags | (0 if ps.flags & 256 else 32))
        self.groups = ps.groups - 1
        self.groupindex = dict(ps.names)
        nslots = 2 * ps.groups
        c = _Compiler(nslots)
        c.node(tree)
        c.emit(_MATCH)
        self._code = c.code
        self._slots = [-1] * (nslots + c.marks + 1)
        # a pattern that starts with a plain character can skip ahead to it
        self._first = c.code[0][1] if c.code[0][0] == _CHAR else None

    def __repr__(self):
        fl = int(self.flags) & ~32
        r = repr(self.pattern)
        if len(r) > 200:
            r = r[:200]
        if fl:
            return "re.compile(%s, %r)" % (r, RegexFlag(fl))
        return "re.compile(%s)" % r

    def __eq__(self, other):
        if not isinstance(other, Pattern):
            return NotImplemented
        return self.pattern == other.pattern and self.flags == other.flags

    def __hash__(self):
        return hash((self.pattern, int(self.flags)))

    def __copy__(self):
        return self

    def __deepcopy__(self, memo):
        return self

    def __class_getitem__(cls, item):
        import types
        return types.GenericAlias(cls, item)

    def _bounds(self, string, pos, endpos):
        if not isinstance(string, str):
            raise TypeError("expected string or bytes-like object, got '%s'" % type(string).__name__)
        n = len(string)
        if pos < 0:
            pos = 0
        if endpos is None or endpos > n:
            endpos = n
        if pos > endpos:
            pos = endpos
        return pos, endpos

    def _at(self, string, i, endpos, avoid=None):
        r = _run(self._code, string, i, endpos, self._slots, avoid)
        if r is None:
            return None
        return r

    def _search(self, string, pos, endpos, avoid=None):
        i = pos
        first = self._first
        while i <= endpos:
            if first is not None:
                i = string.find(first, i, endpos)
                if i < 0:
                    return None
            r = _run(self._code, string, i, endpos, self._slots, avoid)
            if r is not None:
                return Match(self, string, pos, endpos, i, r[0], r[1])
            i += 1
        return None

    def match(self, string, pos=0, endpos=None):
        pos, endpos = self._bounds(string, pos, endpos)
        r = _run(self._code, string, pos, endpos, self._slots, None)
        if r is None:
            return None
        return Match(self, string, pos, endpos, pos, r[0], r[1])

    def fullmatch(self, string, pos=0, endpos=None):
        pos, endpos = self._bounds(string, pos, endpos)
        code = self._full_code()
        r = _run(code, string, pos, endpos, self._slots, None)
        if r is None:
            return None
        return Match(self, string, pos, endpos, pos, r[0], r[1])

    def _full_code(self):
        fc = getattr(self, "_fcode", None)
        if fc is None:
            # the pattern, atomic so far as it must reach the end: append \Z
            fc = [list(x) for x in self._code]
            fc[-1] = [_AT, "Z", False, False]
            fc.append([_MATCH])
            self._fcode = fc
        return fc

    def search(self, string, pos=0, endpos=None):
        pos, endpos = self._bounds(string, pos, endpos)
        return self._search(string, pos, endpos)

    def finditer(self, string, pos=0, endpos=None):
        pos, endpos = self._bounds(string, pos, endpos)
        return iter(self._all(string, pos, endpos, 0))

    def _all(self, string, pos, endpos, count):
        out = []
        avoid = None
        i = pos
        while i <= endpos:
            m = self._search(string, i, endpos, avoid)
            if m is None:
                break
            m.pos = pos
            out.append(m)
            if count and len(out) >= count:
                break
            a, b = m.span()
            if a == b:
                avoid = b
            else:
                avoid = None
            i = b
        return out

    def findall(self, string, pos=0, endpos=None):
        pos, endpos = self._bounds(string, pos, endpos)
        out = []
        for m in self._all(string, pos, endpos, 0):
            if self.groups == 0:
                out.append(m.group())
            elif self.groups == 1:
                out.append(m.group(1) or "")
            else:
                out.append(tuple(g or "" for g in m.groups()))
        return out

    def split(self, string, maxsplit=0):
        pos, endpos = self._bounds(string, 0, None)
        out = []
        last = 0
        for m in self._all(string, 0, endpos, maxsplit if maxsplit > 0 else 0):
            if maxsplit < 0:
                break
            a, b = m.span()
            out.append(string[last:a])
            out.extend(m.groups())
            last = b
        out.append(string[last:])
        return out

    def subn(self, repl, string, count=0):
        pos, endpos = self._bounds(string, 0, None)
        if callable(repl):
            fn = repl
        else:
            if not isinstance(repl, str):
                raise TypeError("expected str instance, %s found" % type(repl).__name__)
            tpl = _template(self, repl)
            if len(tpl) == 1 and isinstance(tpl[0], str):
                lit = tpl[0]
                fn = lambda m: lit
            else:
                fn = lambda m: _expand(m, tpl)
        parts = []
        last = 0
        n = 0
        for m in self._all(string, 0, endpos, count if count > 0 else 0):
            if count < 0:
                break
            a, b = m.span()
            parts.append(string[last:a])
            r = fn(m)
            if r is None:
                r = ""
            if not isinstance(r, str):
                raise TypeError("expected str instance, %s found" % type(r).__name__)
            parts.append(r)
            last = b
            n += 1
        parts.append(string[last:])
        return "".join(parts), n

    def sub(self, repl, string, count=0):
        return self.subn(repl, string, count)[0]

    def scanner(self, string, pos=0, endpos=None):
        return _Scanner(self, string, pos, endpos)


class _Scanner:
    def __init__(self, pattern, string, pos, endpos):
        self.pattern = pattern
        self._s = string
        self._pos, self._end = pattern._bounds(string, pos, endpos)
        self._avoid = None

    def match(self):
        if self._pos > self._end:
            return None
        r = _run(self.pattern._code, self._s, self._pos, self._end, self.pattern._slots, self._avoid)
        if r is None:
            return None
        m = Match(self.pattern, self._s, self._pos, self._end, self._pos, r[0], r[1])
        self._avoid = r[0] if r[0] == self._pos else None
        self._pos = r[0]
        return m

    def search(self):
        if self._pos > self._end:
            return None
        m = self.pattern._search(self._s, self._pos, self._end, self._avoid)
        if m is None:
            self._pos = self._end + 1
            return None
        a, b = m.span()
        self._avoid = b if a == b else None
        self._pos = b
        return m


def _template(pattern, repl):
    # a replacement string as literal parts and group numbers
    out = []
    lit = []
    i = 0
    n = len(repl)
    simple = {"n": "\n", "t": "\t", "r": "\r", "f": "\f", "v": "\v", "a": "\a", "b": "\b", "\\": "\\"}
    while i < n:
        c = repl[i]
        if c != "\\":
            lit.append(c)
            i += 1
            continue
        if i + 1 >= n:
            raise error("bad escape (end of pattern)", repl, i)
        d = repl[i + 1]
        if d == "g":
            if not repl.startswith("<", i + 2):
                raise error("missing <", repl, i + 2)
            j = repl.find(">", i + 3)
            if j < 0:
                raise error("missing >, unterminated name", repl, i + 3)
            name = repl[i + 3:j]
            if not name:
                raise error("missing group name", repl, i + 3)
            if name.isdigit() and name.isascii():
                g = int(name)
                if g > pattern.groups:
                    raise error("invalid group reference %d" % g, repl, i + 3)
            elif name.isidentifier():
                if name not in pattern.groupindex:
                    raise IndexError("unknown group name %r" % name)
                g = pattern.groupindex[name]
            else:
                raise error("bad character in group name %r" % name, repl, i + 3)
            out.append("".join(lit))
            lit = []
            out.append(g)
            i = j + 1
        elif d == "0":
            j = i + 2
            while j < n and j < i + 4 and repl[j] in "01234567":
                j += 1
            lit.append(chr(int(repl[i + 1:j], 8)))
            i = j
        elif d in "123456789":
            j = i + 1
            if (j + 2 < n + 0 and repl[j] in "01234567" and repl[j + 1] in "01234567"
                    and repl[j + 2] in "01234567"):
                v = int(repl[j:j + 3], 8)
                if v > 0o377:
                    raise error("octal escape value \\%s outside of range 0-0o377" % repl[j:j + 3], repl, i)
                lit.append(chr(v))
                i = j + 3
                continue
            k = j
            while k < n and k < j + 2 and repl[k] in _DIGITS:
                k += 1
            g = int(repl[j:k])
            if g > pattern.groups:
                raise error("invalid group reference %d" % g, repl, j)
            out.append("".join(lit))
            lit = []
            out.append(g)
            i = k
        elif d in simple:
            lit.append(simple[d])
            i += 2
        elif d.isascii() and d.isalpha():
            raise error("bad escape \\" + d, repl, i)
        else:
            lit.append("\\" + d)
            i += 2
    out.append("".join(lit))
    return [x for x in out if x != ""] or [""]


def _expand(m, tpl):
    parts = []
    for x in tpl:
        if isinstance(x, str):
            parts.append(x)
        else:
            parts.append(m.group(x) or "")
    return "".join(parts)


class Match:
    def __init__(self, pattern, string, pos, endpos, start, end, slots):
        self.re = pattern
        self.string = string
        self.pos = pos
        self.endpos = endpos
        n = pattern.groups + 1
        sp = [(start, end)]
        for g in range(1, n):
            a = slots[2 * g]
            b = slots[2 * g + 1]
            sp.append((a, b) if a >= 0 and b >= 0 else (-1, -1))
        self._spans = sp
        last = slots[-1] if slots[-1] > 0 else None
        self.lastindex = last
        self.lastgroup = None
        if last is not None:
            for k, v in pattern.groupindex.items():
                if v == last:
                    self.lastgroup = k

    def _index(self, g):
        if isinstance(g, str):
            if g in self.re.groupindex:
                return self.re.groupindex[g]
            raise IndexError("no such group")
        if isinstance(g, int) and not isinstance(g, bool) or hasattr(g, "__index__"):
            try:
                g = g.__index__()
            except AttributeError:
                pass
            if 0 <= g <= self.re.groups:
                return g
        raise IndexError("no such group")

    def _get(self, g, default=None):
        a, b = self._spans[self._index(g)]
        if a < 0:
            return default
        return self.string[a:b]

    def group(self, *args):
        if not args:
            return self._get(0)
        if len(args) == 1:
            return self._get(args[0])
        return tuple(self._get(g) for g in args)

    def __getitem__(self, g):
        return self._get(g)

    def groups(self, default=None):
        return tuple(self._get(g, default) for g in range(1, self.re.groups + 1))

    def groupdict(self, default=None):
        return {k: self._get(v, default) for k, v in self.re.groupindex.items()}

    def start(self, group=0):
        return self._spans[self._index(group)][0]

    def end(self, group=0):
        return self._spans[self._index(group)][1]

    def span(self, group=0):
        return self._spans[self._index(group)]

    @property
    def regs(self):
        return tuple(self._spans)

    def expand(self, template):
        return _expand(self, _template(self.re, template))

    def __repr__(self):
        return "<re.Match object; span=%r, match=%r>" % (self.span(), self.group())

    def __bool__(self):
        return True

    def __copy__(self):
        return self

    def __deepcopy__(self, memo):
        return self

    def __class_getitem__(cls, item):
        import types
        return types.GenericAlias(cls, item)


_cache = {}


def _compile(pattern, flags):
    if isinstance(pattern, Pattern):
        if flags:
            raise ValueError("cannot process flags argument with a compiled pattern")
        return pattern
    key = (type(pattern).__name__, pattern, int(flags))
    p = _cache.get(key)
    if p is None:
        p = Pattern(pattern, flags)
        if len(_cache) >= 512:
            _cache.clear()
        _cache[key] = p
    return p


def compile(pattern, flags=0):
    return _compile(pattern, flags)


def match(pattern, string, flags=0):
    return _compile(pattern, flags).match(string)


def fullmatch(pattern, string, flags=0):
    return _compile(pattern, flags).fullmatch(string)


def search(pattern, string, flags=0):
    return _compile(pattern, flags).search(string)


def findall(pattern, string, flags=0):
    return _compile(pattern, flags).findall(string)


def finditer(pattern, string, flags=0):
    return _compile(pattern, flags).finditer(string)


def sub(pattern, repl, string, count=0, flags=0):
    return _compile(pattern, flags).sub(repl, string, count)


def subn(pattern, repl, string, count=0, flags=0):
    return _compile(pattern, flags).subn(repl, string, count)


def split(pattern, string, maxsplit=0, flags=0):
    return _compile(pattern, flags).split(string, maxsplit)


def purge():
    _cache.clear()


_ALNUM = frozenset("_abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789")


def escape(pattern):
    out = []
    for c in pattern:
        if c in _ALNUM or ord(c) >= 128:
            out.append(c)
        elif c == "\x00":
            out.append("\\000" if False else "\\\x00")
        else:
            out.append("\\" + c)
    return "".join(out)


class Scanner:
    def __init__(self, lexicon, flags=0):
        self.lexicon = lexicon
        self._pats = [(compile(p, flags), action) for p, action in lexicon]

    def scan(self, string):
        result = []
        i = 0
        while True:
            best = None
            for p, action in self._pats:
                m = p.match(string, i)
                if m is not None:
                    best = (m, action)
                    break
            if best is None:
                break
            m, action = best
            j = m.end()
            if j == i:
                break
            if callable(action):
                self.match = m
                action = action(self, m.group())
            if action is not None:
                result.append(action)
            i = j
        return result, string[i:]
