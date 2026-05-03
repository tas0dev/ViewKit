use viewkit::components::VComponent;
use viewkit::components_list;
use viewkit::AppBuilder;
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

    // 画面状態を管理
    let screen_state: Arc<viewkit::State<i32>> = Arc::new(viewkit::State::new(0));

    AppBuilder::new(WIDTH, HEIGHT)
        .with_ui_fn({
            let state = screen_state.clone();
            move || {
                let current_screen = state.get();

                if current_screen == 0 {
                    // ホーム画面
                    let state = state.clone();
                    card()
                        .label("Home Screen - Click to Detail")
                        .on_click(move || {
                            state.set(1);
                            println!("Navigated to detail screen");
                        })
                } else {
                    // 詳細画面
                    let state = state.clone();
                    card()
                        .label("Detail Screen - Click to Home")
                        .on_click(move || {
                            state.set(0);
                            println!("Navigated back to home");
                        })
                }
            }
        })?
        .build()?
        .run()
}

#[cfg(not(unix))]
fn main() {
    eprintln!("stateful_ui requires a unix host with Wayland.");
}
