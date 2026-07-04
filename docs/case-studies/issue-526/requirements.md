# Issue 526 Requirements

| ID | Requirement | Verification |
| --- | --- | --- |
| R526-1 | The translation quality contract is round-trip survival: source -> meta -> target -> meta -> source must preserve meaning. | `tests/unit/specification/translation_round_trip.rs` and `tests/unit/specification/translation_via_links.rs`. |
| R526-2 | Translation to the meta language from any supported language must lose no data for seeded surfaces. | `supported_language_surfaces_survive_meta_language_round_trip`. |
| R526-3 | Every supported natural-language pair must be tested through the meta language. | `every_supported_language_pair_round_trips_via_meta_language`. |
| R526-4 | English, Russian, Hindi, and Chinese must be represented as equal supported-language sources, not only as English targets. | The apple matrix iterates all four language slugs as sources and targets. |
| R526-5 | Rust <-> JavaScript code translation must preserve the same code meaning when routed through the meta representation. | `rust_javascript_code_translation_round_trips_through_code_meaning`. |
| R526-6 | The implementation must not add direct pair-specific natural-language translation paths that bypass `TranslationPipeline`. | The new tests call `translate_via_default_pipeline`; architecture docs state the meta-language-only contract. |
| R526-7 | Vision, requirements, roadmap, architecture, and contributor guidance must describe the round-trip quality rule. | Guarded by `tests/unit/docs_requirements_issue_526.rs`. |
| R526-8 | The case-study directory must preserve issue data, PR data, online research, requirements, and solution plans. | Guarded by `tests/unit/docs_requirements_issue_526.rs`. |

## Open Scope

This PR does not claim universal machine-translation quality. It adds a
repository regression contract for the supported, seeded surfaces and the
currently implemented simple add-function code meaning. Wider vocabulary and
program semantics should extend the same matrix by adding more seeded meanings,
not by bypassing the meta language.
