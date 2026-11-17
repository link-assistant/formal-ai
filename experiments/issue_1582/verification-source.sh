#!/usr/bin/env bash
set -eu
sed -n '230,500p' rust/tests/integration/with_formal_ai/model_metadata.rs
rg -n 'served.*limit|context_window_tokens.*[0-9]|/v1/models|CLAUDE|claude' rust/tests/integration/with_formal_ai.rs | head -85
rg -n 'claude|model.*metadata|context_window' rust/tests/integration/with_formal_ai/*.rs | head -65
cat docs/case-studies/issue-1582/raw-data/research/grok-readme.md
python3 - <<'PY'
import json
p='docs/case-studies/issue-1582/raw-data/research/grok-tree.json'
x=json.load(open(p));print([v['path'] for v in x.get('tree',[]) if any(k in v['path'].lower() for k in ['setting','config','context','token'])][:80])
PY
sed -n '1,180p' rust/tests/web/server-structured-log.test.mjs
rg -n 'requirements|translate|mirror|changelog|status' CONTRIBUTING.md | head -65
node experiments/formal_ai_subagent/local-gates.mjs --list
