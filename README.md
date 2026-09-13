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


## Visual fidelity workflow

The renderer now has one measured keyboard matrix in `src/ui/geometry.rs`,
one keycap-local artwork path in `src/ui/keyboard.rs`, and portable vector
legend contours in `src/ui/glyphs.rs`. Key travel is rigid and snapped to physical
pixels, including on high-DPI displays.

See [the comparison workflow](tools/visual_compare/README.md) for the reference,
measurements, two-pass results, reproducible headless captures and known limits.
This remains a first reconstruction pass; it does not claim pixel-identical
printing or full HP-67 firmware emulation.


The [realism follow-up](tools/visual_compare/REALISM.md) adds a molded
case, surface lighting and grain, rounded key shoulders, ribbed switches and a
15-position display grid with its own decimal cell. Native windows resize from
165 x 310 with no configured maximum; the calculator retains its aspect ratio.


The [frontal-reference correction](tools/visual_compare/FRONTAL-CORRECTION.md)
replaces the exaggerated taper with gently bowed sides, corrects the display
window and places the reciprocal's x lower than its raised 1.

The [glyph and lower-face audit](tools/visual_compare/GLYPH-NOSE-AUDIT.md)
adds curved mathematical lettering, compact exchange marks and a projected
nameplate on the falling lower face, including a vector reconstruction of the
period HP badge.

The [resolution and case-face correction](tools/visual_compare/RESOLUTION-CASE-FACE.md)
integrates the falling nose into the chassis, keeps the nameplate edges straight,
and verifies vector-only rendering with higher-precision character contours.

The [antialiasing and rim pass](tools/visual_compare/AA-RIM.md) gives the white
rim a continuous, uniform stroke and adds edge coverage to keyboard/body
geometry while preserving the display rendering.

The [reference-led lower-case correction](tools/visual_compare/REFERENCE-NOSE.md)
revises the rounded fall of the nose and vintage badge proportions, and checks
the white rim's rendered coverage along both the sides and bottom.

The [exchange-head and alignment correction](tools/visual_compare/HEADS-ALIGNMENT.md)
uses shaftless exchange symbols, centers compound legends and adds a little
height to both printed key faces.
