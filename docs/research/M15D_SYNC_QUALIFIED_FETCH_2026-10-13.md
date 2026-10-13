# M15D SYNC-qualified fetch — 2026-10-13

## Owner scope and source

Owner: advance executable electrical fidelity and assume HP-97 equivalence.
This overrides the earlier requirement to prove model equivalence separately.
HP-97 service manual, figure 2-5 and sections 2-14/2-15, printed 2-4/2-5;
reviewed PDF page 19 visually and the adjacent timing prose:
https://www.teenix.org/97ServMan_2.pdf.
SHA-256: `7a102cd7051ed63b23b62b18f6768083f7e4abf4e607d9aa978541dfb25c5048`.

ACT supplies SYNC. Its b46..b55 pulse qualifies an IS instruction; an IF
suppresses that pulse for the following fetched implied-GOTO address. Existing
HP-67 evidence agrees on those bit windows. No exact PHI edge is supplied by
this manual figure. Do not reopen M15A's withdrawn exact PHI2-rising claim.

## Executable change

All four ordinary/DATA/traced transport loops now drive the ACT SYNC output.
Normal words publish High at the existing b46 coordinate and Low after b55;
IF successors remain Low. Suppression is derived solely from the bound
structural executing word, including normal versus implied-GOTO classification,
not post-oracle state or the fetched word's bits. Receiver samples resolved
SYNC for every IS return bit, requires ten stable qualifiers, and fails
floating, opposing-drive contention or mixed qualification. A partial/IS-only
word cannot supply next-state authority.

After existing completed-word state images pass oracle agreement, live execution
compares the received qualifier against the expected next decode state and
commits the received instruction/data classification. The real firmware path,
IS driver ownership, 56 cells, four PHI transitions/cell and DATA logical tail
remain intact. TSV format stays compatible and still omits electrical SYNC.

## Fidelity classification

HP-97 equivalence is an owner-authorized WORKING ASSUMPTION. Pulse qualification
and return-window behavior are manual-backed under that assumption. Publishing
at coarse bit-cell boundaries is a WORKING APPROXIMATION, like current IS
transport; no physical launch/sample edge, skew, drive strength or propagation
is claimed. The next latch commit remains completed-word structural authority,
not an internal ACT latch-time assertion. Default unpowered/reset nets remain
floating; this slice starts driving only in actual word transport, with coarse
Low at word boundaries. Reset waveform and exact PHI timing remain unresolved.

## Validation

Focused library regressions compare successor pulse decode against every
supported ten-bit architectural word in normal/implied-GOTO modes and both
carry states; exercise normal/IF/implied-GOTO sequencing in all four loops;
inject SYNC contention to fail at b46 before IS reception; and reject floating
or changing qualification. Focused native regression executes 1200 real firmware
words and requires both pulse classes and exact transition totals. Ordinary full
suite includes existing live/traced parity and DATA authority regressions.
Owner local gate additionally runs diagnostic 12/12 and electrical/DATA/live
benchmarks. No Rust compilation/test success is claimed by this environment,
which has no installed Rust toolchain. GitHub Actions is formatting only.
