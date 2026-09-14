# Whole-calculator proportions and last-row clearance

The prior height increments had accumulated in the key constructor while widths
and row anchors still came from an older registration. Numeric keys therefore
became nearly square, and the last row approached the legends below it. This
pass removes those hidden additions and replaces all 35 key measurements as a
coherent set, together with panel legend rows, display, switches and case.

## Independent anchors

HP specifies a length of 152.4 mm and width of 81 mm in its
[HP-67/97 brochure, page 8](https://www.vintage-calculators.nl/HP-67-97-Brochure-1.pdf#page=8).
The actual case outline now has exactly that 1.881481 height/width ratio,
independently of the margins of the 330 x 620 drawing canvas. The former outline
was 614 / 320 = 1.91875, approximately 2% too narrow. Bowed sides, rounded
shoulders and the lower folded face share the corrected case envelope.

`proportion_reference.json` records raw photo bounds for nine representative
keys covering every size family, the display corners, perspective anchors and
source provenance. The photo registration's old top y=11 correspondence was
also stale; it now uses the measured front-plane anchors. Internal values have
approximately +/-2 photo-pixel edge uncertainty and raised-key parallax.

## Production changes

| Part | Before, logical units | After, logical units |
| --- | --- | --- |
| Case envelope, width x height | 320 x 614 | 326.339 x 614 |
| Numeric keys | 36 x 32.5 | 42 x 29.5 |
| Function keys | 35 x 32.5–33 | 36 x 30 |
| ENTER | 84 x 32.5 | 92 x 31 |
| Operator keys | 27 x 32.5 | 28 x 29.5 |
| Display opening | 269.2 x 49 | 282 x 57 |
| Switch label center | y=99 | y=109 |
| First key row | y=169 | y=175 |
| Last key row | y=538 | y=536.5 |

Measured examples from the registered photo are 42.75 x 29.36 for 7,
91.70 x 30.95 for ENTER and 281.60 x 56.90 for the glass. The report exposes
the individual samples rather than averaging away photographic variation.
Function and numeric columns have been realigned with their corresponding
legends. Switch labels/slots and the card rail move with the larger display.
The existing segment renderer is unchanged; its grid is centered in the new
opening. Keyboard lettering retains its vector contours and the previous
symbol-specific spacing corrections.

Paired-function gaps also follow the corrected photo scale: LN/e^x 13.5,
LOG/10^x and radical/x^2 12.25, %/%CH 11 and INT/FRAC 11.5. The earlier gap
normalization had assumed a 36-unit numeric key; these values use the new
front-plane registration and local scale instead.

The last-row key edge is now y=566. The first visible legend ink is y=572.97:
6.97 logical units of clearance, approximately 2.5 times the prior clearance.
Full 2.8-unit rigid travel still leaves 4.17 units. The tests measure actual
legend contours and the actual case outline, not just nominal constants.

## Comparison and validation

Two principal visual passes (`proportions-pass1`, `proportions-final`) plus
the final display-ratio refinement. Rendered at 165 x 310, 330 x 620,
660 x 1240 and 1320 x 2480. The matrix exporter now includes visible ink bounds
and a `profile.json` for the production case/display dimensions.

```powershell
$env:HP67_CAPTURE_DIR='target/visual/proportions-final'
$env:HP67_CAPTURE_SIZES='165x310,330x620,660x1240,1320x2480'
cargo test --release capture_panel -- --ignored --nocapture
cargo test --release export_layout_matrix -- --ignored --nocapture
python tools/visual_compare/matrix_report.py target/visual/proportions-final
python tools/visual_compare/proportions_report.py target/visual/proportions-final
```

The report reads the before capture/CSV from `target/visual/spacing-final`
(71f363e), exports measured ratios and clearances to `proportions.json`, and
produces the photo/before/after comparison, a registration overlay and a final
row crop. The updated `measurements.json` tracks the current production set.

Validation: `cargo fmt --check`, 25 tests and `cargo build --release` pass.
Two opt-in artifact exporters are also run. Existing tests retain click
coverage for every key, rigid artwork translation at multiple scale/DPI values,
vector-only rendering, and resizing without an upper limit. The case's
published ratio is enforced; the photographs cannot establish perfect
manufacturing dimensions or eliminate the remaining font/material differences.
