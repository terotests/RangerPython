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


class EnvironmentVarGuard:
    # after CPython's: changes to os.environ are undone on exit
    def __init__(self):
        self._environ = os.environ
        self._changed = {}

    def __getitem__(self, envvar):
        return self._environ[envvar]

    def __setitem__(self, envvar, value):
        if envvar not in self._changed:
            self._changed[envvar] = self._environ.get(envvar)
        self._environ[envvar] = value

    def __delitem__(self, envvar):
        if envvar not in self._changed:
            self._changed[envvar] = self._environ.get(envvar)
        if envvar in self._environ:
            del self._environ[envvar]

    def __contains__(self, envvar):
        return envvar in self._environ

    def get(self, envvar, default=None):
        return self._environ.get(envvar, default)

    def keys(self):
        return self._environ.keys()

    def __iter__(self):
        return iter(self._environ)

    def __len__(self):
        return len(self._environ)

    def set(self, envvar, value):
        self[envvar] = value

    def unset(self, envvar):
        del self[envvar]

    def copy(self):
        return dict(self._environ)

    def __enter__(self):
        return self

    def __exit__(self, *ignore_exc):
        for (k, v) in self._changed.items():
            if v is None:
                if k in self._environ:
                    del self._environ[k]
            else:
                self._environ[k] = v
        os.environ = self._environ
