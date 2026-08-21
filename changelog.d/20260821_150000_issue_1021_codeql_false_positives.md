---
bump: minor
---

### Changed

- The deterministic sampling seed in `translation::selection` is called a seed,
  which is what it is. Its parameter was named `salt`, and CodeQL's
  `rust/hard-coded-cryptographic-value` treats *any* argument reaching a
  parameter literally named `salt` as a cryptographic salt: every configuration
  literal that flowed into `sample_index` — `0.0`, `1.0`, `0.7`, the
  `SolverConfig` defaults — was reported as a hard-coded salt, 98 critical
  alerts across 24 files. Nothing on that path is cryptography; `fnv1a64` is a
  non-cryptographic hash and the seed only makes a draw reproducible.
- `PromotionApplyOutcome::agent_session_ids` is now
  `PromotionApplyOutcome::agent_session_digests`. The values were already
  content-addressed FNV-1a digests of the recorded session JSON, as the field's
  own documentation said, and are committed as evidence under
  `docs/case-studies/`; the `session_id` spelling made CodeQL's
  `rust/cleartext-logging` heuristic read `formal-ai improve`'s evidence line as
  a session token written to a log.
  The field is `pub` on a re-exported type, so this is a breaking rename and the
  bump is `minor` rather than `patch` -- on a 0.x crate that is where an
  incompatible change goes.
- `tests/unit/ci-cd/codeql_sink_heuristics.rs` now holds both heuristics over
  every Rust file the CodeQL configuration analyses, so a name that a static
  analyser will read as a credential fails here first, at the site that
  introduces it, rather than as a critical alert on a pull request.
