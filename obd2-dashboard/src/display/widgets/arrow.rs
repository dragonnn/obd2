use defmt::trace;
use embassy_time::Instant;
use embedded_graphics::{pixelcolor::Gray4, prelude::*, primitives::*};
use num_traits::float::Float;

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum ArrowDirection {
    #[default]
    Forward,
    Reverse,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Arrow {
    size: Size,
    position: Point,
    arrow_width: u32,
    offset: f64,
    old_offest: i32,
    force_update: bool,
    color: u8,
    speed: f64,
    direction: ArrowDirection,
}

impl Arrow {
    pub fn new(position: Point, size: Size, arrow_width: u32, direction: ArrowDirection) -> Self {
        Self {
            position,
            size,
            arrow_width,
            old_offest: i32::MAX,
            force_update: true,
            color: 0,
            offset: 0.0,
            speed: 0.0,
            direction,
        }
    }

    pub fn update_direction(&mut self, direction: ArrowDirection) {
        if self.direction != direction {
            self.direction = direction;
            self.force_update = true;
        }
    }

    pub fn update_speed(&mut self, speed: f64) {
        let old_speed = self.speed;
        if speed > 0.0 {
            self.speed = speed / 100.0 * 3.5 + 1.0;
        } else {
            self.speed = 0.0;
        }
        self.color = (speed / 100.0 * 16.0).round() as u8;
        if speed != 0.0 && self.color == 0 {
            self.color = 1;
        }
        if speed != old_speed {
            self.force_update = true;
        }
        if self.color > 15 {
            self.color = 15;
        }
        if self.speed > 4.5 {
            self.speed = 4.5;
        }
    }

    pub fn draw<D: DrawTarget<Color = Gray4>>(&mut self, target: &mut D) -> Result<(), D::Error> {
        // Fast path: no animation and no forced redraw.
        if !self.force_update && self.speed == 0.0 {
            return Ok(());
        }
        let now = Instant::now();

        let aw_f = self.arrow_width as f64;
        if self.offset >= aw_f {
            // Keep phase continuous (cheaper and visually smoother than resetting to self.speed).
            self.offset -= aw_f;
        }

        // Floor instead of ceil: redraw only after full-pixel movement.
        let new_offest = self.offset as i32;
        if new_offest != self.old_offest || self.force_update {
            let mut draw_size = self.size;
            draw_size.height += 1;
            let h = draw_size.height as i32;
            let half_h = h / 2;
            let aw_i = self.arrow_width as i32;
            let spacing = (aw_f / 1.2).ceil() as i32;
            let tip = aw_i;
            let gap = aw_i / 3;

            let color = Gray4::new(self.color);
            let fill_color = PrimitiveStyleBuilder::new().fill_color(color).build();
            let fill_black = PrimitiveStyleBuilder::new().fill_color(Gray4::BLACK).build();

            // Clear entire area.
            let area_rect = Rectangle::new(self.position, draw_size);
            area_rect.draw_styled(&fill_black, target)?;
            let mut area = target.clipped(&area_rect);

            let is_forward = self.direction == ArrowDirection::Forward;
            let scroll = if is_forward { new_offest } else { -new_offest };
            let width_count = self.size.width as i32 / aw_i;
            let a_start = if is_forward { -1 } else { 0 };
            let a_end = if is_forward { width_count + 2 } else { width_count + 4 };

            trace!(
                "Arrow params: h={=i32} half_h={=i32} tip={=i32} gap={=i32} spacing={=i32}",
                h, half_h, tip, gap, spacing
            );

            // Draw chevrons scanline-by-scanline using horizontal rectangles
            // instead of triangle rasterization + stroke, which is very expensive.
            for y_rel in 0..h {
                // Distance from nearest horizontal edge (top or bottom).
                let dist = if y_rel <= half_h { y_rel } else { h - 1 - y_rel };
                // How far the arrow tip extends at this scanline.
                let dx = if half_h > 0 { (tip * 2 * dist + half_h) / h } else { 0 };
                if dx == 0 {
                    continue;
                }

                let y = self.position.y + y_rel;

                // Log first chevron only
                let base_x_0 = self.position.x + spacing * a_start + scroll;
                let (vx0, vw0) = if is_forward {
                    if dx > gap { (base_x_0 + dx - gap, gap) } else { (base_x_0, dx) }
                } else {
                    if dx > gap { (base_x_0 - dx, gap) } else { (base_x_0 - dx, dx) }
                };
                trace!("y={=i32} dist={=i32} dx={=i32} vx={=i32} vw={=i32}", y_rel, dist, dx, vx0, vw0);

                for a in a_start..a_end {
                    let base_x = self.position.x + spacing * a + scroll;

                    // Compute the visible colored strip after the gap triangle carves
                    // into the colored triangle.
                    let (vx, vw) = if is_forward {
                        // Chevron points right: colored [base, base+dx], black [base-gap, base-gap+dx]
                        if dx > gap { (base_x + dx - gap, gap) } else { (base_x, dx) }
                    } else {
                        // Chevron points left: colored [base-dx, base], black [base-dx+gap, base+gap]
                        if dx > gap { (base_x - dx, gap) } else { (base_x - dx, dx) }
                    };

                    if vw > 0 {
                        Rectangle::new(Point::new(vx, y), Size::new(vw as u32, 1))
                            .draw_styled(&fill_color, &mut area)?;
                    }
                }
            }

            self.old_offest = new_offest;
            self.force_update = false;

            let elapsed_us = now.elapsed().as_micros() as u32;
            trace!("Arrow draw: {=u32},{=u32:03}ms", elapsed_us / 1000, elapsed_us % 1000);
        }

        self.offset += self.speed;
        Ok(())
    }
}
