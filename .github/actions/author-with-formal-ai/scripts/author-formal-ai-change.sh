#!/usr/bin/env bash
# Drive the Agent CLI against a local `formal-ai serve` for one task.
#
# Issue #1085 (D2.3). The heavy lifting is in
# `author-change-with-formal-ai.sh`, which ships beside this script; this one
# only turns the issue's contract into its arguments.
set -euo pipefail

contains=()
while IFS= read -r expected; do
  [[ -n "$expected" ]] && contains+=(--contains "$expected")
done < contains.txt

artifacts=()
while IFS= read -r produced; do
  [[ -n "$produced" ]] && artifacts+=(--produces "$produced")
done < produces.txt
while IFS= read -r destination; do
  [[ -n "$destination" ]] && artifacts+=(--into "$destination")
done < into.txt

AGENT=agent PORT="${PORT:-8931}" BIN="$PWD/target/release/formal-ai" \
  FORMAL_AI_REPO_ROOT="$PWD" \
  "$RUNNER_TEMP/author-change-with-formal-ai.sh" \
  --task "$TASK" --seed "$SEED" ${artifacts[@]+"${artifacts[@]}"} \
  --evidence "$EVIDENCE" --pull-request "$PULL_REQUEST" \
  --message "$MESSAGE (#$NUMBER)" ${contains[@]+"${contains[@]}"}
