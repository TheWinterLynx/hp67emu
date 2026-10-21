# `src/machines/hp67/fetch.rs`

## Purpose

Provides the structural HP-67 shared-word transport and a single ACT serial endpoint for display output, ROM-address output, ROM-word input and the source-backed lifetime of the concurrently executing instruction on the resolved IS/ISA electrical net.

## Why it exists

Direct HP-67 evidence places ACT/ROM0 display traffic at b0..b7, ACT ROM address at b16..b27 and selected-ROM response at b46..b55 within the same 56-bit word, while the word fetched in cycle N executes during cycle N+1. A maximum-fidelity transport must preserve that overlap: the executing instruction cannot remain an untimed action detached from the physical word, arithmetic must not read state after the architectural fallback has already modified it, the UI cannot inject a display byte, and the downstream cathode driver cannot choose what the ACT emits.

## Relationships

Uses `display_register_index_for_scan_slot()` and `ActArchitecturalState` from `act.rs`; `ActSerialExecution` from `act_serial_execution.rs`; `ActSerialArithmeticResultImage` from `act_serial_result.rs`; `ActSerialControlResultImage` from `act_serial_control.rs`; `ActSerialSpecialRegisterResultImage` from `act_serial_special_register.rs`; `ActSerialModeLatchResultImage` from `act_serial_mode_latch.rs`; `ActSerialStateSnapshot`, `ActSerialAluInputs` and `ActSerialDigitAluResult` from `act_serial_state.rs`; evidence-backed windows from `timing.rs`; wired-high/address/ROM helpers from `isa.rs`; `Rom0DisplayEndpoint` and `Rom0StrEvent` from `display.rs`; and `Hp67ElectricalBackplane` for the weak-low resolved IS net backed by fixed HP-67 driver slots. Firmware remains behind `Hp67RomWordSource`. The 1820-1749 cathode model is strictly downstream.

## Responsibilities

`ActSerialEndpoint` owns every currently modeled ACT role on the structural word: the fifteen-word display phase, source-backed display-code drive at b0..b7 (the immutable pre-instruction A/B snapshot for executing words, live A/B for fetch-only words, and the measured ROM0 blank code `$20` while DISPLAY is off), address drive at b16..b27, returned-word sampling at b46..b55, the b0..b55 lifetime of the word executing concurrently with the fetch, its immutable pre-instruction A/B/C/P/radix snapshot, the ADD/SUB carry-or-borrow chain across selected digits, and the incremental arithmetic result image for all 32 arithmetic opcodes, the M14F completed-word P/status control image, the M14G completed-word special-register image, and the M14H completed-word mode/latch image. It rejects replacing an execution that has not completed. `RomFetchEndpoint` reconstructs the twelve address bits and emits the selected ten-bit ROM word. ROM0 independently reconstructs display data from the same resolved line. Floating/contentious samples remain hard failures.

## Implementation

`begin_execution()` now receives the pre-instruction `ActArchitecturalState`, derives the instruction-state decoder input from it, creates the `ActSerialExecution` and captures an `ActSerialStateSnapshot` before the architectural fallback runs. Starting another word resets the serial arithmetic chain and last digit result. `run_structural_word_transport()` advances that execution exactly once per structural bit after the current four-subphase scheduler scaffold for b0 through b55. The ACT and temporary aggregate ROM responder occupy fixed dense driver slots. They are released once at the word boundary, then the transport publishes only actual driver-state transitions instead of rewriting identical High-Z/high states every bit. Each bit-cell now resolves IS once after both drivers have published their current state; that same resolved sample is reused by contention detection and whichever ROM0/ROM/ACT receiver owns the bit. Resolved IS values, ownership, contention behavior and all b0..b55 sampling points are unchanged. The ACT-owned ROM0 display byte is also encoded once at the start of the word because its selected source is immutable across b0..b7; the eight serial cells then extract their individual bits from that latched byte. The existing conservative blank fallback for unsupported B modifiers is preserved.

For every arithmetic action, the endpoint uses the fourth serial coordinate of each digit as a structural result-image checkpoint. ADD/SUB additionally evaluate the selected digit through `ActSerialDigitAluResult` and chain carry/borrow between selected digits. M14E routes clear/copy/exchange/shift/test operations through the same private `ActSerialArithmeticResultImage`, using the immutable pre-instruction snapshot for all source values. `serial_arithmetic_result_image()` exposes the accumulated final image after the structural word. The checkpoint remains bookkeeping and does not claim the physical PHI-relative register-write edge.

The display path remains ACT-owned. For a word with an active serial execution, the endpoint reads the selected A/B source from that word's immutable pre-instruction snapshot during b0..b7; this prevents the instruction-boundary fallback from exposing post-instruction register contents inside the same physical word. Fetch-only display cycles use the supplied live ACT state. When the architectural DISPLAY latch is off, the endpoint emits the HP-67 ROM0 blank byte `$20` observed on the physical IS bus instead of exposing arbitrary working-register A/B contents to ROM0. Slot 15 wraps the display phase and emits the coarse ACT-owned RCD boundary; ROM0 supplies STR. `StructuralWordResult.display_byte` remains only a ROM0-side observation.

M14E completes this final-state authority bridge for the entire arithmetic opcode family: the actual shared-word traversal now produces the A/B/C/carry image for clear/copy/exchange, ADD/SUB, shifts and zero/nonzero tests, while the architectural executor supplies only an oracle for those four outputs plus still-unmigrated instruction-boundary effects such as `ThenGoto`. The final handoff after b55 is a **WORKING APPROXIMATION**, not a claim that the physical ACT waits until b55 to write its internal registers. Exact intra-word register/carry mutation, PHI launch/sample/write edges and STR/RCD overlap ordering remain explicitly source-blocked or unclaimed.


M12 source audit: the HP-29C service manual documents the Woodstock ACT sending ROM0 one character/blank code per 56-bit word; Tony Nixon's direct HP-67 logic-analyser capture identifies `$20` as a blank ROM0 display code; Nonpareil's Woodstock core applies `display off`/`display toggle` before its same-cycle display scan and emits no visible segments while disabled. The ROM0 decoder continues to reject undocumented values such as `$50`; DISPLAY-off handling belongs at the ACT source, not in the decoder.


M12 cycle-4430 evidence: executing `0132` had pre-state A/B `(5,0)` and post-state `(0,5)` at the selected display digit. The previous transport read the post-state and emitted undocumented ROM0 byte `$50`; the same-word display source now comes from the already-existing serial pre-state snapshot. At that diagnostic point the separate HP-documented B-modifier recoding was still open; the subsequent M12 recoding section below records how that gap was closed without accepting arbitrary B values.


M12 ROM0 modifier recoding: the HP-29C Service Manual states that Woodstock sends four A bits followed by three bits recoded from the four-bit B display modifier, so raw `B<<4 | A` is not a valid hardware model. Direct HP-67 captures identify the ROM0 classes `$0x` for characters, `$20` blank/sign-class traffic, `$30` decimal-point traffic and `$4x` blank traffic. The pinned x11-calc HP-67 display path independently accepts only B formats `0,4,9` as number, `3` as decimal point, and `1,2,F` as blank/sign-related formats. The strict encoder still returns `ActDisplaySerialError::UnsupportedModifier` for any other B nibble rather than inventing an electrical mapping. M14 real-program execution exposed transient B=5 while a valid user program was running. Because the unknown display recode occupies only b0..b7 and must not invalidate the simultaneous ROM fetch at b16..b55, the combined structural transport now substitutes the already-measured ROM0 blank code for that one uncertain display character while continuing the instruction fetch. This is an explicit conservative visual fallback, not a claim that physical B=5 recodes to blank.


## M14B fused DATA experiment

The internal structural word transport is now generic over a monomorphized per-bit visitor. Existing production/fetch/display callers pass a no-op closure, which is intended to optimize away completely; the new `run_structural_display_fetch_data_phase_cycle()` passes `Hp67DataSerialWordPath::visit_transport_word_bit()` so DATA logical phase advances inside the same b0..b55 loop as IS/display/fetch. This function remains an experiment and does not drive `Hp67Net::Data` or choose DATA polarity/PHI edges. A normal regression proves that one DATA frame can span two structural words without disturbing ROM fetch or the word counter.


## M14B monomorphic fast-path correction

After validating live fused DATA, the established no-DATA structural benchmark showed a large slowdown in the fixed-word `full structural` and `production dual` rows even though real-firmware throughput remained near its historical value. The shared transport had been generalized with a per-bit generic visitor to support DATA. To protect the long-lived structural fast path from optimizer/codegen sensitivity, `run_structural_word_transport()` has been restored as a dedicated monomorphic no-DATA function, while `run_structural_word_transport_with_data()` is its DATA-aware twin. The live DATA path uses only the DATA-aware twin; existing non-DATA callers use the original-shaped transport again.

This deliberate duplication is a performance/fidelity boundary: the historical no-DATA path must remain stable, and new DATA work must not impose even compile-time specialization risk on it. Both functions still share the same source-backed IS/display/fetch logic and differ only by the explicit `Hp67DataSerialWordPath::visit_transport_word_bit()` call in the DATA variant.


A differential regression now runs the DATA-aware transport with no active DATA frame beside the monomorphic fast path and requires identical structural result, ROM address reception, display phase, backplane tick, word index and word bit. This protects the intentional duplicated loop from semantic drift while preserving separate code generation.


## M14C pre-word DATA tail

The DATA-aware twin now calls `visit_transport_word_bit()`. If the preceding DATA frame's b0/b1 tail was already consumed by the live-machine authority bridge before the following instruction boundary, the DATA participant skips only those two duplicate logical samples while the structural transport still executes the real b0/b1 IS/display/backplane cells and all four PHI transitions per bit. From b2 onward the DATA participant resumes in the same loop, including a back-to-back new frame. The monomorphic no-DATA transport is unchanged.


M14F adds a second private result image for the focused P/status condition family. It is created from the same immutable pre-instruction snapshot but does not claim a digit checkpoint: the image is marked complete only when the structural execution itself reaches b55. The live machine may then use it as completed-word authority for P, P-change history, status bits, carry, previous-carry and `ThenGoto` state after an exact oracle comparison. This remains a WORKING APPROXIMATION for internal ACT control timing; no P/status PHI edge is claimed.


M14G adds a third private completed-word result image for selected non-arithmetic register-transfer specials. It is captured from the same immutable pre-instruction snapshot and remains incomplete through b0..b54. When the structural execution reaches b55, the image applies the decoded register transformation and becomes eligible for the live authority handoff. The same b55 point also completes M14F control state for load-constant, allowing P and C authority to remain separate without either side reading the other's post-oracle mutation. This is a WORKING APPROXIMATION for final-state authority only; no internal serial-transfer or PHI write edge is claimed.


M14H adds a fourth private completed-word result image for selected mode/latch specials: DISPLAY toggle/off, decimal/hex selection, 14-digit display enable and RAM-address selection from C[1:0]. It remains incomplete through b0..b54 and is completed only after the structural execution reaches b55. Same-word display serialization continues to use the immutable pre-instruction snapshot, so deferring the live DISPLAY latch does not let the oracle's post-instruction value leak into b0..b7. The b55 handoff remains a WORKING APPROXIMATION and makes no PHI-relative latch-timing claim.


The pre-existing startup DISPLAY bridge is now execution-local: `set_execution_display_enable_override(true)` changes only the b0..b7 DISPLAY-enable source for the currently bound word. It does not modify `ActSerialStateSnapshot`, so M14H sees the real architectural pre-instruction DISPLAY latch even during the early power-on window.

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


## M14W optional structural bus trace

`run_structural_display_fetch_cycle_traced` is a diagnostic twin of the dedicated no-DATA word traversal. It drives and samples the same resolved IS net, advances bound serial execution and records all four committed PHI transitions per bit. `StructuralBusTrace` appends actual tick, transport-bit, PHI1/PHI2, IS and ACT/aggregate-ROM drive observations; `to_tsv` exports those values without converting ticks to microseconds. The b55 wrapping transition retains transport-bit 55 rather than mislabelling it b0. A failed word retains only its committed prefix.

The original no-DATA and DATA loops remain untouched: no production trace allocation, option branch or generic observer is introduced. The intentional diagnostic loop duplication is guarded by a full fifteen-slot differential regression with bound serial DISPLAY-toggle execution, result/display phase/ROM address/final-net/tick parity, reconstruction of display/address/word from captured IS, ownership checks and 224 transitions per word. Missing-ROM regression locks the partial prefix. This is a structural comparison aid, not a desktop live-bridge trace or electrical timing proof. Electrical DATA, SYNC, STR/RCD and calibrated PHI/sample edges remain outside the API. See research/M14W_STRUCTURAL_BUS_TRACE_2026-10-11.md.


## M14X bounded live IS/PHI capture

The native diagnostic CLI `hp67emu --trace-is <1..4096 words> <output.tsv>` exports actual live power-on firmware words with frozen no-key inputs. Reset-hold time has no modeled bus transport and is explicitly excluded. Ordinal ticks remain uncalibrated.

The live cycle routes an optional recorder only at the word boundary; ordinary calls pass None. Traced and ordinary words share the existing restore/oracle/authority-commit bridge. A dedicated traced DATA twin advances the same logical DATA visitor, preserving next-word b0/b1 tail authority without recording electrical DATA. Original production transport loops remain unchanged. Capture storage is local, bounded and discarded after export; no UI state or persistent trace session is introduced. Invalid counts or capture errors fail before file writing.

Focused tests compare real firmware boot with and without recording, exercise alternating synthetic RAM ports through the same live bridge and validate export bounds/row count/workload labels. See research/M14X_LIVE_BUS_CAPTURE_2026-10-11.md.

## M15B IS contribution guard

All four no-DATA/DATA and untraced/traced loops use `sample_transport_isa()`
after publishing both endpoint drives and before receiver sampling. Opposite
drives keep the existing `IsaContention` error. Active-low or multiple High
contributions fail with `InvalidIsaDrives` carrying word bit and a u16 mask
indexed by `Hp67Driver`. Cached counts keep the valid path allocation-free;
only the failure path scans contributor identities. Tests inject an additional
High driver into each loop and require failure at the first overlapping b5
display bit, with 20 previous transitions and no completed display byte.
No receiver consumes that invalid cell. Existing transport differential tests
remain the success-path parity checks. This does not prove exact ownership
handoff timing or detect multiple releasing output enables.

## M15D resolved SYNC qualification

ACT now publishes a coarse SYNC pulse over b46..b55 for normal successor
fetch and suppresses it when the bound executing word is an IF. A current
implied-GOTO is treated as data regardless of its opcode bit pattern; its
successor is normal. All four transport loops sample the actual resolved
SYNC net before consuming each IS return bit. The endpoint accumulates a
complete ten-bit qualifier mask, rejects floating/contention or changes
inside the return window, and exposes `fetched_instruction_state()`. Standalone
IS-only sampling cannot manufacture a valid qualifier.

HP-97 manual figure 2-5/section 2-15 is used under the owner's explicit
HP-97 equivalence instruction (2026-10-13). The bit-cell pulse model is a
WORKING APPROXIMATION: immediate publication at existing bit coordinates and
release at the completed word boundary do not assign physical PHI edges or
propagation. V2's absent exact SYNC edge is unchanged. Legacy TSV still exports
IS/PHI only; it does not acquire SYNC columns. No physical DATA model is added.

## M15E DATA net visitor

The two DATA transport variants call `visit_transport_word_bit_through_net()`.
Reference/default callers retain the old logical fixture; live opted-in words
publish/sample resolved DATA during the same 56-bit traversal. Preconsumed b0/b1
tails still replay pin state and validate contributors but do not duplicate sink
completion. The no-DATA loops and TSV columns remain unchanged.

## M15F clocked DATA-tail prefix

`run_structural_data_tail_prefix()` processes following-word b0/b1 through the
shared IS display receiver, resolved DATA and all eight PHI transitions. It freezes
the word's display byte from current A/B and marks a one-use ACT prefix token.
The remaining transport accepts only a verified b2 continuation with ROM0, retains
its first two display samples and resumes through b55. It does not reset IS
ownership between prefix and body. A normal word still starts at b0. Every variant
validates the transport start; mid-cell or unbound continuations fail contextually.

The newly bound ACT execution accounts for its first two serial coordinates as
bookkeeping before resuming b2; those coordinates have no separately clocked
replay. Existing digit/control result images retain their approximated mutation
timing. Prefix splitting fixes DATA transport chronology, not ACT internal latch
edges, analog output stage or exact within-cell sampling. Complete traces retain
224 samples/word and are regression-compared against whole-word transport.

## M15G resolved display controls

All four display-capable transport loops and the shared b0/b1 DATA prefix now
visit the control bridge. Thus prefix continuations retain the same control
receiver state as whole-word traversal. Display wrappers obtain STR/RCD from the
resolved receiver latch before advancing ACT scan phase; they no longer manufacture
those output events at word completion. Fetch-only callers without ROM0 retain
their prior scope. Legacy TSV columns remain unchanged (IS/PHI only).

The pulse bridge is a WORKING APPROXIMATION: STR is Low in b7 and High in
other cells; RCD shares that cell only in slot 15. Publishing before the four
scaffold PHI transitions and restoring High at b8 are not physical edge/width
claims. No extra tick or cathode-to-ACT feedback is added.

## M15H provisional PHI receiver scheduling

All four transport loops and the M15F prefix publish sources at the bit's
starting boundary, commit PHI1 falling/rising, sample resolved IS and DATA, then
commit PHI2 falling/rising. IS sampling is phase-guarded before ROM0, ROM or ACT
receives a bit. ROM0 control publication follows reception of the complete b7
byte at this checkpoint. ACT result bookkeeping remains at cell completion.
Trace twins record both halves and retain every committed transition. Successful
word traces remain 224 rows. Sampling/lookup failures now preserve the first two
transitions of the failing cell: the missing-ROM b27 regression requires 110
rows, last row b27 with both clocks High, rather than the former 108 rows.
Similarly IS ownership, SYNC and display-control failures occur two ticks later;
their original no-receiver/no-scan-advance assertions remain.

This is a WORKING APPROXIMATION with a source-backed stable interval, not
a measured latch-edge/propagation claim. Full words remain 56 cells and 224
transitions. See `docs/research/M15H_PHI_RECEIVER_SCHEDULING_2026-10-16.md`.

## M15I atomic serial publication

All four transports and the clocked b0/b1 prefix prepare source proposals, then
call the same guarded `publish_transport_sources()` commit. Wrong bit/phase fails
before any net mutation. DATA proposal failure likewise prevents partial IS/SYNC
publication. The word-start IS HighZ reset and per-endpoint sequential writes
are removed: committed sources persist until the next complete image replaces
them. Sources remain held through both clock pulses and are sampled at the
unchanged provisional PHI1-rising checkpoint. Existing coarse end-word SYNC
closure remains separate and unchanged; no exact falling-edge claim is added.

Regressions require wrong-phase publication to preserve every resolved net and
time, and an unowned DATA b2 proposal to leave the previous Low IS cell intact
when the next display bit would be High. The receiver remains at a two-bit prefix
and ACT scan phase does not advance. M15F/M15H parity and failure chronology are
preserved, including 224 transitions per complete word.

The launch boundary and PHI1-rising sample checkpoint retain M15H
WORKING APPROXIMATION status. No propagation delay, physical output stage or
unknown latch edge is promoted to proven hardware behavior.

## M15J five-ROM composition

`RomFetchEndpoint::physical_roms()` enables five independent ROM receivers;
its default remains the aggregate reference fixture. Address sampling and IS
source proposals dispatch to that set, while `received_address()` retains its
read-only inspection API. The live receiver still reads actual resolved IS;
no response word is handed directly to ACT. All four transports publish distinct
chip drives. The clocked DATA prefix now accepts the configured ROM endpoint
explicitly, releasing its five IS sources before display b0/b1. Its signature
change is applied to live and standalone callers. Legacy trace `rom_drive`
continues to annotate the combined ROM contribution only.

Focused tests exercise all five profiles through all four paths. The existing
split/whole DATA-tail and missing-ROM prefix regressions now run in both modes,
retaining exact 224-row words and 110-row failed lookup prefixes.

Exact launch/latch delays remain unproven. The part/profile evidence and
remaining bank/RAM bridges are recorded in
`docs/research/M15J_FIVE_ROM_ENDPOINTS_2026-10-16.md`.

## M15K autonomous ROM bank control

All four shared-word transport loops now deliver the same resolved IS/SYNC
return to the autonomous ROM bank observer after ACT validation at the existing
receiver checkpoint. `autonomous_roms()` is the live factory;
`physical_roms()` deliberately preserves M15J's provider-view reference fixture.
`read_word_in_bank()` defaults to bank 0 only for bankless test sources. No clocks,
source publication order or DATA prefix semantics change. A captured firmware
bank roundtrip is covered through all four variants with a deliberately wrong
provider bank view; the old 8192-view fixture remains intact.

See `docs/research/M15K_AUTONOMOUS_ROM_BANKS_2026-10-16.md`.

## M15M ordered ROM reception

SerialFetchError adds UnexpectedRomAddressBit and UnexpectedRomReturnBit,
each reporting the attempted machine-word coordinate and number of valid bits
already received. The five-chip endpoint rejects wrong sequence before mutating
its address/return state. Aggregate reference endpoints retain their historical
behavior. All four transports continue to supply the same resolved samples and
unchanged clock sequence; the existing full-transport bank roundtrip tests
exercise the new ordered receivers without fixture shortcuts.

See `docs/research/M15M_ROM_RECEIVER_ORDER_2026-10-18.md`.


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


## M15R autonomous RAM DATA identity

Autonomous RAM reads now publish through Hp67Driver::RamAggregate, a distinct
logical aggregate slot appended after all existing driver indices. Reference
ROM/RAM fixtures retain StructuralRomResponder. This is device identity, not a
new physical chip map. The shared source commit selects the RAM owner without
clearing foreign contributions, so a rogue ROM/reference drive remains visible
and can cause contention. No extra time edge or intermediate resolution.

The DATA path remembers autonomous receive ownership across body and next-word
b0/b1 tail. Its sole-owner guard requires RamAggregate for live reads; replacing
it with another named source at the same valid level fails before receive state
changes. Reset reconstructs the path and the RAM-capable transport binds its
owner again. The standalone RAM fixture also uses this identity. Exact physical
DATA output stage/polarity, per-IC mapping and PHI edges remain unresolved.
See `docs/research/M15R_RAM_DATA_DRIVER_2026-10-20.md`.


## M15S ACT receive-only RAM frames

Autonomous reads now open the ACT sink without any local payload or serial
source. No dummy register or locally generated read tail remains. Receive-only
body/tail metadata preserves required driver counts, RamAggregate identity and
actual resolved-net sampling. Writes/address frames keep ACT's real source.

The RAM-capable transport chooses this binding from the ACT transfer direction.
Reference no-RAM APIs retain their old payload-driven behavior. Production live
passes None for autonomous read payload; the frozen expected result remains only
in the post-tail oracle guard. No new clocks, allocations or device timings.
See `docs/research/M15S_ACT_DATA_RECEIVE_ONLY_2026-10-21.md`.
