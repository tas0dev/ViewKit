use crate::components::Vcomponent;
use crate::{host_HostDisplay, host_HostSurface, pipeline};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

pub struct AppBuilder {
    width: u32,
    height: u32,
    ui: Option<Vcomponent>,
}

pub struct App {
    host: host_HostDisplay,
    surface: host_HostSurface,
    ui: Vcomponent,
    width: u32,
    height: u32,
}

impl AppBuilder {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            ui: None,
        }
    }

    pub fn with_ui(mut self, ui: Vcomponent) -> Result<Self, String> {
        self.ui = Some(ui);
        Ok(self)
    }

    pub fn build(self) -> Result<App, String> {
        let mut host = host_HostDisplay::new()?;
        let mut surface = host.create_surface(self.width as i32, self.height as i32)?;
        host.set_toplevel(&mut surface)?;

        Ok(App {
            host,
            surface,
            ui: self.ui.ok_or("UI not set".to_string())?,
            width: self.width,
            height: self.height,
        })
    }
}

impl App {
    pub fn new(width: u32, height: u32) -> AppBuilder {
        AppBuilder::new(width, height)
    }

    pub fn run(mut self) -> Result<(), String> {
        // Initial render
        let html = self.ui.render();
        let css = self.ui.css();

        let rendered = pipeline::render_document(&html, &css, self.width, self.height);
        blit_framebuffer_to_surface(&rendered.framebuffer.pixels, self.surface.back_buffer_mut());
        self.surface.swap_and_commit()?;

        let frame_done = Arc::new(AtomicBool::new(false));
        let mut frame_count = 0_u32;

        loop {
            frame_done.store(false, Ordering::SeqCst);
            self.surface.request_frame(frame_done.clone())?;
            self.surface.commit_front()?;

            while !frame_done.load(Ordering::SeqCst) {
                self.host.dispatch()?;
                std::thread::sleep(Duration::from_millis(8));
            }

            frame_count += 1;
            if frame_count % 120 == 0 {
                println!("app: frame {}", frame_count);
            }
        }
    }

    pub fn ui_mut(&mut self) -> &mut Vcomponent {
        &mut self.ui
    }

    pub fn ui(&self) -> &Vcomponent {
        &self.ui
    }
}

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
