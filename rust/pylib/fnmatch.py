# SPDX-License-Identifier: MIT
# fnmatch for rpy: shell-style wildcard matching.

import re

__all__ = ["filter", "fnmatch", "fnmatchcase", "translate"]


def translate(pat):
    out = []
    i, n = 0, len(pat)
    while i < n:
        c = pat[i]
        i += 1
        if c == "*":
            if not out or out[-1] != ".*":
                out.append(".*")
        elif c == "?":
            out.append(".")
        elif c == "[":
            j = i
            if j < n and pat[j] == "!":
                j += 1
            if j < n and pat[j] == "]":
                j += 1
            while j < n and pat[j] != "]":
                j += 1
            if j >= n:
                out.append("\\[")
            else:
                stuff = pat[i:j].replace("\\", "\\\\")
                i = j + 1
                if stuff.startswith("!"):
                    stuff = "^" + stuff[1:]
                elif stuff.startswith("^"):
                    stuff = "\\" + stuff
                out.append("[" + stuff + "]")
        else:
            out.append(re.escape(c))
    return "(?s:" + "".join(out) + ")\\Z"


_cache = {}


def _compile(pat):
    rx = _cache.get(pat)
    if rx is None:
        rx = _cache[pat] = re.compile(translate(pat))
    return rx


def fnmatchcase(name, pat):
    return _compile(pat).match(name) is not None


def fnmatch(name, pat):
    return fnmatchcase(name, pat)


def filter(names, pat):
    rx = _compile(pat)
    return [n for n in names if rx.match(n)]
