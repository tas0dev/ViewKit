---
title: "Canvas Component"
file: "components/canvas.rs"
---

# Canvas Component

Summary

Canvas is a low-level drawable component used by ViewKit to render pixel data into a rectangular area. It is suitable for solid-color fills, simple debug visualizations, and as a primitive for custom software renderers.

## Overview

Canvas implements the Component trait and integrates with the ui_layout crate for size and layout calculation. It exposes a minimal API to fill its region with a Color or draw into the host-provided buffer when running under the host PoC (Wayland/wl_shm).

## API

Representative public API (see source for exact signatures):

```rust
pub struct Canvas { /* fields omitted */ }

impl Canvas {
    pub fn new(w: i32, h: i32, color: Color) -> Self;
    pub fn set_color(&mut self, color: Color);
    pub fn render_into(&self, buf: &mut [u8], buf_width: usize, buf_height: usize, stride: usize, x: i32, y: i32, w: i32, h: i32);
}

impl Component for Canvas { /* layout + render dispatch */ }
```

Refer to the source file listed in `file` for exact types and additional helpers.

## Usage

Quick example (host PoC / examples/ui_test.rs):

```rust
let c = Canvas::new(400, 240, Color::rgb(0x20,0x40,0x60));
let pair = c.view().padding(4).into_pair();
// add pair to a Container and drive render loop via HostDisplay/HostSurface
```

Important: under the host PoC, perform an initial render into the back buffer before attaching the buffer to the xdg surface and ack_configure (see examples/ui_test.rs and libkagami.rs).

## Examples

See `examples/ui_test.rs` for a runnable demo that composes multiple Canvas children in a Container and drives Wayland frame callbacks.

Minimal snippet:

```rust
let mut a = Canvas::new(200, 100, Color::rgb(0xff,0x00,0x00));
let mut b = Canvas::new(200, 100, Color::rgb(0x00,0xff,0x00));
let container = Container::vertical(vec![a.view().into_pair(), b.view().into_pair()]);
// layout + render
```

## Layout / Design

- Layout engine: ui_layout computes the layout boxes. Container constructs a root LayoutNode with child Styles and calls LayoutEngine::layout(root, width, height).
- Rendering model: Canvas's `render_into` draws into a supplied mutable pixel buffer for the specified rectangle. Composition is performed by copying pixels into the host buffer at child positions.
- Host specifics: HostSurface uses wl_shm with double-buffering (memmap + tempfile). The first buffer should be rendered before toplevel attach to avoid blank windows on some compositors.

## Notes and Caveats

- Pixel format: The host PoC uses XRGB8888. Be careful with alpha/premultiplied formats; current software renderer ignores alpha.
- Wayland behavior: Some compositors require ack_configure before attaching mapped buffers. Follow examples/ui_test.rs to avoid unmapped or black windows.
- Performance: Canvas is CPU-bound. For complex visuals, consider GPU-backed approaches in future.

## References

- src/apps/ViewKit/src/components/canvas.rs
- src/apps/ViewKit/examples/ui_test.rs
- src/apps/ViewKit/src/libkagami.rs
