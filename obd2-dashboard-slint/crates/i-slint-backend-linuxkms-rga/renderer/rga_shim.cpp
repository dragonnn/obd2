// SPDX-License-Identifier: GPL-3.0-only OR LicenseRef-Slint-Royalty-free-2.0 OR LicenseRef-Slint-Software-3.0

#include <cerrno>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <fcntl.h>
#include <linux/dma-buf.h>
#include <linux/dma-heap.h>
#include <linux/fb.h>
#include <rga/im2d.h>
#include <sys/ioctl.h>
#include <sys/mman.h>
#include <unistd.h>

namespace {

constexpr const char *kCmaHeap = "/dev/dma_heap/linux,cma";
constexpr int kPixelFormat = RK_FORMAT_BGRA_8888;

struct RgaSurface {
    int dma_fd = -1;
    void *pixels = MAP_FAILED;
    size_t pixel_bytes = 0;
    bool pixels_are_mmap = false;
    rga_buffer_handle_t source_handle = 0;
    rga_buffer_handle_t destination_handle = 0;
    rga_buffer_t source = {};
    rga_buffer_t destination = {};
    bool rga_available = false;
    bool reported_rga_success = false;
    bool reported_rga_failure = false;

    void *framebuffer = MAP_FAILED;
    size_t framebuffer_bytes = 0;
    uint32_t framebuffer_stride_pixels = 0;
    uint32_t source_width = 0;
    uint32_t source_height = 0;
    uint32_t destination_width = 0;
    uint32_t destination_height = 0;
};

bool query_framebuffer(int fd, fb_var_screeninfo *variable, fb_fix_screeninfo *fixed) {
    if (ioctl(fd, FBIOGET_VSCREENINFO, variable) < 0 ||
        ioctl(fd, FBIOGET_FSCREENINFO, fixed) < 0) {
        return false;
    }

    return variable->bits_per_pixel == 32 && variable->red.offset == 16 &&
           variable->green.offset == 8 && variable->blue.offset == 0 &&
           fixed->line_length >= variable->xres * 4;
}

bool dma_sync(int fd, uint64_t flags) {
    if (fd < 0) {
        return true;
    }
    dma_buf_sync sync = {};
    sync.flags = flags;
    return ioctl(fd, DMA_BUF_IOCTL_SYNC, &sync) == 0;
}

bool allocate_cma(RgaSurface *surface) {
    const int heap_fd = open(kCmaHeap, O_RDWR | O_CLOEXEC);
    if (heap_fd < 0) {
        std::fprintf(stderr, "Slint RGA: could not open %s (%s); using CPU buffer\n",
                     kCmaHeap, std::strerror(errno));
        return false;
    }

    dma_heap_allocation_data allocation = {};
    allocation.len = surface->pixel_bytes;
    allocation.fd_flags = O_RDWR | O_CLOEXEC;
    const int result = ioctl(heap_fd, DMA_HEAP_IOCTL_ALLOC, &allocation);
    close(heap_fd);
    if (result < 0) {
        std::fprintf(stderr, "Slint RGA: CMA allocation of %zu bytes failed (%s); "
                             "using CPU buffer\n",
                     surface->pixel_bytes, std::strerror(errno));
        return false;
    }

    surface->dma_fd = static_cast<int>(allocation.fd);
    surface->pixels = mmap(nullptr, surface->pixel_bytes, PROT_READ | PROT_WRITE,
                           MAP_SHARED, surface->dma_fd, 0);
    if (surface->pixels == MAP_FAILED) {
        std::fprintf(stderr, "Slint RGA: DMA-BUF mmap failed (%s); using CPU buffer\n",
                     std::strerror(errno));
        close(surface->dma_fd);
        surface->dma_fd = -1;
        return false;
    }
    surface->pixels_are_mmap = true;

    surface->source_handle = importbuffer_fd(surface->dma_fd,
                                              static_cast<int>(surface->pixel_bytes));
    if (surface->source_handle == 0) {
        std::fprintf(stderr, "Slint RGA: DMA-BUF import failed; using CPU presentation\n");
        return true;
    }

    surface->source = wrapbuffer_handle_t(
        surface->source_handle, static_cast<int>(surface->source_width),
        static_cast<int>(surface->source_height), static_cast<int>(surface->source_width),
        static_cast<int>(surface->source_height), kPixelFormat);
    surface->rga_available = true;
    std::fprintf(stderr,
                 "Slint RGA: CMA DMA-BUF ready: %ux%u BGRA8888, %zu bytes, fd %d\n",
                 surface->source_width, surface->source_height, surface->pixel_bytes,
                 surface->dma_fd);
    return true;
}

void cpu_rotate(RgaSurface *surface, int rotation) {
    auto *source = static_cast<const uint32_t *>(surface->pixels);
    auto *destination = static_cast<uint32_t *>(surface->framebuffer);
    const uint32_t source_stride = surface->source_width;
    const uint32_t destination_stride = surface->framebuffer_stride_pixels;

    for (uint32_t y = 0; y < surface->source_height; ++y) {
        for (uint32_t x = 0; x < surface->source_width; ++x) {
            uint32_t destination_x = x;
            uint32_t destination_y = y;
            switch (rotation) {
            case IM_HAL_TRANSFORM_ROT_90:
                destination_x = surface->source_height - 1 - y;
                destination_y = x;
                break;
            case IM_HAL_TRANSFORM_ROT_180:
                destination_x = surface->source_width - 1 - x;
                destination_y = surface->source_height - 1 - y;
                break;
            case IM_HAL_TRANSFORM_ROT_270:
                destination_x = y;
                destination_y = surface->source_width - 1 - x;
                break;
            default:
                break;
            }
            destination[destination_y * destination_stride + destination_x] =
                source[y * source_stride + x];
        }
    }
}

} // namespace

extern "C" int slint_rga_framebuffer_size(int fd, uint32_t *width, uint32_t *height) {
    fb_var_screeninfo variable = {};
    fb_fix_screeninfo fixed = {};
    if (!query_framebuffer(fd, &variable, &fixed)) {
        return -1;
    }
    *width = variable.xres;
    *height = variable.yres;
    return 0;
}

extern "C" RgaSurface *slint_rga_surface_create(int framebuffer_fd, uint32_t source_width,
                                                  uint32_t source_height) {
    fb_var_screeninfo variable = {};
    fb_fix_screeninfo fixed = {};
    if (!query_framebuffer(framebuffer_fd, &variable, &fixed)) {
        return nullptr;
    }

    auto *surface = new RgaSurface;
    surface->source_width = source_width;
    surface->source_height = source_height;
    surface->destination_width = variable.xres;
    surface->destination_height = variable.yres;
    surface->framebuffer_stride_pixels = fixed.line_length / 4;
    surface->pixel_bytes = static_cast<size_t>(source_width) * source_height * 4;
    surface->framebuffer_bytes = static_cast<size_t>(fixed.line_length) * variable.yres;
    surface->framebuffer = mmap(nullptr, surface->framebuffer_bytes,
                                PROT_READ | PROT_WRITE, MAP_SHARED, framebuffer_fd, 0);
    if (surface->framebuffer == MAP_FAILED) {
        delete surface;
        return nullptr;
    }

    if (!allocate_cma(surface)) {
        surface->pixels = std::calloc(1, surface->pixel_bytes);
        if (surface->pixels == nullptr) {
            munmap(surface->framebuffer, surface->framebuffer_bytes);
            delete surface;
            return nullptr;
        }
    }

    if (surface->rga_available) {
        if (fixed.smem_start != 0) {
            surface->destination = wrapbuffer_physicaladdr_t(
                reinterpret_cast<void *>(fixed.smem_start), static_cast<int>(variable.xres),
                static_cast<int>(variable.yres), static_cast<int>(fixed.line_length / 4),
                static_cast<int>(variable.yres), kPixelFormat);
            std::fprintf(stderr,
                         "Slint RGA: framebuffer target: %ux%u BGRA8888, stride %u pixels, "
                         "physical address\n",
                         surface->destination_width, surface->destination_height,
                         surface->framebuffer_stride_pixels);
        } else {
            surface->destination_handle = importbuffer_virtualaddr(
                surface->framebuffer, static_cast<int>(surface->framebuffer_bytes));
            if (surface->destination_handle != 0) {
                surface->destination = wrapbuffer_handle_t(
                    surface->destination_handle, static_cast<int>(variable.xres),
                    static_cast<int>(variable.yres), static_cast<int>(fixed.line_length / 4),
                    static_cast<int>(variable.yres), kPixelFormat);
                std::fprintf(stderr,
                             "Slint RGA: framebuffer target: %ux%u BGRA8888, stride %u pixels, "
                             "imported virtual mapping\n",
                             surface->destination_width, surface->destination_height,
                             surface->framebuffer_stride_pixels);
            } else {
                std::fprintf(stderr,
                             "Slint RGA: framebuffer mapping import failed; using CPU presentation\n");
                surface->rga_available = false;
            }
        }
    }

    return surface;
}

extern "C" void *slint_rga_surface_pixels(RgaSurface *surface) {
    return surface == nullptr ? nullptr : surface->pixels;
}

extern "C" int slint_rga_surface_begin_cpu(RgaSurface *surface) {
    if (surface == nullptr) {
        return -1;
    }
    return dma_sync(surface->dma_fd, DMA_BUF_SYNC_START | DMA_BUF_SYNC_WRITE) ? 0 : -1;
}

extern "C" int slint_rga_surface_present(RgaSurface *surface, int rotation) {
    if (surface == nullptr) {
        return -1;
    }

    dma_sync(surface->dma_fd, DMA_BUF_SYNC_END | DMA_BUF_SYNC_WRITE);
    if (surface->rga_available) {
        const IM_STATUS status = imrotate_t(surface->source, surface->destination, rotation, 1);
        if (status == IM_STATUS_SUCCESS) {
            if (!surface->reported_rga_success) {
                std::fprintf(stderr,
                             "Slint RGA: hardware presentation active: %ux%u -> %ux%u, "
                             "transform 0x%x\n",
                             surface->source_width, surface->source_height,
                             surface->destination_width, surface->destination_height, rotation);
                surface->reported_rga_success = true;
            }
            return 0;
        }
        if (!surface->reported_rga_failure) {
            std::fprintf(stderr,
                         "Slint RGA: hardware presentation failed (%d); using CPU rotation\n",
                         static_cast<int>(status));
            surface->reported_rga_failure = true;
        }
    }

    dma_sync(surface->dma_fd, DMA_BUF_SYNC_START | DMA_BUF_SYNC_READ);
    cpu_rotate(surface, rotation);
    dma_sync(surface->dma_fd, DMA_BUF_SYNC_END | DMA_BUF_SYNC_READ);
    return 1;
}

extern "C" void slint_rga_surface_destroy(RgaSurface *surface) {
    if (surface == nullptr) {
        return;
    }
    if (surface->source_handle != 0) {
        releasebuffer_handle(surface->source_handle);
    }
    if (surface->destination_handle != 0) {
        releasebuffer_handle(surface->destination_handle);
    }
    if (surface->pixels != MAP_FAILED) {
        if (surface->pixels_are_mmap) {
            munmap(surface->pixels, surface->pixel_bytes);
        } else {
            std::free(surface->pixels);
        }
    }
    if (surface->dma_fd >= 0) {
        close(surface->dma_fd);
    }
    if (surface->framebuffer != MAP_FAILED) {
        munmap(surface->framebuffer, surface->framebuffer_bytes);
    }
    delete surface;
}
