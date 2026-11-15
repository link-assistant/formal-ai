#!/bin/bash
# Poll CI runs for a commit until none are pending; print a compact status line each round.
sha=$1
while true; do
  out=$(gh run list --repo link-assistant/formal-ai --commit "$sha" --json databaseId,name,status,conclusion 2>&1)
  pending=$(echo "$out" | jq -r '[.[]|select(.status!="completed")]|length' 2>/dev/null)
  failed=$(echo "$out" | jq -r '[.[]|select(.status=="completed" and .conclusion!="success" and .conclusion!="skipped")|.name]|join(",")' 2>/dev/null)
  echo "$(date +%H:%M:%S) pending=$pending failed=[$failed]"
  if [ "$pending" = "0" ]; then echo "$out" | jq -c '.[]'; break; fi
  sleep 120
done
