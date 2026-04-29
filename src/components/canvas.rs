use crate::render::{clear, Color};

/// Simple Canvas component: fills its layout area with a solid color.
/// Intended to be used with the ui_layout crate for placement; currently
/// the component exposes a layout() method that callers can use when
/// integrating with a layout engine.
pub struct Canvas {
    color: Color,
    pref_w: i32,
    pref_h: i32,
}

impl Canvas {
    pub fn new(color: Color, pref_w: i32, pref_h: i32) -> Self {
        Canvas { color, pref_w, pref_h }
    }

    pub fn set_color(&mut self, color: Color) { self.color = color; }

    /// A minimal layout pass: given available space, return the size the
    /// canvas will occupy. Replace or call ui_layout here when integrating.
    pub fn layout(&self, avail_w: i32, avail_h: i32) -> (i32,i32) {
        let w = if self.pref_w <= 0 { avail_w } else { self.pref_w.min(avail_w) };
        let h = if self.pref_h <= 0 { avail_h } else { self.pref_h.min(avail_h) };
        (w.max(0), h.max(0))
    }

    /// Render the canvas into the provided buffer (BGRA / XRGB layout).
    /// width/height/stride are the buffer dimensions as used throughout ViewKit.
    pub fn render(&self, buf: &mut [u8], width: usize, height: usize, stride: usize) {
        clear(buf, width, height, stride, self.color);
    }
}
