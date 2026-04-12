use swiftlib::{ipc, task};

const IPC_BUF_SIZE: usize = 4128;
const OP_REQ_CREATE_WINDOW: u32 = 1;
const OP_RES_WINDOW_CREATED: u32 = 2;
const OP_REQ_FLUSH_CHUNK: u32 = 4;
const LAYER_APP: u8 = 1;
const KAGAMI_PROCESS_NAME: &str = "Kagami.app";

const CHUNK_HEADER_SIZE: usize = 20;
const MAX_CHUNK_PIXELS: usize = (IPC_BUF_SIZE - CHUNK_HEADER_SIZE) / 4;

pub fn create_app_window(width: u16, height: u16) -> Result<u32, &'static str> {
    let kagami_tid = task::find_process_by_name(KAGAMI_PROCESS_NAME).ok_or("Kagami not found")?;
    let mut req = [0u8; 9];
    req[0..4].copy_from_slice(&OP_REQ_CREATE_WINDOW.to_le_bytes());
    req[4..6].copy_from_slice(&width.to_le_bytes());
    req[6..8].copy_from_slice(&height.to_le_bytes());
    req[8] = LAYER_APP;
    if ipc::ipc_send(kagami_tid, &req) as i64 <= 0 {
        return Err("failed to send create_window to Kagami");
    }
    let mut recv = [0u8; IPC_BUF_SIZE];
    for _ in 0..256 {
        let (sender, len) = ipc::ipc_recv(&mut recv);
        if sender != kagami_tid || len < 8 {
            task::yield_now();
            continue;
        }
        let op = u32::from_le_bytes([recv[0], recv[1], recv[2], recv[3]]);
        if op != OP_RES_WINDOW_CREATED {
            continue;
        }
        return Ok(u32::from_le_bytes([recv[4], recv[5], recv[6], recv[7]]));
    }
    Err("window creation timed out")
}

pub fn flush_window_chunked(
    window_id: u32,
    width: u16,
    height: u16,
    pixels: &[u32],
) -> Result<(), &'static str> {
    let total = width as usize * height as usize;
    if pixels.len() < total {
        return Err("pixel buffer too small");
    }
    let kagami_tid = task::find_process_by_name(KAGAMI_PROCESS_NAME).ok_or("Kagami not found")?;
    let width_usize = width as usize;
    let height_usize = height as usize;
    let chunk_w = width_usize.min(64).max(1);
    let chunk_h = (MAX_CHUNK_PIXELS / chunk_w).max(1);

    let mut y0 = 0usize;
    while y0 < height_usize {
        let h = (height_usize - y0).min(chunk_h);
        let mut x0 = 0usize;
        while x0 < width_usize {
            let w = (width_usize - x0).min(chunk_w);
            let mut msg = vec![0u8; CHUNK_HEADER_SIZE + (w * h * 4)];
            msg[0..4].copy_from_slice(&OP_REQ_FLUSH_CHUNK.to_le_bytes());
            msg[4..8].copy_from_slice(&window_id.to_le_bytes());
            msg[8..10].copy_from_slice(&width.to_le_bytes());
            msg[10..12].copy_from_slice(&height.to_le_bytes());
            msg[12..14].copy_from_slice(&(x0 as u16).to_le_bytes());
            msg[14..16].copy_from_slice(&(y0 as u16).to_le_bytes());
            msg[16..18].copy_from_slice(&(w as u16).to_le_bytes());
            msg[18..20].copy_from_slice(&(h as u16).to_le_bytes());

            let mut off = CHUNK_HEADER_SIZE;
            for row in 0..h {
                let src_row = (y0 + row) * width_usize;
                for col in 0..w {
                    msg[off..off + 4]
                        .copy_from_slice(&(pixels[src_row + x0 + col] | 0xFF00_0000).to_le_bytes());
                    off += 4;
                }
            }
            if ipc::ipc_send(kagami_tid, &msg) as i64 <= 0 {
                return Err("flush chunk send failed");
            }
            x0 += w;
        }
        y0 += h;
    }
    Ok(())
}
