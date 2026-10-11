# SPDX-License-Identifier: MIT
# _datetime: there is no C datetime here, only the capsule C code would
# find its API in (on a runtime that has capsules)
try:
    from _rpyinternal import capsule as _capsule
except ImportError:
    datetime_CAPI = None
else:
    datetime_CAPI = _capsule("datetime.datetime_CAPI")
    del _capsule
