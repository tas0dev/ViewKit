use super::layout::{LayoutNode, LayoutNodeKind, LayoutTree, Rect};

#[derive(Debug, Clone)]
pub struct DisplayList {
    pub items: Vec<DisplayCommand>,
}

#[derive(Debug, Clone)]
pub enum DisplayCommand {
    FillRect { rect: Rect, color: u32 },
    DrawText { x: i32, y: i32, color: u32, text: String },
}

pub fn build(layout: &LayoutTree) -> DisplayList {
    let mut items = Vec::new();
    build_for_node(&layout.root, &mut items, 0xFFFFFFFF);
    DisplayList { items }
}

fn build_for_node(node: &LayoutNode, out: &mut Vec<DisplayCommand>, inherited_text_color: u32) {
    match &node.kind {
        LayoutNodeKind::Element { .. } => {
            if let Some(color) = background_color_from_styles(&node.styles) {
                out.push(DisplayCommand::FillRect {
                    rect: node.rect,
                    color,
                });
            }
        }
        LayoutNodeKind::Text { content } => {
            let color = text_color_from_styles(&node.styles).unwrap_or(inherited_text_color);
            out.push(DisplayCommand::DrawText {
                x: node.rect.x,
                y: node.rect.y,
                color,
                text: content.clone(),
            });
        }
    }

    let next_text_color = text_color_from_styles(&node.styles).unwrap_or(inherited_text_color);
    for child in &node.children {
        build_for_node(child, out, next_text_color);
    }
}

fn background_color_from_styles(styles: &std::collections::BTreeMap<String, String>) -> Option<u32> {
    if let Some(v) = styles.get("background-color") {
        return parse_css_color(v);
    }
    if let Some(v) = styles.get("background") {
        for token in v.split_whitespace() {
            if let Some(c) = parse_css_color(token) {
                return Some(c);
            }
        }
    }
    None
}

fn text_color_from_styles(styles: &std::collections::BTreeMap<String, String>) -> Option<u32> {
    styles.get("color").and_then(|v| parse_css_color(v))
}

fn parse_css_color(raw: &str) -> Option<u32> {
    let s = raw.trim().to_ascii_lowercase();
    match s.as_str() {
        "white" => Some(0xFFFFFFFF),
        "black" => Some(0xFF000000),
        "red" => Some(0xFFFF0000),
        "green" => Some(0xFF00FF00),
        "blue" => Some(0xFF0000FF),
        _ => parse_hex_color(&s),
    }
}

fn parse_hex_color(s: &str) -> Option<u32> {
    let hex = s.strip_prefix('#')?;
    match hex.len() {
        3 => {
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
            Some(0xFF00_0000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32))
        }
        6 => {
            let rgb = u32::from_str_radix(hex, 16).ok()?;
            Some(0xFF00_0000 | rgb)
        }
        8 => u32::from_str_radix(hex, 16).ok(),
        _ => None,
    }
}
