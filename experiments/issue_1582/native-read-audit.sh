#!/usr/bin/env bash
set -eu
exec > /tmp/issue-1582-native-read-audit-source.log 2>&1
rg -n 'FORMAL_AI_LIVE_FETCH|LIVE_FETCH_ENV_VAR' rust/src js --glob '*.rs' --glob '*.mjs'
rg -n 'CARGO_BIN_EXE_formal-ai|\.args\(\["(ask|solve|run)|\.arg\("(ask|solve|run)|fetch.*http|http.*fetch' rust/tests/integration/with_formal_ai rust/tests/unit/issue_117* rust/src/main.rs
sed -n '110,165p' rust/src/solver_config.rs
sed -n '270,345p' rust/src/solver_handlers/web_requests.rs
sed -n '115,140p' rust/src/solver_unknown_reasoning.rs
sed -n '185,215p' js/agentic/crate/version_resolution.mjs
sed -n '1,45p' docs/requirements/issue-1548-vision-blockers-and-plan.md
sed -n '265,290p' rust/tests/unit/symbolic_kernel_coverage.rs
sed -n '600,620p' rust/tests/unit/specification/external_benchmarks.rs
sed -n '650,670p' rust/tests/unit/specification/external_benchmarks.rs
sed -n '812,830p' rust/tests/unit/specification/external_benchmarks.rs
