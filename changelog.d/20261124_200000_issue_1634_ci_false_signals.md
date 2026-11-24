---
bump: patch
---

### Fixed

- Detect Changes jobs in `release.yml`, `coverage.yml` and `layered-ci.yml` sparse-check out the `#[path]` modules of `detect-code-changes.rs`, so downstream CI is no longer silently skipped; the staged release generator emits the same list.
- A new `scripts/check-sparse-checkout-rust-modules.mjs` guard, run by the Workflows check, fails when a sparse-checked-out Rust script misses one of its `#[path]` modules (`FORMAL_AI_SPARSE_CHECK_VERBOSE=1` prints every checked block).
- The Hive Mind draft adapter inserts diagnostics at the closing brace of `setEnv`, so it accepts upstream keys appended after the shared spread.
- The Rust unit and integration test targets compile again: private test-fixture imports, a stale `SORT_PYTHON` assertion, the private `concepts` module and the undeclared `tempfile` crate are fixed.
