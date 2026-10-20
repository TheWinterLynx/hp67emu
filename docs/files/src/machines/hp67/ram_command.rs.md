# src/machines/hp67/ram_command.rs

## Purpose

Receive and classify RAM commands from the resolved IS/SYNC instruction window.

## Why it exists

Live and independent RAM compositions need one reusable pin-input receiver,
without giving the device an oracle opcode or duplicating command reception.

## Relationships

Uses the backplane's resolved IS/SYNC and provisional PHI receive checkpoint.
Serial RAM consumes the complete frame into its next-word command latch; the
DATA word path hosts an optional observer for live composition.

## Responsibilities

Require ordered b46..b55 samples, valid levels and a constant qualifier; publish
only a complete ten-bit frame; exclude implied-GOTO payloads from RAM commands;
classify address/clear/current and low-selector read/write instructions.

## Implementation

The fixed-size Copy receiver validates phase, coordinate, IS/SYNC resolution and
qualifier before mutation. At b46 it starts a fresh word; b55 publishes word and
qualifier. It never drives pins, advances clocks, takes opcodes or accesses ACT
state. Pure command classification preserves the M15N logical contract. Tests
cover all 1024 words/both qualifiers, real pin faults, duplicates/phase errors,
selectors and an altered valid net bit proving the received word is authoritative.
Exact PHI/latch timing and physical RAM partitioning remain unproven.
