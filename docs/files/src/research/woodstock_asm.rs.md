# `src/research/woodstock_asm.rs`

## Purpose

Assembles the textual Woodstock microinstruction mnemonics used by current Teenix calculator listings into exact 10-bit words for firmware-corpus research.

## Why it exists

The decoded Teenix HP-67 `.pfl` payload contains human-readable instructions rather than a flat ROM byte image. Comparing that current 2026 source against x11-calc and Nonpareil requires a small strict assembler. The production electrical ACT must not depend on a source-listing parser.

## Relationships

Used by `src/research/teenix_hp67.rs` when analysing and normalizing the decoded `cal67.pfl` listing. The encoding rules are cross-checked against the independent semantic decoder in `src/reference/woodstock.rs` and Teenix's published CPU Instruction Notes, but this module remains research-only.

## Responsibilities

Normalize whitespace/case; encode JSB, GOTO/THEN-GOTO, all 32 arithmetic operations and eight fields, status/P families, ROM/data-register families, CRC opcodes and documented fixed Woodstock instructions; accept documented Teenix/Panamatik spelling aliases such as `display reset twf`; reject unknown text and out-of-range operands instead of guessing.

## Implementation

Arithmetic words are encoded as operation bits plus the three-bit field selector and the Woodstock `..10` class. JSB and ordinary GOTO use their documented low-bit classes and eight-bit page offset. THEN-GOTO is different: after an IF-class instruction the next complete 10-bit ROM word is consumed as the destination while the logical 12-bit PC keeps its current upper two bits. Teenix prints full addresses such as `$61C`, so the assembler accepts a 12-bit target and emits its low ten bits (`$61C -> 0x21c`). Special families use their documented operand/high-bit layout. P set/test instructions use the non-linear Woodstock operand maps. Unit tests cover the physical HP-67 startup words, cross-page THEN-GOTO, Teenix `display reset twf`, all arithmetic fields and representative status/P families.
