# M16E — ACT SYNC source integrity, delivery 8/15

Date: 2026-10-25
Base: owner-validated/integrated M16D 87e01acb9bd9ab8c3aa1441cd5c98d9e567810ba
Branch: agent/m16e-sync-source-integrity
Status: code/regressions prepared; owner local runtime gate pending.

## Concrete defect

The ACT and shared RAM/CRC instruction receivers checked resolved binary SYNC
and qualifier consistency, but did not require the ACT to be its sole source.
An extra agreeing driver could remain invisible; replacing ACT with a foreign
source could supply the same instruction/data qualifier. Transport could also
re-drive a missing ACT output before detecting its disappearance. SYNC controls
normal execution versus implied-GOTO payload, RAM commands and CRC commands, so
level agreement alone is insufficient at these boundaries.

## Evidence

Existing HP-67 schematic/wiring inventory and direct HP-67 capture evidence in
HARDWARE_SOURCES.md establish ACT-generated SYNC and the instruction qualifier
window. Existing M15A review deliberately leaves exact SYNC edges unresolved.

Corroborating primary HP-97 Service Manual, section 2-14, printed page 2-4
(PDF page 18): ACT produces SYNC for ROM, PIK and CRC. The complete PDF was
retrieved, text searched and this page rendered and visually inspected:
https://www.teenix.org/97ServMan_2.pdf
SHA256 7a102cd7051ed63b23b62b18f6768083f7e4abf4e607d9aa978541dfb25c5048.
Owner-authorized HP-97 equivalence supports shared ACT/ROM/CRC SYNC identity.
The HP-97 PIK keyboard buffer described on the same page is NOT introduced
into HP-67. It is not evidence for the HP-67 ACT keyboard acquisition edge.

## Implemented contract

The shared source validator requires exactly one active SYNC contributor, ACT,
and a resolved binary level. It reports tick, word bit, level, ACT drive and
all active source identities on failure. Validation never repairs a source.
Actual ACT and RAM/CRC sampling remains at the prior provisional checkpoint;
all source/order/qualifier checks precede receiver mutation. ROM bank reception
is downstream of the ACT validation in the shared return path, preventing bank
latch updates from invalid SYNC in those compositions.

All six word/prefix composition entries validate before endpoint reset, DATA
binding or clock progression. Live validates immediately after PHI, before
contact freezing, keyboard sampling, oracle execution and pending RAM tails.
This prevents a later source launch from hiding a pre-existing missing or
foreign source. Raw fixture publication still permits intentional faults.

The pristine tick-zero undriven fixture may publish its first source image.
Only entry has this exception: receivers always require an active ACT source.
After any clock advancement, absent SYNC is an error. This is explicit simulator
initialization policy, not a physical power-on/reset waveform claim. Reset
reconstructs the same pristine composition; no new startup drive is invented.

## Error ordering and retained tests

Clock failures retain priority over SYNC failures. The DATA prefix also validates its existing start coordinate before source preflight or DATA owner selection, preserving the mid-cell rejection contract without mutation. Floating/contended receiver
samples retain their existing unresolved-level error before source validation.
Binary wrong-source/agreeing-extra images produce the new source fault. The
M15D pre-injected foreign SYNC fixture now demands rejection at tick zero rather
than waiting until b46 contention. Its receive-mask/qualifier assertions remain;
the test has a stronger fail-before-clock requirement, not a weakened check.

## Performance and fidelity limits

No added net, driver, cached field, allocation, per-bit participant or drive hook.
Normal validation uses existing dense contributor counts and the direct ACT slot;
only cold failures enumerate masks. Independent receivers each enforce their
boundary, so normal ACT/RAM/CRC qualification has additional constant-time work.
All 56 cells, 224 PHI edges and existing resolution/device operations remain.
No zero-cost or speedup claim precedes owner measurement. M16D baseline medians:
live 2.745 us/word (116.56x), full structural 1.291, PHI 0.346, RAM 1.065.

This enforces source identity in the current digital model. Normalized SYNC
High/Low output and publication timing remain the existing WORKING APPROXIMATION.
Exact physical launch/sample edges, pulse widths, thresholds and propagation
remain SOURCE-BLOCKED. No previous unknown edge-table value is filled in.

## Acceptance

Five always-enabled library tests cover pristine-only entry, all clock phases,
all twelve foreign sources at both polarities and replacement modes, every
b46..b55 position with both qualifiers across ACT/RAM/CRC/full RAM storage,
missing source and same-sample retry, staged visibility and all six composition
entries with no endpoint/pin/clock/trace mutation. One feature-only test proves
rejected transport produces no phantom pin record. Three native tests cover
all foreign sources before inputs, real pending RAM tail recovery against
unfaulted firmware, and missing/replaced ACT output after a completed word.

Compilation, tests, release, both 12/12 diagnostic builds and electrical/DATA/
RAM/live benchmarks remain owner-only. Actions runs fmt/check/save exclusively.
No merge or runtime success is claimed until the full local gate is green.
