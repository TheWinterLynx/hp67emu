# M14F — P/status completed-word structural authority

Date: 2026-09-25  
Branch: `agent/m14f-p-status-authority`

## Goal

Remove the instruction-boundary executor as the causal final-state source for a focused ACT control family: P, P-change history, status bits, condition carry, previous-carry and `ThenGoto` state.

This slice deliberately does not attempt to assign any internal 1820-2530 control-state write to a specific serial bit or PHI edge.

## Included instruction families

M14F independently decodes and structurally owns the completed-word result for:

- clear status with the documented retained status bits;
- set status bit;
- clear status bit;
- test status clear / test status set;
- decrement P;
- increment P;
- set P;
- test P equal / test P not-equal;
- the P/control side of load-constant-and-decrement-P.

The load-constant C-digit write is not migrated here.

## Universal word bookkeeping covered by the image

For an included control word, the result image also reproduces the ACT boundary bookkeeping that affects the selected control state:

- P-change history ages one position;
- `previous_carry` receives the pre-instruction carry;
- current carry starts cleared before the special operation;
- condition tests can set carry and `ThenGoto`.

The generic P-wrap history case used by HP-67/97 label-search behavior is preserved without a PC-specific firmware hack.

## Causal path

The structural endpoint captures immutable pre-instruction control inputs before the architectural executor runs. The executor then runs once as the semantic oracle and temporary owner of still-unmigrated effects such as PC.

For ACT-owned special instructions in this slice, the oracle-produced P/status/condition fields are captured as expected values and immediately restored to their pre-instruction values. The actual structural execution then traverses b0..b55. Only when b55 has completed does the private control image become complete. The live bridge compares every migrated field exactly against the oracle and commits only the structural result.

CRC-owned words are excluded at the composed-machine ownership boundary even if a bit pattern could otherwise resemble an ACT special family.

## Evidence classification

### Source-backed / semantic contract

- complete 56-bit structural execution lifetime;
- ACT special-opcode semantics already independently cross-checked against the Rust Woodstock reference;
- P/status mappings and P-wrap compatibility behavior used by real HP-67 firmware;
- one-word pipeline ordering.

### Working approximation

The completed-word commit of P/status/control fields is a scheduler authority boundary. It is not a statement that the physical ACT waits until b55 to mutate those latches.

### Source-blocked

M14F does not claim:

- the PHI edge that writes P or status latches;
- the instant at which a status-test result becomes internally visible;
- the internal implementation of P-change history storage;
- the electrical form of the condition/ThenGoto latch;
- the timing relationship between these internal latches and other still-unmigrated ACT control blocks.

## Validation required before merge

Owner-side Windows gate:

`cargo fmt --all -- --check; if ($LASTEXITCODE -ne 0) { throw "FMT FAILED" }; $env:RUSTFLAGS='-Dwarnings'; cargo test --locked --all-targets; if ($LASTEXITCODE -ne 0) { throw "TESTS FAILED" }; cargo build --locked --release --bins; if ($LASTEXITCODE -ne 0) { throw "RELEASE BUILD FAILED" }`

Behavior-impacting release diagnostic:

`cargo test --release --locked --bin hp67emu live_custom_diagnostic_pac_suite_reports_ok_ko -- --ignored --nocapture`

Expected closure: `DIAGNOSTIC SUITE OK: 12/12 passed`.

No validation success is claimed until the repository owner reports it.
