## Issue #960 Enforcing Recorded-But-Unenforced Conventions

Issue [#960](https://github.com/link-assistant/formal-ai/issues/960) collects
three maintainer requirements that each landed once as a practice and were then
written down without anything that fails when they are broken: the 128-record
cache budget (R222-1,
[#222](https://github.com/link-assistant/formal-ai/pull/222#issuecomment-4513844358)),
the tests-as-documentation exact-answer style (R234-2,
[#234](https://github.com/link-assistant/formal-ai/pull/234#issuecomment-4528554549)),
and the `Fixes <url>` pull-request linking rule (R234-4, same thread). A
convention that is recorded but unenforced decays silently: by the time this
issue was filed `data/cache/wikidata/entity` held 406 records against a
documented cap of 128. Timeline, root causes, and the measurements behind the
one deliberate exemption live in `docs/case-studies/issue-960/`.

| ID | Requirement | Status / Evidence |
| --- | --- | --- |
| R960-1 | `data/cache/wikidata/` is inside the 1500-line Links Notation gate, or its exemption is stated with a reason. | Implemented: `scripts/check-file-size.rs` no longer excludes the cache (`EXCLUDE_PATH_FRAGMENTS` is now `dev/log/` only) and `tests/unit/data_files.rs` dropped its blanket `cache` exemption. The cap is actionable rather than aspirational: the largest cached file, `data/cache/wikidata/lexeme/L3302.lino`, is 1347 lines, and `examples/refresh_translation_cache.rs` already splits oversized responses into `<bucket>-partN.lino`. |
| R960-2 | `MAX_SEED_RECORDS_PER_BUCKET = 128` is actively enforced, not merely documented. | Implemented: `scripts/check-cache-budget.rs` (wired into `.github/workflows/release.yml`) fails when a bucket under `data/cache/` exceeds the cap, counting a record once per file stem so `Q1860.json` and `Q1860.lino` are one record. It parses the constant out of `src/translation/cache.rs` so the gate and the library cannot drift. |
| R960-3 | The three closure-driven buckets are exempted explicitly, with a reason and a compensating invariant. | Implemented: `CLOSURE_DRIVEN_BUCKETS` lists `wikidata/entity` (406), `wordnet/en` (332) and `wiktionary/en` (243), each with a written reason — their size is *forced* by the total-closure gate (`scripts/audit-total-closure.py`, `tests/unit/total_closure.rs`), which requires a cached record for every referenced seed token. Measured: 0 of those records are orphans, so trimming to 128 would break closure rather than remove waste. The exemption is paid for by a stricter rule — the check fails if any exempt-bucket record becomes unreferenced — plus a permanent warning so the overflow stays visible. |
| R960-4 | Behavioural tests assert exact answers, enforced repo-wide rather than by convention. | Implemented: `scripts/check-tests-as-docs.rs` flags any `#[test]` that touches `.answer` without an exact assertion (`assert_eq!` on the answer, or membership in an explicit list of exact answers). It is a burn-down ratchet over `scripts/tests-as-docs-allowlist.txt`: new loose-only tests fail, and a row made explicit must be pruned. `tests/unit/assistant_name.rs` is converted as the worked exemplar, which took the allowlist from 399 rows to 398. |
| R960-5 | Pull-request descriptions link their issue with a GitHub closing keyword; `Addresses` is rejected. | Implemented: `scripts/check-pull-request-link.rs` reads `PR_BODY` (or a file) and fails on a missing closing keyword or on a non-closing word (`Addresses`, `Relates to`, `Part of`, `Refs`, `See`) used where one belongs, distinguishing a link verb from prose such as "this fixes the parser crash". CI runs it on every pull request. |
| R960-6 | The conventions are codified where contributors read them. | Implemented: CONTRIBUTING.md § *Project Conventions* rules 12–16 and § *Pull Request Process* steps 5–6, plus `.github/pull_request_template.md`, state the linking syntax, the `docs/case-studies/pull-request-{id}` layout, the `.lino` cap over cached data, the 128-record budget, and the exact-answer test style. |
