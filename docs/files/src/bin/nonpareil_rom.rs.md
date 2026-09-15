# `src/bin/nonpareil_rom.rs`

## Purpose

Provides a local command-line entry point for normalizing official Nonpareil `uasm` Woodstock object files into hp67emu's sparse ROM TSV format.

## Why it exists

Nonpareil's HP-67 sources are split across several assembly files and are intended to be assembled with upstream `uasm`. A small local importer lets us preserve that upstream tool as the assembler authority while still comparing its output reproducibly against Teenix 2026 and x11-calc.

## Relationships

Uses `research::nonpareil_obj` for strict object-line parsing and `research::rom_corpus` for validated storage. Its output can be checked with `rom_compare verify-startup`, compared symmetrically with `rom_compare compare`, or used as a subset/reference with `rom_subset`.

## Responsibilities

Read one or more local Nonpareil `.obj` files, parse their bank/address/opcode records, merge only identical overlaps, reject conflicting overlaps, and write one normalized TSV corpus. It never downloads firmware or embeds Nonpareil object data.

## Implementation

The first CLI argument is the output TSV and all remaining arguments are `.obj` inputs. Each object is parsed independently, then all populated bank/PC slots are merged. A second object may repeat a location only when the ten-bit word is identical. Any conflict is reported with exact bank, PC and both values; malformed object syntax remains a hard error.
