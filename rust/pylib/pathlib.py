# SPDX-License-Identifier: MIT
# pathlib for rpy: PurePath / Path over the in-memory filesystem (POSIX
# paths only). Covers what scripts use to find and read their files.

import os
import re

__all__ = ["PurePath", "PurePosixPath", "Path", "PosixPath"]


def _glob_regex(pattern):
    out = []
    i = 0
    n = len(pattern)
    while i < n:
        c = pattern[i]
        i += 1
        if c == "*":
            out.append("[^/]*")
        elif c == "?":
            out.append("[^/]")
        elif c == "[":
            j = pattern.find("]", i + 1 if i < n and pattern[i] == "!" else i)
            if j < 0:
                out.append("\\[")
            else:
                body = pattern[i:j]
                if body.startswith("!"):
                    body = "^" + body[1:]
                out.append("[" + body.replace("\\", "\\\\") + "]")
                i = j + 1
        else:
            out.append(re.escape(c))
    return re.compile("".join(out) + r"\Z")


class PurePath:
    def __new__(cls, *args):
        if cls is PurePath:
            cls = PurePosixPath
        self = object.__new__(cls)
        parts = []
        for a in args:
            if isinstance(a, PurePath):
                a = a._str
            a = os.fspath(a)
            if a.startswith("/"):
                parts = [a]
            elif a:
                parts.append(a)
        self._str = PurePath._normalize("/".join(parts) if parts else ".")
        return self

    def __init__(self, *args):
        pass

    @staticmethod
    def _normalize(s):
        absolute = s.startswith("/")
        segs = [p for p in s.split("/") if p and p != "."]
        out = ("/" if absolute else "") + "/".join(segs)
        return out or "."

    def __str__(self):
        return self._str

    def __fspath__(self):
        return self._str

    def __repr__(self):
        return "%s(%r)" % (type(self).__name__, self._str)

    def __hash__(self):
        return hash(self._str)

    def __eq__(self, other):
        if not isinstance(other, PurePath):
            return NotImplemented
        return self._str == other._str

    def __lt__(self, other):
        if not isinstance(other, PurePath):
            return NotImplemented
        return self.parts < other.parts

    def __le__(self, other):
        return self == other or self < other

    def __gt__(self, other):
        if not isinstance(other, PurePath):
            return NotImplemented
        return other < self

    def __ge__(self, other):
        return self == other or self > other

    def __truediv__(self, other):
        try:
            return type(self)(self, other)
        except TypeError:
            return NotImplemented

    def __rtruediv__(self, other):
        try:
            return type(self)(other, self)
        except TypeError:
            return NotImplemented

    def joinpath(self, *others):
        return type(self)(self, *others)

    def with_segments(self, *args):
        return type(self)(*args)

    @property
    def parts(self):
        s = self._str
        if s == ".":
            return ()
        if s.startswith("/"):
            return ("/",) + tuple(p for p in s[1:].split("/") if p)
        return tuple(s.split("/"))

    @property
    def drive(self):
        return ""

    @property
    def root(self):
        return "/" if self._str.startswith("/") else ""

    @property
    def anchor(self):
        return self.root

    @property
    def name(self):
        s = self._str
        if s in ("/", "."):
            return ""
        return s.rsplit("/", 1)[-1]

    @property
    def suffix(self):
        name = self.name
        i = name.rfind(".")
        if 0 < i < len(name) - 1:
            return name[i:]
        return ""

    @property
    def suffixes(self):
        name = self.name
        if name.endswith("."):
            return []
        name = name.lstrip(".")
        return ["." + s for s in name.split(".")[1:]]

    @property
    def stem(self):
        name = self.name
        i = name.rfind(".")
        if 0 < i < len(name) - 1:
            return name[:i]
        return name

    @property
    def parent(self):
        s = self._str
        if s in ("/", "."):
            return self
        if "/" not in s:
            return type(self)(".")
        head = s.rsplit("/", 1)[0]
        return type(self)(head or "/")

    @property
    def parents(self):
        out = []
        p = self
        while True:
            q = p.parent
            if q == p:
                break
            out.append(q)
            p = q
        return tuple(out)

    def is_absolute(self):
        return self._str.startswith("/")

    def as_posix(self):
        return self._str

    def as_uri(self):
        if not self.is_absolute():
            raise ValueError("relative path can't be expressed as a file URI")
        return "file://" + self._str

    def with_name(self, name):
        if not self.name:
            raise ValueError("%r has an empty name" % (self,))
        return self.parent / name

    def with_stem(self, stem):
        return self.with_name(stem + self.suffix)

    def with_suffix(self, suffix):
        if suffix and not suffix.startswith("."):
            raise ValueError("Invalid suffix %r" % (suffix,))
        return self.with_name(self.stem + suffix)

    def relative_to(self, other, walk_up=False):
        other = type(self)(other)
        a, b = self.parts, other.parts
        if a[:len(b)] == b:
            rest = a[len(b):]
            return type(self)(*rest) if rest else type(self)(".")
        if walk_up:
            i = 0
            while i < min(len(a), len(b)) and a[i] == b[i]:
                i += 1
            return type(self)(*([".."] * (len(b) - i) + list(a[i:])))
        raise ValueError("%r is not in the subpath of %r" % (self._str, other._str))

    def is_relative_to(self, other):
        try:
            self.relative_to(other)
            return True
        except ValueError:
            return False

    def match(self, pattern):
        pat = PurePosixPath(pattern)
        pparts = pat.parts
        parts = self.parts
        if pat.is_absolute():
            if len(pparts) != len(parts):
                return False
        elif len(pparts) > len(parts):
            return False
        for p, s in zip(reversed(pparts), reversed(parts)):
            if not _glob_regex(p).match(s):
                return False
        return True

    full_match = match


class PurePosixPath(PurePath):
    pass


class Path(PurePath):
    def __new__(cls, *args, **kwargs):
        if cls is Path:
            cls = PosixPath
        return PurePath.__new__(cls, *args)

    @classmethod
    def cwd(cls):
        return cls(os.getcwd())

    @classmethod
    def home(cls):
        return cls(os.path.expanduser("~"))

    def absolute(self):
        if self.is_absolute():
            return self
        return type(self)(os.getcwd(), self)

    def resolve(self, strict=False):
        return type(self)(os.path.realpath(os.path.abspath(self._str)))

    def expanduser(self):
        return type(self)(os.path.expanduser(self._str))

    def stat(self):
        return os.stat(self._str)

    def exists(self):
        return os.path.exists(self._str)

    def is_file(self):
        return os.path.isfile(self._str)

    def is_dir(self):
        return os.path.isdir(self._str)

    def is_symlink(self):
        return False

    def open(self, mode="r", buffering=-1, encoding=None, errors=None, newline=None):
        if "b" in mode:
            return open(self._str, mode)
        return open(self._str, mode, encoding=encoding, errors=errors, newline=newline)

    def read_text(self, encoding=None, errors=None, newline=None):
        with self.open("r", encoding=encoding, errors=errors, newline=newline) as f:
            return f.read()

    def read_bytes(self):
        with self.open("rb") as f:
            return f.read()

    def write_text(self, data, encoding=None, errors=None, newline=None):
        if not isinstance(data, str):
            raise TypeError("data must be str, not %s" % type(data).__name__)
        with self.open("w", encoding=encoding, errors=errors, newline=newline) as f:
            return f.write(data)

    def write_bytes(self, data):
        data = bytes(memoryview(data)) if not isinstance(data, bytes) else data
        with self.open("wb") as f:
            return f.write(data)

    def touch(self, mode=0o666, exist_ok=True):
        if self.exists():
            if not exist_ok:
                raise FileExistsError(17, "File exists", self._str)
            return
        with self.open("w"):
            pass

    def mkdir(self, mode=0o777, parents=False, exist_ok=False):
        if self.is_dir():
            if exist_ok:
                return
            raise FileExistsError(17, "File exists", self._str)
        if parents:
            os.makedirs(self._str, exist_ok=exist_ok)
        else:
            os.mkdir(self._str)

    def unlink(self, missing_ok=False):
        try:
            os.remove(self._str)
        except FileNotFoundError:
            if not missing_ok:
                raise

    def rmdir(self):
        os.rmdir(self._str)

    def rename(self, target):
        os.rename(self._str, str(target))
        return type(self)(target)

    def replace(self, target):
        os.replace(self._str, str(target))
        return type(self)(target)

    def iterdir(self):
        for name in sorted(os.listdir(self._str)):
            yield self / name

    def _walk_all(self):
        for name in sorted(os.listdir(self._str)):
            child = self / name
            yield child
            if child.is_dir():
                yield from child._walk_all()

    def glob(self, pattern, case_sensitive=None):
        pparts = PurePosixPath(pattern).parts
        if not pparts:
            raise ValueError("Unacceptable pattern: %r" % (pattern,))
        yield from self._glob(pparts)

    def _glob(self, pparts):
        if not self.is_dir():
            return
        head, rest = pparts[0], pparts[1:]
        if head == "**":
            candidates = [self] + [p for p in self._walk_all() if p.is_dir()]
            if not rest:
                yield from candidates
                return
            seen = set()
            for d in candidates:
                for p in d._glob(rest):
                    if p._str not in seen:
                        seen.add(p._str)
                        yield p
            return
        rx = _glob_regex(head)
        for name in sorted(os.listdir(self._str)):
            if rx.match(name) and (head.startswith(".") or not name.startswith(".")):
                child = self / name
                if not rest:
                    yield child
                elif child.is_dir():
                    yield from child._glob(rest)

    def rglob(self, pattern, case_sensitive=None):
        return self.glob("**/" + pattern)

    def walk(self, top_down=True, on_error=None, follow_symlinks=False):
        for root, dirs, files in os.walk(self._str, topdown=top_down):
            yield type(self)(root), dirs, files

    def samefile(self, other):
        return self.resolve() == type(self)(other).resolve()


class PosixPath(Path, PurePosixPath):
    pass
