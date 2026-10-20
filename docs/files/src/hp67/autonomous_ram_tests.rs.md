# src/hp67/autonomous_ram_tests.rs

## Purpose

Regress production live autonomous RAM and oracle independence.

## Why it exists

Live command observation without independent read storage was insufficient fidelity.

## Relationships

Uses real firmware, default production RAM, explicit reference composition and the existing differential oracle.

## Responsibilities

Compare complete live ACT/CRC/RAM state, all 64 device registers, traces/ticks, reset and rejection of corrupted oracle RAM.

## Implementation

Two thousand firmware words in each transport must match the reference while device RAM matches every logical slot. Another real-firmware run alters only oracle memory before an installed read; the actual RAM response must remain original and fail the next-b1 authority comparison without committing C. A physical Digit1 contact can exercise firmware beyond idle.
