# SPDX-License-Identifier: MIT
# dis for the Rust runtime: its bytecode is its own, so there is nothing
# CPython-shaped to disassemble.
import collections

Instruction = collections.namedtuple("Instruction", "opname opcode arg argval argrepr offset starts_line is_jump_target")
opname = []
opmap = {}
hasconst = hasname = hasjrel = hasjabs = haslocal = hascompare = hasfree = []


def get_instructions(x, *, first_line=None, show_caches=False, adaptive=False):
    raise NotImplementedError("dis: this runtime's bytecode is not CPython's")


def dis(x=None, *, file=None, depth=None, show_caches=False, adaptive=False):
    raise NotImplementedError("dis: this runtime's bytecode is not CPython's")


disassemble = dis
