# SPDX-License-Identifier: MIT
import sys
import unittest


def import_module(name, deprecated=False, *, required_on=()):
    try:
        return __import__(name, fromlist=["_"])
    except ImportError as e:
        raise unittest.SkipTest(str(e))


def import_fresh_module(name, fresh=(), blocked=(), deprecated=False):
    # there is one implementation of each module here, so a blocked
    # accelerator gives the same module
    try:
        return __import__(name, fromlist=["_"])
    except ImportError:
        return None


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
