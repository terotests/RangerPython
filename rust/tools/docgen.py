# generates rust/src/docs.txt, the docstrings and text signatures of the
# built-in functions, types and methods, from this Python (3.13):
#   python3.13 tools/docgen.py > src/docs.txt
# One record per object: name, \x1f, __text_signature__ (or nothing), \x1f,
# __doc__, \x1e; a method's name is "type.method".
import builtins, math, sys, time, types

assert sys.version_info[:2] == (3, 13)

out = []


def add(key, obj):
    doc = getattr(obj, "__doc__", None)
    if not isinstance(doc, str):
        return
    sig = getattr(obj, "__text_signature__", None) or ""
    assert "\x1e" not in doc and "\x1f" not in doc
    out.append("%s\x1f%s\x1f%s\x1e" % (key, sig, doc))


def add_type(t):
    name = t.__name__
    add(name, t)
    for k, v in sorted(vars(t).items()):
        if k == "__doc__":
            continue
        if isinstance(v, (classmethod, staticmethod)):
            v = v.__func__
        if callable(v) or isinstance(v, (types.MethodDescriptorType, types.WrapperDescriptorType)):
            add("%s.%s" % (name, k), v)


extra_types = [type(None), type(NotImplemented), type(Ellipsis), types.FunctionType, types.CodeType,
               types.GeneratorType, types.CoroutineType, types.AsyncGeneratorType, types.MethodType,
               types.BuiltinFunctionType, types.SimpleNamespace, types.MappingProxyType, types.ModuleType,
               types.CellType, types.TracebackType, types.FrameType, types.WrapperDescriptorType,
               types.MethodDescriptorType, types.ClassMethodDescriptorType, types.MethodWrapperType,
               types.GetSetDescriptorType, types.MemberDescriptorType]
for name, obj in sorted(vars(builtins).items()):
    if isinstance(obj, types.BuiltinFunctionType):
        add(name, obj)
    elif isinstance(obj, type):
        add_type(obj)
for t in extra_types:
    add_type(t)
for m in (math, time, sys):
    for name, obj in sorted(vars(m).items()):
        if isinstance(obj, types.BuiltinFunctionType):
            add("%s.%s" % (m.__name__, name), obj)
sys.stdout.write("".join(out))
