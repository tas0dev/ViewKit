use super::dom::DomNodeKind;
use super::style::{StyleMap, StyledNode, StyledTree};
use ui_layout as _;

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

pub fn compute_layout(styled: &StyledTree, width: u32, height: u32) -> LayoutTree {
    let mut cursor_y = 0_i32;
    let root = build_layout_node(&styled.root, 0, width as i32, &mut cursor_y, height as i32);
    LayoutTree { root }
}

fn build_layout_node(
    node: &StyledNode,
    x: i32,
    max_width: i32,
    cursor_y: &mut i32,
    max_height: i32,
) -> LayoutNode {
    let start_y = *cursor_y;
    let mut children = Vec::new();

    match &node.node.kind {
        DomNodeKind::Text(text) => {
            let height = 20;
            *cursor_y = (*cursor_y + height).min(max_height);
            return LayoutNode {
                rect: Rect {
                    x,
                    y: start_y,
                    width: max_width,
                    height,
                },
                styles: node.styles.clone(),
                kind: LayoutNodeKind::Text {
                    content: text.clone(),
                },
                children,
            };
        }
        DomNodeKind::Element(element) => {
            for child in &node.children {
                children.push(build_layout_node(child, x + 8, max_width - 16, cursor_y, max_height));
            }

            if children.is_empty() {
                *cursor_y = (*cursor_y + 24).min(max_height);
            }

            let end_y = (*cursor_y).max(start_y + 24);
            LayoutNode {
                rect: Rect {
                    x,
                    y: start_y,
                    width: max_width,
                    height: (end_y - start_y).max(1),
                },
                styles: node.styles.clone(),
                kind: LayoutNodeKind::Element {
                    tag_name: element.tag_name.clone(),
                },
                children,
            }
        }
    }
}
