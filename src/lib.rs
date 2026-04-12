mod catalog;
mod component_dsl;
mod render;
mod runtime_theme;

pub use catalog::{component_catalog, UiComponent};
pub use component_dsl::{templates as component_templates, ComponentTemplate, CssDecl};
pub use render::render_component_catalog;

pub const DEFAULT_CATALOG_WIDTH: u16 = 360;
pub const DEFAULT_CATALOG_HEIGHT: u16 = 220;

pub fn build_component_catalog_frame() -> (u16, u16, Vec<u32>) {
    let width = DEFAULT_CATALOG_WIDTH;
    let height = DEFAULT_CATALOG_HEIGHT;
    let pixels = render_component_catalog(width as usize, height as usize, &component_catalog());
    (width, height, pixels)
}

pub fn build_template_catalog_frame() -> (u16, u16, Vec<u32>) {
    let width = DEFAULT_CATALOG_WIDTH;
    let height = DEFAULT_CATALOG_HEIGHT;
    let pixels = if let Some(px) = runtime_theme::build_runtime_theme_frame(
        width as usize,
        height as usize,
        "/Libraries/AppService/ViewKit/themes",
    ) {
        px
    } else {
        render::render_template_catalog(width as usize, height as usize, component_templates())
    };
    (width, height, pixels)
}
