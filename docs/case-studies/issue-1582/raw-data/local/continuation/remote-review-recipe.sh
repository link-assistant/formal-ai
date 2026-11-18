#!/usr/bin/env bash
set -eu
gh api 'repos/link-assistant/formal-ai/pulls/1583/files?per_page=100' --paginate --jq '.[] | {filename,status,sha,additions,deletions,changes}' > /tmp/issue-1582-review/pr-files-final.jsonl
gh pr view 1583 --repo link-assistant/formal-ai --json title,body,isDraft,mergeable,mergeStateStatus,headRefOid,baseRefName,url,statusCheckRollup > /tmp/issue-1582-review/pr-after-push.json
python3 - <<'PY'
import json,subprocess
rows=[json.loads(s) for s in open('/tmp/issue-1582-review/pr-files-final.jsonl')]
expected=subprocess.check_output(['git','diff','--name-only','origin/main','HEAD'],text=True).splitlines()
assert sorted(r['filename'] for r in rows)==sorted(expected)
for r in rows:
 if r['status']=='removed':continue
 sha=subprocess.check_output(['git','rev-parse','HEAD:'+r['filename']],text=True).strip()
 assert sha==r['sha'], (r['filename'],sha,r['sha'])
print(json.dumps({'fileCount':len(rows),'allRemoteBlobIdentitiesMatch':True}))
PY
