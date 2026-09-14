## Issue #207 Natural Translation Pipeline

Issue [#207](https://github.com/link-assistant/formal-ai/issues/207)
followed up on the translation work done for issue #190. The maintainer
reported three remaining problems with the prompt `Переведи "как у тебя
дела?" на английский.`: the response body still looked like a robotic
`meaning: … / surface (…): …` block, the translation discarded the
source fragment's lowercase casing (returning `How are you?` instead of
`how are you?`), and only a single hardcoded meaning id resolved to a
localized surface form — every other prompt fell back to a `[en] …`
placeholder. The maintainer asked for a single
**formalize → meaning → deformalize** pipeline that handles every
registered meaning, preserves source-side formatting, and leaves room
for Wikipedia / Wikidata / Wiktionary enrichment.

| ID | Requirement | Status |
| --- | --- | --- |
| R213 | Translation responses must read like natural conversation: the answer body is the deformalized target surface (quoted when the user quoted the source), not a multi-line `meaning: …` / `surface (…)` template. | Implemented by `try_translation` in `src/solver_handlers/mod.rs` and `tryTranslation` in `src/web/formal_ai_worker.js`. The meaning id, source language, and target language stay available through `evidence_links` so the trace remains inspectable. Covered by `tests/unit/specification/translation_via_links.rs::russian_translate_how_are_you_prompt_returns_english_surface`. |
| R214 | Translations must preserve the source fragment's leading capitalization and terminal punctuation. A lowercase, unpunctuated source must produce a lowercase, unpunctuated target. | Implemented by `match_source_formatting` in `src/translation/formatting.rs` and `matchSourceFormatting` in `src/web/formal_ai_worker.js`. Covered by `tests/unit/specification/translation_via_links.rs::russian_translate_how_are_you_prompt_returns_english_surface`, `russian_capitalized_how_are_you_keeps_target_capitalization`, and `natural_translation_drops_terminal_when_source_has_none`. |
| R215 | The pipeline must translate any surface, not only a hand-written subset, by routing through Wiktionary translation tables and Wikidata sense joins. Raw HTTP responses are cached on disk so unit tests run offline against real data. | Implemented by `TranslationPipeline` in `src/translation/pipeline.rs`, `Wiktionary` in `src/translation/wiktionary.rs`, `Wikidata` in `src/translation/wikidata.rs`, and `CachedHttpClient` in `src/translation/cache.rs`. Cached responses live under `data/translation-cache/`; integration runs can refresh them with `FORMAL_AI_LIVE_API=1`. Covered by `tests/unit/specification/translation_via_links.rs::translation_meaning_registry_covers_extended_phrases` (eight unrelated pairs across en / ru / hi / zh) plus per-module unit tests. The online enrichment design is documented in `ARCHITECTURE.md` section 10 and `docs/case-studies/issue-207/raw-data/online-research.md`. |
