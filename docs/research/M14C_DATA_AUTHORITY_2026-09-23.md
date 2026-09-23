# M14C — installed-RAM DATA authority

Date: 2026-09-23  
Branch: `agent/m14c-data-authority`

## Goal

Move ordinary installed-RAM ACT transfers one step out of the instruction-boundary fallback and into the source-backed logical DATA path without inventing electrical facts that remain unknown.

M14C is deliberately narrower than a physical 1818-* RAM implementation. It does not assign DATA polarity, passive bias, driver-enable timing, PHI launch/sample edges, propagation delay, ACT internal register-write edges, or chip/address partitioning.

## Evidence boundary

Direct HP-67 capture evidence already locked by M14A/M14B establishes:

- one DATA register frame is 56 logical bits;
- serial bit 0 is at machine-word b2;
- b2..b55 therefore carry serial bits 0..53;
- serial bits 54/55 appear at b0/b1 of the following machine word.

The exact electrical representation of logical zero/one and the exact PHI-relative sample/commit edge are still SOURCE-BLOCKED.

## Causal change

Before M14C, the live machine executed the architectural ACT/RAM instruction first. That immediately changed C or RAM, while the fused DATA path merely reconstructed the same payload one word later and compared it.

M14C changes installed RAM addresses only:

1. classify the RAM transfer from the pre-instruction ACT state;
2. capture the logical DATA source register;
3. run the architectural executor as an oracle for the whole instruction;
4. verify its transfer destination equals the captured expected payload;
5. restore only that C/RAM transfer destination while preserving the instruction's other architectural effects;
6. serialize the payload through the existing b2..b55 + following b0/b1 DATA path;
7. reconstruct the 56-bit register;
8. hard-fail if it differs from the architectural oracle;
9. commit the reconstructed register to C or RAM.

The architectural executor therefore no longer causes the installed-RAM transfer destination on the live path.

## Following-word b0/b1 ordering

A DATA frame completes in the following word before that word reaches b2. The next instruction may legitimately depend on the completed C/RAM value. To preserve that causal relationship while DATA is still logical-only, `Hp67DataSerialWordPath::preconsume_previous_tail()` consumes logical DATA b0/b1 before the following instruction-boundary bridge runs.

The later shared structural word still traverses physical b0/b1 for IS/display/fetch/backplane timing. Its DATA visitor skips only those two already-consumed logical samples, then resumes at b2. This is a scheduler bridge, not a claim that the ACT physically commits C or RAM at a particular PHI edge.

## Exclusions

- CRC/card DATA ports `0x99` and `0x9B` are not installed `ActRamImage` slots and remain owned by the CRC/card path.
- The temporary aggregate `ActRamImage` is not split into 1818-0231/0232/0550/0551 devices.
- `Hp67Net::Data` is not driven or sampled.
- No DATA High/Low polarity or bias is chosen.
- No exact PHI edge or propagation delay is introduced.

## Validation required before merge

Owner-side Windows gate:

`cargo fmt --all -- --check; if ($LASTEXITCODE -ne 0) { throw "FMT FAILED" }; $env:RUSTFLAGS='-Dwarnings'; cargo test --locked --all-targets; if ($LASTEXITCODE -ne 0) { throw "TESTS FAILED" }; cargo build --locked --release --bins; if ($LASTEXITCODE -ne 0) { throw "RELEASE BUILD FAILED" }`

Behavior-impacting release diagnostic:

`cargo test --release --locked --bin hp67emu live_custom_diagnostic_pac_suite_reports_ok_ko -- --ignored --nocapture`

Expected diagnostic closure remains `DIAGNOSTIC SUITE OK: 12/12 passed`. This document does not claim that gate has run on M14C until the owner reports it.
