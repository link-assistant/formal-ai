---
bump: minor
---

### Added
- Context-aware concept lookup for "what is X in Y" style prompts in English, Russian, Hindi, and Chinese: the solver now models a definition query as `(concept, optional context)` and ranks records whose `contexts` list matches the user-supplied context phrase (issue #20). The trigger prompt `что такое iir в ml`, which previously returned `intent: unknown`, now resolves to `concept_lookup_in_context` with a localised response template per language.
- New `context_delimiter` pattern kind in `data/seed/prompt-patterns.lino` describing how to split "concept" from "context" per language (` in `, ` for ` for English; ` в `, ` для ` for Russian; ` में `, ` के लिए ` for Hindi; `中`, `中的`, `领域的` for Chinese). Hindi and Chinese also place context before the concept, so the ranker retries with swapped halves.
- New `contexts` field on `ConceptRecord` (Links Notation `context "..."` line) listing applicable domains in every supported language; the IIR seed entry covers signal processing and machine-learning aliases (`ml`, `машинное обучение`, `मशीन लर्निंग`, `机器学习`, `digital signal processing`, etc.).
- New `concept_lookup_in_context` intent with localised response templates in `data/seed/multilingual-responses.lino` (`В контексте «{context}»…`, `In the context of {context}…`, `{context} के संदर्भ में…`, `在「{context}」的语境下…`).
- Append-only event-log debug surface for the concept handler: every lookup emits `concept_lookup:request`, `concept_lookup:context` (when a context is parsed), and one of `concept_lookup:hit` / `concept_lookup:miss` / `concept_lookup:context-match` / `concept_lookup:context-mismatch`. The events are exposed through `evidence_links` in `--format chat` and `--format responses`, satisfying the verbose-trace requirement from the issue without a new flag.
- `docs/case-studies/issue-20/` with raw GitHub data, timeline, root-cause analysis, comparison against Wikipedia/Wikidata/schema.org disambiguation, and per-requirement solution plan.

### Changed
- `concepts.rs` now exposes `ConceptQuery` / `ConceptLookup` and the context-aware `extract_concept_query` + `lookup_concept_query`; the prior single-string `extract_concept_term` / `lookup_concept` shims have been removed.
- `src/web/formal_ai_worker.js` mirrors the Rust pipeline: it parses the context delimiter, emits the same `concept_lookup:*` evidence labels, and renders the in-context template via the seed-loaded `MULTILINGUAL_ANSWERS["concept_lookup_in_context"]` table.
- `tests/unit/mvp/multilingual.rs` pins down the four-language variants of the trigger prompt and the evidence-link debug trail; `src/solver_helpers.rs` gained unit tests for context delimiter parsing across the four languages.
