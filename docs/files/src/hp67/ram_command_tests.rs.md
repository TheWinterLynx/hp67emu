# src/hp67/ram_command_tests.rs

## Purpose

Regress independent RAM control observation in the desktop live cycle.

## Why it exists

A future live RAM device must receive the same real wire command as ACT without
using the currently executing oracle/injected opcode as an input.

## Relationships

Uses actual live firmware transport, optional trace recording, ACT receive
inspection and the attached shared RAM command receiver.

## Responsibilities

Compare raw word/qualifier across independent receivers and both transports;
require normal RAM commands and implied-GOTO exclusion; preserve 224 transitions,
trace length, ACT/RAM results, boot idle and reset clearing.

## Implementation

A 1200-word real-firmware comparison checks each received frame and full state.
A separate short synthetic execution fixture deliberately injects 270 while
ROM fetch 001 returns 3e3. The RAM observer must report 3e3, proving that the
injected opcode cannot feed it. This is control observation only, not autonomous
RAM storage integration or a physical latch-edge claim.
