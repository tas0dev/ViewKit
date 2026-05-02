use super::dom::DomNodeKind;
use super::style::{StyleMap, StyledNode, StyledTree};
use ui_layout::{
    Display, FlexDirection, Fragment, ItemFragment, LayoutBoxes, LayoutEngine,
    LayoutNode as UiLayoutNode, Length, SizeStyle, Style as UiStyle,
};

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone)]
pub struct LayoutTree {
    pub root: LayoutNode,
}

#[derive(Debug, Clone)]
pub struct LayoutNode {
    pub rect: Rect,
    pub styles: StyleMap,
    pub kind: LayoutNodeKind,
    pub children: Vec<LayoutNode>,
}

#[derive(Debug, Clone)]
pub enum LayoutNodeKind {
    Element { tag_name: String },
    Text { content: String },
}

#[derive(Debug, Clone)]
struct MetaNode {
    kind: LayoutNodeKind,
    styles: StyleMap,
    children: Vec<MetaNode>,
}

pub fn compute_layout(styled: &StyledTree, width: u32, height: u32) -> LayoutTree {
    let (mut ui_root, meta_root) = build_ui_tree(&styled.root, true);
    LayoutEngine::layout(&mut ui_root, width as f32, height as f32);

    let root = to_layout_tree(&ui_root, &meta_root, (0.0, 0.0));
    LayoutTree { root }
}

fn build_ui_tree(node: &StyledNode, is_root: bool) -> (UiLayoutNode, MetaNode) {
    match &node.node.kind {
        DomNodeKind::Element(el) => {
            let mut style = style_from_css(&node.styles);
            if is_root {
                style.size.width = Length::Percent(100.0);
                style.size.height = Length::Percent(100.0);
            }

            let mut ui_children = Vec::with_capacity(node.children.len());
            let mut meta_children = Vec::with_capacity(node.children.len());
            for child in &node.children {
                let (ui_child, meta_child) = build_ui_tree(child, false);
                ui_children.push(ui_child);
                meta_children.push(meta_child);
            }

            let ui_node = UiLayoutNode::with_children(style, ui_children);
            let meta = MetaNode {
                kind: LayoutNodeKind::Element {
                    tag_name: el.tag_name.clone(),
                },
                styles: node.styles.clone(),
                children: meta_children,
            };
            (ui_node, meta)
        }
        DomNodeKind::Text(text) => {
            let mut ui_node = UiLayoutNode::new(UiStyle {
                display: Display::Inline,
                ..Default::default()
            });
            ui_node.set_fragments(vec![ItemFragment::Fragment(Fragment {
                width: estimate_text_width(text),
                height: estimate_text_height(text),
            })]);

            let meta = MetaNode {
                kind: LayoutNodeKind::Text {
                    content: text.clone(),
                },
                styles: node.styles.clone(),
                children: Vec::new(),
            };
            (ui_node, meta)
        }
    }
}

fn to_layout_tree(ui: &UiLayoutNode, meta: &MetaNode, parent_content_abs: (f32, f32)) -> LayoutNode {
    let (border_x, border_y, border_w, border_h, content_abs) = match &ui.layout_boxes {
        LayoutBoxes::Single(model) => {
            let border_abs_x = parent_content_abs.0 + model.border_box.x;
            let border_abs_y = parent_content_abs.1 + model.border_box.y;
            let content_abs_x = parent_content_abs.0 + model.content_box.x;
            let content_abs_y = parent_content_abs.1 + model.content_box.y;
            (
                border_abs_x,
                border_abs_y,
                model.border_box.width,
                model.border_box.height,
                (content_abs_x, content_abs_y),
            )
        }
        _ => (0.0, 0.0, 0.0, 0.0, parent_content_abs),
    };

    let children = ui
        .children
        .iter()
        .zip(meta.children.iter())
        .map(|(child_ui, child_meta)| to_layout_tree(child_ui, child_meta, content_abs))
        .collect();

    LayoutNode {
        rect: Rect {
            x: border_x.round() as i32,
            y: border_y.round() as i32,
            width: border_w.max(0.0).round() as i32,
            height: border_h.max(0.0).round() as i32,
        },
        styles: meta.styles.clone(),
        kind: meta.kind.clone(),
        children,
    }
}

fn style_from_css(styles: &StyleMap) -> UiStyle {
    let mut style = UiStyle::default();

    if let Some(display) = styles.get("display") {
        style.display = match display.trim() {
            "flex" => Display::Flex {
                flex_direction: parse_flex_direction(styles.get("flex-direction")),
            },
            "inline" => Display::Inline,
            "none" => Display::None,
            _ => Display::Block,
        };
    }

    style.size = SizeStyle {
        width: parse_length(styles.get("width"), Length::Auto),
        height: parse_length(styles.get("height"), Length::Auto),
        min_width: parse_length(styles.get("min-width"), Length::Auto),
        max_width: parse_length(styles.get("max-width"), Length::Auto),
        min_height: parse_length(styles.get("min-height"), Length::Auto),
        max_height: parse_length(styles.get("max-height"), Length::Auto),
    };

    style
}

fn parse_flex_direction(direction: Option<&String>) -> FlexDirection {
    match direction.map(|v| v.trim()) {
        Some("row") => FlexDirection::Row,
        _ => FlexDirection::Column,
    }
}

fn parse_length(value: Option<&String>, default: Length) -> Length {
    let Some(raw) = value else {
        return default;
    };
    let s = raw.trim().to_ascii_lowercase();

    if s == "auto" {
        return Length::Auto;
    }
    if let Some(px) = s.strip_suffix("px") {
        return px
            .trim()
            .parse::<f32>()
            .map(Length::Px)
            .unwrap_or(default);
    }
    if let Some(pct) = s.strip_suffix('%') {
        return pct
            .trim()
            .parse::<f32>()
            .map(Length::Percent)
            .unwrap_or(default);
    }
    if let Some(vw) = s.strip_suffix("vw") {
        return vw
            .trim()
            .parse::<f32>()
            .map(Length::Vw)
            .unwrap_or(default);
    }
    if let Some(vh) = s.strip_suffix("vh") {
        return vh
            .trim()
            .parse::<f32>()
            .map(Length::Vh)
            .unwrap_or(default);
    }

    default
}

fn estimate_text_width(text: &str) -> f32 {
    (text.chars().count() as f32 * 8.0).max(8.0)
}

fn estimate_text_height(_text: &str) -> f32 {
    16.0
}
