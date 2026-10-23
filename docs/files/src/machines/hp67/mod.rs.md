# `src/machines/hp67/mod.rs`

## Purpose
Defines the HP-67-specific machine module boundary.

## Why it exists
The HP-67 combines a particular ACT, ROM/RAM set, display drivers, serial-word timing, IS/ISA bus behavior and card-reader electronics. Those choices must be explicit and isolated from generic simulation infrastructure.

## Relationships
Exports `wiring` hardware vocabulary, `timing` serial-word coordinates, `isa` ROM-fetch serialization helpers, `fetch` ACT<->ROM serial transport endpoints, `display` for the structural ROM0/anode and cathode scan path, `electrical` for the fixed-topology production fabric and staged scheduler primitive, `act` for the independent complete instruction-boundary ACT bring-up core, `act_serial_execution` for the explicit b0..b55 execution lifetime and arithmetic routing, `act_serial_state` for immutable pre-instruction execution snapshots, `act_serial_result` for the complete arithmetic-family result image, `act_serial_control` for the M14F P/status completed-word result image, `act_serial_special_register` for the M14G completed-word non-arithmetic register-transfer image, `act_serial_mode_latch` for the M14H completed-word mode/latch image, `crc` for the independent CRC control/read-buffer interface, `data` for the source-backed cross-word logical DATA phase engine, `magnetic_card` for whole-card logical Track 1 / Track 2 media and host interchange formats, `card_flux` for the documented physical self-clocking Zero/One flux-track pair underneath one logical track, `card_transport` for the speed-aware selected-track cadence and head gate, `architectural` for the composed ACT+RAM+CRC bring-up machine, and the initial electrical `machine` backplane shell. Later device work will replace architectural/structural scaffolds with fully timed chips while retaining the same machine boundary.

## Responsibilities
Provide one coherent HP-67 namespace, re-export high-value public types/constants, and prevent GUI details or prematurely generalized Woodstock assumptions from entering machine code.

## Implementation
Declares `act`, `act_serial_control`, `act_serial_execution`, `act_serial_mode_latch`, `act_serial_result`, `act_serial_special_register`, `act_serial_state`, `architectural`, `card_flux`, `card_transport`, `crc`, `data`, `display`, `electrical`, `fetch`, `isa`, `magnetic_card`, `machine`, `timing` and `wiring`; re-exports the HP-67 electrical backplane, structural serial fetch runner/endpoints/pipeline latch, complete ACT architectural state/core/RAM scaffold, explicit intra-word ACT execution lifetime/arithmetic routing/pre-instruction state snapshot, `ActSerialArithmeticResultImage`, `ActSerialControlResultImage`, `ActSerialSpecialRegisterResultImage` and `ActSerialModeLatchResultImage`, CRC control decoder/state plus named PROGRAM/buffer/motor/card-present/write flag identities, whole-card logical media/import types, self-clocking physical flux primitives, documented 95-105% transport-speed types/constants, ACT+CRC architectural composition, ROM0 display-byte receiver/decoder, explicit `Rom0StrEvent`, downstream `CathodeScanError`/`1820-1749` cathode state, temporary `PowerOnAct*` compatibility aliases, 4/14/56 geometry, evidenced `b0..b7` display window, `b16..b27` address and `b46..b55` ROM-word windows, the source-backed DATA phase (`b2` = serial bit 0 with 54/55 wrapping through next-word `b0`/`b1`), named HP-67 PHI phases/edges plus the explicitly unavailable exact SYNC edge, the versioned edge-contract table V2 and its source-blocked entries, measured display timing constants, IS wired-high serializers, dense electrical fabric/snapshot/error types, chip/net/driver vocabulary and `CHIPSET`.


The electrical exports include the opaque `Hp67DriverOwner` token used to bind one physical output identity during machine composition without paying a duplicate-ownership check on every scheduler edge.


M14B additionally exports `Hp67DataSerialSource`, `Hp67DataSerialSink`, transfer-plan/direction types and the register-bit helper. These expose only the source-backed logical DATA phase; they deliberately do not imply DATA pin polarity or PHI-relative edge timing.


The M14B exports now include `Hp67DataSerialWordPath`, `run_structural_display_fetch_data_phase_cycle()` and `StructuralDataWordResult` for the fused logical-phase experiment. These names do not promote DATA to an electrical claim.

## M14I flow authority

The ACT endpoint captures `ActSerialFlowResultImage` from real pre-instruction state. The live bridge restores PC/bank/delayed-ROM/return-stack changes made by the oracle. Concurrent successor fetch consumes the independently derived structural preview; completed b55 flow must agree with the oracle before live commit. Low-page bank normalization remains a fetch-boundary rule. Early preview and final commit do not claim physical internal write edges. Existing M14C-H result ownership is unchanged.

## M14J boundary-state authority

`ActSerialBoundaryResultImage` supplies remaining P-change history, previous carry, non-arithmetic carry reset and arithmetic condition-latch final authority. The image excludes M14F and masks out M14E carry and M14I implied-GOTO state. The live bridge restores only owned oracle mutations before transport and compares/commits after b55. Exact physical latch timing remains SOURCE-BLOCKED.

## M14K RAM control authority

`ActSerialRamResultImage` owns low-nibble family 050/070 selection and logical 1260 installed-block clear. Fixed 0070 and M14H C-derived selection are excluded. The live bridge restores oracle address/block changes before transport, compares the independent completed image and commits after b55. DATA destinations keep the M14C next-word b0/b1 lifetime; CRC payloads remain peripheral-owned. No physical-chip mapping or inferred clear bus protocol is claimed.

## M14L CRC control authority

The endpoint binds `ActSerialCrcControlResultImage` once from pre-instruction ACT/CRC state. The live bridge restores the oracle's selected flag/S3 writes before transport and commits matching structural results only after b55. `commit_control_flag` stores an independently staged flag without opcode decode or buffer/external-input mutation. CRC DATA/card electronics remain outside this slice; exact F2/CRC internal edge timing remains SOURCE-BLOCKED.

## M14M CRC logical data authority

CRC data-port reads/writes bind an independent `ActSerialCrcDataResultImage` before b0. The live bridge restores oracle C/FIFO/READY/F7 effects before transport and compares the frozen payload/storage result before commit after b55. FIFO wrap, capacity, read duplication into both C halves and upper-C write packing preserve the existing semantic contract. Only FIFO storage and READY/F7 are applied; other control flags and external inputs retain their owners. A changed pre-state rejects commit. This is a WORKING APPROXIMATION for logical completed-word authority; it does not transport CRC bits on DATA, infer a 28-bit flux order, or extend installed-RAM phase evidence to CRC ports. CRC electrical DATA ownership/edges and sense timing remain SOURCE-BLOCKED.

## M14N complete ACT authority coverage

The flow image now restores the oracle's implied-GOTO completion latch as well as PC/bank/stack state. `ActSerialAbsentRamReadImage` isolates the existing absent-slot zero-read compatibility rule: restore oracle C before transport, compare zero result and commit only after b55. Binding rejects installed RAM and CRC read ports, repeated/late topology binding, incomplete commits and changed topology/pre-state. It does not create a responder or DATA frame for missing RAM; physical absent-address behavior remains SOURCE-BLOCKED.

The live bridge verifies the entire pre-instruction ACT contract before transport, using its immutable snapshot, prior previous-carry and flow state. Any leaked oracle mutation is a hard contextual error. Whole ACT/RAM/CRC differential coverage spans all 1024 words, six address contexts, both carry states and normal/implied-GOTO state. Installed RAM results are compared only after the documented next-word b0/b1 tail. Unknown specials and ROM self-test remain rejected. This closes semantic ownership gaps without proving internal latch edges or removing the oracle.

M14T adds the public display_exposure module for completed-scan nominal ROM0 segment dwell; electrical waveforms and optical routing remain separate concerns.


M14W exports `StructuralBusSample`, `StructuralBusTrace` and `run_structural_display_fetch_cycle_traced` for optional diagnostic IS/PHI capture and ordinal-tick TSV export. The production transport and live authority bridge do not allocate trace storage or execute observer hooks. The trace API does not capture electrical DATA/SYNC/STR/RCD or calibrated PHI widths.


## M14X bounded live IS/PHI capture

The native diagnostic CLI `hp67emu --trace-is <1..4096 words> <output.tsv>` exports actual live power-on firmware words with frozen no-key inputs. Reset-hold time has no modeled bus transport and is explicitly excluded. Ordinal ticks remain uncalibrated.

The live cycle routes an optional recorder only at the word boundary; ordinary calls pass None. Traced and ordinary words share the existing restore/oracle/authority-commit bridge. A dedicated traced DATA twin advances the same logical DATA visitor, preserving next-word b0/b1 tail authority without recording electrical DATA. Original production transport loops remain unchanged. Capture storage is local, bounded and discarded after export; no UI state or persistent trace session is introduced. Invalid counts or capture errors fail before file writing.

Focused tests compare real firmware boot with and without recording, exercise alternating synthetic RAM ports through the same live bridge and validate export bounds/row count/workload labels. See research/M14X_LIVE_BUS_CAPTURE_2026-10-11.md.


## M14Y offline comparison

Adds GUI-free bus_trace module and native `--compare-is expected.tsv actual.tsv report.txt [expected_start actual_start]` adapter. Strict import preserves provenance/unknown levels, bounded inputs and optional driver coverage. Comparison uses explicit row indices and relative ordinal ticks, counts mismatches and unequal lengths, and writes a report. Native exit codes: 0 match, 2 mismatch, 1 input/output error. Input paths cannot be overwritten by report output. Windows GUI-subsystem processes must be invoked with Start-Process -Wait -PassThru in PowerShell gates. No transport loop or firmware execution changes. See research/M14Y_TRACE_COMPARISON_2026-10-12.md.

## M15F shared prefix export

Re-exports `run_structural_data_tail_prefix()` for the live adapter. The API
consumes b0/b1 on shared IS/display/DATA, then requires the paired transport to
continue from b2 without repeating electrical cells or resetting ROM0 samples.

## M15J five-ROM composition

Exports the `rom_devices` module containing fixed chip profiles and ROM-side
receiver state. Existing aggregate fetch APIs remain available for reference
fixtures; live composition opts into the five-chip path explicitly.

Exact launch/latch delays remain unproven. The part/profile evidence and
remaining bank/RAM bridges are recorded in
`docs/research/M15J_FIVE_ROM_ENDPOINTS_2026-10-16.md`.

## M15N autonomous serial RAM

Exports `serial_ram` and Hp67SerialRam/Error/Event plus the 64-word logical
capacity constant. The device is an aggregate logical RAM with independent
IS/SYNC command capture, DATA address/write reception, storage and read drive.
It is available to separate electrical compositions; desktop live is not yet
migrated. See `docs/research/M15N_AUTONOMOUS_SERIAL_RAM_2026-10-18.md`.


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


### M15T autonomous CRC control

The shared resolved sampling path now carries an independent CRC IS/SYNC
receiver and twelve control latches. Production constructors install it; the
explicit injected-opcode reference fixture disables it. Each command executes
one word after qualified reception and publishes a completed b55 result. The
live adapter checks the old mirror before freezing external contacts, compares
against both the oracle and M14L image, and commits the endpoint's result/S3.
Only explicit logical FIFO/transport handshake changes enter READY/F7; motor,
write mode and all other latches retain the endpoint's own state. Reset clears
the endpoint and preserves the selected production/reference composition.
Per-sample CRC/RAM command images commit only after DATA validation succeeds.
No CRC DATA serialization or physical F2/PHI latch timing is asserted. The live
benchmark includes the endpoint only in its production firmware row; injected
RAM/CRC workloads retain the explicit reference bridge.


### M15X retained CRC DATA storage

Exports the crc_owned_data module containing retained logical read/write buffers. No GUI or host media format dependency is added.
This closes logical storage ownership, not physical CRC DATA serialization or PHI timing. See `docs/research/M15X_CRC_OWNED_DATA_2026-10-23.md`.


### M15Y pin-derived display circuit (delivery 2/15)

Exports display_circuit as the GUI-free retained cathode/anode/exposure composition.
The b7 checkpoint, one-cell pulse bridge, whole-word cathode application and nominal segment dwell remain WORKING APPROXIMATION. See `docs/research/M15Y_DISPLAY_CIRCUIT_2026-10-23.md`.
