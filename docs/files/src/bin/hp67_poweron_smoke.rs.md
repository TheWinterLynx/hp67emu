# `src/bin/hp67_poweron_smoke.rs`

## Purpose

Runs a local HP-67 power-on smoke test against an external normalized 5120-word firmware corpus and can continue probing the startup path until the first ACT opcode not yet implemented by the low-level smoke core.

## Why it exists

Unit fixtures can prove transport mechanics, but the project also needs a reproducible demonstration that the reconciled Teenix/x11-calc/Nonpareil microcode actually enters the electrical fetch path and drives the first observed startup control flow. Firmware bytes are intentionally not committed, so the runner consumes the locally generated `.research/teenix-2026-hp67.tsv` corpus by default. The optional probe mode lets the real local corpus tell us exactly which verified ACT behavior must be implemented next instead of guessing a startup sequence.

## Relationships

Uses `research::rom_corpus::RomCorpus` only as a local research-file loader. The actual fetch crosses `Hp67ElectricalBackplane` through `run_structural_fetch_cycle`; `FetchPipelineLatch` enforces the one-word pipeline; `PowerOnActCore` executes the resulting words without calling `reference::woodstock`.

## Responsibilities

Require exactly 5120 populated corpus words, expose bank-0 words through the ROM-source boundary, run four mandatory 56-bit structural machine cycles, verify the physical startup fetches `0x000=0x000`, `0x001=0x3e3`, `0x0f8=0x11a`, require those three words to execute in the following cycles, and fail on any discrepancy. When `--probe-cycles N` is supplied, continue from the already-prefetched word at `0x0f9` for at most N further cycles and stop cleanly with PC/word/octal diagnostics at the first unsupported ACT operation.

## Implementation

Cycle 0 fills the fetch pipeline from PC 0. Cycle 1 executes the fetched NOP and serially fetches PC 1. Cycle 2 executes `0x3e3`, branches to `0x0f8`, and serially fetches `0x11a`. Cycle 3 executes `0x11a` (`0 -> c[w]`) and concurrently fetches the next word at `0x0f9`. Every fetch address travels as twelve resolved IS bits at b16..b27 and every returned ROM word is reconstructed from ten resolved IS bits at b46..b55. After the mandatory PASS, probe mode repeatedly begins the next pipeline cycle, executes only source-backed operations already present in `PowerOnActCore`, performs the next serial fetch, and reports `PROBE STOP` rather than silently delegating an unknown opcode to the semantic reference model. The runner remains a structural bit-cell smoke/probe tool, not yet a claim of final PHI-edge timing.
