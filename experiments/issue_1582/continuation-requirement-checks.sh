#!/usr/bin/env bash
set -eu
node scripts/assemble-requirements.mjs --write > /tmp/issue-1582-review/assemble-programming.log 2>&1
node scripts/generate-requirement-status.mjs --write > /tmp/issue-1582-review/requirements-programming.log 2>&1
node scripts/render-status.mjs --write > /tmp/issue-1582-review/render-programming.log 2>&1
node scripts/mirror-package-data.mjs --write > /tmp/issue-1582-review/mirror-programming.log 2>&1
node scripts/check-requirement-status.mjs > /tmp/issue-1582-review/check-status-programming.log 2>&1
node scripts/render-status.mjs --check > /tmp/issue-1582-review/check-render-programming.log 2>&1
node scripts/render-progressive-plan.mjs --check > /tmp/issue-1582-review/check-plan-programming.log 2>&1
for gate in check-requirement-extraction check-requirement-coverage check-abbreviations check-file-size-js-twin; do
 node experiments/formal_ai_subagent/local-gates.mjs --only "$gate" > "/tmp/issue-1582-review/programming-$gate.log" 2>&1
done
