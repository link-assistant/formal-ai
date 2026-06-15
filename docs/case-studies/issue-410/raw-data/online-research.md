# Online Research Notes For Issue 410

Captured on 2026-06-11 while evaluating whether FormalAI should use
`link-assistant/web-search` and `link-assistant/web-capture` as external
components.

## Primary Sources Checked

| Area | Source | Relevant finding |
|---|---|---|
| Meta-search API | <https://docs.searxng.org/dev/search_api.html> | SearXNG exposes HTTP search endpoints with JSON output when `format=json` is requested. |
| Search API | <https://api-dashboard.search.brave.com/documentation> | Brave has official web/search APIs and LLM-oriented search products, but they require service integration rather than anonymous browser fetches. |
| Search + scrape service | <https://docs.firecrawl.dev/api-reference/v2-introduction> | Firecrawl exposes scrape, crawl, map, and search operations, including web search with full-page content. |
| Search endpoint | <https://docs.firecrawl.dev/api-reference/endpoint/search> | Firecrawl search can return result metadata and optional scraped page content such as Markdown. |
| Browser automation | <https://docs.browserbase.com/welcome/quickstarts/stagehand> | Stagehand wraps browser automation for agent workflows and can interoperate with Playwright-style scripts. |
| Browser network control | <https://playwright.dev/docs/network> | Playwright can observe, mock, modify, and replay page network requests; useful for capture/service tests. |
| Browser automation | <https://pptr.dev/> | Puppeteer provides a high-level API for Chrome/Firefox automation, matching the kind of renderer used by capture services. |
| Browser CORS limits | <https://developer.mozilla.org/en-US/docs/Web/HTTP/Guides/CORS> | Browser JavaScript access to cross-origin responses depends on server CORS headers, which keeps many search engines out of a static browser-only plan. |
| MediaWiki CORS | <https://www.mediawiki.org/wiki/API:Cross-site_requests> | MediaWiki APIs support cross-site requests with an `origin=*` parameter for unauthenticated browser calls. |
| Agent search API | <https://docs.tavily.com/documentation/api-reference/endpoint/search> | Tavily provides an API-oriented search endpoint for AI-agent retrieval, but it is a hosted third-party API. |
| SERP API | <https://serpapi.com/search-api> | SerpApi exposes structured Google SERP results through an API endpoint, requiring third-party service integration. |
| Reader/capture API | <https://jina.ai/reader/> | Jina Reader provides HTML-to-Markdown and extraction capabilities for URL content capture. |
| Google search API | <https://developers.google.com/custom-search/v1/overview> | Google Custom Search JSON API can return search results programmatically, but existing customers must transition by 2027. |

## Conclusion From External Components

The external ecosystem has two broad options:

- API search providers such as Brave, Tavily, SerpApi, SearXNG, and Google
  Custom Search. These are useful as provider backends but require keys,
  service configuration, or hosted infrastructure.
- Browser/capture providers such as Playwright, Puppeteer, Browserbase,
  Firecrawl, and Jina Reader. These are useful for fetch/render/capture, but
  they do not replace a FormalAI-compatible provider registry by themselves.

The preferred path remains `link-assistant/web-search` for provider aggregation
and `link-assistant/web-capture` for fetch/render/capture. Third-party services
should remain optional provider backends, not the core FormalAI contract.
