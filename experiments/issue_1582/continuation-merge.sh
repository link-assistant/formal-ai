#!/usr/bin/env bash
# Supplied atomic merge recipe; no remote mutation.
set -eu
# The 62 merged issue-1588 cases already passed; retain that first observation.
node scripts/check-minimal-core-boundary.mjs
git add docs/case-studies/issue-1582/raw-data/local/governed-witness-refresh.json
git diff --check --cached -- .cargo/audit.toml data/meta/core-boundary-ledger.lino data/meta/worker-line-budget/formal_ai_worker_obligations.lino docs/case-studies/issue-1582/raw-data/local/governed-witness-refresh.json
git commit -m 'Merge main and retain both issue regression inventories'
