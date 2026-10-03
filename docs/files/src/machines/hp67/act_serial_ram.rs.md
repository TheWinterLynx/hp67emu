# `src/machines/hp67/act_serial_ram.rs`

## Purpose

Provides completed-word authority for the remaining RAM low-nibble selector families and logical installed-RAM block clear.

## Why it exists

M14H owns C-derived RAM selection and M14C owns reconstructed DATA destinations, but family 050/070 address selection and 1260 block clear still came from the architectural executor.

## Relationships

`fetch.rs` binds the image from immutable ACT pre-state and completes it only at b55. The live bridge records pre-block topology/data, restores the oracle mutation before transport and compares final address/block against the independently derived result. M14C retains DATA transfer ownership, including the following-word b0/b1 tail. M14H retains 1160 C-derived selection. CRC ports retain their peripheral behavior while the ACT address selector becomes structural.

## Responsibilities

Preserve the high address nibble and select the opcode's low nibble for family 050/070, excluding fixed 0070. Clear only installed registers in the aligned sixteen-address logical block for 1260; preserve uninstalled slots and all neighbouring blocks. Reject commit before complete b0..b55 execution. Never interpret implied-GOTO payload as a RAM command.

## Implementation

`ActSerialRamResultImage` stores a selector result or aligned clear base derived from pre-instruction state. Block snapshots preserve Option-valued installed topology. The live machine restores the oracle's old address/block, then commits a completed matching image. Clear zeros are independently generated without calling the architectural clear-block implementation. This is WORKING APPROXIMATION final logical authority, not a claimed physical 16-register DATA sequence. Chip partitioning, physical clear timing, DATA drive/sample edges and propagation remain SOURCE-BLOCKED.

## M14N complete ACT authority coverage

The flow image now restores the oracle's implied-GOTO completion latch as well as PC/bank/stack state. `ActSerialAbsentRamReadImage` isolates the existing absent-slot zero-read compatibility rule: restore oracle C before transport, compare zero result and commit only after b55. Binding rejects installed RAM and CRC read ports, repeated/late topology binding, incomplete commits and changed topology/pre-state. It does not create a responder or DATA frame for missing RAM; physical absent-address behavior remains SOURCE-BLOCKED.

The live bridge verifies the entire pre-instruction ACT contract before transport, using its immutable snapshot, prior previous-carry and flow state. Any leaked oracle mutation is a hard contextual error. Whole ACT/RAM/CRC differential coverage spans all 1024 words, six address contexts, both carry states and normal/implied-GOTO state. Installed RAM results are compared only after the documented next-word b0/b1 tail. Unknown specials and ROM self-test remain rejected. This closes semantic ownership gaps without proving internal latch edges or removing the oracle.
