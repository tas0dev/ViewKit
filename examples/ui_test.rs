use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::{Duration, Instant};
use viewkit::{host_HostDisplay, register_pointer_and_keyboard};

fn main() {
    // Wayland 接続を shim 経由で作る
    let mut host = host_HostDisplay::new().expect("host_HostDisplay::new failed");

    // HostDisplay 経由で surface を作成
    let width: i32 = 400;
    let height: i32 = 240;
    let stride = (width * 4) as usize;
    let mut surf = host.create_surface(width, height).expect("create_surface failed");

    // 入力ハンドラ登録（デモ用に motion と key をログ出力）
    use wayland_client::WEnum;
    use wayland_client::protocol::wl_keyboard::KeyState;

    let _ = register_pointer_and_keyboard(
        &mut host,
        Some(Arc::new(|x: f64, y: f64| { println!("Pointer motion: {} {}", x, y); })),
        Some(Arc::new(|k: u32, s: WEnum<KeyState>| { println!("Key event: {} {:?}", k, s); })),
    );

    // Try to make surface a toplevel so compositor maps it as a window
    host.set_toplevel(&mut surf).ok();

    // フレームコールバック管理
    let frame_requested = Arc::new(AtomicBool::new(false));
    // 最初のフレームを要求
    surf.request_frame(frame_requested.clone()).expect("request_frame failed");

    let target_frame = Duration::from_millis(16);
    let mut phase: u8 = 0;
    loop {
        let start = Instant::now();

        // イベントを処理
        host.dispatch().ok();

        if frame_requested.load(Ordering::SeqCst) {
            phase = phase.wrapping_add(8);
            let back = surf.back_buffer_mut();
            for y in 0..(height as usize) {
                for x in 0..(width as usize) {
                    let offset = y * stride + x * 4;
                    let r = ((x + phase as usize) % 256) as u8;
                    let g = ((y + phase as usize) % 256) as u8;
                    let b = (((x + y + phase as usize) / 2) % 256) as u8;
                    back[offset + 0] = b;
                    back[offset + 1] = g;
                    back[offset + 2] = r;
                    back[offset + 3] = 0xff;
                }
            }
            surf.swap_and_commit().expect("swap_and_commit failed");
            // 再度フレームを要求
            frame_requested.store(false, Ordering::SeqCst);
            surf.request_frame(frame_requested.clone()).expect("request_frame failed");
        }

        let elapsed = start.elapsed();
        if elapsed < target_frame { thread::sleep(target_frame - elapsed); }
    }
}
