// SPDX-License-Identifier: GPL-2.0-only
// UART1 loopback: connect header pin 27 (TX) to pin 26 (RX).
#define _DEFAULT_SOURCE
#include <errno.h>
#include <fcntl.h>
#include <poll.h>
#include <stdio.h>
#include <string.h>
#include <termios.h>
#include <time.h>
#include <unistd.h>

static long long now_ms(void)
{
    struct timespec now;
    if (clock_gettime(CLOCK_MONOTONIC, &now) < 0) {
        perror("clock_gettime");
        return -1;
    }
    return (long long)now.tv_sec * 1000 + now.tv_nsec / 1000000;
}

int main(int argc, char **argv)
{
    const char message[] = "UART1 loopback: hello from TX pin 27!\r\n";
    char received[sizeof(message)] = {0};
    const size_t length = sizeof(message) - 1;
    const char *device = argc == 2 ? argv[1] : "/dev/ttyS1";
    struct termios saved, config;
    size_t sent = 0, count = 0;
    int result = 1, configured = 0;

    if (argc > 2 || (argc == 2 && !strcmp(argv[1], "--help"))) {
        printf("Usage: %s [/dev/ttyS1]\n"
               "Connect UART1 TX pin 27 to RX pin 26.\n"
               "Uses 115200 baud, 8N1, no flow control, 2-second timeout.\n",
               argv[0]);
        return argc > 2;
    }
    setvbuf(stdout, NULL, _IOLBF, 0);
    int fd = open(device, O_RDWR | O_NOCTTY);
    if (fd < 0) {
        perror(device);
        return 1;
    }
    if (tcgetattr(fd, &saved) < 0) {
        perror("tcgetattr");
        goto out;
    }
    config = saved;
    cfmakeraw(&config); /* Disable terminal echo and byte translations. */
    config.c_cflag &= ~CRTSCTS;
    config.c_cflag |= CLOCAL | CREAD;
    config.c_cc[VMIN] = 0;
    config.c_cc[VTIME] = 0;
    if (cfsetispeed(&config, B115200) < 0 ||
        cfsetospeed(&config, B115200) < 0 ||
        tcsetattr(fd, TCSANOW, &config) < 0) {
        perror("Configure UART");
        goto out;
    }
    configured = 1;
    if (tcflush(fd, TCIOFLUSH) < 0) {
        perror("tcflush");
        goto out;
    }

    printf("%s: 115200 baud, 8N1\nTX (%zu bytes): %s", device, length, message);
    while (sent < length) {
        ssize_t n = write(fd, message + sent, length - sent);
        if (n < 0 && errno == EINTR)
            continue;
        if (n <= 0) {
            if (n < 0) perror("write");
            else fprintf(stderr, "write returned zero bytes\n");
            goto out;
        }
        sent += (size_t)n;
    }

    long long start = now_ms();
    if (start < 0)
        goto out;
    while (count < length) {
        long long now = now_ms();
        if (now < 0)
            goto out;
        int remaining = (int)(start + 2000 - now);
        if (remaining <= 0) {
            fprintf(stderr, "TIMEOUT: received %zu/%zu bytes. Check the TX/RX jumper.\n",
                    count, length);
            goto out;
        }
        struct pollfd uart = {.fd = fd, .events = POLLIN};
        int ready = poll(&uart, 1, remaining);
        if (ready < 0 && errno == EINTR)
            continue;
        if (ready < 0) {
            perror("poll");
            goto out;
        }
        if (ready == 0)
            continue;
        if (uart.revents & (POLLERR | POLLHUP | POLLNVAL)) {
            fprintf(stderr, "UART error: poll revents=0x%x\n", uart.revents);
            goto out;
        }
        ssize_t n = read(fd, received + count, length - count);
        if (n < 0 && errno == EINTR)
            continue;
        if (n < 0) {
            perror("read");
            goto out;
        }
        count += (size_t)n;
    }
    printf("RX (%zu bytes): %s", count, received);
    if (memcmp(message, received, length)) {
        fprintf(stderr, "FAIL: received data differs from transmitted data\n");
        goto out;
    }
    printf("PASS: all %zu bytes match\n", length);
    result = 0;

out:
    if (configured && tcsetattr(fd, TCSANOW, &saved) < 0) {
        perror("Restore UART settings");
        result = 1;
    }
    close(fd);
    return result;
}
