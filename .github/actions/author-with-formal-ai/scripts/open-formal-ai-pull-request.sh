#!/usr/bin/env bash
# Open the draft pull request Formal AI will author on, or resume the open one.
#
# Issue #1085 (D2.3). The pull request is opened first so the authored commit
# can name it in its `Formal-AI-Pull-Request` trailer, and it targets the branch
# the run checked out rather than the repository's default branch: a draft for a
# task belonging to a pull request has to land in that pull request.
#
# The changelog fragment a source change needs is written here, in the bootstrap
# commit, not by Formal AI: `changelog.d/` is one of the trees the version-3
# self-hosting metric excludes from both sides of the share, so it is process
# record rather than authored behaviour.
set -euo pipefail

git config user.name 'github-actions[bot]'
git config user.email '41898282+github-actions[bot]@users.noreply.github.com'
evidence="${EVIDENCE_ROOT:-dev/log/self-authored}/issue-$NUMBER"

# A previous run may already have opened the pull request for this issue; keep
# authoring on its branch instead of opening another.
existing=$(gh pr list --repo "$GITHUB_REPOSITORY" --state open --json headRefName,url \
  --jq ".[] | select(.headRefName | startswith(\"formal-ai/issue-$NUMBER-\")) | .url" | head -1)
if [[ -n "$existing" ]]; then
  branch=$(gh pr view "$existing" --repo "$GITHUB_REPOSITORY" --json headRefName --jq .headRefName)
  git fetch origin "+refs/heads/$branch:refs/remotes/origin/$branch"
  git checkout -B "$branch" "origin/$branch"
  {
    echo "url=$existing"
    echo "branch=$branch"
    echo "evidence=$evidence/evidence"
  } >> "$GITHUB_OUTPUT"
  # Once a commit on the pull request names it in its trailer the task is
  # authored; a re-run must not author it again.
  authored=$("$RUNNER_TEMP/self-authored-commit-count.sh" "$existing")
  if [[ "$authored" != 0 ]]; then
    echo "::notice::$existing already carries the authored commit; waiting for review"
    echo "done=true" >> "$GITHUB_OUTPUT"
  fi
  exit 0
fi

branch="formal-ai/issue-$NUMBER-${GITHUB_RUN_ID}"
git checkout -b "$branch"
mkdir -p "$evidence"
{
  echo "# Self-authored change for issue #$NUMBER"
  echo
  echo "Run: $GITHUB_SERVER_URL/$GITHUB_REPOSITORY/actions/runs/$GITHUB_RUN_ID"
  echo
  echo 'Task:'
  echo
  cat task-body.txt
} > "$evidence/README.md"
git add "$evidence/README.md"

# Only where the repository collects fragments; a repository without
# `changelog.d/` has no such gate to satisfy.
if [[ -d changelog.d ]]; then
  fragment="changelog.d/$(date -u +%Y%m%d_%H%M%S)_issue_${NUMBER}_self_authored.md"
  {
    echo '---'
    echo 'bump: patch'
    echo '---'
    echo
    echo '### Fixed'
    echo "- Issue #$NUMBER: $MESSAGE. Authored by Formal AI through the Agent CLI."
  } > "$fragment"
  git add "$fragment"
fi

git commit -q -m "chore(self-authored): open the pull request Formal AI will author for #$NUMBER"
git remote set-url origin "https://x-access-token:${GH_TOKEN}@github.com/${GITHUB_REPOSITORY}.git"
"$RUNNER_TEMP/push-to-shared-branch.sh" origin "$branch"
url=$(gh pr create --repo "$GITHUB_REPOSITORY" --draft --base "$BASE_BRANCH" --head "$branch" \
  --title "$TITLE (authored by Formal AI, #$NUMBER)" \
  --body "$(printf 'Formal AI authors this change through the Agent CLI in [this run](%s/%s/actions/runs/%s); the pull request was opened first so the authored commit can name it.\n\nCloses #%s.\n\n- [ ] Formal AI authored the change (commit with Formal-AI-Session, Formal-AI-Model, Formal-AI-Evidence and Formal-AI-Pull-Request trailers)\n- [ ] CI is green with no human commit on the branch' \
    "$GITHUB_SERVER_URL" "$GITHUB_REPOSITORY" "$GITHUB_RUN_ID" "$NUMBER")")
{
  echo "url=$url"
  echo "branch=$branch"
  echo "evidence=$evidence/evidence"
} >> "$GITHUB_OUTPUT"
