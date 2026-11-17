#!/usr/bin/env bash
set -eu
python3 experiments/issue_1582/continuation-latest-main-record.py
python3 experiments/issue_1582/continuation-archive.py --source /tmp/issue-1582-merge-review --subdir latest-main
git add experiments/issue_1582 docs/case-studies/issue-1582 docs/case-studies/pull-request-1583
node scripts/check-file-size.mjs > /tmp/issue-1582-latest-main-final-size.log 2>&1
/tmp/issue-1582-js-tools/shellcheck-v0.11.0/shellcheck experiments/issue_1582/continuation-latest-main-*.sh
git diff --cached --check origin/main
git commit -m 'Merge main request-context support and preserve client fixes' > /tmp/issue-1582-latest-main-commit.log 2>&1
