# SPDX-License-Identifier: MIT
# subprocess for the Ranger Python runtime, on the native _rpyprocess. A
# child is started at once; its pipes are fed and read to the end when the
# parent first asks for their data (communicate, wait, or a read), as a
# runtime without threads can do it. A WebAssembly build cannot start one.
import os
import sys
import _rpyprocess

__all__ = ["Popen", "PIPE", "STDOUT", "call", "check_call", "getstatusoutput",
           "getoutput", "check_output", "run", "CalledProcessError", "DEVNULL",
           "SubprocessError", "TimeoutExpired", "CompletedProcess"]

PIPE = -1
STDOUT = -2
DEVNULL = -3

_can_fork_exec = _rpyprocess.available
_USE_POSIX_SPAWN = False
_USE_VFORK = False


class SubprocessError(Exception):
    pass


class CalledProcessError(SubprocessError):
    def __init__(self, returncode, cmd, output=None, stderr=None):
        self.returncode = returncode
        self.cmd = cmd
        self.output = output
        self.stderr = stderr

    def __str__(self):
        if self.returncode and self.returncode < 0:
            return "Command '%s' died with signal %d." % (self.cmd, -self.returncode)
        return "Command '%s' returned non-zero exit status %d." % (self.cmd, self.returncode)

    @property
    def stdout(self):
        return self.output

    @stdout.setter
    def stdout(self, value):
        self.output = value


class TimeoutExpired(SubprocessError):
    def __init__(self, cmd, timeout, output=None, stderr=None):
        self.cmd = cmd
        self.timeout = timeout
        self.output = output
        self.stderr = stderr

    def __str__(self):
        return "Command '%s' timed out after %s seconds" % (self.cmd, self.timeout)

    @property
    def stdout(self):
        return self.output

    @stdout.setter
    def stdout(self, value):
        self.output = value


class CompletedProcess:
    def __init__(self, args, returncode, stdout=None, stderr=None):
        self.args = args
        self.returncode = returncode
        self.stdout = stdout
        self.stderr = stderr

    def __repr__(self):
        args = ["args={!r}".format(self.args), "returncode={!r}".format(self.returncode)]
        if self.stdout is not None:
            args.append("stdout={!r}".format(self.stdout))
        if self.stderr is not None:
            args.append("stderr={!r}".format(self.stderr))
        return "{}({})".format(type(self).__name__, ", ".join(args))

    def check_returncode(self):
        if self.returncode:
            raise CalledProcessError(self.returncode, self.args, self.stdout, self.stderr)


def _fsdecode(name):
    name = os.fspath(name)
    return name.decode("utf-8", "surrogateescape") if isinstance(name, bytes) else name


def _cleanup():
    pass


def list2cmdline(seq):
    # as CPython's: the MS C runtime's quoting rules
    result = []
    needquote = False
    for arg in map(_fsdecode, seq):
        bs_buf = []
        if result:
            result.append(' ')
        needquote = (" " in arg) or ("\t" in arg) or not arg
        if needquote:
            result.append('"')
        for c in arg:
            if c == '\\':
                bs_buf.append(c)
            elif c == '"':
                result.append('\\' * len(bs_buf) * 2)
                bs_buf = []
                result.append('\\"')
            else:
                if bs_buf:
                    result.extend(bs_buf)
                    bs_buf = []
                result.append(c)
        if bs_buf:
            result.extend(bs_buf)
        if needquote:
            result.extend(bs_buf)
            result.append('"')
    return ''.join(result)


class _Pipe:
    """The parent's end of a child's pipe."""

    def __init__(self, proc, name, text):
        self._proc = proc
        self._name = name
        self._text = text
        self._buf = None
        self.closed = False

    # stdin: what is written goes to the child when its output is read
    def write(self, data):
        if self.closed:
            raise ValueError("write to closed file")
        self._proc._input.append(data)
        return len(data)

    def flush(self):
        pass

    def _data(self):
        if self._buf is None:
            self._proc._collect()
            self._buf = self._proc._captured[self._name]
            if self._buf is None:
                self._buf = "" if self._text else b""
        return self._buf

    def read(self, size=-1):
        if self.closed:
            raise ValueError("read of closed file")
        data = self._data()
        if size is None or size < 0:
            size = len(data)
        self._buf = data[size:]
        return data[:size]

    def readline(self, size=-1):
        data = self._data()
        nl = "\n" if self._text else b"\n"
        i = data.find(nl)
        end = len(data) if i < 0 else i + 1
        if size is not None and 0 <= size < end:
            end = size
        self._buf = data[end:]
        return data[:end]

    def readlines(self):
        lines = []
        while True:
            line = self.readline()
            if not line:
                return lines
            lines.append(line)

    def __iter__(self):
        return iter(self.readlines())

    def fileno(self):
        raise OSError("the pipe has no file descriptor on this runtime")

    def close(self):
        self.closed = True

    def __enter__(self):
        return self

    def __exit__(self, *exc):
        self.close()


def _mode(stream):
    # what the native side does with a stream argument; a file object
    # gets the data through a pipe once the child is done
    if stream is None or stream in (PIPE, STDOUT, DEVNULL):
        return stream
    if isinstance(stream, int):
        return None
    if hasattr(stream, "write"):
        return PIPE
    raise TypeError("bad stream argument %r" % (stream,))


class Popen:
    def __init__(self, args, bufsize=-1, executable=None, stdin=None, stdout=None, stderr=None,
                 preexec_fn=None, close_fds=True, shell=False, cwd=None, env=None,
                 universal_newlines=None, startupinfo=None, creationflags=0,
                 restore_signals=True, start_new_session=False, pass_fds=(), *,
                 user=None, group=None, extra_groups=None, encoding=None, errors=None,
                 text=None, umask=-1, pipesize=-1, process_group=None):
        self.args = args
        self.returncode = None
        self.stdin = self.stdout = self.stderr = None
        self._input = []
        self._captured = None
        if universal_newlines is not None and text is not None and bool(universal_newlines) != bool(text):
            raise SubprocessError("Cannot disambiguate when both text and universal_newlines are supplied but different. Pass one or the other.")
        self.text_mode = bool(encoding or errors or text or universal_newlines)
        self.encoding = encoding
        self.errors = errors
        if isinstance(args, (str, bytes)) or hasattr(args, "__fspath__"):
            argv = [args]
        else:
            argv = list(args)
        if shell:
            argv = ["/bin/sh", "-c"] + argv
        argv = [_fsdecode(a) for a in argv]
        if executable is not None:
            argv[0] = _fsdecode(executable)
        if env is None:
            env = os.environ
        env = {_fsdecode(k): _fsdecode(v) for k, v in env.items()}
        if cwd is not None:
            cwd = _fsdecode(cwd)
        self._streams = (stdin, stdout, stderr)
        modes = (_mode(stdin), _mode(stdout), _mode(stderr))
        self._handle, self.pid = _rpyprocess.spawn(argv, env, cwd, *modes)
        if stdin == PIPE:
            self.stdin = _Pipe(self, 0, self.text_mode)
        if stdout == PIPE:
            self.stdout = _Pipe(self, 0, self.text_mode)
        if stderr == PIPE:
            self.stderr = _Pipe(self, 1, self.text_mode)
        self._modes = modes

    def _translate(self, data):
        if data is None or not self.text_mode:
            return data
        data = data.decode(self.encoding or "utf-8", self.errors or "strict")
        return data.replace("\r\n", "\n").replace("\r", "\n")

    def _collect(self, input=None):
        # runs the child to its end, once: feeds stdin, reads the pipes
        if self._captured is not None:
            return
        chunks = self._input
        if input is not None:
            chunks = chunks + [input]
        data = None
        if self._modes[0] == PIPE:
            if self.text_mode:
                data = "".join(chunks).encode(self.encoding or "utf-8", self.errors or "strict")
            else:
                data = b"".join(bytes(c) for c in chunks)
        out, err, self.returncode = _rpyprocess.communicate(self._handle, data)
        out, err = self._translate(out), self._translate(err)
        # a file object given as a stream receives what the child wrote
        for stream, value in ((self._streams[1], out), (self._streams[2], err)):
            if value is not None and stream not in (None, PIPE, STDOUT, DEVNULL) and not isinstance(stream, int):
                stream.write(value)
        self._captured = (out if self._streams[1] == PIPE else None,
                          err if self._streams[2] == PIPE else None)

    def communicate(self, input=None, timeout=None):
        if self._captured is not None and input:
            raise ValueError("Cannot send input after starting communication")
        if self.stdin is not None:
            self.stdin.closed = True
        self._collect(input)
        out = self.stdout.read() if self.stdout is not None and not self.stdout.closed else self._captured[0]
        err = self.stderr.read() if self.stderr is not None and not self.stderr.closed else self._captured[1]
        return (out, err)

    def poll(self):
        if self.returncode is None:
            self.returncode = _rpyprocess.poll(self._handle)
        return self.returncode

    def wait(self, timeout=None):
        if self.returncode is None:
            if self._modes[1] == PIPE or self._modes[2] == PIPE:
                # the pipes are drained so that the child cannot block on them
                self._collect()
            else:
                self.returncode = _rpyprocess.wait(self._handle)
        return self.returncode

    def send_signal(self, sig):
        self.kill()

    def terminate(self):
        self.kill()

    def kill(self):
        if self.returncode is None:
            _rpyprocess.kill(self._handle)

    def __enter__(self):
        return self

    def __exit__(self, exc_type, value, traceback):
        for s in (self.stdout, self.stderr, self.stdin):
            if s is not None:
                s.close()
        self.wait()

    def __repr__(self):
        return "<%s: returncode: %s args: %r>" % (type(self).__name__, self.returncode, self.args)

    __class_getitem__ = classmethod(lambda cls, item: cls)


def call(*popenargs, timeout=None, **kwargs):
    with Popen(*popenargs, **kwargs) as p:
        return p.wait(timeout=timeout)


def check_call(*popenargs, **kwargs):
    retcode = call(*popenargs, **kwargs)
    if retcode:
        cmd = kwargs.get("args")
        if cmd is None:
            cmd = popenargs[0]
        raise CalledProcessError(retcode, cmd)
    return 0


def run(*popenargs, input=None, capture_output=False, timeout=None, check=False, **kwargs):
    if input is not None:
        if kwargs.get("stdin") is not None:
            raise ValueError("stdin and input arguments may not both be used.")
        kwargs["stdin"] = PIPE
    if capture_output:
        if kwargs.get("stdout") is not None or kwargs.get("stderr") is not None:
            raise ValueError("stdout and stderr arguments may not be used with capture_output.")
        kwargs["stdout"] = PIPE
        kwargs["stderr"] = PIPE
    with Popen(*popenargs, **kwargs) as process:
        stdout, stderr = process.communicate(input, timeout=timeout)
        retcode = process.poll()
        if check and retcode:
            raise CalledProcessError(retcode, process.args, output=stdout, stderr=stderr)
    return CompletedProcess(process.args, retcode, stdout, stderr)


def check_output(*popenargs, timeout=None, **kwargs):
    for kw in ("stdout", "check"):
        if kw in kwargs:
            raise ValueError("%s argument not allowed, it will be overridden." % kw)
    if "input" in kwargs and kwargs["input"] is None:
        kwargs["input"] = "" if kwargs.get("universal_newlines") or kwargs.get("text") or kwargs.get("encoding") or kwargs.get("errors") else b""
    return run(*popenargs, stdout=PIPE, timeout=timeout, check=True, **kwargs).stdout


def getstatusoutput(cmd, *, encoding=None, errors=None):
    try:
        data = check_output(cmd, shell=True, text=True, stderr=STDOUT, encoding=encoding, errors=errors)
        exitcode = 0
    except CalledProcessError as ex:
        data = ex.output
        exitcode = ex.returncode
    if data[-1:] == "\n":
        data = data[:-1]
    return exitcode, data


def getoutput(cmd, *, encoding=None, errors=None):
    return getstatusoutput(cmd, encoding=encoding, errors=errors)[1]
