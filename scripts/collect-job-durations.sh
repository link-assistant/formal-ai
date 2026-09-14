#!/usr/bin/env bash
# Collect measured GitHub Actions job durations for the headroom audit.
#
# WHY THIS EXISTS
#
# `timeout-minutes` is meant to be a backstop, not a deadline. When the work a
# job does grows until it routinely uses most of its cap, the cap stops being a
# safety net and becomes the thing that ends the job -- and GitHub reports a
# `timeout-minutes` kill as **cancelled**, not **failed** (issue #977). That is
# a false negative by construction: the run goes grey, not red.
#
# Nothing in the repository noticed that happening to the Coverage job, which
# was measured at 100.7% of its 40-minute cap. Nothing would notice the next
# one either, because a cap is a constant in a YAML file and the duration it is
# supposed to bound is only ever observed on GitHub. This script fetches the
# observations so `scripts/check-job-headroom.rs` can compare the two.
#
# The same blind spot exists one level down. Issue #1081 found the macOS
# specification shard killed at 100.1% of the *step* budget that was supposed
# to be its deadline, twice, while every gate in the repository only ever
# compared that budget upward against the job cap. A budget is also a constant
# in a YAML file, so this script now fetches the step observations too.
#
# OUTPUT
#
# Tab-separated rows, no header, to stdout. The field count says which kind a
# row is -- six fields is a job, eight is a step:
#
#   run_id <TAB> workflow <TAB> job <TAB> conclusion <TAB> started_at <TAB> completed_at
#   run_id <TAB> workflow <TAB> job <TAB> conclusion <TAB> started_at <TAB> completed_at
#            <TAB> step <TAB> step_name
#
# On a step row the conclusion and the timestamps are the *step's*, the literal
# `step` in field seven marks the kind, and field eight is the step's name as
# the workflow writes it. Steps that never ran are omitted: a skipped step
# carries null timestamps and says nothing about how long the work takes.
#
# `job` is the job's *display* name as GitHub reports it, which for a job
# reached through `workflow_call` is "<caller job> / <inner job>", and for a
# matrix leg has the expanded values substituted in.
#
# USAGE
#   scripts/collect-job-durations.sh [BRANCH] [RUNS] > durations.tsv
#
#   BRANCH  branch to sample (default: the repository's default branch)
#   RUNS    how many of the most recent runs to sample (default: 300)
#
# Requires `gh` (authenticated) and `jq`, both preinstalled on GitHub-hosted
# runners. Set GH_TOKEN when running under Actions.
set -euo pipefail

BRANCH="${1:-}"
RUNS="${2:-300}"

REPOSITORY="${GITHUB_REPOSITORY:-$(gh repo view --json nameWithOwner --jq .nameWithOwner)}"
if [ -z "$BRANCH" ]; then
  BRANCH="$(gh api "repos/${REPOSITORY}" --jq .default_branch)"
fi

log() { printf '%s\n' "$*" >&2; }

log "Sampling up to ${RUNS} runs of ${REPOSITORY} on ${BRANCH}."

# Paged explicitly rather than with `gh api --paginate | head`: closing the
# pipe early sends SIGPIPE to `gh`, and under `set -o pipefail` that fails the
# whole command substitution with status 141 and an empty result.
run_ids=""
collected=0
page=1
while [ "${collected}" -lt "${RUNS}" ]; do
  page_rows="$(
    gh api "repos/${REPOSITORY}/actions/runs?branch=${BRANCH}&per_page=100&page=${page}" \
      --jq '.workflow_runs[] | [(.id|tostring), .name] | @tsv'
  )"
  [ -n "${page_rows}" ] || break
  run_ids="${run_ids}${page_rows}"$'\n'
  collected=$((collected + $(printf '%s\n' "${page_rows}" | wc -l)))
  page=$((page + 1))
done
run_ids="$(printf '%s' "${run_ids}" | sed -n "1,${RUNS}p")"

count=0
while IFS=$'\t' read -r run_id workflow; do
  [ -n "${run_id}" ] || continue
  count=$((count + 1))
  # A run whose jobs have aged out of retention returns an empty list rather
  # than an error, so an absent run costs one request and produces no rows.
  # One job row followed by that job's step rows. `gh api --jq` takes no
  # `--arg`, so the run id and workflow name are prefixed by the shell rather
  # than built into the filter, and the row is passed through whole.
  gh api --paginate \
    "repos/${REPOSITORY}/actions/runs/${run_id}/jobs?per_page=100&filter=latest" \
    --jq '.jobs[] | . as $job
          | ([$job.name, ($job.conclusion // "null"), ($job.started_at // ""),
              ($job.completed_at // "")] | @tsv),
            ($job.steps[]?
             | select(.started_at != null and .completed_at != null)
             | [$job.name, (.conclusion // "null"), .started_at, .completed_at,
                "step", .name] | @tsv)' \
    2>/dev/null \
    | while IFS= read -r row; do
        printf '%s\t%s\t%s\n' "${run_id}" "${workflow}" "${row}"
      done
  if [ $((count % 25)) -eq 0 ]; then
    log "  ... ${count} runs"
  fi
done <<< "${run_ids}"

log "Collected jobs from ${count} runs."
