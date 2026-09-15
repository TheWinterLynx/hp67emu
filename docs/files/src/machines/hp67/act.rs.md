# `src/machines/hp67/act.rs`

## Purpose

Implements an independent, instruction-boundary-complete Woodstock ACT core for HP-67 bring-up while the final pin/timing-accurate 1820-2530 is still being built.

## Why it exists

Growing the real startup path one opcode at a time was useful to prove that the first serial fetch/execution chain was genuine, but it is too slow for continued development. The project already has a separately implemented semantic oracle, so the low-level HP-67 path can now implement the full architectural instruction set independently, verify it exhaustively against that oracle in tests, and reserve slow evidence-driven work for actual electrical timing and peripheral behavior.

## Relationships

`fetch.rs` still reconstructs every 10-bit firmware word from resolved IS/ISA levels. `hp67_poweron_smoke.rs` feeds those words through the one-word pipeline into `ActArchitecturalCore`. `ActRamImage` is a temporary architectural RAM image kept outside the ACT state so the chip boundary remains explicit until physical 1818-* RAM devices are connected. Production code in this module does not import `reference::woodstock`; the reference model is used only by `tests/act_architectural_differential.rs` as an external oracle.

## Responsibilities

Maintain all architecturally visible ACT state needed by HP-67 firmware: A/B/C/Y/Z/T/M1/M2, F, P and P-change history, status bits, decimal/binary mode, carry state, 12-bit PC, delayed-ROM selection, bank state, two-level return stack, THEN-GOTO state, key buffer, display state and RAM address. Implement all four Woodstock opcode classes, the 32 arithmetic/register operations across all eight fields, JSB/GOTO/THEN-GOTO, fixed specials, status/P/constant/select-ROM families, bank switching, RAM architectural access and display/key control. Unsupported ROM self-test remains an explicit hard stop.

## Implementation

`ActArchitecturalCore::execute_word()` performs the full instruction-boundary transition independently from the reference implementation. It preserves Woodstock's previous-carry branch convention, one-word delayed-ROM behavior, two-level return stack, P-wrap compatibility behavior, decimal/hex arithmetic and the documented invalid-P field semantics. Unsupported special operations are transactional so a diagnostic stop cannot consume architectural state.

`ActRamImage` installs the HP-67's current architectural 0x00..0x3f RAM range for bring-up and differential testing. It is deliberately separate from `ActArchitecturalState`; it is not a claim that RAM is physically inside the ACT. `prepare_hp67_fetch()` applies the HP-67/Hawkeye rule that bank 1 cannot remain selected while fetching from the first 1K page.

The module retains `PowerOnAct*` type aliases temporarily so existing bring-up tools remain source-compatible while moving to the broader architecture. Exhaustive differential tests compare every 10-bit word across multiple nontrivial states plus all 1024 THEN-GOTO payloads against the independent semantic oracle. Electrical PHI edges, bit-serial register/ALU timing, DATA-bus timing and physical RAM devices remain outside this module's current claim.
