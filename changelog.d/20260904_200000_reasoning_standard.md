---
bump: minor
---

### Added
- Issue #1073: a reasoning standard declared as data (`data/meta/reasoning-standard.lino`) and evaluated as pure predicates (`src/reasoning_standard/`). Seven gates — evidence before claims, documentation by default, formalized instructions, computed source trust, refutation variety, verify-after-act, honest failure reporting — are audited on every request, with no mode in front of the call. A gate that does not fire reports the trigger that was false, so the obligations are enumerated identically on a trivial request and a hard one.
- `data/meta/reasoning-standard-reference-episode.lino` encodes the reference dialog the standard was derived from; every gate is shown to fail under a mutation that removes the behaviour it enforces.
- `data/meta/reasoning-standard-recipe.lino` describes the procedure as data, grounded against the live source by `tests/unit/specification/reasoning_standard_meta_algorithm.rs`.

### Changed
- Source trust is derived rather than declared. Every source in `data/seed/sources-registry.lino` carries a `primacy` chain citing the site's own policy, and `SourceRecord::tier` is now `PrimacyChain::derive_tier()`. The hand-written tier survives only as `asserted_tier` and is checked against the derivation; `tier_from_seed`, with its silent `_ => independent_corroboration` arm, is gone.
- The meta core's depth defaults moved from the quiet setting to the full one: `RecursionMode::Down` → `Both`, `SelectionMode::Off` → `Record`, `SkillMode::Off` → `Accumulate`. The narrow modes remain for deliberately quietening a trace, but reasoning depth is no longer conditional on a caller asking for it.
- The recursive core recipe gains a thirteenth step, the unconditional reasoning-standard audit.

### Fixed
- Two delivery-document tests hard-required a changelog fragment to still be on disk, so they failed for every commit after the release that consumed it — `v0.346.0` deleted the fragments they read. `tests/unit/ci-cd/issue_1014.rs` and `tests/unit/issue_1021_closed_circle.rs` now follow the entry across its lifecycle, reading the fragment before release and the `CHANGELOG.md` section after, the way `tests/unit/docs_requirements_issue_656.rs` already did.
- `examples/regenerate_issue_922_open_proposals.rs` regenerates `examples/issue-922-method-learning/open-proposals.lino` from the live learner instead of leaving its content-addressed candidate id to be hand-edited whenever a pipeline stage is added. It refreshes only the machine-derived fields and keeps the two review decisions the document carries: the single strongest proposal, and the reviewer's own summary sentence.
- `data/seed/learned-methods.lino` is re-derived through the production promotion path instead of hand-edited: the thirteenth pipeline stage lengthens the recurring recursive-core tail from twelve operations to fifteen, so the adopted method is now the 851-byte `learned_recursive_core_e17957243eaaf6db`. The three canonical gates were replayed fresh for it (4/4, 13/13, 12/12) and the decision record is kept in `docs/case-studies/issue-1073/logs/issue-922-promotion-rerun.lino`.
