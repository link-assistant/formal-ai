#!/usr/bin/env bash
set -eu
test "$(git branch --show-current)" = issue-1582-79aa864de807
python3 - <<'PY'
from pathlib import Path
path=Path('docs/requirements-traceability.md')
text=path.read_text().replace('partial — native wrapper awaits CI','partial — native runtime regression unexecuted under pre-merge policy').replace('partial — Gemini input context blocked upstream; native adapters await CI','partial — Gemini input context blocked upstream; native runtime unexecuted under pre-merge policy').replace('partial — available evidence retained; Mac-only logs unavailable; final CI pending','partial — evidence retained; Mac-only logs unavailable; dependency qualification and native runtime unfinished')
path.write_text(text)
PY
node scripts/assemble-requirements.mjs --write > /tmp/issue-1582-review/assemble-final.log 2>&1
node scripts/generate-requirement-status.mjs --write > /tmp/issue-1582-review/requirements-final.log 2>&1
node scripts/render-status.mjs --write > /tmp/issue-1582-review/render-status-final.log 2>&1
node scripts/mirror-package-data.mjs --write > /tmp/issue-1582-review/mirror-status-final.log 2>&1
git restore --staged docs experiments/issue_1582
git add data js ts rust changelog.d docs/requirements docs/requirements-traceability.md docs/status.md
git diff --cached --check
git diff --cached --stat > /tmp/issue-1582-review/repair-commit-stat.log
git commit -m "Fix permission guidance ownership across supported languages"
