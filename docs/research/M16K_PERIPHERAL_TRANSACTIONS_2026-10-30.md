# M16K — peripheral field transactions, delivery 14/15

Date: 2026-10-30
Base: owner-validated/integrated ddb3cc0e5331c20f01cfc5171a4ef4bd64a08297
Branch: agent/m16k-crc-transactions
Status: implementation prepared; owner runtime/performance gate pending.

## Problem and resulting behavior

M16J copies the complete CRC controller in the shared DATA sample, then copies
it again inside CRC reception on each of 56 cells. These images include twelve
retained flags and twelve frozen contacts even when only expected_bit advances.
RAM also proposes an unchanged command receiver outside the return window.

M16K validates the same coordinate/PHI image and prepares only the applicable
change: b0 decoded pending command/result reset, b1..b45 coordinate progress,
b46..b55 validated independent IS/SYNC receiver. Commit is infallible after all
shared DATA/RAM-command/sink checks succeed. It updates the same fields at the
same coordinates. The preceding qualified command still executes at b55; the
just-received word cannot execute itself. The CRC proposal is crate-private and
non-Copy, immediately consumed without intervening endpoint mutation. It contains
no retained flag/contact arrays. Other participants cannot inspect partial commits.

Both the DATA command observer and autonomous RAM propose a changed command
receiver only at b46..b55, retaining it in place otherwise. Cross-word DATA tails,
phase/order/ownership faults, passive IS release, source identities, CRC freeze,
READY/F7 handoff, F2/MOTOR/contact behavior and explicit reference composition are
unchanged. No cache maintenance or new per-edge callback is added.

## Evidence and boundaries

This is a representation change under the existing executable contract; it
introduces no hardware-visible semantic or timing rule. Relevant evidence and
approximation boundaries remain those in HARDWARE_SOURCES, ARCHITECTURE and the
M15T/M15Q/M16F/M16J research. All 56 samples and 224 PHI transitions remain.
Completed-word CRC control, provisional sample checkpoint and logical CRC DATA
remain working approximations. Exact PHI pulse widths/deadtime, launch/latch and
STR/RCD/F2 propagation, analog DATA stage/polarity, physical RAM mapping and
magnetic serialization/sense remain SOURCE-BLOCKED. Delivery 14 is sequencing,
not a claim that those gaps are closed or the emulator is 14/15 electrically exact.

## Regressions

The exact former full-copy algorithm remains solely in the test module, independently
from the prepared implementation. Four mixed internal/contact patterns traverse
all 1024 opcodes under both qualifiers as consecutive streams. Every cell compares
complete controller images and identical wrong-phase/replay errors. Additional tests
cover discarded preparation, shared DATA faults at every cell including b0/b55,
final-bit IS/SYNC faults with a pending command, and successful retry without losing
state. RAM address/write/read/clear discards and DATA faults preserve the complete
storage/receiver image, then complete the normal transaction.

A native test compares 3200 real firmware words through normal/traced production
and explicit reference with key, PROGRAM/RUN and card-present changes. It compares
ACT/CRC/RAM, production endpoint/storage/display images, all named net levels,
224 trace samples and 224 PHI transitions per word. Existing full card roundtrip,
all earlier ownership faults and both 12/12 diagnostic suites stay in the full gate.

## Performance validation

Owner M16J: live 3.047 us/word (105.02x), best 3.023/worst 3.125. M16I live 2.703.
These separate runs do not isolate selector cost and are not a controlled A/B.

The new ignored release kernel rotates prepared and prior nested-copy CRC lanes
in the same process for seven rounds of 50000 words after warmup. Both traverse
the same resolved IS/SYNC source and 224-edge schedule. One black-box transaction
boundary per cell prevents elimination of the copied/proposed image; image widths
intentionally differ. Complete final images and tick counts must match. It reports
median/best/worst without correctness speed thresholds. Kernel results cannot
substitute for production live measurements or establish hardware timing.

The existing electrical, DATA, autonomous RAM and live benchmark workloads, order
and iteration sizes are unchanged. The nine-stage owner gate adds the kernel to
stage 8. Actions only runs cargo fmt/check/save. No tests/builds/benchmarks have
been executed by the agent and no performance recovery is claimed in advance.


## Owner v1 failure and v2 correction — 2026-10-30

Owner focused library gate reported four passed, one failed and one ignored. The failed final-bit fault/retry fixture fetched 0o1000 but asserted flag 0 after execution. The existing independent crc_opcode_families_cover_all_twelve_flags regression and decoder establish 0o1000 as SetFlag { flag: 4 }. This is a fixture expectation error, not authority to change CRC semantics or the production transaction implementation.

v2 explicitly checks the decode, precise b55 InvalidInstruction error, complete old/new state equality on rejection and retry, no premature frame/result/flag publication, exact sole flag 4/result after retry, the separately received next opcode and 448 PHI transitions. The production files and all existing gate stages/workloads are unchanged. No tests/builds/benchmarks were run by the agent; owner v2 acceptance is pending. M16K remains unmerged.
