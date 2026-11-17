#!/usr/bin/env bash
set -eu
sed -n '1,220p' rust/tests/web/support/structured-log-fixture.mjs
rg -n 'log-|logger|verbose|trace|createServeLogger' js/server/main.mjs
sed -n '1,120p' rust/src/cli_env.rs
sed -n '430,452p' rust/src/coding/synthesis_runtime.rs
sed -n '1,46p' rust/src/solver_handlers/web_requests/live_search.rs
rg -n 'flagEnabled' js/agentic/node-host.mjs
sed -n '1,50p' js/worker/formal_ai_worker_capabilities_and_runtime_rules.js
