# Local regression checks

GitHub Actions is intentionally not configured for this project unless the repository owner explicitly authorizes it.

Validation is performed locally after pulling a branch. The current regression command is:

```text
RUSTFLAGS=-D warnings cargo test --all-targets
```

This treats compiler warnings as regressions and runs the source-documentation and architecture-boundary tests together with the emulator tests.

`cargo fmt --check` is intentionally **not** a gate yet because the existing photographic UI files predate the cycle-accurate foundation and are not fully rustfmt-clean. Formatting cleanup is tracked separately; enabling a formatting gate before normalizing that inherited baseline would make an unrelated historical style issue block emulator work.

Once the inherited UI baseline has been normalized, local validation should also run:

```text
cargo fmt --all -- --check
```

Clippy can be promoted to the local regression command after the repository has a clean warning baseline.

No workflow under `.github/workflows/` should be added or enabled without explicit permission from the repository owner.
