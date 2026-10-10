# SPDX-License-Identifier: MIT
# doctest stand-in for the Ranger Python runtime: the names tests import,
# with no examples run.
import unittest

OPTIONFLAGS_BY_NAME = {}
ELLIPSIS = 8
NORMALIZE_WHITESPACE = 4
IGNORE_EXCEPTION_DETAIL = 32
DONT_ACCEPT_TRUE_FOR_1 = 1


def DocTestSuite(*args, **kwargs):
    return unittest.TestSuite()


def DocFileSuite(*args, **kwargs):
    return unittest.TestSuite()


def testmod(*args, **kwargs):
    return (0, 0)
