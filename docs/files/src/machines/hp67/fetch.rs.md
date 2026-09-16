# `src/machines/hp67/fetch.rs`

## Purpose

Provides the structural HP-67 shared-word transport and a single ACT serial endpoint for display output, ROM-address output and ROM-word input on the resolved IS/ISA electrical net.

## Why it exists

Direct HP-67 evidence places ACT/ROM0 display traffic at b0..b7, ACT ROM address at b16..b27 and selected-ROM response at b46..b55 within the same 56-bit word. A maximum-fidelity transport must preserve both the shared physical bus and the physical ACT boundary; it must not accept a prebuilt display byte from the UI or allow the downstream cathode driver to choose which ACT digit is emitted.

## Relationships

Uses `ActDisplayWordSerializer` from `act.rs`, evidence-backed windows from `timing.rs`, wired-high/address/ROM helpers from `isa.rs`, `Rom0DisplayEndpoint` from `display.rs`, and `Hp67ElectricalBackplane` for the weak-low resolved IS net. Firmware remains behind `Hp67RomWordSource` and is not embedded in this layer. The structural 1820-1749 cathode model remains downstream of the ACT/ROM0 word transport.

## Responsibilities

`ActSerialEndpoint` owns every currently modeled ACT role on IS: the fifteen-word display phase, direct A/B display-bit drive at b0..b7, address drive at b16..b27 and returned-word sampling at b46..b55. `RomFetchEndpoint` reconstructs the twelve address bits and emits the selected ten-bit ROM word. ROM0 independently reconstructs the eight display bits from the same resolved line. Floating/contentious samples and non-RCD phase mismatches remain hard failures.

## Implementation

`ActSerialEndpoint` now starts at display slot 1 and advances its own phase after every completed display/fetch word. Slot 15 is the evidenced duplicate exponent-units word; completing it wraps the ACT phase to slot 1 and reports a coarse `rcd_after_word` boundary. `run_structural_display_fetch_cycle()` still receives the cathode slot temporarily, but that value is only a downstream consistency observation: it no longer selects the ACT source digit. Any mismatch other than an explicit observed reset to slot 1 is an error. The slot-1 resynchronization hook exists only for deterministic diagnostic scaffolding until RCD is represented as a resolved electrical net.

The returned `StructuralWordResult.display_byte` is reconstructed only on the ROM0 side and exists for observation/testing, not as ACT input. `display_scan_slot` reports the ACT-owned slot that produced the word, while `rcd_after_word` identifies the coarse end-of-slot-15 reset boundary.

The remaining fidelity boundary is explicit: each word still snapshots one architectural A/B nibble pair at its start. A real 1820-2530 shifts and modifies serial register/ALU state inside the 56-bit word. Exact intra-word ALU/register timing, PHI launch/sample edges, ROM0 sampling edge and the final STR/RCD overlap ordering remain unclaimed until hardware evidence fixes them.
