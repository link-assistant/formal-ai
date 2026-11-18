#!/usr/bin/env bash
# Supplied bounded source-review recipe, zero autonomous authorship credit.
set -eu
git diff --cc -- .cargo/audit.toml data/meta/core-boundary-ledger.lino data/meta/worker-line-budget/formal_ai_worker_obligations.lino
cat experiments/issue_1582/refresh-governed-witness.mjs
rg -n 'tool "|context|permission|read-only' data/seed/client-integrations.lino
rg --files js ts rust/src/client_integrations | rg 'client.integr|read.access|capabilit'
sed -n '1,260p' rust/tests/integration/with_formal_ai/issue_1582.rs
gh issue view https://github.com/google-gemini/gemini-cli/issues/30333 --json title,body,state,comments
gh api repos/google-gemini/gemini-cli/contents/packages/core/src/config/models.ts --jq .content | base64 -d > /tmp/issue-1582-review/gemini-models-current.ts
gh api repos/google-gemini/gemini-cli/contents/packages/core/src/core/tokenLimits.ts --jq .content | base64 -d > /tmp/issue-1582-review/gemini-tokenLimits-current.ts
cat /tmp/issue-1582-review/gemini-tokenLimits-current.ts
