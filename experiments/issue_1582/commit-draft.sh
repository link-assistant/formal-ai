#!/usr/bin/env bash
set -eu
test "$(git branch --show-current)" = issue-1582-79aa864de807
git add REQUIREMENTS.md data changelog.d js ts rust docs/requirements docs/requirements-traceability.md docs/progressive-plan.md docs/status.md
git diff --cached --check
git restore --staged docs/case-studies/issue-1582 experiments/issue_1582
git commit -m "fix: enable client read access, quiet diagnostics, and served context limits"
python3 experiments/issue_1582/archive-evidence.py
git add docs/case-studies/issue-1582 experiments/issue_1582
git diff --cached --check
git commit -m "docs: retain issue 1582 case study, reproductions, and investigation evidence"
git status --short
git log -3 --oneline
