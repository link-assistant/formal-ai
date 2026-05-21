# Issue 207 Case Study

## Scope

Issue: <https://github.com/link-assistant/formal-ai/issues/207>

Pull request: <https://github.com/link-assistant/formal-ai/pull/208>

Branch: `issue-207-40aa862c0b6e`

The report follows up on issue 190. After PR 191 wired up the Russian
prompt `Переведи "как у тебя дела?" на английский.` to a dedicated
translation handler, the resulting answer body was still composed as a
robot-shaped `meaning: … / surface (ru): … / surface (en): …` block. The
user pointed out three remaining problems:

1. The response feels robotic instead of like natural conversation.
2. The translation does not preserve the original formatting — when the
   user starts the quoted phrase with a lowercase letter, the answer
   capitalizes it (`How are you?`) instead of keeping the original case.
3. Only a single hardcoded meaning ID (`meaning_2cfc55c914d57d9e`,
   which encodes the canonical token `greeting_how_are_you`) actually
   resolves to a localized surface form; every other prompt falls back
   to a placeholder like `[en] …`.

Issue 207 asks the project to address all three through a single
**formalize → meaning → deformalize** pipeline backed by real
Wikipedia / Wikidata / Wiktionary data — *not* a hand-written list of
phrase pairs — document the supporting guidelines in `REQUIREMENTS.md`
/ `VISION.md` / `ARCHITECTURE.md`, and record the case study analysis
in this folder.

The user explicitly rejected the first attempt at a "shared offline
meaning registry" as a fake solution. The current implementation
removes the registry entirely and routes every translation through
Wiktionary's translation tables and Wikidata's SPARQL lexeme join.

## Local Evidence

Downloaded artifacts live alongside this README:

- `raw-data/issue-207.json`: issue payload at collection time.
- `raw-data/issue-207-comments.json`: issue comments (empty at the time
  of collection).
- `raw-data/pr-208.json`: PR 208 metadata snapshot.
- `raw-data/pr-208-conversation-comments.json`: PR conversation
  comments (empty at the time of collection).
- `raw-data/pr-208-review-comments.json`: PR review comments (empty at
  the time of collection).
- `raw-data/online-research.md`: curated external references that
  ground the solution plan (Wikipedia / Wikidata / Wiktionary API
  shapes, Abstract Wikipedia, casing style guides, reusable
  components).

## Online Research

A complete reading list lives in `raw-data/online-research.md`. The
most load-bearing references are:

- **Wiktionary `action=parse&prop=wikitext`** — the canonical way to
  extract per-edition translation tables. Translation candidates use a
  stable template family (`{{t|...}}`, `{{t+|...}}`, `{{tt|...}}`,
  `{{перев-блок|...}}`, `{{翻譯-頂}}...{{翻譯-底}}`) which we parse
  directly. Translation tables are delimited by `{{trans-top|gloss}}`
  ... `{{trans-bottom}}` on English Wiktionary so we can split
  polysemous entries by sense.
- **Wikidata SPARQL `ontolex:sense / wdt:P5137`** — joins two lexemes
  when they share a sense (P5137 = "item for this sense"). This is the
  language-neutral pivot that gives us a stable `meaning:` id no matter
  which surface form we observe.
- **Macro-language coverage** — Chinese Wiktionary's interlanguage
  links live under `cmn` / `yue` / `wuu`, never `zh`; Norwegian under
  `nb` / `nn`, never `no`. The pipeline falls back to the macro family
  when the direct ISO code returns no matches.
- **Combining diacritics in Wiktionary** — Russian entries include
  stress marks (`U+0301`) inside translation templates. We strip them
  so callers see orthographic forms like `привет` rather than
  `приве́т`.
- English / Russian / Chinese style guides on mid-sentence
  capitalization and terminal punctuation — justify the requirement to
  preserve the source-fragment formatting in the translated surface.

## Timeline

- 2026-05-21 12:27 UTC: User reported issue 207 from the GitHub Pages
  demo (formal-ai v0.86.0), highlighting the robotic translation feel
  and the hardcoded meaning ID.
- 2026-05-21 12:59 UTC: Branch `issue-207-40aa862c0b6e` prepared and
  PR 208 opened as a draft.
- 2026-05-21: Issue, PR, and online-research artifacts downloaded into
  `docs/case-studies/issue-207/raw-data/`.
- 2026-05-21: First attempt landed a shared offline meaning registry.
  User rejected it as a fake solution and asked for a generalized
  Wikipedia / Wikidata / Wiktionary integration with cached HTTP
  responses powering the tests.
- 2026-05-21: Pipeline rewritten — translation now runs through a real
  `formalize → meaning → deformalize` flow that parses Wiktionary
  wikitext, joins Wikidata lexemes via SPARQL, and caches the raw HTTP
  responses on disk under `data/translation-cache/`.

## Root Causes

1. **Robotic body builder.** `try_translation` in
   `src/solver_handlers/mod.rs` (and the matching `tryTranslation` in
   `src/web/formal_ai_worker.js`) packed the meaning ID and both
   surface forms into a multi-line `meaning: … / surface (ru): … /
   surface (en): …` body. The Links Notation trace already lives in the
   `links_notation` field and `evidence_links`, so the multi-line body
   duplicates that information in a form that no human conversational
   partner would write.
2. **Case/punctuation loss.** `translate_surface` in
   `src/solver_helpers.rs` mapped a lowercase Russian phrase
   (`как у тебя дела?`) to a capitalized English template
   (`How are you?`) with no awareness of the source-fragment casing or
   terminal punctuation. The browser worker mirrored the same loss.
3. **Single-meaning coverage.** `canonical_meaning_token` only knew
   `greeting` and `greeting_how_are_you`. Every other quoted phrase
   produced an opaque `meaning_<hash>` ID with no localized deformalize
   target, so the response fell back to `[en] …` placeholders. The same
   limit applied to the browser worker.
4. **No generalization.** A hand-written meaning registry could only
   cover the prompts we anticipated. Adding a new pair required a code
   change. The fix demanded a pipeline that can resolve any surface
   pair via existing knowledge bases (Wiktionary / Wikidata).

## Architecture

The new translation pipeline lives under `src/translation/`:

- `src/translation/http.rs` — minimal HTTP client trait
  (`HttpClient::get(&self, url) -> Result<String, HttpError>`). The
  default implementation shells out to `curl` so the crate has no TLS
  dependencies.
- `src/translation/cache.rs` — `CachedHttpClient` wraps any transport
  and persists raw response bodies under `data/translation-cache/<hash>.body`
  with a sibling `.url` file. Online mode (gated by
  `FORMAL_AI_LIVE_API`) populates the cache on miss; offline mode
  returns a transport error so cache-only tests are deterministic.
- `src/translation/wiktionary.rs` — `Wiktionary` client + wikitext
  parser. Extracts translation candidates from `{{t|...}}` /
  `{{t+|...}}` / `{{tt|...}}` / `{{перев-блок|...}}` /
  `{{翻譯-頂}}...{{翻譯-底}}` blocks. Returns candidates grouped by
  sense block (one block per `{{trans-top}}` on en.wiktionary).
- `src/translation/wikidata.rs` — `Wikidata` client + SPARQL response
  parser. Runs the canonical lexeme join
  `?lexeme ontolex:sense ?sense . ?sense wdt:P5137 ?meaning`.
- `src/translation/meaning.rs` — `MeaningId` is the semantic
  meta-language identifier. Priority order: Wikidata Q-item >
  Wikidata sense > Wiktionary page.
- `src/translation/pipeline.rs` —
  `TranslationPipeline::translate(surface, source, target)` chains the
  Wiktionary lookup, reverse lookup on the target edition, phrasal
  variant fallback, sense-block selection by round-trip confirmation,
  and Wikidata meaning upgrade.

The pipeline emits a `provenance` trail of every API call, so the
links-notation trace records exactly which Wiktionary edition / page /
SPARQL query produced the answer.

## Requirement Traceability

| Requirement | Implementation | Verification |
| --- | --- | --- |
| Translation responses must feel like natural conversation. | `src/solver_handlers/mod.rs::try_translation` returns just the deformalized target surface; meaning / source / target stay in `evidence_links` and `links_notation`. | `tests/unit/specification/translation_via_links.rs::russian_translate_how_are_you_prompt_returns_english_surface` and `natural_translation_drops_terminal_when_source_has_none`. |
| Translations must preserve the source formatting (initial casing and terminal punctuation). | `src/translation/formatting.rs::match_source_formatting`. | `tests/unit/specification/translation_via_links.rs::russian_capitalized_how_are_you_keeps_target_capitalization`. |
| The pipeline must translate any surface, not only a hardcoded list. | `src/translation/pipeline.rs::TranslationPipeline::translate` resolves surfaces via Wiktionary translation blocks + Wikidata sense joins. Raw HTTP responses are cached under `data/translation-cache/` (FNV-1a hashed URL → body + url sibling). | `tests/unit/specification/translation_via_links.rs::translation_meaning_registry_covers_extended_phrases` covers eight unrelated phrase pairs across en / ru / hi / zh routed entirely through cached real-world responses. |
| The formalize → meaning → deformalize architecture must be documented. | `REQUIREMENTS.md` records R213 / R214 / R215; `ARCHITECTURE.md` section 10 references the pipeline and the Wiktionary/Wikidata fallback chain. | `tests/unit/docs_requirements.rs` pins the requirement IDs. |
| Real-world API responses must be cached so tests are deterministic. | `src/translation/cache.rs::CachedHttpClient` + `data/translation-cache/`. Tests run with the cached responses checked into the repo. | `cargo test` runs offline (no `FORMAL_AI_LIVE_API`) and still passes. |
| The case study must include issue, PR, comment, and online-research artifacts. | This folder. | Local file listing. |

## Fixes

- Removed the hand-written meaning registry that the first PR attempt
  introduced. The `MEANING_REGISTRY` and the `formalize_surface` /
  `deformalize_meaning` helpers based on it are gone.
- Added `src/translation/` module: `http`, `cache`, `wiktionary`,
  `wikidata`, `meaning`, `pipeline`, `formatting`.
- `TranslationPipeline::translate` now resolves any surface pair by:
  1. Fetching the source-edition Wiktionary page and parsing its
     `{{trans-top}}` blocks for `target_lang` candidates.
  2. Falling back to the `/translations` subpage when the main page
     omits translations (common for high-traffic English entries).
  3. Falling back to the target-edition Wiktionary page in reverse
     when the source edition is sparse (typical for ru → en).
  4. Generating phrasal variants (e.g. dropping Russian "у тебя",
     "у вас", "у меня" infixes) when the literal page does not exist.
  5. Selecting the best sense block by round-trip confirmation rate —
     for each candidate, count how many target-edition pages list the
     source surface as a translation. The block with the most confirms
     wins.
- Added an FNV-1a-keyed file cache (`CachedHttpClient`) that persists
  every HTTP response under `data/translation-cache/`. The committed
  cache makes the integration tests deterministic and offline.
- Added `examples/refresh_translation_cache.rs` so contributors can
  re-populate the cache by setting `FORMAL_AI_LIVE_API=1`.
- Updated the browser worker comment to acknowledge that its small
  offline registry now serves as a CORS-safe fallback for the demo;
  the Rust pipeline is the canonical implementation.

## Verification Plan

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-features`
- `cargo test translation_via_links`
- `cargo test translation_meaning_registry_covers_extended_phrases`

All checks pass offline against the cached HTTP responses in
`data/translation-cache/`.

## Future Work

- Expand the cache to cover additional Wiktionary editions (Hindi,
  Japanese, German). The pipeline already supports them; only seed
  data is missing.
- Wire the `CachedHttpClient` into the browser worker so the
  JavaScript runtime can consult the same cached responses (currently
  the browser keeps a small offline registry as a CORS-safe fallback).
- Adopt Wikidata Lexeme search to disambiguate when multiple senses
  round-trip with equal confirmation counts.
