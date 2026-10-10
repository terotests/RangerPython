# SPDX-License-Identifier: MIT
# traceback for the Ranger Python runtime: the exception's type and message.
import sys

def format_exception_only(exc, value=None):
    if value is None and isinstance(exc, BaseException):
        value = exc
    msg = str(value)
    name = type(value).__name__
    return [name + (": " + msg if msg else "") + "\n"]

def format_exception(exc, value=None, tb=None, limit=None, chain=True):
    return ["Traceback (most recent call last):\n"] + format_exception_only(exc, value)

def format_exc(limit=None, chain=True):
    return "NoneType: None\n"

def print_exc(limit=None, file=None, chain=True):
    print(format_exc(), end="", file=file)

def print_exception(exc, value=None, tb=None, limit=None, file=None, chain=True):
    print("".join(format_exception(exc, value)), end="", file=file)

def format_stack(f=None, limit=None):
    return []

def extract_stack(f=None, limit=None):
    return []
