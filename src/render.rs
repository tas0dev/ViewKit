// 簡易レンダラ: 生の BGRA/ARGB バッファに直接描くユーティリティ

pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color { r, g, b, a }
    }
}

/// バッファ全体を塗りつぶす。バッファは BGRA（B,G,R,A）フォーマットを想定する。
pub fn clear(buf: &mut [u8], width: usize, height: usize, stride: usize, color: Color) {
    for y in 0..height {
        let row = y * stride;
        for x in 0..width {
            let off = row + x * 4;
            buf[off + 0] = color.b;
            buf[off + 1] = color.g;
            buf[off + 2] = color.r;
            buf[off + 3] = color.a;
        }
    }
}

/// 軸揃え矩形を描く。クリッピングは最小限処理する。（後々ui_kayoutにする）
pub fn draw_rect(buf: &mut [u8], width: usize, height: usize, stride: usize, x: i32, y: i32, w: i32, h: i32, color: Color) {
    if w <= 0 || h <= 0 { return; }
    let x0 = x.max(0) as usize;
    let y0 = y.max(0) as usize;
    let x1 = (x + w).min(width as i32) as usize;
    let y1 = (y + h).min(height as i32) as usize;
    if x0 >= x1 || y0 >= y1 { return; }
    for yy in y0..y1 {
        let row = yy * stride;
        for xx in x0..x1 {
            let off = row + xx * 4;
            buf[off + 0] = color.b;
            buf[off + 1] = color.g;
            buf[off + 2] = color.r;
            buf[off + 3] = color.a;
        }
    }
}
