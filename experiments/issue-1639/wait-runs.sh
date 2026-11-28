#!/usr/bin/env bash
# Wait until every workflow run for a head SHA on the branch completes, then print failures.
sha=$1; branch=issue-1639-47f386de821a
sleep 120
while :; do
  out=$(gh run list --repo link-assistant/formal-ai --branch "$branch" --limit 60 --json databaseId,status,conclusion,name,headSha)
  pending=$(echo "$out" | python3 -c "import json,sys;print(sum(1 for r in json.load(sys.stdin) if r['headSha']=='$sha' and r['status']!='completed'))")
  [ "$pending" = 0 ] && break
  sleep 120
done
echo "$out" | python3 -c "import json,sys;[print(r['databaseId'],r['conclusion'],r['name']) for r in json.load(sys.stdin) if r['headSha']=='$sha']"
