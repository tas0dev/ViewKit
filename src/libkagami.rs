// libkagami.rs
// Host-side shim for Kagami minimal APIs.
// 目的: Kagami で使われる "surface/buffer" と入力の最小限の機能を Wayland 上で提供する。
// 実装は unix (Linux/WSL) 向けの簡易ラッパーで、非 unix ターゲットではエラースタブを提供する。

#[cfg(unix)]
mod unix_impl {
    use memmap2::MmapMut;
    use std::fs::File;
    use std::io::Result as IoResult;
    use std::os::unix::io::AsRawFd;
    use tempfile::tempfile;
    use wayland_client::protocol::{wl_buffer, wl_compositor, wl_shm, wl_shm::Format};
    use wayland_client::{Display, EventQueue, GlobalManager};

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
    ) -> Result<(File, MmapMut, wl_shm::WlPool, wl_buffer::WlBuffer), String> {
        let stride = (width * 4) as usize;
        let size = stride.checked_mul(height as usize).ok_or("size overflow")?;

        // 匿名テンポラリファイルを作る
        let mut tmp = tempfile().map_err(|e| format!("tempfile failed: {}", e))?;
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
        pub buffer0: wl_buffer::WlBuffer,
        pub buffer1: wl_buffer::WlBuffer,
    }

    impl HostSurface {
        /// surface は composer.create_surface() の戻り値を表す
        /// shm must be available and caller is expected to manage event queue lifecycle
        pub fn new(
            shm: &wl_shm::WlShm,
            width: i32,
            height: i32,
        ) -> Result<(Self, wl_compositor::WlSurface), String> {
            // 一時的に create_shm_buffer を二回呼ぶ
            let (tmp0, mmap0, _pool0, buffer0) = create_shm_buffer(shm, width, height)?;
            let (tmp1, mmap1, _pool1, buffer1) = create_shm_buffer(shm, width, height)?;

            // 注意: tempfile はここで捨てずに所有しておく必要があるが、呼び出し側が mmap を保持している限りファイルの寿命は維持される
            // 型上は tmp0/tmp1 を破棄してしまうとファイルがクローズされるが mmap は生きる場合がある。確実性のため呼び出し側で tmp を保持したい場合は別 API を作る。

            let stride = (width * 4) as usize;

            // surface は呼び出し側で compositor.create_surface() を呼んで得る設計にするため、ここではダミーの surface を返す
            // しかし便利のため、wl_compositor::WlSurface が必要なケースが多いので呼び出し側で create_surface してから使うことを想定する。
            // ここでは Err を返す代わりに使い方をドキュメントで示す。

            Err("HostSurface::new is a helper; use create_shm_buffer and compositor.create_surface() in caller".into())
        }

        /// mmap を取得して描画するためのユーティリティ
        pub fn draw_into<'a>(mmap: &'a mut MmapMut) -> &'a mut [u8] {
            &mut mmap[..]
        }
    }

    // 直接小さな utils を公開
    pub use create_shm_buffer as host_create_shm_buffer;
    pub use connect_wayland as host_connect_wayland;
    pub use HostSurface as host_HostSurface;
}

#[cfg(not(unix))]
mod stub_impl {
    // 非 unix（mochi ターゲット等）向けのスタブ実装
    pub fn host_connect_wayland() -> Result<(), String> {
        Err("libkagami host shim is only available on unix hosts".into())
    }
    pub fn host_create_shm_buffer(_: &(), _: i32, _: i32) -> Result<(), String> {
        Err("libkagami host shim is only available on unix hosts".into())
    }
}

// 公開インターフェース
#[cfg(unix)]
pub use unix_impl::{host_connect_wayland, host_create_shm_buffer, host_HostSurface};
#[cfg(not(unix))]
pub use stub_impl::*;
