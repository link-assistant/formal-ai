---
bump: patch
---

### Fixed
- CI/CD false negatives, false positives, warnings and errors found in the issue #1639 audit:
  - a rustfmt diff and a `large_enum_variant` clippy error reached `main`, because formatting ran only on `main`; the pull request native lane now runs `cargo fmt --check` before clippy;
  - release preflight no longer reports the crates.io token as "unknown" on every `main` push; it verifies the token and crate scope with a no-archive publish probe that cannot publish anything;
  - the Agent CLI E2E harnesses set `FORMAL_AI_TRACE_REQUESTS=1` without `FORMAL_AI_LOG_LEVEL=trace`, so after the request trace moved to the trace level every evidence projection counted zero requests; every launch now raises the level, and a pull request test scans every harness;
  - the issue #707 generalization reference compared dotted tool names with serde variant names, so every held-out case failed on `main`; the comparison moved into a tested module, and a failing run now keeps its logs and plans;
  - a native capture case re-executed its body inside a replay child, so `main` accepted 27 of 28 capture cases; it was removed, and a pull request test now rejects such cases;
  - the Aider issue report matched menu choices only at the start of a line, and the one-line menu broke that;
  - a download-artifact pattern that matches exactly one artifact was extracted flat, so a one-leg Issue 1028 ladder joined as "0 wrote a result"; the shared download action now keeps the artifact folder for every pattern selection;
  - each of these main-only failures is listed in `data/meta/main-only-regressions.lino` with the pull request test that now catches it;
  - the evidence of every failing scheduled, dispatched and `main` run is kept under `dev/log/issues/1639/pulls/1640`, with a comparison against the eight pipeline templates.
