# src/hp67/address_data_tests.rs

## Purpose

Regress live C-to-address DATA transport through real resolved nets.

## Why it exists

An autonomous RAM receiver needs an actual address frame; an ACT scalar latch
alone cannot address an independent device.

## Relationships

Uses Hp67LiveMachine's production traced/untraced cycle, structural snapshot,
clocked DATA phase, five-ROM transport and existing reference RAM comparisons.

## Responsibilities

Cover frozen C, address/write/read dependencies, all clock counts, DATA owner
identity, peripheral addressing without RAM writes, implied-GOTO exclusion,
tail contention, reset and real-firmware trace parity.

## Implementation

Short synthetic command sequences inject executing words through the existing
reference fixture and preserve concurrent ROM fetch. Their DATA transport is
the production path. A separate 1200-word real-firmware boot comparison requires
actual address frames, idle reachability and 224 transitions per word. The
contended-tail case checks pending/state preservation before the next command.
These are digital logical-level tests, not evidence for exact physical edges.


## M15Q autonomous live RAM

Production construction now installs independent Hp67SerialRam storage. The
shared DATA transports and clocked prefix begin its cycle at b0, bind its body
at b2, publish its own read proposal jointly with IS/SYNC/ACT DATA, and receive
actual pins at the existing provisional checkpoint. The read payload passed to
the phase path is a zero frame mask; oracle RAM bits are never published.

Shared receive first prepares the RAM input image, validates/consumes DATA, then
commits the prepared RAM sample. No full RAM clone or allocation occurs per bit.
The old public transport APIs wrap the same loops with no RAM for reference
fixtures. Production live uses Some(RAM), reset preserves that composition, and
the only no-RAM live constructor is test-only. Host-injected opcode/RAM fixtures
explicitly use that constructor because their execution differs from real IS.
Actual firmware boot/key/program/card/diagnostic paths retain autonomous RAM.

Live checks command reception, full address completion, net write against device
storage, selector and block clear against the oracle. Reference RAM remains an
instruction-boundary comparison mirror; ACT read commits still come from resolved
DATA after next b1. Physical per-IC mapping/output stage, precise internal latch
edges and PHI propagation remain unproven. See
`docs/research/M15Q_LIVE_AUTONOMOUS_RAM_2026-10-20.md`.
