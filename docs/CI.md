# Continuous regression checks

The repository has one regression workflow at `.github/workflows/regression.yml`.

It runs on every push and pull request and executes:

```text
RUSTFLAGS=-D warnings cargo test --all-targets
```

This makes compiler warnings regressions rather than informational output. It also runs the source-documentation and architecture-boundary tests automatically whenever GitHub Actions executes for the branch or pull request.

`cargo fmt --check` is intentionally **not** a gate yet because the existing photographic UI files predate this foundation and are not fully rustfmt-clean. Formatting cleanup is tracked separately; enabling a formatting gate before normalizing that inherited baseline would make an unrelated historical style issue block emulator work.

Once the existing UI is normalized, the workflow should add `cargo fmt --all -- --check` as a required step. Clippy can be promoted later after a clean baseline is established.
