# M14G — special-register completed-word structural authority

Date: 2026-09-27  
Branch: `agent/m14g-special-register-authority`

## Goal

Move final data-register authority for a focused ACT non-arithmetic special-opcode family from the instruction-boundary executor onto the structural `b0..b55` execution lifetime, while preserving the existing semantic oracle and leaving exact internal ACT timing source-blocked.

## Included register families

M14G independently decodes and owns the completed-word result for:

- clear A/B/C/Y/Z/T;
- exchange C with M1;
- copy M1 to C;
- exchange C with M2;
- copy M2 to C;
- A/Y/Z/T stack-transfer form;
- C/Y/Z/T down-rotate form;
- Y to A;
- C into Y/Z/T stack;
- F to A digit 0;
- exchange A digit 0 with F;
- key-buffer value to A digits 2:1;
- A rotate-right;
- the C-digit side of load-constant.

The P decrement of load-constant remains owned by the already-validated M14F control image.

## Causal path

The shared serial endpoint captures A/B/C/Y/Z/T/M1/M2/F/key/P from the pre-instruction state and creates a private M14G image only for the included special-register family.

The architectural executor still runs once as semantic oracle and as temporary owner of fields not migrated by this slice. For an ACT-owned M14G word, its post-instruction register image is captured as the expected result and the migrated registers are immediately restored to their pre-instruction values.

The structural execution then traverses b0..b55. The M14G image remains incomplete through b54 and applies the decoded transformation only once the structural execution reaches b55. The live bridge hard-compares every migrated register against the oracle and commits only the structural image.

CRC-owned words are excluded at the composed-machine operation boundary even if their raw ten-bit value could otherwise match a special-opcode pattern.

## Load-constant split authority

Load-constant is intentionally split across two structural result images:

- M14F owns P, P-change history, carry/previous-carry and instruction-state bookkeeping;
- M14G owns the conditional `C[P] = operand` register mutation using the captured pre-instruction P.

Both images are built from the same immutable pre-state and complete at the same structural word boundary. Neither reads the other's post-oracle mutation.

## Evidence classification

### Source-backed / semantic contract

- complete 56-bit structural execution lifetime;
- special-register instruction semantics already cross-checked against the independent Woodstock reference;
- one-word pipeline ordering;
- fourteen-digit register widths and the four-bit F/key nibble semantics used by the HP-67 firmware.

### Working approximation

The b55 completion/commit point is a scheduler authority boundary. It establishes which model produces final live state, not when the physical ACT changes an internal latch.

### Source-blocked

M14G does not claim:

- exact internal shift-register routing for Y/Z/T/M1/M2;
- the PHI edge that captures a transferred register bit;
- whether stack and memory-register transfers become internally visible progressively through the word;
- the physical timing of F/A0 exchange;
- the exact internal path used for key-buffer transfer;
- that load-constant writes C atomically at b55.

## Validation required before merge

Owner-side Windows gate:

`cargo fmt --all -- --check; if ($LASTEXITCODE -ne 0) { throw "FMT FAILED" }; $env:RUSTFLAGS='-Dwarnings'; cargo test --locked --all-targets; if ($LASTEXITCODE -ne 0) { throw "TESTS FAILED" }; cargo build --locked --release --bins; if ($LASTEXITCODE -ne 0) { throw "RELEASE BUILD FAILED" }`

Behavior-impacting release diagnostic:

`cargo test --release --locked --bin hp67emu live_custom_diagnostic_pac_suite_reports_ok_ko -- --ignored --nocapture`

Expected closure: `DIAGNOSTIC SUITE OK: 12/12 passed`.

## Validation result

On 2026-09-27 the repository owner reported `M14G FULL GATE GREEN` after running the full local Windows sequence: formatting, focused M14G tests, warnings-denied all-target tests, release build, and the ignored release diagnostic.

This validates the M14G completed-word special-register authority bridge. It does **not** change the SOURCE-BLOCKED classification of exact intra-word/PHI-relative register-transfer timing.
