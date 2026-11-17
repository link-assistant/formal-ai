#!/usr/bin/env bash
set -eu
export PATH="/tmp/issue-1582-js-tools/shellcheck-v0.11.0:$PATH"
git add experiments/issue_1582/continuation-secrets-push.sh experiments/issue_1582/continuation-secrets-commit.sh
shellcheck experiments/issue_1582/continuation-secrets-*.sh
node experiments/formal_ai_subagent/local-gates.mjs --only check-file-size-js-twin > /tmp/issue-1582-secrets-final-size.log 2>&1
git diff --cached --check
git commit -m 'Bound secret scan batches before npm shell expansion' -m 'Retain the E2BIG failure and a failing-before/passing-after regression; preserve every file and scanner failure status.' -m 'Formal-AI-Authorship: external recipe execution; 0 independently accepted changes' > /tmp/issue-1582-secrets-commit.log 2>&1
