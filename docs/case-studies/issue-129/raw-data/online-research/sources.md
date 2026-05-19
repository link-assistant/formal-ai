# Online Research Sources

These sources were reviewed on 2026-05-19 while designing the issue #129
connectivity diagnostics page.

| Topic | Source | Relevance |
| --- | --- | --- |
| CORS and readable cross-origin fetches | [MDN: Cross-Origin Resource Sharing](https://developer.mozilla.org/en-US/docs/Web/HTTP/Guides/CORS) | Browser `fetch()` and XHR follow same-origin policy unless the response opts in with CORS headers; JavaScript receives only a generic error for many CORS failures. |
| Same-origin policy | [MDN: Same-origin policy](https://developer.mozilla.org/en-US/docs/Web/Security/Same-origin_policy) | Explains why scripts loaded from GitHub Pages cannot freely read arbitrary provider responses. |
| Fetch API | [MDN: Fetch API](https://developer.mozilla.org/en-US/docs/Web/API/Fetch_API) | The dashboard uses browser `fetch()` as the direct diagnostic mechanism. |
| Iframes | [MDN: `<iframe>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/iframe) | The dashboard uses inline frames and an expanded frame overlay to test embeddability. |
| Frame blocking | [MDN: X-Frame-Options](https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/X-Frame-Options) and [MDN: CSP frame-ancestors](https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Content-Security-Policy/frame-ancestors) | Providers can block embedding even when a URL can be opened directly. |
| GitHub Pages static artifacts | [GitHub Docs: Creating a GitHub Pages site](https://docs.github.com/en/pages/getting-started-with-github-pages/creating-a-github-pages-site) | GitHub Pages publishes static files from the configured source/artifact and preserves directory structure, so `src/web/tests/index.html` is the correct artifact shape. |
| Wikimedia search | [Wikimedia API Portal: Search](https://api.wikimedia.org/wiki/Core_REST_API/Reference/Search) | Used for the Wikipedia REST search row. |
| Wikidata API | [Wikidata: Data access](https://www.wikidata.org/wiki/Wikidata:Data_access) | Documents Wikidata data access and `wbsearchentities`, used for the Wikidata row. |
| Open Library search | [Open Library Search API](https://openlibrary.org/dev/docs/api/search) | Used for the Open Library row. |
| OpenAlex API | [OpenAlex API introduction](https://developers.openalex.org/api-reference/introduction) | Used for the OpenAlex works row. |
| Crossref API | [Crossref REST API Swagger](https://api.crossref.org/swagger-ui/index.html) | Used for the Crossref works row. |
| Semantic Scholar Graph API | [Semantic Scholar Academic Graph API](https://api.semanticscholar.org/api-docs/graph) | Used for the Semantic Scholar paper-search row. |
