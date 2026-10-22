# M15U — ACT-owned received instruction pipeline

Date: 2026-10-22
Base: owner-validated M15T `f635ea8732baad5eb38fdc88dba59a76ecab403f`, integrated into main.
Branch: `agent/m15u-act-received-execution`
Status: implemented; owner local gate pending.

## Problem and complete lane

The ACT previously reconstructed IS and SYNC, but the next execution was still
started using the host pipeline's word. IS reception used a completeness mask
that admitted duplicate/reordered callbacks, and the two receive latches were
updated separately. M15U makes reception, retained command, classification and
next-word execution one production lane.

A qualified complete b46..b55 frame enters an ACT-owned pending latch only after
all ten ordered samples. A single sampler validates current provisional clock
phase, IS, SYNC, sequence and stable qualifier before mutating either receive
image. A live pending frame cannot be overwritten. Binding consumes the pending
frame only after the existing execution-availability and qualifier checks pass.
The retained frame's word supplies production execution and the oracle comparison;
the host FetchPipelineLatch is checked as a mirror, never used as opcode source.
The qualifier mirror is checked before execution, preserving implied-GOTO data.

Live mirror checks occur before pipeline advancement and before the next DATA
tail prefix can advance PHI. Production reset clears the pending frame and keeps
this composition enabled. Only explicit injected-opcode fixtures disable received
execution, alongside their existing reference RAM/CRC setup. All four shared
transport variants call the same sampler. Standalone IS-only adapters remain
available with contiguous ordered samples and no qualified pending publication.

## Evidence and limits

This uses the established HP-67 b46..b55 ten-bit LSB-first return, SYNC instruction/
data distinction and execution in the following 56-bit word. See the existing
HARDWARE_SOURCES references, M15D_SYNC_QUALIFIED_FETCH_2026-10-13.md and
M15M_ROM_RECEIVER_ORDER_2026-10-18.md. It closes causal ownership and receive
validation; no opcode semantics or hardware timing value changes.

The M15H PHI1-rising checkpoint remains WORKING APPROXIMATION. Exact internal
instruction latch edges, ACT intra-word write edges, propagation, physical DATA
output stage, display pulses and magnetic serialization remain SOURCE-BLOCKED.
The architectural ACT snapshot and completed-word result images retain their
existing scope. This does not claim a transistor-level ACT implementation.

## Regression and performance scope

Eight focused regressions cover all 1024 words with both SYNC qualifiers, complete
56-cell execution, phase/order/replay failures, atomic IS/SYNC fault preservation,
modified observed pins, mixed IS-only/qualified API rejection, host-binding bypass rejection, retained-frame overwrite and active-execution rejection.
Native acceptance compares traced/untraced production with reference firmware,
real contacts, cross-word DATA and implied GOTO; reset retains the selected mode.
Poisoned host word/qualifier mirrors fail before clocks or DATA-tail consumption.

No full ACT image copy, heap allocation or extra electrical transition is added
per return sample. The production live benchmark includes the new receiver and
pipeline; synthetic injected rows remain reference workloads. Actions is limited
to cargo fmt/check/save. All compilation/runtime/diagnostic/benchmark validation
remains the owner's local gate.
