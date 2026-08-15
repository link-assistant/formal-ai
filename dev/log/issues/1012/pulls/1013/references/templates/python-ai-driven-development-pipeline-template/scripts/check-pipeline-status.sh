#!/usr/bin/env bash
# Make failures and timeout-shaped cancellations visible in the aggregate run.
set -euo pipefail

: "${NEEDS_JSON:?NEEDS_JSON is required (pass toJSON(needs))}"
IS_MAIN="${IS_MAIN:-false}"

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
    echo "::error::Pipeline has cancelled jobs on main: ${cancelled}. A job killed by 'timeout-minutes' is reported as cancelled; check job annotations for an exceeded maximum execution time."
    status=1
  else
    echo "::warning::Cancelled jobs: ${cancelled}. On a non-default ref this is usually a superseded run."
  fi
fi

if [ "$status" -eq 0 ]; then
  echo "All required jobs succeeded or were legitimately skipped."
fi

exit "$status"
