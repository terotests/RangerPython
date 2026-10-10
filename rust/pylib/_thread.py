# SPDX-License-Identifier: MIT
# _thread for the Rust runtime: there is one thread, so the locks come from
# threading's single-thread versions.
from threading import Lock as LockType, RLock, get_ident

allocate_lock = LockType
error = RuntimeError
TIMEOUT_MAX = 4294967.0


def _count():
    return 0


def stack_size(size=0):
    return 0
