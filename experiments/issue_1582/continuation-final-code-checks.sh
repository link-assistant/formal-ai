#!/usr/bin/env bash
set -eu
node --test --test-concurrency=1 rust/tests/web/rust-specification-unicode-identifiers.test.mjs > /tmp/issue-1582-review/witness-final.log 2>&1
rustfmt --edition 2024 --check --config skip_children=true rust/src/solver_handler_how.rs rust/src/solver_handlers/feature_capability.rs rust/tests/unit/issue_1582_permissions.rs rust/tests/unit/mod.rs > /tmp/issue-1582-review/rustfmt-final.log 2>&1
for gate in check-minimal-core-boundary check-minimal-core-boundary-js-twin check-abbreviations check-language-change-parity check_js_rust_translation; do
 node experiments/formal_ai_subagent/local-gates.mjs --only "$gate" > "/tmp/issue-1582-review/final-$gate.log" 2>&1
done
git diff --check HEAD
