# `tools/pac_import/tools/install_documentation_v2_3.py`

## Purpose

Validate and package the owner handbook and eleven PAC manuals.

## Why it exists

The installed local corpus must carry source PDFs as usable emulator documentation.

## Relationships

Called by publish_local_v2_2.ps1 before the owner gate. Reads the v2.0 import manifest, root source location and versioned documentation module template.

## Responsibilities

Validate all source hashes and PDF signatures before mutation, reject different existing documents, preserve card images and media, install the three UI access points and web folder copy.

## Implementation

Check-only mode validates the exact twelve sources and source patch anchors without writes. Installation streams files through a temporary staging folder, verifies SHA256 and writes the packaged availability list after copying. Text patches are idempotent and retain atlas constants and imported catalog contents. Executable lookup and PDF reading remain presentation-only.



Presentation wiring uses the shared alphabetical-order adapter, keeping Help at the far right and preserving previously imported atlas dimensions and card indices.
