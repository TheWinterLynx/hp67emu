# M14L — CRC control flag and ACT S3 authority

Date: 2026-10-04
Branch: `agent/m14l-crc-status-authority`
Status: owner reported M14L FULL GATE GREEN; validated head `4be06a712e88940fb3cec41e3eec81d60c957d6b` integrated into main.

## Scope

Move CRC SetFlag/TestFlagAndClear final control-latch authority and ACT S3 result onto the shared completed b0..b55 lifetime. Control test samples internal OR external flag; clear removes only the internal latch. A true result sets S3; a false result does not clear an already set S3.

## Causal path

Immutable pre-instruction ACT S3 and CRC internal/external flag -> independent image -> shared structural word -> exact oracle comparison -> selected CRC latch/S3 commit. The live bridge restores the oracle's flag/S3 changes before transport. Each word resets the CRC binding; repeated binding and binding after completion are rejected. Implied-GOTO payload never becomes a CRC command.

## Boundaries

This is semantic final-state authority. No PHI-relative F2 pulse, internal CRC latch edge or propagation delay is inferred. Such electrical behavior remains SOURCE-BLOCKED. CRC read/write buffers, DATA-port payloads, head/sense electronics and physical transport remain outside M14L. Existing control decoder evidence and regressions establish instruction meaning, not pin timing.

## Validation

All ten-bit words that decode as valid CRC controls are compared to the composed oracle over internal/external flag and prior-S3 boolean combinations. Exact whole-CRC equality guards unrelated flags, external inputs and buffers. Other regressions lock false-test S3 preservation, external input surviving clear, implied-GOTO exclusion, immutable input binding, b55 completion and a live CRC-to-S3 cycle. Owner formatting/warnings/all-targets/release plus diagnostic 12/12 is required before merge. Actions runs cargo fmt only.
