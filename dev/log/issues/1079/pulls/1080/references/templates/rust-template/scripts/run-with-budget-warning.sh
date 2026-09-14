#!/usr/bin/env bash
# Run a command under an explicit execution budget that expires *before* the
# job's `timeout-minutes` backstop.
#
# Why this exists: GitHub reports a job killed by `timeout-minutes` as
# `cancelled`, not `failed`. On a non-default ref that is indistinguishable
# from a superseded run, so a genuine timeout produces no red anywhere. The
# fix is to make the step own the deadline: when the step's budget expires
# first, the step fails, the job fails, and the annotation names the budget and
# the overrun.
#
# Usage:
#   scripts/run-with-budget-warning.sh <budget_seconds> <label> <command> [args...]
#
# Environment:
#   BUDGET_WARN_PERCENT  - emit a ::warning at this share of the budget (default 70)
#   BUDGET_GRACE_SECONDS - seconds between SIGTERM and SIGKILL (default 10)
#   BUDGET_POLL_SECONDS  - poll interval (default 1)
#
# Exit codes:
#   124 - the command exceeded its budget (matches timeout(1)'s convention)
#     2 - usage error
#     * - the command's own exit status
set -uo pipefail

usage() {
  echo "usage: $0 <budget_seconds> <label> <command> [args...]" >&2
  exit 2
}

[ "$#" -ge 3 ] || usage

BUDGET="$1"
LABEL="$2"
shift 2

case "$BUDGET" in
  '' | *[!0-9]*) echo "budget must be a positive integer number of seconds, got: ${BUDGET}" >&2; usage ;;
esac
[ "$BUDGET" -gt 0 ] || usage

WARN_PERCENT="${BUDGET_WARN_PERCENT:-70}"
GRACE_SECONDS="${BUDGET_GRACE_SECONDS:-10}"
POLL_SECONDS="${BUDGET_POLL_SECONDS:-1}"
warn_at=$(( BUDGET * WARN_PERCENT / 100 ))

# `set -m` gives the background command its own process group. Without it,
# `timeout(1)` or a plain `kill $pid` reaches only the direct child, while
# `cargo test` / `cargo nextest` spawn a tree whose orphans keep the runner
# busy until the job cap fires -- exactly the failure this script prevents.
set -m
"$@" &
command_pid=$!
set +m

# Signal the whole process group when possible, and fall back to the direct
# child on platforms where the group is not addressable (Git Bash on Windows).
signal_tree() {
  kill "-$1" -- "-${command_pid}" 2>/dev/null || kill "-$1" "${command_pid}" 2>/dev/null || true
}

forward() {
  signal_tree TERM
  exit 143
}
trap forward TERM INT

elapsed=0
warned=false
timed_out=false
while kill -0 "${command_pid}" 2>/dev/null; do
  if [ "${elapsed}" -ge "${BUDGET}" ]; then
    timed_out=true
    break
  fi
  if [ "${warned}" = false ] && [ "${elapsed}" -ge "${warn_at}" ]; then
    warned=true
    echo "::warning title=${LABEL} is approaching its execution budget::${LABEL} has been running for ${elapsed}s of its ${BUDGET}s budget (${WARN_PERCENT}%). If it keeps growing, raise the budget and the job's timeout-minutes together, or split the work."
  fi
  sleep "${POLL_SECONDS}"
  elapsed=$(( elapsed + POLL_SECONDS ))
done

if [ "${timed_out}" = true ]; then
  signal_tree TERM
  waited=0
  while [ "${waited}" -lt "${GRACE_SECONDS}" ] && kill -0 "${command_pid}" 2>/dev/null; do
    sleep 1
    waited=$(( waited + 1 ))
  done
  signal_tree KILL
  wait "${command_pid}" 2>/dev/null || true
  echo "::error title=${LABEL} exceeded its execution budget::${LABEL} was terminated after ${BUDGET}s. This budget expires before the job's timeout-minutes backstop on purpose: a job killed by timeout-minutes is reported as 'cancelled' and hides the failure, while this reports 'failure'. Command: $*"
  exit 124
fi

status=0
wait "${command_pid}" || status=$?
exit "${status}"
