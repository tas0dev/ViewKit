use crate::components::Component;
use ui_layout::{LayoutNode, Style, Length, LayoutEngine, LayoutBoxes};

pub struct Container {
    // pair each child with its layout Style
    children: Vec<(Box<dyn Component>, Style)>,
}

impl Container {
    pub fn new() -> Self { Container { children: Vec::new() } }

    /// children provided with explicit Style per child
    pub fn with_children(children: Vec<(Box<dyn Component>, Style)>) -> Self { Container { children } }

    /// add child with default style
    pub fn add(&mut self, c: Box<dyn Component>) { self.children.push((c, Style::default())); }

    /// add child with explicit style
    pub fn add_with_style(&mut self, c: Box<dyn Component>, s: Style) { self.children.push((c, s)); }
}

impl Component for Container {
    fn pref_size(&self) -> (Option<i32>, Option<i32>) {
        // container is auto-sized by default
        (None, None)
    }

    fn render_into(&self, buf: &mut [u8], buf_width: usize, buf_height: usize, stride: usize, x: i32, y: i32, w: i32, h: i32) {
        // Build layout node tree: root with children using provided Styles
        let mut child_nodes = Vec::with_capacity(self.children.len());
        for (_ch, style) in &self.children {
            child_nodes.push(LayoutNode::new(style.clone()));
        }
        let mut root = LayoutNode::with_children(Style::default(), child_nodes);

        // perform layout within viewport w,h
        LayoutEngine::layout(&mut root, w as f32, h as f32);

        // dispatch rendering to children based on computed boxes
        for (i, (child, _style)) in self.children.iter().enumerate() {
            if let Some(node) = root.children.get(i) {
                match &node.layout_boxes {
                    LayoutBoxes::Single(bm) => {
                        let cb = &bm.content_box;
                        let cx = x + cb.x as i32;
                        let cy = y + cb.y as i32;
                        let cw = cb.width as i32;
                        let ch = cb.height as i32;
                        child.render_into(buf, buf_width, buf_height, stride, cx, cy, cw, ch);
                    }
                    LayoutBoxes::Multiple(list) if !list.is_empty() => {
                        let bm = &list[0];
                        let cb = &bm.content_box;
                        let cx = x + cb.x as i32;
                        let cy = y + cb.y as i32;
                        let cw = cb.width as i32;
                        let ch = cb.height as i32;
                        child.render_into(buf, buf_width, buf_height, stride, cx, cy, cw, ch);
                    }
                    _ => {
                        // nothing
                    }
                }
            }
        }
    }
}
