## Context

formal-ai issue link: https://github.com/link-assistant/formal-ai/issues/107

While fixing formal-ai issue #107, we needed a programmatic search path that works from both browser and CLI/server contexts. `web-capture` already exposes URL-oriented capture endpoints such as `/fetch`, `/html`, `/markdown`, `/image`, `/archive`, `/pdf`, and `/docx`, but the current README does not describe a search-specific endpoint or library API that normalizes search provider results.

## Reproducer

From the formal-ai deployed app origin (`https://link-assistant.github.io/formal-ai/`), a browser `fetch(..., { mode: "cors" })` probe produced:

- Google home/search: `Failed to fetch`
- Bing search: `Failed to fetch`
- DuckDuckGo HTML search: `Failed to fetch`
- Brave search: `Failed to fetch`
- Wikipedia REST search: HTTP 200, readable CORS JSON
- Wikipedia summary: HTTP 200, readable CORS JSON

The raw probe script and output are captured in the formal-ai PR case study:

- `experiments/issue-107-browser-provider-probe.mjs`
- `docs/case-studies/issue-107/raw-data/browser-provider-probe.json`

## Current Workaround

formal-ai now uses Wikipedia REST search directly in the browser for explicit web-search prompts. For arbitrary page requests such as `Сделай запрос к google.com`, it attempts browser `fetch()` and then falls back to an iframe when CORS blocks direct reads.

This works for a limited subset of sources, but it does not provide "search as capture" across Google, Bing, DuckDuckGo, Brave, or other providers.

## Suggested Fix

Add a structured search capture contract to both JavaScript and Rust implementations:

- CLI: `web-capture search "formal-ai" --provider google|bing|duckduckgo|brave|wikipedia --format json|markdown`
- HTTP: `GET /search?q=<query>&provider=<provider>&format=json|markdown`
- Library API: `search({ query, provider, limit, captureMode })`

Suggested JSON result shape:

```json
{
  "query": "formal-ai",
  "provider": "google",
  "captureMode": "browser",
  "capturedAt": "2026-05-18T20:30:00Z",
  "results": [
    {
      "rank": 1,
      "title": "Result title",
      "url": "https://example.com/",
      "snippet": "Readable result snippet"
    }
  ],
  "diagnostics": {
    "status": 200,
    "blockedByCors": false,
    "blockedByCaptcha": false,
    "sourceUrl": "https://www.google.com/search?q=formal-ai"
  }
}
```

Implementation notes:

- Prefer provider-native CORS/API routes where available, e.g. Wikipedia.
- Use browser rendering for search engines that block direct CORS reads.
- Preserve screenshots/HTML/Markdown as optional debug artifacts for blocked/captcha pages.
- Keep JavaScript/Rust CLI and HTTP behavior aligned with the existing endpoint parity goal.

## Why This Belongs In web-capture

Search-provider capture is a specialized version of web capture: it needs browser rendering, normalized extracted links/snippets, structured diagnostics, and provider-specific fallbacks. Putting it in web-capture lets formal-ai and other agents consume one consistent local/server/library interface instead of each client reimplementing provider-specific browser scraping.
