// Components module for ViewKit

/// Simple component trait used by ViewKit components.
pub trait Component {
    /// Preferred size in pixels (None = auto)
    fn pref_size(&self) -> (Option<i32>, Option<i32>);

    /// Render into provided buffer rectangle.
    fn render_into(&self, buf: &mut [u8], buf_width: usize, buf_height: usize, stride: usize, x: i32, y: i32, w: i32, h: i32);
}

pub mod canvas;
pub mod container;
pub mod style;
pub mod view;

pub use canvas::Canvas;
pub use container::Container;
pub use style::StyleBuilder;
pub use view::View;
pub mod ext;
pub use ext::ComponentExt;

pub fn make_style_with_padding(
    width: Option<i32>,
    height: Option<i32>,
    padding: Option<(f32, f32, f32, f32)>,
    margin: Option<(f32, f32, f32, f32)>,
    gap: Option<(f32, f32)>,
) -> ui_layout::Style {
    let mut s = ui_layout::Style::default();
    s.size.width = match width { Some(w) => ui_layout::Length::Px(w as f32), None => ui_layout::Length::Auto };
    s.size.height = match height { Some(h) => ui_layout::Length::Px(h as f32), None => ui_layout::Length::Auto };
    if let Some((pt, pr, pb, pl)) = padding {
        s.spacing.padding_top = ui_layout::Length::Px(pt);
        s.spacing.padding_right = ui_layout::Length::Px(pr);
        s.spacing.padding_bottom = ui_layout::Length::Px(pb);
        s.spacing.padding_left = ui_layout::Length::Px(pl);
    }
    if let Some((mt, mr, mb, ml)) = margin {
        s.spacing.margin_top = ui_layout::Length::Px(mt);
        s.spacing.margin_right = ui_layout::Length::Px(mr);
        s.spacing.margin_bottom = ui_layout::Length::Px(mb);
        s.spacing.margin_left = ui_layout::Length::Px(ml);
    }
    if let Some((cg, rg)) = gap {
        s.column_gap = ui_layout::Length::Px(cg);
        s.row_gap = ui_layout::Length::Px(rg);
    }
    s
}