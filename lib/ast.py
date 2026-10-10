# SPDX-License-Identifier: MIT
# ast for the Ranger Python runtime. There is no Python-level syntax tree
# here: literal_eval works, parse() raises NotImplementedError.

PyCF_ONLY_AST = 1024
PyCF_TYPE_COMMENTS = 4096
PyCF_ALLOW_TOP_LEVEL_AWAIT = 8192
PyCF_OPTIMIZED_AST = 33792


class AST:
    _fields = ()
    _attributes = ()

    def __init__(self, *args, **kwargs):
        for name, value in zip(self._fields, args):
            setattr(self, name, value)
        for name, value in kwargs.items():
            setattr(self, name, value)


def _node(name, fields=()):
    return type(name, (AST,), {"_fields": fields})


for _name, _fields in [("Module", ("body", "type_ignores")), ("Expression", ("body",)),
                       ("Interactive", ("body",)), ("expr", ()), ("stmt", ()), ("mod", ()),
                       ("Constant", ("value", "kind")), ("Name", ("id", "ctx")), ("Load", ()),
                       ("Store", ()), ("Del", ()), ("Expr", ("value",)), ("BinOp", ("left", "op", "right")),
                       ("Add", ()), ("Sub", ()), ("Mult", ()), ("Div", ()), ("UnaryOp", ("op", "operand")),
                       ("USub", ()), ("UAdd", ()), ("Not", ()), ("Invert", ()), ("Call", ("func", "args", "keywords")),
                       ("Attribute", ("value", "attr", "ctx")), ("Assign", ("targets", "value", "type_comment")),
                       ("FunctionDef", ("name", "args", "body", "decorator_list", "returns", "type_comment")),
                       ("ClassDef", ("name", "bases", "keywords", "body", "decorator_list")),
                       ("Return", ("value",)), ("If", ("test", "body", "orelse")), ("Pass", ()),
                       ("List", ("elts", "ctx")), ("Tuple", ("elts", "ctx")), ("Dict", ("keys", "values")),
                       ("Set", ("elts",)), ("arguments", ()), ("arg", ("arg", "annotation")), ("keyword", ("arg", "value"))]:
    globals()[_name] = _node(_name, _fields)


def parse(source, filename="<unknown>", mode="exec", *, type_comments=False, feature_version=None, optimize=-1):
    raise NotImplementedError("ast.parse is not available in this runtime")


def dump(node, annotate_fields=True, include_attributes=False, *, indent=None, show_empty=False):
    return repr(node)


def _check_literal(src):
    allowed = set("0123456789abcdefABCDEFxXoOjJ.eE+-_ \t\n()[]{},:'\"\\")
    i = 0
    n = len(src)
    while i < n:
        c = src[i]
        if c in "'\"":
            q = c
            triple = src.startswith(q * 3, i)
            end = q * 3 if triple else q
            i += len(end)
            while i < n and not src.startswith(end, i):
                i += 2 if src[i] == "\\" else 1
            i += len(end)
            continue
        if c.isalpha():
            j = i
            while j < n and (src[j].isalnum() or src[j] == "_"):
                j += 1
            word = src[i:j]
            prev = src[i - 1] if i else ""
            if word in ("True", "False", "None") or (prev.isdigit() or prev == "."):
                i = j
                continue
            if word.lower() in ("b", "r", "u", "br", "rb") and j < n and src[j] in "'\"":
                i = j
                continue
            if word in ("set",) and src.startswith("()", j):
                i = j + 2
                continue
            raise ValueError("malformed node or string: %r" % src)
        if c not in allowed:
            raise ValueError("malformed node or string: %r" % src)
        i += 1


def literal_eval(node_or_string):
    if not isinstance(node_or_string, str):
        raise ValueError("malformed node or string: %r" % (node_or_string,))
    src = node_or_string.strip(" \t")
    _check_literal(src)
    return eval(src, {"__builtins__": {"set": set}}, {})


def unparse(node):
    raise NotImplementedError("ast.unparse is not available in this runtime")


class NodeVisitor:
    def visit(self, node):
        return getattr(self, "visit_" + type(node).__name__, self.generic_visit)(node)

    def generic_visit(self, node):
        pass


class NodeTransformer(NodeVisitor):
    pass


def walk(node):
    return iter([node])


def get_docstring(node, clean=True):
    return None
