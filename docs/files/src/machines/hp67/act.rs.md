# `src/machines/hp67/act.rs`

## Purpose

Implements the smallest independent HP-67 ACT execution core needed to run the first directly observed power-on microinstructions after they have crossed the serial IS/ISA fetch path.

## Why it exists

The project must demonstrate real firmware execution without cheating by calling the instruction-boundary reference machine or by handing opcodes directly to the ACT. At the same time, claiming a complete 1820-2530 before its bit-serial datapath and PHI-edge timing are implemented would be misleading. This module therefore creates a deliberately narrow bridge: it executes only the source-backed startup forms required to prove the first physical trace.

## Relationships

`fetch.rs` reconstructs each 10-bit word from resolved IS/ISA levels and `hp67_poweron_smoke.rs` feeds the resulting one-word pipeline into `PowerOnActCore`. The semantic `reference::woodstock` implementation is not imported. The full future 1820-2530 implementation will replace this smoke core once its serial registers, ALU and timing are available.

## Responsibilities

Maintain a 12-bit architectural PC, carry/previous-carry state and the fourteen-nibble C register; execute NOP, conditional GOTO and `0 -> c[w]`; preserve the observed Woodstock branch semantics needed by startup; and fail explicitly on every unsupported opcode.

## Implementation

`execute_word()` snapshots carry, clears current carry, advances PC, then applies the supported operation. Conditional GOTO uses the previous carry and the encoded eight-bit page offset while preserving the current high PC page. The arithmetic form accepted for the first milestone is exactly operation 8 over field W, which clears all fourteen C digits. Unit tests run the physical `0x000`, `0x3e3`, `0x11a` sequence and require PC `0x000 -> 0x001 -> 0x0f8 -> 0x0f9`. The deterministic zero-filled default C register is not claimed as the physical reset contents; the test seeds it explicitly when verifying the clear operation.
