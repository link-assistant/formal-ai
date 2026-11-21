#!/usr/bin/env bash
set -eu
setsid node experiments/formal_ai_subagent/local-gates.mjs > /tmp/issue-1582-all-gates.log 2>&1 < /dev/null &
pid=$!
echo "$pid" > /tmp/issue-1582-all-gates.pid
echo "Local JavaScript gates started as process $pid"
