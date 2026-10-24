# M16C — CRC FLAG/F2 source, resolved net and sticky ACT S3

Date: 2026-10-25
Base: owner-validated/integrated M16B cache 976e8e4fadf269d0fba22f307ba11e3d4084a613
Branch: agent/m16c-resolved-crc-flag
Delivery: 6/15
Status: code/regressions prepared; owner runtime gate pending.

## Evidence inspected directly

HP-67 logic PCB schematic:
https://www.hpcc.org/cdroms/schematics5.1/handhelds/classic/hp67.pdf
SHA256 c567d6fdf2a924c998a395fe05d593d6466ef9bbf488ff0e9845ecb5f8b09e73
PDF page 9 (logic sheet 1, ACT 1820-2530) connects the barred FLG net to F2.
PDF page 1 (logic sheet 2) identifies the barred FLG output on CRC 1820-1751.
Both pages were rendered, rotated for reading and inspected. The active-low
net name supports Low assertion/High inactivity in normalized digital logic.
It does not establish voltage thresholds, output-stage construction or delay.

Teenix, Notes on HP's Classic Calculators:
https://literature.hpcalc.org/community/classic-notes.pdf
SHA256 bdc7a403e22b03f4305d2d7b638ff4358e49e2d5ae5328f590f6946f243c66da
Printed page 55 describes ACT F2/status 3 connected to CRC flag output, successful
CRC queries asserting the flag, and microcode explicitly clearing S3 before
queries. The existing decoder, independent IS/SYNC receiver and architectural
observer already validate these logical test/set/clear semantics. No HP-97-only
keyboard/PIK feature is imported despite the owner's equivalence allowance.

## Resulting production path

Independent CRC IS/SYNC reception -> prior-word command execution -> CRC-owned
FLAG source -> resolved F2 with strict contributor checks -> retained ACT S3.
The oracle's result stays an agreement check. Its boolean no longer supplies
the production ACT condition. Failed/absent conditions leave sticky S3 intact.
SET commands drive no assertion. Internal test-and-clear behavior remains in
the independent CRC. The source bridge retains only its last published level.

Read-only preflight checks source/ownership before contacts, clocked RAM tails
or input/authority mutation. A sole wrong driver, even at the expected level,
is invalid. Opposite drives remain contended, agreeing extras remain invalid;
no fault is hidden by re-driving a source. Reset reconstructs inactive F2.
Explicit synthetic reference fixtures disable/release this bridge along with
their independent CRC; they retain their prior logical condition path.

## Approximation boundary

The live bridge publishes a positive query result after completed b55 and
releases it before the next word (or the existing b2 resumed-word boundary).
These may be separate ordinal delta publications at the same tick. This is an
explicit logical boundary pulse, NOT a calibrated duration or exact PHI edge.
The existing normalized active High/Low output contribution is a WORKING
APPROXIMATION; open-drain/passive stage, thresholds, physical pulse width, F2
launch/ACT latch edges and propagation remain SOURCE-BLOCKED. No nanoseconds
are assigned, and no existing edge-table unknown is filled with a guessed value.

## Performance and acceptance

No per-bit participant, new PHI edge, driver slot, heap allocation or pin scan
is added. The live word adds constant-size F2 preflight/release/publication and
receiver checks. Unchanged levels are not re-published. Full idle production
benchmark includes this lane; reference stress rows do not. No zero-cost or
speedup claim is made before owner measurement (previous live 3.348 us/word).

Nine regressions: four electrical CRC source tests, one retained ACT pin-port
test, three real-firmware parity/fault/recovery/reset tests and one feature-only
full-pin capture test. Source tests transmit all decoded CRC commands under
both SYNC qualifiers/contact values over real 224-transition words and preserve
one-word command delay. Full all-target suites, release, both 12/12 diagnostic
builds and electrical/DATA/RAM/live benchmarks remain owner-only. Actions formats
and checks/saves formatting only. No PAC or new application semantics added.
