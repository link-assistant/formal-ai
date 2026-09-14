## Issue #526 Translation Quality Test

Issue [#526](https://github.com/link-assistant/formal-ai/issues/526)
defines the translation quality rule as round-trip survival. A translation is
not sufficient merely because it renders a plausible target string; it must
formalize into the meta language, deformalize into the target language, and
return through the same meta-language meaning to the original source surface.

| ID | Requirement | Status |
| --- | --- | --- |
| R526-1 | Translation quality must be measured by round-trip survival: source -> meta -> target -> meta -> source preserves meaning and source surface. | Implemented by `tests/unit/specification/translation_round_trip.rs`, which asserts both `MeaningId` equality and final surface equality. |
| R526-2 | Translation to the meta language from every supported source language must be lossless for seeded surfaces. | Implemented by `supported_language_surfaces_survive_meta_language_round_trip`, covering `apple`, `яблоко`, `सेब`, and `苹果` through language-to-meta-to-same-language projection. |
| R526-3 | Every supported natural-language pair must translate through the shared meta-language meaning rather than a direct pair-only path. | Implemented by `every_supported_language_pair_round_trips_via_meta_language`, which covers all directed pairs across en, ru, hi, and zh with `translate_via_default_pipeline`. |
| R526-4 | Code translation must route through a code meta language (`source -> CodeMeaning -> target`), never a direct `(source, target)` table, and preserve one shared meaning across the round trip — Rust <-> JavaScript included. | Implemented by `src/solver_helpers/code.rs::{CodeMeaning, formalize_code_meaning, render_code_meaning, translate_program, normalize_code_meaning}`, verified by `translation_via_links.rs::rust_javascript_code_translation_round_trips_through_code_meaning` and `code_translation_routes_through_the_code_meta_language_not_direct_pairs` (which proves never-hardcoded pairs such as Python -> JavaScript and Rust -> Go translate through one `meaning:` link). |
| R526-5 | The architecture must state that translation goes through the meta language and that direct translation bypasses are not the quality path. | Implemented in `VISION.md`, `ARCHITECTURE.md` section 10, `ROADMAP.md`, and `CONTRIBUTING.md`. |
| R526-6 | Issue data, online research, requirements, and solution planning must be compiled under `docs/case-studies/issue-526`. | Implemented by `docs/case-studies/issue-526/{README,requirements,solution-plans}.md`, `raw-data/online-research.md`, and raw GitHub snapshots. |
