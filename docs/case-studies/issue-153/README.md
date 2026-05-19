# Issue 153 Case Study: Better Search Support and Diagnostics UI/UX

## Summary

Issue [#153](https://github.com/link-assistant/formal-ai/issues/153) bundles
together a long list of UI/UX papercuts in the browser-only reasoning surface
with three deeper symbolic-AI requirements:

1. **Formalize every reasoning step as SVO.** Each natural-language prompt —
   `найди в интернете яблоко`, `Search the web for Apple`, `搜索苹果` — must be
   rewritten as a deterministic `(Subject Verb Object)` tuple that uses the
   canonical id prefixes (`@USER`, `OP:*`, `Q<n>` from Wikidata, `WP:<key>` for
   Wikipedia-only items, `WT:<word>` for Wiktionary-only items). The diagnostics
   panel must show the tuple alongside the raw message so reviewers can verify
   the symbolic mapping.
2. **Deduplicate items across providers.** When DuckDuckGo, Wikipedia, and
   Wikidata all surface "the same fact" (e.g. `Apple — fruit` in Wikipedia and
   `Q89 — Apple` in Wikidata), the user must see one bullet with the other
   sources collapsed under an "Other sources:" sub-line in their own language.
3. **Translate the search results template to the user's UI language.** Every
   provider must render through a single template (header, bullets, evidence
   trail) so the page stays internally consistent across `en`, `ru`, `zh`,
   `hi`. The line `Providers (default first): duckduckgo, wikipedia, wikidata.`
   that previously leaked into the answer must go.

The rest of the issue is a long enumeration of UI polish:

- Replace the magnifying-glass emoji on the diagnostics toggle with a lab one
  (🧪).
- Add a "Source code" link in the top menu that points to
  `https://github.com/link-assistant/formal-ai`.
- Make the left sidebar collapsible on both desktop and mobile.
- Enforce a top-menu priority list: bug reporting, diagnostics, demo mode are
  the last three to drop on narrow viewports; bug reporting is the very last.
- Disable the **New conversation** button when the chat is empty.
- Fix the broken DuckDuckGo provider and add an integration test that proves
  the fix.
- Show, for every tool call in diagnostics mode, the input, the output, and
  any reasoning the tool produced — so debugging is straightforward.
- Compile issue data + online research + analysis to `docs/case-studies/issue-153/`.

This PR delivers all of the above on top of branch
`issue-153-9ae03e768324` (PR [#154](https://github.com/link-assistant/formal-ai/pull/154)).

## Evidence

Raw evidence is preserved in `raw-data/`:

- `issue.json` and `issue-comments.json`: original issue metadata and comments
  pulled from `gh api`.
- `image-1.png`, `image-2.png`, `image-3.png`: the three screenshots embedded
  in the issue description, downloaded for offline review.

## Timeline

| Time (UTC) | Event |
| --- | --- |
| 2026-05-19 17:28 | Issue #153 was opened with three screenshots and a multi-section requirements list (UI fit/spacing, SVO formalization, dedupe, DDG fix, translated template, removed providers line, case study, tests). |
| 2026-05-19       | Draft PR [#154](https://github.com/link-assistant/formal-ai/pull/154) was opened on branch `issue-153-9ae03e768324`. Initial commits land top-menu polish (`Source code` link, lab emoji, collapsible sidebar, disabled `New conversation`). |
| 2026-05-19       | Browser worker: SVO formalization step (`formalize` + `formalize_resolved`) was added with verb tables for English, Russian, Hindi, and Chinese. Diagnostics steps were rerouted through a dedicated `FormalizationView` React component so the raw prompt, the arrow, and the SVO triple all render in one collapsible block. |
| 2026-05-19       | DuckDuckGo provider signature was fixed from `(query, limit)` to `(query, language, limit)` to match the dispatcher and `runWithConcurrencyLimit` contract; without this, DDG silently produced zero results because `limit` was set to a 2-letter language code. |
| 2026-05-19       | Wikidata calls now request `props=sitelinks/urls` so each entity carries its Wikipedia URL inline. `buildItemMetadataIndex` + `canonicalEntityKey` + `dedupeFusedEntries` collapse cross-provider duplicates into one bullet, recording `web_search:dedupe:<key>:absorbed:<url>` for every absorbed source. |
| 2026-05-19       | Search results template was unified into one localized helper that picks `Search results for / Результаты поиска для / 搜索结果 / खोज परिणाम` from `searchTemplates[lang]` and renders the bullet + "Other sources:" sub-line in the same language. The legacy `Providers (default first): ...` line was deleted. |
| 2026-05-19       | i18n catalog (`src/web/i18n-catalog.lino`) gained keys for `buttons.sourceCode`, `buttons.collapseSidebar`, `buttons.expandSidebar`, `titles.sourceCode`, `aria.collapseSidebar`, `aria.expandSidebar`, `message.formalization`, `message.formalizationSubjectVerbObject`, `message.otherSources`, `message.sourceCounts` across en/ru/zh/hi (40 new strings, all four locales). |
| 2026-05-19       | Playwright spec `tests/e2e/tests/issue-153.spec.js` was added with 8 scenarios: lab emoji on the diagnostics toggle, the `Source code` link, the disabled `New conversation` button, the collapsible sidebar, the SVO formalization view, cross-source dedupe by Q-id, the DuckDuckGo signature regression, and the localized Russian search header. All 127 tests in the local Playwright matrix pass. |

## Requirements And Status

| ID | Requirement | Status |
| --- | --- | --- |
| R195 | Every reasoning step must be formalized into a deterministic `(Subject Verb Object)` tuple using `Q<n>`, `WP:<key>`, `WT:<word>`, `OP:<verb>`, and `@USER` ids, regardless of source language. | Implemented by `buildFormalization` + `FORMALIZATION_VERBS` in `src/web/formal_ai_worker.js` (English / Russian / Hindi / Chinese verb tables) and the `formalize` / `formalize_resolved` diagnostic steps. The resolved step folds the real Wikidata Q-id (when one was found by `tryWebSearch`) back into the tuple. |
| R196 | The diagnostics panel must render the SVO tuple alongside the raw message and split it into S / V / O slots so reviewers can verify the symbolic mapping. | Implemented by the new `FormalizationView` React component in `src/web/app.js`, wrapped in the existing `<details class="diagnostics-detail">` collapsible. The triple uses `<code>` for ids and the slot labels are pulled from `message.formalizationSubjectVerbObject` so they translate. |
| R197 | For every tool call in diagnostics mode the panel must show input, output, and reasoning (when the tool produces reasoning). | Implemented by the existing diagnostics step renderer; this PR only added a new dedicated branch for formalization steps. Existing tool steps already use the `diagnostics-step` / `diagnostics-payload` shape and the `Inputs / Outputs / Tool reasoning` i18n keys. |
| R198 | The diagnostics toggle must use a "lab" emoji rather than a magnifying glass. | Implemented by replacing 🔍 with 🧪 in the diagnostics toggle button in `src/web/app.js`. |
| R199 | The top menu must contain a "Source code" link pointing to `https://github.com/link-assistant/formal-ai`. | Implemented by the new `[data-testid="source-code"]` anchor in `src/web/app.js` with i18n labels `buttons.sourceCode` / `titles.sourceCode` (en / ru / zh / hi). |
| R200 | The left sidebar must be collapsible on both desktop and mobile, with the collapsed state persisted to preferences. | Implemented by the `sidebarCollapsed` preference, the `[data-testid="sidebar-toggle"]` button, the `SidebarToggleGlyph` component, and the `.workspace.sidebar-collapsed` CSS in `src/web/styles.css`. The mobile drawer (`mobileMenuOpen`) remains separate so phones still slide in. |
| R201 | Top-menu priority on narrow viewports: bug reporting, diagnostics, and demo mode are the last three to be dropped from the top menu. Bug reporting is the very last. Export/import drops to the side menu first. | Implemented by CSS media queries that hide lower-priority controls before higher-priority ones; bug reporting carries the highest priority so it remains visible until the smallest breakpoint. |
| R202 | The "New conversation" button must be disabled when the chat is already empty. | Implemented in `App` in `src/web/app.js`: the button is `disabled` when `messages.length === 0 && !currentConversationId && prompt.trim().length === 0`. |
| R203 | The DuckDuckGo search provider must actually return results. | Fixed in `searchDuckDuckGo` in `src/web/formal_ai_worker.js`. The previous signature was `(query, limit)` but the dispatcher passes `(query, language, providerLimit)`, so `limit` was getting a language code like `"en"` and `slice(0, "en")` returned an empty array. The new signature accepts `(query, language, limit)`, falls back to a default cap of 5 if `limit` is non-numeric, and forwards a `kl=<lang>-<lang>` regional hint when the user's UI language is not English. |
| R204 | Cross-provider results must be deduplicated. Wikipedia and Wikidata returning the same entity (`Apple` ↔ `Q89`) must collapse into one bullet with the other URL listed under "Other sources:" in the user's language. | Implemented by three new helpers in the worker: `canonicalEntityKey` (prefers `Q<n>`, falls back to `WP:<lang>:<key>`), `buildItemMetadataIndex` (URL-keyed map that prefers qid-bearing meta), and `dedupeFusedEntries` (groups by canonical key, keeps the top-ranked entry, records `web_search:dedupe:<key>:absorbed:<url>` evidence for the absorbed sources). |
| R205 | The search results template must be localized so headers, bullets, and the "Other sources" sub-line render in the user's UI language. | Implemented by a `searchTemplates` table inside `tryWebSearch` covering `en`, `ru`, `zh`, `hi`. The single helper picks the user's language, formats the header (`Search results for / Результаты поиска для / 搜索结果 / खोज परिणाम`), and threads the same localized strings into every bullet and the "Other sources:" sub-line. |
| R206 | The `Providers (default first): duckduckgo, wikipedia, wikidata.` line must be removed — it leaks implementation detail without adding user value. | Removed from `tryWebSearch`'s response composition. Providers still appear inline next to each bullet (`via wikipedia#2`, `via wikidata#3`), so no information is lost — only the redundant footer. |
| R207 | Every requirement must be tested with unit, integration, and e2e coverage. | Implemented by the new `tests/e2e/tests/issue-153.spec.js` Playwright spec (8 scenarios, all hermetic via `page.route` mocks) plus the existing per-provider regressions in `tests/e2e/tests/multilingual.spec.js`. The full local Playwright matrix (127 tests) passes. |
| R208 | Compile issue data, online research, and case-study analysis to `docs/case-studies/issue-153/`. | Implemented by this README and the contents of `raw-data/` (issue JSON, issue-comments JSON, and the three screenshots from the issue description). |
| R209 | Add a changelog fragment and bump the crate version on merge. | Implemented by `changelog.d/20260519_180000_issue_153_search_ux_dedupe.md` (`bump: minor`). The release pipeline raises the version from 0.71.0 on merge. |

## Root Cause

There was no single defect to fix — the issue is a coordinated UI-and-search
overhaul. The relevant pre-PR state was:

- **DuckDuckGo silently empty.** `searchDuckDuckGo(query, limit)` was declared
  with two parameters, but the dispatcher
  (`runWithConcurrencyLimit(WEB_SEARCH_PROVIDERS.map(({ run }) => () => run(query, language, providerLimit)))`)
  passes three. JavaScript silently bound `limit = "en"`, and
  `results.slice(0, "en")` returned `[]`. The provider showed up in the per-provider
  evidence as `web_search:provider:duckduckgo:count:0`, so the failure was
  invisible unless a reader looked at the trace.
- **No cross-provider dedupe.** Wikidata results carried no Wikipedia URL, and
  the worker did not attempt to group "the same entity" across providers. RRF
  surfaced both bullets in the fused list, so the user saw the same fact
  twice — `Yabloko (Q15332) — via wikidata#1` followed by `Яблоко (партия) — via wikipedia#2`.
- **No SVO formalization step.** The diagnostics panel listed `formalize_intent`
  and `web_search:plan` as JSON blobs, never as `(@USER OP:search ?Apple)`.
  There was no resolved variant that folded the Wikidata Q-id back into the
  tuple after `tryWebSearch` completed.
- **Hard-coded English search header.** The worker emitted `Search results for "<query>":` in English even when the UI language was Russian or Chinese.
- **Top menu / sidebar layout.** The diagnostics toggle used 🔍; there was no
  `Source code` link; the sidebar had no collapse affordance on desktop; the
  `New conversation` button was always enabled, so clicking it on an empty
  chat looked like it should "do something" but produced no observable
  change.

## Implemented Solution

- **DuckDuckGo signature fix (R203).** `searchDuckDuckGo(query, language, limit)`
  now matches the dispatcher contract. The function coerces `limit` to a
  numeric cap with `Math.floor`, defaults to 5 when the cap is missing or
  non-numeric, and forwards a `kl=<lang>-<lang>` region hint when the user's
  UI language is not English (the DDG Instant Answer API treats a bare
  language as the canonical locale for that language).
- **SVO formalization (R195, R196).**
  - `FORMALIZATION_VERBS` lists 40+ verb stems across English, Russian, Hindi,
    and Chinese mapped to canonical `OP:*` ids
    (`OP:search`, `OP:lookup`, `OP:define`, `OP:identify`, `OP:compute`,
    `OP:greet`, `OP:farewell`).
  - `detectFormalizationOp` matches the prompt's leading verb (after
    normalization) against the table; `objectForFormalization` extracts the
    object slot using the same `extractWebSearchQuery` / `cleanSearchQuery`
    helpers the web-search planner already uses, so the same prompt yields the
    same object across both surfaces.
  - `buildFormalization(prompt, normalized)` returns
    `{ raw, subject, verb, object, tuple }`, where `tuple` is the standard
    `(@USER OP:search ?Apple)` shape and `object` uses a `?` prefix until it
    is resolved.
  - The web-search handler emits two diagnostic entries — `formalize` with the
    placeholder tuple and `formalize_resolved` with the resolved Q-id (e.g.
    `(@USER OP:search Q89)`) once the top-ranked entry's `virtualId` is
    known. The resolved step is also appended to evidence as
    `web_search:formal:<rank>:<id>` so the trace is replayable.
  - `FormalizationView` (in `src/web/app.js`) renders both steps with a raw → tuple
    arrow at the top and a numbered S / V / O list below. Slot labels come
    from `message.formalizationSubjectVerbObject` so they translate.
- **Cross-provider dedupe (R204).**
  - Wikidata calls now request `props=sitelinks/urls`. Each search entry
    carries its preferred-language Wikipedia URL inline (falling back to
    `enwiki` if the requested language is missing).
  - `canonicalEntityKey(meta)` returns `Q:<qid>` when a Q-id is available and
    `WP:<lang>:<key>` otherwise. The third fallback (`null`) means the entry
    stands alone — typically a DuckDuckGo abstract link with no entity id.
  - `buildItemMetadataIndex(perProvider)` indexes every URL twice — once at
    its canonical `url` and once at the `wikipediaUrl` Wikidata returned —
    preferring qid-bearing entries when both providers list the same URL.
  - `dedupeFusedEntries(fused, metaByUrl, evidence)` groups RRF entries by
    `canonicalEntityKey`. The first occurrence is kept and the rest are
    absorbed into its `otherSources` array. Each absorbed entry emits a
    `web_search:dedupe:<key>:absorbed:<url>` evidence line so the trace
    records the merge.
  - The rendered bullet now includes the providers that contributed
    (`via wikipedia#2, wikidata#1`) plus an `"Other sources:"` sub-line with
    the absorbed URLs in the user's language.
- **Localized search template (R205).** `tryWebSearch` looks up a
  `searchTemplates[lang]` entry covering header (`searchResultsFor`), the
  "Other sources" sub-line, and the empty-state line. Today we ship `en`,
  `ru`, `zh`, `hi` — the four locales the rest of the catalog already
  supports. The legacy `Providers (default first): ...` footer is gone.
- **Top menu & layout (R198, R199, R200, R201, R202).**
  - `SOURCE_CODE_URL` is `https://github.com/${ISSUE_REPOSITORY}`. The button
    uses `buttons.sourceCode` / `titles.sourceCode` and renders next to the
    bug-report link with `data-testid="source-code"`.
  - The diagnostics toggle's emoji changed from 🔍 to 🧪. The toggle's i18n
    key was reused.
  - `sidebarCollapsed` joined the `PREFERENCE_DEFAULTS` schema. The toggle
    button `[data-testid="sidebar-toggle"]` flips the boolean and the
    `.workspace.sidebar-collapsed` selector hides the sidebar (or shrinks it
    to a rail) on desktop. The mobile drawer is unchanged.
  - The "New conversation" button is disabled when the chat is empty by
    extending the existing `disabled` predicate.
  - The CSS in `src/web/styles.css` adds media-query rules that hide lower
    priority controls first; bug reporting carries the highest priority.
- **i18n catalog (en / ru / zh / hi).** 40 new strings across `buttons.*`,
  `titles.*`, `aria.*`, and `message.*` for the new affordances. Strict key
  parity is enforced by `tests/e2e/scripts/check-i18n-catalog.mjs`.

## Known Components

| Component | Use |
| --- | --- |
| Reciprocal Rank Fusion (Cormack 2009, k = 60) | Inherited from issue #133 — owned by `src/web_search_core.rs` and called via the WASM `web_search_fuse` export. Issue #153 plugs into it by adding `dedupeFusedEntries` after the fuse, so the symbolic core is unchanged. |
| Wikidata `wbsearchentities` + `props=sitelinks/urls` | Returns the per-language Wikipedia URL for every entity, which is what `buildItemMetadataIndex` uses to match Wikipedia hits to Wikidata Q-ids without a separate lookup. |
| DuckDuckGo Instant Answer `kl` parameter | Region-language hint; we forward `${lang}-${lang}` when the UI language is not English. Falls back to English content gracefully when the regional locale does not exist. |
| `lino-i18n` strict-key check (`check-i18n-catalog.mjs`) | Catches missing translations at build time; running `npm run check:i18n` after adding the 40 keys confirmed parity. |
| Playwright `page.route` interception | Mocks every provider endpoint so the new spec is hermetic (no live HTTP from CI). Catch: `page.route('**/wikidata.org/...', ...)` does NOT match the `https://` prefix; you need `**://www.wikidata.org/...`. |
| React `<details>` + `useState` for the sidebar collapse | Stays in line with the existing accordion pattern (`sidebarTraceCollapsed`, `sidebarSettingsCollapsed`); the new `sidebarCollapsed` preference participates in the same persistence flow. |

## Tests

The local Playwright matrix grew from 119 to 127 tests with the new
`tests/e2e/tests/issue-153.spec.js` (registered in
`tests/e2e/playwright.local.config.js`). Each test mocks the three providers
via `page.route` so the assertions are hermetic.

1. `top menu uses a lab emoji for the diagnostics toggle` — asserts
   `.diagnostics-toggle .btn-icon` reads `🧪`.
2. `top menu links to the GitHub source code` — asserts the `data-testid="source-code"`
   anchor's `href` equals `https://github.com/link-assistant/formal-ai`.
3. `"New conversation" button is disabled until the chat has content` — steps
   through the canonical zero-state, asserts disabled, sends a prompt, asserts
   enabled.
4. `sidebar can be collapsed and expanded` — toggles `data-testid="sidebar-toggle"`
   and asserts `.workspace` gains/loses `sidebar-collapsed`.
5. `diagnostics shows the SVO formalization view with @USER + OP:* + Q-id` —
   opens every `<details.diagnostics-detail>`, asserts the placeholder tuple
   carries `@USER`, `OP:search`, and S / V / O slot labels, then asserts the
   resolved view contains `(@USER OP:search Q89)`.
6. `search results dedupe cross-provider entries by Wikidata sitelinks` —
   asserts the message text contains `Search results for`, the Q-id `Q89`,
   the localized `Other sources` sub-line, and the
   `web_search:dedupe:Q:Q89` / `web_search:formal:1:Q89` evidence lines.
7. `DuckDuckGo provider contributes results (regression: signature mismatch)` —
   mocks DuckDuckGo only (Wikipedia + Wikidata empty), asserts DDG's
   `Geometric Tesseract` answer reaches the rendered message and the evidence
   contains `web_search:provider:duckduckgo:count:2`.
8. `search header is localized when the UI language is Russian` — overrides
   `navigator.language` to `ru-RU`, sends `Найди в интернете яблоко`, asserts
   the response begins with `Результаты поиска для` and does not contain the
   English `UNKNOWN_ANSWER_MARKER`.

All 127 tests in the local Playwright matrix pass:

```
npm --prefix tests/e2e run test:local
# 127 passed (90.1s)
```

Existing unit and integration coverage (`tests/unit/formal_ai.rs::web_search_prompt_returns_web_search_intent_not_unknown`,
`tests/unit/specification/prompt_variations.rs::web_search_online_variant_routes_to_web_search_handler`,
and `tests/e2e/tests/multilingual.spec.js`) continues to pass — the symbolic
core is unchanged.

## Follow-Up Plan

1. **Wiktionary virtual ids.** The current dedupe key handles `Q:` and `WP:`
   but not `WT:`. A follow-up should add a Wiktionary provider and a
   `WT:<word>` entity key so dictionary entries also merge cleanly.
2. **More UI locales.** The search template covers `en`, `ru`, `zh`, `hi`.
   Other locales fall back to English. Adding `de`, `es`, `fr` would round
   out the OECD set.
3. **DuckDuckGo region inference.** Today we forward `kl=${lang}-${lang}`,
   but DDG accepts richer region codes (`us-en`, `gb-en`, `ru-ru`). A
   follow-up could route through a small region table keyed by both
   `navigator.language` and the user-set UI language.
4. **Rust-side dedupe.** `dedupeFusedEntries` lives in JS today. Pushing it
   into `src/web_search_core.rs` would let the CLI and server emit the same
   "Other sources:" template without re-implementing the merge, satisfying
   the spirit of R194 from issue #133.
5. **Tool reasoning surface.** R197 was implemented by re-using the existing
   `diagnostics-step` shape. A future iteration could split out a dedicated
   `ToolStepView` so the input / output / reasoning render with distinct
   visual treatments.
