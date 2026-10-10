// SPDX-License-Identifier: MIT
//! Structural pattern matching (`match` statements).

use crate::ast::{Const, Pattern};
use crate::value::*;
use crate::vm::Vm;
use std::rc::Rc;

/// the values tuple holds the value expressions of the pattern, in the
/// order `collect_pattern` in the compiler visits them
struct M<'a> {
    vals: &'a [Value],
    pos: usize,
    binds: Vec<(Name, Value)>,
}

fn count_vals(p: &Pattern) -> usize {
    match p {
        Pattern::Value(_) => 1,
        Pattern::Sequence(ps) | Pattern::Or(ps) => ps.iter().map(count_vals).sum(),
        Pattern::Mapping { keys, pats, .. } => keys.len() + pats.iter().map(count_vals).sum::<usize>(),
        Pattern::Class { args, kw, .. } => 1 + args.iter().map(count_vals).sum::<usize>() + kw.iter().map(|(_, q)| count_vals(q)).sum::<usize>(),
        Pattern::As(q, _) => count_vals(q),
        _ => 0,
    }
}

fn cap_names(p: &Pattern, out: &mut Vec<Name>) {
    match p {
        Pattern::Capture(Some(n)) | Pattern::Star(Some(n)) => out.push(n.clone()),
        Pattern::Sequence(ps) => ps.iter().for_each(|q| cap_names(q, out)),
        Pattern::Or(ps) => {
            if let Some(q) = ps.first() {
                cap_names(q, out)
            }
        }
        Pattern::Mapping { pats, rest, .. } => {
            pats.iter().for_each(|q| cap_names(q, out));
            if let Some(r) = rest {
                out.push(r.clone());
            }
        }
        Pattern::Class { args, kw, .. } => {
            args.iter().for_each(|q| cap_names(q, out));
            kw.iter().for_each(|(_, q)| cap_names(q, out));
        }
        Pattern::As(q, n) => {
            cap_names(q, out);
            out.push(n.clone());
        }
        _ => {}
    }
}

/// the captures as a tuple in pattern order, or None when it does not match
pub fn match_pattern(vm: &mut Vm, p: &Pattern, subject: &Value, vals: &[Value]) -> PyResult<Value> {
    let mut m = M { vals, pos: 0, binds: vec![] };
    if !do_match(vm, &mut m, p, subject)? {
        return Ok(Value::None);
    }
    let mut names = vec![];
    cap_names(p, &mut names);
    let mut out = Vec::with_capacity(names.len());
    for n in &names {
        let v = m.binds.iter().rev().find(|(k, _)| k == n).map(|(_, v)| v.clone()).unwrap_or(Value::None);
        out.push(v);
    }
    Ok(Value::tuple(out))
}

fn seq_items(vm: &Vm, v: &Value) -> Option<Vec<Value>> {
    match v {
        Value::List(l) => Some(l.borrow().clone()),
        Value::Tuple(t) => Some(t.to_vec()),
        Value::Range(r) => Some(crate::builtins::range_items(r).into_iter().map(Value::Int).collect()),
        Value::BigRange(r) => Some(crate::range::items(r)),
        Value::Obj(o) => match &*o.kind.borrow() {
            ObjKind::Deque(d, _) => Some(d.iter().cloned().collect()),
            _ => None,
        },
        Value::Instance(_) => {
            let b = vm.base_of(v);
            match b {
                Value::List(_) | Value::Tuple(_) => seq_items(vm, &b),
                _ => None,
            }
        }
        _ => None,
    }
}

fn do_match(vm: &mut Vm, m: &mut M, p: &Pattern, s: &Value) -> PyResult<bool> {
    match p {
        Pattern::Capture(None) => Ok(true),
        Pattern::Capture(Some(n)) => {
            m.binds.push((n.clone(), s.clone()));
            Ok(true)
        }
        Pattern::Star(_) => Ok(true),
        Pattern::Value(_) => {
            let v = m.vals[m.pos].clone();
            m.pos += 1;
            vm.eq(s, &v)
        }
        Pattern::Singleton(c) => Ok(match c {
            Const::None => s.is_none(),
            Const::True => matches!(s, Value::Bool(true)),
            Const::False => matches!(s, Value::Bool(false)),
            _ => false,
        }),
        Pattern::As(q, n) => {
            if do_match(vm, m, q, s)? {
                m.binds.push((n.clone(), s.clone()));
                Ok(true)
            } else {
                Ok(false)
            }
        }
        Pattern::Or(ps) => {
            let start = m.pos;
            let mut offset = start;
            let mut found = false;
            for q in ps {
                let n = count_vals(q);
                if !found {
                    let save = m.binds.len();
                    m.pos = offset;
                    if do_match(vm, m, q, s)? {
                        found = true;
                    } else {
                        m.binds.truncate(save);
                    }
                }
                offset += n;
            }
            m.pos = offset;
            Ok(found)
        }
        Pattern::Sequence(ps) => {
            let skip = count_vals(p);
            let end = m.pos + skip;
            let items = match seq_items(vm, s) {
                Some(i) => i,
                None => {
                    m.pos = end;
                    return Ok(false);
                }
            };
            let star = ps.iter().position(|q| matches!(q, Pattern::Star(_)));
            let ok = match star {
                None => {
                    if items.len() != ps.len() {
                        false
                    } else {
                        let mut ok = true;
                        for (q, it) in ps.iter().zip(items.iter()) {
                            if !do_match(vm, m, q, it)? {
                                ok = false;
                                break;
                            }
                        }
                        ok
                    }
                }
                Some(si) => {
                    let after = ps.len() - si - 1;
                    if items.len() < ps.len() - 1 {
                        false
                    } else {
                        let mut ok = true;
                        for i in 0..si {
                            if !do_match(vm, m, &ps[i], &items[i])? {
                                ok = false;
                                break;
                            }
                        }
                        if ok {
                            if let Pattern::Star(Some(n)) = &ps[si] {
                                let mid = items[si..items.len() - after].to_vec();
                                m.binds.push((n.clone(), Value::list(mid)));
                            }
                            for j in 0..after {
                                let it = &items[items.len() - after + j];
                                if !do_match(vm, m, &ps[si + 1 + j], it)? {
                                    ok = false;
                                    break;
                                }
                            }
                        }
                        ok
                    }
                }
            };
            m.pos = end;
            Ok(ok)
        }
        Pattern::Mapping { keys, pats, rest } => {
            let end = m.pos + count_vals(p);
            let is_map = match s {
                Value::Dict(_) => true,
                Value::Instance(_) => matches!(vm.base_of(s), Value::Dict(_)) || vm.lookup_special(s, &Rc::new(PyStr::new("keys".into()))).is_some(),
                _ => false,
            };
            if !is_map {
                m.pos = end;
                return Ok(false);
            }
            let kvals: Vec<Value> = m.vals[m.pos..m.pos + keys.len()].to_vec();
            m.pos += keys.len();
            let mut ok = true;
            for (k, q) in kvals.iter().zip(pats.iter()) {
                let got = match s {
                    Value::Dict(d) => vm.dict_get(d, k)?,
                    _ => match vm.getitem(s, k) {
                        Ok(v) => Some(v),
                        Err(e) if vm.exc_is(&e, "KeyError") => None,
                        Err(e) => return Err(e),
                    },
                };
                match got {
                    Some(v) => {
                        if !do_match(vm, m, q, &v)? {
                            ok = false;
                            break;
                        }
                    }
                    None => {
                        ok = false;
                        break;
                    }
                }
            }
            if ok {
                if let Some(r) = rest {
                    let items = vm.mapping_items(s)?;
                    let d = std::rc::Rc::new(std::cell::RefCell::new(Dict::new()));
                    for (k, v) in items {
                        let mut used = false;
                        for kk in &kvals {
                            if vm.eq(&k, kk)? {
                                used = true;
                            }
                        }
                        if !used {
                            vm.dict_set(&d, k, v)?;
                        }
                    }
                    m.binds.push((r.clone(), Value::Dict(d)));
                }
            }
            m.pos = end;
            Ok(ok)
        }
        Pattern::Class { args, kw, .. } => {
            let end = m.pos + count_vals(p);
            let cls = m.vals[m.pos].clone();
            m.pos += 1;
            let c = match &cls {
                Value::Class(c) => c.clone(),
                _ => {
                    return Err(vm.type_error("called match pattern must be a class"));
                }
            };
            if !vm.isinstance(s, &c) {
                m.pos = end;
                return Ok(false);
            }
            let mut ok = true;
            if !args.is_empty() {
                let self_match = matches!(c.builtin.get(), Builtin::Bool | Builtin::ByteArray | Builtin::Bytes | Builtin::Dict | Builtin::Float | Builtin::FrozenSet | Builtin::Int | Builtin::List | Builtin::Set | Builtin::Str | Builtin::Tuple)
                    && c.lookup(&vm.n.match_args.clone()).is_none();
                if self_match {
                    if args.len() > 1 {
                        let cn = c.name();
                        return Err(vm.type_error(format!("{}() accepts 1 positional sub-pattern ({} given)", cn, args.len())));
                    }
                    ok = do_match(vm, m, &args[0], s)?;
                } else {
                    let ma = match c.lookup(&vm.n.match_args.clone()) {
                        Some(v) => vm.to_vec(&v)?,
                        None => vec![],
                    };
                    if args.len() > ma.len() {
                        let cn = c.name();
                        return Err(vm.type_error(format!("{}() accepts {} positional sub-pattern{} ({} given)", cn, ma.len(), if ma.len() == 1 { "" } else { "s" }, args.len())));
                    }
                    for (q, name) in args.iter().zip(ma.iter()) {
                        let n = match name {
                            Value::Str(s) => s.clone(),
                            _ => return Err(vm.type_error("__match_args__ elements must be strings")),
                        };
                        match vm.getattr_opt(s, &n)? {
                            Some(v) => {
                                if !do_match(vm, m, q, &v)? {
                                    ok = false;
                                    break;
                                }
                            }
                            None => {
                                ok = false;
                                break;
                            }
                        }
                    }
                }
            }
            if ok {
                for (name, q) in kw {
                    let n = Rc::new(PyStr::new(name.to_string()));
                    match vm.getattr_opt(s, &n)? {
                        Some(v) => {
                            if !do_match(vm, m, q, &v)? {
                                ok = false;
                                break;
                            }
                        }
                        None => {
                            ok = false;
                            break;
                        }
                    }
                }
            }
            m.pos = end;
            Ok(ok)
        }
    }
}
