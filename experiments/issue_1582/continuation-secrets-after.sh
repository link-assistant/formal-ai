#!/usr/bin/env bash
set -eu
python3 experiments/issue_1582/continuation-secrets-fix.py
node --max-old-space-size=256 --test --test-concurrency=1 rust/tests/web/bounded-subprocess-boundaries.test.mjs
