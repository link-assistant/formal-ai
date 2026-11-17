#!/usr/bin/env bash
# Explicit outer verification exception: combined checks exceed 60 seconds.
set -u
failed=0
for gate in check-file-size-js-twin check-abbreviations check-requirement-extraction check-requirement-coverage check-requirement-status check-generated-files; do
 node experiments/formal_ai_subagent/local-gates.mjs --only "$gate" > "/tmp/issue-1582-review/pre-push-$gate.log" 2>&1 || failed=1
done
echo "$failed" > /tmp/issue-1582-review/pre-push-gates.exit
exit "$failed"
