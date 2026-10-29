# M16H — resolved MTRS card-insertion input, delivery 11/15

Date: 2026-10-29
Base: owner-validated/integrated b2fc0de22c464b3a3eba56f1a3d41f8556b4e5e6 (M16G v2)
Branch: agent/m16h-resolved-card-contacts
Status: owner FULL GATE GREEN v2 at b68f97facc5b3c900eef5a5e03269112065d5d91; exact validated head integrated into main (delivery 11/15).

## Evidence and limits

Reuse the inspected evidence and hashes recorded in M16G_RESOLVED_MOTOR_2026-10-26.md:
HP-67 logic PCB sheet 2 identifies MTRS at CRC 1820-1751 and the card-reader connection.
HP-97 Service Manual 2-36, printed 2-8/PDF 22, states that insertion closes the motor
switch, grounding MTRS; CRC informs firmware, which commands MOTOR. Section 2-37
separates subsequent HDS closure at the head. HP-97 equivalence is owner-authorized.
No timing, voltage or mechanical dimensions are inferred from these statements.

Active-low insertion identity is evidence-backed. A sole External High for open and
Low for closed is explicitly a WORKING APPROXIMATION for the digital contact boundary,
not proof of silicon push-pull drive, pull-up resistance, voltage thresholds or leakage.
The generic net retains Floating until explicitly attached; no passive bias is invented.
Switch bounce, exact insertion/exit geometry and CRC MTRS/PHI latch edges remain
SOURCE-BLOCKED. The existing logical insertion/completed-exit event convention and
word-frozen input policy are preserved; HDS/startup policy remains unchanged.

## Production ownership

Physical insertion -> checked retained contact publication -> resolved MTRS -> CRC
frozen input -> delayed IS/SYNC test-and-clear -> resolved F2 -> ACT S3 -> real ROM
firmware -> CRC MOTOR output -> resolved transport gate.

A receiver requires exactly one External contributor and a binary level. Same-level
foreign sources, opposition, missing/replaced source and a changed source slot are
rejected. Contact publication first verifies the old source; faults cannot erase
themselves through insertion, withdrawal or exit. Publication does not advance PHI.

At word entry, held contact/source and observer equality are checked before inputs,
oracle execution or a pending DATA tail. CRC freeze independently reads the actual
pin, compares the observer and internal CRC latches, then atomically replaces its
word input image. No observer Boolean supplies production card-present. Other
unmigrated external inputs retain their existing logical snapshot; the reference
fixture deliberately releases/disables MTRS and keeps its explicit logical path.

Insertion, unstarted withdrawal, power-off recovery and completed exit update the
contact through this path. The observer follows reception at those physical event
points. Reset attaches inactive/open while retaining the existing media-recovery
contract; this slice does not change power/card retention policy. A test-only direct
observer change now fails rather than injecting a contact into production.

M16C/M15T virtual-contact tests now close the MTRS source through the shared helper
instead of writing the mirror. Their deliberate no-media control fixture and all
assertions remain intact. Actual physical-card acceptance remains covered separately.
The old production card-present smoke test now inserts a real physical card.

## Regression coverage

Five ordinary library tests cover 4096 opcode/contact/qualifier combinations against
an independent CRC oracle and resolved F2; external contact survival through internal
flag clear; old frozen image retained until the next freeze; all twelve foreign
sources in both polarities with open/closed source; missing, replaced and changed
source; internal/external observer mismatch and late freeze with no mutation; legacy
fifteen-pin captures preserving absent MTRS as unknown. One capture-feature test checks
changed-only close/open publications, source/event timing and exact TSV roundtrip.
Its ordinal coordinates 224/448 are fixture locations, not measured contact timing.

Five native tests cover pending RAM-tail preservation and explicit retry; all foreign
sources before word/insertion mutation; rejection of observer substitution; both card
insertion ends, withdrawal/power-off fault preservation, reset/reference release;
and real card-present firmware reaching MOTOR with reference parity and 224 edges
per word. The existing complete 34-record write/read roundtrip additionally compares
resolved MTRS after each word, including card exit, without duplicating that harness.

## Capture and performance

Append MTRS at index 15; all old net/driver indices stay stable. Exports have sixteen
named pins. Optional named-column import leaves absent level/source masks None;
unknown remains unknown on re-export. CLI selection follows the existing inventory.
See PIN_CAPTURE_v1.3.md. Default builds still compile recorder storage/hooks out.

No participant is added to the 56-cell/224-transition loop. Contact work is changed-only
physical publication plus word-boundary preflight/freeze. Fabric staging capacity is
16 x 13 = 208, below the existing u8 marker bound; no persistent cache is introduced.
M16G live baseline is 2.971 us/word (107.72x), full structural 1.312, PHI 0.358,
autonomous RAM 1.077. The appended net changes fixed storage, so no free/speedup claim
is made. The live benchmark includes MTRS; the no-DATA fixture does not attach it.

The checked-in tools/gate_m16h.ps1 retains the validated nine-stage M16G v2 gate:
fmt, both focused library/native builds, warnings-denied all-target tests, both release
builds, both 12/12 diagnostics, serial electrical/DATA/RAM/live benchmarks. Actions
is fmt/check/save only. No compilation or runtime result is claimed here.

## Owner v1 feedback / fixture correction

The owner focused library run reports 4 passed / 1 failed, stopping at m16h_fifteen_pin_capture_keeps_absent_mtrs_unknown. Static inspection shows that its generated legacy capture had only one row, while import_pin_trace correctly rejects captures shorter than two rows. Version 2 supplies two valid rows with PHI1 high-to-low and consistent source masks, explicitly asserts rejection of the one-row prefix, and verifies absent MTRS level/ownership on both accepted rows plus roundtrip parity. The parser, electrical implementation, timing and remaining acceptance assertions are unchanged. The gate markers advance to v2; full owner validation remains pending and no integration is authorized by the failed run.

## Owner acceptance

Owner full gate v2 green at exact b68f97facc5b3c900eef5a5e03269112065d5d91. Both diagnostics 12/12 with unchanged cycles. Live median 2.777 us/word (115.25x), best 2.748/worst 2.799, compared with M16G 2.971 (about 6.5% lower time in this run; no isolated attribution). Full structural 1.338, IS fetch 0.933, PHI 0.355, stage 0.666, staged scheduler 0.654, autonomous RAM 1.088. Exact integration is authorized by the green marker; M16I starts there.
