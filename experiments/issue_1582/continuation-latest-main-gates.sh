#!/usr/bin/env bash
export PATH="/tmp/issue-1582-js-tools/shellcheck-v0.11.0:$PATH"
node experiments/formal_ai_subagent/local-gates.mjs
status=$?
printf '%s\n' "$status" > /tmp/issue-1582-merge-review/all-gates.exit
exit "$status"
