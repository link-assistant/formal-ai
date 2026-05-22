---
bump: minor
---

### Fixed
- Stop faking translation for common nouns: the demo and the Rust pipeline
  now return real Wiktionary/Wikidata-backed translations for every noun in
  the 71-word seed set in both directions, replacing the `[ru]` / `[en]`
  placeholders (issue #221). `Переведи "помидор" на английский.` returns
  `tomato`, `translate "carrot" to russian` returns `морковь`, and the
  unquoted variants resolve through the same path.

### Added
- `examples/build_translation_dictionary.rs` replays the cached
  Wiktionary + Wikidata pipeline against the seed list and emits
  `src/web/translation-dictionary.json` (414 entries, 655 aliases including
  Russian declensions and English plurals) which the browser worker loads
  during `init()` so the GitHub Pages demo answers without leaking the
  user's prompts to Wikimedia.
- `FORMAL_AI_TRANSLATION_DEBUG=1` enables stage-by-stage stderr tracing
  through the translation pipeline (closes the "Future work" item from
  issue #218).
- Polysemy override table in `src/translation/pipeline.rs` rescues
  noun↔verb homographs that the Wikidata SPARQL lexeme join lands on the
  wrong side of (e.g. `milk → молоко` instead of `доить`, `water → вода`
  instead of `поливать`).

### Changed
- `examples/refresh_translation_cache.rs` seed list expanded from one
  noun + greetings to 71 English nouns × 4 target languages + 71 Russian
  nouns × 1 target so the offline cache covers the full demo vocabulary.
- `tests/unit/docs_requirements.rs` skips `data/translation-cache/` when
  scanning for deferred labels so raw Wiktionary bodies (which contain
  three-letter ISO 639-3 language codes that coincide with deferred-label
  vocabulary) don't trip the scanner.

Case study and provenance live in `docs/case-studies/issue-221/`.
