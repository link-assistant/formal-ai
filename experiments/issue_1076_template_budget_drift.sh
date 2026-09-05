#!/usr/bin/env bash
# Issue #1076 / upstream report: the templates' `scripts/run-with-budget-warning.sh`
# tracks elapsed time by *counting poll iterations* instead of reading the clock:
#
#   sleep "${POLL_SECONDS}"
#   elapsed=$(( elapsed + POLL_SECONDS ))
#
# Two consequences, both defeating the wrapper's entire purpose (make the step
# fail before `timeout-minutes` reports the job as `cancelled`):
#
#   A. Non-integer BUDGET_POLL_SECONDS makes `$(( ))` fail every iteration, so
#      `elapsed` stays 0 and the budget NEVER fires.
#   B. Even with an integer poll, the counter ignores the cost of the loop body
#      and `sleep`'s own overshoot, so the deadline drifts later than the wall
#      clock -- worst under the CPU contention that makes budgets matter.
#
# formal-ai's copy reads `$SECONDS` instead and has neither behaviour.
set -uo pipefail
TEMPLATE="${1:?path to the template copy of run-with-budget-warning.sh}"
FORMALAI="${2:-scripts/run-with-budget-warning.sh}"

echo "=== A. fractional poll interval: budget of 2s against a 10s command ==="
for impl in "$TEMPLATE" "$FORMALAI"; do
  echo "--- $impl"
  start=$SECONDS
  BUDGET_POLL_SECONDS=0.5 TEST_BUDGET_POLL_SECONDS=0.5 \
    bash "$impl" 2 demo sleep 10 > /tmp/budget-out.txt 2>&1
  status=$?
  echo "exit=${status} wall=$((SECONDS - start))s (expected: exit 124 at ~2s)"
  grep -c 'syntax error' /tmp/budget-out.txt | sed 's/^/  arithmetic errors: /'
done

echo
echo "=== B. integer poll, loop-body drift: budget 5s, command 60s ==="
for impl in "$TEMPLATE" "$FORMALAI"; do
  echo "--- $impl"
  start=$SECONDS
  bash "$impl" 5 demo sleep 60 > /dev/null 2>&1
  echo "exit=$? wall=$((SECONDS - start))s (expected: exit 124 at ~5s)"
done

# Recorded result (2026-09-05, 6-core box):
#   A. REPRODUCED. Template copy: exit 0 after the full 10s command, one
#      `syntax error: invalid arithmetic operator` -- the budget never fired.
#      formal-ai copy: exit 124 at 2s.
#   B. NOT REPRODUCED. Under 5x CPU oversubscription (32 spinners on 6 cores) a
#      20s budget fired at 21.85s (template) vs 21.95s (formal-ai): the counter
#      drift is dominated by SIGTERM delivery, not by the loop body. The
#      iteration-counting loop is still structurally wrong, but there is no
#      measured impact for integer poll intervals; only defect A is reported.
