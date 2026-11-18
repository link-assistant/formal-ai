#!/usr/bin/env bash
set -eu
export PATH="/tmp/issue-1582-js-tools/shellcheck-v0.11.0:$PATH"
NODE_OPTIONS=--max-old-space-size=256 BASE_REF=origin/main bash scripts/check-secrets.sh > /tmp/issue-1582-secrets-fixed-full.log 2>&1
node experiments/formal_ai_subagent/local-gates.mjs --only lint-shell-scripts > /tmp/issue-1582-secrets-shell-gate.log 2>&1
node experiments/formal_ai_subagent/local-gates.mjs --only check-file-size-js-twin > /tmp/issue-1582-secrets-size-gate.log 2>&1
node experiments/formal_ai_subagent/local-gates.mjs --only check-abbreviations > /tmp/issue-1582-secrets-abbrev-gate.log 2>&1
