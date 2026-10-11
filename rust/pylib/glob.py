# SPDX-License-Identifier: MIT
# glob for rpy: pathname patterns over the in-memory filesystem.

import os
import fnmatch

__all__ = ["glob", "iglob", "escape", "has_magic"]


def has_magic(s):
    return any(c in s for c in "*?[")


def escape(pathname):
    return "".join("[" + c + "]" if c in "*?[" else c for c in pathname)


def glob(pathname, *, root_dir=None, recursive=False, include_hidden=False):
    return list(iglob(pathname, root_dir=root_dir, recursive=recursive, include_hidden=include_hidden))


def iglob(pathname, *, root_dir=None, recursive=False, include_hidden=False):
    absolute = pathname.startswith("/")
    base = "/" if absolute else (root_dir or "")
    parts = [p for p in pathname.split("/") if p]
    trailing = pathname.endswith("/")
    for p in _walk(base, parts, recursive, include_hidden):
        if trailing:
            if os.path.isdir(_join(root_dir, p)):
                yield p + "/"
        else:
            yield p


def _join(root, p):
    if root and not p.startswith("/"):
        return os.path.join(root, p)
    return p


def _list(d, root_dir):
    try:
        return sorted(os.listdir(_join(root_dir, d) if d else (root_dir or ".")))
    except OSError:
        return []


def _walk(base, parts, recursive, hidden, root_dir=None):
    if not parts:
        yield base
        return
    head, rest = parts[0], parts[1:]
    prefix = "" if base == "" else (base if base.endswith("/") else base + "/")
    if recursive and head == "**":
        yield from _walk(base, rest, recursive, hidden, root_dir) if rest else ([prefix] if prefix else [])
        for name in _list(base, root_dir):
            if not hidden and name.startswith("."):
                continue
            p = prefix + name
            if os.path.isdir(p):
                yield from _walk(p, parts, recursive, hidden, root_dir)
            elif not rest:
                yield p
        return
    if not has_magic(head):
        p = prefix + head
        if os.path.exists(p):
            yield from _walk(p, rest, recursive, hidden, root_dir)
        return
    for name in _list(base, root_dir):
        if not hidden and name.startswith(".") and not head.startswith("."):
            continue
        if fnmatch.fnmatchcase(name, head):
            p = prefix + name
            if rest and not os.path.isdir(p):
                continue
            yield from _walk(p, rest, recursive, hidden, root_dir)
