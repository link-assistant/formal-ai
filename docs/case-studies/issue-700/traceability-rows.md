# Issue #700 Traceability Rows

Rows for a shard `docs/requirements/issue-700-si-unit-dimensions.md`,
kept here for the maintainer to merge into
`docs/requirements-traceability.md` (the shared table is not edited by
this change).

| ID | Shard | Delivered | Automated test | Manual confirmation |
| --- | --- | --- | --- | --- |
| R700-1 | docs/requirements/issue-700-si-unit-dimensions.md | 2026-09-30 (this pull request) | rust/tests/unit/issue_700_si_units.rs | not yet confirmed |
| R700-2 | docs/requirements/issue-700-si-unit-dimensions.md | 2026-09-30 (this pull request) | rust/tests/unit/issue_700_si_units.rs | not yet confirmed |
| R700-3 | docs/requirements/issue-700-si-unit-dimensions.md | 2026-09-30 (this pull request) | rust/tests/unit/issue_700_si_units.rs | not yet confirmed |
| R700-4 | docs/requirements/issue-700-si-unit-dimensions.md | 2026-09-30 (this pull request) | rust/tests/unit/issue_700_si_units.rs | not yet confirmed |
| R700-5 | docs/requirements/issue-700-si-unit-dimensions.md | 2026-09-30 (this pull request) | rust/tests/unit/issue_700_si_units.rs | not yet confirmed |

Requirement mapping:

- R700-1 — dimension algebra over the seven SI base dimensions
  (parse/render/multiply/inverse; `Dimension` tests).
- R700-2 — conversion through SI base units with exact rational
  arithmetic, hp→W included (the 40-row table, exact-rational checks).
- R700-3 — unknown units fail honestly with a named gap event
  (`unknown_units_are_named_gaps_and_log_events`).
- R700-4 — multilingual: surfaces in en/ru/hi/zh resolve and prompts in
  four languages convert (`surfaces_in_four_languages_resolve…`,
  `multilingual_prompts_resolve_and_convert`).
- R700-5 — gap matrix and drafted upstream filing for
  link-foundation/si-units, not filed (case-study README +
  `upstream-filing.md`).
