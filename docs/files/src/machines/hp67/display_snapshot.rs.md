# `src/machines/hp67/display_snapshot.rs`

## Purpose

Provides an instruction-boundary bring-up bridge from the ACT architectural display registers to the structural HP-67 display path. It converts the already-latched ACT `A` and `B` nibbles for each observed display scan slot into the eight-bit ROM0 display code, transports that code over the resolved IS net during `b0..b7`, decodes it through the existing ROM0 model, and advances the structural fifteen-slot cathode scan.

## Why it exists

The real HP-67 firmware already reaches the documented no-key idle loop with `display_enable=true`, and the resulting architectural ACT register state is sufficient to reproduce the documented power-on `0.00` pattern. Before the final bit-serial ACT device exists, this module lets the project verify that real microcode state agrees with the source-backed ROM0/display model without bypassing the resolved electrical IS path.

This module is deliberately temporary scaffolding. It must not become the production display implementation: composing a whole display byte from architectural `A` and `B` nibbles is an instruction-boundary diagnostic shortcut, not a claim about the final ACT electrical serializer or exact PHI-relative timing.

## Relationships

Consumes `ActRegister` state from `act.rs`, the source-backed ROM0 decoder and `CathodeDriver1820_1749` from `display.rs`, the shared `Hp67ElectricalBackplane` from `machine.rs`, the `b0..b7` display-bit mapping from `timing.rs`, and the resolved `Hp67Net::Isa` wiring model. `hp67_poweron_smoke.rs` uses `structural_display_scan_from_act_registers()` as a boot-idle regression gate so that the real-firmware checkpoint also verifies the expected structural display result.

The module complements, but does not replace, the structural ACT/ROM fetch path in `fetch.rs`. A later milestone should merge display serialization and instruction fetch into one coherent word-cycle driven by the production ACT electrical state.

## Responsibilities

Map the fifteen observed HP-67 display scan slots to the correct ACT register positions; compose the diagnostic ROM0 display byte for each slot from the corresponding `A` and `B` nibbles; drive logical one as active high and logical zero as release against the evidenced weak-low IS bias; reconstruct each byte through `Rom0DisplayEndpoint`; reject IS contention and other ROM0 display errors; decode the resulting segment mask; advance the structural cathode scan; and expose a complete `StructuralDisplaySlot` record for regression checks.

It also locks the known firmware-idle `A`/`B` state to the expected code sequence `20 20 01 00 30 00 00 0F 0F 0F 0F 0F 0F 0F 20`, which decodes to the source-backed power-on `0.00` display pattern.

## Implementation

`display_register_index_for_scan_slot()` maps exponent units to ACT digit 0, exponent tens to digit 1, the shared-sign position to digit 2, mantissa scan slots to digits 13 down through 3, and the fifteenth duplicate exponent-units slot back to digit 0. `display_byte_from_act_registers()` forms the diagnostic byte as `(B[n] << 4) | A[n]`, masking both values to four bits.

`structural_display_scan_from_act_registers()` creates a structural HP-67 backplane, ROM0 display endpoint, and cathode driver. For each of the fifteen scan slots it advances one complete 56-bit structural word, drives only the evidenced display window `b0..b7`, samples the resolved IS level into ROM0, advances the four-subphase timing scaffold for each word bit, decodes the completed byte, advances the cathode slot, and records the result.

The implementation intentionally does not claim exact PHI launch/sample edges, exact STR/RCD pulse placement, analog LED current, or final bit-serial ACT behavior. Those remain separate evidence-driven milestones.
