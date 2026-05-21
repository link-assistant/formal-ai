# Issue 218 Case Study

## Scope

Umbrella issue: <https://github.com/link-assistant/formal-ai/issues/218>

Branch: `issue-218-71bece6bcc84`
Pull request: <https://github.com/link-assistant/formal-ai/pull/219>

Issue #218 asked for *all* outstanding translation sub-issues to be
resolved in a single PR using the universal Wiktionary + Wikidata
pipeline introduced by PR #208 / PR #211, and for a case-study folder
collecting requirements, root causes, and verification.

Sub-issues swept up by this PR:

- **#216** — `translate apple to russian` produced `[ru]` (browser demo).
- **#217** — `переведи «яблоко» на английский` produced
  `"[en] яблоко"` (browser demo).
- **#210** — Russian translation prompts already pass on `main` (PR #211).
  Regression coverage is retained here.

## Timeline

| Date (UTC) | Event |
| --- | --- |
| 2026-05-19 | PR #208 lands the Wiktionary + Wikidata translation pipeline. |
| 2026-05-21 | Issue #210 filed — translation prompts collide with capabilities/identity handlers. |
| 2026-05-21 | PR #211 merges the precedence fix and a compositional ru→en fallback. |
| 2026-05-21 | Demo report #216 — `translate apple to russian` returns the placeholder `[ru]`. |
| 2026-05-21 | Demo report #217 — `переведи «яблоко» на английский` returns `"[en] яблоко"`. |
| 2026-05-21 | Umbrella issue #218 opened; PR #219 prepared on `issue-218-71bece6bcc84`. |
| 2026-05-21 | Cache for `apple`/`яблоко` seeded via `refresh_translation_cache`; unquoted-surface fallback added; offline registry extended. |

## Requirements (from issue #218)

1. Fix every sub-issue in a single pull request.
2. Use the universal Wiktionary + Wikidata pipeline (PR #208 style)
   rather than hand-tuned per-prompt patches.
3. Support multiple prompt variations across all supported languages.
4. Collect logs and data in `docs/case-studies/issue-218/`.
5. Do online research and link external references.
6. Reconstruct the timeline, list requirements, and document root
   causes plus solution plans.
7. Add debug/verbose output if a root cause can't yet be established.
8. Report related upstream defects (Wiktionary, Wikidata, etc.) if any.

## Artifacts

Local artifacts captured during investigation live under `raw-data/`:

- `issue-218.json`, `issue-218-comments.json`: umbrella issue snapshot.
- `issue-216.json`, `issue-217.json`, `issue-210.json`, `issue-213.json`,
  `issue-207.json`: linked / related issue snapshots.
- `pr-219.json`, `pr-219-conversation-comments.json`,
  `pr-219-review-comments.json`, `pr-219-reviews.json`: prepared PR
  metadata.
- `repro-before-fix.log`: CLI reproduction *before* the fix (placeholders).
- `repro-after-fix.log`: CLI reproduction *after* the fix (concrete
  surfaces, no `[ru]` / `[en]` placeholders).

`online-research.md` collects the external references used while
diagnosing the bugs.

## Root Causes

1. **Empty Wiktionary cache for the single noun `яблоко` / `apple`.**
   `examples/refresh_translation_cache.rs` only seeded the demo phrase
   set (`hello`, `thank you`, `как у тебя дела`, …). The pipeline
   *can* parse the Wiktionary translation table for `apple`/`яблоко`
   correctly — it just had no offline data, so
   `CachedHttpClient::get` returned a not-found error and the handler
   formatted the `[en] яблоко` placeholder. (Issue #217.)
2. **No surface extracted for unquoted translation prompts.**
   `extract_quoted_phrase` (Rust) and `extractQuotedPhrase` (browser
   worker) returned `None` for `translate apple to russian`. The
   handler defaulted to an empty surface, the pipeline returned an
   empty candidate list, and the formatter rendered `[ru]`. (Issue
   #216.)
3. **Browser offline registry missing the apple noun.**
   The browser worker carries a small registry as a CORS-safe fallback
   for the demo. Without an `apple` entry it inherited the same
   placeholder behaviour even when the Rust path produced a concrete
   answer.

## Fixes

### Rust core

- `src/solver_helpers.rs`: add `extract_unquoted_translation_surface`
  helper that recovers the surface between `translate ` and ` to ` (or
  `переведи ` and ` на `), so unquoted prompts feed the pipeline.
- `src/solver_handlers/mod.rs`: wire the new helper as a fallback
  after `extract_quoted_phrase` in `try_translation`.
- `examples/refresh_translation_cache.rs`: add the `apple`/`яблоко`
  pairs so the offline cache covers single-noun translation tests.
- `data/translation-cache/`: refresh by running
  `FORMAL_AI_LIVE_API=1 cargo run --release --example refresh_translation_cache`.
  The pipeline now resolves both directions to the canonical
  `wikidata-sense:` meaning id.

### Browser worker

- `src/web/formal_ai_worker.js`: mirror the unquoted surface extractor
  (`extractUnquotedTranslationSurface`) and wire it into
  `tryTranslation`.
- Add an `apple` entry to `TRANSLATION_MEANING_REGISTRY` with
  English / Russian / Hindi / Chinese aliases (incl. Russian
  declension forms) so the demo can answer offline without hitting
  the Wiktionary API.

### Tests

- `tests/unit/specification/translation_via_links.rs`: new tests
  `issue_216_translate_apple_to_russian_without_quotes`,
  `issue_217_single_russian_noun_quoted`, and
  `issue_218_unquoted_russian_translation` keep coverage tight.
- `tests/e2e/tests/issue-218.spec.js`: Playwright coverage for the
  browser worker, including a round-trip (`ru→en→ru`).

## Before / After

| Prompt | Before (v0.94.0) | After |
| --- | --- | --- |
| `translate apple to russian` | `[ru]` | `"яблоко"` |
| `переведи «яблоко» на английский` | `"[en] яблоко"` | `"apple"` |
| `переведи "яблоко" на английский` | `"[en] яблоко"` | `"apple"` |
| `translate apple to english` | (empty body) | `"apple"` |
| `Переведи "доброе яблоко" на английский.` | `"[en] доброе яблоко"` (pre-#210) / `"good apple"` (post-#210) | `"good apple"` |

Raw CLI reproductions are in `raw-data/repro-before-fix.log` and
`raw-data/repro-after-fix.log`.

## Verification

- `cargo build --release` — clean.
- `cargo test --release --test unit translation_via_links` — 9 active
  tests pass (3 new); 7 forward-looking tests remain ignored.
- `cargo run --release --example repro_issue_218` — every case
  produces the expected surface, no placeholders left.
- `cargo run --release --example refresh_translation_cache` (offline
  re-run) — no gaps reported.

## Upstream-issue reports

None filed. The defects were entirely local to this repository:

- Wiktionary itself serves the correct translation tables; we just
  needed to seed the cache.
- Wikidata lexemes were already linked correctly (`L3257` and
  `L37724`).
- No third-party library reproduction was required.

## Future work

- Extend the browser offline registry to mirror the Rust cache
  automatically (script in `examples/`).
- Lift the 2..=4 word limit on the compositional ru→en fallback and
  use Wiktionary inflection tables to handle longer phrases.
- Add `FORMAL_AI_TRANSLATION_DEBUG=1` verbose logging in the pipeline
  for follow-up cases where the cache hit/miss path is non-obvious.
