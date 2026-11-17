#!/usr/bin/env bash
set -eu
rg -q '^4/4 passed or skipped; failed: none$' /tmp/issue-1582-transport-gates.log
python3 - <<'PY'
from pathlib import Path
import shutil
root = Path('docs/case-studies/issue-1582/raw-data/local')
target = root / 'gates-round-8'
target.mkdir(exist_ok=True)
for file in (root / 'gates').glob('*.log.gz'):
    shutil.copyfile(file, target / file.name)
PY
python3 experiments/issue_1582/record-transport-deadline.py
python3 experiments/issue_1582/archive-evidence.py
git add js/server/read-access.mjs ts/server/read-access.mts rust/tests/web/issue-1582-defaults.test.mjs
git diff --cached --check
git commit -m "fix: bound read transport through response body consumption"
git add docs/case-studies/issue-1582 experiments/issue_1582
git diff --cached --check
git commit -m "docs: retain passing CI and bounded transport regression evidence"
test -z "$(git status --porcelain)"
