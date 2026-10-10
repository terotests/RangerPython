# SPDX-License-Identifier: MIT
#
# A stand-in for CPython's test.support, enough for its test files to import
# and run on the Ranger runtime. Helpers about CPython itself (reference
# counts, subinterpreters, the C stack) skip the tests that use them; any
# helper not written here is a _Missing, which skips the test it decorates
# or is entered in, so a file is not lost to one helper.
import sys
import unittest

# CPython leaves it 1 unless regrtest lowers it
verbose = 1
is_jython = False
is_android = False
is_emscripten = False
is_wasi = False
is_s390x = False
skip_on_s390x = unittest.skipIf(is_s390x, "skipped on s390x")
is_apple_mobile = False
has_subprocess_support = False
Py_DEBUG = False
Py_TRACE_REFS = False
MISSING_C_DOCSTRINGS = True
HAVE_DOCSTRINGS = False
MAX_Py_ssize_t = sys.maxsize
NHASHBITS = 61
SHORT_TIMEOUT = 30.0
LONG_TIMEOUT = 300.0
LOOPBACK_TIMEOUT = 5.0
INTERNET_TIMEOUT = 60.0
TEST_HOME_DIR = "/test"
TESTFN = "@test_tmp"
PGO = False
real_max_memuse = 0
max_memuse = 0
SAVEDCWD = "/"


class TestFailed(Exception):
    pass


class ResourceDenied(unittest.SkipTest):
    pass


class _Missing:
    def __init__(self, name):
        self._name = name

    def _skip(self):
        return unittest.skip("test.support.%s is not available on this runtime" % self._name)

    def __call__(self, *args, **kwargs):
        if len(args) == 1 and not kwargs and (callable(args[0]) or isinstance(args[0], type)):
            return self._skip()(args[0])
        return _Missing(self._name)

    def __enter__(self):
        raise unittest.SkipTest("test.support.%s is not available on this runtime" % self._name)

    def __exit__(self, *exc):
        return False

    def __getattr__(self, name):
        return _Missing(self._name + "." + name)

    def __bool__(self):
        return False


_SUBMODULES = ("import_helper", "os_helper", "script_helper", "threading_helper", "warnings_helper")


def __getattr__(name):
    if name.startswith("__"):
        raise AttributeError(name)
    # `from test.support import os_helper` reads the attribute before it
    # imports the submodule, so a submodule must not come back as a _Missing
    if name in _SUBMODULES:
        return __import__("test.support." + name, fromlist=["_"])
    return _Missing(name)


def _skipper(reason):
    def decorator(*args, **kwargs):
        if len(args) == 1 and not kwargs and callable(args[0]):
            return unittest.skip(reason)(args[0])
        return unittest.skip(reason)
    return decorator


# CPython's own internals: the tests marked with these are skipped
def cpython_only(f):
    return f
refcount_test = unittest.skip("reference counts")
bigaddrspacetest = unittest.skip("big address space")
requires_subprocess = _skipper("no subprocesses")
requires_fork = _skipper("no fork")
# as CPython's on any platform but macOS (sys.platform is "linux" here)
requires_mac_ver = lambda *version: (lambda f: f)
requires_resource = _skipper("resource not enabled")
requires_docstrings = unittest.skip("no docstrings")
no_tracing = lambda f: f
suppress_immortalization = lambda *a, **k: (lambda f: f) if not (len(a) == 1 and callable(a[0])) else a[0]
skip_if_pgo_task = lambda f: f
requires_IEEE_754 = lambda f: f
requires_debug_ranges = _skipper("no debug ranges")
force_not_colorized = lambda f: f
run_with_locale = lambda *a, **k: (lambda f: f)
run_with_tz = lambda *a, **k: (lambda f: f)


def bigmemtest(size, memuse, dry_run=True):
    def decorator(f):
        def wrapper(self):
            return f(self, 5147)
        wrapper.__name__ = getattr(f, "__name__", "test")
        return wrapper
    return decorator


def check_impl_detail(**guards):
    guards = dict(guards)
    if not guards:
        return False
    if "cpython" in guards:
        return not guards["cpython"]
    return True


def impl_detail(msg=None, **guards):
    # run what CPython runs, like cpython_only above
    run = guards.get("cpython", True) if guards else True
    if run:
        return lambda f: f
    return unittest.skip(msg or "implementation detail")


def gc_collect():
    pass


def get_c_recursion_limit():
    return 150


def exceeds_recursion_limit():
    return 300


Py_C_RECURSION_LIMIT = 150


def setswitchinterval(interval):
    pass


def get_attribute(obj, name):
    try:
        return getattr(obj, name)
    except AttributeError:
        raise unittest.SkipTest("object %r has no attribute %r" % (obj, name))


def run_unittest(*classes):
    suite = unittest.TestSuite()
    for c in classes:
        suite.addTests(unittest.defaultTestLoader.loadTestsFromTestCase(c))
    result = unittest.TestResult()
    suite.run(result)
    if not result.wasSuccessful():
        raise TestFailed("tests failed")


class _NullContext:
    def __init__(self, *args, **kwargs):
        pass

    def __enter__(self):
        return self

    def __exit__(self, *exc):
        return False

    def __call__(self, func):
        return func


def infinite_recursion(max_depth=None):
    return _NullContext()


def disable_gc():
    return _NullContext()


class adjust_int_max_str_digits:
    def __init__(self, max_digits):
        self.max_digits = max_digits

    def __enter__(self):
        self.old = sys.get_int_max_str_digits()
        sys.set_int_max_str_digits(self.max_digits)
        return self

    def __exit__(self, *exc):
        sys.set_int_max_str_digits(self.old)
        return False

    def __call__(self, func):
        import functools

        @functools.wraps(func)
        def wrapper(*args, **kwargs):
            with self:
                return func(*args, **kwargs)
        return wrapper


class catch_unraisable_exception:
    def __init__(self):
        self.unraisable = None
        self._old_hook = None

    def _hook(self, unraisable):
        self.unraisable = unraisable

    def __enter__(self):
        self._old_hook = sys.unraisablehook
        sys.unraisablehook = self._hook
        return self

    def __exit__(self, *exc_info):
        sys.unraisablehook = self._old_hook
        del self.unraisable


class swap_attr:
    def __init__(self, obj, attr, new_val):
        self.obj = obj
        self.attr = attr
        self.new_val = new_val

    def __enter__(self):
        self.had = hasattr(self.obj, self.attr)
        if self.had:
            self.old = getattr(self.obj, self.attr)
        setattr(self.obj, self.attr, self.new_val)
        return self.old if self.had else None

    def __exit__(self, *exc):
        if self.had:
            setattr(self.obj, self.attr, self.old)
        else:
            delattr(self.obj, self.attr)
        return False


class swap_item:
    def __init__(self, obj, item, new_val):
        self.obj = obj
        self.item = item
        self.new_val = new_val

    def __enter__(self):
        self.had = self.item in self.obj
        if self.had:
            self.old = self.obj[self.item]
        self.obj[self.item] = self.new_val
        return self.old if self.had else None

    def __exit__(self, *exc):
        if self.had:
            self.obj[self.item] = self.old
        elif self.item in self.obj:
            del self.obj[self.item]
        return False


class _Captured:
    def __init__(self, stream):
        self.stream = stream

    def __enter__(self):
        import io
        self.old = getattr(sys, self.stream)
        self.new = io.StringIO()
        setattr(sys, self.stream, self.new)
        return self.new

    def __exit__(self, *exc):
        setattr(sys, self.stream, self.old)
        return False


def captured_output(stream_name):
    return _Captured(stream_name)


def captured_stdout():
    return _Captured("stdout")


def captured_stderr():
    return _Captured("stderr")


def check_syntax_error(testcase, statement, errtext="", *, lineno=None, offset=None):
    with testcase.assertRaises(SyntaxError) as cm:
        compile(statement, "<test string>", "exec")
    err = cm.exception
    if lineno is not None:
        testcase.assertEqual(err.lineno, lineno)


def check_free_after_iterating(test, iter, cls, args=()):
    done = False
    def wrapper():
        class A(cls):
            def __del__(self):
                nonlocal done
                try:
                    next(it)
                except StopIteration:
                    done = True
        return A
    it = iter(wrapper()(*args))
    # Issue 26494: Shouldn't crash
    test.assertRaises(StopIteration, next, it)
    # The sequence should be deallocated just after the end of iterating
    gc_collect()
    test.assertTrue(done)


def check_disallow_instantiation(testcase, tp, *args, **kwds):
    testcase.assertRaises(TypeError, tp, *args, **kwds)


def check__all__(test_case, module, name_of_module=None, extra=(), not_exported=()):
    raise unittest.SkipTest("check__all__ needs module introspection")


def findfile(filename, subdir=None):
    return filename


class _ALWAYS_EQ:
    def __eq__(self, other):
        return True

    def __ne__(self, other):
        return False


ALWAYS_EQ = _ALWAYS_EQ()


class _NEVER_EQ:
    def __eq__(self, other):
        return False

    def __ne__(self, other):
        return True

    def __hash__(self):
        return 1


NEVER_EQ = _NEVER_EQ()


class _LARGEST:
    def __eq__(self, other):
        return isinstance(other, _LARGEST)

    def __lt__(self, other):
        return False

    def __gt__(self, other):
        return True


LARGEST = _LARGEST()


class _SMALLEST:
    def __eq__(self, other):
        return isinstance(other, _SMALLEST)

    def __gt__(self, other):
        return False

    def __lt__(self, other):
        return True


SMALLEST = _SMALLEST()


class BrokenIter:
    def __init__(self, init_raises=False, next_raises=False, iter_raises=False):
        if init_raises:
            1 / 0
        self.next_raises = next_raises
        self.iter_raises = iter_raises

    def __next__(self):
        if self.next_raises:
            1 / 0

    def __iter__(self):
        if self.iter_raises:
            1 / 0
        return self


def maybe_get_event_loop_policy():
    return None


def calcobjsize(fmt):
    raise unittest.SkipTest("object sizes are CPython's")


calcvobjsize = calcobjsize


def check_sizeof(test, o, size):
    raise unittest.SkipTest("object sizes are CPython's")


class CPUStopwatch:
    def __enter__(self):
        import time
        self.clock_info = time.get_clock_info("process_time")
        self.start = time.process_time()
        self.seconds = 0.0
        return self

    def __exit__(self, *exc):
        import time
        self.seconds = time.process_time() - self.start
        return False
