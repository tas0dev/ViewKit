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

    pub fn set_pixel(&mut self, x: i32, y: i32, color: u32) {
        if x < 0 || y < 0 {
            return;
        }
        let x = x as u32;
        let y = y as u32;
        if x >= self.width || y >= self.height {
            return;
        }

        let index = (y * self.width + x) as usize;
        self.pixels[index] = color;
    }

    pub fn fill_rect(&mut self, x: i32, y: i32, width: i32, height: i32, color: u32) {
        if width <= 0 || height <= 0 {
            return;
        }
        for yy in y..(y + height) {
            for xx in x..(x + width) {
                self.set_pixel(xx, yy, color);
            }
        }
    }
}
