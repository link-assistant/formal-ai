# Isolating rerun cancellation groups

The historical failed-consumer rerun for
[run 45085971679](https://github.com/link-assistant/formal-ai/actions/runs/45085971679)
was requested at 08:31:18 UTC on 2026-11-23. At 08:31:24, the queued Clippy
job and all ten native consumers of the newer
[run 45136124038](https://github.com/link-assistant/formal-ai/actions/runs/45136124038)
were cancelled. The current producer and registry jobs had succeeded.
The Clippy job had no assigned runner; its timestamps do not prove execution.

Reruns retain their original pull-request synchronize event. The previous policy
put the historical rerun and the newer initial run in the same cancellation
group. Their source expressions and the observed timestamps support this cause;
the receipt does not claim an explicit GitHub cancellation-reason message.

The repair keeps initial synchronize attempts in the existing cancellable group.
Every rerun gets a group containing its run ID and attempt, with cancellation
disabled. Other independent events keep separate groups, and writer leases keep
cancellation disabled. The shared policy checker and its fixtures enforce the
same rule across 31 workflow files and 75 policies. A byte-level comparison
preserves all other workflow content, except the native explanatory comment.

The seven-case regression evaluates the real native workflow expression. Three
cases fail against the old policy and all seven pass after the repair. It checks
the historical collision, ordinary initial-run cancellation, separate attempts,
numeric and string attempt contexts, independent events and inherited job
concurrency. The old policy remains a failing negative control. An initial
syntax error is retained separately and earns no behavioral reproduction credit.
The regression is imported by the existing default-branch policy CI test.

Run the focused reproduction with:

    timeout 600 node --test --test-concurrency=1 experiments/issue-1580/native-rerun-concurrency.test.mjs

All affected JavaScript files pass:

| Test file | Passed |
| --- | ---: |
| workflow-concurrency | 15 |
| default-branch-build-policy | 177 |
| desktop-release-source-attribution | 31 |
| documentation-only-ci-policy | 37 |
| fast-pull-request-production-profile | 29 |
| hive-delivery-admission | 24 |
| hive-mind-draft-workflow | 14 |
| native-test-artifact | 20 |
| native-workflow-cache-bindings | 13 |
| release-runtime-fixture | 8 |
| native-rerun-concurrency | 7 |

The last seven cases also run through the default-branch policy file, so the rows
are not independent case counts. Nine affected JavaScript gates, formatting of
both changed Rust files, staged-release currency and whitespace checks pass.
The canonical release regeneration retains all 52 operations and 104 bindings;
the generated callee bytes are unchanged. No local Cargo build ran.

Two initial harness failures remain in the receipt. Thousands of duplicate
expression evaluations reached the existing 100 ms VM bound; memoizing identical
expression/context pairs preserves the cases and that bound. An externally
supplied NODE_OPTIONS value was correctly rejected by the source observer;
the ordinary CI environment passes with that variable unset. Neither guard
was relaxed.

The [receipt](../../../experiments/issue-1580/evidence/checks/rerun-cancellation-policy/manifest.json)
binds repair commit bd586bfd24275b7b2fc757dff9d1a2fe74b6cba2 to the
before/after logs, complete gate logs, fresh feedback and conservation proofs.
Every archive member was reopened and checked against its length and SHA-256.
Supplied edits and orchestration receive zero independent authorship credit.

Hosted qualification of this repair, actual artifact rerun qualification,
aggregate source review and the remaining original acceptance criteria are
unfinished. [PR #1581](https://github.com/link-assistant/formal-ai/pull/1581)
remains a draft and is not ready to merge.
