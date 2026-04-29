use std::fs;
use std::path::PathBuf;
use std::collections::HashMap;
use crate::render::Color;

pub struct TextRenderer {
    font: fontdue::Font,
    cache: HashMap<(char,u32),(fontdue::Metrics, Vec<u8>)>,
}

impl TextRenderer {
    pub fn new() -> Result<Self, String> {
        let path = PathBuf::new();
        let base = path.join("resources");
        let candidates = [
            "NotoSansJP-Regular.ttf",
        ];
        let mut data = None;
        for c in &candidates {
            let p = base.join(c);
            if p.exists() {
                match fs::read(&p) {
                    Ok(b) => { data = Some(b); break; }
                    Err(e) => return Err(format!("failed to read font {}: {}", p.display(), e)),
                }
            }
        }
        if data.is_none() {
            if let Ok(entries) = fs::read_dir(&base) {
                for ent in entries.flatten() {
                    let file_name = ent.file_name().to_string_lossy().to_string();
                    if file_name.to_lowercase().contains("noto") {
                        if let Ok(b) = fs::read(ent.path()) { data = Some(b); break; }
                    }
                }
            }
        }
        let bytes = data.ok_or_else(|| format!("no NotoSans font found in {}", base.display()))?;
        let font = fontdue::Font::from_bytes(bytes.as_slice(), fontdue::FontSettings::default())
            .map_err(|e| format!("fontdue error: {}", e))?;
        Ok(Self { font, cache: HashMap::new() })
    }

    fn get_glyph(&mut self, ch: char, px: f32) -> (fontdue::Metrics, Vec<u8>) {
        let key = (ch, px.to_bits());
        if let Some(v) = self.cache.get(&key) {
            return (v.0.clone(), v.1.clone());
        }
        let (metrics, bitmap) = self.font.rasterize(ch, px);
        let val = (metrics.clone(), bitmap.clone());
        self.cache.insert(key, val.clone());
        val
    }

    /// Draw text (simple, no shaping) at (x,y) top-left in pixels with given font_size (px).
    pub fn draw_text(&mut self, buf: &mut [u8], buf_w: usize, buf_h: usize, stride: usize, mut x: i32, y: i32, font_size: f32, text: &str, color: Color) {
        for ch in text.chars() {
            let (metrics, bitmap) = self.get_glyph(ch, font_size);
            let glyph_x = x + metrics.xmin;
            let glyph_y = y + metrics.ymin;
            // blit
            for gy in 0..metrics.height {
                let dst_y = glyph_y + gy as i32;
                if dst_y < 0 || dst_y as usize >= buf_h { continue; }
                let row = dst_y as usize * stride;
                for gx in 0..metrics.width {
                    let dst_x = glyph_x + gx as i32;
                    if dst_x < 0 || dst_x as usize >= buf_w { continue; }
                    let alpha = bitmap[gy * metrics.width + gx];
                    if alpha == 0 { continue; }
                    let off = row + (dst_x as usize) * 4;
                    if off + 3 >= buf.len() { continue; }
                    // combine alpha with color.a
                    let src_a = (alpha as u32 * color.a as u32) / 255;
                    let inv_sa = 255 - src_a;
                    let dst_b = buf[off + 0] as u32;
                    let dst_g = buf[off + 1] as u32;
                    let dst_r = buf[off + 2] as u32;
                    let out_b = (src_a * color.b as u32 + inv_sa * dst_b) / 255;
                    let out_g = (src_a * color.g as u32 + inv_sa * dst_g) / 255;
                    let out_r = (src_a * color.r as u32 + inv_sa * dst_r) / 255;
                    let out_a = (src_a + (inv_sa * buf[off + 3] as u32) / 255).min(255);
                    buf[off + 0] = out_b as u8;
                    buf[off + 1] = out_g as u8;
                    buf[off + 2] = out_r as u8;
                    buf[off + 3] = out_a as u8;
                }
            }
            x += metrics.advance_width as i32;
        }
    }

    /// Simple layout + draw with word-wrapping within max_width. y is top of first line.
    pub fn layout_and_draw(&mut self, buf: &mut [u8], buf_w: usize, buf_h: usize, stride: usize, x: i32, mut y: i32, max_width: i32, font_size: f32, text: &str, color: Color) {
        let space_w = {
            let (m,_) = self.get_glyph(' ', font_size);
            m.advance_width as i32
        };
        let mut cursor_x = x;
        for word in text.split_whitespace() {
            // measure word width
            let mut word_w: i32 = 0;
            for ch in word.chars() {
                let (m,_) = self.get_glyph(ch, font_size);
                word_w += m.advance_width as i32;
            }
            if cursor_x != x && cursor_x + word_w > x + max_width {
                // wrap
                y += self.font.horizontal_line_metrics(font_size).map(|lm| lm.new_line_size as i32).unwrap_or((font_size*1.2) as i32);
                cursor_x = x;
            }
            // draw each char
            for ch in word.chars() {
                let (m,_) = self.get_glyph(ch, font_size);
                self.draw_text(buf, buf_w, buf_h, stride, cursor_x + m.xmin, y + m.ymin, font_size, &ch.to_string(), color);
                cursor_x += m.advance_width as i32;
            }
            // add space
            cursor_x += space_w;
        }
    }
}
