#!/usr/bin/env bash
# Combined verification exceeds the driver's fixed shell deadline; outer exception.
# Retain the first run before local-gates overwrites its per-gate files.
set -u
mkdir -p /tmp/issue-1582-review/gates-initial
cp experiments/formal_ai_subagent/sandboxes/gates/*.log /tmp/issue-1582-review/gates-initial/
export PATH="/tmp/issue-1582-js-tools/shellcheck-v0.11.0:$PATH"
failed=0
for gate in check-abbreviations check-i18n-catalog-coverage check-runner-labels check-runtime-feature-inventory check-tests-as-docs check-tests-as-documentation-javascript-twin lint-shell-scripts 'layered-ci#4'; do
  node experiments/formal_ai_subagent/local-gates.mjs --only "$gate" > "/tmp/issue-1582-review/repaired-$gate.log" 2>&1 || failed=1
done
rustfmt --edition 2024 --check --config skip_children=true rust/src/solver_handler_how.rs rust/src/solver_handlers/feature_capability.rs rust/tests/unit/issue_1582_permissions.rs rust/tests/unit/mod.rs > /tmp/issue-1582-review/rustfmt-final.log 2>&1 || failed=1
echo "$failed" > /tmp/issue-1582-review/focused-gates.exit
exit "$failed"
