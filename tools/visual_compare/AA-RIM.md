# Continuous white rim and selective antialiasing

The white rim is now one closed path, with a uniform 1.5 logical-unit stroke
and constant bright-metal color. It follows the upper perimeter, continues
through both folds and crosses the bottom. This replaces the shaded inset
band and independent 1.7-unit side / 1.0-unit bottom strokes, avoiding unequal
perceived thickness and disconnected corners.

Keyboard lettering now has a full physical-pixel coverage ramp instead of the
previous 0.55-pixel fringe. Normals are calculated in glyph-local coordinates,
so translation cannot change their coverage through floating-point cancellation.
Body, keycap and switch surface meshes gain an explicit physical-pixel edge
fringe. Surface normals are also calculated before screen translation.

The display uses the original surface renderer and LED geometry. No global
MSAA, display feathering or segment-glow settings were changed. The existing
4x native MSAA remains in place; the new coverage is geometry-local.

Visual passes: `target/visual/aa-rim-pass1` and `target/visual/aa-rim-final`.
Final captures are rendered at 330x620, 660x1240 and 1320x2480 with the opt-in
release capture test. Details are native-size crops, never enlarged thumbnails.
All 16 tests pass, including rigid motion of every key at multiple scales/DPI,
and vector-only rendering of the whole panel. Release build and fmt check pass.

Display verification: glass/LED crops (logical x=32..299, y=25..73)
are pixel-identical to the preceding build at all three capture scales.
