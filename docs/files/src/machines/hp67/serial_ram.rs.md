# `src/machines/hp67/serial_ram.rs`

## Purpose

Provide an autonomous aggregate logical RAM participant with its own 64-register
storage and serial command/address/data state, independent of ACT state and the
architectural RAM image. This is a separate executable composition; desktop live
still uses its previous RAM authority bridge.

## Why it exists

The live DATA bridge currently starts reads from a host-supplied register value.
An autonomous RAM needs to derive commands from the resolved instruction stream,
receive address/write frames, preserve its own contents and drive read data from
those contents. This module supplies that device before live transport can send
the missing C-to-address DATA frame and replace its oracle-backed read source.

## Relationships

Uses resolved IS/SYNC/DATA and named contributions on Hp67ElectricalBackplane,
the current provisional PHI checkpoints, ActRegister only as a 56-bit container,
and the source-backed DATA bit order. It never takes an ActArchitecturalState,
ActRamImage, host opcode or read/write payload as an input. `register()` exposes
read-only inspection; ACT in the test composition receives only resolved DATA.
`mod.rs` exports the device/error/event types. Physical RAM IC mapping is absent.

## Responsibilities

Observe ten returned IS bits with a stable SYNC qualifier; execute the decoded
RAM command in the following word; receive C-to-address and write frames from
DATA; respond to direct/low-selector read families; release absent addresses;
retain cross-word tails; commit writes only after 56 valid bits; preserve state
on invalid input; expose events for subsequent integration and logical reset.

## Implementation

A fixed 64-by-14-nibble array stores logical registers. A byte address selector,
current/next command, receive cursor, partial DATA frame and read snapshot/tail
form independent device state. `begin_cycle()` requires the b0 launch boundary;
`bind_body()` requires b2 after the previous b0/b1 tail has completed. The latter
freezes the read source after a preceding write can commit. `propose_data_drive()`
is immutable and feeds the caller's joint serial-source commit. The receive
method samples existing pins at the provisional PHI1-rise checkpoint. It first
validates phase/coordinate, both serial instruction signals and DATA level/ACT
ownership, then mutates state. It never republishes pins at receive time.

Address/write frames run b2..b55 and next b0/b1. Their completion produces
AddressLatched/WriteCommitted events; reads retain bits 54/55 into the next word.
Low-selector families replace only the address low nibble. Clear uses the existing
logical 16-register block rule at completed b55, preserving other blocks; exact
physical clear behavior remains unknown. Addresses >=40 hex do not make this RAM
respond and cannot affect installed registers. Implied-GOTO payloads never decode
as RAM commands. Zeroed cold reset is the existing logical convention, not proof
of physical power-up RAM contents.

## Evidence and limits

Teenix's HP-67 captures establish instruction pipelining, C-to-address/read/write
traffic and the DATA phase. Opcode constants are independently checked against
the repository firmware. The 64-register aggregate, selector and clear semantics
retain the existing verified logical contract. The output uses M15E's explicit
logical High/Low adapter; physical polarity/bias, transistor circuitry, per-IC
partitioning, propagation and exact latch edges remain unresolved. b2 binding,
full-frame address completion and completed-word clear are named composition
checkpoints, not newly measured silicon timing. See
`docs/research/M15N_AUTONOMOUS_SERIAL_RAM_2026-10-18.md`.

## Validation and benchmark

Tests use a joint electrical launch and 224 actual PHI transitions per word.
They transmit captured firmware opcodes, exercise all 64 addresses under both
port families, assert write-to-read tail dependency, exclude peripheral ranges
and implied-GOTO commands, preserve neighbouring blocks on clear, and inject
phase/ownership/IS/SYNC faults while checking full device equality. A changed
committed ACT bit must be stored and read back even when its original payload
was zero. The ignored release benchmark measures this separate device fixture,
including ACT receive checks; it does not measure the desktop live bridge.


## M15P shared RAM command reception

The reusable ram_command receiver now observes all ten resolved IS/SYNC samples
at the existing provisional receive checkpoint. The live composition attaches
it to the DATA-capable transport on every word, including words with idle DATA,
and compares its complete raw word/qualifier against ACT's independent receiver.
The autonomous RAM uses the same receiver and promotes its decoded command into
its existing next-word pipeline. No host opcode enters either receive path.

This connects live command observation, not autonomous live storage/read drive.
Synthetic execution injection deliberately remains separate from concurrent
ROM fetch; the observer sees the actual ROM word. No comparison against the
injected executing opcode is made. DATA body/tail authority remains M15O.
Exact PHI edges/output circuitry and per-chip RAM mapping stay SOURCE-BLOCKED.
See docs/research/M15P_LIVE_RAM_COMMANDS_2026-10-20.md.
