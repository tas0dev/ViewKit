use crate::render::{Color};

/// Card component with CSS-like configurable properties.
/// Defaults match requested CSS except positioning: width:150px; height:150px; background:#FFFFFF; border-radius:20px
/// By default the Card participates in layout (not absolute). Use set_position_absolute() to enable absolute placement.
pub struct Card {
    width: i32,
    height: i32,
    bg: Color,
    radius: i32,
    absolute: bool,
    left: i32,
    top: i32,
}

impl Card {
    /// Create a card
    pub fn new(w: i32, h:i32, color: Color) -> Self {
        Self {
            width: w,
            height: h,
            bg: color,
            radius: 20,
            absolute: false,
            left: 0,
            top: 0,
        }
    }

    /// Chainable setter: size
    pub fn set_size(mut self, w: i32, h: i32) -> Self {
        self.width = w; self.height = h; self
    }

    /// Chainable setter: background color
    pub fn set_bg(mut self, c: Color) -> Self { self.bg = c; self }

    /// Chainable setter: border radius
    pub fn set_radius(mut self, r: i32) -> Self { self.radius = r; self }

    /// Enable absolute positioning and set left/top
    pub fn set_position_absolute(mut self, left: i32, top: i32) -> Self { self.absolute = true; self.left = left; self.top = top; self }

    /// Disable absolute positioning (use layout allocation)
    pub fn unset_absolute(mut self) -> Self { self.absolute = false; self }

    fn draw_rounded_rect(&self, buf: &mut [u8], buf_w: usize, buf_h: usize, stride: usize, x: i32, y: i32, w: i32, h: i32, color: Color, radius: i32) {
        if w <= 0 || h <= 0 { return; }
        let x0 = x.max(0) as usize;
        let y0 = y.max(0) as usize;
        let x1 = (x + w).min(buf_w as i32) as usize;
        let y1 = (y + h).min(buf_h as i32) as usize;
        if x0 >= x1 || y0 >= y1 { return; }

        let r = radius.max(0) as f32;
        for yy in y0..y1 {
            for xx in x0..x1 {
                let local_x = (xx as i32 - x) as f32 + 0.5;
                let local_y = (yy as i32 - y) as f32 + 0.5;
                // distance to nearest corner region
                let dx = if local_x < r { r - local_x } else if local_x > (w as f32 - r) { local_x - (w as f32 - r) } else { 0.0 };
                let dy = if local_y < r { r - local_y } else if local_y > (h as f32 - r) { local_y - (h as f32 - r) } else { 0.0 };
                let dist = (dx*dx + dy*dy).sqrt();

                // coverage: 1.0 inside shape, 0.0 outside. Smooth transition around the boundary (~1px)
                let coverage: f32;
                if dx == 0.0 && dy == 0.0 {
                    coverage = 1.0;
                } else {
                    // distance from circle boundary: negative when inside
                    let boundary_dist = r - dist;
                    // smoothstep over approx 1.0 pixel band
                    let t = (boundary_dist + 0.5).clamp(0.0, 1.0);
                    coverage = t;
                }

                if coverage <= 0.0 { continue; }

                let row = yy * stride;
                let off = row + xx * 4;
                if off + 3 >= buf.len() { continue; }

                // alpha blending: src over dst
                let src_a = ((color.a as f32) * coverage).round() as u8;
                if src_a == 255 {
                    // opaque write
                    buf[off + 0] = color.b;
                    buf[off + 1] = color.g;
                    buf[off + 2] = color.r;
                    buf[off + 3] = color.a;
                } else {
                    let dst_b = buf[off + 0] as u32;
                    let dst_g = buf[off + 1] as u32;
                    let dst_r = buf[off + 2] as u32;
                    let dst_a = buf[off + 3] as u32;

                    let sa = src_a as u32;
                    let inv_sa = 255 - sa;

                    let out_b = (sa * (color.b as u32) + inv_sa * dst_b) / 255;
                    let out_g = (sa * (color.g as u32) + inv_sa * dst_g) / 255;
                    let out_r = (sa * (color.r as u32) + inv_sa * dst_r) / 255;
                    let out_a = (sa + (inv_sa * dst_a) / 255).min(255);

                    buf[off + 0] = out_b as u8;
                    buf[off + 1] = out_g as u8;
                    buf[off + 2] = out_r as u8;
                    buf[off + 3] = out_a as u8;
                }
            }
        }
    }
}

impl crate::components::Component for Card {
    fn pref_size(&self) -> (Option<i32>, Option<i32>) {
        (Some(self.width), Some(self.height))
    }

    fn render_into(&self, buf: &mut [u8], buf_w: usize, buf_h: usize, stride: usize, x: i32, y: i32, w: i32, h: i32) {
        // Determine drawing origin
        let (cx, cy, draw_w, draw_h) = if self.absolute {
            (self.left, self.top, self.width, self.height)
        } else {
            if w <= 0 || h <= 0 { return; }
            let draw_w = if w < self.width { w } else { self.width };
            let draw_h = if h < self.height { h } else { self.height };
            let cx = if w > draw_w { x + (w - draw_w) / 2 } else { x };
            let cy = if h > draw_h { y + (h - draw_h) / 2 } else { y };
            (cx, cy, draw_w, draw_h)
        };

        self.draw_rounded_rect(buf, buf_w, buf_h, stride, cx, cy, draw_w, draw_h, self.bg, self.radius);
    }
}
