# SPDX-License-Identifier: MIT
# After CPython's test.support.script_helper: the interpreter under test
# runs as a child process through sys.executable, when the runtime can
# start one.
import collections
import os
import sys
import unittest

from test import support


def _no(*args, **kwargs):
    raise unittest.SkipTest("no subprocesses on this runtime")


def interpreter_requires_environment():
    return False


class _PythonRunResult(collections.namedtuple("_PythonRunResult", ("rc", "out", "err"))):
    """Helper for reporting Python subprocess run results"""

    def fail(self, cmd_line):
        maxlen = 300 * 100
        out, err = self.out, self.err
        if len(out) > maxlen:
            out = b'(... truncated stdout ...)' + out[-maxlen:]
        if len(err) > maxlen:
            err = b'(... truncated stderr ...)' + err[-maxlen:]
        out = out.decode('ascii', 'replace').rstrip()
        err = err.decode('ascii', 'replace').rstrip()
        raise AssertionError("Process return code is %d\n"
                             "command line: %r\n"
                             "\n"
                             "stdout:\n"
                             "---\n"
                             "%s\n"
                             "---\n"
                             "\n"
                             "stderr:\n"
                             "---\n"
                             "%s\n"
                             "---"
                             % (self.rc, cmd_line, out, err))


def run_python_until_end(*args, **env_vars):
    import subprocess
    run_using_command = env_vars.pop('__run_using_command', None)
    cwd = env_vars.pop('__cwd', None)
    if '__isolated' in env_vars:
        isolated = env_vars.pop('__isolated')
    else:
        isolated = not env_vars
    cmd_line = [sys.executable, '-X', 'faulthandler']
    if run_using_command:
        cmd_line = run_using_command + cmd_line
    if isolated:
        cmd_line.append('-I')
    elif not env_vars:
        cmd_line.append('-E')
    if env_vars.pop('__cleanenv', None):
        env = {}
    else:
        env = os.environ.copy()
    if 'TERM' not in env_vars:
        env['TERM'] = ''
    env.update(env_vars)
    cmd_line.extend(args)
    proc = subprocess.Popen(cmd_line, stdin=subprocess.PIPE,
                            stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                            env=env, cwd=cwd)
    with proc:
        try:
            out, err = proc.communicate()
        finally:
            proc.kill()
            subprocess._cleanup()
    rc = proc.returncode
    return _PythonRunResult(rc, out, err), cmd_line


def _assert_python(expected_success, /, *args, **env_vars):
    res, cmd_line = run_python_until_end(*args, **env_vars)
    if (res.rc and expected_success) or (not res.rc and not expected_success):
        res.fail(cmd_line)
    return res


def assert_python_ok(*args, **env_vars):
    return _assert_python(True, *args, **env_vars)


def assert_python_failure(*args, **env_vars):
    return _assert_python(False, *args, **env_vars)


def spawn_python(*args, stdout=-1, stderr=-2, **kw):
    import subprocess
    cmd_line = [sys.executable, '-E']
    cmd_line.extend(args)
    env = kw.setdefault('env', dict(os.environ))
    env['TERM'] = 'vt100'
    return subprocess.Popen(cmd_line, stdin=subprocess.PIPE,
                            stdout=stdout, stderr=stderr, **kw)


def kill_python(p):
    p.stdin.close()
    data = p.stdout.read()
    p.stdout.close()
    p.wait()
    return data


def make_script(script_dir, script_basename, source, omit_suffix=False):
    script_filename = script_basename
    if not omit_suffix:
        script_filename += os.extsep + 'py'
    script_name = os.path.join(script_dir, script_filename)
    if isinstance(source, str):
        with open(script_name, 'w', encoding='utf-8') as script_file:
            script_file.write(source)
    else:
        with open(script_name, 'wb') as script_file:
            script_file.write(source)
    return script_name


if not support.has_subprocess_support:
    assert_python_ok = _no
    assert_python_failure = _no
    spawn_python = _no
    run_python_until_end = _no
