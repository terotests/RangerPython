# SPDX-License-Identifier: MIT
# _thread on OS threads under the GIL: the native _rpythread, and _local,
# whose attributes are the running thread's, as Modules/_threadmodule.c's.
from _rpythread import *
from _rpythread import (_count, _get_main_thread_ident, _is_main_interpreter,
                        _make_thread_handle, _shutdown, _ThreadHandle, _local_switch)


class _local:
    """Thread-local data"""
    __slots__ = ('_local__impl', '__dict__')

    def __new__(cls, /, *args, **kw):
        if (args or kw) and (cls.__init__ is object.__init__):
            raise TypeError("Initialization arguments are not supported")
        self = object.__new__(cls)
        object.__setattr__(self, '_local__impl', [get_ident(), {}, args, kw])
        return self

    def __getattribute__(self, name):
        _enter(self)
        return object.__getattribute__(self, name)

    def __setattr__(self, name, value):
        if name == '__dict__':
            raise AttributeError(
                "%r object attribute '__dict__' is read-only"
                % self.__class__.__name__)
        _enter(self)
        return object.__setattr__(self, name, value)

    def __delattr__(self, name):
        if name == '__dict__':
            raise AttributeError(
                "%r object attribute '__dict__' is read-only"
                % self.__class__.__name__)
        _enter(self)
        return object.__delattr__(self, name)


_local.__module__ = '_thread'


def _enter(self):
    impl = object.__getattribute__(self, '_local__impl')
    if _local_switch(self, impl):
        # the first use in this thread: __init__ runs again for it
        type(self).__init__(self, *impl[2], **impl[3])
