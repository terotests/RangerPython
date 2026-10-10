# SPDX-License-Identifier: MIT
import unittest


def _no(*args, **kwargs):
    raise unittest.SkipTest("no subprocesses on this runtime")


assert_python_ok = _no
assert_python_failure = _no
spawn_python = _no
run_python_until_end = _no
make_script = _no
