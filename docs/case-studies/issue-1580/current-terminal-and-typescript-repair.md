# Terminal collection and TypeScript repair

Published `79a024bffdcee2dd94149d929e81b4b65496ed2d` has 33 terminal workflow
records: 29 succeed, two fail, one skips and the Pipeline is deliberately cancelled.
[Layered CI](https://github.com/link-assistant/formal-ai/actions/runs/45011615137)
fails both the translation parity case (raw line 10224, shard four) and the actual
TypeScript runtime case (15287, shard one). The supplied fixture writer mistakenly
hardcoded success even while retaining those two failed job conclusions. Both fresh
GitHub run and job metadata say failure. The original erroneous fixture stays in
its earlier immutable archive; current provenance assertions reject that claim.

Reproduce with `node --test --test-concurrency=1 rust/tests/web/translate-es.test.mjs`
and, separately, `node --test --test-concurrency=1 rust/tests/web/typescript-runtime.test.mjs`.
Both fail before regeneration. `node scripts/translate-es.mjs --write` regenerates
exactly four stale companions. Both tests then pass (12 translation cases and the
actual TypeScript worker/planner test, including missing and altered projection
controls). `--check` passes, as does `node scripts/translate-js-rust.mjs --verify`.
The 915 measured durations are identical; their source-derived file outcomes now
retain 913 completions and two failures. The corrected receipt test fails before
the correction and passes afterward. Hosted verification of the repair is pending. All 12 focused packaging, source,
worker, documentation and secret controls pass after repair. All nine scheduling
receipt cases, Rust formatting and whitespace pass; no native build ran. Their
complete logs and fresh preflight are retained under checks/current-terminal-validation.

The [failed Pipeline](https://github.com/link-assistant/formal-ai/actions/runs/45011615539)
had already rejected the oversized evidence manifest. One native coding group
completed successfully; nine groups still had no executed steps after more than
35 minutes. Supersession first requested ordinary cancellation at 11:05:55 UTC,
then force cancellation at 11:11:05 after the run failed to terminate for 310 seconds.
No executing job was discarded at the initial request. These unacquired groups
receive zero runtime qualification. Cancellation is retained, never called a pass.

Thirty-two complete whole-run CLI streams are retained. The cancelled Pipeline's
whole-run CLI endpoint returns `log not found: 137178465343`; that failure is kept.
Recovery obtains all 12 executed job streams through the REST job-log endpoints.
Their assembled stream is explicitly derived; no logs are invented for jobs without
steps. Complete paginated check metadata and all 31 annotations are retained.
All 36 diagnostic candidates are classified with original line references.
Negative-control Git, browser-import and file-I/O errors retain their passing test
proof. Printed shell guards and passing test names remain distinct from executed
errors. The crates.io publication preflight remains UNKNOWN. Scope-profile notices
and runner-capacity notices provide no production-path qualification.

The archive at `experiments/issue-1580/evidence/hosted/79a024bf-terminal/`
binds complete streams, original collection failure, recovery, cancellation,
before/after regressions and prior post-archive checks to raw/compressed SHA256
hashes. It does not claim a complete literal review of every workflow stream.
Supplied scripts, patches and conclusions receive zero independent-authorship
credit. Wider [PR #1581](https://github.com/link-assistant/formal-ai/pull/1581)
acceptance remains unfinished; the PR stays draft and unmerged.
