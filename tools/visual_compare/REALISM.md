# Follow-up: silhouette, materials, display and resizing

The user supplied an emulator screenshot, a lit frontal photograph (683x1059),
another emulator reference and a large close-up (shown at 1152x2048). These were
reviewed directly for the molded outline, key shoulders/front skirt, surface
finish, slider ribs, display die spacing and printed letter shapes. The original
downloaded Wikimedia photograph is retained for repeatable local comparisons.

## Changes

- The case and rolled rim now follow a tapered outline with rounded shoulders
  and a broader lower body. They are no longer nested rectangles. The outline
  control points are explicit in `materials.rs`; perspective and the real case's
  physical taper cannot be completely disentangled from these photographs.
- Surface-local meshes provide diffuse lighting, restrained material grain,
  shoulder reflections, front-face shading and shadows. They do not filter or
  scale a bitmap of the calculator. Noise is deterministic and moves with the
  material. The grain is intentionally subtle rather than copying dirt/wear.
- Each key is one rounded body with a sloping front and a molded recess. Static
  cap-height adjustments leave the press animation rigid. The ENTER arrow and
  white R-down mark now include shafts.
- Hand-built angular letters have been replaced with portable, open-licensed
  vector outline lettering; weights, tracking and sizes distinguish digits,
  top legends and front print. See `tools/typography/README.md` and its license.
  This is not a claim that Arimo is HP's original typeface.
- Sliders have recessed slots and black ribbed moving cursors, with larger
  printed OFF/ON and W/PRGM/RUN labels. The card rail and glass separator have
  distinct dimensions and material shading.
- The display has 15 fixed cells: mantissa sign, ten mantissa digits plus a
  separate decimal position, exponent sign and two exponent digits. Normal
  entry begins on the left. Blank positions retain barely visible dies.
  The glass is dark red/brown, and the smaller red LED segments have tapered
  ends and local glow. Scientific-format fixtures exercise the whole grid.
  This remains a display adapter over the existing demo core, not an EEX/core
  implementation. Consult the [HP handbook](https://archived.hpcalc.org/greendyk/hp67/44.html)
  for fixed/scientific display examples.
- Native resizing is explicitly enabled from 165x310, without a configured
  maximum. Aspect ratio and letterboxing remain uniform. Four-sample MSAA
  complements the analytic lettering edge coverage in the native renderer.

## Reproduce inspection captures

```powershell
$env:HP67_CAPTURE_DIR='target/visual/realism-final'; $env:HP67_CAPTURE_DISPLAY='-1.234567890e67'; $env:HP67_CAPTURE_SIZES='165x310,330x620,660x1240,1600x900'; cargo test capture_panel -- --ignored
python tools/visual_compare/compare.py target/visual/realism-final
```

The capture test can also press any real key with `HP67_CAPTURE_KEY='enter'`.
Remove those environment variables to return to default captures. The CPU
capture approximates native GPU sampling and does not emulate native MSAA.

`realism1` and `realism2` were inspected before the final capture, at canonical
size and in a wide host. The pre-existing homography straightens the old
reference's inner panel; its aggregate MAE is therefore **not a valid ranking
of the new tapered outline**. It also compares an unlit photograph to lit
display fixtures. No percentage of "1:1 fidelity" is inferred from that metric.

Tests cover the fixed decimal/sign/exponent grid, compact/4K/large scaling,
native minimum/no-maximum configuration, all 35 click targets and invariant
rest/pressed vector meshes at multiple densities. Original demo-state tests
remain in place. No workflow files or runtime dependencies were added.

Remaining limitations: the typeface, HP mark and reflectance are approximations;
glass lens magnification, photographic wear, viewing-angle-dependent reflections
and exact manufacturing dimensions are not reconstructed. The result is still
subject to comparison against a perpendicular, calibrated photograph or unit.
