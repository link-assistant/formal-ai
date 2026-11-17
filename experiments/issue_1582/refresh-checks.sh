#!/usr/bin/env bash
set -eu
git add js/server/read-access.mjs rust/tests/web/issue-1582-defaults.test.mjs rust/tests/integration/with_formal_ai/issue_1582.rs docs/requirements/issue-1582-client-read-access-and-context.md
node scripts/mirror-package-data.mjs --write
node scripts/generate-worker-prerequisites.mjs --write
node scripts/translate-es.mjs --write
node scripts/assemble-requirements.mjs --write
node scripts/generate-requirement-status.mjs --write
node scripts/render-status.mjs --write
node scripts/render-progressive-plan.mjs --write
node scripts/generate-runtime-feature-inventory.mjs --write
node scripts/measure-notation.mjs --write
git diff --name-only HEAD -- '*.rs' | while IFS= read -r source; do rustfmt --edition 2024 "$source"; done
rustfmt --edition 2024 rust/tests/integration/with_formal_ai/issue_1582.rs
git diff --check
sed -n '530,585p' rust/tests/integration/with_formal_ai.rs
sed -n '1,20p' rust/tests/unit/issue_1173_fallback_executes_search.rs
cat .meta-language/js/package.json
find /tmp/issue-1582-shellcheck -maxdepth 2 -type f
