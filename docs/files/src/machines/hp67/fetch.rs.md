# `src/machines/hp67/fetch.rs`

## Purpose

Provides the structural HP-67 shared-word transport and a single ACT serial endpoint for display output, ROM-address output, ROM-word input and the source-backed lifetime of the concurrently executing instruction on the resolved IS/ISA electrical net.

## Why it exists

Direct HP-67 evidence places ACT/ROM0 display traffic at b0..b7, ACT ROM address at b16..b27 and selected-ROM response at b46..b55 within the same 56-bit word, while the word fetched in cycle N executes during cycle N+1. A maximum-fidelity transport must preserve that overlap: the executing instruction cannot remain an untimed action detached from the physical word, arithmetic must not read state after the architectural fallback has already modified it, the UI cannot inject a display byte, and the downstream cathode driver cannot choose what the ACT emits.

## Relationships

Uses `display_register_index_for_scan_slot()` and `ActArchitecturalState` from `act.rs`; `ActSerialExecution` from `act_serial_execution.rs`; `ActSerialStateSnapshot`, `ActSerialAluInputs` and `ActSerialDigitAluResult` from `act_serial_state.rs`; evidence-backed windows from `timing.rs`; wired-high/address/ROM helpers from `isa.rs`; `Rom0DisplayEndpoint` and `Rom0StrEvent` from `display.rs`; and `Hp67ElectricalBackplane` for the weak-low resolved IS net. Firmware remains behind `Hp67RomWordSource`. The 1820-1749 cathode model is strictly downstream.

## Responsibilities

`ActSerialEndpoint` owns every currently modeled ACT role on the structural word: the fifteen-word display phase, source-backed display-code drive at b0..b7 (live A/B while enabled and the measured ROM0 blank code `$20` while DISPLAY is off), address drive at b16..b27, returned-word sampling at b46..b55, the b0..b55 lifetime of the word executing concurrently with the fetch, its immutable pre-instruction A/B/C/P/radix snapshot, and the ADD/SUB carry-or-borrow chain across selected digits. It rejects replacing an execution that has not completed. `RomFetchEndpoint` reconstructs the twelve address bits and emits the selected ten-bit ROM word. ROM0 independently reconstructs display data from the same resolved line. Floating/contentious samples remain hard failures.

## Implementation

`begin_execution()` now receives the pre-instruction `ActArchitecturalState`, derives the instruction-state decoder input from it, creates the `ActSerialExecution` and captures an `ActSerialStateSnapshot` before the architectural fallback runs. Starting another word resets the serial arithmetic chain and last digit result. `run_structural_word_transport()` advances that execution exactly once per structural bit after the current four-subphase scheduler scaffold for b0 through b55.

For ADD/SUB actions, the endpoint evaluates a selected digit after its fourth serial bit coordinate has been traversed. The first selected digit uses the opcode's architectural carry/borrow seed; each following selected digit receives the prior `ActSerialDigitAluResult.chain_out`. This is a source-backed digit-chain checkpoint, not a claim about the still-unknown PHI-relative result-write edge. `serial_execution_state()`, `serial_alu_inputs()`, `serial_arithmetic_chain()` and `last_serial_alu_digit_result()` expose the structural state for tests and diagnostics without writing architectural registers.

The display path remains ACT-owned. For a word with an active serial execution, the endpoint reads the selected A/B source from that word's immutable pre-instruction snapshot during b0..b7; this prevents the instruction-boundary fallback from exposing post-instruction register contents inside the same physical word. Fetch-only display cycles use the supplied live ACT state. When the architectural DISPLAY latch is off, the endpoint emits the HP-67 ROM0 blank byte `$20` observed on the physical IS bus instead of exposing arbitrary working-register A/B contents to ROM0. Slot 15 wraps the display phase and emits the coarse ACT-owned RCD boundary; ROM0 supplies STR. `StructuralWordResult.display_byte` remains only a ROM0-side observation.

This slice advances arithmetic ownership from instruction-only metadata to source-backed pre-state and digit-to-digit carry/borrow propagation. Architectural A/B/C destination writes still remain at the instruction boundary until physical evidence fixes their intra-bit/PHI commit relation; exact PHI launch/sample edges and STR/RCD overlap ordering remain explicitly unclaimed.


M12 source audit: the HP-29C service manual documents the Woodstock ACT sending ROM0 one character/blank code per 56-bit word; Tony Nixon's direct HP-67 logic-analyser capture identifies `$20` as a blank ROM0 display code; Nonpareil's Woodstock core applies `display off`/`display toggle` before its same-cycle display scan and emits no visible segments while disabled. The ROM0 decoder continues to reject undocumented values such as `$50`; DISPLAY-off handling belongs at the ACT source, not in the decoder.


M12 cycle-4430 evidence: executing `0132` had pre-state A/B `(5,0)` and post-state `(0,5)` at the selected display digit. The previous transport read the post-state and emitted undocumented ROM0 byte `$50`; the same-word display source now comes from the already-existing serial pre-state snapshot. The separate HP-documented B-modifier recoding remains an explicit open electrical-fidelity item and is not guessed here.
