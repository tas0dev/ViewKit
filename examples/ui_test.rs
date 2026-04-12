use core::arch::asm;

const IPC_BUF_SIZE: usize = 4128;
const KAGAMI_PROCESS_NAME: &str = "Kagami.app";

const OP_REQ_CREATE_WINDOW: u32 = 1;
const OP_RES_WINDOW_CREATED: u32 = 2;
const OP_REQ_FLUSH_CHUNK: u32 = 4;
const LAYER_APP: u8 = 1;
const ENODATA: u64 = (-61i64) as u64;

const SYS_YIELD: u64 = 512;
const SYS_IPC_SEND: u64 = 514;
const SYS_IPC_RECV: u64 = 515;
const SYS_FIND_PROCESS_BY_NAME: u64 = 518;
const SYS_KEYBOARD_READ: u64 = 526;
const SYS_KEYBOARD_READ_TAP: u64 = 534;

fn main() {
    println!("[VIEWKIT] ui_test start");
    let (width, height, pixels) = viewkit::build_component_catalog_frame();
    let window_id = match create_app_window(width, height) {
        Ok(id) => id,
        Err(e) => {
            eprintln!("[VIEWKIT] failed to create window: {}", e);
            return;
        }
    };
    if let Err(e) = flush_window_chunked(window_id, width, height, &pixels) {
        eprintln!("[VIEWKIT] failed to flush pixels: {}", e);
        return;
    }
    println!("[VIEWKIT] component catalog shown (window_id={})", window_id);

    loop {
        let sc_opt = match read_scancode_tap() {
            Ok(Some(sc)) => Some(sc),
            Ok(None) => read_scancode(),
            Err(_) => read_scancode(),
        };
        if let Some(sc) = sc_opt {
            if sc == 0x01 || sc == 0x81 {
                println!("[VIEWKIT] exit");
                return;
            }
        }
        yield_now();
    }
}

fn create_app_window(width: u16, height: u16) -> Result<u32, &'static str> {
    let kagami_tid = find_process_by_name(KAGAMI_PROCESS_NAME).ok_or("Kagami not found")?;
    let mut req = [0u8; 9];
    req[0..4].copy_from_slice(&OP_REQ_CREATE_WINDOW.to_le_bytes());
    req[4..6].copy_from_slice(&width.to_le_bytes());
    req[6..8].copy_from_slice(&height.to_le_bytes());
    req[8] = LAYER_APP;
    if ipc_send(kagami_tid, &req) as i64 <= 0 {
        return Err("failed to send create_window request");
    }
    let mut recv = [0u8; IPC_BUF_SIZE];
    for _ in 0..256 {
        let (sender, len) = ipc_recv(&mut recv);
        if sender != kagami_tid || len < 8 {
            yield_now();
            continue;
        }
        let op = u32::from_le_bytes([recv[0], recv[1], recv[2], recv[3]]);
        if op != OP_RES_WINDOW_CREATED {
            continue;
        }
        return Ok(u32::from_le_bytes([recv[4], recv[5], recv[6], recv[7]]));
    }
    Err("window create timeout")
}

fn flush_window_chunked(
    window_id: u32,
    width: u16,
    height: u16,
    pixels: &[u32],
) -> Result<(), &'static str> {
    let total = width as usize * height as usize;
    if pixels.len() < total {
        return Err("pixel buffer too small");
    }
    let kagami_tid = find_process_by_name(KAGAMI_PROCESS_NAME).ok_or("Kagami not found")?;
    let chunk_header_size = 20usize;
    let max_chunk_pixels = (IPC_BUF_SIZE - chunk_header_size) / 4;
    let width_usize = width as usize;
    let height_usize = height as usize;
    let chunk_w = width_usize.min(64).max(1);
    let chunk_h = (max_chunk_pixels / chunk_w).max(1);

    let mut y0 = 0usize;
    while y0 < height_usize {
        let h = (height_usize - y0).min(chunk_h);
        let mut x0 = 0usize;
        while x0 < width_usize {
            let w = (width_usize - x0).min(chunk_w);
            let mut msg = vec![0u8; chunk_header_size + (w * h * 4)];
            msg[0..4].copy_from_slice(&OP_REQ_FLUSH_CHUNK.to_le_bytes());
            msg[4..8].copy_from_slice(&window_id.to_le_bytes());
            msg[8..10].copy_from_slice(&width.to_le_bytes());
            msg[10..12].copy_from_slice(&height.to_le_bytes());
            msg[12..14].copy_from_slice(&(x0 as u16).to_le_bytes());
            msg[14..16].copy_from_slice(&(y0 as u16).to_le_bytes());
            msg[16..18].copy_from_slice(&(w as u16).to_le_bytes());
            msg[18..20].copy_from_slice(&(h as u16).to_le_bytes());
            let mut off = chunk_header_size;
            for row in 0..h {
                let src_row = (y0 + row) * width_usize;
                for col in 0..w {
                    msg[off..off + 4]
                        .copy_from_slice(&(pixels[src_row + x0 + col] | 0xFF00_0000).to_le_bytes());
                    off += 4;
                }
            }
            if ipc_send(kagami_tid, &msg) as i64 <= 0 {
                return Err("failed to send flush chunk");
            }
            x0 += w;
        }
        y0 += h;
    }
    Ok(())
}

fn read_scancode() -> Option<u8> {
    let ret = syscall0(SYS_KEYBOARD_READ);
    if ret == ENODATA {
        None
    } else {
        Some(ret as u8)
    }
}

fn read_scancode_tap() -> Result<Option<u8>, u64> {
    let ret = syscall0(SYS_KEYBOARD_READ_TAP);
    if ret == ENODATA {
        Ok(None)
    } else if (ret as i64) < 0 {
        Err(ret)
    } else {
        Ok(Some(ret as u8))
    }
}

fn find_process_by_name(name: &str) -> Option<u64> {
    let bytes = name.as_bytes();
    if bytes.is_empty() || bytes.len() > 64 {
        return None;
    }
    let ret = syscall2(SYS_FIND_PROCESS_BY_NAME, bytes.as_ptr() as u64, bytes.len() as u64);
    if ret == 0 { None } else { Some(ret) }
}

fn ipc_send(dest_thread_id: u64, data: &[u8]) -> u64 {
    syscall3(
        SYS_IPC_SEND,
        dest_thread_id,
        data.as_ptr() as u64,
        data.len() as u64,
    )
}

fn ipc_recv(buf: &mut [u8]) -> (u64, u64) {
    let ret = syscall2(SYS_IPC_RECV, buf.as_mut_ptr() as u64, buf.len() as u64);
    if (ret as i64) < 0 {
        return (0, 0);
    }
    (ret >> 32, ret & 0xFFFF_FFFF)
}

fn yield_now() {
    let _ = syscall0(SYS_YIELD);
}

#[inline(always)]
fn syscall0(num: u64) -> u64 {
    let ret: u64;
    unsafe {
        asm!(
            "int 0x80",
            inlateout("rax") num => ret,
            options(nostack, preserves_flags)
        );
    }
    ret
}

#[inline(always)]
fn syscall2(num: u64, arg0: u64, arg1: u64) -> u64 {
    let ret: u64;
    unsafe {
        asm!(
            "int 0x80",
            inlateout("rax") num => ret,
            in("rdi") arg0,
            in("rsi") arg1,
            options(nostack, preserves_flags)
        );
    }
    ret
}

#[inline(always)]
fn syscall3(num: u64, arg0: u64, arg1: u64, arg2: u64) -> u64 {
    let ret: u64;
    unsafe {
        asm!(
            "int 0x80",
            inlateout("rax") num => ret,
            in("rdi") arg0,
            in("rsi") arg1,
            in("rdx") arg2,
            options(nostack, preserves_flags)
        );
    }
    ret
}
