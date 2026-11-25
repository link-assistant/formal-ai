#!/usr/bin/env bash
# Issue #1637: replay the pull request admission for one real push.
# Builds the synthetic merge commit GitHub would check out, then runs the
# detect mode with the live Actions API (needs `gh auth login`).
#   experiments/issue-1637-replay-push-range.sh <before-sha> <after-sha> [base-ref]
# Example: experiments/issue-1637-replay-push-range.sh b2c928ae8 f6afdaed5
set -euo pipefail
repo=$(git rev-parse --show-toplevel)
before=$(git rev-parse "$1"); after=$(git rev-parse "$2")
base=$(git merge-base "$after" "${3:-origin/main}")
work=$(mktemp -d)
trap 'git -C "$repo" worktree remove --force "$work/tree" >/dev/null 2>&1 || true; rm -rf "$work"' EXIT
git -C "$repo" worktree add -q --detach "$work/tree" "$base"
git -C "$work/tree" -c user.name=replay -c user.email=replay@example.invalid merge -q --no-ff --no-edit "$after" >/dev/null
token=$(gh auth token)
for workflow in release.yml layered-ci.yml coverage.yml; do
  echo "== $workflow"
  (cd "$work/tree" && GITHUB_EVENT_NAME=pull_request GITHUB_EVENT_ACTION=synchronize \
    GITHUB_EVENT_BEFORE="$before" GITHUB_EVENT_AFTER="$after" GITHUB_REPOSITORY=link-assistant/formal-ai \
    GITHUB_WORKFLOW_REF="link-assistant/formal-ai/.github/workflows/$workflow@refs/pull/0/merge" \
    GITHUB_TOKEN="$token" node "$repo/scripts/ci-pull-request-admission.mjs" detect 2>&1)
done
