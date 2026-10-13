# M15B IS contribution guard — 2026-10-13

## Evidence

Teenix 2025 direct HP-67 experiments, printed page 74 / PDF page 75:
https://literature.hpcalc.org/community/classic-notes.pdf.
PDF SHA-256: `bdc7a403e22b03f4305d2d7b638ff4358e49e2d5ae5328f590f6946f243c66da`.
The IS resistor experiment identifies passive-low/active-high behavior; the
shared-bus discussion requires sole-chip control. Do not extend the IS-specific
polarity experiment to DATA.

## Problem and correction

Two committed High contributions resolve High in the generic electrical model,
so checking only Contention misses a simultaneous active drive from two chips.
A sole Low contribution resolves Low but violates observed IS drive polarity.
The four structural IS transport loops now reject both cases before receiver
sampling. Opposite drives retain their existing contention error. The error
includes b0..b55 and the complete active-driver mask (Hp67Driver slot bits).
No partial word is completed or invalid sample accepted.

## Boundaries

This is enforcement of an existing evidenced drive policy at the current
structural bit sample, not a new physical device or latch edge. Several chips
releasing simultaneously cannot be diagnosed without modeled output-enable
state. Immediate bridge updates retain their existing scheduling approximation;
staged handoff is checked only after every proposal commits. Generic resolution
and other buses are unchanged. SYNC V2, DATA, STR/RCD, PHI widths, propagation
and magnetic timing remain source-blocked. Additional externally imposed IS
drives are treated as contributors during transport; no analog resistor model
or harmless probe-current exception is represented.

## Regression and validation

Tests distinguish resolved High from invalid same-level overlap; check release
recovery and active Low; exercise both staged handoff orders; inject a persistent
High into all four transport variants and require first-overlap failure at b5,
20 previous transitions and an incomplete ROM0 display byte. Existing complete
scan/firmware/DATA parity regressions cover valid transport behavior.
The cached valid-path check allocates nothing; diagnostics scan only on failure.
Owner benchmark gates must measure the added per-cell check before integration.
Rust is unavailable in the agent environment. Only formatting is authorized in
GitHub Actions; focused/full tests, release, diagnostic and benchmarks run on
the owner's machine. No passing test result is claimed before those gates.
