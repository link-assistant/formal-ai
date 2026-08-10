#!/usr/bin/env bash
# Exercise scripts/check-pipeline-status.sh against the states that matter for
# issue #977. Run from the repository root:
#
#   bash experiments/issue-977-pipeline-status.sh
#
# The case that motivated the whole issue is the second one: a job killed by
# `timeout-minutes` is reported as `cancelled`, and on `main` -- where
# concurrency cancellation is disabled -- that can only mean a timeout or a
# manual cancel. Both must be red. Off `main`, a cancellation is an ordinary
# superseded run and must stay a warning.
set -euo pipefail

run_case() {
  local label="$1" needs="$2" is_main="$3" expected="$4"
  local out status=0
  out="$(GITHUB_OUTPUT=/dev/null NEEDS_JSON="$needs" IS_MAIN="$is_main" \
    bash scripts/check-pipeline-status.sh 2>&1)" || status=$?

  printf '%-44s exit=%s (expected %s)\n' "$label" "$status" "$expected"
  printf '%s\n\n' "$out" | sed 's/^/    /'

  if [ "$status" -ne "$expected" ]; then
    echo "FAILED: $label" >&2
    exit 1
  fi
}

all_ok='{"lint":{"result":"success"},"test":{"result":"skipped"}}'
timed_out='{"lint":{"result":"success"},"auto-release":{"result":"cancelled"}}'
failed='{"lint":{"result":"failure"}}'

run_case "success + skipped"                "$all_ok"    true  0
run_case "cancelled on main (a timeout)"    "$timed_out" true  1
run_case "cancelled off main (a supersede)" "$timed_out" false 0
run_case "outright failure"                 "$failed"    false 1

echo "All pipeline-status cases behaved as expected."
