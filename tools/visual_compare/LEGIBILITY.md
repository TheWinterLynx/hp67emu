# Taller keys and reference-sized printing

Three visual passes: legibility-pass1, legibility-pass2 and legibility-final
under target/visual. The normalized reference crop uses the existing photo and
registration in compare.py; it is a comparison aid, not a pixel-perfect font
master. Perspective, blur and wear prevent certification of exact 1:1 contours.

Changes relative to the preceding commit:

- Another 1.5 logical units of cap height: 1.0 on the main face, 0.5 on the skirt.
- A-E printing: 11.9 to 13.0; function names: 11.6 to 12.5; ENTER: 10.8 to 11.8.
- Digits: 15.5 to 16.0, a smaller increase because their reference ratio was closer.
- Front legends and compound symbols: 7.5% larger as complete groups, retaining
  their heavier font weight and original symbol proportions.
- Most inter-row legends: 7.8/7.9 to 8.8. The final row remains compact to clear
  the case fold. Upper mathematical legends retain their existing sizes.

Pass 1 enlarged printing but exposed a size-dependent weight change on front
legends. Pass 2 preserves those weights through uniform local scaling. A new
geometry test then found the 7-key y descender too close to the skirt edge;
raising that exchange group 0.45 units resolves it. Side-by-side inspection
led to a smaller A-E increase and larger inter-row legends in the final pass.

Acceptance checks:

- Every opaque printed mark is inside its own main face or skirt safe rectangle
  (2 logical units of side clearance, 1 unit vertically on the main face,
  0.25 vertically on the skirt). The test checks actual tessellated geometry.
- All 35 key edge clicks dispatch correctly; pressed meshes remain identical
  apart from rigid physical-pixel translation at the existing scale/DPI matrix.
- Exchange symbols remain two heads without shafts; all rendering remains vector.
- Native captures at 165x310, 330x620, 660x1240 and 1320x2480 are inspected for
  clipping/spacing. Normal and enlarged captures are compared against the photo.
  The minimum-size capture is a geometry check; tiny raster text cannot have
  the same legibility as the normal-size physical reference.

Validation: cargo fmt --check, cargo test --all-targets (18 pass, capture opt-in),
cargo build --release, opt-in release captures and unchanged display crop checks.

Reproduce:

```powershell
$env:HP67_CAPTURE_DIR='target/visual/legibility-final'
$env:HP67_CAPTURE_SIZES='165x310,330x620,660x1240,1320x2480'
cargo test --release capture_panel -- --ignored --nocapture
python tools/visual_compare/compare.py target/visual/legibility-final
```
