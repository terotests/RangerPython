# SPDX-License-Identifier: MIT
import sys
import unittest


def import_module(name, deprecated=False, *, required_on=()):
    try:
        return __import__(name, fromlist=["_"])
    except ImportError as e:
        raise unittest.SkipTest(str(e))


def _save_and_remove_modules(names):
    orig_modules = {}
    prefixes = tuple(name + '.' for name in names)
    for modname in list(sys.modules):
        if modname in names or modname.startswith(prefixes):
            orig_modules[modname] = sys.modules.pop(modname)
    return orig_modules


def import_fresh_module(name, fresh=(), blocked=(), *, deprecated=False, usefrozen=False):
    # as CPython's: import `name` anew, with the `fresh` modules imported
    # anew too and the `blocked` ones unimportable, then restore sys.modules
    fresh = list(fresh)
    blocked = list(blocked)
    names = {name, *fresh, *blocked}
    orig_modules = _save_and_remove_modules(names)
    for modname in blocked:
        sys.modules[modname] = None
    try:
        try:
            for modname in fresh:
                __import__(modname)
        except ImportError:
            return None
        __import__(name)
        return sys.modules[name]
    finally:
        _save_and_remove_modules(names)
        sys.modules.update(orig_modules)


class CleanImport:
    def __init__(self, *module_names, usefrozen=False):
        pass

    def __enter__(self):
        return self

    def __exit__(self, *exc):
        return False


class DirsOnSysPath(CleanImport):
    pass


def unload(name):
    pass


def forget(name):
    pass


class ready_to_import:
    """a module NAME made from SOURCE in a fresh directory on sys.path"""

    def __init__(self, name=None, source=""):
        self.name = name or "spam"
        self.source = source

    def __enter__(self):
        import os
        self.dir = "/tmp/ready_to_import_%d" % id(self)
        os.makedirs(self.dir, exist_ok=True)
        self.path = os.path.join(self.dir, self.name + ".py")
        with open(self.path, "w") as f:
            f.write(self.source)
        sys.path.insert(0, self.dir)
        self.old = sys.modules.pop(self.name, None)
        return self.name, self.path

    def __exit__(self, *exc):
        import os
        sys.path.remove(self.dir)
        sys.modules.pop(self.name, None)
        if self.old is not None:
            sys.modules[self.name] = self.old
        try:
            os.remove(self.path)
            os.rmdir(self.dir)
        except OSError:
            pass
        return False
