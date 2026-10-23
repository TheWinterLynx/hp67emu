# M15W — retained ACT register file and explicit input ports

Date: 2026-10-23
Base: owner-validated M15V `e1f8e22160c671742b419ac2c1b7e5b8eb9164a2`, integrated into main.
Branch: `agent/m15w-act-owned-registers`
Status: owner FULL GATE GREEN; exact `384c11197c8a30e0482f44aa5658a1bf4d2a058a` integrated into main.
Diagnostic 12/12 with unchanged cycles; live median 2.688 us/word (119.06x hardware).

## Completed scope

The ACT endpoint retains A/B/C/Y/Z/T/M1/M2/F, P and pointer history,
carry/previous carry, radix, status, instruction state, key buffer, display
controls and RAM address. Combined with M15V flow, this is the full ACT state.
All structural execution snapshots and display sources now use retained device
state. The host state is checked before clocks or DATA tails, then replaced from
the owned result only after existing image/oracle comparisons succeed.

Existing arithmetic, control, special-register, mode, boundary, flow and RAM
images apply in the established live authority order at completed b55. Installed
RAM read C waits for the actual reconstructed DATA tail through b1 of the next
word. Keyboard inputs sample physical contact state; CRC S3 uses the independent
CRC command result and preserves sticky true. Received IS/SYNC supplies the next
instruction qualifier. Logical M14M CRC C and M14N absent-RAM compatibility keep
their established result images. Reset reconstructs owned storage; explicit
injected reference fixtures disable it. No heap allocation or full ACT clone is
introduced per bit.

## Evidence boundary

This changes ownership, not opcode semantics or physical timing. Existing M14
result-image evidence and M15 transport contracts apply unchanged. Logical b55
publication and the provisional receiver checkpoint remain WORKING APPROXIMATION.
Exact internal register writes, PHI widths/dead time and launch/sample edges,
STR/RCD pulse edges, propagation, physical RAM output stages/chip mapping and CRC
magnetic sense/serialization remain SOURCE-BLOCKED. HP-97 equivalence remains
accepted by the owner; no invented timing value closes an evidence gap.

## Acceptance

Three library regressions compare supported nonperipheral opcodes across all
1024 raw words, both qualifiers, carry, radix, P and RAM topology contexts;
check storage through b54 and publication at b55; reject wrong-phase/active
ports; preserve sticky keyboard/S3; and demonstrate owned binding below a
poisoned host mirror guard. Installed RAM reads are covered by the live path.
Three native regressions cover traced/untraced/reference firmware with contacts,
DATA tails and reset; individual register/latch mirror poisoning before clocks;
and logical card CRC readiness/consumption. Oracle-rejected words gain no new
semantics. Existing full regressions and ignored 12/12 diagnostics remain required.

Actions runs formatting/check/save only. Compilation, runtime, release and all
benchmarks are the owner's local gate. No runtime pass is claimed here.
