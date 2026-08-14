## Issue #398 Recursive Semantic Meta-Language

Issue [#398](https://github.com/link-assistant/formal-ai/issues/398) requires
the meaning seed to move from English-only descriptions toward a recursive
semantic meta-language where meanings describe meanings, source evidence is
cached and inspectable, and notation, annotation, denotation, and connotation are
represented as meaning links. This PR implements the seed/parser foundation and
records the larger source-import/backfill plan in the case study.

| ID | Requirement | Status |
| --- | --- | --- |
| R266 | Issue #398 research, issue metadata, PR metadata, comments, reviews, and online source analysis must be preserved under `docs/case-studies/issue-398`. | Implemented with `docs/case-studies/issue-398/README.md` and `raw-data/*`, including the online source survey for Wikidata, Wiktionary/Wikipedia dumps, WordNet, SKOS, and OntoLex-Lemon. |
| R267 | Meanings must remain recursively describable by other meanings instead of relying only on prose descriptions. | Preserved and extended by the existing `defined_by` closure/root-reachability tests plus the new meaning-to-meaning semantic facet parser. |
| R268 | Notation, annotation, denotation, and connotation must be represented as seed meanings, not hardcoded Rust vocabulary. | Implemented by `data/seed/meanings-semantic-meta.lino`; the parser recognizes only the generic `facet` container and resolves facet kind/target strings through the lexicon. |
| R269 | The root `link` meaning must declare the required semantic facets as meaning references. | Implemented in `data/seed/meanings-ontology.lino` and verified by `root_link_declares_the_required_semantic_facets`. |
| R270 | Semantic facet blocks must parse as meaning references and resolve through the lexicon. | Implemented by `SemanticFacet`, `Meaning::semantic_facet_targets`, and `Lexicon::semantic_facet_meanings`; verified by `semantic_facet_blocks_are_parsed_as_meaning_references`. |
| R271 | External knowledge sources and cached source responses must become first-class semantic concepts so future imports can preserve provenance. | Seeded as `external_knowledge_source` and `cached_source_response` meanings; full source-response importers are tracked as follow-up work in the case study. |
| R272 | The semantic meta-language must stay small at startup while allowing on-demand expansion from external corpora. | Implemented by adding only the compact facet vocabulary to `data/seed/`; the case study recommends chunked `.lino` source-response caches for large corpora. |
| R273 | Vision documentation must reflect recursive meaning descriptions and semantic facets. | Implemented in `VISION.md` under "Meaning And Identity". |
| R274 | User-facing changes must be captured for release notes. | Implemented by the issue #398 fragment `20260606_201500_issue_398_semantic_facets.md` (collected into `CHANGELOG.md` at release). |
| R275 | The self-defining Links-Theory root draft from PR feedback must be executable seed data, not only prose in a comment. | Implemented by `data/seed/meanings-links-root.lino`, embedded in `src/seed/embedded.rs` and `tests/source/seed/embedded.rs`, with `semantic_root` tests for root terms and closure. |
| R276 | Self-referential primitives must be represented as meaning-backed self-equations. | Implemented by the `self_equation` semantic facet kind plus `type`, `not`, and `same` facet links, covered by `self_equations_are_explicit_semantic_facets`. |
| R277 | Ambiguous symbols must split into one-symbol-one-meaning records. | Implemented by `one_symbol_one_meaning`, `sense_split`, `bank_river`, and `bank_money`; `ambiguous_bank_surface_is_split_into_distinct_symbols` asserts there is no ambiguous bare `bank` meaning. |
