#!/usr/bin/env bash
set -eu
test "$(git branch --show-current)" = issue-1582-79aa864de807
python3 experiments/issue_1582/record-verified-ci.py
python3 experiments/issue_1582/archive-evidence.py
git add docs/case-studies/issue-1582 experiments/issue_1582
git diff --cached --check
git commit -m "docs: record verified source-head CI and remaining acceptance limits"
test -z "$(git status --porcelain)"
