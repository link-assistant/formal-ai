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
  - three Benchmark Corpus Gate shards ran out of memory on the `near_pair` prompt: the composition search applied an amplifying fragment to its own output up to depth 8; it no longer does, and draft verification is capped at 512 MiB, so a runaway fails with a named `MemoryError`;
  - "how many days are in 2 years?" was routed to a concept lookup instead of a unit conversion; a bare duration now stays a unit conversion, and catalog construction records its trace;
  - several main-only Rust tests read live publishers or a shared source cache; they now use isolated caches and deny the network where they assert an answer;
  - the prebuilt full and specification test shards ran default solvers against live publishers (read access is on by default since #1582), which held shards past their budget; they now run with `FORMAL_AI_NETWORK=denied`, and the shard durations are refreshed;
  - the intent-span and minimal-core tests now also run in the pull request native lane;
  - each of these main-only failures is listed in `data/meta/main-only-regressions.lino` with the pull request test that now catches it;
  - the evidence of every failing scheduled, dispatched and `main` run is kept under `dev/log/issues/1639/pulls/1640`, with a comparison against the eight pipeline templates.
