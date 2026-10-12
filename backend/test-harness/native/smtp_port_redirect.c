/*
 * smtp_port_redirect.c - TEST-ONLY, rootless shim for the Kunjika backend.
 *
 * Why this exists
 * ---------------
 * backend/src/config.rs (which we must not modify) builds the mailer with
 *
 *     lettre::transport::smtp::AsyncSmtpTransport::<Tokio1Executor>::unencrypted_localhost()
 *
 * which hard-codes 127.0.0.1:25 and ignores Config::mail_host / mail_port.
 * backend/src/main.rs additionally refuses to run as root, so we cannot use
 * sudo to make the test process privileged and grab :25.
 *
 * Rootless podman containers *do* own their network namespace, so they can
 * bind privileged ports - but running the 240 MB backend inside an image is
 * heavy. Instead we interpose connect(2) via LD_PRELOAD and rewrite the
 * destination port 25 -> $KUNJIKA_SMTP_FORWARD_PORT (2525), where a plain
 * (non-STARTTLS) Mailpit is published.
 *
 * This changes no product code: it is applied only to the backend process
 * spawned by backend_test_common.rs, and only for outbound TCP port 25.
 */
#define _GNU_SOURCE
#include <arpa/inet.h>
#include <dlfcn.h>
#include <netinet/in.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>

typedef int (*connect_fn)(int, const struct sockaddr *, socklen_t);

int connect(int sockfd, const struct sockaddr *addr, socklen_t addrlen)
{
    static connect_fn real_connect = NULL;
    if (!real_connect)
        real_connect = (connect_fn)dlsym(RTLD_NEXT, "connect");

    const char *port_env = getenv("KUNJIKA_SMTP_REDIRECT_FROM");
    if (port_env && addr) {
        unsigned short from = (unsigned short)atoi(port_env);
        unsigned short to = (unsigned short)atoi(getenv("KUNJIKA_SMTP_REDIRECT_TO"));

        if (addr->sa_family == AF_INET && to) {
            struct sockaddr_in sin;
            memcpy(&sin, addr, sizeof(sin));
            if (ntohs(sin.sin_port) == from) {
                sin.sin_port = htons(to);
                fprintf(stderr, "[smtp_port_redirect] %s:%d -> %s:%d\n",
                        inet_ntoa(sin.sin_addr), from, inet_ntoa(sin.sin_addr), to);
                return real_connect(sockfd, (const struct sockaddr *)&sin, sizeof(sin));
            }
        } else if (addr->sa_family == AF_INET6 && to) {
            struct sockaddr_in6 sin6;
            memcpy(&sin6, addr, sizeof(sin6));
            if (ntohs(sin6.sin6_port) == from) {
                sin6.sin6_port = htons(to);
                fprintf(stderr, "[smtp_port_redirect] [ipv6]:%d -> [ipv6]:%d\n", from, to);
                return real_connect(sockfd, (const struct sockaddr *)&sin6, sizeof(sin6));
            }
        }
    }
    return real_connect(sockfd, addr, addrlen);
}