# `src/machines/hp67/act_serial_boundary.rs`

## Purpose

Provides completed-word authority for the ACT boundary effects left outside M14F, and for arithmetic condition instruction-state selection.

## Why it exists

PC/flow and arithmetic results were structural, but ordinary words still took P-change history, previous carry and carry reset from the architectural executor. Arithmetic condition words still took their ThenGoto instruction latch from that executor.

## Relationships

Uses immutable `ActSerialStateSnapshot` and the bound execution class. `fetch.rs` marks the image complete at b55. The live bridge restores oracle-owned-field mutations before transport and compares the completed image before commit. M14E retains arithmetic carry; M14F retains its entire focused P/status/control family; M14I retains implied-GOTO completion.

## Responsibilities

Shift P-change history and latch previous carry for words outside M14F. Reset carry only when M14E does not own it. Select the arithmetic condition state for operations 0x16..0x1b; other arithmetic operations remain Normal. Never overwrite unrelated status/P, registers, flow or peripheral effects.

## Implementation

The image is absent for the focused M14F family. Otherwise it derives final boundary values from immutable pre-state and explicit ownership masks. `restore`, `matches` and `commit` use the same masks. Preview values do not mutate live state; commit requires the complete b0..b55 lifetime. Exact internal latch edges remain SOURCE-BLOCKED, and completed-word authority remains a WORKING APPROXIMATION. Tests cover all ten-bit words over carry, normal/implied-GOTO interpretation and P-change histories against the composed oracle, plus exclusive ownership, completion timing and live boot.
