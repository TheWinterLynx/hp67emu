# M14N — ACT authority coverage and remaining semantic gaps

Date: 2026-10-04
Branch: `agent/m14n-authority-coverage`
Status: owner reported M14N FULL GATE GREEN; validated head `ea53a8f9260fd89b5701c25738c8d25f644f9001` integrated into main.

## Findings

After M14M, most instruction effects had independent result owners, but two effects still leaked from the oracle: implied-GOTO completion changed the instruction-state latch before b0, and reads of missing RAM slots wrote compatibility zeros directly into C. Neither gap justified a physical timing claim.

## Changes

M14I flow restore now also reinstates ThenGoto only when this image owns implied-GOTO completion. Ordinary flow restore does not overwrite M14F/J condition-latch state. The completed image retains final Normal authority.

`ActSerialAbsentRamReadImage` binds the selected address and previous C before b0. It excludes installed RAM and CRC reads, restores oracle C, and commits the existing zero result only after completed b55 plus exact oracle comparison. Early/stale commits and changed installed topology fail. Writes to absent slots retain the existing no-storage mutation rule. No absent device, DATA driver or 56-bit transfer is invented.

The live adapter now checks every ACT semantic field before transport: immutable snapshot fields, previous-carry and flow fields must equal their pre-instruction values. A mismatch fails with cycle/word/PC and state context. This protects structural transport from future unowned oracle side effects; it does not replace final result comparisons.

## Ownership map

| Effect | Authority | Handoff |
|---|---|---|
| A/B/C arithmetic and carry | M14E | Complete word |
| P/status/control conditions | M14F | Complete word |
| Special registers and C constant | M14G | Complete word |
| Decimal/display/RAM C-derived latch | M14H | Complete word |
| PC/bank/delayed ROM/stack and implied-GOTO completion | M14I plus M14N restore correction | Fetch preview / complete word |
| Prelude and arithmetic instruction-state condition | M14J | Complete word |
| Low RAM selector and installed block clear | M14K | Complete word |
| CRC control flags and ACT S3 | M14L | Complete word |
| CRC logical FIFO/payload/READY/F7 | M14M | Complete word |
| Installed RAM C/storage DATA destination | M14C | Next-word b0/b1 logical tail |
| Absent-RAM compatibility C zeros | M14N | Complete word approximation |
| Key contact/key-code/S15 sampling | Existing keyboard bridge | Instruction input approximation |

## Evidence boundary

The zero-read rule is a preserved architectural compatibility behavior, not proven floating-DATA or passive-bias behavior. Its isolation makes the approximation explicit and replaceable when direct evidence establishes absent-address response. All completed-word commits remain WORKING APPROXIMATION for internal visibility. Electrical PHI widths/edges, ACT intra-word writes, DATA polarity/ownership, physical RAM partitioning, CRC serialization/sense, STR/RCD propagation and keyboard pin scanning remain pending or SOURCE-BLOCKED. Unknown opcodes and ROM self-test are still errors. The architectural executor remains the differential oracle, including its diagnostic executed-word counter. No claim of oracle-free execution is made.

## Validation

A whole-state live regression enumerates 1024 words over six address contexts (installed, absent, CRC write/read, high-bank absent and maximum address), both carries and normal/implied-GOTO state: 24,576 attempted cases. Accepted architectural operations must match execution metadata and complete ACT/RAM/CRC state. Installed RAM comparison consumes its pending logical tail first. Rejected architectural words remain rejected rather than assigned invented semantics. Focused tests cover implied-GOTO restoration, absent-read exclusions/early commit/topology change, endpoint binding/lifetime and injected unowned mutations rejected by the guard. Owner local full gate and diagnostic 12/12 are required before integration. Actions runs cargo fmt/check/save only.

## Next priorities

1. Measure the established structural/real-firmware benchmarks after full authority coverage; investigate material regressions without dropping transitions or checks.
2. Audit remaining device-input and transport approximations against direct hardware evidence: keyboard KC/contact path, STR/RCD and CRC/DATA. Implement only supported facts.
3. Broaden official Pac behavioral acceptance through real firmware and acquire external golden hardware traces.
