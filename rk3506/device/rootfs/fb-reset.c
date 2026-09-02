// SPDX-License-Identifier: GPL-2.0-only
// Clear the legacy framebuffer and return the active VT to text mode.

#include <fcntl.h>
#include <linux/fb.h>
#include <linux/kd.h>
#include <linux/vt.h>
#include <stdio.h>
#include <string.h>
#include <sys/ioctl.h>
#include <sys/mman.h>
#include <unistd.h>

int main(void)
{
    struct fb_fix_screeninfo fix;
    int fb = -1;
    int console = -1;
    int tty = -1;
    void *map = MAP_FAILED;
    struct vt_stat state;
    char tty_path[32];
    int result = 1;

    fb = open("/dev/fb0", O_RDWR);
    if (fb < 0) {
        perror("/dev/fb0");
        goto out;
    }
    if (ioctl(fb, FBIOGET_FSCREENINFO, &fix) < 0) {
        perror("framebuffer info");
        goto out;
    }
    map = mmap(NULL, fix.smem_len, PROT_READ | PROT_WRITE, MAP_SHARED, fb, 0);
    if (map == MAP_FAILED) {
        perror("mmap framebuffer");
        goto out;
    }

    memset(map, 0, fix.smem_len);

    console = open("/dev/console", O_RDONLY);
    if (console < 0 || ioctl(console, VT_GETSTATE, &state) < 0) {
        perror("active VT");
        goto out;
    }
    snprintf(tty_path, sizeof(tty_path), "/dev/tty%u", state.v_active);
    tty = open(tty_path, O_RDWR);
    if (tty < 0) {
        perror(tty_path);
        goto out;
    }
    if (ioctl(tty, KDSETMODE, KD_TEXT) < 0) {
        perror("KDSETMODE KD_TEXT");
        goto out;
    }

    /* Clear any cursor/text remnants after leaving graphics mode. */
    {
        static const char clear_sequence[] = "\033[2J\033[H\033[?25h";
        (void)write(tty, clear_sequence, sizeof(clear_sequence) - 1);
    }
    result = 0;

out:
    if (map != MAP_FAILED)
        munmap(map, fix.smem_len);
    if (tty >= 0)
        close(tty);
    if (console >= 0)
        close(console);
    if (fb >= 0)
        close(fb);
    return result;
}
