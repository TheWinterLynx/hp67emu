# Spacing and individual symbol audit

This pass checks the user's close-up against the stored 768 x 1136 frontal
photograph, with all 38 panel groups and the two printed faces of all 35 keys
shown in a local photo/before/after report. Each crop preserves its aspect ratio;
the photograph is scaled by the width of nearby keys. Raised key parallax and
blur remain visible instead of being warped away.

## Measured gaps

Color separation of the photo's yellow and cyan ink gives the following gaps.
The adjacent numeric key is approximately 72 photo pixels wide, corresponding
to 36 logical units. Threshold/blur uncertainty is approximately +/-2 photo px.

| Pair | Photo gap, px | Prior logical gap | New logical gap |
| --- | ---: | ---: | ---: |
| LN / e^x | 23 | 5 | 11.5 |
| LOG / 10^x | 21 | 5 | 10.5 |
| radical / x^2 | 21 | 7 | 10.5 |
| % / %CH | 20 | 5 | 10 |
| INT / FRAC | 21 | 5 | 10.5 |

The matrix stores pair-specific gaps. Keyboard tracking is now 0.070 cap units
instead of 0.028, applied to both measurement and vector rendering. This opens
multi-letter labels without changing their cap height or borrowing space from
neighbors. The existing containment tests cover every key face and legend cell.

## Symbol-by-symbol decisions

| Printing | Geometry and spacing decision |
| --- | --- |
| Top x/y, 7 skirt x/y, indirect x/I, P/S | Compact opposing filled heads; retain short mark and clear letter gaps. |
| R/P, D/R, H/H.MS | Full stacked shafts: upper cyan arrow points right, lower yellow arrow points left. Wider arrow and letter gaps measured separately from compact exchanges. |
| Top R-down, ENTER-up | Full vertical arrow with shaft. |
| 8 R-down, 9 R-up | Filled triangular marks, explicit type independent of font size. Increase space after R. |
| e^x, 10^x, x^2, top/skirt y^x | Ink gap is 0.18 of nominal size, previously 0.025. Shared exponent ratio and raised center. Normalize math contours at every exponent size; e has the photo's forward slant. |
| SIN^-1, COS^-1, TAN^-1 | Separate raised minus-one run with a 1.4-unit gap instead of 0.3. |
| -x- / STK | Increase both gaps around x and the gap before STK. |
| 1/x, radical, x-bar | Retain raised 1/lower x, vector slash, radical overbar and x-bar; inspected in the contact sheet. |
| Sigma+/Sigma-, pi, arithmetic, inequalities | Existing vector forms retained after review; included in the full audit. |
| A-E, a-e, functions and key-skirt words | Shared keyboard tracking; visible-ink centering retained. ENTER's DEG stays right aligned. |

Rendered checks cover the initial spacing pass, measured-gap/exponent correction,
R/triangle and unary-gap corrections, and the radical's visible center. Canonical captures use 330 x 620,
660 x 1240 and 1320 x 2480. The display crop is checked against the preceding
matrix capture. Key geometry, rigid travel, hit regions, case and display
rendering are unchanged.

```powershell
$env:HP67_CAPTURE_DIR='target/visual/spacing-final'
$env:HP67_CAPTURE_SIZES='330x620,660x1240,1320x2480'
cargo test --release capture_panel -- --ignored --nocapture
cargo test --release export_layout_matrix -- --ignored --nocapture
python tools/visual_compare/matrix_report.py target/visual/spacing-final
python tools/visual_compare/symbol_audit.py target/visual/spacing-final
```

The audit's before image is `target/visual/matrix-final/1320x2480.png` from
cafa76b. Outputs include `symbol-report.html`, `legend-audit.png`, `key-audit.png`
and `spacing-measurements.json`. The audit script contains explicit photo crop
coordinates and color thresholds, rather than claiming an overall similarity
score from a perspective photograph.

Validation: fmt check, 23 tests and release build pass; two artifact exporters
are opt-in. New regressions render actual power ink to check clearance,
centering and exponent height, and check conversion shafts/directions/colors
separately from compact exchange heads. Existing tests cover rigid motion,
all click targets, vector-only rendering and resizing.

Remaining differences: the sans-serif contours are an Arimo-based
reconstruction, not HP's printing masters. Some stroke shapes/weights and the
photo's key perspective remain visibly different. The report exposes these
differences; correct gap ratios alone do not establish a complete 1:1 match.
