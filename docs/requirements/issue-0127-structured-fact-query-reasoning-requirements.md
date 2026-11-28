## Issue #127 Structured Fact-Query Reasoning Requirements

Issue [#127](https://github.com/link-assistant/formal-ai/issues/127) reports
that "столица россии" returned the unknown-intent fallback. The maintainer
asked for a real-time reasoning pipeline backed by Wikipedia/Wikidata/Wiktionary
instead of a hardcoded fact database. On a cache miss, the browser worker
and native solver resolve publisher claims from Wikidata. The historical
responses are opt-in fixtures under `rust/tests/fixtures/fact-claims`.

| ID | Requirement | Status |
| --- | --- | --- |
| R173 | Multilingual factual prompts (capital, population, currency, official language, continent, area, head of state, head of government) must route to a typed `fact_query` intent instead of `unknown`. | Implemented by the browser fact-query parser and the Rust live factual QA handler. The grammar recognizes typed relations. The earlier substring match incorrectly accepted the alias "us" inside "australia". Both runtimes apply the word-boundary correction from issue #1172 and verify publisher matches before answering. |
| R174 | Recognized facts must retain structured `(relation, subject_qid, value_qid, subject_label, value_label)` evidence. | Implemented. The word-boundary correction from issue #1172 fixes the substring defect where "us" matched inside "australia". Issue #1613 removed automatic pre-warming. Ranked Wikidata claims provide the subject and value. `rust/tests/web/issue-0127-fact-captures.test.mjs` explicitly replays the historical triples. |
| R175 | The browser must cache verified fact queries for one week using a per-language key. | Implemented. `factCachePut` and `factCacheGet` retain verified results under relation, subject and language. A fresh installation starts with an empty fact cache. Covered by `rust/tests/web/issue-0127-fact-query-cache.test.mjs`. |
| R176 | The user must be able to force a fresh resolution by adding a fresh marker to the prompt in any supported language. | Implemented by `FORCE_FRESH_MARKERS` covering English, Russian, Hindi, and Chinese phrasings; recorded as a `fact_query:force_fresh` event. |
| R177 | On a cache miss, the runtime must resolve the requested subject and property using Wikidata and the user’s language. | Implemented. The word-boundary correction from issue #1172 fixes the substring defect where "us" matched inside "australia". Both runtimes verify exact subject labels or aliases and ranked statements. The lexical relation supplies the property. Ambiguous subjects and missing statements leave the route unresolved. |
| R178 | Every step of the fact-query pipeline must be appended to memory as a `fact_query:*` event so the reasoning trace is reconstructible offline. | Implemented. Browser trace records the request, relation, subject, cache decision, Wikidata requests and response. The native event log records the parsed question, subject and value Q-ids and source capture identity. |
| R179 | The Rust offline solver must emit the same `fact_query:*` evidence shape as the browser worker for seeded facts so memory inspection works uniformly across stacks. | Implemented. Rust and browser results record the question relation, resolved subject and value Q-ids, and source capture provenance. Offline Rust can reuse verified runtime captures. `rust/tests/unit/specification/prompt_variations_facts.rs` supplies replay input through a test transport. |
| R180 | Tests must cover multiple phrasings per country and at least four supported languages so regressions in prompt parsing surface quickly. | Implemented. `rust/tests/unit/specification/prompt_variations_facts.rs` covers the native phrasings. The fact-query pipeline cases in `rust/tests/e2e/tests/multilingual-chat.spec.js` use explicit Wikidata responses for four languages. |
