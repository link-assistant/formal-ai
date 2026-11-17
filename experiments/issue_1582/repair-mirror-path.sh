#!/usr/bin/env bash
set -eu
python3 - <<'PY'
from pathlib import Path
source=Path('js/runtime/read-access.mjs');target=Path('js/server/read-access.mjs')
if source.exists():target.write_bytes(source.read_bytes());source.unlink();source.parent.rmdir()
for name in ['js/agentic/node-host.mjs','js/server/main.mjs','rust/src/cli_env.rs','rust/tests/web/issue-1582-defaults.test.mjs','docs/case-studies/issue-1582/README.md']:
 p=Path(name);s=p.read_text().replace('js/runtime/read-access.mjs','js/server/read-access.mjs').replace('../../../js/runtime/read-access.mjs','../../../js/server/read-access.mjs').replace('../runtime/read-access.mjs','./read-access.mjs' if name=='js/server/main.mjs' else '../server/read-access.mjs');p.write_text(s)
PY
git add -u js/runtime/read-access.mjs
git add js/server/read-access.mjs
node scripts/translate-es.mjs --write
