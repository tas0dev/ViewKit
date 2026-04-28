use memmap2::MmapMut;
use std::os::unix::io::AsRawFd;
use tempfile::tempfile;
use wayland_client::protocol::wl_shm::Format;
use wayland_client::{Display, GlobalManager};

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

    // Create an anonymous tempfile backing the SHM pool
    let tmp = tempfile().expect("failed to create tempfile");
    tmp.set_len(size as u64).expect("failed to set_len");

    // mmap and fill with a simple test pattern
    let mut mmap = unsafe { MmapMut::map_mut(&tmp).expect("mmap failed") };
    for y in 0..(height as usize) {
        for x in 0..(width as usize) {
            let offset = y * stride + x * 4;
            let r = (x % 256) as u8;
            let g = (y % 256) as u8;
            let b = (((x + y) / 2) % 256) as u8;
            // ARGB8888 little-endian in many compositors: B,G,R,A order in bytes
            mmap[offset + 0] = b; // blue
            mmap[offset + 1] = g; // green
            mmap[offset + 2] = r; // red
            mmap[offset + 3] = 0xff; // alpha
        }
    }
    // Ensure pages are flushed
    mmap.flush().expect("flush failed");

    let fd = tmp.as_raw_fd();

    // Create shm pool and buffer
    let pool = shm.create_pool(fd, size as i32);
    let buffer = pool.create_buffer(0, width, height, stride as i32, Format::Argb8888);

    // Attach buffer to surface and commit
    surface.attach(Some(&buffer), 0, 0);
    surface.commit();

    // Roundtrip so compositor processes the attach
    event_queue
        .sync_roundtrip(&mut (), |_, _, _| {})
        .expect("roundtrip failed");

    // Keep dispatching events so the window remains responsive
    loop {
        event_queue
            .dispatch(&mut (), |_, _, _| {})
            .expect("dispatch failed");
    }
}
