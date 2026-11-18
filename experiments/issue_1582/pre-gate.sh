#!/usr/bin/env bash
set -eu
rustfmt --edition 2024 rust/src/client_integrations/global_config.rs
python3 experiments/issue_1582/archive-evidence.py
git add REQUIREMENTS.md data docs changelog.d js ts rust experiments/issue_1582
git diff --check --cached
rg -n -A 6 '\[\[test\]\]' rust/Cargo.toml
cat docs/case-studies/issue-1582/raw-data/pr-comments-current.json
sed -n '250,290p' .github/workflows/release.yml
