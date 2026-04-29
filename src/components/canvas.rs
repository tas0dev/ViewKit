use crate::render::Color;

/// Simple Canvas component: fills its layout area with a solid color.
/// Integrates with ui_layout to compute size when requested.
pub struct Canvas {
    color: Color,
    // preferred size; layout engine may override
    pref_w: i32,
    pref_h: i32,
}

impl Canvas {
    pub fn new(color: Color, pref_w: i32, pref_h: i32) -> Self {
        Canvas { color, pref_w, pref_h }
    }

    pub fn set_color(&mut self, color: Color) { self.color = color; }

    /// Use ui_layout to compute the size this canvas would occupy within the
    /// available space. Returns (width, height) in pixels.
    pub fn layout(&self, avail_w: i32, avail_h: i32) -> (i32,i32) {
        // Build a minimal layout tree: root -> child (this canvas)
        let mut child_style = ui_layout::Style::default();
        // set preferred size if provided
        child_style.size.width = if self.pref_w <= 0 { ui_layout::Length::Auto } else { ui_layout::Length::Px(self.pref_w as f32) };
        child_style.size.height = if self.pref_h <= 0 { ui_layout::Length::Auto } else { ui_layout::Length::Px(self.pref_h as f32) };

        let child = ui_layout::LayoutNode::new(child_style);
        let mut root = ui_layout::LayoutNode::with_children(ui_layout::Style::default(), vec![child]);

        // Run layout engine with viewport equal to available size
        ui_layout::LayoutEngine::layout(&mut root, avail_w as f32, avail_h as f32);

        // Extract computed content box size for the child
        if let Some(first) = root.children.get(0) {
            match &first.layout_boxes {
                ui_layout::LayoutBoxes::Single(bm) => {
                    let w = bm.content_box.width.max(0.0) as i32;
                    let h = bm.content_box.height.max(0.0) as i32;
                    return (w, h);
                }
                ui_layout::LayoutBoxes::Multiple(list) if !list.is_empty() => {
                    let bm = &list[0];
                    let w = bm.content_box.width.max(0.0) as i32;
                    let h = bm.content_box.height.max(0.0) as i32;
                    return (w, h);
                }
                _ => {}
            }
        }
        // fallback to available
        (avail_w.max(0), avail_h.max(0))
    }

    /// Render the canvas into a rectangular region within the provided buffer.
    /// x,y specify the top-left pixel coordinate inside the buffer to paint into.
    pub fn render_into(&self, buf: &mut [u8], buf_width: usize, buf_height: usize, stride: usize, x: i32, y: i32, w: i32, h: i32) {
        let x0 = x.max(0) as usize;
        let y0 = y.max(0) as usize;
        let w = w.max(0) as usize;
        let h = h.max(0) as usize;
        let max_w = buf_width;
        let max_h = buf_height;
        if x0 >= max_w || y0 >= max_h { return; }
        let x1 = (x0 + w).min(max_w);
        let y1 = (y0 + h).min(max_h);

        for yy in y0..y1 {
            let row = yy * stride;
            for xx in x0..x1 {
                let off = row + xx * 4;
                buf[off + 0] = self.color.b;
                buf[off + 1] = self.color.g;
                buf[off + 2] = self.color.r;
                buf[off + 3] = self.color.a;
            }
        }
    }
}

impl crate::components::Component for Canvas {
    fn pref_size(&self) -> (Option<i32>, Option<i32>) {
        let w = if self.pref_w <= 0 { None } else { Some(self.pref_w) };
        let h = if self.pref_h <= 0 { None } else { Some(self.pref_h) };
        (w, h)
    }

    fn render_into(&self, buf: &mut [u8], buf_width: usize, buf_height: usize, stride: usize, x: i32, y: i32, w: i32, h: i32) {
        self.render_into(buf, buf_width, buf_height, stride, x, y, w, h)
    }
}
