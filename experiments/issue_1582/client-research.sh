#!/usr/bin/env bash
set -eu
out=docs/case-studies/issue-1582/raw-data/research
gh search repos 'grok build in:name,description' --limit 10 --json fullName,description,url > "$out/grok-repositories.json"
gh api repos/anthropics/claude-code/issues/35630 > "$out/claude-35630.json"
gh api repos/anthropics/claude-code/issues/55182 > "$out/claude-55182.json"
sed -n '290,355p' rust/src/client_integrations/global_config.rs
sed -n '35,115p' rust/src/solver_handlers/web_requests.rs
sed -n '500,555p' js/worker/formal_ai_worker_web_providers_and_wasm_calls.js
sed -n '1,140p' rust/src/cli_env.rs
rg -n 'web_search_unavailable|default.*network|disabled by default' data/seed/multilingual-responses*.lino
cat "$out/grok-repositories.json"
