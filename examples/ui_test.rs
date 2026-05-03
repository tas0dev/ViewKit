use viewkit::components::Vcomponent;
use viewkit::components_list;
use viewkit::AppBuilder;

components_list! {
    button,
    card,
    text,
    dock,
    appicon,
}

#[cfg(unix)]
fn main() -> Result<(), String> {
    const WIDTH: u32 = 960;
    const HEIGHT: u32 = 540;

    let icons = (0..5).map(|_| appicon());
    let ui = dock().children(icons);

    AppBuilder::new(WIDTH, HEIGHT)
        .with_ui(ui)?
        .build()?
        .run()
}

#[cfg(not(unix))]
fn main() {
    eprintln!("ui_test requires a unix host with Wayland.");
}
