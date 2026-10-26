# M16F — window-specific IS source identity, delivery 9/15

Date: 2026-10-26
Base: owner-validated/integrated M16E cache b86849d5d0f7819fc0b6f7c1c749d7f5ff8f5491
Branch: agent/m16f-is-window-owners
Status: implementation prepared; owner local gate pending.

## Concrete gap and implemented path

The old shared IS validator rejected active Low and multiple High contributors,
but accepted any sole High source. A foreign source, an ACT output in a ROM
return cell, or a nonselected ROM during a zero response bit could therefore
supply a valid High. No contention is necessary for wrong-source substitution.

All five shared sample sites (ordinary/traced, DATA with/without trace, and
clocked DATA prefix) now validate source identity before downstream consumers.
Display b0..b7 and address b16..b27 require ACT when IS is High. Return b46..b55
requires the physical ROM selected by its independently received address/bank
latches. Explicit aggregate reference fixtures require their aggregate responder.
Idle cells require no active High source. A fault includes cell, expected source
and the actual active mask. Existing phase, active-Low and overlap errors retain
priority, preserving their resolver meaning.

Only a High needs a source-role lookup. The normal path reads the expected
source's direct drive slot after the existing count/polarity check; masks are
built only on failure. Physical return identity comes from selected chip state,
never oracle bank/PC, expected instruction bits or a host result. The selected identity is recorded once in word-local transport state after b27;
return-bit checks do not repeatedly scan the chip list. No persistent identity
cache, pin, drive event or per-bit device pass is introduced.

A legitimate source may transmit changed bits: the receivers still consume the
actual resolved value. Expected payload previews never replace pins. Scalar
IS-only helpers remain explicit reference fixtures; they cannot establish
source identity without a backplane. Production uses the strict shared path.

## Evidence and limits

Uses the already locked HP-67 IS polarity/windows and five-ROM composition:
Teenix, Notes on HP's Classic Calculators, printed pp64–66, 71–72, 74;
https://literature.hpcalc.org/community/classic-notes.pdf
SHA256 bdc7a403e22b03f4305d2d7b638ff4358e49e2d5ae5328f590f6946f243c66da.
See HARDWARE_SOURCES.md, isa.rs and M15J_FIVE_ROM_ENDPOINTS_2026-10-16.md.
No part/profile map changes; ROM4=0231 retains its documented working inference.
Owner HP-97 equivalence remains accepted for the existing profile evidence.

A passive Low with all contributors High-Z remains valid zero. This cannot
identify invisible output enables, detect a disabled source transmitting zero,
or prove internal output transistor timing. No active zero or pull-up is added.
The idle rule enforces the currently modeled roles; future evidenced additional
IS roles must explicitly extend it. Exact launch/latch edges, physical ownership
handoff intervals, propagation, pulse widths and analog output stages remain
SOURCE-BLOCKED. This adds a digital composition invariant, not calibrated timing.

Faults stop at the first invalid sample after its two existing PHI transitions;
clocks are not rolled back. Receivers after the guard have not consumed that
sample. A native partial-cycle fault requires reset; it must not be retried as
if the word were untouched. RAM lifecycle may enter a new word before sampling,
but the rejected DATA tail is not consumed and storage is unchanged.

The M15B pre-injected foreign-High fixture now fails at b0/tick2 instead of
waiting for agreeing ACT overlap at b5/tick22. It retains its invalid-display
and trace assertions with the stronger earlier rejection coordinate.

## Measured cache retirement

Owner M16E PERF gate passes both 12/12 diagnostics. Same-process source kernel:
cached 0.254 vs count/slot 0.252 us/word. Live 3.023 (best 2.847, worst 3.083)
versus prior 2.942; no gain demonstrated. Other unchanged controls also varied,
including raw PHI 0.353 to 0.412 and architectural execution 0.029 to 0.035.
Do not assign all live change to the cache or claim isolated regression proof.

Retire its extra boolean and immediate/staged/bundle maintenance. The exact
sole-ACT SYNC policy remains a direct committed query, with binary-level checks,
pristine-only entry, full fault details and all independent receiver checks.
All five scan/staging/bundle/clone/reset/mixed-update tests are retained under
M16F names. The historical ignored query kernel remains explicitly labelled
cache-retired; its two paths now query counts. PHI ownership metadata is unchanged.
This avoids retaining an optimization without demonstrated benefit.

## Regressions and owner gate

Seven always-enabled library tests: every cell/source across all five ROM
profiles; aggregate versus physical identity; existing polarity/overlap/phase
errors; changed legitimate bits reaching ACT/RAM/CRC; received-address-derived
selection; prefix non-consumption; actual autonomous bank roundtrip. One
feature-only capture test preserves the wrong sole source, exact tick and two
PHI edges. Three native tests cover six persistent foreign sources/reset,
real pending DATA-tail non-consumption/storage, and 2400 traced/untraced firmware
words with keyboard contacts, all at 224 transitions each.

Both full feature suites, release builds, both diagnostic configurations and
unchanged electrical/DATA/RAM/live benchmarks are required. No runtime tests or
compilation are run by the agent; Actions formats/checks/saves only. No speedup
or zero-cost claim precedes the owner gate. This completes no analog timing gap.
