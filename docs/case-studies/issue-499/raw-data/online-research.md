# Issue 499 Online Research

Research date: 2026-07-08.

## Google Trends Product Surface

- Google Trends Trending Now: <https://trends.google.com/trending>
  - The Trending Now UI exposes current trending searches, country/region
    filters, time windows, categories, active/lasted status, and export actions
    including RSS and CSV.
  - The RSS endpoint used for this PR is
    `https://trends.google.com/trending/rss?geo=US`.
- Google Search Central, "Get started with Google Trends":
  <https://developers.google.com/search/docs/monitor-debug/trends-start>
  - Identifies Trending Now as the near-real-time trend monitor.
  - Identifies Explore as the historical comparison surface for top/rising
    queries by location, date, category, and search property.
- Google Trends Help, "Understanding the data":
  <https://support.google.com/trends/answer/3076011?hl=en>
  - Documents Trending Now country/region coverage and recent time filters.
  - Documents that the Trending Now data is refreshed frequently, which is why
    this PR commits a snapshot for deterministic tests.

## APIs And Libraries

- Google Search Central blog, "Introducing the Google Trends API alpha":
  <https://developers.google.com/search/blog/2025/07/trends-api>
  - Official API exists as an alpha for limited testers.
  - The alpha is useful future prior art but should not be a required CI
    dependency for issue #499.
- `pytrends`: <https://pypi.org/project/pytrends/>
  - Unofficial Python client for Google Trends.
  - Useful for experiments, but the project itself warns that unofficial clients
    can break when Google changes backend behavior.
- SerpApi Google Trends Trending Now API:
  <https://serpapi.com/google-trends-trending-now-api>
  - Commercial API wrapper around Trending Now.
  - Useful as future operational infrastructure, but unnecessary for the
    committed snapshot converter.
- Apify Google Trends actors: <https://apify.com/search?query=google%20trends>
  - Third-party scraping/automation ecosystem option.
  - Not used in this PR because the RSS feed gives enough traceable source data.

## Local Related Work

- PR #416 / issue #408: broad text/code edit benchmark profiles.
- PR #448 / issue #444: procedural how-to benchmark slice and ratchet tests.
- PR #638 / issue #527: generated question catalog, Agent CLI session pinning,
  and byte-for-byte generated artifact tests.
- Existing fixtures under `data/benchmarks/` use Links Notation records,
  self-authored prompt cases, source provenance, and ratchet tests.

## Design Consequences

- The live Google Trends source is valuable for discovering current demand, but
  tests must consume a saved snapshot.
- The first automated slice should be an RSS snapshot converter because it needs
  no API key, no third-party service, and no network in CI.
- Prompt texts should be self-authored from short topic strings instead of
  copying article bodies or full external datasets.
