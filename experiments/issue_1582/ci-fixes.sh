#!/usr/bin/env bash
set -eu
python3 experiments/issue_1582/ci-fixes.py
node scripts/generate-role-registry.mjs
python3 scripts/generate-role-registry.py --check
node scripts/mirror-package-data.mjs --write
node scripts/generate-worker-prerequisites.mjs --write
node scripts/measure-notation.mjs --write
node scripts/render-status.mjs --check
git diff --check
