# `src/machines/hp67/act.rs`

## Purpose

Implements the deliberately small independent HP-67 ACT execution core used to grow the real-microcode structural power-on trace.

## Why it exists

The project must demonstrate real firmware execution without cheating by calling the instruction-boundary reference machine or by handing opcodes directly to the ACT. At the same time, claiming a complete 1820-2530 before its bit-serial datapath and PHI-edge timing are implemented would be misleading. This module therefore grows only as far as the validated startup trace reaches and fails loudly at the first unsupported word.

## Relationships

`fetch.rs` reconstructs each 10-bit word from resolved IS/ISA levels and `hp67_poweron_smoke.rs` feeds the resulting one-word pipeline into `PowerOnActCore`. The semantic `reference::woodstock` implementation is not imported by this core. The full future 1820-2530 implementation will replace this smoke core once its serial registers, ALU and timing are available.

## Responsibilities

Maintain a 12-bit architectural PC, carry/previous-carry state and the fourteen-nibble C/M1/M2 registers; execute the source-backed startup operations currently reached by the structural trace; preserve Woodstock conditional-branch semantics needed by startup; and fail explicitly on every unsupported opcode.

## Implementation

`execute_word()` snapshots carry, clears current carry, advances PC, then applies one supported operation. Conditional GOTO uses the previous carry and the encoded eight-bit page offset while preserving the current high PC page. Arithmetic operation 8 over field W clears all fourteen C digits. Special word octal `0410` (`0x108`) exchanges all fourteen digits of C and M1. The next probe run reached PC `0x0fb` with octal `0610` (`0x188`), so the low-level core now also implements the source-backed C↔M2 exchange independently. Unit tests cover the physical `0x000`, `0x3e3`, `0x11a` startup sequence, both memory-register exchanges, and the invariant that an unsupported word does not advance PC. Deterministic default register contents are test scaffolding and are not claimed as measured reset state.
