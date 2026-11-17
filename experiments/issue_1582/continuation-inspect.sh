#!/usr/bin/env bash
# Supplied read-only review recipe; zero independent authorship credit.
set -eu
bash experiments/issue_1582/continuation-client-research.sh
rg -n 'seed_client_integrations|context_window_tokens|RenderContext|renderTemplate' js | head -100
rg -n 'native-type-check|native-test|main-only|PR.*native|pull.request.*native' CONTRIBUTING.md scripts/check-pull-request-workflow-cost.mjs .github/workflows/release.yml | head -100
cat docs/requirements/issue-1582-client-read-access-and-context.md
cat rust/tests/web/issue-1582-defaults.test.mjs
sed -n '1,110p' js/agentic/crate/seed_client_integrations.mjs
sed -n '1,160p' experiments/issue_1582/language-parity-probe.mjs 2>/dev/null || true
