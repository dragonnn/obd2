// SPDX-License-Identifier: GPL-2.0-only
// Fill /dev/fb0 with a solid RGB color using the framebuffer's bitfields.

#include <errno.h>
#include <fcntl.h>
#include <linux/fb.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/ioctl.h>
#include <sys/mman.h>
#include <unistd.h>

static uint32_t scale_channel(unsigned value, unsigned length)
{
    if (!length)
        return 0;
    if (length >= 31)
        return value ? ((1u << 31) - 1u) : 0;
    return (value * ((1u << length) - 1u) + 127u) / 255u;
}

static uint32_t channel(unsigned value, struct fb_bitfield field)
{
    return scale_channel(value, field.length) << field.offset;
}

static int color(const char *name, unsigned *r, unsigned *g, unsigned *b)
{
    *r = *g = *b = 0;
    if (!strcmp(name, "red")) *r = 255;
    else if (!strcmp(name, "green")) *g = 255;
    else if (!strcmp(name, "blue")) *b = 255;
    else if (!strcmp(name, "white")) *r = *g = *b = 255;
    else if (!strcmp(name, "black")) return 0;
    else return -1;
    return 0;
}

static int number(const char *text, unsigned *value)
{
    char *end;
    unsigned long parsed;

    errno = 0;
    parsed = strtoul(text, &end, 10);
    if (errno || *text == '\0' || *end != '\0' || parsed > UINT32_MAX)
        return -1;
    *value = (unsigned)parsed;
    return 0;
}

int main(int argc, char **argv)
{
    struct fb_fix_screeninfo fix;
    struct fb_var_screeninfo var;
    unsigned r, g, b, br, bg, bb;
    int fd;
    void *map;
    size_t bytes;
    uint32_t value;
    int border = 0;
    unsigned top = 0, left = 0, right = 0, bottom = 0;
    unsigned display_width, display_height;

    if ((argc != 2 && argc != 3 && argc != 4 && argc != 5 && argc != 8) ||
        argc < 2 ||
        color(argv[1], &r, &g, &b) < 0 ||
        (argc >= 3 && strcmp(argv[2], "border")) ||
        (argc == 4 && color(argv[3], &br, &bg, &bb) < 0) ||
        (argc == 5 && (color(argv[3], &br, &bg, &bb) < 0 ||
                       strcmp(argv[4], "inset"))) ||
        (argc == 8 && (color(argv[3], &br, &bg, &bb) < 0 ||
                       number(argv[4], &top) < 0 ||
                       number(argv[5], &left) < 0 ||
                       number(argv[6], &right) < 0 ||
                       number(argv[7], &bottom) < 0))) {
        fprintf(stderr, "usage: %s red|green|blue|white|black [border [color [inset|top left right bottom]]]\n", argv[0]);
        return 2;
    }
    border = argc == 3;
    if (argc == 4)
        border = 1;
    if (argc == 5) {
        border = 1;
        top = left = right = bottom = 1;
    }
    if (argc == 8) {
        border = 1;
    }
    if (argc < 4)
        br = bg = bb = 0;

    fd = open("/dev/fb0", O_RDWR);
    if (fd < 0) {
        perror("/dev/fb0");
        return 1;
    }
    if (ioctl(fd, FBIOGET_FSCREENINFO, &fix) < 0 ||
        ioctl(fd, FBIOGET_VSCREENINFO, &var) < 0) {
        perror("framebuffer info");
        close(fd);
        return 1;
    }
    if (var.bits_per_pixel != 16 && var.bits_per_pixel != 24 &&
        var.bits_per_pixel != 32) {
        fprintf(stderr, "unsupported framebuffer depth: %u\n", var.bits_per_pixel);
        close(fd);
        return 1;
    }
    /* The board boots with fbcon=rotate:3 (270 degrees counter-clockwise).
     * Interpret custom margins in the displayed orientation, not raw fb0
     * memory coordinates. */
    display_width = var.yres;
    display_height = var.xres;
    if (left + right >= display_width || top + bottom >= display_height) {
        fprintf(stderr, "border margins are larger than the framebuffer\n");
        close(fd);
        return 2;
    }

    bytes = (size_t)fix.smem_len;
    map = mmap(NULL, bytes, PROT_READ | PROT_WRITE, MAP_SHARED, fd, 0);
    if (map == MAP_FAILED) {
        perror("mmap framebuffer");
        close(fd);
        return 1;
    }

    value = channel(r, var.red) | channel(g, var.green) | channel(b, var.blue);
    uint32_t border_value = channel(br, var.red) | channel(bg, var.green) |
        channel(bb, var.blue);
    for (unsigned y = 0; y < var.yres; y++) {
        unsigned char *row = (unsigned char *)map +
            (size_t)(y + var.yoffset) * fix.line_length +
            (size_t)var.xoffset * var.bits_per_pixel / 8;
        for (unsigned x = 0; x < var.xres; x++) {
            unsigned char *p = row + (size_t)x * var.bits_per_pixel / 8;
            unsigned display_x = y;
            unsigned display_y = var.xres - 1 - x;
            unsigned active_right = display_width - 1 - right;
            unsigned active_bottom = display_height - 1 - bottom;
            uint32_t pixel;
            if (display_x < left || display_x > active_right ||
                display_y < top || display_y > active_bottom) {
                /* Keep the requested physical framebuffer margins unused. */
                pixel = 0;
            } else if (border &&
                       (display_x == left || display_y == top ||
                        display_x == active_right || display_y == active_bottom)) {
                pixel = border_value;
            } else {
                pixel = value;
            }
            if (var.bits_per_pixel == 16) {
                uint16_t v = (uint16_t)pixel;
                memcpy(p, &v, sizeof(v));
            } else if (var.bits_per_pixel == 24) {
                p[0] = (unsigned char)pixel;
                p[1] = (unsigned char)(pixel >> 8);
                p[2] = (unsigned char)(pixel >> 16);
            } else {
                uint32_t v = pixel;
                memcpy(p, &v, sizeof(v));
            }
        }
    }

    munmap(map, bytes);
    close(fd);
    return 0;
}
