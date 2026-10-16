# `src/machines/hp67/display.rs`

## Purpose

Defines the source-backed structural HP-67 display path: ROM0 (`1818-0268`) receives the eight-bit display code on IS, decodes it to LED anodes, emits an explicit STR event, and the `1820-1749` cathode driver consumes ROM0 STR plus ACT RCD downstream.

## Why it exists

Direct HP-67 logic-analyser evidence fixes several display facts that no longer need to remain semantic guesses: display data occupies IS bit times `b0..b7` LSB-first; ROM0 decodes the byte; STR occurs at the end of that display window; RCD resets the cathode sequence; the refresh has fifteen STR slots; slot order is exponent units, exponent tens, shared signs, mantissa digits 11 through 1, then a duplicate exponent-units slot; and the ROM0 decode table is directly observed.

This layer deliberately stops short of inventing exact PHI-relative launch/sample edges, pulse widths, transistor propagation or LED current. It gives later electrical-device work a tested display-control boundary rather than deriving visible text in the UI.

## Relationships

Consumes `display_data_serial_bit()` from `timing.rs` and resolved `LogicLevel` values from the generic electrical layer. `Rom0DisplayEndpoint` sits on the same resolved IS net used by the structural ACT/ROM fetch path. `Rom0StrEvent` represents the ROM0-owned falling STR event after one complete display byte. `CathodeDriver1820_1749` consumes that STR event together with the ACT-owned coarse RCD event; it never feeds a scan slot back into ACT.

## Responsibilities

Reconstruct one complete display byte from IS `b0..b7`; reject floating/contentious/incomplete input; decode the direct HP-67 ROM0 codes; special-case the shared sign slot; preserve the fifteen observed scan roles; emit explicit structural STR events; consume ACT RCD downstream; and reject cathode phase or RCD mismatches instead of silently resynchronizing.

## Implementation

`Hp67SegmentMask` uses bits A..G/DP. `Rom0DisplayEndpoint` samples one resolved IS bit per evidenced display bit time and only produces a byte after all eight bits arrive. `str_falling_event()` emits a `Rom0StrEvent` only after a complete byte and valid scan slot. `CathodeDriver1820_1749::apply_control_edges()` validates that the ROM0 STR slot matches the downstream cathode phase, requires RCD only at the final slot, and returns to slot 1 after the observed slot-15 STR/RCD boundary without inventing ordering inside their physical overlap.

The earlier transcription of the reviewed HP-67 capture assigned `0x0A=o` and `0x0C=r`, but that assignment is inconsistent with the calculator's own firmware and documented user-visible behavior. The pinned HP-67/97 card firmware constructs the second-pass prompt by loading display codes 11, 10 and 13 for `C`, `r`, `d`; Hewlett-Packard documentation independently states that the calculator displays `Crd` when another card pass is required. Therefore production decodes `0x0A=r`, `0x0B=C`, `0x0C=o`, and `0x0D=d`. The capture's two semantic labels are treated as transposed, while its electrical timing/byte observations remain valid.

## M15G resolved display controls

The production `visit_control_bit()` bridge publishes named ROM0 STR and ACT RCD
logical contributions, validates both resolved pins and sole-contributor counts,
and records the observed High-to-Low STR transition with the resolved RCD level.
It requires a complete display byte before falling STR. All cells are checked;
same-level extra drives and opposite-level contention fail before edge emission.
`begin_word()` clears the receiver latch. Reference `str_falling_event()` remains
available for isolated structural fixtures, while production reads
`resolved_control_edges()`. Cathode application remains deferred to word completion.

The pulse bridge is a WORKING APPROXIMATION: STR is Low in b7 and High in
other cells; RCD shares that cell only in slot 15. Publishing before the four
scaffold PHI transitions and restoring High at b8 are not physical edge/width
claims. No extra tick or cathode-to-ACT feedback is added.

## M15H provisional PHI receiver scheduling

Production callers now invoke the control bridge after the PHI1-rising IS
receiver checkpoint. STR therefore falls after the final b7 display bit has
been received; it returns High at the corresponding b8 checkpoint. Final-slot
RCD uses that same provisional interval. No ordering inside the physical
STR/RCD overlap is asserted. Reference direct visitors may still exercise the
coarse bit bridge independently of production clock scheduling. A new regression
exercises control publication at the production receiver checkpoint.

This is a WORKING APPROXIMATION with a source-backed stable interval, not
a measured latch-edge/propagation claim. Full words remain 56 cells and 224
transitions. See `docs/research/M15H_PHI_RECEIVER_SCHEDULING_2026-10-16.md`.
