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

The nominal production interval remains:
`HP67_NOMINAL_CARD_RECORD_US = 28_000`.

The documented reader-speed tolerance is now parameterized explicitly through `Hp67CardSpeed`. Production accepts 95% through 105% of nominal and defaults to 100%. The corresponding rounded display intervals are 29,474 us at 95%, 28,000 us at nominal and 26,667 us at 105%, but the moving transport does not round each record separately: it accumulates exact phase in microseconds multiplied by speed percentage against a 2,800,000-unit record threshold. This preserves fractional phase across the complete pass instead of accumulating rounding drift. No random speed is invented; callers must choose a documented in-range unit speed when they want a non-nominal reader.

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

Teenix HP-67 notes describe one magnetic track (also called a side in HP user documentation) as 34 records of 28 bits:

1. status/header;
2-33. sixteen 56-bit storage registers split into upper/lower 28-bit halves;
34. checksum.

That is 952 logical magnetic data bits per track.

Source:
https://literature.hpcalc.org/community/classic-notes.pdf

The semantic `reference::crc` and pinned Nonpareil CRC model independently agree on the same 34 × 28-bit organization.

## CRC read-buffer semantics

The production CRC architectural core now owns the hardware-documented pair of 28-bit read buffers.

A transport record arrival:

- validates the 28-bit width;
- enqueues into the next free 28-bit buffer without replacing unread data;
- raises internal `buffer_ready`;
- fails explicitly with `ReadBufferFull` if both physical buffers remain occupied when another record arrives.

Firmware `fs?c buffer_ready` clears the flag but does not itself consume the 28-bit word. The buffered word remains until the CRC data port is read.

The earlier single-latch implementation could silently replace an unread record and was therefore architecturally incorrect. The first live 34-record round-trip attempt observed only 32 firmware reads, which exposed that defect but did not by itself prove it was the sole cause. HP's November 1976 hardware description explicitly states that the CRC alternates between a pair of 28-bit buffers, so M13 models that pair rather than an overwrite latch.

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
- pin the physical time order used to serialize each 28-bit CRC record into magnetic bit cells;
- connect the documented self-clocking Zero/One flux pair to the moving transport once that bit order is evidenced;
- add lower-level head/sense-amplifier and DATA timing once sourced.


## End-to-end live regression

The live-machine regression `live_firmware_consumes_timed_crc_record_through_real_0x9b_path` boots the real firmware to idle, inserts a synthetic 34-record side, enables the explicit head gate and asserts the external card-present contact. It then executes real firmware words until the bank-1 card routine consumes a transport record through `register -> c 11` / CRC address `0x9B`. The test requires a `CrcDataRead` operation and verifies that the 28-bit record is duplicated into both seven-nibble halves of ACT C. This closes the current read-side slice from card-present through motor, timed record availability, buffer-ready and architectural CRC read.


## Write-side implementation

Teenix documents the CRC 1700 status instruction as the firmware check for whether read data is valid or a write buffer can accept data. The pinned disassembly calls this flag `crc_f7` because the original hardware name is unknown. Production therefore exposes it conservatively as `CRC_FLAG_F7_STATUS`; no stronger semantic name is claimed.

The CRC now models two 28-bit write buffers. Architectural `0x99` writes execute the ACT write-class instruction boundary, then pack only C[13:7] into a 28-bit record, exactly matching the documented firmware/data-bus layout. The transport drains those records FIFO at the same nominal 28 ms cadence used by the magnetic stream. A writable card marks the committed media dirty. A protected side leaves all records unchanged and raises the F7 status path checked by firmware.

The HP-97 service manual and Teenix reader analysis both establish that write protection is sampled when the head switch is reached; a clipped corner delays/prevents the write-protect switch condition and the CRC inhibits writing. The current transport therefore applies protection at the explicit head-active boundary rather than at UI insertion time.

## Validation status

The read-side timed checkpoint passed locally on 2026-09-19. The write-side timed checkpoint also passed locally on 2026-09-19, covering the two-buffer CRC write FIFO, `0x99` packing, nominal transport cadence, write-protect and the live firmware PROGRAM/F7 paths.

The branch now adds two stronger media-preservation regressions. The transport-level test writes all 34 records, ejects the completed side, reinserts that same media and reads all 34 records back through the CRC buffer. The live regression goes further: one real firmware instance writes the complete card in PROGRAM mode, the completed side is transferred unchanged to a second real firmware instance in RUN mode, and the 34 observed `CrcDataRead` words must exactly equal the 34 earlier `CrcDataWrite` words. The 34-record transport and live-firmware round-trip regressions passed the full local gate on 2026-09-19.


### 32/34 round-trip failure diagnosis

The first live full-card round-trip gate wrote all 34 records successfully but observed only 32 `CrcDataRead` operations on reinsertion. Firmware/disassembly confirms that a card write is exactly 34 `0x99` transfers: one header, thirty-two payload records and one checksum. The missing read records were therefore not a card-format exception. The root cause was the production CRC's former single read latch, which allowed the moving transport to replace an unread record. HP Journal documentation explicitly describes a pair of alternating 28-bit CRC buffers; the production read boundary has been corrected to the same two-buffer topology, with FIFO preservation and explicit overflow instead of silent replacement.


### ReadBufferFull follow-up and head-switch sequencing

After the dual-read-buffer correction, the next live round-trip run failed explicitly with `ReadBufferFull` at live cycle 583. This was not evidence for a larger FIFO: the live harness was still asserting `head_active` at the instant of card insertion. HP service documentation distinguishes the motor switch from the head switch: insertion first closes the motor switch, firmware starts the motor, and only when the leading card edge reaches the head does HDS close. Service Note 97-101 further states that head-switch adjustment changes the physical start-bit position. The live tests now preserve that separation. They leave the head contact open through motor startup and the two initial `buffer_ready` clears in `Scr3341`, then close it immediately before the firmware waits for the first real record. This is a test-harness synchronization point, not a claimed mechanical delay constant; exact insertion-to-head travel remains unsourced and therefore unmodeled.


### CRC/head startup-ready handshake

A second `ReadBufferFull` after separating card-present from the head contact showed that record timing was still starting at the wrong logical boundary. `Scr3341` is shared by both card reads and writes. After motor startup it clears `buffer_ready`, delays, clears it again, then calls `Scr3167` and waits for a `buffer_ready` event before any read transfer or any write-data transfer takes place. Because this same sequence runs in write mode, that first event cannot be an incoming 28-bit card record; it is a CRC/head readiness handshake. Production transport now models this explicitly. On head activation it raises one startup `buffer_ready` event without advancing `next_record`. When real firmware test-and-clears that event, the transport enters record-stream state and starts the nominal 28 ms cadence from zero. This preserves the documented continuous two-buffer stream without inventing an extra magnetic record or silently consuming the header.


## Production live-card lifecycle

The firmware-synchronized head/startup sequence proven by the tests is now owned by `Hp67LiveMachine` rather than by a test helper. `insert_magnetic_card()` inserts one complete two-track physical card plus an explicit insertion end and asserts only the physical card-present contact. While motor is running, the live machine observes the same two initial `buffer_ready` clears used to locate the pre-data head boundary; it then closes the modeled head contact and lets `Hp67CardTransport` provide the startup-ready event. Once all 34 record positions cross the head, the live machine deasserts card-present and opens the head contact; the real firmware then leaves `wait_no_card` and clears motor-on. After the selected track has crossed the head, the same physical card remains available through `take_completed_magnetic_card()` and can be rotated 180 degrees for the opposite track.

This is intentionally a media-only lifecycle. The photographed/printed Moon Rocket Lander artwork in `ui/program_card.rs` is still not associated with a magnetic payload because the repository does not contain a verified SD-14A dump. No blank or synthetic payload is silently assigned to that artwork.


## Two-track physical-card correction

The production media boundary now represents the object the user actually handles: one Hp67MagneticCard with independent logical Track 1 and Track 2, following HP's owner-manual terminology for the two end-for-end card streams. CardInsertionEnd::End1 exposes Track 1 and End2 exposes Track 2, so reading the second logical track means rotating the same card 180 degrees in its plane and inserting the opposite end. The transport no longer owns or returns an isolated Hp67CardSide.

Each Hp67MagneticTrack is either TrackMedia::Unrecorded or Recorded([u32; 34]) and carries its own write-protect state. An unrecorded track physically crosses the head without generating 34 invented zero records. In write mode, the first committed CRC word materializes recorded media. Writing one track cannot mutate the other track or its clipped/write-protected state.

Teenix .hpp is now strictly a host import format. TeenixHppImport XOR-decodes 0x55, validates NeWe, parses the calculator/bitmap/name metadata, removes the historical three dummy records, verifies and removes the duplicated checksum, reconstructs the 34 real 28-bit records and derives Track 1 versus Track 2 from the card-header ID. The transport and CRC never parse or know about .hpp.

Two native host formats are implemented at the logical post-CRC level. .hp67raw is exactly 238 bytes for two recorded 952-bit logical tracks and cannot represent unrecorded/protected state. .hp67card v1 is a 250-byte container with HP67CARD magic, version, per-track recorded/protected flags and two 119-byte logical payload slots. Neither format contains artwork, title or description. These are logical CRC-record serializations, not flux captures.

The desktop host accepts dropped .hpp, .hp67raw and .hp67card files. A double-click on the card protruding from the left exit rotates the current insertion end; a single click continues to move the same card into the passive holder above A-E.


## Physical Zero/One flux-track layer

The earlier phrase "two longitudinal magnetic tracks" was ambiguous and could be read as if logical Track 1 and Track 2 were each a single physical stripe. Hewlett-Packard's original engineering description is more specific. The HP-65 reader design reused by the HP-67/97 records one logical bit stream on **two side-by-side physical tracks**: the 1-track contains a flux reversal for every logical 1 and no reversal for 0, while the 0-track contains a flux reversal for every logical 0 and no reversal for 1. Exactly one of the two physical tracks therefore reverses in every bit cell, making the recording self-clocking and tolerant of head skew and motor-speed variation.

Primary source:
- Hewlett-Packard Journal, May 1974, "The Personal Computer: A Fully Programmable Pocket Calculator", sidebar "A Self-Clocking Two-Track Recording Technique".
- Hewlett-Packard Journal, May 1974, R. B. Taggart, "Designing a Tiny Magnetic Card Reader".
- HP-97 Service Manual, section 2-35, which states that information is recorded as flux transitions on two tracks on each usable card edge.

The new `card_flux.rs` layer therefore distinguishes:
- logical Track 1 / Track 2: the two 952-bit end-for-end streams used by firmware, files and the owner manual;
- physical Zero / One flux tracks: the parallel pair underneath whichever logical track is passing the head.

`Hp67FluxCell` represents one bit cell and makes the documented invariant explicit: exactly one physical track reverses. `Hp67SelfClockingFluxPair` round-trips an already ordered 952-bit serial stream through 952 such cells. It stores reversal events, not absolute magnetic polarity, because the sources establish transition placement but this slice has not established a required initial north/south state.

This layer is intentionally not yet wired directly from `[u32; 34]`. The sources above prove the Zero/One transition encoding, but they do not by themselves pin the temporal significance order in which the CRC shifts the 28 bits of each logical record to the sense/write circuitry. The existing .hp67raw MSB-first packing remains explicitly an emulator interchange convention and is not reused as an unsupported physical-bit-order claim.
