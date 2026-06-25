---
bump: patch
---

### Added
- Added the upward construction pass to the meta core (`src/meta_construction.rs`, R338): the construction half of the recursion. A post-order (bottom-up) walk of the work-unit tree composes each answer from leaf to root — every leaf is a base case constructed directly from the method that resolves its route (via the same `method_for_route` bridge the evidence join uses), and every parent is a recursive case composing its already-constructed children in source order. Serialized to Links Notation and emitted as the trace-only `upward_construction` / `upward_construction:steps` events, so both directions of the recursion — decompose and compose — are inspectable link data (issue #559).
- Added the `RecursionMode` knob (`Down` | `Up` | `Both`), surfaced as `SolverConfig::recursion_mode` and the `FORMAL_AI_RECURSION_MODE` env override. The default `Down` reproduces the pre-knob trace exactly, so the upward pass is always an explicit opt-in and the default solver behavior is unchanged (R13).

### Changed
- The self-describing recipe (`data/meta/recursive-core-recipe.lino`) now lists ten ordered steps, adding the upward construction step and pinning `UpwardConstruction::for_unit` and `record_upward_construction` to their source.
