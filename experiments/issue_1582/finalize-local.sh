#!/usr/bin/env bash
set -eu
test "$(git branch --show-current)" = issue-1582-79aa864de807
rg -q '^138/138 passed or skipped; failed: none$' /tmp/issue-1582-gates-round-7.log
python3 - <<'PY'
from pathlib import Path
p=Path('docs/case-studies/issue-1582/README.md')
s=p.read_text().replace('the seventh complete round checks the final tree.', 'the seventh complete round passes or explicitly skips all 138 entries, with no failures.')
p.write_text(s)
p=Path('docs/case-studies/issue-1582/work-plan.md')
s=p.read_text().replace('their repairs are being checked in round seven.', 'round seven verifies the repaired final tree: 138/138 passed or explicitly skipped, no failures.')
p.write_text(s)
PY
git diff --name-only origin/main -- '*.rs' ':!rust/tests/fixtures/js-rust-translation/**' | while IFS= read -r source; do rustfmt --edition 2024 --check "$source"; done
python3 experiments/issue_1582/archive-evidence.py
git add data docs/requirements js ts rust
git diff --cached --check
git commit -m "fix: apply read defaults across native routes and client adapters"
git add docs/case-studies/issue-1582 experiments/issue_1582
git diff --cached --check
git commit -m "docs: retain native route investigation and final local verification"
git status --short
