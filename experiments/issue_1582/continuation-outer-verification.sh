#!/usr/bin/env bash
# Explicit execution-path exception: the Formal AI shell's fixed 60-second
# deadline stops the combined checks and cleans up detached children.
# No Rust build; same maintained checks, sequentially, with complete log files.
set -eu
trap 'echo "$?" > /tmp/issue-1582-review/outer-verification.exit' EXIT
node --test --test-concurrency=1 rust/tests/web/rust-specification-unicode-identifiers.test.mjs > /tmp/issue-1582-review/rust-specification-unicode-identifiers.log 2>&1
node scripts/assemble-requirements.mjs --write > /tmp/issue-1582-review/assemble.log 2>&1
node scripts/generate-requirement-status.mjs --write > /tmp/issue-1582-review/requirements.log 2>&1
node scripts/render-status.mjs --write > /tmp/issue-1582-review/render-status.log 2>&1
node scripts/mirror-package-data.mjs --write > /tmp/issue-1582-review/mirror-status.log 2>&1
git add data js ts rust experiments/issue_1582 docs changelog.d
bash experiments/issue_1582/continuation-gates.sh > /tmp/issue-1582-review/all-gates.log 2>&1
