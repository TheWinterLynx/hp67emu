# M14K — RAM selector and clear-block authority

Date: 2026-10-04
Branch: `agent/m14k-ram-control-authority`
Status: owner reported `M14K FULL GATE GREEN` on 2026-10-04; integrated into main under standing green-gate authorization.

## Scope

Migrate the low-nibble address selection in RAM family 050/070, excluding fixed 0070, and the logical installed-RAM block clear 1260. C-derived 1160 remains M14H-owned. Ordinary DATA read/write destinations remain M14C-owned; the new selector also covers ACT selection when the target is a CRC DATA port, without owning the CRC payload/electronics.

## Causal path

Real immutable pre-instruction ACT address -> independent selector/clear image -> full b0..b55 execution -> exact oracle comparison -> address/block commit. For DATA instructions the selected target is already encoded in the pre-instruction transfer plan; resetting the oracle-selected address does not change that plan or its cross-word lifetime. The selected address commits after b55; C/RAM DATA destination still waits for the next-word b0/b1 frame tail.

For clear, the bridge captures the aligned sixteen slots including absent slots. Oracle clear effects are restored before transport. The expected block must match independent zeros only for slots present in that captured topology. Neighbouring registers and absent slots are preserved.

## Evidence boundary

Opcode meanings and logical address/block semantics are the existing verified ACT contract. Completed-word authority is a WORKING APPROXIMATION. This does not establish physical chip partitioning, an inferred sixteen-transfer bus protocol, clear-loop latency, DATA polarity/bias, launch/sample edges or propagation. Those remain SOURCE-BLOCKED; no invented electrical clear behavior is introduced.

## Validation

Regression tests compare all 256 starting addresses and every low selector for both read/write families against composed oracle behavior, including peripheral-address cases that the oracle accepts. Clear is compared across every starting address using sparse installed topology and full-RAM equality. Other tests lock early-commit rejection, implied-GOTO exclusion, separation from M14H, b55 completion, live neighbouring-block preservation and live selected-read cross-word C authority. Owner full formatting/warnings/all-targets/release and diagnostic 12/12 are required before merge. Actions executes cargo fmt only.
