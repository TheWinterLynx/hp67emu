# `src/machines/hp67/fetch.rs`

## Purpose

Provides the structural HP-67 shared-word transport and a single ACT serial endpoint for display output, ROM-address output and ROM-word input on the resolved IS/ISA electrical net.

## Why it exists

Direct HP-67 evidence places ACT/ROM0 display traffic at b0..b7, ACT ROM address at b16..b27 and selected-ROM response at b46..b55 within the same 56-bit word. A maximum-fidelity transport must preserve both the shared physical bus and the physical ACT boundary; it must not accept a prebuilt display byte from the UI or smoke harness.

## Relationships

Uses `ActDisplayWordSerializer` from `act.rs`, evidence-backed windows from `timing.rs`, wired-high/address/ROM helpers from `isa.rs`, `Rom0DisplayEndpoint` from `display.rs`, and `Hp67ElectricalBackplane` for the weak-low resolved IS net. Firmware remains behind `Hp67RomWordSource` and is not embedded in this layer.

## Responsibilities

`ActSerialEndpoint` owns every currently modeled ACT role on IS: direct A/B display-bit drive at b0..b7, address drive at b16..b27 and returned-word sampling at b46..b55. `RomFetchEndpoint` reconstructs the twelve address bits and emits the selected ten-bit ROM word. ROM0 independently reconstructs the eight display bits from the same resolved line. Floating/contentious samples remain hard failures.

## Implementation

`run_structural_display_fetch_cycle()` now receives the current ACT architectural state and scan slot, not a display byte. It asks `ActSerialEndpoint` to begin a combined word; the endpoint snapshots only the selected A/B nibbles and later emits each source bit when its real b0..b7 coordinate arrives. The API therefore has no `(B << 4) | A` display payload. The returned `StructuralWordResult.display_byte` is reconstructed only on the ROM0 side and exists for observation/testing, not as ACT input.

The remaining fidelity boundary is explicit: the serializer snapshots architectural nibbles at the word boundary, while a real ACT shifts and modifies serial register state inside the word. Exact intra-word ALU/register timing, PHI launch/sample edges, ROM0 edge timing and STR/RCD overlap ordering are not claimed here.
