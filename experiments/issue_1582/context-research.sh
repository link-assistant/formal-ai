#!/usr/bin/env bash
set -eu
out=docs/case-studies/issue-1582/raw-data/research
gh api repos/xai-org/grok-build/git/trees/main?recursive=1 > "$out/grok-tree.json"
gh api repos/xai-org/grok-build/contents/README.md --jq .content | base64 -d > "$out/grok-readme.md"
rg -n -A 14 -B 4 'overrides|maxOutputTokens|modelConfig' "$out/gemini-generation-settings.md" | head -160
rg -n 'context|token|settings|model' "$out/grok-readme.md" | head -60
rg -n 'ADVERTISED_MAX_OUTPUT_TOKENS|served_model_limits' rust/src/server*.rs rust/src/server/*.rs
sed -n '198,225p' data/seed/multilingual-responses-agentic.lino
rg -n -A 25 'feature_capability_web_search_unavailable' rust/src/solver_handlers/feature_capability.rs
rg -n -A 22 -B 8 'feature_capability_web_search_unavailable' js/worker/formal_ai_worker_capabilities_and_runtime_rules.js
