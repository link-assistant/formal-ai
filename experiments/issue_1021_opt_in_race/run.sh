#!/usr/bin/env bash
# Issue #1021: reproduce the opt-in race that failed
# `issue_1021_write_path::publishing_a_contribution_is_planned_only_under_the_opt_in`
# in CI (run 32260417488, job 96093284006) while passing on every local run.
#
# The opt-in lives in a process-wide environment variable and the test harness
# runs tests as threads, so a test reading the variable without holding the
# suite's lock can see the value another test set for its own opted-in case.
# One green pass is not evidence -- the window is microseconds wide -- so this
# runs the file's tests many times and reports how many rounds failed.
#
# Usage: experiments/issue_1021_opt_in_race/run.sh [rounds]
set -uo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
ROUNDS="${1:-200}"
LOG="${LOG:-/tmp/issue_1021_opt_in_race.log}"

cd "$ROOT" || exit 1
: > "$LOG"

# Run the compiled binary rather than `cargo test` per round: the race is in the
# harness's threads, and cargo's own overhead would dominate the measurement.
binary="$(cargo test --no-run --test unit --message-format=json 2>>"$LOG" \
  | grep -o '"executable":"[^"]*unit-[^"]*"' | tail -1 | cut -d'"' -f4)"
if [ ! -x "$binary" ]; then
  echo "!! could not locate the unit test binary; see $LOG" >&2
  exit 1
fi

failed=0
for ((round = 1; round <= ROUNDS; round++)); do
  if ! "$binary" issue_1021_write_path >> "$LOG" 2>&1; then
    failed=$((failed + 1))
  fi
done
echo "rounds=$ROUNDS failed=$failed (log: $LOG)"
[ "$failed" -eq 0 ]
