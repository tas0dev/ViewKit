use crate::components::Component;
use ui_layout::{LayoutNode, Style, Length, LayoutEngine, LayoutBoxes};
use std::env;

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

fn draw_rect_outline(buf: &mut [u8], buf_width: usize, buf_height: usize, stride: usize, x: i32, y: i32, w: i32, h: i32, thickness: usize, r: u8, g: u8, b: u8, a: u8) {
    if w <= 0 || h <= 0 { return; }
    let x0 = x.max(0) as usize;
    let y0 = y.max(0) as usize;
    let x1 = (x0 + w as usize).min(buf_width);
    let y1 = (y0 + h as usize).min(buf_height);
    if x0 >= x1 || y0 >= y1 { return; }

    // draw top and bottom
    for t in 0..thickness {
        let yy_top = y0 + t;
        if yy_top < y1 {
            let row = yy_top * stride;
            for xx in x0..x1 {
                let off = row + xx * 4;
                if off + 3 < buf.len() {
                    buf[off + 0] = b;
                    buf[off + 1] = g;
                    buf[off + 2] = r;
                    buf[off + 3] = a;
                }
            }
        }
        let yy_bot = y1.saturating_sub(1 + t);
        if yy_bot >= y0 && yy_bot < buf_height {
            let row = yy_bot * stride;
            for xx in x0..x1 {
                let off = row + xx * 4;
                if off + 3 < buf.len() {
                    buf[off + 0] = b;
                    buf[off + 1] = g;
                    buf[off + 2] = r;
                    buf[off + 3] = a;
                }
            }
        }
    }

    // draw left and right
    for t in 0..thickness {
        let xx_left = x0 + t;
        if xx_left < x1 {
            for yy in y0..y1 {
                let row = yy * stride;
                let off = row + xx_left * 4;
                if off + 3 < buf.len() {
                    buf[off + 0] = b;
                    buf[off + 1] = g;
                    buf[off + 2] = r;
                    buf[off + 3] = a;
                }
            }
        }
        let xx_right = x1.saturating_sub(1 + t);
        if xx_right >= x0 && xx_right < buf_width {
            for yy in y0..y1 {
                let row = yy * stride;
                let off = row + xx_right * 4;
                if off + 3 < buf.len() {
                    buf[off + 0] = b;
                    buf[off + 1] = g;
                    buf[off + 2] = r;
                    buf[off + 3] = a;
                }
            }
        }
    }
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

        // check debug flag
        let debug = env::var("VIEWKIT_LAYOUT_DEBUG").is_ok();

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
                        if debug {
                            // yellow outline
                            draw_rect_outline(buf, buf_width, buf_height, stride, cx, cy, cw, ch, 2, 0xff, 0xff, 0x00, 0xff);
                        }
                    }
                    LayoutBoxes::Multiple(list) if !list.is_empty() => {
                        let bm = &list[0];
                        let cb = &bm.content_box;
                        let cx = x + cb.x as i32;
                        let cy = y + cb.y as i32;
                        let cw = cb.width as i32;
                        let ch = cb.height as i32;
                        child.render_into(buf, buf_width, buf_height, stride, cx, cy, cw, ch);
                        if debug {
                            draw_rect_outline(buf, buf_width, buf_height, stride, cx, cy, cw, ch, 2, 0x00, 0xff, 0x00, 0xff);
                        }
                    }
                    _ => {
                        // nothing
                    }
                }
            }
        }
    }
}
