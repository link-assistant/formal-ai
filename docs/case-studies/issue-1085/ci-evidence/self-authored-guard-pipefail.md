# The self-authored re-run guard under `pipefail`

Issue #1085 (D2.3). Recorded 2026-09-08 from the run logs and the pull-request
API of link-assistant/formal-ai.

## What was observed

| Pull request | Authored commits | Runs that authored |
| --- | --- | --- |
| #1093 | 6, all for #1091 | one per pull-request-triggered run |
| #1094 | 2 (`45b0271`, `59e10eb`); the second changed only the evidence bundle | 34206087424 (08:43Z) and 34206203600 (08:45Z) |

The two runs on #1094 did not overlap: the second's checkout ran at 08:52:56Z,
after the first had pushed `45b0271` at 08:50:38Z (its pull-request comment
time). The second run's log shows the "existing pull request" branch taken and
then the authoring step, with no `already carries the authored commit` notice.

## Cause

```
if git log "origin/$branch" --format=%B | grep -Fxq "Formal-AI-Pull-Request: $existing"; then
```

under `set -euo pipefail`. `grep -q` exits at the first match; `git log`
then receives SIGPIPE writing the remaining history and exits 141; with
`pipefail` the pipeline's status is 141, the `if` takes the false branch, and
the task is authored again. The guard therefore failed on exactly the runs
where the trailer was present, and passed (correctly, by accident) only when
the history was short enough to fit in the pipe buffer.

## Fix

- `scripts/self-authored-commit-count.sh <pull-request-url>` counts the
  commits on the pull request whose message carries
  `Formal-AI-Pull-Request: <url>`, entirely in jq over `gh pr view --json
  commits`. No pipe, no grep.
- The workflow decides `done` from that count when it finds the open pull
  request, and checks the count once more immediately before pushing a newly
  authored commit, discarding it if another run landed first.
- Both pushes go through `scripts/push-to-shared-branch.sh` (issue #1081).
- #1093 and #1094 were closed; the run triggered by the fix opened #1097.

## The first version of the replacement was also wrong

`gh pr view --json commits --jq --arg trailer "..." '<expr>'` is not valid:
`--jq` takes one argument, so gh answered `accepts at most 1 arg(s), received
4`. In the push step the count was read inside an `if [[ "$(...)" != 0 ]]`,
where a failing command substitution is not fatal under `set -e`, so the empty
output read as "another run authored it" and run 34223865082 discarded the
commit Formal AI had just authored for #1097. The helper now fetches the commit
bodies with a single `--jq` expression and counts with `grep -c` (which reads
all of its input, so it cannot repeat the SIGPIPE), and both callers assign the
count to a variable so a failure stops the step.

## The base a dispatched run opens against

`BASE_BRANCH` was `github.head_ref || github.event.repository.default_branch`.
`github.head_ref` is empty outside a pull-request event, so the dispatched run
34241163195 opened #1102 against `main`: a pull request carrying all 37 commits
of the branch the change was authored on, instead of the one authored commit.
The base is now `github.head_ref || github.ref_name`, the branch the run checked
out, for dispatch, schedule and the issue trigger alike.

## And the fix reached the runner one run late

Run 34224940281 failed with the same `accepts at most 1 arg(s), received 4`
although its head carried the corrected helper. The step had already run
`git checkout -B "$branch" "origin/$branch"` to continue on the bot branch, so
`scripts/self-authored-commit-count.sh` resolved to that branch's copy: the
pre-fix version, committed when the branch was opened. Every script the
workflow runs after that checkout comes from the branch being authored, not
from the run's own head. The workflow now copies both scripts into
`$RUNNER_TEMP` before anything switches branches and runs them from there.

## What the bot pull request's own CI then showed

#1097 carried exactly one authored commit, by `github-actions[bot]`, with the
four trailers and the evidence bundle, and no human commit. Its checks were red
for one reason of its own: a source change with no changelog fragment. The
authoring contract produced a single artifact, so Formal AI could not satisfy a
gate that every other pull request satisfies.

`scripts/author-change-with-formal-ai.sh` now takes `--produces`/`--into` in
pairs and the workflow reads repeated `produces:`/`into:` lines from the issue,
so one session can write more than one artifact. Asking for the fragment that
way did not work: run 34231781135 gave Formal AI one prompt naming both files,
and it edited the first, answered `Final`, and exited after five seconds
without writing the second. The second clause reached it -- the trace records
the whole prompt -- so it was read and dropped (#1099).

The fragment therefore rides in the bootstrap commit that opens the pull
request. `changelog.d/` is one of the trees the version-3 metric excludes from
both sides of the share, so the fragment is process record either way, and
writing it there keeps the authored commit to the behaviour change. #1097 and
#1098 were closed for a run under the contract that followed.

(The same run's `check_minimal_core_boundary` failure was the pull request's
base being older than the branch, not the authored change.)
