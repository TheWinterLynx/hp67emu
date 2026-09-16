# `src/bin/hp67_poweron_smoke.rs`

## Purpose
Runs external HP-67 firmware through the structural shared-word path and proves the documented power-on sequence reaches the no-key idle loop with the source-backed `0.00` display state.

## Why it exists
A long real-microcode smoke catches integration errors that isolated opcode and bus tests cannot. Display verification must use the same ACT-owned serial path as runtime rather than precompose A/B into a display byte or let the cathode choose the ACT scan slot.

## Relationships
Uses `Hp67ArchitecturalMachine`, `ActSerialEndpoint`, `RomFetchEndpoint`, `Rom0DisplayEndpoint`, `CathodeDriver1820_1749` and `FetchPipelineLatch`. Firmware is loaded from the normalized external corpus. The same resolved IS word carries ACT display bits b0..b7, ACT address b16..b27 and ROM response b46..b55. ROM0 emits STR structurally and ACT emits the coarse RCD boundary; the cathode consumes both downstream.

## Responsibilities
Verify startup fetch anchors, execute real firmware to the documented idle landmarks, preserve the one-word pipeline, exercise the physical delayed-ROM path and validate the final fifteen-slot ROM0/cathode scan. Expected idle codes are assertions on the observed ROM0 result, never inputs to ACT transport.

## Implementation
Every runtime fetch calls `run_structural_display_fetch_cycle()` with ACT state only; no cathode slot enters the transport. The ACT serial endpoint chooses and emits the individual A/B source bits, ROM0 reconstructs the display byte and returns a `Rom0StrEvent`, and the 1820-1749 consumes that STR together with ACT `rcd_falling`. At idle, the deterministic verification uses a fresh structural ACT/cathode scan pair beginning at slot 1 while retaining the same backplane and frozen architectural A/B state. Exact PHI-relative STR/RCD edges and intra-word ACT register evolution remain explicitly outside this checkpoint.
