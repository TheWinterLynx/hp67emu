# M15A SYNC evidence review — 2026-10-13

## Source and method

Direct HP-67 evidence: https://literature.hpcalc.org/community/classic-notes.pdf,
Teenix 2025, printed page 70 (PDF page 71). Reviewed the full page rendered
from the retrieved PDF; SHA-256:
`bdc7a403e22b03f4305d2d7b638ff4358e49e2d5ae5328f590f6946f243c66da`.
The first two expanded captures are explicitly labelled rising/falling SYNC.

## Finding

The rising SYNC transition appears slightly before PHI2 rising; the falling
SYNC transition appears after PHI2 rising. The instruction-bit-0/bit-9
captures also show staggered transitions. The original M14A claim that both
SYNC transitions share an exact PHI2-rising launch edge is not justified.
Channel skew, threshold crossings and propagation are not quantified by this
raster. Do not derive a nanosecond delay, latch sample edge or causal internal
clock edge from these relative positions.

## Implementation

V2 withdraws that exact anchor: `HP67_SYNC_TRANSITION_EDGE` is `None`, and
SYNC launch/sample/propagation remain SOURCE-BLOCKED. The regression checks
all eight contracts, including SYNC, for absent edges and propagation bounds.
V2 replaces V1 in the public exports so consumers must use the corrected
contract. The PHI topology, 56-bit coordinates, IS windows and live firmware
execution are unchanged. No new electrical SYNC drive is introduced.

## Next evidence required

Recover original acquisition samples with channel timing/threshold metadata,
or independently measure SYNC/PHI on an HP-67. Separate rising/falling
transition contracts and receiver sample contracts when the evidence supports
them. DATA polarity/ownership, exact IS/DATA edges, STR/RCD and magnetic
serialization remain blocked. Application Pac manuals do not close these gaps.

## Validation boundary

Rust tooling is unavailable in the agent environment. Owner formatting,
focused/full tests and release gates are required before integration.
