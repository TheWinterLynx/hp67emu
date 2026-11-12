# Firmware plot viewer

## Purpose

Draw values explicitly captured from the active HP-67 display.

## Why it exists

PLOT builds a series without re-entering data in a graph form.

## Relationships

The desktop app dispatches PLOT or F8 and hands over a read-only optical frame. The viewer decodes it through hp67/plot_sampler.rs.

## Responsibilities

Append a finite value at the next sample index, retain the series when closed, and offer cursor inspection, CSV copying and clearing.

## Implementation

The native viewport opens independently. X is the one-based sample number and Y is the displayed value. Invalid displays do not append points. The series holds at most 16,384 points. PLOT is a host control and does not alter program memory or interpret pauses automatically.
