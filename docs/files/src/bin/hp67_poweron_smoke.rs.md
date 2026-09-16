# `src/bin/hp67_poweron_smoke.rs`

## Purpose
Runs external HP-67 firmware through the structural shared-word path and proves the documented power-on sequence reaches the no-key idle loop with the source-backed `0.00` display state.

## Why it exists
A long real-microcode smoke catches integration errors that isolated opcode and bus tests cannot. Display verification must use the same ACT-owned serial path as runtime rather than precompose A/B into a display byte.

## Relationships
Uses `Hp67ArchitecturalMachine`, `ActSerialEndpoint`, `RomFetchEndpoint`, `Rom0DisplayEndpoint`, `CathodeDriver1820_1749` and `FetchPipelineLatch`. Firmware is loaded from the normalized external corpus. The same resolved IS word carries ACT display bits b0..b7, ACT address b16..b27 and ROM response b46..b55.

## Responsibilities
Verify startup fetch anchors, execute real firmware to the documented idle landmarks, preserve the one-word pipeline, exercise the physical delayed-ROM path and validate the final fifteen-slot ROM0/cathode scan. Expected idle codes are assertions on the observed ROM0 result, never inputs to ACT transport.

## Implementation
Every runtime fetch calls `run_structural_display_fetch_cycle()` with the current scan slot and ACT state. The ACT serial endpoint chooses and emits the individual A/B source bits; the smoke harness does not call `display_byte_from_act_registers()`. At idle, the coarse cathode counter is reset only to make the fifteen-slot verification deterministic; each verification word still crosses the same shared backplane and ROM0 receiver. Exact PHI/RCD/STR edges and intra-word ACT register evolution remain explicitly outside this checkpoint.
