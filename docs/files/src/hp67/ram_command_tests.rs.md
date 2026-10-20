# src/hp67/ram_command_tests.rs

## Purpose

Regress independent RAM control observation in the desktop live cycle.

## Why it exists

A future live RAM device must receive the same real wire command as ACT without
using the currently executing oracle/injected opcode as an input.

## Relationships

Uses actual live firmware transport, optional trace recording, ACT receive
inspection and the attached shared RAM command receiver.

## Responsibilities

Compare raw word/qualifier across independent receivers and both transports;
require normal RAM commands and implied-GOTO exclusion; preserve 224 transitions,
trace length, ACT/RAM results, boot idle and reset clearing.

## Implementation

A 1200-word real-firmware comparison checks each received frame and full state.
A separate short synthetic execution fixture deliberately injects 270 while
ROM fetch 001 returns 3e3. The RAM observer must report 3e3, proving that the
injected opcode cannot feed it. This is control observation only, not autonomous
RAM storage integration or a physical latch-edge claim.


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
