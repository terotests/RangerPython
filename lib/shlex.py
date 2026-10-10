# SPDX-License-Identifier: MIT
# shlex for the Ranger Python runtime: split, quote and join
import re

_find_unsafe = re.compile(r"[^\w@%+=:,./-]", re.ASCII).search


def quote(s):
    if not s:
        return "''"
    if _find_unsafe(s) is None:
        return s
    return "'" + s.replace("'", "'\"'\"'") + "'"


def join(split_command):
    return " ".join(quote(arg) for arg in split_command)


def split(s, comments=False, posix=True):
    if s is None:
        raise ValueError("s argument must not be None")
    out = []
    cur = []
    have = False
    i = 0
    n = len(s)
    while i < n:
        c = s[i]
        if c in " \t\r\n":
            if have:
                out.append("".join(cur))
                cur = []
                have = False
            i += 1
        elif comments and c == "#":
            while i < n and s[i] != "\n":
                i += 1
        elif c == "'":
            j = s.find("'", i + 1)
            if j < 0:
                raise ValueError("No closing quotation")
            cur.append(s[i + 1:j])
            have = True
            i = j + 1
        elif c == '"':
            i += 1
            while True:
                if i >= n:
                    raise ValueError("No closing quotation")
                if s[i] == '"':
                    break
                if s[i] == "\\" and i + 1 < n and s[i + 1] in '"\\$`\n':
                    cur.append(s[i + 1])
                    i += 2
                    continue
                cur.append(s[i])
                i += 1
            have = True
            i += 1
        elif c == "\\" and posix:
            if i + 1 < n:
                cur.append(s[i + 1])
            have = True
            i += 2
        else:
            cur.append(c)
            have = True
            i += 1
    if have:
        out.append("".join(cur))
    return out
