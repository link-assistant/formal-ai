# Artifact transport and child-memory publication checkpoint

PR: https://github.com/link-assistant/formal-ai/pull/1581

Published `90b5159008da68c5c73534a5b5a4963b81deb04a` has 27 terminal
workflows: 24 successful, one explicitly skipped and two failed. All
312 retained command streams pass compressed/raw hashes,
byte lengths and line counts. The pipeline has no failing job and passes
573 Rust cases plus eight actual registry CLI cases.
All three Hive shards pass within their unchanged twenty-minute clocks.
Per-job durations and original summary locations remain in the exact-head
assessment; this does not restore unmet original budgets or live acceptance.

## Complete observed failure batch

[Workflows 44804037588](https://github.com/link-assistant/formal-ai/actions/runs/44804037588)
uploads the Windows artifact successfully, then its first matching-name lookup
fails 1.073 seconds after finalization. Backend visibility delay is an inference,
not a proven service cause. The bounded adapter repair retries actual pinned
raw transport at most three times, resetting partial output before retries.
Digest, selector, cancellation, extraction and terminal failure boundaries stay
strict. Seven controls fail before repair. All 17 final artifact controls pass.
Actual next-head Windows, macOS and Linux behavior remains to be observed.

[Layered CI 44804037844](https://github.com/link-assistant/formal-ai/actions/runs/44804037844)
rejects an exact sparse-checkout expectation that omitted three newly consumed
timing inputs. Update only that expectation: all 18 profile and 50 shard/checkout
cases pass; production checkout already includes the three actual files.

The original manual Security producer succeeds with clean strict SARIF but
retains the 1,800 MB query-loading warning. The explicit launcher heap was
ineffective because query execution resolves a child evaluator. The candidate
retains a 4 GiB total allowance, resolves the actual dataset with zero off-heap,
requires a 2–4 GiB heap, and rejects retained runtime diagnostic lines before
decoding. The changed policy control fails before repair, and all 19 groups
pass afterward. The bounded 128 MiB configuration probes are not hosted query
execution. Actual resolved allocation and warning elimination remain pending.
See the [child-memory evidence](codeql-child-memory-checkpoint.md).

The final seal also finds thirteen abbreviated evidence filenames introduced
by the previous publication. Rename physical archives and their manifest paths,
preserving every original/compressed hash and historical source label. The
existing ninety-name ratchet is unchanged. Gates run after exact staging so
new files are included; retain the earlier failed gate and its correction.

## Remaining acceptance

Full pinned Rust literal review reaches 347/373 text identities and 44,421
original lines. The complete 1,433-line Cargo parent response distinguishes
compatible semver, platform, feature and dependency-kind constraints. All 267
original CodeQL release-metadata lines are read through five exact byte ranges,
then reassembled and hash-verified. Historical release claims establish no
present runtime or publication result. One binary image is reviewed separately.
Twenty-six identities and every unqualified practice remain open.

Fresh registry responses still show mandatory Chrono in command-stream 1.13.0
and link-calculator 0.24.0. A closed upstream issue is not a published migration.
Complete issue and PR conversation/inline/review feedback is retained. The failed
attempt to retrieve a completed Hive job log while its workflow was still
running is preserved separately from the eventual terminal downloads.

Strict original whole-source acceptance stays 31/53 with 22 separate scoped
assertion reviews. Remaining size/token/dependency warnings, all eight complete
template/practice comparisons, macro child semantics, actual Pages fallback and
publication, served/runtime/model identities, production Docker delivery,
independent reasoning and complete PR review remain unfinished. No local Rust
build or test runs. Supplied automation earns zero independent authorship;
accepted reasoning remains 0/20. Keep this PR draft and unmerged.
