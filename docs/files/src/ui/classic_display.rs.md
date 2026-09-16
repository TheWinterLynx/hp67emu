# `src/ui/classic_display.rs`

## Purpose
Draws photorealistic HP Classic-series LED emission directly from the physical A..G/DP masks produced by the emulated display hardware.

## Why it exists
The photographed filter and bezel are static, while the LED emitters are dynamic. The display geometry must therefore be rendered independently, but it must preserve the real module pitch and character dimensions instead of being stretched to fill an arbitrary UI rectangle.

## Relationships
Called by `panel.rs`. `panel.rs` owns registration to `assets/hp67.png`; this module owns the 5082-7400/7405-family physical dimensions and raw segment artwork. The electrical source remains `machines::hp67` via `HardwareDisplayFrame`.

## Responsibilities
Map the fifteen raw A..G/DP masks onto fifteen physical LED positions at 3.81 mm pitch, preserve the 2.794 mm character height, 1.5748 mm character width and 0.5334 mm decimal emitter diameter, and draw emitted light without formatting a number or text string.

## Implementation
`paint_segments()` takes a clipping rectangle, an optical centre and one pixels-per-millimetre factor. X and Y always use the same scale. Character centres are `(index - 7) * 3.81 mm`, so the complete fifteen-position pitch span is symmetric about the optical axis. The photographed display glass is deliberately not used as a scaling surface. Internal three-bar artwork dimensions are retained from the previous reviewed renderer but are explicitly treated as optical artwork rather than a new die-mask claim.
