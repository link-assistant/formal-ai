---
bump: minor
---

### Added
- Issue #556: generalized the response-language follow-up beyond repository lookups to the whole class of "re-answer the previous request in another language" turns. A bare follow-up such as "I do not understand English, write in Russian" now replays the previous request through the entire solver with the target language forced at a single detection seam, so capabilities, identity, project lookups, and other localizable answers all re-render in any seeded language (English, Russian, Hindi, Chinese) — and reverse back to English on request.
- Grounded the follow-up in a machine-readable meta-algorithm recipe (`data/meta/response-language-followup-recipe.lino`), pinned by `tests/unit/specification/response_language_meta_algorithm.rs`, so the eight recursive-reasoning steps, seed roles, Wikidata groundings, handler functions, forced-language seam, and Rust↔JS parity targets can never silently drift from the live source. Documented in `docs/meta-algorithm.md`.
- Added round-trip translation tests (issue #526) proving English↔Russian/Hindi/Chinese vocabulary survives a source→meta-language→target→source cycle with both meaning and surface preserved.

### Fixed
- Issue #556: repository lookup language-change follow-ups now rerender the previous GitHub lookup in the requested seeded response language instead of falling through to unknown.
