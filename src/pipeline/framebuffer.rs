#[derive(Debug, Clone)]
pub struct Framebuffer {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u32>,
}

impl Framebuffer {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            pixels: vec![0xFF000000; (width * height) as usize],
        }
    }

    pub fn clear(&mut self, color: u32) {
        for p in &mut self.pixels {
            *p = color;
        }
    }

    pub fn blend_pixel(&mut self, x: i32, y: i32, color: u32, opacity: f32) {
        let Some(index) = self.pixel_index(x, y) else {
            return;
        };
        let dst = self.pixels[index];
        self.pixels[index] = blend_argb_over(dst, color, opacity);
    }

    pub fn fill_rect(&mut self, x: i32, y: i32, width: i32, height: i32, color: u32, opacity: f32) {
        if width <= 0 || height <= 0 {
            return;
        }
        for yy in y..(y + height) {
            for xx in x..(x + width) {
                self.blend_pixel(xx, yy, color, opacity);
            }
        }
    }

    pub fn fill_rounded_rect(
        &mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        radius: i32,
        color: u32,
        opacity: f32,
    ) {
        if width <= 0 || height <= 0 {
            return;
        }
        let r = radius.max(0).min(width / 2).min(height / 2);
        if r == 0 {
            self.fill_rect(x, y, width, height, color, opacity);
            return;
        }

        let rf = r as f32;
        for yy in y..(y + height) {
            for xx in x..(x + width) {
                let lx = (xx - x) as f32;
                let ly = (yy - y) as f32;
                let coverage = rounded_rect_coverage(lx, ly, width as f32, height as f32, rf);
                if coverage > 0.0 {
                    self.blend_pixel(xx, yy, color, opacity * coverage);
                }
            }
        }
    }

    fn pixel_index(&self, x: i32, y: i32) -> Option<usize> {
        if x < 0 || y < 0 {
            return None;
        }
        let x = x as u32;
        let y = y as u32;
        if x >= self.width || y >= self.height {
            return None;
        }
        Some((y * self.width + x) as usize)
    }
}

fn rounded_rect_coverage(px: f32, py: f32, width: f32, height: f32, radius: f32) -> f32 {
    // 4x MSAA pattern
    const OFFSETS: [(f32, f32); 4] = [(0.25, 0.25), (0.75, 0.25), (0.25, 0.75), (0.75, 0.75)];
    let mut inside = 0_u32;
    for (ox, oy) in OFFSETS {
        if is_inside_rounded_rect_at(px + ox, py + oy, width, height, radius) {
            inside += 1;
        }
    }
    inside as f32 / OFFSETS.len() as f32
}

fn is_inside_rounded_rect_at(x: f32, y: f32, w: f32, h: f32, radius: f32) -> bool {

    if x >= radius && x <= (w - radius) {
        return true;
    }
    if y >= radius && y <= (h - radius) {
        return true;
    }

    let tl = (x - radius, y - radius);
    let tr = (x - (w - radius), y - radius);
    let bl = (x - radius, y - (h - radius));
    let br = (x - (w - radius), y - (h - radius));

    let rr = radius * radius;
    (tl.0 * tl.0 + tl.1 * tl.1 <= rr)
        || (tr.0 * tr.0 + tr.1 * tr.1 <= rr)
        || (bl.0 * bl.0 + bl.1 * bl.1 <= rr)
        || (br.0 * br.0 + br.1 * br.1 <= rr)
}

fn blend_argb_over(dst: u32, src: u32, opacity: f32) -> u32 {
    let opacity = opacity.clamp(0.0, 1.0);
    let src_a = ((src >> 24) & 0xff) as f32 / 255.0;
    let a = (src_a * opacity).clamp(0.0, 1.0);
    if a <= 0.0 {
        return dst;
    }

    let dr = ((dst >> 16) & 0xff) as f32;
    let dg = ((dst >> 8) & 0xff) as f32;
    let db = (dst & 0xff) as f32;

    let sr = ((src >> 16) & 0xff) as f32;
    let sg = ((src >> 8) & 0xff) as f32;
    let sb = (src & 0xff) as f32;

    let out_r = (sr * a + dr * (1.0 - a)).round().clamp(0.0, 255.0) as u32;
    let out_g = (sg * a + dg * (1.0 - a)).round().clamp(0.0, 255.0) as u32;
    let out_b = (sb * a + db * (1.0 - a)).round().clamp(0.0, 255.0) as u32;

    0xff00_0000 | (out_r << 16) | (out_g << 8) | out_b
}
