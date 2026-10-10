# Stand-in for CPython's internal test module: objects here keep their
# attributes in a dict, not inline.
def has_inline_values(obj):
    return False


from _rpyinternal import dict_getitem_knownhash
