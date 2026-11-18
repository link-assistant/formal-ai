#!/usr/bin/env bash
node experiments/formal_ai_subagent/local-gates.mjs
status=$?
echo "$status" > /tmp/issue-1582-review/all-gates.exit
exit "$status"
