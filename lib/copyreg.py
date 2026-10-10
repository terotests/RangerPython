# SPDX-License-Identifier: MIT
# copyreg for the Ranger Python runtime
dispatch_table = {}
_extension_registry = {}
_inverted_registry = {}
_extension_cache = {}


def pickle(ob_type, pickle_function, constructor_ob=None):
    if not callable(pickle_function):
        raise TypeError("reduction functions must be callable")
    dispatch_table[ob_type] = pickle_function


def constructor(object):
    if not callable(object):
        raise TypeError("constructors must be callable")


def __newobj__(cls, *args):
    return cls.__new__(cls, *args)


def __newobj_ex__(cls, args, kwargs):
    return cls.__new__(cls, *args, **kwargs)


def _reconstructor(cls, base, state):
    obj = object.__new__(cls)
    return obj


def add_extension(module, name, code):
    _extension_registry[(module, name)] = code
    _inverted_registry[code] = (module, name)


def remove_extension(module, name, code):
    del _extension_registry[(module, name)]
    del _inverted_registry[code]


def clear_extension_cache():
    _extension_cache.clear()
