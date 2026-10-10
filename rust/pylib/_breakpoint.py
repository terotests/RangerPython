# SPDX-License-Identifier: MIT
# sys.breakpointhook, after CPython's Python/sysmodule.c
import os
import warnings


def hook(*args, **kws):
    hookname = os.environ.get("PYTHONBREAKPOINT")
    if hookname is None or len(hookname) == 0:
        hookname = "pdb.set_trace"
    elif hookname == "0":
        return None
    modulepath, dot, funcname = hookname.rpartition(".")
    if dot == "":
        modulepath = "builtins"
    try:
        module = __import__(modulepath, fromlist=[funcname])
        hook = getattr(module, funcname)
    except (ImportError, AttributeError):
        warnings.warn('Ignoring unimportable $PYTHONBREAKPOINT: "{}"'.format(hookname), RuntimeWarning)
        return None
    return hook(*args, **kws)
