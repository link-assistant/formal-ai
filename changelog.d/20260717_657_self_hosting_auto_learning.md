---
bump: minor
---

### Added

- Derive a review-gated auto-learning report for the self-hosting metric,
  ranking the attribution observations behind the metric contract as an
  associative links network. Two external Agent CLIs execute the task against
  `formal-ai serve` as their own model provider — Formal AI running issue #657's
  task using Formal AI, with no external model — and must derive a byte-identical
  report.

### Changed

- Generalize the four auto-learning modules into one `LearningReport` descriptor
  table. Identity now lives in the descriptor and derivation in a single
  renderer, so a new report is a row rather than a copied module; the planner
  routes through the table instead of a branch per report.

### Fixed

- Stop rendering every learning report under issue #686's identity and patching
  it back out per-report. The patch failed silently when it matched nothing, so
  a report could claim the wrong issue while ranking a different network.
