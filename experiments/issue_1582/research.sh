#!/usr/bin/env bash
set -eu
dir=docs/case-studies/issue-1582/raw-data/research
mkdir -p "$dir"
gh api repos/QwenLM/qwen-code/contents/docs/users/configuration/settings.md --jq .content | base64 -d > "$dir/qwen-settings.md"
gh api repos/google-gemini/gemini-cli/contents/packages/core/src/core/tokenLimits.ts --jq .content | base64 -d > "$dir/gemini-tokenLimits.ts"
gh api repos/google-gemini/gemini-cli/contents/docs/cli/generation-settings.md --jq .content | base64 -d > "$dir/gemini-generation-settings.md"
rg -n -A 10 -B 6 'contextWindowSize|QWEN_CODE_MAX_OUTPUT_TOKENS' "$dir/qwen-settings.md" | head -100
cat "$dir/gemini-tokenLimits.ts"
rg -n -A 12 -B 4 'customAliases|tokenLimit|maxOutputTokens' "$dir/gemini-generation-settings.md" | head -100
rg -n 'grok-cli|grok-build|xai.*grok|superagent' data/seed data/benchmarks package.json .github | head -55
rg -l 'codex_model_catalog|run_with_formal_ai|CLAUDE_CONFIG_DIR' rust/tests/unit rust/tests/integration rust/tests/source | head -45
