#!/usr/bin/env bash
set -eu
branch=issue-1582-79aa864de807
repo=link-assistant/formal-ai
test "$(git branch --show-current)" = "$branch"
test -z "$(git status --porcelain=v1)"
git fetch origin main "$branch" > /tmp/issue-1582-secrets-fetch-push.log 2>&1
git merge-base --is-ancestor origin/main HEAD
gh run list --repo "$repo" --branch "$branch" --status in_progress --json databaseId,workflowName,headSha > /tmp/issue-1582-secrets-in-progress.json
gh run list --repo "$repo" --branch "$branch" --status queued --json databaseId,workflowName,headSha > /tmp/issue-1582-secrets-queued.json
gh run list --repo "$repo" --branch "$branch" --limit 100 --json databaseId,workflowName,headSha,status,conclusion,createdAt > /tmp/issue-1582-secrets-runs-push.json
gh run view 44152920794 --repo "$repo" --json headSha,conclusion,jobs > /tmp/issue-1582-secrets-pipeline-push.json
python3 - <<'PY'
import json,subprocess
from pathlib import Path
for name in ['in-progress','queued']:
 assert not json.loads(Path('/tmp/issue-1582-secrets-'+name+'.json').read_text()),name
sha=subprocess.check_output(['git','rev-parse','origin/issue-1582-79aa864de807'],text=True).strip()
assert sha=='a0fb531c57a3067f48640519a884ac97adfd7656',sha
runs=[r for r in json.loads(Path('/tmp/issue-1582-secrets-runs-push.json').read_text()) if r['headSha']==sha]
assert len(runs)==22, len(runs)
assert all(r['status']=='completed' for r in runs),runs
failures=[r for r in runs if r['conclusion']!='success']
assert len(failures)==1 and failures[0]['databaseId']==44152920794 and failures[0]['conclusion']=='failure',failures
pipeline=json.loads(Path('/tmp/issue-1582-secrets-pipeline-push.json').read_text())
assert pipeline['headSha']==sha
assert {j['name'] for j in pipeline['jobs'] if j['conclusion'] not in {'success','skipped'}}=={'Secrets Scan','Pipeline Status'}
assert 'xargs -0 -s 16384 npx --yes' in Path('scripts/check-secrets.sh').read_text()
assert 'No secrets found.' in Path('/tmp/issue-1582-secrets-staged-scan.log').read_text()
assert 'pass 14' in Path('/tmp/issue-1582-secrets-after-drive.log').read_text()
print(json.dumps({'previousHead':sha,'successfulRuns':21,'collectedAndRepairedFailure':44152920794,'main':subprocess.check_output(['git','rev-parse','origin/main'],text=True).strip()}))
PY
git push origin HEAD:refs/heads/issue-1582-79aa864de807 > /tmp/issue-1582-secrets-push.log 2>&1
