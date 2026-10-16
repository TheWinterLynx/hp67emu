# M15H — provisional PHI receiver scheduling

M15G head `21adfe7bba003490d56878f2bc04b8567fdc8bcb` was owner-validated with
`HP67-M15G-RESOLVED-DISPLAY-v1` and integrated unchanged into main.

## Why the previous boundary was insufficient

M15G published and consumed serial bits before all four clock transitions of
their cell. Clock counts and pin resolution were real, but receiver chronology
was still a bit-cell bridge. A functional 12/12 diagnostic cannot establish a
chip's physical latch edge. This milestone moves consumption into the clock
sequence and keeps physical certainty separate from execution policy.

## Inspected evidence

Teenix, *Notes on HP's Classic Calculators* (2025), printed page 70 (PDF page 71):
https://literature.hpcalc.org/community/classic-notes.pdf
SHA256 `bdc7a403e22b03f4305d2d7b638ff4358e49e2d5ae5328f590f6946f243c66da`.
The expanded IS/DATA waveforms show transitions around PHI2 and stable levels
across PHI1. This supports using the PHI1 interval as a provisional observation
window. It does not reveal each receiver's internal latch edge or a bounded
propagation delay. The SYNC detail still shows unequal offsets around PHI2
rising; M15A's removal of an exact SYNC-edge claim remains correct.

HP-97 Service Manual, printed pages 2-4/2-5 (PDF pages 18/19), sections 2-14–2-18,
figure 2-5: https://www.teenix.org/97ServMan_2.pdf
SHA256 `7a102cd7051ed63b23b62b18f6768083f7e4abf4e607d9aa978541dfb25c5048`.
Visually reviewed under the owner's HP-97 equivalence instruction. The diagram
confirms word-level IS/SYNC/DATA windows and source roles, but has no PHI1/PHI2
waveforms and cannot itself select latch edges. The attached Pac program
manuals are not electrical timing references for this decision.

## Execution policy: WORKING APPROXIMATION

1. Publish IS/SYNC and opted-in DATA source contributions at the existing
   after-PHI2 interphase boundary (the word's first boundary initializes this).
2. Advance and commit PHI1 falling, then PHI1 rising.
3. Validate the actual clock phase/levels and sample resolved IS and DATA.
   ROM0 receives its bit; ROM receives address bits; ACT receives the fetched
   word plus SYNC. DATA sampling never republishes expected source bits.
4. After complete b7 display reception, publish/sample the existing coarse
   STR/RCD bridge. Its pulse ends at the b8 receiver checkpoint.
5. Advance and commit PHI2 falling/rising; finish existing ACT cell bookkeeping.

PHI1 rising is an explicit provisional checkpoint, not a proven silicon latch
edge. Publishing at the after-PHI2 boundary does not assert zero physical
propagation or exact launch alignment. No nanosecond duration is assigned to
ordinal ticks. STR/RCD simultaneous publication remains a coarse model; physical
falling order/output stages and cathode application inside the word remain
unproven. The physical V2 table retains `None` fields and its version unchanged.
Replace the policy when hardware captures/datasheets settle the actual latch,
launch and propagation constraints. This policy removes pre-clock consumption;
it does not claim to complete all electrical timing.

## Invariants and regression

The four production/diagnostic variants and clocked b0/b1 prefix use the same
phase order. A prefix advances eight transitions; a resumed body advances 216.
Successful words still have 224 transitions and unchanged IS/PHI trace rows.
The legacy direct combined DATA visitors remain reference-only.

Tests reject sampling before PHI1 rising and at later phases without mutating
DATA state; cover opposite-direction transfers and next-word tail completion;
inject same-level and opposite-level extra DATA contributors between publication
and sampling; and prove a changed committed source contribution is observed
without being overwritten at reception. IS phase/ownership checks and ROM0
control publication have focused regressions too. Existing split/whole and
traced/untraced tests cover the shared path.

Failure chronology deliberately changes: an invalid IS b5 sample stops at tick
22, a failed ROM lookup at b27 stops at tick 110, and SYNC contention at b46
stops at tick 186. The trace retains the two committed PHI1 transitions of the
failing cell; no missing rows are discarded. Tests keep their no-consumption and
no-scan-advance requirements and additionally check the final phase.

Rust is unavailable in the agent environment. Diff/code review is performed
here; Actions is formatting-only. Owner focused/full local tests, release,
diagnostic and electrical/DATA/live benchmarks remain required before merging.
