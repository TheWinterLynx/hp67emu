# M14M — CRC logical data-port authority

Date: 2026-10-04
Branch: `agent/m14m-crc-data-authority`
Status: implemented; owner local validation pending.

## Scope and evidence

Preserve the existing semantic CRC contract: fixed/selected 0x9B reads and 0x99 writes, two 28-bit buffers in each direction, read payload duplicated in both C halves, write payload from upper C, FIFO order and READY/F7 transitions. These are instruction-boundary behavioral rules already covered by the architectural/card regressions and M13 source notes. No new electrical hardware behavior is asserted. Official Pac examples and diagnostic media remain firmware acceptance evidence, not clock-edge evidence.

## Causal ownership

Frozen pre-instruction ACT/CRC input -> independent payload/storage image -> shared b0..b55 traversal -> exact oracle comparison -> scoped CRC storage/C commit. Oracle FIFO/C changes are restored before transport. M14K address selection and M14J boundary effects remain separately owned. Physical card transport advances after commit, as before. Only FIFO storage and READY/F7 belong to this image; external inputs and other internal flags are excluded. Staging errors leave live state unchanged. Commit rejects a changed pre-state.

## Fidelity boundary

The completed-word handoff is WORKING APPROXIMATION. CRC records are still logical 28-bit objects, not electrical DATA frames or flux bits. The installed-RAM b2..next-word b1 evidence does not establish CRC port serialization and is not reused here. Exact CRC DATA polarity/bias/drive windows, transfer phase, internal write edges, flux order and sense-amplifier/PHI timing remain SOURCE-BLOCKED. Replacing this bridge requires direct evidence and explicit serialized electrical devices.

## Validation

Differential classification checks 256 addresses times 1024 opcode values with wrapped FIFO heads and queued records, comparing operation, complete C and whole CRC state after restore/commit. Further regressions cover empty/inactive/full staging, unchanged state on failure, stale and early commit rejection, external/unrelated latch preservation, implied-GOTO exclusion, failed-bind recovery and b55 completion. A live cycle regression exercises selected and fixed reads/writes, both buffers and M14K address separation. Owner local full gate including diagnostic 12/12 remains required before merge. GitHub Actions executes cargo fmt/check/save only.
