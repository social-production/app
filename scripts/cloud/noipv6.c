// Local testing only: on kernels without IPv6, socket(AF_INET6) fails with EAFNOSUPPORT, but
// holochain_websocket only falls back to IPv4 on EADDRNOTAVAIL. Translate the errno.
#define _GNU_SOURCE
#include <dlfcn.h>
#include <errno.h>
#include <sys/socket.h>
int socket(int domain, int type, int protocol) {
  static int (*real)(int, int, int);
  if (!real) real = dlsym(RTLD_NEXT, "socket");
  int fd = real(domain, type, protocol);
  if (fd < 0 && domain == AF_INET6 && errno == EAFNOSUPPORT) errno = EADDRNOTAVAIL;
  return fd;
}
