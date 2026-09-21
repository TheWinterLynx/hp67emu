# `src/hp67/diagnostic_tests.rs`

## Purpose

Provide one explicit long-running Cargo regression that loads and executes every native Custom
Diagnostic Pac through the production HP-67 firmware/card/keyboard/display path and reports a
human-readable per-card `OK` or `KO` result.

## Why it exists

The individual diagnostic cards have known final display values, but manually loading twelve cards
is not a repeatable project gate. A single suite is needed to prove that native `.hp67card`
loading, one- and two-pass card transport, real firmware A-key dispatch, stored-program execution,
control flow and the physical display path continue to agree after emulator changes.

The suite is deliberately marked `#[ignore]` because CD-09 through CD-12 execute hundreds or
thousands of stored-program iterations. It is intended to be run explicitly in the release
profile, not to slow every ordinary `cargo test` invocation.

## Relationships

The child test module has test-only access to the private `Hp67LiveMachine` boundary in
`src/hp67.rs`. It consumes the canonical native media under
`programs/HP67/Custom Diagnostic Pacs` and does not parse or generate Teenix `.hpp` files.
Card insertion uses `Hp67MagneticCard::from_hp67card_bytes()`, firmware owns motor/head/card
semantics, and the A key traverses the same physical keyboard dispatch path used by the existing
M11/M13 regressions.

## Responsibilities

Boot a fresh real-firmware machine for every diagnostic; insert the native physical card; complete
Track 1 and, where present, wait for real `Crd` before reinserting the same card by End2; wait for
the reader/motor path to settle; press physical key A through `keys -> A -> ROM address`; execute
until firmware returns to the no-key RUN wait; compare the raw fifteen-position hardware segment
frame with the independently declared expected result; continue running later diagnostics even
after one reports `KO`; print one result row per diagnostic; and fail the Cargo test if any row is
`KO`.

## Implementation

`DIAGNOSTICS` currently covers CD-01 through CD-12. Short diagnostics have bounded 250k/500k
firmware-cycle budgets, while the four burn-ins have larger case-specific limits. Expected values
are converted directly into the already-proven raw HP-67 seven-segment masks; the suite does not
read formatted calculator text or call host-side calculator semantics.

Success requires all of the following simultaneously: native card transport completed, any required
second track was requested by firmware, physical A-key dispatch succeeded, no live-machine error
occurred, program execution returned to the no-key wait with S2/S15 clear, and the hardware display
frame exactly equals the expected value.

Run only this suite with:

```powershell
cargo test --release --locked --bin hp67emu live_custom_diagnostic_pac_suite_reports_ok_ko -- --ignored --nocapture
```

A clean run ends with `DIAGNOSTIC SUITE OK: 12/12 passed`. Any failure prints `KO` with the
reference, expected value, current display, PC and relevant running/key state before the test exits
non-zero.
