#!/usr/bin/env bash
set -eu
python3 experiments/issue_1582/last-review.py
node scripts/mirror-package-data.mjs --write
node scripts/audit-seed-metadata.mjs --write
node scripts/translate-es.mjs --write
rustfmt --edition 2024 rust/src/cli_logs.rs rust/tests/unit/cli_logs.rs rust/tests/integration/with_formal_ai/fixture_env.rs rust/tests/integration/with_formal_ai/issue_1582.rs
node --test --test-concurrency=1 rust/tests/web/server-structured-log.test.mjs
git diff --check
