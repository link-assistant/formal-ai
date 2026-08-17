## Issue #127 Structured Fact-Query Reasoning Requirements

Issue [#127](https://github.com/link-assistant/formal-ai/issues/127) reports
that "столица россии" returned the unknown-intent fallback. The maintainer
asked for a real-time reasoning pipeline backed by Wikipedia/Wikidata/Wiktionary
instead of a hardcoded fact database. The seed entries are now treated as a
pre-warmed cache, and the JS worker reaches the live Wikidata API on cache
miss. Every step is recorded so memory holds the structured trace.

| ID | Requirement | Status |
| --- | --- | --- |
| R173 | Multilingual factual prompts (capital, population, currency, official language, continent, area, head of state, head of government) must route to a typed `fact_query` intent instead of `unknown`. | Implemented by `parseFactQuestion` + `tryFactQuery` in `src/web/formal_ai_worker.js`, mirrored by `try_fact_lookup` in `src/solver_handlers/benchmark_prompts.rs`. |
| R174 | The seed `data/seed/facts.lino` must declare structured `(relation, subject_qid, value_qid, subject_label, value_label)` triples for the pre-warmed countries. | Implemented for Russia, Japan, France, Germany, China, India, the United States, the United Kingdom, and Brazil, each with localized labels for en/ru/hi/zh. |
| R175 | Recognized fact-query questions must be cached for one week with a per-language key, and the cache must be pre-warmed from the seed at worker startup. | Implemented by `factCachePut` / `factCacheGet` (1-week TTL) and `warmFactCacheFromSeed` in `src/web/formal_ai_worker.js`. |
| R176 | The user must be able to force a fresh resolution by adding a fresh marker to the prompt in any supported language. | Implemented by `FORCE_FRESH_MARKERS` covering English, Russian, Hindi, and Chinese phrasings; recorded as a `fact_query:force_fresh` event. |
| R177 | On cache miss the browser worker must resolve the prompt against the live Wikidata API (`wbsearchentities` + `wbgetentities`) using the property table `FACT_RELATIONS` and the user's language. | Implemented by `resolveFactQueryViaWikidata` in `src/web/formal_ai_worker.js`. |
| R178 | Every step of the fact-query pipeline must be appended to memory as a `fact_query:*` event so the reasoning trace is reconstructible offline. | Implemented by the structured `fact_query:request`, `fact_query:relation`, `fact_query:subject`, `fact_query:cache:*`, `fact_query:wikidata:*`, and `fact_query:response` trace events emitted by the JS worker and (for the seed-cache path) the Rust solver. |
| R179 | The Rust offline solver must emit the same `fact_query:*` evidence shape as the browser worker for seeded facts so memory inspection works uniformly across stacks. | Implemented by extending `try_fact_lookup` to emit `fact_query:relation`, `fact_query:subject`, `fact_query:cache:hit:seed`, `fact_query:subject_qid`, and `fact_query:value_qid` events; formatter branches added in `src/event_log.rs::build_evidence_links`. |
| R180 | Tests must cover multiple phrasings per country and at least four supported languages so regressions in prompt parsing surface quickly. | Implemented by `CAPITAL_CASES` and the `capital_matrix_*` tests in `tests/unit/specification/prompt_variations.rs`, and by the parameterized `fact-query pipeline resolves` cases in `tests/e2e/tests/multilingual.spec.js`. |
