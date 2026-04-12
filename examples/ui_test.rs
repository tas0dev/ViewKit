use swiftlib::{keyboard, task};

fn main() {
    println!("[VIEWKIT] ui_test start");
    match viewkit::show_component_catalog() {
        Ok(window_id) => println!("[VIEWKIT] component catalog shown (window_id={})", window_id),
        Err(e) => {
            eprintln!("[VIEWKIT] failed to show component catalog: {}", e);
            return;
        }
    }

    loop {
        let sc_opt = match keyboard::read_scancode_tap() {
            Ok(Some(sc)) => Some(sc),
            Ok(None) => keyboard::read_scancode(),
            Err(_) => keyboard::read_scancode(),
        };
        if let Some(sc) = sc_opt {
            if sc == 0x01 || sc == 0x81 {
                println!("[VIEWKIT] exit");
                return;
            }
        }
        task::yield_now();
    }
}
