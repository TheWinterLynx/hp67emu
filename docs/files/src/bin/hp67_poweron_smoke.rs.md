# `src/bin/hp67_poweron_smoke.rs`

## Purpose

Runs HP-67 power-on firmware from an external normalized 5120-word corpus through the structural serial fetch path and can continue for thousands of real machine cycles using the independent ACT+CRC architectural composition.

## Why it exists

The project needs a reproducible demonstration that reconciled HP-67 microcode actually traverses the modeled IS/ISA path rather than being handed directly to an instruction executor. Once the ACT opcode space was implemented, the long probe reached octal `1000`, a CRC control opcode rather than an ACT special. The runner therefore now resolves chip ownership at instruction boundaries so it stops on genuine unmodeled hardware instead of on valid peripheral commands.

## Relationships

Uses `research::rom_corpus::RomCorpus` only as a local firmware loader. Every fetch still crosses `Hp67ElectricalBackplane` through `run_structural_fetch_cycle`; `FetchPipelineLatch` enforces the one-word pipeline; and `Hp67ArchitecturalMachine` composes the independent ACT core, temporary architectural RAM and CRC control core. The runner does not call `reference::woodstock` or `reference::crc`.

## Responsibilities

Require exactly 5120 populated corpus words; preserve the physical startup checkpoints `0x000=0x000`, `0x001=0x3e3`, `0x0f8=0x11a`; honor the HP-67 bank-zero rule in the first 1K page; choose bank 1 only where that page is physically populated in the normalized corpus; route valid CRC control opcodes without misclassifying them as ACT specials; and continue for `--probe-cycles N` real pipeline cycles. `--trace-limit N` bounds detailed output while the probe itself may run much farther.

## Implementation

Cycle 0 fills the pipeline, cycles 1..3 verify the directly observed startup sequence, and subsequent cycles execute/fetch continuously. Address bits travel at b16..b27 and ROM response bits at b46..b55 through the resolved pull-down-biased IS net. Before each fetch, `prepare_hp67_fetch()` supplies architectural bank state; the corpus adapter mirrors the HP-67 fallback-to-bank-0 rule for pages that have no requested bank populated.

CRC set/test-control words are handled by the composed machine while preserving the ACT's universal instruction-boundary work. Probe execution now stops on a genuinely unknown ACT special, ROM self-test, the first CRC DATA-port access at `0x99`/`0x9b`, a missing serial ROM word, or another hard structural error. This remains a bit-cell structural bring-up tool: final PHI launch/sample edges, physical CRC transport and physical RAM/DATA timing are not claimed yet.
