#!/usr/bin/env bash
set -eu
python3 experiments/issue_1582/continuation-latest-main-resolve.py
node scripts/assemble-requirements.mjs --write
node scripts/generate-requirement-status.mjs --write
node scripts/render-status.mjs --write
node scripts/render-progressive-plan.mjs --write
node scripts/assemble-requirements.mjs
node scripts/generate-requirement-status.mjs
node scripts/check-requirement-status.mjs
node scripts/render-status.mjs --check
node scripts/render-progressive-plan.mjs --check
git add REQUIREMENTS.md data/meta/requirement-status-ledger.lino data/meta/requirement-status-ledger data/meta/progressive-plan-ratchet.lino docs/requirements/assembled docs/progressive-plan.md docs/requirements-traceability.md docs/status.md
git diff --check
