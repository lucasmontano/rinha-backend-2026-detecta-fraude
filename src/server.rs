use crate::index::IndexReader;
use crate::parse::parse_payload;
use crate::response::Responses;
use crate::vectorize::vectorize_q;
use libc::{
    c_int, c_void, cmsghdr, epoll_create1, epoll_ctl, epoll_event, epoll_pwait2, epoll_wait, iovec,
    msghdr, recvmsg, timespec, EAGAIN, EINTR, ENOSYS, EPOLLERR, EPOLLHUP, EPOLLIN, EPOLLOUT,
    EPOLLRDHUP, EPOLL_CTL_ADD, EPOLL_CTL_DEL, EPOLL_CTL_MOD, F_GETFL, F_SETFL, MSG_CMSG_CLOEXEC,
    MSG_DONTWAIT, O_NONBLOCK, SCM_RIGHTS, SOL_SOCKET,
};
use std::io;
use std::mem::MaybeUninit;
use std::os::raw::c_uchar;
use std::path::Path;

const MAX_REQ_HEAD: usize = 4096;
const MAX_BODY: usize = 4096;
const CONN_BUF_CAP: usize = 16384;
const WRITE_BUF_CAP: usize = 512;
const MAX_CLIENT_FD: usize = 65_536;
const DEFAULT_CONN_POOL_INIT: usize = 128;

pub struct Server {
    epfd: c_int,
    uds_fd: c_int,
    responses: &'static Responses,
    index: &'static IndexReader,
    conns: Vec<Option<Box<Conn>>>,
    conn_pool: Vec<Box<Conn>>,
    conn_pool_cap: usize,
}

struct Conn {
    in_buf: [u8; CONN_BUF_CAP],
    in_len: usize,
    write_buf: [u8; WRITE_BUF_CAP],
    write_len: usize,
    write_pos: usize,
}

impl Conn {
    fn new() -> Box<Self> {
        Box::new(Conn {
            in_buf: [0; CONN_BUF_CAP],
            in_len: 0,
            write_buf: [0; WRITE_BUF_CAP],
            write_len: 0,
            write_pos: 0,
        })
    }

    #[inline]
    fn reset(&mut self) {
        self.in_len = 0;
        self.write_len = 0;
        self.write_pos = 0;
    }

    #[inline]
    fn consume(&mut self, n: usize) {
        if n == self.in_len {
            self.in_len = 0;
        } else if n > 0 {
            self.in_buf.copy_within(n..self.in_len, 0);
            self.in_len -= n;
        }
    }

    #[inline]
    fn has_pending_write(&self) -> bool {
        self.write_pos < self.write_len
    }
}

impl Server {
    pub fn new(
        uds_fd: c_int,
        responses: &'static Responses,
        index: &'static IndexReader,
    ) -> io::Result<Self> {
        set_nonblocking(uds_fd)?;
        let epfd = unsafe { epoll_create1(libc::EPOLL_CLOEXEC) };
        if epfd < 0 {
            return Err(io::Error::last_os_error());
        }
        configure_busy_poll(epfd);
        let mut s = Server {
            epfd,
            uds_fd,
            responses,
            index,
            conns: {
                let mut slots = Vec::with_capacity(MAX_CLIENT_FD);
                slots.resize_with(MAX_CLIENT_FD, || None);
                slots
            },
            conn_pool: Vec::new(),
            conn_pool_cap: conn_pool_cap(),
        };
        for _ in 0..s.conn_pool_cap.min(DEFAULT_CONN_POOL_INIT) {
            s.conn_pool.push(Conn::new());
        }
        s.register(uds_fd)?;
        Ok(s)
    }

    fn register(&mut self, fd: c_int) -> io::Result<()> {
        let mut ev = epoll_event {
            events: (EPOLLIN | EPOLLRDHUP) as u32,
            u64: fd as u64,
        };
        let r = unsafe { epoll_ctl(self.epfd, EPOLL_CTL_ADD, fd, &mut ev) };
        if r < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    fn deregister(&mut self, fd: c_int) {
        let mut ev = epoll_event { events: 0, u64: 0 };
        unsafe {
            epoll_ctl(self.epfd, EPOLL_CTL_DEL, fd, &mut ev);
        }
    }

    fn slot_mut(&mut self, fd: c_int) -> Option<&mut Box<Conn>> {
        let idx = fd as usize;
        self.conns.get_mut(idx)?.as_mut()
    }

    fn alloc_conn(&mut self) -> Box<Conn> {
        match self.conn_pool.pop() {
            Some(mut c) => {
                c.reset();
                c
            }
            None => Conn::new(),
        }
    }

    fn put_conn(&mut self, fd: c_int, conn: Box<Conn>) -> bool {
        let idx = fd as usize;
        if idx >= self.conns.len() {
            return false;
        }
        self.conns[idx] = Some(conn);
        true
    }

    fn take_conn(&mut self, fd: c_int) -> Option<Box<Conn>> {
        let idx = fd as usize;
        self.conns.get_mut(idx)?.take()
    }

    pub fn run(&mut self) -> io::Result<()> {
        const MAX_EVENTS: usize = 128;
        let mut events: [epoll_event; MAX_EVENTS] = unsafe { MaybeUninit::zeroed().assume_init() };
        let spin = std::time::Duration::from_micros(epoll_spin_us());
        let idle_us = epoll_idle_us();
        let idle_timeout = epoll_timeout_ms();
        loop {
            let mut n = unsafe { epoll_wait(self.epfd, events.as_mut_ptr(), MAX_EVENTS as i32, 0) };
            if n == 0 && !spin.is_zero() {
                let start = std::time::Instant::now();
                while start.elapsed() < spin {
                    n = unsafe { epoll_wait(self.epfd, events.as_mut_ptr(), MAX_EVENTS as i32, 0) };
                    if n != 0 {
                        break;
                    }
                    std::hint::spin_loop();
                }
            }
            if n == 0 {
                n = wait_idle(
                    self.epfd,
                    events.as_mut_ptr(),
                    MAX_EVENTS as i32,
                    idle_us,
                    idle_timeout,
                );
            }
            if n < 0 {
                let err = io::Error::last_os_error();
                if err.raw_os_error() == Some(EINTR) {
                    continue;
                }
                return Err(err);
            }
            for i in 0..n as usize {
                let fd = events[i].u64 as c_int;
                if fd == self.uds_fd {
                    self.handle_uds()?;
                } else {
                    self.handle_conn(fd, events[i].events);
                }
            }
        }
    }

    fn handle_uds(&mut self) -> io::Result<()> {
        loop {
            match recv_fds(self.uds_fd) {
                Ok(Some(fd)) => {
                    if fd < 0 || fd as usize >= MAX_CLIENT_FD {
                        unsafe { libc::close(fd) };
                        continue;
                    }
                    // The C load balancer accepts client sockets with SOCK_NONBLOCK
                    // and applies TCP options before passing the fd through SCM_RIGHTS.
                    if self.register(fd).is_err() {
                        unsafe { libc::close(fd) };
                        continue;
                    }
                    let conn = self.alloc_conn();
                    if !self.put_conn(fd, conn) {
                        self.deregister(fd);
                        unsafe { libc::close(fd) };
                        continue;
                    }
                    self.handle_conn(fd, EPOLLIN as u32);
                }
                Ok(None) => return Ok(()),
                Err(e) => {
                    if e.kind() == io::ErrorKind::WouldBlock {
                        return Ok(());
                    }
                    return Err(e);
                }
            }
        }
    }

    fn handle_conn(&mut self, fd: c_int, events: u32) {
        let mut close = false;

        if (events & (EPOLLIN as u32)) != 0 {
            let conn = match self.slot_mut(fd) {
                Some(c) => c,
                None => return,
            };
            let space = CONN_BUF_CAP - conn.in_len;
            if space == 0 {
                close = true;
            } else {
                loop {
                    let n = unsafe {
                        libc::recv(
                            fd,
                            conn.in_buf.as_mut_ptr().add(conn.in_len) as *mut c_void,
                            space,
                            0,
                        )
                    };
                    if n > 0 {
                        conn.in_len += n as usize;
                        break;
                    }
                    if n == 0 {
                        close = true;
                        break;
                    }
                    let err = io::Error::last_os_error();
                    let code = err.raw_os_error().unwrap_or(0);
                    if code == EAGAIN || code == libc::EWOULDBLOCK {
                        break;
                    }
                    if code == EINTR {
                        continue;
                    }
                    close = true;
                    break;
                }
            }
        } else if (events & ((EPOLLERR | EPOLLHUP) as u32)) != 0 {
            close = true;
        }

        if !close {
            close = self.process_requests(fd);
        }
        if close {
            self.close_conn(fd);
        }
    }

    fn close_conn(&mut self, fd: c_int) {
        self.deregister(fd);
        if let Some(mut conn) = self.take_conn(fd) {
            conn.reset();
            if self.conn_pool.len() < self.conn_pool_cap {
                self.conn_pool.push(conn);
            }
        }
        unsafe { libc::close(fd) };
    }

    fn process_requests(&mut self, fd: c_int) -> bool {
        let idx = fd as usize;
        let responses = self.responses;
        let index = self.index;
        loop {
            let conn = match self.conns.get_mut(idx).and_then(|s| s.as_mut()) {
                Some(c) => c,
                None => return true,
            };
            if conn.has_pending_write() {
                if !flush_write(fd, conn) {
                    update_interest(self.epfd, fd, true);
                    return false;
                }
                update_interest(self.epfd, fd, false);
            }

            let parse_result = parse_request(&conn.in_buf[..conn.in_len]);
            let (resp, consumed): (&[u8], usize) = match parse_result {
                ParseStatus::Need => return false,
                ParseStatus::Bad => (&responses.fallback[..], conn.in_len),
                ParseStatus::Got {
                    kind,
                    body_start,
                    body_end,
                } => {
                    let body = &conn.in_buf[body_start..body_end];
                    let body_ptr = body.as_ptr();
                    let body_len = body.len();
                    let body: &[u8] = unsafe { std::slice::from_raw_parts(body_ptr, body_len) };
                    let resp = handle_request(kind, body, responses, index);
                    (resp, body_end)
                }
            };

            conn.consume(consumed);

            match write_all_nonblock(fd, resp) {
                WriteResult::Done => continue,
                WriteResult::Pending(remaining) => {
                    if remaining.len() > WRITE_BUF_CAP {
                        return true;
                    }
                    conn.write_buf[..remaining.len()].copy_from_slice(remaining);
                    conn.write_len = remaining.len();
                    conn.write_pos = 0;
                    update_interest(self.epfd, fd, true);
                    return false;
                }
                WriteResult::Closed => return true,
            }
        }
    }
}

enum ParseStatus {
    Need,
    Bad,
    Got {
        kind: RequestKind,
        body_start: usize,
        body_end: usize,
    },
}

#[derive(Clone, Copy)]
enum RequestKind {
    FraudScore,
    Ready,
    Other,
}

fn parse_request(buf: &[u8]) -> ParseStatus {
    let head_end = match find_double_crlf(buf) {
        Some(p) => p,
        None => {
            if buf.len() > MAX_REQ_HEAD {
                return ParseStatus::Bad;
            }
            return ParseStatus::Need;
        }
    };

    let body_start = head_end + 4;

    if buf.starts_with(b"GET /ready ") {
        return ParseStatus::Got {
            kind: RequestKind::Ready,
            body_start,
            body_end: body_start,
        };
    }

    if buf.starts_with(b"POST /fraud-score ") {
        let cl = match find_content_length(&buf[..head_end]) {
            Some(n) => n,
            None => return ParseStatus::Bad,
        };
        if cl > MAX_BODY {
            return ParseStatus::Bad;
        }
        let body_end = body_start + cl;
        if buf.len() < body_end {
            return ParseStatus::Need;
        }
        return ParseStatus::Got {
            kind: RequestKind::FraudScore,
            body_start,
            body_end,
        };
    }

    let line_end = match buf.iter().position(|&b| b == b'\r') {
        Some(p) => p,
        None => return ParseStatus::Bad,
    };
    let line = &buf[..line_end];
    let sp1 = match line.iter().position(|&b| b == b' ') {
        Some(p) => p,
        None => return ParseStatus::Bad,
    };
    let after_method = &line[sp1 + 1..];
    let sp2 = match after_method.iter().position(|&b| b == b' ') {
        Some(p) => p,
        None => return ParseStatus::Bad,
    };
    let method = &line[..sp1];
    let path = &after_method[..sp2];

    if method == b"GET" {
        return ParseStatus::Got {
            kind: if path == b"/ready" {
                RequestKind::Ready
            } else {
                RequestKind::Other
            },
            body_start,
            body_end: body_start,
        };
    }

    let cl = match find_content_length(&buf[..head_end]) {
        Some(n) => n,
        None => return ParseStatus::Bad,
    };
    if cl > MAX_BODY {
        return ParseStatus::Bad;
    }
    let body_end = body_start + cl;
    if buf.len() < body_end {
        return ParseStatus::Need;
    }
    ParseStatus::Got {
        kind: if method == b"POST" && path == b"/fraud-score" {
            RequestKind::FraudScore
        } else {
            RequestKind::Other
        },
        body_start,
        body_end,
    }
}

#[inline]
fn find_double_crlf(buf: &[u8]) -> Option<usize> {
    if buf.len() < 4 {
        return None;
    }
    let mut i = 3;
    while i < buf.len() {
        if buf[i] == b'\n' && buf[i - 1] == b'\r' && buf[i - 2] == b'\n' && buf[i - 3] == b'\r' {
            return Some(i - 3);
        }
        i += 1;
    }
    None
}

fn find_content_length(headers: &[u8]) -> Option<usize> {
    const NEEDLE: &[u8] = b"content-length:";
    let mut i = 0;
    'outer: while i + NEEDLE.len() <= headers.len() {
        for (k, &c) in NEEDLE.iter().enumerate() {
            let h = headers[i + k];
            let lh = if h.is_ascii_uppercase() { h + 32 } else { h };
            if lh != c {
                i += 1;
                continue 'outer;
            }
        }
        let mut j = i + NEEDLE.len();
        while j < headers.len() && (headers[j] == b' ' || headers[j] == b'\t') {
            j += 1;
        }
        let start = j;
        while j < headers.len() && (b'0'..=b'9').contains(&headers[j]) {
            j += 1;
        }
        if j == start {
            return None;
        }
        let mut n = 0usize;
        for &b in &headers[start..j] {
            n = n.checked_mul(10)?.checked_add(usize::from(b - b'0'))?;
        }
        return Some(n);
    }
    None
}

fn handle_request<'a>(
    kind: RequestKind,
    body: &[u8],
    responses: &'a Responses,
    index: &IndexReader,
) -> &'a [u8] {
    match kind {
        RequestKind::FraudScore => {
            let payload = match parse_payload(body) {
                Ok(p) => p,
                Err(_) => return &responses.fallback,
            };
            let q = vectorize_q(&payload);
            let fraud_count = index.fraud_count(&q);
            responses.for_count(fraud_count)
        }
        RequestKind::Ready => &responses.ready,
        RequestKind::Other => &responses.fallback,
    }
}

enum WriteResult<'a> {
    Done,
    Pending(&'a [u8]),
    Closed,
}

#[inline]
fn write_all_nonblock(fd: c_int, mut buf: &[u8]) -> WriteResult<'_> {
    while !buf.is_empty() {
        let n = unsafe {
            libc::send(
                fd,
                buf.as_ptr() as *const c_void,
                buf.len(),
                libc::MSG_NOSIGNAL,
            )
        };
        if n > 0 {
            buf = &buf[n as usize..];
            continue;
        }
        if n == 0 {
            return WriteResult::Closed;
        }
        let err = io::Error::last_os_error();
        let code = err.raw_os_error().unwrap_or(0);
        if code == EAGAIN || code == libc::EWOULDBLOCK {
            return WriteResult::Pending(buf);
        }
        if code == EINTR {
            continue;
        }
        return WriteResult::Closed;
    }
    WriteResult::Done
}

fn flush_write(fd: c_int, conn: &mut Conn) -> bool {
    while conn.write_pos < conn.write_len {
        let remaining = &conn.write_buf[conn.write_pos..conn.write_len];
        let n = unsafe {
            libc::send(
                fd,
                remaining.as_ptr() as *const c_void,
                remaining.len(),
                libc::MSG_NOSIGNAL,
            )
        };
        if n > 0 {
            conn.write_pos += n as usize;
        } else if n == 0 {
            return false;
        } else {
            let err = io::Error::last_os_error();
            let code = err.raw_os_error().unwrap_or(0);
            if code == EAGAIN || code == libc::EWOULDBLOCK {
                return false;
            }
            if code == EINTR {
                continue;
            }
            return false;
        }
    }
    conn.write_len = 0;
    conn.write_pos = 0;
    true
}

fn update_interest(epfd: c_int, fd: c_int, want_write: bool) {
    let mut flags = EPOLLIN | EPOLLRDHUP;
    if want_write {
        flags |= EPOLLOUT;
    }
    let mut ev = epoll_event {
        events: flags as u32,
        u64: fd as u64,
    };
    unsafe {
        epoll_ctl(epfd, EPOLL_CTL_MOD, fd, &mut ev);
    }
}

fn conn_pool_cap() -> usize {
    std::env::var("CONN_POOL_CAP")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(DEFAULT_CONN_POOL_INIT)
}

#[repr(C)]
struct EpollParams {
    busy_poll_usecs: u32,
    busy_poll_budget: u16,
    prefer_busy_poll: u8,
    _pad: u8,
}

const fn iow(ty: u32, nr: u32, size: u32) -> libc::c_ulong {
    ((1u32 << 30) | (size << 16) | (ty << 8) | nr) as libc::c_ulong
}
const EPIOCSPARAMS: libc::c_ulong = iow(0x8A, 0x01, std::mem::size_of::<EpollParams>() as u32);

fn configure_busy_poll(epfd: c_int) {
    let usecs = env_u32("EPOLL_BUSY_POLL_US", 50);
    let prefer = env_u32("EPOLL_PREFER_BUSY_POLL", 1) as u8;
    if usecs == 0 && prefer == 0 {
        return;
    }
    let params = EpollParams {
        busy_poll_usecs: usecs,
        busy_poll_budget: env_u32("EPOLL_BUSY_POLL_BUDGET", 8) as u16,
        prefer_busy_poll: prefer,
        _pad: 0,
    };
    unsafe {
        libc::ioctl(epfd, EPIOCSPARAMS as _, &params as *const EpollParams);
    }
}

fn env_u32(name: &str, default: u32) -> u32 {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn epoll_spin_us() -> u64 {
    std::env::var("EPOLL_SPIN_US")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(50)
}

fn epoll_idle_us() -> u64 {
    std::env::var("EPOLL_IDLE_US")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0)
}

fn epoll_timeout_ms() -> c_int {
    std::env::var("EPOLL_TIMEOUT_MS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1)
}

fn wait_idle(
    epfd: c_int,
    events: *mut epoll_event,
    max_events: c_int,
    idle_us: u64,
    fallback_timeout_ms: c_int,
) -> c_int {
    if idle_us == 0 {
        return unsafe { epoll_wait(epfd, events, max_events, fallback_timeout_ms) };
    }

    let timeout = timespec {
        tv_sec: (idle_us / 1_000_000) as _,
        tv_nsec: ((idle_us % 1_000_000) * 1000) as _,
    };
    let n = unsafe { epoll_pwait2(epfd, events, max_events, &timeout, std::ptr::null()) };
    if n < 0 && std::io::Error::last_os_error().raw_os_error() == Some(ENOSYS) {
        return unsafe { epoll_wait(epfd, events, max_events, fallback_timeout_ms) };
    }
    n
}

fn set_nonblocking(fd: c_int) -> io::Result<()> {
    unsafe {
        let flags = libc::fcntl(fd, F_GETFL);
        if flags < 0 {
            return Err(io::Error::last_os_error());
        }
        let r = libc::fcntl(fd, F_SETFL, flags | O_NONBLOCK);
        if r < 0 {
            return Err(io::Error::last_os_error());
        }
    }
    Ok(())
}

fn set_tcp_options(fd: c_int) {
    let on: c_int = 1;
    unsafe {
        libc::setsockopt(
            fd,
            libc::IPPROTO_TCP,
            libc::TCP_NODELAY,
            &on as *const c_int as *const c_void,
            std::mem::size_of::<c_int>() as libc::socklen_t,
        );
        libc::setsockopt(
            fd,
            libc::IPPROTO_TCP,
            libc::TCP_QUICKACK,
            &on as *const c_int as *const c_void,
            std::mem::size_of::<c_int>() as libc::socklen_t,
        );
    }
}

fn recv_fds(uds_fd: c_int) -> io::Result<Option<c_int>> {
    let mut dummy = [0u8; 1];
    let mut iov = iovec {
        iov_base: dummy.as_mut_ptr() as *mut c_void,
        iov_len: dummy.len(),
    };
    const CMSG_SPACE: usize = 24 + 16;
    let mut cmsg_buf = [0u8; CMSG_SPACE];
    let mut mh: msghdr = unsafe { std::mem::zeroed() };
    mh.msg_iov = &mut iov;
    mh.msg_iovlen = 1;
    mh.msg_control = cmsg_buf.as_mut_ptr() as *mut c_void;
    mh.msg_controllen = cmsg_buf.len() as _;

    let n = unsafe { recvmsg(uds_fd, &mut mh, MSG_DONTWAIT | MSG_CMSG_CLOEXEC) };
    if n < 0 {
        let err = io::Error::last_os_error();
        let code = err.raw_os_error().unwrap_or(0);
        if code == EAGAIN || code == libc::EWOULDBLOCK {
            return Err(err);
        }
        if code == EINTR {
            return Ok(None);
        }
        return Err(err);
    }
    if mh.msg_controllen == 0 {
        return Ok(None);
    }
    let cmsg = unsafe { CMSG_FIRSTHDR(&mh) };
    if cmsg.is_null() {
        return Ok(None);
    }
    let level = unsafe { (*cmsg).cmsg_level };
    let ctype = unsafe { (*cmsg).cmsg_type };
    if level != SOL_SOCKET || ctype != SCM_RIGHTS {
        return Ok(None);
    }
    let data = unsafe { CMSG_DATA(cmsg) };
    let fd = unsafe { std::ptr::read_unaligned(data as *const c_int) };
    Ok(Some(fd))
}

#[allow(non_snake_case)]
unsafe fn CMSG_FIRSTHDR(mh: *const msghdr) -> *mut cmsghdr {
    if (*mh).msg_controllen as usize >= std::mem::size_of::<cmsghdr>() {
        (*mh).msg_control as *mut cmsghdr
    } else {
        std::ptr::null_mut()
    }
}

#[allow(non_snake_case)]
unsafe fn CMSG_DATA(cmsg: *const cmsghdr) -> *mut c_uchar {
    (cmsg as *mut u8).add(cmsg_align(std::mem::size_of::<cmsghdr>())) as *mut c_uchar
}

fn cmsg_align(n: usize) -> usize {
    (n + std::mem::size_of::<usize>() - 1) & !(std::mem::size_of::<usize>() - 1)
}

pub fn create_listener(path: &Path) -> io::Result<c_int> {
    let _ = std::fs::remove_file(path);
    let fd = unsafe { libc::socket(libc::AF_UNIX, libc::SOCK_SEQPACKET | libc::SOCK_CLOEXEC, 0) };
    if fd < 0 {
        return Err(io::Error::last_os_error());
    }
    let mut addr: libc::sockaddr_un = unsafe { std::mem::zeroed() };
    addr.sun_family = libc::AF_UNIX as libc::sa_family_t;
    let bytes = path.as_os_str().as_encoded_bytes();
    if bytes.len() >= addr.sun_path.len() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "path too long"));
    }
    for (i, b) in bytes.iter().enumerate() {
        addr.sun_path[i] = *b as libc::c_char;
    }
    let len = std::mem::size_of::<libc::sa_family_t>() + bytes.len();
    let r = unsafe {
        libc::bind(
            fd,
            &addr as *const libc::sockaddr_un as *const libc::sockaddr,
            len as libc::socklen_t,
        )
    };
    if r < 0 {
        return Err(io::Error::last_os_error());
    }
    if unsafe { libc::listen(fd, 4) } < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(fd)
}

pub fn accept_lb(listener: c_int) -> io::Result<c_int> {
    let fd = unsafe {
        libc::accept4(
            listener,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            libc::SOCK_CLOEXEC,
        )
    };
    if fd < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(fd)
}

pub fn close_fd(fd: c_int) {
    unsafe {
        libc::close(fd);
    }
}
