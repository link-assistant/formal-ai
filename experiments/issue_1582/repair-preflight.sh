#!/usr/bin/env bash
set -eu
python3 - <<'PY'
from pathlib import Path
import json, shutil
root = Path('docs/case-studies/issue-1582/raw-data')
runs = json.loads((root / 'ci/repair-runs-c8de9bf3c2b4.json').read_text())
for run in runs:
    status = json.loads((root / ('ci/' + str(run['id']) + '.log-status.json')).read_text())
    assert status['exit'] == 0, status
assert len(runs) == 22
print('All 22 first-repair-head run logs retained')
prior = root / 'local/gates-round-7'
prior.mkdir(exist_ok=True)
for file in Path('experiments/formal_ai_subagent/sandboxes/gates').glob('*.log'):
    shutil.copyfile(file, prior / file.name)
PY
python3 scripts/generate-role-registry.py --check
node scripts/mirror-package-data.mjs --check
node scripts/generate-worker-prerequisites.mjs --check
node scripts/measure-notation.mjs
node scripts/render-status.mjs --check
python3 experiments/issue_1582/refresh-repair-feedback.py
git diff --check
