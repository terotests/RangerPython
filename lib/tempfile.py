# SPDX-License-Identifier: MIT
# tempfile for the Ranger Python runtime, on its file system
import os

tempdir = "/tmp"
template = "tmp"
_count = [0]


def gettempdir():
    return tempdir


def _name(prefix, suffix, dir):
    _count[0] += 1
    return os.path.join(dir or tempdir, "%s%d%s" % (prefix or template, _count[0], suffix or ""))


def mkdtemp(suffix=None, prefix=None, dir=None):
    path = _name(prefix, suffix, dir)
    os.makedirs(path, exist_ok=True)
    return path


def mkstemp(suffix=None, prefix=None, dir=None, text=False):
    path = _name(prefix, suffix, dir)
    with open(path, "w") as f:
        f.write("")
    return (3, path)


def mktemp(suffix="", prefix=template, dir=None):
    return _name(prefix, suffix, dir)


class TemporaryDirectory:
    def __init__(self, suffix=None, prefix=None, dir=None, ignore_cleanup_errors=False, *, delete=True):
        self.name = mkdtemp(suffix, prefix, dir)

    def __enter__(self):
        return self.name

    def __exit__(self, *exc):
        self.cleanup()

    def cleanup(self):
        import shutil
        shutil.rmtree(self.name, ignore_errors=True)


def NamedTemporaryFile(mode="w+b", buffering=-1, encoding=None, newline=None, suffix=None,
                       prefix=None, dir=None, delete=True, *, errors=None, delete_on_close=True):
    path = _name(prefix, suffix, dir)
    f = open(path, "w")
    return f


TemporaryFile = NamedTemporaryFile
