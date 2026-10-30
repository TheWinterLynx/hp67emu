# src/hp67/program_switch_tests.rs

## Purpose
Native acceptance of resolved selector input, firmware-owned mode changes and mechanical reset persistence.

## Why it exists
Moving the selector must not execute PROGRAM semantics, faults must not consume pending DATA tails, and reset must restore the physical position rather than a poisoned observer.

## Relationships
Uses live/reference firmware pairs, independent ACT/CRC/RAM states, the real selector API and the existing full M12/program/card regressions.

## Responsibilities
Cover pending-tail retry, every foreign source/polarity, observer substitution, fault-clearing reset with retained physical position, real firmware S11 polling and explicit released reference composition.

## Implementation
Five tests compare real firmware through RUN/PROGRAM transitions and repeated power-on resets at both selector positions, retaining 224 transitions per word. Immediate switch publication leaves ACT state unchanged; S11 is acquired by ROM firmware. Reset clears electronic status while preserving mechanical selection. Existing M12 and full 34-record tests exercise the same production selector lane.
