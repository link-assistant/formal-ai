#!/usr/bin/env bash
set -eu
git branch --show-current
git fetch origin main issue-1582-79aa864de807
git log -1 --format='%H %cI %s' origin/main
git merge-base --is-ancestor origin/main HEAD
gh pr view 1583 --repo link-assistant/formal-ai --json title,body,isDraft,headRefOid,mergeStateStatus,reviewDecision > /tmp/issue-1582-pr-current.json
gh run list --repo link-assistant/formal-ai --branch issue-1582-79aa864de807 --limit 5 --json databaseId,conclusion,createdAt,headSha > /tmp/issue-1582-runs-current.json
cat /tmp/issue-1582-runs-current.json
gh api repos/link-assistant/formal-ai/issues/1582/comments --paginate > /tmp/issue-1582-issue-comments-current.json
gh api repos/link-assistant/formal-ai/issues/1583/comments --paginate > /tmp/issue-1582-pr-comments-current.json
gh api repos/link-assistant/formal-ai/pulls/1583/comments --paginate > /tmp/issue-1582-inline-current.json
gh api repos/link-assistant/formal-ai/pulls/1583/reviews --paginate > /tmp/issue-1582-reviews-current.json
cat /tmp/issue-1582-inline-current.json /tmp/issue-1582-reviews-current.json
cat /sys/fs/cgroup/memory.current
cat /sys/fs/cgroup/memory.max
sed -n '1,100p' .github/actions/dispatch-checks/action.yml
