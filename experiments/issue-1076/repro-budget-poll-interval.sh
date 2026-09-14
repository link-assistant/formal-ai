#!/usr/bin/env bash
# Reproduce: the rust pipeline template's scripts/run-with-budget-warning.sh
# never enforces its budget when BUDGET_POLL_SECONDS is not an integer, because
# it counts loop iterations instead of reading wall-clock time.
#
#   elapsed=$(( elapsed + POLL_SECONDS ))   # template
#   elapsed=$((SECONDS - started))          # what it should be
#
# Usage: ./repro-budget-poll-interval.sh [path-to-template-script]
set -uo pipefail

script="${1:-$(dirname "$0")/template-run-with-budget-warning.sh}"
budget=2
sleep_for=10

printf 'script under test: %s\n\n' "$script"

for poll in 1 0.5; do
  printf -- '--- BUDGET_POLL_SECONDS=%s, budget=%ss, command sleeps %ss ---\n' \
    "$poll" "$budget" "$sleep_for"
  start=$SECONDS
  BUDGET_POLL_SECONDS="$poll" "$script" "$budget" "probe" sleep "$sleep_for" \
    > /tmp/budget-probe.out 2>&1
  status=$?
  wall=$((SECONDS - start))
  sed 's/^/    /' /tmp/budget-probe.out
  printf '    => exit %s after %ss\n' "$status" "$wall"
  if [ "$status" -eq 124 ]; then
    printf '    => ENFORCED (expected)\n\n'
  else
    printf '    => NOT ENFORCED: the command ran to completion despite a %ss budget\n\n' "$budget"
  fi
done
