use memmap2::MmapMut;
use std::os::unix::io::AsRawFd;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::{Duration, Instant};
use tempfile::tempfile;
use wayland_client::protocol::wl_shm::Format;
use wayland_client::protocol::{wl_seat, wl_pointer, wl_keyboard, wl_callback};
use wayland_client::{Display, GlobalManager};

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
    // Connect to the compositor provided by the environment (WAYLAND_DISPLAY)
    let display = Display::connect_to_env().expect("Failed to connect to Wayland display");
    let mut event_queue = display.create_event_queue();
    let attached = (*display).attach(event_queue.token());
    let globals = GlobalManager::new(&attached);

    // Sync to get globals
    event_queue
        .sync_roundtrip(&mut (), |_, _, _| {})
        .expect("Failed initial roundtrip");

    // Get required globals
    let compositor = globals
        .instantiate_exact::<wayland_client::protocol::wl_compositor::WlCompositor>(4)
        .expect("Compositor not available");
    let shm = globals
        .instantiate_exact::<wayland_client::protocol::wl_shm::WlShm>(1)
        .expect("wl_shm not available");

    let surface = compositor.create_surface();

    let width: i32 = 400;
    let height: i32 = 240;
    let stride = (width * 4) as usize;
    let size = (stride * height as usize) as usize;

    // Create two anonymous tempfiles backing the SHM pools for double buffering
    let mut tmp0 = tempfile().expect("failed to create tempfile");
    let mut tmp1 = tempfile().expect("failed to create tempfile");
    tmp0.set_len(size as u64).expect("failed to set_len");
    tmp1.set_len(size as u64).expect("failed to set_len");

    let mut mmap0 = unsafe { MmapMut::map_mut(&tmp0).expect("mmap failed") };
    let mut mmap1 = unsafe { MmapMut::map_mut(&tmp1).expect("mmap failed") };

    fill_pattern(&mut mmap0, width, height, stride, 0);
    fill_pattern(&mut mmap1, width, height, stride, 64);
    mmap0.flush().expect("flush failed");
    mmap1.flush().expect("flush failed");

    let fd0 = tmp0.as_raw_fd();
    let fd1 = tmp1.as_raw_fd();

    // Create shm pools and buffers
    let pool0 = shm.create_pool(fd0, size as i32);
    let pool1 = shm.create_pool(fd1, size as i32);
    let buffer0 = pool0.create_buffer(0, width, height, stride as i32, Format::Argb8888);
    let buffer1 = pool1.create_buffer(0, width, height, stride as i32, Format::Argb8888);

    // Input handling: seat -> pointer / keyboard
    if let Ok(seat) = globals.instantiate_exact::<wl_seat::WlSeat>(1) {
        // Pointer
        let pointer = seat.get_pointer();
        pointer.quick_assign(move |_ptr, event, _| {
            match event {
                wl_pointer::Event::Enter { surface, serial, surface_x, surface_y } => {
                    println!("Pointer enter: {} {}", surface_x, surface_y);
                }
                wl_pointer::Event::Leave { surface, serial } => {
                    println!("Pointer leave");
                }
                wl_pointer::Event::Motion { time, surface_x, surface_y } => {
                    println!("Pointer motion: {} {}", surface_x, surface_y);
                }
                wl_pointer::Event::Button { serial, time, button, state } => {
                    println!("Pointer button: {} {:?}", button, state);
                }
                wl_pointer::Event::Axis { time, axis, value } => {
                    println!("Pointer axis: {:?} {}", axis, value);
                }
                _ => {}
            }
        });

        // Keyboard
        let keyboard = seat.get_keyboard();
        keyboard.quick_assign(move |_kb, event, _| {
            match event {
                wl_keyboard::Event::Key { serial, time, key, state } => {
                    println!("Key event: {} {:?}", key, state);
                }
                wl_keyboard::Event::Enter { serial, surface, keys } => {
                    println!("Keyboard enter");
                }
                wl_keyboard::Event::Leave { serial, surface } => {
                    println!("Keyboard leave");
                }
                _ => {}
            }
        });
    }

    // Try to make surface a toplevel via wl_shell (compat)
    if let Ok(wl_shell) = globals.instantiate_exact::<wayland_client::protocol::wl_shell::WlShell>(1) {
        let shell_surface = wl_shell.get_shell_surface(&surface);
        shell_surface.set_toplevel();
    }

    // Attach initial buffer and commit
    surface.attach(Some(&buffer0), 0, 0);
    surface.commit();
    event_queue.sync_roundtrip(&mut (), |_, _, _| {}).expect("roundtrip failed");

    // Double-buffer loop using frame callbacks
    let frame_requested = Arc::new(AtomicBool::new(false));
    let mut front = 0usize;
    let mut phase: u8 = 0;

    // Request first frame callback
    {
        let cb = surface.frame();
        let fr = frame_requested.clone();
        cb.quick_assign(move |_cb, event, _| match event {
            wl_callback::Event::Done { callback_data } => {
                fr.store(true, Ordering::SeqCst);
            }
            _ => {}
        });
    }

    let target_frame = Duration::from_millis(16);
    loop {
        let start = Instant::now();

        // dispatch events (non-blocking with small timeout)
        event_queue.dispatch(&mut (), |_, _, _| {}).expect("dispatch failed");

        if frame_requested.load(Ordering::SeqCst) {
            // prepare next buffer contents
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

            // new frame callback
            let cb2 = surface.frame();
            let fr2 = frame_requested.clone();
            fr2.store(false, Ordering::SeqCst);
            cb2.quick_assign(move |_cb, event, _| match event {
                wl_callback::Event::Done { callback_data } => {
                    fr2.store(true, Ordering::SeqCst);
                }
                _ => {}
            });
        }

        // throttle to ~60fps
        let elapsed = start.elapsed();
        if elapsed < target_frame {
            thread::sleep(target_frame - elapsed);
        }
    }
}
