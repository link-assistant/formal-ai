#!/usr/bin/env bash
set -eu
sed -n '60,95p' rust/src/client_integrations/server.rs
sed -n '1,140p' experiments/formal_ai_subagent/local-gates.mjs
rg --files docs/requirements | tail -12
rg --files docs/requirements | rg '1579|1575|158'
sed -n '1,150p' scripts/generate-requirement-status.mjs
sed -n '1,90p' data/benchmarks/fast-pull-request-javascript-tests.json
sed -n '1415,1450p' CONTRIBUTING.md
rg -n 'outcome|blocked|R1579' data/meta/requirement-status.lino | tail -15
rg -n 'claude|capability' data/benchmarks/fast-pull-request-javascript-tests.json | head -15
