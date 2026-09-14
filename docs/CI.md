# Continuous regression checks

The repository has one required regression workflow at `.github/workflows/regression.yml`.

It runs on every push and pull request and performs:

1. `cargo fmt --all -- --check`
2. `cargo test --all-targets` with `RUSTFLAGS=-D warnings`

This means compiler warnings are treated as regressions, not informational output. It also means the source-documentation and architecture-boundary tests run automatically whenever GitHub Actions executes for the branch/PR.

The workflow intentionally does not add Clippy as a hard gate yet; the first priority is a stable zero-warning compile/test baseline. Clippy can be promoted to a required gate once the existing codebase is verified clean under the chosen toolchain.
