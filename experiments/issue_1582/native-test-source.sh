#!/usr/bin/env bash
set -eu
sed -n '450,610p' rust/tests/integration/with_formal_ai.rs
rg -n '/v1/models|context_window_tokens|served_model_limits' rust/src/server* rust/tests/integration | head -80
sed -n '1,90p' rust/tests/integration/with_formal_ai/fixture_env.rs
sed -n '1,130p' rust/src/structured_log.rs
sed -n '270,335p' rust/src/solver_handlers/feature_capability.rs
gh search issues --repo google-gemini/gemini-cli 'custom context tokenLimit' --limit 15 --json number,title,url,state > docs/case-studies/issue-1582/raw-data/research/gemini-related-issues.json
python3 - <<'PY'
import json
x=json.load(open('docs/case-studies/issue-1582/raw-data/research/grok-tree.json'))
print('\n'.join(v['path'] for v in x.get('tree',[]) if v['path'].endswith('.md') and any(k in v['path'].lower() for k in ['config','model','setting'])))
PY
python3 experiments/issue_1582/extend-tests.py
