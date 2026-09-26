---
bump: patch
---

### Added
- `experiments/issue_1138_feedback_recovery/` — reusable collector that
  recovers genuine user directives from session `.jsonl` transcripts
  (harness wrappers, agent reports, and cron echoes filtered out), with
  the 2026-09-23 recovered snapshot for issue #1138.

### Changed
- Requirement-status ledger regenerated with the post-L1 test path fix
  (shards name `rust/tests/`, older rows `tests/`); all 68 R1138
  requirements now read `implemented` with named automated tests,
  including R1138-B2-6 (external-benchmark floors pinned by
  `rust/tests/unit/specification/external_benchmarks.rs`) and R1138-B2-8
  (OEIS/python_docs replay pins).
- CONTRIBUTING.md records three standing directives: Opus-only
  sub-agents, classify CI failures before fixing, and the
  feedback-recovery audit protocol.
