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
  `fetchWiktionaryEntry` already uses for the external lookup
  fallback. We **could** add live translation fetch in the worker on
  top of this, but the current pull request opts for a pre-compiled
  dictionary because the demo must work without leaking the
  user's browsing patterns to Wikimedia. See PR #222 discussion.
- **Translation subpages** — high-traffic English entries (e.g.
  `apple`, `book`, `milk`) keep translation tables on a `/translations`
  subpage referenced by `{{see translation subpage|...}}`. The Rust
  pipeline already tries this fallback (`pipeline.rs` stage 1a).

## Wikidata SPARQL lexeme join

- **P5137** (`item for this sense`) is the canonical link from a
  Wiktionary sense to a language-neutral Q-item. Joining two
  lexemes through P5137 gives us a deterministic
  `meaning_id (Q…)` regardless of which side served the trigger.
- **Polysemy gotcha** — when a single surface form has both noun and
  verb lexemes (e.g. English `milk` → noun `L4514` and verb
  `L4514-S1`), the SPARQL query returns whichever lexeme appears
  first. The current code path picks `L4514` and follows its sole
  P5137 sense to whatever Wikidata calls the canonical Russian form,
  landing on `доить` (the verb "to milk") instead of `молоко`. This
  is a **known limitation** flagged in the case-study future work;
  the dictionary work-around captures `molokó` directly for the
  demo.
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
  paradigm in wikitext. We do **not** parse them in this PR — the
  alias generator hard-codes the common suffix tables — but
  parsing them is the right long-term answer for issues like
  `Переведи "помидоры" на английский.` (plural form).

## Browser CORS constraints

- Web workers run in the same origin as the host page (GitHub
  Pages: `link-assistant.github.io`). They can `fetch()` any
  same-origin asset (the seed bundles, the WASM blob, and now
  `translation-dictionary.json`).
- Cross-origin fetches succeed only when the remote server emits
  `Access-Control-Allow-Origin: *`. Wiktionary's `api.php` does so
  when you pass `&origin=*`. So a live fallback is technically
  possible — see the "Future work" section in the README.

## Related upstream issues

- Wikimedia phabricator T241992 — long-standing request to include
  P5137 senses in `wbsearchentities`. If it ever lands, our
  formalize step can hit a single API and skip the Wiktionary
  wikitext parse for many words.
- Wiktionary T118720 — the inflection-template normalization
  request that would make plural / case forms machine-extractable.
  Tracking issue, not actionable from our side.
