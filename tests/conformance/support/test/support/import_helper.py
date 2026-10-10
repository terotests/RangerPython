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
