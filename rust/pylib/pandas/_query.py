# SPDX-License-Identifier: MIT
# DataFrame.query / DataFrame.eval: a small interpreter over Python's AST in
# which `and`, `or`, `not` and `in` work elementwise on columns.

import ast
import re
import operator

_BIN = {ast.Add: operator.add, ast.Sub: operator.sub, ast.Mult: operator.mul,
        ast.Div: operator.truediv, ast.FloorDiv: operator.floordiv,
        ast.Mod: operator.mod, ast.Pow: operator.pow, ast.BitAnd: operator.and_,
        ast.BitOr: operator.or_, ast.BitXor: operator.xor}
_CMP = {ast.Eq: operator.eq, ast.NotEq: operator.ne, ast.Lt: operator.lt,
        ast.LtE: operator.le, ast.Gt: operator.gt, ast.GtE: operator.ge}


def evaluate(expr, df, local_vars=None, global_vars=None):
    names = {}

    def tick(m):
        key = "__col_%d" % len(names)
        names[key] = m.group(1)
        return key
    src = re.sub(r"`([^`]*)`", tick, expr.strip())
    src = re.sub(r"@([A-Za-z_]\w*)", r"__at_\1", src)
    tree = ast.parse(src, mode="eval")
    env = _Env(df, names, local_vars or {}, global_vars or {})
    return _eval(tree.body, env)


class _Env:
    def __init__(self, df, names, lv, gv):
        self.df = df
        self.names = names
        self.lv = lv
        self.gv = gv

    def lookup(self, name):
        if name.startswith("__at_"):
            n = name[5:]
            if n in self.lv:
                return self.lv[n]
            if n in self.gv:
                return self.gv[n]
            raise NameError(n)
        if name in self.names:
            return self.df[self.names[name]]
        if name in self.df._data:
            return self.df._col(name)
        if name == "index" or name in self.df.index.names:
            return self.df.index.to_series() if name == "index" or self.df.index.nlevels == 1 else self.df.index.get_level_values(name).to_series()
        if name in self.lv:
            return self.lv[name]
        if name in self.gv:
            return self.gv[name]
        import builtins
        if hasattr(builtins, name):
            return getattr(builtins, name)
        raise NameError("name '%s' is not defined" % name)


def _isin(a, b):
    from ._series import Series
    if isinstance(a, Series):
        if isinstance(b, Series):
            b = b._values
        return a.isin(b if isinstance(b, (list, tuple, set)) else [b])
    return a in b


def _eval(node, env):
    from ._series import Series
    t = type(node)
    if t is ast.Constant:
        return node.value
    if t is ast.Name:
        return env.lookup(node.id)
    if t is ast.BoolOp:
        vals = [_eval(v, env) for v in node.values]
        f = operator.and_ if isinstance(node.op, ast.And) else operator.or_
        out = vals[0]
        for v in vals[1:]:
            out = f(out, v) if isinstance(out, Series) or isinstance(v, Series) else (out and v if f is operator.and_ else out or v)
        return out
    if t is ast.UnaryOp:
        v = _eval(node.operand, env)
        if isinstance(node.op, (ast.Not, ast.Invert)):
            return ~v if isinstance(v, Series) else (not v)
        if isinstance(node.op, ast.USub):
            return -v
        return v
    if t is ast.BinOp:
        return _BIN[type(node.op)](_eval(node.left, env), _eval(node.right, env))
    if t is ast.Compare:
        left = _eval(node.left, env)
        result = None
        for op, comp in zip(node.ops, node.comparators):
            right = _eval(comp, env)
            if isinstance(op, ast.In):
                r = _isin(left, right)
            elif isinstance(op, ast.NotIn):
                r = _isin(left, right)
                r = ~r if isinstance(r, Series) else not r
            else:
                r = _CMP[type(op)](left, right)
            result = r if result is None else (result & r)
            left = right
        return result
    if t is ast.Attribute:
        return getattr(_eval(node.value, env), node.attr)
    if t is ast.Call:
        f = _eval(node.func, env)
        args = [_eval(a, env) for a in node.args]
        kw = {k.arg: _eval(k.value, env) for k in node.keywords}
        return f(*args, **kw)
    if t is ast.Subscript:
        return _eval(node.value, env)[_eval(node.slice, env)]
    if t in (ast.List, ast.Tuple, ast.Set):
        vals = [_eval(e, env) for e in node.elts]
        return tuple(vals) if t is ast.Tuple else vals
    if t is ast.IfExp:
        return _eval(node.body, env) if _eval(node.test, env) else _eval(node.orelse, env)
    raise ValueError("unsupported expression in query: %s" % t.__name__)
