# `src/machines/hp67/timing.rs`

## Purpose
Defines the HP-67 serial machine-word coordinate system, evidenced ROM0/IS windows, the measured DATA-stream phase, and coarse whole-machine/display timing observations that are directly visible in HP-67 logic-analyser captures.

## Why it exists
The Woodstock datapath uses a 56-bit word made from fourteen 4-bit digit times. Tony Nixon's HP-67 captures fix display data at `b0..b7`, ROM address at `b16..b27`, ROM result at `b46..b55`, and DATA serial bit 0 at machine-word `b2` with bits 54/55 wrapping into the next word's `b0`/`b1`. The same captures report about 320 us per 56-bit word, about 4.8 ms for fifteen STR slots, about 40 us normal-segment on-time, about 30 us DP on-time, about 5 us DP-to-STR gap, about 5 us STR pulse width, signals beginning to stabilize around 330 us after switch-on, and valid SYNC activity around 35 ms after switch-on.

## Relationships
Used by the electrical backplane/fetch/display modules for serial coordinates and by `src/hp67.rs` only for coarse wall-clock pacing of the visible startup sequence. `docs/research/HP67_ISA_TIMING.md`, `docs/research/M14A_TIMING_EVIDENCE_LOCK_2026-09-21.md`, and `docs/HARDWARE_SOURCES.md` record the evidence boundaries.

## Responsibilities
Define 4 bits per digit, 14 digits per word and 56 bits per word; encode ROM0 display, address, ROM-return and SYNC windows; encode the source-backed circular DATA phase (`b2 -> serial bit 0`); expose the STR bit coordinate; and publish observed HP-67 word/refresh/display/power-on durations without converting them into unsupported PHI edge claims.

## Implementation
`Hp67WordTiming` still advances through the temporary four-subphase PHI scaffold. The `HP67_OBSERVED_*_US` constants are measured scope observations rather than oscillator design constants. `data_serial_bit_for_word_bit()` captures only the directly observed DATA phase relationship. `Hp67ClockEdge` names PHI transitions and `Hp67ClockPhase::advance()` defines the exact topological edge sequence without assigning durations. `Hp67WordTiming` tracks completed subphases only; the electrical backplane owns the current pin phase. `HP67_SYNC_TRANSITION_EDGE` locks the direct page-70 observation that SYNC changes on PHI2 rising. Exact PHI1/PHI2 pulse widths, dead time, IS/DATA launch/sample edges and per-device propagation remain intentionally open; STR-on-bit-7 and low-going STR/RCD behavior are source-backed at the bit/event level.


M14A also defines versioned edge-contract table V1. It records ownership/source metadata for SYNC, IS address/instruction traffic, DATA in both directions, ROM0 display traffic, STR and RCD. Unknown launch/sample edges and propagation are deliberately represented as source-blocked values and protected by regression tests rather than inferred from the temporary scheduler.
