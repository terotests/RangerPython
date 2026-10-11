# SPDX-License-Identifier: MIT
# dis for the Rust runtime: it disassembles this runtime's own bytecode,
# which _rpydis lays out. Instructions that do what CPython's do carry
# CPython's opnames; the others have names of their own.
import collections
import sys
import types

import _rpydis

__all__ = ["code_info", "dis", "disassemble", "distb", "disco",
           "findlinestarts", "findlabels", "show_code",
           "get_instructions", "Instruction", "Bytecode",
           "opname", "opmap", "cmp_op"]

opname = list(_rpydis.opnames)
opmap = {name: op for op, name in enumerate(opname)}
cmp_op = ('<', '<=', '==', '!=', '>', '>=')

hasconst = [opmap[n] for n in ("LOAD_CONST", "MAKE_FUNCTION", "MATCH_PATTERN")]
hasname = [opmap[n] for n in (
    "LOAD_GLOBAL", "STORE_GLOBAL", "DELETE_GLOBAL", "LOAD_NAME", "STORE_NAME",
    "DELETE_NAME", "LOAD_ATTR", "STORE_ATTR", "DELETE_ATTR", "LOAD_METHOD",
    "IMPORT_NAME", "IMPORT_FROM")]
hasjump = [opmap[n] for n in (
    "JUMP", "POP_JUMP_IF_FALSE", "POP_JUMP_IF_TRUE", "JUMP_IF_FALSE_OR_POP",
    "JUMP_IF_TRUE_OR_POP", "POP_JUMP_IF_NOT_NONE", "FOR_ITER",
    "SETUP_EXCEPT", "SETUP_FINALLY", "SETUP_WITH")]
hasjrel = []
hasjabs = hasjump
haslocal = [opmap[n] for n in ("LOAD_FAST", "STORE_FAST", "DELETE_FAST", "LOAD_FAST_AND_CLEAR")]
hasfree = [opmap[n] for n in (
    "MAKE_CELL", "LOAD_DEREF", "STORE_DEREF", "DELETE_DEREF", "LOAD_CLASSDEREF",
    "LOAD_CLOSURE")]
hascompare = [opmap["COMPARE_OP"]]
hasarg = sorted(set(hasconst + hasname + hasjump + haslocal + hasfree + hascompare + [
    opmap[n] for n in (
        "CALL_METHOD", "BUILD_SLICE", "BINARY_OP", "IS_OP", "CONTAINS_OP", "COPY",
        "SWAP", "BUILD_LIST", "BUILD_TUPLE", "BUILD_SET", "BUILD_MAP",
        "BUILD_STRING", "LIST_APPEND", "SET_ADD", "MAP_ADD", "LIST_EXTEND",
        "SET_UPDATE", "DICT_UPDATE", "DICT_MERGE", "FORMAT_VALUE",
        "UNPACK_SEQUENCE", "UNPACK_EX", "CALL", "CALL_KW", "RAISE_VARARGS",
        "WITH_EXIT")]))
hasexc = []

_Instruction = collections.namedtuple(
    "_Instruction",
    "opname opcode arg argval argrepr offset start_offset starts_line "
    "line_number label positions cache_info",
    defaults=[None, None, None])


class Instruction(_Instruction):
    """Details for a bytecode operation."""

    @property
    def oparg(self):
        return self.arg

    @property
    def baseopcode(self):
        return self.opcode

    @property
    def baseopname(self):
        return self.opname

    @property
    def cache_offset(self):
        return self.offset + 2

    @property
    def end_offset(self):
        return self.offset + 2

    @property
    def jump_target(self):
        return self.argval if self.opcode in hasjump else None

    @property
    def is_jump_target(self):
        return self.label is not None

    def _disassemble(self, lineno_width=3, mark_as_current=False, offset_width=4):
        fields = []
        if lineno_width:
            if self.starts_line:
                fields.append(str(self.line_number).rjust(lineno_width))
            else:
                fields.append(' ' * lineno_width)
        fields.append('-->' if mark_as_current else '   ')
        fields.append(repr(self.offset).rjust(offset_width))
        fields.append(self.opname.ljust(_OPNAME_WIDTH))
        if self.arg is not None:
            fields.append(repr(self.arg).rjust(_OPARG_WIDTH))
            if self.argrepr:
                fields.append('(' + self.argrepr + ')')
        return ' '.join(fields).rstrip()

    def __str__(self):
        return self._disassemble()


_OPNAME_WIDTH = max(map(len, opname)) + 1
_OPARG_WIDTH = 5


def _get_code_object(x):
    if hasattr(x, '__func__'):
        x = x.__func__
    if hasattr(x, '__code__'):
        x = x.__code__
    elif hasattr(x, 'gi_code'):
        x = x.gi_code
    elif hasattr(x, 'ag_code'):
        x = x.ag_code
    elif hasattr(x, 'cr_code'):
        x = x.cr_code
    if isinstance(x, str):
        x = _try_compile(x, "<disassembly>")
    if isinstance(x, types.CodeType):
        return x
    raise TypeError("don't know how to disassemble %s objects" % type(x).__name__)


def _try_compile(source, name):
    """Compiles as an expression if it is one, else as statements."""
    try:
        return compile(source, name, 'eval')
    except SyntaxError:
        pass
    return compile(source, name, 'exec')


def _get_instructions_bytes(co, first_line=None):
    rows = _rpydis.instructions(co)
    targets = {argval for (_, opcode, _, argval, *_) in rows if opcode in hasjump}
    labels = {off: i + 1 for i, off in enumerate(sorted(targets))}
    shift = 0 if first_line is None else first_line - co.co_firstlineno
    previous = None
    for name, opcode, arg, argval, argrepr, offset, line in rows:
        starts = line is not None and line != previous
        if line is not None:
            previous = line
            line += shift
        if opcode in hasjump:
            argrepr = "to L%d" % labels[argval]
        yield Instruction(name, opcode, arg, argval, argrepr, offset, offset,
                          starts, line, labels.get(offset))


def get_instructions(x, *, first_line=None, show_caches=None, adaptive=False):
    """Iterator for the opcodes in methods, functions or code."""
    return _get_instructions_bytes(_get_code_object(x), first_line)


def findlinestarts(code):
    """Finds the (offset, line) of each instruction that starts a line."""
    previous = None
    for start, _end, line in code.co_lines():
        if line is not None and line != previous:
            previous = line
            yield start, line


def findlabels(code):
    """The offsets that are jump targets."""
    return sorted({i.argval for i in get_instructions(code) if i.opcode in hasjump})


def _nested(co):
    return [c for c in co.co_consts if isinstance(c, types.CodeType)]


def _disassemble_recursive(co, *, file=None, depth=None):
    _disassemble(co, file=file)
    if depth is None or depth > 0:
        if depth is not None:
            depth = depth - 1
        for x in _nested(co):
            print(file=file)
            print("Disassembly of %r:" % (x,), file=file)
            _disassemble_recursive(x, file=file, depth=depth)


def _disassemble(co, lasti=-1, *, file=None):
    instrs = list(get_instructions(co))
    lines = [i.line_number for i in instrs if i.line_number is not None]
    width = max(len(str(max(lines))), 3) if lines else 0
    for instr in instrs:
        if instr.starts_line and instr.offset > 0:
            print(file=file)
        if instr.label is not None:
            print("  L%d:" % instr.label, file=file)
        print(instr._disassemble(width, instr.offset == lasti), file=file)


def disassemble(co, lasti=-1, *, file=None, show_caches=False, adaptive=False,
                show_offsets=False):
    """Disassemble a code object."""
    _disassemble(co, lasti, file=file)


disco = disassemble


def dis(x=None, *, file=None, depth=None, show_caches=False, adaptive=False,
        show_offsets=False):
    """Disassemble classes, methods, functions, generators, code or source."""
    if x is None:
        distb(file=file)
        return
    if hasattr(x, '__func__'):
        x = x.__func__
    if hasattr(x, '__code__'):
        x = x.__code__
    if isinstance(x, type) or isinstance(x, types.ModuleType):
        for name, x1 in sorted(x.__dict__.items()):
            if isinstance(x1, (types.MethodType, types.FunctionType, classmethod,
                               staticmethod, type)):
                print("Disassembly of %s:" % name, file=file)
                try:
                    dis(x1, file=file, depth=depth)
                except TypeError as msg:
                    print("Sorry:", msg, file=file)
                print(file=file)
        return
    _disassemble_recursive(_get_code_object(x), file=file, depth=depth)


def distb(tb=None, *, file=None, show_caches=False, adaptive=False, show_offsets=False):
    """Disassemble a traceback (default: last traceback)."""
    if tb is None:
        try:
            tb = sys.last_exc.__traceback__
        except AttributeError:
            try:
                tb = sys.last_traceback
            except AttributeError:
                raise RuntimeError("no last traceback to disassemble") from None
        while tb.tb_next:
            tb = tb.tb_next
    _disassemble(tb.tb_frame.f_code, tb.tb_lasti, file=file)


_FLAG_NAMES = {1: "OPTIMIZED", 2: "NEWLOCALS", 4: "VARARGS", 8: "VARKEYWORDS",
               16: "NESTED", 32: "GENERATOR", 64: "NOFREE", 128: "COROUTINE",
               256: "ITERABLE_COROUTINE", 512: "ASYNC_GENERATOR"}


def pretty_flags(flags):
    names = []
    for i in range(32):
        flag = 1 << i
        if flags & flag:
            names.append(_FLAG_NAMES.get(flag, hex(flag)))
            flags ^= flag
            if not flags:
                break
    else:
        names.append(hex(flags))
    return ", ".join(names)


def code_info(x):
    """Formatted details of methods, functions, or code."""
    co = _get_code_object(x)
    lines = [
        "Name:              %s" % (co.co_name,),
        "Filename:          %s" % (co.co_filename,),
        "Argument count:    %s" % (co.co_argcount,),
        "Positional-only arguments: %s" % (co.co_posonlyargcount,),
        "Kw-only arguments: %s" % (co.co_kwonlyargcount,),
        "Number of locals:  %s" % (co.co_nlocals,),
        "Flags:             %s" % (pretty_flags(co.co_flags),),
    ]
    for title, items, fmt in (("Constants", co.co_consts, "%4d: %r"),
                              ("Names", co.co_names, "%4d: %s"),
                              ("Variable names", co.co_varnames, "%4d: %s"),
                              ("Free variables", co.co_freevars, "%4d: %s"),
                              ("Cell variables", co.co_cellvars, "%4d: %s")):
        if items:
            lines.append(title + ":")
            lines.extend(fmt % i_n for i_n in enumerate(items))
    return "\n".join(lines)


def show_code(co, *, file=None):
    """Print details of methods, functions, or code to *file*."""
    print(code_info(co), file=file)


class Bytecode:
    """The bytecode operations of a piece of code, iterable as Instructions."""

    def __init__(self, x, *, first_line=None, current_offset=None,
                 show_caches=False, adaptive=False, show_offsets=False):
        self.codeobj = co = _get_code_object(x)
        self.first_line = co.co_firstlineno if first_line is None else first_line
        self._line_offset = self.first_line - co.co_firstlineno
        self.current_offset = current_offset
        self._original_object = x

    def __iter__(self):
        return _get_instructions_bytes(self.codeobj, self.first_line)

    def __repr__(self):
        return "{}({!r})".format(self.__class__.__name__, self._original_object)

    @classmethod
    def from_traceback(cls, tb, *, show_caches=False, adaptive=False):
        while tb.tb_next:
            tb = tb.tb_next
        return cls(tb.tb_frame.f_code, current_offset=tb.tb_lasti)

    def info(self):
        return code_info(self.codeobj)

    def dis(self):
        import io
        with io.StringIO() as output:
            _disassemble(self.codeobj,
                         -1 if self.current_offset is None else self.current_offset,
                         file=output)
            return output.getvalue()
