# Resolution and integrated lower case

The lower nose now belongs to `draw_chassis`: the keyboard deck stops at the
fold, then side cheeks, a shaded falling face, continuing metal edges and the
lower return lip complete the case. The nameplate is a thinner inset on that
face, with visible case material around it. Removing the branding leaves a
complete nose. The old separate plate shadow is removed.

The old x*y nameplate projection was bilinear and could bend diagonal lines.
Its replacement is affine: baselines, borders and emblem stems remain straight.
The case facets have straight fold/return edges. The intentional bow of the
long case sides is retained from the frontal references.

All characters are vector contours. Font curves now use adaptive subdivision
with 0.10 font-unit flatness tolerance instead of fixed 12/16 steps. Custom
mathematical curves use 96 samples per cubic instead of 24. These are portable
triangle/contour coordinates, not raster pixels or a scaled texture atlas.
The physical-pixel antialias fringe does not grow with glyph size.

A new full-panel test checks every tessellated vertex at 1x and 4x for solid
white texture coordinates. It initially caught egui's prerasterized-circle
optimization, which substituted atlas discs for small circular marks. The
panel disables that optimization; the test now passes, including division and
LED decimal dots. No bitmap character rendering remains in the panel.

## Visual checks

Two passes: `target/visual/integrated-nose-pass1/` and
`target/visual/integrated-nose-final/`. The second adjusts cheek junctions and
nameplate foreshortening after inspection of the first. A further inspection
removed the old lower rolled rim behind the new nose: the upper rim is clipped
at the fold, leaving a single sloping continuation on each side.

```powershell
$env:HP67_CAPTURE_DIR='target/visual/integrated-nose-final'
$env:HP67_CAPTURE_SIZES='165x310,330x620,660x1240,1320x2480'
cargo test --release capture_panel -- --ignored --nocapture
```

Inspect `1320x2480.png`, `nose-native.png` and `glyphs-native.png` at actual
pixel size. These are freshly rendered from the vector geometry at 4x logical
scale. The detail crops are not enlarged low-resolution PNGs. A PNG itself
still has fixed pixels; use the resizable executable to assess live scaling.
CPU captures use single-sample coverage and native rendering uses 4x MSAA.

Validation: cargo fmt --check, cargo test --all-targets (16 passed, one opt-in
capture), cargo build --release, and captures at all four sizes. Existing
all-key click and rigid-press tests remain passing.
