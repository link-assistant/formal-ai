---
bump: patch
---

### Added
- Added the method-selection trace to the meta core (`src/selection.rs`, R339): for every atomic work-unit leaf, `MethodSelection::for_unit` names the method the single data-driven registry authority resolves (`MethodRegistry::method_for_route`, alias-aware — e.g. `write_program` resolves to `write_script` through its route→method alias), or marks the leaf `unresolved` when no method serves it, and counts resolved vs. unresolved leaves. Serialized to Links Notation and emitted as the trace-only `selection` event, this makes the dispatch the registry performs auditable per request.
- Added the `SelectionMode` knob (`Off` | `Record`), surfaced as `SolverConfig::selection_mode` and the `FORMAL_AI_SELECTION_MODE` env override. The default `Off` records nothing and leaves both routing and the answer unchanged, so the trace is an explicit opt-in (R13).

### Changed
- The self-describing recipe (`data/meta/recursive-core-recipe.lino`) lists the method-selection step, pinning `MethodSelection::for_unit` and `record_selection` to their source.
