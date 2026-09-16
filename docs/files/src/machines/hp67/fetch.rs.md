# `src/machines/hp67/fetch.rs`

## Purpose

Provides the structural HP-67 shared-word transport and a single ACT serial endpoint for display output, ROM-address output and ROM-word input on the resolved IS/ISA electrical net.

## Why it exists

Direct HP-67 evidence places ACT/ROM0 display traffic at b0..b7, ACT ROM address at b16..b27 and selected-ROM response at b46..b55 within the same 56-bit word. A maximum-fidelity transport must preserve both the shared physical bus and the physical ACT boundary; it must not accept a prebuilt display byte from the UI or allow the downstream cathode driver to choose which ACT digit is emitted.

## Relationships

Uses `ActDisplayWordSerializer` from `act.rs`, evidence-backed windows from `timing.rs`, wired-high/address/ROM helpers from `isa.rs`, `Rom0DisplayEndpoint` and `Rom0StrEvent` from `display.rs`, and `Hp67ElectricalBackplane` for the weak-low resolved IS net. Firmware remains behind `Hp67RomWordSource` and is not embedded in this layer. The structural 1820-1749 cathode model is now strictly downstream of this transport.

## Responsibilities

`ActSerialEndpoint` owns every currently modeled ACT role on IS: the fifteen-word display phase, direct A/B display-bit drive at b0..b7, address drive at b16..b27 and returned-word sampling at b46..b55. `RomFetchEndpoint` reconstructs the twelve address bits and emits the selected ten-bit ROM word. ROM0 independently reconstructs the eight display bits from the same resolved line and emits the STR event. Floating/contentious samples remain hard failures.

## Implementation

`ActSerialEndpoint` starts at display slot 1 and advances its own phase after every completed display/fetch word. Slot 15 is the evidenced duplicate exponent-units word; completing it wraps the ACT phase to slot 1 and returns a coarse `rcd_falling=true` event. `run_structural_display_fetch_cycle()` no longer receives any cathode state. It derives the slot only from the ACT endpoint, lets ROM0 reconstruct the display byte from resolved IS, obtains a `Rom0StrEvent` from ROM0, then returns that STR event together with the ACT-owned RCD event.

`StructuralWordResult.display_byte` remains an observation reconstructed only on the ROM0 side, never an ACT input. `str_event.scan_slot` identifies the ACT-owned word that ROM0 strobed, while `rcd_falling` identifies the coarse final-slot reset boundary.

The remaining fidelity boundary is explicit: each word still snapshots one architectural A/B nibble pair at its start. A real 1820-2530 shifts and modifies serial register/ALU state inside the 56-bit word. Exact intra-word ALU/register timing, PHI launch/sample edges, ROM0 sampling edge and the final STR/RCD overlap ordering remain unclaimed until hardware evidence fixes them.
