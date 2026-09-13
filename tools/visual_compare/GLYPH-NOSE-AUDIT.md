# Glyph and lower-face audit

The supplied upper-keyboard close-up and lower-nameplate crop are the visual
references for this pass. These are reconstruction contours, not original HP
manufacturing artwork. The original print is not a uniform modern font.

## Corrections

- Mathematical x and y: dedicated curved outlines, including the hooked y tail.
  Applied to reciprocal, radical, powers, exchanges, comparisons and CLx.
- Reciprocal: retains the smaller raised 1 and lower x in both printed locations.
- Exchange and conversion marks: shortened from 1.04 to 0.50 legend heights;
  exact outline advances place letters close to the mark at either print size.
  Upper and lower arrows retain the corresponding yellow/blue conversion colors.
- Powers: measured base/exponent advances, closer raised exponents. The radical
  is one continuous hooked stroke. Pi uses curved vector outlines.
- Sigma-plus: corrected relative sizes. CLx uses a capital-height script x.
- Upper key text and inter-row legends: lighter weight; black skirt legends
  remain heavier. Reviewed digits, arithmetic signs, inverse trig exponents,
  comparison signs, roll triangles/arrows, x-bar, percent and punctuation.
- Lower nameplate: a separate shaded falling face, a fold edge, inward-sloping
  sides and a recessed border. All artwork shares its projection, including
  the circular slanted hp emblem, dark/blue badge fields, tracked maker name
  and unspaced 67. No system-font text remains in the branding.

## Visual iterations and reproduction

First inspection: `target/visual/glyph-nose-pass1/660x1240.png`, with enlarged
`glyphs.png` and `nose.png`. Inspection prompted a larger plus in sigma-plus
and correction of the pi character passed to its new outline renderer.
The lower-keyboard inspection also tightened comparison terms while separating
the yellow/blue groups and raised the skirt power to clear its lower edge.
Final inspection: `target/visual/glyph-nose-final/` at 165x310, 330x620 and
660x1240. These captures use the actual production meshes; CPU sampling differs
from the native GPU's multisampling. Enlargements are for inspection only.

```powershell
$env:HP67_CAPTURE_DIR='target/visual/glyph-nose-final'
$env:HP67_CAPTURE_SIZES='165x310,330x620,660x1240'
cargo test capture_panel -- --ignored --nocapture
python tools/visual_compare/compare.py target/visual/glyph-nose-final
```

The existing registered-photo overlay is useful for overall alignment. Its
rectangular homography cannot independently certify the new lower-face slope
or rank letter shapes against the new user-supplied close-ups.

Validation: 15 tests pass, plus the opt-in capture. Tests cover compact exchange
spacing, filled x crossings, y descenders, pi outlines, reciprocal baseline,
all 35 key clicks and invariant pressed artwork across scales and DPI densities.
