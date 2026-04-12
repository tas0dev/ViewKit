use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use cssparser::{Parser, ParserInput, ToCss};
use html5ever::{parse_document, tendril::TendrilSink};
use markup5ever_rcdom::{Handle, NodeData, RcDom};
use toml::Value;

const BG: u32 = 0xFF1B_1E28;
const PANEL: u32 = 0xFF22_2633;
const PANEL_BORDER: u32 = 0xFF4C_556E;
const TITLE: u32 = 0xFFE8_ECF7;
const ROW_TEXT: u32 = 0xFFCB_D4EA;
const FONT_W: i32 = 6;
const FONT_H: i32 = 7;

#[derive(Debug, Clone)]
struct CssDeclBuild {
    selector: String,
    property: String,
    value: String,
}

#[derive(Debug, Clone)]
struct ThemeNode {
    tag: String,
    classes: Vec<String>,
    text: String,
    children: Vec<ThemeNode>,
}

#[derive(Debug, Clone)]
struct ThemeComponent {
    name: String,
    root: ThemeNode,
    declarations: Vec<CssDeclBuild>,
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
    position_relative: bool,
    position_absolute: bool,
    display_flex: bool,
    justify_center: bool,
    align_center: bool,
    left: i32,
    top: i32,
    width: i32,
    height: i32,
    padding: i32,
    margin_bottom: i32,
    line_height: i32,
    background: Option<u32>,
    color: Option<u32>,
    border_radius: i32,
    border_width: i32,
    border_color: Option<u32>,
    box_shadow: Option<Shadow>,
}

pub fn build_runtime_theme_frame(
    width: usize,
    height: usize,
    theme_root: &str,
) -> Option<Vec<u32>> {
    let components = load_theme_components(theme_root)?;
    if components.is_empty() {
        return None;
    }

    let mut px = vec![BG; width * height];
    fill_rect(
        &mut px,
        width,
        12,
        12,
        width.saturating_sub(24),
        height.saturating_sub(24),
        PANEL,
    );
    stroke_rect(
        &mut px,
        width,
        12,
        12,
        width.saturating_sub(24),
        height.saturating_sub(24),
        PANEL_BORDER,
    );
    draw_text(
        &mut px,
        width,
        24,
        24,
        "VIEWKIT RUNTIME THEME RENDER",
        TITLE,
    );

    let mut y = 56i32;
    for comp in components.iter() {
        draw_text(
            &mut px,
            width,
            24,
            y - 10,
            comp.name.as_str(),
            ROW_TEXT,
        );
        let preview_w = 160usize;
        let preview_h = 120usize;
        fill_rect(&mut px, width, 24, y, preview_w, preview_h, 0xFF202532);
        stroke_rect(&mut px, width, 24, y, preview_w, preview_h, PANEL_BORDER);
        render_component_into(
            &mut px,
            width,
            &comp,
            24,
            y,
            preview_w as i32,
            preview_h as i32,
        );
        y += preview_h as i32 + 18;
        if y + preview_h as i32 > height as i32 - 16 {
            break;
        }
    }

    Some(px)
}

fn load_theme_components(theme_root: &str) -> Option<Vec<ThemeComponent>> {
    let root = PathBuf::from(theme_root);
    if !root.is_dir() {
        return None;
    }
    let index_toml_path = root.join("index.toml");
    let common_css_path = root.join("common.css");
    let common_css = fs::read_to_string(&common_css_path).unwrap_or_default();
    let common_decls = parse_css_declarations(&common_css);
    let component_names = parse_component_index(&index_toml_path);
    if component_names.is_empty() {
        return None;
    }

    let mut out = Vec::new();
    for name in component_names {
        let comp_dir = root.join(&name);
        let html_path = comp_dir.join("index.html");
        let css_path = comp_dir.join("style.css");
        if !html_path.is_file() {
            continue;
        }
        let html = fs::read_to_string(&html_path).unwrap_or_default();
        let css = fs::read_to_string(&css_path).unwrap_or_default();
        let root_node = match parse_theme_root_node(&html) {
            Some(v) => v,
            None => continue,
        };
        let mut declarations = common_decls.clone();
        declarations.extend(parse_css_declarations(&css));
        out.push(ThemeComponent {
            name,
            root: root_node,
            declarations,
        });
    }
    Some(out)
}

fn parse_component_index(index_toml_path: &Path) -> Vec<String> {
    let text = match fs::read_to_string(index_toml_path) {
        Ok(t) => t,
        Err(_) => return Vec::new(),
    };
    let value: Value = match text.parse() {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    let arr = match value.get("components").and_then(|v| v.as_array()) {
        Some(a) => a,
        None => return Vec::new(),
    };
    let mut out = Vec::new();
    for item in arr {
        if let Some(name) = item.as_str() {
            let n = name.trim();
            if !n.is_empty() {
                out.push(n.to_string());
            }
        }
    }
    out
}

fn parse_theme_root_node(input: &str) -> Option<ThemeNode> {
    let dom: RcDom = parse_document(RcDom::default(), Default::default()).one(input);
    let handle = find_first_renderable_element(&dom.document)?;
    build_node(&handle)
}

fn find_first_renderable_element(handle: &Handle) -> Option<Handle> {
    if let NodeData::Element { name, .. } = &handle.data {
        let tag = name.local.to_string();
        if tag != "html" && tag != "head" && tag != "body" {
            return Some(handle.clone());
        }
    }
    for child in handle.children.borrow().iter() {
        if let Some(found) = find_first_renderable_element(child) {
            return Some(found);
        }
    }
    None
}

fn build_node(handle: &Handle) -> Option<ThemeNode> {
    let NodeData::Element { name, attrs, .. } = &handle.data else {
        return None;
    };
    let tag = name.local.to_string();
    let mut classes = Vec::new();
    for attr in attrs.borrow().iter() {
        if attr.name.local.as_ref() == "class" {
            for c in attr.value.as_ref().split_whitespace() {
                if !c.is_empty() {
                    classes.push(c.to_string());
                }
            }
        }
    }

    let mut text = String::new();
    let mut children = Vec::new();
    for child in handle.children.borrow().iter() {
        match &child.data {
            NodeData::Text { contents } => {
                let t = contents.borrow().trim().to_string();
                if !t.is_empty() {
                    if !text.is_empty() {
                        text.push(' ');
                    }
                    text.push_str(&t);
                }
            }
            NodeData::Element { .. } => {
                if let Some(n) = build_node(child) {
                    children.push(n);
                }
            }
            _ => {}
        }
    }

    Some(ThemeNode {
        tag,
        classes,
        text,
        children,
    })
}

fn parse_css_declarations(css: &str) -> Vec<CssDeclBuild> {
    let mut out = Vec::new();
    for block in css.split('}') {
        let Some((selector_raw, body)) = block.split_once('{') else {
            continue;
        };
        let selector = selector_raw.trim();
        if selector.is_empty() {
            continue;
        }
        for decl in body.split(';') {
            let Some((property_raw, value_raw)) = decl.split_once(':') else {
                continue;
            };
            let property = property_raw.trim();
            if property.is_empty() {
                continue;
            }
            let value_trimmed = value_raw.trim().to_string();
            if value_trimmed.is_empty() {
                continue;
            }
            {
                let mut input = ParserInput::new(value_trimmed.as_str());
                let mut parser = Parser::new(&mut input);
                while let Ok(token) = parser.next_including_whitespace_and_comments() {
                    let _ = token.to_css_string();
                }
            }
            out.push(CssDeclBuild {
                selector: selector.to_string(),
                property: property.to_string(),
                value: value_trimmed,
            });
        }
    }
    out
}

fn render_component_into(
    px: &mut [u32],
    stride: usize,
    comp: &ThemeComponent,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
) {
    let vars = parse_css_vars(&comp.declarations);
    let root_style = style_for_node(&comp.root, &comp.declarations, &vars);
    let root_w = if root_style.width > 0 { root_style.width } else { w };
    let root_h = if root_style.height > 0 {
        root_style.height
    } else {
        h
    };
    draw_node(
        px,
        stride,
        &comp.root,
        &comp.declarations,
        &vars,
        x + root_style.left,
        y + root_style.top,
        root_w,
        root_h,
    );
}

fn draw_node(
    px: &mut [u32],
    stride: usize,
    node: &ThemeNode,
    decls: &[CssDeclBuild],
    vars: &BTreeMap<String, String>,
    x: i32,
    y: i32,
    parent_w: i32,
    parent_h: i32,
) {
    let style = style_for_node(node, decls, vars);
    let w = if style.width > 0 { style.width } else { parent_w };
    let auto_text_h = if !node.text.is_empty() {
        style.line_height.max(FONT_H) + style.padding * 2
    } else {
        0
    };
    let h = if style.height > 0 {
        style.height
    } else if auto_text_h > 0 {
        auto_text_h
    } else {
        parent_h
    };
    let origin_x = if style.position_absolute {
        x + style.left
    } else {
        x + style.left
    };
    let origin_y = if style.position_absolute {
        y + style.top
    } else {
        y + style.top
    };

    if w > 0 && h > 0 {
        draw_style_box(px, stride, origin_x, origin_y, w, h, &style);
    }
    let content_x = origin_x + style.padding.max(0);
    let content_y = origin_y + style.padding.max(0);
    let content_w = (w - style.padding * 2).max(0);
    let content_h = (h - style.padding * 2).max(0);

    if !node.text.is_empty() {
        let mut tx = content_x;
        let mut ty = content_y;
        if style.display_flex && style.justify_center {
            let tw = estimate_text_width(node.text.as_str());
            tx = content_x + (content_w - tw).max(0) / 2;
        }
        if style.display_flex && style.align_center {
            ty = content_y + (content_h - style.line_height.max(FONT_H)).max(0) / 2;
        }
        draw_text(
            px,
            stride,
            tx,
            ty,
            node.text.as_str(),
            style.color.unwrap_or(0xFF1D_1D1F),
        );
    }

    let mut flow_y = content_y;
    for child in &node.children {
        let child_style = style_for_node(child, decls, vars);
        if child_style.position_absolute {
            draw_node(
                px,
                stride,
                child,
                decls,
                vars,
                origin_x,
                origin_y,
                w,
                h,
            );
            continue;
        }
        let mut child_x = content_x;
        let mut child_y = flow_y;
        let child_w = if child_style.width > 0 {
            child_style.width
        } else {
            content_w
        };
        let mut child_h = if child_style.height > 0 {
            child_style.height
        } else if !child.text.is_empty() {
            child_style.line_height.max(FONT_H) + child_style.padding * 2
        } else {
            content_h
        };
        if style.display_flex {
            if style.justify_center {
                child_x = content_x + (content_w - child_w).max(0) / 2;
            }
            if style.align_center {
                child_y = content_y + (content_h - child_h).max(0) / 2;
            }
        }
        draw_node(
            px,
            stride,
            child,
            decls,
            vars,
            child_x,
            child_y,
            child_w,
            child_h,
        );
        if !style.display_flex {
            if child_h <= 0 {
                child_h = FONT_H;
            }
            flow_y += child_h + child_style.margin_bottom.max(0);
        }
    }
}

fn draw_style_box(px: &mut [u32], stride: usize, x: i32, y: i32, w: i32, h: i32, style: &Style) {
    if w <= 0 || h <= 0 {
        return;
    }
    let w = w as usize;
    let h = h as usize;
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

fn style_for_node(
    node: &ThemeNode,
    decls: &[CssDeclBuild],
    vars: &BTreeMap<String, String>,
) -> Style {
    let mut style = Style::default();
    for d in decls {
        if !selector_matches(&d.selector, node) {
            continue;
        }
        let value = resolve_vars(d.value.as_str(), vars);
        match d.property.as_str() {
            "display" => style.display_flex = value == "flex",
            "justify-content" => style.justify_center = value == "center",
            "align-items" => style.align_center = value == "center",
            "position" => {
                style.position_absolute = value == "absolute";
                style.position_relative = value == "relative";
            }
            "left" => style.left = parse_px(&value).unwrap_or(style.left),
            "top" => style.top = parse_px(&value).unwrap_or(style.top),
            "width" => style.width = parse_px(&value).unwrap_or(style.width),
            "height" => style.height = parse_px(&value).unwrap_or(style.height),
            "padding" => style.padding = parse_px(&value).unwrap_or(style.padding),
            "margin-bottom" => {
                style.margin_bottom = parse_px(&value).unwrap_or(style.margin_bottom)
            }
            "line-height" => style.line_height = parse_px(&value).unwrap_or(style.line_height),
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
    if style.line_height <= 0 {
        style.line_height = FONT_H;
    }
    style
}

fn selector_matches(selector: &str, node: &ThemeNode) -> bool {
    let s = selector.trim();
    if s == ":root" {
        return false;
    }
    if s.contains(' ') {
        let last = s.split_whitespace().last().unwrap_or(s);
        return selector_matches(last, node);
    }
    if let Some(class) = s.strip_prefix('.') {
        return node.classes.iter().any(|c| c == class);
    }
    s == node.tag
}

fn parse_css_vars(decls: &[CssDeclBuild]) -> BTreeMap<String, String> {
    let mut vars = BTreeMap::new();
    for d in decls {
        if d.selector == ":root" && d.property.starts_with("--") {
            vars.insert(d.property.clone(), d.value.clone());
        }
    }
    vars
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

fn estimate_text_width(s: &str) -> i32 {
    (s.chars().count() as i32) * FONT_W
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
    let parts: Vec<&str> = v.split_whitespace().collect();
    if parts.len() < 5 {
        return None;
    }
    let color_start = v.find("rgb").unwrap_or(v.len());
    let nums = &v[..color_start];
    let mut it = nums.split_whitespace().filter_map(parse_px);
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
            let inner = inside_rounded_rect(
                xx - 1,
                yy - 1,
                w as i32 - 2,
                h as i32 - 2,
                (r - 1).max(0),
            );
            if outer && !inner {
                put(px, stride, x + xx, y + yy, color);
            }
        }
    }
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
        b'a' => [0x00, 0x0E, 0x01, 0x0F, 0x11, 0x13, 0x0D],
        b'b' => [0x10, 0x10, 0x16, 0x19, 0x11, 0x11, 0x1E],
        b'c' => [0x00, 0x0E, 0x11, 0x10, 0x10, 0x11, 0x0E],
        b'd' => [0x01, 0x01, 0x0D, 0x13, 0x11, 0x11, 0x0F],
        b'e' => [0x00, 0x0E, 0x11, 0x1F, 0x10, 0x11, 0x0E],
        b'i' => [0x04, 0x00, 0x0C, 0x04, 0x04, 0x04, 0x0E],
        b'm' => [0x00, 0x1A, 0x15, 0x15, 0x15, 0x15, 0x15],
        b'n' => [0x00, 0x16, 0x19, 0x11, 0x11, 0x11, 0x11],
        b'o' => [0x00, 0x0E, 0x11, 0x11, 0x11, 0x11, 0x0E],
        b'p' => [0x00, 0x1E, 0x11, 0x1E, 0x10, 0x10, 0x10],
        b'r' => [0x00, 0x16, 0x19, 0x10, 0x10, 0x10, 0x10],
        b't' => [0x04, 0x04, 0x1F, 0x04, 0x04, 0x04, 0x03],
        b'y' => [0x00, 0x11, 0x11, 0x0F, 0x01, 0x11, 0x0E],
        b' ' => [0; 7],
        _ => [0x1F, 0x01, 0x02, 0x04, 0x08, 0x00, 0x08],
    }
}
