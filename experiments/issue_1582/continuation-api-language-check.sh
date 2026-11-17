#!/usr/bin/env bash
set -eu
python3 experiments/issue_1582/continuation-api-language-check.py
node --max-old-space-size=384 --test --test-concurrency=1 rust/tests/web/issue-1582-permissions.test.mjs > /tmp/issue-1582-merge-review/api-permissions.log 2>&1
