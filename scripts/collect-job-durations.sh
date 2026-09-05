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
# OUTPUT
#
# One tab-separated row per job per run, no header, to stdout:
#
#   run_id <TAB> workflow <TAB> job <TAB> conclusion <TAB> started_at <TAB> completed_at
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
  gh api --paginate \
    "repos/${REPOSITORY}/actions/runs/${run_id}/jobs?per_page=100&filter=latest" \
    --jq '.jobs[] | [.name, (.conclusion // "null"), (.started_at // ""), (.completed_at // "")] | @tsv' \
    2>/dev/null \
    | while IFS=$'\t' read -r job conclusion started completed; do
        printf '%s\t%s\t%s\t%s\t%s\t%s\n' \
          "${run_id}" "${workflow}" "${job}" "${conclusion}" "${started}" "${completed}"
      done
  if [ $((count % 25)) -eq 0 ]; then
    log "  ... ${count} runs"
  fi
done <<< "${run_ids}"

log "Collected jobs from ${count} runs."
