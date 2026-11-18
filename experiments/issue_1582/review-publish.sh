#!/usr/bin/env bash
set -eu
sed -n '1,255p' rust/src/structured_log.rs
git diff HEAD -- rust/src/client_integrations.rs rust/src/client_integrations/global_config.rs rust/src/main.rs rust/src/cli_env.rs
rg -n 'permission|allowedTools|deny|auto|trust|readOnly|read-only' data/seed/client-integrations.lino
git diff HEAD -- js/server/main.mjs js/server/worker-host.mjs js/agentic/node-host.mjs rust/src/solver_handlers/web_requests.rs rust/src/solver_handlers/web_requests/live_search.rs rust/src/coding/synthesis_runtime.rs
git diff --check --cached
