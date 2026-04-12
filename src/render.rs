use crate::catalog::UiComponent;
use crate::component_dsl::ComponentTemplate;

const BG: u32 = 0xFF1B_1E28;
const PANEL: u32 = 0xFF22_2633;
const PANEL_BORDER: u32 = 0xFF4C_556E;
const TITLE: u32 = 0xFFE8_ECF7;
const ROW_A: u32 = 0xFF2B_3040;
const ROW_B: u32 = 0xFF2F_3445;
const ROW_TEXT: u32 = 0xFFCB_D4EA;

pub fn render_component_catalog(width: usize, height: usize, components: &[UiComponent]) -> Vec<u32> {
    let mut px = vec![BG; width * height];
    fill_rect(&mut px, width, 12, 12, width.saturating_sub(24), height.saturating_sub(24), PANEL);
    stroke_rect(
        &mut px,
        width,
        12,
        12,
        width.saturating_sub(24),
        height.saturating_sub(24),
        PANEL_BORDER,
    );

    draw_text(&mut px, width, 24, 24, "VIEWKIT COMPONENT CATALOG", TITLE);

    let mut y = 48i32;
    for (idx, c) in components.iter().enumerate() {
        let row = if idx & 1 == 0 { ROW_A } else { ROW_B };
        fill_rect(&mut px, width, 24, y, width.saturating_sub(48), 14, row);
        draw_text(&mut px, width, 30, y + 4, c.name(), ROW_TEXT);
        y += 16;
        if y + 12 >= height as i32 {
            break;
        }
    }

    px
}

pub fn render_template_catalog(
    width: usize,
    height: usize,
    templates: &[ComponentTemplate],
) -> Vec<u32> {
    let mut px = vec![BG; width * height];
    fill_rect(&mut px, width, 12, 12, width.saturating_sub(24), height.saturating_sub(24), PANEL);
    stroke_rect(
        &mut px,
        width,
        12,
        12,
        width.saturating_sub(24),
        height.saturating_sub(24),
        PANEL_BORDER,
    );

    draw_text(&mut px, width, 24, 24, "VIEWKIT HTMX/CSS TEMPLATES", TITLE);
    let mut y = 48i32;
    for (idx, t) in templates.iter().enumerate() {
        let row = if idx & 1 == 0 { ROW_A } else { ROW_B };
        fill_rect(&mut px, width, 24, y, width.saturating_sub(48), 14, row);
        draw_text(&mut px, width, 30, y + 4, t.name, ROW_TEXT);
        draw_text(&mut px, width, 150, y + 4, t.root_tag, ROW_TEXT);
        y += 16;
        if y + 12 >= height as i32 {
            break;
        }
    }
    px
}

fn fill_rect(px: &mut [u32], stride: usize, x: i32, y: i32, w: usize, h: usize, color: u32) {
    if w == 0 || h == 0 {
        return;
    }
    let height = px.len() / stride;
    let x0 = x.max(0) as usize;
    let y0 = y.max(0) as usize;
    let x1 = (x + w as i32).max(0) as usize;
    let y1 = (y + h as i32).max(0) as usize;
    let x1 = x1.min(stride);
    let y1 = y1.min(height);
    for yy in y0..y1 {
        let row = yy * stride;
        for xx in x0..x1 {
            px[row + xx] = color;
        }
    }
}

fn stroke_rect(px: &mut [u32], stride: usize, x: i32, y: i32, w: usize, h: usize, color: u32) {
    if w < 2 || h < 2 {
        return;
    }
    fill_rect(px, stride, x, y, w, 1, color);
    fill_rect(px, stride, x, y + h as i32 - 1, w, 1, color);
    fill_rect(px, stride, x, y, 1, h, color);
    fill_rect(px, stride, x + w as i32 - 1, y, 1, h, color);
}

fn draw_text(px: &mut [u32], stride: usize, x: i32, y: i32, text: &str, color: u32) {
    let mut pen_x = x;
    for ch in text.bytes() {
        draw_char(px, stride, pen_x, y, ch, color);
        pen_x += 6;
    }
}

fn draw_char(px: &mut [u32], stride: usize, x: i32, y: i32, ch: u8, color: u32) {
    let g = glyph(ch);
    for (row, bits) in g.iter().enumerate() {
        for col in 0..5 {
            if (bits >> (4 - col)) & 1 == 1 {
                put(px, stride, x + col as i32, y + row as i32, color);
            }
        }
    }
}

fn put(px: &mut [u32], stride: usize, x: i32, y: i32, color: u32) {
    if x < 0 || y < 0 {
        return;
    }
    let x = x as usize;
    let y = y as usize;
    let h = px.len() / stride;
    if x >= stride || y >= h {
        return;
    }
    px[y * stride + x] = color;
}

fn glyph(ch: u8) -> [u8; 7] {
    match ch {
        b'A' => [0x0E, 0x11, 0x11, 0x1F, 0x11, 0x11, 0x11],
        b'B' => [0x1E, 0x11, 0x11, 0x1E, 0x11, 0x11, 0x1E],
        b'C' => [0x0E, 0x11, 0x10, 0x10, 0x10, 0x11, 0x0E],
        b'D' => [0x1E, 0x11, 0x11, 0x11, 0x11, 0x11, 0x1E],
        b'E' => [0x1F, 0x10, 0x10, 0x1E, 0x10, 0x10, 0x1F],
        b'F' => [0x1F, 0x10, 0x10, 0x1E, 0x10, 0x10, 0x10],
        b'G' => [0x0E, 0x11, 0x10, 0x17, 0x11, 0x11, 0x0E],
        b'H' => [0x11, 0x11, 0x11, 0x1F, 0x11, 0x11, 0x11],
        b'I' => [0x1F, 0x04, 0x04, 0x04, 0x04, 0x04, 0x1F],
        b'K' => [0x11, 0x12, 0x14, 0x18, 0x14, 0x12, 0x11],
        b'L' => [0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x1F],
        b'M' => [0x11, 0x1B, 0x15, 0x15, 0x11, 0x11, 0x11],
        b'N' => [0x11, 0x11, 0x19, 0x15, 0x13, 0x11, 0x11],
        b'O' => [0x0E, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E],
        b'P' => [0x1E, 0x11, 0x11, 0x1E, 0x10, 0x10, 0x10],
        b'R' => [0x1E, 0x11, 0x11, 0x1E, 0x14, 0x12, 0x11],
        b'S' => [0x0E, 0x11, 0x10, 0x0E, 0x01, 0x11, 0x0E],
        b'T' => [0x1F, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04],
        b'U' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E],
        b'V' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x0A, 0x04],
        b'W' => [0x11, 0x11, 0x11, 0x15, 0x15, 0x15, 0x0A],
        b'X' => [0x11, 0x11, 0x0A, 0x04, 0x0A, 0x11, 0x11],
        b'Y' => [0x11, 0x11, 0x0A, 0x04, 0x04, 0x04, 0x04],
        b' ' => [0; 7],
        _ => [0x1F, 0x01, 0x02, 0x04, 0x08, 0x00, 0x08],
    }
}
