# M14T — completed-scan nominal display dwell

## Evidence

Direct HP-67 captures: https://literature.hpcalc.org/community/classic-notes.pdf, printed pages 75-77. Observed approximately 320 us/word, 4800 us per fifteen STR slots, 40 us normal segment, 30 us decimal, 5 us gap and 5 us STR pulse. These constants already existed in timing.rs; M14T consumes them rather than introducing new timing claims.

## Implemented scope

Every live structural word supplies decoded ROM0 output, following cathode phase validation, to a deterministic fifteen-slot dwell accumulator. Completed scans publish atomically. Disabled display words supply blank output. Power reset clears both pending and published histories. Normal words and fused RAM DATA words pass the same live capture boundary. Existing 56-bit/224-transition transport and ACT authority remain unchanged.

Classification: WORKING APPROXIMATION. Approximate observed dwell is assigned per decoded segment, not generated from a physical current waveform. No inferred segment ordering, PHI edge, propagation, DATA polarity or slot-15 optical discard rule. UI masks stay on their existing path until optical mapping is justified. Shared signs and duplicate slot remain raw scan identities.

## Validation

Focused m14t library and binary tests; all-targets warnings gate; release binaries; existing diagnostic suite; electrical, DATA and live-authority benchmarks to quantify added hot-path cost. Rust formatting only runs on Actions. Owner must run all execution gates locally. No additional Pac acceptance tests.

## Next slices

1. Inspect direct HP-67 segment waveforms and transistor routing before turning scan dwell into optical integration. Preserve the approximate/current distinction.
2. Recover DATA drive/bias and PHI-relative launch/sample evidence before electrical RAM devices; b2 stream phase is already implemented.
3. Cross-check shared CRC/sense circuitry against HP-67 schematics before electrical card serialization. Host packing is not magnetic bit order.

Each slice starts from owner-validated merged main. No source-blocked behavior is filled with plausible values.
