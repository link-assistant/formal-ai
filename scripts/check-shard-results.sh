#!/usr/bin/env bash
# The verdict of a sharded check: green only when every shard it needs was.
#
# A long job split into matrix shards reports one check per shard, and a
# matrix job's `needs.<job>.result` is `success` only when every leg was. The
# aggregator job that follows the shards is the one check that speaks for the
# whole measurement, so it must never read "some shards ran" as "it passed":
#
#   failure    a shard found a defect                         -> red
#   cancelled  a shard was killed by its own timeout-minutes  -> red (#977)
#   skipped    a shard never ran (its build or plan failed)    -> red
#
# The aggregator calls this under `if: ${{ !cancelled() }}`, so a run that was
# superseded by a newer push -- the one cancellation that is not a defect --
# skips the verdict instead of reaching it. Anything else that is not
# `success` is a missing measurement, and a missing measurement is a failure:
# that is stricter than scripts/check-pipeline-status.sh on purpose, because a
# pipeline status may legitimately skip a job, while a sharded check that
# skipped a shard measured less than it claims.
#
# INPUT
#   NEEDS_JSON  `toJSON(needs)`: a map of job-id -> {result, outputs}.
#   CHECK       Name of the sharded check, for the messages.
#   JOBS        Optional space-separated job ids to judge. A gate job such as
#               `detect-changes` is skipped by design on some events, so the
#               caller names the jobs that carry the measurement; a named job
#               missing from NEEDS_JSON is reported as `absent`, never ignored.
set -euo pipefail

: "${NEEDS_JSON:?NEEDS_JSON is required (pass toJSON(needs))}"
CHECK="${CHECK:-sharded check}"
JOBS="${JOBS:-}"

if [ -n "$JOBS" ]; then
  # shellcheck disable=SC2086 # JOBS is a word list by contract
  NEEDS_JSON="$(printf '%s' "$NEEDS_JSON" | jq -c --args \
    '. as $needs | $ARGS.positional
     | map({(.): ($needs[.] // {result: "absent"})}) | add // {}' $JOBS)"
fi

not_green="$(printf '%s' "$NEEDS_JSON" \
  | jq -r 'to_entries | map(select(.value.result != "success") | "\(.key)=\(.value.result)") | join(", ")')"
needed="$(printf '%s' "$NEEDS_JSON" | jq -r 'keys | join(", ")')"

if [ -z "$needed" ]; then
  echo "::error title=${CHECK}: nothing to judge::the aggregator needs no jobs, so it would pass without a measurement"
  exit 1
fi

if [ -n "$not_green" ]; then
  echo "::error title=${CHECK} is incomplete::not every shard succeeded (${not_green}). A failed, timed-out (cancelled) or skipped shard is a missing measurement, never a pass."
  exit 1
fi

echo "${CHECK}: every needed job succeeded (${needed})."
