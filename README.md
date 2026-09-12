# hp67emu

A Rust-based Hewlett-Packard 67 emulator project.

This first milestone provides a fully vector-drawn, resolution-independent HP-67 front panel using `eframe`/`egui`. The calculator keeps its aspect ratio while the window is resized, and keys/switches are interactive.

## Current scope

- Vector chassis, bezel, display, switches and keyboard
- Logical 330 × 620 coordinate system
- Aspect-ratio-preserving resize
- Vector seven-segment LED display
- Clickable keys with a small demo input model
- Power and RUN/W/PRGM switches
- Unit tests for layout scaling and display segment mapping

The calculator execution core is intentionally separated from the UI work and can be added behind the key events/state model without changing the vector renderer.

## Build

Install the current stable Rust toolchain from <https://rustup.rs/> and then:

```powershell
git clone https://github.com/TheWinterLynx/hp67emu.git
Set-Location hp67emu
cargo test
cargo run --release
```

## Controls

- Click the left top switch area to toggle power.
- Click the right top switch area to toggle RUN/W/PRGM.
- Numeric keys enter a demo value so the vector LED display can be exercised.
- `CLx` clears the demo display.
- `CHS` changes its sign.

## Design

The reference design space is fixed at 330 × 620 logical units. Rendering uses one uniform scale:

```text
scale = min(available_width / 330, available_height / 620)
```

The panel is then centered in the available window, so it can be rendered at any DPI or window size without stretching.
