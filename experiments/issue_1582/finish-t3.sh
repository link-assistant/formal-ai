#!/usr/bin/env bash
set -eu
python3 experiments/issue_1582/finish-t3.py
node scripts/mirror-package-data.mjs --write
rustfmt --edition 2024 rust/tests/integration/with_formal_ai/issue_1582.rs
git diff --check -- data rust experiments/issue_1582
