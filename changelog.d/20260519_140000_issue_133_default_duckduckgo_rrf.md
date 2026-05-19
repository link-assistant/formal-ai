---
bump: minor
---

### Added

- Issue #133: DuckDuckGo Instant Answer is now the default web search engine
  across the CLI, server, and the browser-only GitHub Pages app. The shared
  provider list `WEB_SEARCH_PROVIDERS = ["duckduckgo", "wikipedia", "wikidata"]`
  is owned by `src/solver_handlers/web_requests.rs` and mirrored by the JS
  worker so every surface dispatches the same plan.
- Combined ranking via Reciprocal Rank Fusion (Cormack, Clarke, Buettcher
  2009) with `k = 60`. Each provider returns up to its top-10 results and the
  worker merges them with `score(d) = Σ 1 / (k + rank_i(d))`, so URLs returned
  by more than one engine bubble up. The fused order is appended to memory as
  `web_search:fused:<rank>:<providers>:<url>` events.
- Per-category concurrency cap of 5 with `runWithConcurrencyLimit` in the
  worker and `CATEGORY_CONCURRENCY = 5` in the browser diagnostics page so the
  per-origin socket budget is never starved.
- Session-scoped CORS auto-disable: when a provider fetch throws a CORS or
  network error, the worker's `WEB_SEARCH_DISABLED` map (and the dashboard's
  `state.disabled` map) record the failure and skip the provider for the rest
  of the session. The decision is appended to memory as
  `web_search:disabled:<provider>`.
- Expanded browser-only diagnostics matrix at `/formal-ai/tests`: now 26
  providers across four categories.
  - `search`: DuckDuckGo (default), Google, Bing, Brave, Yahoo, Yandex,
    Ecosia, Mojeek, Startpage.
  - `knowledge`: Wikipedia, Wikidata, Wiktionary, DBpedia, Open Library,
    OpenAlex, Crossref, Semantic Scholar.
  - `papers`: arXiv, Europe PMC, DOAJ.
  - `code`: GitHub, GitLab, Codeberg, Gitee (China), Bitbucket Cloud,
    GitFlic (Russia).
- Structured `web_search:*` evidence kinds in both the Rust solver and the JS
  worker, with matching formatter branches in
  `src/event_log.rs::build_evidence_links` so the reasoning trace can be
  replayed offline.
- Issue #133 case study under `docs/case-studies/issue-133/`, including raw
  issue/PR JSON, branch log, online research notes, and a deep analysis of
  requirements R181–R194.

### Changed

- The Playwright explicit web-search regression now mocks all three default
  providers (`api.duckduckgo.com`, Wikipedia REST, `wikidata.org/w/api.php`)
  and asserts the new `web_search:provider:duckduckgo`,
  `web_search:provider:wikipedia`, and `web_search:combined:rrf:k=60` evidence
  shape.
