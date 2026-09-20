# `tools/generate_hp67_diagnostic_pacs.py`

## Purpose

Generate and verify the native Standard Pac diagnostic container plus the synthetic Custom Diagnostic Pac fixtures used to probe advanced HP-67 programming behavior.

## Why it exists

The diagnostic media must be reproducible from reviewed inputs instead of depending on opaque hand-built binary files. The Standard Pac native container is derived from the two existing source-backed SD1-15A Teenix sides. The first custom cards use deliberately tiny programs with independently known final results so failures can be localized to GSB/GTO/RTN, flags, conditionals, indirect storage, return-stack depth, second-half label search, ISZ or DSZ. CD-09 through CD-12 extend the same idea into long-running burn-ins with hundreds or thousands of loop iterations and repeated cross-half subroutine calls.

## Relationships

The tool mirrors the documented host-media layout implemented in `src/machines/hp67/magnetic_card.rs`: 34 28-bit records per logical track, 119 packed bytes per track, and the 250-byte `HP67CARD` v1 container. The native `.hp67card` files are the canonical media consumed directly by `src/program_library.rs`. The diagnostic generator has no Teenix dependency; `.hpp` import compatibility is owned by the legacy-library path.

## Responsibilities

Build 112 program bytes into the same record/nibble ordering decoded by the production Program Library; calculate record 34 as the low 28 bits of the running sum of records 1-33; emit native `.hp67card` containers; validate the committed native SD1-15A media directly; validate the exact SD-15C native fixture by its pinned SHA-256 without regenerating or wrapping it; and support `--check` so a local gate can prove every generated binary is reproducible.

## Implementation

Single-pass custom programs use header `0x03100222`, matching the checked-in FIX-2/DEG one-pass program-card convention. CD-06 and CD-12 use `0x03000222` followed by `0x04000222` so real firmware requests the continuation with `Crd`. The exact SD-15C fixture is a source input, not generated output: its SHA-256 is pinned to `f626047e875d919bb22bf7b25bb0d58218955b07ff504f2f546239a2f1fdf75a`, and the runtime library consumes that exact native file directly. Program bytes are expanded low nibble then high nibble and paired into CRC records in the inverse of `program_bytes_from_track()`; native host packing remains MSB-first exactly as defined by the project's interchange format. The tool uses only the Python standard library and does not claim that this host packing is the physical magnetic bit order.
