# `src/bin/hp67_poweron_smoke.rs`

## Purpose

Runs HP-67 power-on firmware from an external normalized 5120-word corpus through one shared structural 56-bit IS word path, continues for thousands of real machine cycles using the independent ACT+CRC architectural composition, and can stop automatically once the firmware has converged into the documented no-key idle loop.

## Why it exists

The project needs a reproducible demonstration that reconciled HP-67 microcode traverses the modeled electrical transport rather than being handed directly to an instruction executor. Instruction fetch and display traffic are both present in the real 56-bit word, so validating them in separate backplanes would leave an artificial fidelity gap. The smoke runner therefore now carries the temporary architectural A/B display-byte bridge at `b0..b7`, the ACT ROM address at `b16..b27`, and the selected-ROM result at `b46..b55` through the same resolved pull-down-biased IS net and the same `Hp67ElectricalBackplane` word coordinate.

## Relationships

Uses `research::rom_corpus::RomCorpus` only as a local firmware loader. Every runtime fetch crosses `Hp67ElectricalBackplane` through `run_structural_display_fetch_cycle`; `FetchPipelineLatch` enforces the one-word pipeline; `Rom0DisplayEndpoint` receives the display byte from the same resolved word; `CathodeDriver1820_1749` provides the current coarse scan slot; and `Hp67ArchitecturalMachine` composes the independent ACT core, temporary architectural RAM and CRC control core. `display_byte_from_act_registers()` remains an explicitly temporary instruction-boundary bridge from A/B state to the eight display bits; it is not the final bit-serial ACT implementation.

## Responsibilities

Require exactly 5120 populated corpus words; preserve the physical startup checkpoints `0x000=0x000`, `0x001=0x3e3`, `0x0f8=0x11a`; honor the HP-67 bank-zero fallback rule; route valid CRC control opcodes without misclassifying them as ACT specials; and continue for `--probe-cycles N` real pipeline cycles. On every real fetch word, reconstruct both the display byte and the ROM result from the same resolved IS transport and hard-fail on disagreement or contention. `--trace-limit N` bounds detailed output. `--stop-at-idle` turns the documented firmware idle path into an automatic convergence criterion and then verifies the source-backed `0.00` display pattern through the combined word transport.

## Implementation

Cycle 0 fills the pipeline, cycles 1..3 verify the directly observed startup sequence, and subsequent cycles execute/fetch continuously. For each fetch, the coarse cathode scan slot selects one A/B register position, `display_byte_from_act_registers()` supplies the temporary eight-bit ACT display bridge, and `run_structural_display_fetch_cycle()` transports display `b0..b7`, address `b16..b27` and ROM result `b46..b55` on one backplane word. The returned display byte must equal the byte observed by ROM0 before the structural cathode slot advances; the returned ROM word is committed to the normal one-word execution pipeline.

`BootMilestones` records source-backed firmware landmarks without embedding ROM contents: octal `0161` (`0x071`) for display initialization, octal `0167` (`0x077`) for the documented main wait-for-key loop, octal `0206` (`0x086`) for the card-present poll, and the directly observed `0x067 -> 0x0fc6` delayed-ROM/JSB flow. Idle convergence requires display initialization, at least two visits to `L0167` with a card-poll pass, `display_enable=true`, and no buffered key. Once that architectural state is reached, the display checkpoint deliberately freezes A/B, applies a structural RCD reset to slot 1, and sends all fifteen source-backed display slots through additional combined display+fetch words on the same backplane. This freeze is a deterministic verification gate, not a claim about final RCD/STR PHI edge timing.

The expected idle codes remain `20 20 01 00 30 00 00 0F 0F 0F 0F 0F 0F 0F 20`, which ROM0 decodes to the power-on `0.00` pattern. Exact ACT bit-serial generation of `b0..b7`, exact PHI launch/sample edges, RCD/STR propagation and LED-current integration remain open fidelity milestones and are not inferred by this runner.
