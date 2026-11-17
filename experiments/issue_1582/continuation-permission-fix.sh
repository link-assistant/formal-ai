#!/usr/bin/env bash
# Supplied repair and maintained projections, no compiler invocation.
set -eu
# The supplied edits were applied before rustfmt's default-edition rejection.
rustfmt --edition 2024 rust/src/solver_handler_how.rs rust/src/solver_handlers/feature_capability.rs rust/tests/unit/issue_1582_permissions.rs
node scripts/generate-worker-prerequisites.mjs --write
node scripts/translate-es.mjs --write > /tmp/issue-1582-review/translate-es.log 2>&1
node scripts/mirror-package-data.mjs --write > /tmp/issue-1582-review/mirror.log 2>&1
node --test --test-concurrency=1 rust/tests/web/issue-1582-permissions.test.mjs > /tmp/issue-1582-review/permissions-after.log 2>&1
