# M13 magnetic-card closure audit — 2026-09-19

## Scope

This audit records the implemented HP-67 magnetic-card path and separates completed functional behavior from lower-level electrical details that remain source-blocked. The rule is the same as the rest of this project: an unsourced electrical detail is not replaced by a convenient emulator convention.

## Functional path now implemented

The production path is:

1. the host owns one complete `Hp67MagneticCard`;
2. `CardInsertionEnd` selects logical Track 1 or Track 2 by physical end-for-end orientation;
3. insertion asserts the CRC external card-present contact;
4. real HP-67 firmware detects the card and asserts motor-on;
5. the live machine preserves the firmware-observed motor/head/startup-ready sequencing;
6. `Hp67CardTransport` advances the selected logical track at the source-backed 28-bit record cadence;
7. the CRC pair of read/write buffers mediates every transfer;
8. real firmware reads through architectural address `0x9B` or writes through `0x99`;
9. after record 34 the card-present contact drops, firmware stops the motor, and the same whole card is returned to the host;
10. if firmware requires another pass it displays physical `Crd`; the same card can be rotated and reinserted by the opposite end.

No CPU or UI path writes program/data RAM from card contents directly.

## Media and host formats

`Hp67MagneticCard` owns two independent logical tracks. Each track can be unrecorded or contain exactly 34 28-bit CRC-visible records and has independent write-protect/dirty state.

Host adapters:

- Teenix `.hpp`: import-only compatibility format. Header IDs describe firmware payload class (1/2 data primary/secondary, 3/4 program first/continuation); they do not universally identify the physical card side. Catalog metadata may therefore bind a Teenix payload to a physical side independently of its header.
- `.hp67raw`: 238-byte logical two-track image requiring both tracks to be recorded;
- `.hp67card`: versioned 250-byte logical container preserving recorded/unrecorded and write-protect state.

These formats are above the magnetic-flux layer. Their byte/nibble packing is not a physical recording-order claim.

## Firmware two-pass proof

Pinned HP-67/97 firmware distinguishes program header IDs 3 and 4:

- ID 3: program steps 001-112;
- ID 4: program steps 113-224.

When the second half is required, the firmware calls `card_prompt`, constructs the display codes for `C`, `r`, `d`, enables the display and waits for a new physical card-present assertion.

The production live regression deliberately makes the second half non-empty, writes the first logical track, requires the physical `Crd` segment frame, returns and rotates the same `Hp67MagneticCard`, writes the second logical track and checks the ID-4 header. It then boots a fresh RUN-mode machine, reads Track 1, requires a second real `Crd` wait, reinserts the same card by the opposite end and requires the non-zero second-half program RAM register to be restored by Track 2. This bidirectional proof is the authority for the UI continuation flow.

## Timing

The source-backed nominal CRC record interval is 28,000 us. HP documents unit-to-unit card-speed variation of approximately +/-5%; `Hp67CardSpeed` accepts only 95-105% of nominal.

The transport accumulates exact phase as elapsed microseconds multiplied by the speed percentage against a 2,800,000-unit record threshold. It does not independently round all 34 record intervals.

Exact insertion-to-head travel and motor acceleration are not assigned invented constants. The current live boundary synchronizes head activation to the real firmware startup sequence that was required to avoid starting magnetic data during the CRC/head readiness handshake.

## Physical flux layer

One logical card stream is physically represented by two parallel self-clocking magnetic tracks. `card_flux.rs` models the documented invariant that every logical bit cell produces one reversal on exactly one physical track:

- logical 0 -> Zero track reversal;
- logical 1 -> One track reversal.

`Hp67SelfClockingFluxPair` represents exactly 952 already-ordered bit cells. Absolute magnetic polarity is not modeled because only reversal placement is required by the sourced encoding.

## Explicit source-blocked boundaries

### 28-bit CRC serialization order

A reviewed HP-65 capture explicitly states LSB-first physical recording. HP's 1976 HP-67/97 article states that the mechanical design and two-track recording scheme were retained, but also documents a new firmware-controlled CRC with two 28-bit buffers. No reviewed HP-67/97 source states which significance bit leaves a 28-bit CRC buffer first.

Therefore the emulator does not infer HP-67/97 bit order from HP-65, Teenix text packing or `.hp67raw`.

### Insertion/head geometry

Service documentation distinguishes motor switch, motor start and later head-switch closure. Exact HP-67 insertion distance, acceleration curve and first-bit phase have not been pinned strongly enough to make a physical-time constant.

### Sense-amplifier and PHI-relative timing

The 1826-0322 sense-amplifier boundary and the CRC-visible result are known, but exact pulse/edge timing relative to the calculator clocks is not yet sourced sufficiently for production electrical scheduling.

These three items belong to the lower electrical-fidelity workstream and must remain open until evidence exists.

## SD-14A Moon Rocket Lander

The committed SD-14A visual card is artwork only. The current application accepts verified external magnetic media and can run it through the live reader, but the repository does not yet contain a redistributable, inspected magnetic image proven to be the SD-14A Moon Rocket Lander card.

A current Teenix distribution advertises 24 HP-67 demo-card files, and HP material identifies Moon Rocket Lander as Standard Pac program 14, but the binary distribution could not be inspected in the present environment. The project therefore does not silently bind any third-party card image to the artwork.

## Closure criterion

For the current integration milestone, M13 is functionally complete when the branch passes the full local formatting/warnings/all-targets gate with the firmware-driven second-pass regression. Lower-level serialization/head/sense timing remains explicitly tracked as electrical fidelity work rather than being approximated inside the functional reader.

The closure gate must not be recorded as passed until it is executed on a machine with the Rust toolchain.

## Final closure gate — 2026-09-20

The full local M13 closure gate passed after the checked-in Teenix corpus exposed and locked three compatibility details: legacy CR-only line endings, decimal nibble records, and least-significant-nibble-first record serialization. The catalog also established that firmware header class is not universally a physical-side identifier: SD1-12A contains two independent header-3 program sides and is mapped explicitly to opposite physical card tracks.

The successful gate executed, in order:

- `cargo fmt --all -- --check`;
- focused `program_library::tests::every_catalog_entry_decodes_to_one_physical_card`;
- `RUSTFLAGS=-Dwarnings cargo test --locked --all-targets`;
- `cargo build --locked --release --bin hp67emu`.

The command reached `M13 FULL CLOSURE GATE PASSED`. M13 is therefore closed for the integration milestone track. The three explicitly source-blocked lower-level electrical items documented above remain future electrical-fidelity work and are not hidden by this closure.


## Printed-face identity

The host presentation no longer paints every magnetic object as SD-14A. Native raw/container images and blank cards have no verified printed identity and therefore use neutral artwork. The Moon Rocket Lander face is selected only from matching Teenix card metadata. This keeps the closure claim about magnetic media independent from the still-unavailable verified built-in SD-14A payload.


## Data-card W/DATA proof

M13 also covers the two data-card header classes instead of treating program cards as representative of all media. The live test uses real `f` + `ENTER` to enter `W/DATA` after `1`, `Σ+` makes secondary registers non-zero. Firmware owns both `Crd` waits. The first physical pass must be header ID 1 for primary registers; the same card rotated end-for-end must carry header ID 2 for secondary registers. A fresh RUN-mode machine then reads those two passes through the real CRC data-read path and must observe the same header sequence with a firmware `Crd` continuation between them.

The functional card-header matrix is therefore: 1 = primary data, 2 = secondary data, 3 = program steps 001-112, 4 = program steps 113-224. All four are covered by live real-firmware regressions. This matrix describes firmware payload meaning, not a universal physical-side number: HP's SD1-12A English-SI Conversions card is explicitly documented as two independent one-pass sides, and both imported Teenix sides correctly carry header 3 while occupying opposite physical card sides.


## Host persistence

The card object returned by firmware writes can now be persisted from the desktop. `Ctrl+S` opens an egui Save Magnetic Card window with an editable path, so no new OS-dialog crate or lockfile dependency is needed. `.hp67card` is the lossless native format; `.hp67raw` export is allowed only when both logical tracks are recorded. Saving while the card is inside the live reader is rejected. A successful host write clears the card's dirty flags; serialization or filesystem failure leaves them set.

## Opposite-end visual proof

The host renderer receives the actual `CardInsertionEnd`. A second pass by End2 renders the same card face rotated 180 degrees, including text, logo and edge marks, while the reader motion remains tied to live record position. This closes the visual half of the physical same-card reinsertion requirement without changing firmware or transport authority.

## Crd display-code correction

The first full M13 closure run exposed an older display-table inconsistency rather than a magnetic-card lifecycle failure. Real HP-67/97 card firmware builds the continuation prompt with display codes `0x0B, 0x0A, 0x0D` and labels them `C, r, d`; Hewlett-Packard documentation independently says the user is prompted by `Crd`. The previous ROM0 table decoded `0x0A` as `o` and `0x0C` as `r`, making the physical `Crd` detector impossible to satisfy even while firmware was executing the correct prompt path. The decoder is corrected to `0x0A=r` and `0x0C=o`; the earlier capture transcription is retained as an evidence discrepancy rather than overriding firmware-plus-user-visible hardware behavior.

## Blank-card reader-entry recovery

The post-closure UI test exposed a host lifecycle bug rather than a CRC bug. Previously, insertion immediately set the presentation to `ReadingFromRight` even if firmware never asserted `motor_on`; after a blank card produced the authentic RUN-mode `Error`, reinserting it while the error remained active therefore left the UI frozen at record progress zero and prevented creation of another blank card. M13 now distinguishes physical insertion from motor takeover. `WaitingAtReader` keeps the exposed right edge visible until firmware starts the motor; right-clicking it withdraws the still-unstarted card by deasserting `card_present` and returning the same media object. Primary click on an otherwise empty reader now creates a blank card directly, so blank-card insertion no longer depends on a hidden secondary-click convention.

## Power-off and withdrawal interaction hardening

A physical HP-67 cannot transport a newly inserted card with POWER OFF, so the host now rejects every insertion route while the mechanical power state is off: loaded media, newly created blank media and opposite-end `Crd` continuation. The UI reader hotspot is hover-only while OFF, while the app-level insertion functions enforce the same rule as a second boundary. Waiting-card withdrawal also no longer relies on the narrow progress-zero card rectangle; the secondary-click target covers both the physical reader hotspot and the visible waiting-card portion. This removes scale/clip-dependent right-click misses without allowing withdrawal once motor/head transport has actually begun.

## Repeated blank cards and power-cycle reset

The right reader is now reusable without first deleting host media manually. A secondary click explicitly requests a fresh blank card whenever the reader is physically free; this includes `ParkedLeft` and passive-holder states, so a previous returned card no longer suppresses the reader hotspot. The fresh blank replaces the previous host-side card object and enters the same firmware-controlled insertion path.

HP's Owner's Handbook states that turning the HP-67 OFF and ON restores the default functions, and HP program-library material notes that loaded program memory is not retained through power-off. The emulator therefore resets the electronic machine on OFF->ON rather than resuming an `Error`, card wait, program counter or RAM state. The mechanical W/PRGM-RUN switch is preserved because power cycling cannot move it. Any physical magnetic card still inside the modeled reader is recovered to the host before reset, preserving its media contents while discarding only transient transport phase.
