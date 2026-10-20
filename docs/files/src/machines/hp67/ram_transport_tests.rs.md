# src/machines/hp67/ram_transport_tests.rs

## Purpose

Regress autonomous RAM on the shared fetch/display/DATA composition.

## Why it exists

The independent M15N bus fixture alone cannot prove the production transport supplies device-owned read bits.

## Relationships

Uses optional-RAM transport APIs, clocked prefix, actual ROM command reception, ACT framing and trace recording.

## Responsibilities

Prove address/write/read/clear dependencies and mask-independent read response in both transports with 224 transitions per word.

## Implementation

A six-word ROM stream sends actual address/write/read/clear commands. Address and data frames finish next b1. Two deliberately wrong read masks must both reconstruct the same stored word; clear then zeroes the selected block. No executing opcode is supplied to RAM.
