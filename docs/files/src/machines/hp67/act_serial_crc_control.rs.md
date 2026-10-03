# `src/machines/hp67/act_serial_crc_control.rs`

## Purpose

Provides completed-word authority for CRC set/test-and-clear flags and the resulting ACT S3 latch.

## Why it exists

The architectural composition still changed CRC control flags and ACT S3 causally. This image derives the same final logical effects independently from frozen pre-instruction ACT/CRC inputs.

## Relationships

Uses the established CRC opcode classifier and immutable ACT snapshot. The endpoint binds CRC inputs once before execution and completes the image only after b55. The live bridge restores oracle flag/S3 effects, then compares and commits the structural result. CRC buffers, external inputs, DATA ports and card transport retain their existing owners.

## Responsibilities

Set the selected internal flag, or test the OR of internal/external input and clear only the internal flag. A true test sets S3; a false test preserves existing S3. Reject implied-GOTO payload decoding, rebinding and early commit. Never modify other flags, status bits, external inputs or buffers.

## Implementation

The image records one pre-instruction internal flag, external level and S3. Its derived condition/result remains immutable even if host/peripheral state subsequently changes. `CrcArchitecturalCore::commit_control_flag` is a validated scalar storage commit with no opcode semantics. Restore/commit change only the selected internal flag and S3. Completion after b55 is WORKING APPROXIMATION authority, not an F2 or CRC internal write-edge claim. Those electrical edges and propagation remain SOURCE-BLOCKED.
