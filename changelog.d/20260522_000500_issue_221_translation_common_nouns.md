---
bump: minor
---

### Fixed
- Stop faking translation for common nouns: the demo and the Rust pipeline
  now return real Wiktionary/Wikidata-backed translations for every noun in
  the 128-entry seed set in both directions, replacing the `[ru]` / `[en]`
  placeholders (issue #221). `Переведи "помидор" на английский.` returns
  `tomato`, `translate "carrot" to russian` returns `морковь`, and the
  unquoted variants resolve through the same path.

### Added
- `data/seed/translations.lino` — the canonical 128-entry common-noun
  dictionary shared between the Rust pipeline (embedded via
  `include_str!`) and the browser worker (fetched through the standard
  seed loader). One file → one source of truth across CLI, browser, and
  tests; the previous `src/web/translation-dictionary.json` is gone.
- `src/translation/dictionary.rs` parses the file into a bidirectional
  `(language, surface) → entry` map plus a reverse
  `(target_lang, target_surface) → entry` map so a single en→ru entry
  covers both directions and the 128-entry cap stays meaningful.
- `FORMAL_AI_TRANSLATION_DEBUG=1` enables stage-by-stage stderr tracing
  through the translation pipeline (closes the "Future work" item from
  issue #218).
- Polysemy override table in `src/translation/pipeline.rs` rescues
  noun↔verb homographs that the Wikidata SPARQL lexeme join lands on the
  wrong side of (e.g. `milk → молоко` instead of `доить`, `water → вода`
  instead of `поливать`).

### Changed
- `examples/build_translation_dictionary.rs` now emits
  `data/seed/translations.lino` (≈1,300 lines, well under the 1,500-line
  Links Notation cap) instead of a JSON file. Inflection forms (Russian
  declensions, English plurals) are generated deterministically and
  stored on each record's `aliases` line.
- `src/translation/cache.rs` reorganises the file cache by **semantic
  data type** instead of URL hash:
  `wikidata-cache/{search,entities,query,sparql}/`,
  `wiktionary-cache/<lang>/`, `http-cache/misc/`. All three roots are
  gitignored — the cache is a local accelerator, not pre-seeded data —
  so the PR diff stays under 3,000 files. Other formalisation flows
  (entity resolution, fact lookup) reuse the same buckets.
- `examples/refresh_translation_cache.rs` seed list expanded so the
  offline cache covers the full demo vocabulary, writing into the new
  semantic-keyed cache root (`data/` by default).
- `tests/unit/docs_requirements.rs` exempts the new cache roots when
  scanning for deferred labels so raw Wiktionary bodies (which contain
  three-letter ISO 639-3 language codes that coincide with deferred-label
  vocabulary) don't trip the scanner.

Case study and provenance live in `docs/case-studies/issue-221/`.
