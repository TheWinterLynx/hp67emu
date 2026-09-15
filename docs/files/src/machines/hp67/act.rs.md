# `src/machines/hp67/act.rs`

## Purpose

Implements the deliberately small independent HP-67 ACT execution core used to grow the real-microcode structural power-on trace.

## Why it exists

The project must demonstrate real firmware execution without cheating by calling the instruction-boundary reference machine or by handing opcodes directly to the ACT. At the same time, claiming a complete 1820-2530 before its bit-serial datapath and PHI-edge timing are implemented would be misleading. This module therefore grows only as far as the validated startup trace reaches and fails loudly at the first unsupported word.

## Relationships

`fetch.rs` reconstructs each 10-bit word from resolved IS/ISA levels and `hp67_poweron_smoke.rs` feeds the resulting one-word pipeline into `PowerOnActCore`. The semantic `reference::woodstock` implementation is not imported by this core. The full future 1820-2530 implementation will replace this smoke core once its serial registers, ALU and timing are available.

## Responsibilities

Maintain a 12-bit architectural PC, carry/previous-carry state, pending delayed-ROM selection and the fourteen-nibble C/M1/M2 registers; execute the source-backed startup operations currently reached by the structural trace; preserve Woodstock conditional-branch and delayed-ROM semantics needed by startup; keep unsupported-opcode failures transactional; and fail explicitly on every unsupported opcode.

## Implementation

`execute_word()` snapshots the current core so an unsupported probe word cannot consume carry or pending control-flow state, then snapshots carry, clears current carry and advances PC. Conditional GOTO uses the previous carry and the encoded eight-bit page offset while preserving the current high PC page. Arithmetic operation 8 over field W clears all fourteen C digits. Special words octal `0410` (`0x108`) and `0610` (`0x188`) exchange C with M1 and M2 respectively.

The latest real-firmware probe reached PC `0x0fd` with octal `0264` (`0x0b4`), a Woodstock delayed-select-ROM operation with operand 2. The core now implements the source-backed delayed-select family (`opcode & 0o77 == 0o64`): executing `0264` at `0x0fd` leaves the immediate next PC at `0x0fe`, so that following word is still fetched from the current ROM. Only after that following word executes is the pending ROM nibble applied to PC bits 11..8 while preserving the resulting low eight bits. This mirrors the ordering already used by the semantic oracle and is essential for the later observed `delayed select rom 15` + `jsb` path that reaches `0x0fc6`.

Unit tests cover the physical `0x000`, `0x3e3`, `0x11a` startup sequence, both memory-register exchanges, delayed-ROM application one word later, and full state rollback when an unsupported opcode is encountered. Deterministic default register contents are test scaffolding and are not claimed as measured reset state.
