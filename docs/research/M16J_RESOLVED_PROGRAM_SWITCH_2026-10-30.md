# M16J — resolved physical RUN/W-PRGM input, delivery 13/15

Date: 2026-10-30
Base: owner-validated/integrated 774f0eb408e98da9469204ec7f350f99fa4bd6e6 (M16I)
Branch: agent/m16j-resolved-program-switch
Status: implementation prepared; owner full runtime/performance gate pending.

## Evidence inspected

The HP-67 schematic https://www.hpcc.org/cdroms/schematics5.1/handhelds/classic/hp67.pdf
was downloaded, all nine pages visually inspected, and relevant sheets re-rendered
upright for the switch and CRC input labels. SHA256:
c567d6fdf2a924c998a395fe05d593d6466ef9bbf488ff0e9845ecb5f8b09e73.

PDF page 6, HP-67 keyboard/display PCB sheet 1 (00067-60002), shows the grounded
PGM/RUN switch and barred PGMSW connection. PDF page 5 lists that connector;
PDF page 1, logic PCB sheet 2 (00067-60003), routes barred PGMSW to CRC 1820-1751
SEL. Existing independent CRC semantics and real-firmware M12 regressions identify
CRC external flag 1 as the PROGRAM input. No emulator is used as pin-timing proof.

Active-low PROGRAM selection follows the switch/label evidence. Normalized High/RUN
output, passive bias/thresholds, voltage, switch bounce and exact CRC/PHI latch
edge remain WORKING APPROXIMATION or SOURCE-BLOCKED as appropriate. Mechanical
position persistence on electronic reset is also required explicitly by AGENTS.md;
the UI already retains its physical slider and reapplies it on power-on. M16J
makes that persistence part of the live adapter for headless/core reset too.

## Ownership and lifecycle

Physical selector -> checked External PGMSW source -> resolved digital receiver ->
CRC frozen external input -> real delayed test-and-clear -> F2/ACT -> firmware
mode entry/exit. set_program_mode publishes a contact and updates its observer;
it never changes ACT S11, enters program mode in host code or edits program RAM.

The source holds its physical position and validates the old conductor before
publishing. Exactly one External contribution and binary resolution are required;
agreeing foreign drives, contention, missing/replaced source and tampered output
fail. Repeated unchanged UI calls do not publish or create capture rows.

At word entry, selector/source and observer checks precede input freezing, oracle
execution and pending DATA tails. freeze_inputs_with_contacts samples MTRS and
PGMSW, checks both observer inputs/internal latches/word phase, and only then
replaces the CRC frozen input array. PGMSW and card-present are conductor-owned;
other unmigrated external flags retain their previous reference snapshot policy.
The older logical and MTRS-only APIs remain explicit standalone/reference fixtures.

Reset preserves the physical selector position, clears the electrical fabric and
electronic machine, reconstructs the source in that position and restores only
the external input observer. Internal CRC control latches, ACT status/registers,
RAM and pipelines still reset. A corrupted observer or conductor cannot move the
physical selector through reset. Explicit reference composition releases PGMSW and
retains its logical selector position on reset for the same mechanical contract.

Existing direct PROGRAM stimuli in the full card-write fixture and native helper
now use the same physical API as the application. Existing M12 program entry,
editing and execution tests consequently exercise the resolved selector input.
No tests or assertions are removed or weakened.

## Regressions

Five ordinary library tests cover all 8192 opcode/qualifier/PROGRAM/card-present
combinations against independent CRC and resolved F2; sticky physical position
through test-and-clear; a frozen input surviving a mid-word source change until
the next freeze; all foreign contributors/polarities; missing/replaced/tampered
sources and failed atomic freezes; and legacy eighteen-pin traces retaining an
absent PGMSW as unknown. The legacy fixture has two valid PHI rows. One capture-
feature test checks changed-only position rows, no PHI time advancement and exact
TSV roundtrip. Ordinal publication is not a measured selector edge.

Five native tests cover pending RAM-tail preservation/retry, all foreign sources
before word/position mutations, observer substitution and reset recovery in both
positions, real firmware RUN/PROGRAM transitions and repeated power cycles with
word-for-word ACT/CRC/RAM parity, and released explicit reference composition.
Immediate source publication leaves ACT unchanged; firmware later acquires S11.
The existing full 34-record write/read test also checks PGMSW after every word.

## Capture and performance

PGMSW appends at index 18 for nineteen optional named pins. Prior net/driver indices
remain stable. Missing observations stay None for level/ownership and re-export
as unknown; CLI name selection follows the existing inventory. See PIN_CAPTURE_v1.5.md.
Default builds still compile capture storage/hooks out.

No per-bit participant or extra tick is introduced; 56 cells and 224 PHI transitions
remain. Work is changed-only position publication plus word-boundary validation and
snapshot. Fixed staging capacity is 19 x 13 = 247, below 255; the existing compile-
time guard blocks a larger unsupported inventory. No cache or zero-cost claim.

Owner M16I baseline: live 2.703 us/word (118.37x), best 2.674/worst 2.728; full
structural 1.313, IS fetch 0.923, PHI 0.350, autonomous RAM 1.094. Live idle includes
the new selector; synthetic reference rows do not claim this production path.
Workload loops/order/sizes remain unchanged. tools/gate_m16j.ps1 retains the nine
owner-only stages and both warning-denied builds/12-of-12 diagnostics. Actions is
fmt/check/save only. Runtime/performance acceptance remains pending.
