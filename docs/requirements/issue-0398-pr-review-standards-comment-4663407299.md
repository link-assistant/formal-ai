## Issue #398 PR Review Standards (comment 4663407299)

The PR [#399](https://github.com/link-assistant/formal-ai/pull/399) review of
2026-06-09 added data-quality standards for the meaning seed and its external
grounding. These standards govern the whole `data/` tree (and `src/`), are
applied through re-runnable migrations rather than manual edits, and each is
locked by a directory-walking CI check that names no individual file.

**Governance — latest requirement overrides any earlier one.** When two
requirements in this document conflict, the one with the higher `R` number
wins; the earlier requirement is superseded for the overlapping scope. Every CI
check listed below maps to exactly one requirement so a reviewer can trace a red
build back to the rule it protects.

| ID | Requirement | Status |
| --- | --- | --- |
| R278 | Semantic facets must use the native Links Notation `subject predicate` form (`notation word_surface`); empty-bodied colon redefinitions (`word_surface:`, `concept:` with no value) are banned across the whole `data/seed` tree. | Implemented by `scripts/migrate-empty-facet-fields.rs` (tree-wide collapse), the dual-form `parse_semantic_facets` in `src/seed/meanings.rs`, and the tree-walking CI guard `seed_lino_files_have_no_empty_redefinition_fields` in `tests/unit/data_files.rs`. |
| R279 | A `data/overrides/` layer must sit beside `data/cache/` with the same per-id structure; resolution is `(cache or live API) then overrides`; every override records why it exists; CI must fail when an override is redundant (the cache already carries its value), forcing removal. | Implemented by `data/overrides/` (mirroring `data/cache/`), `data/overrides/README.md`, `formal_ai::seed::resolve` in `src/seed/grounding_overrides.rs`, and the tree-walking CI suite `overrides_are_disciplined_and_non_redundant` / `overrides_layer_mirrors_the_cache_directory_structure` in `tests/unit/overrides.rs`. |
| R280 | Every seed/grounding data-quality rule must be enforced by a directory-walking CI check with no hard-coded filename, so a new file cannot bypass a rule. | Implemented; R278 and R279 guards both walk their whole trees (`seed_lino_paths()`, `WalkDir` over `data/overrides`) and assert on relative paths rather than fixed names. |
| R281 | Tree-wide fixes must be applied through re-runnable migration/refresh scripts, not one-off manual edits, so the transform is auditable and idempotent. | Implemented by `scripts/migrate-empty-facet-fields.rs` and `scripts/clean-seed-readability.rs`, both std-only and lossless, which also regenerate the embedded browser worker fallback. |
| R282 | Meanings must be grounded in real external sources (Wikidata / Wiktionary / WordNet); per-language surfaces must come from those sources rather than being hand-typed, and the grounding closure must verify external evidence, not only local `defined-by` resolution. | Partially implemented: source-grounded meanings carry `grounded-in <Qid>` links whose recursive closure is verified against checked-in cache records by `wikidata_cache_records_cover_recursive_grounding_closure` and `seed_and_source_wikidata_ids_have_checked_in_cache_records`. Expanding grounding from the current core set to every meaning is tracked as ongoing source-import work in `docs/case-studies/issue-398/README.md`; the override layer (R279) is the mechanism for recording source gaps in the meantime. |
| R283 | The standards introduced by this review must be recorded in `REQUIREMENTS.md` under the latest-overrides-earlier governance, with each CI check mapped to a requirement. | Implemented by this section; the mapping column names the enforcing test for R278-R281. |
