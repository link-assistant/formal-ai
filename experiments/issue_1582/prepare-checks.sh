#!/usr/bin/env bash
set -eu
python3 - <<'PY'
from pathlib import Path
import json
p=Path('rust/tests/integration/with_formal_ai/issue_1582.rs')
s=p.read_text().replace('r#"{"data":[{"id":"formal-ai","context_window_tokens":1234567,"max_output_tokens":32768}]}"#','r#"{"models":[{"id":"formal-ai","context_window_tokens":1234567,"max_output_tokens":32768}]}"#')
p.write_text(s)
p=Path('data/benchmarks/fast-pull-request-javascript-tests.json');v=json.loads(p.read_text());name='rust/tests/web/issue-1582-defaults.test.mjs'
if name not in v['files']:v['files'].append(name)
p.write_text(json.dumps(v,indent=2)+'\n')
p=Path('.git/info/exclude');s=p.read_text()
if '\n/node_modules\n' not in s:p.write_text(s+'\n/node_modules\n')
PY
node scripts/mirror-package-data.mjs --write
node scripts/translate-es.mjs --write
node scripts/generate-worker-prerequisites.mjs --write
rustfmt --edition 2024 rust/src/cli_env.rs rust/src/client_integrations.rs rust/src/client_integrations/global_config.rs rust/src/coding/synthesis_runtime.rs rust/src/main.rs rust/src/solver_handler_how.rs rust/src/solver_handlers/feature_capability.rs rust/src/solver_handlers/web_requests.rs rust/src/solver_handlers/web_requests/live_search.rs rust/src/structured_log.rs rust/tests/integration/with_formal_ai.rs rust/tests/integration/with_formal_ai/issue_1582.rs
