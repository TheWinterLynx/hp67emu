# Uniform mathematical legend sizes

The inter-row legends now use one LEGEND_SIZE (8.8), including comparisons,
P/S and other conversions, H/H.MS, roots, powers and the bottom row. The five
upper function legends use one TOP_LEGEND_SIZE (10.5). Previously many math
helpers retained 7.4/7.5/8.0/8.4 while neighboring words had grown to 8.8.

Matching nominal size alone did not match visible size: the custom x/y ink
occupied a smaller box. Their vertical ink bounds are now fitted to the same
cap height as the row's letters, centered on the same line. The e base of e^x
is likewise matched. The key-front x/y/pi marks use the same correction within
their own print family. Advance widths remain unchanged to retain spacing.
Superscripts and the raised reciprocal 1 retain their intentional placement.

A regression measures actual opaque mesh bounds for x/y/e and requires the
shared cap height and center. Existing face-clearance checks caught the taller
power exponent nearing the top of the 5-key skirt; moving the group down
resolves it without shrinking the symbols. All 35 key-clearance/click/rigid
motion tests remain passing.

Visual passes: target/visual/uniform-legends and uniform-legends-final, rendered
at 330x620, 660x1240 and 1320x2480. lower-native.png is a native-size crop.

Validation: fmt check, all-target tests (19 passed, capture opt-in), release
compilation and captures. The running standard executable was locked, so
`cargo rustc --release --bin hp67emu -- -o target/release/hp67emu-uniform.exe`
produces the review build with two expected output-path warnings.
