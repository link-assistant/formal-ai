#!/usr/bin/env bash
set -eu
python3 experiments/issue_1582/continuation-secrets-regression.py
node --max-old-space-size=256 --test --test-concurrency=1 --test-name-pattern='secret scanner bounds' rust/tests/web/bounded-subprocess-boundaries.test.mjs
