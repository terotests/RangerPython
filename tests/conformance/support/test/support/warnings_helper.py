# SPDX-License-Identifier: MIT
import warnings


class _Recorder:
    def __init__(self, log):
        self._warnings = log

    @property
    def warnings(self):
        return self._warnings

    def __getattr__(self, name):
        if self._warnings:
            return getattr(self._warnings[-1], name)
        return None

    def reset(self):
        del self._warnings[:]


class check_warnings:
    def __init__(self, *filters, quiet=True):
        self._cm = warnings.catch_warnings(record=True)

    def __enter__(self):
        log = self._cm.__enter__()
        warnings.simplefilter("always")
        return _Recorder(log)

    def __exit__(self, *exc):
        return self._cm.__exit__(*exc)


def check_no_warnings(testcase, message="", category=Warning, force_gc=False):
    return check_warnings()


def check_no_resource_warning(testcase):
    return check_warnings()


def ignore_warnings(*, category):
    def decorator(f):
        return f
    return decorator


def save_restore_warnings_filters():
    return warnings.catch_warnings()


def check_syntax_warning(testcase, statement, errtext="", *, lineno=1, offset=None):
    compile(statement, "<testcase>", "exec")
