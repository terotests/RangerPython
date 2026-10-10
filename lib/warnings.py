# SPDX-License-Identifier: MIT
# warnings for the Ranger Python runtime: the filters decide what happens to
# a warning, as in CPython; one that is shown goes to sys.stderr, or to the
# list catch_warnings(record=True) collects.

_filters = []
_log = None
# the filters CPython starts with, consulted after the ones a program sets
_defaults = [("default", DeprecationWarning, "__main__"), ("ignore", DeprecationWarning, None),
             ("ignore", PendingDeprecationWarning, None), ("ignore", ImportWarning, None),
             ("ignore", ResourceWarning, None)]
# the warnings already shown once, for the "default", "module" and "once" actions
_shown = set()
_ACTIONS = {"e": "error", "i": "ignore", "a": "always", "d": "default", "m": "module", "o": "once"}


def _init_options():
    # -W options and -X dev, as CPython's _processoptions
    import sys
    if getattr(sys.flags, "dev_mode", False):
        _defaults[:] = [("default", Warning, None)]
    for arg in getattr(sys, "warnoptions", ()):
        parts = [p.strip() for p in arg.split(":")]
        action = _ACTIONS.get(parts[0], parts[0]) if parts[0] else "default"
        category = Warning
        if len(parts) > 2 and parts[2]:
            import builtins
            category = getattr(builtins, parts[2], Warning)
        _filters.insert(0, (action, category))


class WarningMessage:
    def __init__(self, message, category, filename="", lineno=0, file=None, line=None, source=None):
        self.message = message
        self.category = category
        self.filename = filename
        self.lineno = lineno
        self.file = file
        self.line = line
        self.source = source
        self._category_name = category.__name__ if category else None

    def __str__(self):
        return ("{message : %r, category : %r, filename : %r, lineno : %s, "
                "line : %r}" % (self.message, self._category_name, self.filename, self.lineno, self.line))


def warn(message, category=None, stacklevel=1, source=None, skip_file_prefixes=()):
    if isinstance(message, Warning):
        category = type(message)
    if category is None:
        category = UserWarning
    filename, lineno, module = "sys", 1, None
    try:
        import sys
        f = sys._getframe(stacklevel)
        filename, lineno = f.f_code.co_filename, f.f_lineno
        module = f.f_globals.get("__name__")
    except Exception:
        pass
    warn_explicit(message, category, filename, lineno, module)


def _action(category, module):
    for action, cat in _filters:
        if issubclass(category, cat):
            return action
    for action, cat, mod in _defaults:
        if issubclass(category, cat) and (mod is None or mod == module):
            return action
    return "default"


def warn_explicit(message, category, filename, lineno, module=None, registry=None, module_globals=None, source=None):
    if isinstance(message, Warning):
        category = type(message)
    if module is None and isinstance(filename, str):
        module = filename[:-3] if filename.endswith(".py") else filename
    action = _action(category, module)
    if action == "error":
        raise message if isinstance(message, Warning) else category(message)
    if action == "ignore":
        return
    msg = message if isinstance(message, Warning) else category(message)
    if _log is None:
        key = {"once": (str(msg), category), "module": (str(msg), category, filename)}.get(
            action, (str(msg), category, filename, lineno))
        if action != "always":
            if key in _shown:
                return
            _shown.add(key)
    _showwarnmsg(WarningMessage(msg, category, filename, lineno, source=source))


def _showwarnmsg(msg):
    if _log is not None:
        _log.append(msg)
        return
    showwarning(msg.message, msg.category, msg.filename, msg.lineno)


def showwarning(message, category, filename, lineno, file=None, line=None):
    import sys
    if file is None:
        file = sys.stderr
        if file is None:
            return
    try:
        file.write(formatwarning(message, category, filename, lineno, line))
    except OSError:
        pass


def formatwarning(message, category, filename, lineno, line=None):
    s = "%s:%s: %s: %s\n" % (filename, lineno, category.__name__, message)
    if line is None:
        try:
            import linecache
            line = linecache.getline(filename, lineno)
        except Exception:
            line = None
    if line:
        s += "  %s\n" % line.strip()
    return s


def simplefilter(action, category=Warning, lineno=0, append=False):
    entry = (action, category)
    if append:
        _filters.append(entry)
    else:
        _filters.insert(0, entry)


def filterwarnings(action, message="", category=Warning, module="", lineno=0, append=False):
    simplefilter(action, category, lineno, append)


def resetwarnings():
    del _filters[:]
    del _defaults[:]


class catch_warnings:
    def __init__(self, *, record=False, module=None, action=None, category=Warning, lineno=0, append=False):
        self._record = record
        self._action = action
        self._category = category

    def __enter__(self):
        global _log
        self._saved = (list(_filters), _log, list(_defaults))
        if self._action is not None:
            simplefilter(self._action, self._category)
        if self._record:
            _log = []
            return _log
        return None

    def __exit__(self, *exc):
        global _log
        _filters[:] = self._saved[0]
        _log = self._saved[1]
        _defaults[:] = self._saved[2]
        return False


_DEPRECATED_MSG = "{name!r} is deprecated and slated for removal in Python {remove}"


def _deprecated(name, message=_DEPRECATED_MSG, *, remove, _version=None):
    """Warn that *name* is deprecated or should be removed (CPython's helper)."""
    import sys
    if _version is None:
        _version = sys.version_info
    remove_formatted = f"{remove[0]}.{remove[1]}"
    if (_version[:2] > remove) or (_version[:2] == remove and _version[3] != "alpha"):
        msg = f"{name!r} was slated for removal after Python {remove_formatted} alpha"
        raise RuntimeError(msg)
    else:
        msg = message.format(name=name, remove=remove_formatted)
        warn(msg, DeprecationWarning, stacklevel=3)


_init_options()
