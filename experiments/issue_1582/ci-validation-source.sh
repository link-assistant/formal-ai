#!/usr/bin/env bash
set -u
sed -n '1,195p' .github/workflows/release.yml
rg -n 'workflow_dispatch|inputs:|mode:|options:|test.*integration|with_formal_ai|cargo.*test|if:' .github/workflows/{release,stock-rust,self-hosting,layered-ci}.yml | head -120
rg -n 'test.*integration|cargo.*test' .github/workflows/*.yml | head -60
cat .github/actions/dispatch-checks/action.yml
sed -n '1,240p' rust/tests/integration/with_formal_ai/issue_1582.rs
git diff --stat
