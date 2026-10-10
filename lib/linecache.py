# SPDX-License-Identifier: MIT
# linecache for the Ranger Python runtime: source lines read through open().

cache = {}
# lines of code compiled from strings (interactive input), by code key
_interactive_cache = {}


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


def _getline_from_code(filename, lineno):
    lines = _getlines_from_code(filename)
    if 1 <= lineno <= len(lines):
        return lines[lineno - 1]
    return ""


def _make_key(code):
    return (code.co_filename, code.co_qualname, code.co_firstlineno)


def _getlines_from_code(code):
    entry = _interactive_cache.get(_make_key(code))
    if entry is not None and len(entry) != 1:
        return entry[2]
    return []


def _register_code(code, string, name):
    entry = (len(string), None, [line + "\n" for line in string.splitlines()], name)
    stack = [code]
    while stack:
        code = stack.pop()
        for const in code.co_consts:
            if isinstance(const, type(code)):
                stack.append(const)
        _interactive_cache[_make_key(code)] = entry
