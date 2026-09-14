# hp67emu

Rust/egui HP-67 emulator project focused on hardware fidelity.

## Current front panel

The production UI is photorealistic rather than vector-drawn:

- `assets/hp67.png` is the calculator body/front-panel source image.
- The image is compiled into the executable with `include_bytes!`, so the built EXE does not require an external PNG at runtime.
- Keys use photographed keycaps with animated mechanical travel.
- OFF/ON and W/PRGM/RUN use animated photographed slide switches.
- The LED display is rendered over the photographed glass using the measured HP Classic-series 15-position display geometry.
- The old full vector chassis/keyboard/material renderer and its lettering-generation assets have been removed.

The calculator execution core is still a temporary UI-facing model. The intended next step is to replace formatted display text and logical key events with the real HP-67/Woodstock execution and electrical scan state.

## Build

Install the stable Rust toolchain and run:

```powershell
cargo test; cargo run --release
```

## Asset embedding

The front-panel image lives at:

```text
assets/hp67.png
```

`src/app.rs` embeds it at compile time:

```rust
include_bytes!("../assets/hp67.png")
```

Changing the PNG therefore requires recompiling, but distributing the resulting executable does not require shipping the image separately.

## Current controls

- Click OFF/ON to toggle calculator power.
- Click W/PRGM/RUN to toggle the current mode stub.
- Click the photographed keys to exercise the current UI state model.
- The display currently starts at `0.00` as a temporary stand-in for the power-on state; once the calculator core is implemented, display state should come from emulated hardware signals rather than UI formatting.
