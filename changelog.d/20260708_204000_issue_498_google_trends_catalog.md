---
bump: minor
---

### Added

- Added the issue #498 Google Trends catalog pipeline: parse a Trends RSS snapshot, expand the top 10 searches into multilingual prompt variants, answer every prompt through `FormalAiEngine`, and render the reviewable catalog at `data/meta/google-trends-catalog.lino`.
- Added a `google_trends_catalog` Agent CLI recipe with a pinned session under `docs/case-studies/issue-498`, plus raw Trends/GitHub evidence and tests that keep the seed, generated catalog, recipe routing, and documentation traceable.
