# `src/documentation.rs`

## Purpose

Expose the HP-67 handbook and the full PAC manuals in the emulator.

## Why it exists

Application instructions must be accessible independently of individual magnetic cards.

## Relationships

The desktop app invokes Help/Documentation and selected-PAC access. This presentation module reads only PDF metadata and launches the platform reader; it never changes calculator state.

## Responsibilities

Use full PAC names, show missing-document status, cache availability outside the emulation loop, open only cataloged local PDFs or packaged web links.

## Implementation

The catalog contains the Spanish owner handbook and eleven PAC manuals. Desktop resolution checks documentation beside the executable and in the development checkout, validates the PDF signature and passes paths as direct process arguments. Web builds use the validated available.rs manifest and a Trunk copy-dir resource. The local publisher validates and copies all twelve files before advertising browser availability. A refresh button invalidates UI availability; no PDF decoder or per-frame filesystem scan is added.
