// Host-side shim for Kagami minimal APIs.

#[cfg(unix)]
mod unix_impl {
    use memmap2::MmapMut;
    use std::fs::File;
    use std::os::unix::io::AsRawFd;
    use tempfile::tempfile;
    use wayland_client::protocol::wl_shm_pool::WlShmPool;
    use std::os::unix::io::BorrowedFd;
    use wayland_client::protocol::{
        wl_buffer, wl_compositor, wl_shm, wl_shm::Format, wl_registry, wl_shm_pool,
        wl_surface, wl_pointer, wl_keyboard, wl_seat, wl_callback, wl_shell, wl_shell_surface,
    };
    use wayland_protocols::xdg::shell::client::xdg_wm_base;
    use wayland_protocols::xdg::shell::client::xdg_surface;
    use wayland_protocols::xdg::shell::client::xdg_toplevel;
    use wayland_client::{Connection, EventQueue, QueueHandle, Dispatch};
    use wayland_client::globals::{registry_queue_init, GlobalList, GlobalListContents};
    use wayland_client::protocol::wl_surface::WlSurface;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    // Registry state used only for initial global collection
    struct RegistryState;
    impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for RegistryState {
        fn event(
            _state: &mut RegistryState,
            _proxy: &wl_registry::WlRegistry,
            _event: wl_registry::Event,
            _data: &GlobalListContents,
            _conn: &Connection,
            _qh: &QueueHandle<RegistryState>,
        ) {
            // no-op: global list helper maintains globals
        }
    }

    // Implement empty Dispatch handlers for objects we will create with the same QueueHandle
    // no-op handlers for various protocol objects using () userdata
    impl Dispatch<WlShmPool, ()> for RegistryState {
        fn event(
            _state: &mut RegistryState,
            _proxy: &WlShmPool,
            _event: wl_shm_pool::Event,
            _data: &(),
            _conn: &Connection,
            _qh: &QueueHandle<RegistryState>,
        ) {}
    }
    impl Dispatch<wl_buffer::WlBuffer, ()> for RegistryState {
        fn event(
            _state: &mut RegistryState,
            _proxy: &wl_buffer::WlBuffer,
            _event: wl_buffer::Event,
            _data: &(),
            _conn: &Connection,
            _qh: &QueueHandle<RegistryState>,
        ) {}
    }
    impl Dispatch<WlSurface, ()> for RegistryState {
        fn event(
            _state: &mut RegistryState,
            _proxy: &WlSurface,
            _event: wl_surface::Event,
            _data: &(),
            _conn: &Connection,
            _qh: &QueueHandle<RegistryState>,
        ) {}
    }
    impl Dispatch<wl_compositor::WlCompositor, ()> for RegistryState {
        fn event(
            _state: &mut RegistryState,
            _proxy: &wl_compositor::WlCompositor,
            _event: wl_compositor::Event,
            _data: &(),
            _conn: &Connection,
            _qh: &QueueHandle<RegistryState>,
        ) {}
    }
    impl Dispatch<wl_shm::WlShm, ()> for RegistryState {
        fn event(
            _state: &mut RegistryState,
            _proxy: &wl_shm::WlShm,
            _event: wl_shm::Event,
            _data: &(),
            _conn: &Connection,
            _qh: &QueueHandle<RegistryState>,
        ) {}
    }

    // Pointer/Keyboard callbacks userdata
    struct PointerHandler(Arc<dyn Fn(f64, f64) + Send + Sync>);
    struct KeyboardHandler(Arc<dyn Fn(u32, wayland_client::WEnum<wl_keyboard::KeyState>) + Send + Sync>);

    impl Dispatch<wl_pointer::WlPointer, Arc<PointerHandler>> for RegistryState {
        fn event(
            _state: &mut RegistryState,
            _proxy: &wl_pointer::WlPointer,
            event: wl_pointer::Event,
            data: &Arc<PointerHandler>,
            _conn: &Connection,
            _qh: &QueueHandle<RegistryState>,
        ) {
            match event {
                wl_pointer::Event::Motion { surface_x, surface_y, .. } => {
                    data.0(surface_x, surface_y);
                }
                _ => {}
            }
        }
    }

    impl Dispatch<wl_keyboard::WlKeyboard, Arc<KeyboardHandler>> for RegistryState {
        fn event(
            _state: &mut RegistryState,
            _proxy: &wl_keyboard::WlKeyboard,
            event: wl_keyboard::Event,
            data: &Arc<KeyboardHandler>,
            _conn: &Connection,
            _qh: &QueueHandle<RegistryState>,
        ) {
            match event {
                wl_keyboard::Event::Key { key, state, .. } => {
                    data.0(key, state);
                }
                _ => {}
            }
        }
    }

    impl Dispatch<wl_callback::WlCallback, Arc<AtomicBool>> for RegistryState {
        fn event(
            _state: &mut RegistryState,
            _proxy: &wl_callback::WlCallback,
            event: wl_callback::Event,
            data: &Arc<AtomicBool>,
            _conn: &Connection,
            _qh: &QueueHandle<RegistryState>,
        ) {
            if let wl_callback::Event::Done { .. } = event {
                data.store(true, Ordering::SeqCst);
            }
        }
    }

    // minimal no-op handlers for shell related objects
    impl Dispatch<wl_seat::WlSeat, ()> for RegistryState {
        fn event(
            _state: &mut RegistryState,
            _proxy: &wl_seat::WlSeat,
            _event: wl_seat::Event,
            _data: &(),
            _conn: &Connection,
            _qh: &QueueHandle<RegistryState>,
        ) {}
    }
    impl Dispatch<wl_shell::WlShell, ()> for RegistryState {
        fn event(
            _state: &mut RegistryState,
            _proxy: &wl_shell::WlShell,
            _event: wl_shell::Event,
            _data: &(),
            _conn: &Connection,
            _qh: &QueueHandle<RegistryState>,
        ) {}
    }
    impl Dispatch<wl_shell_surface::WlShellSurface, ()> for RegistryState {
        fn event(
            _state: &mut RegistryState,
            _proxy: &wl_shell_surface::WlShellSurface,
            _event: wl_shell_surface::Event,
            _data: &(),
            _conn: &Connection,
            _qh: &QueueHandle<RegistryState>,
        ) {}
    }

    // xdg toplevel handlers: respond to ping, no-op for surface/toplevel events
    impl Dispatch<xdg_wm_base::XdgWmBase, ()> for RegistryState {
        fn event(
            _state: &mut RegistryState,
            proxy: &xdg_wm_base::XdgWmBase,
            event: xdg_wm_base::Event,
            _data: &(),
            conn: &Connection,
            _qh: &QueueHandle<RegistryState>,
        ) {
            if let xdg_wm_base::Event::Ping { serial } = event {
                // reply pong
                let _ = proxy.pong(serial);
                // flush to ensure delivery
                let _ = conn.flush();
            }
        }
    }
    impl Dispatch<xdg_surface::XdgSurface, ()> for RegistryState {
        fn event(
            _state: &mut RegistryState,
            proxy: &xdg_surface::XdgSurface,
            event: xdg_surface::Event,
            _data: &(),
            _conn: &Connection,
            _qh: &QueueHandle<RegistryState>,
        ) {
            if let xdg_surface::Event::Configure { serial } = event {
                // Acknowledge configure so the compositor can map the surface
                let _ = proxy.ack_configure(serial);
                let _ = _conn.flush();
                println!("libkagami: xdg_surface configure acked (serial={})", serial);
            }
        }
    }
    impl Dispatch<xdg_toplevel::XdgToplevel, ()> for RegistryState {
        fn event(
            _state: &mut RegistryState,
            _proxy: &xdg_toplevel::XdgToplevel,
            _event: xdg_toplevel::Event,
            _data: &(),
            _conn: &Connection,
            _qh: &QueueHandle<RegistryState>,
        ) {}
    }

    fn connect_wayland() -> Result<(Connection, EventQueue<RegistryState>, GlobalList), String> {
        let conn = Connection::connect_to_env().map_err(|e| format!("Wayland connect failed: {}", e))?;
        let (globals, event_queue) = registry_queue_init::<RegistryState>(&conn)
            .map_err(|e| format!("registry init failed: {:?}", e))?;
        Ok((conn, event_queue, globals))
    }

    /// wl_shm を使って匿名ファイル + mmap を作り、Pool と Buffer を返す。
    /// 返り値: (tempfile, mmap, pool, buffer)
    fn create_shm_buffer(
        shm: &wl_shm::WlShm,
        qh: &QueueHandle<RegistryState>,
        width: i32,
        height: i32,
    ) -> Result<(File, MmapMut, WlShmPool, wl_buffer::WlBuffer), String> {
        let stride = (width * 4) as usize;
        let size = stride.checked_mul(height as usize).ok_or("size overflow")?;

        // 匿名テンポラリファイルを作る
        let tmp = tempfile().map_err(|e| format!("tempfile failed: {}", e))?;
        tmp.set_len(size as u64)
            .map_err(|e| format!("set_len failed: {}", e))?;

        // mmap
        let mmap = unsafe { MmapMut::map_mut(&tmp).map_err(|e| format!("mmap failed: {}", e))? };

        // pool と buffer
        let fd = tmp.as_raw_fd();
        // create BorrowedFd from raw fd
        let bfd = unsafe { BorrowedFd::borrow_raw(fd) };
        // Attempt to create pool/buffer using current API (requires QueueHandle)
        let pool = shm.create_pool(bfd, size as i32, qh, ());
        // Use XRGB8888 to avoid alpha blending issues on compositors that ignore alpha
        let buffer = pool.create_buffer(0, width, height, stride as i32, Format::Xrgb8888, qh, ());

        Ok((tmp, mmap, pool, buffer))
    }


    /// 高レベル表示管理: compositor/shm および EventQueue を保持する
    pub struct HostDisplay {
        conn: Connection,
        event_queue: EventQueue<RegistryState>,
        globals: GlobalList,
        compositor: wl_compositor::WlCompositor,
        shm: wl_shm::WlShm,
    }

    // Helper to register input handlers
    #[allow(dead_code)]
    pub fn register_pointer_and_keyboard(
        host: &mut HostDisplay,
        pointer_cb: Option<Arc<dyn Fn(f64,f64) + Send + Sync>>,
        keyboard_cb: Option<Arc<dyn Fn(u32, wayland_client::WEnum<wl_keyboard::KeyState>) + Send + Sync>>,
    ) -> Result<(), String> {
        let qh = host.event_queue.handle();
        // bind seat
        let seat = host.globals.bind::<wl_seat::WlSeat, RegistryState, ()>(&qh, 1..=1, ())
            .map_err(|_| "seat not available".to_string())?;
        if let Some(pcb) = pointer_cb {
            let ud = Arc::new(PointerHandler(pcb));
            let _pointer = seat.get_pointer(&qh, ud.clone());
        }
        if let Some(kcb) = keyboard_cb {
            let ud = Arc::new(KeyboardHandler(kcb));
            let _kb = seat.get_keyboard(&qh, ud.clone());
        }
        Ok(())
    }

    impl HostDisplay {
        /// Wayland 接続して必要なグローバル（compositor, shm）まで取得する
        pub fn new() -> Result<Self, String> {
            let (conn, event_queue, globals) = connect_wayland()?;
            // obtain a queue handle for binding
            let qh = event_queue.handle();
            // 主要なグローバルを取得
            let compositor = globals
                .bind::<wl_compositor::WlCompositor, RegistryState, ()>(&qh, 1..=4, ())
                .map_err(|_| "Compositor not available".to_string())?;
            let shm = globals
                .bind::<wl_shm::WlShm, RegistryState, ()>(&qh, 1..=1, ())
                .map_err(|_| "wl_shm not available".to_string())?;
            println!("libkagami: connected to compositor and wl_shm");
            Ok(HostDisplay { conn, event_queue, globals, compositor, shm })
        }

        /// イベントのディスパッチを行う（呼び出し側でループする）
        pub fn dispatch(&mut self) -> Result<(), String> {
            let mut st = RegistryState;
            self.event_queue
                .dispatch_pending(&mut st)
                .map(|_| ())
                .map_err(|e| format!("dispatch failed: {}", e))
        }

        /// 新しい surface と double-buffer を作る
        pub fn create_surface(&mut self, width: i32, height: i32) -> Result<HostSurface, String> {
            let qh = self.event_queue.handle();
            let surface = self.compositor.create_surface(&qh, ());
            // create buffers
            let (tmp0, mmap0, _pool0, buffer0) = create_shm_buffer(&self.shm, &qh, width, height)?;
            let (tmp1, mmap1, _pool1, buffer1) = create_shm_buffer(&self.shm, &qh, width, height)?;
            let hs = HostSurface {
                surface,
                conn: self.conn.clone(),
                qh,
                width,
                height,
                stride: (width * 4) as usize,
                mmap0,
                mmap1,
                _tmp0: tmp0,
                _tmp1: tmp1,
                buffer0,
                buffer1,
                front: 0,
            };
            // Do not attach a buffer yet when creating the surface. When using xdg,
            // attaching a buffer before the xdg_surface configure is an error on some compositors.
            println!("libkagami: created surface ({}x{}), buffers allocated", width, height);
            Ok(hs)
        }

        /// Try to make a surface a toplevel using wl_shell (best-effort)
        pub fn set_toplevel(&mut self, hs: &mut HostSurface) -> Result<(), String> {
            let qh = self.event_queue.handle();
            // Prefer xdg_wm_base (modern) if available
            if let Ok(xdg) = self.globals.bind::<xdg_wm_base::XdgWmBase, RegistryState, ()>(&qh, 1..=1, ()) {
                let xsurf = xdg.get_xdg_surface(&hs.surface, &qh, ());
                let toplevel = xsurf.get_toplevel(&qh, ());
                // set title and app_id for compositor policies
                let _ = toplevel.set_title("ViewKit".to_string());
                let _ = toplevel.set_app_id("ViewKit".to_string());
                // hint min size to avoid some compositors refusing to map
                let _ = toplevel.set_min_size(hs.width, hs.height);
                // Commit role assignment; do not attach a buffer until we receive and ack the
                // xdg_surface.configure event, otherwise some compositors will error.
                hs.surface.commit();
                self.conn.flush().map_err(|e| format!("conn flush failed: {}", e))?;
                // Wait for compositor to send configure and our Dispatch will ack it.
                let mut st = RegistryState;
                let _ = self.event_queue.roundtrip(&mut st).map_err(|e| format!("roundtrip failed: {}", e))?;
                // After configure/ack, ensure the buffer we rendered into becomes the
                // front buffer and is attached. Use swap_and_commit which flips the
                // back buffer into front and commits it.
                hs.swap_and_commit().map_err(|e| format!("initial buffer attach failed: {}", e))?;
                println!("libkagami: requested xdg_wm_base xdg_surface/xdg_toplevel and attached buffer (via swap)");
                return Ok(());
            }
            // fallback to wl_shell
            match self.globals.bind::<wl_shell::WlShell, RegistryState, ()>(&qh, 1..=1, ()) {
                Ok(wl_shell) => {
                    let shell_surface = wl_shell.get_shell_surface(&hs.surface, &qh, ());
                    shell_surface.set_toplevel();
                    self.conn.flush().map_err(|e| format!("conn flush failed: {}", e))?;
                    println!("libkagami: requested wl_shell.set_toplevel");
                    Ok(())
                }
                Err(_) => {
                    println!("libkagami: no toplevel protocol available; surface may not be mapped as window");
                    Ok(())
                }
            }
        }
    }

    /// Surface と double-buffer の小さなラッパ
    pub struct HostSurface {
        surface: WlSurface,
        conn: Connection,
        qh: QueueHandle<RegistryState>,
        width: i32,
        height: i32,
        stride: usize,
        mmap0: MmapMut,
        mmap1: MmapMut,
        _tmp0: File,
        _tmp1: File,
        buffer0: wl_buffer::WlBuffer,
        buffer1: wl_buffer::WlBuffer,
        front: usize,
    }

    impl HostSurface {
        /// Width accessor
        pub fn width(&self) -> i32 { self.width }
        /// Height accessor
        pub fn height(&self) -> i32 { self.height }
        /// Stride accessor
        pub fn stride(&self) -> usize { self.stride }

        /// 書き込み可能バッファスライスを取得
        pub fn back_buffer_mut(&mut self) -> &mut [u8] {
            if self.front == 0 { &mut self.mmap1[..] } else { &mut self.mmap0[..] }
        }

        /// 現在のフロントを attach + commit する
        pub fn commit_front(&mut self) -> Result<(), String> {
            if self.front == 0 {
                self.surface.attach(Some(&self.buffer0), 0, 0);
                self.front = 0;
            } else {
                self.surface.attach(Some(&self.buffer1), 0, 0);
                self.front = 1;
            }
            self.surface.damage_buffer(0, 0, self.width, self.height);
            self.surface.commit();
            let res = self.conn.flush().map_err(|e| format!("conn flush failed: {}", e));
            match self.front {
                0 => println!("libkagami: commit_front -> front=0 attached buffer0"),
                1 => println!("libkagami: commit_front -> front=1 attached buffer1"),
                _ => println!("libkagami: commit_front -> front={} (unknown)", self.front),
            }
            res
        }

        /// バッファをスワップして commit（back を front にする）
        pub fn swap_and_commit(&mut self) -> Result<(), String> {
            if self.front == 0 {
                // front 0 -> use buffer1 as new front
                self.mmap1.flush().map_err(|e| format!("mmap flush failed: {}", e))?;
                self.surface.attach(Some(&self.buffer1), 0, 0);
                self.front = 1;
            } else {
                self.mmap0.flush().map_err(|e| format!("mmap flush failed: {}", e))?;
                self.surface.attach(Some(&self.buffer0), 0, 0);
                self.front = 0;
            }
            self.surface.damage_buffer(0, 0, self.width, self.height);
            self.surface.commit();
            let res = self.conn.flush().map_err(|e| format!("conn flush failed: {}", e));
            match self.front {
                0 => println!("libkagami: swap_and_commit -> front=0 attached buffer0"),
                1 => println!("libkagami: swap_and_commit -> front=1 attached buffer1"),
                _ => println!("libkagami: swap_and_commit -> front={} (unknown)", self.front),
            }
            res
        }

        /// request a frame callback; provided AtomicBool is set true when done
        pub fn request_frame(&mut self, flag: Arc<AtomicBool>) -> Result<(), String> {
            // create frame callback with AtomicBool userdata so RegistryState::Dispatch handles Done
            let cb = self.surface.frame(&self.qh, flag.clone());
            let _ = cb;
            Ok(())
        }
    }

    // エクスポート
    pub use HostDisplay as host_HostDisplay;
    pub use HostSurface as host_HostSurface;
}

#[cfg(not(unix))]
mod stub_impl {
    // mochiOS向けスタブ
    pub fn host_connect_wayland() -> Result<(), String> {
        Err("libkagami host shim is only available on unix hosts".into())
    }
    pub fn host_create_shm_buffer(_: &(), _: i32, _: i32) -> Result<(), String> {
        Err("libkagami host shim is only available on unix hosts".into())
    }
}

#[cfg(not(unix))]
pub use stub_impl::*;
// 公開インターフェース
#[cfg(unix)]
pub use unix_impl::{host_HostDisplay, host_HostSurface, register_pointer_and_keyboard};
