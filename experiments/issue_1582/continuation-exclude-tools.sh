#!/usr/bin/env bash
# Local prerequisites are temporary SDK files, not repository source.
set -eu
exclude=$(git rev-parse --git-path info/exclude)
printf '\n/.meta-language/\n/node_modules\n' >> "$exclude"
node experiments/formal_ai_subagent/local-gates.mjs --only check-abbreviations > /tmp/issue-1582-review/repaired-abbreviations-final.log 2>&1
