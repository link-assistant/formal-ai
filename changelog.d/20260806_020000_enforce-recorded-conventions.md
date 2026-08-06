---
bump: minor
---

### Added
- `scripts/check-cache-budget.rs`: CI gate enforcing `MAX_SEED_RECORDS_PER_BUCKET = 128`
  for every bucket under `data/cache/`, reading the cap from `src/translation/cache.rs`
  so gate and library cannot drift. The three buckets whose size is forced by the
  total-closure gate are exempted explicitly, with a written reason and a stricter
  no-orphan invariant (issue #960).
- `scripts/check-tests-as-docs.rs` and `scripts/tests-as-docs-allowlist.txt`: burn-down
  ratchet requiring behavioural tests to assert exact answers instead of substrings,
  so a test reads as documentation (issue #960).
- `scripts/check-pull-request-link.rs`: fails a pull request whose description does not
  close its issue with a GitHub keyword, or writes `Addresses #N` where `Fixes #N`
  belongs (issue #960).

### Changed
- The 1500-line Links Notation cap now covers `data/cache/wikidata/` too;
  `scripts/check-file-size.rs` and `tests/unit/data_files.rs` no longer exempt cached
  data (issue #960).
- CONTRIBUTING.md and `.github/pull_request_template.md` codify the issue-linking
  syntax, the `docs/case-studies/pull-request-{id}` layout, the cached-`.lino` cap, the
  128-record cache budget, and the exact-answer test style (issue #960).
