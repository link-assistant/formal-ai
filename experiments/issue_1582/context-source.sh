#!/usr/bin/env bash
set -eu
rg -n 'grok|gemini|qwen|aider|tokenLimit|contextWindow' data/seed/agentic* data/seed/client-integrations.lino | head -90
sed -n '350,430p' data/seed/client-integrations.lino
sed -n '760,810p' rust/src/client_integrations.rs
sed -n '380,410p' js/server/worker-host.mjs
sed -n '505,540p' js/worker/formal_ai_worker_web_providers_and_wasm_calls.js
rg -n 'fn merge_json_settings|fn set_json|parse.*value|json:' rust/src/client_integrations/global_config.rs
rg --files | rg '(client.integration|client-integration|with.claude|with_claude)' | head -65
