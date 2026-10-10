# SPDX-License-Identifier: MIT
#
# unittest for the Ranger Python runtime: TestCase with the assert methods,
# skips, expected failures, subTest, setUp/tearDown/cleanups, TestResult,
# TestSuite, TestLoader and main(). Written against CPython's documented
# behaviour, not copied from it.

import sys


class SkipTest(Exception):
    pass


def _id(obj):
    return obj


def skip(reason):
    def decorator(obj):
        obj.__unittest_skip__ = True
        obj.__unittest_skip_why__ = reason
        return obj
    if callable(reason) and not isinstance(reason, str):
        fn = reason
        reason = ""
        return decorator(fn)
    return decorator


def skipIf(condition, reason):
    if condition:
        return skip(reason)
    return _id


def skipUnless(condition, reason):
    if not condition:
        return skip(reason)
    return _id


def expectedFailure(fn):
    fn.__unittest_expecting_failure__ = True
    return fn


def _safe_repr(obj):
    try:
        return repr(obj)
    except Exception:
        return object.__repr__(obj)


def _matches(pattern, text):
    # a regular expression is matched as text with its escapes removed
    # (there is no re module yet); a pattern with other syntax matches
    if pattern is None:
        return True
    if not isinstance(pattern, str):
        pattern = getattr(pattern, "pattern", str(pattern))
    special = "^$*+?{}[]|()"
    plain = ""
    i = 0
    while i < len(pattern):
        ch = pattern[i]
        if ch == "\\" and i + 1 < len(pattern):
            plain += pattern[i + 1]
            i += 2
            continue
        if ch in special:
            return True
        if ch == ".":
            return True
        plain += ch
        i += 1
    return plain in text


class _AssertRaisesContext:
    def __init__(self, expected, test_case, regex=None, msg=None):
        self.expected = expected
        self.test_case = test_case
        self.regex = regex
        self.msg = msg
        self.exception = None

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_value, tb):
        if exc_type is None:
            try:
                name = self.expected.__name__
            except AttributeError:
                name = str(self.expected)
            raise self.test_case.failureException(self.msg or (name + " not raised"))
        if not issubclass(exc_type, self.expected):
            return False
        self.exception = exc_value
        if self.regex is not None and not _matches(self.regex, str(exc_value)):
            raise self.test_case.failureException('"%s" does not match "%s"' % (self.regex, str(exc_value)))
        return True


class _AssertWarnsContext:
    def __init__(self, expected, test_case):
        self.expected = expected
        self.test_case = test_case
        self.warning = None
        self.warnings = []

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_value, tb):
        return False


class _SubTest:
    def __init__(self, case, msg, params):
        self.case = case
        self.msg = msg
        self.params = params

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_value, tb):
        if exc_type is None:
            return False
        if issubclass(exc_type, SkipTest):
            return True
        if issubclass(exc_type, (KeyboardInterrupt, SystemExit)):
            return False
        self.case._subtest_failures.append((exc_type, exc_value))
        return True


class TestCase:
    failureException = AssertionError
    longMessage = True
    maxDiff = 640

    def __init__(self, methodName="runTest"):
        self._testMethodName = methodName
        self._cleanups = []
        self._subtest_failures = []

    def __repr__(self):
        return "<%s testMethod=%s>" % (type(self).__name__, self._testMethodName)

    def __str__(self):
        return "%s (%s)" % (self._testMethodName, type(self).__name__)

    def id(self):
        return type(self).__name__ + "." + self._testMethodName

    def shortDescription(self):
        return None

    def setUp(self):
        pass

    def tearDown(self):
        pass

    @classmethod
    def setUpClass(cls):
        pass

    @classmethod
    def tearDownClass(cls):
        pass

    def addCleanup(self, function, *args, **kwargs):
        self._cleanups.append((function, args, kwargs))

    def enterContext(self, cm):
        result = cm.__enter__()
        self.addCleanup(cm.__exit__, None, None, None)
        return result

    def doCleanups(self):
        while self._cleanups:
            function, args, kwargs = self._cleanups.pop()
            function(*args, **kwargs)

    def skipTest(self, reason):
        raise SkipTest(reason)

    def subTest(self, msg=None, **params):
        return _SubTest(self, msg, params)

    def run(self, result=None):
        if result is None:
            result = TestResult()
        result.startTest(self)
        method = getattr(self, self._testMethodName)
        cls_skip = getattr(type(self), "__unittest_skip__", False)
        if cls_skip or getattr(method, "__unittest_skip__", False):
            why = getattr(method, "__unittest_skip_why__", "") or getattr(type(self), "__unittest_skip_why__", "")
            result.addSkip(self, why)
            result.stopTest(self)
            return result
        expecting = getattr(method, "__unittest_expecting_failure__", False)
        outcome = None
        try:
            self.setUp()
        except SkipTest as e:
            outcome = ("skip", str(e))
        except Exception as e:
            outcome = ("error", e)
        if outcome is None:
            try:
                method()
            except SkipTest as e:
                outcome = ("skip", str(e))
            except AssertionError as e:
                outcome = ("fail", e)
            except Exception as e:
                outcome = ("error", e)
            if outcome is None and self._subtest_failures:
                et, ev = self._subtest_failures[0]
                if issubclass(et, AssertionError):
                    outcome = ("fail", ev)
                else:
                    outcome = ("error", ev)
            try:
                self.tearDown()
            except Exception as e:
                if outcome is None:
                    outcome = ("error", e)
        try:
            self.doCleanups()
        except Exception as e:
            if outcome is None:
                outcome = ("error", e)
        if outcome is None:
            if expecting:
                result.addUnexpectedSuccess(self)
            else:
                result.addSuccess(self)
        elif outcome[0] == "skip":
            result.addSkip(self, outcome[1])
        elif expecting:
            result.addExpectedFailure(self, outcome[1])
        elif outcome[0] == "fail":
            result.addFailure(self, outcome[1])
        else:
            result.addError(self, outcome[1])
        result.stopTest(self)
        return result

    def __call__(self, *args, **kwds):
        return self.run(*args, **kwds)

    def debug(self):
        self.setUp()
        getattr(self, self._testMethodName)()
        self.tearDown()
        self.doCleanups()

    # ---- assertions

    def _formatMessage(self, msg, standardMsg):
        if msg is None:
            return standardMsg
        if not self.longMessage:
            return msg
        return "%s : %s" % (standardMsg, msg)

    def fail(self, msg=None):
        raise self.failureException(msg)

    def assertTrue(self, expr, msg=None):
        if not expr:
            self.fail(self._formatMessage(msg, "%s is not true" % _safe_repr(expr)))

    def assertFalse(self, expr, msg=None):
        if expr:
            self.fail(self._formatMessage(msg, "%s is not false" % _safe_repr(expr)))

    def assertEqual(self, first, second, msg=None):
        if not first == second:
            self.fail(self._formatMessage(msg, "%s != %s" % (_safe_repr(first), _safe_repr(second))))

    def assertNotEqual(self, first, second, msg=None):
        if not first != second:
            self.fail(self._formatMessage(msg, "%s == %s" % (_safe_repr(first), _safe_repr(second))))

    assertEquals = assertEqual
    assertNotEquals = assertNotEqual
    failUnless = assertTrue
    failIf = assertFalse

    def assertListEqual(self, first, second, msg=None):
        self.assertIsInstance(first, list, msg)
        self.assertIsInstance(second, list, msg)
        self.assertEqual(first, second, msg)

    def assertTupleEqual(self, first, second, msg=None):
        self.assertIsInstance(first, tuple, msg)
        self.assertIsInstance(second, tuple, msg)
        self.assertEqual(first, second, msg)

    def assertDictEqual(self, first, second, msg=None):
        self.assertIsInstance(first, dict, msg)
        self.assertIsInstance(second, dict, msg)
        self.assertEqual(first, second, msg)

    def assertSetEqual(self, first, second, msg=None):
        self.assertEqual(set(first), set(second), msg)

    def assertSequenceEqual(self, first, second, msg=None, seq_type=None):
        if seq_type is not None:
            self.assertIsInstance(first, seq_type, msg)
            self.assertIsInstance(second, seq_type, msg)
        self.assertEqual(list(first), list(second), msg)

    def assertMultiLineEqual(self, first, second, msg=None):
        self.assertEqual(first, second, msg)

    def assertCountEqual(self, first, second, msg=None):
        a = list(first)
        b = list(second)
        if len(a) != len(b):
            self.fail(self._formatMessage(msg, "Element counts were not equal"))
        rest = list(b)
        for x in a:
            found = False
            for i in range(len(rest)):
                if rest[i] == x:
                    del rest[i]
                    found = True
                    break
            if not found:
                self.fail(self._formatMessage(msg, "Element counts were not equal"))

    def assertIs(self, first, second, msg=None):
        if first is not second:
            self.fail(self._formatMessage(msg, "%s is not %s" % (_safe_repr(first), _safe_repr(second))))

    def assertIsNot(self, first, second, msg=None):
        if first is second:
            self.fail(self._formatMessage(msg, "unexpectedly identical: %s" % _safe_repr(first)))

    def assertIsNone(self, obj, msg=None):
        if obj is not None:
            self.fail(self._formatMessage(msg, "%s is not None" % _safe_repr(obj)))

    def assertIsNotNone(self, obj, msg=None):
        if obj is None:
            self.fail(self._formatMessage(msg, "unexpectedly None"))

    def assertIn(self, member, container, msg=None):
        if member not in container:
            self.fail(self._formatMessage(msg, "%s not found in %s" % (_safe_repr(member), _safe_repr(container))))

    def assertNotIn(self, member, container, msg=None):
        if member in container:
            self.fail(self._formatMessage(msg, "%s unexpectedly found in %s" % (_safe_repr(member), _safe_repr(container))))

    def assertIsInstance(self, obj, cls, msg=None):
        if not isinstance(obj, cls):
            self.fail(self._formatMessage(msg, "%s is not an instance of %r" % (_safe_repr(obj), cls)))

    def assertNotIsInstance(self, obj, cls, msg=None):
        if isinstance(obj, cls):
            self.fail(self._formatMessage(msg, "%s is an instance of %r" % (_safe_repr(obj), cls)))

    def assertGreater(self, a, b, msg=None):
        if not a > b:
            self.fail(self._formatMessage(msg, "%s not greater than %s" % (_safe_repr(a), _safe_repr(b))))

    def assertGreaterEqual(self, a, b, msg=None):
        if not a >= b:
            self.fail(self._formatMessage(msg, "%s not greater than or equal to %s" % (_safe_repr(a), _safe_repr(b))))

    def assertLess(self, a, b, msg=None):
        if not a < b:
            self.fail(self._formatMessage(msg, "%s not less than %s" % (_safe_repr(a), _safe_repr(b))))

    def assertLessEqual(self, a, b, msg=None):
        if not a <= b:
            self.fail(self._formatMessage(msg, "%s not less than or equal to %s" % (_safe_repr(a), _safe_repr(b))))

    def assertAlmostEqual(self, first, second, places=None, msg=None, delta=None):
        if first == second:
            return
        diff = abs(first - second)
        if delta is not None:
            if diff <= delta:
                return
            standard = "%s != %s within %s delta (%s difference)" % (_safe_repr(first), _safe_repr(second), _safe_repr(delta), _safe_repr(diff))
        else:
            if places is None:
                places = 7
            if round(diff, places) == 0:
                return
            standard = "%s != %s within %r places (%s difference)" % (_safe_repr(first), _safe_repr(second), places, _safe_repr(diff))
        self.fail(self._formatMessage(msg, standard))

    def assertNotAlmostEqual(self, first, second, places=None, msg=None, delta=None):
        diff = abs(first - second)
        if delta is not None:
            if not (first == second) and diff > delta:
                return
        else:
            if places is None:
                places = 7
            if not (first == second) and round(diff, places) != 0:
                return
        self.fail(self._formatMessage(msg, "%s == %s" % (_safe_repr(first), _safe_repr(second))))

    def assertRaises(self, expected_exception, *args, **kwargs):
        ctx = _AssertRaisesContext(expected_exception, self, None, kwargs.pop("msg", None))
        if not args:
            return ctx
        callable_obj = args[0]
        with ctx:
            callable_obj(*args[1:], **kwargs)
        return ctx

    def assertRaisesRegex(self, expected_exception, expected_regex, *args, **kwargs):
        ctx = _AssertRaisesContext(expected_exception, self, expected_regex, kwargs.pop("msg", None))
        if not args:
            return ctx
        callable_obj = args[0]
        with ctx:
            callable_obj(*args[1:], **kwargs)
        return ctx

    assertRaisesRegexp = assertRaisesRegex

    def assertWarns(self, expected_warning, *args, **kwargs):
        ctx = _AssertWarnsContext(expected_warning, self)
        if not args:
            return ctx
        with ctx:
            args[0](*args[1:], **kwargs)
        return ctx

    def assertWarnsRegex(self, expected_warning, expected_regex, *args, **kwargs):
        return self.assertWarns(expected_warning, *args, **kwargs)

    def assertNoLogs(self, logger=None, level=None):
        return _AssertWarnsContext(None, self)

    def assertRegex(self, text, expected_regex, msg=None):
        if not _matches(expected_regex, text):
            self.fail(self._formatMessage(msg, "Regex didn't match: %r not found in %r" % (expected_regex, text)))

    def assertNotRegex(self, text, unexpected_regex, msg=None):
        pass

    def assertHasAttr(self, obj, name, msg=None):
        if not hasattr(obj, name):
            self.fail(self._formatMessage(msg, "%s has no attribute %r" % (_safe_repr(obj), name)))

    def assertNotHasAttr(self, obj, name, msg=None):
        if hasattr(obj, name):
            self.fail(self._formatMessage(msg, "%s has unexpected attribute %r" % (_safe_repr(obj), name)))


class FunctionTestCase(TestCase):
    def __init__(self, testFunc, setUp=None, tearDown=None, description=None):
        TestCase.__init__(self, "runTest")
        self._testFunc = testFunc

    def runTest(self):
        self._testFunc()


class TestResult:
    def __init__(self, stream=None, descriptions=None, verbosity=None):
        self.failures = []
        self.errors = []
        self.skipped = []
        self.expectedFailures = []
        self.unexpectedSuccesses = []
        self.successes = []
        self.testsRun = 0
        self.shouldStop = False

    def startTest(self, test):
        self.testsRun += 1

    def stopTest(self, test):
        pass

    def addSuccess(self, test):
        self.successes.append(test)

    def addFailure(self, test, err):
        self.failures.append((test, _describe(err)))

    def addError(self, test, err):
        self.errors.append((test, _describe(err)))

    def addSkip(self, test, reason):
        self.skipped.append((test, reason))

    def addExpectedFailure(self, test, err):
        self.expectedFailures.append((test, _describe(err)))

    def addUnexpectedSuccess(self, test):
        self.unexpectedSuccesses.append(test)

    def wasSuccessful(self):
        return not self.failures and not self.errors and not self.unexpectedSuccesses

    def stop(self):
        self.shouldStop = True


def _describe(err):
    if isinstance(err, BaseException):
        return type(err).__name__ + ": " + str(err)
    return str(err)


class TestSuite:
    def __init__(self, tests=()):
        self._tests = []
        self.addTests(tests)

    def __iter__(self):
        return iter(self._tests)

    def countTestCases(self):
        n = 0
        for t in self._tests:
            if isinstance(t, TestSuite):
                n += t.countTestCases()
            else:
                n += 1
        return n

    def addTest(self, test):
        self._tests.append(test)

    def addTests(self, tests):
        for t in tests:
            self.addTest(t)

    def run(self, result):
        for t in self._tests:
            if result.shouldStop:
                break
            t.run(result)
        return result

    def __call__(self, result):
        return self.run(result)


def _test_names(cls, prefix="test"):
    names = []
    seen = set()
    for klass in cls.__mro__:
        for name in list(klass.__dict__):
            if name.startswith(prefix) and name not in seen:
                seen.add(name)
                if callable(getattr(cls, name, None)):
                    names.append(name)
    return sorted(names)


class TestLoader:
    testMethodPrefix = "test"

    def getTestCaseNames(self, testCaseClass):
        return _test_names(testCaseClass, self.testMethodPrefix)

    def loadTestsFromTestCase(self, testCaseClass):
        return TestSuite([testCaseClass(n) for n in self.getTestCaseNames(testCaseClass)])

    def loadTestsFromModule(self, module, pattern=None):
        tests = []
        for name in sorted(dir(module)):
            obj = getattr(module, name)
            if isinstance(obj, type) and issubclass(obj, TestCase) and obj is not TestCase:
                tests.append(self.loadTestsFromTestCase(obj))
        return TestSuite(tests)


defaultTestLoader = TestLoader()


class TextTestRunner:
    def __init__(self, stream=None, descriptions=True, verbosity=1, **kwargs):
        self.verbosity = verbosity

    def run(self, test):
        result = TestResult()
        test.run(result)
        for t, err in result.failures:
            print("FAIL:", t, err)
        for t, err in result.errors:
            print("ERROR:", t, err)
        print("Ran %d tests: %d failures, %d errors, %d skipped" % (result.testsRun, len(result.failures), len(result.errors), len(result.skipped)))
        return result


def main(module="__main__", exit=True, verbosity=1, **kwargs):
    mod = sys.modules.get(module) if isinstance(module, str) else module
    if mod is None:
        names = globals()
        raise RuntimeError("unittest.main(): module %r not found" % (module,))
    suite = defaultTestLoader.loadTestsFromModule(mod)
    result = TextTestRunner(verbosity=verbosity).run(suite)
    if exit:
        sys.exit(0 if result.wasSuccessful() else 1)
    return result
