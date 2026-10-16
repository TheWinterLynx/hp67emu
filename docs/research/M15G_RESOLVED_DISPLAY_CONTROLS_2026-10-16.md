# M15G — resolved STR/RCD display controls

M15F head `394f2ffd7aa7a7ddb35fae5cb5811298107e5f7a` was owner-validated with
`HP67-M15F-CLOCKED-DATA-TAIL-v1` and integrated unchanged into main.

## Evidence and approximation

Teenix, *Notes on HP's Classic Calculators* (2025), printed pages 76–77
(PDF pages 77–78), https://literature.hpcalc.org/community/classic-notes.pdf,
places STR at display-data b7, describes falling STR and falling RCD, and shows
RCD overlapping the final STR of the fifteen-slot scan. These observations
support polarity, source roles and coarse slot/bit placement. The owner's HP-97
equivalence direction remains a working assumption, not an exact PHI claim.

WORKING APPROXIMATION: drive logical High outside b7 and Low during b7; drive
RCD Low in the same cell only for slot 15. Restore High at b8. Publish both
outputs before sampling either pin and before that cell's four scaffold PHI
transitions. This is a one-cell pulse adapter, not a measured width or an
assignment to PHI1/PHI2. Explicit High/Low is abstract logic encoding; actual
output circuitry, voltage, delays and relative STR/RCD falling order remain
SOURCE-BLOCKED. Replace this adapter when captures establish exact edge/width
constraints. No passive bias or inferred transistor implementation is introduced.

## Executable change

ROM0's receiver validates resolved STR/RCD and requires exactly one active
contributor on each pin. High-to-Low STR is accepted only after complete b0..b7
IS display reception and a previously observed High state. The output event
bundle includes the resolved RCD value. Both source contributions are published
before either input is observed; no device evaluates between those writes.
The composition helper uses the named ROM0 STR driver and named ACT RCD driver;
it does not give ROM0 authority over ACT scan selection. All four display
transports plus the clocked DATA prefix share this kernel. The live display
therefore receives pin-derived controls. Downstream cathode/intensity application
remains at completed-word granularity, an explicit remaining approximation.

Same-level extra ownership and opposite-level contention fail contextually;
failed sampling does not emit an event or advance ACT's scan slot. Existing
56-cell/224-transition words and reference-only structural event API remain.
Legacy IS/PHI TSV has no new columns. No magnetic or physical RAM mapping claims.

## Regression and local validation

New tests check all fifteen slots, b7 pulse presence through four PHI transitions,
final-slot overlap, downstream cathode wrap, 3360 transition total, both control
nets under agreeing/opposing extra drivers, and transport failure before scan
advance. M15F split/whole comparison covers the same control receiver during
prefix continuation. Full local tests, release, diagnostic 12/12 and all three
benchmarks are required. Rust is unavailable in the agent environment; code and
diff were reviewed here. Actions is formatting-only; local runtime validation
is pending.
