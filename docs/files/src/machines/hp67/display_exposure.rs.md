# src/machines/hp67/display_exposure.rs

## Purpose

Integrate nominal segment dwell from validated ROM0/cathode output across complete fifteen-word scans.

## Why it exists

Direct HP-67 measurements establish approximate 40 us normal-segment and 30 us decimal dwell. Raw segment masks alone do not preserve this distinction. M14T creates a deterministic scan-level measurement boundary without inventing electrical edges or radiant energy.

## Relationships

Consumes decoded Hp67SegmentMask output from the existing resolved IS/ROM0 path, after cathode control validation in src/hp67.rs. Reuses observed timing constants from timing.rs. No GUI, wall clock, card format or architectural arithmetic participates.

## Responsibilities

Validate sequential slots before mutation; retain all fifteen raw scan slots; publish only complete scans; replace previous dwell, including blank slots; reset electronic history on power reset.

## Implementation

Hp67DisplayExposure stores pending and completed 15-by-8 integer microsecond dwell matrices. Each validated machine word supplies one slot. Enabled mask bits contribute the observed nominal dwell; disabled bits contribute zero. Fifteen 320 us words establish a 4800 us scan. The duplicate slot remains separate: no assumption about discarded or doubled optical exposure. Shared signs remain ROM0 E/G, without a transistor timing claim. These are WORKING APPROXIMATION scan-integrated dwell values, not electrically generated pulses, physical LED energy, measured current or final brightness. Optical integration and PHI-relative control edges remain future slices.

Tests cover ROM0 digit/decimal/sign decoding through publication, no partial-scan publication, phase rejection without mutation, replacement by blank scans, plus live real-firmware transport and reset in src/hp67.rs.
