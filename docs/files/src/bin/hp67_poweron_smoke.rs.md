# `src/bin/hp67_poweron_smoke.rs`

## Purpose

Runs HP-67 power-on firmware from an external normalized 5120-word corpus through the structural serial fetch path, continues for thousands of real machine cycles using the independent ACT+CRC architectural composition, and can stop automatically once the firmware has converged into the documented no-key idle loop.

## Why it exists

The project needs a reproducible demonstration that reconciled HP-67 microcode actually traverses the modeled IS/ISA path rather than being handed directly to an instruction executor. The deterministic idle checkpoint now proves reset/display initialization and one complete no-key loop pass. With that architectural milestone complete, the runner also reports the raw ACT A/B/C register images at idle so display-electrical work can connect the real state to ROM0 without introducing a UI formatter.

## Relationships

Uses `research::rom_corpus::RomCorpus` only as a local firmware loader. Every fetch still crosses `Hp67ElectricalBackplane` through `run_structural_fetch_cycle`; `FetchPipelineLatch` enforces the one-word pipeline; and `Hp67ArchitecturalMachine` composes the independent ACT core, temporary architectural RAM and CRC control core. The runner does not call `reference::woodstock` or `reference::crc`. The idle A/B/C dump is diagnostic input for the new structural `machines::hp67::display` path, not display rendering.

## Responsibilities

Require exactly 5120 populated corpus words; preserve the physical startup checkpoints `0x000=0x000`, `0x001=0x3e3`, `0x0f8=0x11a`; honor the HP-67 bank-zero rule in the first 1K page; choose bank 1 only where that page is physically populated in the normalized corpus; route valid CRC control opcodes without misclassifying them as ACT specials; and continue for `--probe-cycles N` real pipeline cycles. `--trace-limit N` bounds detailed output. `--stop-at-idle` turns the documented firmware idle path into an automatic convergence criterion and prints the exact nibble contents of A, B and C when the checkpoint is reached.

## Implementation

Cycle 0 fills the pipeline, cycles 1..3 verify the directly observed startup sequence, and subsequent cycles execute/fetch continuously. Address bits travel at b16..b27 and ROM response bits at b46..b55 through the resolved pull-down-biased IS net. Before each fetch, `prepare_hp67_fetch()` supplies architectural bank state; the corpus adapter mirrors the HP-67 fallback-to-bank-0 rule for pages that have no requested bank populated.

`BootMilestones` records source-backed firmware landmarks without embedding ROM contents: octal `0161` (`0x071`) for the display-initialization sequence, octal `0167` (`0x077`) for the documented main wait-for-key loop, octal `0206` (`0x086`) for the card-present poll inside that loop, and the directly observed `0x067 -> 0x0fc6` delayed-ROM/JSB flow. Idle convergence requires display initialization, at least two visits to `L0167` with a card-poll pass, `display_enable=true`, and no buffered key. The confirmed local run reaches this state at cycle/executed-word 281. `format_act_register()` prints the 14 ACT nibbles most-significant-first in hexadecimal, and `DISPLAY ARCH STATE` reports A, B, C, `display_14_digit` and `display_enable` at every boot summary.

CRC set/test-control words are handled by the composed machine while preserving the ACT's universal instruction-boundary work. Probe execution still stops on a genuinely unknown ACT special, ROM self-test, the first CRC DATA-port access at `0x99`/`0x9b`, a missing serial ROM word, or another hard structural error. This remains a bit-cell structural bring-up tool: the idle milestone is architectural firmware convergence, not yet proof of physical LED output. Final PHI launch/sample edges, ACT-to-ROM0 display serialization, cathode-driver pulse timing, physical CRC transport and physical RAM/DATA timing remain separate fidelity milestones.
