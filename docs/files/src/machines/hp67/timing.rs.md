# `src/machines/hp67/timing.rs`

## Purpose
Defines the HP-67 serial machine-word coordinate system, evidenced ROM0/IS windows, and coarse whole-machine timing observations that are directly visible in HP-67 logic-analyser captures.

## Why it exists
The Woodstock datapath uses a 56-bit word made from fourteen 4-bit digit times. Tony Nixon's HP-67 captures fix display data at `b0..b7`, ROM address at `b16..b27`, ROM result at `b46..b55`, and also report coarse physical durations: about 320 us per 56-bit HP-67 word, about 4.8 ms for fifteen STR display slots, signals beginning to stabilize around 330 us after switch-on, and valid SYNC activity around 35 ms after switch-on.

## Relationships
Used by the electrical backplane/fetch/display modules for serial coordinates and by `src/hp67.rs` only for coarse wall-clock pacing of the visible startup sequence. `docs/research/HP67_ISA_TIMING.md` and `docs/HARDWARE_SOURCES.md` record the evidence boundaries.

## Responsibilities
Define 4 bits per digit, 14 digits per word and 56 bits per word; encode ROM0 display, address, ROM-return and SYNC windows; expose the coarse STR coordinate; and publish the observed HP-67 word/refresh/power-on durations without converting them into unsupported PHI edge claims.

## Implementation
`Hp67WordTiming` still advances through the temporary four-subphase PHI scaffold. The `HP67_OBSERVED_*_US` constants are separate measured whole-interval observations. They do not determine exact PHI1/PHI2 pulse widths, dead time, launch edges, sample edges, RCD placement or propagation delay; those remain intentionally open.
