// SPDX-License-Identifier: MIT
//! _rpysocket: the host's sockets, on their descriptors, for the socket
//! type of pylib/_socket.py; and name resolution (getaddrinfo and the
//! like). Addresses are Python's: (host, port) for AF_INET, (host, port,
//! flowinfo, scope_id) for AF_INET6, and a Linux abstract name for AF_UNIX
//! (the sandbox has no host paths to bind to). Only a POSIX host has it.
//!
//! An operation with a timeout waits for the socket with poll and raises
//! TimeoutError when the time is up, as CPython's sock_call does; the
//! descriptor is non-blocking then, which _socket.py sees to.

use crate::builtins::*;
use crate::modules::{module, put_fns};
use crate::posix::{os_err, poll_fds};
use crate::value::*;
use crate::vm::Vm;
use std::ffi::{CStr, CString};
use std::net::{Ipv4Addr, Ipv6Addr};
use std::rc::Rc;
use std::time::{Duration, Instant};

fn errno() -> i32 {
    std::io::Error::last_os_error().raw_os_error().unwrap_or(0)
}

fn c_int(vm: &mut Vm, v: &Value) -> PyResult<i32> {
    let n = vm.to_index(v)?;
    i32::try_from(n).map_err(|_| vm.err("OverflowError", "signed integer is greater than maximum"))
}

/// a timeout argument: None, or the seconds left to wait
fn timeout_of(vm: &mut Vm, v: &Value) -> PyResult<Option<f64>> {
    match v {
        Value::None => Ok(None),
        v => Ok(Some(vm.to_f64(v)?)),
    }
}

fn timed_out(vm: &mut Vm) -> Exc {
    vm.err("TimeoutError", "timed out")
}

/// Waits until `fd` can be read (written), at most until `deadline`;
/// false when the time is up.
fn wait_fd(vm: &mut Vm, fd: i32, writing: bool, deadline: Instant) -> PyResult<bool> {
    let left = deadline.saturating_duration_since(Instant::now());
    if left.is_zero() {
        return Ok(false);
    }
    let events = if writing { libc::POLLOUT } else { libc::POLLIN };
    let mut p = [libc::pollfd { fd, events, revents: 0 }];
    Ok(poll_fds(vm, &mut p, Some(left.as_secs_f64() * 1000.0))? > 0)
}

/// One socket operation as CPython's sock_call does it: with a timeout it
/// first waits for the socket and goes on while the operation would
/// block; an interrupted operation runs the signal handlers and is retried.
fn sock_call(vm: &mut Vm, fd: i32, writing: bool, timeout: Option<f64>, mut op: impl FnMut() -> isize) -> PyResult<isize> {
    let deadline = timeout.filter(|t| *t > 0.0).map(|t| Instant::now() + Duration::from_secs_f64(t));
    loop {
        if let Some(d) = deadline {
            if !wait_fd(vm, fd, writing, d)? {
                return Err(timed_out(vm));
            }
        }
        let r = op();
        if r >= 0 {
            return Ok(r);
        }
        let e = errno();
        if e == libc::EINTR {
            crate::posix::check_signals(vm)?;
            continue;
        }
        if deadline.is_some() && (e == libc::EAGAIN || e == libc::EWOULDBLOCK) {
            continue;
        }
        return Err(os_err(vm, e));
    }
}

// -------------------------------------------------------------------------
// addresses

struct SockAddr {
    storage: libc::sockaddr_storage,
    len: libc::socklen_t,
}

impl SockAddr {
    fn new() -> SockAddr {
        // SAFETY: an all-zero sockaddr_storage is a valid (empty) address
        SockAddr {
            storage: unsafe { std::mem::zeroed() },
            len: std::mem::size_of::<libc::sockaddr_storage>() as libc::socklen_t,
        }
    }

    fn ptr(&self) -> *const libc::sockaddr {
        &self.storage as *const _ as *const libc::sockaddr
    }

    fn mut_ptr(&mut self) -> *mut libc::sockaddr {
        &mut self.storage as *mut _ as *mut libc::sockaddr
    }

    fn v4(ip: Ipv4Addr, port: u16) -> SockAddr {
        let mut a = SockAddr::new();
        // SAFETY: sockaddr_storage has room for any sockaddr
        let sin = unsafe { &mut *(a.mut_ptr() as *mut libc::sockaddr_in) };
        sin.sin_family = libc::AF_INET as libc::sa_family_t;
        sin.sin_port = port.to_be();
        sin.sin_addr = libc::in_addr { s_addr: u32::from(ip).to_be() };
        a.len = std::mem::size_of::<libc::sockaddr_in>() as libc::socklen_t;
        a
    }

    fn v6(ip: Ipv6Addr, port: u16, flowinfo: u32, scope_id: u32) -> SockAddr {
        let mut a = SockAddr::new();
        // SAFETY: sockaddr_storage has room for any sockaddr
        let sin6 = unsafe { &mut *(a.mut_ptr() as *mut libc::sockaddr_in6) };
        sin6.sin6_family = libc::AF_INET6 as libc::sa_family_t;
        sin6.sin6_port = port.to_be();
        sin6.sin6_flowinfo = flowinfo.to_be();
        sin6.sin6_addr = libc::in6_addr { s6_addr: ip.octets() };
        sin6.sin6_scope_id = scope_id;
        a.len = std::mem::size_of::<libc::sockaddr_in6>() as libc::socklen_t;
        a
    }
}

/// the Python form of an address the host gave
fn addr_value(a: &SockAddr) -> Value {
    if a.len == 0 {
        return Value::None;
    }
    match a.storage.ss_family as i32 {
        libc::AF_INET => {
            // SAFETY: the family says which sockaddr it is
            let sin = unsafe { &*(a.ptr() as *const libc::sockaddr_in) };
            let ip = Ipv4Addr::from(u32::from_be(sin.sin_addr.s_addr));
            Value::tuple(vec![Value::string(ip.to_string()), Value::Int(u16::from_be(sin.sin_port) as i64)])
        }
        libc::AF_INET6 => {
            // SAFETY: the family says which sockaddr it is
            let sin6 = unsafe { &*(a.ptr() as *const libc::sockaddr_in6) };
            let ip = Ipv6Addr::from(sin6.sin6_addr.s6_addr);
            Value::tuple(vec![
                Value::string(ip.to_string()),
                Value::Int(u16::from_be(sin6.sin6_port) as i64),
                Value::Int(u32::from_be(sin6.sin6_flowinfo) as i64),
                Value::Int(sin6.sin6_scope_id as i64),
            ])
        }
        libc::AF_UNIX => {
            // SAFETY: the family says which sockaddr it is
            let sun = unsafe { &*(a.ptr() as *const libc::sockaddr_un) };
            let off = std::mem::size_of::<libc::sa_family_t>();
            let n = (a.len as usize).saturating_sub(off).min(sun.sun_path.len());
            let path: Vec<u8> = sun.sun_path[..n].iter().map(|c| *c as u8).collect();
            if path.first() == Some(&0) {
                // a Linux abstract name
                Value::Bytes(Rc::new(path))
            } else {
                let end = path.iter().position(|c| *c == 0).unwrap_or(path.len());
                Value::string(String::from_utf8_lossy(&path[..end]).into_owned())
            }
        }
        f => Value::tuple(vec![Value::Int(f as i64), Value::Bytes(Rc::new(vec![]))]),
    }
}

fn gai_error(vm: &mut Vm, code: i32) -> Exc {
    if code == libc::EAI_SYSTEM {
        return os_err(vm, errno());
    }
    // SAFETY: gai_strerror returns a static string
    let msg = unsafe { CStr::from_ptr(libc::gai_strerror(code)) }.to_string_lossy().into_owned();
    let cls = vm.exc_class("socket.gaierror");
    let e = vm.new_exc(&cls, vec![Value::Int(code as i64), Value::string(msg)]);
    Exc { value: e }
}

/// getaddrinfo(3): (family, type, proto, canonname, address) of each result
fn addr_info(vm: &mut Vm, host: Option<&str>, port: Option<&str>, hints: [i32; 4]) -> PyResult<Vec<(i32, i32, i32, String, SockAddr)>> {
    let to_c = |vm: &mut Vm, s: &str| CString::new(s).map_err(|_| vm.value_error("embedded null character"));
    let host = host.map(|h| to_c(vm, h)).transpose()?;
    let port = port.map(|p| to_c(vm, p)).transpose()?;
    // SAFETY: zeroed hints with the fields getaddrinfo reads; the result
    // list is walked once and freed
    unsafe {
        let mut h: libc::addrinfo = std::mem::zeroed();
        h.ai_family = hints[0];
        h.ai_socktype = hints[1];
        h.ai_protocol = hints[2];
        h.ai_flags = hints[3];
        let mut res: *mut libc::addrinfo = std::ptr::null_mut();
        let code = libc::getaddrinfo(
            host.as_ref().map_or(std::ptr::null(), |c| c.as_ptr()),
            port.as_ref().map_or(std::ptr::null(), |c| c.as_ptr()),
            &h,
            &mut res,
        );
        if code != 0 {
            return Err(gai_error(vm, code));
        }
        let mut out = vec![];
        let mut p = res;
        while !p.is_null() {
            let ai = &*p;
            let mut a = SockAddr::new();
            let n = (ai.ai_addrlen as usize).min(std::mem::size_of::<libc::sockaddr_storage>());
            std::ptr::copy_nonoverlapping(ai.ai_addr as *const u8, a.mut_ptr() as *mut u8, n);
            a.len = n as libc::socklen_t;
            let canon = if ai.ai_canonname.is_null() {
                String::new()
            } else {
                CStr::from_ptr(ai.ai_canonname).to_string_lossy().into_owned()
            };
            out.push((ai.ai_family, ai.ai_socktype, ai.ai_protocol, canon, a));
            p = ai.ai_next;
        }
        libc::freeaddrinfo(res);
        Ok(out)
    }
}

/// a host name, given as str, bytes or bytearray
fn host_str(vm: &mut Vm, v: &Value) -> PyResult<String> {
    match v {
        Value::Str(s) => Ok(s.s.to_string()),
        Value::Bytes(_) | Value::ByteArray(_) => Ok(String::from_utf8_lossy(&bytes_of(vm, v)?).into_owned()),
        _ => {
            let tn = vm.type_name(v);
            Err(vm.type_error(format!("str, bytes or bytearray expected, not {}", tn)))
        }
    }
}

/// a host name or numeric address as an IP address of `family`, as
/// CPython's setipaddr: '' is any address, '<broadcast>' the broadcast one
fn ip_of(vm: &mut Vm, host: &str, family: i32) -> PyResult<std::net::IpAddr> {
    use std::net::IpAddr;
    if host.is_empty() {
        return Ok(if family == libc::AF_INET6 {
            IpAddr::V6(Ipv6Addr::UNSPECIFIED)
        } else {
            IpAddr::V4(Ipv4Addr::UNSPECIFIED)
        });
    }
    if family == libc::AF_INET && (host == "<broadcast>" || host == "255.255.255.255") {
        return Ok(IpAddr::V4(Ipv4Addr::BROADCAST));
    }
    if let Ok(ip) = host.parse::<IpAddr>() {
        if (family == libc::AF_INET) == ip.is_ipv4() {
            return Ok(ip);
        }
    }
    let found = addr_info(vm, Some(host), None, [family, 0, 0, 0])?;
    for (_, _, _, _, a) in found {
        match addr_value(&a) {
            Value::Tuple(t) => {
                if let Value::Str(s) = &t[0] {
                    if let Ok(ip) = s.s.parse::<IpAddr>() {
                        return Ok(ip);
                    }
                }
            }
            _ => continue,
        }
    }
    Err(gai_error(vm, libc::EAI_NONAME))
}

fn port_of(vm: &mut Vm, v: &Value, what: &str) -> PyResult<u16> {
    let n = vm.to_index(v)?;
    u16::try_from(n).map_err(|_| vm.err("OverflowError", format!("{}(): port must be 0-65535.", what)))
}

/// the host form of a Python address of `family`
fn sockaddr_of(vm: &mut Vm, family: i32, addr: &Value, what: &str) -> PyResult<SockAddr> {
    let tuple = |vm: &mut Vm, name: &str| -> PyResult<Vec<Value>> {
        match addr {
            Value::Tuple(t) => Ok(t.to_vec()),
            v => {
                let tn = vm.type_name(v);
                Err(vm.type_error(format!("{} address must be tuple, not {}", name, tn)))
            }
        }
    };
    match family {
        libc::AF_INET => {
            let t = tuple(vm, "AF_INET")?;
            if t.len() != 2 {
                return Err(vm.type_error("AF_INET address must be a pair (host, port)"));
            }
            let host = host_str(vm, &t[0])?;
            let port = port_of(vm, &t[1], what)?;
            match ip_of(vm, &host, libc::AF_INET)? {
                std::net::IpAddr::V4(ip) => Ok(SockAddr::v4(ip, port)),
                std::net::IpAddr::V6(_) => Err(os_err(vm, libc::EAFNOSUPPORT)),
            }
        }
        libc::AF_INET6 => {
            let t = tuple(vm, "AF_INET6")?;
            if !(2..=4).contains(&t.len()) {
                return Err(vm.type_error("AF_INET6 address must be a tuple (host, port[, flowinfo[, scopeid]])"));
            }
            let host = host_str(vm, &t[0])?;
            let port = port_of(vm, &t[1], what)?;
            let flowinfo = match t.get(2) {
                Some(v) => u32::try_from(vm.to_index(v)?).map_err(|_| vm.err("OverflowError", format!("{}(): flowinfo must be 0-1048575.", what)))?,
                None => 0,
            };
            let scope = match t.get(3) {
                Some(v) => vm.to_index(v)? as u32,
                None => 0,
            };
            match ip_of(vm, &host, libc::AF_INET6)? {
                std::net::IpAddr::V6(ip) => Ok(SockAddr::v6(ip, port, flowinfo, scope)),
                std::net::IpAddr::V4(_) => Err(os_err(vm, libc::EAFNOSUPPORT)),
            }
        }
        libc::AF_UNIX => {
            let path = match addr {
                Value::Str(s) => s.s.as_bytes().to_vec(),
                v => bytes_of(vm, v)?,
            };
            if path.first() != Some(&0) {
                // a path would be a file of the host's, outside the sandbox
                let cls = vm.exc_class("PermissionError");
                let e = vm.new_exc(
                    &cls,
                    vec![Value::Int(libc::EACCES as i64), Value::str("AF_UNIX paths are outside the sandbox; use an abstract name"), addr.clone()],
                );
                return Err(Exc { value: e });
            }
            let mut a = SockAddr::new();
            // SAFETY: sockaddr_storage has room for any sockaddr
            let sun = unsafe { &mut *(a.mut_ptr() as *mut libc::sockaddr_un) };
            if path.len() > sun.sun_path.len() {
                return Err(os_err(vm, libc::ENAMETOOLONG));
            }
            sun.sun_family = libc::AF_UNIX as libc::sa_family_t;
            for (i, b) in path.iter().enumerate() {
                sun.sun_path[i] = *b as libc::c_char;
            }
            a.len = (std::mem::size_of::<libc::sa_family_t>() + path.len()) as libc::socklen_t;
            Ok(a)
        }
        _ => Err(os_err(vm, libc::EAFNOSUPPORT)),
    }
}

// -------------------------------------------------------------------------
// socket operations: each takes the descriptor first

fn fd_arg(vm: &mut Vm, a: &[Value], n: usize, name: &str) -> PyResult<i32> {
    check_nargs(vm, name, a, n, n)?;
    c_int(vm, &a[0])
}

fn s_socket(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "socket", a, 3, 3)?;
    let (family, ty, proto) = (c_int(vm, &a[0])?, c_int(vm, &a[1])?, c_int(vm, &a[2])?);
    // SAFETY: creates a descriptor
    #[cfg(any(target_os = "linux", target_os = "android"))]
    let fd = unsafe { libc::socket(family, ty | libc::SOCK_CLOEXEC, proto) };
    #[cfg(not(any(target_os = "linux", target_os = "android")))]
    let fd = unsafe { libc::socket(family, ty, proto) };
    if fd < 0 {
        return Err(os_err(vm, errno()));
    }
    Ok(Value::Int(fd as i64))
}

fn s_socketpair(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "socketpair", a, 3, 3)?;
    let (family, ty, proto) = (c_int(vm, &a[0])?, c_int(vm, &a[1])?, c_int(vm, &a[2])?);
    let mut fds = [0i32; 2];
    // SAFETY: fds has room for two descriptors
    #[cfg(any(target_os = "linux", target_os = "android"))]
    let r = unsafe { libc::socketpair(family, ty | libc::SOCK_CLOEXEC, proto, fds.as_mut_ptr()) };
    #[cfg(not(any(target_os = "linux", target_os = "android")))]
    let r = unsafe { libc::socketpair(family, ty, proto, fds.as_mut_ptr()) };
    if r != 0 {
        return Err(os_err(vm, errno()));
    }
    Ok(Value::tuple(vec![Value::Int(fds[0] as i64), Value::Int(fds[1] as i64)]))
}

fn int_opt(fd: i32, level: i32, opt: i32) -> Result<i32, i32> {
    let mut v: libc::c_int = 0;
    let mut n = std::mem::size_of::<libc::c_int>() as libc::socklen_t;
    // SAFETY: v and n are written by getsockopt
    if unsafe { libc::getsockopt(fd, level, opt, &mut v as *mut _ as *mut libc::c_void, &mut n) } != 0 {
        return Err(errno());
    }
    Ok(v)
}

/// (family, type, proto) of an open socket
fn s_info(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let fd = fd_arg(vm, a, 1, "_info")?;
    let mut sa = SockAddr::new();
    // SAFETY: sa and its length are written by getsockname
    if unsafe { libc::getsockname(fd, sa.mut_ptr(), &mut sa.len) } != 0 {
        return Err(os_err(vm, errno()));
    }
    let ty = int_opt(fd, libc::SOL_SOCKET, libc::SO_TYPE).map_err(|e| os_err(vm, e))?;
    #[cfg(any(target_os = "linux", target_os = "android"))]
    let proto = int_opt(fd, libc::SOL_SOCKET, libc::SO_PROTOCOL).unwrap_or(0);
    #[cfg(not(any(target_os = "linux", target_os = "android")))]
    let proto = 0;
    Ok(Value::tuple(vec![Value::Int(sa.storage.ss_family as i64), Value::Int(ty as i64), Value::Int(proto as i64)]))
}

fn s_bind(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let fd = fd_arg(vm, a, 3, "bind")?;
    let family = c_int(vm, &a[1])?;
    let sa = sockaddr_of(vm, family, &a[2], "bind")?;
    // SAFETY: sa is a complete address of its length
    if unsafe { libc::bind(fd, sa.ptr(), sa.len) } != 0 {
        return Err(os_err(vm, errno()));
    }
    Ok(Value::None)
}

fn s_listen(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let fd = fd_arg(vm, a, 2, "listen")?;
    let backlog = c_int(vm, &a[1])?.max(0);
    // SAFETY: a socket operation
    if unsafe { libc::listen(fd, backlog) } != 0 {
        return Err(os_err(vm, errno()));
    }
    Ok(Value::None)
}

/// connect(fd, family, address, timeout) -> errno (0 when connected); a
/// timeout raises TimeoutError
fn s_connect(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let fd = fd_arg(vm, a, 4, "connect")?;
    let family = c_int(vm, &a[1])?;
    let sa = sockaddr_of(vm, family, &a[2], "connect")?;
    let timeout = timeout_of(vm, &a[3])?;
    loop {
        // SAFETY: sa is a complete address of its length
        if unsafe { libc::connect(fd, sa.ptr(), sa.len) } == 0 {
            return Ok(Value::Int(0));
        }
        let e = errno();
        if e == libc::EINTR {
            crate::posix::check_signals(vm)?;
            // the connection goes on; its end is waited for as below
            if timeout.is_none() {
                continue;
            }
        } else if !(e == libc::EINPROGRESS && timeout.is_some_and(|t| t > 0.0)) {
            return Ok(Value::Int(e as i64));
        }
        let deadline = Instant::now() + Duration::from_secs_f64(timeout.unwrap_or(0.0));
        if !wait_fd(vm, fd, true, deadline)? {
            return Err(timed_out(vm));
        }
        return Ok(Value::Int(int_opt(fd, libc::SOL_SOCKET, libc::SO_ERROR).unwrap_or_else(|e| e) as i64));
    }
}

fn s_accept(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let fd = fd_arg(vm, a, 2, "_accept")?;
    let timeout = timeout_of(vm, &a[1])?;
    let mut sa = SockAddr::new();
    let new = sock_call(vm, fd, false, timeout, || {
        sa.len = std::mem::size_of::<libc::sockaddr_storage>() as libc::socklen_t;
        // SAFETY: sa and its length are written by accept
        #[cfg(any(target_os = "linux", target_os = "android"))]
        let r = unsafe { libc::accept4(fd, sa.mut_ptr(), &mut sa.len, libc::SOCK_CLOEXEC) };
        #[cfg(not(any(target_os = "linux", target_os = "android")))]
        let r = unsafe { libc::accept(fd, sa.mut_ptr(), &mut sa.len) };
        r as isize
    })?;
    Ok(Value::tuple(vec![Value::Int(new as i64), addr_value(&sa)]))
}

fn s_name(vm: &mut Vm, a: &[Value], peer: bool) -> PyResult<Value> {
    let fd = fd_arg(vm, a, 1, if peer { "getpeername" } else { "getsockname" })?;
    let mut sa = SockAddr::new();
    // SAFETY: sa and its length are written by the call
    let r = unsafe {
        if peer {
            libc::getpeername(fd, sa.mut_ptr(), &mut sa.len)
        } else {
            libc::getsockname(fd, sa.mut_ptr(), &mut sa.len)
        }
    };
    if r != 0 {
        return Err(os_err(vm, errno()));
    }
    Ok(addr_value(&sa))
}

fn s_getsockname(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    s_name(vm, a, false)
}

fn s_getpeername(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    s_name(vm, a, true)
}

fn data_arg(vm: &mut Vm, v: &Value) -> PyResult<Vec<u8>> {
    if matches!(v, Value::Str(_)) {
        return Err(vm.type_error("a bytes-like object is required, not 'str'"));
    }
    bytes_of(vm, v)
}

/// send(fd, data, flags, timeout) -> bytes sent
fn s_send(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let fd = fd_arg(vm, a, 4, "send")?;
    let data = data_arg(vm, &a[1])?;
    let flags = c_int(vm, &a[2])?;
    let timeout = timeout_of(vm, &a[3])?;
    // SAFETY: data is valid for its length
    let n = sock_call(vm, fd, true, timeout, || unsafe { libc::send(fd, data.as_ptr() as *const libc::c_void, data.len(), flags) })?;
    Ok(Value::Int(n as i64))
}

/// sendall(fd, data, flags, timeout): all of it, the timeout for the whole
fn s_sendall(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let fd = fd_arg(vm, a, 4, "sendall")?;
    let data = data_arg(vm, &a[1])?;
    let flags = c_int(vm, &a[2])?;
    let timeout = timeout_of(vm, &a[3])?;
    let deadline = timeout.filter(|t| *t > 0.0).map(|t| Instant::now() + Duration::from_secs_f64(t));
    let mut rest = &data[..];
    while !rest.is_empty() {
        let left = deadline.map(|d| d.saturating_duration_since(Instant::now()).as_secs_f64());
        if left == Some(0.0) {
            return Err(timed_out(vm));
        }
        let left = left.or(timeout);
        // SAFETY: rest is valid for its length
        let n = sock_call(vm, fd, true, left, || unsafe { libc::send(fd, rest.as_ptr() as *const libc::c_void, rest.len(), flags) })?;
        rest = &rest[n as usize..];
    }
    Ok(Value::None)
}

/// sendto(fd, data, flags, family, address, timeout) -> bytes sent
fn s_sendto(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let fd = fd_arg(vm, a, 6, "sendto")?;
    let data = data_arg(vm, &a[1])?;
    let flags = c_int(vm, &a[2])?;
    let family = c_int(vm, &a[3])?;
    let sa = sockaddr_of(vm, family, &a[4], "sendto")?;
    let timeout = timeout_of(vm, &a[5])?;
    // SAFETY: data and sa are valid for their lengths
    let n = sock_call(vm, fd, true, timeout, || unsafe {
        libc::sendto(fd, data.as_ptr() as *const libc::c_void, data.len(), flags, sa.ptr(), sa.len)
    })?;
    Ok(Value::Int(n as i64))
}

fn size_arg(vm: &mut Vm, v: &Value, what: &str) -> PyResult<usize> {
    let n = vm.to_index(v)?;
    if n < 0 {
        return Err(vm.value_error(format!("negative buffersize in {}", what)));
    }
    Ok(n as usize)
}

/// recv(fd, bufsize, flags, timeout) -> bytes
fn s_recv(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let fd = fd_arg(vm, a, 4, "recv")?;
    let n = size_arg(vm, &a[1], "recv")?;
    let flags = c_int(vm, &a[2])?;
    let timeout = timeout_of(vm, &a[3])?;
    let mut buf = vec![0u8; n];
    // SAFETY: buf has room for n bytes
    let got = sock_call(vm, fd, false, timeout, || unsafe { libc::recv(fd, buf.as_mut_ptr() as *mut libc::c_void, n, flags) })?;
    buf.truncate(got as usize);
    Ok(Value::Bytes(Rc::new(buf)))
}

/// recvfrom(fd, bufsize, flags, timeout) -> (bytes, address)
fn s_recvfrom(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let fd = fd_arg(vm, a, 4, "recvfrom")?;
    let n = size_arg(vm, &a[1], "recvfrom")?;
    let flags = c_int(vm, &a[2])?;
    let timeout = timeout_of(vm, &a[3])?;
    let mut buf = vec![0u8; n];
    let mut sa = SockAddr::new();
    let got = sock_call(vm, fd, false, timeout, || {
        sa.len = std::mem::size_of::<libc::sockaddr_storage>() as libc::socklen_t;
        // SAFETY: buf has room for n bytes; sa and its length are written by recvfrom
        unsafe { libc::recvfrom(fd, buf.as_mut_ptr() as *mut libc::c_void, n, flags, sa.mut_ptr(), &mut sa.len) }
    })?;
    buf.truncate(got as usize);
    Ok(Value::tuple(vec![Value::Bytes(Rc::new(buf)), addr_value(&sa)]))
}

fn s_shutdown(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let fd = fd_arg(vm, a, 2, "shutdown")?;
    let how = c_int(vm, &a[1])?;
    // SAFETY: a socket operation
    if unsafe { libc::shutdown(fd, how) } != 0 {
        return Err(os_err(vm, errno()));
    }
    Ok(Value::None)
}

/// setsockopt(fd, level, option, value): an int, or bytes as they are
fn s_setsockopt(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let fd = fd_arg(vm, a, 4, "setsockopt")?;
    let (level, opt) = (c_int(vm, &a[1])?, c_int(vm, &a[2])?);
    let data = match &a[3] {
        Value::Int(_) | Value::Bool(_) => c_int(vm, &a[3])?.to_ne_bytes().to_vec(),
        v => bytes_of(vm, v)?,
    };
    // SAFETY: data is valid for its length
    if unsafe { libc::setsockopt(fd, level, opt, data.as_ptr() as *const libc::c_void, data.len() as libc::socklen_t) } != 0 {
        return Err(os_err(vm, errno()));
    }
    Ok(Value::None)
}

/// getsockopt(fd, level, option, buflen): an int, or bytes when buflen is given
fn s_getsockopt(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let fd = fd_arg(vm, a, 4, "getsockopt")?;
    let (level, opt) = (c_int(vm, &a[1])?, c_int(vm, &a[2])?);
    if a[3].is_none() {
        let v = int_opt(fd, level, opt).map_err(|e| os_err(vm, e))?;
        return Ok(Value::Int(v as i64));
    }
    let n = vm.to_index(&a[3])?;
    if !(0..=1024).contains(&n) {
        return Err(os_err(vm, libc::EINVAL));
    }
    let mut buf = vec![0u8; n as usize];
    let mut len = n as libc::socklen_t;
    // SAFETY: buf has room for len bytes
    if unsafe { libc::getsockopt(fd, level, opt, buf.as_mut_ptr() as *mut libc::c_void, &mut len) } != 0 {
        return Err(os_err(vm, errno()));
    }
    buf.truncate(len as usize);
    Ok(Value::Bytes(Rc::new(buf)))
}

/// waits until the socket can be read (written): wait(fd, writing, timeout) -> ready
fn s_wait(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    let fd = fd_arg(vm, a, 3, "_wait")?;
    let writing = vm.truthy(&a[1])?;
    let t = vm.to_f64(&a[2])?;
    let deadline = Instant::now() + Duration::from_secs_f64(t.max(0.0));
    Ok(Value::Bool(wait_fd(vm, fd, writing, deadline)?))
}

// -------------------------------------------------------------------------
// names and addresses

fn s_getaddrinfo(vm: &mut Vm, a: &[Value], k: Kw) -> PyResult<Value> {
    check_nargs(vm, "getaddrinfo", a, 2, 6)?;
    let host = match &a[0] {
        Value::None => None,
        v => Some(host_str(vm, v)?),
    };
    let port = match &a[1] {
        Value::None => None,
        Value::Int(n) => Some(n.to_string()),
        Value::Str(s) => Some(s.s.to_string()),
        Value::Bytes(b) => Some(String::from_utf8_lossy(b).into_owned()),
        _ => return Err(vm.err("OSError", "Int or String expected")),
    };
    let mut hints = [0i32; 4];
    for (i, name) in ["family", "type", "proto", "flags"].iter().enumerate() {
        if let Some(v) = arg_or_kw(a, i + 2, k, name) {
            hints[i] = c_int(vm, &v)?;
        }
    }
    let found = addr_info(vm, host.as_deref(), port.as_deref(), hints)?;
    let out = found
        .into_iter()
        .map(|(f, t, p, c, sa)| Value::tuple(vec![Value::Int(f as i64), Value::Int(t as i64), Value::Int(p as i64), Value::string(c), addr_value(&sa)]))
        .collect();
    Ok(Value::list(out))
}

fn s_getnameinfo(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "getnameinfo", a, 2, 2)?;
    let flags = c_int(vm, &a[1])?;
    let Value::Tuple(t) = &a[0] else {
        return Err(vm.type_error("getnameinfo() argument 1 must be a tuple"));
    };
    if !(2..=4).contains(&t.len()) {
        return Err(vm.type_error("getnameinfo(): illegal sockaddr argument"));
    }
    let host = host_str(vm, &t[0])?;
    let family = if t.len() > 2 || host.contains(':') { libc::AF_INET6 } else { libc::AF_INET };
    let sa = sockaddr_of(vm, family, &a[0], "getnameinfo")?;
    let mut h = [0 as libc::c_char; 1025];
    let mut s = [0 as libc::c_char; 32];
    // SAFETY: sa is a complete address; the buffers have their lengths
    let code = unsafe { libc::getnameinfo(sa.ptr(), sa.len, h.as_mut_ptr(), h.len() as _, s.as_mut_ptr(), s.len() as _, flags) };
    if code != 0 {
        return Err(gai_error(vm, code));
    }
    // SAFETY: getnameinfo wrote NUL-terminated strings
    let (h, s) = unsafe { (CStr::from_ptr(h.as_ptr()), CStr::from_ptr(s.as_ptr())) };
    Ok(Value::tuple(vec![Value::string(h.to_string_lossy().into_owned()), Value::string(s.to_string_lossy().into_owned())]))
}

fn s_gethostname(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "gethostname", a, 0, 0)?;
    let mut buf = [0 as libc::c_char; 256];
    // SAFETY: buf has its length
    if unsafe { libc::gethostname(buf.as_mut_ptr(), buf.len()) } != 0 {
        return Err(os_err(vm, errno()));
    }
    // SAFETY: gethostname wrote a NUL-terminated name
    Ok(Value::string(unsafe { CStr::from_ptr(buf.as_ptr()) }.to_string_lossy().into_owned()))
}

fn s_gethostbyname(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "gethostbyname", a, 1, 1)?;
    let host = host_str(vm, &a[0])?;
    Ok(Value::string(ip_of(vm, &host, libc::AF_INET)?.to_string()))
}

fn s_inet_pton(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "inet_pton", a, 2, 2)?;
    let family = c_int(vm, &a[0])?;
    let s = as_pystr(vm, &a[1], "inet_pton")?;
    let bytes = match family {
        libc::AF_INET => s.s.parse::<Ipv4Addr>().ok().map(|ip| ip.octets().to_vec()),
        libc::AF_INET6 => s.s.parse::<Ipv6Addr>().ok().map(|ip| ip.octets().to_vec()),
        _ => return Err(os_err(vm, libc::EAFNOSUPPORT)),
    };
    match bytes {
        Some(b) => Ok(Value::Bytes(Rc::new(b))),
        None => Err(vm.err("OSError", "illegal IP address string passed to inet_pton")),
    }
}

fn s_inet_ntop(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "inet_ntop", a, 2, 2)?;
    let family = c_int(vm, &a[0])?;
    let b = bytes_of(vm, &a[1])?;
    match family {
        libc::AF_INET => match <[u8; 4]>::try_from(b.as_slice()) {
            Ok(o) => Ok(Value::string(Ipv4Addr::from(o).to_string())),
            Err(_) => Err(vm.value_error("invalid length of packed IP address string")),
        },
        libc::AF_INET6 => match <[u8; 16]>::try_from(b.as_slice()) {
            Ok(o) => Ok(Value::string(Ipv6Addr::from(o).to_string())),
            Err(_) => Err(vm.value_error("invalid length of packed IP address string")),
        },
        _ => Err(vm.value_error(format!("unknown address family {}", family))),
    }
}

fn s_inet_aton(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "inet_aton", a, 1, 1)?;
    let s = as_pystr(vm, &a[0], "inet_aton")?;
    let c = CString::new(&*s.s).map_err(|_| vm.value_error("embedded null character"))?;
    let mut addr = libc::in_addr { s_addr: 0 };
    // SAFETY: inet_aton(3) reads a C string and writes addr
    extern "C" {
        fn inet_aton(cp: *const libc::c_char, inp: *mut libc::in_addr) -> libc::c_int;
    }
    if unsafe { inet_aton(c.as_ptr(), &mut addr) } == 0 {
        return Err(vm.err("OSError", "illegal IP address string passed to inet_aton"));
    }
    Ok(Value::Bytes(Rc::new(addr.s_addr.to_ne_bytes().to_vec())))
}

fn s_inet_ntoa(vm: &mut Vm, a: &[Value], _k: Kw) -> PyResult<Value> {
    check_nargs(vm, "inet_ntoa", a, 1, 1)?;
    let b = bytes_of(vm, &a[0])?;
    match <[u8; 4]>::try_from(b.as_slice()) {
        Ok(o) => Ok(Value::string(Ipv4Addr::from(o).to_string())),
        Err(_) => Err(vm.err("OSError", "packed IP wrong length for inet_ntoa")),
    }
}

/// an exception class of the socket module, under its CPython name
fn socket_error(vm: &mut Vm, name: &'static str, key: &'static str) -> Value {
    let base = vm.exc_class("OSError");
    let c = Class::new(name, vec![base.clone()], Builtin::BaseException);
    let mut mro = vec![c.clone()];
    mro.extend(base.mro.borrow().iter().cloned());
    c.set_mro(mro);
    c.dict.borrow_mut().set_s("__module__", Value::str("socket"));
    vm.exc.insert(key, c.clone());
    Value::Class(c)
}

pub fn socket_module(vm: &mut Vm) -> Value {
    let (d, m) = module("_rpysocket");
    put_fns(
        &d,
        &[
            ("_rpysocket._socket", s_socket),
            ("_rpysocket._info", s_info),
            ("_rpysocket._bind", s_bind),
            ("_rpysocket._listen", s_listen),
            ("_rpysocket._connect", s_connect),
            ("_rpysocket._accept", s_accept),
            ("_rpysocket._getsockname", s_getsockname),
            ("_rpysocket._getpeername", s_getpeername),
            ("_rpysocket._send", s_send),
            ("_rpysocket._sendall", s_sendall),
            ("_rpysocket._sendto", s_sendto),
            ("_rpysocket._recv", s_recv),
            ("_rpysocket._recvfrom", s_recvfrom),
            ("_rpysocket._shutdown", s_shutdown),
            ("_rpysocket._setsockopt", s_setsockopt),
            ("_rpysocket._getsockopt", s_getsockopt),
            ("_rpysocket._wait", s_wait),
            ("_rpysocket._socketpair", s_socketpair),
            ("_rpysocket.getaddrinfo", s_getaddrinfo),
            ("_rpysocket.getnameinfo", s_getnameinfo),
            ("_rpysocket.gethostname", s_gethostname),
            ("_rpysocket.gethostbyname", s_gethostbyname),
            ("_rpysocket.inet_pton", s_inet_pton),
            ("_rpysocket.inet_ntop", s_inet_ntop),
            ("_rpysocket.inet_aton", s_inet_aton),
            ("_rpysocket.inet_ntoa", s_inet_ntoa),
        ],
    );
    let gai = socket_error(vm, "gaierror", "socket.gaierror");
    let herror = socket_error(vm, "herror", "socket.herror");
    let capi = crate::modules::capsule(vm, "_socket.CAPI");
    let mut b = d.borrow_mut();
    b.set_s("gaierror", gai);
    b.set_s("herror", herror);
    b.set_s("CAPI", capi);
    b.set_s("has_ipv6", Value::Bool(true));
    let mut ints: Vec<(&str, i32)> = vec![
        ("AF_UNSPEC", libc::AF_UNSPEC),
        ("AF_INET", libc::AF_INET),
        ("AF_INET6", libc::AF_INET6),
        ("AF_UNIX", libc::AF_UNIX),
        ("SOCK_STREAM", libc::SOCK_STREAM),
        ("SOCK_DGRAM", libc::SOCK_DGRAM),
        ("SOCK_RAW", libc::SOCK_RAW),
        ("SOCK_SEQPACKET", libc::SOCK_SEQPACKET),
        ("SOCK_RDM", libc::SOCK_RDM),
        ("SOL_SOCKET", libc::SOL_SOCKET),
        ("SO_REUSEADDR", libc::SO_REUSEADDR),
        ("SO_KEEPALIVE", libc::SO_KEEPALIVE),
        ("SO_BROADCAST", libc::SO_BROADCAST),
        ("SO_LINGER", libc::SO_LINGER),
        ("SO_SNDBUF", libc::SO_SNDBUF),
        ("SO_RCVBUF", libc::SO_RCVBUF),
        ("SO_RCVLOWAT", libc::SO_RCVLOWAT),
        ("SO_SNDLOWAT", libc::SO_SNDLOWAT),
        ("SO_RCVTIMEO", libc::SO_RCVTIMEO),
        ("SO_SNDTIMEO", libc::SO_SNDTIMEO),
        ("SO_ERROR", libc::SO_ERROR),
        ("SO_TYPE", libc::SO_TYPE),
        ("SO_OOBINLINE", libc::SO_OOBINLINE),
        ("SO_DONTROUTE", libc::SO_DONTROUTE),
        ("SO_ACCEPTCONN", libc::SO_ACCEPTCONN),
        ("SO_REUSEPORT", libc::SO_REUSEPORT),
        ("SOMAXCONN", libc::SOMAXCONN),
        ("IPPROTO_IP", libc::IPPROTO_IP),
        ("IPPROTO_TCP", libc::IPPROTO_TCP),
        ("IPPROTO_UDP", libc::IPPROTO_UDP),
        ("IPPROTO_IPV6", libc::IPPROTO_IPV6),
        ("IPPROTO_ICMP", libc::IPPROTO_ICMP),
        ("IPPROTO_ICMPV6", libc::IPPROTO_ICMPV6),
        ("IPPROTO_RAW", libc::IPPROTO_RAW),
        ("IPV6_V6ONLY", libc::IPV6_V6ONLY),
        ("IP_TTL", libc::IP_TTL),
        ("IP_TOS", libc::IP_TOS),
        ("IP_MULTICAST_TTL", libc::IP_MULTICAST_TTL),
        ("IP_MULTICAST_LOOP", libc::IP_MULTICAST_LOOP),
        ("IP_ADD_MEMBERSHIP", libc::IP_ADD_MEMBERSHIP),
        ("IP_DROP_MEMBERSHIP", libc::IP_DROP_MEMBERSHIP),
        ("TCP_NODELAY", libc::TCP_NODELAY),
        ("TCP_KEEPINTVL", libc::TCP_KEEPINTVL),
        ("TCP_KEEPCNT", libc::TCP_KEEPCNT),
        ("MSG_OOB", libc::MSG_OOB),
        ("MSG_PEEK", libc::MSG_PEEK),
        ("MSG_DONTROUTE", libc::MSG_DONTROUTE),
        ("MSG_DONTWAIT", libc::MSG_DONTWAIT),
        ("MSG_WAITALL", libc::MSG_WAITALL),
        ("MSG_TRUNC", libc::MSG_TRUNC),
        ("MSG_CTRUNC", libc::MSG_CTRUNC),
        ("MSG_EOR", libc::MSG_EOR),
        ("SHUT_RD", libc::SHUT_RD),
        ("SHUT_WR", libc::SHUT_WR),
        ("SHUT_RDWR", libc::SHUT_RDWR),
        ("AI_PASSIVE", libc::AI_PASSIVE),
        ("AI_CANONNAME", libc::AI_CANONNAME),
        ("AI_NUMERICHOST", libc::AI_NUMERICHOST),
        ("AI_NUMERICSERV", libc::AI_NUMERICSERV),
        ("AI_V4MAPPED", libc::AI_V4MAPPED),
        ("AI_ALL", libc::AI_ALL),
        ("AI_ADDRCONFIG", libc::AI_ADDRCONFIG),
        ("NI_NUMERICHOST", libc::NI_NUMERICHOST),
        ("NI_NUMERICSERV", libc::NI_NUMERICSERV),
        ("NI_NOFQDN", libc::NI_NOFQDN),
        ("NI_NAMEREQD", libc::NI_NAMEREQD),
        ("NI_DGRAM", libc::NI_DGRAM),
        ("NI_MAXHOST", 1025),
        ("NI_MAXSERV", 32),
        ("EAI_AGAIN", libc::EAI_AGAIN),
        ("EAI_BADFLAGS", libc::EAI_BADFLAGS),
        ("EAI_FAIL", libc::EAI_FAIL),
        ("EAI_FAMILY", libc::EAI_FAMILY),
        ("EAI_MEMORY", libc::EAI_MEMORY),
        ("EAI_NODATA", libc::EAI_NODATA),
        ("EAI_NONAME", libc::EAI_NONAME),
        ("EAI_SERVICE", libc::EAI_SERVICE),
        ("EAI_SOCKTYPE", libc::EAI_SOCKTYPE),
        ("EAI_SYSTEM", libc::EAI_SYSTEM),
        ("INADDR_ANY", 0),
        ("INADDR_LOOPBACK", 0x7f00_0001),
        ("INADDR_NONE", -1),
    ];
    #[cfg(any(target_os = "linux", target_os = "android"))]
    ints.extend([
        ("SOCK_NONBLOCK", libc::SOCK_NONBLOCK),
        ("SOCK_CLOEXEC", libc::SOCK_CLOEXEC),
        ("SO_PROTOCOL", libc::SO_PROTOCOL),
        ("SO_DOMAIN", libc::SO_DOMAIN),
        ("TCP_KEEPIDLE", libc::TCP_KEEPIDLE),
        ("MSG_NOSIGNAL", libc::MSG_NOSIGNAL),
        ("AF_PACKET", libc::AF_PACKET),
        ("AF_NETLINK", libc::AF_NETLINK),
    ]);
    for (n, v) in ints {
        b.set_s(n, Value::Int(v as i64));
    }
    drop(b);
    m
}
