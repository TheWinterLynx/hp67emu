# Reference-led nose, rim and period badge

This pass uses the user's full frontal photograph and the checked-in workflow's
ignored `reference.jpg` footer crop. It corrects the earlier angular nose and
modern-looking, compressed HP mark; it does not claim access to original artwork.

The falling case face now has a continuous rounded lighting roll, almost
parallel sides and a dark lower return. The large triangular inset corners
are removed. The white rim turns inward only slightly at the fold, matching
the frontal photo instead of imposing an exaggerated trapezoid.

The old separately shaded rim inset bands are removed. One path supplies the
same metal cross-section everywhere: 4.4-unit dark surround, 3.0-unit metal,
and a 1.4-unit light core. Rendered white-core coverage is measured in the
1320x2480 capture at side rows 400, 1200 and 2200 and bottom column 660.
With coverage estimated as clamp((min(R,G,B)-145)/76,0,1), the side integrals
are 5.566, 5.553 and 5.513 physical pixels; the bottom is 5.553. This checks
actual brightness-weighted width rather than relying on equal stroke settings.

The badge's projected width/height ratio is now about 2.18 (previously 2.58).
Its silver disc is almost round after projection. The lettering is rebuilt
with narrower stems, cubic h shoulder and p bowl, replacing the heavy angular
mark. The complete name strip no longer has a bright rectangular border;
the badge retains its fine silver outline. The maker-name separator is a
centered dot, as in the reference.

Captures: `target/visual/reference-nose-pass1` and
`target/visual/reference-nose-final`, at 330x620, 660x1240 and 1320x2480.
Native-size footer crops are `nose-native.png`. All lettering remains vector;
keyboard/body antialiasing and rigid key motion are retained. The display
renderer and global antialiasing settings are unchanged.

Validation: fmt check, all-target tests (16 passed, one opt-in capture), release
compilation, production-mesh captures, rim-width sampling and unchanged display crops.
The usual release executable was locked by the running app. The alternate
`cargo rustc --release --bin hp67emu -- -o target/release/hp67emu-reference.exe`
compiled successfully (two expected output-path warnings), preserving that
session. Use hp67emu-reference.exe to inspect this revision.
