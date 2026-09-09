// SPDX-License-Identifier: GPL-2.0-only
// Print complete multitouch frames without grabbing the input device.
#include <errno.h>
#include <fcntl.h>
#include <glob.h>
#include <linux/input.h>
#include <stdio.h>
#include <string.h>
#include <sys/ioctl.h>
#include <unistd.h>

#define MAX_SLOTS 64

struct contact {
    int id, x, y, pressure;
};

static void report(struct contact *now, struct contact *old, int count)
{
    for (int i = 0; i < count; i++) {
        if (old[i].id >= 0 && old[i].id != now[i].id)
            printf("UP   slot=%d id=%d x=%d y=%d\n",
                   i, old[i].id, old[i].x, old[i].y);
        if (now[i].id >= 0 &&
            (now[i].id != old[i].id || now[i].x != old[i].x ||
             now[i].y != old[i].y || now[i].pressure != old[i].pressure))
            printf("%s slot=%d id=%d x=%d y=%d pressure=%d\n",
                   now[i].id != old[i].id ? "DOWN" : "MOVE",
                   i, now[i].id, now[i].x, now[i].y, now[i].pressure);
        old[i] = now[i];
    }
}

static int open_touch(const char *path)
{
    glob_t paths = {0};
    int fd = -1;

    if (path)
        return open(path, O_RDONLY);

    if (glob("/dev/input/event*", 0, NULL, &paths) == 0) {
        for (size_t i = 0; i < paths.gl_pathc; i++) {
            char name[256] = {0};
            fd = open(paths.gl_pathv[i], O_RDONLY);
            if (fd < 0)
                continue;
            if (ioctl(fd, EVIOCGNAME(sizeof(name)), name) >= 0 &&
                strstr(name, "CST3530")) {
                printf("Device: %s (%s)\n", paths.gl_pathv[i], name);
                break;
            }
            close(fd);
            fd = -1;
        }
    }
    globfree(&paths);
    if (fd < 0)
        errno = ENODEV;
    return fd;
}

int main(int argc, char **argv)
{
    struct contact now[MAX_SLOTS] = {0}, old[MAX_SLOTS] = {0};
    struct input_absinfo info;
    struct input_event events[64];
    int fd, count, slot;

    if (argc > 2 || (argc == 2 && !strcmp(argv[1], "--help"))) {
        fprintf(stderr, "Usage: %s [/dev/input/eventN]\n", argv[0]);
        return argc > 2;
    }
    setvbuf(stdout, NULL, _IOLBF, 0);
    fd = open_touch(argc == 2 ? argv[1] : NULL);
    if (fd < 0) {
        perror("Open touchscreen (specify /dev/input/eventN if needed)");
        return 1;
    }
    if (ioctl(fd, EVIOCGABS(ABS_MT_SLOT), &info) < 0 ||
        info.minimum != 0 || info.maximum < 0 || info.maximum >= MAX_SLOTS) {
        fprintf(stderr, "Expected a type-B multitouch device with 1..64 slots\n");
        close(fd);
        return 1;
    }
    count = info.maximum + 1;
    slot = info.value;
    for (int i = 0; i < count; i++)
        now[i].id = old[i].id = -1;

    /* Capture fingers already down, plus coordinates omitted from later events. */
    const int codes[] = {ABS_MT_TRACKING_ID, ABS_MT_POSITION_X,
                         ABS_MT_POSITION_Y, ABS_MT_PRESSURE};
    for (size_t c = 0; c < sizeof(codes) / sizeof(codes[0]); c++) {
        int values[MAX_SLOTS + 1] = {0};
        values[0] = codes[c];
        if (ioctl(fd, EVIOCGMTSLOTS((count + 1) * sizeof(int)), values) < 0) {
            if (codes[c] == ABS_MT_PRESSURE)
                continue; /* Pressure is optional on other controllers. */
            perror("Read initial touch state");
            close(fd);
            return 1;
        }
        for (int i = 0; i < count; i++) {
            if (c == 0) now[i].id = values[i + 1];
            else if (c == 1) now[i].x = values[i + 1];
            else if (c == 2) now[i].y = values[i + 1];
            else now[i].pressure = values[i + 1];
        }
    }
    printf("Listening for touches (%d slots). Ctrl+C to stop.\n", count);
    report(now, old, count);

    for (;;) {
        ssize_t bytes = read(fd, events, sizeof(events));
        if (bytes < 0 && errno == EINTR)
            continue;
        if (bytes <= 0 || bytes % sizeof(events[0])) {
            if (bytes < 0) perror("Read input");
            else fprintf(stderr, "Input stream ended or returned a partial event\n");
            close(fd);
            return 1;
        }
        for (size_t i = 0; i < (size_t)bytes / sizeof(events[0]); i++) {
            const struct input_event *e = &events[i];
            if (e->type == EV_SYN && e->code == SYN_DROPPED) {
                fprintf(stderr, "Input events lost; restart touch-test to resync\n");
                close(fd);
                return 1;
            }
            if (e->type == EV_SYN && e->code == SYN_REPORT)
                report(now, old, count);
            if (e->type != EV_ABS)
                continue;
            if (e->code == ABS_MT_SLOT)
                slot = e->value;
            if (slot < 0 || slot >= count)
                continue;
            if (e->code == ABS_MT_TRACKING_ID) now[slot].id = e->value;
            else if (e->code == ABS_MT_POSITION_X) now[slot].x = e->value;
            else if (e->code == ABS_MT_POSITION_Y) now[slot].y = e->value;
            else if (e->code == ABS_MT_PRESSURE) now[slot].pressure = e->value;
        }
    }
}
