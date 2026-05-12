// ViewKit のホスト向け Shim を公開する
// Shared libkagami is hosted under ../Kagami.
#[path = "../../Kagami/src/libkagami.rs"]
pub mod libkagami;
pub mod pipeline;
pub mod components;
pub mod app;
pub mod state;

pub use libkagami::*;
pub use app::AppBuilder;
pub use state::State;
pub use components::VComponent;

/// VComponent を pixel buffer に render
pub fn render_component_to_pixmap(component: &VComponent, width: u32, height: u32) -> Vec<u32> {
    let html = component.render();
    let css = component.css();
    let output = pipeline::render_document(&html, &css, width, height);
    output.framebuffer.pixels
}

