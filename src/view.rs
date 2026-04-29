use crate::render::{clear, draw_rect, Color};
use crate::{host_HostDisplay, host_HostSurface};

/// 単純なアプリビュー: HostSurface をラップして簡単なフレーム描画を行う
pub struct AppView {
    pub host: host_HostDisplay,
    pub surface: host_HostSurface,
}

impl AppView {
    /// Wayland に接続して surface を作る
    pub fn new(width: i32, height: i32) -> Result<Self, String> {
        let mut host = host_HostDisplay::new()?;
        let surface = host.create_surface(width, height)?;
        Ok(AppView { host, surface })
    }

    /// フレーム描画して表示する。シンプルに背景 + 動く矩形を描く。
    /// phase は外部で管理してアニメーションを制御する。
    pub fn render_frame(&mut self, phase: u8) -> Result<(), String> {
        let width = self.surface.width() as usize;
        let height = self.surface.height() as usize;
        let stride = self.surface.stride();

        // バックバッファを取得して描画
        let back = self.surface.back_buffer_mut();
        // 背景を塗る
        clear(back, width, height, stride, Color::new(0x10, 0x18, 0x20, 0xff));

        // 動く矩形
        let rect_w = 80;
        let rect_h = 60;
        let x = ((phase as usize) * 3) % (width.saturating_sub(rect_w));
        let y = (height / 2).saturating_sub(rect_h / 2);
        draw_rect(back, width, height, stride, x as i32, y as i32, rect_w as i32, rect_h as i32, Color::new(0xd0, 0x30, 0x30, 0xff));

        self.surface.swap_and_commit()?;
        Ok(())
    }

    /// イベントのディスパッチをホストに任せる
    pub fn dispatch_events(&mut self) -> Result<(), String> {
        self.host.dispatch()
    }
}
