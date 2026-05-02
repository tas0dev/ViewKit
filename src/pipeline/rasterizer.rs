use super::display_list::{DisplayCommand, DisplayList};
use super::framebuffer::Framebuffer;

pub fn rasterize(display_list: &DisplayList, width: u32, height: u32) -> Framebuffer {
    let mut fb = Framebuffer::new(width, height);
    fb.clear(0xFF111111);

    for item in &display_list.items {
        match item {
            DisplayCommand::FillRect {
                rect,
                color,
                radius,
                opacity,
            } => {
                fb.fill_rounded_rect(rect.x, rect.y, rect.width, rect.height, *radius, *color, *opacity);
            }
            DisplayCommand::DrawText {
                x,
                y,
                color,
                opacity,
                text,
            } => {
                rasterize_text(&mut fb, *x, *y, *color, *opacity, text);
            }
        }
    }

    fb
}

fn rasterize_text(fb: &mut Framebuffer, x: i32, y: i32, color: u32, opacity: f32, text: &str) {
    // Minimal text stub: draw one 6x10 block per character.
    let mut pen_x = x;
    for _ in text.chars() {
        fb.fill_rect(pen_x, y, 6, 10, color, opacity);
        pen_x += 8;
    }
}
