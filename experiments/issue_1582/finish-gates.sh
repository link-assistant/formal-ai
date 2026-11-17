#!/usr/bin/env bash
set -eu
python3 experiments/issue_1582/finish-gates.py
rustfmt --edition 2024 rust/src/client_integrations/global_config.rs
node scripts/assemble-requirements.mjs --write
node scripts/generate-runtime-feature-inventory.mjs --write
node scripts/measure-notation.mjs --write
git add data js ts rust experiments/issue_1582 docs
PATH=/tmp/issue-1582-shellcheck/shellcheck-v0.11.0:$PATH node experiments/formal_ai_subagent/local-gates.mjs --only check-dependency-direction,check-language-test-coverage,check-minimal-core-boundary,self-development-status#1
