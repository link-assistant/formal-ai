# Issue 107 Case Study: Russian URL Request And Browser Search

## Summary

Issue #107 reported that the deployed demo returned the generic unknown-rule
fallback for the Russian prompt `Сделай запрос к google.com`. The root cause
was narrow intent routing: the Rust solver and browser worker only recognised
English `fetch <url>` prompts, even though the browser already had an HTTP
fetch/iframe fallback capability.

This PR adds multilingual URL-request routing, explicit web-search routing, and
browser tests that pin both behaviours.

## Evidence

Raw evidence is preserved in `raw-data/`:

- `issue-107.json` and `issue-107-comments.json`: the original report and the
  maintainer expansion posted on 2026-05-18.
- `pr-114.json`, `pr-114-conversation-comments.json`,
  `pr-114-review-comments.json`, and `pr-114-reviews.json`: PR state at the
  start of this investigation.
- `ci-runs-before-push.json`: CI run state before the implementation push.
- `browser-provider-probe.json`: browser probe from the deployed formal-ai
  origin against Google, Bing, DuckDuckGo, Brave, and Wikipedia endpoints.
- `formal-ai-tests-headers.txt` and `formal-ai-root-headers.txt`: headers that
  explain why `/formal-ai/tests` currently blocks probes with a GitHub Pages
  404 CSP while `/formal-ai/` is the usable deployed origin.
- `web-capture-*.json`, `web-capture-readme.md`, and
  `web-capture-upstream-issue-body.md`: upstream web-capture research.

## Timeline

| Time (UTC) | Event |
| --- | --- |
| 2026-05-17 23:34 | Issue #107 was opened from the mobile demo. The dialog shows `Сделай запрос к google.com` returning `intent: unknown`. |
| 2026-05-18 20:03 | Maintainer comment expanded the scope to browser provider checks, web-capture research, raw-data preservation, and future search/server planning. |
| 2026-05-18 | Investigation found the exact routing gap: `try_http_fetch` and the browser worker accepted `fetch <url>` only. |
| 2026-05-18 | Regression tests were added first and failed against the old behaviour. |
| 2026-05-18 | The fix added multilingual URL request parsing, explicit `web_search` routing, and browser-side Wikipedia search results. |

## Requirements And Status

| ID | Requirement | Status |
| --- | --- | --- |
| R1 | `Сделай запрос к google.com` must not return the unknown fallback. | Shipped. It now resolves to `http_fetch` and the browser tries `https://google.com`. |
| R2 | Direct page requests should use browser network capability where possible. | Shipped. The browser worker attempts `fetch(url, { mode: "cors" })` and falls back to an iframe when CORS or network policy blocks the direct read. |
| R3 | Add real internet search support. | Shipped for explicit search prompts through Wikipedia REST search, which is CORS-readable from the deployed app origin. |
| R4 | Check Google, Bing, other providers, and Wikipedia from a browser. | Done. `browser-provider-probe.json` shows Google, Bing, DuckDuckGo, and Brave block direct CORS fetches; Wikipedia REST search and summary return readable HTTP 200 JSON. |
| R5 | Use `web-capture` as much as possible and report missing search-specific features. | Done. Current web-capture supports URL capture, not normalized search-result capture. Upstream issue link: https://github.com/link-assistant/web-capture/issues/130 |
| R6 | Preserve issue/PR/log data under `docs/case-studies/issue-107`. | Done. Raw JSON, headers, provider probe output, and web-capture research are stored in this directory. |
| R7 | Add debug output if root cause cannot be found. | Not needed. The root cause was reproducible with deterministic unit and browser tests. |
| R8 | Local WebSocket/WebRTC agent server shared with CLI. | Planned, not implemented in this bug fix. The current repository has a minimal HTTP server and no WebSocket/WebRTC stack; adding that requires a separate architecture PR and dependency decision. |
| R9 | Sync the larger product vision and all historical issues. | Planned. This PR updates capability docs for the behaviour it ships; a full vision-wide reconciliation is beyond this targeted regression. |

## Root Cause

The original issue was not a network failure. It was an intent-routing failure:

- Rust: `try_http_fetch` only accepted prompts normalizing to `fetch <url>`.
- Browser worker: `isFetchPrompt` had the same English-only shape.
- Seeds listed `tool_web_search`, but no specialized solver handler claimed
  explicit web-search prompts.

The prompt `Сделай запрос к google.com` contained a valid hostname, but none of
the URL-request phrases matched, so the solver reached `unknown`.

## Provider Probe Findings

The maintainer suggested probing `https://link-assistant.github.io/formal-ai/tests`.
That path currently returns HTTP 404 with `content-security-policy:
default-src 'none'; ... connect-src 'self'`, so it blocks external fetch
probes. The script records that and falls back to the deployed app root
`https://link-assistant.github.io/formal-ai/`, which returns HTTP 200.

From the deployed app root:

| Provider | Browser CORS fetch result |
| --- | --- |
| Google home/search | `Failed to fetch` |
| Bing search | `Failed to fetch` |
| DuckDuckGo HTML search | `Failed to fetch` |
| Brave search | `Failed to fetch` |
| Wikipedia REST search | HTTP 200, readable JSON |
| Wikipedia summary | HTTP 200, readable JSON |

That makes Wikipedia the only provider in this probe that can be used directly
inside the static browser demo without a local/server capture helper.

## Implemented Solution

- Added `src/solver_handlers/web_requests.rs` with shared Rust handlers for
  multilingual URL requests and explicit web-search prompts.
- Registered `web_search` in the specialized handler chain immediately after
  `http_fetch`.
- Extended the browser worker to extract URLs from English and Russian request
  phrasing, not only `fetch <url>`.
- Added browser-side Wikipedia search result rendering for prompts like
  `Search the web for Nikola Tesla` and `Найди в интернете Никола Тесла`.
- Updated seed routing/tool metadata and demo examples so the capability is
  discoverable.

## Tests

- Unit regression: `fetch_prompt_returns_http_fetch_intent_not_unknown` now
  includes the reported Russian prompt and nearby variants.
- Unit regression: `web_search_prompt_returns_web_search_intent_not_unknown`
  covers English and Russian explicit search prompts.
- Playwright regression: Russian URL request renders `http_fetch` output and an
  iframe fallback instead of the unknown answer.
- Playwright regression: explicit web search renders a stubbed Wikipedia search
  result and exposes `web_search:provider:wikipedia` evidence.

## Follow-Up Plan

The larger "human-like programmatic search" vision needs a server-side capture
layer, not only a static browser worker. The next useful slice is:

1. Add a `web-capture search` API as proposed in upstream issue #130.
2. Teach formal-ai CLI/HTTP server to call that local search endpoint when it is
   configured, while keeping Wikipedia REST as the static-demo fallback.
3. Add diagnostics that distinguish CORS, captcha, blocked iframe, and provider
   extraction failures.
4. Design the WebSocket/WebRTC local agent server as a separate PR on top of the
   existing HTTP server boundary.
