#!/usr/bin/env bash
set -eu
python3 - <<'PY'
from pathlib import Path
p=Path('rust/tests/integration/with_formal_ai.rs')
p.write_text(p.read_text().replace('.map(|entry| std::fs::read(entry.expect("log entry").path()).expect("log"))\n        .flatten()', '.flat_map(|entry| std::fs::read(entry.expect("log entry").path()).expect("log"))'))
PY
rustfmt --edition 2024 rust/tests/integration/with_formal_ai.rs
python3 experiments/issue_1582/archive-evidence.py
git add REQUIREMENTS.md data docs changelog.d js ts rust experiments/issue_1582
git diff --check --cached
git status --short
