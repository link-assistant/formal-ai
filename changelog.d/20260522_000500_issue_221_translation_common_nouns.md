---
bump: minor
---

### Fixed
- Stop faking translation for common nouns: the demo and the Rust pipeline
  now return real Wiktionary/Wikidata-backed translations for every noun
  reachable through the seeded raw-API-response cache, in both directions,
  replacing the `[ru]` / `[en]` placeholders (issue #221).
  `Переведи "помидор" на английский.` returns `tomato`,
  `translate "carrot" to russian` returns `морковь`, and the unquoted
  variants resolve through the same path.
- Wikidata SPARQL lexeme joins now restrict P5137 matches by
  `wikibase:lexicalCategory` so polysemous surfaces like `water` no
  longer cross noun ↔ verb boundaries (`water` resolves to `вода`,
  not `поливать`).

### Added
- `data/seed/api-cache/*.lino` — verbatim Wikidata and Wiktionary API
  response bodies, stored in indented Links Notation with base64-encoded
  payloads (RFC 4648, 76-character chunks). Bundle is capped at 128
  records per semantic bucket (entities, properties, search, sparql,
  pages-per-language) and every file stays under 1500 lines; bodies that
  exceed that cap are split into deterministic `<bucket>-partN.lino`
  parts and re-joined at load time by URL. No pre-extracted dictionary
  lives in the repo — only raw API responses that the formalization
  pipeline replays.
- `build.rs` enumerates `data/seed/api-cache/*.lino` at compile time and
  emits `OUT_DIR/seed_bundle_files.rs`. `src/translation/cache.rs`
  pulls the generated list with
  `include!(concat!(env!("OUT_DIR"), "/seed_bundle_files.rs"))` so new
  part files ship automatically without per-file `include_str!` edits.
- `FORMAL_AI_TRANSLATION_DEBUG=1` enables stage-by-stage stderr tracing
  through the translation pipeline (closes the "Future work" item from
  issue #218).
- Live Wiktionary fallback in `src/web/formal_ai_worker.js`
  (`liveWiktionaryTranslate`): the browser worker now hits
  `*.wiktionary.org/w/api.php?action=parse&...&origin=*` directly for any
  surface that is not already covered by the seed bundle, follows
  `{{see translation subpage|...}}` to the `/translations` subpage when
  present, and extracts the `{{tt+|<lang>|...}}` / `{{t|...}}` template
  payload. Mobile-friendly: no offline dictionary is bundled into the
  worker, the seed bundle stays small, and the MediaWiki action API is
  CORS-friendly through `origin=*`.

### Changed
- `src/translation/cache.rs` reorganises the on-disk accelerator and
  seed bundle by **semantic identity** rather than URL hash:
  - `data/wikidata-cache/{search,entities,query,sparql}/` for Wikidata
    `wbsearchentities` / `wbgetentities` / `action=query` / SPARQL.
  - `data/wiktionary-cache/<lang>/` keyed by page title.
  - `data/http-cache/misc/` (URL-hash) for anything else.
  The `data/wikidata-cache/`, `data/wiktionary-cache/` and
  `data/http-cache/` trees are gitignored — they are local accelerators
  written by `FORMAL_AI_LIVE_API=1` runs. The committed offline source of
  truth is the seed bundle under `data/seed/api-cache/`.
  `CachedHttpClient::get` consults seed bundle → on-disk accelerator →
  live transport in that order, so a clean checkout reproduces every
  test deterministically.
- `examples/refresh_translation_cache.rs` drives the full pipeline
  against a curated 128-noun seed list, populates the on-disk
  accelerator, then re-bundles it into the committed `.lino` seed files,
  splitting oversize records into `<bucket>-partN.lino` parts and
  removing stale parts that no longer back any record.
- `src/translation/wikidata.rs` adds the `wikibase:lexicalCategory`
  filter to the lexeme-join SPARQL so the polysemy edge case described
  in `docs/case-studies/issue-221/online-research.md` no longer crosses
  part-of-speech boundaries.
- `src/web/formal_ai_worker.js` no longer ships an offline translation
  dictionary. Translation flows through the existing meaning registry
  first, then falls back to the live Wiktionary fetch above. Removed:
  the `TRANSLATION_DICTIONARY` and `lookupDictionary` plumbing plus the
  `extractTranslations` parser in `src/web/seed_loader.js`.
- `tests/e2e/tests/issue-221.spec.js` exercises the live worker path
  end-to-end (quoted RU→EN, quoted EN→RU, unquoted prompts, Russian
  inflected forms via MediaWiki redirect, round-trip stability).
- `tests/unit/docs_requirements.rs` exempts the new seed-bundle and
  cache roots when scanning for deferred labels — the bundled wikitext
  bodies contain ISO 639-3 language codes that would otherwise trip the
  scanner.

Case study, raw reproductions, and external references live in
`docs/case-studies/issue-221/`.
