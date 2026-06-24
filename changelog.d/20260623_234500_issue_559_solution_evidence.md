---
bump: minor
---

### Added
- Issue #559 (R334): the evidence pipeline. The meta core now joins its separate link artifacts — the problem frame, the recursive work-unit tree, the need-satisfaction ledger, and the method registry — into one end-to-end `SolutionEvidence` record (`src/solution_evidence.rs`). For every detected need it traces the full chain `frame need → work-unit leaf → ledger status → catalogued method`, with `accounted_for` (every need has a connected, non-pending status) and `fully_resolved` (every need is satisfied) flags, so "ensure every detected need is addressed in the response" is a single auditable fact rather than four projections a reader must reconcile by hand. The ledger rows gained additive `unit_id`/`route` links to support the join. The evidence is serialized to Links Notation and emitted as a trace-only `solution_evidence` loop event, changing neither routing nor answers. Tracked by REQUIREMENTS.md R334.
