#!/usr/bin/env bash
# Resolve the issue this run authors, and read its task contract.
#
# Issue #1085 (D2.3). An issue carries the contract as plain lines in its body:
#
#   task: <the prompt the Agent CLI gives Formal AI>
#   seed: <repository directory copied into the scratch workspace>
#   produces: <path, inside the workspace, the agent must write>
#   into: <repository path the artifact is copied to>
#   contains: <text one of the artifacts must contain>   (repeatable)
#   message: <conventional commit subject>
#
# `produces:` and `into:` are read in order and paired, so one authoring run can
# write more than one artifact.
#
# With REQUIRE_CONTRACT=false an issue without those lines is not an error: the
# task is derived from the title and body, and the run opens a draft attempt
# anyway. That is the mode a repository uses when it attempts *every* new issue
# -- a draft that fails is the evidence it wanted, not a failure of the action.
set -euo pipefail

number="${INPUT_ISSUE:-}"
if [[ -z "$number" && "${EVENT_NAME:-}" == "issues" ]]; then
  number="${EVENT_ISSUE:-}"
fi
if [[ -z "$number" ]]; then
  number=$(gh issue list --repo "$GITHUB_REPOSITORY" --label "${LABEL:-formal-ai-solve}" \
    --state open --search 'sort:created-asc' --json number --jq '.[0].number // empty')
fi
if [[ -z "$number" ]]; then
  echo "::notice::no open issue labelled ${LABEL:-formal-ai-solve}; nothing to author"
  echo "number=" >> "$GITHUB_OUTPUT"
  exit 0
fi

# A task whose pull request already merged is done, even though its issue is
# still open: GitHub closes a linked issue only when the pull request merges
# into the DEFAULT branch, and a task belonging to a working branch merges into
# that branch instead. Run 34294396281 re-attempted #1091 four minutes after
# #1103 landed it, opening a duplicate pull request for work already in the
# tree (issue #1085).
merged=$(gh pr list --repo "$GITHUB_REPOSITORY" --state merged --limit 20 \
  --json number,headRefName \
  --jq "[.[] | select(.headRefName | startswith(\"formal-ai/issue-$number-\"))] | length")
if [[ "$merged" != 0 ]]; then
  echo "::notice::issue #$number already has a merged self-authored pull request; nothing to author"
  echo "number=" >> "$GITHUB_OUTPUT"
  exit 0
fi

gh issue view "$number" --repo "$GITHUB_REPOSITORY" --json body --jq .body > task-body.txt
title=$(gh issue view "$number" --repo "$GITHUB_REPOSITORY" --json title --jq .title)
field() { sed -n "s/^$1: *//p" task-body.txt | head -1; }

sed -n 's/^contains: *//p' task-body.txt > contains.txt
sed -n 's/^produces: *//p' task-body.txt > produces.txt
sed -n 's/^into: *//p' task-body.txt > into.txt

if [[ -n "$(field task)" ]]; then
  for required in task seed produces into message; do
    [[ -n "$(field "$required")" ]] || {
      echo "::error::issue #$number lacks the '$required:' line"
      exit 1
    }
  done
  [[ "$(wc -l < produces.txt)" == "$(wc -l < into.txt)" ]] || {
    echo "::error::issue #$number pairs $(wc -l < produces.txt) 'produces:' lines with $(wc -l < into.txt) 'into:' lines"
    exit 1
  }
  task=$(field task)
  seed=$(field seed)
  message=$(field message)
elif [[ "${REQUIRE_CONTRACT:-true}" == "true" ]]; then
  echo "::error::issue #$number carries no 'task:' line; label it only when it does, or set require-contract: false"
  exit 1
else
  # No contract: attempt the issue as written. The draft is the experiment.
  echo "::notice::issue #$number carries no task contract; attempting it as written"
  task="$title

$(cat task-body.txt)"
  seed="."
  message="fix: $title"
  : > produces.txt
  : > into.txt
fi

{
  echo "number=$number"
  echo "title=$title"
  echo "message=$message"
  echo "seed=$seed"
  echo 'task<<FORMAL_AI_TASK_EOF'
  printf '%s\n' "$task"
  echo 'FORMAL_AI_TASK_EOF'
} >> "$GITHUB_OUTPUT"
