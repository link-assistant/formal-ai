#!/usr/bin/env bash
set -eu
python3 - <<'PY'
from pathlib import Path
p=Path('docs/requirements-traceability.md');s=p.read_text()
rows=[
('1','implemented — default read policy and bounded HTTP regression','rust/tests/web/issue-1582-defaults.test.mjs'),
('2','implemented — five-language harness permission guidance','rust/tests/web/issue-1582-defaults.test.mjs'),
('3','partial — native wrapper awaits CI','rust/tests/web/server-structured-log-cli.test.mjs; rust/tests/integration/with_formal_ai/issue_1582.rs'),
('4','implemented — explicit nested typed logs','rust/tests/web/issue-1582-defaults.test.mjs'),
('5','partial — Gemini input context blocked upstream; native adapters await CI','rust/tests/integration/with_formal_ai/issue_1582.rs'),
('6','partial — available evidence retained; Mac-only logs unavailable; final CI pending','none recorded')]
for number,status,tests in rows:
    if '| R1582-'+number+' |' not in s:s+='| R1582-'+number+' | docs/requirements/issue-1582-client-read-access-and-context.md | '+status+' | '+tests+' | not yet confirmed |\n'
p.write_text(s)
PY
git add js/runtime/read-access.mjs docs/requirements/issue-1582-client-read-access-and-context.md
node scripts/translate-es.mjs --write
node scripts/assemble-requirements.mjs --write
node scripts/generate-requirement-status.mjs --write
node scripts/render-status.mjs --write
node scripts/mirror-package-data.mjs --write
