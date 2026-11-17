#!/usr/bin/env bash
# Supplied bounded audit, no autonomous authorship claim.
set -eu
rg -n 'client|with|command' js/server/main.mjs | head -55 || true
gh api 'repos/google-gemini/gemini-cli/git/trees/main?recursive=1' > /tmp/issue-1582-review/gemini-tree.json
jq -r '.tree[].path | select(test("(modelConfig|chatCompression|tokenLimit)"))' /tmp/issue-1582-review/gemini-tree.json
gh api repos/google-gemini/gemini-cli/contents/packages/core/src/core/client.ts --jq .content | base64 -d > /tmp/issue-1582-review/gemini-client-current.ts
gh api repos/google-gemini/gemini-cli/contents/packages/core/src/context/chatCompressionService.ts --jq .content | base64 -d > /tmp/issue-1582-review/gemini-compression-current.ts
gh api repos/google-gemini/gemini-cli/contents/packages/core/src/services/modelConfigService.ts --jq .content | base64 -d > /tmp/issue-1582-review/gemini-config-current.ts
rg -n -C 5 'tokenLimit|TokenLimit|contextWindow|inputToken' /tmp/issue-1582-review/gemini-*-current.ts
sed -n '1,140p' js/server/main.mjs
sed -n '1,160p' scripts/check-minimal-core-boundary.mjs
rg -n 'modelConfig|contextWindow|tokenLimit|chatCompression' /tmp/issue-1582-review/gemini-config-current.ts
git log -1 --format='%H %cI %s' origin/main
gh run list --repo link-assistant/formal-ai --branch main --limit 5 --json databaseId,workflowName,status,conclusion,createdAt,headSha
