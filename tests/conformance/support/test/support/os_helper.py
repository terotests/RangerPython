# SPDX-License-Identifier: MIT
import os
TESTFN = "@test_tmp"
TESTFN_ASCII = TESTFN
TESTFN_UNICODE = TESTFN + "-\xe0\xf2"
TESTFN_NONASCII = None
TESTFN_UNENCODABLE = None
FS_NONASCII = ""


def unlink(filename):
    try:
        os.remove(filename)
    except OSError:
        pass


def rmtree(path):
    try:
        for name in os.listdir(path):
            full = os.path.join(path, name)
            if os.path.isdir(full):
                rmtree(full)
            else:
                os.remove(full)
        os.rmdir(path)
    except OSError:
        pass


rmdir = rmtree


class temp_cwd:
    def __init__(self, name="tempcwd", quiet=False):
        self.name = name

    def __enter__(self):
        self.old = os.getcwd()
        os.makedirs("/tmp/" + self.name, exist_ok=True)
        os.chdir("/tmp/" + self.name)
        return os.getcwd()

    def __exit__(self, *exc):
        os.chdir(self.old)
        return False


temp_dir = temp_cwd
change_cwd = temp_cwd


class EnvironmentVarGuard(dict):
    def __enter__(self):
        return self

    def __exit__(self, *exc):
        return False

    def set(self, key, value):
        self[key] = value

    def unset(self, key):
        self.pop(key, None)
