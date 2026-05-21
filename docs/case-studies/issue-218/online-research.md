# Online Research — Issue #218 (Translation umbrella)

## Wiktionary translation tables

- Reference page: <https://en.wiktionary.org/wiki/apple>
- Translation table (English → Russian) lists `яблоко (jábloko) n` and links
  to the corresponding lexeme. Cached locally under
  `data/translation-cache/wiktionary-en-apple.json`.
- Reverse direction page: <https://ru.wiktionary.org/wiki/яблоко>
  Cached locally under `data/translation-cache/wiktionary-ru-яблоко.json`.

The translation pipeline already supports parsing these pages — the
fault from issue #217 was simply a missing cache entry for `яблоко`,
because `examples/refresh_translation_cache.rs` only seeded the demo
phrase set used in `tests/unit/specification/translation_via_links.rs`.

## Wikidata lexeme join

- `apple` → lexeme `L3257` (Q89 fruit, Q11004 species)
- `яблоко` → lexeme `L37724`
- Both lexemes share the same Wikidata Q-item (`Q89`) so the
  `MeaningId` synthesised from `wikidata-sense:` resolves identically
  in either direction. SPARQL query template:

  ```
  SELECT ?lemma ?language WHERE {
    wd:Q89 wdt:P5137 ?sense .
    ?lexeme ontolex:sense ?sense ;
            wikibase:lemma ?lemma ;
            dct:language ?lang .
    ?lang wdt:P424 ?language .
  }
  ```

## Prior art in the repository

- PR #208 introduced the Wiktionary + Wikidata pipeline and the
  `CachedHttpClient` covering the `data/translation-cache/` directory.
- PR #211 (issue #210) added the precedence fix for explicit
  translation prompts and a compositional ru→en fallback for short
  phrases. This PR extends that work to cover single-noun prompts and
  unquoted translation requests.

## Related upstream/ecosystem references

- MediaWiki API (Wiktionary): <https://en.wiktionary.org/api/rest_v1/>
- Wikidata Lexeme structure: <https://www.wikidata.org/wiki/Wikidata:Lexicographical_data>
- ICU CLDR list of language codes: <https://github.com/unicode-org/cldr-json>

No upstream-issue reports were filed — both bugs trace to gaps in our
local cache and surface-extraction logic rather than upstream defects.
