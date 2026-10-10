# SPDX-License-Identifier: MIT
# _weakref for the Rust runtime: references here are strong (see weakref).
from weakref import ref, proxy, ReferenceType, ProxyType, CallableProxyType


def getweakrefcount(obj):
    return 0


def getweakrefs(obj):
    return []
