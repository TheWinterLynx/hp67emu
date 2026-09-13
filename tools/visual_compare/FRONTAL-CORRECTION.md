# Frontal-reference correction

The latest user references include a near-frontal 339x589 photograph, a frontal
catalogue view and an oblique view with a magnetic card. The frontal views govern
the outline; convergence in the oblique view must not become physical taper.

The previous pass was wrong to narrow the top strongly. Its upper shoulder
endpoints were x=20/310, versus x=4/326 near the bottom. The replacement uses
matching upper/lower side endpoints at x=12/318 and a gentle 7-unit outward bow
at mid-height. The long sides are curved, not a trapezoid or a rectangular box.
The metal and inner rim share that envelope with rounded corners. These are
explicit reconstruction parameters judged against the supplied frontal views,
not certified manufacturing measurements.

The screen is now independent of the body's side envelope: x=30.4..299.6,
y=24..73, straight sides and lower edge, with small upper corner radii. This
removes the previous sloping wedge-shaped glass. The main panel material starts
below the upper corners so it cannot paint over the metal rim.

The reciprocal is composed from a smaller raised 1 (86% size, y=-1.1 units),
the diagonal slash and a lower x (98% size, y=+0.8 units), relative to a 9-unit
legend. All placements scale together. The same composition is used above A
and on the 4 key's front face. A regression test checks the *actual glyph
contour bottoms*, not just center positions. Inverse-trig exponents have also
been moved closer to their base legends.

The single key artwork path, pixel-snapped rigid animation and unrestricted
resizing above the compact minimum are unchanged. Tests protect all of them.

Reproduce the inspection capture:

```powershell
$env:HP67_CAPTURE_DIR='target/visual/outline-correction'; $env:HP67_CAPTURE_DISPLAY='0.00'; cargo test capture_panel -- --ignored
python tools/visual_compare/compare.py target/visual/outline-correction
```

The lit `0.00` is a capture fixture. Captures use the real panel renderer at
330x620 and 660x1240. The old registered reference remains useful for inspecting
keyboard spacing, but its rectangular registration is not an independent
measurement of the new body's curvature. No overall pixel-identical fidelity
claim is made; photo perspective, reflectance and fine lettering remain limits.
