---
bump: minor
---

### Added
- Issue #103: New `tests/unit/specification/prompt_variations.rs` test module with 5-10 input variations per category (greetings, farewells, identity, clarification, concept lookups, capabilities, hello-world, basic math, refusal, idioms) translated across English, Russian, Hindi, and Chinese. The module ships generalized helpers (`assert_intent_for_each`, `assert_language_for_each`, `assert_answer_contains_for_each`, etc.) so per-language matrices stay declarative.
- Issue #103: New `ARCHITECTURE.md` describing the evolving architecture — context assembly, Links Notation translation, Wikidata P-id/Q-id formalization, temperature-driven interpretation selection, doublets-rs/doublets-web persistence, internet-as-public-database with local cache, and the five transformation-rule shapes (data rules, Rust handlers, JS handlers, dynamic compilation, natural-language skills).
- Issue #103: New `docs/case-studies/issue-103/` case study folder with collected raw data, competitor-test research, and the holistic plan that ties R129-R136 together.
- Issue #103: Added deterministic solver handlers for summarization, brainstorming, factual Q&A with Wikidata anchors, multi-turn coreference, and roleplay so the competitor-derived prompt categories run as active regression tests.

### Changed
- Issue #103: Updated `VISION.md` with a new "Formalization And Temperature" section, expanded "Computation Model" coverage of the five transformation-rule shapes, and meaning-and-identity language tying together natural-language ↔ programming-language translation via Links Notation.
- Issue #103: Updated `REQUIREMENTS.md` with the new R129-R136 entries and the "Issue #103 Test-Matrix And Architecture" matrix that traces each requirement to its enforcing test or document.
- Issue #103: Updated `tests/unit/docs_requirements.rs` to pin the documentation surface for issue #103 alongside the existing issue #12 and issue #16 traceability tests.
