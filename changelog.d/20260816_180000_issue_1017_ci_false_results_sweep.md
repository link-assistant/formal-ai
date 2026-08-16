---
bump: patch
---

### Fixed

- Fix issue #1017 in pull request #1018: make the step execution budget own the
  deadline instead of `timeout-minutes`, so an overrun reports `failure` with an
  `::error` naming the budget rather than degrading into a `cancelled` run and a
  skipped release. Every budget is now checked against the job cap it sits
  under, which surfaced two further at-risk jobs; the macOS core lane runs
  sixteen duration-skew-tolerant slices; `cargo audit` runs on the default
  branch and on a schedule with its one false positive ignored behind a proof
  line CI re-derives; the CodeQL Rust extractor is pinned to a `std` it can
  parse so live code stops being extracted with errors; the link check tests its
  report parser and no longer reports links it never checked; every read-only
  job belongs to a concurrency group that never cancels the default branch; and
  nested CI evidence is no longer silently excluded by `.gitignore`.
