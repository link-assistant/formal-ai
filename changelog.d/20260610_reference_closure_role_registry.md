---
bump: minor
---

### Added

- `meaning_definition_references_resolve_to_defined_meanings` CI gate proving
  the meaning graph is fully reference-closed: every `defined-by` target and
  every semantic-facet reference (notation/annotation/denotation/connotation)
  across all 35 `data/seed/meanings*.lino` files resolves to a meaning defined
  exactly once (issue #398, PR #399 review point #1).
- `data/seed/roles.lino` canonical reserved-role registry declaring all 207
  distinct `role` values exactly once, each classified as `kind meaning` (also
  a defined meaning slug) or `kind predicate` (role-only identifier).
- `scripts/generate-role-registry.py` to regenerate the role registry
  deterministically and idempotently from the meaning seed.
- `every_role_value_is_declared_in_the_registry` and
  `role_registry_is_in_lockstep_with_usage` tests keeping the registry and its
  usage in lockstep (PR #399 review point #2).
- `experiments/closure_audit.py` documenting the meaning-layer closure
  measurement.
