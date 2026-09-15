# `tests/act_architectural_differential.rs`

## Purpose

Exhaustively compares the independent HP-67 ACT architectural bring-up core against the separate Woodstock semantic reference at instruction boundaries.

## Why it exists

The project now implements the whole architectural ACT in one step instead of adding startup opcodes one by one. That is only safe if broad regression coverage immediately detects semantic drift. The semantic reference remains a test oracle only; production HP-67 code does not call it.

## Relationships

Constructs matching `ActArchitecturalState`/`ActRamImage` and `reference::woodstock::ReferenceMachine` fixtures. It exercises `ActArchitecturalCore::execute_word()` directly and compares resulting ACT registers, control state, display/key state and installed HP-67 RAM contents after every successful word.

## Responsibilities

Cover all 1024 ten-bit words across several nontrivial seeds including decimal/binary arithmetic, carry states, status patterns, P values including invalid P=14 behavior, delayed-ROM state, bank state, return stack and RAM contents. Separately cover all 1024 possible words while the machine is in THEN-GOTO state for both carry outcomes. Require corresponding unknown-special and ROM-self-test errors on both implementations.

## Implementation

Each comparison starts from freshly seeded independent states so one opcode cannot influence the next. The test converts only fixture data into the reference model; no production ACT implementation imports reference types. On success it compares every architecturally visible ACT field and all installed RAM words 0x00..0x3f. Error comparisons match semantic error classes without requiring the reference model to share the low-level core's transactional rollback policy.
