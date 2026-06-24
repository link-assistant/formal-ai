---
bump: patch
---

### Added
- Added white-box recursive reasoning to the meta core (`src/meta_reasoning.rs`, R337): every work unit now carries a human-readable thought in both directions — the downward thought (what span was observed, why it was decomposed or judged atomic, and which method an atomic leaf resolves to) and the upward thought (how the unit's answer is composed from its solved children). The reasoning is a parallel tree to the work-unit tree, serialized to Links Notation and emitted as the trace-only `work_unit_reasoning` / `work_unit_reasoning:steps` events, so the box is inspectable by users and developers — the reasoning, not just the predicate, is visible (issue #559).

### Changed
- The self-describing recipe (`data/meta/recursive-core-recipe.lino`) now lists nine ordered steps, adding the white-box reasoning step and pinning `WorkUnitReasoning::for_unit` and `record_work_unit_reasoning` to their source.
