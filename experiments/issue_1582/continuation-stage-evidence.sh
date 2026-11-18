#!/usr/bin/env bash
set -eu
python3 experiments/issue_1582/continuation-archive.py
git add docs/case-studies/issue-1582 docs/case-studies/pull-request-1583 experiments/issue_1582 experiments/formal_ai_subagent/evidence/specification-delivery-1188/unicode-identifier-identity/governed-baseline.v8
git diff --cached --check
git diff --cached --stat > /tmp/issue-1582-review/evidence-commit-stat.log
