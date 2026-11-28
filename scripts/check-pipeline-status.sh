#!/usr/bin/env bash
# Turn a silently-"cancelled" pipeline into a red failure (issue #977).
#
# WHY THIS EXISTS
#
# The most damaging CI/CD defect this repository has had was not a failing
# check -- it was an invisible one. GitHub reports a job killed by its own
# `timeout-minutes` as **cancelled**, not **failed**, and a workflow run whose
# only non-success job is cancelled gets the run conclusion `cancelled` too.
# On the default branch that is indistinguishable from "a human pressed cancel"
# or "a newer push superseded this run": no red X, no failure notification, and
# no entry in the failed-runs view.
#
# That is how eighteen consecutive `CI/CD Pipeline` runs on `main` between
# 2026-08-03 and 2026-08-06 went untriaged while `Auto Release` was timing out
# mid-Docker-build. Eleven versions (0.326.2 .. 0.333.0) were published to
# crates.io and tagged in git with no GitHub Release at all, because the
# `Create GitHub Release` step sits after the Docker push and was never
# reached. The full reconstruction is in
# dev/log/issues/977/pulls/978/ANALYSIS.md.
#
# A cancelled dependency is incomplete on every ref. On a feature branch it
# may be a superseded run, but that does not prove its required checks passed.
# In particular, a job reaching its own deadline must never make the current
# pull request green. Preserve the cancelled measurement as an error.
#
# INPUT
#   NEEDS_JSON  `toJSON(needs)`: a map of job-id -> {result, outputs}.
#   IS_MAIN     "true" when this is a push to the default branch.
set -euo pipefail

: "${NEEDS_JSON:?NEEDS_JSON is required (pass toJSON(needs))}"
IS_MAIN="${IS_MAIN:-false}"

# Missing or unknown observations cannot establish a successful pipeline.
# Slurping also rejects a stream containing more than one JSON document.
if ! printf '%s' "$NEEDS_JSON" | jq -e -s '
  length == 1 and (.[0] |
    type == "object" and length > 0 and
    all(.[]; type == "object" and
      (.result | . == "success" or . == "failure" or
        . == "cancelled" or . == "skipped")))
' >/dev/null; then
  echo "::error::Pipeline dependency results are missing or invalid. Expected a nonempty job map with known results."
  exit 1
fi

# jq is preinstalled on GitHub-hosted runners.
select_by_result() {
  printf '%s' "$NEEDS_JSON" \
    | jq -r --arg want "$1" \
        'to_entries | map(select(.value.result == $want) | .key) | join(", ")'
}

failed="$(select_by_result failure)"
cancelled="$(select_by_result cancelled)"

echo "Failed jobs:    ${failed:-<none>}"
echo "Cancelled jobs: ${cancelled:-<none>}"

status=0

if [ -n "$failed" ]; then
  echo "::error::Pipeline failed. Failing jobs: ${failed}"
  status=1
fi

if [ -n "$cancelled" ]; then
  if [ "$IS_MAIN" = "true" ]; then
    echo "::error::Pipeline has cancelled jobs on main: ${cancelled}. A job killed by 'timeout-minutes' is reported as cancelled, which would otherwise hide the failure. Check each job's annotations for 'has exceeded the maximum execution time'."
    status=1
  else
    echo "::error::Pipeline has cancelled jobs: ${cancelled}. Required checks are incomplete; inspect their logs for a timeout or cancellation."
    status=1
  fi
fi

if [ "$status" -eq 0 ]; then
  echo "All required jobs succeeded or were legitimately skipped."
fi

exit "$status"
