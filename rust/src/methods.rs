// SPDX-License-Identifier: MIT
//! Methods of the built-in types (str and bytes are in strmethods.rs).

use crate::ast::{BinOp, CmpOp, UnOp};
use crate::bigint::BigInt;
use crate::builtins::*;
use crate::value::*;
use crate::vm::Vm;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

/// a built-in value of `cls`, or an instance of a subclass wrapping it
pub fn wrap_new(vm: &mut Vm, cls: &Value, builtin: &Rc<Class>, v: Value) -> Value {
    match cls {
        Value::Class(c) if !Rc::ptr_eq(c, builtin) => {
            let inst = vm.new_instance(c);
            if let Value::Instance(i) = &inst {
                *i.base.borrow_mut() = v;
            }
            inst
        }
        _ => v,
    }
}

fn this(vm: &Vm, a: &[Value]) -> Value {
    vm.base_of(&a[0])
}

macro_rules! binop_dunders {
    ($( $fname:ident, $rname:ident, $op:expr );* $(;)?) => {
        $(
            fn $fname(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
                margs(vm, stringify!($fname), a, 1, 1)?;
                let x = vm.base_of(&a[0]);
                let y = vm.base_of(&a[1]);
                if matches!(y, Value::Instance(_)) { return Ok(Value::NotImplemented); }
                Ok(vm.builtin_binop($op, &x, &y)?.unwrap_or(Value::NotImplemented))
            }
            fn $rname(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
                margs(vm, stringify!($rname), a, 1, 1)?;
                let x = vm.base_of(&a[0]);
                let y = vm.base_of(&a[1]);
                if matches!(y, Value::Instance(_)) { return Ok(Value::NotImplemented); }
                Ok(vm.builtin_binop($op, &y, &x)?.unwrap_or(Value::NotImplemented))
            }
        )*
    };
}

binop_dunders! {
    d_add, d_radd, BinOp::Add;
    d_sub, d_rsub, BinOp::Sub;
    d_mul, d_rmul, BinOp::Mul;
    d_truediv, d_rtruediv, BinOp::Div;
    d_floordiv, d_rfloordiv, BinOp::FloorDiv;
    d_mod, d_rmod, BinOp::Mod;
    d_pow, d_rpow, BinOp::Pow;
    d_lshift, d_rlshift, BinOp::LShift;
    d_rshift, d_rrshift, BinOp::RShift;
    d_and, d_rand, BinOp::BitAnd;
    d_or, d_ror, BinOp::BitOr;
    d_xor, d_rxor, BinOp::BitXor;
}

macro_rules! cmp_dunders {
    ($( $fname:ident, $op:expr );* $(;)?) => {
        $(
            fn $fname(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
                margs(vm, stringify!($fname), a, 1, 1)?;
                let x = vm.base_of(&a[0]);
                let y = vm.base_of(&a[1]);
                if matches!(y, Value::Instance(_)) { return Ok(Value::NotImplemented); }
                match $op {
                    CmpOp::Eq => Ok(Value::Bool(vm.eq_builtin(&x, &y)?)),
                    CmpOp::Ne => Ok(Value::Bool(!vm.eq_builtin(&x, &y)?)),
                    op => match vm.order_builtin(&x, &y)? {
                        Some(o) => Ok(Value::Bool(match op {
                            CmpOp::Lt => o == std::cmp::Ordering::Less,
                            CmpOp::Le => o != std::cmp::Ordering::Greater,
                            CmpOp::Gt => o == std::cmp::Ordering::Greater,
                            _ => o != std::cmp::Ordering::Less,
                        })),
                        None => Ok(Value::NotImplemented),
                    },
                }
            }
        )*
    };
}

cmp_dunders! {
    d_eq, CmpOp::Eq;
    d_ne, CmpOp::Ne;
    d_lt, CmpOp::Lt;
    d_le, CmpOp::Le;
    d_gt, CmpOp::Gt;
    d_ge, CmpOp::Ge;
}

fn d_neg(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = this(vm, a);
    vm.unary(UnOp::Neg, &x)
}
fn d_pos(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = this(vm, a);
    vm.unary(UnOp::Pos, &x)
}
fn d_invert(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = this(vm, a);
    vm.unary(UnOp::Invert, &x)
}
fn d_abs(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let x = this(vm, a);
    let f = vm.builtins.borrow().get_s("abs").unwrap();
    vm.call_vec(&f, vec![x], k)
}
fn d_bool(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = this(vm, a);
    Ok(Value::Bool(vm.truthy(&x)?))
}
fn d_hash(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = this(vm, a);
    let h = vm.hash(&x)?;
    Ok(Value::Int(h as i64))
}
fn d_repr(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = this(vm, a);
    Ok(Value::string(vm.repr(&x)?))
}
fn d_str(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = this(vm, a);
    Ok(Value::string(vm.str_of(&x)?))
}
fn d_format(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    margs(vm, "__format__", a, 1, 1)?;
    let x = this(vm, a);
    let spec = as_pystr(vm, &a[1], "format spec")?;
    if spec.s.is_empty() {
        return Ok(Value::string(vm.str_of(&a[0])?));
    }
    Ok(Value::string(crate::format::format_spec(vm, &x, &spec.s)?))
}
fn d_len(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = this(vm, a);
    Ok(Value::Int(vm.len(&x)? as i64))
}
fn d_iter(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = this(vm, a);
    vm.iter(&x)
}
fn d_contains(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    margs(vm, "__contains__", a, 1, 1)?;
    let x = this(vm, a);
    Ok(Value::Bool(vm.contains(&x, &a[1])?))
}
fn d_getitem(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    margs(vm, "__getitem__", a, 1, 1)?;
    let x = this(vm, a);
    vm.getitem(&x, &a[1])
}
fn d_setitem(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    margs(vm, "__setitem__", a, 2, 2)?;
    let x = this(vm, a);
    vm.setitem(&x, a[1].clone(), a[2].clone())?;
    Ok(Value::None)
}
fn d_delitem(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    margs(vm, "__delitem__", a, 1, 1)?;
    let x = this(vm, a);
    vm.delitem(&x, &a[1])?;
    Ok(Value::None)
}
fn d_iadd(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 2)?;
    let x = this(vm, a);
    vm.inplace(BinOp::Add, &x, &a[1])?;
    Ok(a[0].clone())
}
fn d_imul(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 2)?;
    let x = this(vm, a);
    let r = vm.inplace(BinOp::Mul, &x, &a[1])?;
    if let (Value::List(l), Value::List(r)) = (&x, &r) {
        if !Rc::ptr_eq(l, r) {
            let v = r.borrow().clone();
            *l.borrow_mut() = v;
        }
    }
    Ok(a[0].clone())
}
fn d_divmod(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 2)?;
    let x = this(vm, a);
    let y = vm.base_of(&a[1]);
    builtin_divmod(vm, &x, &y)
}
fn d_rdivmod(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 2)?;
    let x = this(vm, a);
    let y = vm.base_of(&a[1]);
    builtin_divmod(vm, &y, &x)
}
fn builtin_divmod(vm: &mut Vm, x: &Value, y: &Value) -> PyResult<Value> {
    if matches!(x, Value::Instance(_)) || matches!(y, Value::Instance(_)) {
        return Ok(Value::NotImplemented);
    }
    let q = match vm.builtin_binop(BinOp::FloorDiv, x, y)? {
        Some(q) => q,
        None => return Ok(Value::NotImplemented),
    };
    let r = match vm.builtin_binop(BinOp::Mod, x, y)? {
        Some(r) => r,
        None => return Ok(Value::NotImplemented),
    };
    Ok(Value::tuple(vec![q, r]))
}
/// seq.__mul__(n) and seq.__rmul__(n): n is anything with __index__
/// __buffer__(flags): the buffer protocol hands out a memoryview
fn bytes_buffer(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 2)?;
    vm.to_index(&a[1])?;
    let mv = vm.builtins.borrow().get_s("memoryview").unwrap();
    vm.call_vec(&mv, vec![a[0].clone()], &[])
}

fn seq_mul(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    margs(vm, "__mul__", a, 1, 1)?;
    let x = vm.base_of(&a[0]);
    Ok(vm.seq_repeat(&x, &a[1])?.unwrap_or(Value::NotImplemented))
}
fn not_implemented(_vm: &mut Vm, _a: &[Value], _k: Kw) -> PyResult<Value> {
    Ok(Value::NotImplemented)
}
fn return_self(_vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    Ok(a[0].clone())
}
fn no_op(_vm: &mut Vm, _a: &[Value], _k: Kw) -> PyResult<Value> {
    Ok(Value::None)
}

fn def_arith(c: &Rc<Class>, t: &str) {
    let leak = |n: &str| -> &'static str { Box::leak(format!("{}.{}", t, n).into_boxed_str()) };
    let fs: &[(&str, NativeFnPtr)] = &[
        ("__add__", d_add),
        ("__radd__", d_radd),
        ("__sub__", d_sub),
        ("__rsub__", d_rsub),
        ("__mul__", d_mul),
        ("__rmul__", d_rmul),
        ("__truediv__", d_truediv),
        ("__rtruediv__", d_rtruediv),
        ("__floordiv__", d_floordiv),
        ("__rfloordiv__", d_rfloordiv),
        ("__mod__", d_mod),
        ("__rmod__", d_rmod),
        ("__pow__", d_pow),
        ("__rpow__", d_rpow),
        ("__divmod__", d_divmod),
        ("__rdivmod__", d_rdivmod),
        ("__neg__", d_neg),
        ("__pos__", d_pos),
        ("__abs__", d_abs),
        ("__bool__", d_bool),
    ];
    for (n, f) in fs {
        def(c, leak(n), *f);
    }
}

fn def_bits(c: &Rc<Class>, t: &str) {
    let leak = |n: &str| -> &'static str { Box::leak(format!("{}.{}", t, n).into_boxed_str()) };
    let fs: &[(&str, NativeFnPtr)] = &[
        ("__lshift__", d_lshift),
        ("__rlshift__", d_rlshift),
        ("__rshift__", d_rshift),
        ("__rrshift__", d_rrshift),
        ("__and__", d_and),
        ("__rand__", d_rand),
        ("__or__", d_or),
        ("__ror__", d_ror),
        ("__xor__", d_xor),
        ("__rxor__", d_rxor),
        ("__invert__", d_invert),
    ];
    for (n, f) in fs {
        def(c, leak(n), *f);
    }
}

fn def_cmp(c: &Rc<Class>, t: &str) {
    let leak = |n: &str| -> &'static str { Box::leak(format!("{}.{}", t, n).into_boxed_str()) };
    let fs: &[(&str, NativeFnPtr)] = &[("__eq__", d_eq), ("__ne__", d_ne), ("__lt__", d_lt), ("__le__", d_le), ("__gt__", d_gt), ("__ge__", d_ge), ("__hash__", d_hash), ("__repr__", d_repr), ("__str__", d_str)];
    for (n, f) in fs {
        def(c, leak(n), *f);
    }
}

fn def_seq(c: &Rc<Class>, t: &str) {
    let leak = |n: &str| -> &'static str { Box::leak(format!("{}.{}", t, n).into_boxed_str()) };
    let fs: &[(&str, NativeFnPtr)] = &[("__len__", d_len), ("__iter__", d_iter), ("__contains__", d_contains), ("__getitem__", d_getitem)];
    for (n, f) in fs {
        def(c, leak(n), *f);
    }
}

pub fn install_types(vm: &mut Vm) {
    let t = &vm.t;
    // object
    let o = t.object.clone();
    def(&o, "object.__new__", object_new);
    def(&o, "object.__init__", object_init);
    def(&o, "object.__repr__", object_repr);
    def(&o, "object.__str__", object_str);
    def(&o, "object.__eq__", object_eq);
    def(&o, "object.__ne__", object_ne);
    def(&o, "object.__lt__", not_implemented);
    def(&o, "object.__le__", not_implemented);
    def(&o, "object.__gt__", not_implemented);
    def(&o, "object.__ge__", not_implemented);
    def(&o, "object.__hash__", object_hash);
    def(&o, "object.__format__", object_format);
    def(&o, "object.__getattribute__", object_getattribute);
    def(&o, "object.__setattr__", object_setattr);
    def(&o, "object.__delattr__", object_delattr);
    def(&o, "object.__dir__", object_dir);
    def(&o, "object.__reduce_ex__", object_reduce_ex);
    def(&o, "object.__reduce__", object_reduce);
    def(&o, "object.__sizeof__", object_sizeof);
    def(&o, "object.__getstate__", object_getstate);
    def_class(&o, "object.__init_subclass__", object_init_subclass);
    def_class(&o, "object.__subclasshook__", not_implemented);
    // type
    let ty = t.type_.clone();
    def(&ty, "type.__new__", type_new);
    def(&ty, "type.__init__", no_op);
    def(&ty, "type.__call__", type_call);
    def(&ty, "type.mro", type_mro);
    def(&ty, "type.__subclasses__", type_subclasses);
    def(&ty, "type.__instancecheck__", type_instancecheck);
    def(&ty, "type.__subclasscheck__", type_subclasscheck);
    def_class(&ty, "type.__prepare__", type_prepare);
    def(&ty, "type.__or__", type_or);
    def(&ty, "type.__ror__", type_ror);
    // numbers
    let int = t.int.clone();
    def(&int, "int.__new__", int_new);
    def_arith(&int, "int");
    def_bits(&int, "int");
    def_cmp(&int, "int");
    def(&int, "int.__format__", d_format);
    def(&int, "int.__index__", int_int);
    def(&int, "int.__int__", int_int);
    def(&int, "int.__float__", int_float);
    def(&int, "int.__trunc__", int_int);
    def(&int, "int.__floor__", int_int);
    def(&int, "int.__ceil__", int_int);
    def(&int, "int.__round__", int_round);
    def(&int, "int.__getnewargs__", getnewargs);
    def(&int, "int.conjugate", int_int);
    def(&int, "int.bit_length", int_bit_length);
    def(&int, "int.__sizeof__", int_sizeof);
    def(&int, "int.bit_count", int_bit_count);
    def(&int, "int.to_bytes", int_to_bytes);
    def_class(&int, "int.from_bytes", int_from_bytes);
    def(&int, "int.as_integer_ratio", int_ratio);
    def(&int, "int.is_integer", true_fn);
    let b = t.bool_.clone();
    def(&b, "bool.__new__", bool_new);
    def(&b, "bool.__repr__", d_repr);
    def(&b, "bool.__str__", d_repr);
    let fl = t.float.clone();
    def(&fl, "float.__new__", float_new);
    def_arith(&fl, "float");
    def_cmp(&fl, "float");
    def(&fl, "float.__format__", d_format);
    def(&fl, "float.__int__", float_int);
    def(&fl, "float.__float__", float_float);
    def(&fl, "float.__trunc__", float_int);
    def(&fl, "float.__floor__", float_floor);
    def(&fl, "float.__ceil__", float_ceil);
    def(&fl, "float.__round__", float_round);
    def(&fl, "float.__getnewargs__", getnewargs);
    def(&fl, "float.is_integer", float_is_integer);
    def(&fl, "float.as_integer_ratio", float_ratio);
    def(&fl, "float.conjugate", float_float);
    def(&fl, "float.hex", float_hex);
    def_class(&fl, "float.fromhex", float_fromhex);
    def_class(&fl, "float.__getformat__", float_getformat);
    let cx = t.complex.clone();
    def(&cx, "complex.__new__", complex_new);
    def_arith(&cx, "complex");
    def_cmp(&cx, "complex");
    def(&cx, "complex.__format__", d_format);
    def(&cx, "complex.conjugate", complex_conjugate);
    // containers
    let l = t.list.clone();
    def(&l, "list.__new__", list_new);
    def(&l, "list.__init__", list_init);
    def_seq(&l, "list");
    def_cmp(&l, "list");
    l.dict.borrow_mut().set_s("__hash__", Value::None);
    def(&l, "list.__add__", d_add);
    def(&l, "list.__mul__", seq_mul);
    def(&l, "list.__rmul__", seq_mul);
    def(&l, "list.__iadd__", d_iadd);
    def(&l, "list.__imul__", d_imul);
    def(&l, "list.__setitem__", d_setitem);
    def(&l, "list.__delitem__", d_delitem);
    def(&l, "list.__reversed__", list_reversed);
    def(&l, "list.append", list_append);
    def(&l, "list.extend", list_extend);
    def(&l, "list.insert", list_insert);
    def(&l, "list.pop", list_pop);
    def(&l, "list.remove", list_remove);
    def(&l, "list.index", seq_index);
    def(&l, "list.count", seq_count);
    def(&l, "list.clear", list_clear);
    def(&l, "list.copy", list_copy);
    def(&l, "list.reverse", list_reverse);
    def(&l, "list.sort", list_sort);
    let tu = t.tuple.clone();
    def(&tu, "tuple.__new__", tuple_new);
    def_seq(&tu, "tuple");
    def_cmp(&tu, "tuple");
    def(&tu, "tuple.__add__", d_add);
    def(&tu, "tuple.__mul__", seq_mul);
    def(&tu, "tuple.__rmul__", seq_mul);
    def(&t.bytes.clone(), "bytes.__mul__", seq_mul);
    def(&t.bytes.clone(), "bytes.__rmul__", seq_mul);
    def(&t.bytearray.clone(), "bytearray.__mul__", seq_mul);
    def(&t.bytearray.clone(), "bytearray.__rmul__", seq_mul);
    def(&t.bytes.clone(), "bytes.__buffer__", bytes_buffer);
    def(&t.bytearray.clone(), "bytearray.__buffer__", bytes_buffer);
    def(&tu, "tuple.__getnewargs__", getnewargs);
    def(&tu, "tuple.index", seq_index);
    def(&tu, "tuple.count", seq_count);
    let d = t.dict.clone();
    def(&d, "dict.__new__", dict_new);
    def(&d, "dict.__init__", dict_init);
    def_seq(&d, "dict");
    def(&d, "dict.__eq__", d_eq);
    def(&d, "dict.__ne__", d_ne);
    def(&d, "dict.__repr__", d_repr);
    d.dict.borrow_mut().set_s("__hash__", Value::None);
    def(&d, "dict.__setitem__", d_setitem);
    def(&d, "dict.__delitem__", d_delitem);
    def(&d, "dict.__or__", d_or);
    def(&d, "dict.__ror__", d_ror);
    def(&d, "dict.__ior__", dict_ior);
    def(&d, "dict.__reversed__", dict_reversed);
    def(&d, "dict.keys", dict_keys);
    def(&d, "dict.values", dict_values);
    def(&d, "dict.items", dict_items);
    def(&d, "dict.get", dict_get);
    def(&d, "dict.setdefault", dict_setdefault);
    def(&d, "dict.pop", dict_pop);
    def(&d, "dict.popitem", dict_popitem);
    def(&d, "dict.update", dict_update);
    def(&d, "dict.clear", dict_clear);
    def(&d, "dict.copy", dict_copy);
    def_class(&d, "dict.fromkeys", dict_fromkeys);
    for (c, n) in [(t.set.clone(), "set"), (t.frozenset.clone(), "frozenset")] {
        let leak = |m: &str| -> &'static str { Box::leak(format!("{}.{}", n, m).into_boxed_str()) };
        def_seq(&c, n);
        def(&c, leak("__eq__"), d_eq);
        def(&c, leak("__ne__"), d_ne);
        def(&c, leak("__repr__"), set_repr);
        def(&c, leak("__or__"), d_or);
        def(&c, leak("__and__"), d_and);
        def(&c, leak("__sub__"), d_sub);
        def(&c, leak("__xor__"), d_xor);
        def(&c, leak("__ror__"), d_ror);
        def(&c, leak("__rand__"), d_rand);
        def(&c, leak("__rsub__"), d_rsub);
        def(&c, leak("__rxor__"), d_rxor);
        def(&c, leak("__le__"), set_le);
        def(&c, leak("__lt__"), set_lt);
        def(&c, leak("__ge__"), set_ge);
        def(&c, leak("__gt__"), set_gt);
        def(&c, leak("copy"), set_copy);
        def(&c, leak("union"), set_union);
        def(&c, leak("intersection"), set_intersection);
        def(&c, leak("difference"), set_difference);
        def(&c, leak("symmetric_difference"), set_symmetric_difference);
        def(&c, leak("issubset"), set_issubset);
        def(&c, leak("issuperset"), set_issuperset);
        def(&c, leak("isdisjoint"), set_isdisjoint);
        def(&c, leak("__reduce__"), set_reduce);
    }
    let s = t.set.clone();
    s.dict.borrow_mut().set_s("__hash__", Value::None);
    def(&s, "set.__new__", set_new);
    def(&s, "set.__init__", set_init);
    def(&s, "set.add", set_add);
    def(&s, "set.discard", set_discard);
    def(&s, "set.remove", set_remove);
    def(&s, "set.pop", set_pop);
    def(&s, "set.clear", set_clear);
    def(&s, "set.update", set_update);
    def(&s, "set.intersection_update", set_intersection_update);
    def(&s, "set.difference_update", set_difference_update);
    def(&s, "set.symmetric_difference_update", set_symmetric_difference_update);
    def(&s, "set.__ior__", set_ior);
    def(&s, "set.__iand__", set_iand);
    def(&s, "set.__isub__", set_isub);
    def(&s, "set.__ixor__", set_ixor);
    let fs = t.frozenset.clone();
    def(&fs, "frozenset.__new__", frozenset_new);
    def(&fs, "frozenset.__hash__", d_hash);
    for (c, n) in [(t.list.clone(), "list"), (t.tuple.clone(), "tuple"), (t.dict.clone(), "dict"), (t.set.clone(), "set"), (t.frozenset.clone(), "frozenset")] {
        def_class(&c, Box::leak(format!("{}.__class_getitem__", n).into_boxed_str()), class_getitem);
    }
    // the positional arguments each container method takes
    let seq0: &[&str] = &["__len__", "__reduce__", "__iter__", "__reversed__", "__repr__", "__hash__", "__getnewargs__", "clear", "copy", "reverse", "keys", "values", "items", "popitem"];
    let seq1: &[&str] = &["__contains__", "__getitem__", "__delitem__", "__add__", "__mul__", "__rmul__", "__iadd__", "__imul__", "__eq__", "__ne__", "__lt__", "__le__", "__gt__", "__ge__", "__or__", "__ror__", "__ior__", "__and__", "__rand__", "__iand__", "__sub__", "__rsub__", "__isub__", "__xor__", "__rxor__", "__ixor__", "append", "extend", "remove", "count", "add", "discard", "isdisjoint", "issubset", "issuperset"];
    let other: &[(&str, u8, u8)] = &[("__setitem__", 2, 2), ("insert", 2, 2), ("index", 1, 3), ("get", 1, 2), ("setdefault", 1, 2), ("update", 0, 1)];
    for c in [t.list.clone(), t.tuple.clone(), t.dict.clone(), t.set.clone(), t.frozenset.clone()] {
        let is_dict = Rc::ptr_eq(&c, &t.dict);
        let mut d = c.dict.borrow_mut();
        let names: Vec<(Rc<PyStr>, Value)> = d
            .iter()
            .filter_map(|e| match &e.key {
                Value::Str(s) => Some((s.clone(), e.value.clone())),
                _ => None,
            })
            .collect();
        for (k, v) in names {
            let Value::Native(n) = v else { continue };
            let ar = if seq0.contains(&&*k.s) {
                (0, 0)
            } else if seq1.contains(&&*k.s) {
                (1, 1)
            } else if &*k.s == "update" && !is_dict {
                continue;
            } else if &*k.s == "pop" {
                if is_dict { (1, 2) } else if Rc::ptr_eq(&c, &t.set) { (0, 0) } else { (0, 1) }
            } else if let Some((_, lo, hi)) = other.iter().find(|x| x.0 == &*k.s) {
                (*lo, *hi)
            } else {
                continue;
            };
            let mut f = NativeFn::new(n.name, n.f, n.data.clone());
            f.bind = n.bind;
            f.arity = Some(ar);
            d.set_str(k, Value::Native(Rc::new(f)));
        }
    }
    let r = t.range.clone();
    def(&r, "range.__new__", range_new);
    def(&r, "range.__reduce__", range_reduce);
    def_seq(&r, "range");
    def(&r, "range.__eq__", d_eq);
    def(&r, "range.__ne__", d_ne);
    def(&r, "range.__hash__", d_hash);
    def(&r, "range.__repr__", d_repr);
    def(&r, "range.__reversed__", range_reversed);
    def(&r, "range.index", seq_index);
    def(&r, "range.count", seq_count);
    let sl = t.slice.clone();
    def(&sl, "slice.__new__", slice_new);
    def(&sl, "slice.indices", slice_indices_m);
    def(&sl, "slice.__eq__", d_eq);
    def(&sl, "slice.__repr__", d_repr);
    def(&sl, "slice.__hash__", slice_hash);
    def(&sl, "slice.__reduce__", slice_reduce);
    let n = t.none.clone();
    def(&n, "NoneType.__new__", none_new);
    def(&n, "NoneType.__bool__", false_fn);
    def(&n, "NoneType.__repr__", d_repr);
    def(&n, "NotImplementedType.__repr__", d_repr);
    def(&t.notimpl, "NotImplementedType.__new__", notimpl_new);
    def(&t.ellipsis, "ellipsis.__new__", ellipsis_new);
    let f = t.function.clone();
    def_static(&f, "function.__new__", function_new);
    def_static(&t.cell.clone(), "cell.__new__", cell_new);
    def(&f, "function.__get__", function_get);
    def(&f, "function.__repr__", d_repr);
    def(&f, "function.__call__", function_call);
    def(&t.code, "code.replace", code_replace);
    def(&t.code, "code.__replace__", code_replace);
    def(&t.code, "code.co_positions", code_positions);
    def(&t.code, "code.co_lines", code_lines);
    let m = t.method.clone();
    def_static(&m, "method.__new__", method_new);
    def(&m, "method.__call__", function_call);
    def(&m, "method.__get__", return_self);
    let bf = t.builtin_function.clone();
    def(&bf, "builtin_function_or_method.__call__", function_call);
    let md = t.module.clone();
    def(&md, "module.__new__", module_new);
    def(&md, "module.__init__", no_op);
    def(&md, "module.__repr__", d_repr);
    def(&md, "module.__dir__", object_dir);
    let it = t.iterator.clone();
    def(&t.enumerate.clone(), "enumerate.__new__", enumerate_new);
    def(&t.zip.clone(), "zip.__new__", zip_new);
    def(&t.map.clone(), "map.__new__", map_new);
    def(&t.filter.clone(), "filter.__new__", filter_new);
    def(&t.reversed.clone(), "reversed.__new__", reversed_new);
    def(&it, "iterator.__new__", iterator_new);
    def(&it, "iterator.__iter__", return_self);
    def(&it, "iterator.__next__", iter_next);
    def(&it, "iterator.__length_hint__", iter_length_hint);
    def(&it, "iterator.__reduce__", iter_reduce);
    let bf = t.builtin_function.clone();
    def(&bf, "builtin_function_or_method.__reduce__", native_reduce);
    def(&bf, "builtin_function_or_method.__reduce_ex__", native_reduce);
    def(&it, "iterator.__setstate__", iter_setstate);
    for g in [t.generator.clone(), t.coroutine.clone()] {
        let nm = g.name();
        let leak = |x: &str| -> &'static str { Box::leak(format!("{}.{}", nm, x).into_boxed_str()) };
        def(&g, leak("__iter__"), return_self);
        def(&g, leak("__next__"), iter_next);
        def(&g, leak("send"), gen_send);
        def(&g, leak("throw"), gen_throw);
        def(&g, leak("close"), gen_close);
    }
    def(&t.coroutine.clone(), "coroutine.__await__", return_self);
    for (c, nm) in [(t.dict_keys.clone(), "dict_keys"), (t.dict_values.clone(), "dict_values"), (t.dict_items.clone(), "dict_items")] {
        let leak = |x: &str| -> &'static str { Box::leak(format!("{}.{}", nm, x).into_boxed_str()) };
        def(&c, leak("__len__"), d_len);
        def(&c, leak("__iter__"), d_iter);
        def(&c, leak("__contains__"), d_contains);
        def(&c, leak("__reversed__"), view_reversed);
        def(&c, leak("__repr__"), d_repr);
        def(&c, leak("__new__"), cannot_create);
        def(&c, leak("__reduce_ex__"), cannot_pickle);
        def(&c, leak("__reduce__"), cannot_pickle);
        if nm != "dict_values" {
            def(&c, leak("isdisjoint"), set_isdisjoint);
            def(&c, leak("__eq__"), d_eq);
            def(&c, leak("__ne__"), d_ne);
            def(&c, leak("__le__"), set_le);
            def(&c, leak("__lt__"), set_lt);
            def(&c, leak("__ge__"), set_ge);
            def(&c, leak("__gt__"), set_gt);
            def(&c, leak("__or__"), d_or);
            def(&c, leak("__and__"), d_and);
            def(&c, leak("__sub__"), d_sub);
            def(&c, leak("__xor__"), d_xor);
            def(&c, leak("__ror__"), d_ror);
            def(&c, leak("__rand__"), d_rand);
            def(&c, leak("__rsub__"), d_rsub);
            def(&c, leak("__rxor__"), d_rxor);
            c.dict.borrow_mut().set_s("__hash__", Value::None);
        }
    }
    let p = t.property.clone();
    def(&p, "property.__new__", property_new);
    def(&p, "property.__init__", property_init);
    def(&p, "property.__get__", property_get);
    def(&p, "property.__set__", property_set);
    def(&p, "property.__delete__", property_delete);
    def(&p, "property.getter", property_getter);
    def(&p, "property.setter", property_setter);
    def(&p, "property.deleter", property_deleter);
    def(&p, "property.__set_name__", property_set_name);
    let sm = t.staticmethod.clone();
    def(&sm, "staticmethod.__new__", staticmethod_new);
    def(&sm, "staticmethod.__init__", no_op);
    def(&sm, "staticmethod.__get__", staticmethod_get);
    def(&sm, "staticmethod.__call__", staticmethod_call);
    let cm = t.classmethod.clone();
    def(&cm, "classmethod.__new__", classmethod_new);
    def(&cm, "classmethod.__init__", no_op);
    def(&cm, "classmethod.__get__", classmethod_get);
    let su = t.super_.clone();
    def(&su, "super.__new__", super_new);
    def(&su, "super.__init__", no_op);
    def(&su, "super.__repr__", d_repr);
    crate::strmethods::install(vm);
}

fn true_fn(_vm: &mut Vm, _a: &[Value], _k: Kw) -> PyResult<Value> {
    Ok(Value::Bool(true))
}
fn false_fn(_vm: &mut Vm, _a: &[Value], _k: Kw) -> PyResult<Value> {
    Ok(Value::Bool(false))
}

// -------------------------------------------------------------------------
// object

fn cls_arg(vm: &mut Vm, v: &Value, what: &str) -> PyResult<Rc<Class>> {
    match v {
        Value::Class(c) => Ok(c.clone()),
        _ => {
            let tn = vm.type_name(v);
            Err(vm.type_error(format!("{}(X): X is not a type object ({})", what, tn)))
        }
    }
}

fn is_native_named(v: &Option<Value>, name: &str) -> bool {
    matches!(v, Some(Value::Native(n)) if n.name == name)
}

fn object_new(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    if a.is_empty() {
        return Err(vm.type_error("object.__new__(): not enough arguments"));
    }
    let c = cls_arg(vm, &a[0], "object.__new__")?;
    if a.len() > 1 || !k.is_empty() {
        if !is_native_named(&c.lookup(&vm.n.new), "object.__new__") {
            return Err(vm.type_error("object.__new__() takes exactly one argument (the type to instantiate)"));
        }
        if is_native_named(&c.lookup(&vm.n.init), "object.__init__") {
            return Err(vm.type_error(format!("{}() takes no arguments", c.name())));
        }
    }
    let abstract_ = c.dict.borrow().get_s("__abstractmethods__");
    if let Some(am) = abstract_ {
        let names = vm.to_vec(&am)?;
        if !names.is_empty() {
            let mut ns: Vec<String> = names.iter().filter_map(|n| n.as_str().map(|s| s.to_string())).collect();
            ns.sort();
            let cn = c.name();
            return Err(vm.type_error(format!("Can't instantiate abstract class {} without an implementation for abstract method{} {}", cn, if ns.len() == 1 { "" } else { "s" }, ns.iter().map(|n| format!("'{}'", n)).collect::<Vec<_>>().join(", "))));
        }
    }
    Ok(vm.new_instance(&c))
}

fn object_init(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    if a.len() > 1 || !k.is_empty() {
        let c = match a.first() {
            Some(v) => vm.class_of(v),
            None => return Ok(Value::None),
        };
        if !is_native_named(&c.lookup(&vm.n.init), "object.__init__") {
            return Err(vm.type_error("object.__init__() takes exactly one argument (the instance to initialize)"));
        }
        if is_native_named(&c.lookup(&vm.n.new), "object.__new__") {
            return Err(vm.type_error(format!("{}.__init__() takes exactly one argument (the instance to initialize)", c.name())));
        }
    }
    Ok(Value::None)
}

/// types.MethodType(function, instance)
fn method_new(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    if !k.is_empty() {
        return Err(vm.type_error("method() takes no keyword arguments"));
    }
    if a.len() != 3 {
        return Err(vm.type_error(format!("method expected 2 arguments, got {}", a.len().saturating_sub(1))));
    }
    if !crate::builtins::is_callable(vm, &a[1]) {
        return Err(vm.type_error("first argument must be callable"));
    }
    if a[2].is_none() {
        return Err(vm.type_error("instance must not be None"));
    }
    Ok(Value::Method(Rc::new((a[2].clone(), a[1].clone()))))
}

/// types.CellType([contents]): an empty cell, or one holding `contents`
fn cell_new(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    if !k.is_empty() {
        return Err(vm.type_error("cell() takes no keyword arguments"));
    }
    if a.len() > 2 {
        return Err(vm.type_error(format!("cell expected at most 1 argument, got {}", a.len() - 1)));
    }
    let c = Rc::new(RefCell::new(a.get(1).cloned().unwrap_or(Value::Undef)));
    crate::gc::track(crate::gc::Tracked::Cell(Rc::downgrade(&c)));
    Ok(Value::Cell(c))
}

/// types.FunctionType(code, globals, name=None, argdefs=None, closure=None, kwdefaults=None)
fn function_new(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let arg = |i: usize, n: &str| arg_or_kw(a, i, k, n).unwrap_or(Value::None);
    let code = match arg(1, "code") {
        Value::Code(c) => c,
        other => return Err(vm.type_error(format!("function() argument 'code' must be code, not {}", vm.type_name(&other)))),
    };
    let globals = match arg(2, "globals") {
        Value::Dict(d) => d,
        other => return Err(vm.type_error(format!("function() argument 'globals' must be dict, not {}", vm.type_name(&other)))),
    };
    let name = arg(3, "name");
    if !name.is_none() && !matches!(name, Value::Str(_)) {
        return Err(vm.type_error("arg 3 (name) must be None or string"));
    }
    let defaults = match arg(4, "argdefs") {
        Value::None => vec![],
        Value::Tuple(t) => t.to_vec(),
        _ => return Err(vm.type_error("arg 4 (defaults) must be None or tuple")),
    };
    let closure = match arg(5, "closure") {
        Value::None => vec![],
        Value::Tuple(t) => {
            let mut cells = vec![];
            for c in t.iter() {
                match c {
                    Value::Cell(c) => cells.push(c.clone()),
                    other => return Err(vm.type_error(format!("arg 5 (closure) expected cell, found {}", vm.type_name(other)))),
                }
            }
            cells
        }
        _ => return Err(vm.type_error("arg 5 (closure) must be None or tuple")),
    };
    if closure.len() != code.freevars.len() {
        return Err(vm.value_error(format!("{} requires closure of length {}, not {}", code.name.s, code.freevars.len(), closure.len())));
    }
    let kwdefaults = match arg(6, "kwdefaults") {
        Value::None => vec![],
        Value::Dict(d) => {
            let mut kd = vec![];
            for e in d.borrow().iter() {
                if let Value::Str(s) = &e.key {
                    kd.push((Rc::from(s.s.as_str()), e.value.clone()));
                }
            }
            kd
        }
        _ => return Err(vm.type_error("arg 6 (kwdefaults) must be None or dict")),
    };
    let f = vm.make_function(code, globals, defaults, kwdefaults, closure);
    if let (Value::Func(func), Value::Str(n)) = (&f, &name) {
        *func.name.borrow_mut() = n.clone();
    }
    Ok(f)
}

fn object_repr(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    Ok(Value::string(crate::format::object_repr(vm, &a[0])))
}

fn object_str(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    Ok(Value::string(vm.repr(&a[0])?))
}

fn object_eq(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 2)?;
    margs(vm, "__eq__", a, 1, 1)?;
    Ok(if a[0].is(&a[1]) { Value::Bool(true) } else { Value::NotImplemented })
}

fn object_ne(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    margs(vm, "__ne__", a, 1, 1)?;
    let cls = vm.class_of(&a[0]);
    if let Some(f @ Value::Func(_)) = cls.lookup(&vm.n.eq.clone()) {
        let r = vm.call_vec(&f, vec![a[0].clone(), a[1].clone()], &[])?;
        if matches!(r, Value::NotImplemented) {
            return Ok(r);
        }
        let t = vm.truthy(&r)?;
        return Ok(Value::Bool(!t));
    }
    Ok(if a[0].is(&a[1]) { Value::Bool(false) } else { Value::NotImplemented })
}

fn object_hash(_vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    Ok(Value::Int((ptr_id(&a[0]) >> 4) as i64))
}

fn object_format(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 2)?;
    margs(vm, "__format__", a, 1, 1)?;
    let spec = as_pystr(vm, &a[1], "format spec")?;
    if !spec.s.is_empty() {
        let tn = vm.type_name(&a[0]);
        return Err(vm.type_error(format!("unsupported format string passed to {}.__format__", tn)));
    }
    Ok(Value::string(vm.str_of(&a[0])?))
}

fn object_getattribute(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 2)?;
    margs(vm, "__getattribute__", a, 1, 1)?;
    let name = as_pystr(vm, &a[1], "attribute name")?;
    match &a[0] {
        Value::Instance(inst) => {
            let cls = inst.class.borrow().clone();
            // without __getattr__: that is the caller's fallback
            let has = cls.has_getattr.get();
            cls.has_getattr.set(false);
            let r = vm.generic_getattr(&a[0], inst, &cls, &name);
            cls.has_getattr.set(has);
            match r? {
                Some(v) => Ok(v),
                None => Err(vm.no_attr(&a[0], &name)),
            }
        }
        other => vm.getattr(other, &name),
    }
}

fn object_setattr(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 3)?;
    margs(vm, "__setattr__", a, 2, 2)?;
    let name = as_pystr(vm, &a[1], "attribute name")?;
    match &a[0] {
        Value::Instance(inst) => {
            let cls = inst.class.borrow().clone();
            vm.generic_setattr(&a[0], inst, &cls, &name, a[2].clone())?;
        }
        Value::Class(c) => vm.class_setattr(c, &name, a[2].clone())?,
        other => vm.setattr(other, &name, a[2].clone())?,
    }
    Ok(Value::None)
}

fn object_delattr(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 2)?;
    margs(vm, "__delattr__", a, 1, 1)?;
    let name = as_pystr(vm, &a[1], "attribute name")?;
    if let Value::Instance(inst) = &a[0] {
        let cls = inst.class.borrow().clone();
        if let Some(Value::Property(p)) = cls.lookup(&name) {
            let d = p.fdel.clone();
            vm.call_vec(&d, vec![a[0].clone()], &[])?;
            return Ok(Value::None);
        }
        if inst.dict.borrow_mut().remove_str(&name).is_none() {
            return Err(vm.no_attr(&a[0], &name));
        }
        return Ok(Value::None);
    }
    vm.delattr(&a[0], &name)?;
    Ok(Value::None)
}

fn object_dir(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let names = dir_of(vm, &a[0])?;
    Ok(Value::list(names.into_iter().map(Value::string).collect()))
}

fn reduce_helper(vm: &mut Vm, f: &str, args: Vec<Value>) -> PyResult<Value> {
    let m = vm.import("_reduce", &Value::None, 0, &vm.main_globals.clone())?;
    let h = vm.getattr(&m, &ps(f))?;
    vm.call_vec(&h, args, &[])
}

fn object_reduce(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    reduce_helper(vm, "common_reduce", vec![a[0].clone(), Value::Int(0)])
}

fn object_reduce_ex(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    if a.len() != 2 {
        return Err(vm.type_error(format!("object.__reduce_ex__() takes exactly one argument ({} given)", a.len().saturating_sub(1))));
    }
    let proto = vm.to_index(&a[1])?;
    if let Value::Gen(g) = &a[0] {
        let tn = if g.borrow().coroutine { "coroutine" } else { "generator" };
        return Err(vm.type_error(format!("cannot pickle '{}' object", tn)));
    }
    reduce_helper(vm, "reduce_ex", vec![a[0].clone(), Value::Int(proto)])
}

fn set_repr(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let x = this(vm, a);
    let name = match &a[0] {
        Value::Instance(i) => i.class.borrow().name(),
        _ => vm.type_name(&a[0]),
    };
    vm.enter(" while getting the repr of an object")?;
    let r = crate::format::set_repr(vm, &x, &name);
    vm.depth -= 1;
    Ok(Value::string(r?))
}

fn code_of(vm: &mut Vm, a: &[Value], what: &str) -> PyResult<Rc<crate::code::Code>> {
    match a.first() {
        Some(Value::Code(c)) => Ok(c.clone()),
        _ => Err(vm.type_error(format!("descriptor '{}' requires a 'code' object", what))),
    }
}

/// code.co_positions(): (line, end_line, col, end_col) of each instruction
fn code_positions(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let c = code_of(vm, a, "co_positions")?;
    let opt = |v: u32, known: bool| if known { Value::Int(v as i64) } else { Value::None };
    let items = c
        .positions
        .iter()
        .map(|p| {
            let known = p.line > 0;
            let cols = known && (p.col, p.end_line, p.end_col) != (0, p.line, 0);
            Value::tuple(vec![opt(p.line, known), opt(p.end_line, known), opt(p.col, cols), opt(p.end_col, cols)])
        })
        .collect();
    vm.iter(&Value::list(items))
}

/// code.co_lines(): (start, end, line) for each run of instructions on one line
fn code_lines(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let c = code_of(vm, a, "co_lines")?;
    let mut runs: Vec<(usize, usize, u32)> = vec![];
    for (i, p) in c.positions.iter().enumerate() {
        match runs.last_mut() {
            Some(r) if r.2 == p.line => r.1 = 2 * i + 2,
            _ => runs.push((2 * i, 2 * i + 2, p.line)),
        }
    }
    let items = runs
        .into_iter()
        .map(|(s, e, l)| Value::tuple(vec![Value::Int(s as i64), Value::Int(e as i64), if l > 0 { Value::Int(l as i64) } else { Value::None }]))
        .collect();
    vm.iter(&Value::list(items))
}

/// code.replace(**changes): a copy with some co_* fields changed
fn code_replace(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    margs(vm, "replace", a, 0, 0)?;
    let mut c = match &a[0] {
        Value::Code(c) => (**c).clone(),
        _ => return Err(vm.type_error("descriptor 'replace' requires a 'code' object")),
    };
    for (name, v) in k {
        let int = |vm: &mut Vm| match v {
            Value::Int(i) if *i >= 0 => Ok(*i as u32),
            _ => Err(vm.type_error(format!("replace() argument '{}' must be int", name.s))),
        };
        let st = |vm: &mut Vm| match v {
            Value::Str(s) => Ok(s.clone()),
            _ => Err(vm.type_error(format!("replace() argument '{}' must be str, not {}", name.s, vm.type_name(v)))),
        };
        match &*name.s {
            "co_name" => c.name = st(vm)?,
            "co_qualname" => c.qualname = st(vm)?,
            "co_filename" => c.filename = Rc::from(st(vm)?.s.as_str()),
            "co_firstlineno" => c.firstline = int(vm)?,
            "co_flags" => {
                let f = int(vm)?;
                // CO_ASYNC_GENERATOR is both
                c.generator = f & 0x220 != 0;
                c.coroutine = f & 0x280 != 0;
                c.iterable_coroutine = f & 0x100 != 0;
            }
            "co_consts" => {
                // the internal constants co_consts leaves out stay in place
                let vals = vm.to_vec(v)?;
                let slots: Vec<&mut Value> = c.consts.iter_mut().filter(|v| !matches!(v, Value::Obj(_))).collect();
                if slots.len() != vals.len() {
                    return Err(vm.value_error("co_consts: cannot change the number of constants"));
                }
                for (slot, x) in slots.into_iter().zip(vals) {
                    *slot = x;
                }
            }
            // CPython's location table, one op here taken as one code unit
            "co_linetable" => {
                let t = crate::builtins::bytes_of(vm, v)?;
                c.positions = crate::code::decode_linetable(&t, c.firstline, c.ops.len());
            }
            "co_names" | "co_varnames" | "co_freevars" | "co_cellvars" | "co_code" | "co_exceptiontable"
            | "co_argcount" | "co_posonlyargcount" | "co_kwonlyargcount" | "co_nlocals" | "co_stacksize" => {
                // the bytecode is this interpreter's own; only an unchanged value is accepted
                let cur = crate::builtins::special_attr(vm, &a[0], &name.s)?;
                let same = match cur {
                    Some(cur) => vm.eq(&cur, v)?,
                    None => false,
                };
                if !same {
                    return Err(vm.value_error(format!("code.replace() cannot change {} in this implementation", name.s)));
                }
            }
            _ => return Err(vm.type_error(format!("replace() got an unexpected keyword argument '{}'", name.s))),
        }
    }
    // the fast call path would skip making the generator
    c.simple = c.simple && !c.generator && !c.coroutine;
    Ok(Value::Code(Rc::new(c)))
}

fn class_getitem(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    margs(vm, "__class_getitem__", a, 1, 1)?;
    crate::builtins::generic_alias(vm, &a[0], &a[1])
}

fn cannot_create(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let tn = match a.first() {
        Some(Value::Class(c)) => c.name(),
        _ => "object".into(),
    };
    Err(vm.type_error(format!("cannot create '{}' instances", tn)))
}

fn cannot_pickle(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let tn = match a.first() {
        Some(v) => vm.type_name(v),
        None => "object".into(),
    };
    Err(vm.type_error(format!("cannot pickle '{}' object", tn)))
}

fn set_reduce(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    reduce_helper(vm, "set_reduce", vec![a[0].clone()])
}

fn object_sizeof(_vm: &mut Vm, _a: &[Value], _k: Kw) -> PyResult<Value> {
    Ok(Value::Int(16))
}

fn object_getstate(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    if let Value::Instance(i) = &a[0] {
        if i.extra.borrow().is_empty() || i.class.borrow().builtin.get() == Builtin::BaseException {
            return Ok(if i.dict.borrow().is_empty() { Value::None } else { Value::dict(i.dict.borrow().copy()) });
        }
        return reduce_helper(vm, "getstate", vec![a[0].clone()]);
    }
    Ok(Value::None)
}

fn object_init_subclass(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    if !k.is_empty() || a.len() > 1 {
        let n = match &a[0] {
            Value::Class(c) => c.name(),
            _ => "object".into(),
        };
        return Err(vm.type_error(format!("{}.__init_subclass__() takes no keyword arguments", n)));
    }
    Ok(Value::None)
}

fn getnewargs(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = this(vm, a);
    Ok(Value::tuple(vec![x]))
}

// -------------------------------------------------------------------------
// type

/// bases a class statement or type() may not derive from, and layout conflicts
pub(crate) fn check_bases(vm: &mut Vm, bases: &[Rc<crate::value::Class>]) -> PyResult<()> {
    // the built-in layout each base carries; two unrelated ones conflict
    let mut solid: Option<Rc<crate::value::Class>> = None;
    for b in bases {
        if matches!(&*b.name(), "bool" | "NoneType" | "NotImplementedType" | "ellipsis" | "slice" | "range" | "function" | "builtin_function_or_method" | "method" | "generator" | "coroutine" | "code" | "frame" | "traceback" | "memoryview" | "mappingproxy")
            && b.builtin.get() != Builtin::User
        {
            return Err(vm.type_error(format!("type '{}' is not an acceptable base type", b.name())));
        }
        let s = b.mro.borrow().iter().find(|m| !matches!(m.builtin.get(), Builtin::User | Builtin::Object)).cloned();
        if let Some(s) = s {
            match &solid {
                None => solid = Some(s),
                Some(cur) if cur.is_subclass(&s) => {}
                Some(cur) if s.is_subclass(cur) => solid = Some(s),
                Some(cur) if cur.is_subclass(&vm.t.base_exception) && s.is_subclass(&vm.t.base_exception) => {}
                Some(_) => return Err(vm.type_error("multiple bases have instance lay-out conflict")),
            }
        }
    }
    Ok(())
}

fn type_new(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let exact = matches!(a.first(), Some(Value::Class(c)) if Rc::ptr_eq(c, &vm.t.type_));
    if a.len() == 2 && exact && k.is_empty() {
        return Ok(Value::Class(vm.class_of(&a[1])));
    }
    if a.len() != 4 {
        if exact {
            return Err(vm.type_error("type() takes 1 or 3 arguments"));
        }
        return Err(vm.type_error(format!("type.__new__() takes exactly 3 arguments ({} given)", a.len().saturating_sub(1))));
    }
    let meta = cls_arg(vm, &a[0], "type.__new__")?;
    let name = as_pystr(vm, &a[1], "type.__new__() argument 1")?;
    if name.s.contains('\0') {
        return Err(vm.value_error("type name must not contain null characters"));
    }
    if !matches!(vm.base_of(&a[2]), Value::Tuple(_)) {
        let tn = vm.type_name(&a[2]);
        return Err(vm.type_error(format!("type.__new__() argument 2 must be tuple, not {}", tn)));
    }
    let bases_v = vm.to_vec(&a[2])?;
    let mut bases = vec![];
    for b in bases_v {
        match b {
            Value::Class(c) => bases.push(c),
            _ => return Err(vm.type_error("bases must be types")),
        }
    }
    check_bases(vm, &bases)?;
    // the most derived metaclass; one with its own __new__ makes the class
    let mut winner = meta.clone();
    for b in &bases {
        let bm = vm.class_of(&Value::Class(b.clone()));
        if winner.is_subclass(&bm) {
            continue;
        }
        if bm.is_subclass(&winner) {
            winner = bm;
            continue;
        }
        return Err(vm.type_error("metaclass conflict: the metaclass of a derived class must be a (non-strict) subclass of the metaclasses of all its bases"));
    }
    if !Rc::ptr_eq(&winner, &meta) {
        let wnew = winner.lookup(&vm.n.new);
        if !matches!(&wnew, Some(Value::Native(n)) if n.name == "type.__new__") {
            if let Some(f) = wnew {
                let f = match f {
                    Value::StaticMethod(inner) => (*inner).clone(),
                    other => other,
                };
                let mut args = vec![Value::Class(winner.clone())];
                args.extend(a[1..].iter().cloned());
                return vm.call_vec(&f, args, k);
            }
        }
    }
    let meta = winner;
    let ns = match &a[3] {
        Value::Dict(d) => d.clone(),
        other => match vm.base_of(other) {
            // a dict subclass is read through its own order (OrderedDict)
            Value::Dict(_) => {
                let d = Rc::new(RefCell::new(crate::value::Dict::new()));
                for (key, v) in vm.mapping_items(other)? {
                    vm.dict_set(&d, key, v)?;
                }
                d
            }
            _ => {
                let tn = vm.type_name(other);
                return Err(vm.type_error(format!("type.__new__() argument 3 must be dict, not {}", tn)));
            }
        },
    };
    if let Some(q) = ns.borrow().get_s("__qualname__") {
        if !matches!(q, Value::Str(_)) {
            let tn = vm.type_name(&q);
            return Err(vm.type_error(format!("type __qualname__ must be a str, not {}", tn)));
        }
    }
    let cls = vm.type_new(&meta, &name.s, bases, &ns)?;
    if cls.dict.borrow().get_s("__module__").is_none() {
        let m = vm.current_frame().and_then(|f| f.globals.borrow().get_s("__name__")).unwrap_or(Value::str("__main__"));
        cls.dict.borrow_mut().set_s("__module__", m);
    }
    vm.class_created(&cls, k, false)?;
    Ok(Value::Class(cls))
}

fn type_call(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let c = cls_arg(vm, &a[0], "type.__call__")?;
    vm.type_call(&c, a[1..].to_vec(), k)
}

fn type_mro(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let c = cls_arg(vm, &a[0], "mro")?;
    let v: Vec<Value> = c.mro.borrow().iter().map(|x| Value::Class(x.clone())).collect();
    Ok(Value::list(v))
}

fn type_subclasses(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let c = cls_arg(vm, &a[0], "__subclasses__")?;
    // classes are not tracked; scan what the modules hold
    let mut out: Vec<Value> = vec![];
    let mods: Vec<Value> = vm.modules.borrow().values();
    let mut dicts: Vec<Rc<RefCell<Dict>>> = vec![vm.main_globals.clone()];
    for m in mods {
        if let Value::Module(md) = m {
            dicts.push(md.dict.clone());
        }
    }
    let mut seen = std::collections::HashSet::new();
    let mut stack: Vec<Value> = vec![];
    for d in dicts {
        stack.extend(d.borrow().values());
    }
    while let Some(v) = stack.pop() {
        if let Value::Class(k) = &v {
            if !seen.insert(Rc::as_ptr(k) as usize) {
                continue;
            }
            if k.bases.borrow().iter().any(|b| Rc::ptr_eq(b, &c)) {
                out.push(v.clone());
            }
            stack.extend(k.dict.borrow().values().into_iter().filter(|x| matches!(x, Value::Class(_))));
        }
    }
    Ok(Value::list(out))
}

fn type_instancecheck(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 2)?;
    Ok(Value::Bool(crate::builtins::real_isinstance(vm, &a[1], &a[0])?))
}

fn type_subclasscheck(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 2)?;
    Ok(Value::Bool(crate::builtins::real_issubclass(vm, &a[1], &a[0])?))
}

fn type_prepare(_vm: &mut Vm, _a: &[Value], _k: Kw) -> PyResult<Value> {
    Ok(Value::dict(Dict::new()))
}

fn type_or(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 2)?;
    make_union(vm, &a[0], &a[1])
}

fn type_ror(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 2)?;
    make_union(vm, &a[1], &a[0])
}

// -------------------------------------------------------------------------
// int

/// int(text, base) as CPython parses it
pub fn parse_int(s: &str, base: u32) -> Option<BigInt> {
    let (neg, base, clean) = int_digits(s, base)?;
    Some(digits_int(neg, base, &clean)?)
}

/// parse_int for int(): like CPython's PyLong_FromString, a decimal string of
/// more than 6000 digits goes to _pylong.int_from_string
pub fn parse_int_vm(vm: &mut Vm, s: &str, base: u32) -> PyResult<Option<BigInt>> {
    let (neg, base, clean) = match int_digits(s, base) {
        Some(x) => x,
        None => return Ok(None),
    };
    if base != 10 || clean.len() <= 6000 {
        return Ok(digits_int(neg, base, &clean));
    }
    let f = crate::format::pylong_fn(vm, "int_from_string")?;
    let r = vm.call_vec(&f, vec![Value::string(clean)], &[])?;
    match crate::ops::big_of(&vm.base_of(&r)) {
        Some(v) => Ok(Some(if neg { v.neg() } else { v })),
        None => Err(vm.type_error("_pylong.int_from_string did not return an int")),
    }
}

fn digits_int(neg: bool, base: u32, clean: &str) -> Option<BigInt> {
    let v = if clean.len() <= 18 && base == 10 {
        BigInt::from_i64(clean.parse::<i64>().ok()?)
    } else {
        BigInt::from_radix(clean, base)?
    };
    Some(if neg { v.neg() } else { v })
}

/// the sign, base and ASCII digits (underscores dropped) of an int literal
fn int_digits(s: &str, base: u32) -> Option<(bool, u32, String)> {
    let t = s.trim_matches(|c: char| c.is_whitespace());
    let (neg, t) = match t.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, t.strip_prefix('+').unwrap_or(t)),
    };
    let lower = t.to_ascii_lowercase();
    let (base, body) = {
        let pre = |p: &str| lower.starts_with(p);
        if (base == 16 || base == 0) && pre("0x") {
            (16, &t[2..])
        } else if (base == 8 || base == 0) && pre("0o") {
            (8, &t[2..])
        } else if (base == 2 || base == 0) && pre("0b") {
            (2, &t[2..])
        } else if base == 0 {
            if t.len() > 1 && t.starts_with('0') && t.chars().any(|c| c != '0' && c != '_') {
                return None;
            }
            (10, t)
        } else {
            (base, t)
        }
    };
    let body = body.strip_prefix('_').filter(|_| base != 10 || t.len() != body.len()).unwrap_or(body);
    if body.is_empty() || body.starts_with('_') || body.ends_with('_') || body.contains("__") {
        return None;
    }
    let clean: String = body.chars().filter(|c| *c != '_').map(|c| if c.is_ascii_digit() || c.is_ascii_alphabetic() { c } else if let Some(d) = crate::strmethods::decimal_value(c) { char::from_digit(d, 10).unwrap() } else { c }).collect();
    if !clean.chars().all(|c| c.is_digit(base)) {
        return None;
    }
    Some((neg, base, clean))
}

pub fn to_int(vm: &mut Vm, v: &Value) -> PyResult<Value> {
    Ok(match v {
        Value::Int(_) | Value::Big(_) => v.clone(),
        Value::Bool(b) => Value::Int(*b as i64),
        Value::Float(f) => {
            if f.is_nan() {
                return Err(vm.value_error("cannot convert float NaN to integer"));
            }
            if f.is_infinite() {
                return Err(vm.err("OverflowError", "cannot convert float infinity to integer"));
            }
            let t = f.trunc();
            if t.abs() < 9.2e18 {
                Value::Int(t as i64)
            } else {
                Value::from_big(BigInt::from_f64(t).unwrap())
            }
        }
        Value::Str(s) => match { crate::format::check_str_digits(vm, &s.s, 10)?; parse_int_vm(vm, &s.s, 10)? } {
            Some(b) => Value::from_big(b),
            None => {
                let r = crate::format::str_repr(&s.s);
                return Err(vm.value_error(format!("invalid literal for int() with base 10: {}", r)));
            }
        },
        Value::Bytes(_) | Value::ByteArray(_) => {
            let s = bytes_to_latin1(v);
            crate::format::check_str_digits(vm, &s, 10)?;
            match parse_int_vm(vm, &s, 10)? {
                Some(b) => Value::from_big(b),
                None => {
                    let r = crate::format::bytes_repr(&s.chars().map(|c| c as u8).collect::<Vec<u8>>());
                    return Err(vm.value_error(format!("invalid literal for int() with base 10: {}", r)));
                }
            }
        }
        _ => {
            let b = vm.base_of(v);
            if matches!(v, Value::Instance(_)) && !matches!(b, Value::Instance(_)) && vm.lookup_special(v, &ps("__int__")).is_none_or(|f| matches!(&f, Value::Method(m) if matches!(m.1, Value::Native(_)))) {
                return to_int(vm, &b);
            }
            for n in ["__int__", "__index__"] {
                if let Some(f) = vm.lookup_special(v, &ps(n)) {
                    let r = vm.call_vec(&f, vec![], &[])?;
                    return exact_int(vm, r, n);
                }
            }
            if let Some(f) = vm.lookup_special(v, &ps("__trunc__")) {
                crate::tb::warn(vm, "DeprecationWarning", "The delegation of int() to __trunc__ is deprecated.")?;
                let r = vm.call_vec(&f, vec![], &[])?;
                if matches!(vm.base_of(&r), Value::Int(_) | Value::Big(_) | Value::Bool(_)) {
                    return exact_int(vm, r, "__trunc__");
                }
                if let Some(ix) = vm.lookup_special(&r, &ps("__index__")) {
                    let r = vm.call_vec(&ix, vec![], &[])?;
                    return exact_int(vm, r, "__index__");
                }
                let tn = vm.type_name(&r);
                return Err(vm.type_error(format!("__trunc__ returned non-Integral (type {})", tn)));
            }
            // the buffer protocol: memoryview, array
            if let Some(f) = vm.getattr_opt(v, &ps("tobytes")).ok().flatten() {
                if matches!(vm.type_name(v).as_str(), "memoryview" | "array") {
                    let by = vm.call_vec(&f, vec![], &[])?;
                    return to_int(vm, &by);
                }
            }
            let tn = vm.type_name(v);
            return Err(vm.type_error(format!("int() argument must be a string, a bytes-like object or a real number, not '{}'", tn)));
        }
    })
}

/// The int a __int__/__index__ returned, as an exact int
fn exact_int(vm: &mut Vm, r: Value, n: &str) -> PyResult<Value> {
    match r {
        Value::Int(_) | Value::Big(_) => Ok(r),
        Value::Bool(b) => {
            crate::tb::warn(vm, "DeprecationWarning", &format!("{} returned non-int (type bool).  The ability to return an instance of a strict subclass of int is deprecated, and may be removed in a future version of Python.", n))?;
            Ok(Value::Int(b as i64))
        }
        _ => {
            let b = vm.base_of(&r);
            if matches!(r, Value::Instance(_)) && matches!(b, Value::Int(_) | Value::Big(_) | Value::Bool(_)) {
                let tn = vm.type_name(&r);
                crate::tb::warn(vm, "DeprecationWarning", &format!("{} returned non-int (type {}).  The ability to return an instance of a strict subclass of int is deprecated, and may be removed in a future version of Python.", n, tn))?;
                return exact_int(vm, b, n);
            }
            let tn = vm.type_name(&r);
            Err(vm.type_error(format!("{} returned non-int (type {})", n, tn)))
        }
    }
}

fn int_new(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    if let Some(Value::Class(c)) = a.first() {
        if c.builtin.get() == Builtin::Bool {
            return Err(vm.type_error("int.__new__(bool) is not safe, use bool.__new__()"));
        }
    }
    if a.len() > 3 {
        return Err(vm.type_error(format!("int() takes at most 2 arguments ({} given)", a.len() - 1)));
    }
    for (name, _) in k.iter() {
        if name.s.as_str() != "base" {
            return Err(vm.type_error(format!("'{}' is an invalid keyword argument for int()", name.s)));
        }
    }
    let x = a.get(1).cloned();
    let base = arg_or_kw(a, 2, k, "base");
    let v = match (x, base) {
        (None, None) => Value::Int(0),
        (Some(x), None) => to_int(vm, &x)?,
        (x, Some(b)) => {
            let b = match vm.to_index(&b) {
                Ok(b) => b,
                Err(e) if vm.exc_is(&e, "OverflowError") => return Err(vm.value_error("int() base must be >= 2 and <= 36, or 0")),
                Err(e) => return Err(e),
            };
            if b != 0 && !(2..=36).contains(&b) {
                return Err(vm.value_error("int() base must be >= 2 and <= 36, or 0"));
            }
            let x = match x {
                Some(x) => x,
                None => return Err(vm.type_error("int() missing string argument")),
            };
            let s = match vm.base_of(&x) {
                Value::Str(s) => s.s.clone(),
                Value::Bytes(_) | Value::ByteArray(_) => bytes_to_latin1(&vm.base_of(&x)),
                _ => return Err(vm.type_error("int() can't convert non-string with explicit base")),
            };
            let eff = if b != 0 {
                b as u32
            } else {
                let t = s.trim().trim_start_matches(['+', '-']).to_ascii_lowercase();
                if t.starts_with("0x") { 16 } else if t.starts_with("0o") { 8 } else if t.starts_with("0b") { 2 } else { 10 }
            };
            crate::format::check_str_digits(vm, &s, eff)?;
            match parse_int_vm(vm, &s, b as u32)? {
                Some(v) => Value::from_big(v),
                None => {
                    let r = if matches!(vm.base_of(&x), Value::Str(_)) { crate::format::str_repr(&s) } else { crate::format::bytes_repr(&s.chars().map(|c| c as u8).collect::<Vec<u8>>()) };
                    return Err(vm.value_error(format!("invalid literal for int() with base {}: {}", b, r)));
                }
            }
        }
    };
    let t = vm.t.int.clone();
    Ok(wrap_new(vm, &a[0], &t, v))
}

fn int_int(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = this(vm, a);
    to_int(vm, &x)
}

fn int_float(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = this(vm, a);
    Ok(Value::Float(vm.to_f64(&x)?))
}

fn int_round(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let x = this(vm, a);
    let f = vm.builtins.borrow().get_s("round").unwrap();
    let mut args = vec![x];
    args.extend(a[1..].iter().cloned());
    vm.call_vec(&f, args, k)
}

fn int_bit_length(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = this(vm, a);
    let b = crate::ops::big_of(&x).unwrap_or_else(BigInt::zero);
    Ok(Value::Int(b.bit_length() as i64))
}

/// CPython's size: the type's basic size plus one 30-bit digit per
/// started digit, and always at least one
fn int_sizeof(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let cls = Value::Class(vm.class_of(&a[0]));
    let basic = vm.getattr(&cls, &ps("__basicsize__"))?;
    let basic = vm.to_index(&basic)?;
    let x = this(vm, a);
    let b = crate::ops::big_of(&x).unwrap_or_else(BigInt::zero);
    let digits = b.bit_length().div_ceil(30).max(1) as i64;
    Ok(Value::Int(basic + 4 * digits))
}

fn int_bit_count(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = this(vm, a);
    let b = crate::ops::big_of(&x).unwrap_or_else(BigInt::zero);
    Ok(Value::Int(b.mag.iter().map(|l| l.count_ones() as i64).sum()))
}

fn int_to_bytes(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let x = this(vm, a);
    let length = arg_or_kw(a, 1, k, "length").map(|v| vm.to_index(&v)).transpose()?.unwrap_or(1) as usize;
    let order = arg_or_kw(a, 2, k, "byteorder").and_then(|v| v.as_str().map(|s| s.to_string())).unwrap_or("big".into());
    let signed = match kwarg(k, "signed") {
        Some(v) => vm.truthy(&v)?,
        None => false,
    };
    let b = crate::ops::big_of(&x).unwrap();
    if b.neg && !signed {
        return Err(vm.err("OverflowError", "can't convert negative int to unsigned"));
    }
    // two's complement over `length` bytes
    let modulus = BigInt::from_i64(1).shl(length * 8);
    let v = if b.neg { b.add(&modulus) } else { b.clone() };
    if v.neg || v.cmp(&modulus) != std::cmp::Ordering::Less || (signed && !b.neg && b.bit_length() >= length * 8 && length > 0) || (signed && b.neg && v.bit_length() < length * 8 && length > 0 && false) {
        return Err(vm.err("OverflowError", "int too big to convert"));
    }
    if signed && b.neg && v.cmp(&modulus.shr(1)) == std::cmp::Ordering::Less {
        return Err(vm.err("OverflowError", "int too big to convert"));
    }
    let mut out = vec![0u8; length];
    for i in 0..length {
        let limb = v.mag.get(i / 4).copied().unwrap_or(0);
        out[length - 1 - i] = (limb >> ((i % 4) * 8)) as u8;
    }
    if order == "little" {
        out.reverse();
    }
    Ok(Value::Bytes(Rc::new(out)))
}

fn int_from_bytes(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    if a.len() > 3 {
        return Err(vm.type_error(format!("from_bytes() takes at most 2 positional arguments ({} given)", a.len() - 1)));
    }
    let data = arg_or_kw(a, 1, k, "bytes").unwrap_or(Value::Bytes(Rc::new(vec![])));
    if matches!(data, Value::Str(_) | Value::Int(_) | Value::Big(_) | Value::Bool(_) | Value::Float(_)) {
        let tn = vm.type_name(&data);
        return Err(vm.type_error(format!("cannot convert '{}' object to bytes", tn)));
    }
    let dunder = match &data {
        Value::Instance(_) => vm.lookup_special(&data, &ps("__bytes__")),
        _ => None,
    };
    let mut bs = match &data {
        Value::Bytes(_) | Value::ByteArray(_) => bytes_of(vm, &data)?,
        _ if dunder.is_some() => match vm.call_vec(&dunder.unwrap(), vec![], &[])? {
            b @ Value::Bytes(_) => bytes_of(vm, &b)?,
            r => {
                let tn = vm.type_name(&r);
                return Err(vm.type_error(format!("__bytes__ returned non-bytes (type {})", tn)));
            }
        },
        other => match bytes_of(vm, other) {
            Ok(b) if !matches!(other, Value::List(_) | Value::Tuple(_)) => b,
            _ => {
                let v = vm.to_vec(other)?;
                let mut out = vec![];
                for x in v {
                    match u8::try_from(vm.to_index(&x)?) {
                        Ok(b) => out.push(b),
                        Err(_) => return Err(vm.value_error("bytes must be in range(0, 256)")),
                    }
                }
                out
            }
        },
    };
    let order = arg_or_kw(a, 2, k, "byteorder").and_then(|v| v.as_str().map(|s| s.to_string())).unwrap_or("big".into());
    if order != "little" && order != "big" {
        return Err(vm.value_error("byteorder must be either 'little' or 'big'"));
    }
    let signed = match kwarg(k, "signed") {
        Some(v) => vm.truthy(&v)?,
        None => false,
    };
    if order == "little" {
        bs.reverse();
    }
    let mut v = BigInt::zero();
    for b in &bs {
        v = v.shl(8).add(&BigInt::from_i64(*b as i64));
    }
    if signed && !bs.is_empty() && bs[0] & 0x80 != 0 {
        v = v.sub(&BigInt::from_i64(1).shl(bs.len() * 8));
    }
    let r = Value::from_big(v);
    // other classes get cls(value), as CPython does
    match &a[0] {
        Value::Class(c) if Rc::ptr_eq(c, &vm.t.int) => Ok(r),
        cls => vm.call_vec(cls, vec![r], &[]),
    }
}

fn int_ratio(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let x = this(vm, a);
    let x = to_int(vm, &x)?;
    Ok(Value::tuple(vec![x, Value::Int(1)]))
}

fn bool_new(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    if !k.is_empty() {
        return Err(vm.type_error("bool() takes no keyword arguments"));
    }
    if a.len() > 2 {
        return Err(vm.type_error(format!("bool expected at most 1 argument, got {}", a.len() - 1)));
    }
    match a.get(1) {
        Some(v) => Ok(Value::Bool(vm.truthy(v)?)),
        None => Ok(Value::Bool(false)),
    }
}

// -------------------------------------------------------------------------
// float

/// digit (['_'] digit)*, from i; the index after it
fn digit_part(cs: &[u8], mut i: usize) -> Option<usize> {
    if i >= cs.len() || !cs[i].is_ascii_digit() {
        return None;
    }
    while i < cs.len() {
        if cs[i].is_ascii_digit() {
            i += 1;
        } else if cs[i] == b'_' && i + 1 < cs.len() && cs[i + 1].is_ascii_digit() {
            i += 1;
        } else {
            break;
        }
    }
    Some(i)
}

/// float(text) as CPython parses it; None for a malformed string
pub fn parse_float(s: &str) -> Option<f64> {
    let t = s.trim_matches(|c: char| c.is_whitespace());
    let t: String = t.chars().map(|c| if c.is_ascii() { c } else if let Some(d) = crate::strmethods::decimal_value(c) { char::from_digit(d, 10).unwrap() } else { c }).collect();
    let (neg, body) = match t.as_bytes().first() {
        Some(b'-') => (true, &t[1..]),
        Some(b'+') => (false, &t[1..]),
        _ => (false, &t[..]),
    };
    let special = match body.to_ascii_lowercase().as_str() {
        "inf" | "infinity" => Some(f64::INFINITY),
        "nan" => Some(f64::NAN),
        _ => None,
    };
    if let Some(v) = special {
        return Some(if neg { -v } else { v });
    }
    let cs = body.as_bytes();
    let mut i = match digit_part(cs, 0) {
        Some(j) => {
            if j < cs.len() && cs[j] == b'.' {
                digit_part(cs, j + 1).unwrap_or(j + 1)
            } else {
                j
            }
        }
        None if cs.first() == Some(&b'.') => digit_part(cs, 1)?,
        None => return None,
    };
    if i < cs.len() && (cs[i] == b'e' || cs[i] == b'E') {
        let mut j = i + 1;
        if j < cs.len() && (cs[j] == b'+' || cs[j] == b'-') {
            j += 1;
        }
        i = digit_part(cs, j)?;
    }
    if i != cs.len() {
        return None;
    }
    let clean: String = body.chars().filter(|c| *c != '_').collect();
    let v = clean.parse::<f64>().ok()?;
    Some(if neg { -v } else { v })
}

fn float_text(vm: &mut Vm, text: &str, shown: String) -> PyResult<f64> {
    parse_float(text).ok_or_else(|| vm.value_error(format!("could not convert string to float: {}", shown)))
}

/// float(x): the number x stands for
pub fn float_of(vm: &mut Vm, x: &Value) -> PyResult<f64> {
    match x {
        Value::Float(f) => Ok(*f),
        Value::Int(_) | Value::Bool(_) | Value::Big(_) => vm.to_f64(x),
        Value::Str(s) => {
            let r = crate::format::str_repr(&s.s);
            float_text(vm, &s.s, r)
        }
        Value::Bytes(_) | Value::ByteArray(_) => {
            let s = bytes_to_latin1(x);
            let raw: Vec<u8> = s.chars().map(|c| c as u8).collect();
            let r = crate::format::bytes_repr(&raw);
            if !raw.is_ascii() {
                return Err(vm.value_error(format!("could not convert string to float: {}", r)));
            }
            float_text(vm, &s, r)
        }
        _ => {
            let b = vm.base_of(x);
            if let Some(f) = vm.lookup_special(x, &vm.n.float_.clone()) {
                let native = matches!(&f, Value::Method(m) if matches!(m.1, Value::Native(_)));
                if !(native && matches!(x, Value::Instance(_)) && !matches!(b, Value::Instance(_))) {
                    let r = vm.call_vec(&f, vec![], &[])?;
                    if let Value::Float(f) = r {
                        return Ok(f);
                    }
                    if let (Value::Instance(_), Value::Float(f)) = (&r, vm.base_of(&r)) {
                        let tn = vm.type_name(x);
                        let rn = vm.type_name(&r);
                        crate::tb::warn(vm, "DeprecationWarning", &format!("{}.__float__ returned non-float (type {}).  The ability to return an instance of a strict subclass of float is deprecated, and may be removed in a future version of Python.", tn, rn))?;
                        return Ok(f);
                    }
                    let tn = vm.type_name(x);
                    let rn = vm.type_name(&r);
                    return Err(vm.type_error(format!("{}.__float__ returned non-float (type {})", tn, rn)));
                }
            }
            if matches!(x, Value::Instance(_)) && !matches!(b, Value::Instance(_)) {
                return float_of(vm, &b);
            }
            if let Some(f) = vm.lookup_special(x, &vm.n.index.clone()) {
                let r = vm.call_vec(&f, vec![], &[])?;
                return match vm.base_of(&r) {
                    i @ (Value::Int(_) | Value::Big(_) | Value::Bool(_)) => vm.to_f64(&i),
                    _ => {
                        let rn = vm.type_name(&r);
                        Err(vm.type_error(format!("__index__ returned non-int (type {})", rn)))
                    }
                };
            }
            if matches!(vm.type_name(x).as_str(), "memoryview" | "array") {
                if let Some(f) = vm.getattr_opt(x, &ps("tobytes"))? {
                    let by = vm.call_vec(&f, vec![], &[])?;
                    return float_of(vm, &by);
                }
            }
            let tn = vm.type_name(x);
            Err(vm.type_error(format!("float() argument must be a string or a real number, not '{}'", tn)))
        }
    }
}

fn float_new(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    if !k.is_empty() {
        let own_init = match &a[0] {
            Value::Class(c) => matches!(c.lookup(&ps("__init__")), Some(Value::Func(_))),
            _ => false,
        };
        if !own_init {
            return Err(vm.type_error("float() takes no keyword arguments"));
        }
    }
    if a.len() > 2 {
        return Err(vm.type_error(format!("float expected at most 1 argument, got {}", a.len() - 1)));
    }
    let v = match a.get(1) {
        None => Value::Float(0.0),
        Some(x) => Value::Float(float_of(vm, x)?),
    };
    let t = vm.t.float.clone();
    Ok(wrap_new(vm, &a[0], &t, v))
}

fn fself(vm: &mut Vm, a: &[Value]) -> PyResult<f64> {
    let x = this(vm, a);
    vm.to_f64(&x)
}

fn float_int(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let f = fself(vm, a)?;
    to_int(vm, &Value::Float(f))
}
fn float_float(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    Ok(Value::Float(fself(vm, a)?))
}
fn float_floor(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let f = fself(vm, a)?;
    to_int(vm, &Value::Float(f.floor()))
}
fn float_ceil(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let f = fself(vm, a)?;
    to_int(vm, &Value::Float(f.ceil()))
}
fn float_round(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    int_round(vm, a, k)
}
fn float_is_integer(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let f = fself(vm, a)?;
    Ok(Value::Bool(f.is_finite() && f.fract() == 0.0))
}
pub fn float_ratio_parts(vm: &mut Vm, f: f64) -> PyResult<(BigInt, BigInt)> {
    if f.is_nan() {
        return Err(vm.value_error("cannot convert NaN to integer ratio"));
    }
    if f.is_infinite() {
        return Err(vm.err("OverflowError", "cannot convert Infinity to integer ratio"));
    }
    if f == 0.0 {
        return Ok((BigInt::zero(), BigInt::from_i64(1)));
    }
    let bits = f.to_bits();
    let exp = ((bits >> 52) & 0x7ff) as i64;
    let mant = if exp == 0 { (bits & ((1 << 52) - 1)) << 1 } else { (bits & ((1 << 52) - 1)) | (1 << 52) };
    let mut e = exp - 1075;
    let mut m = mant;
    while m & 1 == 0 && e < 0 {
        m >>= 1;
        e += 1;
    }
    let mut num = BigInt::from_i64(m as i64);
    let mut den = BigInt::from_i64(1);
    if e > 0 {
        num = num.shl(e as usize);
    } else {
        den = den.shl((-e) as usize);
    }
    if f < 0.0 {
        num = num.neg();
    }
    Ok((num, den))
}
fn float_ratio(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let f = fself(vm, a)?;
    let (n, d) = float_ratio_parts(vm, f)?;
    Ok(Value::tuple(vec![Value::from_big(n), Value::from_big(d)]))
}
/// float.__getformat__(typestr): how C doubles and floats are stored
fn float_getformat(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    if a.len() != 2 {
        return Err(vm.type_error(format!("float.__getformat__() takes exactly one argument ({} given)", a.len().saturating_sub(1))));
    }
    match &a[1] {
        Value::Str(t) if matches!(&*t.s, "double" | "float") => Ok(Value::str("IEEE, little-endian")),
        Value::Str(_) => Err(vm.value_error("__getformat__() argument 1 must be 'double' or 'float'")),
        v => Err(vm.type_error(format!("__getformat__() argument must be str, not {}", vm.type_name(v)))),
    }
}

fn float_hex(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let f = fself(vm, a)?;
    if f.is_nan() {
        return Ok(Value::str("nan"));
    }
    if f.is_infinite() {
        return Ok(Value::str(if f > 0.0 { "inf" } else { "-inf" }));
    }
    let sign = if f.is_sign_negative() { "-" } else { "" };
    if f == 0.0 {
        return Ok(Value::string(format!("{}0x0.0p+0", sign)));
    }
    let bits = f.abs().to_bits();
    let exp = ((bits >> 52) & 0x7ff) as i64;
    let mant = bits & ((1 << 52) - 1);
    let (lead, e) = if exp == 0 { (0, -1022) } else { (1, exp - 1023) };
    let hex = format!("{:013x}", mant);
    Ok(Value::string(format!("{}0x{}.{}p{}{}", sign, lead, hex, if e >= 0 { "+" } else { "-" }, e.abs())))
}
/// x * 2**e for an x exactly representable at that scale
fn ldexp_exact(mut x: f64, mut e: i64) -> f64 {
    while e > 1000 {
        x *= 2f64.powi(1000);
        e -= 1000;
    }
    while e < -1000 {
        x *= 2f64.powi(-1000);
        e += 1000;
    }
    x * 2f64.powi(e as i32)
}

/// float.fromhex as CPython: correctly rounded, half to even
pub fn parse_hex_float(s: &str) -> Result<f64, &'static str> {
    const BAD: &str = "invalid hexadecimal floating-point string";
    let t = s.trim_matches(|c: char| c.is_whitespace());
    let (neg, t) = match t.as_bytes().first() {
        Some(b'-') => (true, &t[1..]),
        Some(b'+') => (false, &t[1..]),
        _ => (false, t),
    };
    let sign = |v: f64| if neg { -v } else { v };
    match t.to_ascii_lowercase().as_str() {
        "inf" | "infinity" => return Ok(sign(f64::INFINITY)),
        "nan" => return Ok(sign(f64::NAN)),
        _ => {}
    }
    let t = if t.len() >= 2 && t.as_bytes()[..2].eq_ignore_ascii_case(b"0x") { &t[2..] } else { t };
    let cs = t.as_bytes();
    let mut i = 0;
    let mut digits = String::new();
    let mut frac = 0i64;
    while i < cs.len() && cs[i].is_ascii_hexdigit() {
        digits.push(cs[i] as char);
        i += 1;
    }
    let int_len = digits.len();
    if i < cs.len() && cs[i] == b'.' {
        i += 1;
        while i < cs.len() && cs[i].is_ascii_hexdigit() {
            digits.push(cs[i] as char);
            frac += 1;
            i += 1;
        }
    }
    if digits.is_empty() || (int_len == 0 && frac == 0) {
        return Err(BAD);
    }
    let mut exp: i64 = 0;
    if i < cs.len() && (cs[i] == b'p' || cs[i] == b'P') {
        i += 1;
        let mut eneg = false;
        if i < cs.len() && (cs[i] == b'+' || cs[i] == b'-') {
            eneg = cs[i] == b'-';
            i += 1;
        }
        let start = i;
        while i < cs.len() && cs[i].is_ascii_digit() {
            // clamp: anything this large over- or underflows anyway
            exp = (exp * 10 + (cs[i] - b'0') as i64).min(1 << 40);
            i += 1;
        }
        if i == start {
            return Err(BAD);
        }
        if eneg {
            exp = -exp;
        }
    }
    if i != cs.len() {
        return Err(BAD);
    }
    let digits = digits.trim_start_matches('0');
    if digits.is_empty() {
        return Ok(sign(0.0));
    }
    let m = BigInt::from_radix(digits, 16).ok_or(BAD)?;
    let e = exp - 4 * frac;
    let nbits = m.bit_length() as i64;
    let top = nbits - 1 + e;
    const TOO_LARGE: &str = "hexadecimal value too large to represent as a float";
    if top >= 1024 {
        return Err(TOO_LARGE);
    }
    if top < -1075 {
        return Ok(sign(0.0));
    }
    let lsb = (top - 52).max(-1074);
    let shift = lsb - e;
    let q: u64 = if shift <= 0 {
        m.shl((-shift) as usize).to_i64().unwrap() as u64
    } else {
        let sh = shift as usize;
        let q = m.shr(sh);
        let r = m.sub(&q.shl(sh));
        let half = BigInt::from_i64(1).shl(sh - 1);
        let mut q = q.to_i64().unwrap() as u64;
        match r.cmp(&half) {
            std::cmp::Ordering::Greater => q += 1,
            std::cmp::Ordering::Equal if q & 1 == 1 => q += 1,
            _ => {}
        }
        q
    };
    if q >= (1u64 << 53) && lsb + 53 >= 1024 {
        return Err(TOO_LARGE);
    }
    Ok(sign(ldexp_exact(q as f64, lsb)))
}

fn float_fromhex(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 2)?;
    let s = as_pystr(vm, &a[1], "fromhex() argument")?;
    let f = match parse_hex_float(&s.s) {
        Ok(f) => f,
        Err(m) if m.starts_with("hex") => return Err(vm.err("OverflowError", m)),
        Err(m) => return Err(vm.value_error(m)),
    };
    match &a[0] {
        Value::Class(c) if !Rc::ptr_eq(c, &vm.t.float) => vm.call_vec(&a[0], vec![Value::Float(f)], &[]),
        _ => Ok(Value::Float(f)),
    }
}

fn complex_new(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let re = arg_or_kw(a, 1, k, "real");
    let im = arg_or_kw(a, 2, k, "imag");
    let parts = |vm: &mut Vm, v: &Value| -> PyResult<(f64, f64, bool)> {
        match v {
            Value::Complex(c) => Ok((c.0, c.1, true)),
            Value::Str(s) => parse_complex(&s.s).map(|(r, i)| (r, i, true)).ok_or_else(|| vm.value_error("complex() arg is a malformed string")),
            Value::Instance(_) => {
                if let Some(f) = vm.lookup_special(v, &ps("__complex__")) {
                    let r = vm.call_vec(&f, vec![], &[])?;
                    if let Value::Complex(c) = r {
                        return Ok((c.0, c.1, true));
                    }
                }
                Ok((vm.to_f64(v)?, 0.0, false))
            }
            _ => Ok((vm.to_f64(v)?, 0.0, false)),
        }
    };
    let (mut r1, i1, re_cx) = match &re {
        Some(v) => parts(vm, v)?,
        None => (0.0, 0.0, false),
    };
    let (mut r2, i2, im_cx) = match &im {
        Some(v) => parts(vm, v)?,
        None => (0.0, 0.0, false),
    };
    // only complex parts are combined, so a real -0.0 keeps its sign
    if im_cx {
        r1 -= i2;
    }
    if im.is_none() {
        r2 = i1;
    } else if re_cx {
        r2 += i1;
    }
    let v = Value::Complex(Rc::new((r1, r2)));
    let t = vm.t.complex.clone();
    Ok(wrap_new(vm, &a[0], &t, v))
}

pub fn parse_complex(s: &str) -> Option<(f64, f64)> {
    let t: String = s.trim().trim_start_matches('(').trim_end_matches(')').chars().filter(|c| !c.is_whitespace()).collect();
    if let Some(body) = t.strip_suffix(['j', 'J']) {
        // split at the last sign that is not an exponent sign
        let cs: Vec<char> = body.chars().collect();
        let mut split = None;
        for i in (1..cs.len()).rev() {
            if (cs[i] == '+' || cs[i] == '-') && cs[i - 1] != 'e' && cs[i - 1] != 'E' {
                split = Some(i);
                break;
            }
        }
        let (re, im) = match split {
            Some(i) => (&body[..i], &body[i..]),
            None => ("", body),
        };
        let imv = match im {
            "" | "+" => 1.0,
            "-" => -1.0,
            x => parse_float(x)?,
        };
        let rev = if re.is_empty() { 0.0 } else { parse_float(re)? };
        Some((rev, imv))
    } else {
        Some((parse_float(&t)?, 0.0))
    }
}

fn complex_conjugate(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    match this(vm, a) {
        Value::Complex(c) => Ok(Value::Complex(Rc::new((c.0, -c.1)))),
        _ => Err(vm.type_error("descriptor 'conjugate' requires a 'complex' object")),
    }
}

// -------------------------------------------------------------------------
// list

fn list_of(vm: &mut Vm, v: &Value, m: &str) -> PyResult<Rc<RefCell<Vec<Value>>>> {
    match vm.base_of(v) {
        Value::List(l) => Ok(l),
        _ => {
            let tn = vm.type_name(v);
            Err(vm.type_error(format!("descriptor '{}' for 'list' objects doesn't apply to a '{}' object", m, tn)))
        }
    }
}

fn list_new(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let t = vm.t.list.clone();
    let v = Value::list(vec![]);
    Ok(wrap_new(vm, &a[0], &t, v))
}

fn list_init(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    if !k.is_empty() && native_slot(vm, &a[0], "__new__", "list.__new__") {
        return Err(vm.type_error("list() takes no keyword arguments"));
    }
    if a.len() > 2 {
        return Err(vm.type_error(format!("list expected at most 1 argument, got {}", a.len() - 1)));
    }
    let l = list_of(vm, &a[0], "__init__")?;
    if let Some(src) = a.get(1) {
        let items = vm.to_vec(src)?;
        *l.borrow_mut() = items;
    } else {
        l.borrow_mut().clear();
    }
    Ok(Value::None)
}

fn list_append(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    if a.len() != 2 {
        return Err(vm.type_error(format!("list.append() takes exactly one argument ({} given)", a.len().saturating_sub(1))));
    }
    let l = list_of(vm, &a[0], "append")?;
    crate::gc::list_store(&l, &a[1]);
    l.borrow_mut().push(a[1].clone());
    Ok(Value::None)
}

fn list_extend(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 2)?;
    margs(vm, "list.extend", a, 1, 1)?;
    let l = list_of(vm, &a[0], "extend")?;
    let items = vm.to_vec(&a[1])?;
    if let Some(x) = items.iter().find(|x| x.holds_refs()) {
        crate::gc::list_store(&l, x);
    }
    l.borrow_mut().extend(items);
    Ok(Value::None)
}

fn list_insert(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    if a.len() != 3 {
        return Err(vm.type_error(format!("insert expected 2 arguments, got {}", a.len().saturating_sub(1))));
    }
    let l = list_of(vm, &a[0], "insert")?;
    let i = vm.to_index(&a[1])?;
    let n = l.borrow().len() as i64;
    let i = if i < 0 { (i + n).max(0) } else { i.min(n) };
    crate::gc::list_store(&l, &a[2]);
    l.borrow_mut().insert(i as usize, a[2].clone());
    Ok(Value::None)
}

fn list_pop(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    margs(vm, "pop", a, 0, 1)?;
    let l = list_of(vm, &a[0], "pop")?;
    let n = l.borrow().len();
    if n == 0 {
        return Err(vm.index_error("pop from empty list"));
    }
    let i = match a.get(1) {
        Some(v) => {
            let i = vm.to_index(v)?;
            let j = if i < 0 { i + n as i64 } else { i };
            if j < 0 || j >= n as i64 {
                return Err(vm.index_error("pop index out of range"));
            }
            j as usize
        }
        None => n - 1,
    };
    let v = l.borrow_mut().remove(i);
    Ok(v)
}

fn list_remove(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 2)?;
    margs(vm, "list.remove", a, 1, 1)?;
    let l = list_of(vm, &a[0], "remove")?;
    let items = l.borrow().clone();
    for (i, x) in items.iter().enumerate() {
        if x.is(&a[1]) || vm.eq(x, &a[1])? {
            l.borrow_mut().remove(i);
            return Ok(Value::None);
        }
    }
    Err(vm.value_error("list.remove(x): x not in list"))
}

fn seq_items_of(vm: &mut Vm, v: &Value) -> PyResult<Vec<Value>> {
    let b = vm.base_of(v);
    vm.to_vec(&b)
}

fn seq_index(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    margs(vm, "index", a, 1, 3)?;
    let me = this(vm, a);
    if let Value::Range(_) | Value::BigRange(_) = &me {
        if let Some(i) = crate::range::find(vm, &me, &a[1], false)? {
            return Ok(Value::from_big(i));
        }
        let r = vm.repr(&a[1])?;
        return Err(vm.value_error(format!("{} is not in range", r)));
    }
    let items = seq_items_of(vm, &a[0])?;
    let n = items.len() as i64;
    let norm = |i: i64| -> usize { (if i < 0 { (i + n).max(0) } else { i.min(n) }) as usize };
    let bound = |vm: &mut Vm, v: Option<&Value>, dflt: usize| -> PyResult<usize> {
        let Some(v) = v else { return Ok(dflt) };
        match crate::builtins::clamp_index(vm, v)? {
            Some(i) => Ok(norm(i.clamp(-(1 << 62), 1 << 62))),
            None => Err(vm.type_error("slice indices must be integers or have an __index__ method")),
        }
    };
    let start = bound(vm, a.get(2), 0)?;
    let stop = bound(vm, a.get(3), n as usize)?;
    for i in start..stop.max(start) {
        let Some(x) = items.get(i) else { break };
        if x.is(&a[1]) || vm.eq(x, &a[1])? {
            return Ok(Value::Int(i as i64));
        }
    }
    let what = match me {
        Value::Tuple(_) => "tuple.index(x): x not in tuple".to_string(),
        Value::Range(_) => {
            let r = vm.repr(&a[1])?;
            format!("{} is not in range", r)
        }
        _ => {
            let r = vm.repr(&a[1])?;
            format!("{} is not in list", r)
        }
    };
    Err(vm.value_error(what))
}

fn seq_count(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 2)?;
    margs(vm, "count", a, 1, 1)?;
    let me = this(vm, a);
    if let Value::Range(_) | Value::BigRange(_) = &me {
        let n = crate::range::find(vm, &me, &a[1], true)?.unwrap_or_else(BigInt::zero);
        return Ok(Value::from_big(n));
    }
    let items = seq_items_of(vm, &a[0])?;
    let mut n = 0;
    for x in &items {
        if x.is(&a[1]) || vm.eq(x, &a[1])? {
            n += 1;
        }
    }
    Ok(Value::Int(n))
}

fn list_clear(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let l = list_of(vm, &a[0], "clear")?;
    l.borrow_mut().clear();
    Ok(Value::None)
}

fn list_copy(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let l = list_of(vm, &a[0], "copy")?;
    let v = l.borrow().clone();
    Ok(Value::list(v))
}

fn list_reverse(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let l = list_of(vm, &a[0], "reverse")?;
    l.borrow_mut().reverse();
    Ok(Value::None)
}

fn list_reversed(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let l = list_of(vm, &a[0], "__reversed__")?;
    let n = l.borrow().len();
    Ok(mk_iter(Iter::ListRev(l, n)))
}

fn list_sort(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    if a.len() > 1 {
        return Err(vm.type_error("sort() takes no positional arguments"));
    }
    let l = list_of(vm, &a[0], "sort")?;
    let key = kwarg(k, "key").unwrap_or(Value::None);
    let rev = match kwarg(k, "reverse") {
        Some(v) => vm.truthy(&v)?,
        None => false,
    };
    // sorting works on a copy; the list reads as empty meanwhile
    let mut items = std::mem::take(&mut *l.borrow_mut());
    let r = sort_values(vm, &mut items, &key, rev);
    // the keys are gone by now, and their __del__ may touch the list
    vm.finalize_pending();
    // a list that grew, even if emptied again, has a buffer
    let modified = l.borrow().capacity() != 0;
    *l.borrow_mut() = items;
    r?;
    if modified {
        return Err(vm.value_error("list modified during sort"));
    }
    Ok(Value::None)
}

// -------------------------------------------------------------------------
// tuple

/// whether `cls` (a class, or an instance's class) resolves `name` to the native `native`
fn native_slot(vm: &Vm, cls: &Value, name: &str, native: &str) -> bool {
    let c = match cls {
        Value::Class(c) => c.clone(),
        v => vm.class_of(v),
    };
    matches!(c.lookup_s(name), Some(Value::Native(n)) if n.name == native)
}

fn tuple_new(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    if !k.is_empty() && native_slot(vm, &a[0], "__init__", "object.__init__") {
        return Err(vm.type_error("tuple() takes no keyword arguments"));
    }
    if a.len() > 2 {
        return Err(vm.type_error(format!("tuple expected at most 1 argument, got {}", a.len() - 1)));
    }
    let v = match a.get(1) {
        Some(Value::Tuple(t)) => Value::Tuple(t.clone()),
        Some(x) => Value::tuple(vm.to_vec(x)?),
        None => Value::tuple(vec![]),
    };
    let t = vm.t.tuple.clone();
    Ok(wrap_new(vm, &a[0], &t, v))
}

// -------------------------------------------------------------------------
// dict

fn dict_of(vm: &mut Vm, v: &Value, m: &str) -> PyResult<Rc<RefCell<Dict>>> {
    match vm.base_of(v) {
        Value::Dict(d) => Ok(d),
        _ => {
            let tn = vm.type_name(v);
            Err(vm.type_error(format!("descriptor '{}' for 'dict' objects doesn't apply to a '{}' object", m, tn)))
        }
    }
}

fn dict_new(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let t = vm.t.dict.clone();
    let v = Value::dict(Dict::new());
    Ok(wrap_new(vm, &a[0], &t, v))
}

/// dict.update semantics: a mapping or pairs, then keywords
pub fn dict_merge_into(vm: &mut Vm, target: &Value, d: &Rc<RefCell<Dict>>, src: Option<&Value>, k: Kw) -> PyResult<()> {
    let _ = target;
    // like CPython, a subclass's __setitem__ is not consulted
    let set = |vm: &mut Vm, key: Value, v: Value| -> PyResult<()> { vm.dict_set(d, key, v) };
    if let (Some(Value::Dict(o)), false) = (src, matches!(target, Value::Instance(_))) {
        // an exact dict: its stored hashes are reused, and it must not change meanwhile
        let n = o.borrow().len();
        let mut pos = 0;
        loop {
            let e = o.borrow().next_from(pos).map(|(p, e)| (p, e.hash, e.key.clone(), e.value.clone()));
            let Some((p, h, key, v)) = e else { break };
            pos = p + 1;
            match vm.dict_find_h(d, h, &key)?.1 {
                Some(i) => d.borrow_mut().set_at(i, v),
                None => d.borrow_mut().push_new(h, key, v),
            }
            if o.borrow().len() != n {
                return Err(vm.err("RuntimeError", "dict mutated during update"));
            }
        }
        for (key, v) in k {
            vm.dict_set(d, Value::Str(key.clone()), v.clone())?;
        }
        return Ok(());
    }
    if let Some(src) = src {
        let is_map = match src {
            Value::Dict(_) => true,
            Value::Instance(_) | Value::Obj(_) => vm.getattr_opt(src, &ps("keys"))?.is_some(),
            _ => false,
        };
        if is_map {
            let items = vm.mapping_items(src)?;
            for (key, v) in items {
                set(vm, key, v)?;
            }
        } else {
            let it = vm.iter(src)?;
            let mut i = 0;
            while let Some(pair) = vm.next(&it)? {
                let kv = match vm.to_vec(&pair) {
                    Ok(kv) => kv,
                    Err(_) => {
                        let tn = vm.type_name(&pair);
                        return Err(vm.type_error(format!("cannot convert dictionary update sequence element #{} to a sequence", i).replace("{tn}", &tn)));
                    }
                };
                if kv.len() != 2 {
                    return Err(vm.value_error(format!("dictionary update sequence element #{} has length {}; 2 is required", i, kv.len())));
                }
                let mut kv = kv.into_iter();
                let (key, v) = (kv.next().unwrap(), kv.next().unwrap());
                set(vm, key, v)?;
                i += 1;
            }
        }
    }
    for (key, v) in k {
        set(vm, Value::Str(key.clone()), v.clone())?;
    }
    Ok(())
}

fn dict_init(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    if a.len() > 2 {
        return Err(vm.type_error(format!("dict expected at most 1 argument, got {}", a.len() - 1)));
    }
    let d = dict_of(vm, &a[0], "__init__")?;
    dict_merge_into(vm, &a[0], &d, a.get(1), k)?;
    Ok(Value::None)
}

fn dict_update(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    if a.len() > 2 {
        return Err(vm.type_error(format!("update expected at most 1 argument, got {}", a.len() - 1)));
    }
    let d = dict_of(vm, &a[0], "update")?;
    dict_merge_into(vm, &a[0], &d, a.get(1), k)?;
    Ok(Value::None)
}

fn dict_ior(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let d = dict_of(vm, &a[0], "__ior__")?;
    dict_merge_into(vm, &a[0], &d, a.get(1), &[])?;
    Ok(a[0].clone())
}

fn dict_keys(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let d = dict_of(vm, &a[0], "keys")?;
    Ok(Value::DictView(Rc::new((d, 0))))
}
fn dict_values(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let d = dict_of(vm, &a[0], "values")?;
    Ok(Value::DictView(Rc::new((d, 1))))
}
fn dict_items(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let d = dict_of(vm, &a[0], "items")?;
    Ok(Value::DictView(Rc::new((d, 2))))
}

fn dict_get(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    if a.len() < 2 || a.len() > 3 {
        return Err(vm.type_error(format!("get expected at least 1 argument, got {}", a.len().saturating_sub(1))));
    }
    let d = dict_of(vm, &a[0], "get")?;
    Ok(vm.dict_get(&d, &a[1])?.unwrap_or_else(|| a.get(2).cloned().unwrap_or(Value::None)))
}

fn dict_setdefault(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    margs(vm, "setdefault", a, 1, 2)?;
    let d = dict_of(vm, &a[0], "setdefault")?;
    let (h, i) = vm.dict_find(&d, &a[1])?;
    if let Some(i) = i {
        if let Some(Some(e)) = d.borrow().entries.get(i) {
            return Ok(e.value.clone());
        }
    }
    let v = a.get(2).cloned().unwrap_or(Value::None);
    d.borrow_mut().push_new(h, a[1].clone(), v.clone());
    Ok(v)
}

fn dict_pop(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    margs(vm, "pop", a, 1, 2)?;
    let d = dict_of(vm, &a[0], "pop")?;
    match vm.dict_del(&d, &a[1])? {
        Some(v) => Ok(v),
        None => match a.get(2) {
            Some(v) => Ok(v.clone()),
            None => Err(vm.key_error(a[1].clone())),
        },
    }
}

fn dict_popitem(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let d = dict_of(vm, &a[0], "popitem")?;
    let e = d.borrow_mut().pop_last();
    match e {
        Some(e) => Ok(Value::tuple(vec![e.key, e.value])),
        None => Err(vm.err("KeyError", "popitem(): dictionary is empty")),
    }
}

fn dict_clear(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let d = dict_of(vm, &a[0], "clear")?;
    d.borrow_mut().clear();
    Ok(Value::None)
}

fn dict_copy(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let d = dict_of(vm, &a[0], "copy")?;
    let c = d.borrow().copy();
    Ok(Value::dict(c))
}

fn dict_fromkeys(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    if a.len() < 2 {
        return Err(vm.type_error("fromkeys expected at least 1 argument, got 0"));
    }
    let v = a.get(2).cloned().unwrap_or(Value::None);
    let r = vm.call_vec(&a[0], vec![], &[])?;
    if let (Value::Dict(d), Value::Dict(_) | Value::Set(_) | Value::FrozenSet(_)) = (&r, &a[1]) {
        if let Some(keys) = vm.hashed_keys(&a[1]) {
            for (h, k) in keys {
                match vm.dict_find_h(d, h, &k)?.1 {
                    Some(i) => d.borrow_mut().set_at(i, v.clone()),
                    None => d.borrow_mut().push_new(h, k, v.clone()),
                }
            }
            return Ok(r);
        }
    }
    let keys = vm.to_vec(&a[1])?;
    for key in keys {
        vm.setitem(&r, key, v.clone())?;
    }
    Ok(r)
}

fn dict_reversed(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let d = dict_of(vm, &a[0], "__reversed__")?;
    Ok(crate::builtins::dict_rev_iter(&d, 0))
}

fn view_reversed(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let f = vm.builtins.borrow().get_s("reversed").unwrap();
    vm.call_vec(&f, vec![a[0].clone()], k)
}

// -------------------------------------------------------------------------
// set and frozenset

fn set_of(vm: &mut Vm, v: &Value, m: &str) -> PyResult<Rc<RefCell<Dict>>> {
    match vm.base_of(v) {
        Value::Set(d) | Value::FrozenSet(d) => Ok(d),
        Value::DictView(dv) if dv.1 != 1 => match to_set(vm, &Value::DictView(dv))? {
            Value::Set(d) => Ok(d),
            _ => unreachable!(),
        },
        _ => {
            let tn = vm.type_name(v);
            Err(vm.type_error(format!("descriptor '{}' for 'set' objects doesn't apply to a '{}' object", m, tn)))
        }
    }
}

fn set_new(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let t = vm.t.set.clone();
    let v = set_from(Dict::new());
    Ok(wrap_new(vm, &a[0], &t, v))
}

fn set_init(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    if !k.is_empty() {
        return Err(vm.type_error("set() takes no keyword arguments"));
    }
    if a.len() > 2 {
        return Err(vm.type_error(format!("set expected at most 1 argument, got {}", a.len() - 1)));
    }
    let d = set_of(vm, &a[0], "__init__")?;
    d.borrow_mut().clear();
    if let Some(src) = a.get(1) {
        vm.set_extend(&d, src)?;
    }
    Ok(Value::None)
}

fn frozenset_new(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let name = match &a[0] {
        Value::Class(c) => c.name(),
        _ => "frozenset".into(),
    };
    if !k.is_empty() && native_slot(vm, &a[0], "__init__", "object.__init__") {
        return Err(vm.type_error("frozenset() takes no keyword arguments"));
    }
    if a.len() > 2 {
        return Err(vm.type_error(format!("{} expected at most 1 argument, got {}", name, a.len() - 1)));
    }
    let d = new_dict();
    if let Some(src) = a.get(1) {
        if let Value::FrozenSet(_) = src {
            if matches!(&a[0], Value::Class(c) if Rc::ptr_eq(c, &vm.t.frozenset)) {
                return Ok(src.clone());
            }
        }
        vm.set_extend(&d, src)?;
    }
    let t = vm.t.frozenset.clone();
    Ok(wrap_new(vm, &a[0], &t, Value::FrozenSet(d)))
}

fn same_kind(a: &Value, d: Rc<RefCell<Dict>>) -> Value {
    match a {
        Value::FrozenSet(_) => Value::FrozenSet(d),
        _ => Value::Set(d),
    }
}

fn set_add(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 2)?;
    margs(vm, "set.add", a, 1, 1)?;
    let d = set_of(vm, &a[0], "add")?;
    vm.set_add(&d, a[1].clone())?;
    Ok(Value::None)
}

fn set_discard(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 2)?;
    margs(vm, "set.discard", a, 1, 1)?;
    let d = set_of(vm, &a[0], "discard")?;
    let key = Vm::set_key(&a[1]).unwrap_or_else(|| a[1].clone());
    vm.dict_del(&d, &key)?;
    Ok(Value::None)
}

fn set_remove(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 2)?;
    margs(vm, "set.remove", a, 1, 1)?;
    let d = set_of(vm, &a[0], "remove")?;
    let key = Vm::set_key(&a[1]).unwrap_or_else(|| a[1].clone());
    if vm.dict_del(&d, &key)?.is_none() {
        return Err(vm.key_error(a[1].clone()));
    }
    Ok(Value::None)
}

fn set_pop(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let d = set_of(vm, &a[0], "pop")?;
    let first = d.borrow().next_from(0).map(|(i, _)| i);
    match first {
        Some(i) => {
            let e = d.borrow_mut().remove_at(i).unwrap();
            Ok(e.key)
        }
        None => Err(vm.err("KeyError", "pop from an empty set")),
    }
}

fn set_clear(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let d = set_of(vm, &a[0], "clear")?;
    d.borrow_mut().clear();
    Ok(Value::None)
}

fn set_copy(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    if let Value::FrozenSet(_) = &a[0] {
        return Ok(a[0].clone());
    }
    let me = this(vm, a);
    let d = set_of(vm, &a[0], "copy")?;
    let c = d.borrow().copy();
    Ok(same_kind(&me, Rc::new(RefCell::new(c))))
}

fn multi_op(vm: &mut Vm, a: &[Value], op: BinOp) -> PyResult<Value> {
    let me = this(vm, a);
    let mut acc = match &me {
        Value::Set(d) => Value::Set(Rc::new(RefCell::new(d.borrow().copy()))),
        Value::FrozenSet(d) => Value::FrozenSet(Rc::new(RefCell::new(d.borrow().copy()))),
        other => to_set(vm, other)?,
    };
    for o in &a[1..] {
        let s = to_set(vm, o)?;
        acc = set_op(vm, op, &acc, &s)?;
    }
    Ok(acc)
}

fn set_union(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    multi_op(vm, a, BinOp::BitOr)
}
fn set_intersection(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    multi_op(vm, a, BinOp::BitAnd)
}
fn set_difference(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    multi_op(vm, a, BinOp::Sub)
}
fn set_symmetric_difference(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    margs(vm, "symmetric_difference", a, 1, 1)?;
    multi_op(vm, a, BinOp::BitXor)
}

fn in_place(vm: &mut Vm, a: &[Value], op: BinOp) -> PyResult<Value> {
    let d = set_of(vm, &a[0], "update")?;
    let r = multi_op(vm, a, op)?;
    if let Value::Set(nd) | Value::FrozenSet(nd) = r {
        let c = nd.borrow().copy();
        *d.borrow_mut() = c;
    }
    Ok(Value::None)
}

fn set_update(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let d = set_of(vm, &a[0], "update")?;
    for o in &a[1..] {
        vm.set_extend(&d, o)?;
    }
    Ok(Value::None)
}
fn set_intersection_update(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    in_place(vm, a, BinOp::BitAnd)
}
fn set_difference_update(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    in_place(vm, a, BinOp::Sub)
}
fn set_symmetric_difference_update(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    in_place(vm, a, BinOp::BitXor)
}

fn check_set_operand(vm: &mut Vm, v: &Value) -> bool {
    matches!(vm.base_of(v), Value::Set(_) | Value::FrozenSet(_))
}

fn set_ior(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 2)?;
    if !check_set_operand(vm, &a[1]) {
        return Ok(Value::NotImplemented);
    }
    set_update(vm, a, k)?;
    Ok(a[0].clone())
}
fn set_iand(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 2)?;
    if !check_set_operand(vm, &a[1]) {
        return Ok(Value::NotImplemented);
    }
    in_place(vm, a, BinOp::BitAnd)?;
    Ok(a[0].clone())
}
fn set_isub(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 2)?;
    if !check_set_operand(vm, &a[1]) {
        return Ok(Value::NotImplemented);
    }
    in_place(vm, a, BinOp::Sub)?;
    Ok(a[0].clone())
}
fn set_ixor(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 2)?;
    if !check_set_operand(vm, &a[1]) {
        return Ok(Value::NotImplemented);
    }
    in_place(vm, a, BinOp::BitXor)?;
    Ok(a[0].clone())
}

fn set_issubset(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 2)?;
    margs(vm, "issubset", a, 1, 1)?;
    let me = set_of(vm, &a[0], "issubset")?;
    let o = to_set(vm, &a[1])?;
    let od = set_of(vm, &o, "issubset")?;
    Ok(Value::Bool(is_subset(vm, &me, &od)?))
}
fn set_issuperset(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    margs(vm, "issuperset", a, 1, 1)?;
    let me = set_of(vm, &a[0], "issuperset")?;
    let o = to_set(vm, &a[1])?;
    let od = set_of(vm, &o, "issuperset")?;
    Ok(Value::Bool(is_subset(vm, &od, &me)?))
}
fn set_isdisjoint(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 2)?;
    margs(vm, "isdisjoint", a, 1, 1)?;
    let me = set_of(vm, &a[0], "isdisjoint")?;
    let items = vm.to_vec(&a[1])?;
    for x in items {
        if vm.set_contains(&me, &x)? {
            return Ok(Value::Bool(false));
        }
    }
    Ok(Value::Bool(true))
}

fn set_cmp(vm: &mut Vm, a: &[Value], op: CmpOp) -> PyResult<Value> {
    let x = this(vm, a);
    let y = vm.base_of(&a[1]);
    if !matches!(y, Value::Set(_) | Value::FrozenSet(_) | Value::DictView(_)) {
        return Ok(Value::NotImplemented);
    }
    set_compare(vm, op, &x, &y)
}
fn set_le(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    set_cmp(vm, a, CmpOp::Le)
}
fn set_lt(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    set_cmp(vm, a, CmpOp::Lt)
}
fn set_ge(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    set_cmp(vm, a, CmpOp::Ge)
}
fn set_gt(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    set_cmp(vm, a, CmpOp::Gt)
}

// -------------------------------------------------------------------------
// range, slice, None

fn range_new(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    if !k.is_empty() {
        return Err(vm.type_error("range() takes no keyword arguments"));
    }
    let n = a.len().saturating_sub(1);
    if n == 0 {
        return Err(vm.type_error("range expected at least 1 argument, got 0"));
    }
    if n > 3 {
        return Err(vm.type_error(format!("range expected at most 3 arguments, got {}", n)));
    }
    let mut ints = vec![];
    for x in &a[1..] {
        ints.push(crate::mathmod::index_big(vm, x)?);
    }
    let one = BigInt::from_i64(1);
    let (start, stop, step) = match ints.as_slice() {
        [e] => (BigInt::zero(), e.clone(), one),
        [s, e] => (s.clone(), e.clone(), one),
        [s, e, st] => (s.clone(), e.clone(), st.clone()),
        _ => return Err(vm.type_error("range expected at least 1 argument, got 0")),
    };
    if step.is_zero() {
        return Err(vm.value_error("range() arg 3 must not be zero"));
    }
    Ok(crate::range::make(start, stop, step))
}

fn range_reversed(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let f = vm.builtins.borrow().get_s("reversed").unwrap();
    vm.call_vec(&f, vec![a[0].clone()], k)
}

fn slice_new(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let args = &a[1..];
    let s = match args.len() {
        1 => (Value::None, args[0].clone(), Value::None),
        2 => (args[0].clone(), args[1].clone(), Value::None),
        3 => (args[0].clone(), args[1].clone(), args[2].clone()),
        n => return Err(vm.type_error(format!("slice expected at least 1 argument, got {}", n))),
    };
    Ok(Value::Slice(Rc::new(s)))
}

fn slice_hash(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let h = vm.hash(&a[0])?;
    Ok(Value::Int(h as i64))
}

fn slice_reduce(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    match &a[0] {
        Value::Slice(s) => Ok(Value::tuple(vec![Value::Class(vm.t.slice.clone()), Value::tuple(vec![s.0.clone(), s.1.clone(), s.2.clone()])])),
        _ => Err(vm.type_error("descriptor '__reduce__' requires a 'slice' object")),
    }
}

/// slice.indices with arbitrary-size integers (CPython's _PySlice_GetLongIndices)
fn slice_indices_big(vm: &mut Vm, s: &(Value, Value, Value), length: BigInt) -> PyResult<Value> {
    let zero = BigInt::zero();
    let one = BigInt::from_i64(1);
    let step = if s.2.is_none() { one.clone() } else { crate::mathmod::index_big(vm, &s.2)? };
    if step.is_zero() {
        return Err(vm.value_error("slice step cannot be zero"));
    }
    let back = step.neg;
    let (lower, upper) = if back { (one.neg(), length.sub(&one)) } else { (zero.clone(), length.clone()) };
    let clip = |vm: &mut Vm, v: &Value, dflt: &BigInt| -> PyResult<BigInt> {
        if v.is_none() {
            return Ok(dflt.clone());
        }
        let mut x = crate::mathmod::index_big(vm, v)?;
        if x.neg {
            x = x.add(&length);
            if x.cmp(&lower) == std::cmp::Ordering::Less {
                x = lower.clone();
            }
        } else if x.cmp(&upper) == std::cmp::Ordering::Greater {
            x = upper.clone();
        }
        Ok(x)
    };
    let start = clip(vm, &s.0, if back { &upper } else { &lower })?;
    let stop = clip(vm, &s.1, if back { &lower } else { &upper })?;
    Ok(Value::tuple(vec![Value::from_big(start), Value::from_big(stop), Value::from_big(step)]))
}

fn slice_indices_m(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 2)?;
    margs(vm, "indices", a, 1, 1)?;
    if let Value::Slice(s) = &a[0] {
        for v in [&s.0, &s.1, &s.2] {
            if !v.is_none() && !matches!(v, Value::Int(_) | Value::Big(_) | Value::Bool(_)) && vm.lookup_special(v, &vm.n.index.clone()).is_none() {
                return Err(vm.type_error("slice indices must be integers or None or have an __index__ method"));
            }
        }
        let n = crate::mathmod::index_big(vm, &a[1])?;
        if n.neg {
            return Err(vm.value_error("length should not be negative"));
        }
        let s = s.clone();
        return slice_indices_big(vm, &s, n);
    }
    let n = vm.to_index(&a[1])?;
    if n < 0 {
        return Err(vm.value_error("length should not be negative"));
    }
    match &a[0] {
        Value::Slice(s) => {
            let (x, y, z) = slice_indices(vm, s, n as usize)?;
            Ok(Value::tuple(vec![Value::Int(x), Value::Int(y), Value::Int(z)]))
        }
        _ => Err(vm.type_error("descriptor 'indices' requires a 'slice' object")),
    }
}

fn iterator_new(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let tn = match a.first() {
        Some(Value::Class(c)) => c.name().to_string(),
        _ => "iterator".to_string(),
    };
    Err(vm.type_error(format!("cannot create '{}' instances", tn)))
}

fn none_new(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    singleton_new(vm, a, k, "NoneType", Value::None)
}
fn notimpl_new(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    singleton_new(vm, a, k, "NotImplementedType", Value::NotImplemented)
}
fn ellipsis_new(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    singleton_new(vm, a, k, "ellipsis", Value::Ellipsis)
}
fn singleton_new(vm: &mut Vm, a: &[Value], k: Kw, name: &str, v: Value) -> PyResult<Value> {
    if a.len() > 1 || !k.is_empty() {
        return Err(vm.type_error(format!("{} takes no arguments", name)));
    }
    Ok(v)
}

// -------------------------------------------------------------------------
// functions, modules, iterators, generators

fn function_get(_vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    match a.get(1) {
        Some(Value::None) | None => Ok(a[0].clone()),
        Some(obj) => Ok(Value::Method(Rc::new((obj.clone(), a[0].clone())))),
    }
}

fn function_call(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    vm.call_vec(&a[0], a[1..].to_vec(), k)
}

fn module_new(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let name = match arg_or_kw(a, 1, k, "name") {
        Some(Value::Str(s)) => s,
        _ => return Err(vm.type_error("module.__new__() missing required argument 'name'")),
    };
    // an instance of a subclass shares its __dict__ with the module it wraps
    let module_cls = vm.t.module.clone();
    let inst = wrap_new(vm, &a[0], &module_cls, Value::None);
    let d = match &inst {
        Value::Instance(i) => i.dict.clone(),
        _ => new_dict(),
    };
    d.borrow_mut().set_s("__name__", Value::Str(name.clone()));
    d.borrow_mut().set_s("__doc__", arg_or_kw(a, 2, k, "doc").unwrap_or(Value::None));
    d.borrow_mut().set_s("__package__", Value::None);
    d.borrow_mut().set_s("__spec__", Value::None);
    d.borrow_mut().set_s("__loader__", Value::None);
    let m = Value::Module(Rc::new(Module { name, dict: d }));
    match &inst {
        Value::Instance(i) => {
            *i.base.borrow_mut() = m;
            Ok(inst)
        }
        _ => Ok(m),
    }
}

fn iter_next(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let me = vm.base_of(&a[0]);
    match vm.next(&me)? {
        Some(v) => Ok(v),
        None => {
            let rv = vm.last_return.take().unwrap_or(Value::None);
            Err(vm.stop_iteration(rv))
        }
    }
}

fn iter_length_hint(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    if let Value::Iter(it) = &a[0] {
        let rev = match &*it.borrow() {
            Iter::SeqRev(obj, i) => Some((obj.clone(), *i)),
            _ => None,
        };
        if let Some((obj, i)) = rev {
            if i < 0 {
                return Ok(Value::Int(0));
            }
            let n = vm.len(&obj)? as i64;
            return Ok(Value::Int(if n < i + 1 { 0 } else { i + 1 }));
        }
        let n = match &*it.borrow() {
            Iter::List(l, i) => l.borrow().len().saturating_sub(*i),
            Iter::ListRev(l, i) => if *i <= l.borrow().len() { *i } else { 0 },
            Iter::Tuple(t, i) => t.len().saturating_sub(*i),
            Iter::ListRev(l, i) => (*i).min(l.borrow().len()),
            Iter::Range(_, left, _) => (*left).max(0) as usize,
            Iter::BigRange(st) => return Ok(Value::from_big(st.2.clone())),
            Iter::Vec(v) => v.len(),
            Iter::Dict(d, _, _, size, left) => if d.borrow().len() == *size { *left } else { 0 },
            _ => 0,
        };
        return Ok(Value::Int(n as i64));
    }
    Ok(Value::Int(0))
}

fn gen_of(vm: &mut Vm, v: &Value) -> PyResult<Rc<RefCell<crate::vm::Generator>>> {
    match v {
        Value::Gen(g) => Ok(g.clone()),
        _ => Err(vm.type_error("expected a generator")),
    }
}

fn gen_send(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    margs(vm, "send", a, 1, 1)?;
    let g = gen_of(vm, &a[0])?;
    match vm.resume(&g, a[1].clone(), None)? {
        Some(v) => Ok(v),
        None => {
            let rv = vm.last_return.take().unwrap_or(Value::None);
            Err(vm.stop_iteration(rv))
        }
    }
}

fn gen_throw(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    margs(vm, "throw", a, 1, 3)?;
    let g = gen_of(vm, &a[0])?;
    let mut e = a[1].clone();
    if a.len() > 2 {
        crate::tb::warn(vm, "DeprecationWarning", "the (type, exc, tb) signature of throw() is deprecated, use the single-arg signature instead.")?;
    }
    if let Value::Class(c) = &e {
        let args = match a.get(2) {
            Some(Value::None) | None => vec![],
            Some(Value::Tuple(t)) => t.to_vec(),
            Some(v) => vec![v.clone()],
        };
        let c = c.clone();
        // an error making the exception is thrown into the generator instead
        e = match vm.call_vec(&Value::Class(c.clone()), args, &[]) {
            Ok(v) if vm.class_of(&v).is_subclass(&vm.t.base_exception) => v,
            Ok(v) => {
                let tn = vm.type_name(&v);
                vm.type_error(format!("calling {} should have returned an instance of BaseException, not {}", crate::format::class_repr(&c), tn)).value
            }
            Err(x) => x.value,
        };
    }
    let e = vm.make_raisable(e)?;
    match vm.resume(&g, Value::None, Some(e))? {
        Some(v) => Ok(v),
        None => {
            let rv = vm.last_return.take().unwrap_or(Value::None);
            Err(vm.stop_iteration(rv))
        }
    }
}

fn gen_close(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let g = gen_of(vm, &a[0])?;
    let started = g.borrow().started;
    if !started || g.borrow().frame.is_none() {
        g.borrow_mut().frame = None;
        return Ok(Value::None);
    }
    let cls = vm.exc_class("GeneratorExit");
    let e = vm.new_exc(&cls, vec![]);
    match vm.resume(&g, Value::None, Some(e)) {
        Ok(Some(_)) => {
            g.borrow_mut().frame = None;
            Err(vm.err("RuntimeError", "generator ignored GeneratorExit"))
        }
        Ok(None) => Ok(vm.last_return.take().unwrap_or(Value::None)),
        Err(e) if vm.exc_is(&e, "GeneratorExit") || vm.exc_is(&e, "StopIteration") => {
            g.borrow_mut().frame = None;
            Ok(Value::None)
        }
        Err(e) => {
            g.borrow_mut().frame = None;
            Err(e)
        }
    }
}

// -------------------------------------------------------------------------
// property, staticmethod, classmethod, super

/// property(fget, fset, fdel, doc): the doc falls back to fget.__doc__
fn new_property(vm: &mut Vm, fget: Value, fset: Value, fdel: Value, doc: Value) -> PyResult<Property> {
    let mut doc = doc;
    let mut getter_doc = false;
    if doc.is_none() && !fget.is_none() {
        let d = match vm.getattr_opt(&fget, &ps("__doc__")) {
            Ok(d) => d,
            Err(e) if vm.exc_is(&e, "AttributeError") => None,
            Err(e) => return Err(e),
        };
        if let Some(d) = d {
            if !d.is_none() {
                doc = d;
                getter_doc = true;
            }
        }
    }
    Ok(Property { fget, fset, fdel, doc: RefCell::new(doc), name: RefCell::new(Value::Undef), getter_doc: Cell::new(getter_doc) })
}

fn make_property(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Property> {
    if a.len() > 5 {
        return Err(vm.type_error(format!("property() takes at most 4 arguments ({} given)", a.len() - 1)));
    }
    let get = |i: usize, n: &str| arg_or_kw(a, i, k, n).unwrap_or(Value::None);
    new_property(vm, get(1, "fget"), get(2, "fset"), get(3, "fdel"), get(4, "doc"))
}

fn property_new(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let p = Value::Property(Rc::new(make_property(vm, a, k)?));
    let t = vm.t.property.clone();
    Ok(wrap_new(vm, &a[0], &t, p))
}

fn property_init(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    if let Value::Instance(i) = &a[0] {
        let p = make_property(vm, a, k)?;
        let doc = p.doc.borrow().clone();
        let getter_doc = p.getter_doc.get();
        *i.base.borrow_mut() = Value::Property(Rc::new(p));
        // a subclass keeps the doc in the instance, where it shadows the
        // class's own __doc__
        match vm.setattr(&a[0], &ps("__doc__"), doc) {
            Ok(()) => {}
            Err(e) if !getter_doc && vm.exc_is(&e, "AttributeError") => {}
            Err(e) => return Err(e),
        }
    }
    Ok(Value::None)
}

fn prop_of(vm: &mut Vm, v: &Value) -> PyResult<Rc<Property>> {
    match vm.base_of(v) {
        Value::Property(p) => Ok(p),
        _ => Err(vm.type_error("descriptor requires a 'property' object")),
    }
}

/// "property 'x' of 'C' object has no getter"
pub(crate) fn prop_missing(vm: &mut Vm, p: &Property, obj: &Value, what: &str) -> Exc {
    let q = vm.class_of(obj).qualname.borrow().s.to_string();
    let name = p.name.borrow().clone();
    let msg = if name.is_undef() {
        format!("property of '{}' object has no {}", q, what)
    } else {
        match vm.repr(&name) {
            Ok(r) => format!("property {} of '{}' object has no {}", r, q, what),
            Err(e) => return e,
        }
    };
    vm.attr_error(msg)
}

fn property_get(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let p = prop_of(vm, &a[0])?;
    match a.get(1) {
        None | Some(Value::None) => Ok(a[0].clone()),
        Some(obj) => {
            if p.fget.is_none() {
                return Err(prop_missing(vm, &p, obj, "getter"));
            }
            vm.call_vec(&p.fget.clone(), vec![obj.clone()], &[])
        }
    }
}

fn property_set(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 3)?;
    let p = prop_of(vm, &a[0])?;
    if p.fset.is_none() {
        return Err(prop_missing(vm, &p, &a[1], "setter"));
    }
    vm.call_vec(&p.fset.clone(), vec![a[1].clone(), a[2].clone()], &[])?;
    Ok(Value::None)
}

fn property_delete(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 2)?;
    let p = prop_of(vm, &a[0])?;
    if p.fdel.is_none() {
        return Err(prop_missing(vm, &p, &a[1], "deleter"));
    }
    vm.call_vec(&p.fdel.clone(), vec![a[1].clone()], &[])?;
    Ok(Value::None)
}

fn property_set_name(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    if a.len() != 3 {
        return Err(vm.type_error(format!("__set_name__() takes 2 positional arguments but {} were given", a.len().saturating_sub(1))));
    }
    let p = prop_of(vm, &a[0])?;
    *p.name.borrow_mut() = a[2].clone();
    Ok(Value::None)
}

fn prop_copy(vm: &mut Vm, a: &[Value], which: u8) -> PyResult<Value> {
    margs(vm, "property", a, 1, 1)?;
    let p = prop_of(vm, &a[0])?;
    let f = a[1].clone();
    let (g, s, d) = match which {
        0 => (f, p.fset.clone(), p.fdel.clone()),
        1 => (p.fget.clone(), f, p.fdel.clone()),
        _ => (p.fget.clone(), p.fset.clone(), f),
    };
    // a doc taken from the old getter is taken from the new one instead
    let doc = if p.getter_doc.get() && !g.is_none() { Value::None } else { p.doc.borrow().clone() };
    let new = if let Value::Instance(_) = &a[0] {
        let cls = vm.class_of(&a[0]);
        vm.call_vec(&Value::Class(cls), vec![g, s, d, doc], &[])?
    } else {
        Value::Property(Rc::new(new_property(vm, g, s, d, doc)?))
    };
    if let Value::Property(np) = vm.base_of(&new) {
        *np.name.borrow_mut() = p.name.borrow().clone();
    }
    Ok(new)
}

fn property_getter(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    prop_copy(vm, a, 0)
}
fn property_setter(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    prop_copy(vm, a, 1)
}
fn property_deleter(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    prop_copy(vm, a, 2)
}

fn staticmethod_new(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    if a.len() != 2 {
        return Err(vm.type_error(format!("staticmethod expected 1 argument, got {}", a.len().saturating_sub(1))));
    }
    Ok(Value::StaticMethod(Rc::new(a[1].clone())))
}
fn staticmethod_get(_vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    match &a[0] {
        Value::StaticMethod(f) => Ok((**f).clone()),
        o => Ok(o.clone()),
    }
}
fn staticmethod_call(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    match &a[0] {
        Value::StaticMethod(f) => vm.call_vec(&f.clone(), a[1..].to_vec(), k),
        _ => Err(vm.type_error("not a staticmethod")),
    }
}
fn classmethod_new(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    if a.len() != 2 {
        return Err(vm.type_error(format!("classmethod expected 1 argument, got {}", a.len().saturating_sub(1))));
    }
    Ok(Value::ClassMethod(Rc::new(a[1].clone())))
}
fn classmethod_get(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let owner = match (a.get(1), a.get(2)) {
        (_, Some(c @ Value::Class(_))) => c.clone(),
        (Some(o), _) if !o.is_none() => Value::Class(vm.class_of(o)),
        _ => return Err(vm.type_error("__get__(None, None) is invalid")),
    };
    match &a[0] {
        Value::ClassMethod(f) => Ok(Value::Method(Rc::new((owner, (**f).clone())))),
        o => Ok(o.clone()),
    }
}

/// super(type, obj): obj must be an instance or subclass of type
fn super_check(vm: &mut Vm, c: &Rc<Class>, obj: &Value) -> PyResult<()> {
    let ok = match obj {
        Value::Class(k) => k.is_subclass(c) || vm.isinstance(obj, c),
        o => vm.isinstance(o, c),
    };
    if !ok {
        let what = match obj {
            Value::Class(k) => format!("type {}", k.name()),
            o => format!("instance of {}", vm.type_name(o)),
        };
        return Err(vm.type_error(format!("super(type, obj): obj ({}) is not an instance or subtype of type ({}).", what, c.name())));
    }
    Ok(())
}

fn super_new(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    if !k.is_empty() {
        return Err(vm.type_error("super() takes no keyword arguments"));
    }
    let (start, obj) = match a.len() {
        3 => match &a[1] {
            Value::Class(c) => {
                super_check(vm, c, &a[2])?;
                (c.clone(), a[2].clone())
            }
            _ => return Err(vm.type_error(format!("super() argument 1 must be a type, not {}", vm.type_name(&a[1])))),
        },
        1 => {
            // zero arguments: the __class__ cell and the first argument
            let f = match vm.current_frame() {
                Some(f) => f,
                None => return Err(vm.err("RuntimeError", "super(): no current frame")),
            };
            let code = f.code.clone();
            if code.argcount == 0 && !code.varargs {
                return Err(vm.err("RuntimeError", "super(): no arguments"));
            }
            let first = match code.cellvars.iter().position(|c| code.varnames.first().map(|v| v == c).unwrap_or(false)) {
                Some(i) => f.cells.get(i).map(|c| c.borrow().clone()).unwrap_or(Value::Undef),
                None => {
                    if code.argcount == 0 {
                        match f.stack.first() {
                            Some(Value::Tuple(t)) if !t.is_empty() => t[0].clone(),
                            _ => return Err(vm.err("RuntimeError", "super(): no arguments")),
                        }
                    } else {
                        f.stack.first().cloned().unwrap_or(Value::Undef)
                    }
                }
            };
            if first.is_undef() {
                return Err(vm.err("RuntimeError", "super(): arg[0] deleted"));
            }
            let ci = code.freevars.iter().position(|n| &**n == "__class__");
            let cls = match ci.and_then(|i| f.cells.get(code.cellvars.len() + i)) {
                Some(c) => c.borrow().clone(),
                None => return Err(vm.err("RuntimeError", "super(): __class__ cell not found")),
            };
            match cls {
                Value::Class(c) => {
                    super_check(vm, &c, &first)?;
                    (c, first)
                }
                Value::Undef => return Err(vm.err("RuntimeError", "super(): empty __class__ cell")),
                other => {
                    let tn = vm.type_name(&other);
                    return Err(vm.err("RuntimeError", format!("super(): __class__ is not a type ({})", tn)));
                }
            }
        }
        2 => match &a[1] {
            Value::Class(c) => (c.clone(), Value::None),
            _ => return Err(vm.type_error(format!("super() argument 1 must be a type, not {}", vm.type_name(&a[1])))),
        },
        n => return Err(vm.type_error(format!("super() expected at most 2 arguments, got {}", n - 1))),
    };
    let st = vm.t.super_.clone();
    let v = Value::Super(Rc::new((start, obj)));
    Ok(wrap_new(vm, &a[0], &st, v))
}

macro_rules! iter_class_new {
    ($( $fname:ident, $bi:ident, $field:ident );* $(;)?) => {
        $(
            fn $fname(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
                crate::builtins::need(vm, a, 1)?;
                let c = vm.t.$field.clone();
                let mut k = k;
                if !k.is_empty() && matches!(&a[0], Value::Class(x) if !Rc::ptr_eq(x, &c)) && stringify!($field) != "zip" && stringify!($field) != "enumerate" {
                    // a subclass that defines __init__ takes the keywords there
                    let own_init = matches!(&a[0], Value::Class(x) if matches!(x.lookup(&ps("__init__")), Some(Value::Func(_))));
                    if !own_init {
                        return Err(vm.type_error(format!("{}() takes no keyword arguments", stringify!($field))));
                    }
                    k = &[];
                }
                let v = crate::builtins::$bi(vm, &a[1..], k)?;
                Ok(wrap_new(vm, &a[0], &c, v))
            }
        )*
    };
}

iter_class_new! {
    enumerate_new, bi_enumerate, enumerate;
    zip_new, bi_zip, zip;
    map_new, bi_map, map;
    filter_new, bi_filter, filter;
    reversed_new, bi_reversed, reversed;
}

/// pickling iterators, as CPython's do: a callable, its arguments and the
/// position to resume from
fn iter_reduce(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let me = vm.base_of(&a[0]);
    let it = match &me {
        Value::Iter(it) => it.clone(),
        _ => return object_reduce(vm, a, &[]),
    };
    let iter_fn = vm.frame_builtin("iter")?;
    let own = Value::Class(vm.class_of(&a[0]));
    let mut rev = Value::Class(vm.t.reversed.clone());
    if matches!(&*it.borrow(), Iter::ListRev(..)) {
        rev = vm.frame_builtin("reversed")?;
    }
    let tn = vm.type_name(&a[0]);
    let t3 = |f: Value, args: Vec<Value>, state: Option<Value>| match state {
        Some(s) => Value::tuple(vec![f, Value::tuple(args), s]),
        None => Value::tuple(vec![f, Value::tuple(args)]),
    };
    let empty = |f: Value| Value::tuple(vec![f, Value::tuple(vec![Value::list(vec![])])]);
    let empty_t = |f: Value| Value::tuple(vec![f, Value::tuple(vec![Value::tuple(vec![])])]);
    let b = it.borrow();
    Ok(match &*b {
        Iter::List(l, i) => {
            if *i == usize::MAX / 2 {
                empty(iter_fn)
            } else {
                t3(iter_fn, vec![Value::List(l.clone())], Some(Value::Int(*i as i64)))
            }
        }
        Iter::ListRev(l, i) => {
            if *i == usize::MAX || *i == 0 || *i > l.borrow().len() {
                empty(rev)
            } else {
                t3(rev, vec![Value::List(l.clone())], Some(Value::Int(*i as i64 - 1)))
            }
        }
        Iter::Tuple(t, i) => {
            if *i >= t.len() {
                empty_t(iter_fn)
            } else {
                t3(iter_fn, vec![Value::Tuple(t.clone())], Some(Value::Int(*i as i64)))
            }
        }
        Iter::Str(s, i) if *i >= s.s.len() => t3(iter_fn, vec![Value::str("")], None),
        Iter::Str(s, i) => {
            let n = s.s[..(*i).min(s.s.len())].chars().count();
            t3(iter_fn, vec![Value::Str(s.clone())], Some(Value::Int(n as i64)))
        }
        Iter::Bytes(bs, i) if *i >= bs.len() => empty_t(iter_fn),
        Iter::Bytes(bs, i) => t3(iter_fn, vec![Value::Bytes(bs.clone())], Some(Value::Int(*i as i64))),
        Iter::Range(cur, left, step) => {
            let (c, s) = (*cur as i128, *step as i128);
            t3(iter_fn, vec![crate::range::make_i128(c, c + (*left).max(0) as i128 * s, s)], Some(Value::Int(0)))
        }
        Iter::BigRange(st) => {
            let stop = st.0.add(&st.2.mul(&st.1));
            t3(iter_fn, vec![crate::range::make(st.0.clone(), stop, st.1.clone())], Some(Value::Int(0)))
        }
        Iter::Dict(d, pos, kind, _, _) => {
            let db = d.borrow();
            let mut rest = vec![];
            let mut p = *pos;
            let back = *kind & 4 != 0;
            while let Some((q, e)) = if back { db.prev_from(p) } else { db.next_from(p) } {
                rest.push(match *kind & 3 {
                    0 | 3 => e.key.clone(),
                    1 => e.value.clone(),
                    _ => Value::tuple(vec![e.key.clone(), e.value.clone()]),
                });
                p = if back { q } else { q + 1 };
            }
            t3(iter_fn, vec![Value::list(rest)], None)
        }
        Iter::Vec(v) => t3(iter_fn, vec![Value::list(v.as_slice().to_vec())], None),
        Iter::Empty => empty_t(iter_fn),
        Iter::Seq(obj, i) => t3(iter_fn, vec![obj.clone()], Some(Value::Int(*i))),
        Iter::SeqRev(obj, i) => t3(rev, vec![obj.clone()], Some(Value::Int(*i))),
        Iter::Callable(f, s) => t3(iter_fn, vec![f.clone(), s.clone()], None),
        Iter::Enumerate(inner, count, _) => {
            let (inner, count) = (inner.clone(), count.clone());
            let c = own;
            t3(c, vec![inner, count], None)
        }
        Iter::Zip(its, strict, _) => {
            let (its, strict) = (its.clone(), *strict);
            let c = own;
            t3(c, its, if strict { Some(Value::Bool(true)) } else { None })
        }
        Iter::Map(f, its) => {
            let mut args = vec![f.clone()];
            args.extend(its.iter().cloned());
            let c = own;
            t3(c, args, None)
        }
        Iter::Filter(f, inner, false) => {
            let args = vec![f.clone(), inner.clone()];
            let c = own;
            t3(c, args, None)
        }
        _ => {
            drop(b);
            return Err(vm.type_error(format!("cannot pickle '{}' object", tn)));
        }
    })
}

fn iter_setstate(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 2)?;
    margs(vm, "__setstate__", a, 1, 1)?;
    let me = vm.base_of(&a[0]);
    let it = match &me {
        Value::Iter(it) => it.clone(),
        _ => return Ok(Value::None),
    };
    if let Iter::Zip(_, strict, _) = &mut *it.borrow_mut() {
        *strict = vm.truthy(&a[1])?;
        return Ok(Value::None);
    }
    if matches!(&*it.borrow(), Iter::Range(..) | Iter::BigRange(_)) {
        let n = crate::mathmod::index_big(vm, &a[1])?;
        let mut b = it.borrow_mut();
        let st = match &*b {
            Iter::Range(cur, left, step) => (BigInt::from_i64(*cur), BigInt::from_i64(*step), BigInt::from_i64(*left)),
            Iter::BigRange(st) => (**st).clone(),
            _ => return Ok(Value::None),
        };
        let n = if n.neg { BigInt::zero() } else if n.cmp(&st.2) == std::cmp::Ordering::Greater { st.2.clone() } else { n };
        let start = st.0.add(&n.mul(&st.1));
        let left = st.2.sub(&n);
        *b = match &*b {
            Iter::Range(..) => match (start.to_i64(), left.to_i64()) {
                (Some(c), Some(l)) => Iter::Range(c, l, st.1.to_i64().unwrap_or(1)),
                _ => Iter::BigRange(Box::new((start, st.1, left))),
            },
            _ => Iter::BigRange(Box::new((start, st.1, left))),
        };
        return Ok(Value::None);
    }
    let seq = match &*it.borrow() {
        Iter::SeqRev(obj, _) => Some(obj.clone()),
        _ => None,
    };
    if let Some(obj) = seq {
        let n = vm.to_index(&a[1])?;
        let len = vm.len(&obj)? as i64;
        if let Iter::SeqRev(_, i) = &mut *it.borrow_mut() {
            *i = n.max(-1).min(len - 1);
        }
        return Ok(Value::None);
    }
    let raw = vm.to_index(&a[1])?;
    let n = raw.max(0);
    let mut b = it.borrow_mut();
    match &mut *b {
        Iter::List(l, i) => *i = (n as usize).min(l.borrow().len()),
        Iter::ListRev(l, i) => *i = if raw < 0 { 0 } else { ((n + 1) as usize).min(l.borrow().len()) },
        Iter::Tuple(t, i) => *i = (n as usize).min(t.len()),
        Iter::Bytes(bs, i) => *i = (n as usize).min(bs.len()),
        Iter::Str(s, i) => *i = s.s.char_indices().nth(n as usize).map(|(p, _)| p).unwrap_or(s.s.len()),
        Iter::Seq(_, i) => *i = n,
        _ => {}
    }
    Ok(Value::None)
}

/// a built-in function pickles as its name
fn native_reduce(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    match &a[0] {
        Value::Native(n) => Ok(Value::str(n.name.rsplit('.').next().unwrap_or(n.name))),
        Value::Method(m) => {
            let getattr = vm.builtins.borrow().get_s("getattr").unwrap();
            let name = vm.getattr(&a[0], &ps("__name__"))?;
            Ok(Value::tuple(vec![getattr, Value::tuple(vec![m.0.clone(), name])]))
        }
        _ => object_reduce(vm, a, &[]),
    }
}

fn range_reduce(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 1)?;
    let me = vm.base_of(&a[0]);
    match &me {
        Value::Range(r) => {
            let c = Value::Class(vm.t.range.clone());
            Ok(Value::tuple(vec![c, Value::tuple(vec![Value::Int(r.0), Value::Int(r.1), Value::Int(r.2)])]))
        }
        Value::BigRange(r) => {
            let c = Value::Class(vm.t.range.clone());
            let args = vec![Value::from_big(r.start.clone()), Value::from_big(r.stop.clone()), Value::from_big(r.step.clone())];
            Ok(Value::tuple(vec![c, Value::tuple(args)]))
        }
        _ => Err(vm.type_error("descriptor '__reduce__' requires a 'range' object")),
    }
}

// -------------------------------------------------------------------------
// mappingproxy: a read-only view of a mapping (types.MappingProxyType)

pub fn mappingproxy_class(vm: &mut Vm) -> Rc<Class> {
    if let Some(c) = vm.classes.get("mappingproxy") {
        return c.clone();
    }
    let c = Class::new("mappingproxy", vec![vm.t.object.clone()], Builtin::Obj);
    *c.mro.borrow_mut() = vec![c.clone(), vm.t.object.clone()];
    def_static(&c, "mappingproxy.__new__", mp_new);
    def(&c, "mappingproxy.__getitem__", mp_getitem);
    def(&c, "mappingproxy.__contains__", mp_contains);
    def(&c, "mappingproxy.__iter__", mp_iter);
    def(&c, "mappingproxy.__len__", mp_len);
    def(&c, "mappingproxy.__reversed__", mp_reversed);
    def(&c, "mappingproxy.get", mp_get);
    def(&c, "mappingproxy.keys", mp_keys);
    def(&c, "mappingproxy.values", mp_values);
    def(&c, "mappingproxy.items", mp_items);
    def(&c, "mappingproxy.copy", mp_copy);
    def(&c, "mappingproxy.__or__", mp_or);
    def(&c, "mappingproxy.__ror__", mp_ror);
    def(&c, "mappingproxy.__ior__", mp_ior);
    def(&c, "mappingproxy.__eq__", mp_eq);
    def(&c, "mappingproxy.__ne__", mp_ne);
    def(&c, "mappingproxy.__hash__", mp_hash);
    def(&c, "mappingproxy.__repr__", mp_repr);
    def(&c, "mappingproxy.__str__", mp_str);
    def_class(&c, "mappingproxy.__class_getitem__", mp_class_getitem);
    vm.classes.insert("mappingproxy", c.clone());
    c
}

pub fn mappingproxy(vm: &mut Vm, mapping: Value) -> Value {
    let c = mappingproxy_class(vm);
    let o = Rc::new(NativeObj { class: c, gc: Default::default(), kind: RefCell::new(ObjKind::Other(mapping)) });
    crate::gc::track_obj(&o);
    Value::Obj(o)
}

/// the mapping behind a mappingproxy
fn mp_target(v: &Value) -> Option<Value> {
    if let Value::Obj(o) = v {
        if o.class.name() == "mappingproxy" {
            if let ObjKind::Other(m) = &*o.kind.borrow() {
                return Some(m.clone());
            }
        }
    }
    None
}

fn mp_self(vm: &mut Vm, a: &[Value]) -> PyResult<Value> {
    match a.first().and_then(mp_target) {
        Some(m) => Ok(m),
        None => Err(vm.type_error("descriptor requires a 'mappingproxy' object")),
    }
}

fn mp_call(vm: &mut Vm, a: &[Value], name: &str) -> PyResult<Value> {
    let m = mp_self(vm, a)?;
    let f = vm.getattr(&m, &ps(name))?;
    vm.call_vec(&f, a[1..].to_vec(), &[])
}

fn mp_new(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let m = match arg_or_kw(a, 1, k, "mapping") {
        Some(m) if a.len() <= 2 => m,
        _ => return Err(vm.type_error(format!("mappingproxy() takes exactly 1 argument ({} given)", a.len().saturating_sub(1)))),
    };
    let is_mapping = match vm.base_of(&m) {
        Value::List(_) | Value::Tuple(_) => false,
        Value::Dict(_) => true,
        _ => vm.class_of(&m).lookup(&vm.n.getitem).is_some(),
    };
    if !is_mapping {
        let tn = vm.type_name(&m);
        return Err(vm.type_error(format!("mappingproxy() argument must be a mapping, not {}", tn)));
    }
    Ok(mappingproxy(vm, m))
}

fn mp_getitem(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    margs(vm, "__getitem__", a, 1, 1)?;
    let m = mp_self(vm, a)?;
    vm.getitem(&m, &a[1])
}

fn mp_contains(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    margs(vm, "__contains__", a, 1, 1)?;
    let m = mp_self(vm, a)?;
    Ok(Value::Bool(vm.contains(&m, &a[1])?))
}

fn mp_iter(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let m = mp_self(vm, a)?;
    vm.iter(&m)
}

fn mp_len(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let m = mp_self(vm, a)?;
    Ok(Value::Int(vm.len(&m)? as i64))
}

fn mp_reversed(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    let m = mp_self(vm, a)?;
    crate::builtins::bi_reversed(vm, &[m], k)
}

fn mp_get(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    margs(vm, "get", a, 1, 2)?;
    let m = mp_self(vm, a)?;
    let f = vm.getattr(&m, &ps("get"))?;
    let d = a.get(2).cloned().unwrap_or(Value::None);
    vm.call_vec(&f, vec![a[1].clone(), d], &[])
}

fn mp_keys(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    margs(vm, "keys", a, 0, 0)?;
    mp_call(vm, a, "keys")
}

fn mp_values(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    margs(vm, "values", a, 0, 0)?;
    mp_call(vm, a, "values")
}

fn mp_items(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    margs(vm, "items", a, 0, 0)?;
    mp_call(vm, a, "items")
}

fn mp_copy(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    margs(vm, "copy", a, 0, 0)?;
    mp_call(vm, a, "copy")
}

fn mp_or(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    margs(vm, "__or__", a, 1, 1)?;
    let x = mp_target(&a[0]).unwrap_or_else(|| a[0].clone());
    let y = mp_target(&a[1]).unwrap_or_else(|| a[1].clone());
    vm.binop(BinOp::BitOr, &x, &y)
}

fn mp_ror(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    margs(vm, "__ror__", a, 1, 1)?;
    let x = mp_target(&a[0]).unwrap_or_else(|| a[0].clone());
    let y = mp_target(&a[1]).unwrap_or_else(|| a[1].clone());
    vm.binop(BinOp::BitOr, &y, &x)
}

fn mp_ior(vm: &mut Vm, _a: &[Value], _k: Kw) -> PyResult<Value> {
    Err(vm.type_error("'|=' is not supported by mappingproxy; use '|' instead"))
}

fn mp_eq(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    margs(vm, "__eq__", a, 1, 1)?;
    let m = mp_self(vm, a)?;
    vm.compare(CmpOp::Eq, &m, &a[1])
}

fn mp_ne(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    margs(vm, "__ne__", a, 1, 1)?;
    let m = mp_self(vm, a)?;
    vm.compare(CmpOp::Ne, &m, &a[1])
}

fn mp_hash(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let m = mp_self(vm, a)?;
    let h = vm.hash(&m)?;
    Ok(Value::Int(h as i64))
}

fn mp_repr(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let m = mp_self(vm, a)?;
    let r = vm.repr(&m)?;
    Ok(Value::string(format!("mappingproxy({})", r)))
}

fn mp_str(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let m = mp_self(vm, a)?;
    Ok(Value::string(vm.str_of(&m)?))
}

fn mp_class_getitem(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    crate::builtins::need(vm, a, 2)?;
    crate::builtins::generic_alias(vm, &a[0], &a[1])
}
