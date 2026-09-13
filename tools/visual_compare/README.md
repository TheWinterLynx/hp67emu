# HP-67 vector fidelity pass 1

This document records the initial pass. See [the realism follow-up](REALISM.md)
for the current outline, materials, typography, display and resize behavior.

## Reference and registration

No user photographs were available in this session's attachments. The analysis
uses [Hp-67 front.jpg on Wikimedia Commons](https://commons.wikimedia.org/wiki/File:Hp-67_front.jpg),
a 768 x 1136 photograph. Consult that page for attribution/licensing before
redistributing it. Neither the photograph nor derivative comparison images are
committed. `fetch_reference.py` downloads the exact source and checks its SHA-256.

Four manually selected inner-panel corners, clockwise from top left, are
(190,72), (610,74), (649,1078), (139,1078). They register to (30,11), (300,11),
(300,607), (30,607). `compare.py` solves the eight homography coefficients,
then resamples the photograph directly at each output resolution. Registration
is fixed across passes, never optimized to the render. No lens calibration is
claimed; the curved case and raised key faces have residual distortion/parallax.

Measurements from the rectified panel (logical units, approximately +/-1.5 px):

| Feature | Previous visible artwork | Revised measurement |
| --- | --- | --- |
| Function column centers | 65, 116, 167, 218, 269 | 58, 112, 166, 220, 274 |
| Numeric centers | 117.5, 190.5, 263.5 | 119, 194, 270 |
| Operator center / width | 60.5 / 27 | 56 / 27 |
| Numeric width | 40 | 36 |
| ENTER center / width | 90 / 84 | 87 / 84 |
| Key row tops | 169.5, 223.2, 278, 332.2, 384.6, 436.8, 489, 541.2 | 169, 223, 277, 331, 383, 435, 487, 538 |
| Display glass | separate silver rectangle, 37/18/256/77 | contiguous with inner bezel, 31/12/268/64 |
| Branding strip | 43/592/244/12 | 31/586/268/17 |

`measurements.json` records registration, guides and major bounds.
`src/ui/geometry.rs` is the application's single authoritative matrix, including
actions and artwork metadata. Photograph measurements are observations, not
manufacturing dimensions. Chassis extrema are the least reliable anchors because
they are not on the registered panel plane.

## Reproduce locally

Install Pillow in your Python environment (analysis only; no Rust runtime dependency).
From the repository root:

```powershell
python tools/visual_compare/fetch_reference.py
$env:HP67_CAPTURE_DIR='target/visual/final'; cargo test capture_panel -- --ignored
python tools/visual_compare/compare.py target/visual/final
```

For a mechanically held ENTER key:

```powershell
$env:HP67_CAPTURE_KEY='enter'; $env:HP67_CAPTURE_DIR='target/visual/pressed-enter'; cargo test capture_panel -- --ignored; Remove-Item Env:HP67_CAPTURE_KEY
```

The ignored capture test is intentionally opt-in because it writes artifacts.
It invokes the production panel and egui tessellator, then rasterizes their actual
meshes and font atlas in a small CPU renderer. There is no parallel calculator
drawing implementation. It captures 330x620 and 660x1240 after deterministic warmup
frames; the optional key uses actual pointer events and the production animation.
All production geometry remains vector based. The CPU preview approximates GPU
sampling/blending; it is not a byte-identical GPU screenshot oracle.

Comparison writes PNG renders, registered references, 50% alpha overlays, RGB
absolute differences, grayscale edge differences, geometry guides, side-by-side
images and `metrics.json`. Outputs remain in ignored `target/visual/`.

## Two iterations

Baseline was captured from commit `48367aa` before consolidation. Pass 1 unified
hit testing and artwork, applied measured centers/widths, removed the false
display frame, changed the panel/olive palette and introduced vector letters.
Pass 2 refined stroke weight, bowl curves, inter-row baselines, key highlights,
the branding rail and the tapered LED segments/decimal point.

Metrics are recorded in `results.json`. The source is unlit, while the application
shows its default zero. RGB/edge errors include that difference, photographic
texture, reflections, shadows and background. They are diagnostics, not fidelity
percentages. Pass 2's heavier lettering improves legibility but slightly increases
edge MAE relative to pass 1; it is not a monotonic improvement on every metric.

## Mechanical and functional validation

One egui response owns both activation and animation; dragging over another key
does not transfer the held state. The socket remains fixed. The complete cap,
skirt, highlights and printed artwork translate 2.8 logical units, snapped in
physical pixels using display density. Down/return durations are 50/70 ms.

Tests click all 35 keys at their outer edges and require exactly one matching
action on release. Geometry tests compare the tessellated rest/pressed artwork
for every key at three scales and three pixel densities, including intermediate
travel. Topology, colors and UVs must match; coordinates may differ only by the
expected translation (0.002 point numerical tolerance). Existing state/scaling
tests remain. No `.github/workflows` files are used or changed.

## Known differences and limits at the initial pass

This is a measured first pass, **not a museum-grade or pixel-identical reconstruction**.
The custom stroke alphabet is hand constructed, not a tracing of original HP
printing masters: widths, bowls, lowercase italics and optical kerning still
differ. It removes generic-font substitution for all key and panel-keyboard
legends; less critical switch/branding lettering still uses egui's bundled font.
The HP mark is an approximate vector reconstruction. Some arrow forms remain
triangular, and the plastic face/skirt junction is flatter than the photograph.
Colors are photographic estimates; no calibrated reflectance, texture, wear,
lens distortion, glass reflections or LED optical magnification is reproduced.
The chassis is symmetric while this photograph has perspective and parallax.

The existing calculator state remains a demo input model, not an HP-67 firmware
emulator. In particular, the existing EEX-to-Enter action is preserved. This pass
does not claim new arithmetic, programming or card-reader capabilities.
