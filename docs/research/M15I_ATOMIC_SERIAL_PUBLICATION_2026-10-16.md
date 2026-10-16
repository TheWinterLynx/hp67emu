# M15I — atomic serial publication

M15H head `93cc87ce51b82d47849a0a6d76d742885a4131c5` was owner-validated with
`HP67-M15H-PHI-RECEIVERS-v1` and integrated unchanged into main. Diagnostic
12/12 retained the same cycle counts. Reported live medians were 1.212 us/word
idle, 2.047 synthetic RAM, 1.275 synthetic CRC. These are host headroom numbers,
not physical timing measurements.

## Problem and executable change

M15H separated source publication from receiver sampling, but source writes
were sequential and word start briefly reset both modeled IS drivers to HighZ.
A DATA proposal error could occur after publishing the new IS bit, leaving a
partially installed source image even though the cell never began clocking.

Production now prepares immutable proposals and commits ACT IS, aggregate ROM
IS, ACT SYNC and optional ACT/aggregate RAM DATA as one externally observable
image. No receiver/evaluation runs between its contribution updates; resolved
nets and contention accounting are refreshed after the full proposal set has
been installed. Other drivers survive. A missing DATA role leaves DATA untouched;
an opted-in idle DATA cell explicitly releases both modeled source contributions.

The launch helper rejects a wrong bit or launch phase before pin mutation.
DATA order/owner/phase validation also precedes the shared commit. The word-start
IS HighZ reset is removed; the previous committed contributions persist until
replacement. Every loop and the clocked tail prefix uses this publication path.
No transition is dropped or added. PHI1 falling/rising, receiver sampling, then
PHI2 falling/rising remain M15H's sequence. Coarse end-word SYNC closure is still
separate and unchanged; this milestone does not reassign its physical edge.

## Fidelity status and evidence boundary

This is a deterministic resolve/evaluate/commit correction, not new hardware
measurement. Source roles, bit coordinates and the explicitly provisional
launch/sample policy are inherited from M15H and its reviewed HP-67/97 sources:
`docs/research/M15H_PHI_RECEIVER_SCHEDULING_2026-10-16.md`.

PROVEN at the simulator contract level: one observable source image, third-party
driver preservation, valid cached resolution/counts, failure before publication,
and zero time progression during commit. The API is a specialized composition
bridge; it is not yet independent physical ROM/RAM device scheduling.

WORKING APPROXIMATION: publication at the after-PHI2 boundary and sampling at
PHI1 rising, abstract DATA logic levels, aggregate RAM ownership and coarse
STR/RCD pulse timing. Physical launch/latch edges, pulse widths, propagation,
output stages and chip mapping remain SOURCE-BLOCKED. No V2 physical edge-table
field is promoted. Replace the bridge/profile as evidence permits per-device
edge timing while preserving the observable commit and ownership semantics.

## Regression and gate

Generic `Net` resolution independently checks 243 possible five-source images
under each of three additional external drive states (729 commits). Tests check
all affected resolved levels, active counts, external preservation, untouched STR,
contention cache and error reporting without advancing time. Explicit handoff
coverage distinguishes absent DATA participation from requested HighZ release.

Wrong-phase/bit publication must preserve all nets and time. A production DATA
word with missing ownership reaches b2 at tick 8 and fails before publishing the
next High display bit: IS stays at the preceding Low level, DATA is Floating,
ROM0 has only its b0/b1 prefix and ACT's scan slot does not advance. Existing
M15F split/whole, M15H receiver-phase and traced/untraced tests remain required.

Rust is unavailable in the agent environment. Code/diff are reviewed here;
Actions is formatting-only. Owner focused/full local tests, release, diagnostic
and all three benchmarks remain required before M15I integration.
