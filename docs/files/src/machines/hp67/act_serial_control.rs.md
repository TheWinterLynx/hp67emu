# `src/machines/hp67/act_serial_control.rs`

## Purpose

Builds the completed-word structural result image for the focused ACT P/status condition-control family introduced in M14F.

## Why it exists

After M14E, arithmetic A/B/C/carry no longer takes its final causal value from the instruction-boundary executor, but P, status bits, P-change history, condition carry and `ThenGoto` state still do. M14F moves those selected control results behind the same structural `b0..b55` execution lifetime without inventing an unsupported internal bit or PHI write edge.

## Relationships

Uses `ActSerialExecution` only for decoded word class and full-word lifetime, and `ActSerialStateSnapshot` for immutable pre-instruction P/status/carry state. Its special-opcode semantics are implemented independently and differential-tested against `ActArchitecturalCore`; the architectural core remains the semantic oracle, not the causal live source. `ActSerialEndpoint` owns one optional image and marks it complete only when the structural execution reaches b55.

## Responsibilities

Decode the M14F ACT control family; preserve the pre-instruction P/status/carry inputs; model the universal per-word P-change history aging, previous-carry transfer and carry clear; apply status set/clear/test, P increment/decrement/set/test and the P side of load-constant/decrement-P; reproduce the documented P-wrap test rule; expose the completed final control image; and reject non-control words.

## Implementation

`ActSerialControlResultImage::begin()` captures only words in the selected control family. `complete_word()` first applies instruction-boundary bookkeeping equivalent to the ACT word boundary, then applies the decoded control action. Status/P tests produce both condition carry and `ThenGoto` state. The load-constant family deliberately claims only P/control effects; its C-digit write remains architectural fallback for this slice.

The production endpoint invokes `complete_word()` only after the bound structural execution has traversed b0..b55. The live bridge then compares P, P-change history, status, carry, previous-carry and instruction-state exactly against the architectural oracle before committing the structural image. This completed-word handoff is a **WORKING APPROXIMATION** and is not evidence for an internal 1820-2530 P/status write edge.
