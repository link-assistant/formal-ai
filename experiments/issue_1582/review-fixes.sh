#!/usr/bin/env bash
set -eu
python3 experiments/issue_1582/review-fixes.py
node scripts/mirror-package-data.mjs --write
node scripts/translate-es.mjs --write
rustfmt --edition 2024 rust/src/client_integrations/global_config.rs
git diff --check
