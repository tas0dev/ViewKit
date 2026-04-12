mod catalog;
mod render;

pub use catalog::{component_catalog, UiComponent};
pub use render::render_component_catalog;

pub const DEFAULT_CATALOG_WIDTH: u16 = 360;
pub const DEFAULT_CATALOG_HEIGHT: u16 = 220;

pub fn build_component_catalog_frame() -> (u16, u16, Vec<u32>) {
    let width = DEFAULT_CATALOG_WIDTH;
    let height = DEFAULT_CATALOG_HEIGHT;
    let pixels = render_component_catalog(width as usize, height as usize, &component_catalog());
    (width, height, pixels)
}
