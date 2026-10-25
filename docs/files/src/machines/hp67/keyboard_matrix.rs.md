# src/machines/hp67/keyboard_matrix.rs

## Purpose

Own the 35 physical switches and resolve their bidirectional ideal digital connectivity.

## Why it exists

The retained key-code compatibility path did not model the actual cathode/KC switch network.

## Relationships

Uses Hp67Key identities and explicit Drive/LogicLevel inputs. Wiring follows HP-67 keyboard/display PCB sheet 3, PDF page 8. See docs/research/M16D_KEYBOARD_CONTACT_NETWORK_2026-10-25.md.

## Responsibilities

Keep contact closure separate from firmware key meaning, preserve floating and contention states, and expose connected cathode and column wires without implicit bias or scan timing.

## Implementation

Fixed-size contact mask and cached 13-node connected components; topology rebuilds only on contact changes. Resolution combines explicit Low/High sources in each component without allocation. Independent walk regressions compare all two-key chords and all source-node pairs. The live single-key adapter owns this contact bank; its retained key code and S15 compatibility lane still await a source-backed ACT scan/latch implementation.
