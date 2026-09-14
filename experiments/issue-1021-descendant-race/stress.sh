#!/usr/bin/env bash
# Stress the descendant-termination test under CPU load, the condition under
# which the loaded macOS runner failed it (job 96137354605). See the README
# beside this script.
set -uo pipefail
bin="$1"
rounds="${2:-40}"
load="${3:-12}"
pids=()
for _ in $(seq 1 "$load"); do
  bash -c 'while :; do :; done' &
  pids+=("$!")
done
fail=0
for round in $(seq 1 "$rounds"); do
  if ! "$bin" issue_703_orchestration_followup::timeout_terminates_descendant_processes \
      --exact > "/tmp/desc-round-$round.log" 2>&1; then
    fail=$((fail + 1))
    echo "round $round FAILED"
    tail -6 "/tmp/desc-round-$round.log"
  fi
done
kill "${pids[@]}" 2>/dev/null
echo "failures: $fail / $rounds"
