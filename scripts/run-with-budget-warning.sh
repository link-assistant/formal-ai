#!/usr/bin/env bash
# Run a command and emit a GitHub annotation while it can still be acted on.
#
# Usage: run-with-budget-warning.sh <budget-seconds> <label> <command> [args...]
set -euo pipefail

budget_seconds="${1:?budget seconds are required}"
label="${2:?a warning label is required}"
shift 2
[ "$#" -gt 0 ] || {
  echo "a command is required" >&2
  exit 2
}

warn_ratio="${TEST_WARN_RATIO_PERCENT:-70}"
threshold=$((budget_seconds * warn_ratio / 100))
[ "$threshold" -gt 0 ] || threshold=1
started=$SECONDS

(
  sleep "$threshold"
  echo "::warning title=${label} is approaching its timeout::The command is still running after ${threshold}s (${warn_ratio}% of its ${budget_seconds}s execution budget). Speed up or repartition it before the job reports a timeout as a cancellation." >&2
) &
watchdog_pid=$!

# shellcheck disable=SC2329  # invoked indirectly by the trap below
cleanup_watchdog() {
  kill "$watchdog_pid" 2>/dev/null || true
  wait "$watchdog_pid" 2>/dev/null || true
}
trap cleanup_watchdog EXIT INT TERM

set +e
"$@"
status=$?
set -e

elapsed=$((SECONDS - started))
printf '%s took %ss of its %ss execution budget.\n' "$label" "$elapsed" "$budget_seconds"
exit "$status"
