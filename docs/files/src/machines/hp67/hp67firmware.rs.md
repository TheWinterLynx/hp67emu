# `src/machines/hp67/hp67firmware.rs`

## Purpose

Provide the canonical, repository-versioned HP-67 firmware image used by the emulator and its hardware-level regression smokes.

## Why it exists

The emulator must be self-contained: cloning the repository must provide the firmware required to compile, and the resulting executable must not depend on an external ROM, research directory, environment variable or generated source file at runtime.

## Relationships

The ten `hp67firmware.00` through `hp67firmware.09` data blocks are the canonical firmware payload. The first eight blocks contain bank 0; the final two contain the populated bank-1 window at logical addresses `0x400..0x7ff`. `Hp67Firmware` implements the same `Hp67RomWordSource` electrical fetch boundary consumed by the HP-67 structural machine.

The firmware corpus was independently cross-checked during the research phase against multiple public HP-67 emulator/disassembly sources, including the pinned x11-calc corpus, Teenix reader dump and Nonpareil disassembly. Those source/provenance details remain documentation-only; production Rust and firmware data use project-owned names only.

## Responsibilities

Embed all 5120 populated 10-bit firmware words directly from version-controlled project data; preserve the HP-67 bank-0 and bank-1 address mapping; fall back to bank 0 where the selected bank has no physical image; expose deterministic ROM fetches without filesystem or network access; and reject malformed versioned firmware data during initialization.

## Implementation

`include_str!` embeds the ten repository files in the executable at compile time. A `OnceLock<[u16; 5120]>` parses each comma-separated octal word once, verifies the total word count and 10-bit range, and then serves immutable direct lookups. Bank 0 occupies indices `0..4096`; bank 1 occupies indices `4096..5120` and maps to logical PC `0x400..0x7ff`. Selecting bank 1 anywhere else returns the corresponding bank-0 word, retaining the historical reference view (not a physical fallback claim).

## M15J five-ROM composition

Exposes the existing selected-bank image view to ROM endpoint selection via the
trait's `selected_rom_bank(address)` method. Bank 1 selects its populated
0x400..0x7ff window; outside that window the established bank-0 fallback remains.
This is a named compatibility bridge, not a measured ROM bank-select pin or an
autonomous chip latch. Firmware bytes/layout and `read_word()` are unchanged.
An exhaustive regression reconstructs every address under both bank requests
through the five-ROM set and compares the resulting serial word to the provider.

Exact launch/latch delays remain unproven. The part/profile evidence and
remaining bank/RAM bridges are recorded in
`docs/research/M15J_FIVE_ROM_ENDPOINTS_2026-10-16.md`.

## M15K autonomous ROM bank control

`read_word_in_bank(bank, address)` supplies immutable explicit-bank words,
independent of the mutable reference view. Bank 0 covers 4096 words, bank 1 only
PC 400..7ff, and other requests return None. The old bank fallback belongs only
to `read_word()` compatibility; it is not a physical fallback claim. Live chip
selection never calls the reference bank bridge. Firmware bytes are unchanged.

See `docs/research/M15K_AUTONOMOUS_ROM_BANKS_2026-10-16.md`.
