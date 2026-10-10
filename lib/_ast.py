# SPDX-License-Identifier: MIT
# _ast for the Ranger Python runtime: the syntax tree node classes that
# ast.parse() returns and compile() accepts. The table follows CPython's
# Parser/Python.asdl: each class's base, fields with their ASDL types
# ("expr*" a list, "expr?" optional), attributes and docstring.
PyCF_ONLY_AST = 1024
PyCF_TYPE_COMMENTS = 4096
PyCF_ALLOW_TOP_LEVEL_AWAIT = 8192
PyCF_OPTIMIZED_AST = 33792


class AST:
    _fields = ()
    _attributes = ()
    __match_args__ = ()

    def __init__(self, *args, **kwargs):
        cls = type(self)
        fields = cls._fields
        if len(args) > len(fields):
            raise TypeError("%s constructor takes at most %d positional argument%s"
                            % (cls.__name__, len(fields), "" if len(fields) == 1 else "s"))
        for name, value in zip(fields, args):
            setattr(self, name, value)
        for name, value in kwargs.items():
            if name in fields[:len(args)]:
                raise TypeError("%s got multiple values for argument %r" % (cls.__name__, name))
            if name not in fields and name not in cls._attributes:
                _deprecated("%s.__init__ got an unexpected keyword argument %r. Support for "
                            "arbitrary keyword arguments is deprecated and will be removed "
                            "in Python 3.15." % (cls.__name__, name))
            setattr(self, name, value)
        for name in fields[len(args):]:
            if name in kwargs:
                continue
            kind = _kinds.get((cls, name))
            if kind is None:
                continue
            if kind.endswith("*"):
                setattr(self, name, [])
            elif kind.endswith("?"):
                pass
            elif kind == "expr_context":
                setattr(self, name, Load())
            else:
                _deprecated("%s.__init__ missing 1 required positional argument: %r. This "
                            "will become an error in Python 3.15." % (cls.__name__, name))

    def __reduce__(self):
        return type(self), (), dict(self.__dict__)


def _deprecated(message):
    import warnings
    warnings.warn(message, DeprecationWarning, stacklevel=3)


# name, base, "field:type ...", attributes (None: inherited), docstring
_table = [
    ('operator', 'AST', '', '',
     'operator = Add | Sub | Mult | MatMult | Div | Mod | Pow | LShift | RShift | BitOr | BitXor | BitAnd | FloorDiv'),
    ('Add', 'operator', '', None,
     'Add'),
    ('boolop', 'AST', '', '',
     'boolop = And | Or'),
    ('And', 'boolop', '', None,
     'And'),
    ('stmt', 'AST', '', 'lineno col_offset end_lineno end_col_offset',
     'stmt = FunctionDef(identifier name, arguments args, stmt* body, expr* decorator_list, expr? returns, string? type_comment, type_param* type_params)\n     | AsyncFunctionDef(identifier name, arguments args, stmt* body, expr* decorator_list, expr? returns, string? type_comment, type_param* type_params)\n     | ClassDef(identifier name, expr* bases, keyword* keywords, stmt* body, expr* decorator_list, type_param* type_params)\n     | Return(expr? value)\n     | Delete(expr* targets)\n     | Assign(expr* targets, expr value, string? type_comment)\n     | TypeAlias(expr name, type_param* type_params, expr value)\n     | AugAssign(expr target, operator op, expr value)\n     | AnnAssign(expr target, expr annotation, expr? value, int simple)\n     | For(expr target, expr iter, stmt* body, stmt* orelse, string? type_comment)\n     | AsyncFor(expr target, expr iter, stmt* body, stmt* orelse, string? type_comment)\n     | While(expr test, stmt* body, stmt* orelse)\n     | If(expr test, stmt* body, stmt* orelse)\n     | With(withitem* items, stmt* body, string? type_comment)\n     | AsyncWith(withitem* items, stmt* body, string? type_comment)\n     | Match(expr subject, match_case* cases)\n     | Raise(expr? exc, expr? cause)\n     | Try(stmt* body, excepthandler* handlers, stmt* orelse, stmt* finalbody)\n     | TryStar(stmt* body, excepthandler* handlers, stmt* orelse, stmt* finalbody)\n     | Assert(expr test, expr? msg)\n     | Import(alias* names)\n     | ImportFrom(identifier? module, alias* names, int? level)\n     | Global(identifier* names)\n     | Nonlocal(identifier* names)\n     | Expr(expr value)\n     | Pass\n     | Break\n     | Continue'),
    ('AnnAssign', 'stmt', 'target:expr annotation:expr value:expr? simple:int', None,
     'AnnAssign(expr target, expr annotation, expr? value, int simple)'),
    ('Assert', 'stmt', 'test:expr msg:expr?', None,
     'Assert(expr test, expr? msg)'),
    ('Assign', 'stmt', 'targets:expr* value:expr type_comment:str?', None,
     'Assign(expr* targets, expr value, string? type_comment)'),
    ('AsyncFor', 'stmt', 'target:expr iter:expr body:stmt* orelse:stmt* type_comment:str?', None,
     'AsyncFor(expr target, expr iter, stmt* body, stmt* orelse, string? type_comment)'),
    ('AsyncFunctionDef', 'stmt', 'name:str args:arguments body:stmt* decorator_list:expr* returns:expr? type_comment:str? type_params:type_param*', None,
     'AsyncFunctionDef(identifier name, arguments args, stmt* body, expr* decorator_list, expr? returns, string? type_comment, type_param* type_params)'),
    ('AsyncWith', 'stmt', 'items:withitem* body:stmt* type_comment:str?', None,
     'AsyncWith(withitem* items, stmt* body, string? type_comment)'),
    ('expr', 'AST', '', 'lineno col_offset end_lineno end_col_offset',
     'expr = BoolOp(boolop op, expr* values)\n     | NamedExpr(expr target, expr value)\n     | BinOp(expr left, operator op, expr right)\n     | UnaryOp(unaryop op, expr operand)\n     | Lambda(arguments args, expr body)\n     | IfExp(expr test, expr body, expr orelse)\n     | Dict(expr* keys, expr* values)\n     | Set(expr* elts)\n     | ListComp(expr elt, comprehension* generators)\n     | SetComp(expr elt, comprehension* generators)\n     | DictComp(expr key, expr value, comprehension* generators)\n     | GeneratorExp(expr elt, comprehension* generators)\n     | Await(expr value)\n     | Yield(expr? value)\n     | YieldFrom(expr value)\n     | Compare(expr left, cmpop* ops, expr* comparators)\n     | Call(expr func, expr* args, keyword* keywords)\n     | FormattedValue(expr value, int conversion, expr? format_spec)\n     | JoinedStr(expr* values)\n     | Constant(constant value, string? kind)\n     | Attribute(expr value, identifier attr, expr_context ctx)\n     | Subscript(expr value, expr slice, expr_context ctx)\n     | Starred(expr value, expr_context ctx)\n     | Name(identifier id, expr_context ctx)\n     | List(expr* elts, expr_context ctx)\n     | Tuple(expr* elts, expr_context ctx)\n     | Slice(expr? lower, expr? upper, expr? step)'),
    ('Attribute', 'expr', 'value:expr attr:str ctx:expr_context', None,
     'Attribute(expr value, identifier attr, expr_context ctx)'),
    ('AugAssign', 'stmt', 'target:expr op:operator value:expr', None,
     'AugAssign(expr target, operator op, expr value)'),
    ('Await', 'expr', 'value:expr', None,
     'Await(expr value)'),
    ('BinOp', 'expr', 'left:expr op:operator right:expr', None,
     'BinOp(expr left, operator op, expr right)'),
    ('BitAnd', 'operator', '', None,
     'BitAnd'),
    ('BitOr', 'operator', '', None,
     'BitOr'),
    ('BitXor', 'operator', '', None,
     'BitXor'),
    ('BoolOp', 'expr', 'op:boolop values:expr*', None,
     'BoolOp(boolop op, expr* values)'),
    ('Break', 'stmt', '', None,
     'Break'),
    ('Call', 'expr', 'func:expr args:expr* keywords:keyword*', None,
     'Call(expr func, expr* args, keyword* keywords)'),
    ('ClassDef', 'stmt', 'name:str bases:expr* keywords:keyword* body:stmt* decorator_list:expr* type_params:type_param*', None,
     'ClassDef(identifier name, expr* bases, keyword* keywords, stmt* body, expr* decorator_list, type_param* type_params)'),
    ('Compare', 'expr', 'left:expr ops:cmpop* comparators:expr*', None,
     'Compare(expr left, cmpop* ops, expr* comparators)'),
    ('Constant', 'expr', 'value:object kind:str?', None,
     'Constant(constant value, string? kind)'),
    ('Continue', 'stmt', '', None,
     'Continue'),
    ('expr_context', 'AST', '', '',
     'expr_context = Load | Store | Del'),
    ('Del', 'expr_context', '', None,
     'Del'),
    ('Delete', 'stmt', 'targets:expr*', None,
     'Delete(expr* targets)'),
    ('Dict', 'expr', 'keys:expr* values:expr*', None,
     'Dict(expr* keys, expr* values)'),
    ('DictComp', 'expr', 'key:expr value:expr generators:comprehension*', None,
     'DictComp(expr key, expr value, comprehension* generators)'),
    ('Div', 'operator', '', None,
     'Div'),
    ('cmpop', 'AST', '', '',
     'cmpop = Eq | NotEq | Lt | LtE | Gt | GtE | Is | IsNot | In | NotIn'),
    ('Eq', 'cmpop', '', None,
     'Eq'),
    ('excepthandler', 'AST', '', 'lineno col_offset end_lineno end_col_offset',
     'excepthandler = ExceptHandler(expr? type, identifier? name, stmt* body)'),
    ('ExceptHandler', 'excepthandler', 'type:expr? name:str? body:stmt*', None,
     'ExceptHandler(expr? type, identifier? name, stmt* body)'),
    ('Expr', 'stmt', 'value:expr', None,
     'Expr(expr value)'),
    ('mod', 'AST', '', '',
     'mod = Module(stmt* body, type_ignore* type_ignores)\n    | Interactive(stmt* body)\n    | Expression(expr body)\n    | FunctionType(expr* argtypes, expr returns)'),
    ('Expression', 'mod', 'body:expr', None,
     'Expression(expr body)'),
    ('FloorDiv', 'operator', '', None,
     'FloorDiv'),
    ('For', 'stmt', 'target:expr iter:expr body:stmt* orelse:stmt* type_comment:str?', None,
     'For(expr target, expr iter, stmt* body, stmt* orelse, string? type_comment)'),
    ('FormattedValue', 'expr', 'value:expr conversion:int format_spec:expr?', None,
     'FormattedValue(expr value, int conversion, expr? format_spec)'),
    ('FunctionDef', 'stmt', 'name:str args:arguments body:stmt* decorator_list:expr* returns:expr? type_comment:str? type_params:type_param*', None,
     'FunctionDef(identifier name, arguments args, stmt* body, expr* decorator_list, expr? returns, string? type_comment, type_param* type_params)'),
    ('FunctionType', 'mod', 'argtypes:expr* returns:expr', None,
     'FunctionType(expr* argtypes, expr returns)'),
    ('GeneratorExp', 'expr', 'elt:expr generators:comprehension*', None,
     'GeneratorExp(expr elt, comprehension* generators)'),
    ('Global', 'stmt', 'names:str*', None,
     'Global(identifier* names)'),
    ('Gt', 'cmpop', '', None,
     'Gt'),
    ('GtE', 'cmpop', '', None,
     'GtE'),
    ('If', 'stmt', 'test:expr body:stmt* orelse:stmt*', None,
     'If(expr test, stmt* body, stmt* orelse)'),
    ('IfExp', 'expr', 'test:expr body:expr orelse:expr', None,
     'IfExp(expr test, expr body, expr orelse)'),
    ('Import', 'stmt', 'names:alias*', None,
     'Import(alias* names)'),
    ('ImportFrom', 'stmt', 'module:str? names:alias* level:int?', None,
     'ImportFrom(identifier? module, alias* names, int? level)'),
    ('In', 'cmpop', '', None,
     'In'),
    ('Interactive', 'mod', 'body:stmt*', None,
     'Interactive(stmt* body)'),
    ('unaryop', 'AST', '', '',
     'unaryop = Invert | Not | UAdd | USub'),
    ('Invert', 'unaryop', '', None,
     'Invert'),
    ('Is', 'cmpop', '', None,
     'Is'),
    ('IsNot', 'cmpop', '', None,
     'IsNot'),
    ('JoinedStr', 'expr', 'values:expr*', None,
     'JoinedStr(expr* values)'),
    ('LShift', 'operator', '', None,
     'LShift'),
    ('Lambda', 'expr', 'args:arguments body:expr', None,
     'Lambda(arguments args, expr body)'),
    ('List', 'expr', 'elts:expr* ctx:expr_context', None,
     'List(expr* elts, expr_context ctx)'),
    ('ListComp', 'expr', 'elt:expr generators:comprehension*', None,
     'ListComp(expr elt, comprehension* generators)'),
    ('Load', 'expr_context', '', None,
     'Load'),
    ('Lt', 'cmpop', '', None,
     'Lt'),
    ('LtE', 'cmpop', '', None,
     'LtE'),
    ('MatMult', 'operator', '', None,
     'MatMult'),
    ('Match', 'stmt', 'subject:expr cases:match_case*', None,
     'Match(expr subject, match_case* cases)'),
    ('pattern', 'AST', '', 'lineno col_offset end_lineno end_col_offset',
     'pattern = MatchValue(expr value)\n        | MatchSingleton(constant value)\n        | MatchSequence(pattern* patterns)\n        | MatchMapping(expr* keys, pattern* patterns, identifier? rest)\n        | MatchClass(expr cls, pattern* patterns, identifier* kwd_attrs, pattern* kwd_patterns)\n        | MatchStar(identifier? name)\n        | MatchAs(pattern? pattern, identifier? name)\n        | MatchOr(pattern* patterns)'),
    ('MatchAs', 'pattern', 'pattern:pattern? name:str?', None,
     'MatchAs(pattern? pattern, identifier? name)'),
    ('MatchClass', 'pattern', 'cls:expr patterns:pattern* kwd_attrs:str* kwd_patterns:pattern*', None,
     'MatchClass(expr cls, pattern* patterns, identifier* kwd_attrs, pattern* kwd_patterns)'),
    ('MatchMapping', 'pattern', 'keys:expr* patterns:pattern* rest:str?', None,
     'MatchMapping(expr* keys, pattern* patterns, identifier? rest)'),
    ('MatchOr', 'pattern', 'patterns:pattern*', None,
     'MatchOr(pattern* patterns)'),
    ('MatchSequence', 'pattern', 'patterns:pattern*', None,
     'MatchSequence(pattern* patterns)'),
    ('MatchSingleton', 'pattern', 'value:object', None,
     'MatchSingleton(constant value)'),
    ('MatchStar', 'pattern', 'name:str?', None,
     'MatchStar(identifier? name)'),
    ('MatchValue', 'pattern', 'value:expr', None,
     'MatchValue(expr value)'),
    ('Mod', 'operator', '', None,
     'Mod'),
    ('Module', 'mod', 'body:stmt* type_ignores:type_ignore*', None,
     'Module(stmt* body, type_ignore* type_ignores)'),
    ('Mult', 'operator', '', None,
     'Mult'),
    ('Name', 'expr', 'id:str ctx:expr_context', None,
     'Name(identifier id, expr_context ctx)'),
    ('NamedExpr', 'expr', 'target:expr value:expr', None,
     'NamedExpr(expr target, expr value)'),
    ('Nonlocal', 'stmt', 'names:str*', None,
     'Nonlocal(identifier* names)'),
    ('Not', 'unaryop', '', None,
     'Not'),
    ('NotEq', 'cmpop', '', None,
     'NotEq'),
    ('NotIn', 'cmpop', '', None,
     'NotIn'),
    ('Or', 'boolop', '', None,
     'Or'),
    ('type_param', 'AST', '', 'lineno col_offset end_lineno end_col_offset',
     'type_param = TypeVar(identifier name, expr? bound, expr? default_value)\n           | ParamSpec(identifier name, expr? default_value)\n           | TypeVarTuple(identifier name, expr? default_value)'),
    ('ParamSpec', 'type_param', 'name:str default_value:expr?', None,
     'ParamSpec(identifier name, expr? default_value)'),
    ('Pass', 'stmt', '', None,
     'Pass'),
    ('Pow', 'operator', '', None,
     'Pow'),
    ('RShift', 'operator', '', None,
     'RShift'),
    ('Raise', 'stmt', 'exc:expr? cause:expr?', None,
     'Raise(expr? exc, expr? cause)'),
    ('Return', 'stmt', 'value:expr?', None,
     'Return(expr? value)'),
    ('Set', 'expr', 'elts:expr*', None,
     'Set(expr* elts)'),
    ('SetComp', 'expr', 'elt:expr generators:comprehension*', None,
     'SetComp(expr elt, comprehension* generators)'),
    ('Slice', 'expr', 'lower:expr? upper:expr? step:expr?', None,
     'Slice(expr? lower, expr? upper, expr? step)'),
    ('Starred', 'expr', 'value:expr ctx:expr_context', None,
     'Starred(expr value, expr_context ctx)'),
    ('Store', 'expr_context', '', None,
     'Store'),
    ('Sub', 'operator', '', None,
     'Sub'),
    ('Subscript', 'expr', 'value:expr slice:expr ctx:expr_context', None,
     'Subscript(expr value, expr slice, expr_context ctx)'),
    ('Try', 'stmt', 'body:stmt* handlers:excepthandler* orelse:stmt* finalbody:stmt*', None,
     'Try(stmt* body, excepthandler* handlers, stmt* orelse, stmt* finalbody)'),
    ('TryStar', 'stmt', 'body:stmt* handlers:excepthandler* orelse:stmt* finalbody:stmt*', None,
     'TryStar(stmt* body, excepthandler* handlers, stmt* orelse, stmt* finalbody)'),
    ('Tuple', 'expr', 'elts:expr* ctx:expr_context', None,
     'Tuple(expr* elts, expr_context ctx)'),
    ('TypeAlias', 'stmt', 'name:expr type_params:type_param* value:expr', None,
     'TypeAlias(expr name, type_param* type_params, expr value)'),
    ('type_ignore', 'AST', '', '',
     'type_ignore = TypeIgnore(int lineno, string tag)'),
    ('TypeIgnore', 'type_ignore', 'lineno:int tag:str', None,
     'TypeIgnore(int lineno, string tag)'),
    ('TypeVar', 'type_param', 'name:str bound:expr? default_value:expr?', None,
     'TypeVar(identifier name, expr? bound, expr? default_value)'),
    ('TypeVarTuple', 'type_param', 'name:str default_value:expr?', None,
     'TypeVarTuple(identifier name, expr? default_value)'),
    ('UAdd', 'unaryop', '', None,
     'UAdd'),
    ('USub', 'unaryop', '', None,
     'USub'),
    ('UnaryOp', 'expr', 'op:unaryop operand:expr', None,
     'UnaryOp(unaryop op, expr operand)'),
    ('While', 'stmt', 'test:expr body:stmt* orelse:stmt*', None,
     'While(expr test, stmt* body, stmt* orelse)'),
    ('With', 'stmt', 'items:withitem* body:stmt* type_comment:str?', None,
     'With(withitem* items, stmt* body, string? type_comment)'),
    ('Yield', 'expr', 'value:expr?', None,
     'Yield(expr? value)'),
    ('YieldFrom', 'expr', 'value:expr', None,
     'YieldFrom(expr value)'),
    ('alias', 'AST', 'name:str asname:str?', 'lineno col_offset end_lineno end_col_offset',
     'alias(identifier name, identifier? asname)'),
    ('arg', 'AST', 'arg:str annotation:expr? type_comment:str?', 'lineno col_offset end_lineno end_col_offset',
     'arg(identifier arg, expr? annotation, string? type_comment)'),
    ('arguments', 'AST', 'posonlyargs:arg* args:arg* vararg:arg? kwonlyargs:arg* kw_defaults:expr* kwarg:arg? defaults:expr*', '',
     'arguments(arg* posonlyargs, arg* args, arg? vararg, arg* kwonlyargs, expr* kw_defaults, arg? kwarg, expr* defaults)'),
    ('comprehension', 'AST', 'target:expr iter:expr ifs:expr* is_async:int', '',
     'comprehension(expr target, expr iter, expr* ifs, int is_async)'),
    ('keyword', 'AST', 'arg:str? value:expr', 'lineno col_offset end_lineno end_col_offset',
     'keyword(identifier? arg, expr value)'),
    ('match_case', 'AST', 'pattern:pattern guard:expr? body:stmt*', '',
     'match_case(pattern pattern, expr? guard, stmt* body)'),
    ('withitem', 'AST', 'context_expr:expr optional_vars:expr?', '',
     'withitem(expr context_expr, expr? optional_vars)'),
]

_builtin_types = {"str": str, "int": int, "object": object}
# (class, field) -> its ASDL type, for the defaults of omitted fields
_kinds = {}


def _make():
    made = {"AST": AST}
    for name, base, fields, attrs, doc in _table:
        ns = {"__module__": "ast", "__doc__": doc}
        specs = [f.split(":") for f in fields.split()]
        ns["_fields"] = tuple(f for f, _ in specs)
        ns["__match_args__"] = ns["_fields"]
        if attrs is not None:
            ns["_attributes"] = tuple(attrs.split())
            for a in ns["_attributes"]:
                if a.startswith("end_"):
                    ns[a] = None
        for f, kind in specs:
            if kind.endswith("?"):
                ns[f] = None
        cls = type(name, (made[base],), ns)
        for f, kind in specs:
            _kinds[cls, f] = kind
        made[name] = cls
    for cls in made.values():
        types = {}
        for f in cls.__dict__.get("_fields", ()):
            kind = _kinds[cls, f]
            t = made.get(kind.rstrip("*?")) or _builtin_types[kind.rstrip("*?")]
            if kind.endswith("*"):
                t = list[t]
            elif kind.endswith("?"):
                t = t | None
            types[f] = t
        if cls is not AST and "_fields" in cls.__dict__:
            cls._field_types = types
            cls.__annotations__ = types
    AST.__module__ = "ast"
    globals().update(made)


_make()
del _make
