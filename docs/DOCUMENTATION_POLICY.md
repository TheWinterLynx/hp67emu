# Documentation contract

## Goal

Every source file must explain not only *what* it contains, but *why* it exists, how it relates to the rest of the emulator, what responsibility it owns and how it performs that responsibility.

## Companion-file rule

Every Rust file under `src/` and `tests/` must have a Markdown companion under `docs/files/` using the complete repository-relative source path plus `.md`.

Examples:

```text
src/app.rs                         -> docs/files/src/app.rs.md
src/emulation/net.rs               -> docs/files/src/emulation/net.rs.md
tests/documentation_contract.rs    -> docs/files/tests/documentation_contract.rs.md
```

Each companion document must contain these exact sections:

- `## Purpose`
- `## Why it exists`
- `## Relationships`
- `## Responsibilities`
- `## Implementation`

Additional sections are encouraged for invariants, limitations, replacement criteria, timing assumptions and test strategy.

## Regression enforcement

`tests/documentation_contract.rs` recursively enumerates all Rust files in `src/` and `tests/` and fails when:

- the companion Markdown file does not exist;
- any mandatory section is missing;
- any required project-level architecture document is missing.

That means adding a new `.rs` file without documenting it makes `cargo test` fail immediately.

`tests/architecture_contract.rs` separately protects the reusable core from GUI/image dependencies.

## What is not covered by one-file-per-file companions

Generated/binary assets, Cargo metadata and top-level Markdown are documented centrally rather than creating recursive “documentation about documentation.” In particular:

- `assets/hp67.png` is documented in `docs/ASSETS.md`;
- architecture and design decisions live in `docs/ARCHITECTURE.md`;
- hardware evidence lives in `docs/HARDWARE_SOURCES.md`;
- sequencing lives in `docs/ROADMAP.md` and `TODO.md`.

If executable non-Rust source is added later (Python, WGSL, C, scripts, etc.), the regression must be extended in the same change so that the new source type is covered by this policy.

## Change rule

When behavior or ownership changes materially, update the source file and its companion Markdown in the same branch/commit series. Documentation is part of the implementation contract, not a post-release cleanup task.
