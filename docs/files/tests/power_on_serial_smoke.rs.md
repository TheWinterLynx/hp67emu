# `tests/power_on_serial_smoke.rs`

## Purpose

Provides an end-to-end regression for the first physically observed HP-67 startup path across the structural serial bus and minimal ACT executor.

## Why it exists

Separate unit tests for serial transport and instruction semantics can both pass while their cycle ordering is wrong. This integration test locks the one-word pipeline relationship and proves that the branch at PC `0x001` changes the address transmitted during the following active machine cycle.

## Relationships

Uses `Hp67ElectricalBackplane`, `ActFetchEndpoint`, `RomFetchEndpoint`, `FetchPipelineLatch`, `run_structural_fetch_cycle()` and `PowerOnActCore`. The tiny fixture contains only the directly observed startup words plus a dummy responder at `0x0f9`; it is not a redistributable HP-67 ROM image.

## Responsibilities

Require serial fetches `0x000 -> 0x000`, `0x001 -> 0x3e3` and `0x0f8 -> 0x11a`; require those same three words to execute one cycle after fetch; require the conditional branch to reach `0x0f8`; and require the third instruction to advance the architectural PC to `0x0f9` after four structural machine cycles.

## Implementation

Each loop begins by promoting the prior prefetched word, executes it through `PowerOnActCore` when present, then uses the resulting PC as the twelve-bit address sent across IS/ISA. The ROM fixture responds through the same resolved bus and the returned ten-bit word is latched for the next cycle. Assertions compare both fetch and execution traces and verify the backplane completed four exact 56-bit words.
