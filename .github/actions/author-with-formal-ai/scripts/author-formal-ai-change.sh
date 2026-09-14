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

# GitHub refuses a push that creates or updates `.github/workflows/*` from a
# token without the `workflows` scope, and GITHUB_TOKEN cannot be granted it.
# Only FORMAL_AI_BOT_TOKEN can land such a change, so a task that targets a
# workflow file is refused here, with the reason, before a whole authoring run
# is spent on a commit that cannot be pushed (issue #1118, option 2).
if [[ -z "${FORMAL_AI_BOT_TOKEN:-}" ]]; then
  while IFS= read -r destination; do
    if [[ "$destination" == .github/workflows/* ]]; then
      echo "::error::the task writes $destination, but the repository provides no FORMAL_AI_BOT_TOKEN; GITHUB_TOKEN cannot push a workflow file (no 'workflows' scope). Set the secret (a fine-grained token with contents, pull-requests and workflows write) or move the change out of .github/workflows/."
      exit 1
    fi
  done < into.txt
fi

AGENT=agent PORT="${PORT:-8931}" BIN="$PWD/target/release/formal-ai" \
  FORMAL_AI_REPO_ROOT="$PWD" \
  "$RUNNER_TEMP/author-change-with-formal-ai.sh" \
  --task "$TASK" --seed "$SEED" ${artifacts[@]+"${artifacts[@]}"} \
  --evidence "$EVIDENCE" --pull-request "$PULL_REQUEST" \
  --message "$MESSAGE (#$NUMBER)" ${contains[@]+"${contains[@]}"}
