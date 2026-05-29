---
bump: minor
---

### Added
- Software-project follow-up handler so a decomposed agent step such as "test it
  by scraping wikipedia.org and show me the top 10 most frequent words" stays
  bound to the active project dialogue. It formalizes a `software_project_followup`
  meaning (parent request, follow-up kind, target site, expected output) with
  `generated_code`, `test_execution`, and `network_access` approval gates instead
  of running the test. Verification/execution/demonstration verbs are recognized
  across all supported languages (en, ru, hi, zh), and the handler is mirrored in
  both the Rust solver and the browser worker (issue #341).

### Fixed
- A software-project test/run/verify follow-up no longer misroutes to a
  `wikipedia` concept lookup (online) or the unknown-intent opener (offline)
  after the first plan turn (issue #341).
