# Issue 498 Online Research

Research collected on 2026-07-08 for
<https://github.com/link-assistant/formal-ai/issues/498>.

## Google Trends And Search Demand

- The issue points to the Google Trends trending page:
  <https://trends.google.com/trending?hl=ru&&geo=US>. It is a live search
  demand surface, so the top topics change over time.
- The collected RSS feed is
  <https://trends.google.com/trending/rss?geo=US&hl=ru>. The checked-in raw
  snapshot is `google-trends-us-rss.xml`, and the deterministic seed rendered
  from it is `data/seed/google-trends-snapshot.lino`.

Design implication: use the live feed only to refresh seed data. Tests should
consume a checked-in snapshot so failures represent code or artifact drift, not
normal Google Trends churn.

## Google Trends API

- Google documents a Google Trends API alpha at
  <https://developers.google.com/search/apis/trends>. The page says the API
  provides programmatic access to Google Trends for analysis of search behavior
  and trends, with early access through an alpha application process.
- Google's Search Central announcement
  <https://developers.google.com/search/blog/2025/07/trends-api> says the API
  is initially available to a limited number of testers, provides consistently
  scaled search-interest data, supports a rolling five-year window, and includes
  regional/subregional breakdowns.

Design implication: the official Trends API is the right long-term production
direction for richer search demand data, but this PR should not depend on alpha
approval or credentials.

## pytrends

- The `pytrends` repository
  <https://github.com/GeneralMills/pytrends> describes itself as an unofficial
  Google Trends API and says it automates downloading Google Trends reports.
  Its README also warns that it is only good until Google changes its backend.
- Its documented methods include trending searches, realtime search trends,
  related topics, related queries, interest over time, and interest by region.

Design implication: pytrends is useful prior art for what people automate, but
adding it as a dependency would make the Rust test path less deterministic than
the small RSS parser used here.

## Selected Slice

The implemented slice converts one Google Trends RSS snapshot into top 10
topics, multilingual prompt variants, and an answered Formal AI catalog. This
creates the training/test surface requested by the issue while leaving richer
API-backed analytics as a future data-source upgrade.
