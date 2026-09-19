# M13 magnetic-card transport/read boundary — 2026-09-19

## Scope

This slice moves M13 beyond presentation and control into the first timed magnetic-data boundary:

- real firmware already detects external `card_present`;
- real firmware already asserts CRC `motor_on`;
- a separate card transport now advances at the documented nominal record cadence;
- the transport presents 28-bit records to the CRC, which raises `buffer_ready`;
- architectural CRC read port `0x9B` now transfers one buffered 28-bit record into ACT C;
- CRC write port `0x99` now packs ACT C[13:7] into the CRC two-buffer write FIFO.

No UI click is yet authoritative for magnetic data and no card-file persistence is connected.

## Primary timing evidence

The November 1976 Hewlett-Packard Journal article on the HP-67/97 states that:

- the card passes through the reader at a nominal 6 cm/s;
- recorded density is slightly above 160 bits/cm;
- bit rate is approximately 1 kHz;
- the CRC buffers the incoming stream so the microprocessor handles one 28-bit record every approximately 28 ms on average;
- reader speed can vary ±5% from nominal;
- firmware was designed so each record can be processed in less than approximately 84 instruction times.

Source: HP Journal, November 1976, HP-67/97 design article:
https://www.worldradiohistory.com/Archive-Company-Publications/HP-Journal/70s/HPJ-1976-11.pdf

The production constant for this slice is therefore the nominal record interval only:
`HP67_NOMINAL_CARD_RECORD_US = 28_000`.

The ±5% reader-speed tolerance is documented but is not yet randomized or parameterized.

## Firmware evidence

Pinned Nonpareil disassembly:
`brouhaha/nonpareil@c347bc1ab20170c253512042f7aac0d952f304ea`
`ncd/67-97/6797cr.asm`.

Relevant control flow:

- `Scr3341` turns the display off, sets `motor_on`, clears `buffer_ready`, delays and waits for later `buffer_ready` assertions;
- `Scr3167` / `Scr3201` poll `buffer_ready`;
- `sel_crc` selects CRC data block `0x99`;
- `register -> c 11` selects/read-address `0x9B`;
- `crc_read_56` reads two 28-bit halves to reconstruct one 56-bit register;
- `wait_no_card` eventually stops the motor and clears `buffer_ready`.

This proves that motor control and data-ready are distinct firmware-visible events.

## Head-switch boundary

The HP-97 service manual describes the shared reader sequence: card insertion closes the motor switch; firmware commands the CRC to enable the motor; only when the leading edge reaches the head does the head switch close, after which flux transitions are sensed and converted to digital levels.

Because exact HP-67 insertion-to-head distance/acceleration is not yet pinned, the production transport exposes `head_active` explicitly rather than inventing a delay.

Source:
https://literature.hpcalc.org/community/hp97-sm-en.pdf

## Card-side format

Teenix HP-67 notes describe one card side as 34 records of 28 bits:

1. status/header;
2-33. sixteen 56-bit storage registers split into upper/lower 28-bit halves;
34. checksum.

That is 952 magnetic data bits per side.

Source:
https://literature.hpcalc.org/community/classic-notes.pdf

The semantic `reference::crc` and pinned Nonpareil CRC model independently agree on the same 34 × 28-bit organization.

## CRC read-buffer semantics

The production CRC architectural core now owns a one-record read buffer.

A transport record arrival:

- validates the 28-bit width;
- replaces the current one-record input buffer if firmware did not consume it before the next record boundary;
- raises internal `buffer_ready`.

Firmware `fs?c buffer_ready` clears the flag but does not itself consume the 28-bit word. The buffered word remains until the CRC data port is read.

Exact hardware overrun/error signaling for an unread record being replaced is not yet established and is therefore not invented.

## Architectural read port

The read-side CRC data port at `0x9B` is now handled by `Hp67ArchitecturalMachine`.

At an instruction boundary:

1. the ACT performs the real RAM-read-class instruction bookkeeping;
2. the CRC buffered 28-bit word is consumed;
3. its seven nibbles are copied into C[0..6] and duplicated into C[7..13], matching the documented/reference CRC semantic boundary;
4. the execution is reported as `Hp67ArchitecturalOperation::CrcDataRead`.

This is explicitly an instruction-boundary bridge. DATA-bus electrical timing and sense-amplifier pulses remain unimplemented.

## Remaining boundary

The next M13 slices are:

- connect a known card-side image to the live transport;
- source-back the insertion/head-switch geometry and first-record phase;
- prove a real firmware header read end-to-end;
- validate CRC write port `0x99`, dual-buffer draining and write-protect through the full local gate;
- connect UI/card-file lifecycle to the physical transport state;
- add speed-tolerance and lower-level sense/DATA timing once sourced.


## End-to-end live regression

The live-machine regression `live_firmware_consumes_timed_crc_record_through_real_0x9b_path` boots the real firmware to idle, inserts a synthetic 34-record side, enables the explicit head gate and asserts the external card-present contact. It then executes real firmware words until the bank-1 card routine consumes a transport record through `register -> c 11` / CRC address `0x9B`. The test requires a `CrcDataRead` operation and verifies that the 28-bit record is duplicated into both seven-nibble halves of ACT C. This closes the current read-side slice from card-present through motor, timed record availability, buffer-ready and architectural CRC read.


## Write-side implementation

Teenix documents the CRC 1700 status instruction as the firmware check for whether read data is valid or a write buffer can accept data. The pinned disassembly calls this flag `crc_f7` because the original hardware name is unknown. Production therefore exposes it conservatively as `CRC_FLAG_F7_STATUS`; no stronger semantic name is claimed.

The CRC now models two 28-bit write buffers. Architectural `0x99` writes execute the ACT write-class instruction boundary, then pack only C[13:7] into a 28-bit record, exactly matching the documented firmware/data-bus layout. The transport drains those records FIFO at the same nominal 28 ms cadence used by the magnetic stream. A writable card marks the committed media dirty. A protected side leaves all records unchanged and raises the F7 status path checked by firmware.

The HP-97 service manual and Teenix reader analysis both establish that write protection is sampled when the head switch is reached; a clipped corner delays/prevents the write-protect switch condition and the CRC inhibits writing. The current transport therefore applies protection at the explicit head-active boundary rather than at UI insertion time.

## Validation status

The read-side timed checkpoint passed locally on 2026-09-19. The new write-side code and live firmware regressions are implemented in the branch and await the next local focused/full gate.
