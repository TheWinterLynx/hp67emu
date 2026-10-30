# HP-67 pin captures v1.6 — production source integrity audit

The TSV format, nineteen net indices, driver indices and capture hooks are unchanged.
The new native command is:

`hp67emu --integrity-pins input.tsv [new-report.txt]`

Available in ordinary and pin-capture native builds. It reads bounded input and
writes only a new output path, or prints to stdout. It never modifies the trace.
Use --trace-pins with the capture feature to obtain actual live observations.

Exit 0 means the thirteen scoped production bridge contracts were observed over
a whole-word capture with known levels/owners and complete address observations.
Exit 2 means a source/level contract violation. Exit 3 means missing coverage,
including physical traces without internal ownership masks, sparse legacy pins,
incomplete address or a partial capture. Malformed input remains an error.

PHI ownership is audited at every row. Other rules use the model's provisional
PHI1-rising checkpoint, except STR/RCD at b0/b7/b8: the display receiver publishes
then samples the joint pair at the same tick, so v2 audits the final same-tick
delta image for those controls. It never borrows a later tick; other receivers
retain the original edge image. Startup Floating observations are preserved. Sole source roles: PHI1/2, SYNC and RCD ACT; STR ROM0;
F2/MOTOR CRC; MTRS/HDS/WPS/PGMSW External. Binary output stages remain existing
normalized bridge policy. IS rejects active Low, multiple High, wrong-window and
wrong-page sources, but accepts passive Low release. Return candidates use the
observed address page and existing ROM profiles; the bank-1 inference/equivalence
assumptions are inherited, and exact bank selection is not certified. Reference
aggregate IS responses are intentionally outside this production audit scope.
DATA permits Floating/release or one ACT/RamAggregate source; it does not infer
transfer direction, tail payload, physical RAM IC mapping or CRC serialization.

All nineteen pins appear in the report. F1 and KC1..KC5 are explicitly UNASSESSED.
The first 64 fault details include net/time/delta/word-bit/level/owners; counters
continue for all violations. Absence is not High, Low, Floating or successful
ownership. Physical captures must never acquire invented internal owners to pass.

This is not hardware conformance certification. Every report retains unresolved
PHI widths/dead time, exact physical launch/latch/propagation, DATA output stage
and physical RAM mapping, key acquisition and magnetic sense/serialization.
No runtime callback, pin mutation, timing profile or nanosecond calibration is added.
