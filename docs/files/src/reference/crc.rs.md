# `src/reference/crc.rs`

## Purpose

Provides an instruction-boundary semantic model of the HP-67/97 card-reader controller (CRC) for differential validation.

## Why it exists

The HP-67 firmware communicates with the card reader through CRC-specific flag opcodes plus two RAM-like ports. Treating those operations as ordinary ACT instructions or ordinary RAM would make a semantic HP-67 runner fail long before the electrical card-reader model exists. A compact reference peripheral lets firmware behaviour be validated independently of transport timing and magnetic effects.

## Relationships

Exported by `src/reference/mod.rs` and composed into `reference::hp67::Hp67Reference`. It shares the 14-nibble C register representation from `reference::woodstock` and models the two CRC data ports at `0x99` (write) and `0x9b` (read). The later electrical CRC 1820-1751 implementation remains a separate production-fidelity component.

## Responsibilities

Decode CRC flag instructions, maintain the 12 internal/external flags, preserve externally driven switch inputs across reset, model card insertion/completion, enforce motor/read/write/write-protect preconditions, and translate between 28-bit card words and the duplicated seven-nibble C-register representation.

## Implementation

`decode_crc_opcode()` reproduces the two CRC opcode families and maps each word to a flag number. `CrcReference` stores internal flags, external flag inputs, current/completed card sides and head position. Buffer-ready is derived from card presence and motor state. `read_into_c()` duplicates the seven nibbles of one 28-bit card word into both halves of C; `write_from_c()` packs C[7..13] back into a card word and marks the side dirty. A completed 34-word side is handed back through `take_completed_card()`.
