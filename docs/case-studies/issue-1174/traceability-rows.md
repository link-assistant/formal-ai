# Issue #1174 Traceability Rows

Rows for the shard `docs/requirements/issue-1174-text-transform-formal-versions.md`,
kept here for the maintainer to merge into
`docs/requirements-traceability.md` (the shared table is not edited by this
change). Honesty rules follow the table's own header: `not yet confirmed`
means no manual confirmation has been recorded for this shard.

| ID | Shard | Delivered | Automated test | Manual confirmation |
| --- | --- | --- | --- | --- |
| R1174-1 | docs/requirements/issue-1174-text-transform-formal-versions.md | 2026-09-30 (this pull request) — free-text summarization under a ~30% bound that can never echo the input; no-text requests declined to the seeded-topic handlers | rust/tests/unit/issue_1174_text_transform.rs | not yet confirmed |
| R1174-2 | docs/requirements/issue-1174-text-transform-formal-versions.md | 2026-09-30 (this pull request) — register rewriting with casing kept and each substitution logged; already-formal texts reported honestly | rust/tests/unit/issue_1174_text_transform.rs | not yet confirmed |
| R1174-3 | docs/requirements/issue-1174-text-transform-formal-versions.md | 2026-09-30 (this pull request) — subject-verb re-agreement and numeral-noun pluralization over exceptions and uncountables, each fired rule named; clean texts reported honestly | rust/tests/unit/issue_1174_text_transform.rs | not yet confirmed |
| R1174-4 | docs/requirements/issue-1174-text-transform-formal-versions.md | 2026-09-30 (this pull request) — Conventional Commits composition from the described change; missing slots named, not fabricated | rust/tests/unit/issue_1174_text_transform.rs | not yet confirmed |
| R1174-5 | docs/requirements/issue-1174-text-transform-formal-versions.md | 2026-09-30 (this pull request) — email composition filling the seeded part frames in declared order, optional parts on cue only | rust/tests/unit/issue_1174_text_transform.rs | not yet confirmed |
| R1174-6 | docs/requirements/issue-1174-text-transform-formal-versions.md | 2026-09-30 (this pull request) — free-sentence translation word by word with seeded lemmas, dropped function words logged, unknown words named, honest gap when nothing resolves | rust/tests/unit/issue_1174_text_transform.rs | not yet confirmed |
| R1174-7 | docs/requirements/issue-1174-text-transform-formal-versions.md | 2026-09-30 (this pull request) — all vocabulary and response prose in seed data with five-language parity; net-zero unresolved closure tokens; literal-predicate count stays at 536 | rust/tests/unit/issue_1174_text_transform.rs | not yet confirmed |
| R1174-8 | docs/requirements/issue-1174-text-transform-formal-versions.md | 2026-09-30 (this pull request) — handler-level tests pinning every probe and honest boundary; dispatch wiring and test registration owned by the #1175 routing work in the same pull request | rust/tests/unit/issue_1174_text_transform.rs | not yet confirmed |
