---
bump: patch
---

### Fixed

- One timed-out link no longer makes the Broken Link Checker report every
  healthy redirect in the repository as broken. `extractBrokenUrls` narrowed its
  permissive bullet parser to the failure section by searching for a single
  hard-coded `## Errors per input` heading; lychee writes only the sections a
  run actually has links for, so a report whose sole failure was a timeout
  matched nothing and fell through to parsing the whole document -- including
  `## Redirects per input`. Every `## ... per input` section is now sliced out by
  heading and counts as failing unless it is one of the outcomes known to be
  healthy, so a category this parser has not heard of is reported rather than
  silently dropped. Four new tests cover the report shapes the old lookup got
  wrong; all four fail against the previous parser, and the real report from run
  32454084765 is kept as a fixture in
  `experiments/issue-1021-link-checker-false-positive/`.
