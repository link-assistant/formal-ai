#!/usr/bin/env bash
# Supplied observation recipe, zero autonomous authorship credit.
set -eu
git branch --show-current
git status --short
git log -1 --format='%H %cI %s' origin/main
git log --oneline HEAD..origin/main | head -18
git diff --stat origin/main...HEAD > /tmp/issue-1582-review/pr-stat.txt
git diff --name-only origin/main...HEAD > /tmp/issue-1582-review/pr-files.txt
cat /tmp/issue-1582-review/pr-files.txt
gh run list --repo link-assistant/formal-ai --branch issue-1582-79aa864de807 --limit 5 --json databaseId,workflowName,status,conclusion,createdAt,headSha
gh pr list --repo link-assistant/formal-ai --state merged --limit 5 --search 'client context' --json number,title,mergedAt
rg -n 'gemini|context_window|contextWindow|contextSize|tokenLimit|readNetworkAllowed' js/client* js/server rust/src/client_integrations data/seed/client* ts/server 2>/dev/null | head -150
git merge --no-commit origin/main
