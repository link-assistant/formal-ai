---
bump: patch
---

### Added
- Added the gated meta self-improvement loop (`src/meta_self_improvement.rs`, R340): the meta algorithm now reads *itself* — the recursive-core recipe (the algorithm encoded as Links Notation) against the live `record_meta_core` pipeline (the algorithm as code) — and emits the *updated* algorithm as link-encoded output. It detects drift between the recipe's `meta_function` citations and the `record_*` stages the pipeline actually runs, proposing the additions and stale-citation removals that reconcile them (`MetaSelfImprovement::from_repo().propose()` → `MetaRecipeProposal::to_links_notation`). It is gated and proposal-only: the default `SelfImprovementMode::Off` proposes nothing and it never writes the recipe back, so adoption stays a human review step and behaviour is unchanged (issue #559).

### Changed
- Adopted the loop's first real finding: the self-describing recipe (`data/meta/recursive-core-recipe.lino`) now cites `record_solution_evidence` (and lists the `solution_evidence` event), so it describes every stage the pipeline runs and the loop reports the live sources as self-consistent.
