#!/usr/bin/env bash
set -eu
trap 'echo "$?" > /tmp/issue-1582-review/finish-verification.exit' EXIT
node --test --test-concurrency=1 rust/tests/web/server-structured-log-cli.test.mjs > /tmp/issue-1582-review/server-structured-log-cli.log 2>&1
node --test --test-concurrency=1 rust/tests/web/rust-specification-unicode-identifiers.test.mjs > /tmp/issue-1582-review/rust-specification-unicode-identifiers.log 2>&1
node scripts/assemble-requirements.mjs --write > /tmp/issue-1582-review/assemble.log 2>&1
node scripts/generate-requirement-status.mjs --write > /tmp/issue-1582-review/requirements.log 2>&1
node scripts/render-status.mjs --write > /tmp/issue-1582-review/render-status.log 2>&1
node scripts/mirror-package-data.mjs --write > /tmp/issue-1582-review/mirror-status.log 2>&1
git add data js ts rust experiments/issue_1582 docs changelog.d
bash experiments/issue_1582/continuation-gates.sh > /tmp/issue-1582-review/all-gates.log 2>&1
