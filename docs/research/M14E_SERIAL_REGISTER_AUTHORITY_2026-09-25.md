# M14E — serial register arithmetic authority

Date: 2026-09-25  
Branch: `agent/m14e-serial-register-authority`

## Goal

Extend M14D's structural final-state authority from ADD/SUB to the complete Woodstock arithmetic opcode family `0x00..0x1f`, without inventing exact internal 1820-2530 register-write or PHI timing.

## What changes causally

For every arithmetic word, the live machine now treats the structural `ActSerialArithmeticResultImage` as the producer of final A/B/C/carry.

The instruction-boundary architectural executor still runs once because it currently owns non-migrated effects such as PC advancement, `previous_carry`, branch/test state and other control fields. Its resulting A/B/C/carry values are captured only as the semantic oracle and are immediately restored to the pre-instruction values.

The actual shared `b0..b55` structural traversal then constructs the arithmetic result:

- clear writes zero to selected destination digits;
- copy reads the immutable pre-instruction source and writes selected destination digits;
- exchange swaps selected source digits in the private result image;
- ADD/SUB retain the M14D digit result and carry/borrow chain;
- shift-left sources the preceding digit only when it belongs to the selected field, otherwise zero;
- shift-right sources the following digit only when it belongs to the selected field, otherwise zero;
- nonzero tests OR selected source digits into final carry;
- zero tests AND selected source digits into final carry;
- destination-less compare operations preserve A/B/C and derive carry from the structural subtraction chain.

After the structural word reaches b55, the result image is compared against the architectural oracle. Any A/B/C/carry mismatch is a hard live-machine error. Only the structural result is committed.

## Differential coverage

The result-image replay regression covers all 32 arithmetic operations across:

- all eight Woodstock fields;
- P values 0, 2, 7, 13 and an out-of-range P=15 case;
- decimal and hexadecimal modes;
- nontrivial A/B/C patterns;
- initial carry set, proving non-ADD/SUB families still produce the architecturally cleared/test-defined carry result.

Dedicated live-bridge coverage also exercises non-ADD arithmetic families through the same defer → structural traversal → oracle compare → commit path used by production firmware.

## Evidence classification

### Source-backed / semantic contract

- 56-bit word lifetime and the `b0..b55` structural coordinate;
- Woodstock arithmetic operation/field decoding;
- A/B/C source and destination routing;
- ADD/SUB carry/borrow direction and opcode seed;
- field membership and shift direction;
- final instruction-boundary semantics cross-checked against the existing architectural model.

### Working approximation

The private image records completed digit results at the fourth coordinate of each digit and the live machine commits the final image after b55. These are scheduler/result boundaries only.

### Source-blocked

M14E does **not** claim:

- that a physical ACT nibble is written at b3;
- the PHI edge that launches or captures register data;
- exact carry visibility between internal bit cells;
- exact decimal-correction holding-register timing in 1820-2530;
- when an in-flight clear/copy/exchange/shift becomes visible to another internal consumer;
- that the physical ACT performs one atomic whole-register commit at b55.

## Remaining ACT boundary

With M14E implemented, final A/B/C/carry authority for all arithmetic opcodes no longer comes from the instruction-boundary executor. The next architectural-authority removals are control/state families: P/status state, previous-carry/condition flow, return stack, delayed-ROM and related special instructions.

Those migrations must remain separated from true intra-word register timing. Exact electrical timing still requires new hardware evidence.

## Validation required before merge

Owner-side Windows gate:

`cargo fmt --all -- --check; if ($LASTEXITCODE -ne 0) { throw "FMT FAILED" }; $env:RUSTFLAGS='-Dwarnings'; cargo test --locked --all-targets; if ($LASTEXITCODE -ne 0) { throw "TESTS FAILED" }; cargo build --locked --release --bins; if ($LASTEXITCODE -ne 0) { throw "RELEASE BUILD FAILED" }`

Behavior-impacting release diagnostic:

`cargo test --release --locked --bin hp67emu live_custom_diagnostic_pac_suite_reports_ok_ko -- --ignored --nocapture`

Expected closure remains `DIAGNOSTIC SUITE OK: 12/12 passed`. This document does not claim validation success until the owner reports it.
