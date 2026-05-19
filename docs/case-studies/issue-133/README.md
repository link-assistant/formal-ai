# Issue 133 Case Study: DuckDuckGo Default, Combined Ranking, and Expanded Provider Diagnostics

## Summary

Issue [#133](https://github.com/link-assistant/formal-ai/issues/133) asks for
three coupled improvements to formal-ai's web reasoning surface:

1. Make DuckDuckGo the default search engine across CLI, server, and the
   browser-only GitHub Pages app so behavior stays consistent.
2. When several search engines are available, combine the top-10 results from
   each into a single ranked list so URLs that appear in more than one engine
   bubble up.
3. Expand the browser-only diagnostics at
   `https://link-assistant.github.io/formal-ai/tests` with more search engines,
   knowledge databases, code-hosting providers, and scientific paper providers,
   and actually trigger external API calls for reasoning so every step is
   recorded in exportable memory.

The issue also asks that every reasoning step be appended to the memory log
with request, response, and unified-link interpretation; that we run up to five
parallel calls per category; and that providers which CORS-block be temporarily
disabled for the session. As much logic as possible should live in
Rust→WebAssembly with JavaScript reserved for UI; this PR keeps the worker JS
in lock-step with the Rust solver for the new contract and tracks the full
Rust→WASM port as a follow-up so this PR stays reviewable.

This PR delivers the DuckDuckGo-first multi-provider plan, reciprocal rank
fusion (Cormack 2009, `k = 60`), per-category concurrency cap of five, an
auto-disable map for CORS/network failures, expanded diagnostics covering
search, knowledge, papers, and code-hosting categories, structured
`web_search:*` evidence in both stacks, and refreshed e2e coverage that mocks
all three default providers so tests stay deterministic.

## Evidence

Raw evidence is preserved in `raw-data/`:

- `formal-ai/issue-133.json` and `formal-ai/issue-133-comments.json`: original
  issue metadata and comments.
- `formal-ai/pr-134.json`: pre-implementation draft PR state.
- `formal-ai/branch-log.txt`: commit log for `issue-133-3a851f322caa` at the
  time of the case-study capture.
- `online-research/sources.md`: provider docs, CORS reference, browser
  per-origin socket budget, and the Reciprocal Rank Fusion paper used to
  design the multi-engine planner.

## Timeline

| Time (UTC) | Event |
| --- | --- |
| 2026-05-19 14:22 | Issue #133 was opened with three coupled requirements: DuckDuckGo default, combined ranking, and an expanded browser-only diagnostics matrix. |
| 2026-05-19 | Draft PR #134 was opened on branch `issue-133-3a851f322caa`. |
| 2026-05-19 | DuckDuckGo Instant Answer endpoint was confirmed CORS-readable; Wikipedia REST and Wikidata `wbsearchentities` already worked from the dashboard. |
| 2026-05-19 | The Rust solver was updated to emit `web_search:request`, per-provider `web_search:provider`, and `web_search:combined:rrf:k=60`. The shared provider list `WEB_SEARCH_PROVIDERS = ["duckduckgo", "wikipedia", "wikidata"]` became the contract for CLI/server/browser surfaces. |
| 2026-05-19 | The JS worker gained `searchDuckDuckGo`, `searchWikipediaWebProvider`, `searchWikidataEntities`, a `runWithConcurrencyLimit` (cap 5), and `reciprocalRankFusion`. CORS or network failures call `webSearchDisable` so the provider is skipped for the rest of the session. |
| 2026-05-19 | The connectivity dashboard was expanded from 11 to 26 rows across `search`, `knowledge`, `papers`, and `code` categories with a per-category cap of 5 and the same session-scoped disable map. |
| 2026-05-19 | The Playwright explicit web-search regression was rewritten to mock all three default providers and assert the new RRF evidence shape. |

## Requirements And Status

| ID | Requirement | Status |
| --- | --- | --- |
| R181 | DuckDuckGo must be the default search engine in CLI, server, and the browser-only GitHub Pages app. | Implemented by `WEB_SEARCH_PROVIDERS = &["duckduckgo", "wikipedia", "wikidata"]` in `src/solver_handlers/web_requests.rs` and the matching `WEB_SEARCH_PROVIDERS` array in `src/web/formal_ai_worker.js`; DuckDuckGo is first in both lists. |
| R182 | The top-10 results from each available search engine must be combined into a single ranked list so URLs returned by more than one engine bubble up. | Implemented by `reciprocalRankFusion` in the JS worker (Cormack 2009, `k = 60`). Each provider returns up to `WEB_SEARCH_PROVIDER_LIMIT = 10` results. The fused list keeps the higher-ranked title/excerpt and is sorted by combined score with a provider-count tiebreaker. |
| R183 | Both stacks must record the chosen reciprocal rank fusion constant in memory so the reasoning trace stays comparable offline. | Implemented by the `web_search:combined:rrf:k=60` event appended in both `src/solver_handlers/web_requests.rs` and `src/web/formal_ai_worker.js`, with a matching formatter branch in `src/event_log.rs::build_evidence_links`. |
| R184 | The browser-only diagnostics page must add more popular search engines beyond Google/Bing/DuckDuckGo/Brave/Yahoo. | Implemented by adding Yandex, Ecosia, Mojeek, and Startpage to `SERVICES` in `src/web/tests/connectivity.js`. |
| R185 | The diagnostics page must add code-hosting providers including GitHub, GitLab, Bitbucket, and providers from China and Russia. | Implemented by the new `code` category covering GitHub, GitLab, Codeberg, Gitee (China), Bitbucket Cloud, and GitFlic (Russia). |
| R186 | The diagnostics page must add scientific paper providers, only those without paywalls. | Implemented by the new `papers` category covering arXiv, Europe PMC, and DOAJ; Semantic Scholar, OpenAlex, and Crossref already cover open citation data. |
| R187 | The diagnostics page must add general knowledge providers beyond Wikipedia/Wikidata. | Implemented by adding Wiktionary and DBpedia Lookup to the `knowledge` category. |
| R188 | Test cases must actually trigger external API access for reasoning instead of memoized snapshots. | Implemented by the worker default-mode dispatch through `tryWebSearch`, which performs real `fetch()` calls against DuckDuckGo, Wikipedia REST, and Wikidata. Tests mock the same endpoints with `page.route` so they stay deterministic in CI while exercising the real call path. |
| R189 | Each reasoning step must be recorded in exportable memory so the full request, response, and unified-link interpretation can be replayed. | Implemented by the `web_search:request`, `web_search:provider`, `web_search:language`, `web_search:rank`, `web_search:fused`, `web_search:combined`, and `web_search:disabled` event kinds, with formatter branches in `src/event_log.rs::build_evidence_links`. |
| R190 | The planner must run providers in parallel with a cap of five per category. | Implemented by `runWithConcurrencyLimit` (cap `WEB_SEARCH_CONCURRENCY = 5`) in the worker and by the matching `CATEGORY_CONCURRENCY = 5` runner in the dashboard. |
| R191 | When a provider CORS-blocks or fails the network, the planner must temporarily disable it for the rest of the session and record the decision. | Implemented by the `WEB_SEARCH_DISABLED` map in the worker and the `state.disabled` map in the dashboard. Both emit a `web_search:disabled:<provider>` / `disabled:` log entry and skip the provider until the page reloads. |
| R192 | Issue data, online research, and case-study analysis must be compiled to `docs/case-studies/issue-133/`. | Implemented by this README and the contents of `raw-data/` (issue, PR, branch log, and online research). |
| R193 | A changelog fragment must record the user-visible change and trigger an automated minor crate-version bump. | Implemented by `changelog.d/20260519_140000_issue_133_default_duckduckgo_rrf.md`, which declares `bump: minor` so the release pipeline raises the version from 0.69.0 on merge. |

### Out of scope for this PR

| ID | Requirement | Status |
| --- | --- | --- |
| R194 | As much logic as possible should be compiled from Rust to WebAssembly, with JavaScript reserved for UI. | Partially implemented: the contract (provider order, RRF constant, evidence shape) is owned by the Rust solver and mirrored in the worker. The full Rust→WASM port of the search planner is tracked as a follow-up: the worker is 167 KB of JS today, and porting it inside this PR would inflate the diff and delay the DuckDuckGo default and combined ranking that the issue prioritizes. |

## Root Cause

There was no single bug to fix — the issue is a forward-looking feature
expansion. Before this PR:

- The browser worker and the Rust solver agreed on `web_search` only as a
  category; neither named DuckDuckGo as the default, neither combined results
  from multiple providers, and neither emitted structured per-provider
  evidence.
- The diagnostics page covered 11 services across two categories (search and
  knowledge); it lacked code-hosting and scientific-paper categories and was
  missing newer search engines (Yandex, Ecosia, Mojeek, Startpage).
- The page made one request per click and had no concurrency cap; a "Run all"
  click could exceed the per-origin socket budget if the list grew.
- A CORS failure was visible per row but did not propagate to future runs:
  reloading the page or clicking "Run all" again would re-issue the failing
  call.

The underlying browser constraints (CORS, per-origin connection limits, opaque
cross-origin responses) are documented in `raw-data/online-research/sources.md`
and shape the solution: every chosen provider exposes a CORS-readable JSON
endpoint, the planner caps active providers at five, and failures are recorded
and skipped for the rest of the session.

## Implemented Solution

- **Default search provider (Rust + JS).**
  `WEB_SEARCH_PROVIDERS = &["duckduckgo", "wikipedia", "wikidata"]` in
  `src/solver_handlers/web_requests.rs` is mirrored by the JS
  `WEB_SEARCH_PROVIDERS` array. DuckDuckGo is always first.
- **Combined ranking.** `reciprocalRankFusion` in the worker implements
  `score(d) = Σ 1 / (k + rank_i(d))` with `k = 60` (Cormack 2009). The fused
  list is sorted by combined score with a provider-count tiebreaker, so a URL
  returned by both DuckDuckGo and Wikipedia outranks one returned only by
  Wikidata.
- **Concurrency cap.** `runWithConcurrencyLimit(tasks, 5)` schedules at most
  five provider calls in parallel inside the worker; the dashboard runs
  category groups through a mirror runner with `CATEGORY_CONCURRENCY = 5`.
- **CORS auto-disable.** A session-scoped `WEB_SEARCH_DISABLED` map in the
  worker (and `state.disabled` in the dashboard) records `cors` or `network`
  failures the first time they happen and skips that provider until the page
  reloads. Both stacks emit `web_search:disabled:<provider>` /
  `disabled:<service-id>:<kind>` events so the decision is replayable.
- **Expanded diagnostics matrix.** `SERVICES` in `src/web/tests/connectivity.js`
  now covers 26 providers across four categories:
  - `search`: DuckDuckGo (default), Google, Bing, Brave, Yahoo, Yandex,
    Ecosia, Mojeek, Startpage.
  - `knowledge`: Wikipedia, Wikidata, Wiktionary, DBpedia, Open Library,
    OpenAlex, Crossref, Semantic Scholar.
  - `papers`: arXiv, Europe PMC, DOAJ.
  - `code`: GitHub, GitLab, Codeberg, Gitee (China), Bitbucket Cloud,
    GitFlic (Russia).
  - New badges (`.service-badge.code`, `.service-badge.papers`) in
    `src/web/tests/connectivity.css` keep categories visually distinct.
- **Exportable memory.** Both stacks emit structured `web_search:*` events.
  `src/event_log.rs::build_evidence_links` translates them into typed evidence
  links so the trace can be replayed offline. The connectivity dashboard's
  `exportLog` now includes `disabled` and `concurrency` metadata.
- **Tests stay deterministic.** `tests/e2e/tests/multilingual.spec.js` mocks
  `api.duckduckgo.com`, Wikipedia REST, and `wikidata.org/w/api.php` with
  `page.route` so CI runs without external network. The new assertions check
  for `web_search:provider:duckduckgo`, `web_search:provider:wikipedia`, and
  `web_search:combined:rrf:k=60`.

## Known Components

| Component | Use |
| --- | --- |
| DuckDuckGo Instant Answer API (`api.duckduckgo.com`) | Keyless, CORS-readable JSON endpoint used as the default search provider. |
| MediaWiki REST `search/page` and `wbsearchentities` | CORS-readable Wikipedia and Wikidata endpoints; `origin=*` enables cross-origin reads. |
| Reciprocal Rank Fusion (Cormack, Clarke, Buettcher 2009) | Parameter-free way to merge top-N ranked lists; we use `k = 60`. |
| Browser per-origin socket cap (~6) | Justifies the concurrency cap of 5 used by both the worker and the dashboard. |
| `Access-Control-Allow-Origin` / CORS | Defines which providers can be read directly from a browser; non-CORS providers stay page-only or proxy-only. |
| Append-only `EventLog` (`src/event_log.rs`) | Records every reasoning step (request, providers considered, per-rank URLs, fused order, disabled providers) for export. |
| Playwright `page.route` mocking | Keeps the explicit `web_search` test deterministic while still exercising the real dispatch path through `tryWebSearch`. |

## Tests

- `tests/e2e/tests/multilingual.spec.js`: the explicit web-search regression
  now mocks DuckDuckGo, Wikipedia REST, and Wikidata, and asserts that the
  evidence array contains `web_search:provider:duckduckgo`,
  `web_search:provider:wikipedia`, and `web_search:combined:rrf:k=60`.
- Existing Rust unit tests in `src/event_log.rs` continue to assert that
  evidence formatting is stable per event kind.
- `tests/e2e/tests/connectivity.spec.js` (from issue #129) keeps the
  diagnostics route covered locally and after Pages deploy. The expanded
  service rows reuse the same DOM contract so the existing selectors keep
  working.

## Follow-Up Plan

1. **Rust→WASM port of the search planner (R194).** Move
   `tryWebSearch`/`reciprocalRankFusion`/`runWithConcurrencyLimit` from
   `src/web/formal_ai_worker.js` into the Rust WASM worker so the planner
   shares a single implementation across CLI, server, and browser. The Rust
   side already owns the provider order and RRF constant; what remains is the
   browser-only `fetch` integration and the CORS-disable map.
2. **More providers behind feature flags.** The diagnostics matrix is now
   easy to extend: each provider is a `{ id, name, category, pageUrl, apiUrl,
   apiLabel, note }` record. Tracking issues should add Baidu, Naver, Qwant,
   Marginalia, and any other public engine that exposes a CORS-readable JSON
   surface.
3. **Persisted cache layer.** Today the disable map and the worker planner
   are session-scoped. A future PR can persist the chosen ranking and
   per-provider counts so the same prompt across sessions reuses the same
   reasoning trace, satisfying the issue's "preseed should be no different
   from real-time cached data" hint.
4. **Live external API mode in e2e.** Add a CI matrix axis that, on schedule,
   removes the `page.route` mocks and runs against the real providers so we
   notice when a provider quietly disables CORS.
