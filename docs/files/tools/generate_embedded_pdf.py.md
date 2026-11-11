# `tools/generate_embedded_pdf.py`

## Purpose

Generate/check the twelve embedded manual includes and 349 program-to-section links.

## Why it exists

The desktop emulator must carry its documentation in one executable and open the instructions for the selected card.

## Relationships

Presentation-only integration with app.rs, documentation.rs and the reviewed PAC import metadata. No machine or electrical state is changed.

## Responsibilities

Preserve PDF bytes, program references and deterministic machine behavior; keep document processing outside the live calculator path.

## Implementation

Uses the reviewed root-PDF profiles and import_manifest.json. Selects the first physical PDF page for each source application, preserving reference/file identity, and explicitly attaches SD-15C to the correct diagnostic section. No section is invented for supplemental entries without a source PDF. Check mode validates PDF signatures and compares generated tuples without modifying files.
