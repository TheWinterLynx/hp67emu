# HP-67 pin captures v1.2 — MOTOR control

M16G appends the CRC MOTOR conductor to the existing pin inventory. All prior
net/driver indices retain their values; MOTOR is net index 14. Full emulator
exports now contain fifteen named pins, ending KC5, MOTOR. The TSV metadata
remains hp67-pins-v1 because named, optional columns already provide compatible
extension. See PIN_CAPTURE_v1.1.md for the existing domain/ownership/CLI contract.

MOTOR uses the existing CRC 1820-1751 driver identity: Low is motor requested On,
High is inactive in the explicitly normalized logical-level bridge. This is not
an analog motor model, measured pulse width or calibrated PHI-relative edge.
The output is held between CRC motor commands. Unchanged source publication
creates no extra trace row. Changes at completed-word boundaries retain the
existing ordinal tick/delta-order semantics without advancing PHI.

Older captures without MOTOR import with None for its level and ownership.
Unknown remains unknown: it is not replaced by Off, High or Floating. Re-export
includes MOTOR as ? when it was not observed. Full new captures include actual
resolved MOTOR and contributor masks. Existing physical/synthetic sparse capture
support is unchanged. Selecting MOTOR in comparison/timing CLI uses the existing
named-pin selection machinery; absent observations cannot prove equality or
hardware timing.

Normal builds still compile recorder storage/hooks out. Both build modes execute
the same electrical MOTOR source/receiver checks and all 224 PHI transitions.
The additional net extends fixed fabric/staging storage; it does not add a
per-bit participant or change clock topology. Runtime/performance acceptance
remains the owner gate.
