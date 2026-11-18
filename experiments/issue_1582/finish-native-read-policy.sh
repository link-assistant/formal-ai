#!/usr/bin/env bash
set -eu
python3 experiments/issue_1582/finish-native-read-policy.py
rustfmt --edition 2024 rust/src/solver_dispatch.rs rust/src/meta_method_dispatch.rs rust/src/version_resolution.rs rust/src/external_benchmarks/mod.rs rust/src/source_fetch.rs rust/src/solver_unknown_reasoning.rs rust/src/solver_handlers/web_requests.rs rust/tests/unit/specification/external_benchmarks.rs rust/tests/integration/with_formal_ai/issue_1582.rs
node scripts/translate-es.mjs --write > /tmp/issue-1582-native-policy-translation.log 2>&1
node scripts/assemble-requirements.mjs --write > /tmp/issue-1582-native-policy-requirements.log 2>&1
node scripts/generate-requirement-status.mjs --write >> /tmp/issue-1582-native-policy-requirements.log 2>&1
node scripts/render-status.mjs --write >> /tmp/issue-1582-native-policy-requirements.log 2>&1
node scripts/render-progressive-plan.mjs --write >> /tmp/issue-1582-native-policy-requirements.log 2>&1
node scripts/generate-runtime-feature-inventory.mjs --write >> /tmp/issue-1582-native-policy-requirements.log 2>&1
node scripts/measure-notation.mjs --write >> /tmp/issue-1582-native-policy-requirements.log 2>&1
