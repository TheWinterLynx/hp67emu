# Exchange heads, alignment and taller caps

Exchange/conversion marks now consist of two filled opposing arrowheads without
shafts, shared by upper legends, key fronts and conversion labels. Roll/ENTER
arrows keep their distinct reference shapes. A regression requires exactly two
closed triangular paths and no line strokes for an exchange mark.

Every key is 2 logical units taller (roughly 7%): 1.5 units on the main face and
0.5 on the front skirt. Row positions and key widths are unchanged. Hit regions
follow the same KeySpec geometry. Main-face and skirt labels use the midpoint
of their respective faces. The complete cap, faces and printed marks still
translate rigidly during a press.

GSB/f, FIX/SCI, LBL/f, DSZ/(i), ISZ/(i) and inverse-trig labels are centered as
compound groups using measured vector advances. W/DATA and MERGE align to their
columns. ENTER/arrow and R/roll groups use measured widths instead of unrelated
fixed offsets. The decimal key has a centered vector dot, independent of a
font period's low baseline. The reciprocal's intentionally lowered x is retained.

Visual passes: target/visual/heads-alignment-pass1 and heads-alignment-final.
Final captures: 330x620, 660x1240, 1320x2480, plus keyboard-native.png cropped
at native pixel size. Display crops are identical to the preceding build at
all three sizes. Existing case/rim/logo work and body antialiasing are retained.

Validation: cargo fmt --check; cargo test --all-targets (17 pass, capture opt-in);
cargo build --release; release capture at all three sizes. The 35-key edge-click
and rigid-artwork tests pass across their existing scale/DPI matrix.
