# M14H — mode/latch completed-word structural authority

Date: 2026-09-27  
Branch: `agent/m14h-mode-latch-authority`

## Goal

Move final authority for a focused set of ACT mode/latch specials away from the instruction-boundary executor and onto the shared structural `b0..b55` execution lifetime, while keeping fetch-sensitive control state in a later slice.

## Included latch family

M14H independently decodes and owns the completed-word result for:

- DISPLAY toggle;
- DISPLAY off;
- decimal mode;
- hexadecimal mode;
- 14-digit display enable;
- RAM-address selection from pre-instruction `C[1:0]`.

Bank switching, delayed-ROM selection, PC changes and return-stack effects are deliberately excluded because they participate directly in the concurrent successor-fetch/control-flow path.

## Causal path

The serial endpoint captures the real pre-instruction decimal/display/RAM-address state and creates a private M14H image only for the selected special family. The architectural executor still runs once as semantic oracle and owner of unrelated effects. Its post-instruction M14H fields are captured as expected values and immediately restored.

The structural word then traverses b0..b55. The M14H image remains incomplete through b54. At b55 it applies the decoded latch transition from the immutable pre-state. The live bridge requires exact agreement with the oracle before committing decimal mode, DISPLAY enable, 14-digit-display mode and RAM address.

CRC-owned words are excluded at the composed-machine operation boundary even if a raw ten-bit value matches an ACT special pattern.

## Startup DISPLAY bridge separation

The existing power-on evidence requires display traffic before firmware executes its first explicit DISPLAY-control instruction. Previously that bring-up bridge was implemented by cloning the entire ACT pre-state and forcing `display_enable = true` before binding the execution snapshot.

That is no longer safe once DISPLAY enable itself becomes structural authority: a synthetic display bit must not alter the architectural pre-state used to compute the M14H result.

M14H therefore separates the two concerns:

- `ActSerialStateSnapshot` always captures the real ACT pre-instruction state;
- an execution-local DISPLAY-enable override affects only b0..b7 display serialization during the observed startup window.

The override never changes the M14H mode/latch image.

## Same-word DISPLAY behavior

A DISPLAY-control instruction executes while its own structural word is also carrying display traffic. The b0..b7 source remains the immutable pre-instruction display state for that word. The new DISPLAY latch value becomes live only at the completed-word authority handoff.

Host-side display bookkeeping may consult the oracle-expected post-word DISPLAY value to decide whether the completed instruction leaves the display disabled, but that expected value is not fed back into the structural ACT before b55.

## Evidence classification

### Source-backed / semantic contract

- 56-bit structural execution lifetime;
- established ACT semantics for decimal/hex, DISPLAY control, 14-digit-display enable and RAM-address selection;
- RAM address derives from C digits 1:0;
- direct HP-67 startup evidence requiring early display traffic before the first explicit DISPLAY-control instruction.

### Working approximation

The b55 commit point is a final-state authority boundary only.

### Source-blocked

M14H does not claim:

- the PHI edge that writes the decimal/hex latch;
- the internal DISPLAY-control latch edge;
- the internal timing of the 14-digit mode latch;
- the timing or electrical implementation of RAM-address capture;
- that any of these state changes physically wait until b55.

## Validation required before merge

Owner-side Windows gate:

`cargo fmt --all -- --check; if ($LASTEXITCODE -ne 0) { throw "FMT FAILED" }; $env:RUSTFLAGS='-Dwarnings'; cargo test --locked --all-targets; if ($LASTEXITCODE -ne 0) { throw "TESTS FAILED" }; cargo build --locked --release --bins; if ($LASTEXITCODE -ne 0) { throw "RELEASE BUILD FAILED" }`

Behavior-impacting release diagnostic:

`cargo test --release --locked --bin hp67emu live_custom_diagnostic_pac_suite_reports_ok_ko -- --ignored --nocapture`

Expected closure: `DIAGNOSTIC SUITE OK: 12/12 passed`.

## Validation result

On 2026-09-28 the repository owner reported `M14H FULL GATE GREEN` after running the complete local Windows validation sequence: formatting, focused M14H tests, warnings-denied all-target tests, release build, and the ignored release diagnostic.

This validates the M14H completed-word mode/latch authority bridge. It does **not** change the SOURCE-BLOCKED classification of exact intra-word/PHI-relative decimal, DISPLAY, 14-digit-display or RAM-address latch timing.
