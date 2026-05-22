# Issue 221 Case Study

## Scope

Umbrella issue: <https://github.com/link-assistant/formal-ai/issues/221>

Branch: `issue-221-77aad836fb28`
Pull request: <https://github.com/link-assistant/formal-ai/pull/222>

Issue #221 reports that the browser demo still produces fake
placeholders for common nouns the predecessor case study #218 did not
cover:

```
U: Переведи "яблоко" на английский.
A: "apple"
U: Переведи "огурец" на английский.
A: "[en] огурец"
U: Переведи "помидор" на английский.
A: "[en] помидор"
```

The umbrella requirement (verbatim, from the issue): *"We must stop
faking translation, and actually implement it for all possible words we
can find in wikipedia, wikidata, wiktionary, and actually use APIs."*

In other words, #218 fixed the **single-noun apple** case; #221 demands
that the **same machinery works for any common noun**, in any direction
between the supported languages, in **both** the Rust CLI and the
browser demo, *without LLMs* and *without a pre-extracted offline
dictionary*.

## Timeline

| Date (UTC) | Event |
| --- | --- |
| 2026-05-19 | PR #208 lands the Wiktionary + Wikidata translation pipeline. |
| 2026-05-21 | PR #211 fixes translation handler precedence and adds the compositional ru→en fallback. |
| 2026-05-21 | Demo report #216 — `translate apple to russian` returns `[ru]`. |
| 2026-05-21 | Demo report #217 — `переведи «яблоко» на английский` returns `"[en] яблоко"`. |
| 2026-05-21 | Umbrella issue #218 opened; PR #219 fixes the apple noun for all four supported languages and lands a multilingual coverage CI guard. |
| 2026-05-21 | Demo report #221 — the same fix does not cover `помидор` / `огурец` and the request escalates to **all** common nouns plus *texts of any size*. |
| 2026-05-21 | Branch `issue-221-77aad836fb28` cut; PR #222 prepared. |
| 2026-05-21 | CLI reproduction confirms the placeholder lives in two places: the offline cache only has `яблоко`, and the browser registry only has `apple`. |
| 2026-05-21 | First-pass fix expanded the seed list and shipped a 128-entry JSON dictionary to the browser worker. |
| 2026-05-22 | PR #222 review (konard) rejects the dictionary approach: *"we should only store originals of the source data from wikipedia/wikidata/wiktionary, that should be used for formalization steps, we should never cache or preseed `Extract the offline translation dictionary`."* — pipeline must always go through `source → formalize → meaning → deformalize → target`, the only legal cached artefact is the raw API response. |
| 2026-05-22 | `data/seed/translations.lino` and `examples/build_translation_dictionary.rs` deleted. Cache reorganised by **semantic identity** (`wikidata-cache/{search,entities,query,sparql}/`, `wiktionary-cache/<lang>/`, `http-cache/misc/`). Raw API responses are bundled into `data/seed/api-cache/*.lino` (base64-payload Links Notation) capped at 128 records per bucket and 1500 lines per file. |
| 2026-05-22 | `build.rs` added so the bundle's `<bucket>-partN.lino` parts ship in the binary without per-file `include_str!` edits. The refresh tool splits oversize records into deterministic parts and cleans up stale parts. |
| 2026-05-22 | Browser worker's `TRANSLATION_DICTIONARY` removed; replaced with `liveWiktionaryTranslate` calling MediaWiki action API (`origin=*`) directly. |
| 2026-05-22 | `en→ru "water"` debugged: SPARQL lexeme join crossed noun ↔ verb boundary (`L7234-S1` → `поливать`). Fix narrows the join by `wikibase:lexicalCategory`. Stage 1a in `pipeline.rs` also delegates to the `/translations` subpage when the main page carries `{{see translation subpage|...}}`. |

## Requirements (from issue #221 and PR #222 review)

1. Stop faking translation — no `[en] X` / `[ru] X` placeholders for
   any common noun in any supported direction.
2. Use Wikipedia, Wikidata, and Wiktionary as the only sources of
   meaning — the symbolic AI pipeline must remain LLM-free.
3. Follow the *source → meta → target* flow already proven in #218:
   `formalize(source) → meaning_id (Q/L/sense) → deformalize(target)`.
4. **Never** ship a pre-extracted dictionary. The only legal cached
   artefact is a raw API request/response from Wikipedia, Wikidata or
   Wiktionary, kept in `.lino` format. (PR #222 review, 2026-05-22.)
5. Cap committed data at **128 most-frequent records per bucket** so
   the diff stays reviewable and the architecture scales beyond
   translation (entity resolution, fact lookup, etc.).
6. Each `.lino` file must stay under 1500 lines (Links Notation
   readability cap); oversize bodies split into `<bucket>-partN.lino`.
7. The browser worker must stay mobile-friendly: no bundled offline
   dictionary, lookups uncovered by the seed must fall through to live
   MediaWiki calls with `origin=*`.
8. Solve everything in a single pull request that **actually works**
   end-to-end, not just for one phrase.
9. Compile all logs and data into `docs/case-studies/issue-{id}/`,
   reconstruct the timeline, list every requirement, find every root
   cause, and propose solutions. Search externally for additional data.
10. Add debug output and a verbose mode if a root cause cannot yet be
    pinned down.
11. Report upstream defects against any external repo (Wiktionary,
    Wikidata, etc.) that surface during investigation.

## Artifacts

Local artifacts captured during investigation live under `raw-data/`:

- `repro-pomidor-before-fix.txt`, `repro-ogurec-before-fix.txt` — CLI
  reproductions before any change. Offline cache miss → `[en] помидор`.
- `repro-pomidor-live-api.txt`, `repro-ogurec-live-api.txt` — same
  prompt with `FORMAL_AI_LIVE_API=1`. Pipeline resolves
  помидор→tomato and огурец→cucumber correctly, proving the algorithm
  works and the gap is data-seeding-only.
- `test-before-fix.txt` — output of the new `issue_221_*` tests before
  any seeding change, showing two failures (картофель/potato) and one
  pass (помидор/огурец, because earlier live-API runs had already
  warmed those entries).

`online-research.md` collects the external references used while
diagnosing the bug (Wiktionary CORS policy, MediaWiki `origin=*`
convention, Wikidata SPARQL endpoint behaviour for polysemous lexemes).

## Root Causes

1. **Offline cache only contained one noun (`яблоко` / `apple`).**
   `examples/refresh_translation_cache.rs` was the single source of
   ground-truth data fed to the offline pipeline, and PR #219 added
   only the apple pair. Every other common noun returned an `HttpError::
   Transport(... cache miss for {url} and offline mode is active)`,
   which the handler renders as `[en] помидор`. Verified by re-running
   the same prompt with `FORMAL_AI_LIVE_API=1` — the pipeline succeeds
   immediately (logs in `raw-data/repro-*-live-api.log`).
2. **Browser worker only consulted a hand-curated registry.**
   `TRANSLATION_MEANING_REGISTRY` in `src/web/formal_ai_worker.js` was
   built up phrase-by-phrase. Words missing from the registry fell
   through to `[${target}] ${surface}` (line 3905 pre-fix). The Rust
   pipeline never runs in the browser — the worker is the only code
   that handles in-browser translation.
3. **Wikidata SPARQL lexeme join ignored part of speech.** Joining two
   lexemes by P5137 without filtering on `wikibase:lexicalCategory`
   crosses noun ↔ verb boundaries when both senses exist (e.g.
   `water` → `поливать`, `milk` → `доить`). The fix adds the category
   filter to `src/translation/wikidata.rs`.
4. **High-traffic English entries hide translations on a subpage.**
   `apple`, `water`, `milk`, `bread` keep their translation tables on
   `<lemma>/translations` referenced by `{{see translation subpage|...}}`.
   The pipeline's Stage 1a delegates to the subpage when the marker is
   present.
5. **No verbose mode in the pipeline.** When tests failed, the only
   signal was a single line of output. Issue #218 explicitly listed
   `FORMAL_AI_TRANSLATION_DEBUG=1` as future work.

## Fixes

### Rust core

- `src/translation/pipeline.rs`: `FORMAL_AI_TRANSLATION_DEBUG=1`
  verbose tracing through every stage (`stage1` source-edition,
  reverse, variants, Wikidata upgrade, compositional fallback). When
  enabled, every translation prints stage-by-stage to stderr so
  cache-miss vs sparse-Wiktionary-table vs polysemy can be
  distinguished in a single run. Stage 1a also follows
  `{{see translation subpage|...}}` to the `/translations` subpage and
  merges the table back into the parsed wikitext.
- `src/translation/wikidata.rs`: SPARQL lexeme join now requires
  `?source wikibase:lexicalCategory ?cat. ?target wikibase:lexicalCategory ?cat`
  so source and target lexemes share a part of speech.

### Semantic-identity HTTP cache

- `src/translation/cache.rs` routes URLs to per-source folders by the
  **kind of data** they carry, not by URL hash:

  | URL pattern | Cache folder |
  | --- | --- |
  | `wikidata.org/w/api.php?action=wbsearchentities` | `data/wikidata-cache/search/` |
  | `wikidata.org/w/api.php?action=wbgetentities` | `data/wikidata-cache/entities/` |
  | `wikidata.org/w/api.php?action=query` | `data/wikidata-cache/query/` |
  | `query.wikidata.org/sparql` | `data/wikidata-cache/sparql/` |
  | `<lang>.wiktionary.org` | `data/wiktionary-cache/<lang>/` |
  | anything else | `data/http-cache/misc/` |

  Every folder under `data/` listed above is gitignored — the
  on-disk cache is a local accelerator written by `FORMAL_AI_LIVE_API=1`
  runs, not pre-seeded data. Formalisation flows beyond translation
  (entity resolution, fact lookup, etc.) reuse the same buckets so we
  never grow a per-feature cache silo again.

- `CachedHttpClient::get` consults three layers in order:
  1. Committed `data/seed/api-cache/*.lino` seed bundle (deterministic,
     ships in git, base64 payloads decode to the verbatim response
     body).
  2. Gitignored on-disk accelerator under `data/<bucket>-cache/...`.
  3. Live HTTP transport (only when `online == true`).

### Committed seed bundle (raw API responses, not a dictionary)

- `data/seed/api-cache/*.lino` ships **verbatim API response bodies** —
  one indented record per URL, payload base64-encoded (RFC 4648,
  76-character chunks). Buckets:
  - `wikidata-search.lino` (`wbsearchentities` results)
  - `wikidata-entities.lino` (`wbgetentities` results)
  - `wikidata-sparql.lino` (SPARQL responses)
  - `wikidata-properties.lino` (property lookups)
  - `wiktionary-pages.lino` + `wiktionary-pages-partN.lino` (per-language
    `parse` API responses)
- Hard caps: at most 128 records per bucket, each file ≤ 1500 lines.
  `examples/refresh_translation_cache.rs` enforces both during bundling
  and splits oversize records into `<bucket>-partN.lino` parts that
  rejoin transparently at load time (same URL key → bytes concatenated).
- `build.rs` enumerates every `.lino` file in `data/seed/api-cache/` at
  compile time and writes the list into `OUT_DIR/seed_bundle_files.rs`.
  `cache.rs` pulls it via `include!(concat!(env!("OUT_DIR"), "..."))`,
  so adding a new `-partN` file is automatic — no per-file
  `include_str!` edit.

### Browser worker

- `src/web/seed_loader.js`: removed the `extractTranslations` parser
  and the `translations` field — the worker no longer hydrates a
  dictionary from a bundled file.
- `src/web/formal_ai_worker.js`: removed `TRANSLATION_DICTIONARY` and
  `lookupDictionary`. Translation flows through the existing meaning
  registry first, then through a new `liveWiktionaryTranslate(surface,
  source, target)` that calls
  `*.wiktionary.org/w/api.php?action=parse&page=...&prop=wikitext&format=json&origin=*`
  directly. The fetcher follows `{{see translation subpage|...}}` to
  the `/translations` subpage and runs the standard
  `{{tt+|<lang>|...}}` / `{{t|...}}` regex against the merged wikitext.
  `translateSurface` and `tryTranslation` are now async; the single
  caller in `solve` already runs inside `async`.
- Mobile-friendly by design: nothing is pre-bundled, the MediaWiki
  action API is CORS-friendly through `origin=*`, and the seed bundle
  caps keep the worker payload tiny.

### Tests

- `tests/unit/specification/translation_via_links.rs`: three
  `issue_221_*` tests fail loudly when any of
  помидор/огурец/картофель/морковь/хлеб/вода or their English
  counterparts return a placeholder.
- `tests/e2e/tests/issue-221.spec.js`: Playwright coverage for the
  browser worker — quoted RU→EN, quoted EN→RU, unquoted prompts,
  Russian inflected forms via MediaWiki redirect, and the round-trip
  `tomato → помидор → tomato` to prove the live path keeps semantics
  symmetric.

## Before / After

| Prompt | Before (v0.100.0) | After |
| --- | --- | --- |
| `Переведи "помидор" на английский.` | `"[en] помидор"` | `"tomato"` |
| `Переведи "огурец" на английский.` | `"[en] огурец"` | `"cucumber"` |
| `переведи "картофель" на английский` | `"[en] картофель"` | `"potato"` |
| `translate "tomato" to russian` | `"[ru] tomato"` | `"помидор"` |
| `translate "carrot" to russian` | `"[ru] carrot"` | `"морковь"` |
| `translate "water" to russian` | `"поливать"` (verb!) | `"вода"` |
| `переведи помидор на английский` (unquoted) | `"[en] помидор"` | `"tomato"` |

Raw CLI reproductions are in `raw-data/repro-*-before-fix.txt` and
`raw-data/repro-*-live-api.txt`.

## Verification

- `cargo build --release` — clean.
- `cargo test --release --test unit translation_via_links` — issue
  #218 tests still pass; three new `issue_221_*` tests pass.
- `cargo clippy --all-targets --release` — clean.
- `cargo fmt --all -- --check` — clean.
- `FORMAL_AI_LIVE_API=1 cargo run --release --example refresh_translation_cache`
  — refreshes the on-disk accelerator and rebundles
  `data/seed/api-cache/*.lino` from scratch (rerun once a quarter or
  whenever the seed list changes).
- `npm --prefix tests/e2e run test:local -- tests/issue-221.spec.js` —
  browser worker hits live Wiktionary; expects network connectivity.

## Upstream-issue reports

None filed. Wiktionary serves correct data, Wikidata serves correct
lexemes; the gaps were entirely in our seeding pipeline and SPARQL
filter. The polysemy edge case is now mitigated by the
`wikibase:lexicalCategory` filter, with the rationale documented in
`online-research.md`.

## Future work

- **Wikibase lexeme part-of-speech for languages without lexemes** —
  the SPARQL filter only helps when both source and target carry
  lexeme entries. Several Russian and Hindi nouns still lack lexemes
  and fall back to the Q-item join. Tracked under the
  multilingual-coverage CI guard added in #219.
- **Sentence-level translation** — the issue asks for "text of any
  size". The current pipeline plus live worker covers single words and
  short greetings. A proper sentence pipeline (tokenize → translate
  per-token → re-inflect via Wiktionary grammar tables) is the
  long-form follow-up. Tracked in the "согласованность" comment in
  issue #221.
- **Inflection tables from `{{сущ-ru}}`** — Russian Wiktionary carries
  the full declension paradigm in wikitext. Parsing those templates
  would let us drop the heuristic plural fallback. Tracked under the
  same follow-up.
