#!/usr/bin/env bash
# Run a command under a deadline and exit 124 when it expires -- `timeout(1)`
# for a repository whose tests run where `timeout(1)` does not exist.
#
# Issue #1021: `scripts/apt-install-with-retry.sh` bounded each apt attempt with
# GNU `timeout`. Its Linux job passed; the macOS core slices that drive the same
# wrapper failed with `timeout: command not found` (run 32282461075, jobs
# 96170638546 and 96170638704), because macOS ships no `timeout` -- coreutils
# installs it as `gtimeout`, and neither is guaranteed on a runner. Branching on
# whichever binary happens to be present would leave the tested path and the
# shipped path different on the two runner families, which is how the gap got in.
# One deadline, implemented here, runs on both.
#
# The command is started in its own process group, so a stalled tree is
# signalled whole rather than at its root: the `apt-get` this was written for
# spends the stall inside a child, and killing only the parent leaves the child
# holding the lock. `scripts/run-with-budget-warning.sh` bounds a whole CI step
# the same way; this bounds one attempt inside it and stays silent, because an
# attempt that is retried is not a job annotation.
#
# Usage: run-with-deadline.sh <seconds> <command> [args...]
#
# Environment:
#   FORMAL_AI_DEADLINE_GRACE_SECONDS  SIGTERM -> SIGKILL grace period (10)
#   FORMAL_AI_DEADLINE_POLL_SECONDS   liveness polling interval; defaults to the
#                                     finest this `sleep` accepts
set -euo pipefail

deadline_seconds="${1:?deadline seconds are required}"
shift
[ "$#" -gt 0 ] || {
  echo "a command is required" >&2
  exit 2
}
[[ "$deadline_seconds" =~ ^[0-9]+$ ]] || {
  echo "the deadline must be a non-negative number of seconds, got ${deadline_seconds}" >&2
  exit 2
}

grace_seconds="${FORMAL_AI_DEADLINE_GRACE_SECONDS:-10}"

# The poll interval is the accuracy of the deadline: the command is killed at
# the first check past it, and again at the first check past the grace period.
# A one-second poll measured 4.2s on a 3s deadline
# (`experiments/issue-1021-deadline-precision/measure.sh`) -- a whole extra
# second, because a command signalled at the deadline is still alive at the next
# check and costs another full interval. Sub-second sleeps are not in POSIX, so
# ask this `sleep` rather than assume: GNU and BSD both accept them, and a
# `sleep` that does not just makes the deadline coarser, never wrong.
poll_seconds="${FORMAL_AI_DEADLINE_POLL_SECONDS:-}"
if [ -z "$poll_seconds" ]; then
  if sleep 0.1 2> /dev/null; then poll_seconds=0.1; else poll_seconds=1; fi
fi
[[ "$poll_seconds" =~ ^[0-9]+(\.[0-9])?$ ]] && [ "${poll_seconds%.0}" != "0" ] || {
  echo "the poll interval must be a positive number of seconds with at most one \
decimal, got ${poll_seconds}" >&2
  exit 2
}

# Elapsed time is whichever of two lower bounds is larger, because neither is
# good enough alone and a deadline must never expire early:
#
#   * counting poll intervals is exact at the short end -- `sleep` waits at
#     least what it is asked -- but each iteration also pays for a fork, which
#     measured ~20% over on a 0.1s poll here. On a 90s attempt that overshoot
#     is a minute, enough to break the budget guard in
#     `scripts/apt-install-with-retry.sh` that this deadline exists to serve.
#   * `SECONDS` does not drift, but it is a difference of whole-second clock
#     readings, so it can read a second high: it reached 3 just 2.6s into a 3s
#     deadline once the polling got fast enough to expose it
#     (`experiments/issue-1021-deadline-precision/measure.sh`). Subtracting that
#     second turns it back into a bound that is never early.
#
# Their maximum is never early and never more than a fork's-worth or two
# seconds late, whichever is smaller.
tenths() {
  local seconds="${1%.*}" decimal="0"
  case "$1" in *.*) decimal="${1#*.}" ;; esac
  echo $((10#$seconds * 10 + 10#$decimal))
}
poll_tenths=$(tenths "$poll_seconds")
deadline_tenths=$((deadline_seconds * 10))
grace_tenths=$((grace_seconds * 10))

started=$SECONDS

# Job control gives the command its own process group, so the whole tree can be
# signalled at the deadline instead of only its root.
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

# Sets `elapsed_tenths` to the tenths of a second since `$1` (a `SECONDS`
# reading) given `$2` tenths of counted polling since then. A function that
# assigns rather than echoes, and `kill -0` rather than `ps`, keep the whole
# poll to one fork -- the `sleep` itself. Every other fork is drift the counted
# bound pays for.
measure_elapsed() {
  elapsed_tenths=$(((SECONDS - $1 - 1) * 10))
  [ "$elapsed_tenths" -gt "$2" ] || elapsed_tenths="$2"
}

expired=false
counted_tenths=0
elapsed_tenths=0
while kill -0 "$command_pid" 2> /dev/null; do
  measure_elapsed "$started" "$counted_tenths"
  if [ "$elapsed_tenths" -ge "$deadline_tenths" ]; then
    expired=true
    signal_command_tree TERM
    grace_started=$SECONDS
    counted_tenths=0
    measure_elapsed "$grace_started" "$counted_tenths"
    while kill -0 "$command_pid" 2> /dev/null && [ "$elapsed_tenths" -lt "$grace_tenths" ]; do
      sleep "$poll_seconds"
      counted_tenths=$((counted_tenths + poll_tenths))
      measure_elapsed "$grace_started" "$counted_tenths"
    done
    signal_command_tree KILL
    break
  fi
  sleep "$poll_seconds"
  counted_tenths=$((counted_tenths + poll_tenths))
done

set +e
wait "$command_pid"
status=$?
set -e
trap - INT TERM

# 124 is `timeout`'s own status for a command it killed, kept so callers can
# tell a deadline from the command's own failure without knowing which
# implementation ran.
[ "$expired" = false ] || exit 124
exit "$status"
