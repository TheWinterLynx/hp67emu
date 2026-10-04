# M14P — official Pac behavioral acceptance

Date: 2026-10-04
Branch: `agent/m14p-official-pac-acceptance`
Status: implemented; owner local execution pending.

## Evidence and scope

The supplied `01-hp97-pac-standard-en.pdf`, PDF page 16 / printed 01-03, Moving Average Example 1, was visually checked after text extraction. It initializes a six-item window using shifted A and enters 125, 183, 207, 222, 198, 240 with A. D requests the partial mean after the third input. Expected stable outputs are 6.00, 1.00, 2.00, 3.00, 171.67, 4.00, 5.00 and 195.83. Example 2 supplies 225 and expects 212.50; our ninth checkpoint retains the in-memory window instead of exercising its saved-data/power-off recovery. Transient counters before final means are not acceptance assertions.

This is Tier B behavioral corroboration for the existing HP-67 SD1-01A corpus. HP-97 printer operations, printer timing and electrical equivalence are excluded. Until the owner runs the gate, this is a candidate acceptance fixture, not a claimed successful HP-67 example. No new program/manual bytes are redistributed.

## Implementation and ownership

A fresh live machine boots real ROM, imports the existing two-track library physical card, inserts End1, waits for firmware Crd, inserts the returned same card End2, then waits for the reader to settle. Shared diagnostic transport and M12 physical-dispatch helpers remain test-only. Every input is a held/released key contact observed through keys-to-A / A-to-ROM dispatch. Every checkpoint requires return to firmware no-key RUN wait, S2/S15 clear and exact fifteen-slot segment output. Each post-dispatch wait is bounded to one million words and failures identify checkpoint/key/PC/status/raw display. No RAM, PC, register or display result is injected. Expected masks are test oracles only. Production execution is unchanged.

## Validation and next priorities

Actions runs cargo fmt/check/save only. Owner runs focused non-ignored M14P corpus regression, all targets with warnings denied, release binaries, Custom Diagnostic Pac 12/12 and the explicit ignored `m14p_official_moving_average_acceptance` (9/9 checkpoints). No Rust execution was performed in the agent scratch environment.

Next: data-card save/reload recovery, further Standard/Games Pac manual examples, then source-backed electrical gaps. A single successful application cannot establish full Pac compatibility or close DATA ownership, PHI edges, STR/RCD propagation or magnetic sense/serialization.
