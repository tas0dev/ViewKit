use viewkit::components::Vcomponent;
use viewkit::components_list;
use viewkit::{AppBuilder, State};
use std::sync::Arc;

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

    let screen_state: Arc<State<i32>> = Arc::new(State::new(0));

    // ホーム画面
    let home_screen = {
        let state = screen_state.clone();
        card()
            .label("Home Screen")
            .on_click(move || {
                state.set(1);
                println!("Navigated to detail screen");
            })
    };

    // 詳細画面
    let detail_screen = {
        let state = screen_state.clone();
        card()
            .label("Detail Screen")
            .on_click(move || {
                state.set(0);
                println!("Navigated back to home");
            })
    };

    let current_screen = screen_state.get();
    let ui = if current_screen == 0 {
        home_screen
    } else {
        detail_screen
    };

    AppBuilder::new(WIDTH, HEIGHT)
        .with_ui(ui)?
        .build()?
        .run()
}

#[cfg(not(unix))]
fn main() {
    eprintln!("stateful_ui requires a unix host with Wayland.");
}
