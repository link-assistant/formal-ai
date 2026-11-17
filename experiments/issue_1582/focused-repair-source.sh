#!/usr/bin/env bash
set -u
sed -n '50,125p' rust/src/engine.rs
sed -n '1,105p' js/server/service.mjs
sed -n '60,135p' rust/tests/web/support/structured-log-fixture.mjs
sed -n '1,190p' rust/tests/e2e/scripts/check-language-test-coverage.mjs
sed -n '1,18p' data/meta/core-boundary-ledger.lino
sed -n '66,76p' data/meta/core-boundary-ledger.lino
sed -n '216,226p' data/meta/core-boundary-ledger.lino
sed -n '476,491p' data/meta/core-boundary-ledger.lino
sed -n '1,75p' js/server/memory-store.mjs
sed -n '1,110p' scripts/check-source-model.mjs
rg -n 'internet_access_help' data/seed/meanings-feature-capability.lino data/seed/multilingual-responses-capabilities.lino js/worker/formal_ai_worker_capabilities_and_runtime_rules.js js/worker/formal_ai_worker_solve.js rust/src/solver_handlers/feature_capability.rs rust/src/solver_handler_how.rs
rg -n '^  offline|offline:' rust/src/solver_config.rs
