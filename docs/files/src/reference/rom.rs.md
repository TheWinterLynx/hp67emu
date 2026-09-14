# `src/reference/rom.rs`

## Purpose

Provides a host-independent Woodstock ROM container plus HP-67 instruction fetch/step support for the semantic reference machine.

## Why it exists

The reference executor must be able to run real or synthetic microcode without coupling the reusable core to filesystem paths, UI code or a decision to redistribute copyrighted ROM data. Tests also need to construct tiny ROM fixtures directly from words.

## Relationships

Exported by `src/reference/mod.rs`. It consumes `reference::woodstock::ArchitecturalState` and `ReferenceMachine`, using their bank/page/opcode constants. Raw HP-67 ROM words can later be supplied from external development dumps described in `docs/MICROCODE_PROVENANCE.md` without changing this API.

## Responsibilities

Store optional 10-bit ROM words for two banks and four 1K pages, validate bank/address/opcode widths, remember which banks physically exist on each page, reproduce bank-zero fallback when a requested bank is absent, and fetch/step the HP-67 semantic machine without doing host file I/O.

## Implementation

`RomImage` stores `Option<u16>` entries for the complete Woodstock address space and a per-page bank-presence mask. `install_word()` and `install_page()` are caller-fed loaders. `effective_bank()` falls back to bank zero if the selected bank is not populated on the current page. `step_hp67()` applies the Hawkeye/Woodstock implicit page-zero bank reset before fetching and then passes the fetched word to `ReferenceMachine::step_word()`. Missing or malformed ROM data fails explicitly rather than being treated as zero-filled memory.
