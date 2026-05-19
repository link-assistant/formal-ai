# Issue 129 Case Study: GitHub Pages Connectivity Tests

## Summary

Issue #129 reported that
`https://link-assistant.github.io/formal-ai/tests` was broken and asked for a
dedicated browser test page for search engines, public knowledge databases,
iframe embedding, and an optional `web-capture` proxy mode.

The root cause was simple but high-impact: the GitHub Pages artifact did not
contain any `tests/index.html`, so GitHub Pages served its generic 404 page.
That 404 response also carries a restrictive CSP (`connect-src 'self'`), which
means the missing diagnostics route could not be used to probe external
providers.

This PR adds a static diagnostics dashboard at `src/web/tests/`, includes it in
the local and live Playwright matrices, and updates the Pages stamping script so
nested HTML files receive the same version and asset cache-busting placeholders
as the root demo.

## Evidence

Raw evidence is preserved in `raw-data/`:

- `formal-ai/issue-129.json` and `formal-ai/issue-129-comments.json`: original
  issue metadata and comments.
- `formal-ai/pr-130*.json`, `formal-ai/pr-130-before.diff`: PR state before
  this implementation.
- `formal-ai/branch-runs-before.json`: recent workflow runs for
  `issue-129-9c664282dfea` before the fix.
- `formal-ai/live-tests-no-slash.response` and
  `formal-ai/live-tests-slash.response`: live 404 responses for `/tests` and
  `/tests/` captured on 2026-05-19.
- `formal-ai/live-root.response` and `formal-ai/live-deployment.json`: live root
  response and deployed SHA/version evidence.
- `web-capture/repo.json`, `web-capture/README.md`, and
  `web-capture/upstream-issue-130.json`: current upstream proxy capability and
  the already-open search-provider issue.
- `online-research/sources.md`: external documentation used to design the
  browser checks.

## Timeline

| Time (UTC) | Event |
| --- | --- |
| 2026-05-18 | Issue #107 investigation showed `/formal-ai/tests` returned GitHub Pages 404 and that browser-readable search is provider-dependent. |
| 2026-05-18 | `link-assistant/web-capture#130` was opened to track a future structured search-provider API. |
| 2026-05-19 12:00 | Issue #129 was opened with explicit requirements for `/tests`, CORS-free provider checks, iframe expansion, proxy switching, and preserved data. |
| 2026-05-19 12:05 | Live `/formal-ai/tests` and `/formal-ai/tests/` were captured returning 404; the root page returned 200 with deployed SHA `5529e6eb9d6f39001ee97a780ec0c52242cd0295`. |
| 2026-05-19 | Regression coverage was added for `/tests/`, direct fetch, proxy fetch, and iframe expansion. |
| 2026-05-19 | The static dashboard, nested HTML stamping, docs, and changelog were added. |

## Requirements And Status

| ID | Requirement | Status |
| --- | --- | --- |
| R1 | `/formal-ai/tests` and `/formal-ai/tests/` must serve a real page. | Implemented by `src/web/tests/index.html`; the deploy artifact already uploads all of `src/web`. |
| R2 | The page must run interactive browser checks for search engines. | Implemented with Google, Bing, DuckDuckGo, Brave, and Yahoo page rows plus available suggestion/instant-answer endpoints. |
| R3 | The page must run interactive browser checks for public knowledge databases. | Implemented with Wikipedia, Wikidata, Open Library, OpenAlex, Crossref, and Semantic Scholar API rows. |
| R4 | It must test page access through `fetch()`. | Each row has `Fetch page`, using direct CORS `fetch()` or the configured proxy target. |
| R5 | It must test API access through `fetch()`. | Rows with public API endpoints have `Fetch API`; rows without an unauthenticated API show `No API`. |
| R6 | It must expose iframe diagnostics and an expanded iframe view. | Each row has `Frame` and `Expand`; Playwright verifies inline and full-window iframe states. |
| R7 | It must switch to `web-capture` as a proxy. | Proxy mode rewrites fetches through a configurable base URL and `/fetch`, `/html`, or `/markdown` endpoint. |
| R8 | Logs and data must be saved under `docs/case-studies/issue-129`. | Implemented with raw GitHub, live HTTP, upstream, and online-research data in `raw-data/`. |
| R9 | Related upstream issues must be reported when needed. | No new upstream issue was needed: `link-assistant/web-capture#130` already tracks the missing structured search API; this page uses the current URL capture endpoints. |
| R10 | The route must stay covered in CI and Pages e2e. | Implemented by `tests/e2e/tests/connectivity.spec.js` and both Playwright configs. |

## Root Cause

The live failure was not caused by provider CORS behavior. The route itself was
absent from the static artifact:

- `src/web/` contained the root demo but no `src/web/tests/index.html`.
- GitHub Pages therefore returned its generic 404 page for both `/tests` and
  `/tests/`.
- That 404 page includes `content-security-policy: default-src 'none'; ...;
  connect-src 'self'`, so any diagnostic JavaScript placed on the missing route
  could not have made external provider requests.
- The Pages stamping script only processed root `index.html`; adding a nested
  HTML page required stamping all HTML files in the artifact so version markers
  and `?v=__FORMAL_AI_ASSET_VERSION__` cache-busters do not leak to production.

The provider side remains a separate browser/platform constraint. Direct
browser `fetch()` can read a cross-origin response only when the provider opts
in with CORS headers. Iframes can show whether a page can be embedded, but
JavaScript cannot inspect most cross-origin iframe content, and providers can
block embedding with `X-Frame-Options` or CSP `frame-ancestors`.

## Implemented Solution

- Added `src/web/tests/index.html`, `connectivity.css`, and `connectivity.js`.
- Added a provider matrix for search engines and public knowledge databases.
- Added direct browser fetch mode with explicit status, final URL, content type,
  elapsed time, preview, and JSON export.
- Added `web-capture proxy` mode with configurable base URL and endpoint:
  `/fetch`, `/html`, or `/markdown`.
- Added inline iframe diagnostics plus a full-window expanded iframe view.
- Updated `scripts/stamp-pages-artifact.sh` to replace placeholders in every
  HTML file under `src/web`, not only the root `index.html`.
- Added Playwright coverage for the new route to both local and Pages suites.

## Known Components

| Component | Use |
| --- | --- |
| Browser Fetch API | Direct CORS-readable page/API checks. |
| Browser same-origin policy and CORS | Explains why many search pages fail direct readable fetches. |
| HTML iframe | Visual/manual embed diagnostics for page endpoints. |
| `X-Frame-Options` and CSP `frame-ancestors` | Explain why some providers can refuse iframe embedding. |
| GitHub Pages static artifact | Serves `src/web/tests/index.html` at the repository subpath. |
| `web-capture --serve` | Local proxy target using existing `/fetch`, `/html`, and `/markdown` endpoints. |
| Wikimedia/Open Library/OpenAlex/Crossref/Semantic Scholar APIs | Public knowledge endpoints included in the diagnostics matrix. |

## Tests

- `tests/e2e/tests/connectivity.spec.js`: route renders, provider matrix exists,
  direct Wikipedia API fetch works, blocked direct page fetches are recorded,
  proxy mode rewrites through `web-capture`, and iframe expansion opens.
- `tests/unit/ci-cd/workflow_release.rs`: nested HTML stamping smoke coverage,
  cache-busted diagnostics assets, and Pages-safe navigation assertions.

## Follow-Up Plan

1. When `web-capture#130` ships a structured search endpoint, add a fourth proxy
   endpoint mode for normalized search results.
2. Add optional user-entered custom provider rows so maintainers can test a
   newly reported URL without editing the page.
3. Persist exported JSON from failed provider runs as issue attachments when the
   browser demo generates a prefilled report.
