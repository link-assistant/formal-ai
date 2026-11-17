#!/usr/bin/env bash
set -eu
python3 experiments/issue_1582/codex-live-search.py
node scripts/mirror-package-data.mjs --write
rustfmt --edition 2024 --check rust/src/client_integrations/global_config.rs
rustfmt --edition 2024 rust/tests/integration/with_formal_ai/issue_1582.rs
node scripts/assemble-requirements.mjs --write
git diff --check
