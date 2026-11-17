#!/usr/bin/env bash
set -eu
python3 experiments/issue_1582/continuation-secrets-record.py
python3 experiments/issue_1582/continuation-archive.py
git add scripts/check-secrets.sh rust/tests/web/bounded-subprocess-boundaries.test.mjs experiments/issue_1582 docs/case-studies/issue-1582 docs/case-studies/pull-request-1583
git diff --cached --check
mapfile -d '' files < <(git diff --cached --name-only --diff-filter=ACMRT -z)
NODE_OPTIONS=--max-old-space-size=256 bash scripts/check-secrets.sh "${files[@]}" > /tmp/issue-1582-secrets-staged-scan.log 2>&1
git diff --cached --stat > /tmp/issue-1582-secrets-commit-stat.log
