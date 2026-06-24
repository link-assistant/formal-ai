---
bump: patch
---

### Added
- Added the method-selection comparison to the meta core (`src/selection.rs`, R339): for every atomic work-unit leaf it names both the method the hardcoded legacy authority (`specialized_handler_name`) would pick and the one the data-driven registry resolves (`MethodRegistry::method_for_route`, alias-aware), and classifies the pair as `agree`, `registry_rescues` (the legacy names no real handler but a route→method alias resolves one, e.g. `write_program` → `write_script`), `contradict`, or `unresolved`. Serialized to Links Notation and emitted as the trace-only `selection` / `selection:contradictions` events, this proves the registry never contradicts a valid legacy selection — the safety precondition for the registry to drive selection later and retire the hardcoded dispatch authority (issue #559).
- Added the `SelectionMode` knob (`Legacy` | `Registry` | `Compare`), surfaced as `SolverConfig::selection_mode` and the `FORMAL_AI_SELECTION_MODE` env override. The default `Legacy` records nothing and leaves both routing and the answer unchanged, so the comparison is an explicit opt-in (R13).

### Changed
- The self-describing recipe (`data/meta/recursive-core-recipe.lino`) now lists eleven ordered steps, adding the selection-comparison step and pinning `SelectionComparison::for_unit` and `record_selection` to their source.
