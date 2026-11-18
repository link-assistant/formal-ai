# Current-head CI investigation and repair

PR: https://github.com/link-assistant/formal-ai/pull/1596

All 21 runs for `69bcdb797344f2ee4957711675bb235ffbe9c70c` finished before
this repair batch: 18 passed and three failed. These runs were created on
2026-11-17 at 14:38:47 UTC, after that commit was pushed. Older runs were not
used as current evidence. Full downloaded logs, source checks, supplied
procedures and actual driver receipts are retained in
`ci-round-1-evidence.txt.gz` (uncompressed hash in `provenance.json`).

| Failed run | Exact finding and repair |
| --- | --- |
| [Security 44105386994](https://github.com/link-assistant/formal-ai/actions/runs/44105386994) | Lines 543–550 of `security-44105386994.log`: chrono 0.4.45, unmaintained, RUSTSEC-2026-0543; no patched version. Both dependency owners already have open migration reports. Add the repository's existing exact-ID, reachability-checked upstream exception; keep all other audit failures fatal. |
| [CI/CD Pipeline 44105387525](https://github.com/link-assistant/formal-ai/actions/runs/44105387525) | Lines 447–449 of the failed-job log: role registry differs after adding the calendar offset role. Regenerate `data/seed/roles.lino` with its canonical generator. The hosted native type and lint check passed. |
| [Layered CI 44105386828](https://github.com/link-assistant/formal-ai/actions/runs/44105386828) | Lines 4177–4202 and 7325–7335 of `layered-ci-44105386828.log`: both source-witness tests reject the stale hash of `rust/tests/unit/mod.rs` after registering the new regression. Refresh using the maintained explicit reviewed-reader/previous-witness command. Independently compare every complete object: exactly four fixture hashes changed; no tests, assertions, unsupported cases or native-source records changed. |

The role check and both source-witness tests reproduced their failures locally
before regeneration. After the repairs the two witness suites pass 36 tests,
and the issue's 29-test regression passes again. The full 138-entry local gate
inventory was rerun sequentially; build-dependent and hosted-context checks
remain CI-only. Final CI status belongs to the final commit's GitHub checks.

## Maintenance advisory boundary

The newest published command-stream 1.13.0 and link-calculator 0.24.0 still
declare chrono as an unconditional normal dependency. Neither a downstream
version bump nor a feature toggle can remove it. Existing reports:
https://github.com/link-foundation/command-stream/issues/238 and
https://github.com/link-assistant/calculator/issues/265. The audit configuration
acknowledges this informational notice; it does not repair chrono maintenance.
The existing proof checker rejects this exception once chrono leaves the
resolved build graph. No application dependency or audit command was changed.

All repair procedures and conclusions were supplied by Codex and executed
through JavaScript Formal AI. They add zero autonomous reasoning/authorship
credit. Direct read-only inspections are described in the parent README.
