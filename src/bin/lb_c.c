
#define _GNU_SOURCE
#include <arpa/inet.h>
#include <errno.h>
#include <fcntl.h>
#include <netinet/in.h>
#include <netinet/tcp.h>
#include <poll.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <sys/un.h>
#include <time.h>
#include <unistd.h>

#define MAX_BACKENDS 32
#define DEFAULT_ACCEPT_BATCH 64

typedef struct {
    int fd;
    char dummy;
    struct iovec iov;
    union {
        struct cmsghdr cm;
        char buf[CMSG_SPACE(sizeof(int))];
    } control;
    struct msghdr msg;
    struct cmsghdr *cmsg;
} backend_t;

static int getenv_int(const char *name, int fallback) {
    const char *v = getenv(name);
    if (!v || !*v) return fallback;
    int parsed = atoi(v);
    return parsed > 0 ? parsed : fallback;
}

static int connect_backend(const char *path) {
    int fd = socket(AF_UNIX, SOCK_SEQPACKET | SOCK_CLOEXEC, 0);
    if (fd < 0) return -1;
    int sndbuf = 256 * 1024;
    setsockopt(fd, SOL_SOCKET, SO_SNDBUF, &sndbuf, sizeof(sndbuf));
    struct sockaddr_un addr = {0};
    addr.sun_family = AF_UNIX;
    strncpy(addr.sun_path, path, sizeof(addr.sun_path) - 1);
    if (connect(fd, (struct sockaddr *)&addr, sizeof(addr)) < 0) {
        close(fd);
        return -1;
    }
    return fd;
}

static void init_backend(backend_t *b, int fd) {
    memset(b, 0, sizeof(*b));
    b->fd = fd;
    b->dummy = 1;
    b->iov.iov_base = &b->dummy;
    b->iov.iov_len = 1;
    b->msg.msg_iov = &b->iov;
    b->msg.msg_iovlen = 1;
    b->msg.msg_control = b->control.buf;
    b->msg.msg_controllen = sizeof(b->control.buf);
    b->cmsg = CMSG_FIRSTHDR(&b->msg);
    b->cmsg->cmsg_level = SOL_SOCKET;
    b->cmsg->cmsg_type = SCM_RIGHTS;
    b->cmsg->cmsg_len = CMSG_LEN(sizeof(int));
}

static int wait_for_socket(const char *path) {
    int tries = 0;
    while (tries++ < 600) {
        struct stat st;
        if (stat(path, &st) == 0) return 0;
        struct timespec ts = { .tv_sec = 0, .tv_nsec = 100 * 1000 * 1000 };
        nanosleep(&ts, NULL);
    }
    return -1;
}

static int send_fd_with_flags(backend_t *dst, int fd, int flags) {
    dst->msg.msg_controllen = sizeof(dst->control.buf);
    memcpy(CMSG_DATA(dst->cmsg), &fd, sizeof(int));
    for (;;) {
        ssize_t r = sendmsg(dst->fd, &dst->msg, MSG_NOSIGNAL | flags);
        if (r > 0) return 0;
        if (r < 0 && errno == EINTR) continue;
        return -1;
    }
}

static int send_fd(backend_t *dst, int fd) {
    return send_fd_with_flags(dst, fd, MSG_DONTWAIT);
}

static int send_fd_blocking(backend_t *dst, int fd) {
    return send_fd_with_flags(dst, fd, 0);
}

static int parse_backends(const char *env, char *paths[MAX_BACKENDS]) {
    int n = 0;
    char *tmp = strdup(env);
    char *save = NULL;
    char *tok = strtok_r(tmp, ",", &save);
    while (tok && n < MAX_BACKENDS) {
        paths[n++] = strdup(tok);
        tok = strtok_r(NULL, ",", &save);
    }
    free(tmp);
    return n;
}

int main(int argc, char **argv) {
    (void)argc;
    (void)argv;
    signal(SIGPIPE, SIG_IGN);

    int port = 9999;
    if (getenv("LB_PORT")) port = atoi(getenv("LB_PORT"));
    int backlog = getenv_int("LB_BACKLOG", 4096);
    int accept_batch = getenv_int("LB_ACCEPT_BATCH", DEFAULT_ACCEPT_BATCH);
    int tcp_nodelay = getenv_int("LB_TCP_NODELAY", 1);
    int tcp_quickack = getenv_int("LB_TCP_QUICKACK", 1);

    const char *socks_env = getenv("API_SOCKETS");
    if (!socks_env || !*socks_env) socks_env = "/sockets/api1.sock,/sockets/api2.sock";

    char *paths[MAX_BACKENDS] = {0};
    int nb = parse_backends(socks_env, paths);
    if (nb <= 0) {
        fprintf(stderr, "[lb] sem backends\n");
        return 2;
    }

    backend_t backends[MAX_BACKENDS];
    for (int i = 0; i < nb; i++) {
        fprintf(stderr, "[lb] aguardando %s\n", paths[i]);
        if (wait_for_socket(paths[i]) < 0) {
            fprintf(stderr, "[lb] timeout aguardando %s\n", paths[i]);
            return 3;
        }
        int fd = -1;
        for (int t = 0; t < 100; t++) {
            fd = connect_backend(paths[i]);
            if (fd >= 0) break;
            struct timespec ts = { .tv_sec = 0, .tv_nsec = 100 * 1000 * 1000 };
            nanosleep(&ts, NULL);
        }
        if (fd < 0) {
            fprintf(stderr, "[lb] falha conectando %s\n", paths[i]);
            return 4;
        }
        init_backend(&backends[i], fd);
        fprintf(stderr, "[lb] conectado em %s (fd=%d)\n", paths[i], fd);
    }

    int lfd = socket(AF_INET, SOCK_STREAM | SOCK_NONBLOCK | SOCK_CLOEXEC, 0);
    if (lfd < 0) {
        perror("socket");
        return 5;
    }
    int on = 1;
    setsockopt(lfd, SOL_SOCKET, SO_REUSEADDR, &on, sizeof(on));
    setsockopt(lfd, SOL_SOCKET, SO_REUSEPORT, &on, sizeof(on));
    setsockopt(lfd, IPPROTO_TCP, TCP_DEFER_ACCEPT, &on, sizeof(on));

    struct sockaddr_in addr = {0};
    addr.sin_family = AF_INET;
    addr.sin_addr.s_addr = htonl(INADDR_ANY);
    addr.sin_port = htons(port);
    if (bind(lfd, (struct sockaddr *)&addr, sizeof(addr)) < 0) {
        perror("bind");
        return 6;
    }
    if (listen(lfd, backlog) < 0) {
        perror("listen");
        return 7;
    }

    fprintf(stderr, "[lb] escutando :%d backlog=%d batch=%d, %d backends\n", port, backlog, accept_batch, nb);

    int rr = 0;
    for (;;) {
        int accepted = 0;
        while (accepted < accept_batch) {
            int cfd = accept4(lfd, NULL, NULL, SOCK_NONBLOCK | SOCK_CLOEXEC);
            if (cfd < 0) {
                if (errno == EINTR) continue;
                if (errno == EAGAIN || errno == EWOULDBLOCK) break;
                break;
            }
            accepted++;
            int one = 1;
            if (tcp_nodelay) {
                setsockopt(cfd, IPPROTO_TCP, TCP_NODELAY, &one, sizeof(one));
            }
            if (tcp_quickack) {
                setsockopt(cfd, IPPROTO_TCP, TCP_QUICKACK, &one, sizeof(one));
            }
            int target = rr;
            rr = (rr + 1) % nb;
            if (send_fd(&backends[target], cfd) != 0) {
                (void)send_fd_blocking(&backends[target], cfd);
            }
            close(cfd);
        }
        if (accepted == 0) {
            struct pollfd pfd = { .fd = lfd, .events = POLLIN, .revents = 0 };
            poll(&pfd, 1, -1);
        }
    }
}
