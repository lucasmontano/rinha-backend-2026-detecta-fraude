#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("fd-passing load balancer requires Linux");
}

#[cfg(target_os = "linux")]
mod linux {
    use std::env;
    use std::ffi::CString;
    use std::io;
    use std::mem;
    use std::os::fd::RawFd;
    use std::ptr;
    use std::thread;
    use std::time::Duration;

    struct Backend {
        path: String,
        fd: RawFd,
    }

    pub fn main() {
        unsafe {
            libc::signal(libc::SIGPIPE, libc::SIG_IGN);
        }

        let port = env_or("LB_PORT", 9999u16);
        let backlog = env_or("LB_BACKLOG", 65_535i32);
        let accept_batch = env_or("LB_ACCEPT_BATCH", 64usize);
        let socket_paths = env::var("API_SOCKETS")
            .unwrap_or_else(|_| "/sockets/api1.sock,/sockets/api2.sock".to_string());
        let mut backends: Vec<Backend> = socket_paths
            .split(',')
            .map(|path| {
                let path = path.trim().to_string();
                let fd = connect_backend_retry(&path);
                Backend { path, fd }
            })
            .collect();

        if backends.is_empty() {
            eprintln!("API_SOCKETS must contain at least one Unix socket path");
            std::process::exit(1);
        }

        let listener = listen_tcp(port, backlog).unwrap_or_else(|err| {
            eprintln!("failed to listen on :{port}: {err}");
            std::process::exit(1);
        });

        eprintln!(
            "fd lb listening on :{port}; backends={}; batch={accept_batch}",
            backends.len()
        );
        let mut next = 0usize;
        loop {
            let mut accepted = 0usize;
            while accepted < accept_batch {
                let client = match accept_client(listener) {
                    Ok(Some(fd)) => fd,
                    Ok(None) => break,
                    Err(_) => break,
                };
                accepted += 1;
                tune_client(client);

                let first = next;
                next = (next + 1) % backends.len();
                let mut sent = false;
                for offset in 0..backends.len() {
                    let index = (first + offset) % backends.len();
                    if send_client_fd(&mut backends[index], client, true).is_ok() {
                        sent = true;
                        break;
                    }
                }
                if !sent {
                    let _ = send_client_fd(&mut backends[first], client, false);
                }
                close_fd(client);
            }

            if accepted == 0 {
                wait_read(listener);
            }
        }
    }

    fn listen_tcp(port: u16, backlog: i32) -> io::Result<RawFd> {
        let fd = unsafe {
            libc::socket(
                libc::AF_INET,
                libc::SOCK_STREAM | libc::SOCK_NONBLOCK | libc::SOCK_CLOEXEC,
                0,
            )
        };
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }

        set_flag(fd, libc::SOL_SOCKET, libc::SO_REUSEADDR);
        set_flag(fd, libc::SOL_SOCKET, libc::SO_REUSEPORT);
        set_int(fd, libc::IPPROTO_TCP, libc::TCP_DEFER_ACCEPT, 1);

        let addr = libc::sockaddr_in {
            sin_family: libc::AF_INET as libc::sa_family_t,
            sin_port: port.to_be(),
            sin_addr: libc::in_addr {
                s_addr: libc::INADDR_ANY.to_be(),
            },
            sin_zero: [0; 8],
        };
        let bind_result = unsafe {
            libc::bind(
                fd,
                &addr as *const _ as *const libc::sockaddr,
                mem::size_of::<libc::sockaddr_in>() as libc::socklen_t,
            )
        };
        if bind_result < 0 {
            let err = io::Error::last_os_error();
            close_fd(fd);
            return Err(err);
        }
        if unsafe { libc::listen(fd, backlog) } < 0 {
            let err = io::Error::last_os_error();
            close_fd(fd);
            return Err(err);
        }
        Ok(fd)
    }

    fn accept_client(listener: RawFd) -> io::Result<Option<RawFd>> {
        let fd = unsafe {
            libc::accept4(
                listener,
                ptr::null_mut(),
                ptr::null_mut(),
                libc::SOCK_CLOEXEC,
            )
        };
        if fd >= 0 {
            return Ok(Some(fd));
        }

        let err = io::Error::last_os_error();
        match err.raw_os_error() {
            Some(code) if code == libc::EAGAIN || code == libc::EWOULDBLOCK => Ok(None),
            Some(libc::EINTR) => Ok(None),
            _ => Err(err),
        }
    }

    fn connect_backend_retry(path: &str) -> RawFd {
        loop {
            match connect_backend(path) {
                Ok(fd) => return fd,
                Err(_) => thread::sleep(Duration::from_millis(20)),
            }
        }
    }

    fn connect_backend(path: &str) -> io::Result<RawFd> {
        let fd = unsafe { libc::socket(libc::AF_UNIX, libc::SOCK_STREAM | libc::SOCK_CLOEXEC, 0) };
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }
        set_int(fd, libc::SOL_SOCKET, libc::SO_SNDBUF, 256 * 1024);

        let c_path = CString::new(path).map_err(|_| {
            io::Error::new(io::ErrorKind::InvalidInput, "Unix socket path contains NUL")
        })?;
        let bytes = c_path.as_bytes_with_nul();
        let mut addr: libc::sockaddr_un = unsafe { mem::zeroed() };
        addr.sun_family = libc::AF_UNIX as libc::sa_family_t;
        if bytes.len() > addr.sun_path.len() {
            close_fd(fd);
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Unix socket path is too long",
            ));
        }
        for (idx, byte) in bytes.iter().enumerate() {
            addr.sun_path[idx] = *byte as libc::c_char;
        }

        let len = (mem::size_of::<libc::sa_family_t>() + bytes.len()) as libc::socklen_t;
        if unsafe { libc::connect(fd, &addr as *const _ as *const libc::sockaddr, len) } < 0 {
            let err = io::Error::last_os_error();
            close_fd(fd);
            return Err(err);
        }
        Ok(fd)
    }

    fn send_client_fd(backend: &mut Backend, client: RawFd, nonblocking: bool) -> io::Result<()> {
        match send_fd_once(backend.fd, client, nonblocking) {
            Ok(()) => Ok(()),
            Err(_) => {
                close_fd(backend.fd);
                backend.fd = connect_backend_retry(&backend.path);
                send_fd_once(backend.fd, client, nonblocking)
            }
        }
    }

    fn send_fd_once(control: RawFd, client: RawFd, nonblocking: bool) -> io::Result<()> {
        let mut byte = [1u8; 1];
        let mut iov = libc::iovec {
            iov_base: byte.as_mut_ptr().cast(),
            iov_len: byte.len(),
        };
        let mut control_buf = [0u8; 64];
        let mut msg: libc::msghdr = unsafe { mem::zeroed() };
        msg.msg_iov = &mut iov;
        msg.msg_iovlen = 1;
        msg.msg_control = control_buf.as_mut_ptr().cast();
        msg.msg_controllen = control_buf.len();

        let cmsg = unsafe { libc::CMSG_FIRSTHDR(&msg) };
        if cmsg.is_null() {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "missing cmsg"));
        }
        unsafe {
            (*cmsg).cmsg_level = libc::SOL_SOCKET;
            (*cmsg).cmsg_type = libc::SCM_RIGHTS;
            (*cmsg).cmsg_len = libc::CMSG_LEN(mem::size_of::<RawFd>() as u32) as usize;
            ptr::copy_nonoverlapping(client.to_ne_bytes().as_ptr(), libc::CMSG_DATA(cmsg), 4);
        }
        msg.msg_controllen = unsafe { libc::CMSG_SPACE(mem::size_of::<RawFd>() as u32) as usize };

        let flags = libc::MSG_NOSIGNAL | if nonblocking { libc::MSG_DONTWAIT } else { 0 };
        let sent = unsafe { libc::sendmsg(control, &msg, flags) };
        if sent >= 0 {
            Ok(())
        } else {
            Err(io::Error::last_os_error())
        }
    }

    fn tune_client(fd: RawFd) {
        set_flag(fd, libc::IPPROTO_TCP, libc::TCP_NODELAY);
        set_flag(fd, libc::IPPROTO_TCP, libc::TCP_QUICKACK);
    }

    fn set_flag(fd: RawFd, level: libc::c_int, name: libc::c_int) {
        set_int(fd, level, name, 1);
    }

    fn set_int(fd: RawFd, level: libc::c_int, name: libc::c_int, value: libc::c_int) {
        unsafe {
            libc::setsockopt(
                fd,
                level,
                name,
                &value as *const _ as *const libc::c_void,
                mem::size_of::<libc::c_int>() as libc::socklen_t,
            );
        }
    }

    fn wait_read(fd: RawFd) {
        let mut poll_fd = libc::pollfd {
            fd,
            events: libc::POLLIN,
            revents: 0,
        };
        unsafe {
            libc::poll(&mut poll_fd, 1, -1);
        }
    }

    fn close_fd(fd: RawFd) {
        unsafe {
            libc::close(fd);
        }
    }

    fn env_or<T: std::str::FromStr>(key: &str, default: T) -> T {
        env::var(key)
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(default)
    }
}

#[cfg(target_os = "linux")]
fn main() {
    linux::main();
}
