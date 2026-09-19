// Copyright © SixtyFPS GmbH <info@slint.dev>
// SPDX-License-Identifier: GPL-3.0-only OR LicenseRef-Slint-Royalty-free-2.0 OR LicenseRef-Slint-Software-3.0

//! Slint software rendering with Rockchip RGA presentation.

use std::cell::{Cell, RefCell};
use std::os::fd::{AsRawFd, OwnedFd};
use std::ptr::NonNull;
use std::rc::Rc;

use i_slint_core::api::PhysicalSize as PhysicalWindowSize;
use i_slint_core::platform::PlatformError;
use i_slint_core::renderer::DrawOutcome;
use i_slint_renderer_software::{
    PremultipliedRgbaColor, RepaintBufferType, SoftwareRenderer, TargetPixel,
};

use crate::display::RenderingRotation;

enum NativeRgaSurface {}

unsafe extern "C" {
    fn slint_rga_framebuffer_size(fd: i32, width: *mut u32, height: *mut u32) -> i32;
    fn slint_rga_surface_create(
        framebuffer_fd: i32,
        source_width: u32,
        source_height: u32,
    ) -> *mut NativeRgaSurface;
    fn slint_rga_surface_pixels(surface: *mut NativeRgaSurface) -> *mut core::ffi::c_void;
    fn slint_rga_surface_begin_cpu(surface: *mut NativeRgaSurface) -> i32;
    fn slint_rga_surface_present(surface: *mut NativeRgaSurface, rotation: i32) -> i32;
    fn slint_rga_surface_destroy(surface: *mut NativeRgaSurface);
}

#[repr(transparent)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Xrgb8888Pixel(u32);

impl From<Xrgb8888Pixel> for PremultipliedRgbaColor {
    #[inline]
    fn from(pixel: Xrgb8888Pixel) -> Self {
        PremultipliedRgbaColor {
            red: (pixel.0 >> 16) as u8,
            green: (pixel.0 >> 8) as u8,
            blue: pixel.0 as u8,
            alpha: (pixel.0 >> 24) as u8,
        }
    }
}

impl From<PremultipliedRgbaColor> for Xrgb8888Pixel {
    #[inline]
    fn from(pixel: PremultipliedRgbaColor) -> Self {
        Self(
            (u32::from(pixel.alpha) << 24)
                | (u32::from(pixel.red) << 16)
                | (u32::from(pixel.green) << 8)
                | u32::from(pixel.blue),
        )
    }
}

impl TargetPixel for Xrgb8888Pixel {
    fn blend(&mut self, color: PremultipliedRgbaColor) {
        let mut destination = PremultipliedRgbaColor::from(*self);
        destination.blend(color);
        *self = destination.into();
    }

    fn from_rgb(red: u8, green: u8, blue: u8) -> Self {
        Self(0xff00_0000 | (u32::from(red) << 16) | (u32::from(green) << 8) | u32::from(blue))
    }

    fn background() -> Self {
        Self(0)
    }
}

struct RgaSurface {
    native: NonNull<NativeRgaSurface>,
    width: u32,
    height: u32,
}

impl RgaSurface {
    fn new(framebuffer_fd: i32, width: u32, height: u32) -> Result<Self, PlatformError> {
        let native =
            NonNull::new(unsafe { slint_rga_surface_create(framebuffer_fd, width, height) })
                .ok_or_else(|| {
                    PlatformError::Other("Could not allocate the RGA render surface".into())
                })?;
        Ok(Self {
            native,
            width,
            height,
        })
    }

    fn pixels(&mut self) -> Result<&mut [Xrgb8888Pixel], PlatformError> {
        let pixels = unsafe { slint_rga_surface_pixels(self.native.as_ptr()) };
        if pixels.is_null() {
            return Err(PlatformError::Other(
                "RGA render surface has no pixel mapping".into(),
            ));
        }
        Ok(unsafe {
            std::slice::from_raw_parts_mut(
                pixels.cast::<Xrgb8888Pixel>(),
                self.width as usize * self.height as usize,
            )
        })
    }

    fn begin_cpu(&self) {
        if unsafe { slint_rga_surface_begin_cpu(self.native.as_ptr()) } < 0 {
            eprintln!("Slint RGA: DMA buffer CPU synchronization failed");
        }
    }

    fn present(&self, rotation: RenderingRotation) -> Result<(), PlatformError> {
        let rotation = match rotation {
            RenderingRotation::NoRotation => 0,
            RenderingRotation::Rotate90 => 1 << 0,
            RenderingRotation::Rotate180 => 1 << 1,
            RenderingRotation::Rotate270 => 1 << 2,
        };
        let result = unsafe { slint_rga_surface_present(self.native.as_ptr(), rotation) };
        if result < 0 {
            Err(PlatformError::Other(
                "RGA and CPU presentation both failed".into(),
            ))
        } else {
            Ok(())
        }
    }
}

impl Drop for RgaSurface {
    fn drop(&mut self) {
        unsafe { slint_rga_surface_destroy(self.native.as_ptr()) }
    }
}

pub struct RgaRendererAdapter {
    renderer: SoftwareRenderer,
    framebuffer: Rc<OwnedFd>,
    surface: RefCell<Option<RgaSurface>>,
    first_frame: Cell<bool>,
    size: PhysicalWindowSize,
}

impl RgaRendererAdapter {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        device_opener: &crate::DeviceOpener,
        _requested_graphics_api: Option<&i_slint_core::graphics::RequestedGraphicsAPI>,
    ) -> Result<Box<dyn crate::fullscreenwindowadapter::FullscreenRenderer>, PlatformError> {
        let mut errors = Vec::new();
        for framebuffer_number in 0..10 {
            let path = format!("/dev/fb{framebuffer_number}");
            let framebuffer = match device_opener(std::path::Path::new(&path)) {
                Ok(framebuffer) => framebuffer,
                Err(error) => {
                    errors.push(format!("{path}: {error}"));
                    continue;
                }
            };

            let mut width = 0;
            let mut height = 0;
            if unsafe {
                slint_rga_framebuffer_size(framebuffer.as_raw_fd(), &mut width, &mut height)
            } == 0
            {
                eprintln!("Using software renderer with Rockchip RGA presentation");
                return Ok(Box::new(Self {
                    renderer: SoftwareRenderer::new(),
                    framebuffer,
                    surface: RefCell::new(None),
                    first_frame: Cell::new(true),
                    size: PhysicalWindowSize::new(width, height),
                }));
            }
            errors.push(format!("{path}: unsupported framebuffer layout"));
        }

        Err(PlatformError::Other(format!(
            "Could not open an XRGB8888 framebuffer for RGA presentation: {}",
            errors.join(", ")
        )))
    }

    fn source_size(&self, rotation: RenderingRotation) -> (u32, u32) {
        match rotation {
            RenderingRotation::Rotate90 | RenderingRotation::Rotate270 => {
                (self.size.height, self.size.width)
            }
            RenderingRotation::NoRotation | RenderingRotation::Rotate180 => {
                (self.size.width, self.size.height)
            }
        }
    }
}

impl crate::fullscreenwindowadapter::FullscreenRenderer for RgaRendererAdapter {
    fn as_core_renderer(&self) -> &dyn i_slint_core::renderer::Renderer {
        &self.renderer
    }

    fn render_and_present(
        &self,
        rotation: RenderingRotation,
        _draw_mouse_cursor_callback: &dyn Fn(&mut dyn i_slint_core::item_rendering::ItemRenderer),
    ) -> Result<DrawOutcome, PlatformError> {
        let (source_width, source_height) = self.source_size(rotation);
        let mut surface = self.surface.borrow_mut();
        if surface
            .as_ref()
            .is_none_or(|surface| surface.width != source_width || surface.height != source_height)
        {
            *surface = Some(RgaSurface::new(
                self.framebuffer.as_raw_fd(),
                source_width,
                source_height,
            )?);
            self.first_frame.set(true);
        }
        let surface = surface.as_mut().unwrap();

        self.renderer
            .set_rendering_rotation(i_slint_renderer_software::RenderingRotation::NoRotation);
        self.renderer
            .set_repaint_buffer_type(if self.first_frame.replace(false) {
                RepaintBufferType::NewBuffer
            } else {
                RepaintBufferType::ReusedBuffer
            });

        surface.begin_cpu();
        self.renderer
            .render(surface.pixels()?, source_width as usize);
        surface.present(rotation)?;
        Ok(DrawOutcome::Success)
    }

    fn size(&self) -> PhysicalWindowSize {
        self.size
    }
}
