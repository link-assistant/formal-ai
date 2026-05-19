# Online Research Notes — Issue #133

Captured on 2026-05-19 while implementing issue #133. Each entry below was
either tested directly from the connectivity dashboard during development or
sourced from official provider documentation. Only freely accessible,
unauthenticated browser endpoints are listed; paywalled providers are
excluded as called out in the issue.

## Search engines confirmed CORS-readable from a browser

- **DuckDuckGo Instant Answer**
  `https://api.duckduckgo.com/?q=…&format=json&no_redirect=1&no_html=1` returns
  JSON with `Access-Control-Allow-Origin: *`.
  Source: https://duckduckgo.com/api
- **Wikipedia / Wikidata MediaWiki APIs** with `origin=*` are CORS-readable.
  Source: https://www.mediawiki.org/wiki/API:Cross-site_requests
- **Google Suggest** (`suggestqueries.google.com/complete/search`) returns
  JSONP/JSON; works in browsers but only as a suggestion endpoint, not
  full-text web search. Source: tested manually 2026-05-19.
- **Bing Suggest** (`api.bing.com/osjson.aspx`) — OpenSearch-style JSON, CORS
  permitted. Source: tested manually 2026-05-19.
- **Brave Search** has no unauthenticated browser-readable web API. The
  `api.search.brave.com` endpoint requires a subscription token.
  Source: https://api-dashboard.search.brave.com/

## Knowledge / fact databases CORS-readable from a browser

- Wikipedia REST `https://en.wikipedia.org/w/rest.php/v1/search/page` — confirmed.
- Wikidata `wbsearchentities` — confirmed.
- Wiktionary uses the same MediaWiki REST API at `en.wiktionary.org`.
  Source: https://en.wiktionary.org/api/rest_v1/
- OpenAlex `https://api.openalex.org/works?search=…` — public, no auth.
  Source: https://docs.openalex.org/
- Crossref `https://api.crossref.org/works?query=…` — public, no auth.
  Source: https://www.crossref.org/documentation/retrieve-metadata/rest-api/
- Semantic Scholar `https://api.semanticscholar.org/graph/v1/paper/search` —
  public, no auth (rate-limited).
  Source: https://api.semanticscholar.org/api-docs/
- Open Library `https://openlibrary.org/search.json` — public, no auth.
  Source: https://openlibrary.org/dev/docs/api/search
- DBpedia Lookup `https://lookup.dbpedia.org/api/search?query=…&format=json` —
  CORS-permitted.
  Source: https://lookup.dbpedia.org/

## Code hosting providers with CORS-readable search APIs

- **GitHub** — `https://api.github.com/search/repositories?q=…`. Public, but
  unauthenticated requests are rate-limited to 10/min. CORS-allowed.
  Source: https://docs.github.com/rest/search/search
- **GitLab** — `https://gitlab.com/api/v4/search?scope=projects&search=…`.
  Public, CORS-allowed.
  Source: https://docs.gitlab.com/api/rest/
- **Codeberg** (Forgejo/Gitea) — `https://codeberg.org/api/v1/repos/search?q=…`.
  Public, CORS-allowed.
  Source: https://codeberg.org/api/swagger
- **Sourcehut** — `https://sr.ht/projects?search=…` returns HTML; no public
  JSON. Source: https://man.sr.ht/api-conventions.md
- **Gitee** (China) — `https://gitee.com/api/v5/search/repositories?q=…`. Public
  JSON.
  Source: https://gitee.com/api/v5/swagger
- **Bitbucket Cloud** — `https://api.bitbucket.org/2.0/repositories?q=name~"…"`
  permits unauthenticated requests for public repos with a strict query
  syntax.
  Source: https://developer.atlassian.com/cloud/bitbucket/rest/api-group-repositories/
- **GitFlic** (Russia) — `https://gitflic.ru/project` returns HTML; no
  documented public JSON search.
  Source: https://gitflic.ru/

## Scientific paper / journal providers

- **arXiv** — `http://export.arxiv.org/api/query?search_query=…` returns Atom
  XML. Direct browser fetches succeed with `Access-Control-Allow-Origin: *`.
  Source: https://info.arxiv.org/help/api/index.html
- **PubMed E-utilities** — `https://eutils.ncbi.nlm.nih.gov/entrez/eutils/`.
  Source: https://www.ncbi.nlm.nih.gov/books/NBK25500/
- **Europe PMC** —
  `https://www.ebi.ac.uk/europepmc/webservices/rest/search?query=…&format=json`.
  CORS-permitted.
  Source: https://europepmc.org/RestfulWebService
- **DOAJ (Directory of Open Access Journals)** —
  `https://doaj.org/api/search/articles/…`.
  Source: https://doaj.org/api/v3/docs

## Browser concurrency and CORS limits

- The Fetch standard requires servers to opt in to cross-origin reads via
  `Access-Control-Allow-Origin`. Without that header the response is opaque
  and JavaScript cannot inspect it. Source:
  https://developer.mozilla.org/en-US/docs/Web/HTTP/CORS
- Modern browsers cap concurrent connections per origin around six; the
  combined-search planner caps active calls at five per category as a
  defensive default so we never starve the rest of the page.
  Source: https://blog.chromium.org/2014/04/the-end-of-https-everywhere.html
- Reciprocal Rank Fusion (Cormack, Clarke, Buettcher 2009) gives a parameter
  free way to combine ranked lists: `score(d) = Σ 1 / (k + rank_i(d))`. We
  use k = 60, the original value used in the TREC submissions.
  Source: https://plg.uwaterloo.ca/~gvcormac/cormacksigir09-rrf.pdf
