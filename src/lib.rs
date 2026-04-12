mod catalog;
mod ipc;
mod render;

pub use catalog::{component_catalog, UiComponent};

pub fn show_component_catalog() -> Result<u32, &'static str> {
    let window_id = ipc::create_app_window(360, 220)?;
    let pixels = render::render_component_catalog(360, 220, &component_catalog());
    ipc::flush_window_chunked(window_id, 360, 220, &pixels)?;
    Ok(window_id)
}
