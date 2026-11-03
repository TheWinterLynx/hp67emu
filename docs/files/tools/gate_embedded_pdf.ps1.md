# `tools/gate_embedded_pdf.ps1`

## Purpose

Run the owner Windows gate and open an isolated portable executable.

## Why it exists

The desktop emulator must carry its documentation in one executable and open the instructions for the selected card.

## Relationships

Presentation-only integration with app.rs, documentation.rs and the reviewed PAC import metadata. No machine or electrical state is changed.

## Responsibilities

Preserve PDF bytes, program references and deterministic machine behavior; keep document processing outside the live calculator path.

## Implementation

Requires Rust 1.92+, checks generated metadata, formats, runs focused and all-target tests, builds release and runs custom diagnostics. Copies only hp67emu.exe into a fresh temporary folder; executes --verify-embedded-pdfs there and opens that same EXE after success. Restores RUSTFLAGS and working directory. This gate has not been executed in the agent Linux environment.
