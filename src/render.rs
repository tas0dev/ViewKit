use crate::catalog::UiComponent;
use crate::component_dsl::ComponentTemplate;
use std::collections::BTreeMap;
use std::sync::OnceLock;

const BG: u32 = 0xFF1B_1E28;
const PANEL: u32 = 0xFF22_2633;
const PANEL_BORDER: u32 = 0xFF4C_556E;
const TITLE: u32 = 0xFFE8_ECF7;
const ROW_A: u32 = 0xFF2B_3040;
const ROW_B: u32 = 0xFF2F_3445;
const ROW_TEXT: u32 = 0xFFCB_D4EA;
const FONT_BDF_PATH: &str = "/System/fonts/ter-u12b.bdf";
const FONT_HEIGHT: usize = 12;
const GLYPH_COUNT: usize = 96; // ASCII 32..127
const ASCII_START: usize = 32;
const ASCII_END: usize = ASCII_START + GLYPH_COUNT;

struct Font {
    glyphs: [[u8; FONT_HEIGHT]; GLYPH_COUNT],
}

impl Font {
    fn fallback() -> Self {
        let mut glyphs = [[0u8; FONT_HEIGHT]; GLYPH_COUNT];
        for (i, glyph) in glyphs.iter_mut().enumerate() {
            let ch = (ASCII_START + i) as u8;
            if ch == b' ' {
                continue;
            }
            glyph[0] = 0xFC;
            glyph[FONT_HEIGHT - 1] = 0xFC;
            for row in glyph.iter_mut().take(FONT_HEIGHT - 1).skip(1) {
                *row = 0x84;
            }
        }
        Self { glyphs }
    }

    fn load() -> Self {
        let Ok(data) = std::fs::read(FONT_BDF_PATH) else {
            return Self::fallback();
        };
        let mut glyphs = [[0u8; FONT_HEIGHT]; GLYPH_COUNT];
        parse_bdf(&data, &mut glyphs);
        Self { glyphs }
    }

    fn glyph(&self, ch: u8) -> &[u8; FONT_HEIGHT] {
        let idx = if (ASCII_START as u8..ASCII_END as u8).contains(&ch) {
            (ch as usize) - ASCII_START
        } else {
            (b'?' as usize) - ASCII_START
        };
        &self.glyphs[idx]
    }
}

fn parse_bdf(data: &[u8], glyphs: &mut [[u8; FONT_HEIGHT]; GLYPH_COUNT]) {
    let Ok(text) = core::str::from_utf8(data) else {
        return;
    };
    let mut encoding: Option<usize> = None;
    let mut in_bitmap = false;
    let mut row = 0usize;

    for line in text.lines() {
        let line = line.trim();
        if let Some(v) = line.strip_prefix("ENCODING ") {
            encoding = v.trim().parse::<usize>().ok();
            in_bitmap = false;
            row = 0;
        } else if line == "BITMAP" {
            in_bitmap = true;
            row = 0;
        } else if line == "ENDCHAR" {
            in_bitmap = false;
            encoding = None;
            row = 0;
        } else if in_bitmap
            && let Some(enc) = encoding
            && (ASCII_START..ASCII_END).contains(&enc)
            && row < FONT_HEIGHT
        {
            let idx = enc - ASCII_START;
            if let Ok(byte) = u8::from_str_radix(line, 16) {
                glyphs[idx][row] = byte;
            }
            row += 1;
        }
    }
}

fn viewkit_font() -> &'static Font {
    static FONT: OnceLock<Font> = OnceLock::new();
    FONT.get_or_init(Font::load)
}

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

    // 実コンポーネントのプレビュー（MVP）
    render_template_previews(&mut px, width, height, templates);
    px
}

fn render_template_previews(
    px: &mut [u32],
    stride: usize,
    height: usize,
    templates: &[ComponentTemplate],
) {
    let preview_x = 182i32;
    let preview_y = 64i32;
    let preview_w = (stride as i32 - preview_x - 24).max(0) as usize;
    let preview_h = (height as i32 - preview_y - 24).max(0) as usize;
    if preview_w < 100 || preview_h < 80 {
        return;
    }
    fill_rect(px, stride, preview_x, preview_y, preview_w, preview_h, 0xFF20_2431);
    stroke_rect(px, stride, preview_x, preview_y, preview_w, preview_h, PANEL_BORDER);
    draw_text(px, stride, preview_x + 8, preview_y + 8, "PREVIEW", TITLE);

    let mut has_card = false;
    let mut has_button = false;
    for t in templates {
        if t.name.eq_ignore_ascii_case("card") {
            has_card = true;
        } else if t.name.eq_ignore_ascii_case("button") {
            has_button = true;
        }
    }
    if has_card {
        if let Some(card) = templates.iter().find(|t| t.name.eq_ignore_ascii_case("card")) {
            draw_card_preview_from_template(px, stride, preview_x + 10, preview_y + 26, card);
        }
    }
    if has_button {
        if let Some(button) = templates
            .iter()
            .find(|t| t.name.eq_ignore_ascii_case("button"))
        {
            draw_button_preview_from_template(px, stride, preview_x + 14, preview_y + 150, button);
        }
    }
}

#[derive(Clone, Copy)]
struct Shadow {
    dx: i32,
    dy: i32,
    blur: i32,
    spread: i32,
    color: u32,
}

#[derive(Default, Clone)]
struct Style {
    left: i32,
    top: i32,
    width: i32,
    height: i32,
    background: Option<u32>,
    color: Option<u32>,
    border_radius: i32,
    border_width: i32,
    border_color: Option<u32>,
    box_shadow: Option<Shadow>,
}

fn draw_card_preview_from_template(
    px: &mut [u32],
    stride: usize,
    x: i32,
    y: i32,
    template: &ComponentTemplate,
) {
    let vars = parse_css_vars(template);
    let card_bg = style_for_selector(template, ".vk-card-bg", &vars);
    let content = style_for_selector(template, ".vk-card-content-area", &vars);
    let title = style_for_selector(template, ".vk-card-title", &vars);
    let body = style_for_selector(template, ".vk-card-body", &vars);

    draw_style_box(px, stride, x + card_bg.left, y + card_bg.top, &card_bg);
    draw_text(
        px,
        stride,
        x + content.left + 12,
        y + content.top + 12,
        "Card Title",
        title.color.unwrap_or(0xFF1D_1D1F),
    );
    draw_text(
        px,
        stride,
        x + content.left + 12,
        y + content.top + 30,
        "Body text",
        body.color.unwrap_or(0xFF42_4245),
    );
}

fn draw_button_preview_from_template(
    px: &mut [u32],
    stride: usize,
    x: i32,
    y: i32,
    template: &ComponentTemplate,
) {
    let vars = parse_css_vars(template);
    let bg = style_for_selector(template, ".vk-button-bg", &vars);
    let content = style_for_selector(template, ".vk-button-content-area", &vars);
    let label = style_for_selector(template, ".vk-button-label", &vars);

    draw_style_box(px, stride, x + bg.left, y + bg.top, &bg);
    draw_text(
        px,
        stride,
        x + content.left + 8,
        y + content.top + 8,
        "Primary",
        label.color.unwrap_or(0xFF1D_1D1F),
    );
}

fn draw_style_box(px: &mut [u32], stride: usize, x: i32, y: i32, style: &Style) {
    if style.width <= 0 || style.height <= 0 {
        return;
    }
    let w = style.width as usize;
    let h = style.height as usize;
    let radius = style.border_radius.max(0) as usize;
    if let Some(shadow) = style.box_shadow {
        let shadow_layers = shadow.blur.max(2).min(8) as usize;
        for i in 0..shadow_layers {
            let alpha = ((shadow.color >> 24) & 0xFF) as u8;
            let decay = ((i * 18) as u8).min(alpha);
            let layer_color = (((alpha - decay) as u32) << 24) | (shadow.color & 0x00FF_FFFF);
            stroke_rounded_rect(
                px,
                stride,
                x + shadow.dx - i as i32 + shadow.spread,
                y + shadow.dy - i as i32 + shadow.spread,
                w + i * 2,
                h + i * 2,
                radius + i,
                layer_color,
            );
        }
    }
    if let Some(bg) = style.background {
        fill_rounded_rect(px, stride, x, y, w, h, radius, bg);
    }
    if style.border_width > 0 {
        if let Some(border) = style.border_color {
            stroke_rounded_rect(px, stride, x, y, w, h, radius, border);
        }
    }
}

fn parse_css_vars(template: &ComponentTemplate) -> BTreeMap<String, String> {
    let mut vars = BTreeMap::new();
    for d in template.declarations {
        if d.selector == ":root" && d.property.starts_with("--") {
            vars.insert(d.property.to_string(), d.value.to_string());
        }
    }
    vars
}

fn style_for_selector(
    template: &ComponentTemplate,
    selector: &str,
    vars: &BTreeMap<String, String>,
) -> Style {
    let mut style = Style::default();
    for d in template.declarations {
        if d.selector.trim() != selector {
            continue;
        }
        let value = resolve_vars(d.value, vars);
        match d.property {
            "left" => style.left = parse_px(&value).unwrap_or(style.left),
            "top" => style.top = parse_px(&value).unwrap_or(style.top),
            "width" => style.width = parse_px(&value).unwrap_or(style.width),
            "height" => style.height = parse_px(&value).unwrap_or(style.height),
            "background" | "background-color" => style.background = parse_color(&value),
            "color" => style.color = parse_color(&value),
            "border-radius" => style.border_radius = parse_px(&value).unwrap_or(style.border_radius),
            "box-shadow" => style.box_shadow = parse_box_shadow(&value),
            "border" => {
                let (w, c) = parse_border(&value);
                style.border_width = w;
                style.border_color = c;
            }
            _ => {}
        }
    }
    style
}

fn resolve_vars(value: &str, vars: &BTreeMap<String, String>) -> String {
    let s = value.trim();
    if let Some(inner) = s.strip_prefix("var(").and_then(|v| v.strip_suffix(')')) {
        let key = inner.trim();
        if let Some(v) = vars.get(key) {
            return v.clone();
        }
    }
    s.to_string()
}

fn parse_px(v: &str) -> Option<i32> {
    let n = v.trim().strip_suffix("px").unwrap_or(v.trim());
    n.parse::<f32>().ok().map(|f| f.round() as i32)
}

fn parse_border(v: &str) -> (i32, Option<u32>) {
    let mut width = 0;
    let mut color = None;
    for part in v.split_whitespace() {
        if part.ends_with("px") {
            width = parse_px(part).unwrap_or(0);
        } else if color.is_none() {
            color = parse_color(part);
        }
    }
    (width, color)
}

fn parse_box_shadow(v: &str) -> Option<Shadow> {
    // ex: 0px 0px 5px 1px rgba(0, 0, 0, 0.25)
    let parts: Vec<&str> = v.split_whitespace().collect();
    if parts.len() < 5 {
        return None;
    }
    let color_start = v.find("rgb").unwrap_or(v.len());
    let nums = &v[..color_start];
    let mut it = nums
        .split_whitespace()
        .filter_map(parse_px);
    let dx = it.next().unwrap_or(0);
    let dy = it.next().unwrap_or(0);
    let blur = it.next().unwrap_or(0);
    let spread = it.next().unwrap_or(0);
    let color = parse_color(&v[color_start..]).unwrap_or(0x3300_0000);
    Some(Shadow {
        dx,
        dy,
        blur,
        spread,
        color,
    })
}

fn parse_color(v: &str) -> Option<u32> {
    let s = v.trim();
    if let Some(hex) = s.strip_prefix('#') {
        match hex.len() {
            6 => {
                let rgb = u32::from_str_radix(hex, 16).ok()?;
                return Some(0xFF00_0000 | rgb);
            }
            3 => {
                let r = u32::from_str_radix(&hex[0..1], 16).ok()? * 17;
                let g = u32::from_str_radix(&hex[1..2], 16).ok()? * 17;
                let b = u32::from_str_radix(&hex[2..3], 16).ok()? * 17;
                return Some(0xFF00_0000 | (r << 16) | (g << 8) | b);
            }
            _ => {}
        }
    }
    if s.starts_with("rgba(") && s.ends_with(')') {
        let inner = &s[5..s.len() - 1];
        let vals: Vec<&str> = inner.split(',').map(|p| p.trim()).collect();
        if vals.len() == 4 {
            let r = vals[0].parse::<u32>().ok()?;
            let g = vals[1].parse::<u32>().ok()?;
            let b = vals[2].parse::<u32>().ok()?;
            let a = vals[3].parse::<f32>().ok()?.clamp(0.0, 1.0);
            let aa = (a * 255.0).round() as u32;
            return Some((aa << 24) | (r << 16) | (g << 8) | b);
        }
    }
    if s.starts_with("rgb(") && s.ends_with(')') {
        let inner = &s[4..s.len() - 1];
        let vals: Vec<&str> = inner.split(',').map(|p| p.trim()).collect();
        if vals.len() == 3 {
            let r = vals[0].parse::<u32>().ok()?;
            let g = vals[1].parse::<u32>().ok()?;
            let b = vals[2].parse::<u32>().ok()?;
            return Some(0xFF00_0000 | (r << 16) | (g << 8) | b);
        }
    }
    None
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

fn fill_rounded_rect(
    px: &mut [u32],
    stride: usize,
    x: i32,
    y: i32,
    w: usize,
    h: usize,
    radius: usize,
    color: u32,
) {
    if w == 0 || h == 0 {
        return;
    }
    let r = radius.min(w / 2).min(h / 2);
    for yy in 0..h as i32 {
        for xx in 0..w as i32 {
            if inside_rounded_rect(xx, yy, w as i32, h as i32, r as i32) {
                put(px, stride, x + xx, y + yy, color);
            }
        }
    }
}

fn stroke_rounded_rect(
    px: &mut [u32],
    stride: usize,
    x: i32,
    y: i32,
    w: usize,
    h: usize,
    radius: usize,
    color: u32,
) {
    if w < 2 || h < 2 {
        return;
    }
    let r = radius.min(w / 2).min(h / 2) as i32;
    for yy in 0..h as i32 {
        for xx in 0..w as i32 {
            let outer = inside_rounded_rect(xx, yy, w as i32, h as i32, r);
            let inner = inside_rounded_rect(xx - 1, yy - 1, w as i32 - 2, h as i32 - 2, (r - 1).max(0));
            if outer && !inner {
                put(px, stride, x + xx, y + yy, color);
            }
        }
    }
}

fn draw_shadow_rounded_rect(
    px: &mut [u32],
    stride: usize,
    x: i32,
    y: i32,
    w: usize,
    h: usize,
    radius: usize,
    shadow: u32,
) {
    let a = ((shadow >> 24) & 0xFF) as u8;
    let rgb = shadow & 0x00FF_FFFF;
    for dy in 1..=3 {
        let alpha = a.saturating_sub((dy * 20) as u8);
        if alpha == 0 {
            continue;
        }
        stroke_rounded_rect(
            px,
            stride,
            x - dy,
            y - dy,
            w + (dy as usize * 2),
            h + (dy as usize * 2),
            radius + dy as usize,
            ((alpha as u32) << 24) | rgb,
        );
    }
}

fn draw_text(px: &mut [u32], stride: usize, x: i32, y: i32, text: &str, color: u32) {
    let font = viewkit_font();
    let mut pen_x = x;
    for ch in text.bytes() {
        draw_char(px, stride, pen_x, y, ch, color, font);
        pen_x += 9;
    }
}

fn draw_char(px: &mut [u32], stride: usize, x: i32, y: i32, ch: u8, color: u32, font: &Font) {
    let g = font.glyph(ch);
    for (row, bits) in g.iter().enumerate() {
        for col in 0..8 {
            if (bits >> (7 - col)) & 1 == 1 {
                let px_x = x + col as i32;
                let px_y = y + row as i32;
                put_alpha(px, stride, px_x, px_y, color, 220);
                put_alpha(px, stride, px_x + 1, px_y, color, 72);
                put_alpha(px, stride, px_x - 1, px_y, color, 72);
                put_alpha(px, stride, px_x, px_y + 1, color, 72);
                put_alpha(px, stride, px_x, px_y - 1, color, 72);
            }
        }
    }
}

fn put_alpha(px: &mut [u32], stride: usize, x: i32, y: i32, color: u32, alpha: u8) {
    if x < 0 || y < 0 || alpha == 0 {
        return;
    }
    let x = x as usize;
    let y = y as usize;
    let h = px.len() / stride;
    if x >= stride || y >= h {
        return;
    }
    let idx = y * stride + x;
    let dst = px[idx];
    px[idx] = blend_argb(dst, color, alpha);
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

fn blend_argb(dst: u32, src: u32, alpha: u8) -> u32 {
    let a = alpha as u32;
    let inv = 255 - a;
    let sr = (src >> 16) & 0xFF;
    let sg = (src >> 8) & 0xFF;
    let sb = src & 0xFF;
    let dr = (dst >> 16) & 0xFF;
    let dg = (dst >> 8) & 0xFF;
    let db = dst & 0xFF;
    let r = (sr * a + dr * inv) / 255;
    let g = (sg * a + dg * inv) / 255;
    let b = (sb * a + db * inv) / 255;
    0xFF00_0000 | (r << 16) | (g << 8) | b
}

fn inside_rounded_rect(xx: i32, yy: i32, w: i32, h: i32, r: i32) -> bool {
    if xx < 0 || yy < 0 || xx >= w || yy >= h {
        return false;
    }
    if r <= 0 {
        return true;
    }
    if xx >= r && xx < w - r {
        return true;
    }
    if yy >= r && yy < h - r {
        return true;
    }
    let cx = if xx < r { r - 1 } else { w - r };
    let cy = if yy < r { r - 1 } else { h - r };
    let dx = xx - cx;
    let dy = yy - cy;
    dx * dx + dy * dy <= r * r
}
