#!/usr/bin/env bash
set -eu
python3 experiments/issue_1582/repair-native-logging.py
npm ci --prefix .meta-language/js --omit=dev --ignore-scripts --no-audit --no-fund
node scripts/assemble-requirements.mjs --write
node scripts/generate-runtime-feature-inventory.mjs --write
node scripts/measure-notation.mjs --write
git diff --name-only HEAD -- '*.rs' | while IFS= read -r source; do rustfmt --edition 2024 "$source"; done
rustfmt --edition 2024 rust/tests/integration/with_formal_ai/issue_1582.rs
git diff --check
rg -n 'logs|native-|serve-|\.lino|fn configure' rust/src/structured_log.rs
sed -n '1,180p' js/server/convert-log.mjs
sed -n '60,115p' .github/workflows/layered-ci.yml
sed -n '700,770p' .github/workflows/layered-ci.yml
