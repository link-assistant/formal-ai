#!/usr/bin/env bash
set -eu
test "$(git branch --show-current)" = issue-1582-79aa864de807
test -z "$(git status --porcelain)"
git merge-base --is-ancestor origin/main HEAD
gh run list --repo link-assistant/formal-ai --branch issue-1582-79aa864de807 --status in_progress --json databaseId,workflowName,headSha > /tmp/issue-1582-push-in-progress.json
gh run list --repo link-assistant/formal-ai --branch issue-1582-79aa864de807 --status queued --json databaseId,workflowName,headSha > /tmp/issue-1582-push-queued.json
node --input-type=module -e 'import fs from "node:fs";for(const name of ["in-progress","queued"]){const runs=JSON.parse(fs.readFileSync("/tmp/issue-1582-push-"+name+".json"));if(runs.length)throw Error("branch has active runs: "+JSON.stringify(runs));}'
git push origin HEAD:issue-1582-79aa864de807
gh pr edit 1583 --repo link-assistant/formal-ai --title "Fix client read access, diagnostic output and context metadata" --body-file experiments/issue_1582/pull-request.md
gh pr diff 1583 --repo link-assistant/formal-ai > /tmp/issue-1582-pr-final.diff
gh pr view 1583 --repo link-assistant/formal-ai --json url,title,isDraft,headRefOid,mergeStateStatus
