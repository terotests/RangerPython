# SPDX-License-Identifier: MIT
# warnings for the Ranger Python runtime: warnings are collected, not printed,
# unless an "error" filter turns them into exceptions.

_filters = []
_log = None


class WarningMessage:
    def __init__(self, message, category, filename="", lineno=0):
        self.message = message
        self.category = category
        self.filename = filename
        self.lineno = lineno


def warn(message, category=None, stacklevel=1, source=None, skip_file_prefixes=()):
    if isinstance(message, Warning):
        category = type(message)
    if category is None:
        category = UserWarning
    filename, lineno = "", 0
    if _log is not None:
        try:
            import sys
            f = sys._getframe(stacklevel)
            filename, lineno = f.f_code.co_filename, f.f_lineno
        except Exception:
            pass
    warn_explicit(message, category, filename, lineno)


def warn_explicit(message, category, filename, lineno, module=None, registry=None, module_globals=None, source=None):
    if isinstance(message, Warning):
        category = type(message)
    for action, cat in _filters:
        if issubclass(category, cat):
            if action == "error":
                raise message if isinstance(message, Warning) else category(message)
            if action == "ignore":
                return
            break
    if _log is not None:
        msg = message if isinstance(message, Warning) else category(message)
        _log.append(WarningMessage(msg, category, filename, lineno))


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


class catch_warnings:
    def __init__(self, *, record=False, module=None, action=None, category=Warning, lineno=0, append=False):
        self._record = record
        self._action = action
        self._category = category

    def __enter__(self):
        global _log
        self._saved = (list(_filters), _log)
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
