#[cfg(target_os = "linux")]
use detecta_fraude::response::Responses;
#[cfg(target_os = "linux")]
use detecta_fraude::server::{accept_lb, create_listener, Server};
#[cfg(target_os = "linux")]
use std::env;
#[cfg(target_os = "linux")]
use std::path::{Path, PathBuf};
#[cfg(target_os = "linux")]
use std::process;
#[cfg(target_os = "linux")]
use std::thread;

#[cfg(target_os = "linux")]
fn main() {
    let prefix = env::var("API_SOCKET_PREFIX").unwrap_or_else(|_| "/sockets/api1".to_string());
    let workers: usize = env::var("API_WORKERS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(2);
    let responses: &'static Responses = Box::leak(Box::new(Responses::new()));

    eprintln!(
        "[api] modelo gerado carregado. workers: {} prefix: {}",
        workers, prefix
    );

    let mut handles = Vec::with_capacity(workers);
    for w in 0..workers {
        let socket = format!("{}-w{}.sock", prefix, w);
        let prefix_clone = prefix.clone();
        let handle = thread::Builder::new()
            .name(format!("worker-{}", w))
            .spawn(move || run_worker(w, socket, prefix_clone, responses))
            .expect("spawn worker");
        handles.push(handle);
    }
    for h in handles {
        let _ = h.join();
    }
}

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("rinha-fraud uses Linux epoll/socket APIs and must run in the Docker image");
}

#[cfg(target_os = "linux")]
fn set_worker_nice() {
    let nice: i32 = match env::var("WORKER_NICE").ok().and_then(|v| v.parse().ok()) {
        Some(n) => n,
        None => return,
    };
    unsafe {
        let tid = libc::syscall(libc::SYS_gettid) as libc::id_t;
        libc::setpriority(libc::PRIO_PROCESS, tid, nice);
    }
}

#[cfg(target_os = "linux")]
fn run_worker(w: usize, socket: String, prefix: String, responses: &'static Responses) {
    set_worker_nice();
    let listener = match create_listener(&PathBuf::from(&socket)) {
        Ok(fd) => fd,
        Err(e) => {
            eprintln!("[api-w{}] erro criando UDS {}: {}", w, socket, e);
            process::exit(1);
        }
    };

    if let Some(parent) = Path::new(&socket).parent() {
        let stem = Path::new(&socket)
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .to_string();
        let _ = std::fs::write(parent.join(format!("{}.ready", stem)), b"1");
        let prefix_name = Path::new(&prefix)
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();
        let _ = std::fs::write(parent.join(format!("{}.ready", prefix_name)), b"1");
    }

    loop {
        let uds_fd = match accept_lb(listener) {
            Ok(fd) => fd,
            Err(e) => {
                eprintln!("[api-w{}] accept_lb erro: {}", w, e);
                continue;
            }
        };
        eprintln!("[api-w{}] LB conectado (fd={})", w, uds_fd);
        match Server::new(uds_fd, responses) {
            Ok(mut s) => {
                if let Err(e) = s.run() {
                    eprintln!("[api-w{}] server.run erro: {}", w, e);
                }
            }
            Err(e) => {
                eprintln!("[api-w{}] Server::new erro: {}", w, e);
                detecta_fraude::server::close_fd(uds_fd);
            }
        }
    }
}
