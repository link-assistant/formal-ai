#!/usr/bin/env bash
set -u
rg -n 'automatedTest|automated_test|testPath|\.rs|traceability|row\[' scripts/generate-requirement-status.mjs | head -70
rg -n 'R1582-3|R1582-5' docs/requirements-traceability.md docs/requirements/issue-1582-client-read-access-and-context.md data/meta/requirement-status-ledger/issue-1582-client-read-access-and-context.lino
sed -n '530,585p' rust/tests/integration/with_formal_ai.rs
cat .meta-language/js/package.json
find /tmp/issue-1582-shellcheck -maxdepth 2 -type f
