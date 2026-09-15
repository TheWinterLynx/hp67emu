# `src/bin/hp67_poweron_smoke.rs`

## Purpose

Runs HP-67 power-on firmware from an external normalized 5120-word corpus through the structural serial fetch path and can continue for thousands of real machine cycles using the complete independent ACT architectural core.

## Why it exists

The project needs a reproducible demonstration that reconciled HP-67 microcode actually traverses the modeled IS/ISA path rather than being handed directly to an instruction executor. Once the initial startup proof was established, stopping after each newly encountered opcode became unnecessary. The runner now uses the complete architectural ACT bring-up core so probe length is limited by genuinely unsupported hardware/electrical behavior rather than by an arbitrary opcode checklist.

## Relationships

Uses `research::rom_corpus::RomCorpus` only as a local firmware loader. Every fetch still crosses `Hp67ElectricalBackplane` through `run_structural_fetch_cycle`; `FetchPipelineLatch` enforces the one-word pipeline; `ActArchitecturalCore` executes the reconstructed word; and `ActRamImage` temporarily supplies architectural RAM behavior until physical 1818-* RAM devices are connected. The runner does not call `reference::woodstock`.

## Responsibilities

Require exactly 5120 populated corpus words; preserve the physical startup checkpoints `0x000=0x000`, `0x001=0x3e3`, `0x0f8=0x11a`; honor the HP-67 bank-zero rule in the first 1K page; choose bank 1 only where that page is physically populated in the normalized corpus; and continue for `--probe-cycles N` real pipeline cycles. `--trace-limit N` bounds detailed output while the probe itself may run much farther.

## Implementation

Cycle 0 fills the pipeline, cycles 1..3 verify the directly observed startup sequence, and subsequent cycles execute/fetch continuously. Address bits travel at b16..b27 and ROM response bits at b46..b55 through the resolved pull-down-biased IS net. Before each fetch, `prepare_hp67_fetch()` supplies the architectural bank state; the corpus adapter mirrors the HP-67 fallback-to-bank-0 rule for pages that have no requested bank populated.

Probe execution stops only on a genuinely unknown special, the still-unimplemented ROM self-test operation, a missing serial ROM word, or another hard structural error. A detailed trace is emitted only for the first requested number of probe cycles, followed by a summary containing executed-word count, PC and bank. This remains a bit-cell structural bring-up tool: final PHI launch/sample edges and physical RAM/DATA timing are not claimed yet.
