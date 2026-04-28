// libkagami.rs
// Host-side shim for Kagami minimal APIs.

#[cfg(unix)]
mod unix_impl {
    use memmap2::MmapMut;
    use std::fs::File;
    use std::os::unix::io::AsRawFd;
    use tempfile::tempfile;
    use wayland_client::protocol::{wl_buffer, wl_shm, wl_shm::Format};
    use wayland_client::{Display, EventQueue, GlobalManager, Main};
    use wayland_client::protocol::wl_shm_pool::WlShmPool;

    /// Wayland 接続と基本グローバルを返す。
    /// 直接 EventQueue を返すので呼び出し側は dispatch/sync_roundtrip を行ってイベントを処理できる。
    pub fn connect_wayland() -> Result<(Display, EventQueue, GlobalManager), String> {
        let display = Display::connect_to_env().map_err(|e| format!("Wayland connect failed: {}", e))?;
        let mut event_queue = display.create_event_queue();
        let attached = (*display).attach(event_queue.token());
        let globals = GlobalManager::new(&attached);
        // 初回同期でグローバルを列挙する
        event_queue
            .sync_roundtrip(&mut (), |_, _, _| {})
            .map_err(|e| format!("Wayland initial roundtrip failed: {}", e))?;
        Ok((display, event_queue, globals))
    }

    /// wl_shm を使って匿名ファイル + mmap を作り、Pool と Buffer を返す。
    /// 返り値: (tempfile, mmap, pool, buffer)
    pub fn create_shm_buffer(
        shm: &wl_shm::WlShm,
        width: i32,
        height: i32,
    ) -> Result<(File, MmapMut, Main<WlShmPool>, Main<wl_buffer::WlBuffer>), String> {
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
        let pool = shm.create_pool(fd, size as i32);
        let buffer = pool.create_buffer(0, width, height, stride as i32, Format::Argb8888);

        Ok((tmp, mmap, pool, buffer))
    }

    // 便利な小型 API をまとめた構造体
    pub struct HostSurface {
        pub width: i32,
        pub height: i32,
        pub stride: usize,
        // mmap を保持しておくことで呼び出し側が直接ピクセルデータを書ける
        pub mmap0: MmapMut,
        pub mmap1: MmapMut,
        pub buffer0: Main<wl_buffer::WlBuffer>,
        pub buffer1: Main<wl_buffer::WlBuffer>,
    }

    impl HostSurface {
        /// surface は composer.create_surface() の戻り値を表す
        /// shm must be available and caller is expected to manage event queue lifecycle
        pub fn new(
            shm: &wl_shm::WlShm,
            width: i32,
            height: i32,
        ) -> Result<Self, String> {
            // 一時的に create_shm_buffer を二回呼ぶ
            let (_tmp0, _mmap0, _pool0, _buffer0) = create_shm_buffer(shm, width, height)?;
            let (_tmp1, _mmap1, _pool1, _buffer1) = create_shm_buffer(shm, width, height)?;

            let _stride = (width * 4) as usize;

            Err("HostSurface::new is a helper; use create_shm_buffer and compositor.create_surface() in caller".into())
        }

        /// mmap を取得して描画するためのユーティリティ
        pub fn draw_into(mmap: &mut MmapMut) -> &mut [u8] {
            &mut mmap[..]
        }
    }

    // 直接小さな utils を公開
    pub use create_shm_buffer as host_create_shm_buffer;
    pub use connect_wayland as host_connect_wayland;
    pub use HostSurface as host_HostSurface;

    /// 高レベル表示管理: compositor/shm および EventQueue を保持する
    pub struct HostDisplay {
        pub display: Display,
        pub event_queue: EventQueue,
        pub globals: GlobalManager,
        pub compositor: Main<wl_compositor::WlCompositor>,
        pub shm: Main<wl_shm::WlShm>,
    }

    impl HostDisplay {
        /// Wayland 接続して必要なグローバル（compositor, shm）まで取得する
        pub fn new() -> Result<Self, String> {
            let (display, mut event_queue, globals) = connect_wayland()?;
            // 主要なグローバルを取得
            let compositor = globals
                .instantiate_exact::<wl_compositor::WlCompositor>(4)
                .map_err(|_| "Compositor not available".to_string())?;
            let shm = globals
                .instantiate_exact::<wl_shm::WlShm>(1)
                .map_err(|_| "wl_shm not available".to_string())?;
            Ok(HostDisplay { display, event_queue, globals, compositor, shm })
        }

        /// イベントのディスパッチを行う（呼び出し側でループする）
        pub fn dispatch(&mut self) -> Result<(), String> {
            self.event_queue
                .dispatch(&mut (), |_, _, _| {})
                .map_err(|e| format!("dispatch failed: {}", e))
        }

        /// 新しい surface と double-buffer を作る
        pub fn create_surface(&mut self, width: i32, height: i32) -> Result<HostSurface, String> {
            let surface = self.compositor.create_surface();
            // create buffers
            let (tmp0, mmap0, _pool0, buffer0) = create_shm_buffer(&self.shm, width, height)?;
            let (tmp1, mmap1, _pool1, buffer1) = create_shm_buffer(&self.shm, width, height)?;
            Ok(HostSurface {
                surface,
                width,
                height,
                stride: (width * 4) as usize,
                mmap0,
                mmap1,
                buffer0,
                buffer1,
                front: 0,
            })
        }
    }

    /// Surface と double-buffer の小さなラッパ
    pub struct HostSurface {
        pub surface: Main<wl_compositor::WlSurface>,
        pub width: i32,
        pub height: i32,
        pub stride: usize,
        pub mmap0: MmapMut,
        pub mmap1: MmapMut,
        pub buffer0: Main<wl_buffer::WlBuffer>,
        pub buffer1: Main<wl_buffer::WlBuffer>,
        pub front: usize,
    }

    impl HostSurface {
        /// 書き込み可能バッファスライスを取得
        pub fn back_buffer_mut(&mut self) -> &mut [u8] {
            if self.front == 0 { &mut self.mmap1[..] } else { &mut self.mmap0[..] }
        }

        /// 現在のフロントを attach + commit する
        pub fn commit_front(&mut self) {
            if self.front == 0 {
                self.surface.attach(Some(&self.buffer0), 0, 0);
                self.front = 0;
            } else {
                self.surface.attach(Some(&self.buffer1), 0, 0);
                self.front = 1;
            }
            self.surface.commit();
        }

        /// バッファをスワップして commit（back を front にする）
        pub fn swap_and_commit(&mut self) {
            if self.front == 0 {
                // front 0 -> use buffer1 as new front
                self.mmap1.flush().ok();
                self.surface.attach(Some(&self.buffer1), 0, 0);
                self.front = 1;
            } else {
                self.mmap0.flush().ok();
                self.surface.attach(Some(&self.buffer0), 0, 0);
                self.front = 0;
            }
            self.surface.commit();
        }
    }

    // エクスポート
    pub use HostDisplay as host_HostDisplay;
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

// 公開インターフェース
#[cfg(unix)]
pub use unix_impl::{host_connect_wayland, host_create_shm_buffer, host_HostSurface, host_HostDisplay};
#[cfg(not(unix))]
pub use stub_impl::*;
