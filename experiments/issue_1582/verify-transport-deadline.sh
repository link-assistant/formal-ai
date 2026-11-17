#!/usr/bin/env bash
set -eu
python3 experiments/issue_1582/transport-deadline.py
node scripts/translate-es.mjs --write
node --test --test-concurrency=1 rust/tests/web/issue-1582-defaults.test.mjs
git diff --check
