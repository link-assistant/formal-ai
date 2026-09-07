#!/usr/bin/env bash
set -euo pipefail

: "${NEEDS_JSON:?NEEDS_JSON is required (pass toJSON(needs))}"
IS_MAIN="${IS_MAIN:-false}"

select_by_result() {
  printf '%s' "$NEEDS_JSON" \
    | python3 -c 'import json, sys
want = sys.argv[1]
needs = json.load(sys.stdin)
print(", ".join(name for name, job in needs.items() if job.get("result") == want))' "$1"
}

failed="$(select_by_result failure)"
cancelled="$(select_by_result cancelled)"

echo "Failed jobs:    ${failed:-<none>}"
echo "Cancelled jobs: ${cancelled:-<none>}"

status=0
if [[ -n "$failed" ]]; then
  echo "::error::Pipeline failed. Failing jobs: ${failed}"
  status=1
fi

if [[ -n "$cancelled" ]]; then
  if [[ "$IS_MAIN" == "true" ]]; then
    echo "::error::Pipeline has cancelled jobs on main: ${cancelled}. A job killed by 'timeout-minutes' is reported as cancelled, which would otherwise hide the failure."
    status=1
  else
    echo "::warning::Cancelled jobs: ${cancelled}. On a non-default ref this is usually a superseded run — open each job and search for 'has exceeded the maximum execution time' to rule out a real timeout. Steps wrapped in scripts/run-with-budget-warning.sh report a real timeout as a failure instead, which is why long steps carry their own budget."
  fi
fi

exit "$status"
