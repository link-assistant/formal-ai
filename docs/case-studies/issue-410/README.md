# Case Study - Issue #410: web-search and web-capture as FormalAI components

> Source issue: <https://github.com/link-assistant/formal-ai/issues/410>
> Branch: `issue-410-e318f1ae5275` · PR: #414
> Raw data: [`raw-data/`](./raw-data)
> Last refreshed: 2026-06-15 (upstream readiness reached; dependencies updated to latest)

## 1. Summary

Issue #410 asks FormalAI to make sure
<https://github.com/link-assistant/web-search> has all required web-search
features and to use it as the FormalAI web-search component. It also asks
FormalAI to use <https://github.com/link-assistant/web-capture> for web fetch
and capture, and to report upstream gaps before continuing if required features
are missing.

The first pass of this case study (recorded in `raw-data/`) found three
concrete upstream blockers and filed them as issues. **All three have since
been resolved upstream**, so the readiness picture has changed materially:

- `web-capture` was already the mature component and remains so. It is
  published to npm (`1.10.9`) and crates.io (`0.3.31`), provides
  library/CLI/HTTP surfaces, and covers URL fetch, rendered HTML, Markdown,
  screenshots, archives, PDF, DOCX, streaming, and a structured search capture
  endpoint. Its crate now declares `rust-version = 1.96`.
- `web-search` is **now published** to npm (`@link-assistant/web-search@0.10.3`)
  and crates.io (`web-search@0.3.1`, `rust-version = 1.96`). Its provider
  registry is now a **superset** of FormalAI's registry (40 provider IDs vs
  FormalAI's 32) and its default plan now matches FormalAI's live default plan
  exactly. The two remaining differences from the first pass (no published
  package; non-matching providers/defaults) are gone.
- The MSRV mismatch that blocked a direct Rust dependency on `web-capture`
  (FormalAI `1.70` vs `web-capture` `1.88`) is resolved on both sides:
  FormalAI's MSRV is raised to `1.96` in this PR, and the upstream crates now
  target `1.96` as well. `1.96.0` is the current stable Rust toolchain.

What this PR delivers:

1. **Dependency refresh.** Every Rust workspace dependency, the Rust toolchain
   (`rust-version`/Dockerfile), and the web/desktop/VS Code npm dependencies are
   updated to their latest versions (see §9). This satisfies the "use the latest
   versions web-search, web-capture, only stable latest Rust, and all other
   dependencies should be fully updated to the latest ones" requirement.
2. **Refreshed case study** documenting that upstream readiness is reached and
   re-recording the package/provider evidence at the latest versions.

What remains as a deliberate, separate step (per the staged plan in §7): wiring
the published `web-search`/`web-capture` packages into FormalAI's production
paths behind a configuration-gated adapter with provider-parity tests. The case
study continues to recommend an adapter rather than an in-place swap so the
existing `web_search:*` evidence contract and RRF semantics are preserved and
tested before any default changes.

Upstream issues filed from this case study (all now **closed/resolved**):

- `web-search` package publication: <https://github.com/link-assistant/web-search/issues/6> — closed.
- `web-search` provider parity/defaults: <https://github.com/link-assistant/web-search/issues/5> — closed.
- `web-capture` FormalAI integration contract: <https://github.com/link-assistant/web-capture/issues/135> — closed.

## 2. Requirements And Status

| # | Requirement from issue #410 | Status |
|---|---|---|
| R1 | Read issue details/comments and collect a deep case study under `docs/case-studies/issue-410`. | Done. Issue, PR, comments, upstream repo data, package probes, source snapshots, and online research are preserved in `raw-data/`. |
| R2 | Make sure `web-search` has all features required by FormalAI. | **Met upstream.** `web-search@0.10.3` is published with a 40-provider registry that is a superset of FormalAI's 32 providers, and its default plan now matches FormalAI's live default plan. Tracked-and-closed in web-search #5 and #6. |
| R3 | Use `web-search` as the FormalAI web-search component. | **Unblocked; adapter pending.** With publication and parity in place, the remaining work is the configuration-gated FormalAI adapter (§7 Phase 2) plus parity tests, kept separate so current defaults/evidence are preserved until proven. |
| R4 | Use `web-capture` for FormalAI web fetch/capture. | **Unblocked.** HTTP/CLI integration is viable today and the MSRV blocker is gone: FormalAI MSRV is raised to `1.96` in this PR and `web-capture` now targets `1.96`. Contract documented in web-capture #135 (closed). |
| R5 | Both components should support the required library, CLI, and microservice surfaces. | Done. `web-capture` and `web-search` both ship published library, CLI, and HTTP/microservice surfaces. |
| R6 | If a required feature is missing upstream, report an issue and continue after it is resolved. | Done. Three upstream issues were filed with evidence and acceptance criteria; all three are now closed/resolved. |
| R7 | Search online and compare known existing components/libraries. | Done. See [`raw-data/online-research.md`](./raw-data/online-research.md). |
| R8 | Keep the work in PR #414. | Done. This case study plus the dependency refresh are the PR artifacts. |

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

FormalAI's registry has 32 provider IDs:

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

First-pass HEAD inspected: `5ac8abf4fe173e4b01925468b639e8fd7c207530`.
Latest HEAD at refresh: `79752212ad1558d65a9eb1d130beb6ea4131803c`.

Positive findings (now confirmed at published versions):

- JavaScript package `@link-assistant/web-search` is published on npm at
  version `0.10.3`, exposing a `web-search` CLI and an Express HTTP server.
- The Rust crate `web-search` is published on crates.io at version `0.3.1`
  with `rust-version = 1.96`.
- README describes library, CLI, and REST API usage.
- Registry exports categories, provider IDs, defaults, and provider factories.
- Merge strategies include Reciprocal Rank Fusion, weighted merge, and
  interleave.
- `WebCaptureProvider` delegates to `@link-assistant/web-capture` and exposes
  `wc:wikipedia`, `wc:duckduckgo`, `wc:google`, `wc:bing`, and `wc:brave`
  providers.

Resolved findings (previously blocking, now fixed upstream):

- `npm view @link-assistant/web-search` now returns `0.10.3` (was `E404`).
- `cargo info web-search` now returns `0.3.1` (was "not found").
- `getDefaultProviderIds()` now returns `duckduckgo`, `internet-archive`,
  `wikipedia`, `wikidata`, `wiktionary`, `wikinews` — an **exact match** for
  FormalAI's live default plan (was `duckduckgo`, `google`, `bing`,
  `wikipedia`).
- The registry now exposes 40 provider IDs covering every FormalAI provider ID
  plus extras (`searx`, `hackernews`, `lite`, and the `wc:*` capture-backed
  providers). The previously-missing IDs (`internet-archive`, `wiktionary`,
  `wikinews`, `openlibrary`, `semantic-scholar`, `europepmc`, `doaj`,
  `dbpedia`, dictionary providers, `yandex`, and the non-GitHub code hosts) are
  all present.

This means `web-search` is now a viable production target. The remaining work
is the FormalAI-side adapter and parity tests, not an upstream gap.

### 4.2 web-capture

First-pass HEAD inspected: `25e7602093a09f2bf98b537653e348b30162f6c0`.
Latest HEAD at refresh: `bc3678c14be178798cc22aa2b573d541f430cc87`.

Positive findings:

- `@link-assistant/web-capture` is published on npm at version `1.10.9`.
- `web-capture` is published on crates.io at version `0.3.31`.
- CLI, HTTP, and library surfaces exist.
- HTTP endpoints include `/html`, `/txt`, `/markdown`, `/image`, `/archive`,
  `/fetch`, `/stream`, and `/search`.
- Formats include Markdown, rendered HTML, plain text, PNG/JPEG screenshot,
  ZIP archive, PDF, and DOCX.
- Structured search capture returns normalized results with diagnostics.

Resolved findings:

- The Rust crate `web-capture` now declares `rust-version = 1.96` (was `1.88`).
  FormalAI's MSRV is raised to `1.96` in this PR, so a direct Rust dependency no
  longer forces a divergent toolchain.
- The capture/search contract is documented per web-capture #135 (closed).

Remaining integration nuance (not a blocker):

- Structured search capture (`/search`) is still focused on the `wc:*` set
  (`wikipedia`, `duckduckgo`, `google`, `bing`, `brave`); broader provider
  aggregation is `web-search`'s role, with `web-capture` as the fetch/render
  layer beneath it.

## 5. Provider Compatibility Matrix

| Provider area | FormalAI today | web-search `0.10.3` | web-capture `1.10.9` |
|---|---|---|---|
| Live defaults | `duckduckgo`, `internet-archive`, `wikipedia`, `wikidata`, `wiktionary`, `wikinews` | `duckduckgo`, `internet-archive`, `wikipedia`, `wikidata`, `wiktionary`, `wikinews` (exact match) | `wikipedia` for `/search` |
| General search | DuckDuckGo, Google, Bing, Brave, Yahoo, Yandex, Ecosia, Mojeek, Startpage | DuckDuckGo, Google, Bing, Brave, Yahoo, Yandex, Ecosia, Mojeek, Startpage, SearX, DuckDuckGo Lite | Wikipedia, DuckDuckGo, Google, Bing, Brave search capture |
| Knowledge | Wikipedia, Wikidata, Wiktionary, Wikinews, Internet Archive, Open Library, DBpedia, dictionaries, OpenAlex, Crossref, Semantic Scholar | Same set, plus capture-backed `wc:*` knowledge providers | Captures arbitrary URLs; `/search` only has Wikipedia from this group |
| Papers | arXiv, Europe PMC, DOAJ | arXiv, Europe PMC, DOAJ, Crossref, OpenAlex | No paper-search catalog |
| Code | GitHub, GitLab, Codeberg, Gitee, Bitbucket, GitFlic | GitHub, GitLab, Codeberg, Gitee, Bitbucket, GitFlic, Hacker News | No code-search catalog |
| Fetch/capture | Browser/static fetch plus optional local proxy references | Search aggregation; optional `web-capture` delegation | Primary role: fetch/render/capture service |

`web-search@0.10.3`'s 40 provider IDs are a superset of FormalAI's 32, so a
configuration-gated adapter can preserve every current provider.

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

### Phase 0: first pass (recorded in `raw-data/`)

- Preserved issue, PR, package, upstream, and online evidence.
- Filed upstream issues for missing readiness features.
- Did not land a production adapter before parity existed.

### Phase 1: upstream readiness — **complete**

- `web-search` package publication resolved (web-search #6, closed): npm
  `0.10.3`, crates.io `0.3.1`.
- Provider parity/default alignment resolved (web-search #5, closed): 40-provider
  superset, defaults match FormalAI.
- `web-capture` contract documented and MSRV aligned to `1.96`
  (web-capture #135, closed).

### Phase 2: FormalAI adapters — next

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

- Make external `web-search` the default only after provider parity,
  diagnostics parity, and fallback behavior are proven in tests.
- Keep the internal backend available for static/offline builds and regression
  comparison until it is no longer needed.

## 8. Verification Performed

Commands and probes saved under `raw-data/`:

- `gh issue view 410` and issue/PR comment API snapshots.
- `gh repo view`, `gh issue list`, and `gh release list` for both upstream
  repositories.
- `npm view @link-assistant/web-search` → `0.10.3` and
  `cargo info web-search` → `0.3.1` (`rust-version = 1.96`), both now present.
- `npm view @link-assistant/web-capture` → `1.10.9` and
  `cargo info web-capture` → `0.3.31` (`rust-version = 1.96`).
- Provider-registry extraction from `@link-assistant/web-search@0.10.3`:
  40 provider IDs, defaults matching FormalAI's live plan.
- Upstream issue status checks: web-search #5, web-search #6, and
  web-capture #135 are all closed.
- Direct source snapshots for FormalAI web search, `web-search`, and
  `web-capture`.

The first-pass blocker — that the external component did not preserve FormalAI's
provider contract — no longer holds. The remaining gate before a production swap
is FormalAI-side adapter code with parity tests (Phase 2/3), which is kept as a
separate, test-backed change.

## 9. Dependency Refresh (this PR)

Updated to the latest versions to satisfy the "use the latest versions" and
"only stable latest Rust" requirement:

- **Rust toolchain / MSRV:** `rust-version` raised `1.70` → `1.96`; Docker
  builder image `rust:1.82-slim` → `rust:1.96-slim`. `1.96.0` is the current
  stable release and matches the `web-search`/`web-capture` crate MSRVs.
- **Rust dependencies (`Cargo.toml`/`Cargo.lock`):** `clap` `4.4` → `4.6`,
  `doublets` `0.3.0` → `0.4.0`, `link-calculator` `0.17.2` → `0.19.0`,
  `meta-language` `0.39` → `0.45`, plus transitive updates via `cargo update`.
- **Web bundle (`package.json`):** `dompurify` → `3.4.10`, `marked` → `18.0.5`,
  `react` / `react-dom` → `19.2.7`; `lino-i18n` and `tesseract.js` already at
  latest. `src/web/vendor.bundle.js` rebuilt; `bun` pinned to `1.3.14`.
- **Desktop (`desktop/package.json`):** `electron` → `^42.4.0`,
  `electron-builder` → `^26.15.3`.
- **VS Code (`vscode/package.json`):** `@vscode/test-web` → `^0.0.80`,
  `@vscode/vsce` → `^3.9.2`.

The React 19 upgrade surfaced one regression: React 19 compares
`dangerouslySetInnerHTML` by object identity rather than by the inner `__html`
string (React 18's behavior), so a fresh markdown-HTML object on every render
re-assigned `innerHTML` and wiped the out-of-band code-block enhancements added
for issue #330. Fixed by memoizing the rendered markdown object by message
content in `src/web/app.js`, so the object identity is stable while the text is
unchanged. The full Playwright suite passes (261 passed; the 5 issue-221
failures are a pre-existing local WASM-worker translation-data gap that fails
identically on the pre-change baseline).

## 10. Raw Data Index

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
