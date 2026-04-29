use crate::components::Component;

/// View is a chainable builder that wraps a Component and accumulates a ui_layout::Style
/// using a SwiftUI-like modifier API. Use into_pair() to obtain (Box<dyn Component>, Style)
/// which Container::with_children accepts.
pub struct View {
    inner: Box<dyn Component>,
    style: ui_layout::Style,
}

impl View {
    pub fn new(inner: Box<dyn Component>) -> Self {
        Self { inner, style: ui_layout::Style::default() }
    }

    pub fn padding(mut self, pad: f32) -> Self {
        self.style.spacing.padding_top = ui_layout::Length::Px(pad);
        self.style.spacing.padding_right = ui_layout::Length::Px(pad);
        self.style.spacing.padding_bottom = ui_layout::Length::Px(pad);
        self.style.spacing.padding_left = ui_layout::Length::Px(pad);
        self
    }

    pub fn padding_each(mut self, top: f32, right: f32, bottom: f32, left: f32) -> Self {
        self.style.spacing.padding_top = ui_layout::Length::Px(top);
        self.style.spacing.padding_right = ui_layout::Length::Px(right);
        self.style.spacing.padding_bottom = ui_layout::Length::Px(bottom);
        self.style.spacing.padding_left = ui_layout::Length::Px(left);
        self
    }

    pub fn margin(mut self, m: f32) -> Self {
        self.style.spacing.margin_top = ui_layout::Length::Px(m);
        self.style.spacing.margin_right = ui_layout::Length::Px(m);
        self.style.spacing.margin_bottom = ui_layout::Length::Px(m);
        self.style.spacing.margin_left = ui_layout::Length::Px(m);
        self
    }

    pub fn gap(mut self, column: f32, row: f32) -> Self {
        self.style.column_gap = ui_layout::Length::Px(column);
        self.style.row_gap = ui_layout::Length::Px(row);
        self
    }

    pub fn frame(mut self, width: Option<i32>, height: Option<i32>) -> Self {
        self.style.size.width = match width { Some(w) => ui_layout::Length::Px(w as f32), None => ui_layout::Length::Auto };
        self.style.size.height = match height { Some(h) => ui_layout::Length::Px(h as f32), None => ui_layout::Length::Auto };
        self
    }

    /// Set both width and height to fill available space (100%) of the containing block
    pub fn frame_fill(mut self) -> Self {
        self.style.size.width = ui_layout::Length::Percent(100.0);
        self.style.size.height = ui_layout::Length::Percent(100.0);
        self
    }

    /// Set flex-grow for this item when placed in a flex container
    pub fn flex_grow(mut self, v: f32) -> Self {
        self.style.item_style.flex_grow = v;
        self
    }

    pub fn flex_row(mut self) -> Self {
        self.style.display = ui_layout::Display::Flex { flex_direction: ui_layout::FlexDirection::Row };
        self
    }

    pub fn flex_column(mut self) -> Self {
        self.style.display = ui_layout::Display::Flex { flex_direction: ui_layout::FlexDirection::Column };
        self
    }

    pub fn build(self) -> ui_layout::Style {
        self.style
    }

    pub fn into_pair(self) -> (Box<dyn Component>, ui_layout::Style) {
        (self.inner, self.style)
    }
}
