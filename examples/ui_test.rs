use memmap2::MmapMut;
use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
use std::thread;
use std::time::{Duration, Instant};
use viewkit::{host_connect_wayland, host_create_shm_buffer};
use wayland_client::protocol::wl_compositor::WlCompositor;
use wayland_client::protocol::wl_shm::WlShm;
use wayland_client::protocol::{wl_callback, wl_keyboard, wl_pointer, wl_seat};

fn fill_pattern(mmap: &mut MmapMut, width: i32, height: i32, stride: usize, phase: u8) {
    for y in 0..(height as usize) {
        for x in 0..(width as usize) {
            let offset = y * stride + x * 4;
            let r = ((x + phase as usize) % 256) as u8;
            let g = ((y + phase as usize) % 256) as u8;
            let b = (((x + y + phase as usize) / 2) % 256) as u8;
            mmap[offset + 0] = b;
            mmap[offset + 1] = g;
            mmap[offset + 2] = r;
            mmap[offset + 3] = 0xff;
        }
    }
}

fn main() {
    // Wayland 接続を shim 経由で作る
    let (_display, mut event_queue, globals) = host_connect_wayland().expect("host_connect_wayland failed");

    // Compositor と shm を取得
    let compositor = globals
        .instantiate_exact::<WlCompositor>(4)
        .expect("Compositor not available");
    let shm = globals
        .instantiate_exact::<WlShm>(1)
        .expect("wl_shm not available");

    let surface = compositor.create_surface();

    let width: i32 = 400;
    let height: i32 = 240;
    let stride = (width * 4) as usize;

    // shim の create_shm_buffer を使って二重バッファを作成
    let (_tmp0, mut mmap0, _pool0, buffer0) = host_create_shm_buffer(&shm, width, height).expect("create_shm_buffer 0 failed");
    let (_tmp1, mut mmap1, _pool1, buffer1) = host_create_shm_buffer(&shm, width, height).expect("create_shm_buffer 1 failed");

    fill_pattern(&mut mmap0, width, height, stride, 0);
    fill_pattern(&mut mmap1, width, height, stride, 64);
    mmap0.flush().expect("flush failed");
    mmap1.flush().expect("flush failed");

    // 入力ハンドラ
    if let Ok(seat) = globals.instantiate_exact::<wl_seat::WlSeat>(1) {
        let pointer = seat.get_pointer();
        pointer.quick_assign(move |_ptr, event, _| {
            match event {
                wl_pointer::Event::Enter { surface: _, serial: _, surface_x, surface_y } => {
                    println!("Pointer enter: {} {}", surface_x, surface_y);
                }
                wl_pointer::Event::Leave { .. } => {
                    println!("Pointer leave");
                }
                wl_pointer::Event::Motion { surface_x, surface_y, .. } => {
                    println!("Pointer motion: {} {}", surface_x, surface_y);
                }
                wl_pointer::Event::Button { button, state, .. } => {
                    println!("Pointer button: {} {:?}", button, state);
                }
                _ => {}
            }
        });

        let keyboard = seat.get_keyboard();
        keyboard.quick_assign(move |_kb, event, _| {
            match event {
                wl_keyboard::Event::Key { key, state, .. } => {
                    println!("Key event: {} {:?}", key, state);
                }
                wl_keyboard::Event::Enter { .. } => println!("Keyboard enter"),
                wl_keyboard::Event::Leave { .. } => println!("Keyboard leave"),
                _ => {}
            }
        });
    }

    // wl_shell を使って toplevel にする
    if let Ok(wl_shell) = globals.instantiate_exact::<wayland_client::protocol::wl_shell::WlShell>(1) {
        let shell_surface = wl_shell.get_shell_surface(&surface);
        shell_surface.set_toplevel();
    }

    // 初期表示
    surface.attach(Some(&buffer0), 0, 0);
    surface.commit();
    event_queue.sync_roundtrip(&mut (), |_, _, _| {}).expect("roundtrip failed");

    // フレーム駆動でダブルバッファを切り替える
    let frame_requested = Arc::new(AtomicBool::new(false));
    let mut front = 0usize;
    let mut phase: u8 = 0;

    // 最初のフレームコールバックを要求
    {
        let cb = surface.frame();
        let fr = frame_requested.clone();
        cb.quick_assign(move |_cb, event, _| match event {
            wl_callback::Event::Done { .. } => { fr.store(true, Ordering::SeqCst); }
            _ => {}
        });
    }

    let target_frame = Duration::from_millis(16);
    loop {
        let start = Instant::now();

        // イベントを処理
        event_queue.dispatch(&mut (), |_, _, _| {}).expect("dispatch failed");

        if frame_requested.load(Ordering::SeqCst) {
            phase = phase.wrapping_add(8);
            if front == 0 {
                fill_pattern(&mut mmap1, width, height, stride, phase);
                mmap1.flush().ok();
                surface.attach(Some(&buffer1), 0, 0);
                front = 1;
            } else {
                fill_pattern(&mut mmap0, width, height, stride, phase);
                mmap0.flush().ok();
                surface.attach(Some(&buffer0), 0, 0);
                front = 0;
            }
            surface.commit();

            // 新しいコールバックを設定
            let cb2 = surface.frame();
            let fr2 = frame_requested.clone();
            fr2.store(false, Ordering::SeqCst);
            cb2.quick_assign(move |_cb, event, _| match event {
                wl_callback::Event::Done { .. } => { fr2.store(true, Ordering::SeqCst); }
                _ => {}
            });
        }

        let elapsed = start.elapsed();
        if elapsed < target_frame { thread::sleep(target_frame - elapsed); }
    }
}
