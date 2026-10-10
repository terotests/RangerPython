# SPDX-License-Identifier: MIT
# linecache for the Ranger Python runtime: source lines read through open().

cache = {}


def getlines(filename, module_globals=None):
    if filename in cache:
        return cache[filename]
    return updatecache(filename, module_globals)


def getline(filename, lineno, module_globals=None):
    lines = getlines(filename, module_globals)
    if 1 <= lineno <= len(lines):
        return lines[lineno - 1]
    return ""


def updatecache(filename, module_globals=None):
    cache.pop(filename, None)
    if not filename or (filename.startswith("<") and filename.endswith(">")):
        return []
    try:
        with open(filename, encoding="utf-8") as f:
            lines = f.readlines()
    except (OSError, UnicodeDecodeError, ValueError):
        return []
    if lines and not lines[-1].endswith("\n"):
        lines[-1] += "\n"
    cache[filename] = lines
    return lines


def clearcache():
    cache.clear()


def checkcache(filename=None):
    pass


def lazycache(filename, module_globals):
    return False
