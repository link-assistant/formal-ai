#!/usr/bin/env bash
set -u
rg -n 'struct FormalAiEngine|answer_with|context_window|model_limits|contextWindow' rust/src js/server | head -60
sed -n '1,200p' scripts/check-worker-line-budget.mjs
sed -n '1,180p' experiments/issue_1582/generate-docs.sh
sed -n '1,140p' rust/tests/web/support/structured-log-fixture.mjs
sed -n '1,130p' rust/tests/e2e/scripts/check-language-test-coverage.mjs
rg -n 'source rust/src/(solver_handler_how.rs|solver_handlers/(feature_capability.rs|web_requests.rs|web_requests/live_search.rs))|source_lines_max|outside_core_lines_max' data/meta/core-boundary-ledger.lino
rg -n 'meta-language|eb0574' .github/workflows/layered-ci.yml
cat data/meta/worker-line-budget/formal_ai_worker_obligations.lino
sed -n '65,90p' js/server/structured-log.mjs
id -u
command -v shellcheck
