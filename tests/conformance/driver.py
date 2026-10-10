# SPDX-License-Identifier: MIT
#
# Runs one CPython test file test by test and prints a line per test:
#   T <tab> Class.method <tab> pass|fail|error|skip|xfail|xpass <tab> detail
# The same driver runs under CPython (the baseline) and on the Ranger
# runtime, so the two lists compare test for test.
import sys
import unittest

name = sys.argv[1]
budget = getattr(sys, "_ranger_step_budget", None)
STEPS = 4000000


def clean(text):
    return str(text).replace("\t", " ").replace("\n", " ")[:240]


try:
    if budget:
        budget(STEPS * 4)
    mod = __import__(name, fromlist=["_"])
except BaseException as e:
    print("IMPORT\t%s: %s" % (type(e).__name__, clean(e)))
    sys.exit(0)

loader = unittest.TestLoader()
classes = []
for attr in sorted(dir(mod)):
    obj = getattr(mod, attr, None)
    if isinstance(obj, type) and issubclass(obj, unittest.TestCase):
        classes.append((attr, obj))

for attr, cls in classes:
    names = loader.getTestCaseNames(cls)
    if not names:
        continue
    class_problem = None
    try:
        if budget:
            budget(STEPS)
        cls.setUpClass()
    except unittest.SkipTest as e:
        class_problem = ("skip", e)
    except BaseException as e:
        class_problem = ("error", e)
    for method in names:
        label = attr + "." + method
        if class_problem is not None:
            print("T\t%s\t%s\t%s" % (label, class_problem[0], clean("setUpClass: " + type(class_problem[1]).__name__ + ": " + str(class_problem[1]))))
            continue
        if budget:
            budget(STEPS)
        result = unittest.TestResult()
        try:
            cls(method).run(result)
        except BaseException as e:
            print("T\t%s\terror\t%s" % (label, clean("runner: " + type(e).__name__ + ": " + str(e))))
            continue
        if result.errors:
            status, detail = "error", result.errors[0][1]
        elif result.failures:
            status, detail = "fail", result.failures[0][1]
        elif result.skipped:
            status, detail = "skip", result.skipped[0][1]
        elif result.expectedFailures:
            status, detail = "xfail", ""
        elif result.unexpectedSuccesses:
            status, detail = "xpass", ""
        else:
            status, detail = "pass", ""
        # CPython's TestResult keeps formatted tracebacks: the last line is the error
        lines = [l for l in str(detail).strip().split("\n") if l.strip()]
        print("T\t%s\t%s\t%s" % (label, status, clean(lines[-1] if lines else "")))
    if class_problem is None:
        try:
            cls.tearDownClass()
        except BaseException:
            pass
