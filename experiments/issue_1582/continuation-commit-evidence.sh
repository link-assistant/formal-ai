#!/usr/bin/env bash
set -eu
test "$(git branch --show-current)" = issue-1582-79aa864de807
git add data/parity/issue-1582-permissions.json rust/tests/web/issue-1582-permissions.test.mjs data/meta/requirement-status-ledger docs/requirements docs/requirements-traceability.md docs/case-studies/issue-1582 docs/case-studies/pull-request-1583 experiments/issue_1582 experiments/formal_ai_subagent/evidence/specification-delivery-1188/unicode-identifier-identity/governed-baseline.v8
git diff --cached --check
git diff --cached --stat > /tmp/issue-1582-review/final-commit-stat.log
git commit -m "Cover every coding target and preserve issue 1582 review evidence" > /tmp/issue-1582-review/evidence-commit.log 2>&1
git status --short
