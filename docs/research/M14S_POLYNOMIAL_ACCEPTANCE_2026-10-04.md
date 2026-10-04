# M14S — official Polynomial Evaluation acceptance

Date: 2026-10-04
Branch: `agent/m14s-polynomial-acceptance`
Status: implemented; owner local gate pending.

## Source and scope

The supplied HP-97 Standard Pac manual PDF pages 62/63, printed 09-03/09-04, were rendered and visually inspected. Polynomial Evaluation instructions define shifted A initialization, B/C/D/E coefficient input with counters 1/2/3/4 and A evaluation. Example 2 uses coefficients -9140, -7.596, 4.243e-3 and -0.742e-6. It reports -11547.01, -12330.39 and -12881.18 at 400, 600 and 800 K respectively. Independent Decimal arithmetic gave -11547.008, -12330.392 and -12881.184, corroborating the fixed expected rounding.

This is Tier B behavioral corroboration against existing HP-67 SD1-09A corpus media. Shared function keys were checked visually; OCR alone was insufficient for shifted labels and exponent signs. No printer/root-output behavior, full Pac compatibility or HP-67/97 electrical equivalence is claimed. No new manual/program bytes are redistributed.

## Implementation

A fresh live machine boots real ROM and loads both program ends through existing firmware Crd transport. Eight checkpoints exercise initialization, constant/linear negative coefficients, negative exponent entry, negative mantissa plus negative exponent, then three evaluations retaining coefficient state. All inputs are physical key contacts observed through real firmware dispatch. Every expected value requires no-key RUN wait, S2/S15 clear and exact physical segment equality. Bounds and failure context reuse the existing Pac runner. No calculator state or result is injected.

The only shared helper change is test-only expected-frame support for one leading negative mantissa sign. Position zero uses the G segment, independently established by `shared_sign_anodes_are_split_into_two_physical_minus_positions`. Positive digit positions remain unchanged. A normal test asserts a literal -12.34 raw frame and rejects embedded minus signs, as well as requiring both program tracks. This helper constructs assertions, never production display output.

## Validation and next work

Agent reviewed diff, manual renders, independent expected arithmetic and documentation contract; no Rust compilation/tests ran here. Actions performs cargo fmt/check/save only. Owner gate: focused official Pac regressions, warnings-denied all targets, release binaries, Custom Diagnostic Pac 12/12, existing Moving Average suites and explicit ignored `m14s_official_polynomial_evaluation_acceptance` (8/8). Successful execution is pending.

Next: root-output acceptance and Games Pac functional examples, while retaining source-blocked exact DATA/PHI/STR/RCD and magnetic sense/serialization boundaries. Functional numerical agreement cannot establish hardware timing.
