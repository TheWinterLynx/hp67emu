# `tools/pac_import/tools/install_documentation_v2_3.py`

## Purpose

Retain the legacy local documentation bundle installer while protecting integrated-viewer checkouts.

## Why it exists

An older bundle must not downgrade the newer embedded desktop PDF interface.

## Relationships

The historical publisher calls this tool. Integrated builds use src/pdf_viewer.rs and tools/gate_embedded_pdf.ps1 instead.

## Responsibilities

Validate source PDFs before legacy installation, and reject attempts to overwrite an installed integrated viewer.

## Implementation

The presence of src/pdf_viewer.rs stops the legacy installation before any writes. Existing integrated-viewer users update with Git and run the new local gate. Original import behavior remains available on older checkouts.
