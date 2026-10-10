# M14U — completed-scan optical boundary

## Evidence and scope

Direct HP-67 Teenix notes https://literature.hpcalc.org/community/classic-notes.pdf, printed pages 75-77, distinguish shared sign anodes E/G and normal/decimal dwell. M14T already consumes the approximate 40/30 us dwell constants. M14U connects this result to the existing actual panel, preserving geometry.

Raw scan slot 3 E maps to the physical mantissa minus G and slot 3 G maps to the exponent minus G, using the previously validated structural routing. Slots 1/15 both target exponent units; slot 15 replaces slot 1, matching the previous mask renderer. This compatibility choice is not a proven physical discard or summing rule. All sixteen possibilities of analog current/propagation are outside the model; no current waveform is invented.

## Behavior

The live optical getter projects only the last complete fifteen-slot scan. UI renders nonzero per-emitter dwell with the existing artwork gain. A shorter decimal pulse is retained numerically but does not imply proportional brightness: emitter area and current remain uncalibrated. Firmware DISPLAY disable blanks immediately; power reset clears M14T history. Existing immediate mask state remains private for prompt detection and test diagnostics. No host text/number formatting or application-specific behavior is introduced.

## Validation

Projection tests exercise both signs, decimal, exponent and duplicate overwrite; live real-firmware tests check nonzero completed output, consistency at idle, atomic publication, disable and power reset; renderer tests lock geometry selection from dwell. Owner runs warnings/all-targets/release, existing diagnostics and benchmarks. GitHub Actions only formats/checks/saves Rust.

## Remaining boundaries

This remains WORKING APPROXIMATION scan-integrated nominal dwell, not exact PHI-driven light/current integration. Electrical STR/RCD waveforms, analog brightness and actual duplicate optical contribution need stronger captures before refinement. DATA bias/drive and CRC magnetic serialization remain separate source-dependent slices.
