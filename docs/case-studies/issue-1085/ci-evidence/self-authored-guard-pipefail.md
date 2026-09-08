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
