#!/usr/bin/env bash
set -eu
node --max-old-space-size=384 --test --test-concurrency=1 rust/tests/web/issue-1587-request-context.test.mjs > /tmp/issue-1582-merge-review/request-context.log 2>&1
node --max-old-space-size=384 --test --test-concurrency=1 rust/tests/web/issue-1582-permissions.test.mjs > /tmp/issue-1582-merge-review/permissions.log 2>&1
node --max-old-space-size=384 --test --test-concurrency=1 rust/tests/web/issue-1582-defaults.test.mjs > /tmp/issue-1582-merge-review/defaults.log 2>&1
