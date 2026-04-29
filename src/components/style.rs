//! Style builder providing a SwiftUI-like chained API to construct ui_layout::Style
//!
//! Example:
//! let s = StyleBuilder::new().width(200).height(100).padding(4.0).flex_column().build();

pub struct StyleBuilder {
    style: ui_layout::Style,
}

impl StyleBuilder {
    pub fn new() -> Self {
        Self { style: ui_layout::Style::default() }
    }

    pub fn width(mut self, px: i32) -> Self {
        self.style.size.width = ui_layout::Length::Px(px as f32);
        self
    }

    pub fn height(mut self, px: i32) -> Self {
        self.style.size.height = ui_layout::Length::Px(px as f32);
        self
    }

    /// Set padding uniformly
    pub fn padding(mut self, pad: f32) -> Self {
        self.style.spacing.padding_top = ui_layout::Length::Px(pad);
        self.style.spacing.padding_right = ui_layout::Length::Px(pad);
        self.style.spacing.padding_bottom = ui_layout::Length::Px(pad);
        self.style.spacing.padding_left = ui_layout::Length::Px(pad);
        self
    }

    /// Set padding with individual edges (top, right, bottom, left)
    pub fn padding_each(mut self, top: f32, right: f32, bottom: f32, left: f32) -> Self {
        self.style.spacing.padding_top = ui_layout::Length::Px(top);
        self.style.spacing.padding_right = ui_layout::Length::Px(right);
        self.style.spacing.padding_bottom = ui_layout::Length::Px(bottom);
        self.style.spacing.padding_left = ui_layout::Length::Px(left);
        self
    }

    /// Set margin uniformly
    pub fn margin(mut self, m: f32) -> Self {
        self.style.spacing.margin_top = ui_layout::Length::Px(m);
        self.style.spacing.margin_right = ui_layout::Length::Px(m);
        self.style.spacing.margin_bottom = ui_layout::Length::Px(m);
        self.style.spacing.margin_left = ui_layout::Length::Px(m);
        self
    }

    /// Set gap (column_gap, row_gap)
    pub fn gap(mut self, column: f32, row: f32) -> Self {
        self.style.column_gap = ui_layout::Length::Px(column);
        self.style.row_gap = ui_layout::Length::Px(row);
        self
    }

    /// Convenience: set display to flex with direction row
    pub fn flex_row(mut self) -> Self {
        self.style.display = ui_layout::Display::Flex { flex_direction: ui_layout::FlexDirection::Row };
        self
    }

    /// Convenience: set display to flex with direction column
    pub fn flex_column(mut self) -> Self {
        self.style.display = ui_layout::Display::Flex { flex_direction: ui_layout::FlexDirection::Column };
        self
    }

    /// Convenience: set display to block
    pub fn block(mut self) -> Self {
        self.style.display = ui_layout::Display::Block;
        self
    }

    /// Set both width and height to fill available space (100%)
    pub fn frame_fill(mut self) -> Self {
        self.style.size.width = ui_layout::Length::Percent(100.0);
        self.style.size.height = ui_layout::Length::Percent(100.0);
        self
    }

    /// Set flex-grow for items placed in a flex container
    pub fn flex_grow(mut self, v: f32) -> Self {
        self.style.item_style.flex_grow = v;
        self
    }

    /// Build the ui_layout::Style
    pub fn build(self) -> ui_layout::Style {
        self.style
    }
}
