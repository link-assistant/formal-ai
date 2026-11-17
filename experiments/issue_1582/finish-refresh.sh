#!/usr/bin/env bash
set -eu
python3 experiments/issue_1582/finish-refresh.py
node scripts/generate-requirement-status.mjs --write
node scripts/render-status.mjs --write
node scripts/render-progressive-plan.mjs --write
node scripts/generate-runtime-feature-inventory.mjs --write
node scripts/measure-notation.mjs --write
git diff --name-only HEAD -- '*.rs' | while IFS= read -r source; do rustfmt --edition 2024 "$source"; done
rustfmt --edition 2024 rust/tests/integration/with_formal_ai/issue_1582.rs
git diff --check
sed -n '530,585p' rust/tests/integration/with_formal_ai.rs
cat .meta-language/js/package.json
find /tmp/issue-1582-shellcheck -maxdepth 2 -type f
