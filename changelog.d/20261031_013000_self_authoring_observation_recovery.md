### Fixed

- Retry transient GitHub JSON reads within finite time, response and attempt
  bounds, and observe publication jobs in smaller exact-attempt pages.
- Reconcile ambiguous release writes for the same observed GitHub 502/503/504
  error class before another creation attempt, preserving exact tag/source checks.
- Defer the optional post-publication authoring canary after exhausted GitHub
  server errors, with explicit diagnostics and no authoring or self-development
  credit. Source, authorization and code failures remain failures.
- Request the selected author's required App permissions and prevent checkout's
  default credential from shadowing that token. Keep live authoring out of PR
  checks; replay captured publication and HTTP failures in fast JavaScript tests.
