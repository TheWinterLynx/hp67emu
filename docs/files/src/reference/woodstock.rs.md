# `src/reference/woodstock.rs`

## Purpose

Provides a UI-independent, instruction-boundary Woodstock semantic machine for differential validation of the future electrical ACT implementation.

## Why it exists

Nonpareil already demonstrates a mature instruction-level Woodstock implementation and the HP-67 firmware depends on details that are easy to get subtly wrong: BCD carry/borrow behaviour, conditional next-word branches, two-level return-stack wrap, P encoding, delayed ROM selection, key dispatch and RAM-register addressing. Re-discovering those semantics while simultaneously building the electrical model would make failures difficult to localise. This module therefore supplies a small Rust oracle whose results can later be compared against the low-level ACT after every completed microinstruction.

## Relationships

`src/reference/mod.rs` exports this module. `docs/NONPAREIL_ANALYSIS.md` records the upstream analysis and licensing boundary. The HP-67 machine constructor installs the four 16-register RAM blocks described by the calculator definition, while CRC/card-reader behaviour remains a separate reference peripheral. The timed implementation in `src/emulation` and `src/machines/hp67` must eventually reproduce the same architectural snapshots without depending on this reference executor.

## Responsibilities

Define Woodstock register/control state, classify 10-bit opcodes, execute the 32 arithmetic/register operations, implement CPU special instructions used by HP-67 firmware, model JSB/GOTO/THEN-GOTO and return-stack behaviour, expose generic RAM-register semantics, and fail explicitly on unknown special opcodes. Keep semantic execution separate from egui, image handling and electrical timing.

## Implementation

`decode()` uses the two least-significant bits to classify all 1024 words into special, JSB, arithmetic and GOTO forms. Arithmetic words expose a five-bit operation and three-bit field. `ReferenceMachine::step_word()` advances one instruction-boundary word: it snapshots carry, advances PC, maintains recent P movement, handles THEN-GOTO target words, executes the decoded operation, and applies any delayed ROM selection at the correct boundary.

The arithmetic implementation covers all 32 operations over the P/WP/XS/X/S/M/W/MS fields using decimal or hexadecimal nibble arithmetic. Special-op handling covers status bits, P maps and tests, constant loading, direct and delayed ROM selection, RAM/register access, display state, key-code transfers, M1/M2 transfers, RPN stack operations, format mode, return and bank switch. The stack-special regression locks the published Woodstock mapping `0o1110 = down rotate` and `0o1310 = c -> stack`, matching both Teenix's instruction table and the reviewed x11-calc decoder. `ReferenceMachine::hp67()` installs RAM addresses `0x00..0x3f`; CRC ports at `0x99` and `0x9b` are intentionally not treated as ordinary RAM because they require card-reader peripheral semantics.

The P-wrap compatibility behaviour is expressed generically through a three-word P-change history: after two consecutive increments, the zero test accepts the transient P=1 state. This deliberately avoids Nonpareil's firmware-PC-specific workaround and gives the future electrical ACT a behavioural regression target instead of a copied address hack.
