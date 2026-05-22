# Issue 221 — Online Research Notes

External references collected while diagnosing the umbrella common-noun
translation gap reported in issue #221. The order mirrors the
investigation timeline; each reference was followed against live
endpoints before any code change.

## Wiktionary API

- **`action=parse&prop=wikitext`** — the pipeline pulls raw wikitext
  for every page rather than the rendered HTML. The wikitext is the
  only place where the `{{trans-top}}` / `{{перев-блок}}` sense
  blocks are stable, since rendered HTML strips template metadata.
- **CORS** — MediaWiki accepts `&origin=*` for anonymous read-only
  GETs across all `*.wiktionary.org` editions. This is what
  `liveWiktionaryTranslate` in `src/web/formal_ai_worker.js` uses to
  resolve any surface that is not already covered by the committed
  seed bundle. Before 2026-05-22 PR #222 considered shipping a
  pre-extracted dictionary instead; the reviewer rejected the
  approach (*"we should only store originals of the source data from
  wikipedia/wikidata/wiktionary"*), so the worker now always falls
  back to a live MediaWiki call when the seeded response is missing.
- **Translation subpages** — high-traffic English entries (e.g.
  `apple`, `water`, `milk`, `book`) keep translation tables on a
  `/translations` subpage referenced by
  `{{see translation subpage|...}}`. Both the Rust pipeline
  (`pipeline.rs` stage 1a) and the browser worker fetch the subpage
  when they see the marker and merge the wikitext before extracting
  templates.

## Wikidata SPARQL lexeme join

- **P5137** (`item for this sense`) is the canonical link from a
  Wiktionary sense to a language-neutral Q-item. Joining two
  lexemes through P5137 gives us a deterministic
  `meaning_id (Q…)` regardless of which side served the trigger.
- **Polysemy fix** — when a single surface form has both noun and
  verb lexemes (e.g. English `water` → noun and verb lexemes;
  `milk` → noun `L4514` and verb), the unconstrained SPARQL query
  returned whichever lexeme appeared first and could follow its
  P5137 sense to the wrong part of speech (`water → поливать`,
  `milk → доить`). Since 2026-05-22 the SPARQL query in
  `src/translation/wikidata.rs` requires
  `?source wikibase:lexicalCategory ?cat. ?target wikibase:lexicalCategory ?cat`,
  so source and target lexemes share a part of speech before P5137
  is joined.
- **502 errors** — the Wikidata SPARQL endpoint occasionally returns
  HTTP 502 under load. The pipeline records the error in the
  provenance trace and falls back to the Wiktionary block-level
  result instead of failing the whole translation.

## Russian Wiktionary specifics

- The `=== Перевод ===` section is the reverse-lookup anchor. The
  Russian edition is the densest source of `ru → other` data because
  English Wiktionary's `ru:` columns are often sparse for
  intermediate-frequency nouns.
- Inflection templates (`{{сущ-ru|...}}`) carry the full declension
  paradigm in wikitext. We do **not** parse them in this PR — Russian
  inflected forms like `помидоры` resolve through the MediaWiki
  redirect (the `помидоры` page redirects to the lemma `помидор` and
  the `parse` API follows the redirect transparently) — but parsing
  the template directly is the right long-term answer for plural and
  case forms that Wiktionary does not redirect.

## Browser CORS constraints

- Web workers run in the same origin as the host page (GitHub
  Pages: `link-assistant.github.io`). They can `fetch()` any
  same-origin asset (the seed bundles, the WASM blob, and the
  `data/seed/api-cache/*.lino` files served by the dev server).
- Cross-origin fetches succeed only when the remote server emits
  `Access-Control-Allow-Origin: *`. Wiktionary's `api.php` does so
  when you pass `&origin=*`. The worker's `liveWiktionaryTranslate`
  relies on exactly that behaviour — no proxy required, no offline
  dictionary shipped.

## Related upstream issues

- Wikimedia phabricator T241992 — long-standing request to include
  P5137 senses in `wbsearchentities`. If it ever lands, our
  formalize step can hit a single API and skip the Wiktionary
  wikitext parse for many words.
- Wiktionary T118720 — the inflection-template normalization
  request that would make plural / case forms machine-extractable.
  Tracking issue, not actionable from our side.
