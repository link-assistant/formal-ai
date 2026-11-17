#!/usr/bin/env bash
set -eu
git diff origin/main -- rust/src js data/seed/client-integrations.lino data/seed/responses.lino data/seed/lexemes.lino rust/tests/integration/with_formal_ai rust/tests/integration/with_formal_ai.rs rust/tests/web/issue-1582-defaults.test.mjs > /tmp/issue-1582-review-source.diff
wc -l /tmp/issue-1582-review-source.diff
git branch --show-current
git merge-base --is-ancestor origin/main HEAD
python3 - <<'PY'
import json
from pathlib import Path
rows=json.loads(Path('docs/case-studies/issue-1582/raw-data/research/recent-claude-prs.json').read_text())
for row in rows:
    if row['number']==1564:
        print(row['url'])
        print(row['body'])
PY
