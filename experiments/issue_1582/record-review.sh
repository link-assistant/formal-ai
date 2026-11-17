#!/usr/bin/env bash
set -eu
python3 experiments/issue_1582/record-review.py
node --test --test-concurrency=1 rust/tests/web/issue-1582-defaults.test.mjs
node --test --test-concurrency=1 rust/tests/web/server-structured-log-cli.test.mjs
git diff --name-only HEAD -- '*.rs' | while IFS= read -r source; do rustfmt --edition 2024 --check "$source"; done
git diff --check
git diff HEAD --stat -- . ':!docs/case-studies/issue-1582/raw-data'
