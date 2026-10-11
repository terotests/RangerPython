// SPDX-License-Identifier: MIT
//! _rpyinterp: the core of the subinterpreter modules (PEP 734), which
//! pylib/_interpreters.py and pylib/_interpchannels.py build on. Each
//! interpreter is a Vm of its own, with its own builtins, types, sys,
//! modules and __main__, kept between calls; they take turns on the
//! filesystem and the standard streams. Objects never cross between them:
//! what is shared is the data of the shareable ones (None, bool, int,
//! float, str, bytes and tuples of them), which the receiving interpreter
//! makes its own objects of, and objects a module registered a reduction
//! for (channel ends), which it rebuilds by calling the function named.
//! Channels hold such data, so they belong to no interpreter.

use crate::builtins::*;
use crate::modules::{module, put_fns};
use crate::value::*;
use crate::vm::Vm;
use std::cell::RefCell;
use std::collections::{BTreeMap, VecDeque};
use std::rc::Rc;
use std::sync::Mutex;

/// the data of a shareable object, outside any interpreter
enum Xi {
    None,
    Bool(bool),
    Int(i64),
    Float(f64),
    /// a str, as WTF-8
    Str(Vec<u8>),
    Bytes(Vec<u8>),
    Tuple(Vec<Xi>),
    /// module, function and the arguments that rebuild a registered object
    Revive(String, String, Vec<Xi>),
}

/// a channel's item: the data, or None once the interpreter that sent it
/// is gone, and what to give for it then
struct Item {
    data: Option<Xi>,
    interp: i64,
    unbound: i64,
}

struct Channel {
    items: VecDeque<Item>,
    /// the send end's default for unbound items
    unbound: i64,
    /// closed for sending; closed once the items are received
    closing: bool,
    closed: bool,
}

struct Channels {
    next: i64,
    map: BTreeMap<i64, Channel>,
}

static CHANNELS: Mutex<Channels> = Mutex::new(Channels { next: 0, map: BTreeMap::new() });

fn channels() -> std::sync::MutexGuard<'static, Channels> {
    CHANNELS.lock().unwrap_or_else(|e| e.into_inner())
}

/// an idle interpreter, and its integer string limit
struct Sub {
    vm: Box<Vm>,
    digits: usize,
}

/// the interpreters made on this thread: None while one is running
struct Interps {
    next: i64,
    map: BTreeMap<i64, Option<Sub>>,
    /// the interpreters running, innermost last (the main one, 0, below them)
    running: Vec<i64>,
}

crate::gil_static! {
    static INTERPS: RefCell<Interps> = const { RefCell::new(Interps { next: 1, map: BTreeMap::new(), running: Vec::new() }) };
}

pub fn interp_module() -> Value {
    let (d, m) = module("_rpyinterp");
    put_fns(
        &d,
        &[
            ("_rpyinterp.create", create),
            ("_rpyinterp.destroy", destroy),
            ("_rpyinterp.list_all", list_all),
            ("_rpyinterp.current", current),
            ("_rpyinterp.status", status),
            ("_rpyinterp.set_main_attrs", set_main_attrs),
            ("_rpyinterp.exec", exec),
            ("_rpyinterp.channel_create", channel_create),
            ("_rpyinterp.channel_destroy", channel_destroy),
            ("_rpyinterp.channel_list", channel_list),
            ("_rpyinterp.channel_info", channel_info),
            ("_rpyinterp.channel_send", channel_send),
            ("_rpyinterp.channel_recv", channel_recv),
            ("_rpyinterp.channel_close", channel_close),
        ],
    );
    m
}

fn id_arg(vm: &mut Vm, a: &[Value], i: usize) -> PyResult<i64> {
    match a.get(i) {
        Some(v) => vm.to_index(v),
        None => Err(vm.type_error("missing interpreter or channel id")),
    }
}

fn current_id() -> i64 {
    INTERPS.with(|s| s.borrow().running.last().copied().unwrap_or(0))
}

// -------------------------------------------------------------------------
// sharing

/// the data of `v`; `reduce(v)` gives (module, function, args) for the
/// objects it knows, and raises for the others
fn share(vm: &mut Vm, v: &Value, reduce: &Value) -> PyResult<Xi> {
    Ok(match v {
        Value::None => Xi::None,
        Value::Bool(b) => Xi::Bool(*b),
        Value::Int(i) => Xi::Int(*i),
        // as in CPython, an int must fit a Py_ssize_t
        Value::Big(_) => return Err(vm.err("OverflowError", "try sending as bytes")),
        Value::Float(f) => Xi::Float(*f),
        Value::Str(s) => Xi::Str(s.wtf8().to_vec()),
        Value::Bytes(b) => Xi::Bytes(b.to_vec()),
        Value::Tuple(t) => Xi::Tuple(t.iter().map(|x| share(vm, x, reduce)).collect::<PyResult<_>>()?),
        _ => {
            let r = vm.call(reduce, &[v.clone()])?;
            let parts = vm.to_vec(&r)?;
            let (Some(m), Some(f), Some(args)) = (parts.first(), parts.get(1), parts.get(2)) else {
                return Err(vm.type_error("a cross-interpreter reduction must be (module, name, args)"));
            };
            let (m, f) = (vm.str_of(m)?, vm.str_of(f)?);
            let args = vm.to_vec(args)?.iter().map(|x| share(vm, x, reduce)).collect::<PyResult<_>>()?;
            Xi::Revive(m, f, args)
        }
    })
}

/// a new object of `vm`'s with the data `x`
fn revive(vm: &mut Vm, x: &Xi) -> PyResult<Value> {
    Ok(match x {
        Xi::None => Value::None,
        Xi::Bool(b) => Value::Bool(*b),
        Xi::Int(i) => Value::Int(*i),
        Xi::Float(f) => Value::Float(*f),
        Xi::Str(s) => Value::Str(Rc::new(PyStr::from_wtf8(s.clone()))),
        Xi::Bytes(b) => Value::Bytes(Rc::new(b.clone())),
        Xi::Tuple(t) => Value::tuple(t.iter().map(|x| revive(vm, x)).collect::<PyResult<_>>()?),
        Xi::Revive(m, f, args) => {
            let module = vm.import_one(m)?;
            let func = vm.getattr(&module, &ps(f))?;
            let args = args.iter().map(|x| revive(vm, x)).collect::<PyResult<Vec<_>>>()?;
            vm.call_vec(&func, args, &[])?
        }
    })
}

// -------------------------------------------------------------------------
// interpreters

/// runs `f` in interpreter `id`, which takes the filesystem, the integer
/// string limit and the streams for the time; None when there is no such
/// interpreter, Err(()) when it is running already
fn with_interp<T>(vm: &mut Vm, id: i64, f: impl FnOnce(&mut Vm) -> T) -> Result<Option<T>, ()> {
    use crate::format::MAX_STR_DIGITS;
    use std::sync::atomic::Ordering::Relaxed;
    let taken = INTERPS.with(|s| {
        let mut s = s.borrow_mut();
        match s.map.get_mut(&id) {
            None => Ok(None),
            Some(slot) => match slot.take() {
                None => Err(()),
                Some(sub) => {
                    s.running.push(id);
                    Ok(Some(sub))
                }
            },
        }
    })?;
    let Some(Sub { vm: mut sub, digits }) = taken else { return Ok(None) };
    sub.fs = std::mem::replace(&mut vm.fs, crate::fs::Fs::new());
    let outer = MAX_STR_DIGITS.swap(digits, Relaxed);
    let r = f(&mut sub);
    let digits = MAX_STR_DIGITS.swap(outer, Relaxed);
    vm.fs = std::mem::replace(&mut sub.fs, crate::fs::Fs::new());
    let (out, err) = (std::mem::take(&mut sub.out), std::mem::take(&mut sub.err_out));
    vm.out.push_str(&out);
    vm.err_out.push_str(&err);
    INTERPS.with(|s| {
        let mut s = s.borrow_mut();
        s.running.pop();
        s.map.insert(id, Some(Sub { vm: sub, digits }));
    });
    Ok(Some(r))
}

/// create(): a new interpreter, idle, with an empty __main__; its id
fn create(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "create", a, 0, 0)?;
    let mut sub = Box::new(Vm::new(std::mem::replace(&mut vm.fs, crate::fs::Fs::new())));
    vm.fs = std::mem::replace(&mut sub.fs, crate::fs::Fs::new());
    sub.echo = vm.echo;
    sub.search_path = vm.search_path.clone();
    sub.host = vm.host.clone();
    sub.argv = vec![String::new()];
    sub.main_globals.borrow_mut().remove_str(&PyStr::new("__file__".to_string()));
    let id = INTERPS.with(|s| {
        let mut s = s.borrow_mut();
        let id = s.next;
        s.next += 1;
        // the integer string limit starts at the default
        s.map.insert(id, Some(Sub { vm: sub, digits: 4300 }));
        id
    });
    Ok(Value::Int(id))
}

/// destroy(id): finalizes the interpreter; False when there is none, and
/// an error when it is running. The items it sent become unbound.
fn destroy(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let id = id_arg(vm, a, 0)?;
    match with_interp(vm, id, |sub| sub.finalize()) {
        Ok(None) => return Ok(Value::Bool(false)),
        Err(()) => return Err(vm.err("RuntimeError", "interpreter running")),
        Ok(Some(())) => {}
    }
    let sub = INTERPS.with(|s| s.borrow_mut().map.remove(&id));
    drop(sub);
    for ch in channels().map.values_mut() {
        ch.items.retain_mut(|it| {
            if it.interp != id {
                return true;
            }
            // UNBOUND_REMOVE drops the item; the others keep its place
            it.data = None;
            it.unbound != 1
        });
    }
    Ok(Value::Bool(true))
}

/// list_all(): the ids of the interpreters, the main one (0) first
fn list_all(_vm: &mut Vm, _a: &[Value], _k: Kw) -> PyResult<Value> {
    let ids = INTERPS.with(|s| s.borrow().map.keys().copied().collect::<Vec<_>>());
    Ok(Value::list(std::iter::once(0).chain(ids).map(Value::Int).collect()))
}

/// current(): the id of the running interpreter
fn current(_vm: &mut Vm, _a: &[Value], _k: Kw) -> PyResult<Value> {
    Ok(Value::Int(current_id()))
}

/// status(id): None when there is no such interpreter, else whether it runs
fn status(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let id = id_arg(vm, a, 0)?;
    if id == 0 {
        return Ok(Value::Bool(true));
    }
    Ok(INTERPS.with(|s| match s.borrow().map.get(&id) {
        None => Value::None,
        Some(slot) => Value::Bool(slot.is_none()),
    }))
}

fn missing(vm: &mut Vm, id: i64) -> Exc {
    vm.err("LookupError", format!("unrecognized interpreter ID {}", id))
}

fn running(vm: &mut Vm) -> Exc {
    vm.err("RuntimeError", "interpreter already running")
}

/// set_main_attrs(id, ns, reduce): binds the data of ns's values in the
/// interpreter's __main__
fn set_main_attrs(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    need(vm, a, 3)?;
    let id = id_arg(vm, a, 0)?;
    let Value::Dict(ns) = &a[1] else {
        return Err(vm.type_error("expected a dict"));
    };
    let mut items = vec![];
    for (k, v) in ns.borrow().items() {
        let Value::Str(k) = k else {
            return Err(vm.type_error("__main__ attribute names must be str"));
        };
        items.push((k, v));
    }
    let mut shared = vec![];
    for (k, v) in items {
        shared.push((k.wtf8().to_vec(), share(vm, &v, &a[2])?));
    }
    let r = with_interp(vm, id, |sub| -> Result<(), String> {
        let g = sub.main_globals.clone();
        for (k, x) in &shared {
            let v = revive(sub, x).map_err(|e| exc_text(sub, &e.value))?;
            g.borrow_mut().set_str(Rc::new(PyStr::from_wtf8(k.clone())), v);
        }
        Ok(())
    });
    match r {
        Ok(None) => Err(missing(vm, id)),
        Err(()) => Err(running(vm)),
        Ok(Some(Err(msg))) => Err(vm.err("RuntimeError", msg)),
        Ok(Some(Ok(()))) => Ok(Value::None),
    }
}

/// "Type: message" for an exception
fn exc_text(vm: &mut Vm, e: &Value) -> String {
    let name = vm.class_of(e).name().to_string();
    match vm.str_of(e) {
        Ok(m) if !m.is_empty() => format!("{}: {}", name, m),
        _ => name,
    }
}

/// what the traceback module prints for an exception
fn errdisplay(vm: &mut Vm, e: &Value) -> Option<String> {
    let m = vm.import_one("traceback").ok()?;
    let f = vm.getattr(&m, &ps("format_exception")).ok()?;
    let parts = vm.call_vec(&f, vec![e.clone()], &[]).ok()?;
    let mut text = String::new();
    for p in vm.to_vec(&parts).ok()? {
        text.push_str(p.as_str()?);
    }
    Some(text.trim_end_matches('\n').to_string())
}

/// exec(id, code): runs source or a code object in the interpreter's
/// __main__; None, or for an exception it let out (name, qualname, module,
/// message, "Type: message", its traceback)
fn exec(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    need(vm, a, 2)?;
    let id = id_arg(vm, a, 0)?;
    enum Src {
        Text(String),
        Code(Rc<crate::code::Code>),
    }
    let src = match &a[1] {
        Value::Str(s) => Src::Text(s.s.to_string()),
        Value::Code(c) => Src::Code(c.clone()),
        v => {
            let tn = vm.type_name(v);
            return Err(vm.type_error(format!("expected a str or code object, got {}", tn)));
        }
    };
    let r = with_interp(vm, id, |sub| {
        let g = sub.main_globals.clone();
        let r = match src {
            Src::Text(s) => sub.run_source(&s, "<string>", g),
            Src::Code(c) => sub.run_code_ns(c, g.clone(), g),
        };
        match r {
            Ok(_) => None,
            Err(e) => {
                let cls = sub.class_of(&e.value);
                let module = match cls.dict.borrow().get_s("__module__") {
                    Some(Value::Str(m)) => m.s.to_string(),
                    _ => "builtins".to_string(),
                };
                let msg = sub.str_of(&e.value).unwrap_or_default();
                let formatted = exc_text(sub, &e.value);
                let display = errdisplay(sub, &e.value).unwrap_or_else(|| formatted.clone());
                let qualname = cls.qualname.borrow().s.clone();
                Some([cls.name(), qualname, module, msg, formatted, display])
            }
        }
    });
    match r {
        Ok(None) => Err(missing(vm, id)),
        Err(()) => Err(running(vm)),
        Ok(Some(None)) => Ok(Value::None),
        Ok(Some(Some(info))) => Ok(Value::tuple(info.iter().map(|s| Value::str(s)).collect())),
    }
}

// -------------------------------------------------------------------------
// channels: the status codes are 0 for done, -1 for no such channel, -2
// for a closed one, -3 for one with items left (and 1 for an empty one)

/// channel_create(unbound): a new channel's id
fn channel_create(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let unbound = id_arg(vm, a, 0)?;
    let mut chs = channels();
    let id = chs.next;
    chs.next += 1;
    chs.map.insert(id, Channel { items: VecDeque::new(), unbound, closing: false, closed: false });
    Ok(Value::Int(id))
}

/// channel_destroy(id): whether there was such a channel
fn channel_destroy(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let id = id_arg(vm, a, 0)?;
    Ok(Value::Bool(channels().map.remove(&id).is_some()))
}

/// channel_list(): (id, default unbound) of the open channels
fn channel_list(_vm: &mut Vm, _a: &[Value], _k: Kw) -> PyResult<Value> {
    let chs = channels();
    let rows = chs.map.iter().filter(|(_, c)| !c.closed).map(|(id, c)| Value::tuple(vec![Value::Int(*id), Value::Int(c.unbound)]));
    Ok(Value::list(rows.collect()))
}

/// channel_info(id): (status, closing, closed, count, default unbound)
fn channel_info(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let id = id_arg(vm, a, 0)?;
    let chs = channels();
    Ok(match chs.map.get(&id) {
        None => Value::tuple(vec![Value::Int(-1), Value::Bool(false), Value::Bool(false), Value::Int(0), Value::Int(0)]),
        Some(c) => Value::tuple(vec![
            Value::Int(0),
            Value::Bool(c.closing),
            Value::Bool(c.closed),
            Value::Int(c.items.len() as i64),
            Value::Int(c.unbound),
        ]),
    })
}

/// channel_send(id, obj, unbound, reduce): queues obj's data; the status
fn channel_send(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    need(vm, a, 4)?;
    let id = id_arg(vm, a, 0)?;
    let unbound = id_arg(vm, a, 2)?;
    match channels().map.get(&id) {
        None => return Ok(Value::Int(-1)),
        Some(c) if c.closing || c.closed => return Ok(Value::Int(-2)),
        Some(_) => {}
    }
    let data = share(vm, &a[1], &a[3])?;
    let interp = current_id();
    let mut chs = channels();
    let Some(c) = chs.map.get_mut(&id) else { return Ok(Value::Int(-1)) };
    c.items.push_back(Item { data: Some(data), interp, unbound });
    Ok(Value::Int(0))
}

/// channel_recv(id): (status, obj, unbound op); the op is None unless
/// the item's interpreter is gone
fn channel_recv(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let id = id_arg(vm, a, 0)?;
    let item = {
        let mut chs = channels();
        let Some(c) = chs.map.get_mut(&id) else {
            return Ok(Value::tuple(vec![Value::Int(-1), Value::None, Value::None]));
        };
        if c.closed {
            return Ok(Value::tuple(vec![Value::Int(-2), Value::None, Value::None]));
        }
        let item = c.items.pop_front();
        if c.closing && c.items.is_empty() {
            c.closed = true;
        }
        item
    };
    Ok(match item {
        None => Value::tuple(vec![Value::Int(1), Value::None, Value::None]),
        Some(Item { data: Some(x), .. }) => Value::tuple(vec![Value::Int(0), revive(vm, &x)?, Value::None]),
        Some(Item { data: None, unbound, .. }) => Value::tuple(vec![Value::Int(0), Value::None, Value::Int(unbound)]),
    })
}

/// channel_close(id, force, drain): closes the channel at once when it
/// is empty or `force` drops its items, else with `drain` once they are
/// received; the status
fn channel_close(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    need(vm, a, 3)?;
    let id = id_arg(vm, a, 0)?;
    let force = vm.truthy(&a[1])?;
    let drain = vm.truthy(&a[2])?;
    let mut chs = channels();
    let Some(c) = chs.map.get_mut(&id) else { return Ok(Value::Int(-1)) };
    if c.closed || c.closing {
        return Ok(Value::Int(-2));
    }
    if !c.items.is_empty() && !force {
        if !drain {
            return Ok(Value::Int(-3));
        }
        c.closing = true;
        return Ok(Value::Int(0));
    }
    c.items.clear();
    c.closed = true;
    Ok(Value::Int(0))
}
