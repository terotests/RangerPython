# SPDX-License-Identifier: MIT
# platform for the Ranger Python runtime
import sys


def python_implementation():
    return "RangerPython"


def python_version():
    return "%d.%d.%d" % sys.version_info[:3]


def python_version_tuple():
    return tuple(str(x) for x in sys.version_info[:3])


def system():
    return ""


def machine():
    return ""


def node():
    return ""


def release():
    return ""


def version():
    return ""


def platform(aliased=False, terse=False):
    return "ranger"


def architecture(executable=None, bits="", linkage=""):
    return ("64bit", "")


def uname():
    return ("", "", "", "", "", "")


def libc_ver(*args, **kw):
    return ("", "")


def mac_ver(*args, **kw):
    return ("", ("", "", ""), "")


def win32_ver(*args, **kw):
    return ("", "", "", "")


def python_compiler():
    return "Ranger"
