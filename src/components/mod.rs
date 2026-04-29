// Components module for ViewKit

/// Simple component trait used by ViewKit components.
pub trait Component {
    fn pref_size(&self) -> (Option<i32>, Option<i32>);
    fn render_into(&self, buf: &mut [u8], buf_width: usize, buf_height: usize, stride: usize, x: i32, y: i32, w: i32, h: i32);
}

pub mod canvas;
pub mod container;

pub use canvas::Canvas;
pub use container::Container;