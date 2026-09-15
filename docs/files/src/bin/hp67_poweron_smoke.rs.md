# `src/bin/hp67_poweron_smoke.rs`

## Purpose

Runs HP-67 power-on firmware from an external normalized 5120-word corpus through the structural serial fetch path, continues for thousands of real machine cycles using the independent ACT+CRC architectural composition, and can stop automatically once the firmware has converged into the documented no-key idle loop.

## Why it exists

The project needs a reproducible demonstration that reconciled HP-67 microcode actually traverses the modeled IS/ISA path rather than being handed directly to an instruction executor. The long probe now runs ordinary ACT and CRC-control traffic without artificial opcode stops, so the next useful milestone is not simply “more cycles” but proving that reset completes, display initialization is reached, and the real firmware enters and repeats its normal wait-for-key loop.

## Relationships

Uses `research::rom_corpus::RomCorpus` only as a local firmware loader. Every fetch still crosses `Hp67ElectricalBackplane` through `run_structural_fetch_cycle`; `FetchPipelineLatch` enforces the one-word pipeline; and `Hp67ArchitecturalMachine` composes the independent ACT core, temporary architectural RAM and CRC control core. The runner does not call `reference::woodstock` or `reference::crc`.

## Responsibilities

Require exactly 5120 populated corpus words; preserve the physical startup checkpoints `0x000=0x000`, `0x001=0x3e3`, `0x0f8=0x11a`; honor the HP-67 bank-zero rule in the first 1K page; choose bank 1 only where that page is physically populated in the normalized corpus; route valid CRC control opcodes without misclassifying them as ACT specials; and continue for `--probe-cycles N` real pipeline cycles. `--trace-limit N` bounds detailed output. `--stop-at-idle` additionally turns the documented firmware idle path into an automatic convergence criterion.

## Implementation

Cycle 0 fills the pipeline, cycles 1..3 verify the directly observed startup sequence, and subsequent cycles execute/fetch continuously. Address bits travel at b16..b27 and ROM response bits at b46..b55 through the resolved pull-down-biased IS net. Before each fetch, `prepare_hp67_fetch()` supplies architectural bank state; the corpus adapter mirrors the HP-67 fallback-to-bank-0 rule for pages that have no requested bank populated.

`BootMilestones` records source-backed firmware landmarks without embedding ROM contents: octal `0161` (`0x071`) for the display-initialization sequence, octal `0167` (`0x077`) for the documented main wait-for-key loop, octal `0206` (`0x086`) for the card-present poll inside that loop, and the directly observed `0x067 -> 0x0fc6` delayed-ROM/JSB flow. Idle convergence requires display initialization, at least two visits to `L0167` with a card-poll pass between them, `display_enable=true`, and no buffered key. This proves one complete pass through the no-input firmware loop rather than merely touching its entry point.

CRC set/test-control words are handled by the composed machine while preserving the ACT's universal instruction-boundary work. Probe execution still stops on a genuinely unknown ACT special, ROM self-test, the first CRC DATA-port access at `0x99`/`0x9b`, a missing serial ROM word, or another hard structural error. This remains a bit-cell structural bring-up tool: the idle milestone is architectural firmware convergence, not yet proof of physical LED output. Final PHI launch/sample edges, ROM0 anode timing, cathode-driver timing, physical CRC transport and physical RAM/DATA timing remain separate fidelity milestones.
