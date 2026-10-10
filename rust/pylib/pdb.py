# SPDX-License-Identifier: MIT
# pdb for the Rust runtime: there is no terminal to debug from, so
# set_trace() returns at once.


class Pdb:
    def __init__(self, *args, **kwargs):
        pass

    def set_trace(self, frame=None, *, commands=None):
        pass


def set_trace(*, header=None, commands=None):
    pass


def run(statement, globals=None, locals=None):
    exec(statement, globals, locals)


def runcall(func, *args, **kwds):
    return func(*args, **kwds)


def post_mortem(t=None):
    pass


def pm():
    pass
