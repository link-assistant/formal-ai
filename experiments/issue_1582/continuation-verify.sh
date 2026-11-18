#!/usr/bin/env bash
set -eu
python3 experiments/issue_1582/continuation-documentation.py
rustfmt --edition 2024 --check --config skip_children=true rust/src/solver_handler_how.rs rust/src/solver_handlers/feature_capability.rs rust/tests/unit/issue_1582_permissions.rs rust/tests/unit/mod.rs
node experiments/issue_1582/continuation-matrix.mjs > /tmp/issue-1582-review/support-matrix.log 2>&1
node experiments/issue_1582/gemini-context-probe.mjs > /tmp/issue-1582-review/gemini-probe.log 2>&1
for name in issue-1582-permissions issue-1582-defaults server-structured-log server-structured-log-cli rust-specification-unicode-identifiers; do
  node --test --test-concurrency=1 "rust/tests/web/$name.test.mjs" > "/tmp/issue-1582-review/$name.log" 2>&1
done
node scripts/assemble-requirements.mjs --write > /tmp/issue-1582-review/assemble.log 2>&1
node scripts/generate-requirement-status.mjs --write > /tmp/issue-1582-review/requirements.log 2>&1
node scripts/render-status.mjs --write > /tmp/issue-1582-review/render-status.log 2>&1
node scripts/mirror-package-data.mjs --write > /tmp/issue-1582-review/mirror-status.log 2>&1
setsid bash experiments/issue_1582/continuation-gates.sh > /tmp/issue-1582-review/all-gates.log 2>&1 < /dev/null &
echo "$!" > /tmp/issue-1582-review/all-gates.pid
printf 'Focused checks passed; gate runner launched.\n'
