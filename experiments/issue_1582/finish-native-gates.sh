#!/usr/bin/env bash
set -eu
python3 experiments/issue_1582/finish-native-gates.py
node scripts/translate-js-rust.mjs --write --meta-language .meta-language --jobs 1 js/agentic/crate/version_resolution.mjs > /tmp/issue-1582-native-policy-projection.log 2>&1
node scripts/generate-requirement-status.mjs --write > /tmp/issue-1582-native-gates-refresh.log 2>&1
node scripts/render-status.mjs --write >> /tmp/issue-1582-native-gates-refresh.log 2>&1
node scripts/render-progressive-plan.mjs --write >> /tmp/issue-1582-native-gates-refresh.log 2>&1
node scripts/generate-runtime-feature-inventory.mjs --write >> /tmp/issue-1582-native-gates-refresh.log 2>&1
node scripts/measure-notation.mjs --write >> /tmp/issue-1582-native-gates-refresh.log 2>&1
git diff --check
