# M14D — serial arithmetic boundary authority

Date: 2026-09-24  
Branch: `agent/m14d-serial-arithmetic-authority`

## Goal

Remove the instruction-boundary architectural executor from the causal final A/B/C/carry result path for the ADD/SUB-family arithmetic already represented by the source-backed serial traversal, without inventing an internal ACT register-write or PHI edge.

## Evidence boundary

The project evidence already supports a 56-bit machine word, one-word fetch/execution overlap, a serial ACT-generation arithmetic datapath over A/B/C, the current Woodstock field/source/destination routing, and carry/borrow chaining across the selected digit sequence. The existing structural executor therefore has a justified full-word coordinate and arithmetic routing.

The source set still does **not** establish which PHI edge writes an ACT register bit, whether a complete digit is committed at its fourth coordinate, the exact temporal visibility of carry, the physical decimal-correction holding behavior in 1820-2530, or the interaction of an in-flight internal write with the early b0..b7 display window. Those facts remain SOURCE-BLOCKED.

## Causal change

M14D changes only the final ADD/SUB-family A/B/C/carry authority:

1. `ActSerialEndpoint::begin_execution()` captures the immutable pre-instruction state and initializes an `ActSerialArithmeticResultImage` for ADD/SUB routing.
2. The architectural machine executes the word once to provide an oracle for final A/B/C/carry and to apply still-unmigrated instruction-boundary effects such as PC and instruction-state changes.
3. The live bridge immediately restores the pre-instruction A/B/C/carry, so the oracle no longer supplies the live arithmetic result.
4. The shared structural b0..b55 traversal evaluates selected ADD/SUB digits and records those results into the endpoint-owned image while chaining carry/borrow.
5. After the execution has completed b55, the structural image is compared with the architectural oracle. Any mismatch is a hard failure.
6. Only the structural image is then committed to live A/B/C/carry.

Destination-less subtract/compare operations follow the same route: the structural image preserves A/B/C and supplies only the final carry/borrow, while the architectural oracle may still place the ACT into `ThenGoto`. Empty selected fields also preserve the opcode's initial carry/borrow seed rather than losing it because no digit was processed.

## Fidelity classification

**PROVEN / source-backed at the modeled level:** the word has a complete b0..b55 lifetime; ADD/SUB source/destination/field routing and serial digit ordering are explicit; the structural path owns the carry/borrow chain and final image; the oracle and structural result must agree at the instruction boundary.

**WORKING APPROXIMATION:** the live machine applies the completed structural arithmetic image after b55. This is a causal boundary handoff, not a claim that the physical ACT performs one atomic register update at the end of the word.

**SOURCE-BLOCKED:** physical intra-word A/B/C write timing, carry visibility on exact bit/PHI edges, decimal-correction internal timing, and any operation-specific storage qualifier.

## Scope exclusions

- Clear/copy/exchange/shift and zero/nonzero arithmetic families remain on the architectural fallback.
- PC, P, status, return-stack, delayed-ROM and other non-A/B/C/carry effects are not migrated by this slice.
- M14D does not change DATA electrical polarity, RAM chip partitioning, ROM timing, display STR/RCD timing or keyboard/card behavior.
- No `Hp67Net` ownership or PHI contract is changed.

## Validation required before merge

Owner-side Windows gate:

`cargo fmt --all -- --check; if ($LASTEXITCODE -ne 0) { throw "FMT FAILED" }; $env:RUSTFLAGS='-Dwarnings'; cargo test --locked --all-targets; if ($LASTEXITCODE -ne 0) { throw "TESTS FAILED" }; cargo build --locked --release --bins; if ($LASTEXITCODE -ne 0) { throw "RELEASE BUILD FAILED" }`

Behavior-impacting release diagnostic:

`cargo test --release --locked --bin hp67emu live_custom_diagnostic_pac_suite_reports_ok_ko -- --ignored --nocapture`

Expected closure remains `DIAGNOSTIC SUITE OK: 12/12 passed`. This note does not claim that either validation has passed on M14D until the owner reports it.
