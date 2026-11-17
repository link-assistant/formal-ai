#!/usr/bin/env bash
set -u
git diff HEAD -- js/server/read-access.mjs js/server/main.mjs js/server/worker-host.mjs js/agentic/node-host.mjs rust/src/solver_config.rs rust/src/solver_handlers/web_requests.rs rust/src/solver_handlers/web_requests/live_search.rs rust/src/coding/synthesis_runtime.rs
sed -n '60,115p' .github/workflows/layered-ci.yml
sed -n '700,770p' .github/workflows/layered-ci.yml
git log -3 --oneline -- js/worker/formal_ai_worker_obligations.js
git show origin/main:js/worker/formal_ai_worker_obligations.js | wc -l
