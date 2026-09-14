# `tests/documentation_contract.rs`

## Purpose
Enforces the repository's per-source Markdown documentation policy as an automated regression.

## Why it exists
Documentation requirements otherwise decay silently as new files are added. The user's requirement is that every code file remains explainable in terms of purpose, rationale, relationships, responsibilities and implementation.

## Relationships
Implements `docs/DOCUMENTATION_POLICY.md`, protects `docs/files/`, `TODO.md` and required project-level documents including the CI policy. The GitHub Actions regression workflow runs this test automatically.

## Responsibilities
Recursively enumerate Rust files, require a deterministic companion Markdown path, verify mandatory section headings, and require the project-level architecture/documentation/CI documents.

## Implementation
Uses `CARGO_MANIFEST_DIR` plus `std::fs` so no extra dependency is required. Any undocumented new Rust file makes `cargo test` fail with the missing path/section listed.
