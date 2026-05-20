# Issue 180 Case Study: Google-Style Search Rendering, Always-On Deformalize, and Deep Diagnostics

## Summary

Issue [#180](https://github.com/link-assistant/formal-ai/issues/180) was filed
after a `Найди в интернете яблоко` ("find an apple online") run on the
deployed `v0.73.0` build returned a verbose, unstyled search digest that
glued multi-thousand-character Internet Archive descriptions next to single
sentences from DuckDuckGo, never collapsed obvious cross-provider duplicates,
hard-coded a single provider order that ignored the agreed priority list,
silently double-paid the network cost when a provider was CORS-blocked, and
failed to project the symbolic answer back into natural language because the
worker had no explicit `deformalize` step. On top of that, the topbar
collapse/expand button and the source-code button stopped honoring the dark
theme, the diagnostics panel rendered no raw HTTP request/response payloads,
and the left sidebar still wrapped to two columns at narrow widths.

This PR ([#181](https://github.com/link-assistant/formal-ai/pull/181)) lands
the full fix on branch `issue-180-d9c09fe7795a`:

1. **Google-style web_search rendering.** Every bullet now leads with
   `domain — title`, an inline excerpt that highlights the original query, and
   a localized **Read more** link. The Wikipedia + Wikidata + Internet Archive
   payloads run through a single normalizer that keeps the first 280
   characters of the most relevant fragment instead of dumping the whole
   abstract.
2. **Duplicate collapse + "Other sources" sub-line.** Any two providers that
   describe the same canonical entity (`Q89` ↔ `en.wikipedia.org/wiki/Apple`)
   merge into one bullet, and the absorbed URLs are listed under
   `Другие источники / Other sources / 其他来源 / अन्य स्रोत` in the user's
   language.
3. **Source priority pinned to DDG → IA → WP → WD → Wikt → rest.** The
   default plan is now codified in both the JS worker and the Rust
   `web_search_core` registry, with a unit test that pins the order so a
   regression in either layer trips immediately.
4. **Per-session CORS availability cache.** Every provider is probed at most
   once per tab session via a small HEAD/GET pre-flight. Providers that
   return CORS errors or non-2xx statuses are remembered in RAM until the
   browser tab is closed, eliminating the broken-request storm we saw in
   `image-1.png`.
5. **Dark-theme parity in the new UI.** The collapse/expand affordance, the
   source-code button, and the right-edge controls now all inherit the
   `[data-theme="dark"]` token set; the broken `:hover` and `:focus-visible`
   states from `image-3.png` are gone.
6. **Always-single-column collapsible left menu.** Both mobile and desktop
   layouts use a single grid column, and the sidebar collapses on demand to
   yield more room to the conversation list.
7. **Diagnostics badges + markup fixes.** The badges in `image-2.png` were
   overflowing the chip backgrounds because of mismatched padding; the
   typography is now consistent and every chip respects the parent line
   height.
8. **Raw HTTP req/resp panels + unified Links Notation per step.** Every
   reasoning step that talks to the network now records the raw request
   (method, URL, headers, body), the raw response (status, elapsed-ms,
   bytes, snippet), and a unified-Links-Notation projection of the same
   exchange, all displayed under a collapsible `<details>` in diagnostics
   mode.
9. **Real traceable reasoning architecture.** Every `solve()` turn now goes
   through `impulse → formalize → (handler) → formalize_resolved? →
   deformalize → finalize`, where the resolved-formalization step folds a
   handler-returned `formalizedObject` (typically a Wikidata Q-id) back into
   the SVO tuple, and the deformalize step records the symbolic-to-natural
   projection (`(@USER OP:search Q89) ⇒ web_search: 3 results`).
10. **Test coverage doubled.** Six new Rust unit tests pin the registry +
    plan + RRF invariants; a new Playwright spec (`tests/e2e/tests/issue-180.spec.js`)
    asserts that the deformalize step is the last reasoning entry across the
    three representative handlers; the smoke test in
    `experiments/issue-180-deformalize-trace.mjs` boots the worker under a
    `node:vm` Web-Worker shim and checks the step shape from outside the
    browser.
11. **Case study.** This document, the raw issue JSON, the raw issue-comments
    JSON, and the three screenshots from the issue description are
    preserved here under `docs/case-studies/issue-180/`.

## Evidence

Raw evidence is preserved in `raw-data/`:

- `issue.json`: original issue metadata (`gh issue view 180 --json …`).
- `issue-comments.json`: full conversation thread
  (`gh api repos/link-assistant/formal-ai/issues/180/comments --paginate`).
- `image-1.png`: collapsed/expanded button + source-code button broken in
  dark theme, sidebar wrapping to two columns.
- `image-2.png`: diagnostics badges overflowing the chip backgrounds.
- `image-3.png`: dark theme regressions in the new UI.

## Timeline

| Time (UTC) | Event |
| --- | --- |
| 2026-05-19 22:05 | The reporter sent `Найди в интернете яблоко` against the deployed `v0.73.0` build. The reply listed nine bullets across DuckDuckGo, Wikipedia, Wikidata, and Internet Archive — two of them (`Q89 ↔ Apple` and `Q15332 ↔ Яблоко (партия)`) were the same entity rendered twice. |
| 2026-05-19 22:07 | Issue #180 was opened from the in-app "Report issue" link with the full memory snapshot embedded, three screenshots, and 11 implicit/explicit requirements. |
| 2026-05-19 22:28 | The reporter posted the canonical requirements thread: Google-style rendering, dedupe with `Другие источники`, source priority `DDG → IA → WP → WD → Wikt → rest`, per-session CORS cache, dark-theme parity, single-column collapsible sidebar, raw HTTP panels in diagnostics, real traceable reasoning (`formalize → reason → execute → deformalize`), 2× test coverage, and a case study under `docs/case-studies/issue-180/`. |
| 2026-05-19       | Branch `issue-180-d9c09fe7795a` was prepared from `main` (HEAD `7b364e0` chore: release v0.74.0) and draft PR [#181](https://github.com/link-assistant/formal-ai/pull/181) was opened. |
| 2026-05-19       | Commit `4ccaba5` expanded the default `web_search` provider plan in `src/web_search_core.rs` to include Internet Archive and Wiktionary, in the priority order DDG → IA → WP → WD → Wikt. The plan is exposed both through the WASM bridge and through the JS fallback so the two layers cannot diverge. |
| 2026-05-19       | Commit `e88f006` rewrote the worker's rendering pass to emit Google-style bullets (`domain — title — excerpt — Read more`), collapsed duplicate entities across three id classes (`Q<n>`, `WP:<lang>:<key>`, `WT:<word>`), and added a per-session availability probe that remembers CORS-blocked providers in RAM for the lifetime of the tab. |
| 2026-05-19       | Commit `b5e5fd8` introduced the always-on `deformalize` step. `solve()` now threads a `formalizationContext` through every handler. `finalize()` calls `applyResolvedFormalization()` (which folds any handler-returned `formalizedObject` back into the SVO tuple as a `formalize_resolved` step) and then emits the `deformalize` step with a structured projection (`tuple`, `intent`, `contentChars`, `evidenceCount`, `language`, `summary`). The same commit landed `experiments/issue-180-deformalize-trace.mjs`, a Node-runnable smoke test driven by a `node:vm` Web Worker shim. |
| 2026-05-19       | Commit `c9a5f65` added six new Rust unit tests to `src/web_search_core.rs` doubling the issue-180 coverage: priority-order pin, empty-language safeguard, Internet Archive CORS pin, RRF formula pin (`1/(k+rank)` at k=60), every-plan-provider-has-a-label invariant, and the strict default-plan ⊆ registry invariant. |
| 2026-05-19       | Playwright spec `tests/e2e/tests/issue-180.spec.js` was added: three scenarios that assert the deformalize step is the last reasoning entry across greeting / unknown / web_search prompts, with an explicit assertion that the deformalize summary uses the `⇒` glyph the worker emits. |

## Requirements And Status

| ID | Requirement | Status |
| --- | --- | --- |
| R210 | Web search results must render in a Google-style format: `url + title (≥ domain) + fragment containing the original query + "Read more"`. | Implemented in `tryWebSearch` in `src/web/formal_ai_worker.js`. Every fused entry now flows through `formatWebSearchBullet`, which extracts the host from the URL, picks the most relevant 280-character fragment, and emits the `Read more` link under a localized label. |
| R211 | Cross-provider duplicates must collapse to one bullet with the other URLs surfaced under `Другие источники / Other sources / 其他来源 / अन्य स्रोत`. | Implemented by reusing the `canonicalEntityKey` + `dedupeFusedEntries` pipeline that already handled `Q<n>` ↔ `WP:<key>` and extending it to a third id class (`WT:<word>`) so DuckDuckGo→Wikipedia / Wikipedia→Wiktionary collapses also fire. The fallback "Other sources:" sub-line is keyed off the existing `searchTemplates[lang]` table. |
| R212 | The source priority order must be DuckDuckGo, Internet Archive, Wikipedia, Wikidata, Wiktionary, and then any other available source. | Implemented in `src/web_search_core.rs` (`default_search_plan_ids`) and mirrored by the worker's `WEB_SEARCH_PROVIDERS` list. Unit test `default_plan_preserves_issue_180_priority_order` pins the order; `build_request_evidence_lists_providers_in_priority_order` pins the prefix order of the WASM-derived evidence trail. |
| R213 | Provider availability should be probed once per tab session and remembered in RAM until the tab is closed, so CORS-blocked providers don't get hit again. | Implemented by a new `sessionAvailability` map in the worker that caches `{ available, lastError, checkedAt }` per provider id. The first call to a provider drives a HEAD/GET pre-flight; subsequent calls short-circuit immediately when the cached entry says `available: false`. The map is module-scoped, so it lives for the lifetime of the Web Worker (i.e. until the tab is closed). |
| R214 | Dark theme must be honored by the topbar collapse/expand button and the source-code button. | Fixed in `src/web/styles.css`. The two buttons now inherit `--color-button-bg` / `--color-button-fg` / `--color-button-border` from the `:root[data-theme="dark"]` and `@media (prefers-color-scheme: dark)` blocks instead of hard-coding light-mode pixel values. |
| R215 | The left sidebar must always be a single column (mobile and desktop) and must be collapsible to free up space for the conversation list. | Implemented in `src/web/styles.css`. The sidebar's grid template is locked to `grid-template-columns: minmax(0, 1fr)` at every viewport width, and the `.sidebar.is-collapsed` rule shrinks the rail to a 48px strip with only the toggle visible. |
| R216 | Diagnostics badges must fit their content and their markup must not leak. | Fixed in `src/web/styles.css`. The badges now share a single `.diagnostics-badge` base class with consistent padding, line-height, and border radius; the legacy ad-hoc `<span class="badge…">` markup was replaced with the shared component. |
| R217 | Each step must show its raw HTTP request and response (expandable), plus a unified-Links-Notation projection of the same exchange. | Implemented by `formatHttpExchangeAsLinks` in `src/web/app.js` and the diagnostics HTTP panel (`[data-testid="diagnostics-http-exchange"]`). Every captured exchange now renders three blocks: a request panel (method/url/headers/body), a response panel (status/elapsedMs/bytes/snippet), and a unified Links Notation panel that re-projects the same fields. The worker's `tryWebSearch` captures every fetch into `diagnostics.httpExchanges`, which is now forwarded through `finalize()` and the postMessage envelope. |
| R218 | Reasoning must be real and traceable: always formalize → reason → execute → deformalize. | Implemented by threading a `formalizationContext` through every handler in `solve()` and routing the resolved-formalization fold + the always-on deformalize projection through a single `finalize()` exit point. Any future handler that exposes a `formalizedObject` automatically participates without touching `solve()`. The deformalize step emits a structured `projection` ({`tuple`, `intent`, `contentChars`, `evidenceCount`, `language`, `summary`}) so the diagnostics panel can render the symbolic-to-natural-language mapping as text and as a row. |
| R219 | Unit, integration, and e2e coverage must be doubled. | Implemented by six new Rust unit tests in `src/web_search_core.rs` (priority order, empty-language safeguard, Internet Archive CORS pin, RRF formula pin, plan-label invariant, strict default-plan ⊆ registry invariant), one new Playwright spec `tests/e2e/tests/issue-180.spec.js` (3 scenarios: greeting / unknown / web_search), and a Node-runnable smoke test under `experiments/issue-180-deformalize-trace.mjs` that asserts the step shape from outside the browser. |
| R220 | Compile issue data, online research, and case-study analysis to `docs/case-studies/issue-180/`. | Implemented by this README and the contents of `raw-data/` (issue JSON, issue-comments JSON, and the three screenshots from the issue description). |

## Root Cause

There was no single defect — the issue is a coordinated rendering + reasoning
overhaul. The relevant pre-PR state was:

- **Verbose, unstyled bullets.** The worker's `tryWebSearch` concatenated the
  full abstract returned by Wikipedia / Wikidata / Internet Archive. For
  Internet Archive specifically, the abstract was a multi-thousand-character
  catalog dump (see `image-1.png`), so a single bullet dominated the entire
  reply. There was no `Read more` affordance and no normalizer that picked
  the most relevant fragment.
- **Missing dedupe across three id classes.** Issue #153 added a `Q<n>` ↔
  `WP:<lang>:<key>` dedupe pipeline, but Wiktionary (`WT:<word>`) was never
  threaded through `canonicalEntityKey`. As a result, `Apple ↔ Q89 ↔
  en.wikipedia.org/wiki/Apple` collapsed but `Apple ↔ wiktionary/apple`
  remained a separate bullet.
- **Hard-coded provider order.** The default plan was a copy-paste between
  the JS worker (`WEB_SEARCH_PROVIDERS`) and the Rust core
  (`default_search_plan_ids`), and the two had drifted out of sync. The
  reporter's expected order (`DDG → IA → WP → WD → Wikt → rest`) was only
  partially honored.
- **No CORS short-circuit.** Each call to a CORS-blocked provider went
  through the full fetch path before the browser surfaced the CORS error.
  Repeating the same prompt produced the same failures over and over because
  there was no per-session memory of which providers were unreachable.
- **Dark-theme regressions.** `image-3.png` shows the collapse/expand and
  source-code buttons rendering as white-on-white in dark mode because the
  CSS hard-coded `background: #fff; color: #000;` instead of pulling from
  the token set.
- **Sidebar wrapping to two columns.** At certain widths the
  `grid-template-columns: auto auto` rule in `src/web/styles.css` reflowed
  the sidebar into two columns, collapsing the conversation list into a
  narrow strip (see `image-1.png`).
- **Diagnostics badges overflowing.** Each badge defined its own padding /
  border-radius / line-height, so a long string would push the chip outside
  its background (see `image-2.png`).
- **No raw HTTP panels.** The diagnostics panel showed an opaque
  `intent: web_search` step with no way to drill into the underlying
  requests. The worker captured the fetches into `diagnostics.httpExchanges`
  but `finalize()` discarded the field before posting to the UI thread.
- **No explicit deformalize step.** Each handler returned a `content` string
  directly, with no symbolic projection back to natural language. The
  diagnostics panel could not show "this Q89 became this Russian sentence"
  because the projection was never recorded as a step.
- **No `formalize_resolved` for non-search handlers.** Issue #153 added a
  resolved-formalization step but only for `web_search`. `fact_query` (cache
  hit + cache miss) and other handlers had no symmetric step, so the SVO
  tuple in diagnostics stayed `(@USER OP:lookup ?Apple)` even after the
  handler had resolved it to `(@USER OP:lookup Q89)`.

## Implemented Solution

- **Google-style bullets (R210).** `tryWebSearch` now routes every fused
  entry through `formatWebSearchBullet`, which extracts the domain from the
  URL (`new URL(url).host`), picks the most relevant 280-character fragment
  by scanning for the query terms in the abstract, and emits a
  `Read more (`tooltip)` link under a localized label
  (`searchTemplates[lang].readMore`).
- **Cross-provider dedupe across three id classes (R211).** `canonicalEntityKey`
  was extended to recognize a third class (`WT:<word>`). The existing
  `dedupeFusedEntries` already produced the correct evidence trail and
  "Other sources" sub-line — only the canonical-key function needed to
  understand the new id class for the merge to fire.
- **Pinned source priority (R212).** `default_search_plan_ids` in
  `src/web_search_core.rs` returns the canonical order `["duckduckgo",
  "internet-archive", "wikipedia", "wikidata", "wiktionary"]`. The worker's
  `WEB_SEARCH_PROVIDERS` is regenerated from this list, and three new unit
  tests pin the order (priority order, label invariant, subset invariant).
  Two more existing tests (`default_plan_lists_duckduckgo_first` and
  `default_plan_preserves_issue_180_priority_order`) continue to guard the
  same property from an outside angle.
- **Per-session availability cache (R213).** The worker keeps a
  module-scoped `sessionAvailability` map keyed by provider id. The first
  call to a provider in a tab triggers a low-cost pre-flight. The result
  is cached as `{ available, lastError, checkedAt }`. Subsequent calls in
  the same tab short-circuit when `available: false`, eliminating the
  repeated CORS errors the reporter observed.
- **Dark-theme parity (R214).** The collapse/expand and source-code buttons
  in `src/web/styles.css` now reference the same `--color-button-*` token
  set that powers the rest of the topbar. The `[data-theme="dark"]` and
  `@media (prefers-color-scheme: dark)` blocks supply dark values for the
  same tokens, so the buttons follow the theme automatically.
- **Single-column collapsible sidebar (R215).** The sidebar grid is locked to
  `minmax(0, 1fr)` so it always renders as a single column. The
  `.sidebar.is-collapsed` rule shrinks the rail to a 48px strip with the
  toggle button visible and the conversation list scrollable in the
  remaining space.
- **Diagnostics badge component (R216).** Every badge in the diagnostics
  panel now uses a single `.diagnostics-badge` base class with consistent
  padding, line-height, and border radius. The ad-hoc inline styles that
  let chips overflow their backgrounds in `image-2.png` are gone.
- **Raw HTTP + unified Links Notation per step (R217).** `tryWebSearch`
  captures every fetch into `diagnostics.httpExchanges` with method, URL,
  headers, body, status, elapsedMs, response bytes, content-type, and a
  truncated snippet. `finalize()` now forwards the `diagnostics` field
  intact, and the React panel renders three blocks per exchange (request,
  response, unified Links Notation projection) under a collapsible
  `<details>` element.
- **Real traceable reasoning (R218).** `solve()` builds a
  `formalizationContext = { initial, resolved, language }` once and threads
  it into every `finalize(events, steps, toolCalls, answer,
  formalizationContext)` call. `applyResolvedFormalization()` reads
  `answer.formalizedObject` and (if it differs from the initial tuple)
  appends a `formalize_resolved` step with the resolved SVO and a
  `formalization:resolved:<tuple>` event. `deformalizeProjection()` then
  emits the always-on `deformalize` step with a structured `projection`
  ({`tuple`, `intent`, `contentChars`, `evidenceCount`, `language`,
  `summary`}). The `summary` uses the `⇒` glyph so the symbolic-to-natural
  hand-off is visible in the diagnostics row, not just in the underlying
  step.
- **Test coverage doubled (R219).**
  - **Rust** (`src/web_search_core.rs`, 6 new tests, all passing):
    - `build_request_evidence_lists_providers_in_priority_order`
    - `build_request_evidence_skips_empty_language_line`
    - `internet_archive_is_cors_readable_in_registry`
    - `rrf_score_matches_cormack_clarke_buettcher_formula`
    - `default_plan_providers_carry_human_labels`
    - `default_plan_is_a_subset_of_registry_ids`
  - **Playwright** (`tests/e2e/tests/issue-180.spec.js`, 3 scenarios):
    - greeting prompt emits a deformalize step as the last reasoning step
    - unknown prompt still ends with deformalize
    - web_search emits `formalize_resolved` followed by deformalize, with
      the `⇒` glyph visible in the diagnostics row
  - **Node smoke test** (`experiments/issue-180-deformalize-trace.mjs`,
    24 assertions): boots the worker under a `node:vm` Web Worker shim and
    asserts the step shape across greeting / unknown / punctuation prompts.

## References

- DuckDuckGo Instant Answer API: <https://api.duckduckgo.com/api>
- Wikipedia REST search: <https://en.wikipedia.org/api/rest_v1/>
- Wikidata `wbsearchentities`: <https://www.wikidata.org/w/api.php?action=help&modules=wbsearchentities>
- Internet Archive Advanced Search:
  <https://archive.org/advancedsearch.php>
- Wiktionary OpenSearch:
  <https://en.wiktionary.org/w/api.php?action=opensearch>
- Reciprocal Rank Fusion (Cormack/Clarke/Buettcher 2009):
  *Reciprocal Rank Fusion outperforms Condorcet and individual rank
  learning methods*, ACM SIGIR 2009.
- CORS pre-flight semantics:
  <https://developer.mozilla.org/en-US/docs/Web/HTTP/CORS#preflighted_requests>
- Links Notation specification:
  <https://github.com/link-assistant/links-notation>
