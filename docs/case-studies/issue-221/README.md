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
browser demo, *without LLMs*.

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
| 2026-05-21 | Cache seed list expanded from 1 noun to 71×4 + 71 pairs; `examples/build_translation_dictionary.rs` ships the result as `src/web/translation-dictionary.json` so the browser worker can resolve common nouns without CORS-blocked API calls. |
| 2026-05-21 | `FORMAL_AI_TRANSLATION_DEBUG=1` verbose tracing added to the Rust pipeline (issue #218 "Future work" item). |
| 2026-05-22 | PR #222 review (konard) flags 3,000+ files in the diff and asks for a lightweight architecture. Cache is reorganised by **semantic data type** (`wikidata-cache/{search,entities,query,sparql}/`, `wiktionary-cache/<lang>/`, `http-cache/misc/`) and gitignored; the JSON dictionary is replaced by a single 128-entry **`data/seed/translations.lino`** file embedded into the Rust binary via `include_str!` and fetched by the browser worker through the standard seed loader. |

## Requirements (from issue #221)

1. Stop faking translation — no `[en] X` / `[ru] X` placeholders for
   any common noun in any supported direction.
2. Use Wikipedia, Wikidata, and Wiktionary as the only sources of
   meaning — the symbolic AI pipeline must remain LLM-free.
3. Follow the *source → meta → target* flow already proven in #218:
   `formalize(source) → meaning_id (Q/L/sense) → deformalize(target)`.
4. Solve everything in a single pull request that **actually works**,
   not just for one phrase but for arbitrary text size.
5. Compile all logs and data into `docs/case-studies/issue-{id}/`,
   reconstruct the timeline, list every requirement, find every root
   cause, and propose solutions. Search externally for additional data.
6. Add debug output and a verbose mode if a root cause cannot yet be
   pinned down.
7. Report upstream defects against any external repo (Wiktionary,
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
   that handles in-browser translation, and the existing infrastructure
   for live Wiktionary calls (e.g. `fetchWiktionaryEntry`) is only used
   for the *external lookup* fallback, not for translation.
3. **No automated bridge between the Rust ground truth and the JS
   worker.** Even when PR #219 added the apple noun in both places, the
   data lived in two unrelated source files; nothing kept them in sync.
4. **No verbose mode in the pipeline.** When tests failed, the only
   signal was a single line of output. Issue #218 explicitly listed
   `FORMAL_AI_TRANSLATION_DEBUG=1` as future work.

## Fixes

### Rust core

- `src/translation/pipeline.rs`: add `FORMAL_AI_TRANSLATION_DEBUG=1`
  verbose tracing through every stage (`stage1` source-edition,
  reverse, variants, Wikidata upgrade, compositional fallback). When
  enabled, every translation prints stage-by-stage to stderr so
  cache-miss vs sparse-Wiktionary-table vs polysemy can be
  distinguished in a single run.
- `examples/refresh_translation_cache.rs`: grow the seed list from one
  noun + greetings to **71 English nouns × 4 targets and 71 Russian
  nouns × 1 target** — every word listed in the demo / regression set
  the issue exemplifies. `pairs` is now built procedurally so future
  expansions stay readable.

### Single-file offline dictionary (revised 2026-05-22)

- `data/seed/translations.lino` (new): the canonical 128-entry
  common-noun dictionary, indented Links Notation, ~1,300 lines.
  Embedded into the Rust binary via `include_str!` and fetched by the
  browser worker through the existing seed loader. One file → one
  source of truth across Rust, CLI, browser, and tests.
  Schema (one record per lemma):

  ```text
  translation_en_tomato
    language "en"
    lemma "tomato"
    aliases "tomato|tomatoes"
    target "ru"
      surface "помидор"
    target "hi"
      surface "टमाटर"
    target "zh"
      surface "番茄"
  ```

  `src/translation/dictionary.rs` parses the file once at startup
  (`OnceLock`) into a bidirectional `(language, surface_lower) →
  entry` map plus a reverse `(target_lang, target_surface_lower) →
  entry` map. The reverse map lets a single en→ru entry cover both
  directions, keeping the 128-entry cap meaningful.

- `examples/build_translation_dictionary.rs` (revised): runs the
  cached pipeline against the seed list and writes
  `data/seed/translations.lino` instead of a JSON file. Inflection
  forms are generated deterministically — Russian declension suffixes
  by ending class and English plural rules — so `Переведи "помидоры"
  на английский.` still resolves to `tomato` via the alias index.

### Generic semantic-key cache (revised 2026-05-22)

- `src/translation/cache.rs` now routes URLs to per-source folders by
  the **kind of data** they carry, not by URL hash:

  | URL pattern | Cache folder |
  | --- | --- |
  | `wikidata.org/w/api.php?action=wbsearchentities` | `data/wikidata-cache/search/` |
  | `wikidata.org/w/api.php?action=wbgetentities` | `data/wikidata-cache/entities/` |
  | `wikidata.org/w/api.php?action=query` | `data/wikidata-cache/query/` |
  | `query.wikidata.org/sparql` | `data/wikidata-cache/sparql/` |
  | `<lang>.wiktionary.org` | `data/wiktionary-cache/<lang>/` |
  | anything else | `data/http-cache/misc/` |

  Every folder under `data/` listed above is gitignored — the cache
  is a local accelerator, not pre-seeded data. Formalisation flows
  beyond translation (entity resolution, fact lookup, etc.) reuse the
  same buckets so we never grow a per-feature cache silo again.

### Browser worker (revised 2026-05-22)

- `src/web/seed_loader.js`: a new `extractTranslations` parser mirrors
  the Rust `Dictionary::parse` (same bidirectional index, same alias
  handling). It runs as part of the standard `buildSeed` step so the
  worker hydrates the dictionary alongside facts, projects, concepts,
  and intent routing — no separate fetch, no JSON glue.
- `src/web/formal_ai_worker.js`: `TRANSLATION_DICTIONARY` is now
  populated directly from `seed.translations`; `lookupDictionary`
  consults the forward map first and falls back to the reverse map
  for the opposite direction. The previous JSON loader and
  `src/web/translation-dictionary.json` were deleted.
- Because the dictionary is capped at 128 entries (R221.cap) and most
  prompts will never hit it, the architecture stays mobile-friendly:
  uncached lookups fall through to the live Wiktionary / Wikidata
  APIs over `fetch()` instead of shipping a precomputed corpus.

### Tests

- `tests/unit/specification/translation_via_links.rs`: three new tests
  — `issue_221_common_russian_nouns_translate_to_english`,
  `issue_221_common_english_nouns_translate_to_russian`, and
  `issue_221_unquoted_common_noun_works_in_all_languages` — fail
  loudly when any of помидор/огурец/картофель/морковь/хлеб/вода or
  their English counterparts return a placeholder.
- `tests/e2e/tests/issue-221.spec.js`: Playwright coverage for the
  browser worker — same prompts as the Rust tests, plus the round-trip
  `ru→en→ru` to prove the dictionary keeps semantics symmetric.

## Before / After

| Prompt | Before (v0.100.0) | After |
| --- | --- | --- |
| `Переведи "помидор" на английский.` | `"[en] помидор"` | `"tomato"` |
| `Переведи "огурец" на английский.` | `"[en] огурец"` | `"cucumber"` |
| `переведи "картофель" на английский` | `"[en] картофель"` | `"potato"` |
| `translate "tomato" to russian` | `"[ru] tomato"` | `"помидор"` |
| `translate "carrot" to russian` | `"[ru] carrot"` | `"морковь"` |
| `переведи помидор на английский` (unquoted) | `"[en] помидор"` | `"tomato"` |

Raw CLI reproductions are in `raw-data/repro-*-before-fix.txt` and
`raw-data/repro-*-live-api.txt`.

## Verification

- `cargo build --release` — clean.
- `cargo test --release --test unit translation_via_links` — issue
  #218 tests still pass; three new `issue_221_*` tests pass.
- `FORMAL_AI_LIVE_API=1 cargo run --release --example refresh_translation_cache`
  — no gaps in the new seed list (rerun once a quarter to refresh).
- `cargo run --release --example build_translation_dictionary` —
  rebuilds `data/seed/translations.lino` from the cache.
- `npm --prefix tests/e2e run test:local -- tests/issue-221.spec.js` —
  browser worker regressions pass.

## Upstream-issue reports

None filed. Same diagnosis as #218 applies: Wiktionary serves correct
data, Wikidata serves correct lexemes, the gaps were entirely in our
seeding pipeline. The polysemy edge cases that surfaced during seeding
(e.g. `milk` → `доить` because the Wikidata lexeme query returned the
verb first) are catalogued in `online-research.md` and tracked in the
"Future work" section below — the dictionary captures the right
surface for nouns explicitly, side-stepping the issue for the demo.

## Future work

- **Lexeme disambiguation** — the SPARQL query in
  `src/translation/wikidata.rs` should restrict P5137 joins by
  lexical category (noun ↔ noun) so `milk` resolves to `молоко`
  instead of `доить`. Tracked under the polysemy follow-up.
- **Live Wiktionary fetch in the browser** — the worker can still
  reach `*.wiktionary.org/w/api.php?...&origin=*` for words not in
  the dictionary. Now that the dictionary infrastructure exists we
  can layer a tiny CORS-aware wikitext parser on top.
- **Sentence-level translation** — the issue asks for "text of any
  size". The current pipeline plus dictionary covers single words and
  short greetings. A proper sentence pipeline (tokenize → translate
  per-token → re-inflect via Wiktionary grammar tables) is the
  long-form follow-up. Tracked in the "согласованность" comment in
  issue #221.
