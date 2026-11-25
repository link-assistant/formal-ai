---
bump: patch
---

### Fixed
- CI/CD false negatives, false positives, warnings and errors found in the issue #1639 audit:
  - a rustfmt diff and a `large_enum_variant` clippy error reached `main`, because formatting ran only on `main`; the pull request native lane now runs `cargo fmt --check` before clippy;
  - release preflight no longer reports the crates.io token as "unknown" on every `main` push; it verifies the token and crate scope with a no-archive publish probe that cannot publish anything;
  - the evidence of every failing scheduled, dispatched and `main` run is kept under `dev/log/issues/1639/pulls/1640`, with a comparison against the eight pipeline templates.
