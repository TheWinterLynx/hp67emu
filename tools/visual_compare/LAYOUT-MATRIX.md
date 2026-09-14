# Production text and symbol matrix

`src/ui/legend_layout.rs` owns the panel's 38 legend cells in nine rows. Each
cell declares its row, horizontal anchor, available width and symbol-group type.
`geometry.rs` remains the source for the 35 keys and their two printed faces.
The renderer consumes both matrices; the visual report exports them directly.
There is no separate set of overlay coordinates to drift from the app.

## Specific corrections from the supplied crops

Comparison formulas previously placed each character at fixed offsets despite
the newer, larger glyphs. They now use visible glyph widths, 1.15-unit internal
gaps and a 9-unit yellow/blue group gap. Both formula groups are centered as
one unit over the arithmetic key, with identical term columns for =, !=, <, <=
and >. The four row centers follow available space between successive key rows.

Exchange marks use a 0.66-size head envelope and 0.12-size gaps to either letter.
The 7-key mark is centered on the front face instead of retaining an old upward
nudge that compensated for the previous y descender. The 5-key power is centered
as a complete expression. All exchange marks still have exactly two heads.

LN/e^x, LOG/10^x, root/x^2, conversion pairs, percent functions and INT/FRAC now
share measured group placement rather than independently nudged text centers.
Horizontal alignment uses visible ink bounds instead of advance widths/side
bearings. Mathematical e receives the same optical correction as x/y.

## Whole-panel review

The matrix covers all panel legends and the exported key table covers every
main face and skirt. The switch legends retain their shared y=99 center, the
LED retains its fixed 15-cell grid, and the maker name/model retain their shared
projected lower-face center. Those three areas do not inherit keyboard-spacing
changes. Display crops are compared byte-for-byte with the preceding capture.

Two visual passes: `target/visual/matrix-pass1` and `matrix-final`. Inspect
`matrix-report.html` for both exported tables and the annotated panel;
`matrix-overlay.png` shows legend cells/center lines, key outlines and face
centers. The final pass also fixes mathematical e's side bearing and removes
the obsolete 7-key vertical nudge after inspecting the first.

```powershell
$env:HP67_CAPTURE_DIR='target/visual/matrix-final'
$env:HP67_CAPTURE_SIZES='330x620,660x1240,1320x2480'
cargo test --release capture_panel -- --ignored --nocapture
cargo test --release export_layout_matrix -- --ignored --nocapture
python tools/visual_compare/matrix_report.py target/visual/matrix-final
```

Validation: 21 tests pass; capture/export are opt-in. Tests verify positive
formula gaps, cell containment without clipping, face containment, visible
math centering/height, all key clicks and rigid motion across scale/DPI. Fmt
and release build pass. Photo comparison remains limited by perspective and
blur; these geometric checks do not certify original manufacturing artwork.
