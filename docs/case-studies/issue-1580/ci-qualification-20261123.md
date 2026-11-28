# Terminal CI and historical artifact rerun qualification

The terminal cohort at 8190c9efaa82777e576b7bae18297d2de33dcf9d has 31
successful workflows, one explicit skip and one cancelled workflow. Its
[Pipeline run](https://github.com/link-assistant/formal-ai/actions/runs/45136124038)
reports no failed prerequisite jobs and cancelled `native-type-check` checks.
The Pipeline Status job exits 1 at 09:29:06 UTC on 2026-11-23. These are distinct
workflow and job conclusions. The producer and registry checks succeeded; the
queued Clippy job and ten consumers were cancelled without executing.

The complete available status log is indexed in the
[terminal cohort receipt](../../../experiments/issue-1580/evidence/checks/terminal-8190-ci-20261123/manifest.json).
The ordinary CLI whole-log command failed because one cancelled job has no log.
The available ZIP was verified, and missing logs earn no execution credit.
[Layered CI](https://github.com/link-assistant/formal-ai/actions/runs/45136122908)
passes the previously stale fixture witness. The cancellation reproduction and
local repair are described in [rerun cancellation](rerun-cancellation.md).
Qualification of that repair on a new hosted head remains pending.

The actual `gh run rerun --failed` operation on historical
[run 45085971679](https://github.com/link-assistant/formal-ai/actions/runs/45085971679/attempts/2)
reexecuted all three previously failing consumers. It retained artifact
14778573986 from attempt 1, with no producer reexecution or attempt-2 upload.
Each consumer log identifies that artifact and actual native test execution.
GitHub copied successful jobs into the new attempt with new IDs but old start
times; comparison with the rerun request time prevents counting those as runs.

| Reexecuted consumer | Job | Native result |
| --- | --- | --- |
| reasoning-1 | 137573053282 | 80 passed, 2 failed |
| reasoning-2 | 137573053334 | 83 passed, 1 failed |
| interfaces | 137573053365 | 79 passed, 1 failed |

The historical assertions still fail: source-refusal provenance at reasoning-1
log lines 863-870, missing ambiguity wording at 895-899, bare documentation
wording at reasoning-2 lines 883-888, and the named synthesis skill gap at
interfaces lines 881-890. The full logs and available archive remain in the
[actual failed-consumer receipt](../../../experiments/issue-1580/evidence/checks/actual-failed-consumer-rerun-20261123/manifest.json).
This demonstrates original-artifact lookup and execution, not passing tests for
the current source. The compared artifact helper paths match head 8190; this
receipt does not claim that every later edit is identical.

The dated [upstream refresh](../../../experiments/issue-1580/evidence/checks/upstream-blocker-refresh-20261123-0946/manifest.json)
confirms that [command-stream PR 240](https://github.com/link-foundation/command-stream/pull/240)
is merged, while the latest published crate remains 1.13.0. The latest
link-calculator is 0.24.0 and its
[migration issue 265](https://github.com/link-assistant/calculator/issues/265)
is open. Wrong-repository lookups and a mistaken pull-request lookup were
retained with their errors and receive zero verification credit. The
[CodeQL included-file context issue](https://github.com/github/codeql/issues/23198)
and [serving-identity report](https://github.com/link-assistant/hive-mind/issues/3909)
also remain open. A merged upstream change does not establish published
migration availability or repair either dependency graph.

All sealed archive members were reopened and checked against their lengths and
SHA-256 digests. Supplied orchestration receives zero independent authorship
credit. Aggregate source review, complete template practice acceptance and the
remaining original criteria are unfinished.
[PR #1581](https://github.com/link-assistant/formal-ai/pull/1581) stays draft
and unmerged.
