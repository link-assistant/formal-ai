#!/usr/bin/env bash
set -eu
python3 experiments/issue_1582/archive-evidence.py
git add docs/case-studies/issue-1582 experiments/issue_1582
git diff --cached --check
git commit -m "docs: retain complete repair CI logs and witness review evidence"
test -z "$(git status --porcelain)"
