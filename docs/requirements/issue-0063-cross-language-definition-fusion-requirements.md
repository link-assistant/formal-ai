## Issue #63 Cross-Language Definition Fusion Requirements

Issue [#63](https://github.com/link-assistant/formal-ai/issues/63) asks the
assistant to be better than a single Wikipedia language edition by combining
definitions of the same concept across translations, without neural-network
learning. The first implemented step works over the reviewable seed concept
records: every localized block for the same concept anchor is treated as a
source fragment, repeated facts are deterministically deduplicated, and the
answer keeps language/source evidence visible.

| ID | Requirement | Status |
| --- | --- | --- |
| R150 | Recognize requests to merge or combine Wikipedia definitions/translations for a concept. | Implemented by the `definition_merge` method in `src/definition_merge.rs` (migrated out of `solver_handlers/` by #699), mirrored by `tryDefinitionMerge` in `src/web/formal_ai_worker.js`. |
| R151 | Merge only definitions that belong to the same resolved concept anchor, preferring seeded Wikidata Q-ID records when available. | Implemented by resolving the requested term through `lookup_concept_query` before collecting localized fragments; the answer and evidence keep the shared `wikidata:` link. |
| R152 | Preserve source languages and citations for every contributing definition fragment. | Implemented by `definition_merge:language:*` and `source:http:*` evidence links in Rust, matching source-language evidence chips in the browser, plus the user-facing `Source languages:` and `Sources:` sections. |
| R153 | Deduplicate repeated facts deterministically instead of concatenating every source verbatim. | Implemented by sentence-level normalized fact keys in `merged_definition_facts` / `mergedDefinitionFacts`; the output is stable for the same seed data. |
| R154 | Cover cross-language definition fusion with 10-20 self-explanatory examples that assert specific behavior rather than matching full answer markdown. | Implemented by `tests/unit/specification/definition_fusion.rs` and the Playwright regression `merged Wikipedia definitions combine localized seed summaries` in `tests/e2e/tests/multilingual.spec.js`. |
| R155 | Let users choose whether plain definition prompts like `What is IIR?` automatically use definition fusion. | Implemented by `SolverConfig::definition_fusion_by_default`, `FORMAL_AI_DEFINITION_FUSION`, the CLI `--definition-fusion` option, and the browser settings control persisted through `formal-ai.preferences.v1`. |
