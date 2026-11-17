#!/usr/bin/env bash
set -eu
branch=issue-1582-79aa864de807
repo=link-assistant/formal-ai
test "$(git branch --show-current)" = "$branch"
test -z "$(git status --porcelain=v1)"
git fetch origin main "$branch" > /tmp/issue-1582-review/fetch-push.log 2>&1
git merge-base --is-ancestor origin/main HEAD
gh run list --repo "$repo" --branch "$branch" --status in_progress --json databaseId,workflowName,headSha > /tmp/issue-1582-review/in-progress-push.json
gh run list --repo "$repo" --branch "$branch" --status queued --json databaseId,workflowName,headSha > /tmp/issue-1582-review/queued-push.json
gh run list --repo "$repo" --branch "$branch" --limit 100 --json databaseId,workflowName,headSha,status,conclusion,createdAt > /tmp/issue-1582-review/runs-push.json
python3 - <<'PY'
import json,subprocess
from pathlib import Path
p=Path('/tmp/issue-1582-review')
for name in ['in-progress-push.json','queued-push.json']:
 assert not json.loads((p/name).read_text()),name
sha=subprocess.check_output(['git','rev-parse','origin/issue-1582-79aa864de807'],text=True).strip()
runs=[r for r in json.loads((p/'runs-push.json').read_text()) if r['headSha']==sha]
assert len(runs)==22, ('unexpected workflow count',len(runs))
assert all(r['status']=='completed' and r['conclusion']=='success' for r in runs),runs
print(json.dumps({'previousHead':sha,'completedSuccessfulRuns':len(runs),'main':subprocess.check_output(['git','rev-parse','origin/main'],text=True).strip()}))
PY
git push origin HEAD:refs/heads/issue-1582-79aa864de807 > /tmp/issue-1582-review/push.log 2>&1
