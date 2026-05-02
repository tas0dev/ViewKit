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
    build_for_node(&layout.root, &mut items);
    DisplayList { items }
}

fn build_for_node(node: &LayoutNode, out: &mut Vec<DisplayCommand>) {
    match &node.kind {
        LayoutNodeKind::Element { tag_name } => {
            let color = if tag_name == "body" { 0xFF202020 } else { 0xFF2D2D2D };
            out.push(DisplayCommand::FillRect {
                rect: node.rect,
                color,
            });
        }
        LayoutNodeKind::Text { content } => {
            out.push(DisplayCommand::DrawText {
                x: node.rect.x,
                y: node.rect.y,
                color: 0xFFFFFFFF,
                text: content.clone(),
            });
        }
    }

    for child in &node.children {
        build_for_node(child, out);
    }
}
