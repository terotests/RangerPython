# SPDX-License-Identifier: MIT
# shutil for the Ranger Python runtime, on its file system
import os


class Error(OSError):
    pass


def copyfile(src, dst, *, follow_symlinks=True):
    with open(src) as f:
        data = f.read()
    with open(dst, "w") as f:
        f.write(data)
    return dst


def copy(src, dst, *, follow_symlinks=True):
    if os.path.isdir(dst):
        dst = os.path.join(dst, os.path.basename(src))
    return copyfile(src, dst)


copy2 = copy


def copymode(src, dst, *, follow_symlinks=True):
    pass


copystat = copymode


def rmtree(path, ignore_errors=False, onerror=None, *, onexc=None, dir_fd=None):
    try:
        for name in os.listdir(path):
            full = os.path.join(path, name)
            if os.path.isdir(full):
                rmtree(full, ignore_errors)
            else:
                os.remove(full)
        if hasattr(os, "rmdir"):
            os.rmdir(path)
    except OSError:
        if not ignore_errors:
            raise


def move(src, dst):
    copyfile(src, dst)
    os.remove(src)
    return dst


def which(cmd, mode=None, path=None):
    return None


def get_terminal_size(fallback=(80, 24)):
    return os.terminal_size(fallback) if hasattr(os, "terminal_size") else fallback


def disk_usage(path):
    return (0, 0, 0)
