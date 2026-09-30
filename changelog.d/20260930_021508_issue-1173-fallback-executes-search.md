---
bump: patch
---

### Fixed

- The unknown-reasoning fallback now executes its web search for the focus
  phrase instead of answering with the canned "Web search requested for `…`"
  description of the browser demo's machinery. The description arms (default,
  latest-news, open-research, in English and Russian) are deleted as an
  answer surface everywhere: an executed search answers from its captured
  statements with citations and records `web_search:executed`, and an
  offline or cache-miss run degrades to the localized
  `web_search_unavailable` response that names the query, with the live
  transport still behind the `FORMAL_AI_LIVE_FETCH` opt-in (issue #1173).
