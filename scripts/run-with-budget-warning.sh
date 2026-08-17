#!/usr/bin/env bash
# Run a command under an execution budget that expires before the job clock.
#
# GitHub reports a job killed by `timeout-minutes` as **cancelled**, not
# **failed** (issue #977), so a step that is allowed to run until the runner
# kills it converts a real, actionable timeout into a benign-looking
# cancellation. Issue #1017 is that exact regression: `macOS core slice 10/12`
# spent 133s on checkout/toolchain/artifact setup and was killed 467s into a
# test step whose 480s budget could therefore never expire first, so the whole
# `CI/CD Pipeline` run reported `cancelled` and no release was published.
#
# This wrapper owns the deadline instead of the runner: it warns while the
# command can still be acted on and -- unless enforcement is explicitly
# disabled -- terminates the command at the budget and exits 124, so the job
# reports `failure` with an ::error annotation naming the budget it blew.
#
# Usage: run-with-budget-warning.sh <budget-seconds> <label> <command> [args...]
#
# Environment:
#   TEST_WARN_RATIO_PERCENT    warn once this share of the budget is spent (70)
#   TEST_BUDGET_ENFORCE        `false` warns only and never terminates (true)
#   TEST_BUDGET_GRACE_SECONDS  SIGTERM -> SIGKILL grace period (15)
#   TEST_BUDGET_POLL_SECONDS   deadline polling interval (1)
#   FORMAL_AI_CI_VERBOSE       `true` prints periodic progress lines (false)
set -euo pipefail

budget_seconds="${1:?budget seconds are required}"
label="${2:?a warning label is required}"
shift 2
[ "$#" -gt 0 ] || {
  echo "a command is required" >&2
  exit 2
}

warn_ratio="${TEST_WARN_RATIO_PERCENT:-70}"
enforce="${TEST_BUDGET_ENFORCE:-true}"
grace_seconds="${TEST_BUDGET_GRACE_SECONDS:-15}"
poll_seconds="${TEST_BUDGET_POLL_SECONDS:-1}"
verbose="${FORMAL_AI_CI_VERBOSE:-false}"
heartbeat_seconds="${TEST_BUDGET_HEARTBEAT_SECONDS:-60}"

threshold=$((budget_seconds * warn_ratio / 100))
[ "$threshold" -gt 0 ] || threshold=1
started=$SECONDS

# Job control gives the command its own process group, so the whole process
# tree can be signalled at the deadline instead of only its root.
set -m
"$@" &
command_pid=$!
set +m

signal_command_tree() {
  local signal="$1"
  kill "-$signal" "-$command_pid" 2> /dev/null ||
    kill "-$signal" "$command_pid" 2> /dev/null ||
    true
}

# shellcheck disable=SC2329  # invoked indirectly by the trap below
forward_cancellation() {
  signal_command_tree TERM
}
trap forward_cancellation INT TERM

warned=false
terminated=false
last_heartbeat=0

while kill -0 "$command_pid" 2> /dev/null; do
  elapsed=$((SECONDS - started))

  if [ "$warned" = false ] && [ "$elapsed" -ge "$threshold" ]; then
    warned=true
    echo "::warning title=${label} is approaching its timeout::The command is still running after ${elapsed}s (${warn_ratio}% of its ${budget_seconds}s execution budget). Speed up or repartition it before the budget terminates the step." >&2
  fi

  if [ "$verbose" = true ] &&
    [ $((elapsed - last_heartbeat)) -ge "$heartbeat_seconds" ]; then
    last_heartbeat="$elapsed"
    echo "[budget] ${label} has been running for ${elapsed}s of its ${budget_seconds}s budget." >&2
  fi

  if [ "$enforce" = true ] && [ "$elapsed" -ge "$budget_seconds" ]; then
    terminated=true
    echo "::error title=${label} exceeded its execution budget::The command was terminated after ${elapsed}s, its full ${budget_seconds}s execution budget. The budget expires before \`timeout-minutes\` so this reports as a failure instead of a cancelled job (issue #977, issue #1017). Speed up or repartition the work, or raise the budget together with the job timeout." >&2
    signal_command_tree TERM
    grace_deadline=$((SECONDS + grace_seconds))
    while kill -0 "$command_pid" 2> /dev/null && [ "$SECONDS" -lt "$grace_deadline" ]; do
      sleep "$poll_seconds"
    done
    signal_command_tree KILL
    break
  fi

  sleep "$poll_seconds"
done

set +e
wait "$command_pid"
status=$?
set -e
trap - INT TERM

elapsed=$((SECONDS - started))
if [ "$terminated" = true ]; then
  printf '%s was terminated after %ss, its full %ss execution budget.\n' \
    "$label" "$elapsed" "$budget_seconds"
  exit 124
fi

printf '%s took %ss of its %ss execution budget.\n' "$label" "$elapsed" "$budget_seconds"
exit "$status"
