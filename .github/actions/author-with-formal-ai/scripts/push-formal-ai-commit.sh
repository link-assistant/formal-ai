#!/usr/bin/env bash
# Push the authored commit and report it on the pull request and the issue.
set -euo pipefail

# A run that started before the previous one pushed would author the task
# twice; check once more against the pull request before pushing.
authored=$("$RUNNER_TEMP/self-authored-commit-count.sh" "$PULL_REQUEST")
if [[ "$authored" != 0 ]]; then
  echo "::notice::$PULL_REQUEST received its authored commit while this run was authoring; this run's commit is discarded"
  echo "authored=false" >> "$GITHUB_OUTPUT"
  exit 0
fi

git remote set-url origin "https://x-access-token:${GH_TOKEN}@github.com/${GITHUB_REPOSITORY}.git"
"$RUNNER_TEMP/push-to-shared-branch.sh" origin "$BRANCH"
git --no-pager log -1 --format='%H %s%n%b' > authored-commit.txt
fence=$(printf '\x60\x60\x60')
gh pr comment "$PULL_REQUEST" --repo "$GITHUB_REPOSITORY" \
  --body "$(printf 'Formal AI authored this commit in run %s/%s/actions/runs/%s:\n\n%s\n%s\n%s' \
    "$GITHUB_SERVER_URL" "$GITHUB_REPOSITORY" "$GITHUB_RUN_ID" "$fence" "$(cat authored-commit.txt)" "$fence")"
gh issue comment "$NUMBER" --repo "$GITHUB_REPOSITORY" --body "Formal AI opened $PULL_REQUEST for this task."
echo "authored=true" >> "$GITHUB_OUTPUT"
if [[ -z "${FORMAL_AI_BOT_TOKEN:-}" ]]; then
  echo "::notice::pushed with GITHUB_TOKEN, which starts no workflow run; a maintainer must close and reopen $PULL_REQUEST to get checks, or set the FORMAL_AI_BOT_TOKEN secret"
fi
