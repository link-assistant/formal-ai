#!/usr/bin/env bash
set -eu
sed -n '1,245p' rust/tests/integration/with_formal_ai_global.rs
rg -n 'fn served_model_limits|served_model_limits' rust/src/server.rs rust/src/server/ rust/src/client_integrations*
out=docs/case-studies/issue-1582/raw-data/research
for name in 05-configuration 11-custom-models 26-config-reference; do
  gh api "repos/xai-org/grok-build/contents/crates/codegen/xai-grok-pager/docs/user-guide/$name.md" --jq .content | base64 -d > "$out/grok-$name.md"
done
rg -n -A 8 -B 4 'context|window|maxTokens|max_tokens|api.*key|baseURL' "$out/grok-11-custom-models.md" "$out/grok-26-config-reference.md" | head -145
cat "$out/gemini-related-issues.json"
