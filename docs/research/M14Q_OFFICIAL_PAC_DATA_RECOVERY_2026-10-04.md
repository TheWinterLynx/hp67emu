# M14Q — official Moving Average saved-data recovery

Date: 2026-10-04
Branch: `agent/m14q-official-pac-data-recovery`
Status: implemented; owner local gate pending.

## Behavioral source

The supplied HP-97 Standard Pac manual, PDF page 16 / printed 01-03, was visually inspected. Its Moving Average example saves the first six inputs with B and a blank magnetic card, then turns off, reloads both program sides after restart, reads the saved data and enters 225. Expected final mean: 212.50. Printed 01-02 says windows of ten or more need two data-card passes; the six-point fixture requires one. This is Tier B functional corroboration tested against the existing HP-67 SD1-01A corpus, not electrical equivalence. Printer output and transient counters are excluded. No attached source bytes are added to the repository.

## Change and causality

The existing M14P fixture and physical-key/checkpoint runner are reused. M14Q executes its first eight checkpoints through the live machine, then holds/releases physical B through firmware dispatch. It requires firmware Crd/write mode, inserts blank End1 and waits for transport completion. The returned media must contain data header 1, a dirty first track, an unrecorded opposite track and no pending CRC write words after firmware settles.

Only the returned physical card's lossless native bytes cross the boundary. The writer is dropped. A separately constructed and firmware-booted reader loads both original program sides, then the saved data card through transport. It queries D for 195.83, enters 225/A for 212.50, and queries D again for 212.50. No RAM/register/display/control snapshot is copied or injected. Native container byte equality verifies host persistence only; restored calculator state is proved by the independent firmware results.

A new boot isolates electronic state; it does not validate the OFF switch, analog reset waveform or real power decay. Transport remains the existing logical CRC/card model; no flux serialization or sense-amplifier claim is added. Existing one-million-word result timeout and bounded card pass/prompt waits fail with useful checkpoint or transport context. Production machine code and semantics are unchanged.

## Validation

Actions runs cargo fmt/check/save exclusively. Agent reviewed the source/diff and companion documentation contract without running Rust. Owner gate: formatting, focused official Pac tests, warnings-denied all targets, release binaries, Custom Diagnostic Pac 12/12, M14P 9/9 and explicit ignored M14Q saved-data recovery. Successful execution is pending.

## Remaining priorities

Cover the official two-pass data threshold with a separately sourced example and expand to other Standard/Games Pac programs. Keep exact DATA polarity/ownership, physical RAM mapping, PHI/STR/RCD edges and magnetic sense/serialization source-blocked until hardware evidence is available.
