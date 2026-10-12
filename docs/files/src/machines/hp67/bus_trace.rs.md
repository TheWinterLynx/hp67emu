# src/machines/hp67/bus_trace.rs

## Purpose
Import and compare ordinal IS/PHI observations offline.

## Why it exists
M14W/M14X can export actual structural/live traffic. Comparing those observations requires strict parsing and explicit alignment without hiding differences or inventing calibrated physical edges.

## Relationships
Uses generic Drive/LogicLevel vocabulary. Accepts native StructuralBusTrace TSV and normalized pin-only TSV; native CLI in main.rs owns filesystem access. Does not depend on firmware, GUI, hardware scheduling or wall-clock time.

## Responsibilities
Bound inputs to 64 MiB and 4096*224 rows; preserve provenance comments and unresolved levels; reject malformed header/columns/numbers/levels, out-of-range bits and non-increasing ordinal ticks. Compare selected suffixes with explicit start indices, normalized tick origins and exact relative ticks. Report unequal lengths and first sixteen differing row examples; count driver coverage and unresolved rows.

## Implementation
Five pin columns or seven native driver columns are accepted. Absolute tick origins are excluded and disclosed. Missing drivers remain None, never fabricated HighZ. Match means equality at the specified observation scope, including equal suffix lengths, not physical timing proof. No best-fit search, inversion, resampling or ignored tails. CLI report stores input paths and source comments. Regressions cover native roundtrip, explicit offset, level/phase/bit/tick/driver/length mismatch, provenance, CRLF, pin-only unknown levels and malformed input.
