# src/machines/hp67/pin_integrity.rs

## Purpose

Audit source ownership and coverage of the current production digital bridge offline.

## Why it exists

Level-only traces can conceal agreeing foreign contributions, replaced sources and missing observations. A trace audit must distinguish those from an observed scoped pass.

## Relationships

Uses strict pin trace analysis and observed PHI coordinates; source rules follow the existing live bridge, wiring inventory and ROM profiles. Native CLI consumes the report.

## Responsibilities

Audit thirteen implemented contracts, enumerate all nineteen pins, bound diagnostics, preserve input and expose unassessed physical behavior.

## Implementation

Checks PHI ownership at every row and other assessed pins at the provisional PHI1-rising checkpoint. Sole ACT/ROM0/CRC/External roles require one binary named contribution. IS permits passive Low, ACT display/address, page-compatible physical ROM return and release elsewhere. DATA permits release or one ACT/RamAggregate contribution. Full source masks/levels are validated independently of the parser for direct callers. Reports count every violation while storing at most 64 findings. Exit 0 is scoped bridge observation, 2 violation, 3 incomplete coverage. F1/KC remain unassessed; bank identity, DATA direction, physical timing and magnetic serialization are never certified. There are no runtime hooks or new machine semantics.
