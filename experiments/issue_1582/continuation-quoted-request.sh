#!/usr/bin/env bash
set -eu
node experiments/issue_1582/continuation-quoted-request.mjs
node scripts/generate-worker-prerequisites.mjs --write > /tmp/issue-1582-review/prerequisites-final.log 2>&1
rustfmt --edition 2024 rust/src/solver_handler_how.rs rust/src/solver_handlers/feature_capability.rs rust/tests/unit/issue_1582_permissions.rs
node scripts/translate-es.mjs --write > /tmp/issue-1582-review/translate-es-final.log 2>&1
node scripts/mirror-package-data.mjs --write > /tmp/issue-1582-review/mirror-final.log 2>&1
node --test --test-concurrency=1 rust/tests/web/issue-1582-permissions.test.mjs > /tmp/issue-1582-review/permissions-after-final.log 2>&1
node experiments/issue_1582/continuation-census.mjs
node experiments/issue_1582/continuation-matrix.mjs
