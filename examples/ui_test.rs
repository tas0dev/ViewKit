#[cfg(unix)]
fn main() -> Result<(), String> {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::Duration;

    use viewkit::{host_HostDisplay, pipeline};

    const WIDTH: u32 = 960;
    const HEIGHT: u32 = 540;

    let mut host = host_HostDisplay::new()?;
    let mut surface = host.create_surface(WIDTH as i32, HEIGHT as i32)?;

    let html = r#"
        <body>
            <h1>ViewKit Test</h1>
            <div>asdf</div>
        </body>
    "#;
    let css = r#"
        body { background: #202020; color: #efefef; }
        h1 { color: #8ad7ff; }
        p { color: #d9d9d9; }
        div { color: #7af0b4; }
    "#;

    host.set_toplevel(&mut surface)?;
    let rendered = pipeline::render_document(html, css, WIDTH, HEIGHT);
    blit_framebuffer_to_surface(&rendered.framebuffer.pixels, surface.back_buffer_mut());
    surface.swap_and_commit()?;

    let frame_done = Arc::new(AtomicBool::new(false));
    let mut frame_count = 0_u32;
    loop {
        frame_done.store(false, Ordering::SeqCst);
        surface.request_frame(frame_done.clone())?;
        surface.commit_front()?;

        while !frame_done.load(Ordering::SeqCst) {
            host.dispatch()?;
            std::thread::sleep(Duration::from_millis(8));
        }

        frame_count += 1;
        if frame_count % 120 == 0 {
            println!("ui_test: frame {}", frame_count);
        }
    }
}

#[cfg(unix)]
fn blit_framebuffer_to_surface(src_argb: &[u32], dst: &mut [u8]) {
    let pixel_count = src_argb.len().min(dst.len() / 4);
    for i in 0..pixel_count {
        let argb = src_argb[i];
        let bytes = argb.to_ne_bytes();
        let base = i * 4;
        dst[base] = bytes[0];
        dst[base + 1] = bytes[1];
        dst[base + 2] = bytes[2];
        dst[base + 3] = 0x00;
    }
}

#[cfg(not(unix))]
fn main() {
    eprintln!("ui_test requires a unix host with Wayland.");
}
