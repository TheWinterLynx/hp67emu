# `src/bin/hp67_poweron_smoke.rs`

## Purpose
Runs external HP-67 firmware through the structural shared-word path and proves the documented power-on sequence reaches the no-key idle loop with the source-backed `0.00` display state while every executing word occupies one complete b0..b55 ACT lifetime.

## Why it exists
A long real-microcode smoke catches integration errors that isolated opcode and bus tests cannot. Fetch, execution lifetime and display must coexist on the same structural machine word rather than being validated as unrelated shortcuts.

## Relationships
Uses `Hp67ArchitecturalMachine`, `ActSerialEndpoint`, `RomFetchEndpoint`, `Rom0DisplayEndpoint`, `CathodeDriver1820_1749` and `FetchPipelineLatch`. Firmware is loaded from the normalized external corpus. The same resolved word carries ACT display bits b0..b7, ACT address b16..b27 and ROM response b46..b55 while the previously fetched instruction advances through b0..b55 as the current execution.

## Responsibilities
Verify startup fetch anchors, bind each pipeline execution to `ActSerialEndpoint`, reject an instruction that does not reach b55 during its structural word, execute real firmware to the documented idle landmarks, exercise the physical delayed-ROM path and validate the final fifteen-slot ROM0/cathode scan. Expected idle codes are assertions on ROM0 output, never ACT inputs.

## Implementation
Before `Hp67ArchitecturalMachine::execute_word()` applies the temporary instruction-boundary semantics, the smoke records the pre-execution instruction state and starts the corresponding `ActSerialExecution`. `fetch_cycle()` then runs the shared 56-bit transport; the ACT endpoint advances the active execution once for every b0..b55 coordinate. `verify_serial_execution_complete()` requires the active word to match `FetchPipelineLatch::executing_word()` and to be complete before the next cycle.

The idle checkpoint is evaluated after semantic execution but no longer exits before transport. The current instruction first completes its b0..b55 lifetime, then the source-backed display checkpoint is captured. Exact internal register/ALU mutation edges remain deliberately unclaimed until 1820-2530 evidence supports them.
