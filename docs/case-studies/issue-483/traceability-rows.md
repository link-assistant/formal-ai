# Issue #483 Traceability Rows

Rows for a shard `docs/requirements/issue-483-small-model-fallback.md`,
kept here for the maintainer to merge into
`docs/requirements-traceability.md` (the shared table is not edited by
this change).

| ID | Shard | Delivered | Automated test | Manual confirmation |
| --- | --- | --- | --- | --- |
| R483-1 | docs/requirements/issue-483-small-model-fallback.md | 2026-09-30 (this pull request) | rust/tests/unit/issue_483_small_model_fallback.rs | not yet confirmed |
| R483-2 | docs/requirements/issue-483-small-model-fallback.md | 2026-09-30 (this pull request) | rust/tests/unit/issue_483_small_model_fallback.rs | not yet confirmed |
| R483-3 | docs/requirements/issue-483-small-model-fallback.md | 2026-09-30 (this pull request) | rust/tests/unit/issue_483_small_model_fallback.rs | not yet confirmed |
| R483-4 | docs/requirements/issue-483-small-model-fallback.md | 2026-09-30 (this pull request) | rust/tests/unit/issue_483_small_model_fallback.rs | not yet confirmed |
| R483-5 | docs/requirements/issue-483-small-model-fallback.md | 2026-09-30 (this pull request) | rust/tests/unit/issue_483_small_model_fallback.rs | not yet confirmed |
| R483-6 | docs/requirements/issue-483-small-model-fallback.md | 2026-09-30 (this pull request) | rust/tests/unit/issue_483_small_model_fallback.rs | not yet confirmed |

Requirement mapping:

- R483-1 — off by default, loaded only on explicit user enable
  (`off_by_default_and_nothing_loads`; `SmallModelOptions::default`).
- R483-2 — only hardware-fitting models displayed
  (`only_hardware_fitting_models_are_shown_sorted_by_public_rating`).
- R483-3 — sorted by public ratings (same test; rating floor + sort).
- R483-4 — on-demand download only, nothing packaged
  (`nothing_is_packaged_or_downloaded_in_advance`; every seed row
  `packaged false`; module performs no I/O).
- R483-5 — model selects the best match from offered options,
  unit-tested (`the_model_selects_the_best_match_from_offered_options`).
- R483-6 — formal first, LLMs never at the steering wheel
  (`the_proposal_is_advisory_and_the_formal_rules_hold_the_pen`;
  `confirm_proposal` gate).
