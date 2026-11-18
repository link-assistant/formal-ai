#!/usr/bin/env bash
set -eu
node scripts/generate-requirement-status.mjs --write > /tmp/issue-1582-native-gates-refresh.log 2>&1
node scripts/render-status.mjs --write >> /tmp/issue-1582-native-gates-refresh.log 2>&1
node scripts/render-progressive-plan.mjs --write >> /tmp/issue-1582-native-gates-refresh.log 2>&1
node scripts/generate-runtime-feature-inventory.mjs --write >> /tmp/issue-1582-native-gates-refresh.log 2>&1
node scripts/measure-notation.mjs --write >> /tmp/issue-1582-native-gates-refresh.log 2>&1
python3 - <<'PY'
from pathlib import Path
import shutil
destination=Path('docs/case-studies/issue-1582/raw-data/local/gates-round-6')
shutil.copytree('experiments/formal_ai_subagent/sandboxes/gates',destination,dirs_exist_ok=True)
p=Path('docs/case-studies/issue-1582/README.md')
s=p.read_text().replace('The worker\nidentity bootstrap also needed one direct maintained-generator invocation',
    'The final JavaScript-to-Rust projection refresh also hit that 60-second\nbound after four of five modules; it was rerun directly with the pinned\ntranslator and one worker, without Rust compilation. The worker\nidentity bootstrap also needed one direct maintained-generator invocation')
p.write_text(s)
PY
git diff --check
