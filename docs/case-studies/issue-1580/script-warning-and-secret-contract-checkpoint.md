# CI script sizes and complete secret-inventory contracts

Pull request: https://github.com/link-assistant/formal-ai/pull/1581

The published 76371322 Layered CI run fails one source assertion at
`rust/tests/web/fast-pull-request-production-profile.test.mjs:442`.
The complete retained log has 12,590 lines; lines 5,987–6,003 identify its
expectation of the former `git ls-files -z | exclude` spelling. The repaired
scanner uses `read_inventory` to wait for the actual producer, rejecting partial
Git output. The profile now requires both selection calls through this helper
and the producer-status check. History depth, acquisition, permissions, selection
filters, tool versions and finite job phases remain asserted.

The unchanged source assertion fails locally before the update and all 18
production-profile cases pass afterward. The separate subprocess suite exercises
actual shell selection with partial failing all-files, base-diff and uncommitted
inventories, plus a valid empty selection. It also retains the real compressed
secret and invalid-gzip controls. The assertion repair grants no clean-inventory
credit to an unsuccessful producer.

Three scripts previously exceeded the 900-line Rust warning threshold:

| Script | Before | After | Companion |
| --- | ---: | ---: | --- |
| assemble-requirements.rs | 989 | 724 | 289 lines / 22 unchanged tests |
| normalize-ordered-lists.rs | 960 | 684 | 308 lines / 17 unchanged tests |
| self-hosting-metric.rs | 903 | 772 | 136-line CLI module |

The first two move their entire existing test module into a companion admitted
through the same `#[cfg(test)]` declaration. The third moves command-line parsing
and dispatch into a private child module; its entry point is visible only to its
parent. Existing public measurement APIs and all CLI branches remain present.
The exact inverse before formatting and the formatter-normalized inverse after
formatting reproduce every original source byte. No file-size limit or exclusion
changes, and all six files pass rustfmt checks without compiling Rust locally.

Six focused JavaScript files pass one at a time: production-profile, subprocess
boundaries, script twins, ordered module lists, metric policy and metric ledger.
Requirements assembly and all three relevant JavaScript gates pass. The first
rustfmt failure and a rejected invocation naming two nonexistent gates are
retained with their corrected results. These failures are not counted as passes.
The corrected gate run peaks at 143,708 KiB child RSS; no local Rust build or test
is executed. Hosted native execution for these structural changes remains open.

All helper sources, actual traces, outputs, reconstruction proofs and failed
attempts are compressed under
`experiments/issue-1580/evidence/checks/script-warning-and-secret-contract/`.
The manifest binds original/compressed hashes, byte counts and line counts.
Supplied commands and patches earn zero independent authorship or reasoning
credit. This scoped repair leaves the remaining warnings and every broader
unverified original acceptance criterion unfinished; PR #1581 remains draft.
