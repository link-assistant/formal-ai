# Case Study - Issue #410: web-search and web-capture as FormalAI components

> Source issue: <https://github.com/link-assistant/formal-ai/issues/410>
> Branch: `issue-410-e318f1ae5275` · PR: #414
> Raw data: [`raw-data/`](./raw-data)

## 1. Summary

Issue #410 asks FormalAI to make sure
<https://github.com/link-assistant/web-search> has all required web-search
features and to use it as the FormalAI web-search component. It also asks
FormalAI to use <https://github.com/link-assistant/web-capture> for web fetch
and capture, and to report upstream gaps before continuing if required features
are missing.

The result of the investigation is:

- `web-capture` is the more mature component today. It is published to npm and
  crates.io, provides library/CLI/HTTP surfaces, and already covers URL fetch,
  rendered HTML, Markdown, screenshots, archives, PDF, DOCX, streaming, and a
  five-provider structured search capture endpoint.
- `web-search` has the right source-level shape: JavaScript and Rust
  implementations, provider registry, CLI/server/library entry points, RRF and
  other merge strategies, and optional `web-capture` delegation. It is not yet
  safe to make it FormalAI's production component because it is not published to
  npm or crates.io, its provider catalog is not a superset of FormalAI's current
  registry, and its defaults differ from FormalAI's DuckDuckGo-first behavior.
- A direct replacement in this PR would regress existing FormalAI behavior.
  The correct implementation path is staged: fix upstream package/provider
  readiness first, then add a FormalAI adapter behind configuration and parity
  tests, then retire the in-repo implementation after provider parity is proven.

Upstream issues filed from this case study:

- `web-search` package publication: <https://github.com/link-assistant/web-search/issues/6>
- `web-search` provider parity/defaults: <https://github.com/link-assistant/web-search/issues/5>
- `web-capture` FormalAI integration contract: <https://github.com/link-assistant/web-capture/issues/135>

## 2. Requirements And Status

| # | Requirement from issue #410 | Status |
|---|---|---|
| R1 | Read issue details/comments and collect a deep case study under `docs/case-studies/issue-410`. | Done. Issue, PR, comments, upstream repo data, package probes, source snapshots, and online research are preserved in `raw-data/`. |
| R2 | Make sure `web-search` has all features required by FormalAI. | Blocked upstream. It has the desired architecture, but lacks package publication and FormalAI provider parity. Tracked in web-search #5 and #6. |
| R3 | Use `web-search` as the FormalAI web-search component. | Planned after R2. Immediate replacement would drop providers and change defaults. |
| R4 | Use `web-capture` for FormalAI web fetch/capture. | Partially ready. HTTP/CLI integration is viable; direct Rust library integration would raise FormalAI's MSRV from 1.70 to `web-capture`'s 1.88. Contract hardening is tracked in web-capture #135. |
| R5 | Both components should support the required library, CLI, and microservice surfaces. | `web-capture` does today. `web-search` has source-level surfaces but no published install target. |
| R6 | If a required feature is missing upstream, report an issue and continue after it is resolved. | Done. Three upstream issues were filed with evidence and acceptance criteria. |
| R7 | Search online and compare known existing components/libraries. | Done. See [`raw-data/online-research.md`](./raw-data/online-research.md). |
| R8 | Keep the work in PR #414. | Done. This case study is the PR artifact for the current blocker state. |

## 3. FormalAI Current Contract

FormalAI currently owns its web-search contract in `src/web_search_core.rs` and
uses it across native, server, browser worker, and WASM paths. The current core
contract includes:

- RRF constant `WEB_SEARCH_RRF_K = 60`.
- Per-category provider concurrency cap `WEB_SEARCH_CONCURRENCY_PER_CATEGORY = 5`.
- Provider result limit `WEB_SEARCH_PROVIDER_LIMIT = 10`.
- A broad provider registry split into search, knowledge, papers, and code.
- A browser-live default plan of DuckDuckGo, Internet Archive, Wikipedia,
  Wikidata, Wiktionary, and Wikinews.
- Evidence lines such as `web_search:provider:<id>` and
  `web_search:combined:rrf:k=60`.

FormalAI's registry has 31 provider IDs:

| Category | FormalAI provider IDs |
|---|---|
| Search | `duckduckgo`, `google`, `bing`, `brave`, `yahoo`, `yandex`, `ecosia`, `mojeek`, `startpage` |
| Knowledge | `wikipedia`, `wikidata`, `wiktionary`, `wikinews`, `cambridge-dictionary`, `merriam-webster`, `dictionary-com`, `collins-dictionary`, `internet-archive`, `dbpedia`, `openlibrary`, `openalex`, `crossref`, `semantic-scholar` |
| Papers | `arxiv`, `europepmc`, `doaj` |
| Code | `github`, `gitlab`, `codeberg`, `gitee`, `bitbucket`, `gitflic` |

Any component replacement must preserve that contract or ship a deliberate,
tested migration.

## 4. Upstream Component Findings

### 4.1 web-search

Repository HEAD inspected:
`5ac8abf4fe173e4b01925468b639e8fd7c207530`.

Positive findings:

- JavaScript package manifest exposes `@link-assistant/web-search` with a
  `web-search` CLI.
- Rust crate manifest exists under `rust/`.
- README describes library, CLI, and REST API usage.
- Registry exports categories, provider IDs, defaults, and provider factories.
- Merge strategies include Reciprocal Rank Fusion, weighted merge, and
  interleave.
- `WebCaptureProvider` dynamically imports `@link-assistant/web-capture` and
  exposes `wc:wikipedia`, `wc:duckduckgo`, `wc:google`, `wc:bing`, and
  `wc:brave` providers.

Blocking findings:

- `npm view @link-assistant/web-search` returns npm `E404`.
- `cargo info web-search` cannot find a crates.io crate.
- `getDefaultProviderIds()` returns `duckduckgo`, `google`, `bing`,
  `wikipedia`, while FormalAI's live default plan is DuckDuckGo, Internet
  Archive, Wikipedia, Wikidata, Wiktionary, and Wikinews.
- Registry metadata marks Google as `defaultForCategory: true`; FormalAI marks
  DuckDuckGo as the default search provider.
- Missing FormalAI provider IDs include `internet-archive`, `wiktionary`,
  `wikinews`, `openlibrary`, `semantic-scholar`, `europepmc`, `doaj`,
  `dbpedia`, dictionary providers, `yandex`, and the non-GitHub code hosts.

This means `web-search` is a strong future target, but not ready for a direct
FormalAI swap.

### 4.2 web-capture

Repository HEAD inspected:
`25e7602093a09f2bf98b537653e348b30162f6c0`.

Positive findings:

- `@link-assistant/web-capture` is published on npm at version `1.10.6`.
- `web-capture` is published on crates.io at version `0.3.29`.
- CLI, HTTP, and library surfaces exist.
- HTTP endpoints include `/html`, `/txt`, `/markdown`, `/image`, `/archive`,
  `/fetch`, `/stream`, and `/search`.
- Formats include Markdown, rendered HTML, plain text, PNG/JPEG screenshot,
  ZIP archive, PDF, and DOCX.
- Structured search capture returns normalized results with diagnostics.

Integration gaps:

- Structured search capture currently supports only `wikipedia`, `duckduckgo`,
  `google`, `bing`, and `brave`.
- Rust crate `web-capture` declares `rust-version = 1.88`, while FormalAI
  declares `rust-version = 1.70`.
- FormalAI needs a stable documented contract before depending on the HTTP/CLI
  response shapes across releases.

The practical near-term plan is to use `web-capture` as an optional external
HTTP/CLI service for fetch/capture, not as a direct Rust dependency.

## 5. Provider Compatibility Matrix

| Provider area | FormalAI today | web-search HEAD | web-capture HEAD |
|---|---|---|---|
| Live defaults | `duckduckgo`, `internet-archive`, `wikipedia`, `wikidata`, `wiktionary`, `wikinews` | `duckduckgo`, `google`, `bing`, `wikipedia` | `wikipedia` for `/search` |
| General search | DuckDuckGo, Google, Bing, Brave, Yahoo, Yandex, Ecosia, Mojeek, Startpage | DuckDuckGo, Google, Bing, Brave, Yahoo, Ecosia, Mojeek, Startpage, SearXNG, DuckDuckGo Lite | Wikipedia, DuckDuckGo, Google, Bing, Brave search capture |
| Knowledge | Wikipedia, Wikidata, Wiktionary, Wikinews, Internet Archive, Open Library, DBpedia, dictionaries, OpenAlex, Crossref, Semantic Scholar | Wikipedia, Wikidata; Crossref and OpenAlex under papers | Captures arbitrary URLs; `/search` only has Wikipedia from this group |
| Papers | arXiv, Europe PMC, DOAJ | Crossref, OpenAlex, arXiv | No paper-search catalog |
| Code | GitHub, GitLab, Codeberg, Gitee, Bitbucket, GitFlic | GitHub, Hacker News | No code-search catalog |
| Fetch/capture | Browser/static fetch plus optional local proxy references | Search aggregation; optional `web-capture` delegation | Primary role: fetch/render/capture service |

## 6. Existing Component Survey

The online survey shows useful external providers, but none should replace the
preferred internal component boundary:

- Search APIs: Brave Search API, Tavily Search, SerpApi, SearXNG, and Google
  Custom Search can be provider backends, but they need hosted services,
  instance configuration, keys, or service-specific policies.
- Capture/render tools: Playwright, Puppeteer, Browserbase/Stagehand,
  Firecrawl, and Jina Reader cover browser automation or web content capture.
  These are useful references for `web-capture`, not substitutes for a
  FormalAI-compatible provider registry.
- Browser CORS remains a hard constraint for static/browser-only FormalAI
  paths. MediaWiki APIs are friendly to cross-site calls; most search-engine
  HTML pages are not.

The chosen architecture should keep `web-search` as the provider aggregator and
`web-capture` as the fetch/render/capture service, with third-party APIs only as
optional provider implementations.

## 7. Implementation Plan

### Phase 0: current PR

- Preserve issue, PR, package, upstream, and online evidence in
  `docs/case-studies/issue-410/raw-data`.
- File upstream issues for missing readiness features.
- Do not land a production adapter that changes FormalAI's current provider
  behavior before parity exists.

### Phase 1: upstream readiness

- Resolve `web-search` package publication in web-search #6.
- Resolve provider parity/default alignment in web-search #5.
- Resolve or document the `web-capture` FormalAI contract and MSRV strategy in
  web-capture #135.
- Add upstream tests for registry/default parity and stable response shapes.

### Phase 2: FormalAI adapters

- Add a `WebSearchBackend` abstraction with an internal backend and an external
  HTTP/CLI backend.
- Gate external web-search through configuration, for example
  `FORMAL_AI_WEB_SEARCH_BASE_URL` and a CLI flag.
- Add a `WebCaptureBackend` abstraction for URL fetch/capture.
- Gate external web-capture through configuration, for example
  `FORMAL_AI_WEB_CAPTURE_BASE_URL`.
- Preserve existing `web_search:*` evidence lines and RRF semantics.
- Keep the static browser worker on the current CORS-readable path unless an
  explicit web service URL is configured.

### Phase 3: verification

- Unit-test adapters with mock HTTP servers and recorded fixtures.
- Add parity tests asserting that default provider IDs, categories, and fused
  results match current FormalAI behavior before changing defaults.
- Add failure-mode tests for service offline, invalid JSON, timeout, CAPTCHA
  diagnostics, provider missing, and partial provider failure.
- Add live-network smoke tests behind the existing explicit live-test gate.

### Phase 4: migration

- Make external `web-search` the default only after package publication,
  provider parity, diagnostics parity, and fallback behavior are proven.
- Keep the internal backend available for static/offline builds and regression
  comparison until it is no longer needed.

## 8. Verification Performed

Commands and probes saved under `raw-data/`:

- `gh issue view 410` and issue/PR comment API snapshots.
- `gh repo view`, `gh issue list`, and `gh release list` for both upstream
  repositories.
- `npm view @link-assistant/web-search` and
  `cargo info web-search`, both failing as missing packages.
- `npm view @link-assistant/web-capture` and `cargo info web-capture`, both
  succeeding.
- Direct source snapshots for FormalAI web search, `web-search`, and
  `web-capture`.
- A generated `web-search` provider registry snapshot from the package export.

No production code was changed in this PR because the reproducing evidence is
the compatibility matrix: the external component does not yet preserve
FormalAI's existing provider contract. The appropriate test-before-fix artifact
for this issue is the saved package/provider probe data plus the upstream
acceptance criteria.

## 9. Raw Data Index

- [`raw-data/formal-ai/`](./raw-data/formal-ai) - issue, PR, comments, and
  FormalAI source snapshots.
- [`raw-data/web-search/`](./raw-data/web-search) - upstream repo metadata,
  README/manifests, provider registry, and source indexes.
- [`raw-data/web-capture/`](./raw-data/web-capture) - upstream repo metadata,
  README/manifests, package releases, and search contract snapshots.
- [`raw-data/package-probes/`](./raw-data/package-probes) - npm and crates.io
  package availability checks.
- [`raw-data/upstream-issues/`](./raw-data/upstream-issues) - issue bodies and
  created upstream issue URLs.
- [`raw-data/online-research.md`](./raw-data/online-research.md) - primary
  external source notes.
