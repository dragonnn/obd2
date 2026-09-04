// SPDX-License-Identifier: GPL-2.0-only
// Decode a PNG/JPEG and copy it to /dev/fb0 without scaling.

#include <errno.h>
#include <fcntl.h>
#include <stddef.h>
#include <stdio.h>
#include <jpeglib.h>
#include <linux/fb.h>
#include <png.h>
#include <setjmp.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <strings.h>
#include <sys/ioctl.h>
#include <sys/mman.h>
#include <unistd.h>

struct image {
    unsigned width;
    unsigned height;
    unsigned char *rgba;
};

static void image_free(struct image *image)
{
    free(image->rgba);
    image->rgba = NULL;
}

static int load_png(const char *path, struct image *image)
{
    FILE *file = NULL;
    png_structp png = NULL;
    png_infop info = NULL;
    png_bytep *rows = NULL;
    png_uint_32 width, height;
    int bit_depth, color_type, interlace, compression, filter;
    size_t row_bytes, total_bytes;
    unsigned y;
    int result = -1;

    file = fopen(path, "rb");
    if (!file) {
        perror(path);
        return -1;
    }
    png = png_create_read_struct(PNG_LIBPNG_VER_STRING, NULL, NULL, NULL);
    if (!png)
        goto out;
    info = png_create_info_struct(png);
    if (!info)
        goto out;
    if (setjmp(png_jmpbuf(png)))
        goto out;
    png_init_io(png, file);
    png_read_info(png, info);
    png_get_IHDR(png, info, &width, &height, &bit_depth, &color_type,
                 &interlace, &compression, &filter);
    if (width == 0 || height == 0 || width > UINT32_MAX || height > UINT32_MAX)
        goto out;

    if (bit_depth == 16)
        png_set_strip_16(png);
    if (color_type == PNG_COLOR_TYPE_PALETTE)
        png_set_palette_to_rgb(png);
    if (color_type == PNG_COLOR_TYPE_GRAY && bit_depth < 8)
        png_set_expand_gray_1_2_4_to_8(png);
    if (png_get_valid(png, info, PNG_INFO_tRNS))
        png_set_tRNS_to_alpha(png);
    if (color_type == PNG_COLOR_TYPE_GRAY || color_type == PNG_COLOR_TYPE_GRAY_ALPHA)
        png_set_gray_to_rgb(png);
    if (!(color_type & PNG_COLOR_MASK_ALPHA) &&
        !png_get_valid(png, info, PNG_INFO_tRNS))
        png_set_add_alpha(png, 0xff, PNG_FILLER_AFTER);
    png_read_update_info(png, info);
    row_bytes = png_get_rowbytes(png, info);
    if (row_bytes < (size_t)width * 4 ||
        (size_t)height > SIZE_MAX / ((size_t)width * 4))
        goto out;
    total_bytes = (size_t)width * height * 4;
    image->rgba = malloc(total_bytes);
    rows = malloc((size_t)height * sizeof(*rows));
    if (!image->rgba || !rows)
        goto out;
    for (y = 0; y < height; y++)
        rows[y] = image->rgba + (size_t)y * width * 4;
    png_read_image(png, rows);
    png_read_end(png, NULL);
    image->width = (unsigned)width;
    image->height = (unsigned)height;
    result = 0;
out:
    free(rows);
    if (result < 0)
        image_free(image);
    if (png)
        png_destroy_read_struct(&png, &info, NULL);
    fclose(file);
    return result;
}

struct jpeg_error {
    struct jpeg_error_mgr base;
    jmp_buf jump;
};

static void jpeg_fail(j_common_ptr common)
{
    struct jpeg_error *error = (struct jpeg_error *)common->err;
    longjmp(error->jump, 1);
}

static int load_jpeg(const char *path, struct image *image)
{
    FILE *file = fopen(path, "rb");
    struct jpeg_decompress_struct jpeg;
    struct jpeg_error error;
    unsigned y;
    int result = -1;

    if (!file) {
        perror(path);
        return -1;
    }
    memset(&jpeg, 0, sizeof(jpeg));
    jpeg.err = jpeg_std_error(&error.base);
    error.base.error_exit = jpeg_fail;
    if (setjmp(error.jump))
        goto out_destroy;
    jpeg_create_decompress(&jpeg);
    jpeg_stdio_src(&jpeg, file);
    jpeg_read_header(&jpeg, TRUE);
    jpeg.out_color_space = JCS_RGB;
    jpeg_start_decompress(&jpeg);
    if (!jpeg.output_width || !jpeg.output_height ||
        jpeg.output_width > UINT32_MAX || jpeg.output_height > UINT32_MAX ||
        (size_t)jpeg.output_width > SIZE_MAX / ((size_t)jpeg.output_height * 4))
        goto out_finish;
    image->width = jpeg.output_width;
    image->height = jpeg.output_height;
    image->rgba = malloc((size_t)image->width * image->height * 4);
    if (!image->rgba)
        goto out_finish;
    while (jpeg.output_scanline < jpeg.output_height) {
        JSAMPROW row = image->rgba +
            (size_t)jpeg.output_scanline * image->width * 4;
        if (jpeg_read_scanlines(&jpeg, &row, 1) != 1)
            goto out_free;
        for (y = image->width; y-- > 0;) {
            row[y * 4 + 3] = 0xff;
            row[y * 4 + 2] = row[y * 3 + 2];
            row[y * 4 + 1] = row[y * 3 + 1];
            row[y * 4 + 0] = row[y * 3 + 0];
        }
    }
    result = 0;
    goto out_finish;
out_free:
    image_free(image);
out_finish:
    jpeg_finish_decompress(&jpeg);
out_destroy:
    jpeg_destroy_decompress(&jpeg);
    fclose(file);
    return result;
}

static int load_image(const char *path, struct image *image)
{
    const char *extension = strrchr(path, '.');
    if (extension && (!strcasecmp(extension, ".png")))
        return load_png(path, image);
    if (extension && (!strcasecmp(extension, ".jpg") || !strcasecmp(extension, ".jpeg")))
        return load_jpeg(path, image);
    fprintf(stderr, "unsupported image extension (use .png, .jpg, or .jpeg)\n");
    return -1;
}

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

static void put_pixel(unsigned char *p, const struct fb_var_screeninfo *var,
                      unsigned char r, unsigned char g, unsigned char b)
{
    uint32_t pixel = channel(r, var->red) | channel(g, var->green) |
                     channel(b, var->blue);
    if (var->bits_per_pixel == 16) {
        uint16_t value = (uint16_t)pixel;
        memcpy(p, &value, sizeof(value));
    } else if (var->bits_per_pixel == 24) {
        p[0] = (unsigned char)pixel;
        p[1] = (unsigned char)(pixel >> 8);
        p[2] = (unsigned char)(pixel >> 16);
    } else {
        uint32_t value = pixel;
        memcpy(p, &value, sizeof(value));
    }
}

int main(int argc, char **argv)
{
    struct image image = {0};
    struct fb_fix_screeninfo fix;
    struct fb_var_screeninfo var;
    int fd;
    void *map;
    size_t bytes;

    if (argc != 2) {
        fprintf(stderr, "usage: %s image.png|image.jpg\n", argv[0]);
        return 2;
    }
    if (load_image(argv[1], &image) < 0)
        return 1;

    fd = open("/dev/fb0", O_RDWR);
    if (fd < 0) {
        perror("/dev/fb0");
        image_free(&image);
        return 1;
    }
    if (ioctl(fd, FBIOGET_FSCREENINFO, &fix) < 0 ||
        ioctl(fd, FBIOGET_VSCREENINFO, &var) < 0) {
        perror("framebuffer info");
        close(fd);
        image_free(&image);
        return 1;
    }
    if (var.bits_per_pixel != 16 && var.bits_per_pixel != 24 &&
        var.bits_per_pixel != 32) {
        fprintf(stderr, "unsupported framebuffer depth: %u\n", var.bits_per_pixel);
        close(fd);
        image_free(&image);
        return 1;
    }
    /* fbcon=rotate:3 makes the visible screen yres by xres. */
    unsigned display_width = var.yres;
    unsigned display_height = var.xres;
    if (image.width > display_width || image.height > display_height)
        fprintf(stderr, "warning: image clipped to visible framebuffer (%ux%u)\n",
                display_width, display_height);

    bytes = fix.smem_len;
    map = mmap(NULL, bytes, PROT_READ | PROT_WRITE, MAP_SHARED, fd, 0);
    if (map == MAP_FAILED) {
        perror("mmap framebuffer");
        close(fd);
        image_free(&image);
        return 1;
    }
    for (unsigned sy = 0; sy < image.height && sy < display_height; sy++) {
        for (unsigned sx = 0; sx < image.width && sx < display_width; sx++) {
            unsigned raw_x = display_height - 1 - sy;
            unsigned raw_y = sx;
            unsigned char *p = (unsigned char *)map +
                (size_t)(raw_y + var.yoffset) * fix.line_length +
                (size_t)(raw_x + var.xoffset) * var.bits_per_pixel / 8;
            unsigned char *pixel = image.rgba + ((size_t)sy * image.width + sx) * 4;
            put_pixel(p, &var, pixel[0], pixel[1], pixel[2]);
        }
    }
    munmap(map, bytes);
    close(fd);
    image_free(&image);
    return 0;
}
